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
    /// Reflectable `@requires`/`@ensures` predicate metadata (U-ANNOT-CONTRACTS
    /// plan §3.5 "Contracts are reflectable", D-contract-1).
    ///
    /// Each entry is `(Symbol, Value)` — a name like `#requires_0`/`#ensures_1`
    /// (source declaration order, one counter per attribute kind) paired with
    /// the **un-woven** predicate compiled standalone as a zero-argument,
    /// receiver-shaped [`crate::heap::ClosureObject`] (`self` in slot 0, same
    /// shape as a getter's closure) — for property-testing harnesses and a
    /// future `Method>>contracts` accessor.
    ///
    /// `None` when the method carries no `@requires`/`@ensures` attributes, or
    /// when metadata retention is stripped for the active
    /// [`crate::compiler::attributes::CompileMode`]
    /// (`ExpandCtx::strip_metadata`, plan §3.6's independent metadata axis) —
    /// in the stripped case the predicate closures are never compiled in the
    /// first place, not compiled-then-discarded.
    pub contracts: Option<Vec<(Symbol, Value)>>,
    /// Attribute instances attached via `Object#__attach` (M-ATTR-ROOT,
    /// `attribute-classes.md`) — the retention store behind
    /// `Method#attributes`/`attributesOfType(_)`.
    pub attributes: Vec<Value>,
    /// Set by `Object#__freezeAttributes` once this method's member-level
    /// `@AttrName(...)` attaches have all run — further `__attach` calls are
    /// rejected (`attr.frozen`).
    pub attributes_frozen: bool,
}

impl MethodObject {
    /// Builds a method with the given `kind`, deriving its signature.
    pub fn new(selector: Symbol, sig_kind: SignatureKind, kind: MethodKind, holder: Option<ClassId>) -> Self {
        let signature = Signature::new(selector, sig_kind);
        MethodObject {
            kind,
            signature,
            holder,
            contracts: None,
            attributes: Vec::new(),
            attributes_frozen: false,
        }
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

    /// Appends `attr` to this method's attribute-retention store.
    ///
    /// # Errors
    ///
    /// Returns `Err(())` if [`Self::attributes_frozen`] is set.
    pub fn attach_attribute(&mut self, attr: Value) -> Result<(), ()> {
        if self.attributes_frozen {
            return Err(());
        }
        self.attributes.push(attr);
        Ok(())
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
