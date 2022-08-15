#![feature(mixed_integer_ops)]

#[cfg(test)]
mod test;

use colored::*;
use std::{
    cmp::PartialEq,
    fmt,
    io::{stdin, stdout, Read, Write},
    ops::{Add, Mul, Sub},
};

use Value::*;

// instructions
const EXIT: isize = 0;
const CHICKEN: isize = 1;
const ADD: isize = 2;
const SUBTRACT: isize = 3;
const MULTIPLY: isize = 4;
const COMPARE: isize = 5;
const LOAD: isize = 6;
const STORE: isize = 7;
const JUMP: isize = 8;
const CHAR: isize = 9;

/// a value on the stack
#[derive(Debug, Clone)]
pub enum Value {
    /// a signed number
    Num(isize),

    /// a string
    String(std::string::String),

    /// a pointer to some area of the stack
    Ptr(usize),

    /// truthy value
    True,

    /// falsy value
    False,

    /// undefined
    Undefined,

    /// not a number
    NaN,
}

impl Value {
    /// tries to convert this Value into a [number](Value::Num) or [NaN](Value::NaN) if we can't
    pub fn to_num(&self) -> Self {
        match self {
            Num(n) => Num(*n),
            String(s) => match s.parse::<isize>() {
                Ok(n) => Num(n),
                Err(_) => NaN,
            },
            True => Num(1),
            False => Num(0),
            _ => NaN,
        }
    }

    /// the same as to_num but returns an Option instead of [Value::Num] or [Value::NaN]
    pub fn to_num_option(&self) -> Option<isize> {
        match self.to_num() {
            Num(n) => Some(n),
            _ => None,
        }
    }

    /// gets whether this Value is truthy or not
    pub fn is_truthy(&self) -> bool {
        match self {
            Ptr(_) => true,
            Num(n) => *n > 0,
            String(s) => !s.is_empty(),
            True => true,
            False => false,
            Undefined => false,
            NaN => false,
        }
    }
}

impl From<isize> for Value {
    fn from(n: isize) -> Self {
        Num(n)
    }
}

impl TryFrom<usize> for Value {
    type Error = std::num::TryFromIntError;

    fn try_from(n: usize) -> Result<Self, Self::Error> {
        Ok(Num(n.try_into()?))
    }
}

impl From<std::string::String> for Value {
    fn from(s: std::string::String) -> Self {
        String(s)
    }
}

impl From<&str> for Value {
    fn from(s: &str) -> Self {
        String(s.to_string())
    }
}

impl From<bool> for Value {
    fn from(b: bool) -> Self {
        match b {
            true => True,
            false => False,
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Num(n) => write!(f, "{}", n),
            String(s) => write!(f, "{}", s),
            True => write!(f, "true"),
            False => write!(f, "false"),
            Undefined => write!(f, "undefined"),
            NaN => write!(f, "NaN"),
            _ => write!(f, "{:?}", self),
        }
    }
}

impl Add for Value {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        // handle string conversion/concatenation if applicable
        if let String(a) = self {
            String(format!("{}{}", a, other))
        } else if let String(b) = other {
            String(format!("{}{}", self, b))
        } else {
            // no strings, just add
            match self.to_num() {
                Num(a) => match other.to_num() {
                    Num(b) => Num(a + b),
                    _ => NaN,
                },
                _ => NaN,
            }
        }
    }
}

impl Sub for Value {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        match self.to_num() {
            Num(a) => match other.to_num() {
                Num(b) => Num(a - b),
                _ => NaN,
            },
            _ => NaN,
        }
    }
}

impl Mul for Value {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        match self.to_num() {
            Num(a) => match other.to_num() {
                Num(b) => Num(a * b),
                _ => NaN,
            },
            _ => NaN,
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Num(a) => match other {
                Num(b) => a == b,
                String(b) => &a.to_string() == b,
                True => *a == 1,
                False => *a == 0,
                _ => false,
            },
            String(a) => match other {
                Num(b) => a == &b.to_string(),
                String(b) => a == b,
                True => a == "1",
                False => a == "0",
                _ => false,
            },
            Ptr(a) => match other {
                Ptr(b) => a == b,
                _ => false,
            },
            True => match other {
                Num(b) => 1 == *b,
                String(b) => "1" == b,
                True => true,
                _ => false,
            },
            False => match other {
                Num(b) => 0 == *b,
                String(b) => "0" == b,
                False => true,
                _ => false,
            },
            Undefined => matches!(other, Undefined),
            NaN => matches!(other, NaN),
        }
    }
}

