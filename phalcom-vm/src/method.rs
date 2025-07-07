use crate::chunk::Chunk;
use crate::error::PhResult;
use crate::interner::Symbol;
use crate::value::Value;
use crate::vm::VM;

pub type PrimitiveFn = fn(&mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value>;

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SigKind {
    /// `foo()`
    Method(u8),

    /// `foo`
    Getter,

    /// `foo=(_)`
    Setter,

    /// `[_]`
    SubscriptGet(u8),

    /// `[_]=(_)`
    SubscriptSet(u8),
}

#[derive(Clone, Debug)]
pub struct Signature {
    pub selector: Symbol,
    pub kind: SigKind,
    pub arity: u8,
}

#[derive(Debug)]
pub enum MethodKind {
    /// Phalcom code compiled to bytecode.
    Bytecode(Chunk),

    /// A native Rust function for core library methods.
    Primitive(PrimitiveFn),
}

#[derive(Debug)]
pub struct MethodObject {
    pub kind: MethodKind,
    pub arity: u8,
    pub signature: Signature,
}

impl MethodObject {
    pub fn new(kind: MethodKind, arity: u8, selector: Symbol, sig_kind: SigKind) -> Self {
        let signature = Signature {
            selector,
            kind: sig_kind,
            arity,
        };

        MethodObject {
            kind,
            arity,
            signature,
        }
    }

    pub fn is_native(&self) -> bool {
        matches!(self.kind, MethodKind::Primitive(_))
    }

    pub fn is_bytecode(&self) -> bool {
        matches!(self.kind, MethodKind::Bytecode(_))
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
