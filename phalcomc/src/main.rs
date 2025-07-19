use anyhow::Result;
use phalcom_compiler::compile;
use phalcom_vm::vm::VM;
use std::{fs, path::Path};
mod disasm;

fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);
    let subcmd = args.next().unwrap_or_else(|| {
        eprintln!("usage: phalcomc <file.phalcom> | disasm <source string>");
        std::process::exit(1);
    });

    if subcmd == "disasm" {
        let source = args.next().unwrap_or_else(|| {
            eprintln!("usage: phalcomc disasm <source string>");
            std::process::exit(1);
        });

        match disasm::disassemble_source(&source) {
            Ok(()) => {}
            Err(e) => eprintln!("Error: {}", e),
        }
        return Ok(());
    }

    // Default: compile and run file
    let path = subcmd;
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
