use crate::bytecode::{Bytecode, PackAccess, PackSendKind};
use crate::callable::Callable;
use crate::diagnostics::print_compile;
use crate::error::{PhError, PhResult, RuntimeError};
use crate::frame::{CallContext, CallFrame, FrameToken};
use crate::heap::BlockObject;
use crate::heap::ClosureObject;
use crate::heap::Upvalue;
use crate::heap::{ObjRef, Object, is_strict_subclass, lookup_method_with_definer};
use crate::interner::Symbol;
use crate::method::{ArgumentView, InvocationLayout, SignatureKind, decode_selector, encode_selector};
use crate::value::Value;
use crate::value::{FALSE, TRUE};
use crate::vm::control::{ControlStepOutcome, Transfer};
use phalcom_common::range::SourceRange;
use std::rc::Rc;
use std::sync::Arc;
#[cfg(feature = "vm-trace")]
use tracing::{Level, debug, span};

use super::VM;

#[derive(Clone, Copy)]
struct ConditionalInvocation {
    arity: u8,
    selector: u16,
    declaring_module: u16,
    declaring_name: u16,
    class_side: bool,
    fallback_method: u16,
    call_environment_recipe: u16,
}

impl VM {
    fn pack_builder_error(&self, error: crate::heap::PackBuilderError) -> RuntimeError {
        match error {
            crate::heap::PackBuilderError::DuplicateLabel(label) => RuntimeError::DuplicateArgumentLabel {
                label: self.resolve_symbol(label).to_owned(),
            },
            crate::heap::PackBuilderError::MissingPendingLabel => RuntimeError::Internal("pack fill without a reserved label".into()),
            crate::heap::PackBuilderError::PendingLabelExists => RuntimeError::Internal("pack contribution attempted while a label is pending".into()),
        }
    }

    /// Captures a rest Pack from already-evaluated positional arguments and labeled argument entries.
    ///
    /// Empty lanes normalize directly to `Value::empty_pack()`.
    /// Otherwise, allocates an immutable root Pack object.
    pub fn capture_rest_pack(&mut self, positionals: &[Value], labeled: &[(Symbol, Value)]) -> PhResult<Value> {
        self.capture_rest_pack_owned(positionals.to_vec(), labeled.to_vec())
    }

    /// Captures owned staging buffers without copying them again at the VM
    /// rest-binding boundary.
    pub fn capture_rest_pack_owned(&mut self, positionals: Vec<Value>, labeled: Vec<(Symbol, Value)>) -> PhResult<Value> {
        if positionals.is_empty() && labeled.is_empty() {
            return Ok(Value::empty_pack());
        }
        crate::product::finish_pack(self, positionals, labeled).map_err(|error| crate::product::runtime_error(self, "Pack label", error).into())
    }

    /// Captures a positional rest Pack from already-evaluated positional arguments.
    pub fn capture_positional_rest_pack(&mut self, positionals: &[Value]) -> PhResult<Value> {
        if positionals.is_empty() {
            return Ok(Value::empty_pack());
        }
        self.capture_positional_rest_pack_owned(positionals.to_vec())
    }

    /// Captures an owned positional staging buffer without copying it again.
    pub fn capture_positional_rest_pack_owned(&mut self, positionals: Vec<Value>) -> PhResult<Value> {
        if positionals.is_empty() {
            return Ok(Value::empty_pack());
        }
        crate::product::finish_pack(self, positionals, Vec::new()).map_err(|error| crate::product::runtime_error(self, "Pack label", error).into())
    }

    /// Captures a labeled rest Pack from already-evaluated labeled argument entries.
    pub fn capture_labeled_rest_pack(&mut self, labeled: &[(Symbol, Value)]) -> PhResult<Value> {
        if labeled.is_empty() {
            return Ok(Value::empty_pack());
        }
        self.capture_labeled_rest_pack_owned(labeled.to_vec())
    }

    /// Captures an owned labeled staging buffer without copying it again.
    pub fn capture_labeled_rest_pack_owned(&mut self, labeled: Vec<(Symbol, Value)>) -> PhResult<Value> {
        if labeled.is_empty() {
            return Ok(Value::empty_pack());
        }
        crate::product::finish_pack(self, Vec::new(), labeled).map_err(|error| crate::product::runtime_error(self, "Pack label", error).into())
    }

