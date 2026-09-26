use crate::error::{PhResult, RuntimeError};
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
    /// A VM-owned control activation was pushed onto the control stack.
    EnteredControl,
    /// Execution switched to another fiber.
    SwitchedFiber,
}

/// The old native ABI, retained only as a mechanical migration adapter.
pub type LegacyPrimitiveFn = fn(_vm: &mut VM, _receiver: &Value, _args: &[Value]) -> PhResult<Value>;

/// The physical argument lanes for one selector-shaped invocation.
///
/// Selector slots describe behavior identity. This layout describes the
/// values occupying the runtime window: structural positional values,
/// structural labeled values, and (for setters) the distinguished trailing
/// assigned value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvocationLayout {
    structural_positionals: usize,
    labels: Box<[Symbol]>,
    setter_value: bool,
}

impl InvocationLayout {
    /// Creates an ordinary method/getter/subscript-get layout.
    pub(crate) fn ordinary(structural_positionals: usize, labels: Box<[Symbol]>) -> Self {
        Self {
            structural_positionals,
            labels,
            setter_value: false,
        }
    }

    /// Creates a setter layout with a distinguished trailing value lane.
    pub(crate) fn setter(structural_positionals: usize, labels: Box<[Symbol]>) -> Self {
        Self {
            structural_positionals,
            labels,
            setter_value: true,
        }
    }

    /// Returns the number of structural positional values.
    pub fn structural_positionals(&self) -> usize {
        self.structural_positionals
    }

    /// Returns structural labels in their physical call order.
    pub fn labels(&self) -> &[Symbol] {
        &self.labels
    }

    /// Returns the number of structural labeled values.
    pub fn labeled_count(&self) -> usize {
        self.labels.len()
    }

    /// Returns whether this invocation has a distinguished setter value lane.
    pub fn has_setter_value(&self) -> bool {
        self.setter_value
    }

    /// Returns the number of values after the receiver in the physical stack window.
    pub fn physical_arity(&self) -> usize {
        self.structural_positionals + self.labels.len() + usize::from(self.setter_value)
    }

    /// Validates the lane invariant required by a resolved signature kind.
    #[cfg(test)]
    pub(crate) fn validate_for_kind(&self, kind: SignatureKind) -> Result<(), RuntimeError> {
        Self::validate_parts_for_kind(kind, self.structural_positionals, self.labels.len(), self.setter_value)
    }

    fn validate_parts_for_kind(kind: SignatureKind, structural_positionals: usize, labeled_slots: usize, setter_value: bool) -> Result<(), RuntimeError> {
        let requires_setter = matches!(kind, SignatureKind::Setter | SignatureKind::SubscriptSet(_));
        if setter_value != requires_setter {
            return Err(RuntimeError::Internal(format!(
                "invocation layout setter lane does not match signature kind {kind:?}"
            )));
        }

        match kind {
            SignatureKind::Getter | SignatureKind::Setter if structural_positionals != 0 || labeled_slots != 0 => {
                Err(RuntimeError::Internal(format!("accessor invocation layout has structural lanes for {kind:?}")))
            }
            SignatureKind::Method(slots) | SignatureKind::SubscriptGet(slots) | SignatureKind::SubscriptSet(slots)
                if structural_positionals + labeled_slots != slots as usize =>
            {
                Err(RuntimeError::Internal(format!(
                    "invocation layout has {} structural slots for {kind:?}",
                    structural_positionals + labeled_slots
                )))
            }
            _ => Ok(()),
        }
    }

    /// Validates a selected invocation against the complete runtime signature.
    /// Exact signatures must match their structural slot count and positional
    /// decomposition. Rest signatures validate their fixed prefix and capture
    /// mode through `RestLayout`; setter values remain a separate lane.
    pub(crate) fn validate_for_signature(&self, signature: &Signature) -> Result<(), RuntimeError> {
        Self::validate_signature_shape(signature, self.structural_positionals, &self.labels, self.setter_value)
    }

