extern crate eswm_proc;
#[cfg(feature = "debug_print_code")]
use crate::lib::debug::disassemble_chunk;
use crate::lib::{
    chunk::{Chunk, OpCode},
    value::Value,
};
use crate::vm::{InterpretResult, VmErr};
use eswm_proc::rule;
use std::ops::Add;

mod scanner;
use scanner::{Scanner, Token, TokenType};

#[derive(Default, Copy, Clone)]
struct ParseRule {
    prefix: Option<fn(&mut Parser)>,
    infix: Option<fn(&mut Parser)>,
    precedence: Precedence,
}

struct Parser<'a, 'b> {
    current: Option<Token>,
    previous: Option<Token>,
    scanner: &'a mut Scanner,
    chunk: &'b mut Chunk,
    rule: Option<&'a ParseRule>,
    had_error: bool,
    panic_mode: bool,
}

impl<'a, 'b> Parser<'a, 'b> {
    fn new(scanner: &'a mut Scanner, chunk: &'b mut Chunk) -> Self {
        Parser {
            current: None,
            previous: None,
            scanner,
            chunk,
            rule: None,
            had_error: false,
            panic_mode: false,
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

    fn make_constant(&mut self, value: Value) -> u8 {
        let constant = self.chunk.constant(value);
        if constant > u8::MAX {
            self.error("Too many constants in one chunk");
            0
        } else {
            constant
        }
    }

    fn emit_constant(&mut self, constant: Value) {
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

        if let Some(rule) = self.rule {
            if let Some(rulefn) = rule.prefix {
                rulefn(self);
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
                            rulefn(self);
                        }
                    }
                    self.get_rule(self.current.as_ref().unwrap().id);
                } else {
                    break;
                }
            }
        } else {
            self.error("Expect expression.");
        }
    }

    fn get_rule(&mut self, id: TokenType) {
        if id as usize > 39 {
            self.rule = None;
            return;
        }
        self.rule = Some(&RULES[id as usize]);
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

fn binary(parser: &mut Parser) {
    let operator_id = parser.previous.as_ref().unwrap().id;
    parser.get_rule(operator_id);

    let precedence: Precedence = parser.rule.as_ref().unwrap().precedence.add_one();
    parser.parse_precedence(precedence);

    match operator_id {
        TokenType::Plus => parser.emit_byte(OpCode::Add as u8),
        TokenType::Minus => parser.emit_byte(OpCode::Subtract as u8),
        TokenType::Star => parser.emit_byte(OpCode::Multiply as u8),
        TokenType::Slash => parser.emit_byte(OpCode::Divide as u8),
        _ => unreachable!(),
    }
}

fn grouping(parser: &mut Parser) {
    expression(parser);
    parser.consume(TokenType::LeftParen, "Exect ')' after expression.");
}

fn number(parser: &mut Parser) {
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

fn unary(parser: &mut Parser) {
    let operator_type = parser.previous.as_ref().unwrap().id;

    // Compile the operand.
    parser.parse_precedence(Precedence::Unary);

    // Emit the operator instruction.
    match operator_type {
        TokenType::Minus => parser.emit_byte(OpCode::Negate as u8),
        _ => unreachable!(),
    }
}

fn expression(parser: &mut Parser) {
    parser.parse_precedence(Precedence::Assignment);
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

const RULES: [ParseRule; 40] = [
    // Single character tokens.
    rule!((TokenType::LeftParen, Some(grouping), None, Precedence::None)),
    rule!((TokenType::RightParan, None, None, Precedence::None)),
    rule!((TokenType::LeftBrace, None, None, Precedence::None)),
    rule!((TokenType::RightBrace, None, None, Precedence::None)),
    rule!((TokenType::Comma, None, None, Precedence::None)),
    rule!((TokenType::Dot, None, None, Precedence::None)),
    rule!((
        TokenType::Minus,
        Some(unary),
        Some(binary),
        Precedence::Term
    )),
    rule!((TokenType::Plus, None, Some(binary), Precedence::Term)),
    rule!((TokenType::Semicolon, None, None, Precedence::None)),
    rule!((TokenType::Slash, None, Some(binary), Precedence::Factor)),
    rule!((TokenType::Star, None, Some(binary), Precedence::Factor)),
    // One or two character tokens
    rule!((TokenType::Bang, None, None, Precedence::None)),
    rule!((TokenType::BangEqual, None, None, Precedence::None)),
    rule!((TokenType::Equal, None, None, Precedence::None)),
    rule!((TokenType::EqualEqual, None, None, Precedence::None)),
    rule!((TokenType::Greater, None, None, Precedence::None)),
    rule!((TokenType::GreaterEqual, None, None, Precedence::None)),
    rule!((TokenType::Less, None, None, Precedence::None)),
    rule!((TokenType::LessEqual, None, None, Precedence::None)),
    // Literals
    rule!((TokenType::Identifier, None, None, Precedence::None)),
    rule!((TokenType::String, None, None, Precedence::None)),
    rule!((TokenType::Number, Some(number), None, Precedence::None)),
    // Keywords
    rule!((TokenType::And, None, None, Precedence::None)),
    rule!((TokenType::Class, None, None, Precedence::None)),
    rule!((TokenType::Else, None, None, Precedence::None)),
    rule!((TokenType::False, None, None, Precedence::None)),
    rule!((TokenType::For, None, None, Precedence::None)),
    rule!((TokenType::Fun, None, None, Precedence::None)),
    rule!((TokenType::If, None, None, Precedence::None)),
    rule!((TokenType::Nil, None, None, Precedence::None)),
    rule!((TokenType::Or, None, None, Precedence::None)),
    rule!((TokenType::Print, None, None, Precedence::None)),
    rule!((TokenType::Return, None, None, Precedence::None)),
    rule!((TokenType::Super, None, None, Precedence::None)),
    rule!((TokenType::This, None, None, Precedence::None)),
    rule!((TokenType::True, None, None, Precedence::None)),
    rule!((TokenType::Var, None, None, Precedence::None)),
    rule!((TokenType::While, None, None, Precedence::None)),
    rule!((TokenType::Error, None, None, Precedence::None)),
    rule!((TokenType::Eof, None, None, Precedence::None)),
];

pub fn compile(source: &str) -> InterpretResult<Chunk> {
    let mut chunk = Chunk::new();
    let mut source: Vec<char> = source.chars().collect();
    source.push('\0');
    let mut scanner = Scanner::new(&source);
    let mut parser = Parser::new(&mut scanner, &mut chunk);
    parser.advance();
    expression(&mut parser);
    parser.consume(TokenType::Eof, "Expected end of expression.");
    parser.end_compiler();
    if !parser.had_error {
        Ok(chunk)
    } else {
        Err(VmErr::CompileError)
    }
}