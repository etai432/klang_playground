#![allow(unused)]

use super::expr::Expr;
use super::stmt::Stmt;
use crate::error::KlangError;
use crate::scanner::Scanner;
use crate::scanner::{Token, TokenType, Value};

pub struct Parser {
    pub tokens: Vec<Token>,
    current: usize,
}
impl Parser {
    pub fn new(tokens: Vec<Token>) -> Parser {
        Parser { tokens, current: 0 }
    }
    pub fn parse(&mut self) -> Result<Vec<Stmt>, String> {
        let mut statements: Vec<Stmt> = Vec::new();
        while !self.is_at_end() {
            statements.push(match self.declaration() {
                Ok(t) => t,
                Err(s) => return Err(s),
            });
        }
        Ok(statements)
    }
    fn declaration(&mut self) -> Result<Stmt, String> {
        if self.match_tokens(&[TokenType::Let]) {
            self.var_decl()
        } else if self.match_tokens(&[TokenType::Fn]) {
            self.fn_decl()
        } else {
            self.statement()
        }
    }

    fn fn_decl(&mut self) -> Result<Stmt, String> {
        let return_t = self.previous();
        let name = match self.consume(TokenType::Identifier, "must have a function name") {
            Ok(t) => t,
            Err(s) => return Err(s),
        };
        if self.match_tokens(&[TokenType::LeftParen]) {
            if self.match_tokens(&[TokenType::RightParen]) {
                return Ok(Stmt::Fn {
                    name,
                    params: Vec::new(),
                    body: Box::new(match self.block() {
                        Ok(t) => t,
                        Err(s) => return Err(s),
                    }),
                });
            }
            let mut vec: Vec<Token> = Vec::new();
            let mut iden =
                match self.consume(TokenType::Identifier, "argument must be an identifier") {
                    Ok(t) => t,
                    Err(s) => return Err(s),
                };
            vec.push(iden);
            while self.match_tokens(&[TokenType::Comma]) {
                iden = match self.consume(TokenType::Identifier, "parameter must be an identifier")
                {
                    Ok(t) => t,
                    Err(s) => return Err(s),
                };
                vec.push(iden);
            }
            match self.consume(TokenType::RightParen, "gotta close the call dude") {
                Ok(t) => t,
                Err(s) => return Err(s),
            };
            return Ok(Stmt::Fn {
                name,
                params: vec,
                body: Box::new(match self.block() {
                    Ok(t) => t,
                    Err(s) => return Err(s),
                }),
            });
        }
        Err(self.error("not possible!"))
    }
    fn var_decl(&mut self) -> Result<Stmt, String> {
        let name = match self.consume(TokenType::Identifier, "must define a variable name") {
            Ok(t) => t,
            Err(s) => return Err(s),
        };
        if self.match_tokens(&[TokenType::Equal]) {
            let value = self.logical();
            self.consume(TokenType::Semicolon, "missing ; at the end of the line");
            return Ok(Stmt::Var {
                name,
                value: Some(match value {
                    Ok(t) => t,
                    Err(s) => return Err(s),
                }),
            });
        }
        match self.consume(TokenType::Semicolon, "missing ; at the end of the line") {
            Ok(t) => t,
            Err(s) => return Err(s),
        };
        Ok(Stmt::Var { name, value: None })
    }

    fn statement(&mut self) -> Result<Stmt, String> {
        if self.match_tokens(&[TokenType::Print]) {
            self.print_stmt()
        } else if self.check(TokenType::LeftBrace) {
            self.block()
        } else if self.match_tokens(&[TokenType::If]) {
            self.if_stmt()
        } else if self.match_tokens(&[TokenType::While]) {
            self.while_stmt()
        } else if self.match_tokens(&[TokenType::For]) {
            self.for_stmt()
        } else if self.match_tokens(&[TokenType::Return]) {
            self.return_stmt()
        } else {
            self.expr_stmt()
        }
    }

    fn return_stmt(&mut self) -> Result<Stmt, String> {
        if self.match_tokens(&[TokenType::Semicolon]) {
            return Ok(Stmt::Return(None, self.previous().line));
        }
        let value = self.logical();
        match self.consume(TokenType::Semicolon, "missing ; at the end of lien") {
            Ok(t) => t,
            Err(s) => return Err(s),
        };
        Ok(Stmt::Return(
            Some(match value {
                Ok(t) => t,
                Err(s) => return Err(s),
            }),
            self.previous().line,
        ))
    }

