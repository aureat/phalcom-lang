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
#[cfg(feature = "opcode-histogram")]
pub mod opcode_stats;
pub mod parameters;
pub mod primitive;
pub(crate) mod product;
pub mod resource;
pub mod universe;
pub mod value;
pub mod vm;

#[cfg(test)]
mod tests {}
