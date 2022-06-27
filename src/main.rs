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
