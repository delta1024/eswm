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

extern crate eswm_proc;
// use std::rc::Rc;
// use std::cell::RefCell;
use crate::lib::chunk::{Chunk, OpCode};
#[cfg(feature = "debug_print_code")]
use crate::lib::debug::disassemble_chunk;
use crate::value::Value;
use crate::vm::{allocate_string, InterpretResult, Vm, VmErr};

use eswm_proc::rule;

mod scanner;
use scanner::{Scanner, Token, TokenType};

const UINT8_COUNT: usize = u8::MAX as usize + 1;

#[derive(Default, Copy, Clone)]
struct ParseRule {
    prefix: Option<fn(&mut Parser, bool)>,
    infix: Option<fn(&mut Parser, bool)>,
    precedence: Precedence,
}

fn gen_compiler_stack() -> Vec<Local> {
    let mut n = Vec::new();
    n.resize(UINT8_COUNT, Local::default());
    n
}

#[derive(Clone)]
pub struct Local {
    name: Token,
    depth: isize,
}

impl Default for Local {
    fn default() -> Local {
	Local {
	    name: Token::default(),
	    depth: 0,
	}
    }
}


pub struct Compiler {
    locals: Vec<Local>,
    local_count: usize,
    scope_depth: isize,
}

impl Compiler {
    pub fn new() -> Compiler {
	Compiler {
	    locals: gen_compiler_stack(),
	    local_count: 0,
	    scope_depth: 0,
	}
    }

    fn mark_initialized(&mut self) {
	let depth = self.scope_depth;
	self.locals[self.local_count - 1].depth = depth;
    }

}
struct Parser<'a, 'b> {
    current: Option<Token>,
    previous: Option<Token>,
    scanner: &'a mut Scanner,
    chunk: &'b mut Chunk,
    rule: Option<&'a ParseRule>,
    had_error: bool,
    panic_mode: bool,
    vm: &'b mut Vm,
    compiler: &'b mut Compiler,
}

impl<'a, 'b> Parser<'a, 'b> {
    fn new(vm: &'b mut Vm, compiler: &'b mut Compiler, scanner: &'a mut Scanner, chunk: &'b mut Chunk) -> Self {
        Parser {
            current: None,
            previous: None,
            scanner,
            chunk,
            rule: None,
            had_error: false,
            panic_mode: false,
            vm,
	    compiler,
        }
    }

    fn advance(&mut self) {
        self.previous = self.current.clone();

        loop {
            self.current = Some(self.scanner.scan_token());
            if self.current.as_ref().unwrap().id != TokenType::Error {
                break;
            }

            self.error_at_current(&self.current.as_ref().unwrap().string());
        }
    }

    fn consume(&mut self, id: TokenType, message: &str) {
        if self.current.as_ref().unwrap().id == id {
            self.advance();
            return;
        }

        self.error_at_current(message);
    }

    fn check(&mut self, id: TokenType) -> bool {
        self.current.as_ref().unwrap().id == id
    }

    fn matches(&mut self, id: TokenType) -> bool {
        if !self.check(id) {
            return false;
        }
        self.advance();
        true
    }

    fn emit_byte(&mut self, byte: u8) {
        self.chunk.write(byte, self.previous.as_ref().unwrap().line);
    }

    fn emit_bytes(&mut self, byte1: u8, byte2: u8) {
        self.emit_byte(byte1);
        self.emit_byte(byte2);
    }

    fn emit_return(&mut self) {
        self.emit_byte(OpCode::Return as u8);
    }

    fn make_constant<T: Into<Value>>(&mut self, value: T) -> u8 {
        let constant = self.chunk.constant(value.into());
        if constant > u8::MAX {
            self.error("Too many constants in one chunk");
            0
        } else {
            constant
        }
    }

    fn emit_constant<T: Into<Value>>(&mut self, constant: T) {
        let constant = self.make_constant(constant);
        self.emit_bytes(OpCode::Constant as u8, constant);
    }

    fn end_compiler(&mut self) {
        self.emit_return();
        #[cfg(feature = "debug_print_code")]
        {
            if !self.had_error {
                disassemble_chunk(&self.chunk, "code");
            }
        }
    }

