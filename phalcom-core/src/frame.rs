use crate::class::ClassObject;
use crate::closure::ClosureObject;
use crate::instance::InstanceObject;
use crate::module::ModuleObject;
use phalcom_common::PhRef;

/// Represents the callee
#[derive(Debug, Clone)]
pub enum CallContext {
    Instance { instance: PhRef<InstanceObject> },
    Class { class: PhRef<ClassObject> },
    Module { module: PhRef<ModuleObject> },
}

/// Represents a single function call's execution context.
#[derive(Debug, Clone)]
pub struct CallFrame {
    /// Closure being executed
    pub closure: PhRef<ClosureObject>,

    /// Context
    pub context: CallContext,

    /// The instruction pointer for this frame. It's an index into the
    /// method's bytecode chunk.
    pub ip: usize,

    /// The index into the VM's main value stack where this frame's
    /// stack window begins. The receiver and arguments are located here.
    pub stack_offset: usize,
}

impl CallFrame {
    pub fn new(closure: PhRef<ClosureObject>, context: CallContext, ip: usize, stack_offset: usize) -> Self {
        Self {
            context,
            closure,
            ip,
            stack_offset,
        }
    }

    pub fn context(&self) -> &CallContext {
        &self.context
    }

    pub fn closure(&self) -> PhRef<ClosureObject> {
        self.closure.clone()
    }
}
