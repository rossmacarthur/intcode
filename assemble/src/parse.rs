//! Parse the input into a syntax tree.

mod integer;
mod string;
mod unpack;

use std::collections::HashSet;
use std::result;

use self::integer::Sign;
use self::unpack::TryUnpack;
use crate::ast::{Instr, Mode, Param, Program, RawParam, Stmt};
use crate::error::{Error, Result};
use crate::lex::{Token, Tokens};
use crate::span::Span;

struct Parser<'i> {
    /// The original input string.
    input: &'i str,
    /// An iterator over the tokens in the input.
    tokens: Tokens<'i>,
    /// Labels that we have already seen.
    labels: HashSet<&'i str>,
}

impl Token {
    fn is_hash(&self) -> bool {
        matches!(self, Token::Hash)
    }

    fn is_eof(&self) -> bool {
        matches!(self, Token::Eof)
    }

    fn is_newline_or_eof(&self) -> bool {
        matches!(self, Token::Newline | Token::Eof)
    }

    fn is_not_newline_or_eof(&self) -> bool {
        !self.is_newline_or_eof()
    }

    fn is_delimiter(&self) -> bool {
        matches!(self, Token::Comma) || self.is_not_newline_or_eof()
    }

    fn is_interesting(&self) -> bool {
        !matches!(self, Token::Whitespace | Token::Comment)
    }
}

impl<'i> Parser<'i> {
    fn new(input: &'i str) -> Self {
        let tokens = Tokens::new(input);
        Self {
            input,
            tokens,
            labels: HashSet::new(),
        }
    }

    /// Returns the next token, ignoring whitespace and comments.
    fn peek(&self) -> Result<(Span, Token)> {
        self.tokens.clone().find(Token::is_interesting)
    }

    fn is_next<P: FnOnce(&Token) -> bool>(&self, predicate: P) -> Result<bool> {
        Ok(matches!(self.peek()?, (_, tk) if predicate(&tk)))
    }

    /// Consumes the next token, skipping over whitespace and comments.
    fn eat(&mut self) -> Result<(Span, Token)> {
        self.tokens.find(Token::is_interesting)
    }

    /// Advances the iterator by one token.
    fn advance(&mut self) {
        self.eat().unwrap();
    }

    /// Consumes zero or more tokens matching the given one.
    fn eat_all(&mut self, want: Token) -> Result<()> {
        while self.is_next(|tk| *tk == want)? {
            self.advance();
        }
        Ok(())
    }

    /// Consumes a token that must match the given one.
    fn expect(&mut self, want: Token) -> Result<(Span, Token)> {
        match self.peek()? {
            (span, tk) if tk == want => {
                self.advance();
                Ok((span, tk))
            }
            (span, tk) => Err(Error::new(
                format!("expected {}, found {}", want.human(), tk.human()),
                span,
            )),
        }
    }

    fn _eat_raw_param(&mut self) -> Result<(Span, RawParam<'i>)> {
        match self.eat()? {
            (span, Token::String) => {
                let value = string::parse(self.input, span)?;
                Ok((span, RawParam::String(value)))
            }
            (span, Token::Minus) => {
                let (s, _) = self.expect(Token::Number)?;
                let value = integer::parse(self.input, s, Sign::Negative)?;
                Ok((span.include(s), RawParam::Number(value)))
            }
            (span, Token::Number) => {
                let value = integer::parse(self.input, span, Sign::Positive)?;
                Ok((span, RawParam::Number(value)))
            }
            (span, Token::Ident) => {
                let ident = span.as_str(self.input);
                match self.peek()? {
                    (_, Token::Minus) => {
                        self.advance();
                        let (s, _) = self.expect(Token::Number)?;
                        let offset = integer::parse(self.input, s, Sign::Negative)?;
                        Ok((span.include(s), RawParam::Ident(ident, offset)))
                    }
                    (_, Token::Plus) => {
                        self.advance();
                        let (s, _) = self.expect(Token::Number)?;
                        let offset = integer::parse(self.input, s, Sign::Positive)?;
                        Ok((span.include(s), RawParam::Ident(ident, offset)))
                    }
                    _ => Ok((span, RawParam::Ident(ident, 0))),
                }
            }
            (span, tk) => Err(Error::new(
                format!("expected a parameter, found {}", tk.human()),
                span,
            )),
        }
    }

