use crate::heap::BlockObject;
use crate::value::{FALSE, TRUE};
use crate::bytecode::Bytecode;
use crate::heap::ClosureObject;
use crate::diagnostics::{print_rt, SourceLoc};
use crate::error::{PhError, PhResult, RuntimeError};
use crate::frame::{CallContext, CallFrame};
use crate::heap::{ObjRef, Object};
use crate::method::{decode_selector, SignatureKind};
use crate::heap::CORE_MODULE_NAME;
use crate::heap::Upvalue;
use crate::value::Value;
use phalcom_common::range::SourceRange;
#[cfg(feature = "vm-trace")]
use tracing::{debug, span, Level};

use super::VM;

impl VM {
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
        let cell = self.heap.alloc(Object::Upvalue(Upvalue::Open { fiber: self.current, slot: stack_index }));
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

    /// Prints a runtime error with a source-mapped stack trace and returns it.
    ///
    /// # Errors
    ///
    /// Always returns `err` (the trace is a side effect).
    pub fn runtime_error(&mut self, err: PhError) -> PhResult<()> {
        let mut frames = Vec::new();
        for frame in self.frames.clone().iter().rev() {
            let closure = self.heap.closure(frame.closure);
            let module_id = closure.module;
            let name_sym = closure.callable.name_sym;
            let span = closure.callable.chunk.spans[frame.ip - 1];

            let module = self.heap.module(module_id);
            let module_name = module.name.clone();
            let module_source = module.source.clone();
            let method_name = self.resolve_symbol(name_sym).to_string();

            frames.push(SourceLoc {
                source: module_source.unwrap(),
                module_name,
                method_name,
                span,
            });
        }

        print_rt(&err.to_string(), &frames);
        Err(err)
    }

    /// Placeholder for compiler-error reporting (currently a no-op).
    pub fn compiler_error(&mut self, err: PhError) {}

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

    /// Surfaces the private `nil` sentinel as `None` at a read boundary.
    ///
    /// A thin wrapper over [`sentinel_to_option`](crate::value::sentinel_to_option)
    /// that supplies the shared `None` singleton. Called wherever the VM can
    /// read an unwritten slot (uninitialized `var`/local, unassigned field,
    /// bare-`return` default, a method falling off its end) so the private
    /// `Value::Nil` sentinel ([ADR-0010](../../../docs/adr/0010-tagged-value-enum.md))
    /// is replaced by `None` and never reaches user code (Invariant 4,
    /// [ADR-0007](../../../docs/adr/0007-option-some-none.md)).
    #[inline]
    pub(crate) fn surface_absence(&self, value: Value) -> Value {
        crate::value::sentinel_to_option(value, self.universe.classes.none_singleton)
    }

