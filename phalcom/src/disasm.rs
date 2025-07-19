#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(dead_code)]
use phalcom_vm::vm::VM;
use phalcom_vm::value::Value;
use phalcom_vm::chunk::Chunk;
use phalcom_vm::closure::ClosureObject;
use phalcom_vm::bytecode::Bytecode;
use phalcom_common::PhRef;
use phalcom_compiler::compile;
use phalcom_vm::error::PhError;

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
