//! Parse the input into a syntax tree.

use std::collections::HashSet;

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

fn is_not_whitespace_or_comment(token: &Token) -> bool {
    !matches!(token, Token::Whitespace | Token::Comment)
}

fn is_interesting(token: &Token) -> bool {
    !matches!(token, Token::Newline | Token::Whitespace | Token::Comment)
}

fn with_mode<'i>(span: Span, data: Data<'i>, mode: Mode) -> Result<Param<'i>> {
    match data {
        Data::Ident(ident, offset) => Ok(Param::Ident(mode, ident, offset)),
        Data::Number(value) => Ok(Param::Number(mode, value)),
        _ => Err(Error::new("string parameter not allowed here", span)),
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

    /// Returns the next token, ignoring over whitespace and comments.
    fn peek(&self) -> Result<Option<(Span, Token)>> {
        self.tokens.clone().find(is_not_whitespace_or_comment)
    }

    /// Returns the next interesting token.
    fn peek_interesting(&self) -> Result<Option<(Span, Token)>> {
        self.tokens.clone().find(is_interesting)
    }

    /// Consumes the next token, skipping over whitespace and comments.
    fn eat(&mut self) -> Result<(Span, Token)> {
        match self.tokens.find(is_not_whitespace_or_comment)? {
            Some(tk) => Ok(tk),
            None => {
                let m = self.input.chars().count();
                Err(Error::new("unexpected end of input", m..m + 1))
            }
        }
    }

    /// Consumes a token matching the given one.
    fn eat_token(&mut self, want: Token) -> Result<(Span, Token)> {
        match self.eat()? {
            (span, tk) if tk == want => Ok((span, tk)),
            (span, tk) => Err(Error::new(
                format!("expected {}, found {}", want.human(), tk.human()),
                span,
            )),
        }
    }

    /// Consumes one or more tokens matching the given one.
    fn eat_all(&mut self, want: Token) -> Result<()> {
        while matches!(self.peek()?, Some((_, tk)) if tk == want) {
            self.eat_token(want)?;
        }
        Ok(())
    }

    fn eat_ident(&mut self, span: Span) -> Result<(&'i str, i64)> {
        let ident = span.as_str(self.input);
        let offset = match self.peek()? {
            Some((_, Token::Plus)) => {
                self.eat_token(Token::Plus)?;
                let (span, _) = self.eat_token(Token::Number)?;
                let value: i64 = span.parse(self.input);
                value
            }
            Some((_, Token::Minus)) => {
                self.eat_token(Token::Minus)?;
                let (span, _) = self.eat_token(Token::Number)?;
                let value: i64 = span.parse(self.input);
                -value
            }
            _ => 0,
        };
        Ok((ident, offset))
    }

    fn eat_raw_param(&mut self) -> Result<(Span, Data<'i>)> {
        match self.eat()? {
            (Span { m, n }, Token::String) => {
                let value = Span::new(m + 1, n - 1).as_str(self.input);
                Ok((Span { m, n }, Data::String(value)))
            }
            (Span { m, .. }, Token::Minus) => {
                let (span, _) = self.eat_token(Token::Number)?;
                let value: i64 = span.parse(self.input);
                Ok((Span { m, n: span.n }, Data::Number(-value)))
            }
            (span, Token::Number) => {
                let value = span.parse(self.input);
                Ok((span, Data::Number(value)))
            }
            (span, Token::Ident) => {
                let (ident, offset) = self.eat_ident(span)?;
                Ok((span, Data::Ident(ident, offset)))
            }
            (span, tk) => Err(Error::new(
                format!("expected a parameter, found {}", tk.human()),
                span,
            )),
        }
    }

    /// Consumes the next parameter.
    fn eat_param(&mut self) -> Result<Param<'i>> {
        match self.peek()? {
            Some((_, Token::Tilde)) => {
                self.eat_token(Token::Tilde)?;
                let (span, data) = self.eat_raw_param()?;
                with_mode(span, data, Mode::Relative)
            }
            Some((_, Token::Hash)) => {
                self.eat_token(Token::Hash)?;
                let (span, data) = self.eat_raw_param()?;
                with_mode(span, data, Mode::Immediate)
            }
            _ => {
                let (span, data) = self.eat_raw_param()?;
                with_mode(span, data, Mode::Positional)
            }
        }
    }

    /// Consumes the next two parameters.
    fn eat_params2(&mut self) -> Result<(Param<'i>, Param<'i>)> {
        let x = self.eat_param()?;
        self.eat_token(Token::Comma)?;
        let y = self.eat_param()?;
        Ok((x, y))
    }

    /// Consumes the next three parameters.
    fn eat_params3(&mut self) -> Result<(Param<'i>, Param<'i>, Param<'i>)> {
        let x = self.eat_param()?;
        self.eat_token(Token::Comma)?;
        let y = self.eat_param()?;
        self.eat_token(Token::Comma)?;
        let z = self.eat_param()?;
        Ok((x, y, z))
    }

    /// Consumes multiple data params.
    fn eat_data_params(&mut self) -> Result<Vec<Data<'i>>> {
        let mut data = Vec::new();
        data.push(self.eat_raw_param()?.1);
        loop {
            match self.peek()? {
                Some((_, Token::Comma)) => {
                    self.eat_token(Token::Comma)?;
                    data.push(self.eat_raw_param()?.1)
                }
                _ => break Ok(data),
            }
        }
    }

    /// Consumes the next instruction.
    fn eat_instr(&mut self) -> Result<Instr<'i>> {
        let (span, _) = self.eat_token(Token::Mnemonic)?;
        let opcode = span.as_str(self.input);
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
            _ => return Err(Error::new("unknown operation mnemonic", span)),
        };
        Ok(instr)
    }

    /// Consumes the next statement.
    fn eat_stmt(&mut self) -> Result<Stmt<'i>> {
        let label = match self.peek()? {
            Some((span, Token::Ident)) => {
                self.eat_token(Token::Ident)?;
                self.eat_token(Token::Colon)?;
                let label = span.as_str(self.input);
                if !self.labels.insert(label) {
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
            stmts.push(self.eat_stmt()?);
            // Either a newline or EOF is okay here, we don't want trailing
            // newlines to be required.
            match self.peek_interesting()? {
                None => break,
                Some(_) => {
                    self.eat_token(Token::Newline)?;
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
            ("ADD #-a", (6, 7), "expected a number, found an identifier"),
            ("ADD MUL", (4, 7), "expected a parameter, found a mnemonic"),
            ("YUP", (0, 3), "unknown operation mnemonic"),
            ("label: DB 0\nlabel: DB 0\n", (12, 17), "label already used"),
        ];
        for (asm, (m, n), msg) in tests {
            assert_eq!(program(asm).unwrap_err(), Error::new(*msg, *m..*n));
        }
    }
}