    fn eat_raw_param(&mut self) -> Result<(bool, Span, RawParam<'i>)> {
        if self.is_next(Token::is_hash)? {
            let (span, _) = self.expect(Token::Hash)?;
            let (s, raw) = self._eat_raw_param()?;
            Ok((true, span.include(s), raw))
        } else {
            let (span, raw) = self._eat_raw_param()?;
            Ok((false, span, raw))
        }
    }

    fn eat_raw_params(&mut self) -> Result<Vec<(bool, Span, RawParam<'i>)>> {
        let mut params = Vec::new();
        if self.is_next(Token::is_not_newline_or_eof)? {
            params.push(self.eat_raw_param()?);
            while self.is_next(Token::is_delimiter)? {
                self.expect(Token::Comma)?;
                params.push(self.eat_raw_param()?);
            }
        }
        Ok(params)
    }

    fn eat_params<T>(&mut self, span: Span) -> Result<T>
    where
        T: TryUnpack<Param<'i>>,
    {
        let params: Vec<_> = self
            .eat_raw_params()?
            .into_iter()
            .map(|(prefix, span, raw)| {
                let mode = || match prefix {
                    true => Mode::Immediate,
                    false => Mode::Positional,
                };
                match (prefix, raw) {
                    (_, RawParam::String(_)) => {
                        Err(Error::new("string parameter only allowed with `DB`", span))
                    }
                    (true, RawParam::Ident("rb", _)) => Err(Error::new(
                        "both immediate and relative mode specified",
                        span,
                    )),
                    (false, RawParam::Ident("rb", offset)) => {
                        Ok(Param::Number(Mode::Relative, offset))
                    }
                    (_, RawParam::Ident(ident, offset)) => Ok(Param::Ident(mode(), ident, offset)),
                    (_, RawParam::Number(value)) => Ok(Param::Number(mode(), value)),
                }
            })
            .collect::<Result<_>>()?;
        T::try_unpack(params).map_err(|(exp, got)| {
            let msg = format!(
                "expected {} parameter{}, found {}",
                exp,
                if exp != 1 { "s" } else { "" },
                got,
            );
            Error::new(msg, span)
        })
    }

    fn eat_data_params(&mut self) -> Result<Vec<RawParam<'i>>> {
        self.eat_raw_params()?
            .into_iter()
            .map(|(prefix, span, raw)| {
                if prefix {
                    return Err(Error::new("immediate mode not allowed with `DB`", span.m));
                }
                if matches!(raw, RawParam::Ident("rb", _)) {
                    return Err(Error::new(
                        "relative mode not allowed with `DB`",
                        span.m..span.m + 2,
                    ));
                }
                Ok(raw)
            })
            .collect()
    }

    /// Consumes the next instruction.
    fn eat_instr(&mut self) -> Result<Instr<'i>> {
        let (span, _) = self.expect(Token::Mnemonic)?;
        let opcode = span.as_str(self.input);
        let instr = match opcode {
            "ADD" => {
                let (x, y, z) = self.eat_params(span)?;
                Instr::Add(x, y, z)
            }
            "MUL" => {
                let (x, y, z) = self.eat_params(span)?;
                Instr::Multiply(x, y, z)
            }
            "JNZ" => {
                let (x, y) = self.eat_params(span)?;
                Instr::JumpNonZero(x, y)
            }
            "JZ" => {
                let (x, y) = self.eat_params(span)?;
                Instr::JumpZero(x, y)
            }
            "LT" => {
                let (x, y, z) = self.eat_params(span)?;
                Instr::LessThan(x, y, z)
            }
            "EQ" => {
                let (x, y, z) = self.eat_params(span)?;
                Instr::Equal(x, y, z)
            }
            "IN" => {
                let (p,) = self.eat_params(span)?;
                Instr::Input(p)
            }
            "OUT" => {
                let (p,) = self.eat_params(span)?;
                Instr::Output(p)
            }
            "ARB" => {
                let (p,) = self.eat_params(span)?;
                Instr::AdjustRelativeBase(p)
            }
            "HLT" => {
                self.eat_params(span)?;
                Instr::Halt
            }
            "DB" => {
                let data = self.eat_data_params()?;
                Instr::Data(data)
            }
            _ => return Err(Error::new("unknown operation mnemonic", span)),
        };
        Ok(instr)
    }

    /// Consumes the next statement.
    fn eat_stmt(&mut self) -> Result<Option<Stmt<'i>>> {
        self.eat_all(Token::Newline)?;
        if self.is_next(Token::is_eof)? {
            return Ok(None);
        }
        let label = match self.peek()? {
            (span, Token::Ident) => {
                self.advance();
                self.expect(Token::Colon)?;
                let label = span.as_str(self.input);
                if label == "rb" {
                    return Err(Error::new("label is reserved for the relative base", span));
                } else if !self.labels.insert(label) {
                    return Err(Error::new("label already used", span));
                }
                Some(label)
            }
            _ => None,
        };
        self.eat_all(Token::Newline)?;
        let instr = self.eat_instr()?;
        if !self.is_next(Token::is_eof)? {
            self.expect(Token::Newline)?;
        }
        Ok(Some(Stmt { label, instr }))
    }

    /// Consumes the next program.
    fn eat_program(&mut self) -> result::Result<Program<'i>, Vec<Error>> {
        let mut stmts = Vec::new();
        let mut errors = Vec::new();
        while let Some(stmt) = self.eat_stmt().transpose() {
            match stmt {
                Ok(stmt) => stmts.push(stmt),
                Err(err) => {
                    errors.push(err);
                    // Go to the end of the line...
                    while !self.is_next(Token::is_newline_or_eof).unwrap_or(false) {
                        drop(self.eat());
                    }
                }
            }
        }
        match errors.is_empty() {
            true => Ok(Program { stmts }),
            false => Err(errors),
        }
    }
}

