# Intcode

Compiler and runner for the Intcode computer from Advent of Code 2019.

## Assembly

### Instruction set

The compiler can assemble the following instruction set specification into an
Intcode program. The following operations are supported, roughly described in
the order they are introduced in Advent of Code.

| Mnemonic | Brief description                            |
| -------- | -------------------------------------------- |
| ADD      | Adds two operands together                   |
| MUL      | Multiplies two operands together             |
| IN       | Read a single number                         |
| OUT      | Write a single number                        |
| JNZ      | Jump if the operand is non-zero              |
| JZ       | Jump if the operand is zero                  |
| LT       | Check if one operand is less than another    |
| EQ       | Check if two operands are equal              |
| ARB      | Adjusts the relative base to the given value |
| HLT      | End the program                              |

There are three ways to specify the operands for different opcodes.

| Operand type | Example                                |
| ------------ | -------------------------------------- |
| Identifier   | `x` specifies the address of label `x` |
| Exact        | `19` specifies the value at address 19 |

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
