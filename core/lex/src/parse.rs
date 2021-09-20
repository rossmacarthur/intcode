use std::string::String as StdString;

use dairy::String;

use intcode_error::span::Span;
use intcode_error::{Error, Result};

#[derive(Debug, Clone, Copy)]
pub enum Sign {
    Negative,
    Positive,
}

/// Parse an integer.
pub fn integer(input: &str, span: Span, sign: Sign) -> Result<i64> {
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
                let m = span.m + i + j;
                Error::new(
                    format!("invalid digit for base {} literal", radix),
                    m..m + 1,
                )
            })?;
            let err = || {
                Error::new(
                    format!("base {} literal out of range for 64-bit integer", radix),
                    span,
                )
            };
            let value = acc.checked_mul(radix.into()).ok_or_else(err)?;
            match sign {
                Sign::Positive => value.checked_add(x.into()),
                Sign::Negative => value.checked_sub(x.into()),
            }
            .ok_or_else(err)
        })
}

/// Parse a string.
pub fn string(input: &str, span: Span) -> Result<String<'_>> {
    let raw = span.as_str(input);
    if raw.contains('\\') {
        let mut iter = raw.char_indices().map(|(i, c)| (span.m + i, c));
        let mut value = StdString::new();
        while let Some((_, c)) = iter.next() {
            match c {
                '"' => continue,
                '\\' => {
                    let (i, esc) = iter.next().unwrap();
                    let c = match esc {
                        'n' => '\n',
                        'r' => '\r',
                        't' => '\t',
                        '\\' => '\\',
                        '"' => '"',
                        _ => {
                            let j = iter.next().unwrap().0;
                            return Err(Error::new("unknown escape character", i..j));
                        }
                    };
                    value.push(c);
                }
                c => value.push(c),
            }
        }
        Ok(String::owned(value))
    } else {
        Ok(String::borrowed(&raw[1..raw.len() - 1]))
    }
}
