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

#[shuttle_runtime::main]
async fn rocket() -> shuttle_rocket::ShuttleRocket {
    Ok(rocket::build().mount("/", routes![index, run]).into())
}

#[post("/", data = "<source>")]
fn run(source: String) -> String {
    let mut scanner = scanner::Scanner::new(&source);
    let tokens = match scanner.scan_tokens() {
        Ok(t) => t,
        Err(err) => return err,
    };
    let mut parser = parser::Parser::new(tokens);
    let ast = match parser.parse() {
        Ok(t) => t,
        Err(err) => {
            return err;
        }
    };
    let chunk = compiler::Chunk::new(compiler::compile(ast));
    let mut vm = vm::VM::new(chunk);
    vm.run()
}
