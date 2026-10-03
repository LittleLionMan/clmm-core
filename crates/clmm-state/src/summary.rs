use crate::accounts::ClmmAccount;
use raydium_clmm::libraries::tick_math;
use raydium_clmm::states::{AmmConfig, PersonalPositionState, PoolState, PoolStatusBitIndex, TickArrayState};

pub enum CheckOutcome {
    Passed,
    Failed,
}

pub struct Check {
    pub name: &'static str,
    pub outcome: CheckOutcome,
    pub detail: String,
}

pub struct Summary {
    pub kind: &'static str,
    pub fields: Vec<(&'static str, String)>,
    pub checks: Vec<Check>,
}

fn check(name: &'static str, passed: bool, detail: String) -> Check {
    Check {
        name,
        outcome: if passed { CheckOutcome::Passed } else { CheckOutcome::Failed },
        detail,
    }
}

pub fn human_price(sqrt_price_x64: u128, decimals_0: u8, decimals_1: u8) -> f64 {
    let sqrt_price = sqrt_price_x64 as f64 / 2f64.powi(64);
    sqrt_price * sqrt_price * 10f64.powi(i32::from(decimals_0) - i32::from(decimals_1))
}

fn summarize_pool(pool: &PoolState) -> Summary {
    let amm_config = pool.amm_config;
    let mint_0 = pool.token_mint_0;
    let mint_1 = pool.token_mint_1;
    let decimals_0 = pool.mint_decimals_0;
    let decimals_1 = pool.mint_decimals_1;
    let tick_spacing = pool.tick_spacing;
    let liquidity = pool.liquidity;
    let sqrt_price_x64 = pool.sqrt_price_x64;
    let tick_current = pool.tick_current;
    let fee_growth_0 = pool.fee_growth_global_0_x64;
    let fee_growth_1 = pool.fee_growth_global_1_x64;
    let fee_on = pool.fee_on;
    let open_time = pool.open_time;

    let statuses = [
        ("OpenPosition", PoolStatusBitIndex::OpenPositionOrIncreaseLiquidity),
        ("DecreaseLiquidity", PoolStatusBitIndex::DecreaseLiquidity),
        ("CollectFee", PoolStatusBitIndex::CollectFee),
        ("CollectReward", PoolStatusBitIndex::CollectReward),
        ("Swap", PoolStatusBitIndex::Swap),
        ("LimitOrder", PoolStatusBitIndex::LimitOrder),
    ]
    .into_iter()
    .map(|(name, bit)| format!("{name}={}", if pool.get_status_by_bit(bit) { "an" } else { "aus" }))
    .collect::<Vec<_>>()
    .join(", ");

    let dynamic_fee = match pool.get_dynamic_fee_info() {
        None => "deaktiviert".to_owned(),
        Some(info) => {
            let control = info.dynamic_fee_control;
            let max_accumulator = info.max_volatility_accumulator;
            let accumulator = info.volatility_accumulator;
            let reference = info.volatility_reference;
            let last_update = info.last_update_timestamp;
            format!(
                "aktiv (control {control}, accumulator {accumulator}/{max_accumulator}, reference {reference}, letztes Update {last_update})"
            )
        }
    };

    let mut checks = Vec::new();
    match tick_math::get_tick_at_sqrt_price(sqrt_price_x64) {
        Ok(derived_tick) => {
            let at_boundary = tick_math::get_sqrt_price_at_tick(derived_tick).map(|boundary| boundary == sqrt_price_x64).unwrap_or(false);
            let consistent = derived_tick == tick_current || (at_boundary && tick_current == derived_tick - 1);
            checks.push(check(
                "tick_current passt zu sqrt_price_x64",
                consistent,
                format!("gespeichert {tick_current}, aus Preis abgeleitet {derived_tick}"),
            ));
        }
        Err(error) => checks.push(check("sqrt_price_x64 im gültigen Bereich", false, format!("{error}"))),
    }
    checks.push(check(
        "tick_spacing > 0",
        tick_spacing > 0,
        format!("{tick_spacing}"),
    ));
    let start_index = TickArrayState::get_array_start_index(tick_current, tick_spacing);
    checks.push(check(
        "Start-Index des aktuellen Tick-Arrays gültig",
        TickArrayState::check_is_valid_start_index(start_index, tick_spacing),
        format!("{start_index}"),
    ));

    Summary {
        kind: "PoolState",
        fields: vec![
            ("amm_config", amm_config.to_string()),
            ("token_mint_0", format!("{mint_0} ({decimals_0} Dezimalstellen)")),
            ("token_mint_1", format!("{mint_1} ({decimals_1} Dezimalstellen)")),
            ("tick_spacing", tick_spacing.to_string()),
            ("tick_current", tick_current.to_string()),
            ("Preis token1/token0", format!("{:.8}", human_price(sqrt_price_x64, decimals_0, decimals_1))),
            ("aktive Liquidität", liquidity.to_string()),
            ("fee_growth_global_0_x64", fee_growth_0.to_string()),
            ("fee_growth_global_1_x64", fee_growth_1.to_string()),
            ("fee_on", fee_on.to_string()),
            ("dynamische Fee", dynamic_fee),
            ("Status", statuses),
            ("open_time", open_time.to_string()),
        ],
        checks,
    }
}

fn summarize_tick_array(array: &TickArrayState) -> Summary {
    let pool_id = array.pool_id;
    let start_tick_index = array.start_tick_index;
    let stored_count = array.initialized_tick_count;
    let ticks = array.ticks;
    let initialized = ticks.iter().filter(|tick| tick.is_initialized()).count();
    let with_liquidity = ticks.iter().filter(|tick| tick.has_liquidity()).count();
    let with_limit_orders = ticks.iter().filter(|tick| tick.has_limit_orders()).count();
    let net_sum: i128 = ticks.iter().map(|tick| tick.liquidity_net).sum();

    Summary {
        kind: "TickArrayState",
        fields: vec![
            ("pool_id", pool_id.to_string()),
            ("start_tick_index", start_tick_index.to_string()),
            ("initialisierte Ticks", initialized.to_string()),
            ("Ticks mit Liquidität", with_liquidity.to_string()),
            ("Ticks mit Limit Orders", with_limit_orders.to_string()),
            ("Summe liquidity_net", net_sum.to_string()),
        ],
        checks: vec![check(
            "initialized_tick_count stimmt mit Tick-Daten überein",
            usize::from(stored_count) == initialized,
            format!("gespeichert {stored_count}, gezählt {initialized}"),
        )],
    }
}

fn summarize_amm_config(config: &AmmConfig) -> Summary {
    Summary {
        kind: "AmmConfig",
        fields: vec![
            ("index", config.index.to_string()),
            ("trade_fee_rate", format!("{} ({:.4} %)", config.trade_fee_rate, f64::from(config.trade_fee_rate) / 10_000.0)),
            ("protocol_fee_rate", format!("{} ({:.2} % der Trade Fee)", config.protocol_fee_rate, f64::from(config.protocol_fee_rate) / 10_000.0)),
            ("fund_fee_rate", format!("{} ({:.2} % der Trade Fee)", config.fund_fee_rate, f64::from(config.fund_fee_rate) / 10_000.0)),
            ("tick_spacing", config.tick_spacing.to_string()),
        ],
        checks: vec![check(
            "Protocol- und Fund-Anteil zusammen höchstens 100 %",
            u64::from(config.protocol_fee_rate) + u64::from(config.fund_fee_rate) <= 1_000_000,
            format!("{} + {}", config.protocol_fee_rate, config.fund_fee_rate),
        )],
    }
}

fn summarize_position(position: &PersonalPositionState) -> Summary {
    Summary {
        kind: "PersonalPositionState",
        fields: vec![
            ("nft_mint", position.nft_mint.to_string()),
            ("pool_id", position.pool_id.to_string()),
            ("Range", format!("{} .. {}", position.tick_lower_index, position.tick_upper_index)),
            ("liquidity", position.liquidity.to_string()),
        ],
        checks: vec![check(
            "tick_lower < tick_upper",
            position.tick_lower_index < position.tick_upper_index,
            format!("{} < {}", position.tick_lower_index, position.tick_upper_index),
        )],
    }
}

pub fn summarize(account: &ClmmAccount) -> Summary {
    match account {
        ClmmAccount::Pool(pool) => summarize_pool(pool),
        ClmmAccount::TickArray(array) => summarize_tick_array(array),
        ClmmAccount::AmmConfig(config) => summarize_amm_config(config),
        ClmmAccount::PersonalPosition(position) => summarize_position(position),
        ClmmAccount::LimitOrder(order) => Summary {
            kind: "LimitOrderState",
            fields: vec![
                ("pool_id", order.pool_id.to_string()),
                ("tick_index", order.tick_index.to_string()),
                ("zero_for_one", order.zero_for_one.to_string()),
                ("total_amount", order.total_amount.to_string()),
                ("filled_amount", order.filled_amount.to_string()),
            ],
            checks: Vec::new(),
        },
        ClmmAccount::BitmapExtension(extension) => {
            let pool_id = extension.pool_id;
            Summary {
                kind: "TickArrayBitmapExtension",
                fields: vec![("pool_id", pool_id.to_string())],
                checks: Vec::new(),
            }
        }
        ClmmAccount::DynamicFeeConfig(_) => Summary {
            kind: "DynamicFeeConfig",
            fields: Vec::new(),
            checks: Vec::new(),
        },
        ClmmAccount::Observation(_) => Summary {
            kind: "ObservationState",
            fields: Vec::new(),
            checks: Vec::new(),
        },
    }
}
