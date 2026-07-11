#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(dead_code)]
use phalcom_core::error::PhError;
use phalcom_core::vm::VM;

pub fn disassemble_source(source: &str) -> Result<(), PhError> {
    let mut vm = VM::new();
    let module = vm.create_module("main", "<main>");
    let closure = vm.compile_closure(module, source)?;
    let chunk = vm.heap.closure(closure).callable.chunk.clone();
    println!("Constants:");
    for (i, constant) in chunk.constants.iter().enumerate() {
        println!("  [{}] {:?}", i, constant);
    }
    println!("\nBytecode:");
    for (i, instr) in chunk.code.iter().enumerate() {
        println!("  {:04}: {:?}", i, instr);
    }
    Ok(())
}
