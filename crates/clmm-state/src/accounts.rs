use anchor_lang::prelude::Pubkey;
use anchor_lang::{AccountDeserialize, Discriminator};
use bytemuck::Pod;
use raydium_clmm::states::{
    AmmConfig, DynamicFeeConfig, LimitOrderState, ObservationState, PersonalPositionState, PoolState,
    TickArrayBitmapExtension, TickArrayState,
};
use std::mem::size_of;

pub const CLMM_PROGRAM_ID: Pubkey = raydium_clmm::ID;

#[derive(Debug, thiserror::Error)]
pub enum DecodeError {
    #[error("account data is shorter than the 8-byte discriminator")]
    TooShort,
    #[error("unknown discriminator {0}")]
    UnknownDiscriminator(String),
    #[error("{kind}: expected {expected} bytes, got {actual}")]
    SizeMismatch {
        kind: &'static str,
        expected: usize,
        actual: usize,
    },
    #[error("{kind}: deserialization failed: {message}")]
    Deserialize { kind: &'static str, message: String },
    #[error("owner {0} is not the Raydium CLMM program")]
    WrongOwner(Pubkey),
}

pub enum ClmmAccount {
    Pool(Box<PoolState>),
    AmmConfig(AmmConfig),
    TickArray(Box<TickArrayState>),
    BitmapExtension(Box<TickArrayBitmapExtension>),
    PersonalPosition(PersonalPositionState),
    LimitOrder(LimitOrderState),
    DynamicFeeConfig(DynamicFeeConfig),
    Observation(Box<ObservationState>),
}

impl ClmmAccount {
    pub fn kind(&self) -> &'static str {
        match self {
            Self::Pool(_) => "PoolState",
            Self::AmmConfig(_) => "AmmConfig",
            Self::TickArray(_) => "TickArrayState",
            Self::BitmapExtension(_) => "TickArrayBitmapExtension",
            Self::PersonalPosition(_) => "PersonalPositionState",
            Self::LimitOrder(_) => "LimitOrderState",
            Self::DynamicFeeConfig(_) => "DynamicFeeConfig",
            Self::Observation(_) => "ObservationState",
        }
    }
}

fn decode_zero_copy<T: Pod + Discriminator>(kind: &'static str, data: &[u8]) -> Result<T, DecodeError> {
    let expected = 8 + size_of::<T>();
    if data.len() != expected {
        return Err(DecodeError::SizeMismatch {
            kind,
            expected,
            actual: data.len(),
        });
    }
    Ok(bytemuck::pod_read_unaligned::<T>(&data[8..]))
}

fn decode_borsh<T: AccountDeserialize>(kind: &'static str, data: &[u8], expected_len: usize) -> Result<T, DecodeError> {
    if data.len() != expected_len {
        return Err(DecodeError::SizeMismatch {
            kind,
            expected: expected_len,
            actual: data.len(),
        });
    }
    let mut slice = data;
    T::try_deserialize(&mut slice).map_err(|error| DecodeError::Deserialize {
        kind,
        message: error.to_string(),
    })
}

pub fn decode_account(owner: &Pubkey, data: &[u8]) -> Result<ClmmAccount, DecodeError> {
    if *owner != CLMM_PROGRAM_ID {
        return Err(DecodeError::WrongOwner(*owner));
    }
    if data.len() < 8 {
        return Err(DecodeError::TooShort);
    }
    let discriminator = &data[..8];

    if discriminator == PoolState::DISCRIMINATOR {
        return decode_zero_copy::<PoolState>("PoolState", data).map(|state| ClmmAccount::Pool(Box::new(state)));
    }
    if discriminator == TickArrayState::DISCRIMINATOR {
        return decode_zero_copy::<TickArrayState>("TickArrayState", data)
            .map(|state| ClmmAccount::TickArray(Box::new(state)));
    }
    if discriminator == TickArrayBitmapExtension::DISCRIMINATOR {
        return decode_zero_copy::<TickArrayBitmapExtension>("TickArrayBitmapExtension", data)
            .map(|state| ClmmAccount::BitmapExtension(Box::new(state)));
    }
    if discriminator == ObservationState::DISCRIMINATOR {
        return decode_zero_copy::<ObservationState>("ObservationState", data)
            .map(|state| ClmmAccount::Observation(Box::new(state)));
    }
    if discriminator == AmmConfig::DISCRIMINATOR {
        return decode_borsh::<AmmConfig>("AmmConfig", data, AmmConfig::LEN).map(ClmmAccount::AmmConfig);
    }
    if discriminator == PersonalPositionState::DISCRIMINATOR {
        return decode_borsh::<PersonalPositionState>("PersonalPositionState", data, PersonalPositionState::LEN)
            .map(ClmmAccount::PersonalPosition);
    }
    if discriminator == LimitOrderState::DISCRIMINATOR {
        return decode_borsh::<LimitOrderState>("LimitOrderState", data, LimitOrderState::LEN).map(ClmmAccount::LimitOrder);
    }
    if discriminator == DynamicFeeConfig::DISCRIMINATOR {
        return decode_borsh::<DynamicFeeConfig>("DynamicFeeConfig", data, DynamicFeeConfig::LEN)
            .map(ClmmAccount::DynamicFeeConfig);
    }
    Err(DecodeError::UnknownDiscriminator(
        discriminator.iter().map(|byte| format!("{byte:02x}")).collect(),
    ))
}

pub fn tick_array_address(pool: &Pubkey, start_tick_index: i32) -> Pubkey {
    Pubkey::find_program_address(
        &[
            raydium_clmm::states::TICK_ARRAY_SEED.as_bytes(),
            pool.as_ref(),
            &start_tick_index.to_be_bytes(),
        ],
        &CLMM_PROGRAM_ID,
    )
    .0
}

pub fn bitmap_extension_address(pool: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(
        &[raydium_clmm::states::POOL_TICK_ARRAY_BITMAP_SEED.as_bytes(), pool.as_ref()],
        &CLMM_PROGRAM_ID,
    )
    .0
}
