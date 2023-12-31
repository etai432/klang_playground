#![allow(unused)]
use super::expr::Expr;
use crate::{error, KlangError};
use std::collections::HashMap;
use std::fmt;
use std::iter::Peekable;
use std::str::Chars;

#[derive(Debug, Clone)]
pub struct Scanner<'a> {
    pub source: &'a str,
    pub chars: Peekable<Chars<'a>>,
    pub line: usize,
    pub tokens: Vec<Token>,
    had_error: bool,
}

impl<'a> Scanner<'a> {
    pub fn new(source: &'a str) -> Scanner<'a> {
        Scanner {
            source,
            chars: source.chars().peekable(),
            line: 1,
            tokens: Vec::new(),
            had_error: false,
        }
    }

    pub fn scan_tokens(&mut self) -> Result<Vec<Token>, String> {
        let mut error_string = String::new();
        while let Some(ch) = self.chars.next() {
            match ch {
                '(' => self.make_token(TokenType::LeftParen, ch.to_string(), self.line, None),
                ')' => self.make_token(TokenType::RightParen, ch.to_string(), self.line, None),
                '{' => self.make_token(TokenType::LeftBrace, ch.to_string(), self.line, None),
                '}' => self.make_token(TokenType::RightBrace, ch.to_string(), self.line, None),
                ',' => self.make_token(TokenType::Comma, ch.to_string(), self.line, None),
                '-' => {
                    if self.tokens.len() >= 2
                        && self.tokens[self.tokens.len() - 1].tt == TokenType::Minus
                        && self.tokens[self.tokens.len() - 2].tt != TokenType::Int
                        && self.tokens[self.tokens.len() - 2].tt != TokenType::Float
                    {
                        error_string += error::KlangError::error(
                            KlangError::ScannerError,
                            "we shall not allow minus spamming. use 1 bitch",
                            self.line,
                        )
                        .as_str();
                        error_string += "\n";
                        self.had_error = true;
                    }
                    self.make_token(TokenType::Minus, ch.to_string(), self.line, None)
                }
                '+' => self.make_token(TokenType::Plus, ch.to_string(), self.line, None),
                ';' => self.make_token(TokenType::Semicolon, ch.to_string(), self.line, None),
                '*' => self.make_token(TokenType::Star, ch.to_string(), self.line, None),
                '%' => self.make_token(TokenType::Modulo, ch.to_string(), self.line, None),
                '[' => self.make_token(TokenType::LeftSquare, ch.to_string(), self.line, None),
                ']' => self.make_token(TokenType::RightSquare, ch.to_string(), self.line, None),
                '/' => {
                    if self.is_next('/') {
                        while self.chars.next() != Some('\n') && self.chars.peek().is_some() {}
                        self.line += 1;
                    } else {
                        self.make_token(TokenType::Slash, ch.to_string(), self.line, None);
                    }
                }
                '!' => {
                    if self.is_next('=') {
                        let next = self.chars.next().unwrap();
                        self.make_token(
                            TokenType::BangEqual,
                            String::from(ch) + &String::from(next),
                            self.line,
                            None,
                        );
                    } else {
                        if !self.tokens.is_empty()
                            && self.tokens[self.tokens.len() - 1].tt == TokenType::Bang
                        {
                            error_string += error::KlangError::error(
                                KlangError::ScannerError,
                                "we shall not allow bang spamming. use 1 bitch",
                                self.line,
                            )
                            .as_str();
                            error_string += "\n";
                            self.had_error = true;
                        }
                        self.make_token(TokenType::Bang, ch.to_string(), self.line, None)
                    }
                }
                '=' => {
                    if self.is_next('=') {
                        let next = self.chars.next().unwrap();
                        self.make_token(
                            TokenType::EqualEqual,
                            String::from(ch) + &String::from(next),
                            self.line,
                            None,
                        );
                    } else {
                        self.make_token(TokenType::Equal, ch.to_string(), self.line, None)
                    }
                }
                '>' => {
                    if self.is_next('=') {
                        let next = self.chars.next().unwrap();
                        self.make_token(
                            TokenType::GreaterEqual,
                            String::from(ch) + &String::from(next),
                            self.line,
                            None,
                        );
                    } else {
                        self.make_token(TokenType::Greater, ch.to_string(), self.line, None)
                    }
                }
                '<' => {
                    if self.is_next('=') {
                        let next = self.chars.next().unwrap();
                        self.make_token(
                            TokenType::LessEqual,
                            String::from(ch) + &String::from(next),
                            self.line,
                            None,
                        );
                    } else {
                        self.make_token(TokenType::Less, ch.to_string(), self.line, None)
                    }
                }
                '.' => {
                    if self.is_next('.') {
                        let next = self.chars.next().unwrap();
                        self.make_token(
                            TokenType::Range,
                            String::from(ch) + &String::from(next),
                            self.line,
                            None,
                        );
                    } else {
                        self.make_token(TokenType::Dot, ch.to_string(), self.line, None)
                    }
                }
                '&' => {
                    if self.is_next('&') {
                        let next = self.chars.next().unwrap();
                        self.make_token(
                            TokenType::And,
                            String::from(ch) + &String::from(next),
                            self.line,
                            None,
                        )
                    } else {
                        error_string += error::KlangError::error(
                            KlangError::ScannerError,
                            "missing a second & you fat fuck",
                            self.line,
                        )
                        .as_str();
                        error_string += "\n";
                        self.had_error = true;
                    }
                }
                '|' => {
                    if self.is_next('|') {
                        let next = self.chars.next().unwrap();
                        self.make_token(
                            TokenType::Or,
                            String::from(ch) + &String::from(next),
                            self.line,
                            None,
                        )
                    } else {
                        error_string += error::KlangError::error(
                            KlangError::ScannerError,
                            "missing a second | you stupid gay",
                            self.line,
                        )
                        .as_str();
                        error_string += "\n";
                        self.had_error = true;
                    }
                }
                '"' => error_string += self.string().as_str(),
                ' ' => (),
                '\r' => (),
                '\t' => (),
                '\n' => self.line += 1,
                _ => {
                    if ch.is_ascii_digit() {
                        error_string += self.number(ch).as_str();
                    } else if ch.is_ascii_alphabetic() {
                        error_string += self.identifier(ch).as_str();
                    } else {
                        error_string += error::KlangError::error(
                            KlangError::ScannerError,
                            "unexpected character",
                            self.line,
                        )
                        .as_str();
                        error_string += "\n";
                        self.had_error = true;
                    }
                }
            }
        }
        self.make_token(TokenType::Eof, String::from(""), self.line, None);
        if self.had_error {
            Err(error_string)
        } else {
            Ok(std::mem::take(&mut self.tokens))
        }
    }
    fn make_token(&mut self, tt: TokenType, text: String, line: usize, value: Option<Value>) {
        self.tokens.push(Token {
            tt,
            lexeme: text,
            literal: value,
            line,
        })
    }
    fn is_next(&mut self, ch: char) -> bool {
        self.chars.peek() == Some(&ch)
    }

    fn identifier(&mut self, ch: char) -> String {
        let mut word = String::from(ch);
        while self.chars.peek().unwrap_or(&'\0').is_ascii_alphanumeric() {
            word.push(self.chars.next().unwrap());
        }
        match word.as_str() {
            "let" => self.make_token(TokenType::Let, "".to_string(), self.line, None),
            "in" => self.make_token(TokenType::In, "".to_string(), self.line, None),
            "else" => self.make_token(TokenType::Else, "".to_string(), self.line, None),
            "for" => self.make_token(TokenType::For, "".to_string(), self.line, None),
            "if" => self.make_token(TokenType::If, "".to_string(), self.line, None),
            "print" => self.make_token(TokenType::Print, "".to_string(), self.line, None),
            "while" => self.make_token(TokenType::While, "".to_string(), self.line, None),
            "int" => self.make_token(TokenType::Int, "".to_string(), self.line, None),
            "float" => self.make_token(TokenType::Float, "".to_string(), self.line, None),
            "string" => self.make_token(TokenType::String, "".to_string(), self.line, None),
            "bool" => self.make_token(TokenType::Bool, "".to_string(), self.line, None),
            "fn" => self.make_token(TokenType::Fn, "".to_string(), self.line, None),
            "return" => self.make_token(TokenType::Return, "".to_string(), self.line, None),
            "true" => self.make_token(
                TokenType::Bool,
                "true".to_string(),
                self.line,
                Some(Value::Bool(true)),
            ),
            "false" => self.make_token(
                TokenType::Bool,
                "false".to_string(),
                self.line,
                Some(Value::Bool(false)),
            ),
            "std" => {
                if self.chars.next().unwrap() == ':' && self.chars.next().unwrap() == ':' {
                    self.make_token(TokenType::NativeCall, "".to_string(), self.line, None)
                } else {
                    self.had_error = true;
                    return error::KlangError::error(
                        KlangError::ScannerError,
                        "cannot use std without calling a native fn",
                        self.line,
                    ) + "\n";
                }
            }
            _ => self.make_token(TokenType::Identifier, word, self.line, None),
        }
        String::new()
    }
    fn number(&mut self, ch: char) -> String {
        let mut number = String::from(ch);
        while self.chars.peek().unwrap_or(&'\0').is_ascii_digit() {
            number.push(self.chars.next().unwrap());
        }
        if self.chars.peek().unwrap_or(&'\0') == &'.' {
            number.push(self.chars.next().unwrap());
            if self.chars.peek().unwrap_or(&'\0') == &'.' {
                number.pop();
                let value = match number.parse::<i64>() {
                    Ok(e) => Some(Value::Number(e as f64)),
                    Err(_) => {
                        self.had_error = true;
                        return error::KlangError::error(
                            KlangError::ScannerError,
                            "failed to parse integer",
                            self.line,
                        ) + "\n";
                    }
                };
                self.make_token(TokenType::Int, "".to_string(), self.line, value);
                self.make_token(TokenType::Range, "..".to_string(), self.line, None);
                self.chars.next(); //consume 2nd dot
                String::new()
            } else {
                while self.chars.peek().unwrap_or(&'\0').is_ascii_digit() {
                    number.push(self.chars.next().unwrap());
                }
                if number.ends_with('.') {
                    self.had_error = true;
                    return error::KlangError::error(
                        KlangError::ScannerError,
                        "float cant end with a dot",
                        self.line,
                    ) + "\n";
                }
                let value = match number.parse::<f64>() {
                    Ok(e) => Some(Value::Number(e)),
                    Err(_) => {
                        self.had_error = true;
                        return error::KlangError::error(
                            KlangError::ScannerError,
                            "failed to parse float",
                            self.line,
                        ) + "\n";
                    }
                };
                self.make_token(TokenType::Float, "".to_string(), self.line, value);
                String::new()
            }
        } else {
            let value = match number.parse::<i64>() {
                Ok(e) => Some(Value::Number(e as f64)),
                Err(_) => {
                    self.had_error = true;
                    return error::KlangError::error(
                        KlangError::ScannerError,
                        "failed to parse integer",
                        self.line,
                    ) + "\n";
                }
            };
            self.make_token(TokenType::Int, "".to_string(), self.line, value);
            String::new()
        }
    }
    fn string(&mut self) -> String {
        let mut printables: Vec<Token> = Vec::new();
        let mut string = String::new();
        while self.chars.peek().unwrap_or(&'\0') != &'"' {
            if self.chars.peek().unwrap_or(&'\0') == &'\n' {
                self.line += 1
            }
            if self.chars.peek().is_none() {
                self.had_error = true;
                return error::KlangError::error(
                    KlangError::ScannerError,
                    "unterminated string",
                    self.line,
                ) + "\n";
            } else {
                match self.chars.peek().unwrap() {
                    '{' => {
                        string.push(self.chars.next().unwrap());
                        let mut string1 = String::new();
                        if self.chars.peek() == Some(&'}') {
                            self.had_error = true;
                            return error::KlangError::error(
                                KlangError::ScannerError,
                                "cannot print an empty identifier",
                                self.line,
                            ) + "\n";
                        }
                        let mut counter = 1;
                        while self.chars.peek().is_some() {
                            let ch1 = self.chars.next().unwrap();
                            string1.push(ch1);
                            match self.chars.peek().unwrap() {
                                '{' => counter += 1,
                                '}' => counter -= 1,
                                _ => (),
                            }
                            if counter == 0 {
                                printables.push(Token {
                                    tt: TokenType::Printable,
                                    lexeme: string1,
                                    literal: None,
                                    line: self.line,
                                });
                                break;
                            }
                        }
                    }
                    _ => string.push(self.chars.next().unwrap()),
                }
            }
        }
        self.chars.next(); //consume the 2nd "
        self.make_token(TokenType::String, string, self.line, None);
        for i in printables.into_iter() {
            self.tokens.push(i);
        }
        String::new()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum TokenType {
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Slash,
    Star,
    Modulo,
    Semicolon,
    LeftSquare,
    RightSquare,

    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    And,
    Or,

    Let,
    Identifier,
    String,
    Int,
    Float,
    Bool,
    If,
    Else,
    For,
    Range,
    In,
    While,
    Print,
    Fn,
    Return,
    Printable,
    NativeCall,
    Eof,
}

impl fmt::Display for TokenType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenType::LeftParen => write!(f, "LeftParen"),
            TokenType::RightParen => write!(f, "RightParen"),
            TokenType::LeftSquare => write!(f, "LeftSquare"),
            TokenType::RightSquare => write!(f, "RightSquare"),
            TokenType::LeftBrace => write!(f, "LeftBrace"),
            TokenType::RightBrace => write!(f, "RightBrace"),
            TokenType::Comma => write!(f, "Comma"),
            TokenType::Dot => write!(f, "Dot"),
            TokenType::Minus => write!(f, "Minus"),
            TokenType::Plus => write!(f, "Plus"),
            TokenType::Slash => write!(f, "Slash"),
            TokenType::Star => write!(f, "Star"),
            TokenType::Modulo => write!(f, "Modulo"),
            TokenType::Semicolon => write!(f, "Semicolon"),
            TokenType::Bang => write!(f, "Bang"),
            TokenType::BangEqual => write!(f, "BangEqual"),
            TokenType::Equal => write!(f, "Equal"),
            TokenType::EqualEqual => write!(f, "EqualEqual"),
            TokenType::Greater => write!(f, "Greater"),
            TokenType::GreaterEqual => write!(f, "GreaterEqual"),
            TokenType::Less => write!(f, "Less"),
            TokenType::LessEqual => write!(f, "LessEqual"),
            TokenType::And => write!(f, "And"),
            TokenType::Or => write!(f, "Or"),
            TokenType::Let => write!(f, "Let"),
            TokenType::Identifier => write!(f, "Identifier"),
            TokenType::String => write!(f, "String"),
            TokenType::Int => write!(f, "Int"),
            TokenType::Float => write!(f, "Float"),
            TokenType::Bool => write!(f, "Bool"),
            TokenType::If => write!(f, "If"),
            TokenType::Else => write!(f, "Else"),
            TokenType::For => write!(f, "For"),
            TokenType::Range => write!(f, "Range"),
            TokenType::In => write!(f, "In"),
            TokenType::While => write!(f, "While"),
            TokenType::Print => write!(f, "Print"),
            TokenType::Fn => write!(f, "function"),
            TokenType::Return => write!(f, "return"),
            TokenType::Eof => write!(f, "Eof"),
            TokenType::Printable => write!(f, "Printable"),
            TokenType::NativeCall => write!(f, "NativeCall"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Value {
    String {
        string: String,
        printables: Vec<Expr>,
    },
    Number(f64),
    Bool(bool),
    Vec(Vec<Value>),
    None,
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::String { string, .. } => write!(f, "{}", string),
            Value::Number(i) => write!(f, "{}", i),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Vec(v) => {
                let mut vec = v.clone();
                write!(f, "[");
                let x = vec.pop();
                for i in vec {
                    match i {
                        Value::String {
                            string,
                            printables: _,
                        } => write!(f, "{string}, "),
                        _ => write!(f, "{i}, "),
                    };
                }
                match x {
                    Some(x) => match x {
                        Value::String {
                            string,
                            printables: _,
                        } => write!(f, "{string}"),
                        _ => write!(f, "{x}"),
                    },
                    None => Ok(()),
                };
                write!(f, "]")
            }
            Value::None => write!(f, "nada"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Token {
    pub tt: TokenType,
    pub lexeme: String,
    pub literal: Option<Value>,
    pub line: usize,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Token: {}, Lexeme: {}, Literal: {:?}, Line: {}",
            self.tt, self.lexeme, self.literal, self.line
        )
    }
}
