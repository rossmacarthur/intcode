//! Parse the input into a syntax tree.

use anyhow::{bail, Result};

use crate::ast::{Instr, Param, Program, Stmt};
use crate::lex::{Kind, Span, Token, Tokens};

struct Parser<'i> {
    /// The original input string.
    input: &'i str,
    /// An iterator over the tokens in the input.
    tokens: Tokens<'i>,
}

impl Kind {
    fn is_interesting(&self) -> bool {
        !matches!(*self, Self::Whitespace | Self::Comment)
    }
}

impl<'i> Parser<'i> {
    fn new(input: &'i str) -> Self {
        let tokens = Tokens::new(input);
        Self { input, tokens }
    }

    fn str(&self, span: Span) -> &'i str {
        &self.input[span.m..span.n]
    }

    /// Returns the next interesting token.
    fn peek(&self) -> Result<Option<Token>> {
        self.tokens.clone().find(Kind::is_interesting)
    }

    /// Consumes the next interesting token.
    fn eat(&mut self) -> Result<Option<Token>> {
        self.tokens.find(Kind::is_interesting)
    }

    /// Consumes a token matching the given kind.
    fn eat_kind(&mut self, k: Kind) -> Result<Token> {
        match self.eat()? {
            Some(t) if t.kind == k => Ok(t),
            t => bail!("eat_kind(): expected {:?}, got `{:?}`", k, t),
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
            Some(t) if t.kind == Kind::Number => {
                let value = self.str(t.span).parse().unwrap();
                Ok(Param::Exact(value))
            }
            Some(t) if t.kind == Kind::Ident => {
                let value = self.str(t.span);
                Ok(Param::Ident(value))
            }
            t => bail!("eat_param(): unexpected token `{:?}`", t),
        }
    }

    /// Consumes the next three parameters.
    fn eat_params(&mut self) -> Result<(Param<'i>, Param<'i>, Param<'i>)> {
        let a = self.eat_param()?;
        self.eat_kind(Kind::Comma)?;
        let b = self.eat_param()?;
        self.eat_kind(Kind::Comma)?;
        let c = self.eat_param()?;
        Ok((a, b, c))
    }

    /// Consumes the next instruction.
    fn eat_instr(&mut self) -> Result<Instr<'i>> {
        let t = self.eat_kind(Kind::Opcode)?;
        let opcode = self.str(t.span);
        let instr = match opcode {
            "ADD" => {
                let (a, b, c) = self.eat_params()?;
                Instr::Add(a, b, c)
            }
            "MUL" => {
                let (a, b, c) = self.eat_params()?;
                Instr::Multiply(a, b, c)
            }
            "DB" => {
                let p = self.eat_param()?;
                Instr::DataByte(p)
            }
            "HLT" => Instr::Halt,
            o => bail!("eat_instr(): unknown opcode `{}`", o),
        };
        Ok(instr)
    }

    /// Consumes the next statement.
    fn eat_stmt(&mut self) -> Result<Stmt<'i>> {
        let label = match self.peek()? {
            Some(t) if t.kind == Kind::Ident => {
                self.eat_kind(Kind::Ident)?;
                self.eat_kind(Kind::Colon)?;
                Some(self.str(t.span))
            }
            _ => None,
        };
        self.eat_all(Kind::Newline)?;
        let instr = self.eat_instr()?;
        self.eat_all(Kind::Newline)?;
        Ok(Stmt { label, instr })
    }

    /// Consumes the next program.
    fn eat_program(&mut self) -> Result<Program<'i>> {
        let mut stmts = Vec::new();
        while self.peek()?.is_some() {
            stmts.push(self.eat_stmt()?)
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
    fn error() {
        let asm = "ADD MUL a, b, 3";
        assert_eq!(
            program(asm).unwrap_err().to_string(),
            "eat_param(): unexpected token `Some(Token { kind: Opcode, span: Span { m: 4, n: 7 } })"
        );
    }
}
