use crate::error::{PhResult, RuntimeError};
use crate::heap::{ClassId, ObjRef, Object};
use crate::interner::Symbol;
use crate::method::{ArgumentView, CallOutcome, MemberVisibility, MethodKind, PrimitiveFn, SignatureKind, decode_selector};
use crate::value::Value;
use indexmap::IndexMap;
use phalcom_common::range::SourceRange;
use phalcom_common::selector::{SelectorBase, SelectorKind, SelectorKindPattern};

use super::VM;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum FamilyInvocationKind {
    Method,
    Getter,
    Setter,
}

impl VM {
    /// Captures the currently effective methods selected by `pattern`.
    ///
    /// This is deliberately a VM operation rather than primitive-local lookup:
    /// it walks live method dictionaries, applies the same visibility relation
    /// used by reflection, and snapshots both exact bindings and rest fallback
    /// candidates without invoking the receiver or `doesNotUnderstand`.
    pub(crate) fn capture_method_family(
        &mut self,
        behavior: ClassId,
        pattern_id: ObjRef,
        caller_authority: (Option<ClassId>, bool),
    ) -> PhResult<crate::heap::MethodFamilyObject> {
        let pattern = match self.heap.get(pattern_id) {
            Object::SelectorPattern(pattern) => pattern.pattern.clone(),
            _ => {
                return Err(RuntimeError::Type {
                    expected: "SelectorPattern",
                    found: "object",
                }
                .into());
            }
        };

        let mut hierarchy = Vec::new();
        let mut current = Some(behavior);
        while let Some(class) = current {
            hierarchy.push(class);
            current = self.heap.class(class).superclass;
        }

        let mut exact_methods = IndexMap::new();
        let mut seen_selectors = std::collections::HashSet::new();
        for class in &hierarchy {
            let methods = self.heap.class(*class).methods.values().copied().collect::<Vec<_>>();
            for method in methods {
                let selector = self.heap.method(method).signature.selector;
                if !seen_selectors.insert(selector) {
                    continue;
                }
                if self.heap.method(method).signature.rest.is_some() {
                    continue;
                }
                let structural = phalcom_common::selector::Selector::decode(self.resolve_symbol(selector));
                if pattern.matches(&structural) && self.authorize_method_access_as(method, caller_authority.0, caller_authority.1).is_ok() {
                    exact_methods.insert(selector, method);
                }
            }
        }

        let mut rest_candidates = Vec::new();
        if let phalcom_common::selector::SelectorBase::Named(base) = &pattern.base
            && matches!(
                pattern.kind,
                phalcom_common::selector::SelectorKindPattern::AnyNamed
                    | phalcom_common::selector::SelectorKindPattern::Exact(phalcom_common::selector::SelectorKind::Method)
            )
        {
            let base = self.interner.intern(base);
            let mut seen = std::collections::HashSet::new();
            for class in &hierarchy {
                let Some(method) = self.heap.class(*class).get_rest_method(base) else {
                    continue;
                };
                if seen.insert(method) && self.authorize_method_access_as(method, caller_authority.0, caller_authority.1).is_ok() {
                    rest_candidates.push(method);
                }
            }
        }

        Ok(crate::heap::MethodFamilyObject {
            source_behavior: behavior,
            pattern: pattern_id,
            exact_methods,
            rest_candidates: rest_candidates.into_boxed_slice(),
        })
    }

    /// Checks whether `receiver` can execute a method held by its defining
    /// class. Class-side methods use the receiver's metaclass automatically
    /// because `Value::class` returns that class for class values.
    pub(crate) fn method_receiver_nominally_compatible(&self, method: ObjRef, receiver: Value) -> bool {
        let Some(holder) = self.heap.method(method).holder else {
            return true;
        };
        let mut class = receiver.class(self);
        loop {
            if class == holder {
                return true;
            }
            let Some(superclass) = self.heap.class(class).superclass else {
                return false;
            };
            class = superclass;
        }
    }

    /// Returns the lexical authority of code currently executing in this VM.
    /// Blocks carry their defining method's source class on their closure, so
    /// this remains stable across nested closure calls.
    pub(crate) fn current_access_class(&self) -> Option<ClassId> {
        if let Some(context) = self.native_method_contexts.last() {
            return context.access_owner;
        }
        self.frames.last().and_then(|frame| self.heap.closure(frame.closure).lexical_class)
    }

    /// True only while executing code compiled from the bootstrap core module.
    /// Module-handle identity, rather than a mutable source name, is the
    /// authority boundary. Nested blocks retain their defining module.
    pub(crate) fn current_has_internal_privilege(&self) -> bool {
        if self.compiler_internal_dispatch_depth != 0 {
            return true;
        }
        if let Some(context) = self.native_method_contexts.last() {
            return context.internal;
        }
        let Some(frame) = self.frames.last() else {
            return false;
        };
        let closure_module = self.heap.closure(frame.closure).module;
        self.is_privileged_core_module(closure_module)
    }

    fn is_subclass_of(&self, mut class: ClassId, ancestor: ClassId) -> bool {
        loop {
            if class == ancestor {
                return true;
            }
            match self.heap.class(class).superclass {
                Some(parent) => class = parent,
                None => return false,
            }
        }
    }

    pub(crate) fn guard_foreign_layout_access(&self, receiver: Value, guard: crate::frame::ForeignReceiverGuard) -> PhResult<()> {
        let compatible = if let Some(id) = receiver.as_obj() {
            if self.heap.as_instance(id).is_some() {
                self.is_subclass_of(self.heap.instance(id).class, guard.layout_owner)
            } else if self.heap.as_class(id).is_some() {
                self.is_subclass_of(self.heap.class(id).class, guard.layout_owner)
            } else {
                false
            }
        } else {
            false
        };
        if compatible {
            return Ok(());
        }

        let required = self.heap.class(guard.layout_owner).name.clone();
        let found = if let Some(id) = receiver.as_obj() {
            if self.heap.as_instance(id).is_some() {
                self.heap.class(self.heap.instance(id).class).name.clone()
            } else if self.heap.as_class(id).is_some() {
                self.heap.class(id).name.clone()
            } else {
                receiver.type_name().to_owned()
            }
        } else {
            receiver.type_name().to_owned()
        };
        Err(RuntimeError::IncompatibleMethodLayout {
            selector: self.resolve_symbol(guard.selector).to_owned(),
            required,
            found,
        }
        .into())
    }

    /// Enforces member visibility for every invocation path. Lookup remains
    /// separate: an existing inaccessible selector is an access violation,
    /// never a `doesNotUnderstand` miss.
    pub(crate) fn authorize_method_access(&self, method: ObjRef) -> PhResult<()> {
        self.authorize_method_access_as(method, self.current_access_class(), self.current_has_internal_privilege())
    }

