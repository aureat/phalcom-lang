//! Portable executable recipes, independent of live VM state.
//!
//! Instruction decoding is the first implemented boundary. Complete image
//! validation and VM materialization are separate prerequisites to execution.

pub mod compatibility;
pub mod constant;
pub mod image;
pub mod instruction;
pub mod module;
pub mod prototype;
pub mod semantic;
pub mod staging;

pub const EXECUTABLE_SCHEMA_V1: u32 = 1;
pub const PORTABLE_BYTECODE_ABI_V1: u32 = 1;
pub const EMITTER_ABI_V1: u32 = 1;
pub const VM_OPERAND_FRAME_ABI_V1: u32 = 1;
pub const NATIVE_RUNTIME_ABI_V1: u32 = 1;
