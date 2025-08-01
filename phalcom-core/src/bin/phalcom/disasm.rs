#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(dead_code)]
use phalcom_common::PhRef;
use phalcom_core::closure::ClosureObject;
use phalcom_core::compiler::lib::compile;
use phalcom_core::error::PhError;
use phalcom_core::vm::VM;

pub fn disassemble_source(source: &str) -> Result<(), PhError> {
    let mut vm = VM::new();
    let closure = compile(&mut vm, source)?;
    let closure_ref: PhRef<ClosureObject> = closure;
    let chunk = closure_ref.borrow().callable.chunk.clone();
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