    /// Enforces visibility against an explicitly captured caller authority.
    ///
    /// Forwarding gateways use this instead of the gateway primitive's own
    /// lexical authority: `bound.call(...)` is authorized by the code that
    /// called `bound`, while sends made *inside* a primitive use its native
    /// method context.
    pub(crate) fn authorize_method_access_as(&self, method: ObjRef, caller: Option<ClassId>, caller_internal: bool) -> PhResult<()> {
        let (visibility, owner, selector) = {
            let method = self.heap.method(method);
            (method.visibility, method.access_owner, method.signature.selector)
        };
        let allowed = match visibility {
            MemberVisibility::Public => true,
            MemberVisibility::Private => caller == owner,
            MemberVisibility::Protected => owner.is_some_and(|owner| caller.is_some_and(|caller| self.is_subclass_of(caller, owner))),
            MemberVisibility::Internal => caller_internal,
        };
        if allowed {
            return Ok(());
        }

        let category = match visibility {
            MemberVisibility::Private => "member.private_access",
            MemberVisibility::Protected => "member.protected_access",
            MemberVisibility::Internal => "internal.selector_access",
            MemberVisibility::Public => unreachable!("public methods are authorized"),
        };
        let selector = self.resolve_symbol(selector);
        let owner = owner.map(|id| self.heap.class(id).name.as_str()).unwrap_or("<unbound>");
        let caller = caller.map(|id| self.heap.class(id).name.as_str()).unwrap_or("<top-level>");
        Err(RuntimeError::NotAllowed(format!("{category}: `{selector}` is owned by `{owner}` and cannot be called from `{caller}`")).into())
    }

    /// Dispatches a call to `method` on `callee` with `arity` arguments.
    ///
    /// A primitive runs its native function in place; a closure pushes a new
    /// [`crate::frame::CallFrame`] to be executed by [`Self::run`].
    ///
    /// # Errors
    ///
    /// Propagates errors returned by a primitive implementation.
    fn call_method_legacy(
        &mut self,
        callee: &Value,
        method: ObjRef,
        arity: usize,
        source_range: SourceRange,
        caller_authority: (Option<ClassId>, bool),
    ) -> PhResult<()> {
        self.authorize_method_access_as(method, caller_authority.0, caller_authority.1)?;
        let kind = self.heap.method(method).kind;
        match kind {
            MethodKind::Primitive(PrimitiveFn::Legacy(native_fn)) => {
                let receiver_idx = self.stack.len() - 1 - arity;
                let receiver = self.stack[receiver_idx];

                let method_obj = self.heap.method(method);
                let selector_sym = method_obj.signature.selector;
                let class_name_str = if let Some(holder_id) = method_obj.holder {
                    self.heap.class(holder_id).name.clone()
                } else {
                    let class_id = callee.class(self);
                    self.heap.class(class_id).name.clone()
                };
                let class_sym = self.interner.intern(&class_name_str);
                self.native_selector = Some(selector_sym);
                self.native_class = Some(class_sym);
                let native_context = crate::vm::NativeMethodContext {
                    access_owner: method_obj.access_owner.or(method_obj.holder),
                    internal: true,
                };
                let frames_before = self.frames.len();
                self.switch_pending = false;
                const INLINE_ARGS: usize = 8;
                self.native_method_contexts.push(native_context);
                let result = if arity <= INLINE_ARGS {
                    let mut args = [Value::nil(); INLINE_ARGS];
                    args[..arity].copy_from_slice(&self.stack[receiver_idx + 1..]);
                    native_fn(self, &receiver, &args[..arity])
                } else {
                    let args: Vec<Value> = self.stack[receiver_idx + 1..].to_vec();
                    native_fn(self, &receiver, &args)
                };
                self.native_method_contexts.pop();
                if result.is_ok() {
                    self.native_selector = None;
                    self.native_class = None;
                }
                result.map(|result| {
                    if self.switch_pending {
                        self.switch_pending = false;
                    } else if self.frames.len() >= frames_before {
                        self.stack.truncate(receiver_idx);
                        self.stack.push(result);
                    } else {
                        self.stack.push(result);
                    }
                })
            }
            MethodKind::Primitive(PrimitiveFn::Shape(_)) => Err(RuntimeError::Internal("shape-aware primitive reached legacy activation".into()).into()),
            MethodKind::Closure(closure_id) => {
                let context = callee.to_context(&self.heap);
                let receiver_idx = self.stack.len() - arity - 1;
                let stack_offset = receiver_idx;
                let new_frame = self.new_call_frame(closure_id, context, 0, stack_offset, Some(source_range));
                self.push_frame(new_frame)?;
                Ok(())
            }
        }
    }

    /// Dispatches one selected method using the current call-site shape.
    ///
    /// The existing legacy primitives still use [`Self::call_method_legacy`].
    /// Shape-aware gateways enter through this path, which is the only path
    /// allowed to return `EnteredFrame` from a native primitive.
    pub(super) fn call_method_with_selector(
        &mut self,
        callee: &Value,
        method: ObjRef,
        arity: usize,
        selector: Symbol,
        shape: Option<(usize, usize)>,
        source_range: SourceRange,
    ) -> PhResult<()> {
        let caller_authority = (self.current_access_class(), self.current_has_internal_privilege());
        self.call_method_with_selector_as(callee, method, arity, selector, shape, source_range, caller_authority)
    }

