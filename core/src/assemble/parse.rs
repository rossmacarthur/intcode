//! Parse the input into a syntax tree.

mod integer;
mod string;
mod unpack;

use std::result;

use self::integer::Sign;
use self::unpack::TryUnpack;
use crate::assemble::ast::{Instr, Label, Mode, Param, Program, RawParam, Stmt};
use crate::assemble::lex::{Token, Tokens};
use crate::error::{Error, Result};
use crate::span::{Span, S};

struct Parser<'i> {
    input: &'i str,
    tokens: Tokens<'i>,
}

enum Ident {
    Mnemonic,
    Label,
}

impl Ident {
    fn new(s: &str) -> Self {
        match s.chars().all(|c| matches!(c, '0'..='9'| 'A'..='Z')) {
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

impl<'i> From<&'i str> for Label<'i> {
    fn from(s: &'i str) -> Self {
        match s {
            "_" => Self::Underscore,
            "ip" => Self::InstructionPointer,
            label => Self::Fixed(label),
        }
    }
}

impl<'i> Parser<'i> {
    fn new(input: &'i str) -> Self {
        let tokens = Tokens::new(input);
        Self { input, tokens }
    }

    fn peek(&self) -> Result<S<Token>> {
        self.tokens.clone().find(Token::is_interesting)
    }

    fn is_next<P: FnOnce(&Token) -> bool>(&self, predicate: P) -> Result<bool> {
        Ok(matches!(self.peek()?, tk if predicate(&tk)))
    }

    fn eat(&mut self) -> Result<S<Token>> {
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

    fn expect(&mut self, want: Token) -> Result<S<Token>> {
        match self.peek()? {
            token if *token == want => {
                self.advance();
                Ok(token)
            }
            S(tk, span) => Err(Error::new(
                format!("expected {}, found {}", want.human(), tk.human()),
                span,
            )),
        }
    }

    fn _eat_raw_param(&mut self) -> Result<S<RawParam<'i>>> {
        match self.eat()? {
            S(Token::String, span) => {
                let value = string::parse(self.input, span)?;
                Ok(S(RawParam::String(value), span))
            }
            S(Token::Minus, span) => {
                let S(_, s) = self.expect(Token::Number)?;
                let value = integer::parse(self.input, s, Sign::Negative)?;
                Ok(S(RawParam::Number(value), span.include(s)))
            }
            S(Token::Number, span) => {
                let value = integer::parse(self.input, span, Sign::Positive)?;
                Ok(S(RawParam::Number(value), span))
            }
            S(Token::Ident, span) => {
                let value = span.as_str(self.input);
                if Ident::new(value).is_mnemonic() {
                    return Err(Error::new("expected a parameter, found a mnemonic", span));
                }
                let label = S(Label::from(value), span);
                match *self.peek()? {
                    Token::Minus => {
                        self.advance();
                        let S(_, s) = self.expect(Token::Number)?;
                        let offset = integer::parse(self.input, s, Sign::Negative)?;
                        Ok(S(RawParam::Label(label, offset), span.include(s)))
                    }
                    Token::Plus => {
                        self.advance();
                        let S(_, s) = self.expect(Token::Number)?;
                        let offset = integer::parse(self.input, s, Sign::Positive)?;
                        Ok(S(RawParam::Label(label, offset), span.include(s)))
                    }
                    _ => Ok(S(RawParam::Label(label, 0), span)),
                }
            }
            S(tk, span) => Err(Error::new(
                format!("expected a parameter, found {}", tk.human()),
                span,
            )),
        }
    }

    fn eat_raw_param(&mut self) -> Result<(Option<Span>, S<RawParam<'i>>)> {
        if self.is_next(Token::is_hash)? {
            let S(_, span) = self.expect(Token::Hash)?;
            let S(raw, s) = self._eat_raw_param()?;
            Ok((Some(span), S(raw, span.include(s))))
        } else {
            Ok((None, self._eat_raw_param()?))
        }
    }

    fn eat_raw_params(&mut self) -> Result<Vec<(Option<Span>, S<RawParam<'i>>)>> {
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

    fn eat_params<T: TryUnpack<S<Param<'i>>>>(&mut self, span: Span) -> Result<T> {
        let params: Vec<_> = self
            .eat_raw_params()?
            .into_iter()
            .map(|(prefix, raw_param)| {
                let mode = || match prefix {
                    Some(_) => Mode::Immediate,
                    None => Mode::Positional,
                };
                match (prefix, raw_param) {
                    (_, S(RawParam::String(_), span)) => {
                        Err(Error::new("string parameter only allowed with `DB`", span))
                    }
                    (Some(_), S(RawParam::Label(S(Label::Fixed("rb"), _), _), span)) => Err(
                        Error::new("both immediate and relative mode specified", span),
                    ),
                    (None, S(RawParam::Label(S(Label::Fixed("rb"), _), offset), span)) => {
                        Ok(S(Param::Number(Mode::Relative, offset), span))
                    }
                    (_, S(RawParam::Label(value, offset), span)) => {
                        Ok(S(Param::Label(mode(), value, offset), span))
                    }
                    (_, S(RawParam::Number(value), span)) => {
                        Ok(S(Param::Number(mode(), value), span))
                    }
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

    fn eat_data_params(&mut self) -> Result<Vec<S<RawParam<'i>>>> {
        self.eat_raw_params()?
            .into_iter()
            .map(|(prefix, raw_param)| {
                if let Some(span) = prefix {
                    return Err(Error::new("immediate mode not allowed with `DB`", span));
                }
                if let S(RawParam::Label(S(Label::Fixed("rb"), span), _), _) = raw_param {
                    return Err(Error::new("relative mode not allowed with `DB`", span));
                }
                Ok(raw_param)
            })
            .collect()
    }

    fn eat_instr(&mut self) -> Result<S<Instr<'i>>> {
        let S(_, span) = self.expect(Token::Ident)?;
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
        let S(_, s) = self.peek()?;
        Ok(S(instr, span.include(s.m..s.m)))
    }

    fn eat_stmt(&mut self) -> Result<Option<Stmt<'i>>> {
        self.eat_all(Token::Newline)?;
        if self.is_next(Token::is_eof)? {
            return Ok(None);
        }
        let label = match self.peek()? {
            S(Token::Ident, span) => {
                let value = span.as_str(self.input);
                if Ident::new(value).is_label() {
                    self.advance();
                    self.expect(Token::Colon)?;
                    Some(S(Label::from(value), span))
                } else {
                    None
                }
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
pub fn program(input: &str) -> result::Result<Program<'_>, Vec<Error>> {
    Parser::new(input).eat_program()
}
