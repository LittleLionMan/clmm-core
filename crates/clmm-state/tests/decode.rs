use anchor_lang::prelude::Pubkey;
use anchor_lang::{AccountSerialize, Discriminator};
use clmm_state::accounts::{decode_account, tick_array_address, ClmmAccount, DecodeError, CLMM_PROGRAM_ID};
use clmm_state::summary::{summarize, CheckOutcome};
use raydium_clmm::libraries::tick_math;
use raydium_clmm::states::{AmmConfig, PoolState, TickArrayState};

fn zero_copy_bytes<T: bytemuck::Pod + Discriminator>(value: &T) -> Vec<u8> {
    let mut data = T::DISCRIMINATOR.to_vec();
    data.extend_from_slice(bytemuck::bytes_of(value));
    data
}

fn sample_pool(tick: i32) -> PoolState {
    let mut pool = PoolState::default();
    pool.tick_spacing = 60;
    pool.tick_current = tick;
    pool.sqrt_price_x64 = tick_math::get_sqrt_price_at_tick(tick).expect("valid tick");
    pool.liquidity = 5_124_165_121_219;
    pool.mint_decimals_0 = 9;
    pool.mint_decimals_1 = 6;
    pool.amm_config = Pubkey::new_unique();
    pool
}

fn failed_checks(account: &ClmmAccount) -> Vec<&'static str> {
    summarize(account)
        .checks
        .into_iter()
        .filter(|check| matches!(check.outcome, CheckOutcome::Failed))
        .map(|check| check.name)
        .collect()
}

#[test]
fn pool_state_roundtrip_preserves_fields() {
    let pool = sample_pool(-32_395);
    let data = zero_copy_bytes(&pool);
    assert_eq!(data.len(), PoolState::LEN);
    let ClmmAccount::Pool(decoded) = decode_account(&CLMM_PROGRAM_ID, &data).expect("decodes") else {
        panic!("expected PoolState");
    };
    let (tick, liquidity, sqrt_price) = (decoded.tick_current, decoded.liquidity, decoded.sqrt_price_x64);
    assert_eq!(tick, -32_395);
    assert_eq!(liquidity, 5_124_165_121_219);
    let expected_sqrt_price = pool.sqrt_price_x64;
    assert_eq!(sqrt_price, expected_sqrt_price);
}

#[test]
fn consistent_pool_passes_all_checks() {
    let data = zero_copy_bytes(&sample_pool(-32_395));
    let decoded = decode_account(&CLMM_PROGRAM_ID, &data).expect("decodes");
    assert!(failed_checks(&decoded).is_empty());
}

#[test]
fn inconsistent_tick_is_detected() {
    let mut pool = sample_pool(-32_395);
    pool.tick_current = 100;
    let decoded = decode_account(&CLMM_PROGRAM_ID, &zero_copy_bytes(&pool)).expect("decodes");
    assert_eq!(failed_checks(&decoded), vec!["tick_current passt zu sqrt_price_x64"]);
}

#[test]
fn size_mismatch_is_reported() {
    let mut data = zero_copy_bytes(&sample_pool(0));
    data.push(0);
    assert!(matches!(
        decode_account(&CLMM_PROGRAM_ID, &data),
        Err(DecodeError::SizeMismatch { kind: "PoolState", .. })
    ));
}

#[test]
fn foreign_owner_is_rejected() {
    let data = zero_copy_bytes(&sample_pool(0));
    assert!(matches!(decode_account(&Pubkey::new_unique(), &data), Err(DecodeError::WrongOwner(_))));
}

#[test]
fn unknown_discriminator_is_reported() {
    assert!(matches!(
        decode_account(&CLMM_PROGRAM_ID, &[1, 2, 3, 4, 5, 6, 7, 8, 9]),
        Err(DecodeError::UnknownDiscriminator(_))
    ));
}

#[test]
fn amm_config_borsh_roundtrip() {
    let config = AmmConfig {
        index: 4,
        trade_fee_rate: 2_500,
        protocol_fee_rate: 120_000,
        fund_fee_rate: 40_000,
        tick_spacing: 60,
        ..Default::default()
    };
    let mut data = Vec::new();
    config.try_serialize(&mut data).expect("serializes");
    data.resize(AmmConfig::LEN, 0);
    let ClmmAccount::AmmConfig(decoded) = decode_account(&CLMM_PROGRAM_ID, &data).expect("decodes") else {
        panic!("expected AmmConfig");
    };
    assert_eq!(decoded.trade_fee_rate, 2_500);
    assert_eq!(decoded.index, 4);
}

#[test]
fn tick_array_address_matches_raydium_key_derivation() {
    let pool_id = Pubkey::new_unique();
    let mut array = TickArrayState::default();
    array.initialize(-36_000, 60, pool_id).expect("valid start index");
    assert_eq!(tick_array_address(&pool_id, -36_000), array.key());
}
