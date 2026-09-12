//! VM-owned control continuations, activation records, and transfer routing.
//!
//! Language-level control operations (`on`, `ensure`, `whileTrue`, `Bool`/`Option`
//! fallbacks) represent their semantic continuations in VM/Fiber-owned control
//! activations ([`ControlActivation`]) rather than holding Rust call stack frames
//! across arbitrary guest code execution.

use crate::error::{PhError, PhResult, RuntimeError};
use crate::frame::FrameToken;
use crate::heap::{InstanceObject, ObjRef, Object};
use crate::interner::Symbol;
use crate::method::{ArgumentView, CallOutcome, InvocationLayout};
use crate::value::Value;
use crate::vm::VM;
use phalcom_common::range::SourceRange;

/// Unique monotonic identifier for a control activation within a fiber.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ControlId(pub u64);

/// Explicit destination where a control callback or resumed value/error should be delivered.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlDestination {
    /// Deliver into the caller's operand stack window at the specified index.
    StackOperand { target_index: usize },
    /// Deliver to the enclosing or target control activation.
    ControlActivation { id: ControlId },
}

/// Kind of Boolean conditional branch being executed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoolBranchKind {
    IfTrue,
    IfFalse,
    IfTrueOption,
    IfFalseOption,
    IfTrueIfFalse,
}

/// Semantic control transfer representing value return, exception raise, or non-local return.
#[derive(Debug, Clone)]
pub enum Transfer {
    /// Normal return of a value.
    Returned(Value),
    /// Exception raised during execution.
    Raise(PhError),
    /// Non-local return unwinding to an enclosing method frame token.
    NonLocalReturn { target: FrameToken, value: Value },
}

impl Transfer {
    /// Traces GC heap references inside this transfer.
    pub fn trace_roots(&self, push: &mut impl FnMut(ObjRef)) {
        match self {
            Transfer::Returned(val) | Transfer::NonLocalReturn { value: val, .. } => {
                if let Some(obj) = val.as_obj() {
                    push(obj);
                }
            }
            Transfer::Raise(err) => {
                if let PhError::Runtime(RuntimeError::Raise { error, .. }) = err {
                    if let Some(obj) = error.as_obj() {
                        push(obj);
                    }
                }
            }
        }
    }
}

/// Execution phase of a VM-owned control activation.
#[derive(Debug, Clone)]
pub enum ControlPhase {
    /// Protected body of `Block.on(class, handler)`.
    OnBody { class: Value, handler: Value },
    /// `error.is(class)` matching phase after an error occurred in `OnBody`.
    OnMatch {
        class: Value,
        handler: Value,
        error: Value,
        original_err: PhError,
    },
    /// Handler invocation phase for `Block.on`.
    OnHandler { original_error: Value },
    /// Protected body of `Block.ensure(cleanup)`.
    EnsureBody { cleanup: Value },
    /// Cleanup block execution phase for `Block.ensure`. Holds the saved prior transfer.
    EnsureCleanup { saved_transfer: Box<Transfer> },
    /// Condition block evaluation of `Block.whileTrue(body)`.
    WhileCondition { condition: Value, body: Value },
    /// Loop body evaluation of `Block.whileTrue(body)`.
    WhileBody { condition: Value, body: Value },
    /// Boolean conditional callback execution.
    BoolBranch { kind: BoolBranchKind, branch: Value },
    /// Option pattern matching callback execution.
    OptionBranch { branch: Value },
    /// Ordering comparison reversal callback.
    OrderingReverse { reverse_selector: Symbol },
}

impl ControlPhase {
    /// Traces GC heap references inside this control phase.
    pub fn trace_roots(&self, push: &mut impl FnMut(ObjRef)) {
        match self {
            ControlPhase::OnBody { class, handler } => {
                if let Some(obj) = class.as_obj() {
                    push(obj);
                }
                if let Some(obj) = handler.as_obj() {
                    push(obj);
                }
            }
            ControlPhase::OnMatch {
                class,
                handler,
                error,
                original_err,
            } => {
                if let Some(obj) = class.as_obj() {
                    push(obj);
                }
                if let Some(obj) = handler.as_obj() {
                    push(obj);
                }
                if let Some(obj) = error.as_obj() {
                    push(obj);
                }
                if let PhError::Runtime(RuntimeError::Raise { error: err_val, .. }) = original_err {
                    if let Some(obj) = err_val.as_obj() {
                        push(obj);
                    }
                }
            }
            ControlPhase::OnHandler { original_error } => {
                if let Some(obj) = original_error.as_obj() {
                    push(obj);
                }
            }
            ControlPhase::EnsureBody { cleanup } => {
                if let Some(obj) = cleanup.as_obj() {
                    push(obj);
                }
            }
            ControlPhase::EnsureCleanup { saved_transfer } => {
                saved_transfer.trace_roots(push);
            }
            ControlPhase::WhileCondition { condition, body } | ControlPhase::WhileBody { condition, body } => {
                if let Some(obj) = condition.as_obj() {
                    push(obj);
                }
                if let Some(obj) = body.as_obj() {
                    push(obj);
                }
            }
            ControlPhase::BoolBranch { branch, .. } | ControlPhase::OptionBranch { branch } => {
                if let Some(obj) = branch.as_obj() {
                    push(obj);
                }
            }
            ControlPhase::OrderingReverse { .. } => {}
        }
    }
}

