use intcode_assemble::Intcode;
use intcode_disassemble::Run;

use pretty_assertions::assert_eq;

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
fn advent_day2_example1() {
    let asm = "\
a: ADD b, c, a+3
MUL a+3, d, 0
HLT
b: DB 30
c: DB 40
d: DB 50
";
    let intcode = "1,9,10,3,2,3,11,0,99,30,40,50";
    assert(asm, intcode, Run::once());
}

#[test]
fn advent_day5_example_immediate() {
    let asm = "\
MUL a, #3, a
a: DB 33
";
    let intcode = "1002,4,3,4,33";
    assert(asm, intcode, Run::once());
}

#[test]
fn advent_day5_example_eq_positional() {
    let asm = "\
IN a
EQ a, b, a
OUT a
HLT
a: DB -1
b: DB 8
";
    let intcode = "3,9,8,9,10,9,4,9,99,-1,8";
    assert(asm, intcode, Run::once());
}

#[test]
fn advent_day5_example_lt_positional() {
    let asm = "\
IN a
LT a, b, a
OUT a
HLT
a: DB -1
b: DB 8
";
    let intcode = "3,9,7,9,10,9,4,9,99,-1,8";
    assert(asm, intcode, Run::once());
}

#[test]
fn advent_day5_example_eq_immediate() {
    let asm = "\
IN a+1
a: EQ #-1, #8, a+1
OUT a+1
HLT
";
    let intcode = "3,3,1108,-1,8,3,4,3,99";
    assert(asm, intcode, Run::once());
}

#[test]
fn advent_day5_example_lt_immediate() {
    let asm = "\
IN a+1
a: LT #-1, #8, a+1
OUT a+1
HLT
";
    let intcode = "3,3,1107,-1,8,3,4,3,99";
    assert(asm, intcode, Run::once());
}

#[test]
fn advent_day5_example_jump_positional() {
    let asm = "\
IN a
JZ a, d
ADD b, c, b
OUT b
HLT
a: DB -1
b: DB 0
c: DB 1
d: DB 9
";
    let intcode = "3,12,6,12,15,1,13,14,13,4,13,99,-1,0,1,9";
    assert(asm, intcode, Run::twice(0, 1));
}

#[test]
fn advent_day5_example_jump_immediate() {
    let asm = "\
IN ip+1
JNZ #-1, #a
ADD #0, #0, b
a: OUT b
HLT
b: DB 1
";
    let intcode = "3,3,1105,-1,9,1101,0,0,12,4,12,99,1";
    assert(asm, intcode, Run::once());
}

#[test]
fn advent_day9_example_quine() {
    let asm = "\
ARB #1
OUT rb-1
ADD 100, #1, 100
EQ 100, #16, 101
JZ 101, #0
HLT
";
    let intcode = "109,1,204,-1,1001,100,1,100,1008,100,16,101,1006,101,0,99";
    assert(asm, intcode, Run::once());
}

#[test]
fn advent_day9_example_16_digit_number() {
    let asm = "\
MUL #34915192, #34915192, a
OUT a
HLT
a: DB 0
";
    let intcode = "1102,34915192,34915192,7,4,7,99,0";
    assert(asm, intcode, Run::once());
}
