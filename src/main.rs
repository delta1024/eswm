use std::fmt::{self, Display};

type Value = f64;

/// Prints [`Value`] to stdout.
fn print_value(value: Value) {
    print!("{}", value);
}

/// Code representing and instruction to execute. 
#[derive(Clone, Copy)]
pub enum OpCode {
    Return,
    Constant,
}

impl From<u8> for OpCode {
    fn from(value: u8) -> OpCode {
	match value {
	    0 => OpCode::Return,
	    1 => OpCode::Constant,
	    _ => unreachable!(),
	}
    }
}

impl Display for OpCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
	match *self {
	    OpCode::Return => write!(f, "OP_RETURN"),
	    OpCode::Constant => write!(f, "OP_CONSTANT"),
	}
    }
}

/// A chunk houses the instructions for the vm to execute 
struct Chunk {
    code: Vec<u8>,
    constants: Vec<Value>,
    lines: Vec<usize>,
} 

impl Chunk {
    fn new() -> Self {
	Self {
	    code: Vec::new(),
	    constants: Vec::new(),
	    lines: Vec::new(),
	}
    }
    /// Writes to [`chunk.code`] and [`chunk.lines`]
    fn write(&mut self, code: u8, line: usize) {
	self.code.push(code);
	self.lines.push(line);
    }
    /// Writes to [`chunk.constans`] and returns its position in the array.
    fn constant(&mut self, value: Value) -> u8 {
	self.constants.push(value);
	(self.constants.len() - 1) as u8
    }
}

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
    print_value(chunk.constants[constant]);
    print!("'\n");
    offset + 2
}

/// Prints code at chunk offset. 
fn disassemble_instruction(chunk: &Chunk, offset: usize) -> usize {
    print!("{:04} ", offset);

    if offset > 0 &&
	chunk.lines[offset] == chunk.lines[offset - 1] {
	    print!("   | ");
	} else {
	    print!("{:4} ", chunk.lines[offset]);
	}
    
    let instruction = OpCode::from(chunk.code[offset]);
    match instruction {
	OpCode::Return => simple_instruction(instruction, offset),
	OpCode::Constant => constant_instruction(instruction, chunk, offset),
    }
}

/// Prints the contents of [`Chunk`] to std out.
fn disassemble_chunk(chunk: &Chunk, name: &str) {
    println!("== {} ==", name);
    let mut offset = 0;
    while offset < chunk.code.len() {
	offset = disassemble_instruction(chunk, offset);
    }
}

fn main() {
    let mut chunk = Chunk::new();
    chunk.write(OpCode::Return as u8, 123);
    let constant = chunk.constant(1.2);
    chunk.write(OpCode::Constant as u8,  123);
    chunk.write(constant, 123);

    disassemble_chunk(&chunk, "test chunk");
    
}
