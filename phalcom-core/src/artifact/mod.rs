//! Portable executable projection and relocation adapters.

mod canonicalize;
pub mod capture;
mod instruction;
pub(crate) mod materialize;
pub mod project;
mod semantic;
mod semantic_materialize;
pub(crate) mod validate;
