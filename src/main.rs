use error::KlangError;
mod compiling;
use compiling::{compiler, vm};
mod error;
mod interpreter;
use interpreter::{parser, scanner};

#[macro_use]
extern crate rocket;

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![index])
}

fn run_file(source: String) {
    let relfilename = "playground.klang";
    let mut scanner = scanner::Scanner::new(&source, relfilename);
    let mut parser = parser::Parser::new(scanner.scan_tokens(), relfilename);
    let ast = parser.parse();
    let chunk = compiler::Chunk::new(compiler::compile(ast));
    let mut vm = vm::VM::new(chunk, relfilename);
    let output = vm.run();
}