    // Keep activation inputs explicit: each field maps to a distinct VM
    // invariant (receiver window, selector shape, source span, authority).
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn call_method_with_selector_as(
        &mut self,
        callee: &Value,
        method: ObjRef,
        arity: usize,
        selector: Symbol,
        shape: Option<(usize, usize)>,
        source_range: SourceRange,
        caller_authority: (Option<ClassId>, bool),
    ) -> PhResult<()> {
        if matches!(self.heap.method(method).kind, MethodKind::Primitive(PrimitiveFn::Legacy(_))) {
            return self.call_method_legacy(callee, method, arity, source_range, caller_authority);
        }

        self.authorize_method_access_as(method, caller_authority.0, caller_authority.1)?;
        let receiver_idx = self.stack.len() - arity - 1;
        let receiver = self.stack[receiver_idx];
        match self.heap.method(method).kind {
            MethodKind::Closure(closure_id) => {
                let context = callee.to_context(&self.heap);
                let frame = self.new_call_frame(closure_id, context, 0, receiver_idx, Some(source_range));
                self.push_frame(frame)?;
                Ok(())
            }
            MethodKind::Primitive(PrimitiveFn::Shape(native_fn)) => {
                let view = match shape {
                    Some((positionals, labeled_count)) => {
                        let labels = if labeled_count > 0 {
                            let (_, slots, _) = crate::method::decode_selector(self.resolve_symbol(selector));
                            slots
                                .into_iter()
                                .filter_map(|slot| slot.map(|label| self.interner.intern(&label)))
                                .collect::<Vec<_>>()
                                .into_boxed_slice()
                        } else {
                            Box::default()
                        };
                        ArgumentView::shaped_with_labels(receiver_idx, positionals, labels, selector, caller_authority.0, caller_authority.1)
                    }
                    None => ArgumentView::positional_window(receiver_idx, arity, caller_authority.0, caller_authority.1),
                };
                let method_obj = self.heap.method(method);
                let selector_sym = method_obj.signature.selector;
                let class_name = method_obj
                    .holder
                    .map(|holder| self.heap.class(holder).name.clone())
                    .unwrap_or_else(|| self.heap.class(callee.class(self)).name.clone());
                self.native_selector = Some(selector_sym);
                self.native_class = Some(self.interner.intern(&class_name));
                self.native_method_contexts.push(crate::vm::NativeMethodContext {
                    access_owner: method_obj.access_owner.or(method_obj.holder),
                    internal: true,
                });
                self.switch_pending = false;
                let frames_before = self.frames.len();
                let result = native_fn(self, receiver, view);
                self.native_method_contexts.pop();
                match result? {
                    CallOutcome::EnteredFrame => Ok(()),
                    CallOutcome::Returned(value) => {
                        if self.switch_pending {
                            self.switch_pending = false;
                        } else if self.frames.len() >= frames_before {
                            self.stack.truncate(receiver_idx);
                            self.stack.push(value);
                        } else {
                            self.stack.push(value);
                        }
                        self.native_selector = None;
                        self.native_class = None;
                        Ok(())
                    }
                }
            }
            MethodKind::Primitive(PrimitiveFn::Legacy(_)) => unreachable!("legacy primitive returned above"),
        }
    }

    /// Compatibility entry point for ordinary exact sends.
    pub(super) fn call_method(&mut self, callee: &Value, method: ObjRef, arity: usize, source_range: SourceRange) -> PhResult<()> {
        let selector = self.heap.method(method).signature.selector;
        self.call_method_with_selector(callee, method, arity, selector, None, source_range)
    }

