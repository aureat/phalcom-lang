//! Production-faithful source-language acceptance corpus.
//!
//! Each fixture is compiled through `ProgramCompiler`, then executed on a
//! fresh VM with a fixture-local output sink. The corpus intentionally lives in
//! its own integration target so broad fixture work does not contend with the
//! rest of `core`.

#[path = "../support/mod.rs"]
mod support;

#[path = "../core/language/corpus.rs"]
mod corpus;

#[path = "output.rs"]
mod output;
