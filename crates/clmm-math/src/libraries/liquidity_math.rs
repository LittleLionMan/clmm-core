use super::big_num::U128;
use super::big_num::U256;
use super::fixed_point_64;
use super::full_math::{mul_pow2_div_ceil, mul_pow2_div_floor, MulDiv};
use super::tick_math;
use crate::error::MathError;
use crate::error::Result;

pub fn add_delta(x: u128, y: i128) -> Result<u128> {
    let z: u128;
    if y < 0 {
        z = x - y.unsigned_abs();
        require_gt!(x, z, MathError::LiquiditySubValueErr);
    } else {
        z = x + y.unsigned_abs();
        require_gte!(z, x, MathError::LiquidityAddValueErr);
    }
    Ok(z)
}

pub fn get_liquidity_from_amount_0(
    mut sqrt_ratio_a_x64: u128,
    mut sqrt_ratio_b_x64: u128,
    amount_0: u64,
) -> Result<u128> {
    if amount_0 == 0 {
        return Ok(0);
    }
    if sqrt_ratio_a_x64 > sqrt_ratio_b_x64 {
        std::mem::swap(&mut sqrt_ratio_a_x64, &mut sqrt_ratio_b_x64);
    };
    let intermediate = U128::from(sqrt_ratio_a_x64)
        .mul_div_floor(
            U128::from(sqrt_ratio_b_x64),
            U128::from(fixed_point_64::Q64),
        )
        .ok_or(MathError::CalculateOverflow)?;

    Ok(U128::from(amount_0)
        .mul_div_floor(
            intermediate,
            U128::from(sqrt_ratio_b_x64 - sqrt_ratio_a_x64),
        )
        .ok_or(MathError::CalculateOverflow)?
        .as_u128())
}

pub fn get_liquidity_from_amount_1(
    mut sqrt_ratio_a_x64: u128,
    mut sqrt_ratio_b_x64: u128,
    amount_1: u64,
) -> Result<u128> {
    if amount_1 == 0 {
        return Ok(0);
    }
    if sqrt_ratio_a_x64 > sqrt_ratio_b_x64 {
        std::mem::swap(&mut sqrt_ratio_a_x64, &mut sqrt_ratio_b_x64);
    };

    Ok(U128::from(amount_1)
        .mul_div_floor(
            U128::from(fixed_point_64::Q64),
            U128::from(sqrt_ratio_b_x64 - sqrt_ratio_a_x64),
        )
        .ok_or(MathError::CalculateOverflow)?
        .as_u128())
}

pub fn get_liquidity_from_amounts(
    sqrt_ratio_x64: u128,
    mut sqrt_ratio_a_x64: u128,
    mut sqrt_ratio_b_x64: u128,
    amount_0: u64,
    amount_1: u64,
) -> Result<u128> {
    if sqrt_ratio_a_x64 > sqrt_ratio_b_x64 {
        std::mem::swap(&mut sqrt_ratio_a_x64, &mut sqrt_ratio_b_x64);
    };

    if sqrt_ratio_x64 <= sqrt_ratio_a_x64 {
        get_liquidity_from_amount_0(sqrt_ratio_a_x64, sqrt_ratio_b_x64, amount_0)
    } else if sqrt_ratio_x64 < sqrt_ratio_b_x64 {
        Ok(u128::min(
            get_liquidity_from_amount_0(sqrt_ratio_x64, sqrt_ratio_b_x64, amount_0)?,
            get_liquidity_from_amount_1(sqrt_ratio_a_x64, sqrt_ratio_x64, amount_1)?,
        ))
    } else {
        get_liquidity_from_amount_1(sqrt_ratio_a_x64, sqrt_ratio_b_x64, amount_1)
    }
}

pub fn get_liquidity_from_single_amount_0(
    sqrt_ratio_x64: u128,
    mut sqrt_ratio_a_x64: u128,
    mut sqrt_ratio_b_x64: u128,
    amount_0: u64,
) -> Result<u128> {
    if sqrt_ratio_a_x64 > sqrt_ratio_b_x64 {
        std::mem::swap(&mut sqrt_ratio_a_x64, &mut sqrt_ratio_b_x64);
    };

    if sqrt_ratio_x64 <= sqrt_ratio_a_x64 {
        get_liquidity_from_amount_0(sqrt_ratio_a_x64, sqrt_ratio_b_x64, amount_0)
    } else if sqrt_ratio_x64 < sqrt_ratio_b_x64 {
        get_liquidity_from_amount_0(sqrt_ratio_x64, sqrt_ratio_b_x64, amount_0)
    } else {
        Ok(0)
    }
}

