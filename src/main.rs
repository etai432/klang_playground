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
    RawHtml(include_str!("playground.html"))
}

#[get("/info")]
fn info() -> RawHtml<&'static str> {
    RawHtml(include_str!("info.html"))
}

#[launch]
async fn rocket() -> _ {
    rocket::build().mount("/", routes![index, run, info]).into()
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
    let vm = vm::VM::new(chunk);
    match run_vm(vm) {
        Ok(s) => s,
        Err(s) => s,
    }
}

fn run_vm(mut vm: vm::VM) -> Result<String, String> {
    let mut output = String::new();
    let mut jumps = 0;

    while vm.index < vm.chunk.code.len() as i32 {
        match vm.once(&mut jumps) {
            Ok(s) => output.push_str(&s),
            Err(s) => {
                return Err(error::KlangError::error(
                    KlangError::RuntimeError,
                    s.as_str(),
                    0,
                ))
            }
        }
        vm.index += 1;
        if jumps > 10000 {
            return Err(error::KlangError::error(
                KlangError::RuntimeError,
                "infinite loop detected",
                0,
            ));
        }
    }

    Ok(output)
}