/// A VM-owned control activation record.
#[derive(Debug, Clone)]
pub struct ControlActivation {
    pub id: ControlId,
    pub owner: Option<FrameToken>,
    pub stack_base: usize,
    pub callback_floor: usize,
    pub caller_authority: (Option<ObjRef>, bool),
    pub source_range: SourceRange,
    pub destination: ControlDestination,
    pub phase: ControlPhase,
}

impl ControlActivation {
    /// Traces GC heap references inside this control activation.
    pub fn trace_roots(&self, push: &mut impl FnMut(ObjRef)) {
        if let (Some(auth), _) = self.caller_authority {
            push(auth);
        }
        self.phase.trace_roots(push);
    }
}

/// A routed transfer paired with an optional target destination.
#[derive(Debug, Clone)]
pub struct RoutedTransfer {
    pub transfer: Transfer,
    pub destination: Option<ControlDestination>,
}

impl RoutedTransfer {
    pub fn trace_roots(&self, push: &mut impl FnMut(ObjRef)) {
        self.transfer.trace_roots(push);
    }
}

/// A per-fiber stack of control activations and pending routed transfers.
#[derive(Debug, Clone, Default)]
pub struct ControlStack {
    pub records: Vec<ControlActivation>,
    pub pending: Option<RoutedTransfer>,
    pub next_id: u64,
}

impl ControlStack {
    /// Creates an empty control stack.
    pub fn new() -> Self {
        Self {
            records: Vec::new(),
            pending: None,
            next_id: 1,
        }
    }

    /// Pushes a new control activation and returns its unique [`ControlId`].
    pub fn push(
        &mut self,
        owner: Option<FrameToken>,
        stack_base: usize,
        callback_floor: usize,
        caller_authority: (Option<ObjRef>, bool),
        source_range: SourceRange,
        destination: ControlDestination,
        phase: ControlPhase,
    ) -> ControlId {
        let id = ControlId(self.next_id);
        self.next_id += 1;
        self.records.push(ControlActivation {
            id,
            owner,
            stack_base,
            callback_floor,
            caller_authority,
            source_range,
            destination,
            phase,
        });
        id
    }

    /// Pops the topmost control activation record.
    pub fn pop(&mut self) -> Option<ControlActivation> {
        self.records.pop()
    }

    /// Returns a reference to the topmost control activation.
    pub fn top(&self) -> Option<&ControlActivation> {
        self.records.last()
    }

    /// Returns a mutable reference to the topmost control activation.
    pub fn top_mut(&mut self) -> Option<&mut ControlActivation> {
        self.records.last_mut()
    }

    /// Finds a control activation by its [`ControlId`].
    pub fn find_control(&self, id: ControlId) -> Option<&ControlActivation> {
        self.records.iter().find(|r| r.id == id)
    }

    /// Finds a mutable control activation by its [`ControlId`].
    pub fn find_control_mut(&mut self, id: ControlId) -> Option<&mut ControlActivation> {
        self.records.iter_mut().find(|r| r.id == id)
    }

    /// Returns true if there are no active control records.
    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    /// Number of active control records.
    pub fn len(&self) -> usize {
        self.records.len()
    }

    /// Clears all control records and pending transfers.
    pub fn clear(&mut self) {
        self.records.clear();
        self.pending = None;
    }

    /// Traces all heap roots in this control stack.
    pub fn trace_roots(&self, push: &mut impl FnMut(ObjRef)) {
        for record in &self.records {
            record.trace_roots(push);
        }
        if let Some(pending) = &self.pending {
            pending.trace_roots(push);
        }
    }
}

