//! `Either<L, R>` generic-conformance package.
//!
//! The suite intentionally combines semantic-structure assertions and runtime
//! execution around one ordinary user-defined generic ADT.

mod higher_order;
mod inference;
mod isolation;
mod nested;
mod rejection;
mod runtime;
mod substitution;
