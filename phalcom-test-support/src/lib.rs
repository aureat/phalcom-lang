//! Shared test-only support for repository integration fixtures.
//!
//! Production crates must not depend on this crate. Add it only under
//! `[dev-dependencies]` when a test needs the canonical golden workspace.

mod expectations;
mod golden;
mod markers;
mod mutation;

pub use expectations::{BaselineExpectations, CompilerExpectation, DiagnosticsExpectation, WorkspaceExpectation};
pub use golden::{GoldenWorkspace, GoldenWorkspaceError};
pub use markers::{MarkedSource, MarkerPosition};
pub use mutation::{Mutation, MutationError};
