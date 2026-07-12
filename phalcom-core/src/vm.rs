//! The bytecode virtual machine: dispatch loop, call stack, and heap ownership.
//!
//! The [`VM`] owns exactly one [`Heap`] ([ADR-0009](../../../docs/adr/0009-handle-arena-heap.md)):
//! every class, instance, method, module, closure and string lives there and is
//! reached through a `Copy` [`ObjRef`] handle rather than an `Rc<RefCell<T>>`.
//! Values on the operand stack are `Copy` [`Value`]s
//! ([ADR-0010](../../../docs/adr/0010-tagged-value-enum.md)); call frames
//! ([`CallFrame`]) are `Copy` too, so the interpreter carries no borrow-panic
//! surface. Method lookup keys on signature symbols (`object-model.md` §3).

use crate::block::BlockObject;
use crate::boolean::{FALSE, TRUE};
use crate::bytecode::Bytecode;
use crate::class::ClassObject;
use crate::closure::ClosureObject;
use crate::diagnostics::{print_rt, SourceLoc, SOURCE_MAP};
use crate::error::{PhError, PhResult, RuntimeError};
use crate::frame::{CallContext, CallFrame};
use crate::heap::{ClassId, Heap, ObjRef, Object};
use crate::interner::{Interner, Symbol};
use crate::method::{MethodKind, SignatureKind, decode_selector};
use crate::module::{ModuleObject, CORE_MODULE_NAME};
use crate::universe::Universe;
use crate::upvalue::Upvalue;
use crate::value::Value;
use phalcom_common::range::SourceRange;
use std::time::Instant;
use std::{collections::BTreeMap, collections::HashMap, sync::Arc};
use tracing::{debug, span, Level};
use indexmap::IndexMap;

/// Layout description for a compiled class (ADR-0011/ADR-0017).
#[derive(Debug, Clone)]
pub struct ClassLayout {
    pub name: Symbol,
    pub field_slots: IndexMap<Symbol, u16>,
    pub field_count: u16,
    pub static_field_slots: IndexMap<Symbol, u16>,
    pub static_field_count: u16,
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
    pub(crate) frames: Vec<CallFrame>,
    /// The operand stack of `Copy` [`Value`]s.
    pub(crate) stack: Vec<Value>,

    /// Loaded modules by name [`Symbol`], each a [`ModuleObject`] handle.
    pub modules: HashMap<Symbol, ObjRef>,
    /// Handle to the program entry module, once known.
    pub main_module: Option<ObjRef>,
    /// Handle to the most recently imported module, for `import` resolution.
    pub last_imported_module: Option<ObjRef>,

    /// Named classes by name [`Symbol`], each a [`ClassId`] handle.
    pub classes: HashMap<Symbol, ClassId>,
    /// The symbol interner backing selectors, names and string identity.
    pub interner: Interner,
    /// VM start time, used for `System` timing primitives.
    pub start_time: Instant,
    /// The kernel: handles to the bootstrapped core classes.
    pub universe: Universe,
    /// Monotonically-assigned generation counter for frame tokens.
    pub(crate) next_frame_generation: u64,
    /// Live **open** upvalue cells keyed by absolute value-stack index.
    ///
    /// Realizes the Lua-style shared-cell rule
    /// ([ADR-0013](../../../docs/adr/0013-block-closure-upvalues.md)): every
    /// closure capturing the same live local shares one cell here, so mutation
    /// is shared while the slot is on the stack. On frame/scope exit the cell is
    /// promoted to [`Upvalue::Closed`] and removed from this map.
    pub(crate) open_upvalues: BTreeMap<usize, ObjRef>,
    /// Registered class layouts for slot mapping.
    pub field_layouts: HashMap<Symbol, ClassLayout>,
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
    pub constructor_aliases: HashMap<(Symbol, Symbol), Symbol>,
    /// Class names (by [`Symbol`]) that declare at least one `construct new(...)`.
    ///
    /// Once a class opts into a `new`-named constructor, it has no
    /// user-visible bare allocator (U7-plan §3/§6): a call-site `new(...)`
    /// whose arity/labels match none of the class's declared constructors is
    /// a compile error rather than a silent fall-through to the inherited
    /// `Object::new` primitive.
    pub has_new_construct: std::collections::HashSet<Symbol>,
}

