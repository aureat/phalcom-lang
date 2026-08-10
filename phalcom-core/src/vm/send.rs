use crate::error::{PhResult, RuntimeError};
use crate::heap::{ClassId, ObjRef, Object};
use crate::interner::Symbol;
use crate::method::{ArgumentView, CallOutcome, MemberVisibility, MethodKind, PrimitiveFn, SignatureKind, decode_selector};
use crate::value::Value;
use phalcom_common::range::SourceRange;

use super::VM;

impl VM {
    /// Checks whether `receiver` can execute a method held by its defining
    /// class. Class-side methods use the receiver's metaclass automatically
    /// because `Value::class` returns that class for class values.
    pub(crate) fn method_receiver_compatible(&self, method: ObjRef, receiver: Value) -> bool {
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
        let Some(core_name) = self.interner.find(crate::heap::CORE_MODULE_NAME) else {
            return false;
        };
        self.modules.get(&core_name).is_some_and(|&core_module| core_module == closure_module)
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
                // Snapshot the frame count so we can detect a non-local return
                // that fired *inside* `native_fn` (e.g. `block_call` running a
                // block whose `return` unwound past this call site). See the
                // guard below.
                let frames_before = self.frames.len();
                self.switch_pending = false;
                // Hand the primitive its receiver+args window through an
                // on-stack buffer rather than a per-send heap `Vec` (Tier 2
                // U-PRIM-ABI; performance.md §4 / ADR-0051). The primitive path
                // pushes no `CallFrame`, so this argument copy was the *only*
                // per-send heap allocation on the native fast path — and the
                // measured hottest one: U-BENCH attribution puts malloc/free as
                // the top mechanism on the arithmetic micro-bench, where every
                // `1 + 2` send heap-allocated a one-element argument `Vec`.
                // `Value` is `Copy`, so for the overwhelmingly common small
                // arity we copy the window into a fixed `[Value; INLINE_ARGS]`
                // on the Rust stack and pass a slice of it; only a rare wider
                // call falls back to a heap `Vec`. Behavior-invariant — the
                // primitive still sees an identical `&[Value]`.
                const INLINE_ARGS: usize = 8;
                self.native_method_contexts.push(native_context);
                let result = if arity <= INLINE_ARGS {
                    let mut args = [Value::Nil; INLINE_ARGS];
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
                        // A fiber switch (ADR-0030 §5, D5) — `self.frames`/
                        // `self.stack` were just repointed to a *different*
                        // fiber by the primitive itself (`fiber_call`/
                        // `fiber_try`/`fiber_yield`); `receiver_idx` was
                        // computed against the now-parked fiber's stack and
                        // no longer means anything here. The typed signal
                        // (not the `frames.len()` heuristic below) is what
                        // distinguishes this from an ordinary return or a
                        // non-local return: neither `result` nor the stack
                        // is touched — the switching primitive already left
                        // the new current fiber's stack exactly as it should
                        // be for the dispatch loop to resume it.
                        self.switch_pending = false;
                    } else if self.frames.len() >= frames_before {
                        // Ordinary primitive return: collapse the receiver+args
                        // window and land the result in the receiver slot.
                        self.stack.truncate(receiver_idx);
                        self.stack.push(result);
                    } else {
                        // A `Bytecode::ReturnNonLocal` fired *inside* `native_fn`
                        // (e.g. `block_call` ran a block whose `return` unwound to
                        // a method at or below this call site), popping one or
                        // more frames. `receiver_idx` — computed against the
                        // pre-call stack — now points *above* the unwound stack
                        // top, so the normal `truncate(receiver_idx)` would be a
                        // silent no-op and mis-place the value; skip it. But the
                        // handler pushed the return value at the home frame's
                        // offset only for the *innermost* `run_until` to drain
                        // (its top-of-loop check pops and returns it), so `result`
                        // must be re-pushed here to re-establish it for the outer
                        // frame that resumes next. Pushing exactly once per
                        // unwound level, balanced against each level's drain-pop,
                        // keeps the stack consistent — no duplicate, no loss
                        // (U10-implementation-spec.md §2 point 3, corrected: the
                        // drain check *pops* the pushed value, so the arm must
                        // re-push rather than skip entirely).
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
                    Some((positionals, labels)) => ArgumentView::shaped(receiver_idx, positionals, labels, selector, caller_authority.0, caller_authority.1),
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
            Ok(CallOutcome::Returned(*self.stack.last().unwrap_or(&Value::Nil)))
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
            Ok(CallOutcome::Returned(*self.stack.last().unwrap_or(&Value::Nil)))
        }
    }

