// eswm -- Emacs Standalown WindowManager
// Copyright (C) 2022 Jacob Stannix

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

pub mod value {
    use std::ops::{Add, Div, Mul, Sub};
    #[derive(PartialEq)]
    pub enum ValueType {
        Bool,
        Nil,
        Number,
    }

    #[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
    /// eswm's internal value representation.
    pub enum Value {
        Bool(bool),
        Number(f64),
        None,
    }

    impl Value {
        pub fn is_type(&self, val_type: ValueType) -> bool {
            match *self {
                Self::Bool(_) => ValueType::Bool == val_type,
                Self::Number(_) => ValueType::Number == val_type,
                Self::None => ValueType::Nil == val_type,
            }
        }

        pub fn nil() -> Value {
            Value::None
        }

        pub fn as_bool(&self) -> bool {
            match *self {
                Self::Bool(val) => val,
                _ => unreachable!(),
            }
        }

        pub fn as_number(&self) -> f64 {
            match *self {
                Self::Number(val) => val,
                _ => unreachable!(),
            }
        }
    }

    impl From<f64> for Value {
        fn from(value: f64) -> Value {
            Value::Number(value)
        }
    }

    impl From<bool> for Value {
        fn from(value: bool) -> Value {
            Value::Bool(value)
        }
    }

    impl Add for Value {
        type Output = Self;

        fn add(self, other: Self) -> Self {
            match self {
                Self::Number(val) => match other {
                    Self::Number(val2) => (val + val2).into(),
                    _ => unreachable!(),
                },
                _ => unreachable!(),
            }
        }
    }

    impl Sub for Value {
        type Output = Self;
        fn sub(self, other: Self) -> Self {
            match self {
                Self::Number(val) => match other {
                    Self::Number(val2) => (val - val2).into(),
                    _ => unreachable!(),
                },
                _ => unreachable!(),
            }
        }
    }

    impl Div for Value {
        type Output = Self;
        fn div(self, other: Self) -> Self {
            match self {
                Self::Number(val) => match other {
                    Self::Number(val2) => (val / val2).into(),
                    _ => unreachable!(),
                },
                _ => unreachable!(),
            }
        }
    }

    impl Mul for Value {
        type Output = Self;
        fn mul(self, other: Self) -> Self {
            match self {
                Self::Number(val) => match other {
                    Self::Number(val2) => (val * val2).into(),
                    _ => unreachable!(),
                },
                _ => unreachable!(),
            }
        }
    }

    /// Prints [`Value`] to stdout.
    pub fn print_value(value: Value) {
        match value {
            Value::None => print!("nil"),
            Value::Bool(_) => print!("{}", value.as_bool()),
            Value::Number(_) => print!("{}", value.as_number()),
        }
    }
}

pub mod chunk {
    use super::value::Value;
    use std::fmt::{self, Display};
    /// Code representing and instruction to execute.
    #[derive(Clone, Copy)]
    pub enum OpCode {
        Return,
        Constant,

        Negate,
        Add,
        Subtract,
        Multiply,
        Divide,
        Nil,
        True,
        False,
        Not,
	Equal,
	Greater,
	Less,
    }

    impl From<u8> for OpCode {
        fn from(value: u8) -> OpCode {
            match value {
                0 => OpCode::Return,
                1 => OpCode::Constant,
                2 => OpCode::Negate,
                3 => OpCode::Add,
                4 => OpCode::Subtract,
                5 => OpCode::Multiply,
                6 => OpCode::Divide,
                7 => OpCode::Nil,
                8 => OpCode::True,
                9 => OpCode::False,
                10 => OpCode::Not,
		11 => OpCode::Equal,
		12 => OpCode::Greater,
		13 => OpCode::Less,
                _ => unreachable!(),
            }
        }
    }

    impl Display for OpCode {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match *self {
                OpCode::Return => write!(f, "OP_RETURN"),
                OpCode::Constant => write!(f, "OP_CONSTANT"),
                OpCode::Negate => write!(f, "OP_NEGATE"),
                OpCode::Add => write!(f, "OP_ADD"),
                OpCode::Subtract => write!(f, "OP_SUBTRACT"),
                OpCode::Multiply => write!(f, "OP_MULTIPLY"),
                OpCode::Divide => write!(f, "OP_DIVIDE"),
                OpCode::Nil => write!(f, "OP_NIL"),
                OpCode::True => write!(f, "OP_TRUE"),
                OpCode::False => write!(f, "OP_FALSE"),
                OpCode::Not => write!(f, "OP_NOT"),
		OpCode::Equal => write!(f, "OP_EQUAL"),
		OpCode::Greater => write!(f, "OP_GREATER"),
		OpCode::Less => write!(f, "OP_LESS"),
            }
        }
    }

    /// A chunk houses the instructions for the vm to execute
    pub struct Chunk {
        pub code: Vec<u8>,
        pub constants: Vec<Value>,
        pub lines: Vec<usize>,
    }

    impl Chunk {
        pub fn new() -> Self {
            Self {
                code: Vec::new(),
                constants: Vec::new(),
                lines: Vec::new(),
            }
        }
        /// Writes to [`chunk.code`] and [`chunk.lines`]
        pub fn write(&mut self, code: u8, line: usize) {
            self.code.push(code);
            self.lines.push(line);
        }
        /// Writes to [`chunk.constans`] and returns its position in the array.
        pub fn constant(&mut self, value: Value) -> u8 {
            self.constants.push(value);
            (self.constants.len() - 1) as u8
        }
    }
}

pub mod debug {

    use super::chunk::{Chunk, OpCode};
    /// Outputs `code` to stdout.
    /// Returns offset + 1.
    fn simple_instruction(code: OpCode, offset: usize) -> usize {
        println!("{}", code);
        offset + 1
    }

    /// Outputs `code` to stdout along with its assosiated value.
    /// Returns offset + 2.
    fn constant_instruction(code: OpCode, chunk: &Chunk, offset: usize) -> usize {
        let constant = chunk.code[offset + 1] as usize;
        print!("{:-16} {:4} '", code, constant);
        super::value::print_value(chunk.constants[constant]);
        println!("'");
        offset + 2
    }

    /// Prints code at chunk offset.
    pub fn disassemble_instruction(chunk: &Chunk, offset: usize) -> usize {
        print!("{:04} ", offset);

        if offset > 0 && chunk.lines[offset] == chunk.lines[offset - 1] {
            print!("   | ");
        } else {
            print!("{:4} ", chunk.lines[offset]);
        }

        let instruction = OpCode::from(chunk.code[offset]);
        match instruction {
            OpCode::Return
            | OpCode::Negate
            | OpCode::Add
            | OpCode::Subtract
            | OpCode::Multiply
            | OpCode::Divide
            | OpCode::Nil
            | OpCode::True
            | OpCode::False
            | OpCode::Not | OpCode::Equal | OpCode::Greater | OpCode::Less => simple_instruction(instruction, offset),
            OpCode::Constant => constant_instruction(instruction, chunk, offset),
        }
    }

    /// Prints the contents of [`Chunk`] to std out.
    pub fn disassemble_chunk(chunk: &Chunk, name: &str) {
        println!("== {} ==", name);
        let mut offset = 0;
        while offset < chunk.code.len() {
            offset = disassemble_instruction(chunk, offset);
        }
    }
}
