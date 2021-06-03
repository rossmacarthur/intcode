//! Tokenize the input.

use std::ops;
use std::str;

use anyhow::{bail, Result};

/// Represents a location in the original input.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    /// The start index.
    m: usize,
    /// The end index.
    n: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// `:`
    Colon,
    /// `,`
    Comma,
    /// An LF line ending (0x0A).
    Newline,
    /// A sequence of tab (0x09) and/or spaces (0x20).
    Whitespace,
    /// A decimal number like `19` or `-7`.
    Number,
    /// A variable or label identifier, like `start` or `rb`.
    Ident,
    /// An instruction mnemonic, like `EQ` or `HLT`.
    Opcode,
    /// Comment contents including the `;` prefix.
    Comment,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Token {
    /// The kind of token.
    kind: Kind,
    /// The location in the original string.
    span: Span,
}

/// An iterator over (index, char) in a string.
#[derive(Debug, Clone)]
struct CharIndices<'i> {
    iter: str::CharIndices<'i>,
    len: usize,
}

/// An iterator over input tokens.
#[derive(Debug, Clone)]
pub struct Tokens<'i> {
    /// The original input string.
    input: &'i str,
    /// The input as (index, char) values.
    iter: CharIndices<'i>,
}

////////////////////////////////////////////////////////////////////////////////
// Implementations
////////////////////////////////////////////////////////////////////////////////

impl Token {
    /// Construct a new token with the given kind and span.
    fn new(kind: Kind, m: usize, n: usize) -> Self {
        Self {
            kind,
            span: Span { m, n },
        }
    }
}

impl<'i> CharIndices<'i> {
    /// Construct a new iterator over indexes and characters of a string.
    fn new(input: &'i str) -> Self {
        Self {
            iter: input.char_indices(),
            len: input.len(),
        }
    }

    /// Returns the next index of the iterator.
    fn peek_index(&self) -> usize {
        self.iter
            .clone()
            .next()
            .map(|(i, _)| i)
            .unwrap_or_else(|| self.len)
    }

    /// Returns the next character of the iterator.
    fn peek_char(&self) -> Option<char> {
        self.iter.clone().next().map(|(_, c)| c)
    }
}

impl<'i> ops::Deref for CharIndices<'i> {
    type Target = str::CharIndices<'i>;

    fn deref(&self) -> &Self::Target {
        &self.iter
    }
}

impl<'i> ops::DerefMut for CharIndices<'i> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.iter
    }
}

impl<'i> Tokens<'i> {
    /// Construct a new iterator over the input tokens.
    pub fn new(input: &'i str) -> Self {
        let iter = CharIndices::new(input);
        Self { input, iter }
    }

    /// Eats the next character if the predicate is satisfied.
    fn eat_if<P>(&mut self, predicate: P) -> bool
    where
        P: Fn(&char) -> bool,
    {
        match self.iter.peek_char() {
            Some(c) if predicate(&c) => self.iter.next().is_some(),
            _ => false,
        }
    }

    /// Eats the next token, including all characters satisfying the predicate.
    fn eat_token<P>(&mut self, kind: Kind, i: usize, predicate: P) -> Token
    where
        P: Fn(&char) -> bool + Copy,
    {
        while self.eat_if(predicate) {}
        Token::new(kind, i, self.iter.peek_index())
    }

    /// Returns the next token in the iterator.
    pub fn next(&mut self) -> Result<Option<Token>> {
        let token = match self.iter.next() {
            Some((i, ';')) => Some(self.eat_token(Kind::Comment, i, |&c| c != '\n')),
            Some((i, ':')) => Some(Token::new(Kind::Colon, i, i + 1)),
            Some((i, ',')) => Some(Token::new(Kind::Comma, i, i + 1)),
            Some((i, '\n')) => Some(Token::new(Kind::Newline, i, i + 1)),
            Some((i, c)) if c.is_ascii_whitespace() => {
                Some(self.eat_token(Kind::Whitespace, i, char::is_ascii_whitespace))
            }
            Some((i, c)) if matches!(c, '0'..='9' | '-') => {
                Some(self.eat_token(Kind::Number, i, char::is_ascii_digit))
            }
            Some((i, c)) if c.is_ascii_lowercase() => {
                Some(self.eat_token(Kind::Ident, i, char::is_ascii_lowercase))
            }
            Some((i, c)) if c.is_ascii_uppercase() => {
                Some(self.eat_token(Kind::Opcode, i, char::is_ascii_uppercase))
            }

            Some((i, c)) => bail!("unexpected character `{}` at index {}", c, i),
            None => None,
        };
        Ok(token)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use pretty_assertions::assert_eq;

    impl<'i> Iterator for Tokens<'i> {
        type Item = Result<Token>;

        fn next(&mut self) -> Option<Self::Item> {
            self.next().transpose()
        }
    }

    #[track_caller]
    fn tokenize(input: &str) -> Vec<Token> {
        Tokens::new(input).collect::<Result<_>>().unwrap()
    }

    #[test]
    fn basic() {
        let tokens = tokenize("start:\nADD tmp, 19, rb \t ; this is a comment\n");
        assert_eq!(
            tokens,
            [
                Token::new(Kind::Ident, 0, 5),
                Token::new(Kind::Colon, 5, 6),
                Token::new(Kind::Newline, 6, 7),
                Token::new(Kind::Opcode, 7, 10),
                Token::new(Kind::Whitespace, 10, 11),
                Token::new(Kind::Ident, 11, 14),
                Token::new(Kind::Comma, 14, 15),
                Token::new(Kind::Whitespace, 15, 16),
                Token::new(Kind::Number, 16, 18),
                Token::new(Kind::Comma, 18, 19),
                Token::new(Kind::Whitespace, 19, 20),
                Token::new(Kind::Ident, 20, 22),
                Token::new(Kind::Whitespace, 22, 25),
                Token::new(Kind::Comment, 25, 44),
                Token::new(Kind::Newline, 44, 45),
            ]
        );
    }

    #[test]
    fn error() {
        let err = Tokens::new("@").next().unwrap_err();
        assert_eq!(err.to_string(), "unexpected character `@` at index 0");
    }
}
