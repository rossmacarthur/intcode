//! Tokenize the input.

use std::ops;
use std::str;

use crate::error::{Error, Result};
use crate::span::{s, S};

/// The type of token yielded by the lexer.
#[derive(Debug, Clone, PartialEq, Eq)]
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
    /// An LF line ending (0x0A).
    Newline,
    /// A sequence of tab (0x09) and/or spaces (0x20).
    Whitespace,
    /// A mnemonic, variable or label identifier, like `HLT`, `the_end` or `rb`.
    Ident,
    /// A decimal number like `19`, `0b1011, or `0o777`, or `0x7f`.
    Number,
    /// A string like `"Hello World!\n"`.
    String,
    /// Comment contents including the `;` prefix.
    Comment,
    /// The end of input.
    Eof,
}

/// An iterator over (index, char) in a string.
#[derive(Debug, Clone)]
pub struct CharIndices<'i> {
    iter: str::CharIndices<'i>,
    len: usize,
}

/// An iterator over input tokens.
#[derive(Debug, Clone)]
pub struct Tokens<'i> {
    input: &'i str,
    iter: CharIndices<'i>,
}

////////////////////////////////////////////////////////////////////////////////
// Implementations
////////////////////////////////////////////////////////////////////////////////

impl Token {
    pub fn human(&self) -> &'static str {
        match *self {
            Self::Colon => "a colon",
            Self::Comma => "a comma",
            Self::Hash => "a hash",
            Self::Plus => "a plus",
            Self::Minus => "a minus",
            Self::Newline => "a newline",
            Self::Whitespace => "whitespace",
            Self::Ident => "an identifier",
            Self::Number => "a number",
            Self::String => "a string",
            Self::Comment => "a comment",
            Self::Eof => "end of input",
        }
    }
}

impl<'i> CharIndices<'i> {
    fn new(input: &'i str) -> Self {
        Self {
            iter: input.char_indices(),
            len: input.len(),
        }
    }

    fn peek_index(&self) -> usize {
        self.iter
            .clone()
            .next()
            .map(|(i, _)| i)
            .unwrap_or_else(|| self.len)
    }

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

fn is_identifier_start(c: &char) -> bool {
    matches!(c, 'a'..='z' | 'A'..='Z' | '_')
}

fn is_identifier(c: &char) -> bool {
    matches!(*c, '0'..='9' | 'A'..='Z' | 'a'..='z' | '_')
}

impl<'i> Tokens<'i> {
    /// Constructs a new iterator over the input tokens.
    pub fn new(input: &'i str) -> Self {
        let iter = CharIndices::new(input);
        Self { input, iter }
    }

    /// Lexes the next character if the predicate is satisfied.
    fn lex_if<P>(&mut self, predicate: P) -> bool
    where
        P: Fn(&char) -> bool,
    {
        match self.iter.peek_char() {
            Some(c) if predicate(&c) => self.iter.next().is_some(),
            _ => false,
        }
    }

    /// Lexes the next token, including all characters satisfying the predicate.
    fn lex_token<P>(&mut self, tk: Token, i: usize, predicate: P) -> S<Token>
    where
        P: Fn(&char) -> bool + Copy,
    {
        while self.lex_if(predicate) {}
        s(tk, i..self.iter.peek_index())
    }

    /// Lexes the next string.
    fn lex_string(&mut self, i: usize) -> Result<S<Token>> {
        let mut curr = '"';
        loop {
            match self.iter.next() {
                None => {
                    break Err(Error::new("undelimited string", i..self.iter.peek_index()));
                }
                Some((j, '\n')) => {
                    break Err(Error::new("undelimited string", i..j));
                }
                Some((_, '"')) if curr != '\\' => {
                    break Ok(s(Token::String, i..self.iter.peek_index()));
                }
                Some((_, c)) => {
                    curr = c;
                }
            }
        }
    }

    /// Returns the next token in the lexer.
    pub fn next(&mut self) -> Result<S<Token>> {
        let next = match self.iter.next() {
            None => {
                let i = self.iter.peek_index();
                return Ok(s(Token::Eof, i..i));
            }
            Some(next) => next,
        };
        let token = match next {
            // Single character to token mappings.
            (i, ':') => s(Token::Colon, i..i + 1),
            (i, ',') => s(Token::Comma, i..i + 1),
            (i, '#') => s(Token::Hash, i..i + 1),
            (i, '+') => s(Token::Plus, i..i + 1),
            (i, '-') => s(Token::Minus, i..i + 1),
            (i, '\n') => s(Token::Newline, i..i + 1),

            // Multi-character tokens with a distinct starting character.
            (i, ';') => self.lex_token(Token::Comment, i, |&c| c != '\n'),
            (i, '"') => self.lex_string(i)?,

            // Multi-character tokens that use many different characters.
            (i, c) if c.is_ascii_whitespace() => {
                self.lex_token(Token::Whitespace, i, char::is_ascii_whitespace)
            }
            (i, c) if c.is_ascii_digit() => self.lex_token(Token::Number, i, is_identifier),
            (i, c) if is_identifier_start(&c) => self.lex_token(Token::Ident, i, is_identifier),

            // Any other character is considered invalid.
            (i, _) => {
                return Err(Error::new(
                    "unexpected character",
                    i..self.iter.peek_index(),
                ))
            }
        };
        Ok(token)
    }

    /// Finds the next token matching the predicate.
    pub fn find<P>(&mut self, mut predicate: P) -> Result<S<Token>>
    where
        P: FnMut(&Token) -> bool,
    {
        loop {
            match self.next()? {
                S(Token::Eof, span) => break Ok(S(Token::Eof, span)),
                S(tk, span) if predicate(&tk) => break Ok(S(tk, span)),
                _ => continue,
            }
        }
    }
}
