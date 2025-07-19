use std::env;
use std::fs;
use std::process;

use phalcom_vm::vm::VM;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <file>", args[0]);
        process::exit(1);
    }

    let file_path = &args[1];
    let source = match fs::read_to_string(file_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error reading file '{}': {}", file_path, e);
            process::exit(1);
        }
    };

    let mut vm = VM::new();
    if let Err(e) = vm.interpret(&source) {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}