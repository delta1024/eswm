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

#[derive(PartialEq, Debug, Clone, Copy)]
pub(super) enum TokenType {
    // Single-character tokens.
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,
    // One or two character tokens.
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    // Literals.
    Identifier,
    String,
    Number,
    // Keywords.
    And,
    Class,
    Else,
    False,
    For,
    Fun,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,

    Error,
    Eof,
}

#[derive(Clone)]
pub(super) struct Token {
    pub(super) id: TokenType,
    pub(super) start: Option<*const char>,
    pub(super) length: Option<usize>,
    pub(super) line: usize,
    pub(super) error_string: Option<String>,
}

impl Default for Token {
    fn default() -> Token {
	Token {
	    id: TokenType::Error,
	    start: None,
	    length: None,
	    line: 0,
	    error_string: None,
	}
    }
}

impl Token {
    pub fn string(&self) -> String {
        if self.id != TokenType::Error {
            let mut string = String::new();
            let temp_ptr = self.start.unwrap();
            for i in 0..self.length.unwrap() {
                unsafe {
                    let character = *temp_ptr.add(i);
                    string.push(character);
                }
            }
            string
        } else {
            self.error_string.clone().unwrap()
        }
    }
}
pub(super) struct Scanner {
    start: *const char,
    current: *const char,
    line: usize,
}

impl Scanner {
    pub fn new(source: &Vec<char>) -> Self {
        Scanner {
            start: &source[0],
            current: &source[0],
            line: 1,
        }
    }

    fn is_alpha(c: char) -> bool {
        c >= 'a' && c <= 'z' || c >= 'A' && c <= 'Z' || c == '_'
    }

    fn is_digit(c: char) -> bool {
        c >= '0' && c <= '9'
    }

    fn is_at_end(&mut self) -> bool {
        //	println!("{:?}, {}", unsafe{*self.current as char}, unsafe{*self.current as char == '\0'});
        unsafe { *self.current as char == '\0' }
    }

    fn advance(&mut self) -> char {
        unsafe {
            self.current = self.current.add(1);
            *self.current.sub(1) as char
        }
    }

    fn peek(&mut self) -> char {
        unsafe { *self.current as char }
    }

    fn peek_next(&mut self) -> char {
        if self.is_at_end() {
            '\0'
        } else {
            unsafe { *self.current.add(1) as char }
        }
    }

    fn matches(&mut self, expected: char) -> bool {
        if self.is_at_end() {
            return false;
        }
        if unsafe { *self.current } as char != expected {
            return false;
        }

        unsafe {
            self.current = self.current.add(1);
        }
        true
    }

    fn make_token(&mut self, id: TokenType) -> Token {
        unsafe {
            Token {
                id,
                start: Some(self.start),
                length: Some(self.current.offset_from(self.start) as usize),
                line: self.line,
                error_string: None,
            }
        }
    }

    fn error_token(&mut self, message: &str) -> Token {
        Token {
            id: TokenType::Error,
            start: None,
            length: None,
            line: self.line,
            error_string: Some(message.to_string()),
        }
    }

    fn skip_whitespace(&mut self) {
        loop {
            let c = self.peek();
            match c {
                ' ' | '\r' | '\t' => {
                    self.advance();
                }
                '\n' => {
                    self.line += 1;
                    self.advance();
                }
                '/' => {
                    if self.peek_next() == '/' {
                        while self.peek() != '\n' && !self.is_at_end() {
                            self.advance();
                        }
                    } else {
                        break;
                    }
                }

                _ => break,
            }
        }
    }

    fn check_keyword(
        &mut self,
        start: usize,
        length: usize,
        rest: &str,
        id: TokenType,
    ) -> TokenType {
        let scan_len = unsafe { self.current.offset_from(self.start) as usize };
        let mut scan_str = String::new();
        let temp_ptr = unsafe { self.start.add(start) };
        for i in 0..length {
            unsafe {
                scan_str.push(*temp_ptr.add(i));
            }
        }

        if scan_len == (start + length) && scan_str == rest {
            id
        } else {
            TokenType::Identifier
        }
    }

