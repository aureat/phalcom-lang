use crate::class::ClassObject;
use crate::closure::ClosureObject;
use crate::error::PhResult;
use crate::interner::Symbol;
use crate::value::Value;
use crate::vm::VM;
use phalcom_common::{PhRef, PhWeakRef};

pub type PrimitiveFn = fn(&mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value>;

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SignatureKind {
    /// `init new(_,_)`
    Initializer(u8),

    /// `foo(_,_,_)`
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
    pub selector: String,
    pub kind: SignatureKind,
}

impl Signature {
    pub fn new(selector: &str, kind: SignatureKind) -> Self {
        Signature {
            selector: selector.to_string(),
            kind,
        }
    }
}

#[derive(Debug)]
pub enum MethodKind {
    /// Phalcom code compiled to bytecode.
    Closure(PhRef<ClosureObject>),

    /// A native Rust function for core library methods.
    Primitive(PrimitiveFn),
}

#[derive(Debug)]
pub struct MethodObject {
    pub kind: MethodKind,
    pub signature: Signature,
    pub holder: PhWeakRef<ClassObject>,
}

impl MethodObject {
    pub fn new(kind: MethodKind, selector: Symbol, sig_kind: SignatureKind) -> Self {
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

    pub fn primitive(selector: Symbol, arity: u8, primitive: PrimitiveFn) -> Self {
        let signature = Signature::new(selector, SignatureKind::Method(arity));
        MethodObject {
            kind: MethodKind::Primitive(primitive),
            signature,
            holder: PhWeakRef::default(),
        }
    }

    pub fn is_native(&self) -> bool {
        matches!(self.kind, MethodKind::Primitive(_))
    }

    pub fn is_bytecode(&self) -> bool {
        matches!(self.kind, MethodKind::Closure(_))
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
