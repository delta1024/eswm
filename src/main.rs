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
use std::env;
use std::io;
use std::io::prelude::*;
use std::io::Write;
mod compiler;
#[allow(dead_code)]
mod lib;
#[allow(dead_code)]
mod vm;

fn repl() -> io::Result<()> {
    let mut vm: vm::Vm = vm::Vm::new();
    loop {
        let mut input = String::new();

        print!("> ");
        io::stdout().flush()?;

        io::stdin().read_line(&mut input)?;

        if let Err(_err) = vm.interpret(&input) {
            vm.chunk.take();
        }
        vm.chunk.take();
    }
}

fn run_file(file: &str) -> io::Result<()> {
    let mut vm: vm::Vm = vm::Vm::new();
    let mut file = std::fs::File::open(file)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    if let Err(result) = vm.interpret(&contents) {
        match result {
            vm::VmErr::CompileError => std::process::exit(65),
            vm::VmErr::RuntimeError => std::process::exit(70),
        }
    }

    Ok(())
}

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() == 1 {
        repl()?;
    } else if args.len() == 2 {
        run_file(&args[1])?;
    } else {
        eprintln!("Usage: eswm [path]");
        std::process::exit(64);
    }
    Ok(())
}
