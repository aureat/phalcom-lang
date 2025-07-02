use crate::method::MethodObject;
use phalcom_common::PhRef;

/// Represents a single function call's execution context.
#[derive(Debug, Clone)]
pub struct CallFrame {
    /// The method or function being executed.
    pub method: PhRef<MethodObject>,

    /// The instruction pointer for this frame. It's an index into the
    /// method's bytecode chunk.
    pub ip: usize,

    /// The index into the VM's main value stack where this frame's
    /// stack window begins. The receiver and arguments are located here.
    pub stack_offset: usize,
}