impl Default for VM {
    fn default() -> Self {
        todo!()
    }
}

impl VM {
    /// Creates a new VM: builds the heap, bootstraps the kernel tower, and
    /// installs the core module and native primitives.
    pub fn new() -> Self {
        let interner = Interner::with_capacity(100);
        let mut heap = Heap::new();
        let universe = Universe::new(&mut heap);

        let mut vm = Self {
            heap,
            frames: Vec::with_capacity(256),
            stack: Vec::with_capacity(1024),
            interner,
            start_time: Instant::now(),
            modules: HashMap::new(),
            main_module: None,
            last_imported_module: None,
            classes: HashMap::new(),
            universe,
            next_frame_generation: 0,
            open_upvalues: BTreeMap::new(),
            field_layouts: HashMap::new(),
            constructor_aliases: HashMap::new(),
            has_new_construct: std::collections::HashSet::new(),
        };

        // Bootstrap core module and primitive methods
        vm.install_core();

        // Initialize Some class field layout (ADR-0011)
        {
            let some_class = vm.universe.classes.some_class;
            let value_sym = vm.interner.intern("_value");
            vm.heap.class_mut(some_class).field_slots.insert(value_sym, 0);
            vm.heap.class_mut(some_class).field_count = 1;
        }

        // Stamp the kernel `Message` class's fixed-slot count (U8,
        // method-lookup.md §2). Like `Some`, `Message` instances are built
        // directly in Rust ([`VM::new_message`]) — its four slots
        // (selector/name/labels/args) carry no `.ph` field layout, so the
        // count is set here rather than by the compiler's class lowering.
        {
            let message_class = vm.universe.classes.message_class;
            vm.heap.class_mut(message_class).field_count = 4;
        }
        Universe::install_primitives(&mut vm);

        // Compile and run the registered core module now that every native
        // primitive is installed: this is what actually attaches each
        // `core.ph` class-reopen (`List`, `Option`, `Some`, `None`, `System`,
        // …) to its bootstrapped kernel row. Must run after
        // `install_primitives` so a reopen can call the primitives it wraps
        // (e.g. `List.at(_:)` calling `rawAt(_:)`). Previously `install_core`
        // only registered the source text (for diagnostics) without ever
        // compiling or executing it, so every `.ph` skeleton was inert; U-LIST
        // is the first unit whose surface protocol actually depends on a
        // reopen taking effect, which surfaced the gap.
        vm.run_core_module().expect("core module (core.ph) must compile and run cleanly");

        // R-INV-0.3 (global half) — the `None` **global** resolves to the shared
        // singleton *value*, not the `None` class object (ADR-0007/0010). This
        // half needs the core module (its globals table), so it lives here rather
        // than in `verify_invariants`, which is heap-structural (`&Heap` only) and
        // cannot read module globals (U-CORE-1 spec SD-1).
        {
            let core = vm.get_module_from_str(CORE_MODULE_NAME).expect("core module registered by install_core");
            let none_sym = vm.interner.intern("None");
            let none_value = vm.heap.module(core).get(none_sym).expect("None global must be bound by install_core");
            assert!(matches!(none_value, Value::Obj(id) if id == vm.universe.classes.none_singleton), "None global must resolve to the shared singleton value, not the None class");
            assert_ne!(none_value, Value::Obj(vm.universe.classes.none_class), "None global must not resolve to the None class object");
        }

        vm.universe
            .verify_invariants(&vm.heap)
            .expect("kernel invariants (object-model.md §5-6)");

        vm
    }

    /// Compiles and runs the registered core module (`core.ph`).
    ///
    /// See the call site in [`Self::new`] for why this must run after
    /// [`Universe::install_primitives`].
    ///
    /// # Errors
    ///
    /// Returns any [`PhError`] raised while compiling or executing `core.ph`.
    fn run_core_module(&mut self) -> PhResult<()> {
        let module = self.get_module_from_str(CORE_MODULE_NAME).expect("core module registered by install_core");
        let source = include_str!("../core/core.ph");
        let closure = self.compile_closure(module, source)?;
        self.run_in_module(module, closure)
    }

