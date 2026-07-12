//! Upvalue representation for Phalcom closures.
//!
//! Realizes [ADR-0013](../../docs/adr/0013-block-closure-upvalues.md) and
//! [ADR-0009](../../docs/adr/0009-handle-arena-heap.md).
//! Upvalues are heap-allocated cells that capture local variables from enclosing scopes.
//! They start as `Open`, pointing to a slot index in the VM stack. When the stack frame
//! exits, the VM promotes open upvalues to `Closed`, copying the stack slot's value
//! into the cell so they can outlive the frame.

use crate::heap::ObjRef;
use crate::value::Value;

/// An upvalue cell captured by a closure.
///
/// Realizes [ADR-0013](../../docs/adr/0013-block-closure-upvalues.md).
/// While the enclosing scope is active, the upvalue is [`Upvalue::Open`] and points
/// to the index of the variable's slot on the VM's value stack. When the scope is
/// popped, the upvalue is promoted to [`Upvalue::Closed`], copying the value from
/// the stack into the cell.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Upvalue {
    /// An open upvalue pointing to a live slot on a fiber's value stack.
    ///
    /// The stack is per-fiber and swapped in and out of the VM on a fiber
    /// switch ([ADR-0030](../../docs/adr/0030-fibers-and-futures-cooperative-concurrency.md)),
    /// so `slot` alone is ambiguous: it must be resolved against the stack of
    /// the fiber that *owns* the home frame, which is the VM's live stack only
    /// while that fiber is current and otherwise lives parked in the owning
    /// [`FiberObject`](crate::heap::FiberObject). `fiber` records that owner so
    /// a closure resumed on a *different* fiber reads and writes the correct
    /// stack rather than indexing whichever fiber happens to be running.
    Open {
        /// The fiber whose stack holds the home slot.
        fiber: ObjRef,
        /// The absolute slot index on that fiber's stack.
        slot: usize,
    },
    /// A closed upvalue holding the copied value after the frame exited.
    Closed(Value),
}
