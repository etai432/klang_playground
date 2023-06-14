#![allow(clippy::enum_variant_names)]
#[derive(Debug)]
pub enum KlangError {
    ScannerError,
    ParserError,
    RuntimeError,
}

impl KlangError {
    pub fn error(et: KlangError, msg: &str, line: usize) -> String {
        format!("[{et:?}] at line {}: {}", line, msg)
    }
}