    /// Interns `name`, returning its [`Symbol`].
    pub fn get_or_intern(&mut self, name: &str) -> Symbol {
        self.interner.intern(name)
    }

    /// Resolves `symbol` back to its interned string.
    pub fn resolve_symbol(&self, symbol: Symbol) -> &str {
        self.interner.lookup(symbol)
    }

    /// Allocates an immutable string on the heap and returns it as a [`Value`].
    ///
    /// Native primitives that produce strings (e.g. `toString`, string `+`) use
    /// this to move an owned [`String`] into an
    /// [`Object::Str`] and hand back a
    /// [`Value::Obj`] handle ([ADR-0009](../../../docs/adr/0009-handle-arena-heap.md)).
    pub fn alloc_string_value(&mut self, text: String) -> Value {
        Value::Obj(self.heap.alloc_string(text))
    }

    /// Allocates a bare class named `name`, wired only to `superclass`.
    ///
    /// The metaclass link is left unset; callers such as [`Self::create_class`]
    /// patch it. Realizes the allocate-then-patch bootstrap
    /// ([ADR-0009](../../../docs/adr/0009-handle-arena-heap.md)).
    pub fn create_single_class(&mut self, name: &str, superclass: Option<ClassId>) -> ClassId {
        let id = self.heap.alloc_class(ClassObject::bare(name));
        self.heap.class_mut(id).set_superclass(superclass);
        id
    }

    /// Creates a user class `name` with its own metaclass and wires the tower.
    ///
    /// Follows the metaclass parallel rule
    /// ([ADR-0002](../../../docs/adr/0002-metaclass-tower-parallel-rule.md)):
    /// the metaclass `"{name}.class"` is an instance of `Metaclass` whose
    /// superclass is `superclass`'s own metaclass (`Class` if `superclass` is
    /// `None`), and the class itself is an instance of that metaclass with the
    /// requested `superclass`.
    pub fn create_class(&mut self, name: &str, superclass: Option<ClassId>) -> ClassId {
        let metaclass_class = self.universe.classes.metaclass_class;
        let metaclass_superclass = match superclass {
            Some(sc) => self.heap.class(sc).class,
            None => self.universe.classes.class_class,
        };

        let metaclass_name = name.to_owned() + ".class";
        let metaclass = self.create_single_class(&metaclass_name, Some(metaclass_superclass));
        self.heap.class_mut(metaclass).set_class(metaclass_class);

        let class = self.create_single_class(name, superclass);
        self.heap.class_mut(class).set_class(metaclass);

        let name_sym = self.interner.intern(name);
        let meta_sym = self.interner.intern(&metaclass_name);

        if let Some(layout) = self.field_layouts.get(&name_sym).cloned() {
            self.heap.class_mut(class).field_slots = layout.field_slots;
            self.heap.class_mut(class).field_count = layout.field_count;
            self.heap.class_mut(metaclass).field_slots = layout.static_field_slots;
            self.heap.class_mut(metaclass).field_count = layout.static_field_count;
            self.heap.class_mut(class).static_slots = vec![Value::Nil; layout.static_field_count as usize].into_boxed_slice();
        }

        self.classes.insert(name_sym, class);
        self.classes.insert(meta_sym, metaclass);

        class
    }

    /// Allocates a module with `logical_name`/`abs_path` and registers it.
    pub fn create_module(&mut self, logical_name: &str, abs_path: &str) -> ObjRef {
        let module_sym = self.interner.intern(logical_name);
        let module = ModuleObject::new(logical_name.to_string(), module_sym, abs_path.to_string(), None);
        let id = self.heap.alloc(Object::Module(module));
        self.modules.insert(module_sym, id);
        id
    }

    /// Updates the absolute filesystem path of the module named `module_sym`.
    pub fn register_path(&mut self, module_sym: Symbol, abs_path: &str) {
        if let Some(&module_id) = self.modules.get(&module_sym) {
            self.heap.module_mut(module_id).path = abs_path.to_string();
        } else {
            debug!("Module with symbol {:?} not found for path registration", module_sym);
        }
    }