pub fn get_liquidity_from_single_amount_1(
    sqrt_ratio_x64: u128,
    mut sqrt_ratio_a_x64: u128,
    mut sqrt_ratio_b_x64: u128,
    amount_1: u64,
) -> Result<u128> {
    if sqrt_ratio_a_x64 > sqrt_ratio_b_x64 {
        std::mem::swap(&mut sqrt_ratio_a_x64, &mut sqrt_ratio_b_x64);
    };

    if sqrt_ratio_x64 <= sqrt_ratio_a_x64 {
        Ok(0)
    } else if sqrt_ratio_x64 < sqrt_ratio_b_x64 {
        get_liquidity_from_amount_1(sqrt_ratio_a_x64, sqrt_ratio_x64, amount_1)
    } else {
        get_liquidity_from_amount_1(sqrt_ratio_a_x64, sqrt_ratio_b_x64, amount_1)
    }
}

pub fn get_delta_amount_0_unsigned(
    mut sqrt_ratio_a_x64: u128,
    mut sqrt_ratio_b_x64: u128,
    liquidity: u128,
    round_up: bool,
) -> Result<u64> {
    if sqrt_ratio_a_x64 > sqrt_ratio_b_x64 {
        std::mem::swap(&mut sqrt_ratio_a_x64, &mut sqrt_ratio_b_x64);
    };
    require_gt!(sqrt_ratio_a_x64, 0, MathError::ZeroSqrtPrice);

    let numerator_1 = U256::from(liquidity) << fixed_point_64::RESOLUTION;
    let numerator_2 = U256::from(sqrt_ratio_b_x64 - sqrt_ratio_a_x64);
    let denominator = U256::from(sqrt_ratio_a_x64)
        .checked_mul(U256::from(sqrt_ratio_b_x64))
        .ok_or(MathError::CalculateOverflow)?;

    let result = if round_up {
        numerator_1.mul_div_ceil(numerator_2, denominator)
    } else {
        numerator_1.mul_div_floor(numerator_2, denominator)
    }
    .ok_or(MathError::CalculateOverflow)?;
    if result > U256::from(u64::MAX) {
        return Err(MathError::MaxTokenOverflow.into());
    }
    return Ok(result.as_u64());
}

pub fn get_delta_amount_1_unsigned(
    mut sqrt_ratio_a_x64: u128,
    mut sqrt_ratio_b_x64: u128,
    liquidity: u128,
    round_up: bool,
) -> Result<u64> {
    if sqrt_ratio_a_x64 > sqrt_ratio_b_x64 {
        std::mem::swap(&mut sqrt_ratio_a_x64, &mut sqrt_ratio_b_x64);
    };

    let result = if round_up {
        U256::from(liquidity).mul_div_ceil(
            U256::from(sqrt_ratio_b_x64 - sqrt_ratio_a_x64),
            U256::from(fixed_point_64::Q64),
        )
    } else {
        U256::from(liquidity).mul_div_floor(
            U256::from(sqrt_ratio_b_x64 - sqrt_ratio_a_x64),
            U256::from(fixed_point_64::Q64),
        )
    }
    .ok_or(MathError::CalculateOverflow)?;
    if result > U256::from(u64::MAX) {
        return Err(MathError::MaxTokenOverflow.into());
    }
    return Ok(result.as_u64());
}

pub fn get_delta_amounts_for_swap(
    mut sqrt_ratio_a_x64: u128,
    mut sqrt_ratio_b_x64: u128,
    liquidity: u128,
    zero_for_one: bool,
) -> Result<(u64, u64)> {
    if sqrt_ratio_a_x64 > sqrt_ratio_b_x64 {
        std::mem::swap(&mut sqrt_ratio_a_x64, &mut sqrt_ratio_b_x64);
    }
    require_gt!(sqrt_ratio_a_x64, 0, MathError::ZeroSqrtPrice);

    let sqrt_price_diff = sqrt_ratio_b_x64 - sqrt_ratio_a_x64;
    let amount_1_x64 = U256::from(liquidity)
        .checked_mul(U256::from(sqrt_price_diff))
        .ok_or(MathError::CalculateOverflow)?;
    let sqrt_price_product = U256::from(sqrt_ratio_a_x64)
        .checked_mul(U256::from(sqrt_ratio_b_x64))
        .ok_or(MathError::CalculateOverflow)?;

    let (amount_in_u256, amount_out_u256) = if zero_for_one {
        let amount_in = mul_pow2_div_ceil(amount_1_x64, 64, sqrt_price_product)
            .ok_or(MathError::CalculateOverflow)?;
        let amount_out = amount_1_x64 >> 64;
        (amount_in, amount_out)
    } else {
        let amount_in = amount_1_x64
            .checked_add(U256::from(u64::MAX))
            .ok_or(MathError::CalculateOverflow)?
            >> 64;
        let amount_out = mul_pow2_div_floor(amount_1_x64, 64, sqrt_price_product)
            .ok_or(MathError::CalculateOverflow)?;
        (amount_in, amount_out)
    };

    if amount_in_u256 > U256::from(u64::MAX) {
        return Err(MathError::MaxTokenOverflow.into());
    }
    if amount_out_u256 > U256::from(u64::MAX) {
        return Err(MathError::MaxTokenOverflow.into());
    }
    Ok((amount_in_u256.as_u64(), amount_out_u256.as_u64()))
}