    fn identifier_id(&mut self) -> TokenType {
        unsafe {
            match *self.start {
                'a' => self.check_keyword(1, 2, "nd", TokenType::And),
                'c' => self.check_keyword(1, 4, "less", TokenType::Class),
                'e' => self.check_keyword(1, 3, "lse", TokenType::Else),
                'f' => {
                    if self.current.offset_from(self.start) > 1 {
                        match *self.start.add(1) {
                            'a' => self.check_keyword(2, 3, "lse", TokenType::False),
                            'o' => self.check_keyword(2, 1, "r", TokenType::For),
                            'u' => self.check_keyword(2, 1, "n", TokenType::Fun),
                            _ => TokenType::Identifier,
                        }
                    } else {
                        TokenType::Identifier
                    }
                }
                'i' => self.check_keyword(1, 1, "f", TokenType::If),
                'n' => self.check_keyword(1, 2, "il", TokenType::Nil),
                'o' => self.check_keyword(1, 1, "r", TokenType::Or),
                'p' => self.check_keyword(1, 4, "rint", TokenType::Print),
                'r' => self.check_keyword(1, 5, "eturn", TokenType::Return),
                's' => self.check_keyword(1, 4, "uper", TokenType::Super),
                't' => {
                    if self.current.offset_from(self.start) > 1 {
                        match *self.start.add(1) {
                            'h' => self.check_keyword(2, 2, "is", TokenType::This),
                            'r' => self.check_keyword(2, 2, "ue", TokenType::True),
                            _ => TokenType::Identifier,
                        }
                    } else {
                        TokenType::Identifier
                    }
                }
                'v' => self.check_keyword(1, 2, "ar", TokenType::Var),
                'w' => self.check_keyword(1, 4, "hile", TokenType::While),
                _ => TokenType::Identifier,
            }
        }
    }

    fn identifier(&mut self) -> Token {
        while Scanner::is_alpha(self.peek()) || Scanner::is_digit(self.peek()) {
            self.advance();
        }
        let id = self.identifier_id();
        self.make_token(id)
    }

    fn number(&mut self) -> Token {
        while Scanner::is_digit(self.peek()) {
            self.advance();
        }

        // Look for a fractional part.
        if self.peek() == '.' && Scanner::is_digit(self.peek_next()) {
            // Consume the ".".
            self.advance();

            while Scanner::is_digit(self.peek()) {
                self.advance();
            }
        }

        self.make_token(TokenType::Number)
    }

    fn string(&mut self) -> Token {
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
            }
            self.advance();
        }

        if self.is_at_end() {
            return self.error_token("Unterminated string.");
        }

        // The closing quote.
        self.advance();
        self.make_token(TokenType::String)
    }

    pub fn scan_token(&mut self) -> Token {
        self.skip_whitespace();
        self.start = self.current;

        if self.is_at_end() {
            return self.make_token(TokenType::Eof);
        }

        let c = self.advance();

        if Scanner::is_alpha(c) {
            return self.identifier();
        } else if Scanner::is_digit(c) {
            return self.number();
        }

        match c {
            '(' => self.make_token(TokenType::LeftParen),
            ')' => self.make_token(TokenType::RightParen),
            '{' => self.make_token(TokenType::LeftBrace),
            '}' => self.make_token(TokenType::RightBrace),
            ';' => self.make_token(TokenType::Semicolon),
            ',' => self.make_token(TokenType::Comma),
            '.' => self.make_token(TokenType::Dot),
            '-' => self.make_token(TokenType::Minus),
            '+' => self.make_token(TokenType::Plus),
            '/' => self.make_token(TokenType::Slash),
            '*' => self.make_token(TokenType::Star),
            '!' => {
                if self.matches('=') {
                    self.make_token(TokenType::BangEqual)
                } else {
                    self.make_token(TokenType::Bang)
                }
            }
            '=' => {
                if self.matches('=') {
                    self.make_token(TokenType::EqualEqual)
                } else {
                    self.make_token(TokenType::Equal)
                }
            }
            '<' => {
                if self.matches('=') {
                    self.make_token(TokenType::LessEqual)
                } else {
                    self.make_token(TokenType::Less)
                }
            }
            '>' => {
                if self.matches('=') {
                    self.make_token(TokenType::GreaterEqual)
                } else {
                    self.make_token(TokenType::Greater)
                }
            }
            '"' => self.string(),
            _ => self.error_token("Unexpected character."),
        }
    }
}
