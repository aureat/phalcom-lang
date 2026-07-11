//! Shared, compiler-stage-agnostic utilities for the Phalcom toolchain.
//!
//! This crate is intentionally small — it contains only primitives that are
//! stable across VM redesigns and safe to document and test independently.
//!
//! ## Modules
//!
//! - [`range`] — [`CopyRange<T>`](range::CopyRange) and the
//!   [`SourceRange`](range::SourceRange) alias used by every AST node to
//!   carry byte-offset source locations.
//!
//! ## What is NOT here
//!
//! `PhRef` / `PhWeakRef` / `MaybeWeak` / `phref_new` / `phref_weak` are
//! defined inline below but are being **replaced** by the handle/arena heap
//! redesign (ADR-0009).  They are retained temporarily for compilation
//! compatibility but must not be documented or depended upon — the VM heap
//! unit (U1) will remove them.

pub mod range;

use std::cell::RefCell;
use std::rc::{Rc, Weak};

pub type PhRef<T> = Rc<RefCell<T>>;
pub type PhWeakRef<T> = Weak<RefCell<T>>;

#[derive(Debug, Clone)]
pub enum MaybeWeak<A> {
    Strong(PhRef<A>),
    Weak(PhWeakRef<A>),
}

pub fn phref_new<T>(value: T) -> PhRef<T> {
    Rc::new(RefCell::new(value))
}

pub fn phref_weak<T>(value: &PhRef<T>) -> PhWeakRef<T> {
    Rc::downgrade(value)
}

#[cfg(test)]
mod tests {}
