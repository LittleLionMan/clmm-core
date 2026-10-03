use clmm_state::accounts::{bitmap_extension_address, decode_account, tick_array_address, ClmmAccount};
use clmm_state::rpc_json::parse_accounts;
use clmm_state::summary::{summarize, CheckOutcome};
use raydium_clmm::states::TickArrayState;
use anchor_lang::prelude::Pubkey;
use std::process::ExitCode;
use std::str::FromStr;

const USAGE: &str = "Verwendung:
  clmm-inspect decode <antwort.json>...
  clmm-inspect addresses <pool-adresse> <pool-antwort.json> [radius]";

fn read_accounts(path: &str) -> Result<Vec<Option<clmm_state::rpc_json::RawAccount>>, String> {
    let json = std::fs::read_to_string(path).map_err(|error| format!("{path}: {error}"))?;
    parse_accounts(&json).map_err(|error| format!("{path}: {error}"))
}

fn run_decode(paths: &[String]) -> Result<bool, String> {
    let mut all_passed = true;
    for path in paths {
        for (index, account) in read_accounts(path)?.into_iter().enumerate() {
            println!("== {path} [{index}]");
            let Some(raw) = account else {
                println!("   Account existiert nicht");
                continue;
            };
            match decode_account(&raw.owner, &raw.data) {
                Err(error) => {
                    all_passed = false;
                    println!("   FEHLER: {error}");
                }
                Ok(decoded) => {
                    let summary = summarize(&decoded);
                    println!("   Typ: {} ({} Bytes)", summary.kind, raw.data.len());
                    for (name, value) in &summary.fields {
                        println!("   {name}: {value}");
                    }
                    for check in &summary.checks {
                        let label = match check.outcome {
                            CheckOutcome::Passed => "OK  ",
                            CheckOutcome::Failed => {
                                all_passed = false;
                                "FAIL"
                            }
                        };
                        println!("   [{label}] {}: {}", check.name, check.detail);
                    }
                }
            }
        }
    }
    Ok(all_passed)
}

fn run_addresses(pool_address: &str, path: &str, radius: i32) -> Result<(), String> {
    let pool_id = Pubkey::from_str(pool_address).map_err(|_| format!("ungültige Pool-Adresse: {pool_address}"))?;
    let accounts = read_accounts(path)?;
    let Some(Some(raw)) = accounts.into_iter().next() else {
        return Err(format!("{path}: kein Account in der Antwort"));
    };
    let decoded = decode_account(&raw.owner, &raw.data).map_err(|error| error.to_string())?;
    let ClmmAccount::Pool(pool) = decoded else {
        return Err(format!("{path}: Account ist kein PoolState, sondern {}", decoded.kind()));
    };
    let derived = pool.key();
    if derived != pool_id {
        return Err(format!("Pool-Adresse passt nicht zu den Seeds des Accounts (abgeleitet: {derived})"));
    }
    let tick_spacing = pool.tick_spacing;
    let tick_current = pool.tick_current;
    let amm_config = pool.amm_config;
    let width = TickArrayState::tick_count(tick_spacing);
    let current_start = TickArrayState::get_array_start_index(tick_current, tick_spacing);
    println!("{amm_config}");
    println!("{}", bitmap_extension_address(&pool_id));
    for offset in -radius..=radius {
        println!("{}", tick_array_address(&pool_id, current_start + offset * width));
    }
    Ok(())
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let result = match args.first().map(String::as_str) {
        Some("decode") if args.len() > 1 => run_decode(&args[1..]).map(|passed| if passed { ExitCode::SUCCESS } else { ExitCode::from(2) }),
        Some("addresses") if args.len() > 2 => {
            let radius = args.get(3).map(|value| value.parse::<i32>()).transpose();
            match radius {
                Ok(radius) => run_addresses(&args[1], &args[2], radius.unwrap_or(3)).map(|()| ExitCode::SUCCESS),
                Err(_) => Err("radius muss eine ganze Zahl sein".to_owned()),
            }
        }
        _ => Err(USAGE.to_owned()),
    };
    match result {
        Ok(code) => code,
        Err(message) => {
            eprintln!("{message}");
            ExitCode::FAILURE
        }
    }
}
