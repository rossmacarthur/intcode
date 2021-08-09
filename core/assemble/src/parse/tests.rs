use super::*;

use pretty_assertions::assert_eq;

#[test]
fn eat_raw_param_strings() {
    let tests = [
        ("\"Hello World!\"", "Hello World!", 0..14, true),
        ("\"Hello World!\\n\"", "Hello World!\n", 0..16, false),
        ("\"😎\"", "😎", 0..6, true),
        ("\"😎\\t\"", "😎\t", 0..8, false),
    ];
    for (asm, string, range, is_borrowed) in tests {
        let (_, span, raw) = Parser::new(asm).eat_raw_param().unwrap();
        assert_eq!(span, range.into());
        match raw {
            RawParam::String(value) => {
                assert_eq!(value, string);
                assert_eq!(value.is_borrowed(), is_borrowed);
            }
            _ => unreachable!(),
        }
    }
}

#[test]
fn eat_program_basic() {
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
                        Param::Label(Mode::Positional, "a", 0),
                        Param::Label(Mode::Positional, "b", 0),
                        Param::Number(Mode::Positional, 3)
                    )
                },
                Stmt {
                    label: None,
                    instr: Instr::Multiply(
                        Param::Number(Mode::Positional, 3),
                        Param::Label(Mode::Positional, "c", 0),
                        Param::Number(Mode::Positional, 0)
                    )
                },
                Stmt {
                    label: None,
                    instr: Instr::Halt,
                },
                Stmt {
                    label: Some("a"),
                    instr: Instr::Data(vec![RawParam::Number(30)]),
                },
                Stmt {
                    label: Some("b"),
                    instr: Instr::Data(vec![RawParam::Number(40)]),
                },
                Stmt {
                    label: Some("c"),
                    instr: Instr::Data(vec![RawParam::Number(50)]),
                },
            ]
        }
    );
}

#[test]
fn eat_program_errors() {
    let tests = [
        ("ADD @", 4..5, "unexpected character"),
        ("label x", 6..7, "expected a colon, found an identifier"),
        ("ADD \"\"", 4..6, "string parameter only allowed with `DB`"),
        (
            "ADD #rb+1",
            4..9,
            "both immediate and relative mode specified",
        ),
        ("DB #1", 3..4, "immediate mode not allowed with `DB`"),
        ("DB #rb+1", 3..4, "immediate mode not allowed with `DB`"),
        ("DB rb+1", 3..5, "relative mode not allowed with `DB`"),
        ("ADD x,", 6..6, "expected a parameter, found end of input"),
        ("ADD", 0..3, "expected 3 parameters, found 0"),
        ("ADD x, y", 0..3, "expected 3 parameters, found 2"),
        ("ADD x, y, z, w", 0..3, "expected 3 parameters, found 4"),
        ("ADD #-a", 6..7, "expected a number, found an identifier"),
        ("ADD MUL", 4..7, "expected a parameter, found a mnemonic"),
        ("YUP", 0..3, "unknown operation mnemonic"),
        ("rb: DB 0", 0..2, "label is reserved for the relative base"),
        ("label: DB 0\nlabel: DB 0", 12..17, "label already used"),
    ];
    for (asm, span, msg) in tests {
        assert_eq!(program(asm).unwrap_err(), [Error::new(msg, span)]);
    }
}
