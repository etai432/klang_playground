#![allow(unused)]
use error::KlangError;
mod compiling;
use compiling::{compiler, vm};
mod error;
mod interpreter;
use interpreter::{parser, scanner};
use rocket::response::Responder;
use serde_json::{Result, Value};
#[macro_use]
extern crate rocket;
use rocket::http::ContentType;
use rocket::response::content::RawHtml;

#[get("/")]
fn index() -> RawHtml<&'static str> {
    RawHtml(include_str!("temp.html"))
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
