//! Parse the input into a syntax tree.

use std::collections::HashSet;

use crate::ast::{Data, Instr, Mode, Param, Program, Stmt};
use crate::error::{Error, Result};
use crate::lex::{Kind, Span, Token, Tokens};

struct Parser<'i> {
    /// The original input string.
    input: &'i str,
    /// An iterator over the tokens in the input.
    tokens: Tokens<'i>,
    /// Labels that we have already seen.
    labels: HashSet<&'i str>,
}

fn is_not_whitespace_or_comment(k: &Kind) -> bool {
    !matches!(*k, Kind::Whitespace | Kind::Comment)
}

fn is_interesting(k: &Kind) -> bool {
    !matches!(*k, Kind::Newline | Kind::Whitespace | Kind::Comment)
}

enum RawParam<'i> {
    Ident(&'i str, i64),
    Number(i64),
}

impl<'i> RawParam<'i> {
    fn with_mode(self, mode: Mode) -> Param<'i> {
        match self {
            Self::Ident(ident, offset) => Param::Ident(mode, ident, offset),
            Self::Number(value) => Param::Number(mode, value),
        }
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

    fn str(&self, span: Span) -> &'i str {
        &self.input[span.range()]
    }

    /// Returns the next token, ignoring over whitespace and comments.
    fn peek(&self) -> Result<Option<Token>> {
        self.tokens.clone().find(is_not_whitespace_or_comment)
    }

    /// Returns the next interesting token.
    fn peek_interesting(&self) -> Result<Option<Token>> {
        self.tokens.clone().find(is_interesting)
    }

    /// Consumes the next token, skipping over whitespace and comments.
    fn eat(&mut self) -> Result<Token> {
        match self.tokens.find(is_not_whitespace_or_comment)? {
            Some(t) => Ok(t),
            None => {
                let m = self.input.chars().count();
                Err(Error::new(Span { m, n: m + 1 }, "unexpected end of input"))
            }
        }
    }

    /// Consumes a token matching the given kind.
    fn eat_kind(&mut self, k: Kind) -> Result<Token> {
        match self.eat()? {
            t if t.kind == k => Ok(t),
            t => Err(Error::new(
                t.span,
                format!("expected {}, found {}", k.human(), t.kind.human()),
            )),
        }
    }

    /// Consumes one or more tokens matching the given kind.
    fn eat_all(&mut self, k: Kind) -> Result<()> {
        while matches!(self.peek()?, Some(t) if t.kind == k) {
            self.eat_kind(k)?;
        }
        Ok(())
    }

    fn eat_raw_param(&mut self) -> Result<RawParam<'i>> {
        match self.eat()? {
            t if t.kind == Kind::Minus => {
                let t = self.eat_kind(Kind::Number)?;
                let value: i64 = self.str(t.span).parse().unwrap();
                Ok(RawParam::Number(-value))
            }
            t if t.kind == Kind::Number => {
                let value = self.str(t.span).parse().unwrap();
                Ok(RawParam::Number(value))
            }
            t if t.kind == Kind::Ident => {
                let ident = self.str(t.span);
                let offset = match self.peek()? {
                    Some(t) if t.kind == Kind::Plus => {
                        self.eat_kind(Kind::Plus)?;
                        let t = self.eat_kind(Kind::Number)?;
                        let value: i64 = self.str(t.span).parse().unwrap();
                        value
                    }
                    Some(t) if t.kind == Kind::Minus => {
                        self.eat_kind(Kind::Minus)?;
                        let t = self.eat_kind(Kind::Number)?;
                        let value: i64 = self.str(t.span).parse().unwrap();
                        -value
                    }
                    _ => 0,
                };
                Ok(RawParam::Ident(ident, offset))
            }
            t => Err(Error::new(
                t.span,
                format!("expected a number or identifier, found {}", t.kind.human()),
            )),
        }
    }

    /// Consumes the next parameter.
    fn eat_param(&mut self) -> Result<Param<'i>> {
        match self.peek()? {
            Some(t) if t.kind == Kind::Tilde => {
                self.eat_kind(Kind::Tilde)?;
                let p = self.eat_raw_param()?;
                Ok(p.with_mode(Mode::Relative))
            }
            Some(t) if t.kind == Kind::Hash => {
                self.eat_kind(Kind::Hash)?;
                let p = self.eat_raw_param()?;
                Ok(p.with_mode(Mode::Immediate))
            }
            _ => {
                let p = self.eat_raw_param()?;
                Ok(p.with_mode(Mode::Positional))
            }
        }
    }

    /// Consumes the next two parameters.
    fn eat_params2(&mut self) -> Result<(Param<'i>, Param<'i>)> {
        let x = self.eat_param()?;
        self.eat_kind(Kind::Comma)?;
        let y = self.eat_param()?;
        Ok((x, y))
    }

    /// Consumes the next three parameters.
    fn eat_params3(&mut self) -> Result<(Param<'i>, Param<'i>, Param<'i>)> {
        let x = self.eat_param()?;
        self.eat_kind(Kind::Comma)?;
        let y = self.eat_param()?;
        self.eat_kind(Kind::Comma)?;
        let z = self.eat_param()?;
        Ok((x, y, z))
    }

    /// Consumes the next data.
    fn eat_data(&mut self) -> Result<Data<'i>> {
        match self.eat()? {
            t if t.kind == Kind::Minus => {
                let t = self.eat_kind(Kind::Number)?;
                let value: i64 = self.str(t.span).parse().unwrap();
                Ok(Data::Number(-value))
            }
            t if t.kind == Kind::Number => {
                let value = self.str(t.span).parse().unwrap();
                Ok(Data::Number(value))
            }
            t if t.kind == Kind::String => {
                let value = self.str(Span::from(t.span.m + 1..t.span.n - 1));
                Ok(Data::String(value))
            }
            t => Err(Error::new(
                t.span,
                format!("expected a number or string, found {}", t.kind.human()),
            )),
        }
    }

    /// Consumes multiple data params.
    fn eat_data_params(&mut self) -> Result<Vec<Data<'i>>> {
        let mut data = Vec::new();
        data.push(self.eat_data()?);
        loop {
            match self.peek()? {
                Some(t) if t.kind == Kind::Comma => {
                    self.eat_kind(Kind::Comma)?;
                    data.push(self.eat_data()?)
                }
                _ => break Ok(data),
            }
        }
    }

    /// Consumes the next instruction.
    fn eat_instr(&mut self) -> Result<Instr<'i>> {
        let t = self.eat_kind(Kind::Mnemonic)?;
        let opcode = self.str(t.span);
        let instr = match opcode {
            "ADD" => {
                let (x, y, z) = self.eat_params3()?;
                Instr::Add(x, y, z)
            }
            "MUL" => {
                let (x, y, z) = self.eat_params3()?;
                Instr::Multiply(x, y, z)
            }
            "JNZ" => {
                let (x, y) = self.eat_params2()?;
                Instr::JumpNonZero(x, y)
            }
            "JZ" => {
                let (x, y) = self.eat_params2()?;
                Instr::JumpZero(x, y)
            }
            "LT" => {
                let (x, y, z) = self.eat_params3()?;
                Instr::LessThan(x, y, z)
            }
            "EQ" => {
                let (x, y, z) = self.eat_params3()?;
                Instr::Equal(x, y, z)
            }
            "IN" => {
                let p = self.eat_param()?;
                Instr::Input(p)
            }
            "OUT" => {
                let p = self.eat_param()?;
                Instr::Output(p)
            }
            "ARB" => {
                let p = self.eat_param()?;
                Instr::AdjustRelativeBase(p)
            }
            "DB" => {
                let data = self.eat_data_params()?;
                Instr::Data(data)
            }
            "HLT" => Instr::Halt,
            _ => return Err(Error::new(t.span, "unknown operation mnemonic")),
        };
        Ok(instr)
    }

    /// Consumes the next statement.
    fn eat_stmt(&mut self) -> Result<Stmt<'i>> {
        let label = match self.peek()? {
            Some(t) if t.kind == Kind::Ident => {
                self.eat_kind(Kind::Ident)?;
                self.eat_kind(Kind::Colon)?;
                let label = self.str(t.span);
                if !self.labels.insert(label) {
                    return Err(Error::new(t.span, "label already used"));
                }
                Some(label)
            }
            _ => None,
        };
        self.eat_all(Kind::Newline)?;
        let instr = self.eat_instr()?;
        Ok(Stmt { label, instr })
    }

    /// Consumes the next program.
    fn eat_program(&mut self) -> Result<Program<'i>> {
        let mut stmts = Vec::new();
        loop {
            self.eat_all(Kind::Newline)?;
            stmts.push(self.eat_stmt()?);
            // Either a newline or EOF is okay here, we don't want trailing
            // newlines to be required.
            match self.peek_interesting()? {
                None => break,
                Some(_) => {
                    self.eat_kind(Kind::Newline)?;
                }
            }
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
    fn basic() {
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
    fn errors() {
        let tests: &[(&str, (usize, usize), &str)] = &[
            ("ADD", (3, 4), "unexpected end of input"),
            ("ADD @", (4, 5), "unexpected character"),
            ("ADD x y", (6, 7), "expected a comma, found an identifier"),
            (
                "ADD MUL",
                (4, 7),
                "expected a number or identifier, found a mnemonic",
            ),
            ("ADD #-a", (6, 7), "expected a number, found an identifier"),
            ("YUP", (0, 3), "unknown operation mnemonic"),
        ];
        for (asm, (m, n), msg) in tests {
            assert_eq!(program(asm).unwrap_err(), Error::new(*m..*n, *msg));
        }
    }
}
