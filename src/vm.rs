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

use crate::lib::chunk::{Chunk, OpCode};
use crate::lib::debug::disassemble_instruction;
use crate::lib::value::{print_value, Value};
use std::result::Result;

const STACK_MAX: usize = 256;

enum BinaryOp {
    Add,
    Sub,
    Div,
    Mul,
}

#[allow(dead_code)]
pub enum VmErr {
    CompileError,
    RuntimeError,
}

pub type InterpretResult<T> = Result<T, VmErr>;

pub struct Vm<'a> {
    pub chunk: Option<&'a Chunk>,
    pub ip: *const u8,
    pub stack: Vec<Value>,
    pub stack_top: *mut Value,
}

fn generate_stack() -> Vec<Value> {
    let mut vector = Vec::new();
    vector.resize(STACK_MAX, 0.0);
    vector
}

impl<'a> Vm<'a> {
    pub fn new() -> Self {
	let stack = generate_stack();
	let mut vm = Vm {
	    chunk: None,
	    ip: &0,
	    stack,
	    stack_top: &mut 0.0,
	};
	vm.reset_stack();
	vm
    }

    fn reset_stack(&mut self) {
	self.stack_top = &mut self.stack[0];
    }

    pub fn interpret(&mut self, chunk: &'a Chunk) -> InterpretResult<()> {
	self.chunk = Some(chunk);
	self.ip = &self.chunk.unwrap().code[0];
	self.run()
    }

    fn push(&mut self, value: Value) {
	unsafe {
	    *self.stack_top = value;
	    self.stack_top = self.stack_top.add(1);
	}
    }

    fn pop(&mut self) -> Value {
	unsafe {
	    self.stack_top = self.stack_top.sub(1);
	    *self.stack_top
	}
    }

    fn read_byte(&mut self) -> u8 {
	let instruction: u8 = unsafe { *self.ip };

	self.ip = unsafe { self.ip.add(1) };

	instruction
    }

    fn read_constant(&mut self) -> Value {
	let const_location = self.read_byte() as usize;
	self.chunk.unwrap().constants[const_location]
    }

    fn binary_op(&mut self, op: BinaryOp) {
	let b = self.pop();
	let a = self.pop();
	self.push( match op {
	    BinaryOp::Add => a + b,
	    BinaryOp::Sub => a - b,
	    BinaryOp::Div => a / b,
	    BinaryOp::Mul => a * b,
	});
    }

    fn run(&mut self) -> InterpretResult<()> {
	loop {
	    #[cfg(feature = "debug_trace_execution")]
	    {
		print!("          ");
		let temp_ptr: *const u8 = &self.chunk.unwrap().code[0];
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
		disassemble_instruction(&self.chunk.unwrap(), unsafe {
		    self.ip.offset_from(temp_ptr) as usize
		});
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
		    let val = -(self.pop());
		    self.push(val);
		}
		OpCode::Add => self.binary_op(BinaryOp::Add),
		OpCode::Subtract => self.binary_op(BinaryOp::Sub),
		OpCode::Divide => self.binary_op(BinaryOp::Div),
		OpCode::Multiply => self.binary_op(BinaryOp::Mul),
	    }
	}
    }
}
