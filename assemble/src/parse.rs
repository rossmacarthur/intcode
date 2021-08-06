//! Parse the input into a syntax tree.

mod integer;
mod string;
mod unpack;

use std::collections::HashSet;

use self::integer::Sign;
use self::unpack::TryUnpack;
use crate::ast::{Data, Instr, Mode, Param, Program, Stmt};
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

fn is_eof(tk: Option<Token>) -> bool {
    matches!(tk, None)
}

fn is_hash(tk: Option<Token>) -> bool {
    matches!(tk, Some(Token::Hash))
}

fn is_not_newline_or_eof(tk: Option<Token>) -> bool {
    !matches!(tk, Some(Token::Newline) | None)
}

fn is_another_param(tk: Option<Token>) -> bool {
    matches!(tk, Some(Token::Comma)) || is_not_newline_or_eof(tk)
}

fn is_not_whitespace_or_comment(tk: &Token) -> bool {
    !matches!(tk, Token::Whitespace | Token::Comment)
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
    fn peek(&self) -> Result<Option<(Span, Token)>> {
        self.tokens.clone().find(is_not_whitespace_or_comment)
    }

    fn peek_if<P>(&self, predicate: P) -> Result<bool>
    where
        P: FnOnce(Option<Token>) -> bool,
    {
        Ok(predicate(self.peek()?.map(|(_, tk)| tk)))
    }

    /// Consumes the next token, skipping over whitespace and comments.
    fn eat(&mut self) -> Result<(Span, Token)> {
        match self.tokens.find(is_not_whitespace_or_comment)? {
            Some(tk) => Ok(tk),
            None => {
                let m = self.input.chars().count();
                Err(Error::new("unexpected end of input", m..m))
            }
        }
    }

    /// Advances the iterator by one token.
    fn advance(&mut self) {
        self.eat().unwrap();
    }

    /// Consumes zero or more tokens matching the given one.
    fn eat_all(&mut self, want: Token) -> Result<()> {
        while matches!(self.peek()?, Some((_, tk)) if tk == want) {
            self.advance();
        }
        Ok(())
    }

    /// Consumes a token that must match the given one.
    fn expect(&mut self, want: Token) -> Result<(Span, Token)> {
        match self.eat()? {
            (span, tk) if tk == want => Ok((span, tk)),
            (span, tk) => Err(Error::new(
                format!("expected {}, found {}", want.human(), tk.human()),
                span,
            )),
        }
    }

    fn eat_data_param(&mut self) -> Result<(Span, Data<'i>)> {
        match self.eat()? {
            (span, Token::String) => {
                let value = string::parse(self.input, span)?;
                Ok((span, Data::String(value)))
            }
            (span, Token::Minus) => {
                let (s, _) = self.expect(Token::Number)?;
                let value = integer::parse(self.input, s, Sign::Negative)?;
                Ok((span.include(s), Data::Number(value)))
            }
            (span, Token::Number) => {
                let value = integer::parse(self.input, span, Sign::Positive)?;
                Ok((span, Data::Number(value)))
            }
            (span, Token::Ident) => {
                let ident = span.as_str(self.input);
                match self.peek()? {
                    Some((_, Token::Minus)) => {
                        self.advance();
                        let (s, _) = self.expect(Token::Number)?;
                        let offset = integer::parse(self.input, s, Sign::Negative)?;
                        Ok((span.include(s), Data::Ident(ident, offset)))
                    }
                    Some((_, Token::Plus)) => {
                        self.advance();
                        let (s, _) = self.expect(Token::Number)?;
                        let offset = integer::parse(self.input, s, Sign::Positive)?;
                        Ok((span.include(s), Data::Ident(ident, offset)))
                    }
                    _ => Ok((span, Data::Ident(ident, 0))),
                }
            }
            (span, tk) => Err(Error::new(
                format!("expected a parameter, found {}", tk.human()),
                span,
            )),
        }
    }

    fn eat_param(&mut self) -> Result<Param<'i>> {
        let with_mode = |span, data, mode| {
            let param = match data {
                Data::Ident("rb", offset) => Param::Number(Mode::Relative, offset),
                Data::Ident(ident, offset) => Param::Ident(mode, ident, offset),
                Data::Number(value) => Param::Number(mode, value),
                _ => return Err(Error::new("string parameter not allowed here", span)),
            };
            Ok(param)
        };
        if self.peek_if(is_hash)? {
            self.advance();
            let (span, data) = self.eat_data_param()?;
            with_mode(span, data, Mode::Immediate)
        } else {
            let (span, data) = self.eat_data_param()?;
            with_mode(span, data, Mode::Positional)
        }
    }

    fn eat_params<T>(&mut self, span: Span) -> Result<T>
    where
        Vec<Param<'i>>: TryUnpack<T>,
    {
        let mut params = Vec::new();
        if self.peek_if(is_not_newline_or_eof)? {
            params.push(self.eat_param()?);
            while self.peek_if(is_another_param)? {
                self.expect(Token::Comma)?;
                params.push(self.eat_param()?);
            }
        }
        params.try_unpack().map_err(|(exp, got)| {
            let msg = format!(
                "expected {} parameter{}, found {}",
                exp,
                if exp != 1 { "s" } else { "" },
                got,
            );
            Error::new(msg, span)
        })
    }

    fn eat_data_params(&mut self) -> Result<Vec<Data<'i>>> {
        let mut params = Vec::new();
        if self.peek_if(is_not_newline_or_eof)? {
            params.push(self.eat_data_param()?.1);
            while self.peek_if(is_another_param)? {
                self.advance();
                params.push(self.eat_data_param()?.1);
            }
        }
        Ok(params)
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
    fn eat_stmt(&mut self) -> Result<Stmt<'i>> {
        let label = match self.peek()? {
            Some((span, Token::Ident)) => {
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
        Ok(Stmt { label, instr })
    }

    /// Consumes the next program.
    fn eat_program(&mut self) -> Result<Program<'i>> {
        let mut stmts = Vec::new();
        loop {
            self.eat_all(Token::Newline)?;
            if self.peek_if(is_eof)? {
                break;
            }
            stmts.push(self.eat_stmt()?);
            if self.peek_if(is_eof)? {
                break;
            }
            self.expect(Token::Newline)?;
        }
        Ok(Program { stmts })
    }
}

/// Parse intcode assembly.
pub fn program(input: &str) -> Result<Program> {
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
            let (span, data) = Parser::new(asm).eat_data_param().unwrap();
            assert_eq!(span, range.into());
            match data {
                Data::String(value) => {
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
                        instr: Instr::Data(vec![Data::Number(30)]),
                    },
                    Stmt {
                        label: Some("b"),
                        instr: Instr::Data(vec![Data::Number(40)]),
                    },
                    Stmt {
                        label: Some("c"),
                        instr: Instr::Data(vec![Data::Number(50)]),
                    },
                ]
            }
        );
    }

    #[test]
    fn eat_program_errors() {
        let tests = [
            ("ADD x,", 6..6, "unexpected end of input"),
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
            assert_eq!(program(asm).unwrap_err(), Error::new(msg, span));
        }
    }
}
