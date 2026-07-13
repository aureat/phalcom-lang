//! Call frames and their receiver context.
//!
//! A [`CallFrame`] is a single method/closure activation. Because every link it
//! holds is now a `Copy` handle ([ADR-0009](../../../docs/adr/0009-handle-arena-heap.md))
//! the whole frame is `Copy`, so the VM keeps frames in a plain `Vec` with no
//! `Rc<RefCell<T>>` and no borrow-panic surface.

use crate::heap::ObjRef;
use crate::value::Value;
use phalcom_common::range::SourceRange;

/// A token identifying a particular call-frame activation.
///
/// Realizes the frame-token infrastructure from [ADR-0013](../../docs/adr/0013-block-closure-upvalues.md).
/// A token pairs a frame index with a monotonically-assigned generation so a
/// block can later tell whether the activation it was created in is still the
/// same live frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FrameToken {
    /// The VM frame index the token refers to.
    pub frame_index: usize,
    /// The generation associated with that frame activation.
    pub generation: u64,
}

impl FrameToken {
    /// Creates a new frame token from `frame_index` and `generation`.
    pub fn new(frame_index: usize, generation: u64) -> Self {
        Self { frame_index, generation }
    }
}

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
    /// Executing a closure-backed (non-primitive) method on an **immediate**
    /// receiver (`Bool`/`Number`/`Symbol`) — e.g. a user-defined sacred
    /// selector reopened onto the kernel `Bool` class (U5, ADR-0018: needed
    /// to make the sacred-selector inliner's override-epoch deopt guard
    /// exercisable, since only a closure method — never a primitive — needs
    /// a `CallContext` at all). Carries the receiver `Value` itself, since
    /// there is no [`ObjRef`] to point at.
    Immediate {
        /// The immediate receiver value.
        value: Value,
    },
}

/// A single closure activation: its code handle, receiver, and stack window.
#[derive(Debug, Clone, Copy)]
pub struct CallFrame {
    /// Handle to the [`ClosureObject`](crate::heap::ClosureObject) executing.
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
    /// Monotonically-assigned generation for this activation.
    pub generation: u64,
    /// The home-frame token this activation returns *through* on a non-local
    /// `return`, or `None` for an ordinary method/closure activation.
    ///
    /// Populated **only** when the frame is a block invocation: [`block_call`](crate::primitive::block::block_call)
    /// copies it from the invoked [`BlockObject`](crate::heap::BlockObject)'s
    /// `home_frame_token`, so the [`Bytecode::ReturnNonLocal`](crate::bytecode::Bytecode::ReturnNonLocal)
    /// handler can read "what activation am I unwinding to" straight off the
    /// currently-executing frame (the `BlockObject` that carried the token is not
    /// otherwise reachable from a live `CallFrame`, which only stores the
    /// `ClosureObject` handle). Ordinary method/closure calls leave this `None`;
    /// their `return` compiles to [`Bytecode::Return`](crate::bytecode::Bytecode::Return) and never reads it
    /// ([ADR-0013](../../docs/adr/0013-block-closure-upvalues.md), blocks.md §5).
    /// Because [`FrameToken`] is `Copy`, `Option<FrameToken>` keeps [`CallFrame`]
    /// `Copy`.
    pub home_frame_token: Option<FrameToken>,
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
            generation: 0,
            home_frame_token: None,
        }
    }

    /// Returns this frame's receiver context.
    pub fn context(&self) -> &CallContext {
        &self.context
    }

    /// Returns the frame token for this activation at `frame_index`.
    pub fn token(&self, frame_index: usize) -> FrameToken {
        FrameToken::new(frame_index, self.generation)
    }
}
