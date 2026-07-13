use crate::error::PhResult;
use crate::heap::{ClassId, ObjRef};
use crate::interner::Symbol;
use crate::value::Value;
use crate::vm::VM;

use super::{Signature, SignatureKind};

/// A native Rust method implementation for a core-library method.
///
/// Receives the VM (hence the [`Heap`](crate::heap::Heap)), the receiver, and the
/// arguments, and returns a result [`Value`].
pub type PrimitiveFn = fn(_vm: &mut VM, _receiver: &Value, _args: &[Value]) -> PhResult<Value>;

/// The implementation strategy behind a [`MethodObject`].
#[derive(Debug, Clone, Copy)]
pub enum MethodKind {
    /// Phalcom code compiled to bytecode, by [`ClosureObject`](crate::heap::ClosureObject) handle.
    Closure(ObjRef),
    /// A native Rust function for a core-library method.
    Primitive(PrimitiveFn),
}

/// A callable method: its implementation, signature, and holder class.
#[derive(Debug)]
pub struct MethodObject {
    /// Whether this method is a bytecode closure or a native primitive.
    pub kind: MethodKind,
    /// The method's calling signature.
    pub signature: Signature,
    /// Handle to the class that owns this method, once bound (`None` until set).
    pub holder: Option<ClassId>,
}

impl MethodObject {
    /// Builds a method with the given `kind`, deriving its signature.
    pub fn new(selector: Symbol, sig_kind: SignatureKind, kind: MethodKind, holder: Option<ClassId>) -> Self {
        let signature = Signature::new(selector, sig_kind);
        MethodObject { kind, signature, holder }
    }

    /// Builds an unbound method (holder `None`), typically a compiler-produced
    /// closure that is attached to its class later by `Bytecode::Method`.
    pub fn new_single(selector: Symbol, sig_kind: SignatureKind, kind: MethodKind) -> Self {
        Self::new(selector, sig_kind, kind, None)
    }

    /// Builds a native primitive method held by `holder`.
    pub fn new_primitive(selector: Symbol, sig_kind: SignatureKind, primitive: PrimitiveFn, holder: ClassId) -> Self {
        Self::new(selector, sig_kind, MethodKind::Primitive(primitive), Some(holder))
    }

    /// Returns this method's selector [`Symbol`].
    pub fn selector(&self) -> Symbol {
        self.signature.selector
    }

    /// Returns whether this method is a native primitive.
    pub fn is_primitive(&self) -> bool {
        matches!(self.kind, MethodKind::Primitive(_))
    }

    /// Returns whether this method is a bytecode closure.
    pub fn is_closure(&self) -> bool {
        matches!(self.kind, MethodKind::Closure(_))
    }

    /// Binds this method to its `holder` class handle.
    pub fn set_holder(&mut self, holder: ClassId) {
        self.holder = Some(holder);
    }

    /// Renders this method's debug form, `"<method Holder::selector>"`.
    ///
    /// # Panics
    ///
    /// Panics if the method has no holder set (an internal invariant violation).
    pub fn to_debug(&self, vm: &VM) -> String {
        let name = vm.resolve_symbol(self.signature.selector);
        let holder_name = self
            .holder
            .map_or_else(|| panic!("this shouldn't happen"), |holder| vm.heap.class(holder).name.clone());
        format!("<method {holder_name}::{name}>")
    }
}