    /// Returns Unit/EmptyPack/Tuple/Pack positional-lane values without invoking ordinary
    /// iteration. The copied lane can safely cross the target append
    /// operation, including any future GC safepoint.
    fn positional_lane(&self, value: Value) -> Option<Vec<Value>> {
        if value.is_unit() || value.is_empty_pack() {
            Some(Vec::new())
        } else if let Some(view) = self.pack_view_value(value) {
            let mut pos = Vec::with_capacity(view.positional_len());
            for i in 0..view.positional_len() {
                pos.push(view.get_positional(i)?);
            }
            Some(pos)
        } else if let Some(id) = value.as_obj() {
            if matches!(self.heap.get(id), Object::Tuple(_)) {
                Some(self.tuple_view(id)?.positionals())
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Executes the conditional call edge carried by one `InvokeConditional`
    /// instruction. Kept as a single private seam so direct bytecode dispatch
    /// can be tested independently from the interpreter's surrounding loop.
    fn invoke_conditional_at(&mut self, callable: &Callable, ip: usize, invocation: ConditionalInvocation) -> PhResult<()> {
        let ConditionalInvocation {
            arity,
            selector,
            declaring_module,
            declaring_name,
            class_side,
            fallback_method,
            call_environment_recipe,
        } = invocation;
        let arity = arity as usize;
        let receiver_idx = self
            .stack
            .len()
            .checked_sub(arity + 1)
            .ok_or(RuntimeError::Internal("stack underflow in InvokeConditional".into()))?;
        let receiver = self.stack[receiver_idx];
        let selector_sym = callable.chunk.constants[selector as usize].as_symbol().map_err(RuntimeError::Internal)?;
        let declaring_module = callable.chunk.constants[declaring_module as usize]
            .as_obj()
            .ok_or_else(|| RuntimeError::Internal("conditional declaring module is not an object".into()))?;
        let declaring_sym = callable.chunk.constants[declaring_name as usize].as_symbol().map_err(RuntimeError::Internal)?;
        let fallback_val = callable.chunk.constants[fallback_method as usize];
        let fallback_method_ref = fallback_val
            .as_obj()
            .ok_or_else(|| RuntimeError::Internal("fallback method is not an object".into()))?;

        let declaring_class_id = self
            .classes
            .get(&crate::vm::ClassKey {
                module: declaring_module,
                name: declaring_sym,
            })
            .copied()
            .map(|class| if class_side { self.heap.class(class).class } else { class });

        let outcome = if let Some(declaring_class) = declaring_class_id {
            self.select_conditional_method(receiver, selector_sym, declaring_class, fallback_method_ref)
        } else {
            super::send::ConditionalSelectionOutcome::SelectedFallback(fallback_method_ref)
        };
        let (method_to_call, type_environment, conformance_environment) = match outcome {
            super::send::ConditionalSelectionOutcome::OrdinaryOverride(method) => (
                method,
                crate::typing::RuntimeTypeEnvironmentId::EMPTY,
                crate::typing::RuntimeConformanceEnvironmentId::EMPTY,
            ),
            super::send::ConditionalSelectionOutcome::SelectedFallback(method) => {
                let recipe = callable.chunk.executable_semantics.call_environment_recipe(call_environment_recipe).clone();
                self.validate_specialized_call(&recipe, method)?;
                (
                    method,
                    self.materialize_call_environment(&recipe)?,
                    self.materialize_conformance_environment(receiver, &recipe)?,
                )
            }
        };

        let foreign_guard = self.frames.last().and_then(|frame| frame.foreign_receiver_guard);
        let frames_before = self.frames.len();
        let layout = self.invocation_layout_for_selector(selector_sym, arity)?;
        self.call_method_with_selector_as_and_environments(
            &receiver,
            method_to_call,
            arity,
            selector_sym,
            Some(layout),
            callable.chunk.span_at(ip),
            self.current_caller_authority(),
            type_environment,
            conformance_environment,
        )?;
        if self.frames.len() > frames_before {
            self.frames.last_mut().unwrap().foreign_receiver_guard = foreign_guard;
        }
        Ok(())
    }

    fn append_positional_values(&mut self, target: Value, values: Vec<Value>) -> Result<(), RuntimeError> {
        let Some(id) = target.as_obj() else {
            return Err(RuntimeError::Internal("positional expansion target is not an object".into()));
        };
        if matches!(self.heap.get(id), Object::PackBuilder(_)) {
            for value in values {
                self.heap.pack_builder_mut(id).push_positional(value);
            }
            return Ok(());
        }
        if matches!(self.heap.get(id), Object::List(_)) {
            for value in values {
                self.heap.list_mut(id).push(value);
            }
            return Ok(());
        }
        if matches!(self.heap.get(id), Object::ArrayLiteralBuilder(_)) {
            for value in values {
                self.heap.array_literal_builder_mut(id).push(value);
            }
            return Ok(());
        }
        Err(RuntimeError::Internal(
            "positional expansion target is not a pack builder, List, or Array builder".into(),
        ))
    }

    fn try_append_tuple_positionals(&mut self, source: Value, target: Value) -> Result<bool, RuntimeError> {
        let Some(values) = self.positional_lane(source) else {
            // Context-free interpreter compilation has no semantic lowering
            // product. Preserve its dynamic compatibility path, but surface
            // the established boundary when the source cannot even expose
            // the iteration entry point; canonical compiled programs fail
            // closed before reaching this probe.
            let iterator = self.interner.intern("iterator");
            if lookup_method_with_definer(&self.heap, source.class(self), iterator).is_none() {
                return Err(RuntimeError::NonIterableStarOperand { found: source.type_name() });
            }
            return Ok(false);
        };
        self.append_positional_values(target, values)?;
        Ok(true)
    }

    /// Copies the ordered labeled lane exposed by a `**` source. The returned
    /// associations own no heap borrows, so destinations may re-enter the VM
    /// while consuming them.
    fn snapshot_labeled_lane(&mut self, source: Value) -> Result<Vec<(Symbol, Value)>, RuntimeError> {
        if source.is_unit() || source.is_empty_record() || source.is_empty_pack() {
            Ok(Vec::new())
        } else if let Some(view) = self.pack_view_value(source) {
            let mut entries = Vec::with_capacity(view.labeled_len());
            for i in 0..view.labeled_len() {
                let label = view.get_label(i).ok_or_else(|| RuntimeError::Internal("missing pack label".into()))?;
                let val = view.get_labeled_value(i).ok_or_else(|| RuntimeError::Internal("missing pack value".into()))?;
                entries.push((label, val));
            }
            Ok(entries)
        } else if let Some(id) = source.as_obj() {
            match self.heap.get(id) {
                Object::Tuple(_) => self
                    .tuple_view(id)
                    .map(|tuple| tuple.labeled_entries())
                    .ok_or_else(|| RuntimeError::Internal("Tuple descriptor is unavailable".into())),
                Object::Record(_) => {
                    let entries = self
                        .record_view(id)
                        .map(|record| record.entries())
                        .ok_or_else(|| RuntimeError::Internal("Record descriptor is unavailable".into()))?;
                    entries
                        .into_iter()
                        .map(|(key, value)| match key {
                            phalcom_semantic::field_key::FieldKey::Symbol(text) => Ok((self.interner.intern(&text), value)),
                            key => Err(RuntimeError::NonSymbolRecordKeyInExpansion { key: key.to_string() }),
                        })
                        .collect()
                }
                Object::Map(_) => self
                    .heap
                    .map(id)
                    .entries()
                    .map(|(key, value)| match key.symbol_value() {
                        Some(label) => Ok((label, value)),
                        None => Err(RuntimeError::NonSymbolMapKeyInExpansion { found: key.type_name() }),
                    })
                    .collect(),
                _ => Err(RuntimeError::InvalidStarStarOperand { found: source.type_name() }),
            }
        } else {
            Err(RuntimeError::InvalidStarStarOperand { found: source.type_name() })
        }
    }

    /// Copies Record-compatible labeled fields with their canonical key kind.
    /// Record-to-Record expansion preserves heterogeneous keys; Tuple and Map
    /// contributors retain their existing Symbol-only label contract.
    fn snapshot_record_fields(&self, source: Value) -> Result<Vec<(phalcom_semantic::field_key::FieldKey, Value)>, RuntimeError> {
        use phalcom_semantic::field_key::FieldKey;
        if source.is_unit() || source.is_empty_record() || source.is_empty_pack() {
            return Ok(Vec::new());
        }
        if let Some(view) = self.pack_view_value(source) {
            return Ok(view
                .labeled_entries()
                .into_iter()
                .map(|(label, value)| (FieldKey::symbol(self.resolve_symbol(label)), value))
                .collect());
        }
        let Some(id) = source.as_obj() else {
            return Err(RuntimeError::InvalidStarStarOperand { found: source.type_name() });
        };
        match self.heap.get(id) {
            Object::Tuple(_) => self
                .tuple_view(id)
                .map(|tuple| {
                    tuple
                        .labeled_entries()
                        .into_iter()
                        .map(|(label, value)| (FieldKey::symbol(self.resolve_symbol(label)), value))
                        .collect()
                })
                .ok_or_else(|| RuntimeError::Internal("Tuple descriptor is unavailable".into())),
            Object::Record(_) => self
                .record_view(id)
                .map(|record| record.entries())
                .ok_or_else(|| RuntimeError::Internal("Record descriptor is unavailable".into())),
            Object::Map(_) => self
                .heap
                .map(id)
                .entries()
                .map(|(key, value)| match key.symbol_value() {
                    Some(label) => Ok((FieldKey::symbol(self.resolve_symbol(label)), value)),
                    None => Err(RuntimeError::NonSymbolMapKeyInExpansion { found: key.type_name() }),
                })
                .collect(),
            _ => Err(RuntimeError::InvalidStarStarOperand { found: source.type_name() }),
        }
    }

    fn dynamic_pack_selector(&mut self, base: Symbol, kind: PackSendKind, positionals: usize, labels: &[Symbol]) -> PhResult<Symbol> {
        let base = self.resolve_symbol(base).to_owned();
        let mut slots = Vec::with_capacity(positionals + labels.len());
        slots.extend(std::iter::repeat_n(None, positionals));
        slots.extend(labels.iter().map(|label| Some(self.resolve_symbol(*label).to_owned())));
        let signature = match kind {
            PackSendKind::Method => SignatureKind::Method(u8::try_from(slots.len()).map_err(|_| RuntimeError::SendArityExceedsLimit {
                found: slots.len(),
                limit: u8::MAX as usize,
            })?),
            PackSendKind::SubscriptGet => SignatureKind::SubscriptGet(u8::try_from(slots.len()).map_err(|_| RuntimeError::SendArityExceedsLimit {
                found: slots.len(),
                limit: u8::MAX as usize,
            })?),
            PackSendKind::SubscriptSet => SignatureKind::SubscriptSet(u8::try_from(slots.len()).map_err(|_| RuntimeError::SendArityExceedsLimit {
                found: slots.len(),
                limit: u8::MAX as usize,
            })?),
        };
        Ok(self.interner.intern(&encode_selector(&base, &slots, signature)))
    }

    fn invoke_dynamic_selector(&mut self, receiver_idx: usize, selector: Symbol, arity: usize, source_range: SourceRange) -> PhResult<()> {
        let layout = self.invocation_layout_for_selector(selector, arity)?;
        self.dispatch_selector_window_as(receiver_idx, selector, layout, source_range, self.current_caller_authority())
            .map(|_| ())
    }

    /// Finds an accepting rest method after an already-complete exact walk has
    /// failed. `start` is the first class eligible for lookup (the receiver
    /// class for ordinary sends, the defining class's parent for `super`).
    pub(crate) fn lookup_rest_method(
        &self,
        mut start: crate::heap::ClassId,
        base: Symbol,
        positional_count: usize,
        labels: &[Symbol],
    ) -> Option<crate::heap::ObjRef> {
        loop {
            let class = self.heap.class(start);
            if let Some(method) = class.get_rest_method(base)
                && crate::method::InvocationLayout::validate_signature_shape(&self.heap.method(method).signature, positional_count, labels, false).is_ok()
            {
                return Some(method);
            }
            start = class.superclass?;
        }
    }

    /// Rewrites an accepted incoming pack into declaration-local slots before
    /// entering the closure frame. The raw pack never becomes visible to user
    /// bytecode.
    #[allow(clippy::too_many_arguments)]
    #[allow(dead_code)]
    pub(crate) fn call_rest_method_as(
        &mut self,
        receiver: &Value,
        method: crate::heap::ObjRef,
        receiver_idx: usize,
        positional_count: usize,
        labels: &[Symbol],
        source_range: SourceRange,
        caller_authority: (Option<crate::heap::ClassId>, crate::method::RuntimeAccessOrigin),
    ) -> PhResult<()> {
        self.call_rest_method_as_with_environments(
            receiver,
            method,
            receiver_idx,
            positional_count,
            labels,
            source_range,
            caller_authority,
            crate::typing::environment::RuntimeTypeEnvironmentId::EMPTY,
            crate::typing::RuntimeConformanceEnvironmentId::EMPTY,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn call_rest_method_as_with_environments(
        &mut self,
        receiver: &Value,
        method: crate::heap::ObjRef,
        receiver_idx: usize,
        positional_count: usize,
        labels: &[Symbol],
        source_range: SourceRange,
        caller_authority: (Option<crate::heap::ClassId>, crate::method::RuntimeAccessOrigin),
        type_environment: crate::typing::environment::RuntimeTypeEnvironmentId,
        conformance_environment: crate::typing::RuntimeConformanceEnvironmentId,
    ) -> PhResult<()> {
        let layout = self.heap.method(method).signature.rest.clone().expect("rest lookup only returns rest methods");
        let total = self.stack.len() - receiver_idx - 1;
        let arguments = self.stack[receiver_idx + 1..receiver_idx + 1 + total].to_vec();
        let fixed_pos = layout.fixed_positionals() as usize;
        let fixed_labels = layout.fixed_labels().len();
        debug_assert!(layout.accepts(positional_count, labels));
        self.stack.truncate(receiver_idx + 1);
        self.stack.extend_from_slice(&arguments[..fixed_pos]);
        match layout.mode() {
            crate::method::RestMode::Positional { .. } => {
                let capture = self.capture_positional_rest_pack(&arguments[fixed_pos..positional_count])?;
                self.stack.push(capture);
                self.stack.extend_from_slice(&arguments[positional_count..positional_count + fixed_labels]);
            }
            crate::method::RestMode::Labeled { .. } => {
                self.stack.extend_from_slice(&arguments[positional_count..positional_count + fixed_labels]);
                let labeled: Vec<(Symbol, Value)> = labels[fixed_labels..]
                    .iter()
                    .copied()
                    .zip(arguments[positional_count + fixed_labels..].iter().copied())
                    .collect();
                let capture = self.capture_labeled_rest_pack_owned(labeled)?;
                self.stack.push(capture);
            }
            crate::method::RestMode::Split { .. } => {
                let positional_capture = self.capture_positional_rest_pack(&arguments[fixed_pos..positional_count])?;
                self.stack.push(positional_capture);
                self.stack.extend_from_slice(&arguments[positional_count..positional_count + fixed_labels]);
                let labeled: Vec<(Symbol, Value)> = labels[fixed_labels..]
                    .iter()
                    .copied()
                    .zip(arguments[positional_count + fixed_labels..].iter().copied())
                    .collect();
                let labeled_capture = self.capture_labeled_rest_pack_owned(labeled)?;
                self.stack.push(labeled_capture);
            }
            crate::method::RestMode::Complete { .. } => {
                self.stack.extend_from_slice(&arguments[positional_count..positional_count + fixed_labels]);
                let labeled: Vec<(Symbol, Value)> = labels[fixed_labels..]
                    .iter()
                    .copied()
                    .zip(arguments[positional_count + fixed_labels..].iter().copied())
                    .collect();
                let capture = self.capture_rest_pack_owned(arguments[fixed_pos..positional_count].to_vec(), labeled)?;
                self.stack.push(capture);
            }
        }
        let local_arity = self.stack.len() - receiver_idx - 1;
        let selector = self.heap.method(method).signature.selector;
        self.call_method_with_selector_as_and_environments(
            receiver,
            method,
            local_arity,
            selector,
            None,
            source_range,
            caller_authority,
            type_environment,
            conformance_environment,
        )
    }

    /// Enters a rest-family method with the incoming shape intact for native
    /// implementations, or materializes declaration-local Tuple/Unit captures
    /// for bytecode methods.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn activate_rest_method(
        &mut self,
        receiver: &Value,
        method: crate::heap::ObjRef,
        receiver_idx: usize,
        positional_count: usize,
        labels: &[Symbol],
        selector: Symbol,
        source_range: SourceRange,
    ) -> PhResult<()> {
        let caller_authority = self.current_caller_authority();
        self.activate_rest_method_as(
            receiver,
            method,
            receiver_idx,
            positional_count,
            labels,
            selector,
            source_range,
            caller_authority,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn activate_rest_method_with_environment(
        &mut self,
        receiver: &Value,
        method: crate::heap::ObjRef,
        receiver_idx: usize,
        positional_count: usize,
        labels: &[Symbol],
        selector: Symbol,
        source_range: SourceRange,
        type_environment: crate::typing::environment::RuntimeTypeEnvironmentId,
        conformance_environment: crate::typing::RuntimeConformanceEnvironmentId,
    ) -> PhResult<()> {
        let caller_authority = self.current_caller_authority();
        self.activate_rest_method_as_with_environment(
            receiver,
            method,
            receiver_idx,
            positional_count,
            labels,
            selector,
            source_range,
            caller_authority,
            type_environment,
            conformance_environment,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn activate_rest_method_as(
        &mut self,
        receiver: &Value,
        method: crate::heap::ObjRef,
        receiver_idx: usize,
        positional_count: usize,
        labels: &[Symbol],
        selector: Symbol,
        source_range: SourceRange,
        caller_authority: (Option<crate::heap::ClassId>, crate::method::RuntimeAccessOrigin),
    ) -> PhResult<()> {
        self.activate_rest_method_as_with_environment(
            receiver,
            method,
            receiver_idx,
            positional_count,
            labels,
            selector,
            source_range,
            caller_authority,
            crate::typing::environment::RuntimeTypeEnvironmentId::EMPTY,
            crate::typing::RuntimeConformanceEnvironmentId::EMPTY,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn activate_rest_method_as_with_environment(
        &mut self,
        receiver: &Value,
        method: crate::heap::ObjRef,
        receiver_idx: usize,
        positional_count: usize,
        labels: &[Symbol],
        selector: Symbol,
        source_range: SourceRange,
        caller_authority: (Option<crate::heap::ClassId>, crate::method::RuntimeAccessOrigin),
        type_environment: crate::typing::environment::RuntimeTypeEnvironmentId,
        conformance_environment: crate::typing::RuntimeConformanceEnvironmentId,
    ) -> PhResult<()> {
        let total = self.stack.len() - receiver_idx - 1;
        let signature = self.heap.method(method).signature.clone();
        crate::method::InvocationLayout::validate_signature_shape(&signature, positional_count, labels, false)?;
        let layout = crate::method::InvocationLayout::ordinary(positional_count, labels.to_vec().into_boxed_slice());
        layout.validate_window(receiver_idx, self.stack.len())?;
        if layout.physical_arity() != total {
            return Err(RuntimeError::Internal(format!(
                "rest invocation layout arity {} does not match physical arity {total}",
                layout.physical_arity()
            ))
            .into());
        }
        if matches!(
            self.heap.method(method).kind,
            crate::method::MethodKind::Primitive(crate::method::PrimitiveFn::Shape(_))
        ) {
            return self.call_method_with_selector_as_and_environments(
                receiver,
                method,
                total,
                selector,
                Some(InvocationLayout::ordinary(positional_count, labels.to_vec().into_boxed_slice())),
                source_range,
                caller_authority,
                type_environment,
                conformance_environment,
            );
        }
        self.call_rest_method_as_with_environments(
            receiver,
            method,
            receiver_idx,
            positional_count,
            labels,
            source_range,
            caller_authority,
            type_environment,
            conformance_environment,
        )
    }

    /// Validates all rest-specific installation invariants before mutating
    /// method tables. Both bytecode and shape-aware native methods may occupy
    /// a rest family; the ABI choice is carried by `MethodKind`, not rejected
    /// at installation time.
    fn validate_method_installation(
        &mut self,
        target_class: crate::heap::ClassId,
        selector: Symbol,
        method_id: ObjRef,
    ) -> Result<Option<Symbol>, RuntimeError> {
        let method = self.heap.method(method_id);
        if method.signature.rest.is_none() {
            return Ok(None);
        }

        let (base, _, _) = decode_selector(self.resolve_symbol(selector));
        let base = self.interner.intern(&base);
        if let Some(existing) = self.heap.class(target_class).get_rest_method(base)
            && existing != method_id
            && self.heap.method(existing).signature.selector != selector
        {
            return Err(RuntimeError::DuplicateRestMethodFamily {
                class: self.heap.class(target_class).name.clone(),
                base_family: self.resolve_symbol(base).to_owned(),
                first_selector: self.resolve_symbol(self.heap.method(existing).signature.selector).to_owned(),
                second_selector: self.resolve_symbol(selector).to_owned(),
            });
        }
        Ok(Some(base))
    }

    /// Commits one already-allocated Method into the class dictionary and,
    /// for a rest Method, its secondary base-family index. Validation runs
    /// before either mutation so a duplicate rest family cannot leave the
    /// two indexes out of sync.
    fn install_method_binding(&mut self, target_class: crate::heap::ClassId, selector: Symbol, method: ObjRef) -> PhResult<()> {
        let rest_base = self.validate_method_installation(target_class, selector, method)?;
        self.heap.class_mut(target_class).add_method(selector, method);
        if let Some(base) = rest_base {
            self.heap.class_mut(target_class).add_rest_method(base, method);
        }
        self.world_version += 1;
        Ok(())
    }

    /// Builds a [`CallFrame`] stamped with a fresh, monotonically-increasing
    /// generation for the frame-token infrastructure
    /// ([ADR-0013](../../../docs/adr/0013-block-closure-upvalues.md)).
    ///
    /// Every pushed activation gets its own generation so a [`BlockObject`]
    /// created inside it can later (in U10) tell whether its home activation is
    /// still live. U4 only mints and stores the token; it performs no unwinding.
    pub(crate) fn new_call_frame(
        &mut self,
        closure: ObjRef,
        context: CallContext,
        ip: usize,
        stack_offset: usize,
        caller_source: Option<SourceRange>,
    ) -> CallFrame {
        let generation = self.next_frame_generation;
        self.next_frame_generation = self.next_frame_generation.wrapping_add(1);
        let mut frame = CallFrame::new(closure, context, ip, stack_offset, caller_source);
        frame.generation = generation;
        frame
    }

    /// Returns the [`crate::frame::FrameToken`] of the current (innermost)
    /// activation, or `None` if no frame is active.
    ///
    /// Used by the [`Bytecode::Closure`] handler to stamp a new
    /// [`BlockObject`] with the token of the frame that created it
    /// ([ADR-0013](../../../docs/adr/0013-block-closure-upvalues.md)).
    fn current_frame_token(&self) -> Option<crate::frame::FrameToken> {
        self.frames.last().map(|frame| frame.token(self.frames.len() - 1))
    }

    /// Returns the shared **open** [`Upvalue`] cell for absolute stack slot
    /// `stack_index`, allocating one if this is the first capture of that slot.
    ///
    /// Two closures capturing the same live local therefore receive the *same*
    /// heap cell, so writes through either are observed by both and by the
    /// enclosing frame while the slot is open (blocks.md §5,
    /// [ADR-0013](../../../docs/adr/0013-block-closure-upvalues.md)).
    fn capture_upvalue(&mut self, stack_index: usize) -> ObjRef {
        if let Some(&existing) = self.open_upvalues.get(&stack_index) {
            return existing;
        }
        let cell = self.heap.alloc(Object::Upvalue(Upvalue::Open {
            fiber: self.current,
            slot: stack_index,
        }));
        self.open_upvalues.insert(stack_index, cell);
        cell
    }

    /// Closes (heap-promotes) every open upvalue at absolute stack index
    /// `>= from`, copying the live slot value into the cell before the slot is
    /// reclaimed.
    ///
    /// Called on scope exit ([`Bytecode::CloseUpvalue`]) and on every frame
    /// return so escaping closures keep working after their defining frame is
    /// gone ([ADR-0013](../../../docs/adr/0013-block-closure-upvalues.md)). The
    /// caller must invoke this **before** truncating the stack, while the slot
    /// values are still present.
    fn close_upvalues_from(&mut self, from: usize) {
        let to_close: Vec<usize> = self.open_upvalues.range(from..).map(|(&idx, _)| idx).collect();
        for idx in to_close {
            let cell = self.open_upvalues.remove(&idx).expect("open upvalue present");
            let value = self.stack[idx];
            *self.heap.upvalue_mut(cell) = Upvalue::Closed(value);
        }
    }

    /// [`Self::close_upvalues_from`]'s counterpart for a **parked** fiber:
    /// closes every open upvalue at absolute stack index `>= from` recorded
    /// in `fiber_ref`'s own [`crate::heap::FiberObject::open_upvalues`],
    /// copying out of `fiber_ref`'s own parked
    /// [`crate::heap::FiberObject::stack`] rather than [`Self::stack`].
    ///
    /// `close_upvalues_from` only closes upvalues open against the *live*
    /// mirror (`self.stack`/`self.open_upvalues`), which is exactly the
    /// currently-running fiber's state. A fiber that isn't currently running
    /// keeps its own open upvalues — and the stack slots they read from — in
    /// its parked `FiberObject` fields instead, so closing them needs this
    /// fiber-scoped variant (E002,
    /// [`docs/errors/E002-fiber-floor-upvalue-crash.md`](../../../docs/errors/E002-fiber-floor-upvalue-crash.md)).
    ///
    /// Used by the fiber-floor failure cascade
    /// ([`Self::run_until`]) to close each intermediate `Call`-mode
    /// resumer's open upvalues — captured against *its* parked stack, not
    /// the failing fiber's — before that resumer's parked state is
    /// discarded, mirroring [`Self::unwind_to`]'s close-before-truncate
    /// order for the parked case.
    fn close_fiber_upvalues_from(&mut self, fiber_ref: ObjRef, from: usize) {
        let to_close: Vec<(usize, ObjRef)> = self
            .heap
            .fiber(fiber_ref)
            .open_upvalues
            .range(from..)
            .map(|(&idx, &cell)| (idx, cell))
            .collect();
        for (idx, cell) in to_close {
            let value = self.heap.fiber(fiber_ref).stack[idx];
            *self.heap.upvalue_mut(cell) = Upvalue::Closed(value);
            self.heap.fiber_mut(fiber_ref).open_upvalues.remove(&idx);
        }
    }

    /// Discards every frame/stack slot pushed since a caught `Raise` snapshot,
    /// restoring the VM to the state it was in immediately before the
    /// protected block ran ([`primitive::block::block_on`](crate::primitive::block::block_on),
    /// error-handling.md §2, [ADR-0038](../../docs/adr/0038-amend-floor-admit-block-on-ensure.md)).
    ///
    /// `run_until` deliberately leaves a throwing block's frames and stack
    /// live on an `Err` (so the top-level renderer can source-map the full
    /// trace on an *uncaught* `throw`); once a matching `on(_)` handler has
    /// decided to catch it, those abandoned frames must be torn down before
    /// the handler runs. Order matters: **close upvalues first**, then
    /// truncate — mirroring [`Bytecode::Return`](crate::bytecode::Bytecode::Return)/
    /// [`Bytecode::ReturnNonLocal`](crate::bytecode::Bytecode::ReturnNonLocal)'s
    /// own unwind order exactly (`close_upvalues_from` before
    /// `stack.truncate`) — so a closure that escaped the throwing block still
    /// observes its captured locals rather than a use-after-free once its
    /// stack slot is reclaimed.
    ///
    /// `stack_len`/`frames_len` are **relative** snapshots (`vm.stack.len()`/
    /// `vm.frames.len()` taken by the caller, never an absolute index or
    /// `0`), so this stays fiber-local by construction once a fiber owns its
    /// own frame/stack buffers (ADR-0030 D7) — do not hardcode the main
    /// stack.
    pub(crate) fn unwind_to(&mut self, stack_len: usize, frames_len: usize) {
        self.close_upvalues_from(stack_len);
        self.frames.truncate(frames_len);
        self.stack.truncate(stack_len);
    }

    /// Delivers a non-local return to the target home frame, closing escaping upvalues
    /// and truncating intervening frames and stack.
    pub(crate) fn execute_non_local_return(&mut self, token: FrameToken, return_value: Value) -> PhResult<()> {
        let is_live = self.frames.get(token.frame_index).is_some_and(|home| home.generation == token.generation);
        if !is_live {
            return Err(RuntimeError::DeadFrameError.into());
        }
        let home_stack_offset = self.frames[token.frame_index].stack_offset;
        let return_value = self.surface_absence(return_value);
        self.close_upvalues_from(home_stack_offset);
        self.stack.truncate(home_stack_offset);
        self.stack.push(return_value);
        self.frames.truncate(token.frame_index);
        Ok(())
    }

    /// Renders a runtime-error traceback to stderr and returns the error.
    ///
    /// Uses [`crate::diagnostics::traceback::render_traceback`] — the new renderer
    /// that walks the live call stack through [`crate::vm::walk::StackWalk`] (oldest→newest,
    /// no `.rev()`, no `frames.clone()`) and emits a Python-style traceback with the
    /// innermost-frame caret block (IS §5.1, plan.md T4).
    ///
    /// # Errors
    ///
    /// Always returns `err` (the traceback is a side effect on stderr).
    pub fn report_runtime_error(&mut self, err: &PhError) {
        let config = crate::diagnostics::active_render_config();
        crate::diagnostics::traceback::render_traceback(self, err, &config, self.trace_core, self.trace_format_json);
    }

    /// Renders a runtime-error traceback to stderr and returns the error.
    ///
    /// Uses [`crate::diagnostics::traceback::render_traceback`] — the new renderer
    /// that walks the live call stack through [`crate::vm::walk::StackWalk`] (oldest→newest,
    /// no `.rev()`, no `frames.clone()`) and emits a Python-style traceback with the
    /// innermost-frame caret block (IS §5.1, plan.md T4).
    ///
    /// # Errors
    ///
    /// Always returns `err` (the traceback is a side effect on stderr).
    pub fn runtime_error(&mut self, err: PhError) -> PhResult<()> {
        self.report_runtime_error(&err);
        Err(err)
    }

    /// Reports a compile error to the user.
    ///
    /// This was an empty function body until
    /// [PDR-0008](../../../docs/decisions/0008-cell-boundary-diagnostics-and-state-hygiene.md) §1.
    /// Both callers — [`VM::interpret_source`](crate::vm::VM::interpret_source) and
    /// the REPL's cell loop — believed it printed, so a compile error produced no
    /// diagnostic from either. In the REPL that meant no output whatsoever; in file
    /// mode the message still surfaced, but only because the CLI prints the returned
    /// `Err`, without the source excerpt this renders.
    pub fn compiler_error(&mut self, err: &PhError, module: ObjRef, source_id: u32) {
        if let PhError::Compile(comp_err) = err {
            let (msg, range) = match comp_err {
                crate::compiler::lib::CompilerError::DestructuringWithoutInitializer(range) => (comp_err.to_string(), Some(*range)),
                crate::compiler::lib::CompilerError::BreakOutsideLoop(range) => (comp_err.to_string(), Some(*range)),
                crate::compiler::lib::CompilerError::ContinueOutsideLoop(range) => (comp_err.to_string(), Some(*range)),
                crate::compiler::lib::CompilerError::ThrowNonError(range) => (comp_err.to_string(), Some(*range)),
                crate::compiler::lib::CompilerError::ClassAlreadyDefined(_, _, range, _, _) => (comp_err.to_string(), Some(*range)),
                crate::compiler::lib::CompilerError::DuplicateSelector(_, _, _, range, _, _) => (comp_err.to_string(), Some(*range)),
                crate::compiler::lib::CompilerError::DuplicateField(_, _, _, range, _, _) => (comp_err.to_string(), Some(*range)),
                crate::compiler::lib::CompilerError::ClassReservedName(_, range) => (comp_err.to_string(), Some(*range)),
                crate::compiler::lib::CompilerError::Parse(syntax_err) => (
                    syntax_err.kind.to_string(),
                    Some(phalcom_common::range::SourceRange::from(syntax_err.range.clone())),
                ),
                _ => (comp_err.to_string(), None),
            };
            if let Some(r) = range {
                let m = self.heap.module(module);
                if let Some(src) = m.source_at(source_id) {
                    let config = crate::diagnostics::active_render_config();
                    let styler = crate::diagnostics::style::Styler::new(&config);
                    eprintln!(
                        "{}{} {}",
                        styler.paint(crate::diagnostics::style::Role::SeverityError, "error"),
                        styler.paint(crate::diagnostics::style::Role::SeverityError, ":"),
                        msg
                    );
                    let label = crate::diagnostics::caret::Label {
                        span: r,
                        text: &msg,
                        kind: crate::diagnostics::caret::LabelKind::Primary,
                    };
                    let snippet = crate::diagnostics::caret::Snippet::with_file(m.path.clone());
                    let rendered = snippet.render(src, &[label], &config);
                    eprint!("{rendered}");
                    return;
                }
            }
        }
        print_compile(&err.to_string());
    }

    /// Pops a value from the operand stack.
    ///
    /// # Errors
    ///
    /// Returns [`RuntimeError::Internal`] on stack underflow.
    pub fn pop(&mut self) -> Result<Value, PhError> {
        self.stack.pop().ok_or_else(|| RuntimeError::Internal("Stack underflow".to_string()).into())
    }

    /// Adds a relative instruction-count `offset` to the current (innermost)
    /// frame's instruction pointer.
    ///
    /// Shared by [`Bytecode::Jump`], [`Bytecode::JumpIfFalse`],
    /// [`Bytecode::Loop`], [`Bytecode::GuardBool`] and
    /// [`Bytecode::GuardBlock`] — see [`Bytecode::Jump`]'s doc for the offset
    /// convention (`ip` is already advanced past the jump instruction itself
    /// by the time this runs).
    fn apply_jump_offset(&mut self, offset: i32) {
        let frame = self.frames.last_mut().unwrap();
        frame.ip = (frame.ip as i64 + offset as i64) as usize;
    }

    /// Surfaces the private `nil` sentinel as immediate `None` at a read boundary.
    ///
    /// A thin wrapper over [`sentinel_to_option`](crate::value::sentinel_to_option)
    /// Called wherever the VM can
    /// read an unwritten slot (uninitialized `var`/local, unassigned field,
    /// bare-`return` default, a method falling off its end) so the private
    /// `Value::Nil` sentinel ([ADR-0010](../../../docs/adr/0010-tagged-value-enum.md))
    /// is replaced by `None` and never reaches user code (Invariant 4,
    /// [ADR-0007](../../../docs/adr/0007-option-some-none.md)).
    #[inline]
    pub(crate) fn surface_absence(&self, value: Value) -> Value {
        crate::value::sentinel_to_option(value)
    }

    /// Returns immediate `None` as the surface absence value.
    ///
    /// This is the value every opcode and surface-reachable primitive yields to
    /// signal "no value" — the private [`Value::Nil`] sentinel is never handed
    /// to user code (Invariant 4, [ADR-0007](../../../docs/adr/0007-option-some-none.md)).
    #[inline]
    pub(crate) fn none_value(&self) -> Value {
        Value::none()
    }

    #[inline]
    pub(crate) fn unit_value(&self) -> Value {
        Value::unit()
    }

    /// Runs the dispatch loop until the call stack empties, returning the result.
    ///
    /// # Errors
    ///
    /// Returns any [`RuntimeError`] raised during execution (undefined variable,
    /// method-not-found, unsupported operator, and so on).
    pub fn run(&mut self) -> PhResult<Value> {
        self.run_until(0)
    }

    fn mark_fiber_done(&mut self, fiber: ObjRef, value: Value) {
        self.heap.fiber_mut(fiber).status = crate::heap::FiberStatus::Done;
        self.heap.fiber_mut(fiber).result = value;
    }

    fn mark_fiber_failed(&mut self, fiber: ObjRef, error: Value) {
        self.heap.fiber_mut(fiber).status = crate::heap::FiberStatus::Failed;
        self.heap.fiber_mut(fiber).result = error;
    }

    /// Detaches a terminal observer and admits it as fresh scheduler work.
    /// Observer bytecode never executes inline at the fiber floor.
    fn enqueue_completion_observer(&mut self, fiber: ObjRef) -> PhResult<()> {
        let observer = self.heap.fiber_mut(fiber).completion_observer.take();
        let Some(observer) = observer else {
            return Ok(());
        };
        let observer_fiber = crate::primitive::fiber::new_fiber_ref(self, Value::obj(observer))?;
        self.enqueue_unowned_fiber(observer_fiber)
    }

    /// Runs the dispatch loop until the frame stack shrinks back to
    /// `base_frames`, returning the value produced by the frame that dropped it
    /// there.
    ///
    /// With `base_frames == 0` this is the top-level driver ([`Self::run`]).
    /// Block application ([`crate::primitive::block::block_call`]) uses a
    /// non-zero base to run a single block activation re-entrantly and recover
    /// its result without draining the caller's frames.
    ///
    /// # Errors
    ///
    /// Returns any [`RuntimeError`] raised during execution (undefined variable,
    /// method-not-found, unsupported operator, and so on).
    pub(crate) fn run_until(&mut self, base_frames: usize) -> PhResult<Value> {
        // The fiber-floor capture (ADR-0030 §6, D7/DEC-FIB-A) only applies at
        // `base_frames == 0` — the top-level call. A fiber switch is only
        // ever legal at `native_reentry_depth == 0` (the restricted-yield
        // guard, `fiber.rs`), and every `native_reentry_depth == 0` call site
        // passes `base_frames == 0` (this is `run` calling `run_until(0)`, or
        // this very wrapper recursing after a switch). A nested re-entrant
        // call (`block_call`/`send_dynamic`/`invoke_method_object`) always
        // passes `base_frames == self.frames.len() >= 1` at that point (there
        // is always at least the frame dispatching the send), so it can never
        // hit this branch — its `run_until_inner` result passes straight
        // through unchanged, exactly as before U-FIBER (existing
        // non-local-return goldens, C-FIB-7).
        if base_frames != 0 {
            return self.run_until_inner(base_frames);
        }
        let mut draining_root = false;
        loop {
            match self.run_until_inner(0) {
                Ok(value) => {
                    let finished = self.current;
                    if self.heap.fiber(finished).is_root {
                        if !draining_root {
                            // Keep the host activation's outcome independently
                            // of values returned by the scheduler pump. The root
                            // Fiber is GC-traced while child work is running.
                            self.heap.fiber_mut(finished).result = value;
                            draining_root = true;
                        }
                        if let Some(next) = self.pop_next_queued() {
                            crate::primitive::fiber::store_live_into(self, finished);
                            self.resume_queued_fiber(next)?;
                            continue;
                        }
                        self.drain_and_deliver_reactor_completions()?;
                        if let Some(next) = self.pop_next_queued() {
                            crate::primitive::fiber::store_live_into(self, finished);
                            self.resume_queued_fiber(next)?;
                            continue;
                        }
                        if self.reactor.has_pending_progress() {
                            crate::primitive::fiber::store_live_into(self, finished);
                            self.reactor_idle_wait()?;
                            if let Some(next) = self.pop_next_queued() {
                                self.resume_queued_fiber(next)?;
                                continue;
                            }
                        }
                        self.report_unhandled_scheduler_failures()?;
                        return Ok(self.heap.fiber(finished).result);
                    }

                    // Fiber-floor capture, success path (spec §3.2): `finished`
                    // is a non-root fiber whose entry activation just drained
                    // to nothing.
                    self.mark_fiber_done(finished, value);
                    self.enqueue_completion_observer(finished)?;

                    if self.trace_fibers {
                        let finished_seq = self.heap.fiber(finished).seq;
                        let frames_cnt = self.frames.len();
                        let val_str = value.to_string(self);
                        if self.trace_format_json {
                            tracing::debug!(target: "fibers", "{{\"ev\":\"done\",\"fiber\":{},\"result\":{},\"frames\":{}}}", finished_seq, crate::diagnostics::traceback::json_str(&val_str), frames_cnt);
                        } else {
                            tracing::debug!(target: "fibers", "[fiber] done    #{}  result={}  frames={}", finished_seq, val_str, frames_cnt);
                        }
                    }

                    // Recycle the finished fiber's buffers into the pool to
                    // avoid allocations (U-GC step 5, `fiber-pool` feature —
                    // off by default; measured net negative, perf-log 2026-07-14).
                    #[cfg(feature = "fiber-pool")]
                    {
                        let mut frames = std::mem::take(&mut self.frames);
                        let mut stack = std::mem::take(&mut self.stack);
                        frames.clear();
                        stack.clear();
                        if self.fiber_pool.len() < 100 {
                            self.fiber_pool.push((stack, frames));
                        }
                    }

                    let consumer = self.heap.fiber_mut(finished).consumer.take();
                    if let Some(consumer) = consumer {
                        self.switch_to_fiber_and_deliver(consumer.fiber, value);
                        continue;
                    }

                    // No consumer: detached scheduled work finished.
                    self.drain_and_deliver_reactor_completions()?;
                    if let Some(next) = self.pop_next_queued() {
                        self.resume_queued_fiber(next)?;
                        continue;
                    }

                    if let Some(driver) = self.scheduler_drivers.pop() {
                        self.switch_to_fiber_and_deliver(driver, Value::nil());
                        continue;
                    }

                    if self.reactor.has_pending_progress() {
                        self.reactor_idle_wait()?;
                        if let Some(next) = self.pop_next_queued() {
                            self.resume_queued_fiber(next)?;
                            continue;
                        }
                        if let Some(driver) = self.scheduler_drivers.pop() {
                            self.switch_to_fiber_and_deliver(driver, Value::nil());
                            continue;
                        }
                    }

                    if let Some(parked_leaf) = self.find_root_blocking_parked_leaf() {
                        self.heap.fiber_mut(parked_leaf).park_generation += 1;
                        self.heap.fiber_mut(parked_leaf).status = crate::heap::FiberStatus::Running;
                        crate::primitive::fiber::load_live_from(self, parked_leaf);
                        let msg = "await: the future is still pending and the scheduler is empty; nothing can settle it".to_string();
                        let class = self.universe.classes.error_class;
                        let field_count = self.heap.class(class).field_count;
                        let mut inst = crate::heap::InstanceObject::new(class, field_count);
                        inst.slots[0] = self.alloc_string_value(msg.clone());
                        let error = Value::obj(self.heap.alloc(Object::Instance(inst)));
                        return Err(RuntimeError::Raise {
                            error,
                            rendered: msg,
                            traceback: None,
                            help: None,
                        }
                        .into());
                    }

                    self.report_unhandled_scheduler_failures()?;
                    return Ok(self.heap.fiber(self.root_fiber).result);
                }
                Err(mut e) => {
                    if !self.control_stack.is_empty() {
                        match self.step_control_transfer(Transfer::Raise(e.clone()))? {
                            ControlStepOutcome::Continued => continue,
                            ControlStepOutcome::Completed(v) => {
                                let finished = self.current;
                                if self.heap.fiber(finished).is_root {
                                    return Ok(v);
                                }
                                continue;
                            }
                            ControlStepOutcome::Propagate(Transfer::Raise(e2)) => {
                                e = e2;
                            }
                            ControlStepOutcome::Propagate(Transfer::Returned(v)) => {
                                self.stack.push(v);
                                continue;
                            }
                            ControlStepOutcome::Propagate(Transfer::NonLocalReturn { target, value }) => {
                                self.execute_non_local_return(target, value)?;
                                continue;
                            }
                        }
                    }

                    let error_value = self.capture_error_value(&e);
                    self.close_upvalues_from(0);
                    let mut failed = self.current;
                    loop {
                        self.mark_fiber_failed(failed, error_value);

                        if self.trace_fibers {
                            let failed_seq = self.heap.fiber(failed).seq;
                            let error_str = error_value.to_string(self);
                            if self.trace_format_json {
                                tracing::debug!(target: "fibers", "{{\"ev\":\"fail\",\"fiber\":{},\"error\":{}}}", failed_seq, crate::diagnostics::traceback::json_str(&error_str));
                            } else {
                                tracing::debug!(target: "fibers", "[fiber] fail    #{}  error={}", failed_seq, error_str);
                            }
                        }

                        if failed == self.current {
                            self.close_upvalues_from(0);
                        } else {
                            self.close_fiber_upvalues_from(failed, 0);
                        }

                        let consumer = self.heap.fiber_mut(failed).consumer.take();
                        let had_completion_owner = self.heap.fiber(failed).completion_observer.is_some();

                        self.heap.fiber_mut(failed).frames.clear();
                        self.heap.fiber_mut(failed).stack.clear();
                        self.heap.fiber_mut(failed).open_upvalues.clear();

                        if consumer.is_none() && !had_completion_owner {
                            self.record_unhandled_scheduler_failure(failed, error_value);
                        }
                        self.enqueue_completion_observer(failed)?;

                        let Some(consumer) = consumer else {
                            self.drain_and_deliver_reactor_completions()?;
                            if let Some(next) = self.pop_next_queued() {
                                self.resume_queued_fiber(next)?;
                                break;
                            }
                            if let Some(driver) = self.scheduler_drivers.pop() {
                                self.switch_to_fiber_and_deliver(driver, Value::nil());
                                break;
                            }
                            if self.reactor.has_pending_progress() {
                                self.reactor_idle_wait()?;
                                if let Some(next) = self.pop_next_queued() {
                                    self.resume_queued_fiber(next)?;
                                    break;
                                }
                                if let Some(driver) = self.scheduler_drivers.pop() {
                                    self.switch_to_fiber_and_deliver(driver, Value::nil());
                                    break;
                                }
                            }
                            if self.heap.fiber(failed).is_root {
                                return Err(e);
                            }
                            if let Some(parked_leaf) = self.find_root_blocking_parked_leaf() {
                                self.heap.fiber_mut(parked_leaf).park_generation += 1;
                                self.heap.fiber_mut(parked_leaf).status = crate::heap::FiberStatus::Running;
                                crate::primitive::fiber::load_live_from(self, parked_leaf);
                                let msg = "await: the future is still pending and the scheduler is empty; nothing can settle it".to_string();
                                let class = self.universe.classes.error_class;
                                let field_count = self.heap.class(class).field_count;
                                let mut inst = crate::heap::InstanceObject::new(class, field_count);
                                inst.slots[0] = self.alloc_string_value(msg.clone());
                                let error = Value::obj(self.heap.alloc(Object::Instance(inst)));
                                return Err(RuntimeError::Raise {
                                    error,
                                    rendered: msg,
                                    traceback: None,
                                    help: None,
                                }
                                .into());
                            }
                            self.report_unhandled_scheduler_failures()?;
                            return Ok(self.heap.fiber(self.root_fiber).result);
                        };

                        let resumer = consumer.fiber;
                        let mode = consumer.mode;

                        let (native_error, unwrapped_error) = e.into_underlying();
                        e = native_error;
                        if !matches!(e, PhError::Runtime(RuntimeError::Raise { .. })) {
                            let error = error_value;
                            let rendered = e.to_string();
                            let mut tb = if failed == self.current {
                                self.capture_frames(0)
                            } else {
                                self.capture_parked_frames(failed)
                            };
                            if let Some(origin) = unwrapped_error {
                                tb.push(crate::error::FrameRecord::Native {
                                    class: origin.class,
                                    selector: origin.selector,
                                });
                            }
                            e = PhError::Runtime(RuntimeError::Raise {
                                error,
                                rendered,
                                traceback: Some(tb),
                                help: None,
                            });
                        } else if let Some(origin) = unwrapped_error {
                            if let PhError::Runtime(RuntimeError::Raise { traceback, .. }) = &mut e {
                                let records = traceback.get_or_insert_with(Vec::new);
                                if !records.iter().any(|record| {
                                    matches!(
                                        record,
                                        crate::error::FrameRecord::Native { class, selector }
                                            if *class == origin.class && *selector == origin.selector
                                    )
                                }) {
                                    records.push(crate::error::FrameRecord::Native {
                                        class: origin.class,
                                        selector: origin.selector,
                                    });
                                }
                            }
                        }

                        if let PhError::Runtime(RuntimeError::Raise { traceback: Some(tb), .. }) = &mut e {
                            let failed_fiber = self.heap.fiber(failed);
                            tb.push(crate::error::FrameRecord::FiberBoundary {
                                seq: failed_fiber.seq,
                                spawn_file: failed_fiber.spawn_file,
                                spawn_line: failed_fiber.spawn_line,
                            });
                            let resumer_frames = self.capture_parked_frames(resumer);
                            tb.extend(resumer_frames);
                        }

                        match mode {
                            crate::heap::FiberConsumerMode::Try => {
                                self.switch_to_fiber_and_deliver(resumer, error_value);
                                break;
                            }
                            crate::heap::FiberConsumerMode::Call => {
                                self.switch_to_fiber_without_deliver(resumer);
                                if !self.control_stack.is_empty() {
                                    match self.step_control_transfer(Transfer::Raise(e.clone()))? {
                                        ControlStepOutcome::Continued => break,
                                        ControlStepOutcome::Completed(_) => break,
                                        ControlStepOutcome::Propagate(Transfer::Raise(e2)) => {
                                            e = e2;
                                            failed = resumer;
                                            continue;
                                        }
                                        ControlStepOutcome::Propagate(Transfer::Returned(_)) => break,
                                        ControlStepOutcome::Propagate(Transfer::NonLocalReturn { target, value }) => {
                                            self.execute_non_local_return(target, value)?;
                                            break;
                                        }
                                    }
                                } else {
                                    failed = resumer;
                                }
                            }
                        }
                    }
                    // Loop again: keep draining, now as the fiber the
                    // cascade stopped at.
                }
            }
        }
    }

    /// Switches [`Self::current`] to `target`, restoring its parked live
    /// state and delivering `value` into the stack window its own park
    /// recorded ([`crate::heap::FiberObject::resume_slot`]) — the shared
    /// "land a value at a fiber's resume point" step used by every direction
    /// a switch can complete (an ordinary `yield`/`call` handoff, and the
    /// fiber-floor capture's success/`try`-failure delivery, ADR-0030 §3/§6).
    ///
    /// `pub(crate)` so [`crate::primitive::fiber::fiber_yield`] can reuse it
    /// instead of hand-inlining the same four-step sequence.
    pub(crate) fn switch_to_fiber_and_deliver(&mut self, target: ObjRef, value: Value) {
        debug_assert_eq!(self.heap.fiber(target).status, crate::heap::FiberStatus::BlockedOnChild);
        if self.trace_fibers {
            let from_seq = self.heap.fiber(self.current).seq;
            let to_seq = self.heap.fiber(target).seq;

            // Resolve location where switch/resume happened
            let mut from_file = None;
            let mut from_line = 0;
            let mut from_name = None;
            if let Some(top) = self.frames.last() {
                let closure = self.heap.closure(top.closure);
                let module = self.heap.module(closure.module);
                let span_index = top.ip.saturating_sub(1);
                from_line = closure
                    .callable
                    .chunk
                    .line_at(span_index, module.source_at(closure.callable.chunk.source_id).map_or("", |s| s.as_str()));
                from_file = Some(module.name_sym);
                from_name = Some(closure.callable.name_sym);
            }

            let mut to_file = None;
            let mut to_line = 0;
            // Target fiber's frames are parked in target.frames
            if let Some(top) = self.heap.fiber(target).frames.last() {
                let closure = self.heap.closure(top.closure);
                let module = self.heap.module(closure.module);
                let span_index = top.ip.saturating_sub(1);
                to_line = closure
                    .callable
                    .chunk
                    .line_at(span_index, module.source_at(closure.callable.chunk.source_id).map_or("", |s| s.as_str()));
                to_file = Some(module.name_sym);
            }

            if self.trace_format_json {
                let mut json = format!("{{\"ev\":\"switch\",\"from\":{},\"to\":{}", from_seq, to_seq);
                if let Some(f) = from_file {
                    json.push_str(&format!(",\"from_at\":{{\"file\":\"{}.ph\",\"line\":{}}}", self.resolve_symbol(f), from_line));
                }
                if let Some(f) = to_file {
                    json.push_str(&format!(",\"to_at\":{{\"file\":\"{}.ph\",\"line\":{}}}", self.resolve_symbol(f), to_line));
                }
                json.push('}');
                tracing::debug!(target: "fibers", "{}", json);
            } else {
                let mut from_str = String::new();
                if let Some(f) = from_file {
                    let name_str = from_name.map_or(String::new(), |s| format!(" <{}>", self.resolve_symbol(s)));
                    from_str = format!("{} @ {}.ph:{}", name_str, self.resolve_symbol(f), from_line);
                }
                let mut to_str = String::new();
                if let Some(f) = to_file {
                    to_str = format!(" @ {}.ph:{}", self.resolve_symbol(f), to_line);
                }
                tracing::debug!(target: "fibers", "[fiber] switch  #{}{}  ──→  #{}{}", from_seq, from_str, to_seq, to_str);
            }
        }

        self.current = target;
        crate::primitive::fiber::load_live_from(self, target);
        let slot = self.heap.fiber(target).resume_slot;
        self.stack.truncate(slot);
        self.stack.push(value);
        self.heap.fiber_mut(target).status = crate::heap::FiberStatus::Running;
    }

    /// Restores [`Self::current`] to `target` without pushing a success operand.
    /// Used when a child fiber fails under `Call` mode and an exception must be injected
    /// into the parent's control continuation / resume site.
    pub(crate) fn switch_to_fiber_without_deliver(&mut self, target: ObjRef) {
        debug_assert_eq!(self.heap.fiber(target).status, crate::heap::FiberStatus::BlockedOnChild);
        if self.trace_fibers {
            let from_seq = self.heap.fiber(self.current).seq;
            let to_seq = self.heap.fiber(target).seq;
            if self.trace_format_json {
                tracing::debug!(target: "fibers", "{{\"ev\":\"restore\",\"from\":{},\"to\":{}}}", from_seq, to_seq);
            } else {
                tracing::debug!(target: "fibers", "[fiber] restore  #{}  ──→  #{}", from_seq, to_seq);
            }
        }
        self.current = target;
        crate::primitive::fiber::load_live_from(self, target);
        let slot = self.heap.fiber(target).resume_slot;
        self.stack.truncate(slot);
        self.heap.fiber_mut(target).status = crate::heap::FiberStatus::Running;
    }

    /// Extracts the surface `Error` [`Value`] a fiber-floor capture stores in
    /// [`crate::heap::FiberObject::result`] (ADR-0030 §6, spec §3.2).
    ///
    /// [`RuntimeError::Raise`]'s `error` is already the surface instance
    /// (U-CORE-6); any other terminal error (a native VM error with no
    /// surface reification) is wrapped in a bare [`crate::heap::Object::Instance`]
    /// of the kernel `Error` class carrying its rendered message, so a
    /// fiber's failure is always a catchable `Error` value, never a native
    /// Rust error type leaking across the fiber boundary.
    fn capture_error_value(&mut self, e: &PhError) -> Value {
        if let PhError::Runtime(RuntimeError::Raise { error, .. }) = e {
            return *error;
        }
        let error_class = self.universe.classes.error_class;
        let field_count = self.heap.class(error_class).field_count;
        let mut inst = crate::heap::InstanceObject::new(error_class, field_count);
        inst.slots[0] = self.alloc_string_value(e.to_string());
        if let PhError::Runtime(runtime_err) = e {
            if let Some(kind_val) = self.error_kind_symbol(runtime_err) {
                inst.slots[1] = kind_val;
            }
        }
        Value::obj(self.heap.alloc(Object::Instance(inst)))
    }

    /// Executes one `Invoke`-shaped send: IC probe, exact-selector lookup + refill,
    /// rest-family fallback, then `doesNotUnderstand(_)` forward — method-lookup.md §1's
    /// miss order, in order.
    ///
    /// `cache_ip` is the index whose `caches`/`spans` slots this send owns. For a
    /// plain [`Bytecode::Invoke`] that is the instruction's own `ip`; for the fused
    /// [`Bytecode::InvokeLocal`]/[`Bytecode::InvokeConst`] it is `ip + 1` — the dead
    /// `Invoke` the fusion left behind (`Chunk::fuse_superinstructions`) — so a fused
    /// send probes the same cache slot and reports the same span as the pair it
    /// replaced.
    ///
    /// `callable` is the caller's hoisted [`Callable`] (F14 S1a). It is borrowed from a
    /// local `Rc`, not from `self.heap`, which is what lets this take `&mut self`.
    ///
    /// # Errors
    ///
    /// Propagates any [`RuntimeError`] the dispatched method raises.
    fn trait_method_for_selection(
        &mut self,
        receiver: Value,
        selection: &phalcom_semantic::impls::RequirementSelectionTemplate,
        requirement_selector: &phalcom_common::selector::Selector,
    ) -> PhResult<ObjRef> {
        use phalcom_semantic::impls::RequirementSelectionTemplate;
        if let RequirementSelectionTemplate::DataComponent { component, .. } = selection {
            // Semantic evidence has already selected this exact data component.
            // Runtime lookup only materializes the representation-level getter
            // installed by register_data_from_spec; it does not select a witness.
            let selector = self.interner.intern(&requirement_selector.encode());
            return receiver
                .lookup_method(self, selector)
                .ok_or_else(|| {
                    RuntimeError::Internal(format!(
                        "trait-selected data component {component:?} has no runtime projection getter for `{}`",
                        requirement_selector.encode()
                    ))
                })
                .map_err(Into::into);
        }
        let callable = match selection {
            RequirementSelectionTemplate::ConformanceCallable { callable } | RequirementSelectionTemplate::TraitDefault { callable } => Some(callable),
            RequirementSelectionTemplate::InherentCallable { callable, .. } => Some(callable),
            RequirementSelectionTemplate::ConditionalInherent { candidate, .. } => Some(&candidate.callable),
            RequirementSelectionTemplate::DataComponent { .. } => None,
        };
        let Some(callable) = callable else {
            return Err(RuntimeError::Internal("data component trait target requires projection lowering".into()).into());
        };
        self.resolve_conformance_method(callable, &callable.selector, receiver)
    }

    fn materialize_trait_environment(
        &mut self,
        receiver: Value,
        spec: &crate::modules::semantic_lowering::TraitInvocationSpec,
    ) -> PhResult<(crate::typing::RuntimeConformanceEnvironmentId, ObjRef, crate::typing::RuntimeTypeEnvironmentId)> {
        let mut bindings = spec
            .type_bindings
            .iter()
            .cloned()
            .map(|(parameter, recipe)| (parameter, crate::typing::RuntimeCallArgumentRecipe::Term(recipe)))
            .collect::<std::collections::BTreeMap<_, _>>();
        if let Some(recipe) = &spec.call_environment_recipe {
            for (parameter, ty) in recipe.bindings.iter().cloned() {
                if bindings.insert(parameter, ty.clone()).is_some_and(|previous| previous != ty) {
                    return Err(RuntimeError::Internal("trait owner and method type bindings disagree".into()).into());
                }
            }
        }
        let type_environment = self.materialize_type_bindings(&bindings.into_iter().collect::<Vec<_>>())?;
        let is_trait_default = matches!(
            spec.selection.selection,
            phalcom_semantic::impls::RequirementSelectionTemplate::TraitDefault { .. }
        );
        let max_req_slot = if is_trait_default {
            spec.requirement_slots.iter().map(|(_, slot)| *slot as usize + 1).max().unwrap_or(0)
        } else {
            0
        };
        let max_ant_slot = spec.antecedent_slots.iter().map(|(slot, _, _)| *slot as usize + 1).max().unwrap_or(0);
        let mut slots = vec![None; max_req_slot.max(max_ant_slot)];
        if let Some(recipe) = &spec.call_environment_recipe {
            let environment = self.materialize_conformance_environment(receiver, recipe)?;
            for (index, entry) in self
                .runtime_conformance_environments
                .get(environment)
                .ok_or_else(|| RuntimeError::Internal("trait call-entry conformance environment is unavailable".into()))?
                .slots
                .iter()
                .copied()
                .enumerate()
            {
                if index >= slots.len() {
                    slots.resize(index + 1, None);
                }
                slots[index] = entry;
            }
        }

        for (slot, target, selector) in spec.antecedent_slots.iter() {
            let method = self.trait_method_for_selection(receiver, target, selector)?;
            if (*slot as usize) < slots.len() {
                slots[*slot as usize] = Some(crate::typing::RuntimeConformanceSlot {
                    method,
                    type_environment,
                    conformance_environment: crate::typing::RuntimeConformanceEnvironmentId::EMPTY,
                });
            }
        }

        if is_trait_default {
            for (requirement, slot) in spec.requirement_slots.iter() {
                if let Some(target) = spec.selection.requirement_targets.get(requirement) {
                    let method = self.trait_method_for_selection(receiver, target, &requirement.selector)?;
                    if (*slot as usize) < slots.len() {
                        slots[*slot as usize] = Some(crate::typing::RuntimeConformanceSlot {
                            method,
                            type_environment,
                            conformance_environment: crate::typing::RuntimeConformanceEnvironmentId::EMPTY,
                        });
                    }
                }
            }
        }
        let environment = self
            .runtime_conformance_environments
            .intern(crate::typing::RuntimeConformanceEnvironment::new(slots));
        let method = self.trait_method_for_selection(receiver, &spec.selection.selection, &spec.selection.signature.selector)?;
        if let Some(recipe) = &spec.call_environment_recipe {
            self.validate_specialized_call(recipe, method)?;
        }
        Ok((environment, method, type_environment))
    }

    fn invoke_trait_selected(&mut self, callable: &Callable, selection_index: u16, arity: u8) -> PhResult<()> {
        let spec = callable.chunk.executable_semantics.trait_invocation(selection_index).clone();
        let receiver_idx = self
            .stack
            .len()
            .checked_sub(arity as usize + 1)
            .ok_or(RuntimeError::Internal("trait invocation stack underflow".into()))?;
        let receiver = self.stack[receiver_idx];
        let (conformance_environment, method, type_environment) = self.materialize_trait_environment(receiver, &spec)?;
        let selector = self.interner.intern(&spec.selection.signature.selector.encode());
        let layout = self.invocation_layout_for_selector(selector, arity as usize)?;
        self.call_method_with_selector_as_and_environments(
            &receiver,
            method,
            arity as usize,
            selector,
            Some(layout),
            callable.chunk.span_at(self.frames.last().map(|frame| frame.ip.saturating_sub(1)).unwrap_or(0)),
            self.current_caller_authority(),
            type_environment,
            conformance_environment,
        )
    }

    fn invoke_trait_requirement(&mut self, callable: &Callable, spec_index: u16, arity: u8) -> PhResult<()> {
        let spec = callable.chunk.executable_semantics.trait_requirement_invocation(spec_index).clone();
        let conformance_environment = self
            .frames
            .last()
            .map(|frame| frame.conformance_environment)
            .unwrap_or(crate::typing::RuntimeConformanceEnvironmentId::EMPTY);
        let slot_entry = self
            .runtime_conformance_environments
            .slot(conformance_environment, spec.requirement_slot)
            .ok_or_else(|| RuntimeError::Internal(format!("no active conformance target for requirement {:?}", spec.requirement)))?;
        let method = slot_entry.method;
        let receiver_idx = self
            .stack
            .len()
            .checked_sub(arity as usize + 1)
            .ok_or(RuntimeError::Internal("trait requirement stack underflow".into()))?;
        let receiver = if spec.requirement.side == phalcom_semantic::identity::DispatchSide::Class {
            if let Some(holder) = self.heap.method(method).holder {
                self.stack[receiver_idx] = Value::obj(holder);
                Value::obj(holder)
            } else {
                self.stack[receiver_idx]
            }
        } else {
            self.stack[receiver_idx]
        };
        let selector = self.interner.intern(&spec.requirement.selector.encode());
        let layout = self.invocation_layout_for_selector(selector, arity as usize)?;
        let type_environment = if slot_entry.type_environment != crate::typing::RuntimeTypeEnvironmentId::EMPTY {
            slot_entry.type_environment
        } else {
            self.frames
                .last()
                .map(|frame| frame.type_environment)
                .unwrap_or(crate::typing::environment::RuntimeTypeEnvironmentId::EMPTY)
        };
        let method_conf_env = if slot_entry.conformance_environment != crate::typing::RuntimeConformanceEnvironmentId::EMPTY {
            slot_entry.conformance_environment
        } else {
            conformance_environment
        };
        self.call_method_with_selector_as_and_environments(
            &receiver,
            method,
            arity as usize,
            selector,
            Some(layout),
            callable.chunk.span_at(self.frames.last().map(|frame| frame.ip.saturating_sub(1)).unwrap_or(0)),
            self.current_caller_authority(),
            type_environment,
            method_conf_env,
        )
    }

    fn invoke_at(
        &mut self,
        callable: &Callable,
        cache_ip: usize,
        arity: u8,
        selector_idx: u16,
        call_environment: Option<&crate::typing::RuntimeCallEnvironmentRecipe>,
    ) -> PhResult<()> {
        let arity = arity as usize;
        let receiver_idx = self.stack.len() - 1 - arity;
        let receiver = self.stack[receiver_idx];
        let receiver_class = receiver.class(self);
        let selector_val = callable.chunk.constants[selector_idx as usize];
        let selector_sym = selector_val.as_symbol().unwrap();
        let layout = self.invocation_layout_for_selector(selector_sym, arity)?;
        let source_range = callable.chunk.span_at(cache_ip);

        // A specialized call through Function.call on a behavioral Family is
        // still a live Family dispatch. Defer recipe validation and
        // materialization until the Family has derived its selector and
        // selected the captured receiver's actual method.
        if let Some(recipe) = call_environment
            && self.resolve_symbol(selector_sym) == "call(***)"
            && receiver.as_obj().is_some_and(|id| matches!(self.heap.get(id), Object::Family(_)))
        {
            let authority = self.current_caller_authority();
            let view = ArgumentView::from_layout(receiver_idx, layout, authority.0, authority.1);
            self.activate_family_with_kind_and_recipe(view, super::send::FamilyInvocationKind::Method, source_range, Some(recipe))?;
            return Ok(());
        }

        let type_environment = call_environment.map(|recipe| self.materialize_call_environment(recipe)).transpose()?;
        let conformance_environment = call_environment
            .map(|recipe| self.materialize_conformance_environment(receiver, recipe))
            .transpose()?;

        // A Family owns the selector shape of a callable reference. Bracket
        // sends therefore activate the family directly instead of looking up
        // the Family protocol's ordinary methods.
        let (_, _, kind) = decode_selector(self.resolve_symbol(selector_sym));
        if matches!(kind, SignatureKind::SubscriptGet(_) | SignatureKind::SubscriptSet(_))
            && receiver
                .as_obj()
                .is_some_and(|id| matches!(self.heap.get(id), Object::Family(_) | Object::AssociatedFamily(_)))
        {
            self.dispatch_selector_window_as(
                receiver_idx,
                selector_sym,
                layout.clone(),
                callable.chunk.span_at(cache_ip),
                self.current_caller_authority(),
            )?;
            return Ok(());
        }

        // Cache probe. `chunk` is a shared borrow; the `Cell` is what lets us
        // write back through it (U-IC §2.1). `span_at(cache_ip)` rides along in
        // the same borrow (S2): the loop head no longer reads it, and this arm
        // needs it for whichever `call_method` / dNU forward it reaches.
        let (cached, source_range) = {
            let chunk = &callable.chunk;
            let cached = chunk.caches[cache_ip]
                .get()
                .filter(|slot| slot.class == receiver_class && slot.version == self.world_version)
                .map(|slot| slot.method);
            (cached, chunk.span_at(cache_ip))
        };

        let type_environment = type_environment.unwrap_or(crate::typing::RuntimeTypeEnvironmentId::EMPTY);
        let conformance_environment = conformance_environment.unwrap_or(crate::typing::RuntimeConformanceEnvironmentId::EMPTY);
        if let Some(()) = self.try_module_export_send_with_environment(
            receiver_idx,
            selector_sym,
            arity,
            layout.clone(),
            source_range,
            type_environment,
            call_environment,
        )? {
            return Ok(());
        }

        if let Some(method) = cached {
            if let Some(recipe) = call_environment {
                self.validate_specialized_call(recipe, method)?;
            }
            if self.heap.method(method).signature.rest.is_some() {
                let (name, slots, kind) = decode_selector(self.resolve_symbol(selector_sym));
                let positional_count = slots.iter().filter(|slot| slot.is_none()).count();
                let labels = slots
                    .iter()
                    .filter_map(|slot| slot.as_ref())
                    .map(|label| self.interner.intern(label))
                    .collect::<Vec<_>>();
                if !matches!(kind, SignatureKind::Method(_)) {
                    return self.forward_does_not_understand(receiver_idx, selector_sym, source_range);
                }
                self.activate_rest_method_with_environment(
                    &receiver,
                    method,
                    receiver_idx,
                    positional_count,
                    &labels,
                    selector_sym,
                    source_range,
                    type_environment,
                    conformance_environment,
                )?;
            } else {
                self.call_method_with_selector_as_and_environments(
                    &receiver,
                    method,
                    arity,
                    selector_sym,
                    Some(layout.clone()),
                    source_range,
                    self.current_caller_authority(),
                    type_environment,
                    conformance_environment,
                )?;
            }
        } else {
            if let Some(method) = receiver.lookup_method(self, selector_sym) {
                // Refill. Both `receiver_class` and `world_version` are read
                // AFTER the lookup on purpose — see U-IC §2.3 hazard 2.
                let entry = crate::chunk::InlineCache {
                    class: receiver_class,
                    method,
                    version: self.world_version,
                };
                callable.chunk.caches[cache_ip].set(Some(entry));
                if let Some(recipe) = call_environment {
                    self.validate_specialized_call(recipe, method)?;
                }
                self.call_method_with_selector_as_and_environments(
                    &receiver,
                    method,
                    arity,
                    selector_sym,
                    Some(layout.clone()),
                    source_range,
                    self.current_caller_authority(),
                    type_environment,
                    conformance_environment,
                )?;
            } else {
                let (name, slots, kind) = decode_selector(self.resolve_symbol(selector_sym));
                let positional_count = slots.iter().filter(|slot| slot.is_none()).count();
                let labels = slots
                    .iter()
                    .filter_map(|slot| slot.as_ref())
                    .map(|label| self.interner.intern(label))
                    .collect::<Vec<_>>();
                let rest = matches!(kind, SignatureKind::Method(_))
                    .then(|| self.interner.intern(&name))
                    .and_then(|base| self.lookup_rest_method(receiver_class, base, positional_count, &labels));
                if let Some(method) = rest {
                    let entry = crate::chunk::InlineCache {
                        class: receiver_class,
                        method,
                        version: self.world_version,
                    };
                    callable.chunk.caches[cache_ip].set(Some(entry));
                    self.activate_rest_method_with_environment(
                        &receiver,
                        method,
                        receiver_idx,
                        positional_count,
                        &labels,
                        selector_sym,
                        source_range,
                        type_environment,
                        conformance_environment,
                    )?;
                } else {
                    self.forward_does_not_understand(receiver_idx, selector_sym, source_range)?;
                }
            }
        }
        Ok(())
    }

    /// Confirms that a specialized callsite still dispatches to the canonical
    /// callable identity published by semantic analysis. The selector check is
    /// the fail-closed floor; loaded metadata supplies the owner/side identity
    /// through the VM method semantic index.
    pub(crate) fn validate_specialized_call(
        &self,
        recipe: &crate::typing::RuntimeCallEnvironmentRecipe,
        method: crate::heap::ObjRef,
    ) -> Result<(), RuntimeError> {
        let selected_selector = self.resolve_symbol(self.heap.method(method).signature.selector);
        if selected_selector != recipe.callable.selector.as_ref() {
            return Err(RuntimeError::Internal(format!(
                "specialized call selected `{selected_selector}` for `{}`",
                recipe.callable.selector
            )));
        }
        let Some(call_ref) = self.typing_registry.method_semantics.get(method) else {
            if matches!(
                recipe.callable.owner.enclosing_declaration().map(|declaration| &declaration.module.project),
                Some(phalcom_type_meta::identity::StableProjectRef::Builtin { .. })
            ) {
                return Ok(());
            }
            return Err(RuntimeError::Internal(
                "specialized call: selected method has no canonical runtime callable metadata".into(),
            ));
        };
        let actual = self
            .typing_registry
            .get_pool(call_ref.pool)
            .and_then(|pool| pool.bundle.callables.get(call_ref.record.0 as usize))
            .ok_or_else(|| RuntimeError::Internal("specialized call metadata reference is unavailable".into()))?;
        if actual.callable != recipe.callable {
            return Err(RuntimeError::Internal("specialized call canonical callable identity mismatch".into()));
        }
        Ok(())
    }

    pub(super) fn materialize_call_environment(
        &mut self,
        recipe: &crate::typing::RuntimeCallEnvironmentRecipe,
    ) -> Result<crate::typing::RuntimeTypeEnvironmentId, RuntimeError> {
        self.materialize_type_bindings(&recipe.bindings)
    }

    fn materialize_type_bindings(
        &mut self,
        bindings: &[(phalcom_type_meta::StableTypeParameterRef, crate::typing::RuntimeCallArgumentRecipe)],
    ) -> Result<crate::typing::RuntimeTypeEnvironmentId, RuntimeError> {
        let parent_id = self
            .frames
            .last()
            .map(|frame| frame.type_environment)
            .unwrap_or(crate::typing::RuntimeTypeEnvironmentId::EMPTY);
        let parent = self
            .runtime_type_environments
            .get(parent_id)
            .cloned()
            .ok_or_else(|| RuntimeError::Internal(format!("missing caller runtime type environment {}", parent_id.0)))?;
        let mut materialized = Vec::with_capacity(bindings.len());
        for (parameter, argument_recipe) in bindings.iter().cloned() {
            let argument = match argument_recipe {
                crate::typing::RuntimeCallArgumentRecipe::Term(type_recipe) => {
                    let ty = crate::typing::instantiate_type_recipe(
                        type_recipe,
                        &parent,
                        &mut self.runtime_typing_context,
                        &self.typing_registry,
                    )
                    .ok_or_else(|| {
                        RuntimeError::Internal(format!(
                            "generic call-entry type recipe could not be instantiated for parameter {parameter:?}, recipe {type_recipe:?}, against parent {:?}",
                            parent.bindings
                        ))
                    })?;
                    crate::typing::overlay::RuntimeStaticArgument::Term(ty)
                }
                crate::typing::RuntimeCallArgumentRecipe::RecordRow(row) => {
                    let row =
                        crate::typing::environment::instantiate_runtime_record_row(&row, &parent, &mut self.runtime_typing_context, &self.typing_registry)
                            .ok_or_else(|| RuntimeError::Internal(format!("generic call-entry row recipe could not be instantiated for {parameter:?}")))?;
                    crate::typing::overlay::RuntimeStaticArgument::RecordRow(row)
                }
                crate::typing::RuntimeCallArgumentRecipe::DynamicType => crate::typing::overlay::RuntimeStaticArgument::DynamicType,
            };
            materialized.push((parameter, argument));
        }
        Ok(self.runtime_type_environments.intern(crate::typing::RuntimeTypeEnvironment::new(materialized)))
    }

    fn resolve_conformance_method(
        &mut self,
        callable: &phalcom_semantic::identity::CallableId,
        selector: &phalcom_common::selector::Selector,
        receiver: Value,
    ) -> PhResult<ObjRef> {
        if let Some(&method) = self.detached_method_objects.get(callable) {
            return Ok(method);
        }
        let selector_sym = self.interner.intern(&selector.encode());
        if let Some(method) = receiver.lookup_method(self, selector_sym) {
            return Ok(method);
        }
        let class_id = callable.owner.try_declaration().and_then(|decl| self.resolve_declaration_class(decl).ok());
        let method = class_id.and_then(|cls| {
            let class_obj = if callable.side.is_class() { self.heap.class(cls).class } else { cls };
            crate::heap::lookup_method_in_hierarchy(&self.heap, class_obj, selector_sym)
        });
        method.ok_or_else(|| RuntimeError::Internal(format!("trait-selected callable {callable:?} has no executable method")).into())
    }

    pub(super) fn materialize_conformance_environment(
        &mut self,
        receiver: Value,
        recipe: &crate::typing::RuntimeCallEnvironmentRecipe,
    ) -> PhResult<crate::typing::RuntimeConformanceEnvironmentId> {
        if recipe.conformance_slots.is_empty() {
            return Ok(crate::typing::RuntimeConformanceEnvironmentId::EMPTY);
        }
        let caller_conformance_env_id = self
            .frames
            .last()
            .map(|frame| frame.conformance_environment)
            .unwrap_or(crate::typing::RuntimeConformanceEnvironmentId::EMPTY);

        let mut slots = Vec::with_capacity(recipe.conformance_slots.len());
        for slot_recipe in recipe.conformance_slots.iter() {
            match slot_recipe {
                crate::typing::RuntimeConformanceSlotRecipe::Selection {
                    callable,
                    selector,
                    nested_slots,
                } => {
                    let method = self.resolve_conformance_method(callable, selector, receiver)?;
                    let nested_environment = if nested_slots.is_empty() {
                        crate::typing::RuntimeConformanceEnvironmentId::EMPTY
                    } else {
                        let mut nested = vec![None; nested_slots.iter().map(|(slot, _)| *slot as usize + 1).max().unwrap_or(0)];
                        for (slot, nested_recipe) in nested_slots.iter() {
                            let nested_slot = match nested_recipe {
                                crate::typing::RuntimeConformanceSlotRecipe::Selection {
                                    callable,
                                    selector,
                                    nested_slots,
                                } => {
                                    let method = self.resolve_conformance_method(callable, selector, receiver)?;
                                    let conformance_environment = if nested_slots.is_empty() {
                                        crate::typing::RuntimeConformanceEnvironmentId::EMPTY
                                    } else {
                                        return Err(RuntimeError::Internal("nested selected conformance recipes cannot nest recursively".into()).into());
                                    };
                                    Some(crate::typing::RuntimeConformanceSlot {
                                        method,
                                        type_environment: crate::typing::RuntimeTypeEnvironmentId::EMPTY,
                                        conformance_environment,
                                    })
                                }
                                crate::typing::RuntimeConformanceSlotRecipe::Forwarded(caller_slot) => {
                                    self.runtime_conformance_environments.slot(caller_conformance_env_id, *caller_slot)
                                }
                                crate::typing::RuntimeConformanceSlotRecipe::DerivedConditional { .. } => None,
                            };
                            nested[*slot as usize] = nested_slot;
                        }
                        self.runtime_conformance_environments
                            .intern(crate::typing::RuntimeConformanceEnvironment::new(nested))
                    };
                    slots.push(Some(crate::typing::RuntimeConformanceSlot {
                        method,
                        type_environment: crate::typing::RuntimeTypeEnvironmentId::EMPTY,
                        conformance_environment: nested_environment,
                    }));
                }
                crate::typing::RuntimeConformanceSlotRecipe::Forwarded(caller_slot) => {
                    let slot_val = self.runtime_conformance_environments.slot(caller_conformance_env_id, *caller_slot);
                    slots.push(slot_val);
                }
                crate::typing::RuntimeConformanceSlotRecipe::DerivedConditional {
                    callable,
                    selector,
                    type_bindings,
                    nested_slots,
                } => {
                    let method = self.resolve_conformance_method(callable, selector, receiver)?;

                    let parent_id = self
                        .frames
                        .last()
                        .map(|frame| frame.type_environment)
                        .unwrap_or(crate::typing::RuntimeTypeEnvironmentId::EMPTY);
                    let parent = self
                        .runtime_type_environments
                        .get(parent_id)
                        .cloned()
                        .unwrap_or_else(|| crate::typing::RuntimeTypeEnvironment::new(Vec::new()));
                    let mut bindings = Vec::with_capacity(type_bindings.len());
                    for (parameter, type_recipe) in type_bindings.iter().cloned() {
                        let ty = crate::typing::instantiate_type_recipe(type_recipe, &parent, &mut self.runtime_typing_context, &self.typing_registry)
                            .ok_or_else(|| RuntimeError::Internal("derived conditional impl type recipe could not be instantiated".into()))?;
                        bindings.push((parameter, crate::typing::overlay::RuntimeStaticArgument::Term(ty)));
                    }
                    let type_env_id = self.runtime_type_environments.intern(crate::typing::RuntimeTypeEnvironment::new(bindings));

                    let nested_conformance_env_id = if nested_slots.is_empty() {
                        caller_conformance_env_id
                    } else {
                        let mut nested_conformance_slots = vec![None; nested_slots.iter().map(|(slot, _)| *slot as usize + 1).max().unwrap_or(0)];
                        for (slot, nested_slot_recipe) in nested_slots.iter() {
                            let nested_slot_val = match nested_slot_recipe {
                                crate::typing::RuntimeConformanceSlotRecipe::Selection {
                                    callable,
                                    selector,
                                    nested_slots,
                                } => {
                                    if !nested_slots.is_empty() {
                                        return Err(RuntimeError::Internal(
                                            "derived nested selected conformance recipe cannot require a further nested environment".into(),
                                        )
                                        .into());
                                    }
                                    let method = self.resolve_conformance_method(callable, selector, receiver)?;
                                    Some(crate::typing::RuntimeConformanceSlot {
                                        method,
                                        type_environment: crate::typing::RuntimeTypeEnvironmentId::EMPTY,
                                        conformance_environment: crate::typing::RuntimeConformanceEnvironmentId::EMPTY,
                                    })
                                }
                                crate::typing::RuntimeConformanceSlotRecipe::Forwarded(caller_slot) => {
                                    self.runtime_conformance_environments.slot(caller_conformance_env_id, *caller_slot)
                                }
                                crate::typing::RuntimeConformanceSlotRecipe::DerivedConditional { .. } => None,
                            };
                            nested_conformance_slots[*slot as usize] = nested_slot_val;
                        }
                        self.runtime_conformance_environments
                            .intern(crate::typing::RuntimeConformanceEnvironment::new(nested_conformance_slots))
                    };

                    slots.push(Some(crate::typing::RuntimeConformanceSlot {
                        method,
                        type_environment: type_env_id,
                        conformance_environment: nested_conformance_env_id,
                    }));
                }
            }
        }
        Ok(self
            .runtime_conformance_environments
            .intern(crate::typing::RuntimeConformanceEnvironment::new(slots)))
    }

    /// Executes a compiler-emitted call to an internal runtime helper.
    ///
    /// The capability covers method authorization only. It is removed before
    /// any callee frame executes, so generated helpers cannot confer ambient
    /// internal authority to ordinary user code.
    fn invoke_compiler_internal_at(&mut self, callable: &Callable, cache_ip: usize, arity: u8, selector_idx: u16) -> PhResult<()> {
        self.compiler_internal_dispatch_depth += 1;
        let result = self.invoke_at(callable, cache_ip, arity, selector_idx, None);
        self.compiler_internal_dispatch_depth -= 1;
        result
    }

    /// The inner dispatch loop, unaware of fibers: drains bytecode until
    /// [`Self::frames`] shrinks to `base_frames`, or a [`RuntimeError`]
    /// propagates out. [`Self::run_until`] wraps this with the fiber-floor
    /// capture (ADR-0030 §6); this function's own behavior is otherwise
    /// exactly the pre-U-FIBER `run_until`.
    ///
    /// # Errors
    ///
    /// Returns any [`RuntimeError`] raised during execution (undefined variable,
    /// method-not-found, unsupported operator, and so on).
    fn run_until_inner(&mut self, base_frames: usize) -> PhResult<Value> {
        // The hoisted callable of the frame we are executing (F14 S1a, U-HOTPATH
        // Change 1, resolution (a)). Refreshed by a one-compare guard below rather
        // than re-derived from `self.heap` every instruction.
        //
        // Holding the `Rc` — not a `&Chunk` — is what makes this work: a borrow out
        // of `self.heap` cannot live across the `&mut self` calls the arms make,
        // whereas an `Rc` in a local keeps the `Callable` alive independently of the
        // heap, so the arms keep full `&mut self`. Cut 004 made the `Callable`
        // shared precisely so this hoist would need no `unsafe`; this is that
        // payoff. Soundness rests on 004's invariant that a `Callable` is never
        // mutated after construction.
        let mut hoisted: Option<(ObjRef, Rc<Callable>)> = None;
        loop {
            if self.frames.len() <= base_frames {
                // A drained frame stack with nothing left to yield means the
                // top-level program (or a block activation) fell off its end:
                // surface that absence as `None`, never the private sentinel.
                let result = self.stack.pop().unwrap_or(Value::nil());
                return Ok(self.surface_absence(result));
            }

            // Safepoint (memory-management.md §4): the *only* place collection runs. Here
            // `stack`/`frames` are coherent — no opcode is mid-flight with a value popped
            // into a Rust local. Servicing before reading `frame` is deliberate: a
            // non-moving collector cannot invalidate the `CallFrame` we are about to copy,
            // but keeping the whole read-decode-execute sequence GC-free is what makes that
            // independent of the collector's future shape.
            self.service_gc_safepoint();

            let frame = *self.frames.last().unwrap();
            let closure_id = frame.closure;
            let ip = frame.ip;
            let stack_offset = frame.stack_offset;

            // One compare replaces a SlotMap lookup (bounds + generation + enum
            // match) plus an `Rc` deref, per instruction (F14 S1a). The `Rc::clone`
            // is per *frame change*, not per opcode — a refcount bump on call and
            // return only, which is why this does not re-pay cut 004's per-instruction
            // `Rc` hop.
            //
            // Guarding on `closure_id` alone is sound **because only the callable is
            // hoisted, never `ip`**. A chunk is a property of the closure, so any
            // path that reaches this point with the same `closure_id` — including a
            // fiber switch, which swaps `self.frames` wholesale — is entitled to the
            // same chunk. `ip` and `stack_offset` are still read from the live frame
            // every iteration, so a switch into the *same* closure at a *different*
            // `ip` reloads the correct `ip` and executes correctly.
            //
            // **Do not extend this guard to cover a hoisted `ip`.** That is the
            // stale-across-fiber-switch bug U-HOTPATH §4 names as the one this unit
            // could ship: two fibers in the same closure at different `ip`s compare
            // equal here. Hoisting `ip` needs a frame-identity guard, not this one.
            let callable = match &hoisted {
                Some((id, callable)) if *id == closure_id => callable,
                _ => {
                    let callable = Rc::clone(&self.heap.closure(closure_id).callable);
                    &hoisted.insert((closure_id, callable)).1
                }
            };

            // Only `code[ip]` is read here. The parallel `spans[ip]` is needed by
            // exactly two arms (`Invoke`, `SuperSend`, which pass it to
            // `call_method`/`forward_does_not_understand` for the error path) and
            // is re-read there, inside a borrow those arms already take — so the
            // send path pays nothing for it and every other opcode stops paying a
            // bounds-checked load it discards (perf-log S2).
            let opcode = callable.chunk.code[ip];

            // Per-opcode instrumentation is compiled out unless `vm-trace` is on. A runtime
            // filter is not enough: even with every subscriber at `LevelFilter::OFF`, leaving
            // these callsites in the loop costs a measured 18.2% of arith wall-clock, split
            // evenly between the span and the `debug!`s (perf-log 003). The cost is the code
            // the compiler must still emit per opcode, which no subscriber setting can remove.
            #[cfg(feature = "vm-trace")]
            let _opcode_span = {
                let entered = span!(Level::DEBUG, "vm_opcode", opcode = ?opcode).entered();
                debug!("Stack before: {:?}", self.stack);
                entered
            };

            // Counts only — never read a timing from a build with this on; the
            // increment is exactly the per-opcode cost `vm-trace` was gated for.
            // Counts are deterministic, so the protocol is two runs: counts here,
            // wall-clock from a default build (`benchmarks/vm/opcode-cost.py`).
            // `closure_id` and `ip` are what let the counter tell a *fusible* pair
            // (static successor in the same chunk) from a mere execution-order
            // neighbour across a call or back-edge — H13. `ip` is pre-increment.
            #[cfg(feature = "opcode-histogram")]
            crate::opcode_stats::record(&opcode, closure_id, ip);

            self.frames.last_mut().unwrap().ip += 1;

            match opcode {
                Bytecode::Constant(idx) => {
                    let constant = callable.chunk.constants[idx as usize];
                    #[cfg(feature = "vm-trace")]
                    debug!("Pushing constant: {:?}", constant);
                    self.stack.push(constant);
                }
                Bytecode::GetEllipsis => self.stack.push(self.require_semantic_roots()?.ellipsis),
                Bytecode::Closure(idx) => {
                    // The constant is the *template* closure the compiler emitted
                    // (empty upvalue list). Materialize a fresh instance whose
                    // upvalue cells are captured from the current activation per
                    // the callable's descriptors, then wrap it in a BlockObject
                    // stamped with the home frame token (ADR-0013, functions.md §2).
                    let template = callable.chunk.constants[idx as usize];
                    let Some(template_id) = template.as_obj() else {
                        return Err(RuntimeError::Internal("Closure constant is not a closure".to_string()).into());
                    };
                    let descriptors = self.heap.closure(template_id).callable.upvalues.clone();
                    let callable = Rc::clone(&self.heap.closure(template_id).callable);
                    let module = self.heap.closure(template_id).module;
                    // Block templates are compiled before their enclosing
                    // method is installed, so inherit the active closure's
                    // already-bound source class when the template itself
                    // has none.
                    let lexical_class = self.heap.closure(template_id).lexical_class.or(self.heap.closure(closure_id).lexical_class);
                    let super_anchor = self.heap.closure(template_id).super_anchor.or(self.heap.closure(closure_id).super_anchor);
                    let foreign_receiver_guard = self.frames.last().and_then(|frame| frame.foreign_receiver_guard);

                    let mut upvalues = Vec::with_capacity(descriptors.len());
                    for desc in &descriptors {
                        let cell = if desc.is_local {
                            self.capture_upvalue(stack_offset + desc.index)
                        } else {
                            self.heap.closure(closure_id).upvalues[desc.index]
                        };
                        upvalues.push(cell);
                    }

                    let new_closure = self.heap.alloc(Object::Closure(Box::new(ClosureObject {
                        callable,
                        module,
                        upvalues,
                        lexical_class,
                        super_anchor,
                        foreign_receiver_guard,
                    })));
                    let token = self.current_frame_token().expect("closure created inside a frame");
                    let type_env = self
                        .frames
                        .last()
                        .map(|f| f.type_environment)
                        .unwrap_or(crate::typing::environment::RuntimeTypeEnvironmentId::EMPTY);
                    let block = self.heap.alloc(Object::Block(BlockObject::with_type_environment(new_closure, token, type_env)));
                    self.stack.push(Value::obj(block));
                }
                // `Bytecode::Nil` pushes immediate `None` (the surface
                // absence value), never the raw private sentinel. It is emitted
                // both to seed uninitialized slots (e.g. a `var x` with no
                // initializer) and as the result of sacred-inlined one-armed
                // control flow (`ifTrue`, `ifFalse`, `whileTrue`), whose value
                // can flow straight into an arg or `print` without crossing a
                // read boundary — so it must already be `None` here (Invariant 4,
                // [ADR-0007]). The raw `Value::Nil` sentinel is never pushed by
                // any opcode; it only backs unread allocator storage and is
                // surfaced to immediate `None` at reads (see the `Get*` handlers and
                // `surface_absence`). See [`Bytecode::Nil`]'s rustdoc.
                Bytecode::Nil => self.stack.push(self.none_value()),
                Bytecode::UninitializedBinding => self.stack.push(Value::uninitialized_binding()),
                Bytecode::True => self.stack.push(TRUE),
                Bytecode::False => self.stack.push(FALSE),
                Bytecode::Pop => {
                    self.stack.pop();
                }
                Bytecode::DefineGlobal(idx) => {
                    let name_val = callable.chunk.constants[idx as usize];
                    if let Some(name_sym) = name_val.symbol_value() {
                        let module_id = self.heap.closure(closure_id).module;
                        let value = *self.stack.last().unwrap();
                        self.heap.module_mut(module_id).define(name_sym, value).unwrap();
                        self.stack.pop();
                    }
                }
                Bytecode::GetGlobal(idx) => {
                    let name_val = callable.chunk.constants[idx as usize];
                    if let Some(name_sym) = name_val.symbol_value() {
                        let module_id = self.heap.closure(closure_id).module;
                        let version = self.heap.module(module_id).globals_version;

                        // Fast path: this site already resolved the name, and the
                        // accessing module has declared nothing since (so nothing
                        // can have shadowed it). One integer compare replaces the
                        // SipHash probe — and the second, core-module probe that a
                        // kernel name would otherwise pay on every access.
                        let cached = callable.chunk.gcaches[ip].get();
                        if let Some(hit) = cached
                            && hit.version == version
                            && let Some(value) = self.heap.module(hit.module).get_by_slot(hit.slot)
                        {
                            if value.is_uninitialized_binding() {
                                let name = self.resolve_symbol(name_sym).to_string();
                                return Err(RuntimeError::UninitializedBinding { name }.into());
                            }
                            let value = self.surface_absence(value);
                            self.stack.push(value);
                            continue;
                        }

                        let (resolved_module, slot) = match self.heap.module(module_id).slot_of(name_sym) {
                            Some(slot) => (module_id, slot),
                            None => {
                                let name = self.resolve_symbol(name_sym).to_string();
                                return Err(RuntimeError::UndefinedVariable { name }.into());
                            }
                        };

                        callable.chunk.gcaches[ip].set(Some(crate::chunk::GlobalCache {
                            module: resolved_module,
                            slot,
                            version,
                        }));

                        let value = self.heap.module(resolved_module).get_by_slot(slot).expect("resolved slot");
                        if value.is_uninitialized_binding() {
                            let name = self.resolve_symbol(name_sym).to_string();
                            return Err(RuntimeError::UninitializedBinding { name }.into());
                        }
                        // A declared-but-unassigned global holds the private
                        // sentinel (module slots initialize to `Value::Nil`);
                        // surface it as `None` on read.
                        let value = self.surface_absence(value);
                        self.stack.push(value);
                    }
                }
                Bytecode::GetGlobalChecked(idx) => {
                    let name_val = callable.chunk.constants[idx as usize];
                    if let Some(name_sym) = name_val.symbol_value() {
                        let module_id = self.heap.closure(closure_id).module;
                        let version = self.heap.module(module_id).globals_version;

                        let cached = callable.chunk.gcaches[ip].get();
                        if let Some(hit) = cached
                            && hit.version == version
                            && let Some(value) = self.heap.module(hit.module).get_by_slot(hit.slot)
                        {
                            if value.is_uninitialized_binding() {
                                let name = self.resolve_symbol(name_sym).to_string();
                                return Err(RuntimeError::UninitializedBinding { name }.into());
                            }
                            let value = self.surface_absence(value);
                            self.stack.push(value);
                            continue;
                        }

                        let (resolved_module, slot) = match self.heap.module(module_id).slot_of(name_sym) {
                            Some(slot) => (module_id, slot),
                            None => {
                                let name = self.resolve_symbol(name_sym).to_string();
                                return Err(RuntimeError::UndefinedVariable { name }.into());
                            }
                        };

                        callable.chunk.gcaches[ip].set(Some(crate::chunk::GlobalCache {
                            module: resolved_module,
                            slot,
                            version,
                        }));

                        let value = self.heap.module(resolved_module).get_by_slot(slot).expect("resolved slot");
                        if value.is_uninitialized_binding() {
                            let name = self.resolve_symbol(name_sym).to_string();
                            return Err(RuntimeError::UninitializedBinding { name }.into());
                        }
                        let value = self.surface_absence(value);
                        self.stack.push(value);
                    }
                }
                Bytecode::SetGlobal(idx) => {
                    let name_val = callable.chunk.constants[idx as usize];
                    if let Some(name_sym) = name_val.symbol_value() {
                        let module_id = self.heap.closure(closure_id).module;
                        let version = self.heap.module(module_id).globals_version;

                        // Unlike `GetGlobal`, assignment has **no** core-module
                        // fallback: writing a kernel name the module never declared
                        // is an error, not a write into core. So a hit here always
                        // names this module's own slot.
                        let cached = callable.chunk.gcaches[ip].get();
                        let slot = match cached {
                            Some(hit) if hit.version == version => Some(hit.slot),
                            _ => {
                                let resolved = self.heap.module(module_id).slot_of(name_sym);
                                if let Some(slot) = resolved {
                                    callable.chunk.gcaches[ip].set(Some(crate::chunk::GlobalCache {
                                        module: module_id,
                                        slot,
                                        version,
                                    }));
                                }
                                resolved
                            }
                        };

                        if let Some(slot) = slot {
                            let value = *self.stack.last().unwrap();
                            self.heap.module_mut(module_id).set_global(slot, value).unwrap();
                        } else {
                            let name = self.resolve_symbol(name_sym).to_string();
                            return Err(RuntimeError::UndefinedVariable { name }.into());
                        }
                    }
                }
                Bytecode::GetLocal(slot) => {
                    let local_idx = stack_offset + slot as usize;
                    if local_idx < self.stack.len() {
                        self.stack.push(self.stack[local_idx]);
                    } else {
                        return Err(RuntimeError::Internal(format!("Local variable slot {slot} out of bounds")).into());
                    }
                }
                Bytecode::GetLocalChecked(slot, name_idx) => {
                    let local_idx = stack_offset + slot as usize;
                    if local_idx < self.stack.len() {
                        let value = self.stack[local_idx];
                        if value.is_uninitialized_binding() {
                            let name_val = callable.chunk.constants[name_idx as usize];
                            let name = name_val
                                .symbol_value()
                                .map(|s| self.resolve_symbol(s).to_string())
                                .unwrap_or_else(|| "variable".to_string());
                            return Err(RuntimeError::UninitializedBinding { name }.into());
                        }
                        self.stack.push(value);
                    } else {
                        return Err(RuntimeError::Internal(format!("Local variable slot {slot} out of bounds")).into());
                    }
                }
                Bytecode::SetLocal(slot) => {
                    let local_idx = stack_offset + slot as usize;
                    if local_idx < self.stack.len() {
                        let value = *self.stack.last().unwrap();
                        self.stack[local_idx] = value;
                    } else {
                        return Err(RuntimeError::Internal(format!("Local variable slot {slot} out of bounds")).into());
                    }
                }
                Bytecode::ReserveScratchLocal(slot) => {
                    let local_idx = stack_offset + slot as usize;
                    if local_idx > self.stack.len() {
                        return Err(RuntimeError::Internal(format!(
                            "scratch local slot {slot} insertion index {local_idx} exceeds stack length {}",
                            self.stack.len()
                        ))
                        .into());
                    }
                    let higher_keys: Vec<usize> = self.open_upvalues.range(local_idx..).map(|(&idx, _)| idx).collect();
                    for old_idx in higher_keys.into_iter().rev() {
                        let cell = self.open_upvalues.remove(&old_idx).unwrap();
                        if let Upvalue::Open { slot: s, .. } = self.heap.upvalue_mut(cell) {
                            *s += 1;
                        }
                        self.open_upvalues.insert(old_idx + 1, cell);
                    }
                    self.stack.insert(local_idx, Value::nil());
                }
                Bytecode::ReleaseScratchLocal(slot) => {
                    let local_idx = stack_offset + slot as usize;
                    if local_idx >= self.stack.len() {
                        return Err(RuntimeError::Internal(format!(
                            "scratch local slot {slot} removal index {local_idx} exceeds stack length {}",
                            self.stack.len()
                        ))
                        .into());
                    }
                    if let Some(cell) = self.open_upvalues.remove(&local_idx) {
                        let value = self.stack[local_idx];
                        *self.heap.upvalue_mut(cell) = Upvalue::Closed(value);
                    }
                    self.stack.remove(local_idx);
                    let higher_keys: Vec<usize> = self.open_upvalues.range((local_idx + 1)..).map(|(&idx, _)| idx).collect();
                    for old_idx in higher_keys {
                        let cell = self.open_upvalues.remove(&old_idx).unwrap();
                        if let Upvalue::Open { slot: s, .. } = self.heap.upvalue_mut(cell) {
                            *s -= 1;
                        }
                        self.open_upvalues.insert(old_idx - 1, cell);
                    }
                }
                Bytecode::Class(idx) => {
                    let name_val = callable.chunk.constants[idx as usize];
                    if let Some(name_sym) = name_val.symbol_value() {
                        let name = self.resolve_symbol(name_sym).to_string();
                        let superclass = self.stack.pop().unwrap();
                        if let Some(sc_id) = superclass.as_obj() {
                            if self.heap.as_class(sc_id).is_some() {
                                let closure_module = self.heap.closure(closure_id).module;
                                let new_class = self.create_class(closure_module, &name, Some(sc_id));
                                let key = crate::vm::ClassKey {
                                    module: closure_module,
                                    name: name_sym,
                                };
                                if self.field_layouts.get(&key).is_some_and(|layout| layout.is_abstract) {
                                    self.heap.class_mut(new_class).is_abstract = true;
                                }
                                self.stack.push(Value::obj(new_class));
                            } else {
                                return Err(RuntimeError::InvalidSuperClass(format!("{superclass}")).into());
                            }
                        } else {
                            return Err(RuntimeError::InvalidSuperClass(format!("{superclass}")).into());
                        }
                    }
                }
                Bytecode::GetLinked(index) => {
                    let module = self.heap.closure(closure_id).module;
                    let linked = self
                        .heap
                        .module(module)
                        .linked_reads
                        .get(index as usize)
                        .copied()
                        .ok_or_else(|| RuntimeError::Internal(format!("linked read index {index} is not materialized")))?;
                    let value = match linked {
                        crate::modules::RuntimeLinkedRead::Module(module) => Value::obj(module),
                        crate::modules::RuntimeLinkedRead::Binding(binding) => self
                            .heap
                            .module(binding.module)
                            .get_by_slot(binding.slot as usize)
                            .ok_or_else(|| RuntimeError::Internal(format!("linked binding slot {} is not materialized", binding.slot)))
                            .map(|value| self.surface_absence(value))?,
                        crate::modules::RuntimeLinkedRead::StaticConstant(reference) => self.resolve_static_constant(reference)?,
                    };
                    self.stack.push(value);
                }
                Bytecode::MakeFamily { spec, kind } => {
                    let spec_value = callable.chunk.constants[spec as usize];
                    let family_spec = match kind {
                        crate::bytecode::FamilySpecKind::Exact => {
                            let Some(symbol) = spec_value.symbol_value() else {
                                return Err(RuntimeError::Internal("exact MakeFamily constant is not a Symbol".into()).into());
                            };
                            crate::heap::FamilySpec::Exact(symbol)
                        }
                        crate::bytecode::FamilySpecKind::Pattern => {
                            let Some(pattern) = spec_value.as_obj() else {
                                return Err(RuntimeError::Internal("pattern MakeFamily constant is not an object".into()).into());
                            };
                            if !matches!(self.heap.get(pattern), Object::SelectorPattern(_)) {
                                return Err(RuntimeError::Internal("pattern MakeFamily constant is not a selector pattern".into()).into());
                            }
                            crate::heap::FamilySpec::Pattern(pattern)
                        }
                    };
                    let recv = self.stack.pop().unwrap();
                    let family = Object::Family(crate::heap::FamilyObject {
                        receiver: recv,
                        spec: family_spec,
                        conditional: Arc::from([]),
                    });
                    let family_id = self.heap.alloc(family);
                    self.stack.push(Value::obj(family_id));
                }
                Bytecode::MakeConditionalFamily { spec, kind, conditional } => {
                    let spec_value = callable.chunk.constants[spec as usize];
                    let family_spec = match kind {
                        crate::bytecode::FamilySpecKind::Exact => {
                            let Some(symbol) = spec_value.symbol_value() else {
                                return Err(RuntimeError::Internal("exact MakeConditionalFamily constant is not a Symbol".into()).into());
                            };
                            crate::heap::FamilySpec::Exact(symbol)
                        }
                        crate::bytecode::FamilySpecKind::Pattern => {
                            let Some(pattern) = spec_value.as_obj() else {
                                return Err(RuntimeError::Internal("pattern MakeConditionalFamily constant is not an object".into()).into());
                            };
                            if !matches!(self.heap.get(pattern), Object::SelectorPattern(_)) {
                                return Err(RuntimeError::Internal("pattern MakeConditionalFamily constant is not a selector pattern".into()).into());
                            }
                            crate::heap::FamilySpec::Pattern(pattern)
                        }
                    };
                    let recv = *self
                        .stack
                        .last()
                        .ok_or(RuntimeError::Internal("stack underflow in MakeConditionalFamily".into()))?;
                    let entries = callable
                        .chunk
                        .executable_semantics
                        .conditional_family_descriptor(conditional)
                        .entries
                        .iter()
                        .map(|entry| {
                            let declaring_class = self.resolve_declaration_class(&entry.declaring_owner)?;
                            let declaring_class = if entry.side == phalcom_semantic::identity::DispatchSide::Class {
                                self.heap.class(declaring_class).class
                            } else {
                                declaring_class
                            };
                            let fallback_method = callable.chunk.constants[entry.fallback_method as usize]
                                .as_obj()
                                .ok_or_else(|| RuntimeError::Internal("conditional family fallback is not an object".into()))?;
                            let (type_environment, conformance_environment) = if let Some(recipe) = &entry.call_environment_recipe {
                                self.validate_specialized_call(recipe, fallback_method)?;
                                (
                                    self.materialize_call_environment(recipe)?,
                                    self.materialize_conformance_environment(recv, recipe)?,
                                )
                            } else {
                                return Err(RuntimeError::Internal("conditional family entry is missing its selected call environment recipe".into()).into());
                            };
                            Ok(crate::heap::ConditionalFamilyDispatchEntry {
                                operation: entry.operation.clone(),
                                selector: entry.selector,
                                declaring_class,
                                fallback_method,
                                type_environment,
                                conformance_environment,
                            })
                        })
                        .collect::<PhResult<Vec<_>>>()?;
                    self.stack.pop();
                    let family = Object::Family(crate::heap::FamilyObject {
                        receiver: recv,
                        spec: family_spec,
                        conditional: Arc::from(entries.into_boxed_slice()),
                    });
                    let family_id = self.heap.alloc(family);
                    self.stack.push(Value::obj(family_id));
                }
                Bytecode::FinalizeClass => {
                    if let Some(class_id) = self.stack.last().unwrap().as_obj() {
                        self.finalize_class_base_names(class_id);
                        let meta_id = self.heap.class(class_id).class;
                        self.finalize_class_base_names(meta_id);
                        self.bind_metadata_for_class(class_id)?;
                    }
                }
                Bytecode::SuperSend(argc, selector_idx) => {
                    let selector_val = callable.chunk.constants[selector_idx as usize];
                    // Read here rather than in the loop head (S2) — this arm and
                    // `Invoke` are the only consumers.
                    let source_range = callable.chunk.span_at(ip);
                    let argc = argc as usize;
                    let selector_sym = selector_val.as_symbol().unwrap();

                    // The receiver stays the original `self` pushed by the
                    // compiler; only the lookup start moves above the closure's
                    // lexical anchor (method-lookup.md §1.14, U-INH §3.4).
                    let receiver_idx = self.stack.len() - 1 - argc;
                    let receiver = self.stack[receiver_idx];

                    let anchor = self
                        .heap
                        .closure(closure_id)
                        .super_anchor
                        .ok_or_else(|| RuntimeError::Internal("super send missing closure anchor".into()))?;
                    let parent = self.heap.class(anchor).superclass;

                    let method = parent.and_then(|p| crate::heap::lookup_method_in_hierarchy(&self.heap, p, selector_sym));

                    if let Some(method) = method {
                        let foreign_guard = self.frames.last().and_then(|frame| frame.foreign_receiver_guard);
                        let frames_before = self.frames.len();
                        self.call_method(&receiver, method, argc, source_range)?;
                        if self.frames.len() > frames_before {
                            self.frames.last_mut().unwrap().foreign_receiver_guard = foreign_guard;
                        }
                    } else {
                        let (name, slots, kind) = decode_selector(self.resolve_symbol(selector_sym));
                        let positional_count = slots.iter().filter(|slot| slot.is_none()).count();
                        let labels = slots
                            .iter()
                            .filter_map(|slot| slot.as_ref())
                            .map(|label| self.interner.intern(label))
                            .collect::<Vec<_>>();
                        let rest = matches!(kind, SignatureKind::Method(_))
                            .then(|| self.interner.intern(&name))
                            .and_then(|base| parent.and_then(|class| self.lookup_rest_method(class, base, positional_count, &labels)));
                        if let Some(method) = rest {
                            let foreign_guard = self.frames.last().and_then(|frame| frame.foreign_receiver_guard);
                            let frames_before = self.frames.len();
                            self.activate_rest_method(&receiver, method, receiver_idx, positional_count, &labels, selector_sym, source_range)?;
                            if self.frames.len() > frames_before {
                                self.frames.last_mut().unwrap().foreign_receiver_guard = foreign_guard;
                            }
                        } else {
                            self.forward_does_not_understand(receiver_idx, selector_sym, source_range)?;
                        }
                    }
                }
                Bytecode::Method(selector_idx, is_static) => {
                    let selector_val = callable.chunk.constants[selector_idx as usize];
                    let selector = selector_val.as_symbol().unwrap();
                    let method_val = self.stack.pop().unwrap();
                    let class_val = *self.stack.last().unwrap();
                    if let (Some(method_id), Some(class_id)) = (method_val.as_obj(), class_val.as_obj()) {
                        let target_class = if is_static { self.heap.class(class_id).class } else { class_id };
                        if is_static {
                            let meta = self.heap.class(class_id).class;
                            let closure = {
                                let method = self.heap.method_mut(method_id);
                                method.set_holder(meta);
                                method.access_owner = Some(class_id);
                                match method.kind {
                                    crate::method::MethodKind::Closure(closure) => Some(closure),
                                    crate::method::MethodKind::Primitive(_) => None,
                                }
                            };
                            if let Some(closure) = closure {
                                self.heap.closure_mut(closure).lexical_class = Some(class_id);
                                self.heap.closure_mut(closure).super_anchor = Some(meta);
                            }
                            self.install_method_binding(target_class, selector, method_id)?;
                        } else {
                            let closure = {
                                let method = self.heap.method_mut(method_id);
                                method.set_holder(class_id);
                                method.access_owner = Some(class_id);
                                match method.kind {
                                    crate::method::MethodKind::Closure(closure) => Some(closure),
                                    crate::method::MethodKind::Primitive(_) => None,
                                }
                            };
                            if let Some(closure) = closure {
                                self.heap.closure_mut(closure).lexical_class = Some(class_id);
                                self.heap.closure_mut(closure).super_anchor = Some(class_id);
                            }
                            self.install_method_binding(target_class, selector, method_id)?;
                            // Sacred-selector override-epoch tracking (ADR-0018):
                            // any (re)definition of a sacred selector directly on
                            // the kernel Bool/Block class dirties the pristine
                            // flag the inliner's GuardBool/GuardBlock read.
                            self.universe.note_method_installed(class_id, selector, &self.interner);
                        }
                    } else {
                        return Err(RuntimeError::Internal("Invalid types for method definition.".to_string()).into());
                    }
                }
                Bytecode::GetSelf => {
                    let receiver = self.stack[stack_offset];
                    self.stack.push(receiver);
                }
                Bytecode::GetField(slot) => {
                    let receiver = self.stack.pop().ok_or("Stack underflow for GetField receiver")?;
                    if let Some(guard) = self.frames.last().and_then(|frame| frame.foreign_receiver_guard) {
                        self.guard_foreign_layout_access(receiver, guard)?;
                    }
                    if let Some(id) = receiver.as_obj() {
                        if let Some(instance) = self.heap.as_instance(id) {
                            let val = instance.slots.get(slot as usize).copied().unwrap_or(Value::nil());
                            self.stack.push(self.surface_absence(val));
                        } else if let Some(class) = self.heap.as_class(id) {
                            let val = class.static_slots.get(slot as usize).copied().unwrap_or(Value::nil());
                            self.stack.push(self.surface_absence(val));
                        } else if let Object::AdtCase(_) = self.heap.get(id) {
                            let val = self.case_payload_at(receiver, slot as usize)?;
                            self.stack.push(self.surface_absence(val));
                        } else {
                            let mut val_str = receiver.to_string(self);
                            if val_str.chars().count() > 40 {
                                val_str = val_str.chars().take(40).collect();
                            }
                            return Err(RuntimeError::AccessFieldsNonInstance {
                                value: val_str,
                                found: receiver.type_name(),
                            }
                            .into());
                        }
                    } else {
                        let mut val_str = receiver.to_string(self);
                        if val_str.chars().count() > 40 {
                            val_str = val_str.chars().take(40).collect();
                        }
                        return Err(RuntimeError::AccessFieldsNonInstance {
                            value: val_str,
                            found: receiver.type_name(),
                        }
                        .into());
                    }
                }
                Bytecode::GetFieldChecked(slot, name_idx) => {
                    let receiver = self.stack.pop().ok_or("Stack underflow for GetFieldChecked receiver")?;
                    if let Some(guard) = self.frames.last().and_then(|frame| frame.foreign_receiver_guard) {
                        self.guard_foreign_layout_access(receiver, guard)?;
                    }
                    let Some(id) = receiver.as_obj() else {
                        let mut val_str = receiver.to_string(self);
                        if val_str.chars().count() > 40 {
                            val_str = val_str.chars().take(40).collect();
                        }
                        return Err(RuntimeError::AccessFieldsNonInstance {
                            value: val_str,
                            found: receiver.type_name(),
                        }
                        .into());
                    };
                    let value = if let Some(instance) = self.heap.as_instance(id) {
                        instance.slots.get(slot as usize).copied().unwrap_or(Value::nil())
                    } else if let Some(class) = self.heap.as_class(id) {
                        class.static_slots.get(slot as usize).copied().unwrap_or(Value::nil())
                    } else if matches!(self.heap.get(id), Object::AdtCase(_)) {
                        self.case_payload_at(receiver, slot as usize)?
                    } else {
                        let mut val_str = receiver.to_string(self);
                        if val_str.chars().count() > 40 {
                            val_str = val_str.chars().take(40).collect();
                        }
                        return Err(RuntimeError::AccessFieldsNonInstance {
                            value: val_str,
                            found: receiver.type_name(),
                        }
                        .into());
                    };
                    if value.is_uninitialized_binding() {
                        let name_val = callable.chunk.constants[name_idx as usize];
                        let name = name_val
                            .symbol_value()
                            .map(|symbol| self.resolve_symbol(symbol).to_string())
                            .unwrap_or_else(|| "field".to_string());
                        return Err(RuntimeError::UninitializedField { name }.into());
                    }
                    self.stack.push(self.surface_absence(value));
                }
                Bytecode::SetField(slot) => {
                    let value_to_assign = self.stack.pop().ok_or("Stack underflow on field assignment")?;
                    let receiver = self.stack.pop().ok_or("Stack underflow for SetField receiver")?;
                    if let Some(guard) = self.frames.last().and_then(|frame| frame.foreign_receiver_guard) {
                        self.guard_foreign_layout_access(receiver, guard)?;
                    }
                    if let Some(id) = receiver.as_obj() {
                        if let Object::AdtCase(_) = self.heap.get(id) {
                            return Err(RuntimeError::ImmutableCasePayload.into());
                        }
                        if self.heap.as_instance(id).is_some() {
                            let instance = self.heap.instance_mut(id);
                            if (slot as usize) < instance.slots.len() {
                                instance.slots[slot as usize] = value_to_assign;
                            } else {
                                return Err(RuntimeError::Internal(format!("Field slot {slot} out of bounds")).into());
                            }
                            self.stack.push(value_to_assign);
                        } else if self.heap.as_class(id).is_some() {
                            let class = self.heap.class_mut(id);
                            if (slot as usize) < class.static_slots.len() {
                                class.static_slots[slot as usize] = value_to_assign;
                            } else {
                                return Err(RuntimeError::Internal(format!("Static field slot {slot} out of bounds")).into());
                            }
                            self.stack.push(value_to_assign);
                        } else {
                            let mut val_str = receiver.to_string(self);
                            if val_str.chars().count() > 40 {
                                val_str = val_str.chars().take(40).collect();
                            }
                            return Err(RuntimeError::AccessFieldsNonInstance {
                                value: val_str,
                                found: receiver.type_name(),
                            }
                            .into());
                        }
                    } else {
                        let mut val_str = receiver.to_string(self);
                        if val_str.chars().count() > 40 {
                            val_str = val_str.chars().take(40).collect();
                        }
                        return Err(RuntimeError::AccessFieldsNonInstance {
                            value: val_str,
                            found: receiver.type_name(),
                        }
                        .into());
                    }
                }
                Bytecode::NewInstance => {
                    let class_val = self.stack.pop().ok_or("Stack underflow for NewInstance class")?;
                    if let Some(class_id) = class_val.as_obj() {
                        if self.adt_registry.is_enum_root(class_id) {
                            return Err(RuntimeError::DirectEnumRootInstantiation.into());
                        }
                        if self.heap.as_class(class_id).is_some() {
                            let instance_ref = self.allocate_class_instance(class_id);
                            self.stack.push(Value::obj(instance_ref));
                        } else if self.heap.as_instance(class_id).is_some() {
                            // Super-construct (U-INH §3.5): a parent initializer
                            // reached via `super.construct(…)` runs on the child
                            // instance already allocated by the subclass
                            // constructor's prologue — its `self` slot holds an
                            // instance, not a class. `NewInstance` is idempotent
                            // here: it reuses that instance (filling the inherited
                            // slots in place) rather than allocating a fresh one.
                            self.stack.push(class_val);
                        } else {
                            let mut val_str = class_val.to_string(self);
                            if val_str.chars().count() > 40 {
                                val_str = val_str.chars().take(40).collect();
                            }
                            return Err(RuntimeError::InstantiateNonClass {
                                value: val_str,
                                found: class_val.type_name(),
                            }
                            .into());
                        }
                    } else {
                        let mut val_str = class_val.to_string(self);
                        if val_str.chars().count() > 40 {
                            val_str = val_str.chars().take(40).collect();
                        }
                        return Err(RuntimeError::InstantiateNonClass {
                            value: val_str,
                            found: class_val.type_name(),
                        }
                        .into());
                    }
                }
                Bytecode::Dup => {
                    let val = *self.stack.last().ok_or("Stack underflow on Dup")?;
                    self.stack.push(val);
                }
                Bytecode::WrapSome => {
                    let value = self.stack.pop().ok_or("Stack underflow for WrapSome")?;
                    let wrapped = crate::primitive::option::wrap_some(self, value)?;
                    self.stack.push(wrapped);
                }
                Bytecode::Invoke(arity, selector_idx) => {
                    self.invoke_at(callable, ip, arity, selector_idx, None)?;
                }
                Bytecode::InvokeSpecialized(arity, selector_idx, recipe_idx) => {
                    let recipe = callable.chunk.executable_semantics.call_environment_recipe(recipe_idx).clone();
                    self.invoke_at(callable, ip, arity, selector_idx, Some(&recipe))?;
                }
                Bytecode::InvokeTraitSelected { selection, arity } => {
                    self.invoke_trait_selected(callable, selection, arity)?;
                }
                Bytecode::InvokeTraitRequirement { slot, arity } => {
                    self.invoke_trait_requirement(callable, slot, arity)?;
                }
                Bytecode::InvokeConditional {
                    arity,
                    selector,
                    declaring_module,
                    declaring_name,
                    class_side,
                    fallback_method,
                    call_environment_recipe,
                } => {
                    self.invoke_conditional_at(
                        callable,
                        ip,
                        ConditionalInvocation {
                            arity,
                            selector,
                            declaring_module,
                            declaring_name,
                            class_side,
                            fallback_method,
                            call_environment_recipe,
                        },
                    )?;
                }
                Bytecode::BilateralPreferReflected(selector_idx) => {
                    let rhs = *self.stack.last().ok_or(RuntimeError::Internal("missing bilateral rhs".into()))?;
                    let lhs = *self
                        .stack
                        .get(self.stack.len().checked_sub(2).ok_or(RuntimeError::Internal("missing bilateral lhs".into()))?)
                        .ok_or(RuntimeError::Internal("missing bilateral lhs".into()))?;
                    let reflected = callable.chunk.constants[selector_idx as usize].as_symbol().map_err(RuntimeError::Internal)?;
                    let lhs_class = lhs.class(self);
                    let rhs_class = rhs.class(self);
                    let prefer = is_strict_subclass(&self.heap, rhs_class, lhs_class)
                        && lookup_method_with_definer(&self.heap, rhs_class, reflected)
                            .is_some_and(|(_, defining)| is_strict_subclass(&self.heap, defining, lhs_class));
                    self.stack.push(Value::bool(prefer));
                }
                Bytecode::TryInvokeExact {
                    arity,
                    selector,
                    missing_offset,
                } => {
                    let arity = arity as usize;
                    let receiver_idx = self
                        .stack
                        .len()
                        .checked_sub(arity + 1)
                        .ok_or(RuntimeError::Internal("bilateral stack underflow".into()))?;
                    let receiver = self.stack[receiver_idx];
                    let selector = callable.chunk.constants[selector as usize].as_symbol().map_err(RuntimeError::Internal)?;
                    if let Some(method) = receiver.lookup_method(self, selector) {
                        self.call_method(&receiver, method, arity, callable.chunk.span_at(ip))?;
                    } else {
                        self.stack.truncate(receiver_idx);
                        self.apply_jump_offset(missing_offset);
                    }
                }
                Bytecode::ValidateOrdering { reverse } => {
                    let value = self.pop()?;
                    let class = value.class(self);
                    let ordering = self.require_semantic_roots()?.ordering_class;
                    if class != ordering && !is_strict_subclass(&self.heap, class, ordering) {
                        return Err(RuntimeError::InvalidCompareReturn {
                            found: self.heap.class(class).name.clone(),
                        }
                        .into());
                    }
                    let value = if reverse {
                        let reverse_selector = self.interner.intern("reverse");
                        self.send_dynamic(value, reverse_selector, &[])?
                    } else {
                        value
                    };
                    let reversed_class = value.class(self);
                    if reversed_class != ordering && !is_strict_subclass(&self.heap, reversed_class, ordering) {
                        return Err(RuntimeError::InvalidCompareReturn {
                            found: self.heap.class(reversed_class).name.clone(),
                        }
                        .into());
                    }
                    self.stack.push(value);
                }
                Bytecode::Same => {
                    let rhs = self.pop()?;
                    let lhs = self.pop()?;
                    self.stack.push(Value::bool(self.semantic_same(lhs, rhs)));
                }
                Bytecode::RaiseUnsupported { operator, direct, reflected } => {
                    let rhs = self.pop()?;
                    let lhs = self.pop()?;
                    let symbol_text = |index: u16| -> Result<String, RuntimeError> {
                        callable.chunk.constants[index as usize]
                            .as_symbol()
                            .map(|symbol| self.resolve_symbol(symbol).to_owned())
                            .map_err(RuntimeError::Internal)
                    };
                    return Err(RuntimeError::UnsupportedOperation(Box::new(crate::error::UnsupportedOperationContext {
                        operator: symbol_text(operator)?,
                        lhs: self.heap.class(lhs.class(self)).name.clone(),
                        rhs: self.heap.class(rhs.class(self)).name.clone(),
                        direct: symbol_text(direct)?,
                        reflected: symbol_text(reflected)?,
                    }))
                    .into());
                }
                Bytecode::RaiseNonIterableStarOperand => {
                    let source = self.pop()?;
                    return Err(RuntimeError::NonIterableStarOperand { found: source.type_name() }.into());
                }
                Bytecode::Enum(spec_idx) => {
                    let spec = callable.chunk.executable_semantics.enum_spec(spec_idx).clone();
                    let root_class_id = self.register_enum_from_spec(&spec)?;
                    self.stack.push(Value::obj(root_class_id));
                }
                Bytecode::VariantMethod { variant, selector } => {
                    let variant_id = callable.chunk.executable_semantics.variant_target(variant).clone();
                    let selector_val = callable.chunk.constants[selector as usize];
                    let selector_sym = selector_val.as_symbol().unwrap();
                    let method_val = self.pop()?;
                    let method_ref = method_val.as_obj().ok_or("method value is not an object")?;
                    let runtime_var_id = self.adt_registry.variant_by_semantic(&variant_id).ok_or("unregistered variant")?;
                    let case_class_id = self.adt_registry.variant_descriptor(runtime_var_id).unwrap().behavior_class;
                    self.heap.class_mut(case_class_id).methods.insert(selector_sym, method_ref);
                }
                Bytecode::FinalizeEnum(spec_idx) => {
                    let spec = callable.chunk.executable_semantics.enum_spec(spec_idx);
                    if let Some(enum_id) = self.adt_registry.enum_by_declaration(&spec.owner) {
                        if let Some(desc) = self.adt_registry.enum_descriptor(enum_id).cloned() {
                            self.finalize_class_base_names(desc.root_class);
                            let meta_id = self.heap.class(desc.root_class).class;
                            self.finalize_class_base_names(meta_id);
                            for &v_id in desc.variants.iter() {
                                if let Some(vdesc) = self.adt_registry.variant_descriptor(v_id) {
                                    self.finalize_class_base_names(vdesc.behavior_class);
                                }
                            }
                        }
                    }
                }
                Bytecode::LoadVariantSingleton(variant_idx) => {
                    let variant_id = callable.chunk.executable_semantics.variant_target(variant_idx);
                    let runtime_var_id = match self.adt_registry.variant_by_semantic(variant_id) {
                        Some(runtime_var_id) => runtime_var_id,
                        None if phalcom_semantic::core_surface::CoreDeclarationIds::default().is_option(&variant_id.owner)
                            && matches!(
                                &variant_id.selector.base,
                                phalcom_common::selector::SelectorBase::Named(name) if name == "None"
                            ) =>
                        {
                            // Native Option represents None as an immediate
                            // absence value. Preserve that representation even
                            // when a canonical source closure is loaded before
                            // the optional runtime registry row is materialized.
                            self.stack.push(Value::none());
                            continue;
                        }
                        None => return Err(crate::error::RuntimeError::Internal("unregistered variant".into()).into()),
                    };
                    let vdesc = self.adt_registry.variant_descriptor(runtime_var_id).unwrap();
                    let val = vdesc.singleton.unwrap_or_else(|| Value::adt_singleton(runtime_var_id));
                    self.stack.push(val);
                }
                Bytecode::ConstructVariant { variant, arity } => {
                    let variant_id = callable.chunk.executable_semantics.variant_target(variant);
                    let runtime_var_id = self
                        .adt_registry
                        .variant_by_semantic(variant_id)
                        .ok_or_else(|| crate::error::RuntimeError::Internal("unregistered variant".into()))?;
                    let argc = arity as usize;
                    let payload: Vec<Value> = self.stack.drain(self.stack.len() - argc..).collect();
                    let val = self.construct_variant_value(runtime_var_id, payload)?;
                    self.stack.push(val);
                }
                Bytecode::Data(spec_idx) => {
                    let spec = callable.chunk.executable_semantics.data_spec(spec_idx).clone();
                    let class_id = self.register_data_from_spec(&spec)?;
                    self.stack.push(Value::obj(class_id));
                }
                Bytecode::FinalizeData(spec_idx) => {
                    let spec = callable.chunk.executable_semantics.data_spec(spec_idx);
                    if let Some(behavior_class) = self.data_registry.behavior_class_for_declaration(&spec.owner) {
                        self.finalize_class_base_names(behavior_class);
                        let meta_id = self.heap.class(behavior_class).class;
                        self.finalize_class_base_names(meta_id);
                    }
                }
                Bytecode::LoadDataSingleton(constructor_idx) => {
                    let cache = callable.chunk.executable_semantics.data_construction_cache(constructor_idx);
                    let spec = callable.chunk.executable_semantics.data_construction(constructor_idx);
                    let desc_id = if matches!(spec.type_recipe, crate::typing::RuntimeTypeRecipe::Template(_)) {
                        // A generic nullary constructor can select a different
                        // exact descriptor in each invocation. Keep the
                        // per-site cache only for closed recipes; descriptor
                        // interning handles repeated exact template instances.
                        self.get_or_bind_data_descriptor(spec)?
                    } else {
                        match cache.get() {
                            Some(id) => id,
                            None => {
                                let id = self.get_or_bind_data_descriptor(spec)?;
                                cache.set(Some(id));
                                id
                            }
                        }
                    };
                    self.stack.push(Value::data_singleton(desc_id));
                }
                Bytecode::ConstructData { constructor, arity } => {
                    let cache = callable.chunk.executable_semantics.data_construction_cache(constructor);
                    let spec = callable.chunk.executable_semantics.data_construction(constructor);
                    let desc_id = if matches!(spec.type_recipe, crate::typing::RuntimeTypeRecipe::Template(_)) {
                        // A generic method may construct the same data
                        // declaration at several applied type identities. Do
                        // not cache one specialization across invocations.
                        self.get_or_bind_data_descriptor(spec)?
                    } else {
                        match cache.get() {
                            Some(id) => id,
                            None => {
                                let id = self.get_or_bind_data_descriptor(spec)?;
                                cache.set(Some(id));
                                id
                            }
                        }
                    };
                    let argc = arity as usize;
                    let components: Vec<Value> = self.stack.drain(self.stack.len() - argc..).collect();
                    let val = self.construct_data_value_with_mapping(desc_id, &spec.argument_to_component, components)?;
                    self.stack.push(val);
                }
                Bytecode::GetDataComponent(logical_index) => {
                    let receiver = self.pop()?;
                    let component = self.get_data_component(receiver, logical_index as u32)?;
                    self.stack.push(component);
                }
                Bytecode::GetDataProductSlot(logical_index) => {
                    let receiver = self.pop()?;
                    let component = self.get_data_product_slot(receiver, logical_index as u32)?;
                    self.stack.push(component);
                }
                Bytecode::MakeResolvedBoundMethod(target_idx) => {
                    let target = callable.chunk.executable_semantics.associated_target(target_idx).clone();
                    let resolved = self.bind_behavioral_associated_target(&target)?;
                    let bound_ref = self.heap.alloc(Object::BoundMethod(crate::heap::BoundMethodObject {
                        method: resolved.method,
                        receiver: resolved.receiver,
                        conformance_environment: crate::typing::RuntimeConformanceEnvironmentId::EMPTY,
                    }));
                    self.stack.push(Value::obj(bound_ref));
                }
                Bytecode::MakeTraitBoundMethod(target_idx) => {
                    let invocation = callable.chunk.executable_semantics.trait_invocation(target_idx).clone();
                    let receiver = self.pop()?;
                    let (conformance_environment, method, _) = self.materialize_trait_environment(receiver, &invocation)?;
                    let bound_ref = self.heap.alloc(Object::BoundMethod(crate::heap::BoundMethodObject {
                        method,
                        receiver,
                        conformance_environment,
                    }));
                    self.stack.push(Value::obj(bound_ref));
                }
                Bytecode::InvokeResolvedAssociated { target, arity } => {
                    let source_range = callable.chunk.span_at(ip);
                    let argc = arity as usize;
                    let target = callable.chunk.executable_semantics.associated_target(target).clone();
                    let resolved = self.bind_behavioral_associated_target(&target)?;
                    let receiver_idx = self.stack.len() - argc;
                    self.stack.insert(receiver_idx, resolved.receiver);
                    let foreign_guard = self.frames.last().and_then(|frame| frame.foreign_receiver_guard);
                    let frames_before = self.frames.len();
                    self.call_method(&resolved.receiver, resolved.method, argc, source_range)?;
                    if self.frames.len() > frames_before {
                        self.frames.last_mut().unwrap().foreign_receiver_guard = foreign_guard;
                    }
                }
                Bytecode::MakeAssociatedFamily(desc_idx) => {
                    let descriptor = callable.chunk.executable_semantics.family_descriptor(desc_idx).clone();
                    let family_obj = crate::heap::AssociatedFamilyObject { descriptor, bound_owner: None };
                    let family_ref = self.heap.alloc(Object::AssociatedFamily(Box::new(family_obj)));
                    self.stack.push(Value::obj(family_ref));
                }
                Bytecode::InvokeAssociatedFamilyStatic { operation, arity } => {
                    let source_range = callable.chunk.span_at(ip);
                    let argc = arity as usize;
                    let callee_idx = self.stack.len() - argc - 1;
                    let callee = self.stack[callee_idx];
                    let entry_target = if let Some(obj_ref) = callee.as_obj() {
                        if let Object::AssociatedFamily(family) = self.heap.get(obj_ref) {
                            let op_shape = callable.chunk.executable_semantics.family_operation(operation);
                            let entry = family
                                .descriptor
                                .entries
                                .iter()
                                .find(|e| &e.operation == op_shape)
                                .ok_or("no matching family operation")?;
                            entry.target.clone()
                        } else {
                            return Err(RuntimeError::NotAnAssociatedFamily.into());
                        }
                    } else {
                        return Err(RuntimeError::NotAnAssociatedFamily.into());
                    };
                    match &entry_target {
                        crate::modules::semantic_lowering::ExecutableFamilyTarget::Singleton { variant } => {
                            let runtime_var_id = self.adt_registry.variant_by_semantic(variant).ok_or("unregistered variant")?;
                            self.stack.pop();
                            let value = self.construct_variant_value(runtime_var_id, Vec::new())?;
                            self.stack.push(value);
                        }
                        crate::modules::semantic_lowering::ExecutableFamilyTarget::VariantConstructor { variant } => {
                            let runtime_var_id = self.adt_registry.variant_by_semantic(variant).ok_or("unregistered variant")?;
                            let payload: Vec<Value> = self.stack.drain(callee_idx + 1..).collect();
                            self.stack.pop();
                            let value = self.construct_variant_value(runtime_var_id, payload)?;
                            self.stack.push(value);
                        }
                        crate::modules::semantic_lowering::ExecutableFamilyTarget::Behavioral { target } => {
                            let resolved = self.bind_behavioral_associated_target(target)?;
                            self.stack[callee_idx] = resolved.receiver;
                            let foreign_guard = self.frames.last().and_then(|frame| frame.foreign_receiver_guard);
                            let frames_before = self.frames.len();
                            self.call_method(&resolved.receiver, resolved.method, argc, source_range)?;
                            if self.frames.len() > frames_before {
                                self.frames.last_mut().unwrap().foreign_receiver_guard = foreign_guard;
                            }
                        }
                        crate::modules::semantic_lowering::ExecutableFamilyTarget::DataConstructor { constructor: _, construction } => {
                            let args: Vec<Value> = self.stack.drain(callee_idx + 1..).collect();
                            self.stack.pop();
                            let val = self.construct_data_from_spec(construction, args)?;
                            self.stack.push(val);
                        }
                    }
                }
                Bytecode::InvokeAssociatedFamilyPack { candidates } => {
                    let candidates = callable.chunk.executable_semantics.family_candidate_set(candidates).clone();
                    let builder = self
                        .pop()?
                        .as_obj()
                        .ok_or_else(|| RuntimeError::Internal("family pack is not an object".into()))?;
                    if self.heap.pack_builder(builder).has_pending() {
                        return Err(RuntimeError::Internal("unfinished family pack label".into()).into());
                    }
                    let (positionals, labels, values) = self.heap.pack_builder_mut(builder).take_parts();
                    let mut slots = vec![phalcom_common::selector::SelectorSlot::Positional; positionals.len()];
                    slots.extend(
                        labels
                            .iter()
                            .map(|label| phalcom_common::selector::SelectorSlot::Label(self.resolve_symbol(*label).to_owned())),
                    );
                    let operation = phalcom_semantic::types::family::FamilyOperationShape::method(slots.into_boxed_slice());
                    if !candidates.candidates.iter().any(|candidate| candidate.operation == operation) {
                        return Err(RuntimeError::AssociatedFamilyNoMatchingCandidate.into());
                    }
                    let receiver_idx = self.stack.len().checked_sub(1).ok_or(RuntimeError::NotAnAssociatedFamily)?;
                    let arity = positionals.len() + values.len();
                    let base = self.get_or_intern("call");
                    let selector = self.dynamic_pack_selector(base, PackSendKind::Method, positionals.len(), &labels)?;
                    self.stack.extend(positionals);
                    self.stack.extend(values);
                    self.invoke_dynamic_selector(receiver_idx, selector, arity, callable.chunk.span_at(ip))?;
                }
                Bytecode::IsVariant(variant_idx) => {
                    let variant_id = callable.chunk.executable_semantics.variant_target(variant_idx);
                    let runtime_var_id = self.adt_registry.variant_by_semantic(variant_id);
                    let val = self.pop()?;
                    let result = runtime_var_id.map(|expected| self.value_is_variant(val, expected)).unwrap_or(false);
                    self.stack.push(Value::bool(result));
                }
                Bytecode::GetVariantPayload(slot) => {
                    let val = self.pop()?;
                    let payload_val = self.case_payload_at(val, slot as usize)?;
                    self.stack.push(payload_val);
                }
                Bytecode::MatchInvariantFailure => {
                    return Err(RuntimeError::Internal("statically exhaustive match reached runtime fallthrough".into()).into());
                }
                Bytecode::InvokeCompilerInternal(arity, selector_idx) => {
                    self.invoke_compiler_internal_at(callable, ip, arity, selector_idx)?;
                }
                // The fused pairs (cut 008). Each does its first half's work inline,
                // then steps `ip` past the dead `Invoke` the fusion left at `ip + 1`
                // and runs the identical send through `invoke_at`, keyed on that
                // dead instruction's cache/span slots — so the only difference from
                // the unfused pair is one dispatch not taken.
                //
                // The extra `ip` bump must happen BEFORE the send: `call_method`
                // pushes a frame, after which `frames.last_mut()` is the callee's,
                // and this frame's `ip` is the return address.
                Bytecode::InvokeLocal(slot, arity, selector_idx) => {
                    let local_idx = stack_offset + slot as usize;
                    if local_idx >= self.stack.len() {
                        return Err(RuntimeError::Internal(format!("Local variable slot {slot} out of bounds")).into());
                    }
                    let value = self.stack[local_idx];
                    self.stack.push(value);
                    self.frames.last_mut().unwrap().ip += 1;
                    self.invoke_at(callable, ip + 1, arity, selector_idx, None)?;
                }
                Bytecode::InvokeConst(idx, arity, selector_idx) => {
                    let constant = callable.chunk.constants[idx as usize];
                    self.stack.push(constant);
                    self.frames.last_mut().unwrap().ip += 1;
                    self.invoke_at(callable, ip + 1, arity, selector_idx, None)?;
                }
                Bytecode::GetUpvalue(idx) => {
                    let cell = self.heap.closure(closure_id).upvalues[idx as usize];
                    let value = match *self.heap.upvalue(cell) {
                        // Resolve against the owning fiber's stack: the live VM
                        // stack when that fiber is current, else its parked
                        // stack in the `FiberObject` (ADR-0030).
                        Upvalue::Open { fiber, slot } => {
                            if fiber == self.current {
                                self.stack[slot]
                            } else {
                                self.heap.fiber(fiber).stack[slot]
                            }
                        }
                        Upvalue::Closed(value) => value,
                    };
                    if value.is_uninitialized_binding() {
                        return Err(RuntimeError::UninitializedBinding {
                            name: "captured variable".into(),
                        }
                        .into());
                    }
                    let value = self.surface_absence(value);
                    self.stack.push(value);
                }
                Bytecode::GetUpvalueChecked(idx, name_idx) => {
                    let cell = self.heap.closure(closure_id).upvalues[idx as usize];
                    let value = match *self.heap.upvalue(cell) {
                        Upvalue::Open { fiber, slot } => {
                            if fiber == self.current {
                                self.stack[slot]
                            } else {
                                self.heap.fiber(fiber).stack[slot]
                            }
                        }
                        Upvalue::Closed(value) => value,
                    };
                    if value.is_uninitialized_binding() {
                        let name_val = callable.chunk.constants[name_idx as usize];
                        let name = name_val
                            .symbol_value()
                            .map(|s| self.resolve_symbol(s).to_string())
                            .unwrap_or_else(|| "captured variable".to_string());
                        return Err(RuntimeError::UninitializedBinding { name }.into());
                    }
                    let value = self.surface_absence(value);
                    self.stack.push(value);
                }
                Bytecode::SetUpvalue(idx) => {
                    // Assignment is an expression: leave the value on the stack.
                    let value = *self.stack.last().unwrap();
                    let cell = self.heap.closure(closure_id).upvalues[idx as usize];
                    match *self.heap.upvalue(cell) {
                        // Write through to the owning fiber's stack — live when
                        // that fiber is current, else its parked stack (ADR-0030).
                        Upvalue::Open { fiber, slot } => {
                            if fiber == self.current {
                                self.stack[slot] = value;
                            } else {
                                self.heap.fiber_mut(fiber).stack[slot] = value;
                            }
                        }
                        Upvalue::Closed(_) => *self.heap.upvalue_mut(cell) = Upvalue::Closed(value),
                    }
                }
                Bytecode::CloseUpvalue(slot) => {
                    // Promote any open upvalue at this frame slot (or above) to a
                    // heap-owned `Closed` copy before the slot is reclaimed.
                    self.close_upvalues_from(stack_offset + slot as usize);
                }
                Bytecode::Return => {
                    // A bare `return;` (no operand) or a method that produced no
                    // value yields `None`, not the private sentinel
                    // (values-and-absence.md; bare-`return`→`None` pre-authorized).
                    let return_value = self.stack.pop().unwrap_or(Value::nil());
                    let return_value = self.surface_absence(return_value);
                    let popped = self.frames.pop().unwrap();
                    // Close any upvalues that still alias this frame's window so
                    // escaping closures survive the frame's disappearance
                    // (ADR-0013). Must run before the stack is truncated.
                    self.close_upvalues_from(popped.stack_offset);
                    self.stack.truncate(popped.stack_offset);

                    if self.control_stack.top().is_some_and(|c| c.callback_floor == self.frames.len()) {
                        match self.step_control_transfer(Transfer::Returned(return_value))? {
                            ControlStepOutcome::Continued => {
                                hoisted = None;
                                continue;
                            }
                            ControlStepOutcome::Completed(val) => {
                                if self.frames.len() <= base_frames {
                                    return Ok(val);
                                }
                                hoisted = None;
                                continue;
                            }
                            ControlStepOutcome::Propagate(Transfer::Returned(val)) => {
                                if self.frames.len() <= base_frames {
                                    return Ok(val);
                                }
                                self.stack.push(val);
                                hoisted = None;
                                continue;
                            }
                            ControlStepOutcome::Propagate(Transfer::Raise(e)) => {
                                return Err(e);
                            }
                            ControlStepOutcome::Propagate(Transfer::NonLocalReturn { target, value }) => {
                                self.execute_non_local_return(target, value)?;
                                hoisted = None;
                                continue;
                            }
                        }
                    }

                    if self.frames.len() <= base_frames {
                        return Ok(return_value);
                    }
                    self.stack.push(return_value);
                }
                Bytecode::ReturnNonLocal => {
                    // Non-local return: unwind to the block's home method
                    // activation, not just this block frame (ADR-0013,
                    // blocks.md §5). The whole unwind happens eagerly, here, in
                    // one shot — control cannot be handed back to the outer
                    // `run_until`/`block_call` Rust frames that sit between here
                    // and the home frame any other way than by mutating the
                    // shared `self.frames`/`self.stack` up front
                    // (U10-implementation-spec.md §2 point 2).
                    let token = self
                        .frames
                        .last()
                        .and_then(|frame| frame.home_frame_token)
                        .ok_or_else(|| RuntimeError::Internal("ReturnNonLocal in a frame with no home-frame token".to_string()))?;

                    // Locate the still-live home frame by (index, generation).
                    // A stale index or a generation mismatch means the home
                    // method already returned — raise `DeadFrameError` *before*
                    // touching any VM state, so a caught error leaves the stack
                    // consistent.
                    let is_live = self.frames.get(token.frame_index).is_some_and(|home| home.generation == token.generation);
                    if !is_live {
                        return Err(RuntimeError::DeadFrameError.into());
                    }

                    // The return value is already on the stack top (the compiler
                    // emits the expression, or `Bytecode::Nil` for a bare
                    // `return`); surface bare/absent to `None`, never the
                    // private sentinel — matching `Bytecode::Return`.
                    let return_value = self.stack.pop().unwrap_or(Value::nil());
                    let return_value = self.surface_absence(return_value);

                    if !self.control_stack.is_empty() {
                        match self.step_control_transfer(Transfer::NonLocalReturn {
                            target: token,
                            value: return_value,
                        })? {
                            ControlStepOutcome::Continued => {
                                hoisted = None;
                                continue;
                            }
                            ControlStepOutcome::Completed(v) => {
                                if self.frames.len() <= base_frames {
                                    return Ok(v);
                                }
                                hoisted = None;
                                continue;
                            }
                            ControlStepOutcome::Propagate(Transfer::NonLocalReturn { target, value }) => {
                                self.execute_non_local_return(target, value)?;
                                hoisted = None;
                                continue;
                            }
                            ControlStepOutcome::Propagate(Transfer::Returned(v)) => {
                                if self.frames.len() <= base_frames {
                                    return Ok(v);
                                }
                                self.stack.push(v);
                                hoisted = None;
                                continue;
                            }
                            ControlStepOutcome::Propagate(Transfer::Raise(e)) => {
                                return Err(e);
                            }
                        }
                    } else {
                        self.execute_non_local_return(token, return_value)?;
                        hoisted = None;
                        continue;
                    }
                }
                Bytecode::Jump(offset) => self.apply_jump_offset(offset),
                Bytecode::JumpIfFalse(offset) => {
                    let cond = self.pop()?;
                    match cond.as_bool() {
                        Some(true) => {}
                        Some(false) => self.apply_jump_offset(offset),
                        None => {
                            let found = cond.type_name();
                            return Err(RuntimeError::Type { expected: "Bool", found }.into());
                        }
                    }
                }
                // Pops the top of stack and, if it is immediate `None`, branches to `offset`.
                // Otherwise, falls through. Realizes the cursor-protocol
                // end-of-iteration test (ADR-0048 §2, iteration.md §2).
                Bytecode::JumpIfNone(offset) => {
                    let cursor = self.pop()?;
                    if cursor.is_none() {
                        self.apply_jump_offset(offset);
                    }
                }
                Bytecode::JumpIfUnsupported(offset) => {
                    let value = self.pop()?;
                    let unsupported = self.require_semantic_roots()?.unsupported;
                    if value.same_as(&unsupported) {
                        self.apply_jump_offset(offset);
                    } else {
                        self.stack.push(value);
                    }
                }
                Bytecode::Loop(offset) => self.apply_jump_offset(offset),
                Bytecode::GuardBool(offset) => {
                    let top = *self.stack.last().ok_or(RuntimeError::Internal("Stack underflow for GuardBool".to_string()))?;
                    let takes_fast_path = top.as_bool().is_some() && self.universe.bool_sacred_pristine;
                    if !takes_fast_path {
                        self.apply_jump_offset(offset);
                    }
                }
                Bytecode::GuardBlock(offset) => {
                    if !self.universe.block_sacred_pristine {
                        self.apply_jump_offset(offset);
                    }
                }
                Bytecode::GuardSymbol => {
                    let value = *self.stack.last().ok_or(RuntimeError::Internal("Stack underflow for GuardSymbol".to_string()))?;
                    if value.symbol_value().is_none() {
                        return Err(RuntimeError::Type {
                            expected: "Symbol product label",
                            found: value.type_name(),
                        }
                        .into());
                    }
                }
                Bytecode::GuardFieldKey => {
                    let value = *self
                        .stack
                        .last()
                        .ok_or(RuntimeError::Internal("Stack underflow for GuardFieldKey".to_string()))?;
                    crate::product::validate_field_key_value(self, value)?;
                }
                Bytecode::RecordLiteralCheckKey => {
                    let key_value = *self
                        .stack
                        .last()
                        .ok_or(RuntimeError::Internal("Stack underflow for RecordLiteralCheckKey".to_string()))?;
                    let builder = *self
                        .stack
                        .get(self.stack.len().saturating_sub(2))
                        .ok_or(RuntimeError::Internal("RecordLiteralCheckKey requires builder and key".to_string()))?;
                    let key = crate::product::value_to_field_key(self, key_value)?;
                    let Some(id) = builder.as_obj() else {
                        return Err(RuntimeError::Internal("Record literal builder is not an object".into()).into());
                    };
                    let is_builder = matches!(self.heap.get(id), Object::RecordLiteralBuilder(_));
                    if !is_builder {
                        return Err(RuntimeError::Internal("Record literal builder has wrong object kind".into()).into());
                    }
                    if self.heap.record_literal_builder(id).contains_key(&key) {
                        return Err(RuntimeError::DuplicateProductLabel {
                            product: "Record field",
                            label: key.to_string(),
                        }
                        .into());
                    }
                }
                Bytecode::LoadTypeParameterClass(index) => {
                    let parameter = callable.chunk.executable_semantics.type_parameter_class_receiver(index).clone();
                    let environment_id = self
                        .frames
                        .last()
                        .map(|frame| frame.type_environment)
                        .unwrap_or(crate::typing::environment::RuntimeTypeEnvironmentId::EMPTY);
                    let argument = self
                        .runtime_type_environments
                        .get(environment_id)
                        .and_then(|environment| environment.get_argument(&parameter))
                        .ok_or_else(|| RuntimeError::Internal(format!("no active runtime type binding for {parameter:?}")))?;
                    let runtime_type = match argument {
                        crate::typing::overlay::RuntimeStaticArgument::Term(runtime_type) => *runtime_type,
                        crate::typing::overlay::RuntimeStaticArgument::DynamicType => {
                            return Err(RuntimeError::Internal(format!("explicit Dynamic binding for {parameter:?} has no nominal class boundary")).into());
                        }
                        crate::typing::overlay::RuntimeStaticArgument::RecordRow(_) => {
                            return Err(RuntimeError::Internal(format!("record-row binding for {parameter:?} has no nominal class boundary")).into());
                        }
                    };
                    let class = crate::typing::inspect::nominal_origin_class(&self.runtime_typing_context, &self.typing_registry, runtime_type)
                        .ok_or_else(|| RuntimeError::Internal(format!("runtime type binding for {parameter:?} has no nominal class object")))?;
                    self.stack.push(crate::value::Value::obj(class));
                }
                Bytecode::NewArgumentPack => {
                    let id = self.heap.alloc(Object::PackBuilder(Box::new(crate::heap::ArgumentPackBuilderObject::new())));
                    self.stack.push(Value::obj(id));
                }
                Bytecode::PackPushPositional => {
                    let builder = self.pop()?;
                    let value = self.pop()?;
                    let Some(id) = builder.as_obj() else {
                        return Err(RuntimeError::Internal("pack builder is not an object".into()).into());
                    };
                    self.heap.pack_builder_mut(id).push_positional(value);
                }
                Bytecode::PackReserveStaticLabel(index) => {
                    let builder = self.pop()?;
                    let label = callable.chunk.constants[index as usize].as_symbol().map_err(RuntimeError::Internal)?;
                    let Some(id) = builder.as_obj() else {
                        return Err(RuntimeError::Internal("pack builder is not an object".into()).into());
                    };
                    self.heap
                        .pack_builder_mut(id)
                        .reserve_label(label)
                        .map_err(|error| self.pack_builder_error(error))?;
                }
                Bytecode::PackReserveComputedLabel => {
                    let builder = self.pop()?;
                    let label = self.pop()?;
                    let Some(label) = label.symbol_value() else {
                        return Err(RuntimeError::ComputedLabelNotSymbol { found: label.type_name() }.into());
                    };
                    let Some(id) = builder.as_obj() else {
                        return Err(RuntimeError::Internal("pack builder is not an object".into()).into());
                    };
                    self.heap
                        .pack_builder_mut(id)
                        .reserve_label(label)
                        .map_err(|error| self.pack_builder_error(error))?;
                }
                Bytecode::PackFillReservedLabel => {
                    let builder = self.pop()?;
                    let value = self.pop()?;
                    let Some(id) = builder.as_obj() else {
                        return Err(RuntimeError::Internal("pack builder is not an object".into()).into());
                    };
                    self.heap
                        .pack_builder_mut(id)
                        .fill_reserved(value)
                        .map_err(|error| RuntimeError::Internal(format!("invalid pack reservation state: {error:?}")))?;
                }
                opcode @ (Bytecode::PackExpandLabels | Bytecode::PackExpandComplete) => {
                    let complete = matches!(opcode, Bytecode::PackExpandComplete);
                    let builder = self.pop()?;
                    let operand = self.pop()?;
                    let Some(builder) = builder.as_obj() else {
                        return Err(RuntimeError::Internal("pack builder is not an object".into()).into());
                    };
                    let entries = if complete {
                        if operand.is_unit() || operand.is_empty_pack() {
                            Vec::new()
                        } else if let Some(view) = self.pack_view_value(operand) {
                            view.labeled_entries()
                        } else if let Some(id) = operand.as_obj() {
                            if matches!(self.heap.get(id), Object::Tuple(_)) {
                                self.snapshot_labeled_lane(operand)?
                            } else {
                                return Err(RuntimeError::InvalidStarStarStarOperand { found: operand.type_name() }.into());
                            }
                        } else {
                            return Err(RuntimeError::InvalidStarStarStarOperand { found: operand.type_name() }.into());
                        }
                    } else {
                        self.snapshot_labeled_lane(operand)?
                    };
                    for (label, value) in entries {
                        self.heap
                            .pack_builder_mut(builder)
                            .append_labeled(label, value)
                            .map_err(|error| self.pack_builder_error(error))?;
                    }
                    if complete {
                        if let Some(view) = self.pack_view_value(operand) {
                            for value in view.positionals() {
                                self.heap.pack_builder_mut(builder).push_positional(value);
                            }
                        } else if let Some(id) = operand.as_obj()
                            && matches!(self.heap.get(id), Object::Tuple(_))
                        {
                            let values = self
                                .tuple_view(id)
                                .ok_or_else(|| RuntimeError::Internal("Tuple descriptor is unavailable".into()))?
                                .positionals();
                            for value in values {
                                self.heap.pack_builder_mut(builder).push_positional(value);
                            }
                        }
                    }
                }
                Bytecode::PackTryExpandTuplePositionals | Bytecode::ListTryExpandTuplePositionals | Bytecode::ArrayTryExpandTuplePositionals => {
                    let target = self.pop()?;
                    let source = self.pop()?;
                    if self.try_append_tuple_positionals(source, target)? {
                        self.stack.push(TRUE);
                    } else {
                        self.stack.push(FALSE);
                    }
                }
                Bytecode::FinishTuplePack => {
                    let builder = self.pop()?;
                    let Some(id) = builder.as_obj() else {
                        return Err(RuntimeError::Internal("pack builder is not an object".into()).into());
                    };
                    if self.heap.pack_builder(id).has_pending() {
                        return Err(RuntimeError::Internal("unfinished pack label at tuple finish".into()).into());
                    }
                    let (positionals, labels, values) = self.heap.pack_builder_mut(id).take_parts();
                    let labeled = labels.into_iter().zip(values).collect();
                    let tuple =
                        crate::product::finish_tuple(self, positionals, labeled).map_err(|error| crate::product::runtime_error(self, "Tuple label", error))?;
                    self.stack.push(tuple);
                }
                Bytecode::FinishPack => {
                    let builder = self.pop()?;
                    let Some(id) = builder.as_obj() else {
                        return Err(RuntimeError::Internal("pack builder is not an object".into()).into());
                    };
                    if self.heap.pack_builder(id).has_pending() {
                        return Err(RuntimeError::Internal("unfinished pack label at pack finish".into()).into());
                    }
                    let (positionals, labels, values) = self.heap.pack_builder_mut(id).take_parts();
                    let labeled = labels.into_iter().zip(values).collect();
                    let pack =
                        crate::product::finish_pack(self, positionals, labeled).map_err(|error| crate::product::runtime_error(self, "Pack label", error))?;
                    self.stack.push(pack);
                }
                Bytecode::InvokePack { base_name, kind, access } => {
                    let builder = self.pop()?;
                    let receiver_idx = self
                        .stack
                        .len()
                        .checked_sub(1)
                        .ok_or(RuntimeError::Internal("missing dynamic send receiver".into()))?;
                    let Some(id) = builder.as_obj() else {
                        return Err(RuntimeError::Internal("pack builder is not an object".into()).into());
                    };
                    if self.heap.pack_builder(id).has_pending() {
                        return Err(RuntimeError::Internal("unfinished pack label at dynamic send".into()).into());
                    }
                    let (positionals, labels, values) = self.heap.pack_builder_mut(id).take_parts();
                    let arity = positionals.len() + values.len();
                    if arity > u8::MAX as usize {
                        return Err(RuntimeError::SendArityExceedsLimit {
                            found: arity,
                            limit: u8::MAX as usize,
                        }
                        .into());
                    }
                    let base = callable.chunk.constants[base_name as usize].as_symbol().map_err(RuntimeError::Internal)?;
                    let selector = self.dynamic_pack_selector(base, kind, positionals.len(), &labels)?;
                    self.stack.extend(positionals);
                    self.stack.extend(values);
                    let range = callable.chunk.span_at(ip);
                    if access == PackAccess::CompilerInternal {
                        self.compiler_internal_dispatch_depth += 1;
                    }
                    let result = self.invoke_dynamic_selector(receiver_idx, selector, arity, range);
                    if access == PackAccess::CompilerInternal {
                        self.compiler_internal_dispatch_depth -= 1;
                    }
                    result?;
                }
                Bytecode::InvokeSubscriptSetPack { base_name, access } => {
                    let rhs = self.pop()?;
                    let builder = self.pop()?;
                    let receiver_idx = self
                        .stack
                        .len()
                        .checked_sub(1)
                        .ok_or(RuntimeError::Internal("missing dynamic subscript setter receiver".into()))?;
                    let Some(id) = builder.as_obj() else {
                        return Err(RuntimeError::Internal("pack builder is not an object".into()).into());
                    };
                    if self.heap.pack_builder(id).has_pending() {
                        return Err(RuntimeError::Internal("unfinished pack label at dynamic subscript setter".into()).into());
                    }
                    let (positionals, labels, values) = self.heap.pack_builder_mut(id).take_parts();
                    let index_arity = positionals.len() + values.len();
                    let arity = index_arity + 1;
                    if arity > u8::MAX as usize {
                        return Err(RuntimeError::SendArityExceedsLimit {
                            found: arity,
                            limit: u8::MAX as usize,
                        }
                        .into());
                    }
                    let base = callable.chunk.constants[base_name as usize].as_symbol().map_err(RuntimeError::Internal)?;
                    let selector = self.dynamic_pack_selector(base, PackSendKind::SubscriptSet, positionals.len(), &labels)?;
                    self.stack.extend(positionals);
                    self.stack.extend(values);
                    self.stack.push(rhs);
                    let range = callable.chunk.span_at(ip);
                    if access == PackAccess::CompilerInternal {
                        self.compiler_internal_dispatch_depth += 1;
                    }
                    let result = self.invoke_dynamic_selector(receiver_idx, selector, arity, range);
                    if access == PackAccess::CompilerInternal {
                        self.compiler_internal_dispatch_depth -= 1;
                    }
                    result?;
                }
                Bytecode::SuperSendPack { base_name } => {
                    let anchor = self
                        .heap
                        .closure(closure_id)
                        .super_anchor
                        .ok_or_else(|| RuntimeError::Internal("dynamic super send missing closure anchor".into()))?;
                    let builder = self.pop()?;
                    let receiver_idx = self
                        .stack
                        .len()
                        .checked_sub(1)
                        .ok_or(RuntimeError::Internal("missing dynamic super receiver".into()))?;
                    let receiver = self.stack[receiver_idx];
                    let Some(id) = builder.as_obj() else {
                        return Err(RuntimeError::Internal("pack builder is not an object".into()).into());
                    };
                    if self.heap.pack_builder(id).has_pending() {
                        return Err(RuntimeError::Internal("unfinished pack label at dynamic super send".into()).into());
                    }
                    let (positionals, labels, values) = self.heap.pack_builder_mut(id).take_parts();
                    let arity = positionals.len() + values.len();
                    if arity > u8::MAX as usize {
                        return Err(RuntimeError::SendArityExceedsLimit {
                            found: arity,
                            limit: u8::MAX as usize,
                        }
                        .into());
                    }
                    let base = callable.chunk.constants[base_name as usize].as_symbol().map_err(RuntimeError::Internal)?;
                    let selector = self.dynamic_pack_selector(base, PackSendKind::Method, positionals.len(), &labels)?;
                    self.stack.extend(positionals);
                    self.stack.extend(values);
                    let parent = self.heap.class(anchor).superclass;
                    let range = callable.chunk.span_at(ip);
                    let exact = parent.and_then(|class| crate::heap::lookup_method_in_hierarchy(&self.heap, class, selector));
                    if let Some(method) = exact {
                        let foreign_guard = self.frames.last().and_then(|frame| frame.foreign_receiver_guard);
                        let frames_before = self.frames.len();
                        self.call_method(&receiver, method, arity, range)?;
                        if self.frames.len() > frames_before {
                            self.frames.last_mut().unwrap().foreign_receiver_guard = foreign_guard;
                        }
                    } else {
                        let (name, slots, kind) = decode_selector(self.resolve_symbol(selector));
                        let positional_count = slots.iter().filter(|slot| slot.is_none()).count();
                        let labels = slots
                            .iter()
                            .filter_map(|slot| slot.as_ref())
                            .map(|label| self.interner.intern(label))
                            .collect::<Vec<_>>();
                        let rest = matches!(kind, SignatureKind::Method(_))
                            .then(|| self.interner.intern(&name))
                            .and_then(|base| parent.and_then(|class| self.lookup_rest_method(class, base, positional_count, &labels)));
                        if let Some(method) = rest {
                            let foreign_guard = self.frames.last().and_then(|frame| frame.foreign_receiver_guard);
                            let frames_before = self.frames.len();
                            self.activate_rest_method(&receiver, method, receiver_idx, positional_count, &labels, selector, range)?;
                            if self.frames.len() > frames_before {
                                self.frames.last_mut().unwrap().foreign_receiver_guard = foreign_guard;
                            }
                        } else {
                            self.forward_does_not_understand(receiver_idx, selector, range)?;
                        }
                    }
                }
                Bytecode::BuildTuple { positional, labeled } => {
                    let mut entries = Vec::with_capacity(labeled as usize);
                    for _ in 0..labeled {
                        let value = self.pop()?;
                        let label = self.pop()?;
                        let Some(label) = label.symbol_value() else {
                            return Err(RuntimeError::Type {
                                expected: "Symbol tuple label",
                                found: label.type_name(),
                            }
                            .into());
                        };
                        entries.push((label, value));
                    }
                    entries.reverse();
                    let mut positionals = Vec::with_capacity(positional as usize);
                    for _ in 0..positional {
                        positionals.push(self.pop()?);
                    }
                    positionals.reverse();
                    let product =
                        crate::product::finish_tuple(self, positionals, entries).map_err(|error| crate::product::runtime_error(self, "Tuple label", error))?;
                    self.stack.push(product);
                }
                Bytecode::BuildPack { positional, labeled } => {
                    let mut entries = Vec::with_capacity(labeled as usize);
                    for _ in 0..labeled {
                        let value = self.pop()?;
                        let label = self.pop()?;
                        let Some(label) = label.symbol_value() else {
                            return Err(RuntimeError::Type {
                                expected: "Symbol pack label",
                                found: label.type_name(),
                            }
                            .into());
                        };
                        entries.push((label, value));
                    }
                    entries.reverse();
                    let mut positionals = Vec::with_capacity(positional as usize);
                    for _ in 0..positional {
                        positionals.push(self.pop()?);
                    }
                    positionals.reverse();
                    let product =
                        crate::product::finish_pack(self, positionals, entries).map_err(|error| crate::product::runtime_error(self, "Pack label", error))?;
                    self.stack.push(product);
                }
                Bytecode::BuildRecord { fields } => {
                    let mut entries = Vec::with_capacity(fields as usize);
                    for _ in 0..fields {
                        let value = self.pop()?;
                        let key_value = self.pop()?;
                        let key = crate::product::value_to_field_key(self, key_value)?;
                        entries.push((key, value));
                    }
                    entries.reverse();
                    let product = crate::product::finish_record(self, entries).map_err(|error| crate::product::runtime_error(self, "Record field", error))?;
                    self.stack.push(product);
                }
                Bytecode::BuildStaticTuple { spec } => {
                    let spec = callable.chunk.executable_semantics.anonymous_product_spec(spec);
                    let count = match &spec.kind {
                        crate::modules::semantic_lowering::AnonymousProductConstructionKind::Tuple { positional_len, labels } => {
                            usize::try_from(*positional_len).unwrap_or(usize::MAX).saturating_add(labels.len())
                        }
                        _ => return Err(RuntimeError::Internal("Tuple bytecode carried a Record construction spec".into()).into()),
                    };
                    let mut values = Vec::with_capacity(count);
                    for _ in 0..count {
                        values.push(self.pop()?);
                    }
                    values.reverse();
                    let product =
                        crate::product::finish_tuple_from_spec(self, spec, values).map_err(|error| crate::product::runtime_error(self, "Tuple", error))?;
                    self.stack.push(product);
                }
                Bytecode::BuildStaticRecord { spec } => {
                    let spec = callable.chunk.executable_semantics.anonymous_product_spec(spec);
                    let count = match &spec.kind {
                        crate::modules::semantic_lowering::AnonymousProductConstructionKind::Record { logical_keys, .. } => logical_keys.len(),
                        _ => return Err(RuntimeError::Internal("Record bytecode carried a Tuple construction spec".into()).into()),
                    };
                    let mut values = Vec::with_capacity(count);
                    for _ in 0..count {
                        values.push(self.pop()?);
                    }
                    values.reverse();
                    let product =
                        crate::product::finish_record_from_spec(self, spec, values).map_err(|error| crate::product::runtime_error(self, "Record", error))?;
                    self.stack.push(product);
                }
                Bytecode::BuildStaticPack { spec } => {
                    let spec = callable.chunk.executable_semantics.anonymous_product_spec(spec);
                    let count = match &spec.kind {
                        crate::modules::semantic_lowering::AnonymousProductConstructionKind::Pack { positional_len, labels } => {
                            usize::try_from(*positional_len).unwrap_or(usize::MAX).saturating_add(labels.len())
                        }
                        _ => return Err(RuntimeError::Internal("Pack bytecode carried a non-Pack construction spec".into()).into()),
                    };
                    let mut values = Vec::with_capacity(count);
                    for _ in 0..count {
                        values.push(self.pop()?);
                    }
                    values.reverse();
                    let product =
                        crate::product::finish_pack_from_spec(self, spec, values).map_err(|error| crate::product::runtime_error(self, "Pack", error))?;
                    self.stack.push(product);
                }
                Bytecode::NewRecordLiteralBuilder => {
                    let id = self.heap.alloc_record_literal_builder();
                    self.stack.push(Value::obj(id));
                }
                Bytecode::RecordLiteralAppend => {
                    let value = self.pop()?;
                    let key_value = self.pop()?;
                    let key = crate::product::value_to_field_key(self, key_value)?;
                    let builder = self.pop()?;
                    let Some(id) = builder.as_obj() else {
                        return Err(RuntimeError::Internal("Record literal builder is not an object".into()).into());
                    };
                    if !matches!(self.heap.get(id), Object::RecordLiteralBuilder(_)) {
                        return Err(RuntimeError::Internal("Record literal builder has wrong object kind".into()).into());
                    }
                    self.heap.record_literal_builder_mut(id).append(key, value);
                }
                Bytecode::RecordLiteralExpandLabels => {
                    let source = self.pop()?;
                    let builder = self.pop()?;
                    let Some(id) = builder.as_obj() else {
                        return Err(RuntimeError::Internal("Record literal builder is not an object".into()).into());
                    };
                    if !matches!(self.heap.get(id), Object::RecordLiteralBuilder(_)) {
                        return Err(RuntimeError::Internal("Record literal builder has wrong object kind".into()).into());
                    }
                    for (key, value) in self.snapshot_record_fields(source)? {
                        if self.heap.record_literal_builder(id).contains_key(&key) {
                            return Err(RuntimeError::DuplicateProductLabel {
                                product: "Record field",
                                label: key.to_string(),
                            }
                            .into());
                        }
                        self.heap.record_literal_builder_mut(id).append(key, value);
                    }
                }
                Bytecode::FinishRecordLiteral => {
                    let builder = self.pop()?;
                    let Some(id) = builder.as_obj() else {
                        return Err(RuntimeError::Internal("Record literal builder is not an object".into()).into());
                    };
                    if !matches!(self.heap.get(id), Object::RecordLiteralBuilder(_)) {
                        return Err(RuntimeError::Internal("Record literal builder has wrong object kind".into()).into());
                    }
                    let entries = self.heap.record_literal_builder_mut(id).take_entries();
                    let product = crate::product::finish_record(self, entries).map_err(|error| crate::product::runtime_error(self, "Record field", error))?;
                    self.stack.push(product);
                }
                Bytecode::BeginMapLiteral => self.stack.push(Value::obj(self.heap.alloc_map())),
                Bytecode::MapLiteralInsertUnique => {
                    let value = self.pop()?;
                    let key = self.pop()?;
                    let map = *self
                        .stack
                        .last()
                        .ok_or(RuntimeError::Internal("Stack underflow for MapLiteralInsertUnique".to_string()))?;
                    let Some(id) = map.as_obj() else {
                        return Err(RuntimeError::Internal("Map literal builder is not an object".to_string()).into());
                    };
                    if !matches!(self.heap.get(id), crate::heap::Object::Map(_)) {
                        return Err(RuntimeError::Internal("Map literal builder is not a Map".to_string()).into());
                    }
                    crate::primitive::map::map_literal_insert_unique(self, id, key, value)?;
                }
                Bytecode::MapLiteralExpandLabels => {
                    let source = self.pop()?;
                    let map = *self
                        .stack
                        .last()
                        .ok_or(RuntimeError::Internal("Stack underflow for MapLiteralExpandLabels".into()))?;
                    let Some(id) = map.as_obj() else {
                        return Err(RuntimeError::Internal("Map literal builder is not an object".into()).into());
                    };
                    if !matches!(self.heap.get(id), Object::Map(_)) {
                        return Err(RuntimeError::Internal("Map literal builder is not a Map".into()).into());
                    }
                    let entries = self.snapshot_labeled_lane(source)?;
                    let count = entries.len();
                    for (label, value) in entries.into_iter().rev() {
                        self.stack.push(Value::symbol(label));
                        self.stack.push(value);
                    }
                    for _ in 0..count {
                        let value = self.pop()?;
                        let key = self.pop()?;
                        crate::primitive::map::map_literal_insert_unique(self, id, key, value)?;
                    }
                }
                Bytecode::FinishMapLiteral => {}
                Bytecode::BeginArrayLiteral | Bytecode::BeginVectorLiteral => self.stack.push(Value::obj(self.heap.alloc_array_literal_builder())),
                Bytecode::ArrayLiteralAppend => {
                    let value = self.pop()?;
                    let builder = self.pop()?;
                    let Some(id) = builder.as_obj() else {
                        return Err(RuntimeError::Internal("Array literal builder is not an object".to_string()).into());
                    };
                    if !matches!(self.heap.get(id), Object::ArrayLiteralBuilder(_)) {
                        return Err(RuntimeError::Internal("Array literal builder is not an ArrayLiteralBuilder".to_string()).into());
                    }
                    self.heap.array_literal_builder_mut(id).push(value);
                    self.stack.push(builder);
                }
                Bytecode::FinishArrayLiteral => {
                    let builder = self.pop()?;
                    let Some(id) = builder.as_obj() else {
                        return Err(RuntimeError::Internal("Array literal builder is not an object".to_string()).into());
                    };
                    if self.heap.as_array_literal_builder(id).is_none() {
                        return Err(RuntimeError::Internal("Array literal builder is not an ArrayLiteralBuilder".to_string()).into());
                    }
                    let elements = self.heap.array_literal_builder_mut(id).take_elements();
                    *self.heap.get_mut(id) = Object::Array(crate::heap::ArrayObject::from_vec(elements));
                    self.stack.push(builder);
                }
                Bytecode::VectorExpandExactPositionals { vector, expected } => {
                    let source = self.pop()?;
                    let builder = self.pop()?;
                    let id = builder
                        .as_obj()
                        .ok_or_else(|| RuntimeError::Internal("Vector builder is not an object".into()))?;
                    let values = if vector {
                        let source_id = source
                            .as_obj()
                            .ok_or_else(|| RuntimeError::Internal("exact Vector spread is not an object".into()))?;
                        self.heap
                            .as_vector(source_id)
                            .ok_or_else(|| RuntimeError::Internal("exact Vector spread has incorrect native storage".into()))?
                            .elements()
                            .to_vec()
                    } else {
                        self.positional_lane(source)
                            .ok_or_else(|| RuntimeError::NonIterableStarOperand { found: source.type_name() })?
                    };
                    if values.len() != expected as usize || self.heap.as_array_literal_builder(id).is_none() {
                        return Err(RuntimeError::Internal("proved Vector spread cardinality differs from runtime storage".into()).into());
                    }
                    for value in values {
                        self.heap.array_literal_builder_mut(id).push(value);
                    }
                    self.stack.push(builder);
                }
                Bytecode::FinishVectorLiteral(expected) => {
                    let builder = self.pop()?;
                    let id = builder
                        .as_obj()
                        .ok_or_else(|| RuntimeError::Internal("Vector builder is not an object".into()))?;
                    if self
                        .heap
                        .as_array_literal_builder(id)
                        .is_none_or(|builder| builder.elements().len() != expected as usize)
                    {
                        return Err(RuntimeError::Internal("proved Vector extent differs from constructed storage".into()).into());
                    }
                    let elements = self.heap.array_literal_builder_mut(id).take_elements();
                    *self.heap.get_mut(id) = Object::Vector(crate::heap::VectorObject::new(elements.into_boxed_slice()));
                    self.stack.push(builder);
                }
                Bytecode::BeginListLiteral => self.stack.push(Value::obj(self.heap.alloc_list(Vec::new()))),
                Bytecode::ListLiteralAppend => {
                    let value = self.pop()?;
                    let list = self.pop()?;
                    let Some(id) = list.as_obj() else {
                        return Err(RuntimeError::Internal("List literal builder is not an object".to_string()).into());
                    };
                    if !matches!(self.heap.get(id), Object::List(_)) {
                        return Err(RuntimeError::Internal("List literal builder is not a List".to_string()).into());
                    }
                    self.heap.list_mut(id).push(value);
                    self.stack.push(list);
                }
                Bytecode::FinishListLiteral => {
                    let list = *self
                        .stack
                        .last()
                        .ok_or(RuntimeError::Internal("Stack underflow for FinishListLiteral".to_string()))?;
                    let Some(id) = list.as_obj() else {
                        return Err(RuntimeError::Internal("List literal builder is not an object".to_string()).into());
                    };
                    if !matches!(self.heap.get(id), Object::List(_)) {
                        return Err(RuntimeError::Internal("List literal builder is not a List".to_string()).into());
                    }
                }
                Bytecode::BeginSetLiteral => self.stack.push(Value::obj(self.heap.alloc_set())),
                Bytecode::SetLiteralAdd => {
                    let value = self.pop()?;
                    let set = *self
                        .stack
                        .last()
                        .ok_or(RuntimeError::Internal("Stack underflow for SetLiteralAdd".to_string()))?;
                    let Some(id) = set.as_obj() else {
                        return Err(RuntimeError::Internal("Set literal builder is not an object".to_string()).into());
                    };
                    if !matches!(self.heap.get(id), crate::heap::Object::Set(_)) {
                        return Err(RuntimeError::Internal("Set literal builder is not a Set".to_string()).into());
                    }
                    crate::primitive::set::set_literal_add(self, id, value)?;
                }
                Bytecode::FinishSetLiteral => {}
                Bytecode::BuildRange {
                    has_lower,
                    has_upper,
                    upper_inclusive,
                } => {
                    let upper = if has_upper { Some(self.pop()?) } else { None };
                    let lower = if has_lower { Some(self.pop()?) } else { None };
                    let range = self.heap.alloc_range(lower, upper, upper_inclusive);
                    self.stack.push(Value::obj(range));
                }
                Bytecode::BuildArray(count) => {
                    let mut elements = Vec::with_capacity(count as usize);
                    for _ in 0..count {
                        elements.push(self.pop()?);
                    }
                    elements.reverse();
                    let array = self.heap.alloc_array(elements);
                    self.stack.push(Value::obj(array));
                }
                Bytecode::CopyVector => {
                    let source = self.pop()?;
                    let source_id = source
                        .as_obj()
                        .ok_or_else(|| RuntimeError::Internal("proved Vector identity source is not an object".into()))?;
                    let elements = self
                        .heap
                        .as_vector(source_id)
                        .ok_or_else(|| RuntimeError::Internal("proved Vector identity source has incorrect native storage".into()))?
                        .elements()
                        .to_vec();
                    let vector = self.heap.alloc_vector(elements);
                    self.stack.push(Value::obj(vector));
                }
                Bytecode::BuildVector(count) => {
                    let mut elements = Vec::with_capacity(count as usize);
                    for _ in 0..count {
                        elements.push(self.pop()?);
                    }
                    elements.reverse();
                    let vector = self.heap.alloc_vector(elements);
                    self.stack.push(Value::obj(vector));
                }
                Bytecode::BuildList(count) => {
                    let mut elements = Vec::with_capacity(count as usize);
                    for _ in 0..count {
                        elements.push(self.pop()?);
                    }
                    elements.reverse();
                    let list = self.heap.alloc_list(elements);
                    self.stack.push(Value::obj(list));
                }
            }
            #[cfg(feature = "vm-trace")]
            debug!("Stack after opcode {:?}: {:?}", opcode, self.stack);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{ConditionalInvocation, VM};
    use crate::callable::Callable;
    use crate::chunk::Chunk;
    use crate::compiler::lib::UnitKind;
    use crate::frame::CallContext;
    use crate::heap::{ClassObject, FiberObject, FiberStatus, InstanceObject, Object};
    use crate::method::{MethodKind, MethodObject, RestLayout, RestMode, Signature, SignatureKind};
    use crate::parameters::ParameterShape;
    use crate::value::Value;
    use crate::vm::BufferedOutput;

    #[test]
    fn duplicate_rest_family_is_rejected_before_index_mutation() {
        let mut vm = VM::new();
        let module = vm.create_module("main", "duplicate_rest_family");
        let closure = vm
            .compile_closure_as(module, "let marker = 0\n", UnitKind::File)
            .expect("test closure should compile");
        let target = vm.universe.classes.object_class;
        let base = vm.get_or_intern("sum");
        let first_selector = vm.get_or_intern("sum(*)");
        let first_signature = crate::method::Signature::new_with_arity(
            first_selector,
            crate::method::SignatureKind::Method(0),
            0,
            Some(RestLayout::new(0, Vec::new().into_boxed_slice(), RestMode::Positional { param_index: 0 })),
        );
        let first = vm.heap.alloc(Object::Method(Box::new(MethodObject::new_single(
            first_signature,
            MethodKind::Closure(closure),
        ))));
        vm.install_method_binding(target, first_selector, first)
            .expect("first rest family should install");

        let second_selector = vm.get_or_intern("sum(**)");
        let second_signature = crate::method::Signature::new_with_arity(
            second_selector,
            crate::method::SignatureKind::Method(0),
            0,
            Some(RestLayout::new(0, Vec::new().into_boxed_slice(), RestMode::Labeled { param_index: 0 })),
        );
        let second = vm.heap.alloc(Object::Method(Box::new(MethodObject::new_single(
            second_signature,
            MethodKind::Closure(closure),
        ))));

        let error = vm
            .install_method_binding(target, second_selector, second)
            .expect_err("second rest family must be rejected");
        assert!(error.to_string().contains("already has rest method"), "unexpected error: {error}");
        assert_eq!(
            vm.heap.class(target).methods.get(&first_selector).copied(),
            Some(first),
            "failed install must preserve the first structural method binding"
        );
        assert_eq!(
            vm.heap.class(target).get_rest_method(base),
            Some(first),
            "failed install must preserve existing index entry"
        );
        assert!(
            !vm.heap.class(target).methods.contains_key(&second_selector),
            "rejected rest family must not leak into the structural method dictionary"
        );
    }

    #[test]
    fn invoke_conditional_strict_subclass_override_uses_empty_evidence_environment() {
        let mut vm = VM::new_kernel();
        let module = vm.create_module("audit", "direct_conditional_override_environment_isolation");
        let base = vm.heap.alloc_class(ClassObject::bare("ConditionalBase"));
        let subclass = vm.heap.alloc_class(ClassObject::bare("ConditionalSubclass"));
        vm.heap.class_mut(subclass).set_superclass(Some(base));

        let selector_sym = vm.get_or_intern("render");
        let selector = vm.get_or_intern("render");
        let declaring_name = vm.get_or_intern("ConditionalBase");
        let signature = Signature::new(selector, SignatureKind::Getter, 0);
        let body = vm.compile_closure(module, "42\n").expect("override body");
        let fallback = vm
            .heap
            .alloc(Object::Method(Box::new(MethodObject::new_single(signature.clone(), MethodKind::Closure(body)))));
        let override_method = vm
            .heap
            .alloc(Object::Method(Box::new(MethodObject::new_single(signature, MethodKind::Closure(body)))));
        vm.heap.class_mut(subclass).add_method(selector, override_method);
        let receiver = Value::obj(vm.heap.alloc(Object::Instance(InstanceObject::new(subclass, 0))));
        vm.classes.insert(crate::vm::ClassKey { module, name: declaring_name }, base);

        let mut chunk = Chunk::new();
        let selector_index = chunk.add_constant(Value::symbol(selector_sym), Default::default()).unwrap();
        let module_index = chunk.add_constant(Value::obj(module), Default::default()).unwrap();
        let declaring_name_index = chunk.add_constant(Value::symbol(declaring_name), Default::default()).unwrap();
        let fallback_index = chunk.add_constant(Value::obj(fallback), Default::default()).unwrap();
        chunk
            .executable_semantics
            .call_environment_recipes
            .push(std::sync::Arc::new(crate::typing::RuntimeCallEnvironmentRecipe {
                callable: phalcom_type_meta::identity::StableCallableRef {
                    owner: phalcom_type_meta::identity::StableCallableOwnerRef::Declaration(phalcom_type_meta::identity::StableDeclarationRef {
                        module: phalcom_type_meta::identity::StableModuleRef {
                            project: phalcom_type_meta::identity::StableProjectRef::Session {
                                session_fingerprint: phalcom_type_meta::fingerprint::Fingerprint128::from_u128(0),
                            },
                            path: Box::new([]),
                        },
                        path: Box::new([]),
                    }),
                    side: phalcom_type_meta::identity::StableDispatchSide::Instance,
                    selector: "render".into(),
                },
                bindings: Box::new([]),
                conformance_slots: Box::new([crate::typing::RuntimeConformanceSlotRecipe::Forwarded(0)]),
            }));
        let callable = Callable {
            chunk,
            max_slots: 1,
            num_upvalues: 0,
            upvalues: Vec::new(),
            arity: 0,
            parameter_shape: ParameterShape::closure(0, false),
            name_sym: selector_sym,
            local_names: Vec::new(),
        };

        vm.stack.push(receiver);
        vm.invoke_conditional_at(
            &callable,
            0,
            ConditionalInvocation {
                arity: 0,
                selector: selector_index,
                declaring_module: module_index,
                declaring_name: declaring_name_index,
                class_side: false,
                fallback_method: fallback_index,
                call_environment_recipe: 0,
            },
        )
        .expect("direct conditional override should activate");

        assert_eq!(vm.frames.len(), 1, "override body should be entered");
        assert_eq!(
            vm.frames.last().expect("override frame").conformance_environment,
            crate::typing::RuntimeConformanceEnvironmentId::EMPTY,
            "ordinary override must not receive the fallback's forwarded evidence recipe"
        );
    }

    #[test]
    fn terminal_observer_is_detached_and_admitted_once() {
        let mut vm = VM::new();
        let module = vm.create_module("main", "completion_observer");
        let action = vm.compile_closure_as(module, "42\n", UnitKind::File).expect("action closure should compile");
        let observer = vm.compile_closure_as(module, "1\n", UnitKind::File).expect("observer closure should compile");
        let fiber = vm.heap.alloc(Object::Fiber(Box::new(FiberObject::new_entry(action))));
        vm.mark_fiber_done(fiber, crate::value::Value::int(42));
        vm.heap.fiber_mut(fiber).completion_observer = Some(observer);

        vm.enqueue_completion_observer(fiber).expect("valid observer should enqueue");
        assert_eq!(vm.heap.fiber(fiber).completion_observer, None);
        let observer_fiber = vm.pop_next_queued().expect("observer should be queued");
        assert_eq!(vm.heap.fiber(observer_fiber).status, FiberStatus::Queued);
        assert_eq!(vm.heap.fiber(observer_fiber).entry, Some(observer));
        vm.enqueue_completion_observer(fiber).expect("detached observer is a no-op");
        assert_eq!(vm.pop_next_queued(), None);
    }

    #[test]
    fn root_result_survives_scheduler_drain_and_gc() {
        let mut vm = VM::new();
        let module = vm.create_module("main", "root_result_drain");
        let root = vm
            .compile_closure_as(
                module,
                "System.schedule(|| { System.gc; \"discarded\" })\n\"root-\" + \"result\"\n",
                UnitKind::File,
            )
            .expect("root closure should compile");
        let frame = vm.new_call_frame(root, CallContext::Module { module }, 0, 0, None);
        vm.push_frame(frame).expect("root frame should push");
        let value = vm.run().expect("scheduled work should drain");
        assert_eq!(value.to_string(&vm), "root-result");
    }

    #[test]
    fn failing_completion_observer_becomes_unhandled_scheduler_work() {
        let output = BufferedOutput::new();
        let handle = output.handle();
        let mut vm = VM::new_with_output(Box::new(output));
        let module = vm.create_module("main", "completion_observer_failure");
        let action = vm.compile_closure_as(module, "42\n", UnitKind::File).expect("action closure should compile");
        let observer = vm
            .compile_closure_as(module, "Error.new(\"observer boom\").raise()\n", UnitKind::File)
            .expect("observer closure should compile");
        let root = vm.compile_closure_as(module, "1\n", UnitKind::File).expect("root closure should compile");
        let action_fiber = vm.heap.alloc(Object::Fiber(Box::new(FiberObject::new_entry(action))));
        vm.mark_fiber_done(action_fiber, crate::value::Value::int(42));
        vm.heap.fiber_mut(action_fiber).completion_observer = Some(observer);
        vm.enqueue_completion_observer(action_fiber).expect("observer should be admitted");

        let frame = vm.new_call_frame(root, CallContext::Module { module }, 0, 0, None);
        vm.push_frame(frame).expect("root frame should push");
        vm.run().expect("observer failure must stay isolated");

        assert_eq!(
            String::from_utf8(handle.bytes()).expect("runtime output should be UTF-8"),
            "Unhandled scheduled Fiber failure:\nobserver boom\n"
        );
    }
}
