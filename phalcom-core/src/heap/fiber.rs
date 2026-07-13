use std::collections::{BTreeMap, HashSet};

use crate::frame::CallFrame;
use crate::value::Value;

use super::ObjRef;

/// The lifecycle state of a [`FiberObject`]
/// ([ADR-0030](../../../docs/adr/0030-fibers-and-futures-cooperative-concurrency.md) §1,
/// `concurrency.md` §1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FiberStatus {
    /// Created but not yet started, or suspended at a `yield` — resumable via
    /// `Fiber#call`/`Fiber#try`.
    Suspended,
    /// Currently executing on the VM: this is the `vm.current` fiber, whose
    /// live stacks are mirrored in [`VM::frames`](crate::vm::VM)/`stack`.
    Running,
    /// The entry function returned normally; [`FiberObject::result`] holds the
    /// return value and the fiber can no longer be resumed.
    Done,
    /// The entry raised an uncaught error; [`FiberObject::result`] holds the
    /// captured `Error` value (the fiber-floor capture, ADR-0030 §6) and the
    /// fiber can no longer be resumed.
    Failed,
}

/// How a fiber was last resumed — `call` re-raises the callee's failure into
/// the resumer, `try` captures it instead (ADR-0030 §6, spec §3.2/§5.2).
///
/// Recorded on the *callee* [`FiberObject`] at resume time so the fiber-floor
/// capture (in `VM::run_until`) knows how to deliver a `Failed`
/// outcome once it later happens — the resume call itself returns
/// immediately (an O(1) switch), long before the callee's eventual
/// success/failure is known.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FiberResumeMode {
    /// Resumed via `Fiber#call`/`call(_:)` — an uncaught failure re-raises
    /// into the resumer as if it had been raised at the `call` site.
    Call,
    /// Resumed via `Fiber#try`/`try(_:)` — a failure is captured and
    /// delivered as the `Error` value instead of raised.
    Try,
}