    fn for_stmt(&mut self) -> Result<Stmt, String> {
        let identifier = match self.consume(TokenType::Identifier, "missing identifier 8=D") {
            Ok(t) => t,
            Err(s) => return Err(s),
        };
        let line = self.previous().line;
        match self.consume(TokenType::In, "missing in") {
            Ok(t) => t,
            Err(s) => return Err(s),
        };
        let iterable = match self.range() {
            Ok(t) => t,
            Err(s) => return Err(s),
        };
        let block = Box::new(match self.block() {
            Ok(t) => t,
            Err(s) => return Err(s),
        });
        Ok(Stmt::For {
            identifier,
            iterable,
            block,
            line,
        })
    }

    fn if_stmt(&mut self) -> Result<Stmt, String> {
        let condition = match self.logical() {
            Ok(t) => t,
            Err(s) => return Err(s),
        };
        let start = self.previous().line;
        let block = Box::new(match self.block() {
            Ok(t) => t,
            Err(s) => return Err(s),
        });
        if self.match_tokens(&[TokenType::Else]) {
            let end = self.previous().line;
            let elseblock = Some(Box::new(match self.block() {
                Ok(t) => t,
                Err(s) => return Err(s),
            }));
            return Ok(Stmt::If {
                condition,
                block,
                elseblock,
                lines: (start, Some(end)),
            });
        }
        Ok(Stmt::If {
            condition,
            block,
            elseblock: None,
            lines: (start, None),
        })
    }

    fn while_stmt(&mut self) -> Result<Stmt, String> {
        let condition = match self.logical() {
            Ok(t) => t,
            Err(s) => return Err(s),
        };
        let line = self.previous().line;
        let block = match self.block() {
            Ok(t) => t,
            Err(s) => return Err(s),
        };

        Ok(Stmt::While {
            condition,
            block: Box::new(block),
            line,
        })
    }

    fn block(&mut self) -> Result<Stmt, String> {
        match self.consume(TokenType::LeftBrace, "must start block with a {") {
            Ok(t) => t,
            Err(s) => return Err(s),
        };
        let start = self.previous().line;
        let mut statements: Vec<Stmt> = Vec::new();
        while !self.is_at_end() && !self.check(TokenType::RightBrace) {
            statements.push(match self.declaration() {
                Ok(t) => t,
                Err(s) => return Err(s),
            });
        }
        match self.consume(TokenType::RightBrace, "must end block with a }") {
            Ok(t) => t,
            Err(s) => return Err(s),
        };
        Ok(Stmt::Block(statements, (start, self.previous().line)))
    }

    fn print_stmt(&mut self) -> Result<Stmt, String> {
        match self.consume(
            TokenType::LeftParen,
            "gotta put ( after a print yk how it is..",
        ) {
            Ok(t) => t,
            Err(s) => return Err(s),
        };
        let stmt = Stmt::Print(
            match self.primary() {
                Ok(Expr::Literal(Value::String { string, printables }, _)) => {
                    Value::String { string, printables }
                }
                Err(e) => return Err(e),
                _ => {
                    return Err(self.error("can only print strings"));
                }
            },
            self.peek().line,
        );
        match self.consume(
            TokenType::RightParen,
            "gotta put ) at the end of a print yk how it is..",
        ) {
            Ok(t) => t,
            Err(s) => return Err(s),
        };
        match self.consume(TokenType::Semicolon, "missing ; at the end of the line") {
            Ok(t) => t,
            Err(s) => return Err(s),
        };
        Ok(stmt)
    }
    fn expr_stmt(&mut self) -> Result<Stmt, String> {
        let stmt = Stmt::Expression(match self.assignment() {
            Ok(t) => t,
            Err(s) => return Err(s),
        });
        match self.consume(TokenType::Semicolon, "missing ; at the end of the line") {
            Ok(t) => t,
            Err(s) => return Err(s),
        };
        Ok(stmt)
    }

    pub fn assignment(&mut self) -> Result<Expr, String> {
        let identifier = match self.logical() {
            Ok(t) => t,
            Err(s) => return Err(s),
        };
        if self.match_tokens(&[TokenType::Equal]) {
            let value = match self.logical() {
                Ok(t) => t,
                Err(s) => return Err(s),
            };
            match identifier {
                Expr::Variable(name) => {
                    return Ok(Expr::Assign {
                        name,
                        value: Box::new(value),
                    })
                }
                _ => return Err(self.error("cannot assign to a non variable")),
            }
        }
        Ok(identifier)
    }

