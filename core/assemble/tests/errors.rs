use intcode_assemble::to_ast;

use pretty_assertions::assert_eq;

#[track_caller]
fn assemble(asm: &str) -> String {
    yansi::Paint::disable();
    to_ast(asm)
        .unwrap_err()
        .into_iter()
        .map(|e| e.pretty(asm, "<input>"))
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn lex_unexpected_char() {
    let asm = "ADD @";
    let expected = "
  --> <input>:1:5
   |
 1 | ADD @
   |     ^ unexpected character
";
    assert_eq!(assemble(asm), expected);
}

#[test]
fn lex_undelimited_string() {
    let asm = r#"ADD "test"#;
    let expected = r#"
  --> <input>:1:5
   |
 1 | ADD "test
   |     ^^^^^ undelimited string
"#;
    assert_eq!(assemble(asm), expected);
}

#[test]
fn lex_undelimited_string_escape() {
    let asm = r#"ADD "te\"st"#;
    let expected = r#"
  --> <input>:1:5
   |
 1 | ADD "te\"st
   |     ^^^^^^^ undelimited string
"#;
    assert_eq!(assemble(asm), expected);
}

#[test]
fn lex_undelimited_string_newline() {
    let asm = "ADD \"test\n";
    let expected = r#"
  --> <input>:1:5
   |
 1 | ADD "test
   |     ^^^^^ undelimited string
"#;
    assert_eq!(assemble(asm), expected);
}

#[test]
fn parse_unexpected_token() {
    let asm = "label+";
    let expected = "
  --> <input>:1:6
   |
 1 | label+
   |      ^ expected a colon, found a plus
";
    assert_eq!(assemble(asm), expected);
}

#[test]
fn parse_invalid_base_2_integer() {
    let asm = "DB 0b021";
    let expected = "
  --> <input>:1:7
   |
 1 | DB 0b021
   |       ^ invalid digit for base 2 literal
";
    assert_eq!(assemble(asm), expected);
}

#[test]
fn parse_invalid_base_10_integer() {
    let asm = "DB 0a21";
    let expected = "
  --> <input>:1:5
   |
 1 | DB 0a21
   |     ^ invalid digit for base 10 literal
";
    assert_eq!(assemble(asm), expected);
}

#[test]
fn parse_invalid_base_16_integer() {
    let asm = "DB 0x2Ga1";
    let expected = "
  --> <input>:1:7
   |
 1 | DB 0x2Ga1
   |       ^ invalid digit for base 16 literal
";
    assert_eq!(assemble(asm), expected);
}

#[test]
fn parse_invalid_integer_overflow() {
    let asm = "DB 0xFFFFFFFFFFFFFFFF";
    let expected = "
  --> <input>:1:4
   |
 1 | DB 0xFFFFFFFFFFFFFFFF
   |    ^^^^^^^^^^^^^^^^^^ base 16 literal out of range for 64-bit integer
";
    assert_eq!(assemble(asm), expected);
}

#[test]
fn parse_invalid_string_escape() {
    let asm = r#"ADD "te\st""#;
    let expected = r#"
  --> <input>:1:9
   |
 1 | ADD "te\st"
   |         ^ unknown escape character
"#;
    assert_eq!(assemble(asm), expected);
}

#[test]
fn parse_expected_parameter() {
    let asm = "ADD +";
    let expected = "
  --> <input>:1:5
   |
 1 | ADD +
   |     ^ expected a parameter, found a plus
";
    assert_eq!(assemble(asm), expected);
}

#[test]
fn parse_expected_parameters() {
    let asm = "ADD ; comment";
    let expected = "
  --> <input>:1:1
   |
 1 | ADD ; comment
   | ^^^ expected 3 parameters, found 0
";
    assert_eq!(assemble(asm), expected);
}

#[test]
fn parse_invalid_string_parameter() {
    let asm = r#"ADD "test""#;
    let expected = r#"
  --> <input>:1:5
   |
 1 | ADD "test"
   |     ^^^^^^ string parameter only allowed with `DB`
"#;
    assert_eq!(assemble(asm), expected);
}

#[test]
fn parse_invalid_parameter_mode() {
    let asm = "ADD #rb+1";
    let expected = "
  --> <input>:1:5
   |
 1 | ADD #rb+1
   |     ^^^^^ both immediate and relative mode specified
";
    assert_eq!(assemble(asm), expected);
}

#[test]
fn parse_invalid_parameter_count() {
    let asm = "ADD x, y";
    let expected = "
  --> <input>:1:1
   |
 1 | ADD x, y
   | ^^^ expected 3 parameters, found 2
";
    assert_eq!(assemble(asm), expected);
}

#[test]
fn parse_invalid_immediate_mode() {
    let asm = "DB #0";
    let expected = "
  --> <input>:1:4
   |
 1 | DB #0
   |    ^ immediate mode not allowed with `DB`
";
    assert_eq!(assemble(asm), expected);
}

#[test]
fn parse_invalid_relative_mode() {
    let asm = "DB rb+1";
    let expected = "
  --> <input>:1:4
   |
 1 | DB rb+1
   |    ^^ relative mode not allowed with `DB`
";
    assert_eq!(assemble(asm), expected);
}

#[test]
fn parse_invalid_immediate_and_relative_mode() {
    let asm = "DB #rb+1";
    let expected = "
  --> <input>:1:4
   |
 1 | DB #rb+1
   |    ^ immediate mode not allowed with `DB`
";
    assert_eq!(assemble(asm), expected);
}

#[test]
fn parse_invalid_opcode() {
    let asm = "TEST x, y, z";
    let expected = "
  --> <input>:1:1
   |
 1 | TEST x, y, z
   | ^^^^ unknown operation mnemonic
";
    assert_eq!(assemble(asm), expected);
}

#[test]
fn parse_reserved_label_underscore() {
    let asm = "_: ADD x, y, z";
    let expected = "
  --> <input>:1:1
   |
 1 | _: ADD x, y, z
   | ^ label is reserved to indicate a runtime value
";
    assert_eq!(assemble(asm), expected);
}

#[test]
fn parse_reserved_label_ip() {
    let asm = "ip: ADD x, y, z";
    let expected = "
  --> <input>:1:1
   |
 1 | ip: ADD x, y, z
   | ^^ label is reserved to refer to the instruction pointer
";
    assert_eq!(assemble(asm), expected);
}

#[test]
fn parse_reserved_label_rb() {
    let asm = "rb: ADD x, y, z";
    let expected = "
  --> <input>:1:1
   |
 1 | rb: ADD x, y, z
   | ^^ label is reserved to refer to the relative base
";
    assert_eq!(assemble(asm), expected);
}

#[test]
fn parse_duplicate_label() {
    let asm = "\
test: ADD x, y, z
test: HLT
";
    let expected = "
  --> <input>:2:1
   |
 2 | test: HLT
   | ^^^^ label already used
";
    assert_eq!(assemble(asm), expected);
}

#[test]
fn parse_invalid_second_label() {
    let asm = "\
test:
another: HLT
";
    let expected = "
  --> <input>:2:1
   |
 2 | another: HLT
   | ^^^^^^^ expected a mnemonic, found an identifier
";
    assert_eq!(assemble(asm), expected);
}

#[test]
fn parse_multi_statement_error() {
    let asm = "\
ADD @
HLT #0
";
    let expected = "
  --> <input>:1:5
   |
 1 | ADD @
   |     ^ unexpected character


  --> <input>:2:1
   |
 2 | HLT #0
   | ^^^ expected 0 parameters, found 1
";
    assert_eq!(assemble(asm), expected);
}
