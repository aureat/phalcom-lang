//! The central object [`Heap`]: arena storage keyed by `Copy` generational handles.
//!
//! Realizes [ADR-0009](../../../docs/adr/0009-handle-arena-heap.md). Every heap
//! object (instances, classes, methods, modules, closures and strings) lives in
//! one [`Heap`] and is referred to by an [`ObjRef`] — a small `Copy` handle, not
//! a pointer. Dereferencing goes through the heap ([`Heap::get`] and the typed
//! accessors), so the cyclic kernel graph (a metaclass that is an instance of
//! itself, [`crate::universe`]) is expressed as handles that point at each other
//! with no ownership paradox and no `Rc<RefCell<T>>` borrow-panic surface.
//!
//! ## Why `slotmap`
//!
//! The arena is a [`slotmap::SlotMap`]. `slotmap` gives us exactly the shape
//! [ADR-0009](../../../docs/adr/0009-handle-arena-heap.md) asks for with **zero
//! `unsafe`** in this crate:
//!
//! - keys ([`ObjRef`]) are `Copy` and generational, so a stale handle resolves
//!   to a clean `None` rather than undefined behavior (no use-after-free);
//! - interior mutability lives here, in the arena, instead of in a per-object
//!   `RefCell`, which removes the double-borrow panic hazard entirely;
//! - the stable-key design leaves room for a future tracing collector to
//!   relocate or reclaim entries behind the same [`ObjRef`] surface.
//!
//! `generational-arena` was the fallback; `slotmap` was chosen for its richer
//! typed-key ergonomics and `no unsafe`-at-the-call-site guarantee.
//!
//! NaN-boxing of [`crate::value::Value`] stays deferred behind this API
//! ([ADR-0010](../../../docs/adr/0010-tagged-value-enum.md)); it does not affect
//! the heap contract.

use slotmap::{new_key_type, SlotMap};
use std::collections::BTreeMap;

use crate::block::BlockObject;
use crate::class::ClassObject;
use crate::closure::ClosureObject;
use crate::frame::CallFrame;
use crate::instance::InstanceObject;
use crate::list::ListObject;
use crate::method::MethodObject;
use crate::module::ModuleObject;
use crate::string::StringObject;
use crate::upvalue::Upvalue;
use crate::value::Value;

new_key_type! {
    /// A `Copy` generational handle to an [`Object`] stored in the [`Heap`].
    ///
    /// An `ObjRef` is an index-plus-generation into the arena, **not** a
    /// pointer. It is cheap to copy, hash and compare, and comparing two
    /// `ObjRef`s tests *object identity*. Resolve it through the heap
    /// ([`Heap::get`] / [`Heap::class`] / …). Realizes
    /// [ADR-0009](../../../docs/adr/0009-handle-arena-heap.md).
    pub struct ObjRef;
}

/// An [`ObjRef`] whose referent is statically intended to be a [`ClassObject`].
///
/// This is a documentation alias — it sharpens intent at class-typed fields and
/// signatures without introducing a distinct key type. Resolve it with
/// [`Heap::class`] / [`Heap::class_mut`].
pub type ClassId = ObjRef;

/// The tagged payload stored at each live [`ObjRef`] in the [`Heap`].
///
/// Every heap-allocated Phalcom object is one of these variants. Immediate
/// values (`nil`, booleans, numbers, interned symbols) are *not* here — they
/// live inline in [`crate::value::Value`] per
/// [ADR-0010](../../../docs/adr/0010-tagged-value-enum.md).
pub enum Object {
    /// A user-defined object with per-instance fields ([`InstanceObject`]).
    Instance(InstanceObject),
    /// A class or metaclass row in the tower ([`ClassObject`]).
    Class(ClassObject),
    /// A method — primitive or bytecode closure ([`MethodObject`]).
    Method(MethodObject),
    /// A loaded module and its global slots ([`ModuleObject`]).
    Module(ModuleObject),
    /// A compiled closure over a [`crate::callable::Callable`] ([`ClosureObject`]).
    Closure(ClosureObject),
    /// An immutable interned-by-content string ([`StringObject`]).
    Str(StringObject),
    /// A first-class block closure ([`BlockObject`]).
    Block(BlockObject),
    /// A method closed over a receiver — the result of `Method#bind(_)`
    /// ([ADR-0006](../../../docs/adr/0006-function-as-abstract-callable-root.md)).
    /// Its surface class is `Block`; it responds to the `Function` call
    /// protocol by delegating to [`crate::vm::VM::invoke_method_object`]
    /// (U-CORE-3, [ADR-0028](../../../docs/adr/0028-amend-floor-admit-method-reflection.md)).
    BoundMethod(BoundMethodObject),
    /// A heap-allocated upvalue cell ([`Upvalue`]).
    Upvalue(Upvalue),
    /// A native array-backed list ([`ListObject`],
    /// [ADR-0020](../../../docs/adr/0020-kernel-list-native-array-protocol.md)).
    List(ListObject),
    /// A cooperative fiber — the sole concurrency primitive
    /// ([`FiberObject`], [ADR-0030](../../../docs/adr/0030-fibers-and-futures-cooperative-concurrency.md) §2).
    ///
    /// Reached through [`Value::Obj`] exactly as a
    /// [`Object::List`] is — there is **no** `Value::Fiber` arm (ADR-0030 §2,
    /// forward-compat §7 D2). It owns its own value + call stacks so it can be
    /// suspended and resumed by an O(1) pointer swap of `vm.current`.
    Fiber(FiberObject),
}

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
    /// The entry [`Object::Block`]/[`Object::Closure`] the fiber runs on first
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
}

