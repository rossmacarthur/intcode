//! Parse the input into a syntax tree.

use std::collections::HashSet;

use crate::ast::{Instr, Param, Program, Stmt};
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

    /// Consumes the next parameter.
    fn eat_param(&mut self) -> Result<Param<'i>> {
        match self.eat()? {
            t if t.kind == Kind::Minus => {
                let t = self.eat_kind(Kind::Number)?;
                let value: i64 = self.str(t.span).parse().unwrap();
                Ok(Param::Exact(-value))
            }
            t if t.kind == Kind::Number => {
                let value = self.str(t.span).parse().unwrap();
                Ok(Param::Exact(value))
            }
            t if t.kind == Kind::Ident => {
                let value = self.str(t.span);
                Ok(Param::Ident(value))
            }
            t => Err(Error::new(
                t.span,
                format!("expected a parameter, found {}", t.kind.human()),
            )),
        }
    }

    /// Consumes the next two parameters.
    fn eat_params2(&mut self) -> Result<(Param<'i>, Param<'i>, Param<'i>)> {
        let x = self.eat_param()?;
        self.eat_kind(Kind::Comma)?;
        let y = self.eat_param()?;
        self.eat_kind(Kind::Comma)?;
        let z = self.eat_param()?;
        Ok((x, y, z))
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
                let p = self.eat_param()?;
                Instr::DataByte(p)
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
pub fn program<'i>(input: &'i str) -> Result<Program<'i>> {
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
                        instr: Instr::Add(Param::Ident("a"), Param::Ident("b"), Param::Exact(3))
                    },
                    Stmt {
                        label: None,
                        instr: Instr::Multiply(Param::Exact(3), Param::Ident("c"), Param::Exact(0))
                    },
                    Stmt {
                        label: None,
                        instr: Instr::Halt,
                    },
                    Stmt {
                        label: Some("a"),
                        instr: Instr::DataByte(Param::Exact(30)),
                    },
                    Stmt {
                        label: Some("b"),
                        instr: Instr::DataByte(Param::Exact(40)),
                    },
                    Stmt {
                        label: Some("c"),
                        instr: Instr::DataByte(Param::Exact(50)),
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
            ("ADD MUL", (4, 7), "expected a parameter, found a mnemonic"),
            ("YUP", (0, 3), "unknown operation mnemonic"),
        ];
        for (asm, (m, n), msg) in tests {
            assert_eq!(program(asm).unwrap_err(), Error::new(*m..*n, *msg));
        }
    }
}
