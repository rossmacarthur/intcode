use std::convert::TryInto;

use intcode_assemble::Intcode;
use intcode_run::{Computer, State};

#[track_caller]
fn assemble(asm: &str) -> String {
    let Intcode { output, .. } = intcode_assemble::to_intcode(asm).unwrap();
    output
        .into_iter()
        .map(|d| d.to_string())
        .collect::<Vec<_>>()
        .join(",")
}

#[track_caller]
fn run_utf8(asm: &str) -> String {
    let Intcode { output, warnings } = intcode_assemble::to_intcode(asm).unwrap();
    assert_eq!(warnings.len(), 0);
    let mut c = Computer::new(output);
    let mut w = Vec::new();
    loop {
        match c.next().unwrap() {
            State::Yielded(value) => {
                w.push(value.try_into().unwrap());
            }
            State::Waiting => {
                unreachable!();
            }
            State::Complete => {
                break String::from_utf8(w).unwrap();
            }
        }
    }
}

#[test]
fn hello_world() {
    let asm = r#"
    ARB #message

loop:
    OUT rb
    ARB #1
    JNZ rb, #loop
    HLT

message:
    DB "Hello World!\n"
"#;
    assert_eq!(run_utf8(asm), "Hello World!\n");
}

#[test]
fn db_with_ip() {
    let asm = r#"DB _, ip+1, "abc""#;
    assert_eq!(assemble(asm), "0,6,97,98,99");
}
