use intcode_assemble::Intcode;
use intcode_disassemble::{Input, Run};

use pretty_assertions::assert_eq;

fn run_once() -> impl IntoIterator<Item = Run> {
    Run::once(Input::Forever(0))
}

fn disassemble(intcode: &str, runs: impl IntoIterator<Item = Run>) -> String {
    let intcode: Vec<_> = intcode
        .split(',')
        .map(str::parse)
        .map(Result::unwrap)
        .collect();
    intcode_disassemble::to_ast(intcode, runs)
        .unwrap()
        .to_string()
}

fn assemble(asm: &str) -> String {
    let Intcode { output, .. } = intcode_assemble::to_intcode(asm).unwrap();
    output
        .into_iter()
        .map(|d| d.to_string())
        .collect::<Vec<_>>()
        .join(",")
}

#[track_caller]
fn assert(asm: &str, intcode: &str, runs: impl IntoIterator<Item = Run>) {
    assert_eq!(disassemble(intcode, runs), asm);
    assert_eq!(assemble(asm), intcode);
}

#[test]
fn hello_world() {
    let asm = r#"ARB #10
a: OUT rb
ARB #1
JNZ rb, #a
HLT
DB "Hello World!\n"
"#;
    let intcode = "109,10,204,0,109,1,1205,0,2,99,72,101,108,108,111,32,87,111,114,108,100,33,10";
    assert(asm, intcode, run_once());
}