/// Parse intcode assembly.
pub fn program(input: &str) -> result::Result<Program, Vec<Error>> {
    Parser::new(input).eat_program()
}

#[cfg(test)]
mod tests {
    use super::*;

    use pretty_assertions::assert_eq;

    #[test]
    fn eat_raw_param_strings() {
        let tests = [
            ("\"Hello World!\"", "Hello World!", 0..14, true),
            ("\"Hello World!\\n\"", "Hello World!\n", 0..16, false),
            ("\"😎\"", "😎", 0..6, true),
            ("\"😎\\t\"", "😎\t", 0..8, false),
        ];
        for (asm, string, range, is_borrowed) in tests {
            let (_, span, raw) = Parser::new(asm).eat_raw_param().unwrap();
            assert_eq!(span, range.into());
            match raw {
                RawParam::String(value) => {
                    assert_eq!(value, string);
                    assert_eq!(value.is_borrowed(), is_borrowed);
                }
                _ => unreachable!(),
            }
        }
    }

    #[test]
    fn eat_program_basic() {
        let asm = r#"
; An example program from Advent of Code 2019 Day 2.

ADD a, b, 3
MUL 3, c, 0 ; Inline comment
HLT

a: DB 30
b: DB 40
c: DB 50"#;
        assert_eq!(
            program(asm).unwrap(),
            Program {
                stmts: vec![
                    Stmt {
                        label: None,
                        instr: Instr::Add(
                            Param::Ident(Mode::Positional, "a", 0),
                            Param::Ident(Mode::Positional, "b", 0),
                            Param::Number(Mode::Positional, 3)
                        )
                    },
                    Stmt {
                        label: None,
                        instr: Instr::Multiply(
                            Param::Number(Mode::Positional, 3),
                            Param::Ident(Mode::Positional, "c", 0),
                            Param::Number(Mode::Positional, 0)
                        )
                    },
                    Stmt {
                        label: None,
                        instr: Instr::Halt,
                    },
                    Stmt {
                        label: Some("a"),
                        instr: Instr::Data(vec![RawParam::Number(30)]),
                    },
                    Stmt {
                        label: Some("b"),
                        instr: Instr::Data(vec![RawParam::Number(40)]),
                    },
                    Stmt {
                        label: Some("c"),
                        instr: Instr::Data(vec![RawParam::Number(50)]),
                    },
                ]
            }
        );
    }

    #[test]
    fn eat_program_errors() {
        let tests = [
            ("ADD x,", 6..6, "expected a parameter, found end of input"),
            ("ADD", 0..3, "expected 3 parameters, found 0"),
            ("ADD @", 4..5, "unexpected character"),
            ("ADD x, y", 0..3, "expected 3 parameters, found 2"),
            ("ADD x, y, z, w", 0..3, "expected 3 parameters, found 4"),
            ("ADD #-a", 6..7, "expected a number, found an identifier"),
            ("ADD MUL", 4..7, "expected a parameter, found a mnemonic"),
            ("YUP", 0..3, "unknown operation mnemonic"),
            ("rb: DB 0", 0..2, "label is reserved for the relative base"),
            ("label: DB 0\nlabel: DB 0", 12..17, "label already used"),
        ];
        for (asm, span, msg) in tests {
            assert_eq!(program(asm).unwrap_err(), [Error::new(msg, span)]);
        }
    }
}
