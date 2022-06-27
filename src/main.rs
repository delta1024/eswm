mod lib;
mod vm;
use crate::lib::{chunk, debug};

fn main() {
    let mut vm = vm::Vm::new();
    let mut chunk = chunk::Chunk::new();

    let mut constant = chunk.constant(1.2);
    chunk.write(chunk::OpCode::Constant as u8, 123);
    chunk.write(constant, 123);

    constant = chunk.constant(3.4);
    chunk.write(chunk::OpCode::Constant as u8, 123);
    chunk.write(constant, 123);

    chunk.write(chunk::OpCode::Add as u8, 123);

    constant = chunk.constant(5.6);
    chunk.write(chunk::OpCode::Constant as u8, 123);
    chunk.write(constant, 123);

    chunk.write(chunk::OpCode::Divide as u8, 123);
    chunk.write(chunk::OpCode::Negate as u8, 123);

    chunk.write(chunk::OpCode::Return as u8, 123);

    debug::disassemble_chunk(&chunk, "test chunk");
    if let Err(_vm_err) = vm.interpret(&chunk) {
        eprintln!("Error");
    }
}