/// A cooperative, single-threaded fiber: its own value + call stacks, a
/// lifecycle [`FiberStatus`], a dynamic resumer link, a result slot, and its
/// entry closure ([ADR-0030](../../../docs/adr/0030-fibers-and-futures-cooperative-concurrency.md) §2,
/// `concurrency.md` §1).
///
/// A fiber owns its execution state so it can be parked and resumed by an O(1)
/// pointer swap of `vm.current` (ADR-0030 §3): while a fiber is
/// [`FiberStatus::Running`] its stacks live in the VM's live mirror
/// ([`VM::frames`](crate::vm::VM)/`stack`/`open_upvalues`) and the fields here
/// are empty; while parked they hold the fiber's state. Keeping the stacks
/// **inside the arena object** (never in native Rust memory) is what lets a
/// future tracing GC reach a parked fiber's roots (ADR-0030 §7, D1).
///
/// The `resumer` link + `result` slot are deliberately **general**, not
/// generator-specific, so the deferred `Future`/`await` layer can suspend
/// through exactly them (ADR-0030 §Consequences, forward-compat §7.2).
pub struct FiberObject {
    /// The fiber's private operand stack (empty while running — mirrored by
    /// [`VM::stack`](crate::vm::VM)). `stack_offset`s are window-relative
    /// (frame.rs, D3), so a per-fiber stack always based at index 0 needs no
    /// rebasing on switch.
    pub stack: Vec<Value>,
    /// The fiber's private call stack (empty while running — mirrored by
    /// [`VM::frames`](crate::vm::VM)). Because the frame-generation counter
    /// stays VM-global (D4), a non-local `return` token whose home lives on
    /// another fiber fails the generation check → `DeadFrameError`.
    pub frames: Vec<CallFrame>,
    /// The fiber's private open-upvalue map, keyed by absolute value-stack
    /// index (empty while running — mirrored by
    /// [`VM::open_upvalues`](crate::vm::VM)). Kept per-fiber because it is
    /// stack-index-keyed and each fiber has its own stack; swapping it with
    /// `stack`/`frames` prevents a cross-fiber slot-index collision.
    pub open_upvalues: BTreeMap<usize, ObjRef>,
    /// The fiber's lifecycle state ([`FiberStatus`]).
    pub status: FiberStatus,
    /// The fiber to hand control back to on `yield`/return/failure — a dynamic
    /// caller chain, not a fixed parent (`None` for the root fiber).
    pub resumer: Option<ObjRef>,
    /// The last yielded/returned value, or the captured `Error` when
    /// [`FiberStatus::Failed`] (ADR-0030 §6).
    pub result: Value,
    /// The entry [`super::Object::Block`]/[`super::Object::Closure`] the fiber runs on first
    /// resume; `None` for the root fiber (which has no entry).
    pub entry: Option<ObjRef>,
    /// Whether the entry frame has been pushed yet — `false` until the first
    /// `call`/`try`, then `true` for the fiber's life.
    pub started: bool,
    /// The value-stack length to truncate to (then push the delivered value)
    /// when this fiber is next resumed — recorded at the `yield` send whose
    /// window the resume value replaces (ADR-0030 §3).
    pub resume_slot: usize,
    /// The `run_until` nesting depth captured when the fiber last began
    /// running — the fiber floor the restricted-yield guard compares against
    /// (ADR-0030 §4): a `yield` is legal iff no native re-entrant `run_until`
    /// (a `block_call` and friends) has been entered since.
    pub floor_depth: usize,
    /// How this fiber was last resumed ([`FiberResumeMode`]) — read at the
    /// fiber-floor capture when this fiber later finishes/fails. Meaningless
    /// while [`FiberStatus::Suspended`] pre-first-resume; set on every
    /// `call`/`try`.
    pub resume_mode: FiberResumeMode,
    /// The identity set of receivers currently under `@invariant`
    /// re-entrancy-guard checking on this fiber (empty while running —
    /// mirrored by [`VM::checking`](crate::vm::VM)), populated/drained by the
    /// native `__invariantEnter`/`__invariantExit` primitives the
    /// `@invariant` weave calls
    /// ([ADR-0052](../../../docs/adr/0052-invariant-reentrancy-scope-and-layout-confined-decorator-state.md)
    /// Fix 1). Kept per-fiber, swapped alongside `stack`/`frames`/
    /// `open_upvalues` on fiber switch, because a guarded call can `yield`
    /// mid-body: a VM-global set would leak one fiber's in-flight
    /// invariant-checking bookkeeping into whatever fiber resumes next.
    pub checking: HashSet<ObjRef>,
}

impl FiberObject {
    /// Builds a fresh, not-yet-started fiber wrapping `entry` (its
    /// [`super::Object::Block`]/[`super::Object::Closure`] entry), status
    /// [`FiberStatus::Suspended`] (ADR-0030 §2).
    pub fn new_entry(entry: ObjRef) -> Self {
        Self {
            stack: Vec::new(),
            frames: Vec::new(),
            open_upvalues: BTreeMap::new(),
            status: FiberStatus::Suspended,
            resumer: None,
            result: Value::Nil,
            entry: Some(entry),
            started: false,
            resume_slot: 0,
            floor_depth: 0,
            resume_mode: FiberResumeMode::Call,
            checking: HashSet::new(),
        }
    }

    /// Builds the **root** fiber wrapping the main program: no entry, already
    /// started, [`FiberStatus::Running`] (ADR-0030 §1). Its live stacks are the
    /// VM's own [`VM::frames`](crate::vm::VM)/`stack` mirror, so the fields
    /// here stay empty while it runs.
    pub fn root() -> Self {
        Self {
            stack: Vec::new(),
            frames: Vec::new(),
            open_upvalues: BTreeMap::new(),
            status: FiberStatus::Running,
            resumer: None,
            result: Value::Nil,
            entry: None,
            started: true,
            resume_slot: 0,
            floor_depth: 0,
            resume_mode: FiberResumeMode::Call,
            checking: HashSet::new(),
        }
    }
}
