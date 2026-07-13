//! The concrete [`BlockObject`] heap representation.
//!
//! Realizes [ADR-0013](../../docs/adr/0013-block-closure-upvalues.md) and
//! [ADR-0006](../../docs/adr/0006-function-as-abstract-callable-root.md).
//! A block is a first-class lexical closure. It wraps a [`ClosureObject`](crate::heap::ClosureObject)
//! (the compiled bytecode and its captured upvalues) along with a [`FrameToken`]
//! identifying the home stack frame where the block was created (used for non-local returns).

use crate::heap::ObjRef;
use crate::frame::FrameToken;

/// A first-class block (lexical closure) object.
///
/// Realizes [ADR-0013](../../docs/adr/0013-block-closure-upvalues.md).
/// Blocks are concrete instances of the abstract `Function` class, and siblings of `Method`.
/// They wrap a compiled closure handle along with the frame token of their home activation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BlockObject {
    /// Handle to the underlying [`ClosureObject`](crate::heap::ClosureObject).
    pub closure: ObjRef,
    /// The frame token of the activation in which this block was created.
    pub home_frame_token: FrameToken,
}

impl BlockObject {
    /// Creates a new `BlockObject` wrapping `closure` and stamped with `home_frame_token`.
    pub fn new(closure: ObjRef, home_frame_token: FrameToken) -> Self {
        BlockObject {
            closure,
            home_frame_token,
        }
    }
}