impl FiberObject {
    /// Builds a fresh, not-yet-started fiber wrapping `entry` (its
    /// [`Object::Block`]/[`Object::Closure`] entry), status
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
        }
    }
}

/// The payload of an [`Object::BoundMethod`] — a reified [`MethodObject`]
/// closed over an explicit receiver, the runtime value
/// [`crate::primitive::method::method_bind`] (`Method#bind(_)`, U-CORE-3)
/// constructs.
///
/// Unlike [`BlockObject`], a `BoundMethodObject` carries no closure or
/// home-frame token: it must work for **primitive** methods too (which have
/// no [`ClosureObject`]), and it is not itself a lexical block, so it has no
/// non-local return and introduces no frame-indexing
/// ([ADR-0028](../../../docs/adr/0028-amend-floor-admit-method-reflection.md)
/// forward-compat §1). Calling it (`bound.call(args)`) and
/// `method.invokeOn(receiver, args)` funnel through the same
/// [`crate::vm::VM::invoke_method_object`] workhorse, so the two are
/// identical by construction (R-INV-3.3).
#[derive(Debug, Clone, Copy)]
pub struct BoundMethodObject {
    /// The wrapped [`Object::Method`] handle.
    pub method: ObjRef,
    /// The receiver this method is closed over.
    pub receiver: Value,
}

/// The central arena owning every heap [`Object`], keyed by [`ObjRef`].
///
/// The [`crate::vm::VM`] owns exactly one `Heap`. Methods that historically
/// called `self.borrow()` / `self.borrow_mut()` now take `&Heap` / `&mut Heap`
/// and dereference a handle through it. Realizes
/// [ADR-0009](../../../docs/adr/0009-handle-arena-heap.md).
#[derive(Default)]
pub struct Heap {
    /// Backing arena. Generational keys make stale handles resolve to `None`.
    objects: SlotMap<ObjRef, Object>,
}

impl Heap {
    /// Creates an empty heap.
    pub fn new() -> Self {
        Self { objects: SlotMap::with_key() }
    }

    /// Allocates `object` and returns its fresh [`ObjRef`].
    pub fn alloc(&mut self, object: Object) -> ObjRef {
        self.objects.insert(object)
    }

    /// Allocates a [`ClassObject`] and returns its [`ClassId`].
    pub fn alloc_class(&mut self, class: ClassObject) -> ClassId {
        self.objects.insert(Object::Class(class))
    }

    /// Allocates a [`StringObject`] from `value` and returns its [`ObjRef`].
    pub fn alloc_string(&mut self, value: String) -> ObjRef {
        self.objects.insert(Object::Str(StringObject::from_string(value)))
    }

    /// Allocates a [`ListObject`] from `elements` and returns its [`ObjRef`].
    pub fn alloc_list(&mut self, elements: Vec<crate::value::Value>) -> ObjRef {
        self.objects.insert(Object::List(ListObject::new(elements)))
    }

    /// Borrows the [`Object`] behind `id`.
    ///
    /// # Panics
    ///
    /// Panics if `id` is stale or was never allocated in this heap.
    pub fn get(&self, id: ObjRef) -> &Object {
        self.objects.get(id).unwrap_or_else(|| panic!("dangling ObjRef {id:?}"))
    }

    /// Mutably borrows the [`Object`] behind `id`.
    ///
    /// # Panics
    ///
    /// Panics if `id` is stale or was never allocated in this heap.
    pub fn get_mut(&mut self, id: ObjRef) -> &mut Object {
        self.objects.get_mut(id).unwrap_or_else(|| panic!("dangling ObjRef {id:?}"))
    }

    /// Borrows the [`ClassObject`] behind `id`.
    ///
    /// # Panics
    ///
    /// Panics if `id` is stale or does not refer to an [`Object::Class`].
    pub fn class(&self, id: ClassId) -> &ClassObject {
        match self.get(id) {
            Object::Class(class) => class,
            _ => panic!("ObjRef {id:?} is not a ClassObject"),
        }
    }

