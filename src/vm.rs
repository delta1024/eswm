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

use crate::compiler::compile;
use crate::lib::chunk::{Chunk, OpCode};
use crate::lib::debug::disassemble_instruction;
use crate::value::{print_value, Value, ValueType};
use std::result::Result;

const STACK_MAX: usize = 256;

enum BinaryOp {
    Add,
    Sub,
    Div,
    Mul,
    Less,
    Greater,
}

#[allow(dead_code)]
pub enum VmErr {
    CompileError,
    RuntimeError,
}

pub type InterpretResult<T> = Result<T, VmErr>;

pub struct Vm {
    pub chunk: Option<Chunk>,
    pub ip: *const u8,
    pub stack: Vec<Value>,
    pub stack_top: *mut Value,
}

fn generate_stack() -> Vec<Value> {
    let mut vector = Vec::new();
    vector.resize(STACK_MAX, Value::None);
    vector
}

fn is_falsy(value: Value) -> bool {
    value.is_type(ValueType::Nil) || (value.is_type(ValueType::Bool) && !value.as_bool())
}

impl Vm {
    pub fn new() -> Self {
        let stack = generate_stack();
        let mut vm = Vm {
            chunk: None,
            ip: &mut 0,
            stack,
            stack_top: &mut Value::None,
        };
        vm.reset_stack();
        vm
    }

    fn reset_stack(&mut self) {
        self.stack_top = &mut self.stack[0];
    }

    fn runtime_error(&mut self, message: &str) {
        eprintln!("{}", message);

        let instruction =
            unsafe { (self.ip.offset_from(&self.chunk.as_ref().unwrap().code[0]) - 1) as usize };
        let line = self.chunk.as_ref().unwrap().lines[instruction];
        eprintln!("[line {}] in script", line);
        self.reset_stack();
    }

    pub fn interpret(&mut self, source: &str) -> InterpretResult<()> {
        let chunk = compile(source)?;

        self.ip = &chunk.code[0];
        self.chunk = Some(chunk);
        self.run()
    }

    fn push<T: Into<Value>>(&mut self, value: T) {
        unsafe {
            *self.stack_top = value.into();
            self.stack_top = self.stack_top.add(1);
        }
    }

    fn pop(&mut self) -> Value {
        unsafe {
            self.stack_top = self.stack_top.sub(1);
            *self.stack_top
        }
    }

    fn peek(&self, distance: usize) -> Value {
        unsafe { *self.stack_top.sub(1).sub(distance) }
    }

    
    fn read_byte(&mut self) -> u8 {
        let instruction: u8 = unsafe { *self.ip };

        self.ip = unsafe { self.ip.add(1) };

        instruction
    }

    fn read_constant(&mut self) -> Value {
        let const_location = self.read_byte() as usize;
        self.chunk.as_ref().unwrap().constants[const_location]
    }

    fn binary_op(&mut self, op: BinaryOp) {
        if !self.peek(0).is_type(ValueType::Number) || !self.peek(1).is_type(ValueType::Number) {
            self.runtime_error("Operands must be numbers.");
        }
        let b = self.pop();
        let a = self.pop();
        self.push(match op {
            BinaryOp::Add => a + b,
            BinaryOp::Sub => a - b,
            BinaryOp::Div => a / b,
            BinaryOp::Mul => a * b,
	    BinaryOp::Greater => (a > b).into(),
	    BinaryOp::Less => (a < b).into(),
        });
    }

    fn run(&mut self) -> InterpretResult<()> {
        loop {
            #[cfg(feature = "debug_trace_execution")]
            {
                print!("          ");

                let mut slot: *mut Value = &mut self.stack[0];
                let stack_dif = unsafe { self.stack_top.offset_from(slot) };
                loop {
                    let cur_dif = unsafe { slot.offset_from(&mut self.stack[0]) };
                    if cur_dif == stack_dif {
                        break;
                    }

                    print!("[ ");
                    print_value(unsafe { *slot });
                    print!(" ]");
                    unsafe {
                        slot = slot.add(1);
                    }
                }
                println!();
                unsafe {
                    disassemble_instruction(
                        &self.chunk.as_ref().unwrap(),
                        self.ip.offset_from(&self.chunk.as_ref().unwrap().code[0]) as usize,
                    );
                }
            }

            let instruction = OpCode::from(self.read_byte());
            match instruction {
                OpCode::Return => {
                    print_value(self.pop());
                    println!();
                    return Ok(());
                }
                OpCode::Constant => {
                    let constant = self.read_constant();
                    self.push(constant);
                }
                OpCode::Negate => {
                    if !self.peek(0).is_type(ValueType::Number) {
                        self.runtime_error("Operand must be a number.");
                        return Err(VmErr::RuntimeError);
                    }
                    let val = -(self.pop().as_number());
                    self.push(val);
                }
                OpCode::Nil => self.push(Value::None),
                OpCode::True => self.push(true),
                OpCode::False => self.push(false),
		OpCode::Equal => {
		    let b = self.pop();
		    let a = self.pop();
		    self.push(a == b);
			
		}
                OpCode::Add => self.binary_op(BinaryOp::Add),
		OpCode::Greater => self.binary_op(BinaryOp::Greater),
		OpCode::Less => self.binary_op(BinaryOp::Less),
                OpCode::Subtract => self.binary_op(BinaryOp::Sub),
                OpCode::Divide => self.binary_op(BinaryOp::Div),
                OpCode::Multiply => self.binary_op(BinaryOp::Mul),
                OpCode::Not => {
                    let val = is_falsy(self.pop());
                    self.push(val);
                }


            }
        }
    }
}