    /// Activates the concrete representation behind the sealed Function
    /// hierarchy. The current stack window is reused in place; no message or
    /// argument vector is created for ordinary calls.
    pub(crate) fn activate_function(&mut self, receiver: Value, view: ArgumentView, source_range: SourceRange) -> PhResult<CallOutcome> {
        let Value::Obj(id) = receiver else {
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
            Object::Family(family) => self.activate_family(*family, view, source_range),
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
            instance: match receiver {
                Value::Obj(id) => id,
                _ => unreachable!("Function representations are heap values"),
            },
        };
        let mut frame = self.new_call_frame(closure_id, context, 0, receiver_idx, Some(source_range));
        frame.home_frame_token = home_frame_token;
        self.push_frame(frame)?;
        Ok(CallOutcome::EnteredFrame)
    }

    fn activate_bound_method(&mut self, bound: crate::heap::BoundMethodObject, view: ArgumentView, source_range: SourceRange) -> PhResult<CallOutcome> {
        let method = bound.method;
        self.authorize_method_access_as(method, view.caller_authority().0, view.caller_authority().1)?;
        if !self.method_receiver_compatible(method, bound.receiver) {
            return Err(RuntimeError::NotAllowed(format!(
                "receiver of `{}` is incompatible with reified method",
                self.resolve_symbol(self.heap.method(method).signature.selector)
            ))
            .into());
        }
        let method_selector = self.heap.method(method).signature.selector;
        let selector_text = self.resolve_symbol(method_selector).to_owned();
        let (_, expected_slots, _) = decode_selector(&selector_text);
        let actual_labels = view.labels(self);
        let expected_positional = expected_slots.iter().filter(|slot| slot.is_none()).count();
        let expected_labels = expected_slots
            .iter()
            .filter_map(|slot| slot.as_ref())
            .map(|label| self.interner.intern(label))
            .collect::<Vec<_>>();
        let accepted = if let Some(rest) = self.heap.method(method).signature.rest.as_ref() {
            rest.accepts(view.positional_count(), &actual_labels)
        } else {
            expected_positional == view.positional_count() && expected_labels == actual_labels
        };
        if !accepted {
            return Err(RuntimeError::Arity {
                signature: "call",
                expected: expected_positional,
                found: view.positional_count() + view.labeled_count(),
            }
            .into());
        }
        let receiver_idx = view.receiver_index();
        self.stack[receiver_idx] = bound.receiver;
        let total = view.positional_count() + view.labeled_count();
        if self.heap.method(method).signature.rest.is_some() && matches!(self.heap.method(method).kind, MethodKind::Closure(_)) {
            self.call_rest_method_as(
                &bound.receiver,
                method,
                receiver_idx,
                view.positional_count(),
                &actual_labels,
                source_range,
                view.caller_authority(),
            )?;
            return Ok(if !self.frames.is_empty() {
                CallOutcome::EnteredFrame
            } else {
                CallOutcome::Returned(*self.stack.last().unwrap_or(&Value::Nil))
            });
        }
        self.dispatch_selected_method_as(
            &bound.receiver,
            method,
            total,
            view.selector().unwrap_or(method_selector),
            Some((view.positional_count(), view.labeled_count())),
            source_range,
            view.caller_authority(),
        )
    }

    fn activate_family(&mut self, family: crate::heap::FamilyObject, view: ArgumentView, source_range: SourceRange) -> PhResult<CallOutcome> {
        let labels = view.labels(self);
        let selector = if family.open {
            let base = self.resolve_symbol(family.selector).to_owned();
            let mut slots = Vec::with_capacity(view.positional_count() + labels.len());
            slots.extend(std::iter::repeat_n(None, view.positional_count()));
            slots.extend(labels.iter().map(|label| Some(self.resolve_symbol(*label).to_owned())));
            self.get_or_intern(&crate::method::encode_selector(
                &base,
                &slots,
                SignatureKind::Method(u8::try_from(slots.len()).map_err(|_| RuntimeError::SendArityExceedsLimit {
                    found: slots.len(),
                    limit: u8::MAX as usize,
                })?),
            ))
        } else {
            let (base, slots, _) = decode_selector(self.resolve_symbol(family.selector));
            if slots.len() != view.positional_count() + labels.len() {
                return Err(RuntimeError::Message(format!(
                    "pinned method reference `{base}` expects {} argument(s), got {}",
                    slots.len(),
                    view.positional_count() + labels.len()
                ))
                .into());
            }
            family.selector
        };
        let receiver_idx = view.receiver_index();
        self.stack[receiver_idx] = family.recv;
        self.dispatch_shape_at_as(receiver_idx, selector, view.positional_count(), &labels, source_range, view.caller_authority())
    }