/// an error that can be thrown by the chicken interpreter
#[derive(Debug, PartialEq)]
pub struct ChickenError {
    /// the error message
    pub message: std::string::String,

    /// the value of the program counter when the error was thrown
    pub program_counter: usize,

    /// a copy of the stack for debugging purposes
    pub stack: Vec<Value>,
}

impl fmt::Display for ChickenError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "{}{}", "error: ".red().bold(), self.message.bold())?;
        writeln!(
            f,
            "    program counter: {} ({:?})",
            self.program_counter, self.stack[self.program_counter]
        )?;
        writeln!(f, "    stack dump: {:?}", self.stack)
    }
}

/// allows for easy construction of a Chicken VM
pub struct VMBuilder {
    opcodes: Vec<isize>,
    input: Value,
    debug: bool,
    normal_char: bool,
}

impl VMBuilder {
    /// creates a new VMBuilder from a Chicken program
    ///
    /// # Example
    ///
    /// ```rust
    /// use chicken::VMBuilder;
    ///
    /// // starts building a VM with the Chicken quine
    /// let mut builder = VMBuilder::from_chicken("chicken");
    ///
    /// assert_eq!(builder.build().run(), Ok("chicken".to_string()))
    /// ```
    pub fn from_chicken<T: AsRef<str>>(chicken: T) -> Self {
        Self::from_opcodes(
            chicken
                .as_ref()
                .split('\n')
                .map(|l| l.matches("chicken").count() as isize)
                .collect::<Vec<_>>(),
        )
    }

    /// creates a new VMBuilder from the individual opcodes of a Chicken program
    ///
    /// # Example
    ///
    /// ```rust
    /// use chicken::VMBuilder;
    ///
    /// // starts building a VM with the Chicken "cat program", in raw opcode form
    /// let mut builder = VMBuilder::from_opcodes([11, 6, 0]);
    ///
    /// assert_eq!(
    ///     builder.input("Chicken Power").build().run(),
    ///     Ok("Chicken Power".to_string())
    /// )
    /// ```
    pub fn from_opcodes<T: Into<Vec<isize>>>(opcodes: T) -> Self {
        Self {
            opcodes: opcodes.into(),
            input: Undefined,
            debug: false,
            normal_char: false,
        }
    }

    /// sets the debug flag, causing the resulting VM to single step through the program and provide debug information
    pub fn debug(mut self) -> Self {
        self.debug = true;
        self
    }

    /// sets the value of the debug flag in the resulting VM
    pub fn set_debug(mut self, debug: bool) -> Self {
        self.debug = debug;
        self
    }

    /// sets the normal_char flag, causing the resulting VM to convert characters to their proper ASCII representations instead of to HTML entities
    pub fn normal_char(mut self) -> Self {
        self.normal_char = true;
        self
    }

    /// sets the value of the normal_char flag in the resulting VM
    pub fn set_normal_char(mut self, normal_char: bool) -> Self {
        self.normal_char = normal_char;
        self
    }

    /// passes the provided input to the VM
    pub fn input<T: Into<Value>>(mut self, input: T) -> Self {
        self.input = input.into();
        self
    }

    /// consumes this VMBuilder and builds a VMState, which can then be run with [VMState::run] or stepped through with [VMState::step]
    pub fn build(self) -> VMState {
        let mut stack: Vec<Value> = vec![
            // reference to the stack
            Ptr(0),
            // the input from the user, usually a string
            self.input,
        ];

        // push the program onto the stack
        stack.append(&mut self.opcodes.iter().map(|c| Num(*c)).collect());

        // push the axe opcode to the stack right after the program, to ensure that we'll exit cleanly unless shenanigans occur
        stack.push(Num(0));

        // return our new VM state
        VMState {
            stack,
            program_counter: 2, // start the program counter at the start of the program
            debug: self.debug,
            normal_char: self.normal_char,
            exited: false,
        }
    }
}

/// the state of the Chicken VM
pub struct VMState {
    /// the stack of the VM
    pub stack: Vec<Value>,

    /// the program counter, or instruction pointer of the VM
    pub program_counter: usize,

    /// whether to run the debugger or not
    pub debug: bool,

    /// whether the Char instruction should produce an actual character instead of an HTML entity string
    pub normal_char: bool,

