# Intcode

Assembler and runner for the Intcode computer from Advent of Code 2019.

## Hello World! program

The following program outputs "Hello World!".

```asm
    ARB #message  ; move the relative base to the beginning of our message

loop:
    OUT rb        ; output the current character in the message
    ARB #1        ; move the relative base to the next character
    JNZ rb, #loop ; if the next character is non-zero then go back to `loop`
    HLT

message:
    DB "Hello World!\n"
```

## Assembly language

The compiler can assemble the following instruction set specification into an
Intcode program.

### General

Intcode assembly must be written in a UTF-8 encoded file with Unix line endings.
Comments start with a semicolon `;`.

### Operand types

There are two types of operands.

- **Label**

  A label refers to an address in a program. For example: `end` in the following program refers the address of the `HLT` instruction.
  ```asm
      JZ #0, #end
  end:
      HLT
  ```

  If the label is not defined by the programmer in a program then the assembler
  will allocate memory at the end for it. For example: the following program
  reads in a value to the address `x`, increments it, and outputs the
  incremented value.
  ```asm
  IN  x
  ADD x, #1, x
  OUT x
  HLT
  ```

- **Number**

  A binary, octal, decimal, or hexadecimal number. This can be used for
  specifying manual addresses, address offsets, or exact values. For example:
  the following reads in a value, minuses 3 from it, and outputs the result.
  ```asm
  IN  x
  ADD x, #-0b11, x+1
  OUT x+0x1
  HLT
  ```

### Operand modes

There are three ways to specify the operands for different instructions.

- **Positional**

  Specifies a value by specifying the *address* it should be read from. For
  example:
  - `19` specifies the value at address 19.
  - `x+3` specifies the value at the label `x` with an offset of 3.

- **Immediate**

  Specifies a value by specifying the exact value. For example:
  - `#19` specifies the exact value 19.
  - `#x+3` specifies the exact label address `x` with an offset of 3.

- **Relative**

  Specifies a value by specifying the *address* it should be read from as an
  offset of the *relative base*. For example:
  - `rb+3` specifies the value at the relative base address with an offset of 3.

### Opcodes

The following operations are supported, roughly described in the order they are
introduced in Advent of Code.

- **`ADD`**

  Adds the first two operands and stores the result in the third. For example:
  increment the value at `x`:
  ```asm
  ADD x, #1, x
  ```

- **`MUL`**

  Multiplies the first two operands and stores the result in the third. For
  example: multiply the value at `x` by 2:
  ```asm
  MUL x, #2, x
  ```

- **`IN`**

  Reads a single number and stores it in the first operand. For example: store
  input at `x`:
  ```asm
  IN x
  ```

- **`OUT`**

  Outputs a single number and stores it in the first operand. For example:
  output the value at `x`:
  ```asm
  OUT x
  ```

- **`JNZ`**

  Checks if the first operand is non-zero and then jumps to the value of the
  second operand. For example: set the instruction pointer to `label` if the
  value at `x` is non-zero:
  ```asm
  JNZ x, #label
  ```

- **`JZ`**

  Checks if the first operand is zero and then jumps to the value of the second
  operand. For example: set the instruction pointer to `label` if the value at
  `x` is zero:
  ```asm
  JZ x, #label
  ```

- **`LT`**

  Checks if the first operand is less than the second. If true, stores 1 in the
  third operand else stores 0. For example: check if the value at `x` is less
  than 7 and store the result in `result`:
  ```asm
  LT x, #7, result
  ```

- **`EQ`**

  Checks if the first operand is equal to the second. If true, stores 1 in the
  third operand else stores 0. For example: check if the value at `x` is equal
  to 7 and store the result in `result`:
  ```asm
  EQ x, #7, result
  ```

- **`ARB`**

  Adjusts the relative base to the value of the first operand. For example: sets
  the relative base to the `message` address:
  ```asm
  ARB #message
  ```

- **`HLT`**

  Halts the program. For example:
  ```asm
  HLT
  ```

### Pseudo-opcodes

- **`DB`**

  Places raw data into the program. This must be a sequence of numbers or
  strings. Strings will be encoded as UTF-8. A label on a `DB` instruction will
  refer to the start of the data. For example the following specifies the string
  "Hello World!" with a newline.
  ```asm
  message:
      DB "Hello World!", 10
  ```

### Special labels

- **`_`**

  Refers to an undefined address. This should be used to indicate that the
  address will be set at runtime.

- **`ip`**

  Refers to the address of the next instruction. This can be used to dereference
  a pointer.

Consider the following example where `ptr` refers to some address and we want to
read a value into that address. `_` is used because the value of `ptr` will be
filled as the parameter for the `IN` instruction by the `ADD` instruction.

```asm
ADD ptr, #0, ip+1
IN  _
HLT
```

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
