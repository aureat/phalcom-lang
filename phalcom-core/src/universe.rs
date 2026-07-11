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

use crate::heap::{ClassId, Heap};
use crate::interner::Symbol;
use crate::method::MethodObject;
use crate::method::SignatureKind;
use crate::primitive::boolean::{bool_and, bool_class_new, bool_if_false, bool_if_true, bool_if_true_if_false, bool_not, bool_or};
use crate::primitive::block::{block_arity, block_call, block_call_with, block_name, block_while_true};
use crate::primitive::class::{class_add, class_new, class_set_superclass, class_superclass};
use crate::primitive::method::method_class_new;
use crate::primitive::module::module_class_new;
use crate::primitive::nil::nil_class_new;
use crate::primitive::number::{
    number_add, number_class_new, number_div, number_ge, number_gt, number_le, number_lt, number_mod, number_mul, number_negated, number_sub,
};
use crate::primitive::object::{object_class, object_class_new, object_eq, object_name, object_neq, object_set_class};
use crate::primitive::primitive;
use crate::primitive::primitive_static;
use crate::primitive::string::{string_add, string_class_new};
use crate::primitive::symbol::{symbol_class_new, symbol_tostring};
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
        let method_class = make_core_class(heap, "Method", object_class, metaclass_class);
        let function_class = make_core_class(heap, "Function", object_class, metaclass_class);
        let block_class = make_core_class(heap, "Block", function_class, metaclass_class);
        let symbol_class = make_core_class(heap, "Symbol", object_class, metaclass_class);
        let module_class = make_core_class(heap, "Module", object_class, metaclass_class);
        let system_class = make_core_class(heap, "System", object_class, metaclass_class);

        CoreClasses {
            object_class,
            behavior_class,
            class_class,
            metaclass_class,
            number_class,
            string_class,
            nil_class,
            bool_class,
            method_class,
            function_class,
            block_class,
            symbol_class,
            module_class,
            system_class,
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
        primitive!(vm, object_cls, "toString", SignatureKind::Getter, object_name);
        primitive_static!(vm, object_cls, "new", SignatureKind::Method(0), object_class_new);
        // U5 (control-flow.md §1): `==`/`!=` are ordinary sends, not opcodes.
        primitive!(vm, object_cls, "==", SignatureKind::Method(1), object_eq);
        primitive!(vm, object_cls, "!=", SignatureKind::Method(1), object_neq);

        let behavior_cls = vm.universe.classes.behavior_class;
        primitive!(vm, behavior_cls, "superclass", SignatureKind::Getter, class_superclass);
        primitive!(vm, behavior_cls, "superclass", SignatureKind::Setter, class_set_superclass);

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
        primitive_static!(vm, number_cls, "new", SignatureKind::Method(0), number_class_new);
        primitive_static!(vm, number_cls, "new", SignatureKind::Method(1), number_class_new);

        let string_cls = vm.universe.classes.string_class;
        primitive!(vm, string_cls, "+", SignatureKind::Method(1), string_add);
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

        let symbol_cls = vm.universe.classes.symbol_class;
        primitive!(vm, symbol_cls, "toString", SignatureKind::Getter, symbol_tostring);
        primitive_static!(vm, symbol_cls, "new", SignatureKind::Method(1), symbol_class_new);

        let nil_cls = vm.universe.classes.nil_class;
        primitive_static!(vm, nil_cls, "new", SignatureKind::Method(0), nil_class_new);

        let bool_cls = vm.universe.classes.bool_class;
        primitive_static!(vm, bool_cls, "new", SignatureKind::Method(1), nil_class_new);

        let method_cls = vm.universe.classes.method_class;
        primitive_static!(vm, method_cls, "new", SignatureKind::Method(1), method_class_new);

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

        // Parallel rule holding for an ordinary core class (Number).
        let number_meta = heap.class(c.number_class).class;
        let expected_number_meta_super = heap.class(c.object_class).class;
        if heap.class(number_meta).superclass != Some(expected_number_meta_super) {
            return Err("Number.class.superclass should be Object.class (parallel rule)".to_string());
        }

        // Every metaclass's superclass chain terminates (bounded walk guards
        // against a cycle turning into a hang instead of a failure).
        let mut current = number_meta;
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
    /// `Bool`.
    pub bool_class: ClassId,
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
}
