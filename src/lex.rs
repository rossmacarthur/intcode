//! Tokenize the input.

use std::ops;
use std::str;

use crate::error::{Error, Result};

/// Represents a location in the original input.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    /// The start index.
    pub m: usize,
    /// The end index.
    pub n: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// `:`
    Colon,
    /// `,`
    Comma,
    /// `-`
    Minus,
    /// An LF line ending (0x0A).
    Newline,
    /// A sequence of tab (0x09) and/or spaces (0x20).
    Whitespace,
    /// A decimal number like `19` or `-7`.
    Number,
    /// A variable or label identifier, like `start` or `rb`.
    Ident,
    /// An instruction mnemonic, like `EQ` or `HLT`.
    Mnemonic,
    /// Comment contents including the `;` prefix.
    Comment,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Token {
    /// The kind of token.
    pub kind: Kind,
    /// The location in the original string.
    pub span: Span,
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
    /// The input as (index, char) values.
    iter: CharIndices<'i>,
}

////////////////////////////////////////////////////////////////////////////////
// Implementations
////////////////////////////////////////////////////////////////////////////////

impl Span {
    pub fn width(&self) -> usize {
        self.n - self.m
    }

    pub fn range(&self) -> ops::Range<usize> {
        self.m..self.n
    }
}

impl From<ops::Range<usize>> for Span {
    fn from(r: ops::Range<usize>) -> Self {
        Self {
            m: r.start,
            n: r.end,
        }
    }
}

impl Kind {
    pub fn human(&self) -> &'static str {
        match *self {
            Self::Colon => "a colon",
            Self::Comma => "a comma",
            Self::Minus => "a minus",
            Self::Newline => "a newline",
            Self::Whitespace => "whitespace",
            Self::Number => "a number",
            Self::Ident => "an identifier",
            Self::Mnemonic => "a mnemonic",
            Self::Comment => "a comment",
        }
    }
}

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

fn is_identifier(c: &char) -> bool {
    matches!(c, 'a'..='z' | '_')
}

impl<'i> Tokens<'i> {
    /// Construct a new iterator over the input tokens.
    pub fn new(input: &'i str) -> Self {
        let iter = CharIndices::new(input);
        Self { iter }
    }

    /// Eats the next character if the predicate is satisfied.
    fn lex_if<P>(&mut self, predicate: P) -> bool
    where
        P: Fn(&char) -> bool,
    {
        match self.iter.peek_char() {
            Some(c) if predicate(&c) => self.iter.next().is_some(),
            _ => false,
        }
    }

    /// Eats the next token, including all characters satisfying the predicate.
    fn lex_token<P>(&mut self, kind: Kind, i: usize, predicate: P) -> Token
    where
        P: Fn(&char) -> bool + Copy,
    {
        while self.lex_if(predicate) {}
        Token::new(kind, i, self.iter.peek_index())
    }

    /// Returns the next token in the iterator.
    pub fn next(&mut self) -> Result<Option<Token>> {
        let token = match self.iter.next() {
            Some((i, ';')) => Some(self.lex_token(Kind::Comment, i, |&c| c != '\n')),
            Some((i, ':')) => Some(Token::new(Kind::Colon, i, i + 1)),
            Some((i, ',')) => Some(Token::new(Kind::Comma, i, i + 1)),
            Some((i, '-')) => Some(Token::new(Kind::Minus, i, i + 1)),
            Some((i, '\n')) => Some(Token::new(Kind::Newline, i, i + 1)),
            Some((i, c)) if c.is_ascii_whitespace() => {
                Some(self.lex_token(Kind::Whitespace, i, char::is_ascii_whitespace))
            }
            Some((i, c)) if c.is_ascii_digit() => {
                Some(self.lex_token(Kind::Number, i, char::is_ascii_digit))
            }
            Some((i, c)) if is_identifier(&c) => {
                Some(self.lex_token(Kind::Ident, i, is_identifier))
            }
            Some((i, c)) if c.is_ascii_uppercase() => {
                Some(self.lex_token(Kind::Mnemonic, i, char::is_ascii_uppercase))
            }

            Some((i, _)) => return Err(Error::new(i..(i + 1), "unexpected character")),
            None => None,
        };
        Ok(token)
    }

    /// Finds the next token matching the predicate.
    pub fn find<P>(&mut self, mut predicate: P) -> Result<Option<Token>>
    where
        P: FnMut(&Kind) -> bool,
    {
        loop {
            match self.next()? {
                Some(token) if predicate(&token.kind) => break Ok(Some(token)),
                None => break Ok(None),
                Some(_) => continue,
            }
        }
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
                Token::new(Kind::Mnemonic, 7, 10),
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
        assert_eq!(
            Tokens::new("@").next().unwrap_err(),
            Error::new(0..1, "unexpected character")
        );
    }
}
