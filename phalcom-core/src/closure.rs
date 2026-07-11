//! Compiled closures.
//!
//! A [`ClosureObject`] pairs a compiled [`Callable`] with the module it was
//! defined in and its captured upvalues. It is a heap
//! [`Object`](crate::heap::Object); its module link is a handle
//! ([ADR-0009](../../../docs/adr/0009-handle-arena-heap.md)).

use crate::callable::Callable;
use crate::heap::ObjRef;
use crate::value::Value;

/// A compiled closure: its code, its defining module handle, and its upvalues.
#[derive(Debug, Clone)]
pub struct ClosureObject {
    /// The compiled bytecode and metadata this closure runs.
    pub callable: Callable,
    /// Handle to the [`ModuleObject`](crate::module::ModuleObject) this closure
    /// was compiled in.
    pub module: ObjRef,
    /// Captured upvalues, as `Copy` [`Value`]s.
    pub upvalues: Vec<Value>,
}
