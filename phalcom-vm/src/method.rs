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
    Initializer,

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
}

impl Signature {
    pub fn new(selector: Symbol, kind: SignatureKind) -> Self {
        Signature { selector, kind }
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
    pub fn new(selector: Symbol, sig_kind: SignatureKind, kind: MethodKind) -> Self {
        let signature = Signature::new(selector, sig_kind);

        MethodObject {
            kind,
            signature,
            holder: PhWeakRef::default(),
        }
    }

    pub fn primitive(selector: Symbol, sig_kind: SignatureKind, primitive: PrimitiveFn) -> Self {
        MethodObject::new(selector, sig_kind, MethodKind::Primitive(primitive))
    }

    pub fn make_name(holder: PhRef<ClassObject>, selector: &str) -> PhRef<StringObject> {
        let name = holder.borrow().name_copy().add("::").add(selector);
        phref_new(StringObject::from_string(name))
    }

    pub fn make_weak_name(holder: PhWeakRef<ClassObject>, selector: &str) -> PhRef<StringObject> {
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

    pub fn name(&self, vm: &VM) -> PhString {
        let name = vm.resolve_symbol(self.signature.selector);
        phstring_new(name.to_string())
    }

    pub fn to_phalcom_string(&self, vm: &VM) -> PhRef<StringObject> {
        let name = vm.resolve_symbol(self.signature.selector);
        let holder_name = self.holder.upgrade().map_or_else(|| String::from("Unknown"), |c| c.borrow().name_copy());
        let full_name = format!("{}::{}", holder_name, name);
        phstring_new(full_name)
    }
}
