use anyhow::Result;
use phalcom_compiler::compile;
use phalcom_vm::vm::VM;
use std::{fs, path::Path};

fn main() -> Result<()> {
    let path = std::env::args()
        .nth(1)
        .expect("usage: phalcomc <file.phalcom>");
    let source = fs::read_to_string(Path::new(&path))?;

    let mut vm = VM::new();
    
    match compile(&mut vm, &source) {
        Ok(closure) => {
            let module = vm.module_from_str("<main>");
            match vm.run_module(module, closure) {
                Ok(value) => {
                    println!("{}", value);
                }
                Err(e) => {
                    eprintln!("{}", e);
                }
            }
        }
        Err(e) => {
            eprintln!("{}", e);
        }
    }

    Ok(())
}