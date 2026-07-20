//! The bytecode virtual machine: dispatch loop, call stack, and heap ownership.
//!
//! The [`VM`] owns exactly one [`Heap`] ([ADR-0009](../../../docs/adr/0009-handle-arena-heap.md)):
//! every class, instance, method, module, closure and string lives there and is
//! reached through a `Copy` [`ObjRef`] handle rather than an `Rc<RefCell<T>>`.
//! Values on the operand stack are `Copy` [`Value`]s
//! ([ADR-0010](../../../docs/adr/0010-tagged-value-enum.md)); call frames
//! ([`CallFrame`]) are `Copy` too, so the interpreter carries no borrow-panic
//! surface. Method lookup keys on signature symbols (`object-model.md` §3).

mod api;
mod bootstrap;
mod dispatch;
mod gc;
mod send;

use crate::frame::CallFrame;
use crate::heap::{ClassId, Heap, ObjRef};
use crate::interner::{Interner, Symbol};
use crate::universe::Universe;
use crate::value::Value;
use std::time::Instant;
use std::{collections::BTreeMap, collections::HashMap, collections::VecDeque};
use indexmap::IndexMap;
use phalcom_common::range::SourceRange;

/// Maximum depth of the `.ph` call-frame stack before
/// [`RuntimeError::DepthExceeded`](crate::error::RuntimeError::DepthExceeded).
///
/// Set well above Python's default of 1000 and Ruby's comparable ceiling, because
/// Phalcom's message-send idiom is frame-hungry — a `for` body already costs several
/// `.ph` frames per element, so a limit tuned for a C-style language would reject
/// ordinary code.
///
/// Raising this is *safe* in a way it is not for most interpreters: frames live in a
/// heap `Vec`, not on the Rust stack, so a higher ceiling cannot turn a clean error
/// into a segfault. CPython's `sys.setrecursionlimit` is a footgun precisely because
/// its frames still consume C stack; Phalcom's cannot. That is why this ceiling is
/// the generous one and [`MAX_NATIVE_REENTRY`] is not
/// ([PDR-0007](../../docs/decisions/0007-bounded-call-depth-and-native-reentrancy.md) §1).
pub const MAX_CALL_DEPTH: usize = 10_000;

/// Maximum nesting of re-entrant native `run_until` calls before
/// [`RuntimeError::DepthExceeded`](crate::error::RuntimeError::DepthExceeded).
///
/// Far below [`MAX_CALL_DEPTH`] because each native re-entry consumes a real
/// **Rust** stack frame. Overflowing that aborts the process — an abort
/// `catch_unwind` cannot intercept — so this ceiling is the one that must stay
/// conservative, and it is deliberately not user-adjustable (PDR-0007 §3).
///
/// **Measured, not guessed.** PDR-0007 originally proposed 200. On a standard 2 MiB
/// thread stack (what Rust gives test and spawned threads, as opposed to the main
/// thread's 8 MiB) that still aborts: 128 aborts, 64 and 32 survive. 32 is taken to
/// leave margin for deeper interpreter frames and for embedders with smaller stacks.
/// A ceiling that only holds on the main thread does not protect the resource it
/// exists to protect.
///
/// Reached through `send_dynamic`, `block_call`, and any primitive that drives the
/// dispatch loop to get a synchronous value back. Note a `doesNotUnderstand` chain
/// is *not* on this path — it pushes ordinary `.ph` frames and trips
/// [`MAX_CALL_DEPTH`] instead.
pub const MAX_NATIVE_REENTRY: usize = 32;

/// Identity of a class: the module that declares it, plus its name.
///
/// Class *bindings* have always been module-scoped (`VM::define_global`
/// writes into the module object's own globals), but class *identity* was
/// keyed by bare name VM-wide, so two modules declaring the same class name
/// silently collapsed into one class
/// ([PDR-0001](../../../docs/decisions/0001-classes-are-closed.md)
/// ruling 1). This key restores the symmetry: since file = module
/// (ADR-0045), "same module" and "same file" are the same check.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ClassKey {
    /// The declaring module's [`ModuleObject`](crate::heap::ModuleObject) handle.
    pub module: ObjRef,
    /// The class's name, interned.
    pub name: Symbol,
}

