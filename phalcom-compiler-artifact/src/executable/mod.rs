//! Portable executable recipes, independent of live VM state.
//!
//! Instruction decoding is the first implemented boundary. Complete image
//! validation and VM materialization are separate prerequisites to execution.

pub mod constant;
pub mod instruction;
pub mod prototype;

pub const EXECUTABLE_SCHEMA_V1: u32 = 1;
pub const PORTABLE_BYTECODE_ABI_V1: u32 = 1;