    /// Calls a selected method and reports whether it pushed a bytecode frame.
    /// Native forwarding gateways use this to flatten their target activation.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn dispatch_selected_method_as(
        &mut self,
        callee: &Value,
        method: ObjRef,
        arity: usize,
        selector: Symbol,
        shape: Option<(usize, usize)>,
        source_range: SourceRange,
        caller_authority: (Option<ClassId>, bool),
    ) -> PhResult<CallOutcome> {
        let before = self.frames.len();
        self.call_method_with_selector_as(callee, method, arity, selector, shape, source_range, caller_authority)?;
        if self.frames.len() > before {
            Ok(CallOutcome::EnteredFrame)
        } else {
            Ok(CallOutcome::Returned(self.stack.last().copied().unwrap_or_else(Value::nil)))
        }
    }

    /// Dispatches the already-shaped stack window without entering a nested
    /// interpreter loop. This is the common target path for Function gateways,
    /// `perform`, and Family forwarding.
    pub(crate) fn dispatch_shape_at_as(
        &mut self,
        receiver_idx: usize,
        selector: Symbol,
        positional_count: usize,
        labels: &[Symbol],
        source_range: SourceRange,
        caller_authority: (Option<ClassId>, bool),
    ) -> PhResult<CallOutcome> {
        let receiver = self.stack[receiver_idx];
        let before = self.frames.len();
        if let Some(method) = receiver.lookup_method(self, selector) {
            self.call_method_with_selector_as(
                &receiver,
                method,
                positional_count + labels.len(),
                selector,
                Some((positional_count, labels.len())),
                source_range,
                caller_authority,
            )?;
        } else {
            let (name, _, kind) = decode_selector(self.resolve_symbol(selector));
            let rest = matches!(kind, SignatureKind::Method(_))
                .then(|| self.interner.intern(&name))
                .and_then(|base| self.lookup_rest_method(receiver.class(self), base, positional_count, labels));
            if let Some(method) = rest {
                self.activate_rest_method_as(
                    &receiver,
                    method,
                    receiver_idx,
                    positional_count,
                    labels,
                    selector,
                    source_range,
                    caller_authority,
                )?;
            } else {
                self.forward_does_not_understand_as(receiver_idx, selector, source_range, caller_authority)?;
            }
        }
        if self.frames.len() > before {
            Ok(CallOutcome::EnteredFrame)
        } else {
            Ok(CallOutcome::Returned(self.stack.last().copied().unwrap_or_else(Value::nil)))
        }
    }

    /// Activates the concrete representation behind the sealed Function
    /// hierarchy. The current stack window is reused in place; no message or
    /// argument vector is created for ordinary calls.
    pub(crate) fn activate_function(&mut self, receiver: Value, view: ArgumentView, source_range: SourceRange) -> PhResult<CallOutcome> {
        let Some(id) = receiver.as_obj() else {
            return Err(RuntimeError::Type {
                expected: "Function",
                found: receiver.type_name(),
            }
            .into());
        };
        match self.heap.get(id) {
            Object::Block(block) => self.activate_closure_call(receiver, block.closure, Some(block.home_frame_token), view, source_range),
            Object::Closure(_) => self.activate_closure_call(receiver, id, None, view, source_range),
            Object::BoundMethod(bound) => self.activate_bound_method(*bound, view, source_range),
            Object::Family(_) => self.activate_family_with_kind(view, FamilyInvocationKind::Method, source_range),
            Object::BoundMethodFamily(bound) => self.activate_bound_method_family(*bound, view, source_range),
            _ => Err(RuntimeError::Type {
                expected: "Function",
                found: receiver.type_name(),
            }
            .into()),
        }
    }

    fn activate_closure_call(
        &mut self,
        receiver: Value,
        closure_id: ObjRef,
        home_frame_token: Option<crate::frame::FrameToken>,
        view: ArgumentView,
        source_range: SourceRange,
    ) -> PhResult<CallOutcome> {
        if view.labeled_count() != 0 {
            return Err(RuntimeError::Arity {
                signature: "call",
                expected: view.positional_count(),
                found: view.positional_count() + view.labeled_count(),
            }
            .into());
        }
        let shape = self.heap.closure(closure_id).callable.parameter_shape.clone();
        let positional_count = view.positional_count();
        if positional_count < shape.fixed_positionals || (shape.rest.is_none() && positional_count != shape.fixed_positionals) {
            return Err(RuntimeError::Arity {
                signature: "call",
                expected: shape.fixed_positionals,
                found: positional_count,
            }
            .into());
        }
        let receiver_idx = view.receiver_index();
        if shape.rest.is_some() {
            let residual = (shape.fixed_positionals..positional_count)
                .filter_map(|index| view.positional(self, index))
                .collect::<Vec<_>>();
            let capture = crate::product::finish_tuple(self, residual, Vec::new())
                .map_err(|error| RuntimeError::Internal(format!("closure rest capture failed: {error:?}")))?;
            self.stack.truncate(receiver_idx + 1 + shape.fixed_positionals);
            self.stack.push(capture);
        } else {
            self.stack.truncate(receiver_idx + 1 + shape.fixed_positionals);
        }
        let context = crate::frame::CallContext::Instance {
            instance: receiver.as_obj().expect("Function representations are heap values"),
        };
        let mut frame = self.new_call_frame(closure_id, context, 0, receiver_idx, Some(source_range));
        frame.home_frame_token = home_frame_token;
        frame.foreign_receiver_guard = self.heap.closure(closure_id).foreign_receiver_guard;
        self.push_frame(frame)?;
        Ok(CallOutcome::EnteredFrame)
    }

    pub(crate) fn validate_captured_method_shape(&mut self, method: ObjRef, positional_count: usize, actual_labels: &[Symbol]) -> PhResult<Symbol> {
        let method_selector = self.heap.method(method).signature.selector;
        let selector_text = self.resolve_symbol(method_selector).to_owned();
        let (base, expected_slots, _) = decode_selector(&selector_text);
        let signature_kind = self.heap.method(method).signature.kind;
        let (expected_positional, expected_labels) = match signature_kind {
            SignatureKind::Getter => (0, Vec::new()),
            SignatureKind::Setter => (1, Vec::new()),
            SignatureKind::SubscriptGet(_) => (
                expected_slots.iter().filter(|slot| slot.is_none()).count(),
                expected_slots
                    .iter()
                    .filter_map(|slot| slot.as_ref())
                    .map(|label| self.interner.intern(label))
                    .collect(),
            ),
            SignatureKind::SubscriptSet(_) => (
                expected_slots.iter().filter(|slot| slot.is_none()).count() + 1,
                expected_slots
                    .iter()
                    .filter_map(|slot| slot.as_ref())
                    .map(|label| self.interner.intern(label))
                    .collect(),
            ),
            SignatureKind::Method(_) => (
                expected_slots.iter().filter(|slot| slot.is_none()).count(),
                expected_slots
                    .iter()
                    .filter_map(|slot| slot.as_ref())
                    .map(|label| self.interner.intern(label))
                    .collect(),
            ),
        };
        let rest_layout = self.heap.method(method).signature.rest.clone();
        if let Some(rest) = rest_layout.as_ref() {
            let mut slots = Vec::with_capacity(positional_count + actual_labels.len());
            slots.extend(std::iter::repeat_n(None, positional_count));
            slots.extend(actual_labels.iter().map(|label| Some(self.resolve_symbol(*label).to_owned())));
            let selector = self.get_or_intern(&crate::method::encode_selector(
                &base,
                &slots,
                SignatureKind::Method(
                    u8::try_from(positional_count + actual_labels.len()).map_err(|_| RuntimeError::SendArityExceedsLimit {
                        found: positional_count + actual_labels.len(),
                        limit: u8::MAX as usize,
                    })?,
                ),
            ));
            if !rest.accepts(positional_count, actual_labels) {
                return Err(RuntimeError::Arity {
                    signature: "call",
                    expected: expected_positional,
                    found: positional_count + actual_labels.len(),
                }
                .into());
            }
            return Ok(selector);
        }

        if expected_positional != positional_count || expected_labels != actual_labels {
            return Err(RuntimeError::Arity {
                signature: "call",
                expected: expected_positional,
                found: positional_count + actual_labels.len(),
            }
            .into());
        }
        Ok(method_selector)
    }

    pub(crate) fn activate_captured_method_as(
        &mut self,
        receiver: Value,
        method: ObjRef,
        view: ArgumentView,
        source_range: SourceRange,
    ) -> PhResult<CallOutcome> {
        self.authorize_method_access_as(method, view.caller_authority().0, view.caller_authority().1)?;
        let method_selector = self.heap.method(method).signature.selector;
        let actual_labels = view.labels();
        let actual_selector = self.validate_captured_method_shape(method, view.positional_count(), actual_labels)?;
        let receiver_idx = view.receiver_index();
        self.stack[receiver_idx] = receiver;
        let total = view.positional_count() + view.labeled_count();
        let before = self.frames.len();
        let outcome =
            if self.heap.method(method).signature.rest.is_some() && !matches!(self.heap.method(method).kind, MethodKind::Primitive(PrimitiveFn::Shape(_))) {
                self.call_rest_method_as(
                    &receiver,
                    method,
                    receiver_idx,
                    view.positional_count(),
                    actual_labels,
                    source_range,
                    view.caller_authority(),
                )?;
                if self.frames.len() > before {
                    CallOutcome::EnteredFrame
                } else {
                    CallOutcome::Returned(self.stack.last().copied().unwrap_or_else(Value::nil))
                }
            } else {
                self.dispatch_selected_method_as(
                    &receiver,
                    method,
                    total,
                    actual_selector,
                    Some((view.positional_count(), view.labeled_count())),
                    source_range,
                    view.caller_authority(),
                )?
            };

        if self.frames.len() > before
            && matches!(self.heap.method(method).kind, MethodKind::Closure(_))
            && !self.method_receiver_nominally_compatible(method, receiver)
            && let Some(layout_owner) = self.heap.method(method).holder
            && let Some(frame) = self.frames.last_mut()
        {
            frame.foreign_receiver_guard = Some(crate::frame::ForeignReceiverGuard {
                layout_owner,
                selector: method_selector,
            });
        }
        Ok(outcome)
    }

    fn activate_bound_method(&mut self, bound: crate::heap::BoundMethodObject, view: ArgumentView, source_range: SourceRange) -> PhResult<CallOutcome> {
        self.activate_captured_method_as(bound.receiver, bound.method, view, source_range)
    }

    fn selectors_for_bound_method_family(&mut self, pattern_id: ObjRef, view: ArgumentView) -> PhResult<Vec<(Symbol, usize, Vec<Symbol>)>> {
        let pattern = match self.heap.get(pattern_id) {
            Object::SelectorPattern(pattern) => pattern.pattern.clone(),
            _ => return Err(RuntimeError::Internal("MethodFamily pattern handle is not a selector pattern".into()).into()),
        };
        let labels = view.labels().to_vec();
        let mut candidates = Vec::new();
        match (&pattern.base, &pattern.kind) {
            (SelectorBase::Named(base), SelectorKindPattern::AnyNamed) => {
                let mut slots = Vec::with_capacity(view.positional_count() + labels.len());
                slots.extend(std::iter::repeat_n(None, view.positional_count()));
                slots.extend(labels.iter().map(|label| Some(self.resolve_symbol(*label).to_owned())));
                let arity = u8::try_from(slots.len()).map_err(|_| RuntimeError::SendArityExceedsLimit {
                    found: slots.len(),
                    limit: u8::MAX as usize,
                })?;
                let method_selector = self.get_or_intern(&crate::method::encode_selector(base, &slots, SignatureKind::Method(arity)));
                let method_structural = phalcom_common::selector::Selector::decode(self.resolve_symbol(method_selector));
                if pattern.matches(&method_structural) {
                    candidates.push((method_selector, view.positional_count(), labels.clone()));
                }
                if view.positional_count() == 0 && labels.is_empty() {
                    let selector = self.get_or_intern(&crate::method::make_signature(base, SignatureKind::Getter));
                    let structural = phalcom_common::selector::Selector::decode(self.resolve_symbol(selector));
                    if pattern.matches(&structural) {
                        candidates.push((selector, 0, Vec::new()));
                    }
                } else if view.positional_count() == 1 && labels.is_empty() {
                    let selector = self.get_or_intern(&crate::method::make_signature(base, SignatureKind::Setter));
                    let structural = phalcom_common::selector::Selector::decode(self.resolve_symbol(selector));
                    if pattern.matches(&structural) {
                        candidates.push((selector, 1, Vec::new()));
                    }
                }
            }
            (SelectorBase::Named(base), SelectorKindPattern::Exact(SelectorKind::Method)) => {
                let mut slots = Vec::with_capacity(view.positional_count() + labels.len());
                slots.extend(std::iter::repeat_n(None, view.positional_count()));
                slots.extend(labels.iter().map(|label| Some(self.resolve_symbol(*label).to_owned())));
                let arity = u8::try_from(slots.len()).map_err(|_| RuntimeError::SendArityExceedsLimit {
                    found: slots.len(),
                    limit: u8::MAX as usize,
                })?;
                let selector = self.get_or_intern(&crate::method::encode_selector(base, &slots, SignatureKind::Method(arity)));
                let structural = phalcom_common::selector::Selector::decode(self.resolve_symbol(selector));
                if pattern.matches(&structural) {
                    candidates.push((selector, view.positional_count(), labels.clone()));
                }
            }
            (SelectorBase::Named(base), SelectorKindPattern::Exact(SelectorKind::Getter)) => {
                if view.positional_count() != 0 || !labels.is_empty() {
                    return Err(RuntimeError::Arity {
                        signature: "call",
                        expected: 0,
                        found: view.positional_count() + labels.len(),
                    }
                    .into());
                }
                let selector = self.get_or_intern(&crate::method::make_signature(base, SignatureKind::Getter));
                let structural = phalcom_common::selector::Selector::decode(self.resolve_symbol(selector));
                if pattern.matches(&structural) {
                    candidates.push((selector, 0, Vec::new()));
                }
            }
            (SelectorBase::Named(base), SelectorKindPattern::Exact(SelectorKind::Setter)) => {
                if view.positional_count() != 1 || !labels.is_empty() {
                    return Err(RuntimeError::Arity {
                        signature: "call",
                        expected: 1,
                        found: view.positional_count() + labels.len(),
                    }
                    .into());
                }
                let selector = self.get_or_intern(&crate::method::make_signature(base, SignatureKind::Setter));
                let structural = phalcom_common::selector::Selector::decode(self.resolve_symbol(selector));
                if pattern.matches(&structural) {
                    candidates.push((selector, 1, Vec::new()));
                }
            }
            (SelectorBase::Subscript, SelectorKindPattern::Exact(SelectorKind::SubscriptGet | SelectorKind::SubscriptSet)) => {
                let is_setter = matches!(&pattern.kind, SelectorKindPattern::Exact(SelectorKind::SubscriptSet));
                let setter_values = if is_setter { 1 } else { 0 };
                let slot_positionals = view.positional_count().checked_sub(setter_values).ok_or_else(|| RuntimeError::Arity {
                    signature: "call",
                    expected: setter_values,
                    found: view.positional_count(),
                })?;
                let mut slots = Vec::with_capacity(slot_positionals + labels.len());
                slots.extend(std::iter::repeat_n(None, slot_positionals));
                slots.extend(labels.iter().map(|label| Some(self.resolve_symbol(*label).to_owned())));
                let arity = u8::try_from(slots.len()).map_err(|_| RuntimeError::SendArityExceedsLimit {
                    found: slots.len(),
                    limit: u8::MAX as usize,
                })?;
                let kind = if is_setter {
                    SignatureKind::SubscriptSet(arity)
                } else {
                    SignatureKind::SubscriptGet(arity)
                };
                let selector = self.get_or_intern(&crate::method::encode_selector("[]", &slots, kind));
                let structural = phalcom_common::selector::Selector::decode(self.resolve_symbol(selector));
                if pattern.matches(&structural) {
                    candidates.push((selector, view.positional_count(), labels.clone()));
                }
            }
            _ => {
                return Err(RuntimeError::Message("captured MethodFamily selector pattern has incompatible base and kind".into()).into());
            }
        }
        if candidates.is_empty() {
            return Err(RuntimeError::Message("captured MethodFamily does not accept this call shape".into()).into());
        }
        Ok(candidates)
    }

    fn activate_bound_method_family(
        &mut self,
        bound: crate::heap::BoundMethodFamilyObject,
        view: ArgumentView,
        source_range: SourceRange,
    ) -> PhResult<CallOutcome> {
        let family = self.heap.method_family(bound.family).clone();
        for (selector, positional_count, labels) in self.selectors_for_bound_method_family(family.pattern, view.clone())? {
            let method = family.exact_methods.get(&selector).copied().or_else(|| {
                family.rest_candidates.iter().copied().find(|method| {
                    self.heap
                        .method(*method)
                        .signature
                        .rest
                        .as_ref()
                        .is_some_and(|rest| rest.accepts(positional_count, &labels))
                })
            });
            let Some(method) = method else {
                continue;
            };
            let shaped = view.with_selector(selector, positional_count, labels.into_boxed_slice());
            return self.activate_captured_method_as(bound.receiver, method, shaped, source_range);
        }
        Err(RuntimeError::Message("captured MethodFamily has no method for this call shape".into()).into())
    }

    pub(crate) fn activate_family_with_kind(
        &mut self,
        view: ArgumentView,
        invocation: FamilyInvocationKind,
        source_range: SourceRange,
    ) -> PhResult<CallOutcome> {
        let receiver_idx = view.receiver_index();
        let Some(family_id) = self.stack[receiver_idx].as_obj() else {
            return Err(RuntimeError::Type {
                expected: "Family",
                found: self.stack[receiver_idx].type_name(),
            }
            .into());
        };
        let family = match self.heap.get(family_id) {
            Object::Family(family) => *family,
            _ => {
                return Err(RuntimeError::Type {
                    expected: "Family",
                    found: self.stack[receiver_idx].type_name(),
                }
                .into());
            }
        };
        let labels = view.labels();
        let selector = match family.spec {
            crate::heap::FamilySpec::Exact(selector) => {
                let (base, slots, kind) = decode_selector(self.resolve_symbol(selector));
                let kind_matches = match invocation {
                    FamilyInvocationKind::Method => matches!(kind, SignatureKind::Method(_)),
                    FamilyInvocationKind::Getter => matches!(kind, SignatureKind::Getter),
                    FamilyInvocationKind::Setter => matches!(kind, SignatureKind::Setter),
                };
                if !kind_matches {
                    return Err(RuntimeError::Message(format!("exact family `{base}` does not accept this invocation kind")).into());
                }
                let expected_positional = slots.iter().filter(|slot| slot.is_none()).count();
                let expected_labels = if matches!(invocation, FamilyInvocationKind::Setter) {
                    Vec::new()
                } else {
                    slots
                        .iter()
                        .filter_map(|slot| slot.as_ref())
                        .map(|label| self.interner.intern(label))
                        .collect::<Vec<_>>()
                };
                let expected_positional = match invocation {
                    FamilyInvocationKind::Setter => 1,
                    _ => expected_positional,
                };
                if expected_positional != view.positional_count() || expected_labels.as_slice() != labels {
                    return Err(RuntimeError::Message(format!("exact family `{base}` does not accept this call shape")).into());
                }
                selector
            }
            crate::heap::FamilySpec::Pattern(pattern_id) => {
                let selector_kind = match invocation {
                    FamilyInvocationKind::Getter => phalcom_common::selector::SelectorKind::Getter,
                    FamilyInvocationKind::Setter => phalcom_common::selector::SelectorKind::Setter,
                    FamilyInvocationKind::Method => phalcom_common::selector::SelectorKind::Method,
                };

                let structural_positionals = match invocation {
                    FamilyInvocationKind::Getter | FamilyInvocationKind::Setter => 0,
                    FamilyInvocationKind::Method => view.positional_count(),
                };

                let (base_sym, matches) = {
                    let pattern = match self.heap.get(pattern_id) {
                        Object::SelectorPattern(pattern) => pattern,
                        _ => return Err(RuntimeError::Internal("Family pattern handle is not a selector pattern".into()).into()),
                    };
                    let base_sym = match pattern.runtime.base {
                        crate::heap::selector_pattern::RuntimeSelectorBase::Named(sym) => sym,
                        crate::heap::selector_pattern::RuntimeSelectorBase::Subscript => {
                            return Err(RuntimeError::Message("subscript selector patterns require index activation".into()).into());
                        }
                    };
                    let matches = pattern.runtime.matches_call(selector_kind, structural_positionals, labels);
                    (base_sym, matches)
                };

                let base_name = self.resolve_symbol(base_sym);

                if !matches {
                    let pattern = match self.heap.get(pattern_id) {
                        Object::SelectorPattern(p) => p.pattern.clone(),
                        _ => unreachable!(),
                    };
                    let selector = match invocation {
                        FamilyInvocationKind::Getter => {
                            let s = crate::method::make_signature(base_name, SignatureKind::Getter);
                            self.get_or_intern(&s)
                        }
                        FamilyInvocationKind::Setter => {
                            let s = crate::method::make_signature(base_name, SignatureKind::Setter);
                            self.get_or_intern(&s)
                        }
                        FamilyInvocationKind::Method => {
                            let total = u8::try_from(view.positional_count() + labels.len()).map_err(|_| RuntimeError::SendArityExceedsLimit {
                                found: view.positional_count() + labels.len(),
                                limit: u8::MAX as usize,
                            })?;
                            let s = crate::method::encode_selector_symbols(
                                base_name,
                                view.positional_count(),
                                labels,
                                SignatureKind::Method(total),
                                &self.interner,
                            );
                            self.get_or_intern(&s)
                        }
                    };
                    let structural = phalcom_common::selector::Selector::decode(self.resolve_symbol(selector));
                    return Err(RuntimeError::selector_pattern_mismatch(pattern, structural, Value::obj(family_id), family.receiver).into());
                }

                match invocation {
                    FamilyInvocationKind::Getter => {
                        let s = crate::method::make_signature(base_name, SignatureKind::Getter);
                        self.get_or_intern(&s)
                    }
                    FamilyInvocationKind::Setter => {
                        let s = crate::method::make_signature(base_name, SignatureKind::Setter);
                        self.get_or_intern(&s)
                    }
                    FamilyInvocationKind::Method => {
                        let total = u8::try_from(view.positional_count() + labels.len()).map_err(|_| RuntimeError::SendArityExceedsLimit {
                            found: view.positional_count() + labels.len(),
                            limit: u8::MAX as usize,
                        })?;
                        let s =
                            crate::method::encode_selector_symbols(base_name, view.positional_count(), labels, SignatureKind::Method(total), &self.interner);
                        self.get_or_intern(&s)
                    }
                }
            }
        };
        self.stack[receiver_idx] = family.receiver;
        let positional_count = match invocation {
            FamilyInvocationKind::Setter => 1,
            _ => view.positional_count(),
        };
        self.dispatch_shape_at_as(receiver_idx, selector, positional_count, labels, source_range, view.caller_authority())
    }

    /// Reifies a message send as a `Message` instance (method-lookup.md §2,
    /// ADR-0012), for the `doesNotUnderstand(_)` miss path.
    pub fn new_message(&mut self, selector: Symbol, args: &[Value]) -> Value {
        let selector_str = self.resolve_symbol(selector).to_string();
        let (name, mut labels, kind) = crate::method::decode_selector(&selector_str);
        if let SignatureKind::SubscriptSet(_) = kind {
            labels.push(Some("put".to_string()));
        }

        let name_val = self.alloc_string_value(name);

        let mut label_texts: Vec<String> = labels.into_iter().map(|label| label.unwrap_or_default()).collect();
        label_texts.resize(args.len(), String::new());
        let label_values: Vec<Value> = label_texts.into_iter().map(|text| self.alloc_string_value(text)).collect();

        let labels_list = Value::obj(self.heap.alloc_list(label_values));
        let args_pack = crate::product::finish_tuple(self, args.to_vec(), Vec::new())
            .map_err(|error| crate::product::runtime_error(self, "Message args", error))
            .expect("message arguments contain no duplicate labels");

        let message_class = self.universe.classes.message_class;
        let mut instance = crate::heap::InstanceObject::new(message_class, 4);
        instance.slots[0] = Value::symbol(selector);
        instance.slots[1] = name_val;
        instance.slots[2] = labels_list;
        instance.slots[3] = args_pack;
        Value::obj(self.heap.alloc(Object::Instance(instance)))
    }

    /// Forwards a missed send to the receiver's `doesNotUnderstand(_)`
    pub(super) fn forward_does_not_understand(&mut self, receiver_idx: usize, selector: Symbol, source_range: SourceRange) -> PhResult<()> {
        let caller_authority = (self.current_access_class(), self.current_has_internal_privilege());
        self.forward_does_not_understand_as(receiver_idx, selector, source_range, caller_authority)
    }

    /// Forwards a miss while preserving authority of code that initiated the send.
    pub(super) fn forward_does_not_understand_as(
        &mut self,
        receiver_idx: usize,
        selector: Symbol,
        source_range: SourceRange,
        caller_authority: (Option<ClassId>, bool),
    ) -> PhResult<()> {
        let receiver = self.stack[receiver_idx];
        let args: Vec<Value> = self.stack[receiver_idx + 1..].to_vec();
        self.stack.truncate(receiver_idx + 1);
        let message = self.new_message(selector, &args);
        self.stack.push(message);

        let dnu_str = crate::method::encode_selector("doesNotUnderstand", &[None], crate::method::SignatureKind::Method(1));
        let dnu_sym = self.get_or_intern(&dnu_str);
        match receiver.lookup_method(self, dnu_sym) {
            Some(method) => self.call_method_with_selector_as(&receiver, method, 1, dnu_sym, None, source_range, caller_authority),
            None => Err(RuntimeError::Internal("doesNotUnderstand(_) missing from Object — kernel invariant violated".into()).into()),
        }
    }

    /// Sends `selector` to `receiver` with `args`, runs the resolved method to
    /// completion, and returns its result value (messages-and-selectors.md §5).
    pub fn send_dynamic(&mut self, receiver: Value, selector: Symbol, args: &[Value]) -> PhResult<Value> {
        if let Some(res) = self.try_module_export_send_dynamic(receiver, selector, args)? {
            return Ok(res);
        }

        let receiver_idx = self.stack.len();
        self.stack.push(receiver);
        self.stack.extend_from_slice(args);

        let base_frames = self.frames.len();
        if let Some(method) = receiver.lookup_method(self, selector) {
            self.call_method(&receiver, method, args.len(), SourceRange::default())?;
        } else {
            let (name, slots, kind) = decode_selector(self.resolve_symbol(selector));
            let positional_count = slots.iter().filter(|slot| slot.is_none()).count();
            let labels = slots
                .iter()
                .filter_map(|slot| slot.as_ref())
                .map(|label| self.interner.intern(label))
                .collect::<Vec<_>>();
            let rest = (args.len() == slots.len() && matches!(kind, SignatureKind::Method(_)))
                .then(|| self.interner.intern(&name))
                .and_then(|base| self.lookup_rest_method(receiver.class(self), base, positional_count, &labels));
            if let Some(method) = rest {
                self.activate_rest_method(&receiver, method, receiver_idx, positional_count, &labels, selector, SourceRange::default())?;
            } else {
                self.forward_does_not_understand(receiver_idx, selector, SourceRange::default())?;
            }
        }
        self.check_native_reentry()?;
        self.native_reentry_depth += 1;
        let result = self.run_until(base_frames);
        self.native_reentry_depth -= 1;
        result
    }

    /// Runs exact method `method_id` against `receiver` with `args` for
    /// synchronous host/native callers.
    pub fn invoke_method_object(&mut self, method_id: ObjRef, receiver: Value, args: &[Value]) -> PhResult<Value> {
        let positional = {
            let sig = &self.heap.method(method_id).signature;
            sig.positional_arity as usize
        };
        let ok = args.len() == positional;
        if !ok {
            return Err(RuntimeError::Arity {
                signature: "invokeOn",
                expected: positional,
                found: args.len(),
            }
            .into());
        }
        let caller_authority = (self.current_access_class(), self.current_has_internal_privilege());
        self.authorize_method_access_as(method_id, caller_authority.0, caller_authority.1)?;

        let receiver_idx = self.stack.len();
        self.stack.push(receiver);
        self.stack.extend_from_slice(args);

        let base_frames = self.frames.len();
        let view = ArgumentView::positional_window(receiver_idx, args.len(), caller_authority.0, caller_authority.1);
        self.activate_captured_method_as(receiver, method_id, view, SourceRange::default())?;
        self.check_native_reentry()?;
        self.native_reentry_depth += 1;
        let result = self.run_until(base_frames);
        self.native_reentry_depth -= 1;
        result
    }

    /// Checks if the receiver on the stack is a `ModuleObject` and attempts export dispatch before class method lookup.
    pub(crate) fn try_module_export_send(
        &mut self,
        receiver_idx: usize,
        selector_sym: Symbol,
        arity: usize,
        source_range: SourceRange,
    ) -> PhResult<Option<()>> {
        let receiver = self.stack[receiver_idx];
        let Some(obj_id) = receiver.as_obj() else {
            return Ok(None);
        };
        let Object::Module(module) = self.heap.get(obj_id) else {
            return Ok(None);
        };

        let selector_str = self.resolve_symbol(selector_sym).to_string();
        let (name, slots, kind) = decode_selector(&selector_str);
        let name_sym = self.interner.intern(&name);

        let Some(export_ref) = module.export(name_sym) else {
            return Ok(None);
        };

        let target_val = match export_ref {
            crate::heap::RuntimeExportRef::Module(target_mod) => {
                if matches!(kind, SignatureKind::Getter) {
                    self.stack.truncate(receiver_idx);
                    self.stack.push(Value::obj(target_mod));
                    return Ok(Some(()));
                }
                Value::obj(target_mod)
            }
            crate::heap::RuntimeExportRef::Binding(binding) => {
                let val = self
                    .heap
                    .module(binding.module)
                    .get_by_slot(binding.slot as usize)
                    .map(|v| self.surface_absence(v))
                    .ok_or_else(|| RuntimeError::Internal(format!("binding slot {} out of range", binding.slot)))?;

                if matches!(kind, SignatureKind::Getter) {
                    self.stack.truncate(receiver_idx);
                    self.stack.push(val);
                    return Ok(Some(()));
                }
                val
            }
        };

        self.stack[receiver_idx] = target_val;
        let call_selector = crate::method::encode_selector("call", &slots, kind);
        let call_sym = self.get_or_intern(&call_selector);
        let caller_authority = (self.current_access_class(), self.current_has_internal_privilege());

        if let Some(method) = target_val.lookup_method(self, call_sym) {
            self.call_method_with_selector_as(&target_val, method, arity, call_sym, None, source_range, caller_authority)?;
        } else {
            let positional_count = slots.iter().filter(|slot| slot.is_none()).count();
            let labels = slots
                .iter()
                .filter_map(|slot| slot.as_ref())
                .map(|label| self.interner.intern(label))
                .collect::<Vec<_>>();
            let rest = (arity == slots.len() && matches!(kind, SignatureKind::Method(_)))
                .then(|| self.interner.intern("call"))
                .and_then(|base| self.lookup_rest_method(target_val.class(self), base, positional_count, &labels));
            if let Some(method) = rest {
                self.activate_rest_method(&target_val, method, receiver_idx, positional_count, &labels, call_sym, source_range)?;
            } else {
                self.forward_does_not_understand(receiver_idx, call_sym, source_range)?;
            }
        }
        Ok(Some(()))
    }

    /// Dynamically dispatches an export send on a Module receiver if applicable.
    pub(crate) fn try_module_export_send_dynamic(&mut self, receiver: Value, selector: Symbol, args: &[Value]) -> PhResult<Option<Value>> {
        let Some(obj_id) = receiver.as_obj() else {
            return Ok(None);
        };
        let Object::Module(module) = self.heap.get(obj_id) else {
            return Ok(None);
        };

        let selector_str = self.resolve_symbol(selector).to_string();
        let (name, slots, kind) = decode_selector(&selector_str);
        let name_sym = self.interner.intern(&name);

        let Some(export_ref) = module.export(name_sym) else {
            return Ok(None);
        };

        let target_val = match export_ref {
            crate::heap::RuntimeExportRef::Module(target_mod) => {
                if matches!(kind, SignatureKind::Getter) {
                    return Ok(Some(Value::obj(target_mod)));
                }
                Value::obj(target_mod)
            }
            crate::heap::RuntimeExportRef::Binding(binding) => {
                let val = self
                    .heap
                    .module(binding.module)
                    .get_by_slot(binding.slot as usize)
                    .map(|v| self.surface_absence(v))
                    .ok_or_else(|| RuntimeError::Internal(format!("binding slot {} out of range", binding.slot)))?;

                if matches!(kind, SignatureKind::Getter) {
                    return Ok(Some(val));
                }
                val
            }
        };

        let call_selector = crate::method::encode_selector("call", &slots, kind);
        let call_sym = self.get_or_intern(&call_selector);
        self.send_dynamic(target_val, call_sym, args).map(Some)
    }
}