    /// whether this VM has finished execution
    pub exited: bool,
}

impl VMState {
    /// runs the VM until it finishes execution, then returns the top value on the stack if it's a string, or an error if it's not.
    /// any error that occurs during execution will also be returned, along with hopefully useful debug information
    pub fn run(&mut self) -> Result<std::string::String, ChickenError> {
        if self.debug {
            // print some debug info
            println!("no opcode");
            println!("program counter {:?}", self.program_counter);
            println!("stack {:?}", self.stack);
            println!("press enter to step, ctrl+c to exit");

            // wait for enter to be pressed
            stdout().flush().unwrap();
            stdin().read_exact(&mut [0]).unwrap();
        }

        while !self.exited {
            self.step()?;
        }

        // return the top value of the stack if it's a string
        // also converts all HTML entities back to their normal character representations
        match self.stack.pop() {
            Some(String(s)) => Ok(html_escape::decode_html_entities(&s).to_string()),

            s => Err(ChickenError {
                message: format!("invalid value {:?} on exit", s),
                program_counter: self.program_counter,
                stack: self.stack.to_vec(),
            })?,
        }
    }

    /// single steps the VM, running one instruction at a time
    pub fn step(&mut self) -> Result<(), ChickenError> {
        if self.exited {
            return Ok(());
        }

        let op = self.stack.get(self.program_counter);

        if self.debug {
            // print some debug information
            println!("program counter {:?}", self.program_counter);
            print!("opcode {:?}", op);
            println!(
                " ({})",
                match &op {
                    Some(Num(EXIT)) => "axe/exit".to_string(),
                    Some(Num(CHICKEN)) => "chicken".to_string(),
                    Some(Num(ADD)) => "add".to_string(),
                    Some(Num(SUBTRACT)) => "fox/subtract".to_string(),
                    Some(Num(MULTIPLY)) => "rooster/multiply".to_string(),
                    Some(Num(COMPARE)) => "compare".to_string(),
                    Some(Num(LOAD)) => format!(
                        "pick/load from {:?}",
                        self.stack
                            .get(self.program_counter + 1)
                            .unwrap_or(&Undefined)
                    ),
                    Some(Num(STORE)) => "peck/store".to_string(),
                    Some(Num(JUMP)) => "fr/jump".to_string(),
                    Some(Num(CHAR)) => "bbq/chr".to_string(),
                    Some(Num(n)) => format!("literal {}", n),
                    _ => "unknown".to_string(),
                }
            );
        }

        self.program_counter += 1;

        match &op {
            // terminates the program
            Some(Num(EXIT)) => self.exited = true,

            // pushes the string "chicken" onto the stack
            Some(Num(CHICKEN)) => self.stack.push(String("chicken".to_string())),

            // pops the two values off the stack, adds them together, then pushes the result back on the stack
            // all math operations have the 2nd value from the top as the right hand value, and the top value as the left hand value
            // if one of the values is a string, the two values are concatenated like in javascript and any numbers are converted to decimal strings
            // if both of the values are numbers, they will be added like normal
            Some(Num(ADD)) => {
                let b = self.stack.pop().unwrap_or(Undefined);
                let a = self.stack.pop().unwrap_or(Undefined);
                self.stack.push(a + b)
            }

            // subtracts the two values at the top of the stack
            // if either or both of the values are strings, they will be converted to numbers then subtracted
            Some(Num(SUBTRACT)) => {
                let b = self.stack.pop().unwrap_or(Undefined);
                let a = self.stack.pop().unwrap_or(Undefined);
                self.stack.push(a - b)
            }

            // multiplies the two values at the top of the stack
            // if either or both of the values are strings, they will be converted to numbers then multiplied
            Some(Num(MULTIPLY)) => {
                let b = self.stack.pop().unwrap_or(Undefined);
                let a = self.stack.pop().unwrap_or(Undefined);
                self.stack.push(a * b)
            }

            // pops the two stack values, compares them for equality, then pushes the result as a truthy or falsy value
            Some(Num(COMPARE)) => {
                let b = self.stack.pop() == self.stack.pop();
                self.stack.push(b.into())
            }

            // double wide instruction. next opcode indicates the address on the stack to load from
            // the top value on the stack is popped and used as an index into that address
            // the address of 0 is a pointer to the entire stack, and as such indexing into it will index into the stack
            // any other address will index into the stack at that address, and if there's a string there you can access the individual characters in it
            // the behavior of indexing into numbers is not yet known
            Some(Num(LOAD)) => {
                let addr: usize = match self
                    .stack
                    .get(self.program_counter)
                    .unwrap_or(&Undefined)
                    .to_num_option()
                    .and_then(|n| n.try_into().ok())
                {
                    Some(n) => n,
                    None => {
                        self.program_counter += 1;
                        self.stack.push(Undefined);
                        return Ok(());
                    }
                };
                self.program_counter += 1;

                let index: usize = match self
                    .stack
                    .pop()
                    .unwrap_or(Undefined)
                    .to_num_option()
                    .and_then(|n| n.try_into().ok())
                {
                    Some(n) => n,
                    None => {
                        self.stack.push(Undefined);
                        return Ok(());
                    }
                };

                match self.stack.get(addr) {
                    Some(String(s)) => match s.chars().nth(index) {
                        Some(c) => self.stack.push(String(c.to_string())),
                        None => self.stack.push(Undefined),
                    },
                    Some(Ptr(p)) => match self.stack.get(p + index) {
                        Some(v) => self.stack.push(v.clone()),
                        None => self.stack.push(Undefined),
                    },
                    _ => self.stack.push(Undefined),
                }
            }

            // top of the stack contains the address on the stack to store to. the second topmost value on the stack gets stored at that address
            // both values are popped off the stack
            Some(Num(STORE)) => {
                let val = self.stack.pop();
                match val.as_ref().and_then(|v| v.to_num_option()) {
                    Some(n) => {
                        // TODO: add error checking here
                        self.stack[n as usize] = self
                            .stack
                            .pop()
                            .ok_or_else(|| ChickenError {
                                message: "no more items in stack".to_string(),
                                program_counter: self.program_counter,
                                stack: self.stack.to_vec(),
                            })?
                    }
                    None => Err(ChickenError {
                        message: format!("invalid address {:?}", val),
                        program_counter: self.program_counter,
                        stack: self.stack.to_vec(),
                    })?,
                }
            },

            // top of the stack is a relative offset to jump to. the value below that is the condition. jumps only occur if the condition is truthy
            Some(Num(JUMP)) => {
                let val = self.stack.pop();
                match val.as_ref().and_then(|v| v.to_num_option()) {
                    Some(rel) => {
                        if self.stack.pop().map(|v| v.is_truthy()).unwrap_or(false) {
                            self.program_counter = self
                                .program_counter
                                .checked_add_signed(rel)
                                .ok_or_else(|| ChickenError {
                                    message: format!("jump to relative addr {:?} overflowed", val),
                                    program_counter: self.program_counter,
                                    stack: self.stack.to_vec(),
                                })?;
                        }
                    }
                    None => Err(ChickenError {
                        message: format!("invalid relative address {:?}", val),
                        program_counter: self.program_counter,
                        stack: self.stack.to_vec(),
                    })?,
                }
            },

            // interprets the value at the top of the stack as ASCII and either pushes its corresponding HTML entity or character
            Some(Num(CHAR)) => {
                if self.normal_char {
                    let val = self.stack.pop();
                    match val
                        .as_ref()
                        .and_then(|v| v.to_num_option())
                        .and_then(|n| n.try_into().ok())
                        .and_then(char::from_u32)
                    {
                        Some(c) => self.stack.push(String(c.to_string())),
                        None => Err(ChickenError {
                            message: format!("{:?} not a number", val),
                            program_counter: self.program_counter,
                            stack: self.stack.to_vec(),
                        })?,
                    }
                } else {
                    let s = self.stack.pop().unwrap_or(Undefined).to_string();
                    self.stack.push(String(format!("&#{};", s)))
                }
            }

            // pushes n - 10 to the stack
            Some(Num(n)) => self.stack.push(Num(n - 10)),

            s => Err(ChickenError {
                message: format!("invalid opcode {:?}", s),
                program_counter: self.program_counter,
                stack: self.stack.to_vec(),
            })?,
        }

        if self.debug {
            // print some more debug info
            println!("program counter now {:?}", self.program_counter);
            println!("stack now {:?}", self.stack);

            // wait for enter to be pressed, effectively single stepping
            stdout().flush().unwrap();
            stdin().read_exact(&mut [0]).unwrap();
        }

        Ok(())
    }
}