    /// Shared shape validator used both by invocation layouts and rest-family
    /// lookup, so candidate selection and activation enforce the same contract.
    pub(crate) fn validate_signature_shape(
        signature: &Signature,
        structural_positionals: usize,
        labels: &[Symbol],
        setter_value: bool,
    ) -> Result<(), RuntimeError> {
        if let Some(rest) = signature.rest.as_ref() {
            let SignatureKind::Method(structural_slots) = signature.kind else {
                return Err(RuntimeError::Internal(format!(
                    "rest signature has non-method selector kind {:?}",
                    signature.kind
                )));
            };
            let fixed_structural_slots = rest.fixed_positionals() as usize + rest.fixed_labels().len();
            if structural_slots as usize != fixed_structural_slots || signature.positional_arity as usize != rest.fixed_positionals() as usize {
                return Err(RuntimeError::Internal(format!(
                    "rest signature metadata disagrees with fixed layout: kind={:?}, positional arity={}, fixed positionals={}, fixed labels={}",
                    signature.kind,
                    signature.positional_arity,
                    rest.fixed_positionals(),
                    rest.fixed_labels().len()
                )));
            }
            if setter_value || !rest.accepts(structural_positionals, labels) {
                return Err(RuntimeError::Arity {
                    signature: "call",
                    expected: signature.positional_arity as usize,
                    found: structural_positionals + labels.len() + usize::from(setter_value),
                });
            }
            return Ok(());
        }

        Self::validate_parts_for_kind(signature.kind, structural_positionals, labels.len(), setter_value)?;

        let setter_lane = u8::from(matches!(signature.kind, SignatureKind::Setter | SignatureKind::SubscriptSet(_)));
        let expected_positionals = signature
            .positional_arity
            .checked_sub(setter_lane)
            .ok_or_else(|| RuntimeError::Internal(format!("signature positional arity {} omits its setter value lane", signature.positional_arity)))?
            as usize;
        let structural_slots = match signature.kind {
            SignatureKind::Method(slots) | SignatureKind::SubscriptGet(slots) | SignatureKind::SubscriptSet(slots) => slots as usize,
            SignatureKind::Getter | SignatureKind::Setter => 0,
        };
        let expected_labels = structural_slots.checked_sub(expected_positionals).ok_or_else(|| {
            RuntimeError::Internal(format!(
                "signature positional arity {} exceeds its {structural_slots} structural slots",
                signature.positional_arity
            ))
        })?;
        if structural_positionals != expected_positionals || labels.len() != expected_labels {
            return Err(RuntimeError::Internal(format!(
                "invocation layout has {structural_positionals} positional and {} labeled slots for signature {:?} with {expected_positionals} positional and {expected_labels} labeled slots",
                labels.len(),
                signature.kind
            )));
        }
        Ok(())
    }

    /// Validates that this layout describes exactly the stack window beginning
    /// at `receiver_index`. Invocation windows are topmost when dispatch starts.
    pub(crate) fn validate_window(&self, receiver_index: usize, stack_len: usize) -> Result<(), RuntimeError> {
        let expected_end = receiver_index
            .checked_add(1)
            .and_then(|start| start.checked_add(self.physical_arity()))
            .ok_or_else(|| RuntimeError::Internal("invocation window bounds overflow".into()))?;
        if expected_end != stack_len {
            return Err(RuntimeError::Internal(format!(
                "invocation layout expects stack window [{receiver_index}..{expected_end}), stack ends at {stack_len}"
            )));
        }
        Ok(())
    }
}

/// Runtime access origin representing the defining module, owning package, and project.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct RuntimeAccessOrigin {
    pub module: Option<ObjRef>,
    pub package: Option<ObjRef>,
    pub project: Option<phalcom_modules::ProjectIdentity>,
}

impl RuntimeAccessOrigin {
    pub const fn universe() -> Self {
        Self {
            module: None,
            package: None,
            project: Some(phalcom_modules::ProjectIdentity::Universe),
        }
    }

    pub const fn none() -> Self {
        Self {
            module: None,
            package: None,
            project: None,
        }
    }
}

/// A compact, non-borrowing view of the current argument window.
///
/// `labels` is always pre-decoded: it is populated at the call site from the
/// already-interned selector constituents, so the Family hot path never has to
/// call `decode_selector()` to recover them. The lane counts and offsets are
/// delegated to [`InvocationLayout`], which keeps setter values out of the
/// ordinary positional lane.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArgumentView {
    receiver_index: usize,
    layout: InvocationLayout,
    caller_access: Option<ClassId>,
    caller_origin: RuntimeAccessOrigin,
}

