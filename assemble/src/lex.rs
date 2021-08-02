//! Tokenize the input.

use std::ops;
use std::str;
use std::string::String as StdString;

use dairy::String;

use crate::error::{Error, Result};
use crate::span::Span;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token<'i> {
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
    /// A decimal number like `19`.
    Number,
    /// A variable or label identifier, like `start` or `rb`.
    Ident,
    /// An instruction mnemonic, like `EQ` or `HLT`.
    Mnemonic,
    /// String contents include the `"` prefix and suffix.
    String(String<'i>),
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
    /// The original input.
    input: &'i str,
    /// The input as (index, char) values.
    iter: CharIndices<'i>,
}

////////////////////////////////////////////////////////////////////////////////
// Implementations
////////////////////////////////////////////////////////////////////////////////

fn span(token: Token, span: impl Into<Span>) -> (Span, Token) {
    (span.into(), token)
}

impl Token<'_> {
    pub fn human(&self) -> &'static str {
        match *self {
            Self::Colon => "a colon",
            Self::Comma => "a comma",
            Self::Hash => "a hash",
            Self::Plus => "a plus",
            Self::Minus => "a minus",
            Self::Newline => "a newline",
            Self::Whitespace => "whitespace",
            Self::Number => "a number",
            Self::Ident => "an identifier",
            Self::Mnemonic => "a mnemonic",
            Self::String(..) => "a string",
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
        Self { iter, input }
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
    fn lex_token<P>(&mut self, token: Token<'i>, i: usize, predicate: P) -> (Span, Token<'i>)
    where
        P: Fn(&char) -> bool + Copy,
    {
        while self.lex_if(predicate) {}
        span(token, i..self.iter.peek_index())
    }

    /// Eats the next string.
    fn lex_string(&mut self, i: usize) -> Result<(Span, Token<'i>)> {
        let Self { input, iter } = self;

        let mut string: Option<StdString> = None;
        let m = i + 1;
        let mut next = || {
            iter.next()
                .ok_or_else(|| Error::new("undelimited string", m..iter.peek_index()))
        };
        loop {
            match next()? {
                (n, '"') => {
                    let tk = match string {
                        Some(o) => Token::String(String::owned(o)),
                        None => Token::String(String::borrowed(&input[m..n])),
                    };
                    break Ok(span(tk, i..iter.peek_index()));
                }
                (n, '\\') => {
                    let c = match next()?.1 {
                        'n' => '\n',
                        'r' => '\r',
                        't' => '\t',
                        '\\' => '\\',
                        '"' => '"',
                        _ => {
                            return Err(Error::new(
                                "unknown escape character",
                                n..iter.peek_index(),
                            ))
                        }
                    };
                    match string {
                        Some(ref mut o) => o.push(c),
                        None => {
                            let mut o = StdString::from(&input[m..n]);
                            o.push(c);
                            string = Some(o)
                        }
                    }
                }
                (_, c) => {
                    if let Some(o) = string.as_mut() {
                        o.push(c);
                    }
                }
            }
        }
    }

    /// Returns the next token in the iterator.
    pub fn next(&mut self) -> Result<Option<(Span, Token<'i>)>> {
        let token = match self.iter.next() {
            Some((i, '"')) => Some(self.lex_string(i)?),
            Some((i, ';')) => Some(self.lex_token(Token::Comment, i, |&c| c != '\n')),
            Some((i, ':')) => Some(span(Token::Colon, i)),
            Some((i, ',')) => Some(span(Token::Comma, i)),
            Some((i, '#')) => Some(span(Token::Hash, i)),
            Some((i, '+')) => Some(span(Token::Plus, i)),
            Some((i, '-')) => Some(span(Token::Minus, i)),
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
    pub fn find<P>(&mut self, mut predicate: P) -> Result<Option<(Span, Token<'i>)>>
    where
        P: FnMut(&Token<'i>) -> bool,
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
        type Item = Result<(Span, Token<'i>)>;

        fn next(&mut self) -> Option<Self::Item> {
            self.next().transpose()
        }
    }

    #[track_caller]
    fn tokenize<'i>(input: &'i str) -> Vec<(Span, Token<'i>)> {
        Tokens::new(input).collect::<Result<_>>().unwrap()
    }

    #[test]
    fn basic() {
        let tokens = tokenize("start:\nADD tmp, #19, rb+1 \t ; this is a comment\n");
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
                span(Token::Ident, 21..23),
                span(Token::Plus, 23..24),
                span(Token::Number, 24..25),
                span(Token::Whitespace, 25..28),
                span(Token::Comment, 28..47),
                span(Token::Newline, 47..48),
            ]
        );
    }

    #[test]
    fn strings() {
        let tests = [
            ("\"Hello World!\"", "Hello World!", 0..14, true),
            ("\"Hello World!\\n\"", "Hello World!\n", 0..16, false),
            ("\"😎\"", "😎", 0..6, true),
            ("\"😎\\t\"", "😎\t", 0..8, false),
        ];
        for (input, output, range, is_borrowed) in tests {
            let tokens = tokenize(input);
            match &*tokens {
                &[(_, Token::String(ref s))] => assert_eq!(s.is_borrowed(), is_borrowed),
                _ => unreachable!(),
            }
            assert_eq!(tokens, [span(Token::String(String::from(output)), range)]);
        }
    }

    #[test]
    fn error() {
        assert_eq!(
            Tokens::new("@").next().unwrap_err(),
            Error::new("unexpected character", 0..1)
        );
    }
}
