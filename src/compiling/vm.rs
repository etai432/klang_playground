use super::{
    compiler::Chunk,
    native::{create_natives, NativeFn},
    opcode::OpCode,
};
use crate::interpreter::scanner::{TokenType, Value};
use crate::KlangError;
use std::collections::HashMap;
pub struct VM {
    pub chunk: Chunk,
    pub global: Scope,
    pub index: i32,
    pub functions: HashMap<String, (Vec<OpCode>, Vec<String>)>,
    pub native: Vec<NativeFn>,
}

impl VM {
    pub fn new(chunk: Chunk) -> VM {
        VM {
            chunk,
            global: Scope::new(),
            index: 0,
            functions: HashMap::new(),
            native: create_natives(),
        }
    }
    pub fn run(&mut self) -> String {
        let mut output = String::new();
        //executes the code on the chunk
        while self.index < self.chunk.code.len() as i32 {
            output += match self.once() {
                Ok(s) => s,
                Err(s) => return s,
            }
            .as_str();
            self.index += 1;
        }
        output
    }
    pub fn once(&mut self) -> Result<String, String> {
        // println!("{:?}", self.global);
        match self.chunk.code[self.index as usize].clone() {
            OpCode::Constant(x) => self.push(x),
            OpCode::Store(x) => self.set_var(x),
            OpCode::Load(x) => {
                let var = match VM::get_var(&x, &mut self.global).0 {
                    Some(x) => x,
                    None => {
                        return Err(self.error(format!("variable \"{x}\" do not exist").as_str()))
                    }
                };
                self.push(var);
            }
            OpCode::Add => {
                if let Some(s) = self.bin_op(TokenType::Plus) {
                    return Err(s);
                }
            }
            OpCode::Subtract => {
                if let Some(s) = self.bin_op(TokenType::Minus) {
                    return Err(s);
                }
            }
            OpCode::Multiply => {
                if let Some(s) = self.bin_op(TokenType::Star) {
                    return Err(s);
                }
            }
            OpCode::Divide => {
                if let Some(s) = self.bin_op(TokenType::Slash) {
                    return Err(s);
                }
            }
            OpCode::Modulo => {
                if let Some(s) = self.bin_op(TokenType::Modulo) {
                    return Err(s);
                }
            }
            OpCode::EqualEqual => {
                if let Some(s) = self.bin_op(TokenType::EqualEqual) {
                    return Err(s);
                }
            }
            OpCode::NotEqual => {
                if let Some(s) = self.bin_op(TokenType::BangEqual) {
                    return Err(s);
                }
            }
            OpCode::Less => {
                if let Some(s) = self.bin_op(TokenType::Less) {
                    return Err(s);
                }
            }
            OpCode::LessEqual => {
                if let Some(s) = self.bin_op(TokenType::LessEqual) {
                    return Err(s);
                }
            }
            OpCode::Greater => {
                if let Some(s) = self.bin_op(TokenType::Greater) {
                    return Err(s);
                }
            }
            OpCode::GreaterEqual => {
                if let Some(s) = self.bin_op(TokenType::GreaterEqual) {
                    return Err(s);
                }
            }
            OpCode::LogicalAnd => {
                if let Some(s) = self.bin_op(TokenType::And) {
                    return Err(s);
                }
            }
            OpCode::LogicalOr => {
                if let Some(s) = self.bin_op(TokenType::Or) {
                    return Err(s);
                }
            }
            OpCode::LogicalNot => {
                if let Some(s) = self.un_op(TokenType::Bang) {
                    return Err(s);
                }
            }
            OpCode::Negate => {
                if let Some(s) = self.un_op(TokenType::Minus) {
                    return Err(s);
                }
            }
            OpCode::Jump(x) => {
                if self.index + x > self.chunk.code.len() as i32 {
                    return Err(self.error("cannot jump out of bounds like ur dad jumped out of the 50th story window bozo"));
                }
                self.index += x;
            }
            OpCode::JumpIf(x, t) => {
                if t {
                    if let Value::Bool(true) = match self.pop() {
                        Some(x) => x,
                        None => return Err(self.error("stack overflow (cant pop an empty stack)")),
                    } {
                        if self.index + x > self.chunk.code.len() as i32 {
                            return Err(self.error("cannot jump out of bounds like ur dad jumped out of the 50th story window bozo"));
                        }
                        self.index += x;
                    }
                } else if let Ok(Value::Bool(true)) = self.top() {
                    if self.index + x > self.chunk.code.len() as i32 {
                        return Err(self.error("cannot jump out of bounds like ur dad jumped out of the 50th story window bozo"));
                    }
                    self.index += x;
                }
            }
            OpCode::Call(x) => {
                self.call(x, self.index);
            }
            OpCode::NativeCall(x, y) => {
                if let Some(s) = self.native_call(x, y) {
                    return Err(s);
                }
            }
            OpCode::Print => return self.print(),
            OpCode::Range(x) => {
                if let Some(s) = self.range(x) {
                    return Err(s);
                }
            }

            OpCode::Scope => self.create_inner(),
            OpCode::EndScope => self.close_inner(),
            OpCode::EndFn => {}
            OpCode::Return(x) => {
                if x {
                    let val = match self.pop() {
                        Some(x) => x,
                        None => Value::None,
                    };
                    let mut counter = 1;
                    while !matches!(self.chunk.code[self.index as usize], OpCode::EndFn) {
                        self.index += 1;
                        if matches!(self.chunk.code[self.index as usize], OpCode::Scope) {
                            counter -= 1;
                        }
                        if matches!(self.chunk.code[self.index as usize], OpCode::EndScope) {
                            counter += 1;
                        }
                    }
                    for _ in 0..counter {
                        self.close_inner()
                    }
                    self.push(val);
                } else {
                    let mut counter = 1;
                    while !matches!(self.chunk.code[self.index as usize], OpCode::EndFn) {
                        self.index += 1;
                        if matches!(self.chunk.code[self.index as usize], OpCode::Scope) {
                            counter -= 1;
                        }
                        if matches!(self.chunk.code[self.index as usize], OpCode::EndScope) {
                            counter += 1;
                        }
                    }
                    for _ in 0..counter {
                        self.close_inner()
                    }
                }
            }
            OpCode::For => {
                if let Some(s) = self.for_loop() {
                    return Err(s);
                }
            }
            OpCode::Fn => self.function(),
            OpCode::Iterable(x) => {
                if let Some(s) = self.iterable(x) {
                    return Err(s);
                }
            }
            OpCode::Eof => {}
        }
        Ok(String::new())
    }
    fn iterable(&mut self, x: i32) -> Option<String> {
        let mut vec: Vec<Value> = Vec::with_capacity(x as usize);
        for _ in 0..x {
            vec.push(match self.pop() {
                Some(x) => x,
                None => {
                    return Some(self.error("stack overflow (cant pop an empty stack)"));
                }
            });
        }
        let mut vec1: Vec<Value> = Vec::with_capacity(x as usize);
        for i in vec.into_iter().rev() {
            vec1.push(i);
        }
        self.push(Value::Vec(vec1));
        None
    }
    fn function(&mut self) {
        self.index += 1; //consume fn
        let mut args: Vec<String> = Vec::new();
        while match self.chunk.code[self.index as usize].clone() {
            OpCode::Store(x) => {
                args.push(x);
                true
            }
            _ => false,
        } {
            self.index += 1; //consume arg
        }
        let mut bytes: Vec<OpCode> = Vec::new();
        self.index += 1;
        let mut counter = 1;
        while counter != 0 {
            bytes.push(self.chunk.code[self.index as usize].clone());
            self.index += 1;
            if matches!(self.chunk.code[self.index as usize], OpCode::EndScope) {
                counter -= 1;
            }
            if matches!(self.chunk.code[self.index as usize], OpCode::Scope) {
                counter += 1;
            }
        }
        self.index += 1;
        match self.chunk.code[self.index as usize].clone() {
            OpCode::Store(x) => self.functions.insert(x, (bytes, args)),
            _ => {
                self.error("ksang made a little oopsy");
                panic!();
            }
        };
    }
    fn range(&mut self, cstep: bool) -> Option<String> {
        if cstep {
            let step = match self.pop() {
                Some(Value::Number(x)) => x,
                _ => return Some(self.error("step is not a number")),
            };
            let end = match self.pop() {
                Some(Value::Number(x)) => x,
                _ => return Some(self.error("end is not a number")),
            };
            let start = match self.pop() {
                Some(Value::Number(x)) => x,
                _ => return Some(self.error("start is not a number")),
            };
            let mut vec: Vec<Value> = Vec::new();
            for i in (start as usize..end as usize).step_by(step as usize) {
                vec.push(Value::Number(i as f64));
            }
            self.push(Value::Vec(vec));
            None
        } else {
            let end = match self.pop() {
                Some(Value::Number(x)) => x,
                _ => return Some(self.error("end is not a number")),
            };
            let start = match self.pop() {
                Some(Value::Number(x)) => x,
                _ => return Some(self.error("start is not a number")),
            };
            let mut vec: Vec<Value> = Vec::new();
            for i in start as usize..end as usize {
                vec.push(Value::Number(i as f64));
            }
            self.push(Value::Vec(vec));
            None
        }
    }
    fn for_loop(&mut self) -> Option<String> {
        let range = match self.pop() {
            Some(x) => x,
            None => return Some(self.error("invalid witewabwe!")),
        };
        let mut vector = match range {
            Value::Vec(x) => x,
            _ => return Some(self.error("invalid witewabwe!")),
        };
        self.index += 1;
        self.create_inner();
        if vector.is_empty() {
            self.push(Value::Bool(true));
            self.push(Value::None);
            return None;
        } else {
            self.push(Value::Bool(false));
            self.push(vector.remove(0));
        }
        let mut scope: &mut Scope = &mut self.global;
        while scope.inner.as_mut().unwrap().inner.is_some() {
            scope = scope.inner.as_mut().unwrap();
        }
        scope.stack.push(Value::Vec(vector));
        None
    }
    fn print(&mut self) -> Result<String, String> {
        let mut print = match self.pop() {
            Some(Value::String {
                string,
                printables: _,
            }) => string,
            _ => {
                panic!()
            }
        };
        for _ in 0..self.count_braces(print.as_str()) {
            let repl = match self.pop() {
                Some(Value::String {
                    string,
                    printables: _,
                }) => string,
                Some(Value::Number(x)) => x.to_string(),
                Some(Value::Bool(x)) => x.to_string(),
                Some(Value::Vec(x)) => format!("{}", Value::Vec(x)),
                Some(Value::None) => "None".to_string(),
                None => {
                    return Err(self.error("Stack overflow (cant pop an empty stack)"));
                }
            };
            print = self.replace_last_braces(print.as_str(), repl.as_str());
        }
        Ok(print + "\n")
    }
    fn count_braces(&self, string: &str) -> usize {
        let mut count = 0;
        let mut braces = 0;
        for c in string.chars() {
            match c {
                '{' => braces += 1,
                '}' => {
                    if braces > 0 {
                        braces -= 1;
                        if braces == 0 {
                            count += 1;
                        }
                    }
                }
                _ => {}
            }
        }
        count
    }
    fn replace_last_braces(&self, string: &str, replacement: &str) -> String {
        if let Some((start, _)) = string.rmatch_indices("{}").next() {
            let mut modified = String::with_capacity(string.len() - 2 + replacement.len());
            modified.push_str(&string[..start]);
            modified.push_str(replacement);
            modified.push_str(&string[start + 2..]);
            modified
        } else {
            String::from(string)
        }
    }
    fn get_var(name: &str, scope: &mut Scope) -> (Option<Value>, bool) {
        //gets a variable from the most inner scope, if its not there searches on the outer scopes, return true when found the variable
        if scope.inner.is_some() {
            let i = VM::get_var(name, scope.inner.as_mut().unwrap());
            if !i.1 {
                return match scope.callframe.get(name) {
                    Some(val) => (Some(val.clone()), true),
                    None => (None, false),
                };
            }
            return i;
        }
        match scope.callframe.get(name) {
            Some(val) => (Some(val.clone()), true),
            None => (None, false),
        }
    }
    fn set_var(&mut self, name: String) {
        //sets a variable in the most inner scope, to the top value of the stack
        let pop = match self.pop() {
            Some(x) => x,
            None => Value::None,
        };
        let mut scope: &mut Scope = &mut self.global;
        while scope.inner.is_some() {
            if let std::collections::hash_map::Entry::Occupied(mut e) =
                scope.callframe.entry(name.clone())
            {
                e.insert(pop);
                return;
            }
            scope = scope.inner.as_mut().unwrap();
        }
        scope.callframe.insert(name, pop);
    }
    fn set_var_inner(&mut self, name: String) {
        //sets a variable in the most inner scope, to the top value of the stack
        let pop = match self.pop() {
            Some(x) => x,
            None => Value::None,
        };
        let mut scope: &mut Scope = &mut self.global;
        while scope.inner.is_some() {
            scope = scope.inner.as_mut().unwrap();
        }
        scope.callframe.insert(name, pop);
    }
    fn create_inner(&mut self) {
        let mut scope: &mut Scope = &mut self.global;
        while scope.inner.is_some() {
            scope = scope.inner.as_mut().unwrap();
        }
        scope.inner = Some(Box::new(Scope::new()));
    }
    fn close_inner(&mut self) {
        let mut scope: &mut Scope = &mut self.global;
        while scope.inner.as_mut().unwrap().inner.is_some() {
            scope = scope.inner.as_mut().unwrap();
        }
        scope.inner = None;
    }
    fn error(&self, msg: &str) -> String {
        KlangError::error(
            KlangError::RuntimeError,
            msg,
            self.chunk.lines[self.index as usize],
        )
    }