#[cfg(test)]
mod tests {
    use crate::error::{PhError, RuntimeError};
    use crate::vm::VM;

    #[test]
    fn invoke_method_object_rejects_inaccessible_method_without_stack_mutation() {
        let mut vm = VM::new();
        let module = vm.create_module("main", "invoke_method_object_access_check");
        vm.interpret_source(module, "class Vault {\n  @private\n  secret { 42 }\n}\nlet vault = Vault.new()\n")
            .expect("class and instance should compile and run");

        let vault_symbol = vm.interner.intern("vault");
        let vault = vm.heap.module(module).get(vault_symbol).expect("`vault` global should exist");
        let secret_selector = vm.get_or_intern("secret");
        let method = vault.lookup_method(&vm, secret_selector).expect("Vault should define secret");
        let before = vm.stack.clone();

        let result = vm.invoke_method_object(method, vault, &[]);

        assert!(
            matches!(result, Err(PhError::Runtime(RuntimeError::NotAllowed(ref message))) if message.contains("member.private_access")),
            "expected private access error, got {result:?}"
        );
        assert_eq!(vm.stack, before, "rejected invokeOn must not mutate the value stack");
    }

    #[test]
    fn compiler_internal_authority_does_not_escape_generated_dispatch() {
        let mut vm = VM::new();
        let module = vm.create_module("main", "compiler_internal_authority");
        vm.interpret_source(module, "class Vault {}\nlet vault = Vault.new()\n")
            .expect("class compilation should use and release its internal authority");

        let vault_symbol = vm.interner.intern("vault");
        let vault = vm.heap.module(module).get(vault_symbol).expect("`vault` global should exist");
        let guard_selector = vm.get_or_intern("_$invariantEnter()");
        let guard = vault.lookup_method(&vm, guard_selector).expect("Object should define _$invariantEnter()");

        let result = vm.invoke_method_object(guard, vault, &[]);
        assert!(
            matches!(result, Err(PhError::Runtime(RuntimeError::NotAllowed(ref message))) if message.contains("internal.selector_access")),
            "generated authority must not outlive its dispatch, got {result:?}"
        );
    }

