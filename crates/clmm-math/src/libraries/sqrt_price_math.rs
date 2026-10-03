use super::full_math::MulDiv;
use super::tick_math;
use super::unsafe_math::UnsafeMathTrait;
use super::{fixed_point_64, U128, U256};
use crate::error::MathError;
use crate::error::Result;

pub fn get_next_sqrt_price_from_amount_0_rounding_up(
    sqrt_price_x64: u128,
    liquidity: u128,
    amount: u64,
    add: bool,
) -> Result<u128> {
    if amount == 0 {
        return Ok(sqrt_price_x64);
    };
    let numerator_1 = U256::from(liquidity) << fixed_point_64::RESOLUTION;

    if add {
        if let Some(product) = U256::from(amount).checked_mul(U256::from(sqrt_price_x64)) {
            let denominator = numerator_1 + product;
            if denominator >= numerator_1 {
                return Ok(numerator_1
                    .mul_div_ceil(U256::from(sqrt_price_x64), denominator)
                    .ok_or(MathError::CalculateOverflow)?
                    .as_u128());
            };
        }

        Ok(U256::div_rounding_up(
            numerator_1,
            (numerator_1 / U256::from(sqrt_price_x64))
                .checked_add(U256::from(amount))
                .ok_or(MathError::CalculateOverflow)?,
        )
        .as_u128())
    } else {
        let product = U256::from(amount)
            .checked_mul(U256::from(sqrt_price_x64))
            .ok_or(MathError::CalculateOverflow)?;
        let denominator = numerator_1
            .checked_sub(product)
            .ok_or(MathError::CalculateOverflow)?;
        Ok(numerator_1
            .mul_div_ceil(U256::from(sqrt_price_x64), denominator)
            .ok_or(MathError::CalculateOverflow)?
            .as_u128())
    }
}

pub fn get_next_sqrt_price_from_amount_1_rounding_down(
    sqrt_price_x64: u128,
    liquidity: u128,
    amount: u64,
    add: bool,
) -> Result<u128> {
    if amount == 0 {
        return Ok(sqrt_price_x64);
    }

    if add {
        let quotient = (U128::from(amount) << fixed_point_64::RESOLUTION) / U128::from(liquidity);
        sqrt_price_x64
            .checked_add(quotient.as_u128())
            .ok_or(MathError::CalculateOverflow.into())
    } else {
        let quotient = U128::div_rounding_up(
            U128::from(amount) << fixed_point_64::RESOLUTION,
            U128::from(liquidity),
        );
        sqrt_price_x64
            .checked_sub(quotient.as_u128())
            .ok_or(MathError::CalculateOverflow.into())
    }
}

pub fn get_next_sqrt_price_from_input(
    sqrt_price_x64: u128,
    liquidity: u128,
    amount_in: u64,
    zero_for_one: bool,
) -> Result<u128> {
    require!(
        sqrt_price_x64 >= tick_math::MIN_SQRT_PRICE_X64
            && sqrt_price_x64 < tick_math::MAX_SQRT_PRICE_X64,
        MathError::SqrtPriceX64
    );
    require_gt!(liquidity, 0, MathError::ZeroLiquidity);

    if zero_for_one {
        get_next_sqrt_price_from_amount_0_rounding_up(sqrt_price_x64, liquidity, amount_in, true)
    } else {
        get_next_sqrt_price_from_amount_1_rounding_down(sqrt_price_x64, liquidity, amount_in, true)
    }
}

pub fn get_next_sqrt_price_from_output(
    sqrt_price_x64: u128,
    liquidity: u128,
    amount_out: u64,
    zero_for_one: bool,
) -> Result<u128> {
    require!(
        sqrt_price_x64 >= tick_math::MIN_SQRT_PRICE_X64
            && sqrt_price_x64 < tick_math::MAX_SQRT_PRICE_X64,
        MathError::SqrtPriceX64
    );
    require_gt!(liquidity, 0, MathError::ZeroLiquidity);

    if zero_for_one {
        get_next_sqrt_price_from_amount_1_rounding_down(
            sqrt_price_x64,
            liquidity,
            amount_out,
            false,
        )
    } else {
        get_next_sqrt_price_from_amount_0_rounding_up(sqrt_price_x64, liquidity, amount_out, false)
    }
}