    fn bin_op(&mut self, operation: TokenType) -> Option<String> {
        let pop2 = match self.pop2() {
            Ok(a) => a,
            Err(s) => return Some(s),
        };
        self.push(match operation {
            TokenType::Plus => match pop2 {
                (Value::Number(x), Value::Number(y)) => Value::Number(x + y),
                _ => return Some(self.error("can only add numbers")),
            },
            TokenType::Minus => match pop2 {
                (Value::Number(x), Value::Number(y)) => Value::Number(y - x),
                _ => return Some(self.error("can only subtract numbers")),
            },
            TokenType::Star => match pop2 {
                (Value::Number(x), Value::Number(y)) => Value::Number(x * y),
                _ => return Some(self.error("can only multiply numbers")),
            },
            TokenType::Slash => match pop2 {
                (Value::Number(x), Value::Number(y)) => {
                    if x == 0.0 {
                        return Some(self.error("division by zero"));
                    }
                    Value::Number(y / x)
                }
                _ => return Some(self.error("can only divide numbers")),
            },
            TokenType::Modulo => match pop2 {
                (Value::Number(x), Value::Number(y)) => {
                    if x == 0.0 {
                        return Some(self.error("no modulo by zero"));
                    }
                    Value::Number(y % x)
                }
                _ => {
                    return Some(
                        self.error("can only use the modulo operator on numbers, dickfuck"),
                    )
                }
            },
            TokenType::EqualEqual => match pop2 {
                (Value::Number(x), Value::Number(y)) => Value::Bool(x == y),
                (Value::Bool(x), Value::Bool(y)) => Value::Bool(x == y),
                (Value::String { string: x, .. }, Value::String { string: y, .. }) => {
                    Value::Bool(x == y)
                }
                _ => Value::Bool(false),
            },
            TokenType::BangEqual => match pop2 {
                (Value::Number(x), Value::Number(y)) => Value::Bool(x != y),
                (Value::Bool(x), Value::Bool(y)) => Value::Bool(x != y),
                (Value::String { string: x, .. }, Value::String { string: y, .. }) => {
                    Value::Bool(x != y)
                }
                _ => Value::Bool(true),
            },
            TokenType::Less => match pop2 {
                (Value::Number(x), Value::Number(y)) => Value::Bool(x > y),
                _ => return Some(self.error("can only compare numbers")),
            },
            TokenType::LessEqual => match pop2 {
                (Value::Number(x), Value::Number(y)) => Value::Bool(x >= y),
                _ => return Some(self.error("can only compare numbers")),
            },
            TokenType::Greater => match pop2 {
                (Value::Number(x), Value::Number(y)) => Value::Bool(x < y),
                _ => return Some(self.error("can only compare numbers")),
            },
            TokenType::GreaterEqual => match pop2 {
                (Value::Number(x), Value::Number(y)) => Value::Bool(x <= y),
                _ => return Some(self.error("can only compare numbers")),
            },
            TokenType::And => match pop2 {
                (Value::Bool(x), Value::Bool(y)) => Value::Bool(x && y),
                _ => return Some(self.error("can only perform logical AND on bool values")),
            },
            TokenType::Or => match pop2 {
                (Value::Bool(x), Value::Bool(y)) => Value::Bool(x || y),
                _ => return Some(self.error("can only perform logical OR on bool values")),
            },
            _ => return Some(self.error("unsupported binary operation")),
        });
        None
    }
    fn un_op(&mut self, operation: TokenType) -> Option<String> {
        let pop = match self.pop() {
            Some(x) => x,
            None => return Some(self.error("stack overflow (cant pop an empty stack)")),
        };
        self.push(match operation {
            TokenType::Bang => match pop {
                Value::Bool(x) => Value::Bool(!x),
                _ => return Some(self.error("can only use ! on bools")),
            },
            TokenType::Minus => match pop {
                Value::Number(x) => Value::Number(-x),
                _ => return Some(self.error("can only use minus on ints and floats")),
            },
            _ => return Some(self.error("unsupported unary operation")),
        });
        None
    }
    fn call(&mut self, callee: String, index: i32) -> Option<String> {
        let fun = match self.functions.remove(&callee) {
            Some(x) => x,
            None => return Some(self.error("please call a real function next time stupid ass mf")),
        };
        self.functions.insert(callee.clone(), fun.clone());
        self.create_inner();
        for i in fun.1.into_iter().rev() {
            let mut scope: &mut Scope = &mut self.global;
            while scope.inner.as_mut().unwrap().inner.is_some() {
                scope = scope.inner.as_mut().unwrap();
            }
            let pop = match scope.stack.pop() {
                Some(x) => x,
                None => return Some(self.error("not enough arguments!")),
            };
            self.push(pop);
            self.set_var_inner(i);
        }
        let mut b = 0;
        for (i, op) in fun.0.into_iter().enumerate() {
            self.chunk.code.insert(index as usize + i + 1, op);
            self.chunk.lines.push(0);
            b = index as usize + i + 2;
        }
        self.chunk.code.insert(b, OpCode::EndFn);
        None
    }
    fn native_call(&mut self, callee: String, arg_num: i32) -> Option<String> {
        let mut found = false;
        for i in 0..self.native.len() {
            if self.native[i].name == callee {
                if arg_num != self.native[i].args {
                    return Some(
                        self.error(
                            format!(
                                "the function takes {} arguments but you only gave it {arg_num}",
                                self.native[i].args
                            )
                            .as_str(),
                        ),
                    );
                }
                let mut args: Vec<Value> = Vec::new();
                for _ in 0..arg_num {
                    args.insert(
                        0,
                        match self.pop() {
                            Some(x) => x,
                            None => return Some(self.error("not enough arguments!")),
                        },
                    )
                }
                if let Some(x) = self.native[i].call(args) {
                    self.push(x)
                }
                found = true;
                break;
            }
        }
        if !found {
            return Some(self.error("not a real native function dumbass"));
        }
        None
    }

