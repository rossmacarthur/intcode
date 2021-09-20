use intcode_assemble::Intcode;
use intcode_error::fmt;

#[track_caller]
fn assemble(asm: &str) -> String {
    let Intcode { warnings, .. } = intcode_assemble::to_intcode(asm).unwrap();
    let fmt = fmt::Plain::new(asm);
    warnings
        .iter()
        .map(|w| fmt.warning(w))
        .collect::<Vec<String>>()
        .join("\n")
}

#[test]
fn assemble_unused_label() {
    let asm = "x: HLT";
    let expected = "
  --> <input>:1:1
   |
 1 | x: HLT
   | ^ label is never used
";
    assert_eq!(assemble(asm), expected);
}