    fn parse_precedence(&mut self, precedence: Precedence) {
        self.advance();
        let new_rule = self.previous.as_ref().unwrap().id;
        self.get_rule(new_rule);
	let can_assign = precedence <= Precedence::Assignment;
        if let Some(rule) = self.rule {
            
	    if let Some(rulefn) = rule.prefix {
                rulefn(self, can_assign);
            }

	    
            loop {
                let new_rule = {
                    self.get_rule(self.current.as_ref().unwrap().id);
                    self.rule.as_ref().unwrap().precedence
                };

                if precedence <= new_rule {
                    self.advance();
                    self.get_rule(self.previous.as_ref().unwrap().id);

                    if let Some(rule) = self.rule {
                        if let Some(rulefn) = rule.infix {
                            rulefn(self, can_assign);
                        }
                    }
                    self.get_rule(self.current.as_ref().unwrap().id);
                } else {
                    break;
                }
            }
	    if can_assign && self.matches(TokenType::Equal) {
		self.error("Invalid assignment target.");		
	    }
	}
    }

    fn identifier_constant(&mut self, name: &Token) -> u8 {
         let value = allocate_string(self.vm, name.string());
        self.make_constant(value)
    }

    fn add_local(&mut self, name: Token) {
	if self.compiler.local_count == UINT8_COUNT {
	    self.error("Too many local variables in fuction.");
	    return;
	}
	
	let mut local = &mut self.compiler.locals[self.compiler.local_count];
	self.compiler.local_count += 1;
	local.name = name;
	local.depth = -1;
    }
    
    fn declare_variable(&mut self) {
	if self.compiler.scope_depth == 0 {
	    return;
	}

	let name = if let Some(ref t) = self.previous {
	    t.clone()
	} else {
	    panic!("Expected Token");
	};

	let mut i = self.compiler.local_count as isize - 1;
	while i >= 0 {
	    let local = &self.compiler.locals[i as usize];
	    if local.depth != -1 && local.depth < self.compiler.scope_depth {
		break;
	    }

	    if identifiers_equal(&name, &local.name){
		self.error("Already a variable with this name in this scope.");
	    }
	    i -= 1;
	}
	self.add_local(name);
    }
    
    fn parse_variable(&mut self, error_message: &str) -> u8 {
        self.consume(TokenType::Identifier, error_message);
	self.declare_variable();

	if self.compiler.scope_depth > 0 {
	    return 0;
	}
	
        let token = self.previous.as_ref().unwrap().clone();
        self.identifier_constant(&token)
    }

    fn define_variable(&mut self, global: u8) {
	if self.compiler.scope_depth > 0 {
	    self.compiler.mark_initialized();
	    return;
	}       
	self.emit_bytes(OpCode::DefineGlobal as u8, global);
    }

    fn get_rule(&mut self, id: TokenType) {
        if id as usize > 39 {
            self.rule = None;
            return;
        }
        self.rule = Some(&RULES[id as usize]);
    }

    fn begin_scope(&mut self) {
	self.compiler.scope_depth += 1;
    }

    fn end_scope(&mut self) {
	self.compiler.scope_depth -= 1;

	while self.compiler.local_count > 0 && (self.compiler.locals[self.compiler.local_count - 1].depth) as isize > self.compiler.scope_depth {
	    self.emit_byte(OpCode::Pop as u8);
	    self.compiler.local_count -= 1;
	}

    }

    fn resolve_local(&mut self, name: &Token) -> Result<u8, ()> {
	if self.compiler.local_count == 0 {
	    return Err(());
	}
	
	let mut i = (self.compiler.local_count - 1) as isize;
	while i >= 0 {
	    let local = &self.compiler.locals[i as usize];
	    if identifiers_equal(name, &local.name) {
		if local.depth == -1 {
		    self.error("Can't read variable in its own initializer.");
		}

		return Ok(i as u8);
	    }
	    i -= 1;
	}
	Err(())
    }


