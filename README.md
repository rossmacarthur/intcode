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
| IN       | Reads a single number                        |
| OUT      | Writes a single number                       |
| JNZ      | Jumps if the operand is non-zero             |
| JZ       | Jumps if the operand is zero                 |
| LT       | Checks if one operand is less than another   |
| EQ       | Checks if two operands are equal             |
| ARB      | Adjusts the relative base to the given value |
| HLT      | Ends the program                             |

There are three ways to specify the operands for different opcodes.

| Operand type | Examples                                                                                                |
| ------------ | ------------------------------------------------------------------------------------------------------- |
| Positional   | `19` specifies the value at address 19. `x+3` specifies the value at the label "x" with an offset of 3. |
| Immediate    | `#19` specifies the exact value 19. `#x+3` specifies the exact "x" label address plus 3.                |
| Relative     | `~19` specifies the value at the relative base address with an offset of 19.                            |

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
