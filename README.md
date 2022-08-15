# chicken-rs
chicken-rs (very creative name, I know) is a [Chicken](https://esolangs.org/wiki/Chicken) interpreter written in Rust that aims to be as accurate to the original JavaScript implementation as possible,
with all of its JavaScript-related jankiness intact. This also means it can run *all* the example Chicken programs, unlike many other interpreters.

## Compiling

chicken-rs can be compiled by simply cloning the repository and running `cargo build --release` in it.
If you don't have Rust and/or Cargo installed, there's a great guide to downloading it [on the Rust website](https://www.rust-lang.org/tools/install).

The compiled executable will be located in `target/release/`, and should either be named `chicken`, `chicken.exe`, etc. depending on your OS.

## Usage

* You provide the Chicken program to run with `--file /path/to/file.chicken`
    * There are copies of the Chicken example programs located in the `examples/` directory, for convenience
* You provide input to programs with `--input "input"`
* You can single step through programs and see a real time view of the stack with `--debug`
* You can disable the original behavior of the Char instruction (see below) with `--normal-char`

# the Chicken programming language reference

Chicken is a somewhat unusual language in that it only has one keyword, `chicken`.
How would you do anything with that? Good question!
An opcode in Chicken is defined by the amount of times `chicken` is repeated on each line of the program (and yes zero instances of `chicken` on a line counts).
Therefore, you can do things by changing the amount of times `chicken` is repeated from line to line.

## Instructions

Chicken supports 11 distinct instructions. All instructions that consume data will do so by popping it from the stack, and all instructions that produce data will push it to the stack.

| Opcode (`chicken`s per line) | Name     | Official Name | Description |
| ---------------------------- | -------- | ------------- | ----------- |
| `0`                          | Exit     | Axe           | Exits the program, outputting the top value on the stack if it's a string. |
| `1`                          | Chicken  | Chicken       | Pushes the string "chicken" onto the stack. |
| `2`                          | Add      | Add           | Adds the 2nd value from the top of the stack to the value at the top of the stack (i.e. `stack[stack_pointer] + stack[stack_pointer - 1]`), pushing the result. |
| `3`                          | Subtract | Fox           | Subtracts the 2nd value from the top of the stack from the value at the top of the stack, pushing the result. |
| `4`                          | Multiply | Rooster       | Multiplies the two topmost values on the stack and pushes the result. |
| `5`                          | Compare  | Compare       | Checks whether the two topmost values on the stack are loosely equal and pushes the resulting boolean. |
| `6`                          | Load     | Pick          | The opcode following this instruction is treated as an address on the stack to index into like an array, and the topmost value on the stack is popped and treated as index into said array. Since Chicken stores a reference to the entire stack at address `0`, indexing into this will allow for reading any value on the stack, and providing the address of a string in the stack allows reading individual characters from it. Indexing into anything else will result in `undefined` being pushed to the stack. The resulting value will then be pushed onto the stack. |
| `7`                          | Store    | Peck          | The top two values on the stack are popped. The topmost value is used as an index into the stack to write the second topmost value into. Note that unlike the Load instruction, this cannot index into anything and is limited to only writing to addresses in the stack. |
| `8`                          | Jump     | Fr            | Adds the value on the top of the stack to the instruction pointer/program counter if the value 2nd from the top of the stack is truthy. |
| `9`                          | Char     | BBQ           | Interprets the value at the top of the stack as ASCII, and pushes the corresponding character. |
| `10+`                        | Push     |               | Pushes the literal number `n - 10` (where `n` is the opcode or the number of `chicken`s on this line) onto the stack. |

## Memory Layout

Chicken uses a stack-based memory model with 3 distinct (but not separate!) regions, each one located directly after the last.

* The first region (at the very start of the stack) contains, in order, a reference to the entire stack, and the input from the user as a string. This is what allows for indexing into the stack and user input with the Load instruction.
* The second region contains the opcodes of the program in numerical form (i.e. the number of chickens per line), with an extra Exit opcode appended just in case.
* The third region is the normal part of the stack that's accessed in typical programs.

## Quirks

Chicken has many quirks (that don't seem to be implemented in Chicken interpreters other than the original), mainly because the original Chicken interpreter was written in JavaScript and without type and error checking.
Listed here are the ones I'm currently aware of:

* Original interpreter throws an error if anything other than a string is at the top of the stack upon exiting any program.
* Since the Add instruction uses the JavaScript `+` operator, if one or both of the operands is a string they will be converted to strings and concatenated together.
* If one or both of the operands to the Subtract or Multiply instruction are strings, they will be converted to numbers before the operation is performed. If they're not valid numbers, NaN will be returned.
* In math operations, `true` will automatically be converted to `1` and `false` will automatically be converted to `0`.
* The Compare instruction uses JavaScript's normal equality rules, so, for example, `"123" == 123`, `true == "1"` (and `"false" == "0"`), however `true != "true"`, `undefined != "undefined"`, etc.
* You can use Literally Anything as an address for the Load instruction. Negative numbers work, numbers above the stack pointer work, strings work, etc.
* When converting characters to ASCII, the original interpreter was lazy and instead pushed their corresponding HTML entities. For example, instead of interpreting `104` as the letter `h`, it would instead be interpreted as the HTML entity `&#104;`, which corresponds to the same character but also means that the character isn't actually pushed to the stack. This behavior is ridiculously stupid, however it is enabled by default because it's required for the example programs to work. It can be disabled with the `--normal-char` argument.
