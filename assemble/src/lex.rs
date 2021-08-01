//! Tokenize the input.

use std::ops;
use std::str;

use crate::error::{Error, Result};
use crate::span::Span;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Token {
    /// `:`
    Colon,
    /// `,`
    Comma,
    /// `#`
    Hash,
    /// `+`
    Plus,
    /// `-`
    Minus,
    /// `~`
    Tilde,
    /// An LF line ending (0x0A).
    Newline,
    /// A sequence of tab (0x09) and/or spaces (0x20).
    Whitespace,
    /// A decimal number like `19`.
    Number,
    /// A variable or label identifier, like `start` or `rb`.
    Ident,
    /// An instruction mnemonic, like `EQ` or `HLT`.
    Mnemonic,
    /// String contents include the `"` prefix and suffix.
    String,
    /// Comment contents including the `;` prefix.
    Comment,
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

fn span(token: Token, span: impl Into<Span>) -> (Span, Token) {
    (span.into(), token)
}

impl Token {
    pub fn human(&self) -> &'static str {
        match *self {
            Self::Colon => "a colon",
            Self::Comma => "a comma",
            Self::Hash => "a hash",
            Self::Plus => "a plus",
            Self::Minus => "a minus",
            Self::Tilde => "a tilde",
            Self::Newline => "a newline",
            Self::Whitespace => "whitespace",
            Self::Number => "a number",
            Self::Ident => "an identifier",
            Self::Mnemonic => "a mnemonic",
            Self::String => "a string",
            Self::Comment => "a comment",
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
    fn lex_token<P>(&mut self, token: Token, i: usize, predicate: P) -> (Span, Token)
    where
        P: Fn(&char) -> bool + Copy,
    {
        while self.lex_if(predicate) {}
        span(token, i..self.iter.peek_index())
    }

    /// Eats the next string.
    fn lex_string(&mut self, i: usize) -> Result<(Span, Token)> {
        while self.lex_if(|c| !matches!(c, '"' | '\r' | '\n')) {}
        match self.lex_if(|&c| c == '"') {
            true => Ok(span(Token::String, i..self.iter.peek_index())),
            false => Err(Error::new("undelimited string", i..self.iter.peek_index())),
        }
    }

    /// Returns the next token in the iterator.
    pub fn next(&mut self) -> Result<Option<(Span, Token)>> {
        let token = match self.iter.next() {
            Some((i, '"')) => Some(self.lex_string(i)?),
            Some((i, ';')) => Some(self.lex_token(Token::Comment, i, |&c| c != '\n')),
            Some((i, ':')) => Some(span(Token::Colon, i)),
            Some((i, ',')) => Some(span(Token::Comma, i)),
            Some((i, '#')) => Some(span(Token::Hash, i)),
            Some((i, '+')) => Some(span(Token::Plus, i)),
            Some((i, '-')) => Some(span(Token::Minus, i)),
            Some((i, '~')) => Some(span(Token::Tilde, i)),
            Some((i, '\n')) => Some(span(Token::Newline, i)),
            Some((i, c)) if c.is_ascii_whitespace() => {
                Some(self.lex_token(Token::Whitespace, i, char::is_ascii_whitespace))
            }
            Some((i, c)) if c.is_ascii_digit() => {
                Some(self.lex_token(Token::Number, i, char::is_ascii_digit))
            }
            Some((i, c)) if is_identifier(&c) => {
                Some(self.lex_token(Token::Ident, i, is_identifier))
            }
            Some((i, c)) if c.is_ascii_uppercase() => {
                Some(self.lex_token(Token::Mnemonic, i, char::is_ascii_uppercase))
            }

            Some((i, _)) => return Err(Error::new("unexpected character", i..i + 1)),
            None => None,
        };
        Ok(token)
    }

    /// Finds the next token matching the predicate.
    pub fn find<P>(&mut self, mut predicate: P) -> Result<Option<(Span, Token)>>
    where
        P: FnMut(&Token) -> bool,
    {
        loop {
            match self.next()? {
                Some((span, token)) if predicate(&token) => break Ok(Some((span, token))),
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
        type Item = Result<(Span, Token)>;

        fn next(&mut self) -> Option<Self::Item> {
            self.next().transpose()
        }
    }

    #[track_caller]
    fn tokenize(input: &str) -> Vec<(Span, Token)> {
        Tokens::new(input).collect::<Result<_>>().unwrap()
    }

    #[test]
    fn basic() {
        let tokens = tokenize("start:\nADD tmp, #19, ~a+1 \t ; this is a comment\n");
        assert_eq!(
            tokens,
            [
                span(Token::Ident, 0..5),
                span(Token::Colon, 5..6),
                span(Token::Newline, 6..7),
                span(Token::Mnemonic, 7..10),
                span(Token::Whitespace, 10..11),
                span(Token::Ident, 11..14),
                span(Token::Comma, 14..15),
                span(Token::Whitespace, 15..16),
                span(Token::Hash, 16..17),
                span(Token::Number, 17..19),
                span(Token::Comma, 19..20),
                span(Token::Whitespace, 20..21),
                span(Token::Tilde, 21..22),
                span(Token::Ident, 22..23),
                span(Token::Plus, 23..24),
                span(Token::Number, 24..25),
                span(Token::Whitespace, 25..28),
                span(Token::Comment, 28..47),
                span(Token::Newline, 47..48),
            ]
        );
    }

    #[test]
    fn basic_string() {
        let tokens = tokenize("\"Hello World!\"");
        assert_eq!(tokens, [span(Token::String, 0..14)])
    }

    #[test]
    fn error() {
        assert_eq!(
            Tokens::new("@").next().unwrap_err(),
            Error::new("unexpected character", 0..1)
        );
    }
}
