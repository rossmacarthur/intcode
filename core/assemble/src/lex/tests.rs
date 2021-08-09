use super::*;

use pretty_assertions::assert_eq;

impl<'i> Iterator for Tokens<'i> {
    type Item = Result<(Span, Token)>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.next() {
            Ok((_, Token::Eof)) => None,
            ts => Some(ts),
        }
    }
}

#[track_caller]
fn tokenize(input: &str) -> Result<Vec<(Span, Token)>> {
    Tokens::new(input).collect()
}

#[test]
fn basic() {
    let tokens = tokenize("start:\nADD tmp, #19, rb+1 \t ; this is a comment\n").unwrap();
    assert_eq!(
        tokens,
        [
            span(Token::Ident, 0..5),
            span(Token::Colon, 5..6),
            span(Token::Newline, 6..7),
            span(Token::Ident, 7..10),
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
fn numbers() {
    let tests = [
        "0b10011", "0o23", "19", "0x13", "0b1_0011", "0o_2_3_", "1_9_", "0x_13_",
    ];
    for input in tests {
        let tokens = tokenize(input).unwrap();
        assert!(matches!(&*tokens, &[(_, Token::Number)]));
    }
}

#[test]
fn strings() {
    let tests = [
        ("\"Hello World!\"", 0..14),
        ("\"Hello World!\\n\"", 0..16),
        ("\"😎\"", 0..6),
        ("\"😎\\t\"", 0..8),
    ];
    for (input, range) in tests {
        let tokens = tokenize(input).unwrap();
        assert_eq!(tokens, [span(Token::String, range)]);
    }
}

#[test]
fn error() {
    assert_eq!(
        tokenize("@").unwrap_err(),
        Error::new("unexpected character", 0..1)
    );
}