    fn error_at_current(&mut self, message: &str) {
        self.error_at(&self.current.clone().unwrap(), message);
    }

    fn error(&mut self, message: &str) {
        self.error_at(&self.previous.clone().unwrap(), message);
    }

    fn error_at(&mut self, token: &Token, message: &str) {
        if self.panic_mode {
            return;
        }

        self.panic_mode = true;
        eprint!("[line {}] Error", token.line);

        if token.id == TokenType::Eof {
            eprint!(" at end");
        } else if token.id == TokenType::Error {
            // Nothing.
        } else {
            eprint!(" at '{}'", token.string())
        }

        eprintln!(": {}", message);
        self.had_error = true;
    }
}
fn identifiers_equal(a: &Token, b: &Token ) -> bool {
    a.string() == b.string()
}
fn binary(parser: &mut Parser, _can_assign: bool) {
    let operator_id = parser.previous.as_ref().unwrap().id;
    parser.get_rule(operator_id);

    let precedence: Precedence = parser.rule.as_ref().unwrap().precedence.add_one();
    parser.parse_precedence(precedence);

    match operator_id {
        TokenType::BangEqual => parser.emit_bytes(OpCode::Equal as u8, OpCode::Not as u8),
        TokenType::EqualEqual => parser.emit_byte(OpCode::Equal as u8),
        TokenType::Greater => parser.emit_byte(OpCode::Greater as u8),
        TokenType::GreaterEqual => parser.emit_bytes(OpCode::Less as u8, OpCode::Not as u8),
        TokenType::Less => parser.emit_byte(OpCode::Less as u8),
        TokenType::LessEqual => parser.emit_bytes(OpCode::Greater as u8, OpCode::Not as u8),
        TokenType::Plus => parser.emit_byte(OpCode::Add as u8),
        TokenType::Minus => parser.emit_byte(OpCode::Subtract as u8),
        TokenType::Star => parser.emit_byte(OpCode::Multiply as u8),
        TokenType::Slash => parser.emit_byte(OpCode::Divide as u8),
        _ => unreachable!(),
    }
}

fn literal(parser: &mut Parser, _can_assign: bool) {
    match parser.previous.as_ref().unwrap().id {
        TokenType::False => parser.emit_byte(OpCode::False as u8),
        TokenType::True => parser.emit_byte(OpCode::True as u8),
        TokenType::Nil => parser.emit_byte(OpCode::Nil as u8),
        _ => unreachable!(),
    }
}

fn grouping(parser: &mut Parser, _can_assign: bool) {
    expression(parser);
    parser.consume(TokenType::LeftParen, "Exect ')' after expression.");
}

fn number(parser: &mut Parser, _can_assign: bool) {
    let value: f64 = parser
        .previous
        .as_ref()
        .unwrap()
        .string()
        .as_str()
        .parse()
        .unwrap();
    parser.emit_constant(value);
}

fn string(parser: &mut Parser, _can_assign: bool) {
    let string = String::from(&parser.previous.as_ref().unwrap().string());
    let mut string = string.chars();
    string.next();
    string.next_back();
    let string = String::from(string.as_str());

    let string = allocate_string(parser.vm, string);
    parser.emit_constant(string);
}

fn variable(parser: &mut Parser, can_assign: bool) {
    let token = parser.previous.as_ref().unwrap().clone();
    named_variable(parser, &token, can_assign);
}

fn named_variable(parser: &mut Parser, token: &Token, can_assign: bool) {
    let get_op: u8;
    let set_op: u8;
    let arg: u8;

    if let Ok(v) = parser.resolve_local(token) {
	arg = v;
	get_op = OpCode::GetLocal as u8;
	set_op = OpCode::SetLocal as u8;
    } else {
	arg = parser.identifier_constant(token);
	get_op = OpCode::GetGlobal as u8;
	set_op = OpCode::SetGlobal as u8;
    }


    if can_assign && parser.matches(TokenType::Equal) {
        expression(parser);
        parser.emit_bytes(set_op, arg);
    } else {
        parser.emit_bytes(get_op, arg);
    }
}

