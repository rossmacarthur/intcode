use intcode_assemble::program as assemble;

use pretty_assertions::assert_eq;

#[test]
fn advent_day2_example1() {
    let asm = "\
ADD a, b, 3
MUL 3, c, 0
HLT
a: DB 30
b: DB 40
c: DB 50
";
    let expected = "1,9,10,3,2,3,11,0,99,30,40,50";
    assert_eq!(assemble(asm).unwrap(), expected);
}

#[test]
fn advent_day5_example_immediate() {
    let asm = "\
MUL a, #3, 4
a: DB 33
";
    let expected = "1002,4,3,4,33";
    assert_eq!(assemble(asm).unwrap(), expected);
}

#[test]
fn advent_day5_example_cmp_positional() {
    let asm = "\
IN a
EQ a, b, a
OUT a
HLT
a: DB -1
b: DB 8
";
    let expected = "3,9,8,9,10,9,4,9,99,-1,8";
    assert_eq!(assemble(asm).unwrap(), expected);
}

#[test]
fn advent_day5_example_cmp_immediate() {
    let asm = "\
IN 3
EQ #-1, #8, 3
OUT 3
HLT
";
    let expected = "3,3,1108,-1,8,3,4,3,99";
    assert_eq!(assemble(asm).unwrap(), expected);
}

#[test]
fn advent_day5_example_jump_positional() {
    let asm = "\
IN a
JZ a, d
ADD b, c, b
o: OUT b
HLT
a: DB -1
b: DB 0
c: DB 1
d: DB o
";
    let expected = "3,12,6,12,15,1,13,14,13,4,13,99,-1,0,1,9";
    assert_eq!(assemble(asm).unwrap(), expected);
}

#[test]
fn advent_day5_example_jump_immediate() {
    let asm = "\
IN  j+1
j: JNZ #-1, #o
ADD #0, #0, a
o: OUT a
HLT
a: DB  1
";
    let expected = "3,3,1105,-1,9,1101,0,0,12,4,12,99,1";
    assert_eq!(assemble(asm).unwrap(), expected);
}

#[test]
fn advent_day9_example_quine() {
    let asm = "\
ARB #1
OUT rb-1
ADD 100, #1, 100
EQ  100, #16, 101
JZ  101, #0
HLT
";
    let expected = "109,1,204,-1,1001,100,1,100,1008,100,16,101,1006,101,0,99";
    assert_eq!(assemble(asm).unwrap(), expected);
}

#[test]
fn advent_day9_example_16_digit_number() {
    let asm = "\
MUL #34915192, #34915192, x
OUT x
HLT
";
    let expected = "1102,34915192,34915192,7,4,7,99,0";
    assert_eq!(assemble(asm).unwrap(), expected);
}