    pub fn logical(&mut self) -> Result<Expr, String> {
        let left: Expr = match self.equality() {
            Ok(t) => t,
            Err(s) => return Err(s),
        };
        if self.match_tokens(&[TokenType::And, TokenType::Or]) {
            let operator = self.previous();
            let right: Expr = match self.logical() {
                Ok(t) => t,
                Err(s) => return Err(s),
            };
            return Ok(Expr::Binary {
                left: Box::new(left),
                operator,
                right: Box::new(right),
            });
        }
        Ok(left)
    }

    fn equality(&mut self) -> Result<Expr, String> {
        let left: Expr = match self.comparison() {
            Ok(t) => t,
            Err(s) => return Err(s),
        };
        if self.match_tokens(&[TokenType::BangEqual, TokenType::EqualEqual]) {
            let operator = self.previous();
            let right: Expr = match self.comparison() {
                Ok(t) => t,
                Err(s) => return Err(s),
            };
            return Ok(Expr::Binary {
                left: Box::new(left),
                operator,
                right: Box::new(right),
            });
        }
        Ok(left)
    }
    fn comparison(&mut self) -> Result<Expr, String> {
        let left: Expr = match self.range() {
            Ok(t) => t,
            Err(s) => return Err(s),
        };
        if self.match_tokens(&[
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::Less,
            TokenType::LessEqual,
        ]) {
            let operator = self.previous();
            let right: Expr = match self.term() {
                Ok(t) => t,
                Err(s) => return Err(s),
            };
            return Ok(Expr::Binary {
                left: Box::new(left),
                operator,
                right: Box::new(right),
            });
        }
        Ok(left)
    }
    pub fn range(&mut self) -> Result<Expr, String> {
        let start = match self.term() {
            Ok(t) => t,
            Err(s) => return Err(s),
        };
        if self.match_tokens(&[TokenType::Range]) {
            let end = match self.term() {
                Ok(t) => t,
                Err(s) => return Err(s),
            };
            if self.match_tokens(&[TokenType::Range]) {
                let step = match self.term() {
                    Ok(t) => t,
                    Err(s) => return Err(s),
                };
                return Ok(Expr::Range {
                    min: Box::new(start),
                    max: Box::new(end),
                    step: Some(Box::new(step)),
                    line: self.previous().line,
                });
            }
            return Ok(Expr::Range {
                min: Box::new(start),
                max: Box::new(end),
                step: None,
                line: self.previous().line,
            });
        }
        Ok(start)
    }
    fn term(&mut self) -> Result<Expr, String> {
        let left: Expr = match self.factor() {
            Ok(t) => t,
            Err(s) => return Err(s),
        };
        if self.match_tokens(&[TokenType::Plus, TokenType::Minus]) {
            let operator = self.previous();
            let right: Expr = match self.term() {
                Ok(t) => t,
                Err(s) => return Err(s),
            };
            return Ok(Expr::Binary {
                left: Box::new(left),
                operator,
                right: Box::new(right),
            });
        }
        Ok(left)
    }
    fn factor(&mut self) -> Result<Expr, String> {
        let left: Expr = match self.unary() {
            Ok(t) => t,
            Err(s) => return Err(s),
        };
        if self.match_tokens(&[TokenType::Slash, TokenType::Star, TokenType::Modulo]) {
            let operator = self.previous();
            let right: Expr = match self.factor() {
                Ok(t) => t,
                Err(s) => return Err(s),
            };
            return Ok(Expr::Binary {
                left: Box::new(left),
                operator,
                right: Box::new(right),
            });
        }
        Ok(left)
    }
    fn unary(&mut self) -> Result<Expr, String> {
        if self.match_tokens(&[TokenType::Bang, TokenType::Minus]) {
            let operator = self.previous();
            let e = match self.primary() {
                Ok(t) => t,
                Err(s) => return Err(s),
            };
            return Ok(Expr::Unary {
                operator,
                expression: Box::new(e),
            });
        }
        self.call(false)
    }

    fn call(&mut self, native: bool) -> Result<Expr, String> {
        let expr = match self.primary() {
            Ok(t) => t,
            Err(s) => return Err(s),
        };
        if self.match_tokens(&[TokenType::LeftParen]) {
            if !matches!(expr, Expr::Variable(_)) {
                return Err(self.error("sir were you trying to call a function USING AN INTEGER?"));
            }
            if self.match_tokens(&[TokenType::RightParen]) {
                return Ok(Expr::Call {
                    callee: Box::new(expr),
                    arguments: Vec::new(),
                    native,
                });
            }
            let mut vec: Vec<Expr> = Vec::new();
            vec.push(match self.logical() {
                Ok(t) => t,
                Err(s) => return Err(s),
            });
            while self.match_tokens(&[TokenType::Comma]) {
                vec.push(match self.logical() {
                    Ok(t) => t,
                    Err(s) => return Err(s),
                });
            }
            match self.consume(TokenType::RightParen, "gotta close the call dude") {
                Ok(t) => t,
                Err(s) => return Err(s),
            };
            return Ok(Expr::Call {
                callee: Box::new(expr),
                arguments: vec,
                native,
            });
        }
        Ok(expr)
    }