    /// Registers `source` text for the module `logical_name` in the source map.
    pub fn register_source(&mut self, logical_name: &str, source: &str) {
        let source_ref = Arc::new(String::from(source));
        let module_sym = self.interner.intern(logical_name);
        SOURCE_MAP.write().unwrap().insert(module_sym, source_ref.clone());

        let module_sym = self.interner.intern(logical_name);
        let src_ref = Arc::new(String::from(source));
        SOURCE_MAP.write().unwrap().insert(module_sym, src_ref.clone());
    }

    /// Returns the module handle for `module_sym`, if loaded.
    pub fn get_module(&mut self, module_sym: Symbol) -> Option<ObjRef> {
        self.modules.get(&module_sym).copied()
    }

    /// Returns the module handle for the module named `name`, if loaded.
    pub fn get_module_from_str(&mut self, name: &str) -> Option<ObjRef> {
        let sym = self.interner.intern(name);
        self.modules.get(&sym).copied()
    }

    /// Defines global `name_sym = val` in the module `module_sym`.
    ///
    /// # Errors
    ///
    /// Propagates [`ModuleObject::define`](crate::module::ModuleObject::define)
    /// errors (e.g. too many globals).
    pub fn define_global(&mut self, module_sym: Symbol, name_sym: Symbol, val: Value) -> PhResult<usize> {
        let module = self.get_module(module_sym).expect("correct module");
        self.heap.module_mut(module).define(name_sym, val)
    }

    /// Bootstraps the core module and exposes each kernel class as a global.
    pub fn install_core(&mut self) {
        let m = self.create_module(CORE_MODULE_NAME, "<internal core module>");
        self.register_source(CORE_MODULE_NAME, include_str!("../core/core.ph"));
        let core_sym = self.heap.module(m).symbol();
        self.modules.insert(core_sym, m);

        macro_rules! add_class {
            ($field:ident) => {
                let class_id = self.universe.classes.$field;
                let name = self.heap.class(class_id).name.clone();
                let name_sym = self.interner.intern(&name);
                self.define_global(core_sym, name_sym, Value::Obj(class_id)).ok();
                self.classes.insert(name_sym, class_id);
            };
        }

        add_class!(object_class);
        add_class!(behavior_class);
        add_class!(class_class);
        add_class!(metaclass_class);
        add_class!(number_class);
        add_class!(string_class);
        add_class!(bool_class);
        // The boolean tower (ADR-0004): `True`/`False` are ordinary class
        // globals — unlike `None`, their global names bind to the class objects
        // (not to a singleton value), so a `class True {}` / `class False {}`
        // reopen in core.ph re-emits the identical `DefineGlobal` binding (a
        // harmless no-op) and resolves the bootstrapped `self.classes` rows.
        add_class!(true_class);
        add_class!(false_class);
        add_class!(method_class);
        add_class!(symbol_class);
        add_class!(system_class);
        add_class!(function_class);
        add_class!(block_class);
        // Absence type (ADR-0007). `Option` and `Some` are ordinary class
        // globals. `None`, however, is a *value* global bound to the shared
        // singleton — not the `None` class — so `None` in source resolves to the
        // singleton object (values-and-absence.md §3.1).
        add_class!(option_class);
        add_class!(some_class);
        add_class!(list_class);
        add_class!(message_class);

        // The `None` class row is *not* exposed under a class global (that name
        // is the singleton), but it must live in `self.classes` so a
        // `class None { ... }` skeleton in `core.ph` reopens this bootstrapped
        // row (compiler `Statement::Class` handling) instead of forging a fresh
        // one that would clobber the `None` global.
        let none_class = self.universe.classes.none_class;
        let none_class_name = self.heap.class(none_class).name.clone();
        let none_class_sym = self.interner.intern(&none_class_name);
        self.classes.insert(none_class_sym, none_class);

        // Bind the `None` global to the shared singleton object.
        let none_global_sym = self.interner.intern("None");
        let none_value = Value::Obj(self.universe.classes.none_singleton);
        self.define_global(core_sym, none_global_sym, none_value).ok();

        // The private `nil` sentinel has no surface class global: there is no
        // `Nil` name reachable from user code (Invariant 4). The `Nil` class row
        // still exists in the tower to back `Value::Nil::class`, but it is
        // internal only.
    }