    fn pop2(&mut self) -> Result<(Value, Value), String> {
        Ok((
            match self.pop() {
                Some(x) => x,
                None => {
                    return Err(self.error("stack overflow (cant pop an empty stack)"));
                }
            },
            match self.pop() {
                Some(x) => x,
                None => {
                    return Err(self.error("stack overflow (cant pop an empty stack)"));
                }
            },
        ))
    }
    fn pop(&mut self) -> Option<Value> {
        let mut scope: &mut Scope = &mut self.global;
        while scope.inner.is_some() {
            scope = scope.inner.as_mut().unwrap();
        }
        scope.stack.pop()
    }
    fn top(&mut self) -> Result<Value, String> {
        let mut scope: &mut Scope = &mut self.global;
        while scope.inner.is_some() {
            scope = scope.inner.as_mut().unwrap();
        }
        if scope.stack.is_empty() {
            return Err(self.error("stack overflow (cant top an empty stack)"));
        }
        let val = scope.stack.pop().unwrap();
        scope.stack.push(val.clone());
        Ok(val)
    }
    fn push(&mut self, v: Value) {
        let mut scope: &mut Scope = &mut self.global;
        while scope.inner.is_some() {
            scope = scope.inner.as_mut().unwrap();
        }
        scope.stack.push(v);
    }
}

#[derive(Debug, Clone)]
pub struct Scope {
    pub callframe: HashMap<String, Value>,
    pub inner: Option<Box<Scope>>,
    pub stack: Vec<Value>,
}
impl Scope {
    pub fn new() -> Self {
        Self {
            callframe: HashMap::new(),
            inner: None,
            stack: Vec::new(),
        }
    }
}
