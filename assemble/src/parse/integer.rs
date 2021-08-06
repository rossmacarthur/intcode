//! Parse an integer.

use crate::error::{Error, Result};
use crate::span::Span;

pub enum Sign {
    Negative,
    Positive,
}

pub fn parse(input: &str, span: Span, sign: Sign) -> Result<i64> {
    let digits = span.as_str(input).as_bytes();
    let (i, radix) = match digits {
        [b'0', b'b', ..] => (2, 2),
        [b'0', b'o', ..] => (2, 8),
        [b'0', b'x', ..] => (2, 16),
        _ => (0, 10),
    };
    digits[i..]
        .iter()
        .enumerate()
        .filter(|(_, &d)| d != b'_')
        .try_fold(0i64, |acc, (j, &d)| {
            let x = (d as char).to_digit(radix).ok_or_else(|| {
                Error::new(
                    format!("invalid digit for base {} literal", radix),
                    span.m + i + j,
                )
            })?;
            let err = || {
                Error::new(
                    format!("base {} literal out of range for 64-bit integer", radix),
                    span,
                )
            };
            let value = acc.checked_mul(radix.into()).ok_or_else(err)?;
            Ok(match sign {
                Sign::Positive => value.checked_add(x.into()),
                Sign::Negative => value.checked_sub(x.into()),
            }
            .ok_or_else(err)?)
        })
}