/// Outcome of stepping a control activation through the reducer.
#[derive(Debug)]
pub enum ControlStepOutcome {
    /// A new guest frame/control was entered or fiber switched; continue dispatch loop.
    Continued,
    /// The control completed with a return value delivered to its destination.
    Completed(Value),
    /// The control completed with a transfer that must be propagated outwards.
    Propagate(Transfer),
}

impl VM {
    /// Helper to activate a callable receiver with positional arguments on top of the stack.
    pub(crate) fn enter_callable_activation(
        &mut self,
        callable: Value,
        args: &[Value],
        source_range: SourceRange,
    ) -> PhResult<CallOutcome> {
        let receiver_idx = self.stack.len();
        self.stack.push(callable);
        self.stack.extend_from_slice(args);
        let layout = InvocationLayout::ordinary(args.len(), Box::new([]));
        let view = ArgumentView::from_layout(
            receiver_idx,
            layout,
            self.current_access_class(),
            self.current_has_internal_privilege(),
        );
        self.activate_function(callable, view, source_range)
    }

    /// Drives the control reducer when a callback returns, raises, or non-locally returns.
    pub(crate) fn step_control_transfer(&mut self, mut transfer: Transfer) -> PhResult<ControlStepOutcome> {
        while let Some(mut control) = self.control_stack.pop() {
            let stack_base = control.stack_base;
            let callback_floor = control.callback_floor;
            let source_range = control.source_range;

            match &mut control.phase {
                ControlPhase::OnBody { class, handler } => {
                    match transfer {
                        Transfer::Returned(val) => {
                            self.deliver_control_value(control.destination, val)?;
                            return Ok(ControlStepOutcome::Completed(val));
                        }
                        Transfer::Raise(mut err) => {
                            // Extract surface error value and ensure traceback is captured.
                            let captured_tb = self.capture_frames(callback_floor);
                            let error_val = match &err {
                                PhError::Runtime(RuntimeError::Raise { error, .. }) => *error,
                                _ => {
                                    let error_class = self.universe.classes.error_class;
                                    let field_count = self.heap.class(error_class).field_count;
                                    let mut inst = InstanceObject::new(error_class, field_count);
                                    inst.slots[0] = self.alloc_string_value(err.to_string());
                                    Value::obj(self.heap.alloc(Object::Instance(inst)))
                                }
                            };
                            match err {
                                PhError::Runtime(RuntimeError::Raise {
                                    error,
                                    rendered,
                                    mut traceback,
                                    help,
                                }) => {
                                    if traceback.is_none() {
                                        traceback = Some(captured_tb);
                                    }
                                    err = PhError::Runtime(RuntimeError::Raise {
                                        error,
                                        rendered,
                                        traceback,
                                        help,
                                    });
                                }
                                _ => {
                                    let rendered = err.to_string();
                                    err = PhError::Runtime(RuntimeError::Raise {
                                        error: error_val,
                                        rendered,
                                        traceback: Some(captured_tb),
                                        help: None,
                                    });
                                }
                            }

                            // Unwind down to the protected region's base before dynamic `is` matching.
                            self.unwind_to(stack_base, callback_floor);

                            // Push updated control record in OnMatch phase.
                            let class_val = *class;
                            let handler_val = *handler;
                            control.phase = ControlPhase::OnMatch {
                                class: class_val,
                                handler: handler_val,
                                error: error_val,
                                original_err: err,
                            };
                            self.control_stack.records.push(control);

                            // Perform `error_val.is(class_val)` dynamic send.
                            let isa_sig = crate::method::encode_selector("is", &[None], crate::method::SignatureKind::Method(1));
                            let isa_sym = self.get_or_intern(&isa_sig);
                            let receiver_idx = self.stack.len();
                            self.stack.push(error_val);
                            self.stack.push(class_val);
                            let layout = self.invocation_layout_for_selector(isa_sym, 1)?;
                            self.dispatch_selector_window_as(
                                receiver_idx,
                                isa_sym,
                                layout,
                                source_range,
                                (self.current_access_class(), self.current_has_internal_privilege()),
                            )?;
                            return Ok(ControlStepOutcome::Continued);
                        }
                        Transfer::NonLocalReturn { .. } => {
                            // Non-local return unwinds through `on` without catching.
                            // Continue outward transfer propagation.
                            continue;
                        }
                    }
                }
                ControlPhase::OnMatch {
                    class: _,
                    handler,
                    error,
                    original_err,
                } => {
                    match transfer {
                        Transfer::Returned(matched) => {
                            if matched.as_bool() == Some(true) {
                                // Matched! Transition to OnHandler and invoke handler with caught error.
                                let err_val = *error;
                                let h_val = *handler;
                                control.phase = ControlPhase::OnHandler { original_error: err_val };
                                self.control_stack.records.push(control);

                                self.enter_callable_activation(h_val, &[err_val], source_range)?;
                                return Ok(ControlStepOutcome::Continued);
                            } else {
                                // Match failed: re-raise original error to outer controls.
                                transfer = Transfer::Raise(original_err.clone());
                                continue;
                            }
                        }
                        Transfer::Raise(e) => {
                            // Error during `is` dispatch overrides.
                            transfer = Transfer::Raise(e);
                            continue;
                        }
                        Transfer::NonLocalReturn { .. } => {
                            continue;
                        }
                    }
                }
                ControlPhase::OnHandler { .. } => {
                    // Handler outcome passes straight through (matching/handler errors are not caught by same region).
                    match transfer {
                        Transfer::Returned(val) => {
                            self.deliver_control_value(control.destination, val)?;
                            return Ok(ControlStepOutcome::Completed(val));
                        }
                        Transfer::Raise(_) | Transfer::NonLocalReturn { .. } => {
                            continue;
                        }
                    }
                }
                ControlPhase::EnsureBody { cleanup } => {
                    let cleanup_val = *cleanup;
                    // Unwind any leftover child frames down to callback floor.
                    self.unwind_to(stack_base, callback_floor);

                    control.phase = ControlPhase::EnsureCleanup {
                        saved_transfer: Box::new(transfer.clone()),
                    };
                    self.control_stack.records.push(control);

                    self.enter_callable_activation(cleanup_val, &[], source_range)?;
                    return Ok(ControlStepOutcome::Continued);
                }
                ControlPhase::EnsureCleanup { saved_transfer } => {
                    match transfer {
                        Transfer::Returned(_) => {
                            // Cleanup completed normally: restore and resume saved prior transfer!
                            let saved = *saved_transfer.clone();
                            match saved {
                                Transfer::Returned(v) => {
                                    self.deliver_control_value(control.destination, v)?;
                                    return Ok(ControlStepOutcome::Completed(v));
                                }
                                Transfer::Raise(_) | Transfer::NonLocalReturn { .. } => {
                                    transfer = saved;
                                    continue;
                                }
                            }
                        }
                        Transfer::Raise(cleanup_err) => {
                            // Cleanup raised: supersedes prior transfer.
                            transfer = Transfer::Raise(cleanup_err);
                            continue;
                        }
                        Transfer::NonLocalReturn { .. } => {
                            // Cleanup non-locally returned: supersedes prior transfer.
                            continue;
                        }
                    }
                }
                ControlPhase::WhileCondition { condition, body } => {
                    match transfer {
                        Transfer::Returned(cond_val) => {
                            let Some(cond_bool) = cond_val.as_bool() else {
                                transfer = Transfer::Raise(
                                    RuntimeError::Type {
                                        expected: "Bool",
                                        found: cond_val.type_name(),
                                    }
                                    .into(),
                                );
                                continue;
                            };
                            if !cond_bool {
                                let none_val = self.none_value();
                                self.deliver_control_value(control.destination, none_val)?;
                                return Ok(ControlStepOutcome::Completed(none_val));
                            }
                            let cond_block = *condition;
                            let body_block = *body;
                            control.phase = ControlPhase::WhileBody {
                                condition: cond_block,
                                body: body_block,
                            };
                            self.control_stack.records.push(control);

                            self.enter_callable_activation(body_block, &[], source_range)?;
                            return Ok(ControlStepOutcome::Continued);
                        }
                        Transfer::Raise(_) | Transfer::NonLocalReturn { .. } => {
                            continue;
                        }
                    }
                }
                ControlPhase::WhileBody { condition, body } => {
                    match transfer {
                        Transfer::Returned(_) => {
                            let cond_block = *condition;
                            let body_block = *body;
                            control.phase = ControlPhase::WhileCondition {
                                condition: cond_block,
                                body: body_block,
                            };
                            self.control_stack.records.push(control);

                            self.enter_callable_activation(cond_block, &[], source_range)?;
                            return Ok(ControlStepOutcome::Continued);
                        }
                        Transfer::Raise(_) | Transfer::NonLocalReturn { .. } => {
                            continue;
                        }
                    }
                }
                ControlPhase::BoolBranch { kind, branch: _ } => {
                    match transfer {
                        Transfer::Returned(branch_val) => {
                            let delivered = match kind {
                                BoolBranchKind::IfTrue | BoolBranchKind::IfFalse | BoolBranchKind::IfTrueIfFalse => branch_val,
                                BoolBranchKind::IfTrueOption | BoolBranchKind::IfFalseOption => {
                                    let some_class = self.universe.classes.some_class;
                                    let field_count = self.heap.class(some_class).field_count;
                                    let mut inst = InstanceObject::new(some_class, field_count);
                                    inst.slots[0] = branch_val;
                                    Value::obj(self.heap.alloc(Object::Instance(inst)))
                                }
                            };
                            self.deliver_control_value(control.destination, delivered)?;
                            return Ok(ControlStepOutcome::Completed(delivered));
                        }
                        Transfer::Raise(_) | Transfer::NonLocalReturn { .. } => {
                            continue;
                        }
                    }
                }
                ControlPhase::OptionBranch { .. } => {
                    match transfer {
                        Transfer::Returned(val) => {
                            self.deliver_control_value(control.destination, val)?;
                            return Ok(ControlStepOutcome::Completed(val));
                        }
                        Transfer::Raise(_) | Transfer::NonLocalReturn { .. } => {
                            continue;
                        }
                    }
                }
                ControlPhase::OrderingReverse { .. } => {
                    match transfer {
                        Transfer::Returned(val) => {
                            self.deliver_control_value(control.destination, val)?;
                            return Ok(ControlStepOutcome::Completed(val));
                        }
                        Transfer::Raise(_) | Transfer::NonLocalReturn { .. } => {
                            continue;
                        }
                    }
                }
            }
        }

        // No more controls on the control stack. Propagate to fiber floor / caller.
        Ok(ControlStepOutcome::Propagate(transfer))
    }

