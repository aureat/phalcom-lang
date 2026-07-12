//! Bootstrap of the kernel class tower and installation of core primitives.
//!
//! The kernel is a cyclic graph closed through a distinct `Metaclass class`
//! row (`Metaclass.class == Metaclass class`, `(Metaclass class).class ==
//! Metaclass`; `object-model.md` §5–6). Under
//! [ADR-0009](../../../docs/adr/0009-handle-arena-heap.md) that cycle is built
//! by **allocate-then-patch**: every class row is first allocated bare in the
//! [`Heap`] to obtain its [`ClassId`], then its `class` and `superclass`
//! handles are written in place.
//!
//! The metaclass hierarchy runs *parallel* to the instance hierarchy
//! ([ADR-0002](../../../docs/adr/0002-metaclass-tower-parallel-rule.md)):
//! `(X class).superclass == (X.superclass) class`. `Behavior`
//! ([ADR-0003](../../../docs/adr/0003-introduce-behavior-kernel-class.md)) is
//! the shared abstract superclass of `Class` and `Metaclass`, so the tower
//! closes at an 8-row apex instead of collapsing `Metaclass`/`Class` into
//! their own metaclasses (F6).

use crate::heap::{ClassId, Heap, ObjRef};
use crate::interner::Symbol;
use crate::method::MethodObject;
use crate::method::SignatureKind;
use crate::primitive::boolean::{bool_and, bool_class_new, bool_hash, bool_if_false, bool_if_true, bool_if_true_if_false, bool_not, bool_or};
use crate::primitive::block::{block_arity, block_call, block_call_with, block_name, block_while_true};
use crate::primitive::class::{behavior_methods, behavior_name, class_add, class_new, class_set_superclass, class_superclass};
use crate::primitive::error::{error_message, error_raise};
use crate::primitive::fiber::{fiber_abort, fiber_call, fiber_current, fiber_new, fiber_try, fiber_yield};
use crate::primitive::list::{list_class_new, list_raw_at, list_raw_length, list_raw_push, list_raw_set, list_to_string};
use crate::primitive::map::{map_class_new, map_raw_get, map_raw_has, map_raw_key_at, map_raw_put, map_raw_remove, map_raw_size, map_raw_value_at};
use crate::primitive::method::{method_bind, method_class_new, method_holder, method_invoke_on, method_selector};
use crate::primitive::module::module_class_new;
use crate::primitive::nil::{option_match, some_new};
use crate::primitive::number::{
    number_add, number_class_new, number_div, number_ge, number_gt, number_hash, number_le, number_lt, number_mod, number_mul, number_negated, number_sub,
    number_to_string,
};
use crate::primitive::object::{
    message_args, message_labels, message_name, message_selector, object_class, object_class_new, object_does_not_understand, object_eq, object_hash,
    object_method_for, object_name, object_neq, object_perform, object_perform_with, object_responds_to, object_set_class, object_to_string,
};
use crate::primitive::primitive;
use crate::primitive::primitive_static;
use crate::primitive::set::{set_class_new, set_raw_add, set_raw_at, set_raw_has, set_raw_remove, set_raw_size};
use crate::primitive::string::{string_add, string_class_new, string_hash};
use crate::primitive::symbol::{symbol_class_new, symbol_hash, symbol_tostring};
use crate::primitive::system::{system_class_new, system_class_print};
use crate::vm::VM;

/// The kernel: handles to the bootstrapped core classes.
#[derive(Debug, Clone)]
pub struct Universe {
    /// Handles to every bootstrapped core class and metaclass.
    pub classes: CoreClasses,
    /// Override-epoch flag for the `Bool`-receiver sacred selectors
    /// (`and(_)`, `or(_)`, `not()`, `ifTrue(_)`, `ifFalse(_)`,
    /// `ifTrue(_)ifFalse(_)`). `true` from bootstrap until any of them is
    /// (re)installed directly on the kernel `Bool` class, at which point the
    /// sacred-selector inliner's [`crate::bytecode::Bytecode::GuardBool`]
    /// deopts every inlined call site back to a real send
    /// ([ADR-0018](../../../docs/adr/0018-sacred-selector-inliner-and-override-guard.md)).
    pub bool_sacred_pristine: bool,
    /// Override-epoch flag for the `Block`-receiver sacred selectors
    /// (`whileTrue(_)`), mirroring [`Universe::bool_sacred_pristine`] for the
    /// kernel `Block` class.
    pub block_sacred_pristine: bool,
}

impl Universe {
    /// Bootstraps the core class tower into `heap` and returns the [`Universe`].
    pub fn new(heap: &mut Heap) -> Self {
        Universe {
            classes: Self::create_core_classes(heap),
            bool_sacred_pristine: true,
            block_sacred_pristine: true,
        }
    }

