use crate::callable::Callable;
use crate::module::ModuleObject;
use crate::value::Value;
use phalcom_common::PhRef;

#[derive(Debug, Clone)]
pub struct ClosureObject {
    pub callable: Callable,
    pub module: PhRef<ModuleObject>,
    pub upvalues: Vec<Value>,
}