    /// Delivers a completed control value into its designated target.
    fn deliver_control_value(&mut self, destination: ControlDestination, value: Value) -> PhResult<()> {
        match destination {
            ControlDestination::StackOperand { target_index } => {
                self.stack.truncate(target_index);
                self.stack.push(value);
                Ok(())
            }
            ControlDestination::ControlActivation { id } => {
                if let Some(control) = self.control_stack.find_control(id) {
                    let dest = control.destination;
                    self.deliver_control_value(dest, value)
                } else {
                    Ok(())
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::Value;

    #[test]
    fn control_stack_lifecycle_and_lookup() {
        let mut cs = ControlStack::new();
        assert!(cs.is_empty());
        assert_eq!(cs.len(), 0);

        let id1 = cs.push(
            None,
            0,
            0,
            (None, false),
            SourceRange::default(),
            ControlDestination::StackOperand { target_index: 0 },
            ControlPhase::WhileCondition {
                condition: Value::nil(),
                body: Value::nil(),
            },
        );
        let id2 = cs.push(
            None,
            1,
            1,
            (None, false),
            SourceRange::default(),
            ControlDestination::ControlActivation { id: id1 },
            ControlPhase::EnsureBody {
                cleanup: Value::nil(),
            },
        );

        assert_eq!(cs.len(), 2);
        assert!(!cs.is_empty());
        assert_eq!(cs.find_control(id1).map(|c| c.id), Some(id1));
        assert_eq!(cs.find_control(id2).map(|c| c.id), Some(id2));
        assert_eq!(cs.top().map(|c| c.id), Some(id2));

        let popped = cs.pop();
        assert_eq!(popped.map(|c| c.id), Some(id2));
        assert_eq!(cs.len(), 1);

        cs.clear();
        assert!(cs.is_empty());
    }

    #[test]
    fn control_phase_root_tracing_covers_all_variants() {
        let mut cs = ControlStack::new();
        cs.push(
            None,
            0,
            0,
            (None, false),
            SourceRange::default(),
            ControlDestination::StackOperand { target_index: 0 },
            ControlPhase::OnBody {
                class: Value::int(1),
                handler: Value::int(2),
            },
        );
        cs.push(
            None,
            0,
            0,
            (None, false),
            SourceRange::default(),
            ControlDestination::StackOperand { target_index: 0 },
            ControlPhase::EnsureCleanup {
                saved_transfer: Box::new(Transfer::Returned(Value::int(42))),
            },
        );

        let mut roots = Vec::new();
        cs.trace_roots(&mut |obj| roots.push(obj));
        // Primitives like ints are not heap objects, so no roots pushed
        assert!(roots.is_empty());
    }
}
