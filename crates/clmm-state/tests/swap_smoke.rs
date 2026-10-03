use anchor_lang::prelude::Pubkey;
use raydium_clmm::instructions::{swap_internal, SwapInternalResult};
use raydium_clmm::states::{AmmConfig, ObservationState, PoolState, TickArrayState, TickState, POOL_SEED};
use std::cell::{RefCell, RefMut};
use std::collections::VecDeque;

fn tick(index: i32, liquidity_gross: u128, liquidity_net: i128) -> TickState {
    let mut state = TickState::default();
    state.tick = index;
    state.liquidity_gross = liquidity_gross;
    state.liquidity_net = liquidity_net;
    state
}

fn tick_array(pool_id: Pubkey, start: i32, spacing: u16, ticks: &[TickState]) -> RefCell<TickArrayState> {
    let mut array = TickArrayState::default();
    array.initialize(start, spacing, pool_id).expect("valid start index");
    array.initialized_tick_count = u8::try_from(ticks.len()).expect("at most 60 ticks");
    for state in ticks {
        let offset = usize::try_from((state.tick - start) / i32::from(spacing)).expect("tick inside array");
        array.ticks[offset] = *state;
    }
    RefCell::new(array)
}

#[test]
fn original_swap_loop_crosses_tick_arrays_off_chain() {
    let spacing = 60;
    let tick_current = -32_395;
    let liquidity = 5_124_165_121_219u128;
    let sqrt_price_x64 = 3_651_942_632_306_380_802u128;

    let mut pool = PoolState::default();
    pool.tick_current = tick_current;
    pool.tick_spacing = spacing;
    pool.sqrt_price_x64 = sqrt_price_x64;
    pool.liquidity = liquidity;
    pool.amm_config = Pubkey::new_unique();
    pool.token_mint_0 = Pubkey::new_unique();
    pool.token_mint_1 = Pubkey::new_unique();
    pool.bump = [Pubkey::find_program_address(
        &[
            POOL_SEED.as_bytes(),
            pool.amm_config.as_ref(),
            pool.token_mint_0.as_ref(),
            pool.token_mint_1.as_ref(),
        ],
        &raydium_clmm::ID,
    )
    .1];
    let pool = RefCell::new(pool);
    let pool_id = pool.borrow().key();

    let amm_config = AmmConfig {
        trade_fee_rate: 1_000,
        tick_spacing: spacing,
        ..Default::default()
    };
    let observation = RefCell::new(ObservationState::default());
    observation.borrow_mut().pool_id = pool_id;

    let arrays = [
        tick_array(
            pool_id,
            -32_400,
            spacing,
            &[tick(-32_400, 277_065_331_032, -277_065_331_032), tick(-29_220, 1_330_680_689, -1_330_680_689), tick(-28_860, 6_408_486_554, -6_408_486_554)],
        ),
        tick_array(
            pool_id,
            -36_000,
            spacing,
            &[
                tick(-32_460, 1_194_569_667_438, 536_061_033_698),
                tick(-32_520, 790_917_615_645, 790_917_615_645),
                tick(-32_580, 152_146_472_301, 128_451_145_459),
                tick(-32_640, 2_625_605_835_354, -1_492_054_447_712),
            ],
        ),
    ];
    for array in &arrays {
        let start = array.borrow().start_tick_index;
        pool.borrow_mut().flip_tick_array_bit(None, start).expect("bitmap flip");
    }

    let mut loaded: VecDeque<RefMut<TickArrayState>> = arrays.iter().map(RefCell::borrow_mut).collect();
    let SwapInternalResult { amount_0, amount_1, .. } = swap_internal(
        &amm_config,
        &mut pool.borrow_mut(),
        &mut loaded,
        &mut observation.borrow_mut(),
        None,
        12_188_240_002,
        3_049_500_711_113_990_606,
        true,
        true,
        1_700_000_000,
    )
    .expect("swap succeeds");
    drop(loaded);

    let state = pool.borrow();
    let (new_tick, new_liquidity, new_sqrt, fee_growth_0) =
        (state.tick_current, state.liquidity, state.sqrt_price_x64, state.fee_growth_global_0_x64);
    assert_eq!(amount_0, 12_188_240_002);
    assert!(amount_1 > 0);
    assert!(new_tick > -32_460 && new_tick < -32_400);
    assert!(new_sqrt < sqrt_price_x64);
    assert_eq!(new_liquidity, liquidity + 277_065_331_032);
    assert!(fee_growth_0 > 0);
}
