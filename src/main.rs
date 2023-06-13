use error::KlangError;
mod compiling;
use compiling::{compiler, vm};
mod error;
mod interpreter;
use interpreter::{parser, scanner};
#[macro_use]
extern crate rocket;
use rocket::response::content::RawHtml;

#[get("/")]
fn index() -> RawHtml<&'static str> {
    RawHtml(include_str!("temp.html"))
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![index, run])
}

#[post("/", data = "<source>")]
fn run(source: String) -> String {
    let relfilename = "playground.klang";
    let mut scanner = scanner::Scanner::new(&source, relfilename);
    let mut parser = parser::Parser::new(scanner.scan_tokens(), relfilename);
    let ast = parser.parse();
    let chunk = compiler::Chunk::new(compiler::compile(ast));
    let mut vm = vm::VM::new(chunk, relfilename);
    vm.run()
}
