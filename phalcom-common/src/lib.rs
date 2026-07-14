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
//! The old `Rc<RefCell<T>>` aliases (`PhRef` / `PhWeakRef` / `MaybeWeak` /
//! `phref_new` / `phref_weak`) have been **retired** by the handle/arena heap
//! redesign ([ADR-0009](../../../docs/adr/accepted/0009-handle-arena-heap.md)). Every
//! heap reference is now a `Copy` generational handle,
//! `phalcom_core::heap::ObjRef`, resolved through the VM-owned
//! `phalcom_core::heap::Heap` — see `phalcom-core/src/heap.rs`. No shared-owner
//! reference type lives in this crate anymore.

pub mod range;
