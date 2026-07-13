#![allow(unused_variables)]
extern crate core;

pub mod bytecode;
pub mod callable;
pub mod chunk;

pub mod compiler;
pub mod diagnostics;
pub mod error;
pub mod frame;
pub mod heap;
pub mod interner;
pub mod interpret;
pub mod method;
pub mod primitive;
pub mod universe;
pub mod value;
pub mod vm;

#[cfg(test)]
mod tests {}
