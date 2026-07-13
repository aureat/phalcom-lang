use crate::heap::{ClassId, Heap, ObjRef};

use super::Universe;

impl Universe {
    /// Allocates and wires the kernel class tower via allocate-then-patch.
    ///
    /// Follows the seven-step order of `object-model.md` §6: allocate the 8
    /// apex rows bare, wire instance-of, wire instance-side superclasses, wire
    /// metaclass-side superclasses by the parallel rule
    /// ([ADR-0002](../../../docs/adr/0002-metaclass-tower-parallel-rule.md)),
    /// then create the remaining core classes through `make_core_class`.
    /// Step 7 (`verify_invariants`) is run by the caller ([`crate::vm::VM::new`]) once
    /// primitives are installed.
    pub fn create_core_classes(heap: &mut Heap) -> CoreClasses {
        // 1. Allocate the 8 apex rows bare (object-model.md §6 step 1).
        let object_class = heap.alloc_class(crate::heap::ClassObject::bare("Object"));
        let behavior_class = heap.alloc_class(crate::heap::ClassObject::bare("Behavior"));
        let class_class = heap.alloc_class(crate::heap::ClassObject::bare("Class"));
        let metaclass_class = heap.alloc_class(crate::heap::ClassObject::bare("Metaclass"));
        let object_metaclass = heap.alloc_class(crate::heap::ClassObject::bare("Object class"));
        let behavior_metaclass = heap.alloc_class(crate::heap::ClassObject::bare("Behavior class"));
        let class_metaclass = heap.alloc_class(crate::heap::ClassObject::bare("Class class"));
        let metaclass_metaclass = heap.alloc_class(crate::heap::ClassObject::bare("Metaclass class"));

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
        let none_singleton = heap.alloc(crate::heap::Object::Instance(crate::heap::InstanceObject::new(none_class, 0)));

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

        // Kernel `Tuple` (ADR-0032 §1, ADR-0039, U-COLLTYPES Phase 2): a fixed
        // immutable slice — the opposite mutability corner from `List`/`Map`/
        // `Set` (Q5: immutable ⇒ value-hashable, a valid `Map`/`Set` key).
        let tuple_class = make_core_class(heap, "Tuple", object_class, metaclass_class);

        // Kernel `Range` (ADR-0032 §1, ADR-0039, U-COLLTYPES Phase 3): a lazy
        // numeric interval, three bound fields, no element storage (RG-2).
        // Immutable ⇒ value-hashable, a valid `Map`/`Set` key (Q5).
        let range_class = make_core_class(heap, "Range", object_class, metaclass_class);

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

        // `Family` (selectors.md §3, U16-Open, ADR-0047): the callable value
        // a `::` method reference produces, a native `Object::Family` heap
        // variant (no `Value::Family` arm) sitting directly under `Object`,
        // mirroring `Fiber`/`List`.
        let family_class = make_core_class(heap, "Family", object_class, metaclass_class);

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
            tuple_class,
            range_class,
            message_class,
            error_class,
            message_not_understood_class,
            fiber_class,
            cannot_yield_across_native_frame_class,
            family_class,
        }
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

    let metaclass = heap.alloc_class(crate::heap::ClassObject::bare(&format!("{name} class")));
    {
        let meta = heap.class_mut(metaclass);
        meta.class = metaclass_class;
        meta.superclass = Some(metaclass_superclass);
    }
    let class = heap.alloc_class(crate::heap::ClassObject::bare(name));
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
    /// `InstanceObject` — see [`crate::heap::ListObject`].
    pub list_class: ClassId,
    /// `Map`, the native insertion-ordered hash map
    /// ([ADR-0032](../../../docs/adr/0032-collections-representation-and-literals.md) §1,
    /// [ADR-0039](../../../docs/adr/0039-amend-floor-admit-collection-container-primitives.md)).
    /// A dedicated [`crate::heap::Object::Map`] heap variant over
    /// [`crate::heap::MapObject`] — mutable, so it inherits identity
    /// `Object#hash` and is not a valid `Map`/`Set` key (Q5).
    pub map_class: ClassId,
    /// `Set`, the native hash set — a keys-only [`Self::map_class`] sibling
    /// sharing [`crate::heap::MapObject`]'s backing struct (DEC-CT-B), reached
    /// through the distinct [`crate::heap::Object::Set`] heap variant.
    pub set_class: ClassId,
    /// `Tuple`, the native fixed-arity immutable product
    /// ([ADR-0032](../../../docs/adr/0032-collections-representation-and-literals.md) §1,
    /// [ADR-0039](../../../docs/adr/0039-amend-floor-admit-collection-container-primitives.md)).
    /// A dedicated [`crate::heap::Object::Tuple`] heap variant over
    /// [`crate::heap::TupleObject`] — immutable, so it value-hashes and is a
    /// valid `Map`/`Set` key (Q5).
    pub tuple_class: ClassId,
    /// `Range`, the native lazy numeric interval
    /// ([ADR-0032](../../../docs/adr/0032-collections-representation-and-literals.md) §1,
    /// [ADR-0039](../../../docs/adr/0039-amend-floor-admit-collection-container-primitives.md)).
    /// A dedicated [`crate::heap::Object::Range`] heap variant over
    /// [`crate::heap::RangeObject`] — three bound fields, no element
    /// storage (RG-2 laziness). Immutable ⇒ value-hashable, a valid
    /// `Map`/`Set` key (Q5).
    pub range_class: ClassId,
    /// `Message`, the reified message-send handed to `doesNotUnderstand(_:)`
    /// on a lookup miss (method-lookup.md §2, ADR-0012). An ordinary
    /// fixed-slot [`InstanceObject`](crate::heap::InstanceObject) built by
    /// [`crate::vm::VM::new_message`]; its four slots hold
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
    /// ([`crate::primitive::object::object_does_not_understand`])
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
    /// `Family`, the callable value a `::` method reference produces
    /// (selectors.md §3, U16-Open, [ADR-0047](../../../docs/adr/0047-amend-floor-admit-family-call-router.md)).
    /// Backed by [`crate::heap::Object::Family`] — no `Value::Family` arm.
    pub family_class: ClassId,
}