    /// Returns the shared `None` singleton as a [`Value`] (the surface absence
    /// value).
    ///
    /// This is the value every opcode and surface-reachable primitive yields to
    /// signal "no value" — the private [`Value::Nil`] sentinel is never handed
    /// to user code (Invariant 4, [ADR-0007](../../../docs/adr/0007-option-some-none.md)).
    /// It is `None`-identity-stable and zero-allocation because it reuses the
    /// process-wide [`CoreClasses::none_singleton`](crate::universe::CoreClasses::none_singleton).
    #[inline]
    pub(crate) fn none_value(&self) -> Value {
        Value::Obj(self.universe.classes.none_singleton)
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
        loop {
            match self.run_until_inner(0) {
                Ok(value) => {
                    let finished = self.current;
                    let Some(resumer) = self.heap.fiber(finished).resumer else {
                        // The root fiber's own top-level activation ended.
                        // Root-drive pump (concurrency.md §2, "a root-drive
                        // pump: `VM::run` drains the ready-queue once the
                        // top-level program's own activation ends"): before
                        // finishing, resume any fiber `System.schedule(_)`
                        // queued, exactly like the `.ph`-level manual pump
                        // (`System.runScheduled`, `core.ph`) does, so `await`
                        // can rely on this firing automatically without the
                        // top-level program calling `runScheduled` itself.
                        //
                        // `fiber_try` expects the ordinary Invoke calling
                        // convention (a receiver value already sitting on
                        // `self.stack`, consumed via its own `resume_slot`
                        // bookkeeping) — push a placeholder so its
                        // `receiver_idx` arithmetic doesn't underflow; it is
                        // parked into this (root) fiber's stack by
                        // `store_live_into` and overwritten on restore
                        // (`switch_to_fiber_and_deliver`'s `stack.truncate`),
                        // so it never leaks.
                        if let Some(next) = self.ready_queue.pop_front() {
                            self.stack.push(Value::Obj(next));
                            crate::primitive::fiber::fiber_try(self, &Value::Obj(next), &[])?;
                            continue;
                        }
                        return Ok(value);
                    };
                    // Fiber-floor capture, success path (spec §3.2): `finished`
                    // is a non-root fiber whose entry activation just drained
                    // to nothing. Deliver `value` to the resumer's `call`/
                    // `try` expression and switch back to it.
                    self.heap.fiber_mut(finished).status = crate::heap::FiberStatus::Done;
                    self.heap.fiber_mut(finished).result = value;
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
                    self.switch_to_fiber_and_deliver(resumer, value);
                    // Loop again: keep draining, now as `resumer`.
                }
                Err(e) => {
                    // Fiber-floor capture, failure path (spec §3.2, the
                    // DEC-FIB-A fix): the U-CORE-6 unwind reached the top of
                    // the current fiber's own activation uncaught. Capture it
                    // into its result slot instead of propagating past the
                    // fiber boundary. Under `call`, cascade the same capture
                    // straight up the resumer chain — without executing any
                    // of an intermediate `call`-mode resumer's own bytecode,
                    // exactly as if `e` had been raised at each `call()` site
                    // in turn with no handler — until a `try`-mode resumer
                    // (which gets the `Error` delivered as a value instead)
                    // or the root fiber (which ends the whole run) is
                    // reached. The host, and every fiber the failure doesn't
                    // reach, survives.
                    let error_value = self.capture_error_value(&e);
                    let mut failed = self.current;
                    loop {
                        self.heap.fiber_mut(failed).status = crate::heap::FiberStatus::Failed;
                        self.heap.fiber_mut(failed).result = error_value;
                        // Spec §5.1: a `Failed` fiber can never resume, so its
                        // parked state is pure retention — clear all three
                        // parked fields here (not just `frames`). The
                        // originating fiber's own `FiberObject` fields are
                        // already empty (they were `vm.frames`/`stack`/
                        // `open_upvalues`, the live mirror, when it raised);
                        // this matters for an intermediate `Call`-mode
                        // resumer walked by this cascade, whose fields still
                        // hold the state it parked when it resumed the fiber
                        // that ultimately failed.
                        self.heap.fiber_mut(failed).frames.clear();
                        self.heap.fiber_mut(failed).stack.clear();
                        self.heap.fiber_mut(failed).open_upvalues.clear();
                        let mode = self.heap.fiber(failed).resume_mode;
                        let Some(resumer) = self.heap.fiber(failed).resumer else {
                            return Err(e);
                        };
                        match mode {
                            crate::heap::FiberResumeMode::Try => {
                                self.switch_to_fiber_and_deliver(resumer, error_value);
                                break;
                            }
                            crate::heap::FiberResumeMode::Call => {
                                failed = resumer;
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
        self.current = target;
        crate::primitive::fiber::load_live_from(self, target);
        let slot = self.heap.fiber(target).resume_slot;
        self.stack.truncate(slot);
        self.stack.push(value);
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
        Value::Obj(self.heap.alloc(Object::Instance(inst)))
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
        loop {
            if self.frames.len() <= base_frames {
                // A drained frame stack with nothing left to yield means the
                // top-level program (or a block activation) fell off its end:
                // surface that absence as `None`, never the private sentinel.
                let result = self.stack.pop().unwrap_or(Value::Nil);
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

            let (opcode, source_range) = {
                let chunk = &self.heap.closure(closure_id).callable.chunk;
                (chunk.code[ip], chunk.spans[ip])
            };

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

            self.frames.last_mut().unwrap().ip += 1;

            match opcode {
                Bytecode::Constant(idx) => {
                    let constant = self.heap.closure(closure_id).callable.chunk.constants[idx as usize];
                    #[cfg(feature = "vm-trace")]
                    debug!("Pushing constant: {:?}", constant);
                    self.stack.push(constant);
                }
                Bytecode::Closure(idx) => {
                    // The constant is the *template* closure the compiler emitted
                    // (empty upvalue list). Materialize a fresh instance whose
                    // upvalue cells are captured from the current activation per
                    // the callable's descriptors, then wrap it in a BlockObject
                    // stamped with the home frame token (ADR-0013, functions.md §2).
                    let template = self.heap.closure(closure_id).callable.chunk.constants[idx as usize];
                    let Value::Obj(template_id) = template else {
                        return Err(RuntimeError::Internal("Closure constant is not a closure".to_string()).into());
                    };
                    let descriptors = self.heap.closure(template_id).callable.upvalues.clone();
                    let callable = self.heap.closure(template_id).callable.clone();
                    let module = self.heap.closure(template_id).module;

                    let mut upvalues = Vec::with_capacity(descriptors.len());
                    for desc in &descriptors {
                        let cell = if desc.is_local {
                            self.capture_upvalue(stack_offset + desc.index)
                        } else {
                            self.heap.closure(closure_id).upvalues[desc.index]
                        };
                        upvalues.push(cell);
                    }

                    let new_closure = self.heap.alloc(Object::Closure(Box::new(ClosureObject { callable, module, upvalues })));
                    let token = self.current_frame_token().expect("closure created inside a frame");
                    let block = self.heap.alloc(Object::Block(BlockObject::new(new_closure, token)));
                    self.stack.push(Value::Obj(block));
                }
                // `Bytecode::Nil` pushes the `None` singleton (the surface
                // absence value), never the raw private sentinel. It is emitted
                // both to seed uninitialized slots (e.g. a `var x` with no
                // initializer) and as the result of sacred-inlined one-armed
                // control flow (`ifTrue`, `ifFalse`, `whileTrue`), whose value
                // can flow straight into an arg or `print` without crossing a
                // read boundary — so it must already be `None` here (Invariant 4,
                // [ADR-0007]). The raw `Value::Nil` sentinel is never pushed by
                // any opcode; it only backs unread allocator storage and is
                // surfaced to `None` at reads (see the `Get*` handlers and
                // `surface_absence`). See [`Bytecode::Nil`]'s rustdoc.
                Bytecode::Nil => self.stack.push(self.none_value()),
                Bytecode::True => self.stack.push(TRUE),
                Bytecode::False => self.stack.push(FALSE),
                Bytecode::Pop => {
                    self.stack.pop();
                }
                Bytecode::DefineGlobal(idx) => {
                    let name_val = self.heap.closure(closure_id).callable.chunk.constants[idx as usize];
                    if let Value::Symbol(name_sym) = name_val {
                        let module_id = self.heap.closure(closure_id).module;
                        let value = *self.stack.last().unwrap();
                        self.heap.module_mut(module_id).define(name_sym, value).unwrap();
                        self.stack.pop();
                    }
                }
                Bytecode::GetGlobal(idx) => {
                    let name_val = self.heap.closure(closure_id).callable.chunk.constants[idx as usize];
                    if let Value::Symbol(name_sym) = name_val {
                        let module_id = self.heap.closure(closure_id).module;
                        if let Some(value) = self.heap.module(module_id).get(name_sym) {
                            // A declared-but-unassigned global holds the private
                            // sentinel (module slots initialize to `Value::Nil`);
                            // surface it as `None` on read.
                            let value = self.surface_absence(value);
                            self.stack.push(value);
                        } else {
                            // If not in the current module, try the core module.
                            let core_module_sym = self.interner.intern(CORE_MODULE_NAME);
                            let core_module = self.get_module(core_module_sym).expect("core module");
                            if let Some(value) = self.heap.module(core_module).get(name_sym) {
                                let value = self.surface_absence(value);
                                self.stack.push(value);
                            } else {
                                let name = self.resolve_symbol(name_sym).to_string();
                                return Err(RuntimeError::Message(format!("Undefined variable '{}'.", name)).into());
                            }
                        }
                    }
                }
                Bytecode::SetGlobal(idx) => {
                    let name_val = self.heap.closure(closure_id).callable.chunk.constants[idx as usize];
                    if let Value::Symbol(name_sym) = name_val {
                        let module_id = self.heap.closure(closure_id).module;
                        let slot = self.heap.module(module_id).name_to_slot.get(&name_sym).copied();
                        if let Some(slot) = slot {
                            let value = *self.stack.last().unwrap();
                            self.heap.module_mut(module_id).set_global(slot, value).unwrap();
                        } else {
                            let name = self.resolve_symbol(name_sym).to_string();
                            return Err(RuntimeError::Message(format!("Undefined variable '{}'.", name)).into());
                        }
                    }
                }
                Bytecode::GetLocal(slot) => {
                    let local_idx = stack_offset + slot as usize;
                    if local_idx < self.stack.len() {
                        // An uninitialized `var` slot is backed by the private
                        // sentinel; surface it as `None` on read (Invariant 4).
                        let value = self.surface_absence(self.stack[local_idx]);
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
                Bytecode::Class(idx) => {
                    let name_val = self.heap.closure(closure_id).callable.chunk.constants[idx as usize];
                    if let Value::Symbol(name_sym) = name_val {
                        let name = self.resolve_symbol(name_sym).to_string();
                        let superclass = self.stack.pop().unwrap();
                        match superclass {
                            Value::Obj(sc_id) if self.heap.as_class(sc_id).is_some() => {
                                // Reopening (ADR-0018: "attaches methods, not
                                // shadows"). A second `class A { ... }` block for
                                // a name already registered in `self.classes`
                                // must APPEND its methods to the existing
                                // `ClassId` rather than allocate a fresh one —
                                // a fresh class would orphan every method the
                                // earlier block installed (U-REOPEN-FIX).
                                //
                                // The compiler's own compile-time reuse guard
                                // (`compiler/lib.rs` `Statement::Class`, keyed
                                // on `self.vm.classes`) cannot see this case: a
                                // whole compile unit lowers to one closure
                                // before any `Bytecode::Class` executes, so
                                // `self.classes` is still empty for both blocks
                                // at compile time. This runtime check is the
                                // real reopen seam for same-unit reopens; it
                                // also covers the bootstrap case (a Rust stub
                                // pre-registers the class before its `.ph` body
                                // compiles) as a no-op fallthrough, since that
                                // case already resolves at compile time and
                                // never reaches here.
                                if let Some(&existing) = self.classes.get(&name_sym) {
                                    // A reopen must not change the superclass
                                    // (U13 sealed inheritance): the compiler's
                                    // `Statement::Class` handler rejects this
                                    // earlier via `class_parents`, but that
                                    // check only sees explicit `extends`
                                    // clauses recorded in the *same* compile
                                    // unit — mirror it here so a mismatch can
                                    // never silently reuse the wrong class.
                                    let existing_superclass = self.heap.class(existing).superclass;
                                    if existing_superclass != Some(sc_id) {
                                        return Err(RuntimeError::Message(format!(
                                            "Cannot reopen class '{name}': its superclass cannot be changed by a later `class {name}` block."
                                        ))
                                        .into());
                                    }
                                    self.stack.push(Value::Obj(existing));
                                } else {
                                    let new_class = self.create_class(&name, Some(sc_id));
                                    self.stack.push(Value::Obj(new_class));
                                }
                            }
                            _ => return Err(RuntimeError::InvalidSuperClass(format!("{superclass}")).into()),
                        }
                    }
                }
                Bytecode::Import(idx) => {
                    let path_val = self.heap.closure(closure_id).callable.chunk.constants[idx as usize];
                    let Value::Symbol(path_sym) = path_val else {
                        return Err(RuntimeError::Internal("Import constant is not a Symbol".into()).into());
                    };
                    let import_path = self.resolve_symbol(path_sym).to_string();
                    let importer_module = self.heap.closure(closure_id).module;
                    let importer_path = self.heap.module(importer_module).path.clone();
                    let imported = self.import_module(&importer_path, &import_path)?;
                    self.stack.push(Value::Obj(imported));
                }
                Bytecode::MakeFamily(name_idx) => {
                    let name_val = self.heap.closure(closure_id).callable.chunk.constants[name_idx as usize];
                    let recv = self.stack.pop().unwrap();
                    let Value::Symbol(sym) = name_val else {
                        return Err(RuntimeError::Internal("MakeFamily constant is not a Symbol".into()).into());
                    };
                    let class_id = recv.class(self);

                    // Open vs Pinned discriminator (U16-Pinned): the compiler
                    // (`compile_expr`'s `Expr::MethodRef` arm) always runs a
                    // Pinned reference's selector through
                    // `method::encode_selector`, which unconditionally
                    // appends a `(...)` argument-list suffix — even the
                    // zero-arg `name()` shape — while an Open reference's
                    // constant is always a bare base name from
                    // `parse_property_name` (an identifier/keyword, never
                    // containing `(`). No new discriminator field on the
                    // opcode is needed: the interned string's shape already
                    // tells the two apart.
                    let sym_str = self.resolve_symbol(sym).to_string();
                    let open = !sym_str.contains('(');

                    // Reference-time empty-family check (selectors.md §3
                    // error table): error unless the receiver's class
                    // responds to the reference (base name for Open, the
                    // exact selector for Pinned), or has a
                    // `doesNotUnderstand(_)` override.
                    let responds = if open {
                        self.heap.class(class_id).responds_to_base_name(sym)
                    } else {
                        crate::heap::lookup_method_in_hierarchy(&self.heap, class_id, sym).is_some()
                    };
                    if !responds {
                        let dnu_str = crate::method::encode_selector("doesNotUnderstand", &[None], SignatureKind::Method(1));
                        let dnu_sym = self.get_or_intern(&dnu_str);
                        let object_cls = self.universe.classes.object_class;
                        let default_dnu = crate::heap::lookup_method_in_hierarchy(&self.heap, object_cls, dnu_sym);
                        let actual_dnu = crate::heap::lookup_method_in_hierarchy(&self.heap, class_id, dnu_sym);
                        if actual_dnu == default_dnu {
                            let class_name = self.heap.class(class_id).name.clone();
                            return Err(RuntimeError::Message(format!(
                                "{class_name} does not understand `{sym_str}` — no method named `{sym_str}` and no `doesNotUnderstand(_)` override (`::` empty family)"
                            ))
                            .into());
                        }
                    }

                    let family = Object::Family(crate::heap::FamilyObject { recv, selector: sym, open });
                    let family_id = self.heap.alloc(family);
                    self.stack.push(Value::Obj(family_id));
                }
                Bytecode::FinalizeClass => {
                    if let Value::Obj(class_id) = *self.stack.last().unwrap() {
                        self.finalize_class_base_names(class_id);
                        let meta_id = self.heap.class(class_id).class;
                        self.finalize_class_base_names(meta_id);
                    }
                }
                Bytecode::SuperSend(argc, selector_idx, defining_idx) => {
                    let selector_val = self.heap.closure(closure_id).callable.chunk.constants[selector_idx as usize];
                    let defining_val = self.heap.closure(closure_id).callable.chunk.constants[defining_idx as usize];
                    let argc = argc as usize;
                    let selector_sym = selector_val.as_symbol().unwrap();
                    let defining_sym = defining_val.as_symbol().unwrap();

                    // The receiver stays the original `self` pushed by the
                    // compiler; only the lookup *start* moves above the defining
                    // class (method-lookup.md §1.14, U-INH §3.4).
                    let receiver_idx = self.stack.len() - 1 - argc;
                    let receiver = self.stack[receiver_idx];

                    // Start the walk at `defining.superclass` (DEC-INH-B: the
                    // defining class is resolved by name and its superclass read
                    // at dispatch, so a future `superclass=` mutation stays
                    // correct). The defining class was created before any of its
                    // methods could run, so it is present in `self.classes`.
                    // No superclass ⇒ the walk is empty ⇒ `doesNotUnderstand`.
                    let parent = self.classes.get(&defining_sym).and_then(|&c| self.heap.class(c).superclass);

                    let mut method = parent.and_then(|p| crate::heap::lookup_method_in_hierarchy(&self.heap, p, selector_sym));

                    // Super-construct fallback (U-INH §3.5): constructors are
                    // installed on the *metaclass* under a `SignatureKind::
                    // Initializer` selector. A positional `super.new(args)` /
                    // `super.construct(args)` (encoded as a `Method` selector)
                    // that misses the instance side re-encodes to its
                    // `Initializer` form and walks the superclass's metaclass
                    // chain, keeping the original instance as the receiver so the
                    // parent initializer fills the inherited slots in place.
                    if method.is_none()
                        && let Some(p) = parent
                    {
                        let sel_str = self.resolve_symbol(selector_sym).to_string();
                        let (name, labels, kind) = decode_selector(&sel_str);
                        if let SignatureKind::Method(a) = kind {
                            let init_selector = crate::method::encode_selector(&name, &labels, SignatureKind::Initializer(a));
                            let init_sym = self.interner.intern(&init_selector);
                            let parent_meta = self.heap.class(p).class;
                            method = crate::heap::lookup_method_in_hierarchy(&self.heap, parent_meta, init_sym);
                        }
                    }

                    if let Some(method) = method {
                        self.call_method(&receiver, method, argc, source_range)?;
                    } else {
                        self.forward_does_not_understand(receiver_idx, selector_sym, source_range)?;
                    }
                }
                Bytecode::Method(selector_idx, is_static) => {
                    let selector_val = self.heap.closure(closure_id).callable.chunk.constants[selector_idx as usize];
                    let selector = selector_val.as_symbol().unwrap();
                    let method_val = self.stack.pop().unwrap();
                    let class_val = *self.stack.last().unwrap();
                    if let (Value::Obj(method_id), Value::Obj(class_id)) = (method_val, class_val) {
                        self.heap.method_mut(method_id).set_holder(class_id);
                        if is_static {
                            let meta = self.heap.class(class_id).class;
                            self.heap.class_mut(meta).add_method(selector, method_id);
                            self.world_version += 1;
                        } else {
                            self.heap.class_mut(class_id).add_method(selector, method_id);
                            self.world_version += 1;
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
                    match receiver {
                        Value::Obj(id) => {
                            if let Some(instance) = self.heap.as_instance(id) {
                                let val = instance.slots.get(slot as usize).copied().unwrap_or(Value::Nil);
                                self.stack.push(self.surface_absence(val));
                            } else if let Some(class) = self.heap.as_class(id) {
                                let val = class.static_slots.get(slot as usize).copied().unwrap_or(Value::Nil);
                                self.stack.push(self.surface_absence(val));
                            } else {
                                return Err(RuntimeError::Internal(format!("Only instances and classes can have fields: {:?}", receiver)).into());
                            }
                        }
                        _ => return Err(RuntimeError::Internal(format!("Only instances and classes can have fields: {:?}", receiver)).into()),
                    }
                }
                Bytecode::SetField(slot) => {
                    let value_to_assign = self.stack.pop().ok_or("Stack underflow on field assignment")?;
                    let receiver = self.stack.pop().ok_or("Stack underflow for SetField receiver")?;
                    match receiver {
                        Value::Obj(id) => {
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
                                return Err(RuntimeError::Internal(format!("Only instances and classes can have fields: {:?}", receiver)).into());
                            }
                        }
                        _ => return Err(RuntimeError::Internal(format!("Only instances and classes can have fields: {:?}", receiver)).into()),
                    }
                }
                Bytecode::NewInstance => {
                    let class_val = self.stack.pop().ok_or("Stack underflow for NewInstance class")?;
                    if let Value::Obj(class_id) = class_val {
                        if self.heap.as_class(class_id).is_some() {
                            let field_count = self.heap.class(class_id).field_count;
                            let instance_ref = self.heap.alloc(Object::Instance(
                                crate::heap::InstanceObject::new(class_id, field_count)
                            ));
                            self.stack.push(Value::Obj(instance_ref));
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
                            return Err(RuntimeError::Internal(format!("NewInstance operand is not a class: {:?}", class_val)).into());
                        }
                    } else {
                        return Err(RuntimeError::Internal(format!("NewInstance operand is not a class object: {:?}", class_val)).into());
                    }
                }
                Bytecode::Dup => {
                    let val = *self.stack.last().ok_or("Stack underflow on Dup")?;
                    self.stack.push(val);
                }
                Bytecode::WrapSome => {
                    let value = self.stack.pop().ok_or("Stack underflow for WrapSome")?;
                    let wrapped = crate::primitive::nil::wrap_some(self, value);
                    self.stack.push(wrapped);
                }
                Bytecode::Invoke(arity, selector_idx) => {
                    let selector_val = self.heap.closure(closure_id).callable.chunk.constants[selector_idx as usize];
                    let arity = arity as usize;
                    let receiver_idx = self.stack.len() - 1 - arity;
                    let receiver = self.stack[receiver_idx];

                    let selector_sym = selector_val.as_symbol().unwrap();

                    if let Some(method) = receiver.lookup_method(self, selector_sym) {
                        self.call_method(&receiver, method, arity, source_range)?;
                    } else {
                        // Exact-selector probe missed. The method-lookup.md §1
                        // miss order is:
                        //   IC -> exact-probe -> variadic probe -> doesNotUnderstand(_).
                        //
                        // Only an all-positional `Method` selector may probe for a
                        // variadic candidate (never a labelled, getter, setter, or
                        // subscript selector) — derive the bare name, build the
                        // canonical `name(*)` selector, and do one ordinary
                        // `lookup_method` walk. A hit dispatches only if the call
                        // supplied at least the fixed prefix (`arity >=
                        // positional_arity`); otherwise, same as a miss, this falls
                        // through to the dNU forward (U9-implementation-spec.md §2
                        // "Runtime dispatch rule", ADR-0012, method-lookup.md §1-2).
                        let (name, labels, kind) = decode_selector(self.resolve_symbol(selector_sym));
                        let eligible = matches!(kind, SignatureKind::Method(_)) && labels.iter().all(Option::is_none);
                        let variadic_hit = eligible
                            .then(|| self.interner.intern(&format!("{name}(*)")))
                            .and_then(|variadic_selector| receiver.lookup_method(self, variadic_selector))
                            .and_then(|m| {
                                let sig = &self.heap.method(m).signature;
                                (arity >= sig.positional_arity as usize).then_some(m)
                            });
                        if let Some(method) = variadic_hit {
                            self.call_method(&receiver, method, arity, source_range)?;
                        } else {
                            self.forward_does_not_understand(receiver_idx, selector_sym, source_range)?;
                        }
                    }
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
                    // A captured-but-uninitialized `var` cell surfaces as `None`.
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
                    let return_value = self.stack.pop().unwrap_or(Value::Nil);
                    let return_value = self.surface_absence(return_value);
                    let popped = self.frames.pop().unwrap();
                    // Close any upvalues that still alias this frame's window so
                    // escaping closures survive the frame's disappearance
                    // (ADR-0013). Must run before the stack is truncated.
                    self.close_upvalues_from(popped.stack_offset);
                    self.stack.truncate(popped.stack_offset);
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
                    let is_live = self
                        .frames
                        .get(token.frame_index)
                        .is_some_and(|home| home.generation == token.generation);
                    if !is_live {
                        return Err(RuntimeError::DeadFrameError.into());
                    }
                    let home_stack_offset = self.frames[token.frame_index].stack_offset;

                    // The return value is already on the stack top (the compiler
                    // emits the expression, or `Bytecode::Nil` for a bare
                    // `return`); surface bare/absent to `None`, never the
                    // private sentinel — matching `Bytecode::Return`.
                    let return_value = self.stack.pop().unwrap_or(Value::Nil);
                    let return_value = self.surface_absence(return_value);

                    // Close every open upvalue at or above the home frame's
                    // window before the stack is truncated, so captures escaping
                    // *any* of the popped frames survive (one call covers all
                    // popped frames: each popped frame's `stack_offset` is
                    // `>= home_stack_offset`). Must run before truncation.
                    self.close_upvalues_from(home_stack_offset);
                    self.stack.truncate(home_stack_offset);
                    self.stack.push(return_value);
                    // Remove the home frame and everything above it. The value is
                    // now exactly where an ordinary `Return` from the home method
                    // would have left it; the unmodified top-of-loop drain check
                    // (in whichever nested `run_until` first finds its floor at
                    // or above the new length) yields it for real. Do NOT
                    // `return Ok(_)` here — let the loop continue.
                    self.frames.truncate(token.frame_index);
                }
                Bytecode::Jump(offset) => self.apply_jump_offset(offset),
                Bytecode::JumpIfFalse(offset) => {
                    let cond = self.pop()?;
                    match cond {
                        Value::Bool(true) => {}
                        Value::Bool(false) => self.apply_jump_offset(offset),
                        other => {
                            let found = other.type_name();
                            return Err(RuntimeError::Type { expected: "Bool", found }.into());
                        }
                    }
                }
                // Pops the top of stack and, if it is the `None` singleton (tested by identity),
                // branches to `offset`. Otherwise, falls through. Realizes the cursor-protocol
                // end-of-iteration test (ADR-0048 §2, iteration.md §2).
                Bytecode::JumpIfNone(offset) => {
                    let cursor = self.pop()?;
                    if cursor == Value::Obj(self.universe.classes.none_singleton) {
                        self.apply_jump_offset(offset);
                    }
                }
                Bytecode::Loop(offset) => self.apply_jump_offset(offset),
                Bytecode::GuardBool(offset) => {
                    let top = *self.stack.last().ok_or(RuntimeError::Internal("Stack underflow for GuardBool".to_string()))?;
                    let takes_fast_path = matches!(top, Value::Bool(_)) && self.universe.bool_sacred_pristine;
                    if !takes_fast_path {
                        self.apply_jump_offset(offset);
                    }
                }
                Bytecode::GuardBlock(offset) => {
                    if !self.universe.block_sacred_pristine {
                        self.apply_jump_offset(offset);
                    }
                }
            }
            #[cfg(feature = "vm-trace")]
            debug!("Stack after opcode {:?}: {:?}", opcode, self.stack);
        }
    }
}