fn unary(parser: &mut Parser, _can_assign: bool) {
    let operator_type = parser.previous.as_ref().unwrap().id;

    // Compile the operand.
    parser.parse_precedence(Precedence::Unary);

    // Emit the operator instruction.
    match operator_type {
        TokenType::Bang => parser.emit_byte(OpCode::Not as u8),
        TokenType::Minus => parser.emit_byte(OpCode::Negate as u8),
        _ => unreachable!(),
    }
}

fn expression(parser: &mut Parser) {
    parser.parse_precedence(Precedence::Assignment);
}

fn block(parser: &mut Parser) {
    while !parser.check(TokenType::RightBrace) && !parser.check(TokenType::Eof) {
	decleration(parser);
    }

    parser.consume(TokenType::RightBrace, "Expect '}' after block.");
}

fn var_decleration(parser: &mut Parser) {
    let global: u8 = parser.parse_variable("Expect variable name.");

    if parser.matches(TokenType::Equal) {
        expression(parser);
    } else {
        parser.emit_byte(OpCode::Nil as u8);
    }

    parser.consume(
        TokenType::Semicolon,
        "Expect ';' after variable declaration.",
    );

    parser.define_variable(global);
}

fn decleration(parser: &mut Parser) {
    if parser.matches(TokenType::Var) {
        var_decleration(parser);
    } else {
        statement(parser);
    }

    if parser.panic_mode {
        synchronize(parser);
    }
}

fn statement(parser: &mut Parser) {
    if parser.matches(TokenType::Print) {
        print_statement(parser);
    } else if parser.matches(TokenType::LeftBrace) {
	parser.begin_scope();
	block(parser);
	parser.end_scope();
    } else {
        expression_statement(parser);
    }
}

fn print_statement(parser: &mut Parser) {
    expression(parser);
    parser.consume(TokenType::Semicolon, "Expected ';' after value.");
    parser.emit_byte(OpCode::Print as u8);
}

fn synchronize(parser: &mut Parser) {
    parser.panic_mode = false;

    loop {
        if parser.current.as_ref().unwrap().id == TokenType::Eof {
            break;
        } else if parser.previous.as_ref().unwrap().id == TokenType::Semicolon {
            return;
        } else {
            match parser.current.as_ref().unwrap().id {
                TokenType::Class
                | TokenType::Fun
                | TokenType::Var
                | TokenType::For
                | TokenType::If
                | TokenType::While
                | TokenType::Print
                | TokenType::Return => return,
                _ => parser.advance(),
            }
        }
    }
}

fn expression_statement(parser: &mut Parser) {
    expression(parser);
    parser.consume(TokenType::Semicolon, "Expect ';' after expression.");
    parser.emit_byte(OpCode::Pop as u8);
}

#[derive(Eq, Ord, PartialEq, PartialOrd, Copy, Clone, Debug)]
enum Precedence {
    None,
    /// =
    Assignment,
    /// or
    Or,
    /// and
    And,
    /// == !=
    Equality,
    /// < > <= >=
    Comparison,
    /// + -
    Term,
    /// * /
    Factor,
    /// ! -
    Unary,
    /// . ()
    Call,
    Primary,
    Ext,
}
impl Precedence {
    fn add_one(&self) -> Precedence {
        match *self {
            Precedence::None => Precedence::Assignment,
            Precedence::Assignment => Precedence::Or,
            Precedence::Or => Precedence::And,
            Precedence::And => Precedence::Equality,
            Precedence::Equality => Precedence::Comparison,
            Precedence::Comparison => Precedence::Term,
            Precedence::Term => Precedence::Factor,
            Precedence::Factor => Precedence::Unary,
            Precedence::Unary => Precedence::Call,
            Precedence::Call => Precedence::Primary,
            Precedence::Primary => Precedence::Ext,
            Precedence::Ext => Precedence::Ext,
        }
    }
}

impl Default for Precedence {
    fn default() -> Self {
        Precedence::None
    }
}

