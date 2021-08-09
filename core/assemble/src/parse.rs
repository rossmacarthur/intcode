//! Parse the input into a syntax tree.

mod integer;
mod string;
#[cfg(test)]
mod tests;
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
    input: &'i str,
    tokens: Tokens<'i>,
    labels: HashSet<&'i str>,
}

enum Ident {
    Mnemonic,
    Label,
}

impl Ident {
    fn new(s: &str) -> Self {
        match s.chars().all(|c| matches!(c, '0'..='9'| 'A'..='Z' | '_')) {
            false => Self::Label,
            true => Self::Mnemonic,
        }
    }

    fn is_label(&self) -> bool {
        matches!(self, Self::Label)
    }

    fn is_mnemonic(&self) -> bool {
        matches!(self, Self::Mnemonic)
    }
}

impl Token {
    fn is_hash(&self) -> bool {
        matches!(self, Self::Hash)
    }

    fn is_eof(&self) -> bool {
        matches!(self, Self::Eof)
    }

    fn is_newline_or_eof(&self) -> bool {
        matches!(self, Self::Newline | Self::Eof)
    }

    fn is_not_newline_or_eof(&self) -> bool {
        !self.is_newline_or_eof()
    }

    fn is_delimiter(&self) -> bool {
        matches!(self, Self::Comma) || self.is_not_newline_or_eof()
    }

    fn is_interesting(&self) -> bool {
        !matches!(self, Self::Whitespace | Self::Comment)
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

    fn peek(&self) -> Result<(Span, Token)> {
        self.tokens.clone().find(Token::is_interesting)
    }

    fn is_next<P: FnOnce(&Token) -> bool>(&self, predicate: P) -> Result<bool> {
        Ok(matches!(self.peek()?, (_, tk) if predicate(&tk)))
    }

    fn eat(&mut self) -> Result<(Span, Token)> {
        self.tokens.find(Token::is_interesting)
    }

    fn advance(&mut self) {
        self.eat().unwrap();
    }

    fn eat_all(&mut self, want: Token) -> Result<()> {
        while self.is_next(|tk| *tk == want)? {
            self.advance();
        }
        Ok(())
    }

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
                let value = span.as_str(self.input);
                if Ident::new(value).is_mnemonic() {
                    return Err(Error::new("expected a parameter, found a mnemonic", span));
                }
                match self.peek()? {
                    (_, Token::Minus) => {
                        self.advance();
                        let (s, _) = self.expect(Token::Number)?;
                        let offset = integer::parse(self.input, s, Sign::Negative)?;
                        Ok((span.include(s), RawParam::Label(value, offset)))
                    }
                    (_, Token::Plus) => {
                        self.advance();
                        let (s, _) = self.expect(Token::Number)?;
                        let offset = integer::parse(self.input, s, Sign::Positive)?;
                        Ok((span.include(s), RawParam::Label(value, offset)))
                    }
                    _ => Ok((span, RawParam::Label(value, 0))),
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

    fn eat_params<T: TryUnpack<Param<'i>>>(&mut self, span: Span) -> Result<T> {
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
                    (true, RawParam::Label("rb", _)) => Err(Error::new(
                        "both immediate and relative mode specified",
                        span,
                    )),
                    (false, RawParam::Label("rb", offset)) => {
                        Ok(Param::Number(Mode::Relative, offset))
                    }
                    (_, RawParam::Label(value, offset)) => Ok(Param::Label(mode(), value, offset)),
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
                if matches!(raw, RawParam::Label("rb", _)) {
                    return Err(Error::new(
                        "relative mode not allowed with `DB`",
                        span.m..span.m + 2,
                    ));
                }
                Ok(raw)
            })
            .collect()
    }

    fn eat_instr(&mut self) -> Result<Instr<'i>> {
        let (span, _) = self.expect(Token::Ident)?;
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
            s => {
                let msg = match Ident::new(s) {
                    Ident::Mnemonic => "unknown operation mnemonic",
                    Ident::Label => "expected a mnemonic, found an identifier",
                };
                return Err(Error::new(msg, span));
            }
        };
        Ok(instr)
    }

    fn eat_stmt(&mut self) -> Result<Option<Stmt<'i>>> {
        self.eat_all(Token::Newline)?;
        if self.is_next(Token::is_eof)? {
            return Ok(None);
        }
        let label = match self.peek()? {
            (span, Token::Ident) if Ident::new(span.as_str(self.input)).is_label() => {
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

    fn eat_program(mut self) -> result::Result<Program<'i>, Vec<Error>> {
        let mut stmts = Vec::new();
        let mut errors = Vec::new();
        while let Some(stmt) = self.eat_stmt().transpose() {
            match stmt {
                Ok(stmt) => stmts.push(stmt),
                Err(err) => {
                    errors.push(err);
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