    /// Dispatches a call to `method` on `callee` with `arity` arguments.
    ///
    /// A primitive runs its native function in place; a closure pushes a new
    /// [`CallFrame`] to be executed by [`Self::run`].
    ///
    /// # Errors
    ///
    /// Propagates errors returned by a primitive implementation.
    fn call_method(&mut self, callee: &Value, method: ObjRef, arity: usize, source_range: SourceRange) -> PhResult<()> {
        let kind = self.heap.method(method).kind;
        match kind {
            MethodKind::Primitive(native_fn) => {
                let receiver_idx = self.stack.len() - 1 - arity;
                let receiver = self.stack[receiver_idx];
                let args: Vec<Value> = self.stack[receiver_idx + 1..].to_vec();
                // Snapshot the frame count so we can detect a non-local return
                // that fired *inside* `native_fn` (e.g. `block_call` running a
                // block whose `return` unwound past this call site). See the
                // guard below.
                let frames_before = self.frames.len();
                let result = native_fn(self, &receiver, &args);
                result.map(|result| {
                    if self.frames.len() >= frames_before {
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
            MethodKind::Closure(closure_id) => {
                let context = callee.to_context(&self.heap);
                let receiver_idx = self.stack.len() - arity - 1;
                let (variadic, fixed_arity) = {
                    let sig = &self.heap.method(method).signature;
                    (sig.variadic, sig.positional_arity as usize)
                };
                if variadic {
                    // Collect the trailing `arity - fixed_arity` positional args into
                    // a `List` bound to the rest slot. `receiver_idx` does not move —
                    // only the tail past the fixed prefix collapses into one `List`
                    // value, so `stack_offset` below is unaffected (U9,
                    // U9-implementation-spec.md §2 "Call prologue").
                    let rest = self.stack.split_off(receiver_idx + 1 + fixed_arity);
                    let list_id = self.heap.alloc_list(rest);
                    self.stack.push(Value::Obj(list_id));
                }
                let stack_offset = receiver_idx;
                let new_frame = self.new_call_frame(closure_id, context, 0, stack_offset, Some(source_range));
                self.frames.push(new_frame);
                Ok(())
            }
        }
    }

    /// Reifies a message send as a `Message` instance (method-lookup.md §2,
    /// ADR-0012), for the `doesNotUnderstand(_:)` miss path.
    ///
    /// The returned `Message` is an ordinary fixed-slot
    /// [`InstanceObject`](crate::instance::InstanceObject) of the kernel
    /// `Message` class ([`CoreClasses::message_class`](crate::universe::CoreClasses::message_class)),
    /// built directly in Rust (no `.ph` `construct`) with four slots:
    ///
    /// 0. `selector` — the interned [`Symbol`] as sent;
    /// 1. `name` — the bare method name [`String`] (encoder-inverse, `+` for `+(_:)`);
    /// 2. `labels` — a [`List`](crate::list::ListObject) of `String`, one per
    ///    argument, `""` for a positional (unlabeled) argument so that
    ///    `labels.size == args.size` and callers can zip them;
    /// 3. `args` — a [`List`](crate::list::ListObject) of the argument values.
    ///
    /// The `""`-for-positional convention (rather than a separate absence
    /// marker) keeps the two lists index-aligned; it is a deliberate U8 choice,
    /// not spec-pinned.
    pub fn new_message(&mut self, selector: Symbol, args: &[Value]) -> Value {
        let selector_str = self.resolve_symbol(selector).to_string();
        let (name, labels, _kind) = crate::method::decode_selector(&selector_str);

        let name_val = self.alloc_string_value(name);

        // Index-align labels with args: pad or truncate to `args.len()`, using
        // `""` for positional arguments (kinds whose decoded arity differs from
        // the call arity, e.g. subscripts, are made consistent here).
        let mut label_texts: Vec<String> = labels.into_iter().map(|label| label.unwrap_or_default()).collect();
        label_texts.resize(args.len(), String::new());
        let label_values: Vec<Value> = label_texts.into_iter().map(|text| self.alloc_string_value(text)).collect();

        let labels_list = Value::Obj(self.heap.alloc_list(label_values));
        let args_list = Value::Obj(self.heap.alloc_list(args.to_vec()));

        let message_class = self.universe.classes.message_class;
        let mut instance = crate::instance::InstanceObject::new(message_class, 4);
        instance.slots[0] = Value::Symbol(selector);
        instance.slots[1] = name_val;
        instance.slots[2] = labels_list;
        instance.slots[3] = args_list;
        Value::Obj(self.heap.alloc(Object::Instance(instance)))
    }

    /// Forwards a missed send to the receiver's `doesNotUnderstand(_:)`
    /// (method-lookup.md §2, ADR-0012).
    ///
    /// Precondition: `self.stack[receiver_idx..]` holds `[receiver, args…]`.
    /// The arguments are replaced by a single synthesized
    /// [`Message`](Self::new_message) and the receiver's
    /// `doesNotUnderstand(_:)` is dispatched via [`Self::call_method`] (a
    /// primitive runs in place; a user override pushes a frame). Because
    /// `doesNotUnderstand(_:)` is looked up by the *exact* selector, it always
    /// resolves to at least `Object`'s default handler — a receiver whose chain
    /// somehow lacks it is a kernel-invariant violation, surfaced as
    /// [`RuntimeError::Internal`] rather than recursing (the recursion guard:
    /// a missing dNU is never itself re-sent as a dNU).
    ///
    /// # Errors
    ///
    /// Returns [`RuntimeError::Internal`] if `doesNotUnderstand(_:)` is missing
    /// from the receiver's chain, or propagates any error raised by the handler.
    fn forward_does_not_understand(&mut self, receiver_idx: usize, selector: Symbol, source_range: SourceRange) -> PhResult<()> {
        let receiver = self.stack[receiver_idx];
        let args: Vec<Value> = self.stack[receiver_idx + 1..].to_vec();
        // Keep the receiver, drop the original argument values.
        self.stack.truncate(receiver_idx + 1);
        let message = self.new_message(selector, &args);
        self.stack.push(message);

        let dnu_str = crate::method::encode_selector("doesNotUnderstand", &[None], crate::method::SignatureKind::Method(1));
        let dnu_sym = self.get_or_intern(&dnu_str);
        match receiver.lookup_method(self, dnu_sym) {
            Some(method) => self.call_method(&receiver, method, 1, source_range),
            None => Err(RuntimeError::Internal("doesNotUnderstand(_:) missing from Object — kernel invariant violated".into()).into()),
        }
    }

    /// Sends `selector` to `receiver` with `args`, runs the resolved method to
    /// completion, and returns its result value (messages-and-selectors.md §5).
    ///
    /// This is the shared runtime-send workhorse behind reflective dispatch:
    /// its three consumers are [`object_perform`](crate::primitive::object::object_perform)
    /// / [`object_perform_with`](crate::primitive::object::object_perform_with),
    /// the `doesNotUnderstand(_:)` forward (indirectly, via the same
    /// lookup+`call_method`+dNU path), and — deferred to U9 — a `SendDynamic`
    /// spread call-site opcode. Unlike the [`Bytecode::Invoke`] handler it can
    /// be called from *inside* a native primitive: it saves the frame count,
    /// pushes `receiver`+`args` at a fresh stack window, dispatches, then
    /// re-enters `run_until` to drain that one activation and recover a
    /// synchronous [`Value`] (the same re-entrancy pattern as
    /// [`block_call`](crate::primitive::block::block_call)). A miss routes
    /// through `doesNotUnderstand(_:)` exactly once — a `perform` of an unknown
    /// selector re-enters dNU, it does not loop.
    ///
    /// # Errors
    ///
    /// Propagates any [`RuntimeError`] raised by lookup, the dispatched method,
    /// or the `doesNotUnderstand(_:)` forward.
    pub fn send_dynamic(&mut self, receiver: Value, selector: Symbol, args: &[Value]) -> PhResult<Value> {
        let receiver_idx = self.stack.len();
        self.stack.push(receiver);
        self.stack.extend_from_slice(args);

        let base_frames = self.frames.len();
        if let Some(method) = receiver.lookup_method(self, selector) {
            self.call_method(&receiver, method, args.len(), SourceRange::default())?;
        } else {
            self.forward_does_not_understand(receiver_idx, selector, SourceRange::default())?;
        }
        self.run_until(base_frames)
    }

    /// Runs the exact method `method_id` against `receiver` with `args`,
    /// re-entering `run_until` to recover a synchronous result — the
    /// shared engine behind `Method#invokeOn(_,_)` and `Method#bind(_)`'s
    /// `call` (U-CORE-3, [ADR-0028](../../../docs/adr/0028-amend-floor-admit-method-reflection.md)).
    ///
    /// Mirrors [`Self::send_dynamic`]'s re-entrancy exactly, except there is
    /// **no lookup**: `method_id` is already resolved, so a mismatched
    /// receiver misbehaves inside the method body rather than raising
    /// `doesNotUnderstand(_:)` (the caller is responsible for receiver
    /// compatibility, functions.md §3). Arity is validated **before** the
    /// receiver/args are pushed onto the stack, so a mismatch leaves the stack
    /// exactly as it was found (R-INV-3.4).
    ///
    /// # Errors
    ///
    /// Returns [`RuntimeError::Arity`] if `args.len()` does not match the
    /// method's signature (respecting a variadic minimum), or propagates any
    /// [`RuntimeError`] raised while running the method body — including a
    /// [`RuntimeError::DeadFrameError`] from a non-local `return` inside an
    /// escaping block whose home frame is no longer live (R-INV-3.2).
    pub fn invoke_method_object(&mut self, method_id: ObjRef, receiver: Value, args: &[Value]) -> PhResult<Value> {
        let (positional, variadic) = {
            let sig = &self.heap.method(method_id).signature;
            (sig.positional_arity as usize, sig.variadic)
        };
        let ok = if variadic { args.len() >= positional } else { args.len() == positional };
        if !ok {
            return Err(RuntimeError::Arity { signature: "invokeOn", expected: positional, found: args.len() }.into());
        }

        self.stack.push(receiver);
        self.stack.extend_from_slice(args);

        let base_frames = self.frames.len();
        self.call_method(&receiver, method_id, args.len(), SourceRange::default())?;
        self.run_until(base_frames)
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
        let cell = self.heap.alloc(Object::Upvalue(Upvalue::Open(stack_index)));
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
    fn surface_absence(&self, value: Value) -> Value {
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
        loop {
            if self.frames.len() <= base_frames {
                // A drained frame stack with nothing left to yield means the
                // top-level program (or a block activation) fell off its end:
                // surface that absence as `None`, never the private sentinel.
                let result = self.stack.pop().unwrap_or(Value::Nil);
                return Ok(self.surface_absence(result));
            }

            let frame = *self.frames.last().unwrap();
            let closure_id = frame.closure;
            let ip = frame.ip;
            let stack_offset = frame.stack_offset;

            let (opcode, source_range) = {
                let chunk = &self.heap.closure(closure_id).callable.chunk;
                (chunk.code[ip], chunk.spans[ip])
            };

            let span = span!(Level::DEBUG, "vm_opcode", opcode = ?opcode);
            let _enter = span.enter();
            debug!("Stack before: {:?}", self.stack);

            self.frames.last_mut().unwrap().ip += 1;

            match opcode {
                Bytecode::Constant(idx) => {
                    let constant = self.heap.closure(closure_id).callable.chunk.constants[idx as usize];
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

                    let new_closure = self.heap.alloc(Object::Closure(ClosureObject { callable, module, upvalues }));
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
                                let new_class = self.create_class(&name, Some(sc_id));
                                self.stack.push(Value::Obj(new_class));
                            }
                            _ => return Err(RuntimeError::InvalidSuperClass(format!("{superclass}")).into()),
                        }
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
                        } else {
                            self.heap.class_mut(class_id).add_method(selector, method_id);
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
                                crate::instance::InstanceObject::new(class_id, field_count)
                            ));
                            self.stack.push(Value::Obj(instance_ref));
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
                        //   IC -> exact-probe -> variadic probe -> doesNotUnderstand(_:).
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
                        Upvalue::Open(slot) => self.stack[slot],
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
                        Upvalue::Open(slot) => self.stack[slot] = value,
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
            debug!("Stack after opcode {:?}: {:?}", opcode, self.stack);
        }
    }
}