#[rustfmt::skip]
const RULES: [ParseRule; 40] = [
    // Single character tokens.
    rule!((TokenType::LeftParen   , Some(grouping), None        , Precedence::None      )),
    rule!((TokenType::RightParan  , None          , None        , Precedence::None      )),
    rule!((TokenType::LeftBrace   , None          , None        , Precedence::None      )),
    rule!((TokenType::RightBrace  , None          , None        , Precedence::None      )),
    rule!((TokenType::Comma       , None          , None        , Precedence::None      )),
    rule!((TokenType::Dot         , None          , None        , Precedence::None      )),
    rule!((TokenType::Minus       , Some(unary)   , Some(binary), Precedence::Term      )),
    rule!((TokenType::Plus        , None          , Some(binary), Precedence::Term      )),
    rule!((TokenType::Semicolon   , None          , None        , Precedence::None      )),
    rule!((TokenType::Slash       , None          , Some(binary), Precedence::Factor    )),
    rule!((TokenType::Star        , None          , Some(binary), Precedence::Factor    )),
    // One or two character tokens						        
    rule!((TokenType::Bang        , Some(unary)   , None        , Precedence::None      )),
    rule!((TokenType::BangEqual   , None          , Some(binary), Precedence::Equality  )),
    rule!((TokenType::Equal       , None          , None        , Precedence::None      )),
    rule!((TokenType::EqualEqual  , None          , Some(binary), Precedence::Equality  )),
    rule!((TokenType::Greater     , None          , Some(binary), Precedence::Comparison)),
    rule!((TokenType::GreaterEqual, None          , Some(binary), Precedence::Comparison)),
    rule!((TokenType::Less        , None          , Some(binary), Precedence::Comparison)),
    rule!((TokenType::LessEqual   , None          , Some(binary), Precedence::Comparison)),
    // Literals						        		    
    rule!((TokenType::Identifier  , Some(variable), None        , Precedence::None      )),
    rule!((TokenType::String      , Some(string)  , None        , Precedence::None      )),
    rule!((TokenType::Number      , Some(number)  , None        , Precedence::None      )),
    // Keywords						        		        
    rule!((TokenType::And         , None          , None        , Precedence::None      )),
    rule!((TokenType::Class       , None          , None        , Precedence::None      )),
    rule!((TokenType::Else        , None          , None        , Precedence::None      )),
    rule!((TokenType::False       , Some(literal) , None        , Precedence::None      )),
    rule!((TokenType::For         , None          , None        , Precedence::None      )),
    rule!((TokenType::Fun         , None          , None        , Precedence::None      )),
    rule!((TokenType::If          , None          , None        , Precedence::None      )),
    rule!((TokenType::Nil         , Some(literal) , None        , Precedence::None      )),
    rule!((TokenType::Or          , None          , None        , Precedence::None      )),
    rule!((TokenType::Print       , None          , None        , Precedence::None      )),
    rule!((TokenType::Return      , None          , None        , Precedence::None      )),
    rule!((TokenType::Super       , None          , None        , Precedence::None      )),
    rule!((TokenType::This        , None          , None        , Precedence::None      )),
    rule!((TokenType::True        , Some(literal) , None        , Precedence::None      )),
    rule!((TokenType::Var         , None          , None        , Precedence::None      )),
    rule!((TokenType::While       , None          , None        , Precedence::None      )),
    rule!((TokenType::Error       , None          , None        , Precedence::None      )),
    rule!((TokenType::Eof         , None          , None        , Precedence::None      )),
];

pub fn compile<'a, 'b>(vm: &'a mut Vm, source: &'b str) -> InterpretResult<Chunk> {
    let mut chunk = Chunk::new();
    let mut source: Vec<char> = source.chars().collect();
    source.push('\0');
    let mut scanner = Scanner::new(&source);
    let mut compiler = Compiler::new();
    let mut parser = Parser::new(vm, &mut compiler, &mut scanner, &mut chunk);
    parser.advance();

    while !parser.matches(TokenType::Eof) {
        decleration(&mut parser);
    }

    parser.end_compiler();
    if !parser.had_error {
        Ok(chunk)
    } else {
        Err(VmErr::CompileError)
    }
}
