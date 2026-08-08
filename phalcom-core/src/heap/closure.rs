//! Compiled closures.
//!
//! A [`ClosureObject`] pairs a compiled [`Callable`] with the module it was
//! defined in and its captured upvalues. It is a heap
//! [`Object`](crate::heap::Object); its module link is a handle
//! ([ADR-0009](../../../docs/adr/accepted/0009-handle-arena-heap.md)).

use std::rc::Rc;

use crate::callable::Callable;
use crate::heap::{ClassId, ObjRef};

/// A compiled closure: its code, its defining module handle, and its upvalues.
///
/// A method/getter/setter body and a block literal both compile to a
/// `ClosureObject`. Its `upvalues` are handles to heap-allocated
/// [`Upvalue`](crate::heap::Upvalue) cells
/// ([ADR-0013](../../../docs/adr/accepted/0013-block-closure-upvalues.md),
/// [ADR-0009](../../../docs/adr/accepted/0009-handle-arena-heap.md)): the template
/// closure the compiler emits carries an empty list, and the VM materializes a
/// fresh, upvalue-filled instance each time it executes
/// [`Bytecode::Closure`](crate::bytecode::Bytecode::Closure).
#[derive(Debug, Clone)]
pub struct ClosureObject {
    /// The compiled bytecode and metadata this closure runs. Shared across
    /// multiple closure instances to avoid allocating/cloning the `Chunk` on
    /// every block literal materialization (U-HOTPATH).
    pub callable: Rc<Callable>,
    /// Handle to the [`ModuleObject`](crate::heap::ModuleObject) this closure
    /// was compiled in.
    pub module: ObjRef,
    /// Captured upvalue cells, as heap [`ObjRef`] handles into the
    /// [`Heap`](crate::heap::Heap). Indexed by the callable's upvalue
    /// descriptors ([`UpvalueDescriptor`](crate::callable::UpvalueDescriptor)).
    pub upvalues: Vec<ObjRef>,
    /// Source class whose lexical privileges this code carries. Blocks copy
    /// this from their template closure when materialized.
    pub lexical_class: Option<ClassId>,
}
