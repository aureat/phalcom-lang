use crate::chunk::Chunk;
use crate::value::Value;
use crate::vm::VM;

pub type NativeFn = fn(&mut VM, receiver: &Value, args: &[Value]) -> Result<Value, String>;

#[derive(Debug, Clone)]
pub struct Parameter {
    pub name: String,
    pub type_hint: Option<String>,
}

#[derive(Debug)]
pub enum MethodKind {
    /// Phalcom code compiled to bytecode.
    Bytecode(Chunk),

    /// A native Rust function for core library methods.
    Native(NativeFn),
}

#[derive(Debug)]
pub struct MethodObject {
    pub kind: MethodKind,
    pub arity: usize,               // The number of parameters.
    pub parameters: Vec<Parameter>, // The full description of each parameter.
}

impl MethodObject {
    pub fn new(kind: MethodKind, arity: usize, parameters: Vec<Parameter>) -> Self {
        Self {
            kind,
            arity,
            parameters,
        }
    }

    pub fn new_native(arity: usize, func: NativeFn) -> Self {
        Self {
            kind: MethodKind::Native(func),
            arity,
            parameters: Vec::new(),
        }
    }

    pub fn is_native(&self) -> bool {
        matches!(self.kind, MethodKind::Native(_))
    }

    pub fn is_bytecode(&self) -> bool {
        matches!(self.kind, MethodKind::Bytecode(_))
    }

    pub fn is_empty(&self) -> bool {
        self.arity == 0 && self.parameters.is_empty()
    }

    pub fn is_variadic(&self) -> bool {
        self.arity == 0 && !self.parameters.is_empty()
    }
}

pub fn generate_method_selector(name: &str, labels: &[Option<&str>]) -> String {
    if labels.is_empty() {
        return name.to_string();
    }

    let mut selector = format!("{}:", name);
    for label in labels {
        if let Some(label_str) = label {
            selector.push_str(label_str);
        }
        selector.push(':');
    }
    selector
}
