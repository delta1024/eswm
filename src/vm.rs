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
use crate::value::{print_value, Value, ValueType}; //objects::{ObjList, ObjString, Object, ObjPtr, ObjId}};
use std::result::Result;
// use std::rc::Rc;
// use std::cell::RefCell;
use std::collections::{HashMap, HashSet};

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
    pub globals: HashMap<String, Value>,
    pub strings: HashSet<String>,
    // pub objects: Option<Box<ObjList>>,
}

// pub fn allocate_obj(vm: &mut Vm, object: ObjPtr, id: ObjId) -> Object {
//     let list = vm.objects.take();
//     let new_list = Box::new(ObjList {
// 	value: object,
// 	next: list,
//     });
//     let object: *const ObjPtr = &new_list.value;
//     let _ = vm.objects.insert(new_list);
//     Object::new(id, object)

// }

pub fn allocate_string(vm: &mut Vm, to_allocate: String) -> *const String {
    let key = to_allocate.clone();
    vm.strings.insert(to_allocate);
    vm.strings.get(&key).unwrap()
}

fn generate_stack() -> Vec<Value> {
    let mut vector = Vec::new();
    vector.resize(STACK_MAX, Value::None);
    vector
}

fn is_falsy(value: Value) -> bool {
    value.is_type(ValueType::Nil) || (value.is_type(ValueType::Bool) && !value.as_bool())
}

/// concatenates the two values on the stack into a new value
fn concatenate(vm: &mut Vm) {
    let b = String::from(vm.pop().as_rstring());
    let a = String::from(vm.pop().as_rstring());
    let c = format!("{}{}", a, b);
    let c = allocate_string(vm, c);
    vm.push(c);
}

impl Vm {
    pub fn new() -> Self {
        let stack = generate_stack();
        let mut vm = Vm {
            chunk: None,
            ip: &mut 0,
            stack,
            stack_top: &mut Value::None,
            globals: HashMap::new(),
            strings: HashSet::new(),
            // objects: Some(Box::new(ObjList {
            // 	value: Rc::new(RefCell::new(ObjString(String::new()))),
            // 	next: None,
            // }
            // ))
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
        let chunk = compile(self, source)?;

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

    fn read_string(&mut self) -> String {
        self.read_constant().as_rstring()
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
                OpCode::Add => {
                    if self.peek(0).is_type(ValueType::String)
                        && self.peek(1).is_type(ValueType::String)
                    {
                        concatenate(self);
                    } else if self.peek(0).is_type(ValueType::Number)
                        && self.peek(1).is_type(ValueType::Number)
                    {
                        self.binary_op(BinaryOp::Add)
                    } else {
                        self.runtime_error("Operands must be two numbers or two strings.");
                        return Err(VmErr::RuntimeError);
                    }
                }
                OpCode::Greater => self.binary_op(BinaryOp::Greater),
                OpCode::Less => self.binary_op(BinaryOp::Less),
                OpCode::Subtract => self.binary_op(BinaryOp::Sub),
                OpCode::Divide => self.binary_op(BinaryOp::Div),
                OpCode::Multiply => self.binary_op(BinaryOp::Mul),
                OpCode::Not => {
                    let val = is_falsy(self.pop());
                    self.push(val);
                }
                OpCode::Print => {
                    print_value(self.pop());
		    println!();
                }
                OpCode::Pop => {
                    self.pop();
                }
                OpCode::DefineGlobal => {
                    let name = self.read_string();
                    let value = self.peek(0);
                    self.globals.insert(name, value);
                    self.pop();
                }
                OpCode::GetGlobal => {
                    let name = self.read_string();
                    let value = match self.globals.get(&name) {
                        Some(n) => n.clone(),
                        None => {
                            self.runtime_error(&format!("Undefined varialbe: '{}'.", name));
                            return Err(VmErr::RuntimeError);
                        }
                    };
                    self.push(value);
                }
                OpCode::SetGlobal => {
                    let name = self.read_string();
                    let val = self.peek(0);
                    let result = self.globals.insert(name.clone(), val);
                    if let None = result {
                        self.globals.remove_entry(&name);
                        self.runtime_error(&format!("Undefined variable '{}'.", name));
                        return Err(VmErr::RuntimeError);
                    }
                }
		OpCode::GetLocal => {
		    let slot = self.read_byte();
		    let val = self.stack[slot as usize];
		    self.push(val);
		}
		OpCode::SetLocal => {
		    let slot = self.read_byte();
		    let val = self.peek(0);
		    self.stack[slot as usize] = val;
		}
            }
        }
    }
}
