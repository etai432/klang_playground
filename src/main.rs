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

#[shuttle_runtime::main]
async fn rocket() -> shuttle_rocket::ShuttleRocket {
    Ok(rocket::build().mount("/", routes![index, run, info]).into())
}

use std::sync::{Arc, Mutex};

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
    let vm = Arc::new(Mutex::new(vm::VM::new(chunk)));

    let result = std::panic::catch_unwind(|| run_vm(Arc::clone(&vm)));

    match result {
        Ok(output) => {
            output.unwrap_or_else(|_| "Error: Panic occurred during execution".to_string())
        }
        Err(_) => "Error: Panic occurred during execution".to_string(),
    }
}

fn run_vm(vm: Arc<Mutex<vm::VM>>) -> Result<String, String> {
    let mut output = String::new();
    let mut jumps = 0;

    while vm.lock().unwrap().index < vm.lock().unwrap().chunk.code.len() as i32 {
        let once_result = {
            let mut vm_ref_mut = vm.lock().unwrap();
            vm_ref_mut.once(&mut jumps)
        };
        match once_result {
            Ok(s) => output.push_str(&s),
            Err(s) => {
                return Err(error::KlangError::error(
                    KlangError::RuntimeError,
                    s.as_str(),
                    0,
                ))
            }
        }
        vm.lock().unwrap().index += 1;
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
