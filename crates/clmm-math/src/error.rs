use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum MathError {
    #[error("calculation overflow")]
    CalculateOverflow,
    #[error("invalid tick array boundary")]
    InvalidTickArrayBoundary,
    #[error("invalid tick index")]
    InvalidTickIndex,
    #[error("liquidity add value error")]
    LiquidityAddValueErr,
    #[error("liquidity sub value error")]
    LiquiditySubValueErr,
    #[error("max token overflow")]
    MaxTokenOverflow,
    #[error("sqrt price limit overflow")]
    SqrtPriceLimitOverflow,
    #[error("sqrt price out of range")]
    SqrtPriceX64,
    #[error("tick upper overflow")]
    TickUpperOverflow,
    #[error("zero liquidity")]
    ZeroLiquidity,
    #[error("zero sqrt price")]
    ZeroSqrtPrice,
    #[error("require_gte violated")]
    RequireGteViolated,
    #[error("require_gt violated")]
    RequireGtViolated,
}

pub type Result<T> = core::result::Result<T, MathError>;