    /// Mutably borrows the [`ClassObject`] behind `id`.
    ///
    /// # Panics
    ///
    /// Panics if `id` is stale or does not refer to an [`Object::Class`].
    pub fn class_mut(&mut self, id: ClassId) -> &mut ClassObject {
        match self.get_mut(id) {
            Object::Class(class) => class,
            _ => panic!("ObjRef {id:?} is not a ClassObject"),
        }
    }

    /// Returns the [`ClassObject`] behind `id`, or `None` if `id` is not a class.
    pub fn as_class(&self, id: ObjRef) -> Option<&ClassObject> {
        match self.objects.get(id) {
            Some(Object::Class(class)) => Some(class),
            _ => None,
        }
    }

    /// Borrows the [`InstanceObject`] behind `id`.
    ///
    /// # Panics
    ///
    /// Panics if `id` is stale or does not refer to an [`Object::Instance`].
    pub fn instance(&self, id: ObjRef) -> &InstanceObject {
        match self.get(id) {
            Object::Instance(instance) => instance,
            _ => panic!("ObjRef {id:?} is not an InstanceObject"),
        }
    }

    /// Mutably borrows the [`InstanceObject`] behind `id`.
    ///
    /// # Panics
    ///
    /// Panics if `id` is stale or does not refer to an [`Object::Instance`].
    pub fn instance_mut(&mut self, id: ObjRef) -> &mut InstanceObject {
        match self.get_mut(id) {
            Object::Instance(instance) => instance,
            _ => panic!("ObjRef {id:?} is not an InstanceObject"),
        }
    }

    /// Returns the [`InstanceObject`] behind `id`, or `None` if it is not one.
    pub fn as_instance(&self, id: ObjRef) -> Option<&InstanceObject> {
        match self.objects.get(id) {
            Some(Object::Instance(instance)) => Some(instance),
            _ => None,
        }
    }

    /// Borrows the [`MethodObject`] behind `id`.
    ///
    /// # Panics
    ///
    /// Panics if `id` is stale or does not refer to an [`Object::Method`].
    pub fn method(&self, id: ObjRef) -> &MethodObject {
        match self.get(id) {
            Object::Method(method) => method,
            _ => panic!("ObjRef {id:?} is not a MethodObject"),
        }
    }

    /// Mutably borrows the [`MethodObject`] behind `id`.
    ///
    /// # Panics
    ///
    /// Panics if `id` is stale or does not refer to an [`Object::Method`].
    pub fn method_mut(&mut self, id: ObjRef) -> &mut MethodObject {
        match self.get_mut(id) {
            Object::Method(method) => method,
            _ => panic!("ObjRef {id:?} is not a MethodObject"),
        }
    }

    /// Borrows the [`ModuleObject`] behind `id`.
    ///
    /// # Panics
    ///
    /// Panics if `id` is stale or does not refer to an [`Object::Module`].
    pub fn module(&self, id: ObjRef) -> &ModuleObject {
        match self.get(id) {
            Object::Module(module) => module,
            _ => panic!("ObjRef {id:?} is not a ModuleObject"),
        }
    }

    /// Mutably borrows the [`ModuleObject`] behind `id`.
    ///
    /// # Panics
    ///
    /// Panics if `id` is stale or does not refer to an [`Object::Module`].
    pub fn module_mut(&mut self, id: ObjRef) -> &mut ModuleObject {
        match self.get_mut(id) {
            Object::Module(module) => module,
            _ => panic!("ObjRef {id:?} is not a ModuleObject"),
        }
    }

    /// Borrows the [`ClosureObject`] behind `id`.
    ///
    /// # Panics
    ///
    /// Panics if `id` is stale or does not refer to an [`Object::Closure`].
    pub fn closure(&self, id: ObjRef) -> &ClosureObject {
        match self.get(id) {
            Object::Closure(closure) => closure,
            _ => panic!("ObjRef {id:?} is not a ClosureObject"),
        }
    }

    /// Borrows the [`StringObject`] behind `id`.
    ///
    /// # Panics
    ///
    /// Panics if `id` is stale or does not refer to an [`Object::Str`].
    pub fn string(&self, id: ObjRef) -> &StringObject {
        match self.get(id) {
            Object::Str(string) => string,
            _ => panic!("ObjRef {id:?} is not a StringObject"),
        }
    }

    /// Returns the [`StringObject`] behind `id`, or `None` if it is not one.
    pub fn as_string(&self, id: ObjRef) -> Option<&StringObject> {
        match self.objects.get(id) {
            Some(Object::Str(string)) => Some(string),
            _ => None,
        }
    }