    /// The `Bool`-receiver sacred selectors watched by
    /// [`Universe::bool_sacred_pristine`]
    /// ([ADR-0018](../../../docs/adr/0018-sacred-selector-inliner-and-override-guard.md)).
    pub const BOOL_SACRED_SELECTORS: &'static [&'static str] =
        &["and(_:)", "or(_:)", "not()", "ifTrue(_:)", "ifFalse(_:)", "ifTrue(_:ifFalse:)"];

    /// The `Block`-receiver sacred selectors watched by
    /// [`Universe::block_sacred_pristine`]
    /// ([ADR-0018](../../../docs/adr/0018-sacred-selector-inliner-and-override-guard.md)).
    pub const BLOCK_SACRED_SELECTORS: &'static [&'static str] = &["whileTrue(_:)"];

    /// Allocates and wires the kernel class tower via allocate-then-patch.
    ///
    /// Follows the seven-step order of `object-model.md` §6: allocate the 8
    /// apex rows bare, wire instance-of, wire instance-side superclasses, wire
    /// metaclass-side superclasses by the parallel rule
    /// ([ADR-0002](../../../docs/adr/0002-metaclass-tower-parallel-rule.md)),
    /// then create the remaining core classes through `make_core_class`.
    /// Step 7 (`verify_invariants`) is run by the caller ([`VM::new`]) once
    /// primitives are installed.
    pub fn create_core_classes(heap: &mut Heap) -> CoreClasses {
        // 1. Allocate the 8 apex rows bare (object-model.md §6 step 1).
        let object_class = heap.alloc_class(crate::class::ClassObject::bare("Object"));
        let behavior_class = heap.alloc_class(crate::class::ClassObject::bare("Behavior"));
        let class_class = heap.alloc_class(crate::class::ClassObject::bare("Class"));
        let metaclass_class = heap.alloc_class(crate::class::ClassObject::bare("Metaclass"));
        let object_metaclass = heap.alloc_class(crate::class::ClassObject::bare("Object class"));
        let behavior_metaclass = heap.alloc_class(crate::class::ClassObject::bare("Behavior class"));
        let class_metaclass = heap.alloc_class(crate::class::ClassObject::bare("Class class"));
        let metaclass_metaclass = heap.alloc_class(crate::class::ClassObject::bare("Metaclass class"));

        // 2. Wire instance-of (§6 step 2): every metaclass is an instance of
        //    Metaclass; Metaclass itself is an instance of Metaclass class,
        //    closing the loop; each ordinary class is an instance of its own
        //    metaclass.
        heap.class_mut(object_metaclass).class = metaclass_class;
        heap.class_mut(behavior_metaclass).class = metaclass_class;
        heap.class_mut(class_metaclass).class = metaclass_class;
        heap.class_mut(metaclass_metaclass).class = metaclass_class;
        heap.class_mut(metaclass_class).class = metaclass_metaclass;
        heap.class_mut(object_class).class = object_metaclass;
        heap.class_mut(behavior_class).class = behavior_metaclass;
        heap.class_mut(class_class).class = class_metaclass;

        // 3. Wire instance-side superclasses (§6 step 3).
        heap.class_mut(object_class).superclass = None;
        heap.class_mut(behavior_class).superclass = Some(object_class);
        heap.class_mut(class_class).superclass = Some(behavior_class);
        heap.class_mut(metaclass_class).superclass = Some(behavior_class);

        // 4. Wire metaclass-side superclasses by the parallel rule (§6 step 4,
        //    ADR-0002): (X class).superclass == (X.superclass) class.
        heap.class_mut(object_metaclass).superclass = Some(class_class);
        heap.class_mut(behavior_metaclass).superclass = Some(object_metaclass);
        heap.class_mut(class_metaclass).superclass = Some(behavior_metaclass);
        heap.class_mut(metaclass_metaclass).superclass = Some(behavior_metaclass);

        // 5. The remaining core classes, each with its own metaclass wired by
        //    the same parallel rule (§6 step 5).
        let number_class = make_core_class(heap, "Number", object_class, metaclass_class);
        let string_class = make_core_class(heap, "String", object_class, metaclass_class);
        let nil_class = make_core_class(heap, "Nil", object_class, metaclass_class);
        let bool_class = make_core_class(heap, "Bool", object_class, metaclass_class);
        // The boolean tower (ADR-0004): `Bool` is abstract — no value is ever
        // *directly* an instance of it. Its two concrete singleton subclasses,
        // `True` and `False`, are the surface classes of the `true`/`false`
        // immediates (`Value::class`, value.rs), so `true.class == True` and
        // `false.class == False`. The six sacred control-flow selectors
        // (`not`/`and`/`or`/`ifTrue`/`ifFalse`/`ifTrue:ifFalse:`) stay as native
        // primitives on `Bool` and are reached by inheritance (see
        // `floor-census.md` §2.6/§5; ADR-0004 dispatches by class, and walking
        // `True`/`False` to their shared parent *is* dispatch by class).
        let true_class = make_core_class(heap, "True", bool_class, metaclass_class);
        let false_class = make_core_class(heap, "False", bool_class, metaclass_class);
        // Callables (ADR-0006, decisions.md §4.1): `Function` is the abstract
        // callable root; `Block` and `Method` are its siblings. `Method` must be
        // allocated *after* `Function` because `make_core_class` reads
        // `heap.class(Function).class` to wire the parallel rule — its
        // superclass must already have its `class` link. `Method` therefore
        // re-parents from `Object` to `Function` and inherits the call protocol
        // (`arity`/`name`/`call…`/`callWith`) rather than redefining it.
        let function_class = make_core_class(heap, "Function", object_class, metaclass_class);
        let block_class = make_core_class(heap, "Block", function_class, metaclass_class);
        let method_class = make_core_class(heap, "Method", function_class, metaclass_class);
        let symbol_class = make_core_class(heap, "Symbol", object_class, metaclass_class);
        let module_class = make_core_class(heap, "Module", object_class, metaclass_class);
        let system_class = make_core_class(heap, "System", object_class, metaclass_class);

        // The absence type (ADR-0007): `Option` is abstract, with concrete
        // subclasses `Some` (one field, `_value`) and `None`. This mirrors the
        // abstract-`Bool` / `True`-`False` shape (ADR-0004): dispatch, not a
        // variant tag, distinguishes present from absent. There is no surface
        // `nil` class — the private `Value::Nil` sentinel (ADR-0010) is surfaced
        // to `None` at read boundaries and can never be produced by user code.
        let option_class = make_core_class(heap, "Option", object_class, metaclass_class);
        let some_class = make_core_class(heap, "Some", option_class, metaclass_class);
        let none_class = make_core_class(heap, "None", option_class, metaclass_class);

        // The single shared `None`: one heap instance, reused everywhere so
        // `None` is identity-comparable and zero-allocation. The `None` global
        // (bound in `VM::install_core`) points at *this* object, not the `None`
        // class.
        let none_singleton = heap.alloc(crate::heap::Object::Instance(crate::instance::InstanceObject::new(none_class, 0)));

        // Kernel `List` (ADR-0020): a native heap variant, not an
        // `InstanceObject`, so it has no field layout and needs no `construct`
        // lowering — created here the same way `Option`/`Bool`/`String` are,
        // positioned after the absence type per ADR-0020's load order
        // (`Bool, Option, Number, Symbol, String → List → …`) and before
        // anything that will depend on it (`Message.args`/rest-params, U8/U9).
        let list_class = make_core_class(heap, "List", object_class, metaclass_class);

        // Kernel `Map`/`Set` (ADR-0032 §1, ADR-0039, U-COLLTYPES): native heap
        // variants over the shared `MapObject` ordered-hash backing struct
        // (DEC-CT-B) — distinct `Object::Map`/`Object::Set` arms, distinct
        // classes, positioned directly after `List` (same "no field layout,
        // no `.ph` construct" load-order rationale as ADR-0020).
        let map_class = make_core_class(heap, "Map", object_class, metaclass_class);
        let set_class = make_core_class(heap, "Set", object_class, metaclass_class);

        // Kernel `Message` (method-lookup.md §2, ADR-0012): the reified miss
        // send handed to `doesNotUnderstand(_:)`. An ordinary fixed-slot
        // `InstanceObject` (four slots: selector/name/labels/args) built
        // directly in Rust by `VM::new_message` — no `.ph` `construct`, its
        // field count is stamped in `VM::new` mirroring `Some`. Its accessors
        // are native primitives (`primitive/object.rs`).
        let message_class = make_core_class(heap, "Message", object_class, metaclass_class);

        // `Error` root + `MessageNotUnderstood < Error` (U-CORE-6, ADR-0008):
        // the minimal reification slice of the surface error hierarchy. Like
        // `Message`, both are ordinary fixed-slot `InstanceObject`s stamped in
        // `VM::new`'s Phase E rather than given a `.ph` field layout (avoids
        // the compiler's read-before-write check on a getter that only reads
        // `_message`, never assigns it). `error_class` must be created before
        // `message_not_understood_class` since the latter's superclass is the
        // former (mirrors the `Option → Some/None` ordering above).
        let error_class = make_core_class(heap, "Error", object_class, metaclass_class);
        let message_not_understood_class = make_core_class(heap, "MessageNotUnderstood", error_class, metaclass_class);

        // `Fiber` (ADR-0030, U-FIBER): the sole concurrency primitive, a
        // native `Object::Fiber` heap variant (D2 — no `Value::Fiber` arm),
        // mirroring how `List` sits directly under `Object`.
        // `CannotYieldAcrossNativeFrame < Error`: the restricted-yield guard's
        // catchable error (D-FIB-1) — a `yield` attempted across a re-entrant
        // native frame (e.g. under `.each { }`'s `block_call`) raises this
        // instead of corrupting the suspended position (ADR-0030 §4).
        let fiber_class = make_core_class(heap, "Fiber", object_class, metaclass_class);
        let cannot_yield_across_native_frame_class = make_core_class(heap, "CannotYieldAcrossNativeFrame", error_class, metaclass_class);

        CoreClasses {
            object_class,
            behavior_class,
            class_class,
            metaclass_class,
            number_class,
            string_class,
            nil_class,
            bool_class,
            true_class,
            false_class,
            method_class,
            function_class,
            block_class,
            symbol_class,
            module_class,
            system_class,
            option_class,
            some_class,
            none_class,
            none_singleton,
            list_class,
            map_class,
            set_class,
            message_class,
            error_class,
            message_not_understood_class,
            fiber_class,
            cannot_yield_across_native_frame_class,
        }
    }

    /// Flags a (re)definition of `selector` directly on `class_id`, flipping
    /// the relevant override-epoch flag if it is a sacred selector on the
    /// kernel `Bool`/`Block` class.
    ///
    /// Called from the [`crate::bytecode::Bytecode::Method`] handler
    /// (`vm.rs`) every time a class body attaches a method — the only place
    /// user code can (re)install a method on a class row, whether that row
    /// is the *original* kernel `Bool`/`Block` (impossible for surface
    /// Phalcom today — there is no class-reopening syntax) or a
    /// same-named redeclaration that U5 makes *reopen* the existing row
    /// (see `compiler/lib.rs`'s `Statement::Class` handling) specifically so
    /// this deopt path is exercisable and testable
    /// ([ADR-0018](../../../docs/adr/0018-sacred-selector-inliner-and-override-guard.md)).
    /// A cheap `==` on two [`ClassId`]s per method definition; not on any
    /// hot path.
    pub fn note_method_installed(&mut self, class_id: ClassId, selector: Symbol, interner: &crate::interner::Interner) {
        let name = interner.lookup(selector);
        if class_id == self.classes.bool_class && Self::BOOL_SACRED_SELECTORS.contains(&name) {
            self.bool_sacred_pristine = false;
        }
        if class_id == self.classes.block_class && Self::BLOCK_SACRED_SELECTORS.contains(&name) {
            self.block_sacred_pristine = false;
        }
    }

    /// Installs every native primitive method onto the core classes.
    pub fn install_primitives(vm: &mut VM) {
        let object_cls = vm.universe.classes.object_class;
        primitive!(vm, object_cls, "name", SignatureKind::Getter, object_name);
        primitive!(vm, object_cls, "class", SignatureKind::Getter, object_class);
        primitive!(vm, object_cls, "class", SignatureKind::Setter, object_set_class);
        primitive!(vm, object_cls, "toString", SignatureKind::Getter, object_to_string);
        // Identity `hash` (object-model.md §8, ADR-0023): immediates override it
        // per-type below; every heap object inherits this handle digest.
        primitive!(vm, object_cls, "hash", SignatureKind::Getter, object_hash);
        primitive_static!(vm, object_cls, "new", SignatureKind::Method(0), object_class_new);
        // U5 (control-flow.md §1): `==`/`!=` are ordinary sends, not opcodes.
        primitive!(vm, object_cls, "==", SignatureKind::Method(1), object_eq);
        primitive!(vm, object_cls, "!=", SignatureKind::Method(1), object_neq);
        // Reflective-send + miss-handler surface (U8, messages-and-selectors.md
        // §5, method-lookup.md §2, ADR-0012). `doesNotUnderstand(_:)` is the
        // terminal miss fallback the `Bytecode::Invoke` handler forwards to;
        // it is an ordinary overridable method so a proxy subclass can
        // intercept. `respondsTo(_:)` is a pure probe that never triggers dNU.
        primitive!(vm, object_cls, "perform", SignatureKind::Method(1), object_perform);
        primitive!(vm, object_cls, "perform", SignatureKind::Method(2), object_perform_with);
        primitive!(vm, object_cls, "respondsTo", SignatureKind::Method(1), object_responds_to);
        primitive!(vm, object_cls, "doesNotUnderstand", SignatureKind::Method(1), object_does_not_understand);
        // `Method` reflection surface (U-CORE-3, ADR-0028): reifies the
        // resolved `MethodObject` for a selector, a pure probe like
        // `respondsTo` (never fires dNU on a miss).
        primitive!(vm, object_cls, "methodFor", SignatureKind::Method(1), object_method_for);

        // `Message` accessors (U8): native getters reading the reified-send
        // slots directly (`VM::new_message`); `Message` has no `.ph` surface.
        // `name` deliberately shadows `Object::name` on a `Message` receiver —
        // it returns the sent *method* name, not the class name.
        let message_cls = vm.universe.classes.message_class;
        primitive!(vm, message_cls, "selector", SignatureKind::Getter, message_selector);
        primitive!(vm, message_cls, "name", SignatureKind::Getter, message_name);
        primitive!(vm, message_cls, "labels", SignatureKind::Getter, message_labels);
        primitive!(vm, message_cls, "args", SignatureKind::Getter, message_args);

        let behavior_cls = vm.universe.classes.behavior_class;
        primitive!(vm, behavior_cls, "superclass", SignatureKind::Getter, class_superclass);
        primitive!(vm, behavior_cls, "superclass", SignatureKind::Setter, class_set_superclass);
        // Class-side reflection (ADR-0023): `name` returns the receiver class's
        // OWN name (shadowing `Object#name`, which yields the *metaclass* name
        // "C class" for a class receiver); `methods` enumerates the receiver's
        // own method dictionary as selector Symbols. Installed on `Behavior` so
        // `Class` and `Metaclass` both inherit them (mirrors `superclass`).
        primitive!(vm, behavior_cls, "name", SignatureKind::Getter, behavior_name);
        primitive!(vm, behavior_cls, "methods", SignatureKind::Getter, behavior_methods);

        let class_cls = vm.universe.classes.class_class;
        primitive!(vm, class_cls, "+", SignatureKind::Method(1), class_add);
        primitive!(vm, class_cls, "new", SignatureKind::Method(0), class_new);

        let number_cls = vm.universe.classes.number_class;
        // U5 (control-flow.md §1): every arithmetic/comparison operator is an
        // ordinary, non-sacred send — never inlined (U5-plan.md §4.1).
        primitive!(vm, number_cls, "+", SignatureKind::Method(1), number_add);
        primitive!(vm, number_cls, "-", SignatureKind::Method(1), number_sub);
        primitive!(vm, number_cls, "*", SignatureKind::Method(1), number_mul);
        primitive!(vm, number_cls, "/", SignatureKind::Method(1), number_div);
        primitive!(vm, number_cls, "%", SignatureKind::Method(1), number_mod);
        primitive!(vm, number_cls, "<", SignatureKind::Method(1), number_lt);
        primitive!(vm, number_cls, "<=", SignatureKind::Method(1), number_le);
        primitive!(vm, number_cls, ">", SignatureKind::Method(1), number_gt);
        primitive!(vm, number_cls, ">=", SignatureKind::Method(1), number_ge);
        primitive!(vm, number_cls, "negated", SignatureKind::Method(0), number_negated);
        // Value digest (ADR-0023): overrides `Object#hash` with a hash of the
        // mathematical value, class-agnostically (forward-compat §4).
        primitive!(vm, number_cls, "hash", SignatureKind::Getter, number_hash);
        // Decimal-string render (U-CORE-4, ADR-0019 amendment): unreachable
        // from `.ph` (no number->string path), so it earns the one new floor
        // binding this unit adds. Delegates to `Value::to_string`.
        primitive!(vm, number_cls, "toString", SignatureKind::Getter, number_to_string);
        primitive_static!(vm, number_cls, "new", SignatureKind::Method(0), number_class_new);
        primitive_static!(vm, number_cls, "new", SignatureKind::Method(1), number_class_new);

        let string_cls = vm.universe.classes.string_class;
        primitive!(vm, string_cls, "+", SignatureKind::Method(1), string_add);
        // Value digest (ADR-0023): the cached djb2 CONTENT hash, so two
        // distinct-handle equal-content strings hash equal (R-INV-1.3).
        primitive!(vm, string_cls, "hash", SignatureKind::Getter, string_hash);
        primitive_static!(vm, string_cls, "new", SignatureKind::Method(0), string_class_new);
        primitive_static!(vm, string_cls, "new", SignatureKind::Method(1), string_class_new);

        let bool_cls = vm.universe.classes.bool_class;
        primitive_static!(vm, bool_cls, "new", SignatureKind::Method(0), bool_class_new);
        primitive_static!(vm, bool_cls, "new", SignatureKind::Method(1), bool_class_new);
        // Sacred selectors (control-flow.md §2–3): registered here as the
        // real send targets; ADR-0018's inliner special-cases literal-block
        // call sites but always deopts to exactly these on override/mismatch.
        primitive!(vm, bool_cls, "and", SignatureKind::Method(1), bool_and);
        primitive!(vm, bool_cls, "or", SignatureKind::Method(1), bool_or);
        primitive!(vm, bool_cls, "not", SignatureKind::Method(0), bool_not);
        primitive!(vm, bool_cls, "ifTrue", SignatureKind::Method(1), bool_if_true);
        primitive!(vm, bool_cls, "ifFalse", SignatureKind::Method(1), bool_if_false);
        // control-flow.md §3's `ifTrue(_)ifFalse(_)` is Smalltalk's
        // independently-worded `ifTrue:ifFalse:` keyword pair; Phalcom's
        // selector model (ADR-0012) has no such shape — one base name plus
        // per-argument *labels* on that name. U5 realizes the same paired
        // conditional as `ifTrue(_:ifFalse:)`: one positional arg plus one
        // `ifFalse:`-labeled arg, fully expressible via the existing
        // `encode_selector`, and is what `if/else` desugars to (ADR-0018).
        {
            let sig_str = crate::method::encode_selector("ifTrue", &[None, Some("ifFalse".to_string())], SignatureKind::Method(2));
            let symbol = vm.get_or_intern(&sig_str);
            let method = MethodObject::new_primitive(symbol, SignatureKind::Method(2), bool_if_true_if_false, bool_cls);
            let method_id = vm.heap.alloc(crate::heap::Object::Method(method));
            vm.heap.class_mut(bool_cls).add_method(symbol, method_id);
        }
        // Value digest (ADR-0023): 1 for `true`, 0 for `false` — distinct and
        // stable. NOT a sacred selector (no deopt budget; see `bool_hash`).
        primitive!(vm, bool_cls, "hash", SignatureKind::Getter, bool_hash);

        let symbol_cls = vm.universe.classes.symbol_class;
        primitive!(vm, symbol_cls, "toString", SignatureKind::Getter, symbol_tostring);
        // Value digest (ADR-0023): the interned id, so equal symbols agree.
        primitive!(vm, symbol_cls, "hash", SignatureKind::Getter, symbol_hash);
        primitive_static!(vm, symbol_cls, "new", SignatureKind::Method(1), symbol_class_new);

        // Absence substrate (ADR-0007): `Some(_)` construction and the
        // `match(some:none:)` eliminator. Bootstrapped as Rust primitives so U6
        // does not depend on U7's user-facing `construct`. The rich combinator
        // suite (`map`/`flatMap`/`orElse`/…) is U-STD's job, defined over
        // `match` in `core.ph`. There is intentionally no surface `Nil` class or
        // `nil` constructor — the private `Value::Nil` sentinel (ADR-0010) is
        // surfaced to `None` at read boundaries only.
        let some_cls = vm.universe.classes.some_class;
        primitive_static!(vm, some_cls, "new", SignatureKind::Method(1), some_new);

        // `match(some:none:)` is installed on the abstract `Option` class so
        // both `Some` and `None` inherit it (values-and-absence.md §3.2). Both
        // arguments are labeled, so `make_signature` (label-free) can't build
        // the selector; encode it explicitly like `ifTrue(_:ifFalse:)`.
        let option_cls = vm.universe.classes.option_class;
        {
            let sig_str = crate::method::encode_selector("match", &[Some("some".to_string()), Some("none".to_string())], SignatureKind::Method(2));
            let symbol = vm.get_or_intern(&sig_str);
            let method = MethodObject::new_primitive(symbol, SignatureKind::Method(2), option_match, option_cls);
            let method_id = vm.heap.alloc(crate::heap::Object::Method(method));
            vm.heap.class_mut(option_cls).add_method(symbol, method_id);
        }

        let method_cls = vm.universe.classes.method_class;
        primitive_static!(vm, method_cls, "new", SignatureKind::Method(1), method_class_new);
        // `Method` reflection surface (U-CORE-3, ADR-0028): applying a
        // reified method to an explicit receiver, closing it over one, and
        // reading its selector/holder. See `primitive::method` module doc.
        primitive!(vm, method_cls, "invokeOn", SignatureKind::Method(2), method_invoke_on);
        primitive!(vm, method_cls, "bind", SignatureKind::Method(1), method_bind);
        primitive!(vm, method_cls, "selector", SignatureKind::Getter, method_selector);
        primitive!(vm, method_cls, "holder", SignatureKind::Getter, method_holder);

        // `call` is registered per arity (functions.md §1: `call`, `call(_:)`,
        // `call(_:_:)`, …) since Phalcom dispatch keys on the arity-encoded
        // selector, not a single variadic entry point. `callWith(_:)` takes one
        // packed argument (deferred to a plain forward until `List` lands, see
        // `docs/forge/DEFERRED.md`).
        const MAX_CALL_ARITY: u8 = 4;

        let function_cls = vm.universe.classes.function_class;
        primitive!(vm, function_cls, "arity", SignatureKind::Getter, block_arity);
        primitive!(vm, function_cls, "name", SignatureKind::Getter, block_name);
        primitive!(vm, function_cls, "callWith", SignatureKind::Method(1), block_call_with);
        for n in 0..=MAX_CALL_ARITY {
            primitive!(vm, function_cls, "call", SignatureKind::Method(n), block_call);
        }

        let block_cls = vm.universe.classes.block_class;
        primitive!(vm, block_cls, "arity", SignatureKind::Getter, block_arity);
        primitive!(vm, block_cls, "name", SignatureKind::Getter, block_name);
        primitive!(vm, block_cls, "callWith", SignatureKind::Method(1), block_call_with);
        for n in 0..=MAX_CALL_ARITY {
            primitive!(vm, block_cls, "call", SignatureKind::Method(n), block_call);
        }
        // Sacred loop fallback (control-flow.md §1/§3); `repeat(_)` is
        // deferred — its receiver/semantics aren't pinned by the spec
        // (U5-plan.md BD-U5-2) — see `docs/forge/DEFERRED.md`.
        primitive!(vm, block_cls, "whileTrue", SignatureKind::Method(1), block_while_true);

        let system_cls = vm.universe.classes.system_class;
        primitive_static!(vm, system_cls, "print", SignatureKind::Method(1), system_class_print);
        primitive_static!(vm, system_cls, "new", SignatureKind::Method(0), system_class_new);

        let module_cls = vm.universe.classes.module_class;
        primitive_static!(vm, module_cls, "new", SignatureKind::Method(0), module_class_new);

        // Kernel `List` (ADR-0019/0020): five native floor primitives.
        // `rawLength`/`rawAt`/`rawSet`/`rawPush` are internal — `.ph`'s
        // `size`/`at(_:)`/`add(_:)` wrap the first three (`rawSet` is
        // implemented but not yet surfaced, see DEFERRED.md); amortized growth
        // folds into `rawPush` (`Vec::push`'s own doubling), so there is no
        // separate "grow" primitive. `new()` and `toString` are public
        // primitives directly, mirroring `String`'s `+(_)` (see the U-LIST
        // return contract for why `toString` is native this unit).
        let list_cls = vm.universe.classes.list_class;
        primitive_static!(vm, list_cls, "new", SignatureKind::Method(0), list_class_new);
        primitive!(vm, list_cls, "rawLength", SignatureKind::Getter, list_raw_length);
        primitive!(vm, list_cls, "rawAt", SignatureKind::Method(1), list_raw_at);
        primitive!(vm, list_cls, "rawSet", SignatureKind::Method(2), list_raw_set);
        primitive!(vm, list_cls, "rawPush", SignatureKind::Method(1), list_raw_push);
        primitive!(vm, list_cls, "toString", SignatureKind::Getter, list_to_string);

        // Kernel `Map`/`Set` (ADR-0039, U-COLLTYPES Phase 1): the raw
        // hash-collection floor. `Map` gets 8 bindings (`new` + 7 raw
        // instance ops); `Set` gets 6 (`new` + 5 raw instance ops) — a keys-only
        // sibling reusing `Map`'s backing struct (DEC-CT-B) but with its own
        // distinct bindings. `rawKeyAt`/`rawValueAt`/`rawAt` back
        // `keys`/`values`/`each(_)`; `rawPut`/`rawAdd` re-enter the VM to send
        // `hash`/`==` on keys (see `primitive::map`'s module doc) and reject a
        // mutable-collection key (DEC-CT-C). `.ph`'s public protocol
        // (`at(_)`/`at(_,put:)`/`size`/`includes(_)`/`remove(_)`/`keys`/
        // `values`/`each(_)`/`add(_)`) wraps these in `core.ph`.
        let map_cls = vm.universe.classes.map_class;
        primitive_static!(vm, map_cls, "new", SignatureKind::Method(0), map_class_new);
        primitive!(vm, map_cls, "rawSize", SignatureKind::Getter, map_raw_size);
        primitive!(vm, map_cls, "rawGet", SignatureKind::Method(1), map_raw_get);
        primitive!(vm, map_cls, "rawPut", SignatureKind::Method(2), map_raw_put);
        primitive!(vm, map_cls, "rawHas", SignatureKind::Method(1), map_raw_has);
        primitive!(vm, map_cls, "rawRemove", SignatureKind::Method(1), map_raw_remove);
        primitive!(vm, map_cls, "rawKeyAt", SignatureKind::Method(1), map_raw_key_at);
        primitive!(vm, map_cls, "rawValueAt", SignatureKind::Method(1), map_raw_value_at);

        let set_cls = vm.universe.classes.set_class;
        primitive_static!(vm, set_cls, "new", SignatureKind::Method(0), set_class_new);
        primitive!(vm, set_cls, "rawSize", SignatureKind::Getter, set_raw_size);
        primitive!(vm, set_cls, "rawAdd", SignatureKind::Method(1), set_raw_add);
        primitive!(vm, set_cls, "rawHas", SignatureKind::Method(1), set_raw_has);
        primitive!(vm, set_cls, "rawRemove", SignatureKind::Method(1), set_raw_remove);
        primitive!(vm, set_cls, "rawAt", SignatureKind::Method(1), set_raw_at);

        // `Error` root (U-CORE-6, ADR-0008): `message` is a native slot-0
        // accessor (mirrors `Message`'s accessors — a `.ph` getter over this
        // field would trip the read-before-write check); `raise` initiates
        // the unified unwind's `Raise` payload (`throw expr === expr.raise()`,
        // ADR-0031 §1). Installed only on `Error`, so a non-`Error` receiver
        // has no `raise` (R-INV-6.3). +2 floor bindings, ADR-0023-cleared.
        let error_cls = vm.universe.classes.error_class;
        primitive!(vm, error_cls, "message", SignatureKind::Getter, error_message);
        primitive!(vm, error_cls, "raise", SignatureKind::Method(0), error_raise);

        // `Fiber` (U-FIBER, ADR-0030): `new(_)` builds; `call`/`try` (arity
        // 0 or 1) resume; `yield`/`current`/`abort` are class-side (sent to
        // `Fiber` itself, not an instance).
        let fiber_cls = vm.universe.classes.fiber_class;
        primitive_static!(vm, fiber_cls, "new", SignatureKind::Method(1), fiber_new);
        primitive!(vm, fiber_cls, "call", SignatureKind::Method(0), fiber_call);
        primitive!(vm, fiber_cls, "call", SignatureKind::Method(1), fiber_call);
        primitive!(vm, fiber_cls, "try", SignatureKind::Method(0), fiber_try);
        primitive!(vm, fiber_cls, "try", SignatureKind::Method(1), fiber_try);
        primitive_static!(vm, fiber_cls, "yield", SignatureKind::Method(0), fiber_yield);
        primitive_static!(vm, fiber_cls, "yield", SignatureKind::Method(1), fiber_yield);
        primitive_static!(vm, fiber_cls, "current", SignatureKind::Getter, fiber_current);
        primitive_static!(vm, fiber_cls, "abort", SignatureKind::Method(1), fiber_abort);
    }

    /// Asserts the kernel tower's shape (`object-model.md` §5–6 step 7).
    ///
    /// Checks every apex `.class`/`.superclass` relationship in the §5 table
    /// plus the four sanity checks (§5): the closed metaclass loop, the
    /// parallel rule holding for an ordinary core class, and that every
    /// metaclass's superclass chain terminates. Called once from [`VM::new`]
    /// right after [`Universe::install_primitives`]; the caller
    /// `.expect()`s the result, since a malformed kernel cannot run any
    /// program correctly.
    ///
    /// # Errors
    ///
    /// Returns `Err` with a description of the first violated invariant.
    pub fn verify_invariants(&self, heap: &Heap) -> Result<(), String> {
        let c = &self.classes;

        let object_metaclass = heap.class(c.object_class).class;
        let behavior_metaclass = heap.class(c.behavior_class).class;
        let class_metaclass = heap.class(c.class_class).class;
        let metaclass_metaclass = heap.class(c.metaclass_class).class;

        if object_metaclass == c.object_class {
            return Err("Object.class must not equal Object itself".to_string());
        }
        if heap.class(c.behavior_class).superclass != Some(c.object_class) {
            return Err("Behavior.superclass should be Object".to_string());
        }
        if heap.class(c.class_class).superclass != Some(c.behavior_class) {
            return Err("Class.superclass should be Behavior".to_string());
        }
        if heap.class(c.metaclass_class).superclass != Some(c.behavior_class) {
            return Err("Metaclass.superclass should be Behavior".to_string());
        }
        if heap.class(c.object_class).superclass.is_some() {
            return Err("Object.superclass should be None".to_string());
        }

        if heap.class(object_metaclass).class != c.metaclass_class {
            return Err("Object.class.class should be Metaclass".to_string());
        }
        if heap.class(behavior_metaclass).class != c.metaclass_class {
            return Err("Behavior.class.class should be Metaclass".to_string());
        }
        if heap.class(class_metaclass).class != c.metaclass_class {
            return Err("Class.class.class should be Metaclass".to_string());
        }
        if heap.class(metaclass_metaclass).class != c.metaclass_class {
            return Err("Metaclass.class.class should be Metaclass".to_string());
        }
        // The closed loop: Metaclass.class == Metaclass class, and
        // (Metaclass class).class == Metaclass.
        if heap.class(c.metaclass_class).class != metaclass_metaclass {
            return Err("Metaclass.class should be Metaclass class".to_string());
        }

        if heap.class(object_metaclass).superclass != Some(c.class_class) {
            return Err("Object.class.superclass should be Class".to_string());
        }
        if heap.class(behavior_metaclass).superclass != Some(object_metaclass) {
            return Err("Behavior.class.superclass should be Object.class".to_string());
        }
        if heap.class(class_metaclass).superclass != Some(behavior_metaclass) {
            return Err("Class.class.superclass should be Behavior.class".to_string());
        }
        if heap.class(metaclass_metaclass).superclass != Some(behavior_metaclass) {
            return Err("Metaclass.class.superclass should be Behavior.class".to_string());
        }

        // R-INV-0.2 — the parallel rule ([ADR-0002](../../../docs/adr/0002-metaclass-tower-parallel-rule.md))
        // holds for *every* ordinary (non-apex) core row, not just `Number`:
        // `X.class.superclass == X.superclass.class`. Includes the U11 `True`/
        // `False` rows (both resolve to `Bool class`) and the absence /
        // collection / message rows. Any newly-added row that breaks the rule
        // fails boot rather than silently mis-dispatching statics.
        let ordinary_rows: [(&str, ClassId); 21] = [
            ("Number", c.number_class),
            ("String", c.string_class),
            ("Nil", c.nil_class),
            ("Bool", c.bool_class),
            ("True", c.true_class),
            ("False", c.false_class),
            ("Method", c.method_class),
            ("Function", c.function_class),
            ("Block", c.block_class),
            ("Symbol", c.symbol_class),
            ("Module", c.module_class),
            ("System", c.system_class),
            ("Option", c.option_class),
            ("Some", c.some_class),
            ("None", c.none_class),
            ("List", c.list_class),
            ("Map", c.map_class),
            ("Set", c.set_class),
            ("Message", c.message_class),
            ("Error", c.error_class),
            ("MessageNotUnderstood", c.message_not_understood_class),
        ];
        for (name, class_id) in ordinary_rows {
            let meta = heap.class(class_id).class;
            let superclass = heap
                .class(class_id)
                .superclass
                .ok_or_else(|| format!("{name}.superclass should be set (parallel rule)"))?;
            let expected_meta_super = heap.class(superclass).class;
            if heap.class(meta).superclass != Some(expected_meta_super) {
                return Err(format!("{name}.class.superclass should be {name}.superclass.class (parallel rule)"));
            }
        }

        // R-INV-1.5 (boot half) — `Method` re-parents under `Function`
        // ([ADR-0006](../../../docs/adr/0006-function-as-abstract-callable-root.md),
        // decisions.md §4.1), so it inherits the call protocol instead of
        // redefining it. Guards the load-order fix in `create_core_classes`.
        if heap.class(c.method_class).superclass != Some(c.function_class) {
            return Err("Method.superclass should be Function (ADR-0006 re-parent)".to_string());
        }

        // R-INV-3.1 (boot half) — the callable tower: `Block` is `Function`'s
        // other sibling row, so both `Method` and `Block` share the call
        // protocol root (U-CORE-3, [ADR-0028](../../../docs/adr/0028-amend-floor-admit-method-reflection.md)).
        // The `ordinary_rows` parallel-rule loop above already covers the
        // metaclass-level half for both rows; this asserts the plain
        // superclass link explicitly, mirroring the `Method` check above.
        if heap.class(c.block_class).superclass != Some(c.function_class) {
            return Err("Block.superclass should be Function (ADR-0006)".to_string());
        }

        // R-INV-0.3 (structural half) — absence never surfaces at boot: the
        // shared `None` singleton is an `Instance` of `None` (not a class
        // object), and `None` is a distinct class from the unreachable `Nil`
        // (ADR-0007/0010). The global-resolves-to-the-singleton-*value* half is
        // asserted inline in `VM::new` (it needs the core module).
        if c.none_class == c.nil_class {
            return Err("None and Nil must be distinct classes".to_string());
        }
        match heap.get(c.none_singleton) {
            crate::heap::Object::Instance(instance) if instance.class == c.none_class => {}
            crate::heap::Object::Instance(_) => return Err("None singleton must be an instance of None".to_string()),
            _ => return Err("None singleton must be an instance object, not a class object".to_string()),
        }

        // R-INV-0.4 — fixed-slot layout for the two directly-stamped classes
        // ([ADR-0011](../../../docs/adr/0011-static-instance-slot-layout.md)):
        // `Some` has one field (`_value`) and `Message` has four. Fences the
        // E→F bootstrap edge (bootstrap-phases §4).
        if heap.class(c.some_class).field_count != 1 {
            return Err("Some.field_count should be 1 (ADR-0011)".to_string());
        }
        if heap.class(c.message_class).field_count != 4 {
            return Err("Message.field_count should be 4 (ADR-0011)".to_string());
        }
        if heap.class(c.error_class).field_count != 1 {
            return Err("Error.field_count should be 1 (ADR-0011, U-CORE-6)".to_string());
        }
        if heap.class(c.message_not_understood_class).field_count != 2 {
            return Err("MessageNotUnderstood.field_count should be 2 (ADR-0011, U-CORE-6)".to_string());
        }

        // R-INV-6.1 — `MessageNotUnderstood < Error < Object`, explicit beyond
        // the generic parallel-rule loop above (U-CORE-6,
        // invariant-requirements.md §U-CORE-6).
        if heap.class(c.error_class).superclass != Some(c.object_class) {
            return Err("Error.superclass should be Object (U-CORE-6)".to_string());
        }
        if heap.class(c.message_not_understood_class).superclass != Some(c.error_class) {
            return Err("MessageNotUnderstood.superclass should be Error (U-CORE-6)".to_string());
        }

        // Every metaclass's superclass chain terminates (bounded walk guards
        // against a cycle turning into a hang instead of a failure).
        let mut current = heap.class(c.number_class).class;
        let mut steps = 0;
        loop {
            steps += 1;
            if steps > 64 {
                return Err("metaclass superclass chain did not terminate within 64 steps".to_string());
            }
            match heap.class(current).superclass {
                Some(next) => current = next,
                None => break,
            }
        }

        Ok(())
    }
}