    #[test]
    fn module_export_distinguishes_getter_from_zero_argument_method() {
        let mut vm = VM::new();
        let module = vm.create_module("main", "module_export_selector_kind");
        vm.interpret_source(module, "class Callable {\n  call() { 42 }\n}\nlet exported = Callable.new()\n")
            .expect("callable export fixture should compile and run");

        let exported_sym = vm.interner.intern("exported");
        let exported = vm.heap.module(module).get(exported_sym).expect("fixture should define exported");
        let slot = vm.heap.module(module).slot_of(exported_sym).expect("fixture should allocate an exported slot");
        let public_sym = vm.interner.intern("service");
        vm.heap.module_mut(module).exports.insert(
            public_sym,
            crate::heap::RuntimeExportRef::Binding(crate::modules::BindingRef {
                module,
                slot: u16::try_from(slot).expect("test slot fits u16"),
            }),
        );

        let getter = vm.get_or_intern(&crate::method::make_signature("service", crate::method::SignatureKind::Getter));
        let getter_value = vm
            .try_module_export_send_dynamic(crate::value::Value::obj(module), getter, &[])
            .expect("getter export dispatch should succeed")
            .expect("service is an export");
        assert_eq!(getter_value, exported, "getter must read the exported binding without invoking it");

        let method = vm.get_or_intern(&crate::method::make_signature("service", crate::method::SignatureKind::Method(0)));
        let call = vm.get_or_intern(&crate::method::make_signature("call", crate::method::SignatureKind::Method(0)));
        let expected = vm.send_dynamic(exported, call, &[]).expect("direct call() should succeed");
        let method_value = vm
            .try_module_export_send_dynamic(crate::value::Value::obj(module), method, &[])
            .expect("zero-argument method export dispatch should succeed")
            .expect("service is an export");
        assert_eq!(method_value, expected, "method(0) must invoke the exported value's call() protocol");
        assert_ne!(method_value, exported, "method(0) must not be collapsed into getter semantics");
    }
}
