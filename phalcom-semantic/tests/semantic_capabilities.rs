//! High-density semantic capability probes for `phalcom-semantic`.
//!
//! This is intentionally ONE Cargo integration-test target. The category files
//! below are ordinary Rust modules, so Cargo compiles the whole suite into one
//! test binary instead of one binary per file.

#[path = "semantic_capabilities/branches.rs"]
mod branches;
#[path = "semantic_capabilities/callables.rs"]
mod callables;
#[path = "semantic_capabilities/dispatch.rs"]
mod dispatch;
#[path = "semantic_capabilities/generics.rs"]
mod generics;
#[path = "semantic_capabilities/iteration_advisory.rs"]
mod iteration_advisory;
#[path = "semantic_capabilities/loops_blocks.rs"]
mod loops_blocks;
#[path = "semantic_capabilities/structural.rs"]
mod structural;
#[path = "semantic_capabilities/support.rs"]
mod support;
