//! Parse an escaped string.

use std::string::String as StdString;

use dairy::String;

use crate::error::{Error, Result};
use crate::span::Span;

pub fn parse(input: &str, span: Span) -> Result<String<'_>> {
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
