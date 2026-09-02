//! Higher-kinded / generic-inheritance conformance package.
//!
//! These tests intentionally combine canonical semantic-product assertions,
//! inference explanation checks, generic-body verification, negative semantics,
//! override selection, and VM execution over ordinary Phalcom code.

mod bodies;
mod composition;
mod constructor_agreement;
mod inference;
mod inheritance;
mod inherited_methods;
mod kinds;
mod overrides;
mod rejection;
mod runtime;
mod support;
mod type_lambdas;