#[cfg(test)]
mod tests {
    use super::{ArgumentView, InvocationLayout, RuntimeAccessOrigin};
    use crate::interner::Symbol;
    use crate::value::Value;
    use crate::vm::VM;

    #[test]
    fn argument_view_keeps_labeled_and_setter_offsets_independent() {
        let mut vm = VM::new();
        vm.stack = vec![Value::nil(), Value::int(10), Value::int(20), Value::bool(true), Value::int(99)];
        let view = ArgumentView::from_layout(0, InvocationLayout::setter(2, Box::new([Symbol(7)])), None, RuntimeAccessOrigin::none());

        assert_eq!(view.positional(&vm, 0), Some(Value::int(10)));
        assert_eq!(view.positional(&vm, 1), Some(Value::int(20)));
        assert_eq!(view.labeled_value(&vm, 0), Some(Value::bool(true)));
        assert_eq!(view.setter_value(&vm), Some(Value::int(99)));
        assert_eq!(view.physical_arity(), 4);
    }
}

impl ArgumentView {
    /// Describes an ordinary selector-shaped window at `receiver_index`.
    pub(crate) fn ordinary_window(
        receiver_index: usize,
        structural_positionals: usize,
        labels: Box<[Symbol]>,
        caller_access: Option<ClassId>,
        caller_origin: RuntimeAccessOrigin,
    ) -> Self {
        Self::from_layout(
            receiver_index,
            InvocationLayout::ordinary(structural_positionals, labels),
            caller_access,
            caller_origin,
        )
    }

    /// Describes a setter-shaped window at `receiver_index`.
    pub(crate) fn setter_window(
        receiver_index: usize,
        structural_positionals: usize,
        labels: Box<[Symbol]>,
        caller_access: Option<ClassId>,
        caller_origin: RuntimeAccessOrigin,
    ) -> Self {
        Self::from_layout(
            receiver_index,
            InvocationLayout::setter(structural_positionals, labels),
            caller_access,
            caller_origin,
        )
    }

    /// Builds a view from an explicit invocation layout.
    pub(crate) fn from_layout(receiver_index: usize, layout: InvocationLayout, caller_access: Option<ClassId>, caller_origin: RuntimeAccessOrigin) -> Self {
        Self {
            receiver_index,
            layout,
            caller_access,
            caller_origin,
        }
    }

    /// Number of positional values in the argument lane.
    pub fn positional_count(&self) -> usize {
        self.layout.structural_positionals()
    }

    /// Number of labeled values in the argument lane.
    pub fn labeled_count(&self) -> usize {
        self.layout.labeled_count()
    }

    /// Returns positional value `index`.
    pub fn positional(&self, vm: &VM, index: usize) -> Option<Value> {
        if index >= self.positional_count() {
            return None;
        }
        self.receiver_index
            .checked_add(1)?
            .checked_add(index)
            .and_then(|offset| vm.stack.get(offset).copied())
    }

    /// Returns labeled value `index` in label order.
    pub fn labeled_value(&self, vm: &VM, index: usize) -> Option<Value> {
        if index >= self.labeled_count() {
            return None;
        }
        self.receiver_index
            .checked_add(1)?
            .checked_add(self.positional_count())?
            .checked_add(index)
            .and_then(|offset| vm.stack.get(offset).copied())
    }

    /// Returns the distinguished setter value, when this is a setter window.
    pub fn setter_value(&self, vm: &VM) -> Option<Value> {
        if !self.layout.has_setter_value() {
            return None;
        }
        self.receiver_index
            .checked_add(self.layout.physical_arity())
            .and_then(|offset| vm.stack.get(offset).copied())
    }

    /// Returns the label for labeled lane position `index`.
    pub fn label(&self, index: usize) -> Option<Symbol> {
        self.layout.labels().get(index).copied()
    }

    /// Returns pre-decoded label symbols in call order.
    ///
    /// Never calls `decode_selector()`; labels were decoded once at call-site
    /// construction and cached in this view.
    pub fn labels(&self) -> &[Symbol] {
        self.layout.labels()
    }

