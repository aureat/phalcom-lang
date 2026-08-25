//! Plan 3 golden semantic composition fixtures.
//!
//! The suite source remains in `tests/golden.rs` for now, but is compiled only
//! as this ordinary module of the canonical `semantic` integration target.
//! `Cargo.toml` disables automatic integration-target discovery so this does
//! not create a second test binary.

#[path = "../../golden.rs"]
mod suite;