    fn primary(&mut self) -> Result<Expr, String> {
        if self.match_tokens(&[TokenType::Bool]) {
            if self.previous().lexeme == "true" {
                return Ok(Expr::Literal(Value::Bool(true), self.previous().line));
            } else {
                return Ok(Expr::Literal(Value::Bool(false), self.previous().line));
            }
        }
        if self.match_tokens(&[TokenType::LeftSquare]) {
            let mut vec: Vec<Expr> = Vec::new();
            if self.match_tokens(&[TokenType::RightSquare]) {
                return Ok(Expr::Vec(vec));
            }
            vec.push(match self.logical() {
                Ok(t) => t,
                Err(s) => return Err(s),
            });
            while self.match_tokens(&[TokenType::Comma]) {
                vec.push(match self.logical() {
                    Ok(t) => t,
                    Err(s) => return Err(s),
                });
            }
            match self.consume(TokenType::RightSquare, "gotta close the vec") {
                Ok(t) => t,
                Err(s) => return Err(s),
            };
            return Ok(Expr::Vec(vec));
        }
        if self.match_tokens(&[TokenType::String]) {
            let string = self.previous().lexeme;
            let mut printables_t: Vec<Vec<Token>> = Vec::new();
            while self.match_tokens(&[TokenType::Printable]) {
                let lexeme = self.previous().lexeme;
                if lexeme.contains('"') {
                    return Err(self
                        .error("why would you use a string inside a string?? are you retarded??"));
                }
                let mut s = Scanner::new(&lexeme);
                let mut s1 = match s.scan_tokens() {
                    Ok(s) => s,
                    Err(err) => return Err(err),
                };
                s1.pop();
                printables_t.push(s1);
                self.match_tokens(&[TokenType::Comma]);
            }
            let mut printables: Vec<Expr> = Vec::new();
            for i in printables_t {
                self.tokens.splice(self.current..self.current, i);
                printables.push(match self.logical() {
                    Ok(t) => t,
                    Err(s) => return Err(s),
                });
            }
            return Ok(Expr::Literal(
                Value::String { string, printables },
                self.previous().line,
            ));
        }

        if self.match_tokens(&[TokenType::Int, TokenType::Float]) {
            return Ok(Expr::Literal(
                self.previous().literal.unwrap(),
                self.previous().line,
            ));
        }
        if self.match_tokens(&[TokenType::LeftParen]) {
            let expression = match self.logical() {
                Ok(t) => t,
                Err(s) => return Err(s),
            };
            match self.consume(
                TokenType::RightParen,
                "expected \")\" after expression u piece of shit",
            ) {
                Ok(t) => t,
                Err(s) => return Err(s),
            };
            return Ok(Expr::Grouping(Box::new(expression)));
        }
        if self.match_tokens(&[TokenType::NativeCall]) {
            return self.call(true);
        }
        if self.match_tokens(&[TokenType::Identifier]) {
            return Ok(Expr::Variable(self.previous()));
        }
        Err(self.error(&format!("expected value found {}", self.peek().tt)))
    }

    fn match_tokens(&mut self, types: &[TokenType]) -> bool {
        for &tt in types {
            if self.check(tt) {
                self.advance();
                return true;
            }
        }
        false
    }

    fn check(&self, t_type: TokenType) -> bool {
        if self.is_at_end() {
            return false;
        }
        self.peek().tt == t_type
    }

    fn advance(&mut self) -> Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }
    fn is_at_end(&self) -> bool {
        self.peek().tt == TokenType::Eof
    }
    fn peek(&self) -> Token {
        self.tokens[self.current].clone()
    }
    fn previous(&self) -> Token {
        self.tokens[self.current - 1].clone()
    }
    fn error(&self, msg: &str) -> String {
        KlangError::error(KlangError::ParserError, msg, self.peek().line)
    }
    fn consume(&mut self, t_type: TokenType, msg: &str) -> Result<Token, String> {
        if self.peek().tt == t_type {
            return Ok(self.advance());
        }
        Err(self.error(msg))
    }
}
