//! Call frames and their receiver context.
//!
//! A [`CallFrame`] is a single method/closure activation. Because every link it
//! holds is now a `Copy` handle ([ADR-0009](../../../docs/adr/0009-handle-arena-heap.md))
//! the whole frame is `Copy`, so the VM keeps frames in a plain `Vec` with no
//! `Rc<RefCell<T>>` and no borrow-panic surface.

use crate::heap::ObjRef;
use phalcom_common::range::SourceRange;

/// The receiver a frame is executing against.
#[derive(Debug, Clone, Copy)]
pub enum CallContext {
    /// Executing a method on a user-defined instance.
    Instance {
        /// Handle to the receiver instance.
        instance: ObjRef,
    },
    /// Executing a (static) method on a class.
    Class {
        /// Handle to the receiver class.
        class: ObjRef,
    },
    /// Executing top-level module code.
    Module {
        /// Handle to the running module.
        module: ObjRef,
    },
}

/// A single closure activation: its code handle, receiver, and stack window.
#[derive(Debug, Clone, Copy)]
pub struct CallFrame {
    /// Handle to the [`ClosureObject`](crate::closure::ClosureObject) executing.
    pub closure: ObjRef,
    /// The receiver context (`self`) for this activation.
    pub context: CallContext,
    /// Instruction pointer: an index into the closure's bytecode chunk.
    pub ip: usize,
    /// Index into the VM value stack where this frame's window begins (receiver
    /// then arguments).
    pub stack_offset: usize,
    /// Source span of the call site, for stack traces.
    pub caller_source: Option<SourceRange>,
}

impl CallFrame {
    /// Creates a frame for `closure` with receiver `context`, starting at `ip`
    /// over the stack window at `stack_offset`.
    pub fn new(closure: ObjRef, context: CallContext, ip: usize, stack_offset: usize, caller_source: Option<SourceRange>) -> Self {
        Self {
            closure,
            context,
            ip,
            stack_offset,
            caller_source,
        }
    }

    /// Returns this frame's receiver context.
    pub fn context(&self) -> &CallContext {
        &self.context
    }
}
