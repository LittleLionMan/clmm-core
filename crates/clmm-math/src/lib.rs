use core as core_;

macro_rules! require {
    ($condition:expr, $error:expr $(,)?) => {
        if !($condition) {
            return Err($error);
        }
    };
}

macro_rules! require_gt {
    ($left:expr, $right:expr, $error:expr $(,)?) => {
        if !($left > $right) {
            return Err($error);
        }
    };
    ($left:expr, $right:expr $(,)?) => {
        if !($left > $right) {
            return Err($crate::error::MathError::RequireGtViolated);
        }
    };
}

macro_rules! require_gte {
    ($left:expr, $right:expr, $error:expr $(,)?) => {
        if !($left >= $right) {
            return Err($error);
        }
    };
    ($left:expr, $right:expr $(,)?) => {
        if !($left >= $right) {
            return Err($crate::error::MathError::RequireGteViolated);
        }
    };
}

macro_rules! err {
    ($error:expr) => {
        Err($error)
    };
}

macro_rules! error {
    ($error:expr) => {
        $error
    };
}

pub mod constants;
pub mod error;
pub mod libraries;

pub use error::{MathError, Result};
pub use libraries::*;