/// Layout description for a compiled class (ADR-0011/ADR-0017).
#[derive(Debug, Clone)]
pub struct ClassLayout {
    pub name: Symbol,
    pub field_slots: IndexMap<Symbol, u16>,
    pub field_count: u16,
    pub static_field_slots: IndexMap<Symbol, u16>,
    pub static_field_count: u16,
    /// Names of this class's own fields (instance or static) declared with
    /// `const` (ADR-0064 §3, U-BINDINGS §5). A write to one of these outside
    /// a constructor body is `field.const_write`. Fields are non-inherited
    /// (ADR-0011), so this set holds only this class's own declarations —
    /// no superclass merge.
    pub const_fields: std::collections::HashSet<Symbol>,
    /// Source range of the `class` declaration that produced this layout.
    ///
    /// Stored by U-CLASSNS, consumed by
    /// [`U-CLASSCLOSE`](../../../docs/forge/units/U-CLASSCLOSE/implementation-spec.md):
    /// PDR-0001 ruling 2's `X is already defined` diagnostic carries **both**
    /// spans, and the *first* declaration's location is otherwise unrecoverable —
    /// no sibling map records it. Dead until unit B lands; that is intentional,
    /// so the struct rewrite happens once.
    pub declared_at: SourceRange,
}

/// The bytecode virtual machine: owns the [`Heap`], the operand stack, and the
/// call stack, and drives dispatch.
///
/// See the module docs for the ownership model
/// ([ADR-0009](../../../docs/adr/0009-handle-arena-heap.md)).
pub struct VM {
    /// The single object heap; all class/instance/method/module/closure/string
    /// storage lives here, keyed by [`ObjRef`].
    pub heap: Heap,
    /// The active call stack, innermost frame last. [`CallFrame`] is `Copy`.
    ///
    /// This is the **live mirror** of the currently-[`Object::Fiber`]-running
    /// fiber's own `frames` buffer ([`crate::heap::FiberObject`],
    /// [ADR-0030](../../../docs/adr/0030-fibers-and-futures-cooperative-concurrency.md)
    /// §3, D-FIB-4): while [`Self::current`] runs, its state lives here; a
    /// fiber switch stores this back into the parking fiber and loads the
    /// resuming fiber's state in, an O(1) pointer-free copy (a `Vec` swap).
    pub(crate) frames: Vec<CallFrame>,
    /// The operand stack of `Copy` [`Value`]s — the live mirror of
    /// [`Self::current`]'s stack, mirroring [`Self::frames`] (see its doc).
    pub(crate) stack: Vec<Value>,
    /// The currently-running [`crate::heap::Object::Fiber`]
    /// ([ADR-0030](../../../docs/adr/0030-fibers-and-futures-cooperative-concurrency.md)
    /// §2–§3). [`Self::frames`]/[`Self::stack`]/[`Self::open_upvalues`] are its
    /// live state; a parked (non-current) fiber holds its own state in its
    /// [`crate::heap::FiberObject`] fields instead. Initialized in [`VM::new`]
    /// to a fresh **root** fiber ([`crate::heap::FiberObject::root`]) that
    /// wraps the whole program — every VM has always run "inside a fiber",
    /// this just makes it addressable. `Object::Fiber` is not reached through
    /// any new `Value` arm (D2); this handle is the VM's own bookkeeping.
    pub(crate) current: ObjRef,
    /// Set by a fiber-switch primitive (`fiber_call`/`fiber_try`/
    /// `fiber_yield`) just before it returns, to tell
    /// [`Self::call_method`]'s `Primitive` arm that [`Self::frames`]/
    /// [`Self::stack`] were already repointed to a **different** fiber and
    /// the ordinary post-call stack reconciliation (the `frames.len()`
    /// heuristic) must be skipped — the **typed switch signal**
    /// ([ADR-0030](../../../docs/adr/0030-fibers-and-futures-cooperative-concurrency.md)
    /// §5, D5), replacing that heuristic rather than reusing it. Cleared by
    /// the arm that consumes it. See D-FIB-5: this flag is the "VM flag"
    /// alternative to threading a typed return through every `PrimitiveFn`
    /// (which would touch all ~70 existing primitives) — explicitly sanctioned
    /// by the implementation spec as the pragmatic choice.
    pub(crate) switch_pending: bool,
    /// Nesting depth of **re-entrant native `run_until`** calls currently on
    /// the real Rust call stack (`block_call`, [`Self::send_dynamic`],
    /// [`Self::invoke_method_object`] — every native primitive that
    /// recursively drives the dispatch loop to get a synchronous [`Value`]
    /// back). A fiber switch is legal **only** at depth 0 (ADR-0030 §4's
    /// restricted Option A): a nested re-entrant call's `base_frames` is
    /// computed against the *currently* running fiber's frame count, and a
    /// switch swaps that vector out from under it, so any depth `>0` switch
    /// would corrupt the enclosing re-entrant call's own drain condition.
    /// `Fiber#yield`'s restricted-yield guard and `Fiber#call`/`#try`'s
    /// resume gate both check this against 0.
    pub(crate) native_reentry_depth: usize,
    // Ceilings for both counters live at module scope — see `MAX_CALL_DEPTH` and
    // `MAX_NATIVE_REENTRY` below.