    /// Returns the invocation lane metadata backing this view.
    pub(crate) fn layout(&self) -> &InvocationLayout {
        &self.layout
    }

    /// Returns whether this view carries a distinguished setter value.
    pub fn has_setter_value(&self) -> bool {
        self.layout.has_setter_value()
    }

    /// Returns the physical argument arity after the receiver.
    pub fn physical_arity(&self) -> usize {
        self.layout.physical_arity()
    }

    /// Returns the source stack receiver index for VM activation helpers.
    pub(crate) fn receiver_index(&self) -> usize {
        self.receiver_index
    }

    /// Returns the caller authority captured before entering the native gateway.
    pub(crate) fn caller_authority(&self) -> (Option<ClassId>, RuntimeAccessOrigin) {
        (self.caller_access, self.caller_origin)
    }

    /// Returns a view with a newly resolved selector and invocation layout.
    pub(crate) fn with_layout(self, layout: InvocationLayout) -> Self {
        Self {
            receiver_index: self.receiver_index,
            layout,
            caller_access: self.caller_access,
            caller_origin: self.caller_origin,
        }
    }

    /// Reclassifies a unary ordinary method window as a named setter window.
    ///
    /// `Family.set(_)` receives one ordinary positional argument at its public
    /// protocol boundary. The captured setter receives the same physical value
    /// as its dedicated trailing lane, so this operation changes metadata only.
    pub(crate) fn reclassify_unary_method_as_setter(mut self) -> Result<Self, RuntimeError> {
        if self.has_setter_value() || self.positional_count() != 1 || self.labeled_count() != 0 {
            return Err(RuntimeError::Internal("Family.set(_) requires one ordinary positional value".into()));
        }
        self.layout = InvocationLayout::setter(0, Box::default());
        Ok(self)
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
    /// Lexical source class controlling private/protected access.
    pub access_owner: Option<ClassId>,
    /// Runtime access origin for module/package/internal visibility.
    pub origin: RuntimeAccessOrigin,
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
    /// Builds a method with its complete calling signature.
    pub fn new(signature: Signature, kind: MethodKind, holder: Option<ClassId>) -> Self {
        MethodObject {
            kind,
            signature,
            holder,
            visibility: super::MemberVisibility::Public,
            access_owner: holder,
            origin: RuntimeAccessOrigin::none(),
            contracts: None,
            attributes: Vec::new(),
            attributes_frozen: false,
        }
    }

    /// Builds an unbound method (holder `None`), typically a compiler-produced
    /// closure that is attached to its class later by `Bytecode::Method`.
    pub fn new_single(signature: Signature, kind: MethodKind) -> Self {
        Self::new(signature, kind, None)
    }

    /// Builds a holderless method whose structural selector slots are all
    /// positional. Labeled methods should pass an explicit [`Signature`].
    pub fn new_single_positional(selector: Symbol, sig_kind: SignatureKind, kind: MethodKind) -> Self {
        Self::new_single(Signature::positional(selector, sig_kind), kind)
    }

    /// Builds a native primitive method held by `holder`.
    pub fn new_primitive(signature: Signature, primitive: LegacyPrimitiveFn, holder: ClassId) -> Self {
        Self::new(signature, MethodKind::Primitive(PrimitiveFn::Legacy(primitive)), Some(holder))
    }

    /// Builds a legacy primitive method from an all-positional selector shape.
    pub fn new_positional_primitive(selector: Symbol, sig_kind: SignatureKind, primitive: LegacyPrimitiveFn, holder: ClassId) -> Self {
        Self::new_primitive(Signature::positional(selector, sig_kind), primitive, holder)
    }

    /// Builds a method using the shape-aware native ABI and explicit signature.
    pub fn new_shape_primitive(
        selector: Symbol,
        signature: Signature,
        primitive: fn(&mut VM, Value, ArgumentView) -> PhResult<CallOutcome>,
        holder: ClassId,
    ) -> Self {
        debug_assert_eq!(selector, signature.selector);
        Self::new(signature, MethodKind::Primitive(PrimitiveFn::Shape(primitive)), Some(holder))
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