/// Allocates a core class `name` (with its own metaclass) and wires it.
///
/// The metaclass `"{name} class"` is an instance of `metaclass_class` with
/// superclass `superclass.class` (the parallel rule,
/// [ADR-0002](../../../docs/adr/0002-metaclass-tower-parallel-rule.md)); the
/// class itself is an instance of that metaclass with the given
/// `superclass`. `superclass` must already have its `class` link wired.
fn make_core_class(heap: &mut Heap, name: &str, superclass: ClassId, metaclass_class: ClassId) -> ClassId {
    let metaclass_superclass = heap.class(superclass).class;

    let metaclass = heap.alloc_class(crate::class::ClassObject::bare(&format!("{name} class")));
    {
        let meta = heap.class_mut(metaclass);
        meta.class = metaclass_class;
        meta.superclass = Some(metaclass_superclass);
    }
    let class = heap.alloc_class(crate::class::ClassObject::bare(name));
    {
        let class_ref = heap.class_mut(class);
        class_ref.class = metaclass;
        class_ref.superclass = Some(superclass);
    }
    class
}

/// Handles to the bootstrapped kernel classes and their metaclasses.
#[derive(Debug, Clone, Copy)]
pub struct CoreClasses {
    /// The root class, `Object`.
    pub object_class: ClassId,
    /// `Behavior`, the shared abstract superclass of `Class` and `Metaclass`
    /// ([ADR-0003](../../../docs/adr/0003-introduce-behavior-kernel-class.md)).
    pub behavior_class: ClassId,
    /// `Class`, the class of every ordinary class.
    pub class_class: ClassId,
    /// `Metaclass`, the class of every metaclass (instance of `Metaclass class`).
    pub metaclass_class: ClassId,
    /// `Number`.
    pub number_class: ClassId,
    /// `String`.
    pub string_class: ClassId,
    /// `Nil`.
    pub nil_class: ClassId,
    /// `Bool`, the abstract boolean superclass
    /// ([ADR-0004](../../../docs/adr/0004-boolean-as-abstract-bool-with-true-false.md)). No value is ever a
    /// direct instance of it; it holds the six sacred control-flow primitives
    /// that [`Self::true_class`]/[`Self::false_class`] inherit.
    pub bool_class: ClassId,
    /// `True`, the concrete singleton subclass of [`Self::bool_class`] whose sole
    /// inhabitant is the `true` immediate
    /// ([ADR-0004](../../../docs/adr/0004-boolean-as-abstract-bool-with-true-false.md)). Selected by
    /// [`Value::class`](crate::value::Value::class), so `true.class == True`.
    pub true_class: ClassId,
    /// `False`, the concrete singleton subclass of [`Self::bool_class`] whose sole
    /// inhabitant is the `false` immediate
    /// ([ADR-0004](../../../docs/adr/0004-boolean-as-abstract-bool-with-true-false.md)). Selected by
    /// [`Value::class`](crate::value::Value::class), so `false.class == False`.
    pub false_class: ClassId,
    /// `Method`.
    pub method_class: ClassId,
    /// `Function`.
    pub function_class: ClassId,
    /// `Block`.
    pub block_class: ClassId,
    /// `Symbol`.
    pub symbol_class: ClassId,
    /// `Module`.
    pub module_class: ClassId,
    /// `System`.
    pub system_class: ClassId,
    /// `Option`, the abstract absence type
    /// ([ADR-0007](../../../docs/adr/0007-option-some-none.md)); superclass of
    /// `Some` and `None`, and holder of the `match(some:none:)` eliminator.
    pub option_class: ClassId,
    /// `Some`, the present-value `Option` subclass (one field, `_value`).
    pub some_class: ClassId,
    /// `None`, the absent-value `Option` subclass. Its sole instance is
    /// [`Self::none_singleton`].
    pub none_class: ClassId,
    /// The single shared `None` object (an instance of [`Self::none_class`]).
    ///
    /// Reused for every surfaced absence so `None` is identity-comparable and
    /// zero-allocation ([ADR-0007](../../../docs/adr/0007-option-some-none.md));
    /// [`sentinel_to_option`](crate::value::sentinel_to_option) hands back a
    /// [`Value::Obj`](crate::value::Value::Obj) over this handle, and the `None`
    /// global (`VM::install_core`) is bound to it.
    pub none_singleton: ObjRef,
    /// `List`, the native array-backed kernel list
    /// ([ADR-0020](../../../docs/adr/0020-kernel-list-native-array-protocol.md)).
    /// A dedicated [`crate::heap::Object::List`] heap variant, not an
    /// `InstanceObject` — see [`crate::list::ListObject`].
    pub list_class: ClassId,
    /// `Map`, the native insertion-ordered hash map
    /// ([ADR-0032](../../../docs/adr/0032-collections-representation-and-literals.md) §1,
    /// [ADR-0039](../../../docs/adr/0039-amend-floor-admit-collection-container-primitives.md)).
    /// A dedicated [`crate::heap::Object::Map`] heap variant over
    /// [`crate::map::MapObject`] — mutable, so it inherits identity
    /// `Object#hash` and is not a valid `Map`/`Set` key (Q5).
    pub map_class: ClassId,
    /// `Set`, the native hash set — a keys-only [`Self::map_class`] sibling
    /// sharing [`crate::map::MapObject`]'s backing struct (DEC-CT-B), reached
    /// through the distinct [`crate::heap::Object::Set`] heap variant.
    pub set_class: ClassId,
    /// `Message`, the reified message-send handed to `doesNotUnderstand(_:)`
    /// on a lookup miss (method-lookup.md §2, ADR-0012). An ordinary
    /// fixed-slot [`InstanceObject`](crate::instance::InstanceObject) built by
    /// [`VM::new_message`](crate::vm::VM::new_message); its four slots hold
    /// `selector`/`name`/`labels`/`args`.
    pub message_class: ClassId,
    /// `Error`, the raisable root of the surface error hierarchy
    /// ([ADR-0008](../../../docs/adr/0008-layered-exceptions-and-result.md),
    /// U-CORE-6). Holds one field (`_message`, slot 0) and the native
    /// `message`/`raise` primitives; `raise` initiates the unified unwind's
    /// `Raise` payload ([`crate::error::RuntimeError::Raise`]). Only `Error`
    /// and its subclasses respond to `raise`.
    pub error_class: ClassId,
    /// `MessageNotUnderstood`, the sole surface subclass of
    /// [`Self::error_class`] this unit reifies — raised by the default
    /// `doesNotUnderstand(_:)` handler
    /// ([`object_does_not_understand`])
    /// on a genuine dispatch miss (method-lookup.md §2, ADR-0012, U-CORE-6).
    /// Adds one field beyond `Error`'s `_message`: `_reifiedMessage` (slot 1),
    /// the reified `Message` that missed ([`Self::message_class`]).
    pub message_not_understood_class: ClassId,
    /// `Fiber`, the sole concurrency primitive
    /// ([ADR-0030](../../../docs/adr/0030-fibers-and-futures-cooperative-concurrency.md)).
    /// A dedicated [`crate::heap::Object::Fiber`] heap variant — see
    /// [`crate::heap::FiberObject`] — not an `InstanceObject`.
    pub fiber_class: ClassId,
    /// `CannotYieldAcrossNativeFrame`, the catchable error the restricted-yield
    /// guard raises when a `Fiber#yield` is attempted across a re-entrant
    /// native frame (ADR-0030 §4, D-FIB-1). A direct subclass of
    /// [`Self::error_class`].
    pub cannot_yield_across_native_frame_class: ClassId,
}
