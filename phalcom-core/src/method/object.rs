use crate::error::PhResult;
use crate::heap::{ClassId, ObjRef};
use crate::interner::Symbol;
use crate::value::Value;
use crate::vm::VM;

use super::{Signature, SignatureKind};

/// Result of entering a callable from a native method.
///
/// `Returned` completes the current native activation. `EnteredFrame` means the
/// native method rewrote the current stack window and pushed a bytecode frame;
/// the dispatch loop must continue with that frame instead of recursively
/// calling `run_until`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CallOutcome {
    /// Native work completed immediately.
    Returned(Value),
    /// A bytecode activation was pushed onto the current VM loop.
    EnteredFrame,
}

/// The old native ABI, retained only as a mechanical migration adapter.
pub type LegacyPrimitiveFn = fn(_vm: &mut VM, _receiver: &Value, _args: &[Value]) -> PhResult<Value>;

/// A compact, non-borrowing view of the current argument window.
///
/// `labels` is always pre-decoded: it is populated at the call site from the
/// already-interned selector constituents, so the Family hot path never has to
/// call `decode_selector()` to recover them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArgumentView {
    receiver_index: usize,
    positional_count: usize,
    labeled_count: usize,
    selector: Option<Symbol>,
    /// Pre-decoded label symbols in call order; empty for positional-only
    /// and getter/setter calls.
    labels: Box<[Symbol]>,
    caller_access: Option<ClassId>,
    caller_internal: bool,
}

impl ArgumentView {
    /// Describes a positional-only window at `receiver_index`.
    pub(crate) fn positional_window(receiver_index: usize, positional_count: usize, caller_access: Option<ClassId>, caller_internal: bool) -> Self {
        Self {
            receiver_index,
            positional_count,
            labeled_count: 0,
            selector: None,
            labels: Box::default(),
            caller_access,
            caller_internal,
        }
    }

    /// Describes a selector-shaped window with pre-decoded labels.
    pub(crate) fn shaped_with_labels(
        receiver_index: usize,
        positional_count: usize,
        labels: Box<[Symbol]>,
        selector: Symbol,
        caller_access: Option<ClassId>,
        caller_internal: bool,
    ) -> Self {
        let labeled_count = labels.len();
        Self {
            receiver_index,
            positional_count,
            labeled_count,
            selector: Some(selector),
            labels,
            caller_access,
            caller_internal,
        }
    }

    /// Number of positional values in the argument lane.
    pub fn positional_count(&self) -> usize {
        self.positional_count
    }

    /// Number of labeled values in the argument lane.
    pub fn labeled_count(&self) -> usize {
        self.labeled_count
    }

    /// Returns positional value `index`.
    pub fn positional(&self, vm: &VM, index: usize) -> Option<Value> {
        (index < self.positional_count).then(|| vm.stack[self.receiver_index + 1 + index])
    }

    /// Returns labeled value `index` in label order.
    pub fn labeled_value(&self, vm: &VM, index: usize) -> Option<Value> {
        (index < self.labeled_count).then(|| vm.stack[self.receiver_index + 1 + self.positional_count + index])
    }

    /// Returns the label for labeled lane position `index`.
    pub fn label(&self, index: usize) -> Option<Symbol> {
        self.labels.get(index).copied()
    }

    /// Returns pre-decoded label symbols in call order.
    ///
    /// Never calls `decode_selector()`; labels were decoded once at call-site
    /// construction and cached in this view.
    pub fn labels(&self) -> &[Symbol] {
        &self.labels
    }

    /// Returns the source stack receiver index for VM activation helpers.
    pub(crate) fn receiver_index(&self) -> usize {
        self.receiver_index
    }

    /// Returns the caller authority captured before entering the native gateway.
    pub(crate) fn caller_authority(&self) -> (Option<ClassId>, bool) {
        (self.caller_access, self.caller_internal)
    }

    /// Returns a view with a newly encoded call-site selector and shape.
    pub(crate) fn with_selector(self, selector: Symbol, positional_count: usize, labels: Box<[Symbol]>) -> Self {
        let labeled_count = labels.len();
        Self {
            receiver_index: self.receiver_index,
            positional_count,
            labeled_count,
            selector: Some(selector),
            labels,
            caller_access: self.caller_access,
            caller_internal: self.caller_internal,
        }
    }
}

/// Native Rust method implementation.
///
/// `Shape` is the ratified ABI. `Legacy` is an internal compatibility arm used
/// to migrate the existing fixed-arity primitive corpus mechanically; the VM
/// feeds it an on-stack small buffer and allocates only for unusually wide
/// legacy calls.
#[derive(Clone, Copy)]
pub enum PrimitiveFn {
    /// Shape-aware primitive ABI.
    Shape(fn(&mut VM, Value, ArgumentView) -> PhResult<CallOutcome>),
    /// Temporary adapter for existing fixed-arity primitives.
    Legacy(LegacyPrimitiveFn),
}

impl std::fmt::Debug for PrimitiveFn {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::Shape(_) => "PrimitiveFn::Shape",
            Self::Legacy(_) => "PrimitiveFn::Legacy",
        })
    }
}

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
    /// Member visibility.
    pub visibility: super::MemberVisibility,
    /// Lexical source class controlling @private/@protected access.
    pub access_owner: Option<ClassId>,
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
            visibility: super::MemberVisibility::Public,
            access_owner: holder,
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
    pub fn new_primitive(selector: Symbol, sig_kind: SignatureKind, primitive: LegacyPrimitiveFn, holder: ClassId) -> Self {
        Self::new(selector, sig_kind, MethodKind::Primitive(PrimitiveFn::Legacy(primitive)), Some(holder))
    }

    /// Builds a method using the shape-aware native ABI and explicit signature.
    pub fn new_shape_primitive(
        selector: Symbol,
        signature: Signature,
        primitive: fn(&mut VM, Value, ArgumentView) -> PhResult<CallOutcome>,
        holder: ClassId,
    ) -> Self {
        let mut method = Self::new(selector, signature.kind, MethodKind::Primitive(PrimitiveFn::Shape(primitive)), Some(holder));
        method.signature = signature;
        method
    }

    /// Returns this method's selector [`Symbol`].
    pub fn selector(&self) -> Symbol {
        self.signature.selector
    }

    /// Appends `attr` to this method's attribute-retention store.
    ///
    /// # Errors
    ///
    /// Returns `false` if [`Self::attributes_frozen`] is set.
    pub fn attach_attribute(&mut self, attr: Value) -> bool {
        if self.attributes_frozen {
            return false;
        }
        self.attributes.push(attr);
        true
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
