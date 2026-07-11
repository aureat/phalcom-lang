use crate::class::ClassObject;
use crate::closure::ClosureObject;
use crate::error::PhResult;
use crate::interner::Symbol;
use crate::string::{phstring_new, PhString, StringObject};
use crate::value::Value;
use crate::vm::VM;
use phalcom_common::{phref_new, PhRef, PhWeakRef};
use std::ops::Add;

pub type PrimitiveFn = fn(_vm: &mut VM, _receiver: &Value, _args: &[Value]) -> PhResult<Value>;

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
    pub selector: Symbol,
    pub kind: SignatureKind,
    pub positional_arity: u8,
    pub variadic: bool,
}

impl Signature {
    pub fn new(selector: Symbol, kind: SignatureKind) -> Self {
        let positional_arity = match kind {
            SignatureKind::Initializer(n) => n,
            SignatureKind::Method(n) => n,
            SignatureKind::Getter => 0,
            SignatureKind::Setter => 1,
            SignatureKind::SubscriptGet(n) => n,
            SignatureKind::SubscriptSet(n) => n + 1,
        };
        Signature {
            selector,
            kind,
            positional_arity,
            variadic: false,
        }
    }

    pub fn new_with_arity(selector: Symbol, kind: SignatureKind, positional_arity: u8, variadic: bool) -> Self {
        Signature {
            selector,
            kind,
            positional_arity,
            variadic,
        }
    }
}

/// Helper to generate canonical label-encoded selector string.
pub fn encode_selector(name: &str, labels: &[Option<String>], kind: SignatureKind) -> String {
    match kind {
        SignatureKind::Initializer(0) => format!("init {name}()"),
        SignatureKind::Initializer(_) => {
            let mut s = format!("init {name}(");
            for label in labels {
                if let Some(lbl) = label {
                    s.push_str(lbl);
                    s.push(':');
                } else {
                    s.push_str("_:");
                }
            }
            s.push(')');
            s
        }
        SignatureKind::Method(0) => format!("{name}()"),
        SignatureKind::Method(_) => {
            let mut s = format!("{name}(");
            for label in labels {
                if let Some(lbl) = label {
                    s.push_str(lbl);
                    s.push(':');
                } else {
                    s.push_str("_:");
                }
            }
            s.push(')');
            s
        }
        SignatureKind::Getter => name.to_string(),
        SignatureKind::Setter => format!("{name}=(_:)"),
        SignatureKind::SubscriptGet(n) => {
            let mut s = "[".to_string();
            for _ in 0..n {
                s.push_str("_:");
            }
            s.push(']');
            s
        }
        SignatureKind::SubscriptSet(n) => {
            let mut s = "[".to_string();
            for _ in 0..n {
                s.push_str("_:");
            }
            s.push_str("]=(_:)");
            s
        }
    }
}

/// Turn a base name like `"foo"` plus a `SignatureKind` into the textual
/// signature used by the compiler/VM.
pub fn make_signature(base: &str, kind: SignatureKind) -> String {
    let arity = match kind {
        SignatureKind::Initializer(n) => n,
        SignatureKind::Method(n) => n,
        SignatureKind::Getter => 0,
        SignatureKind::Setter => 0, // Setter has 1 arg but labels list for it is empty in AST
        SignatureKind::SubscriptGet(n) => n,
        SignatureKind::SubscriptSet(n) => n,
    };
    let labels = vec![None; arity as usize];
    encode_selector(base, &labels, kind)
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
    pub fn new(selector: Symbol, sig_kind: SignatureKind, kind: MethodKind, holder: PhWeakRef<ClassObject>) -> Self {
        let signature = Signature::new(selector, sig_kind);

        MethodObject { kind, signature, holder }
    }

    pub fn new_single(selector: Symbol, sig_kind: SignatureKind, kind: MethodKind) -> Self {
        Self::new(selector, sig_kind, kind, PhWeakRef::default())
    }

    pub fn new_closure(selector: Symbol, sig_kind: SignatureKind, closure: PhRef<ClosureObject>, holder: PhWeakRef<ClassObject>) -> Self {
        Self::new(selector, sig_kind, MethodKind::Closure(closure), holder)
    }

    pub fn new_primitive(selector: Symbol, sig_kind: SignatureKind, primitive: PrimitiveFn, holder: PhWeakRef<ClassObject>) -> Self {
        Self::new(selector, sig_kind, MethodKind::Primitive(primitive), holder)
    }

    pub fn make_name(holder: PhRef<ClassObject>, selector: &str) -> PhString {
        let name = holder.borrow().name_copy().add("::").add(selector);
        phref_new(StringObject::from_string(name))
    }

    pub fn make_weak_name(holder: PhWeakRef<ClassObject>, selector: &str) -> PhString {
        let name = holder
            .upgrade()
            .map_or_else(|| String::from("Unknown"), |c| c.borrow().name_copy())
            .add("::")
            .add(selector);
        phref_new(StringObject::from_string(name))
    }

    pub fn selector(&self) -> Symbol {
        self.signature.selector
    }

    pub fn is_primitive(&self) -> bool {
        matches!(self.kind, MethodKind::Primitive(_))
    }

    pub fn is_closure(&self) -> bool {
        matches!(self.kind, MethodKind::Closure(_))
    }

    pub fn set_holder(&mut self, holder: PhWeakRef<ClassObject>) {
        self.holder = holder;
    }

    pub fn name(&self, vm: &VM) -> PhString {
        let name = vm.resolve_symbol(self.signature.selector);
        phstring_new(name.to_string())
    }

    pub fn to_debug(&self, vm: &VM) -> String {
        let name = vm.resolve_symbol(self.signature.selector);
        let holder_name = self
            .holder
            .upgrade()
            .map_or_else(|| panic!("this shouldn't happen"), |c| c.borrow().name_copy());
        let full_name = format!("<method {}::{}>", holder_name, name);
        full_name
    }
}
