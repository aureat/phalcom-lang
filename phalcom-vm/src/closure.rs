use crate::callable::Callable;
use crate::value::Value;

#[derive(Debug, Clone)]
pub struct ClosureObject {
    pub callable: Callable,
    pub upvalues: Vec<Value>,
}