    /// Reifies a message send as a `Message` instance (method-lookup.md §2,
    /// ADR-0012), for the `doesNotUnderstand(_)` miss path.
    ///
    /// The returned `Message` is an ordinary fixed-slot
    /// [`InstanceObject`](crate::heap::InstanceObject) of the kernel
    /// `Message` class ([`CoreClasses::message_class`](crate::universe::CoreClasses::message_class)),
    /// built directly in Rust (no `.ph` `construct`) with four slots:
    ///
    /// 0. `selector` — the interned [`Symbol`] as sent;
    /// 1. `name` — the bare method name [`String`] (encoder-inverse, `+` for `+(_)`);
    /// 2. `labels` — a [`List`](crate::heap::ListObject) of `String`, one per
    ///    argument, `""` for a positional (unlabeled) argument so that
    ///    `labels.size == args.size` and callers can zip them;
    /// 3. `args` — the canonical complete argument pack (`Unit` for empty,
    ///    otherwise a [`Tuple`](crate::heap::TupleObject)).
    ///
    /// The `""`-for-positional convention (rather than a separate absence
    /// marker) keeps the two lists index-aligned; it is a deliberate U8 choice,
    /// not spec-pinned.
    pub fn new_message(&mut self, selector: Symbol, args: &[Value]) -> Value {
        let selector_str = self.resolve_symbol(selector).to_string();
        let (name, mut labels, kind) = crate::method::decode_selector(&selector_str);
        if let SignatureKind::SubscriptSet(_) = kind {
            labels.push(Some("put".to_string()));
        }

        let name_val = self.alloc_string_value(name);

        // Index-align labels with args: pad or truncate to `args.len()`, using
        // `""` for positional arguments (kinds whose decoded arity differs from
        // the call arity, e.g. subscripts, are made consistent here).
        let mut label_texts: Vec<String> = labels.into_iter().map(|label| label.unwrap_or_default()).collect();
        label_texts.resize(args.len(), String::new());
        let label_values: Vec<Value> = label_texts.into_iter().map(|text| self.alloc_string_value(text)).collect();

        let labels_list = Value::Obj(self.heap.alloc_list(label_values));
        let args_pack = crate::product::finish_tuple(self, args.to_vec(), Vec::new())
            .map_err(|error| crate::product::runtime_error(self, "Message args", error))
            .expect("message arguments contain no duplicate labels");

        let message_class = self.universe.classes.message_class;
        let mut instance = crate::heap::InstanceObject::new(message_class, 4);
        instance.slots[0] = Value::Symbol(selector);
        instance.slots[1] = name_val;
        instance.slots[2] = labels_list;
        instance.slots[3] = args_pack;
        Value::Obj(self.heap.alloc(Object::Instance(instance)))
    }

    /// Forwards a missed send to the receiver's `doesNotUnderstand(_)`
    /// (method-lookup.md §2, ADR-0012).
    ///
    /// Precondition: `self.stack[receiver_idx..]` holds `[receiver, args…]`.
    /// The arguments are replaced by a single synthesized
    /// [`Message`](Self::new_message) and the receiver's
    /// `doesNotUnderstand(_)` is dispatched via [`Self::call_method`] (a
    /// primitive runs in place; a user override pushes a frame). Because
    /// `doesNotUnderstand(_)` is looked up by the *exact* selector, it always
    /// resolves to at least `Object`'s default handler — a receiver whose chain
    /// somehow lacks it is a kernel-invariant violation, surfaced as
    /// [`RuntimeError::Internal`] rather than recursing (the recursion guard:
    /// a missing dNU is never itself re-sent as a dNU).
    ///
    /// # Errors
    ///
    /// Returns [`RuntimeError::Internal`] if `doesNotUnderstand(_)` is missing
    /// from the receiver's chain, or propagates any error raised by the handler.
    pub(super) fn forward_does_not_understand(&mut self, receiver_idx: usize, selector: Symbol, source_range: SourceRange) -> PhResult<()> {
        let caller_authority = (self.current_access_class(), self.current_has_internal_privilege());
        self.forward_does_not_understand_as(receiver_idx, selector, source_range, caller_authority)
    }