    /// Borrows the [`ListObject`] behind `id`.
    ///
    /// # Panics
    ///
    /// Panics if `id` is stale or does not refer to an [`Object::List`].
    pub fn list(&self, id: ObjRef) -> &ListObject {
        match self.get(id) {
            Object::List(list) => list,
            _ => panic!("ObjRef {id:?} is not a ListObject"),
        }
    }

    /// Mutably borrows the [`ListObject`] behind `id`.
    ///
    /// # Panics
    ///
    /// Panics if `id` is stale or does not refer to an [`Object::List`].
    pub fn list_mut(&mut self, id: ObjRef) -> &mut ListObject {
        match self.get_mut(id) {
            Object::List(list) => list,
            _ => panic!("ObjRef {id:?} is not a ListObject"),
        }
    }

    /// Returns the [`ListObject`] behind `id`, or `None` if it is not one.
    pub fn as_list(&self, id: ObjRef) -> Option<&ListObject> {
        match self.objects.get(id) {
            Some(Object::List(list)) => Some(list),
            _ => None,
        }
    }

    /// Borrows the [`FiberObject`] behind `id`
    /// ([ADR-0030](../../../docs/adr/0030-fibers-and-futures-cooperative-concurrency.md) §2).
    ///
    /// # Panics
    ///
    /// Panics if `id` is stale or does not refer to an [`Object::Fiber`].
    pub fn fiber(&self, id: ObjRef) -> &FiberObject {
        match self.get(id) {
            Object::Fiber(fiber) => fiber,
            _ => panic!("ObjRef {id:?} is not a FiberObject"),
        }
    }

    /// Mutably borrows the [`FiberObject`] behind `id`
    /// ([ADR-0030](../../../docs/adr/0030-fibers-and-futures-cooperative-concurrency.md) §2).
    ///
    /// # Panics
    ///
    /// Panics if `id` is stale or does not refer to an [`Object::Fiber`].
    pub fn fiber_mut(&mut self, id: ObjRef) -> &mut FiberObject {
        match self.get_mut(id) {
            Object::Fiber(fiber) => fiber,
            _ => panic!("ObjRef {id:?} is not a FiberObject"),
        }
    }

    /// Returns the [`FiberObject`] behind `id`, or `None` if it is not one
    /// ([ADR-0030](../../../docs/adr/0030-fibers-and-futures-cooperative-concurrency.md) §2).
    pub fn as_fiber(&self, id: ObjRef) -> Option<&FiberObject> {
        match self.objects.get(id) {
            Some(Object::Fiber(fiber)) => Some(fiber),
            _ => None,
        }
    }

    /// Borrows the [`BlockObject`] behind `id`.
    ///
    /// # Panics
    ///
    /// Panics if `id` is stale or does not refer to an [`Object::Block`].
    pub fn block(&self, id: ObjRef) -> &BlockObject {
        match self.get(id) {
            Object::Block(block) => block,
            _ => panic!("ObjRef {id:?} is not a BlockObject"),
        }
    }

    /// Mutably borrows the [`BlockObject`] behind `id`.
    ///
    /// # Panics
    ///
    /// Panics if `id` is stale or does not refer to an [`Object::Block`].
    pub fn block_mut(&mut self, id: ObjRef) -> &mut BlockObject {
        match self.get_mut(id) {
            Object::Block(block) => block,
            _ => panic!("ObjRef {id:?} is not a BlockObject"),
        }
    }

    /// Borrows the [`BoundMethodObject`] behind `id`.
    ///
    /// # Panics
    ///
    /// Panics if `id` is stale or does not refer to an [`Object::BoundMethod`].
    pub fn bound_method(&self, id: ObjRef) -> &BoundMethodObject {
        match self.get(id) {
            Object::BoundMethod(bound) => bound,
            _ => panic!("ObjRef {id:?} is not a BoundMethodObject"),
        }
    }

    /// Borrows the [`Upvalue`] behind `id`.
    ///
    /// # Panics
    ///
    /// Panics if `id` is stale or does not refer to an [`Object::Upvalue`].
    pub fn upvalue(&self, id: ObjRef) -> &Upvalue {
        match self.get(id) {
            Object::Upvalue(upvalue) => upvalue,
            _ => panic!("ObjRef {id:?} is not an Upvalue"),
        }
    }

    /// Mutably borrows the [`Upvalue`] behind `id`.
    ///
    /// # Panics
    ///
    /// Panics if `id` is stale or does not refer to an [`Object::Upvalue`].
    pub fn upvalue_mut(&mut self, id: ObjRef) -> &mut Upvalue {
        match self.get_mut(id) {
            Object::Upvalue(upvalue) => upvalue,
            _ => panic!("ObjRef {id:?} is not an Upvalue"),
        }
    }
}
