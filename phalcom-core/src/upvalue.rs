//! Upvalue representation for Phalcom closures.
//!
//! Realizes [ADR-0013](../../docs/adr/0013-block-closure-upvalues.md) and
//! [ADR-0009](../../docs/adr/0009-handle-arena-heap.md).
//! Upvalues are heap-allocated cells that capture local variables from enclosing scopes.
//! They start as `Open`, pointing to a slot index in the VM stack. When the stack frame
//! exits, the VM promotes open upvalues to `Closed`, copying the stack slot's value
//! into the cell so they can outlive the frame.

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
    /// An open upvalue pointing to a live slot on the VM stack.
    Open(usize),
    /// A closed upvalue holding the copied value after the frame exited.
    Closed(Value),
}