    /// Forwards a miss while preserving authority of code that initiated the
    /// send. Native gateways execute under their own authority, but their
    /// `doesNotUnderstand(_)` target must still observe original caller
    /// visibility.
    pub(super) fn forward_does_not_understand_as(
        &mut self,
        receiver_idx: usize,
        selector: Symbol,
        source_range: SourceRange,
        caller_authority: (Option<ClassId>, bool),
    ) -> PhResult<()> {
        let receiver = self.stack[receiver_idx];
        let args: Vec<Value> = self.stack[receiver_idx + 1..].to_vec();
        // Keep the receiver, drop the original argument values.
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
    ///
    /// This is the shared runtime-send workhorse behind reflective dispatch:
    /// Its consumers are host/native synchronous helpers and legacy dynamic
    /// pack callers. Ordinary language-level `perform`, `invokeOn`, and
    /// Function/Family calls use the flat shape-aware gateways instead.
    /// Unlike the [`crate::bytecode::Bytecode::Invoke`] handler it can
    /// be called from *inside* a native primitive: it saves the frame count,
    /// pushes `receiver`+`args` at a fresh stack window, dispatches, then
    /// re-enters `run_until` to drain that one activation and recover a
    /// synchronous [`Value`] (the same re-entrancy pattern as
    /// [`block_call`](crate::primitive::block::block_call)). A miss routes
    /// through `doesNotUnderstand(_)` exactly once — a `perform` of an unknown
    /// selector re-enters dNU, it does not loop.
    ///
    /// # Errors
    ///
    /// Propagates any [`RuntimeError`] raised by lookup, the dispatched method,
    /// or the `doesNotUnderstand(_)` forward.
    pub fn send_dynamic(&mut self, receiver: Value, selector: Symbol, args: &[Value]) -> PhResult<Value> {
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
        // Re-entrant native frame (ADR-0030 §4): a fiber switch is forbidden
        // while this recursive `run_until` is on the Rust call stack, since
        // its `base_frames` is computed against *this* fiber and would be
        // corrupted by a switch underneath it (see `native_reentry_depth`'s doc).
        self.check_native_reentry()?;
        self.native_reentry_depth += 1;
        let result = self.run_until(base_frames);
        self.native_reentry_depth -= 1;
        result
    }

    /// Runs exact method `method_id` against `receiver` with `args` for
    /// synchronous host/native callers, re-entering `run_until` to recover a
    /// result. Language-level `Method#invokeOn(_,***)` and bound Function calls
    /// use the flat shape-aware gateways instead (U-CORE-3, [ADR-0028](../../../docs/adr/accepted/0028-amend-floor-admit-method-reflection.md)).
    ///
    /// Mirrors [`Self::send_dynamic`]'s re-entrancy exactly, except there is
    /// **no lookup**: `method_id` is already resolved, so a mismatched
    /// receiver misbehaves inside the method body rather than raising
    /// `doesNotUnderstand(_)` (the caller is responsible for receiver
    /// compatibility, functions.md §3). Arity and visibility authorization are
    /// validated **before** the receiver/args are pushed onto the stack, so a
    /// rejected invocation leaves the stack exactly as it was found (R-INV-3.4).
    ///
    /// # Errors
    ///
    /// Returns [`RuntimeError::Arity`] if `args.len()` does not match the
    /// method's exact signature, or propagates any
    /// [`RuntimeError`] raised while running the method body — including a
    /// [`RuntimeError::DeadFrameError`] from a non-local `return` inside an
    /// escaping block whose home frame is no longer live (R-INV-3.2).
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
        self.authorize_method_access(method_id)?;

        self.stack.push(receiver);
        self.stack.extend_from_slice(args);

        let base_frames = self.frames.len();
        self.call_method(&receiver, method_id, args.len(), SourceRange::default())?;
        // See `send_dynamic`'s matching comment — re-entrant native frame,
        // fiber switch forbidden underneath (ADR-0030 §4).
        self.check_native_reentry()?;
        self.native_reentry_depth += 1;
        let result = self.run_until(base_frames);
        self.native_reentry_depth -= 1;
        result
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
}