pub fn get_delta_amount_0_signed(
    sqrt_ratio_a_x64: u128,
    sqrt_ratio_b_x64: u128,
    liquidity: i128,
) -> Result<u64> {
    if liquidity < 0 {
        get_delta_amount_0_unsigned(
            sqrt_ratio_a_x64,
            sqrt_ratio_b_x64,
            u128::try_from(
                liquidity
                    .checked_neg()
                    .ok_or(MathError::CalculateOverflow)?,
            )
            .map_err(|_| MathError::CalculateOverflow)?,
            false,
        )
    } else {
        get_delta_amount_0_unsigned(
            sqrt_ratio_a_x64,
            sqrt_ratio_b_x64,
            u128::try_from(liquidity).map_err(|_| MathError::CalculateOverflow)?,
            true,
        )
    }
}

pub fn get_delta_amount_1_signed(
    sqrt_ratio_a_x64: u128,
    sqrt_ratio_b_x64: u128,
    liquidity: i128,
) -> Result<u64> {
    if liquidity < 0 {
        get_delta_amount_1_unsigned(
            sqrt_ratio_a_x64,
            sqrt_ratio_b_x64,
            u128::try_from(
                liquidity
                    .checked_neg()
                    .ok_or(MathError::CalculateOverflow)?,
            )
            .map_err(|_| MathError::CalculateOverflow)?,
            false,
        )
    } else {
        get_delta_amount_1_unsigned(
            sqrt_ratio_a_x64,
            sqrt_ratio_b_x64,
            u128::try_from(liquidity).map_err(|_| MathError::CalculateOverflow)?,
            true,
        )
    }
}

pub fn get_delta_amounts_signed(
    tick_current: i32,
    sqrt_price_x64_current: u128,
    tick_lower: i32,
    tick_upper: i32,
    liquidity_delta: i128,
) -> Result<(u64, u64)> {
    let mut amount_0 = 0;
    let mut amount_1 = 0;
    if tick_current < tick_lower {
        amount_0 = get_delta_amount_0_signed(
            tick_math::get_sqrt_price_at_tick(tick_lower)?,
            tick_math::get_sqrt_price_at_tick(tick_upper)?,
            liquidity_delta,
        )?;
    } else if tick_current < tick_upper {
        amount_0 = get_delta_amount_0_signed(
            sqrt_price_x64_current,
            tick_math::get_sqrt_price_at_tick(tick_upper)?,
            liquidity_delta,
        )?;
        amount_1 = get_delta_amount_1_signed(
            tick_math::get_sqrt_price_at_tick(tick_lower)?,
            sqrt_price_x64_current,
            liquidity_delta,
        )?;
    } else {
        amount_1 = get_delta_amount_1_signed(
            tick_math::get_sqrt_price_at_tick(tick_lower)?,
            tick_math::get_sqrt_price_at_tick(tick_upper)?,
            liquidity_delta,
        )?;
    }
    Ok((amount_0, amount_1))
}

#[cfg(test)]
mod delta_amounts_for_swap_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn combined_matches_individual_unsigned(
            sqrt_a in tick_math::MIN_SQRT_PRICE_X64..tick_math::MAX_SQRT_PRICE_X64,
            sqrt_b in tick_math::MIN_SQRT_PRICE_X64..tick_math::MAX_SQRT_PRICE_X64,
            liquidity in prop_oneof![1u128..(1u128 << 80), 1u128..=u128::MAX],
            zero_for_one in proptest::bool::ANY,
        ) {
            prop_assume!(sqrt_a != sqrt_b);

            let (expected_in_res, expected_out_res) = if zero_for_one {
                (
                    get_delta_amount_0_unsigned(sqrt_a, sqrt_b, liquidity, true),
                    get_delta_amount_1_unsigned(sqrt_a, sqrt_b, liquidity, false),
                )
            } else {
                (
                    get_delta_amount_1_unsigned(sqrt_a, sqrt_b, liquidity, true),
                    get_delta_amount_0_unsigned(sqrt_a, sqrt_b, liquidity, false),
                )
            };

            let combined = get_delta_amounts_for_swap(sqrt_a, sqrt_b, liquidity, zero_for_one);

            match (expected_in_res, expected_out_res, combined) {
                (Ok(expected_in), Ok(expected_out), Ok((got_in, got_out))) => {
                    assert_eq!(got_in, expected_in, "amount_in mismatch (zfo={})", zero_for_one);
                    assert_eq!(got_out, expected_out, "amount_out mismatch (zfo={})", zero_for_one);
                }
                (Err(_), _, Err(_)) | (_, Err(_), Err(_)) => {}
                (Ok(_), Ok(_), Err(e)) => {
                    panic!("combined errored where both individual succeeded: {:?}", e);
                }
                (Err(e1), _, Ok(_)) | (_, Err(e1), Ok(_)) => {
                    panic!("combined succeeded where individual errored: {:?}", e1);
                }
            }
        }
    }
}
