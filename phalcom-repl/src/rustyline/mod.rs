//! [`rustyline`]-backed helper bundle for the **experimental** rustyline REPL stack.
//!
//! # Status: EXPERIMENTAL / UNUSED
//!
//! This subdirectory contains an alternate REPL implementation that was started
//! using `rustyline` (versus the active `reedline`-based stack in `src/*.rs`).
//!
//! **Nothing in `src/main.rs` or the active REPL modules declares `mod rustyline;`**,
//! so this code is currently unreachable from the binary.
//!
//! ## Disposition (DEFERRED)
//!
//! The rustyline editor and helper types here are kept as a reference while the
//! active `reedline` stack matures.  Once the reedline REPL is feature-complete,
//! this directory should be removed.  Tracked as a DEFERRED cleanup — do not
//! invest further in this module until the decision is made.
