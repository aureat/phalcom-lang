#![allow(unused_variables)]
pub mod boolean;
pub mod bytecode;
pub mod callable;
pub mod chunk;
pub mod class;
pub mod closure;
pub mod compiler;
pub mod error;
pub mod frame;
pub mod instance;
pub mod interner;
pub mod method;
pub mod module;
pub mod nil;
pub mod primitive;
pub mod signature;
pub mod string;
pub mod universe;
pub mod value;
pub mod vm;

#[cfg(test)]
mod tests {}