    /// Loaded modules by name [`Symbol`], each a [`ModuleObject`](crate::heap::ModuleObject) handle.
    pub modules: HashMap<Symbol, ObjRef>,
    /// Handle to the program entry module, once known.
    pub main_module: Option<ObjRef>,
    /// Handle to the most recently imported module, for `import` resolution.
    pub last_imported_module: Option<ObjRef>,

    /// Named classes by identity [`ClassKey`], each a [`ClassId`] handle.
    pub classes: HashMap<ClassKey, ClassId>,
    /// Kernel class names reserved to the core module (PDR-0001 ruling
    /// 3, U-CLASSCLOSE §4): exactly the names `VM::install_core`'s
    /// `add_class!` macro binds — the Rust-installed primitives (`Object`,
    /// `List`, `Number`, `Error`, …), **not** every class `core.ph` itself
    /// goes on to declare in `.ph` (`ArgumentError`, `PreconditionError`,
    /// and friends are ordinary core-library classes, module-scoped like
    /// any other — subclassing or shadowing them from a non-core module is
    /// not the trap this ruling guards against, since they carry no
    /// literal-bound [`ClassId`] the way the primitives do).
    ///
    /// Populated by `add_class!` itself as each primitive installs, so this
    /// can never drift from what bootstrap actually binds — no hand-copied
    /// list, and no need to touch this comment when a primitive is added or
    /// removed.
    pub kernel_class_names: std::collections::HashSet<Symbol>,
    /// The symbol interner backing selectors, names and string identity.
    pub interner: Interner,
    /// VM start time, used for `System` timing primitives.
    pub start_time: Instant,
    /// The kernel: handles to the bootstrapped core classes.
    pub universe: Universe,
    /// Monotonically-assigned generation counter for frame tokens.
    pub(crate) next_frame_generation: u64,
    /// Global version counter for inline-cache invalidation.
    ///
    /// Incremented whenever a method is added or replaced on any class.
    /// Each monomorphic inline cache records the `world_version` at the time
    /// its entry was populated; if `world_version` changes, the cache entry
    /// is discarded and the method lookup is re-run (ADR-FUTURE: inline cache).
    pub(crate) world_version: u64,
    /// Live **open** upvalue cells keyed by absolute value-stack index.
    ///
    /// Realizes the Lua-style shared-cell rule
    /// ([ADR-0013](../../../docs/adr/0013-block-closure-upvalues.md)): every
    /// closure capturing the same live local shares one cell here, so mutation
    /// is shared while the slot is on the stack. On frame/scope exit the cell is
    /// promoted to [`Upvalue::Closed`] and removed from this map.
    pub(crate) open_upvalues: BTreeMap<usize, ObjRef>,
    /// FIFO of scheduled-but-not-yet-run fibers ([`crate::primitive::system::system_schedule`]).
    ///
    /// Populated by `System.schedule(_)`; drained by the root-drive pump
    /// ([`VM::run`]) and by any `.ph`-level pump loop (`System.runScheduled`,
    /// `core.ph`) via [`crate::primitive::system::system_next_scheduled`]. A
    /// fiber in this queue has never been resumed
    /// (`FiberObject::started == false`) — draining it resumes it as a fresh
    /// entry call, exactly like `Fiber#call`'s first-resume path
    /// (`primitive/fiber.rs` `fiber_resume`).
    pub(crate) ready_queue: VecDeque<ObjRef>,
    /// Handles a native primitive holds in a Rust local across a **re-entrant
    /// call**, kept reachable for the collector ([ADR-0050](../../../docs/adr/accepted/0050-non-moving-mark-sweep-collector.md) §7).
    ///
    /// The `wrenPushRoot`/`luaL_ref` escape hatch. [`VM::stack`] and
    /// [`VM::frames`] are the complete root truth only for values the
    /// *interpreter* holds; a value living solely in a Rust local of a native
    /// primitive is reachable from nothing. That is safe while the primitive
    /// merely allocates (Invariant L: `Heap::alloc` latches, never collects),
    /// but **not** while it re-enters the interpreter — the back-edge safepoint
    /// inside that re-entrant call collects, and a value held only in Rust is
    /// swept out from under its holder.
    ///
    /// Push before the re-entrant call, pop after it returns, on every path.
    /// See [`VM::push_temp_root`].
    pub(crate) temp_roots: Vec<ObjRef>,
    /// Registered class layouts for slot mapping.
    pub field_layouts: HashMap<ClassKey, ClassLayout>,
    /// Maps a `construct`'s ordinary call-site selector (as an
    /// `Expr::MethodCall` on the class name would encode it, `SignatureKind::Method`)
    /// to the `SignatureKind::Initializer` selector it was actually installed
    /// under, keyed by `(class name, call-site selector)`.
    ///
    /// User source always calls a constructor as an ordinary send
    /// (`Counter.new()`), but `construct` installs under a distinct
    /// `init `-prefixed selector (ADR-0011/`method.rs`) so overloaded
    /// constructors can coexist with an inherited `new` primitive. The
    /// compiler consults this table at the call site (only for a literal
    /// `ClassName.method(...)` receiver) to redirect the emitted selector to
    /// the constructor instead of silently falling through to
    /// `Object::new`'s bare-allocation primitive.
    pub constructor_aliases: HashMap<(ClassKey, Symbol), Symbol>,
    /// Classes (by [`ClassKey`]) that declare at least one `construct new(...)`.
    ///
    /// Once a class opts into a `new`-named constructor, it has no
    /// user-visible bare allocator (U7-plan §3/§6): a call-site `new(...)`
    /// whose arity/labels match none of the class's declared constructors is
    /// a compile error rather than a silent fall-through to the inherited
    /// `Object::new` primitive.
    pub has_new_construct: std::collections::HashSet<ClassKey>,
    /// Compile-time superclass edges: a class (by [`ClassKey`]) mapped
    /// to the [`ClassKey`] of its `extends` superclass.
    ///
    /// Populated by the compiler as each `class B extends A { … }` is lowered
    /// (the superclass is required to be defined earlier in the same pass, so
    /// the edge is always known here). Lets constructor-guard and
    /// constructor-alias lookups walk the inheritance chain at compile time —
    /// so a subclass that *inherits* a `new`-named `construct` but declares
    /// none still has no user-visible bare allocator, and an inherited
    /// `construct` is redirected to its `Initializer` selector at a subclass
    /// call site exactly as a locally declared one is (U-INH follow-on;
    /// `docs/forge/DEFERRED.md` correctness entry). Only user classes appear;
    /// the implicit `Object` root is absent (chain-walks terminate there).
    pub class_parents: HashMap<ClassKey, ClassKey>,
    /// Classes (by [`ClassKey`]) declared `@sealed` (U-ANNOT-LAYOUT §3.4,
    /// `annotations-data.md` §"`@sealed`"), mapped to the compiling
    /// [`ModuleObject`](crate::heap::ModuleObject) handle (`ObjRef`) whose
    /// `Compiler::compile` call declared them — the "compile-unit-scoped"
    /// tracking table `@sealed`'s per-unit closed-world check needs (its own
    /// section: "the compiler finishes processing the compilation unit...
    /// no further subclass may be declared").
    ///
    /// A module `ObjRef` is a natural, already-unique-per-compile-unit
    /// identifier — `Compiler::new(vm, module)` is constructed fresh for
    /// every top-level `.ph` file compile (including every `import`-loaded
    /// module, `compiler::lib::mod::compile_import`), so two classes sharing
    /// a module handle were declared in the same unit, and two sharing a
    /// name but different module handles were not. Checked at *subclass*
    /// compile time (`compiler::lib::class_decl::Compiler::compile_class`),
    /// immediately after the subclass's superclass is resolved — the
    /// existing single-pass top-down discipline (a superclass must already
    /// be defined earlier in its own unit, U-INH) guarantees a sealed
    /// class's entry here is always populated before any of its same-unit
    /// subclasses compile, so an immediate check is equivalent to (and
    /// simpler than) deferring to a dedicated end-of-unit pass.
    pub sealed_classes: HashMap<ClassKey, ObjRef>,
    /// Live mirror of [`Self::current`]'s
    /// [`FiberObject::checking`](crate::heap::FiberObject::checking) — the
    /// identity set of receivers currently under `@invariant`
    /// re-entrancy-guard checking (U-ANNOT-CONTRACTS,
    /// [ADR-0052](../../../docs/adr/0052-invariant-reentrancy-scope-and-layout-confined-decorator-state.md)
    /// Fix 1). Mutated only by the native `__invariantEnter`/
    /// `__invariantExit` primitives on `Object`; swapped with the parking/
    /// resuming fiber's own `checking` set alongside `frames`/`stack`/
    /// `open_upvalues` on every fiber switch (`primitive/fiber.rs`
    /// `store_live_into`/`load_live_from`).
    pub(crate) checking: std::collections::HashSet<ObjRef>,
    /// The active contract [`CompileMode`](crate::compiler::attributes::CompileMode)
    /// (U-ANNOT-CONTRACTS plan §3.6), selected by the `phalcom` CLI's
    /// `--release`/`--unchecked` flags (default `Debug`) and read by
    /// `Compiler::compile_class` (`compiler::lib`) when it builds each
    /// class's [`crate::compiler::attributes::ExpandCtx`].
    pub compile_mode: crate::compiler::attributes::CompileMode,
    /// Mirrors the CLI's `--strip-contract-metadata` flag: forces reflectable
    /// contract metadata (`MethodObject::contracts`, plan §3.5) to be stripped
    /// even in `Release` mode, where it is retained by default. Has no effect
    /// in `Unchecked` mode, where metadata is already stripped by default
    /// (plan §3.6's table) — never coupled to [`Self::compile_mode`] beyond
    /// this one opt-in-to-stripping override.
    pub strip_contract_metadata: bool,
    /// Whether the active compilation/execution unit is a whole file or a single REPL cell.
    pub unit_kind: crate::compiler::lib::UnitKind,
    /// `sel` → the interned `<name>(*)` variadic selector, or `None` when `sel`
    /// is not variadic-probe-eligible (labelled/getter/setter/subscript)
    /// (U-HOTPATH). Memoizes the eligibility decision and derived selector
    /// construction to save `decode_selector` allocations.
    pub(crate) variadic_selector_cache: HashMap<Symbol, Option<Symbol>>,
    /// Bounded free-list for recycling fiber stacks/frames to avoid
    /// allocations (U-GC step 5, `fiber-pool` feature). Measured net
    /// negative in whole-process A/B benchmarking (perf-log, 2026-07-14);
    /// gated off by default, kept for future re-measurement.
    #[cfg(feature = "fiber-pool")]
    pub(crate) fiber_pool: Vec<(Vec<Value>, Vec<CallFrame>)>,
}

impl Default for VM {
    /// Delegates to [`VM::new`] — a `VM` has exactly one valid initial state
    /// (the bootstrapped kernel tower), so `Default` and `new` coincide.
    fn default() -> Self {
        Self::new()
    }
}
