//! Bootstrap of the kernel class tower and installation of core primitives.
//!
//! The kernel is a cyclic graph (`Metaclass` is an instance of itself, the tower
//! closes at the top; `object-model.md` §5–6). Under
//! [ADR-0009](../../../docs/adr/0009-handle-arena-heap.md) that cycle is built by
//! **allocate-then-patch**: every class row is first allocated bare in the
//! [`Heap`] to obtain its [`ClassId`], then its `class` and
//! `superclass` handles are written in place. This supersedes ADR-0002's
//! `Rc::new_cyclic` mechanism while preserving the *exact* observable wiring —
//! the metaclass parallel-superclass deviation (F2) is deliberately left as-is
//! for a later unit (see `tests/invariants.rs`).

use crate::heap::{ClassId, Heap};
use crate::method::MethodObject;
use crate::method::SignatureKind;
use crate::primitive::boolean::bool_class_new;
use crate::primitive::class::{class_add, class_new, class_set_superclass, class_superclass};
use crate::primitive::method::method_class_new;
use crate::primitive::module::module_class_new;
use crate::primitive::nil::nil_class_new;
use crate::primitive::number::{number_add, number_class_new, number_div};
use crate::primitive::object::{object_class, object_class_new, object_name, object_set_class};
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
}

impl Universe {
    /// Bootstraps the core class tower into `heap` and returns the [`Universe`].
    pub fn new(heap: &mut Heap) -> Self {
        Universe {
            classes: Self::create_core_classes(heap),
        }
    }

    /// Allocates and wires the kernel class tower via allocate-then-patch.
    ///
    /// Reproduces today's observable wiring exactly (`object-model.md` §5–6,
    /// [ADR-0002](../../../docs/adr/0002-metaclass-tower-parallel-rule.md)):
    /// `Metaclass.class == Metaclass`, `Class.superclass == Object`, and every
    /// core class `X` gets a metaclass `X.class` that is an instance of
    /// `Metaclass` with superclass `Class`, while `X.superclass == Object`.
    pub fn create_core_classes(heap: &mut Heap) -> CoreClasses {
        // 1. Allocate the four apex rows bare, capturing their handles.
        let metaclass_class = heap.alloc_class(crate::class::ClassObject::bare("Metaclass"));
        let class_class = heap.alloc_class(crate::class::ClassObject::bare("Class"));
        let object_class = heap.alloc_class(crate::class::ClassObject::bare("Object"));
        let object_metaclass = heap.alloc_class(crate::class::ClassObject::bare("Object.class"));

        // 2. Patch their `class`/`superclass` handles in place.
        //    Metaclass: class == Metaclass (self-cycle), superclass == Class.
        {
            let metaclass = heap.class_mut(metaclass_class);
            metaclass.class = metaclass_class;
            metaclass.superclass = Some(class_class);
        }
        //    Class: class == Metaclass, superclass == Object.
        {
            let class = heap.class_mut(class_class);
            class.class = metaclass_class;
            class.superclass = Some(object_class);
        }
        //    Object: class == Object.class, no superclass (tower apex).
        {
            let object = heap.class_mut(object_class);
            object.class = object_metaclass;
            object.superclass = None;
        }
        //    Object.class: class == Metaclass, superclass == Class.
        {
            let object_meta = heap.class_mut(object_metaclass);
            object_meta.class = metaclass_class;
            object_meta.superclass = Some(class_class);
        }

        // 3. The remaining core classes, each with its own metaclass.
        let number_class = make_core_class(heap, "Number", object_class, metaclass_class, class_class);
        let string_class = make_core_class(heap, "String", object_class, metaclass_class, class_class);
        let nil_class = make_core_class(heap, "Nil", object_class, metaclass_class, class_class);
        let bool_class = make_core_class(heap, "Bool", object_class, metaclass_class, class_class);
        let method_class = make_core_class(heap, "Method", object_class, metaclass_class, class_class);
        let symbol_class = make_core_class(heap, "Symbol", object_class, metaclass_class, class_class);
        let module_class = make_core_class(heap, "Module", object_class, metaclass_class, class_class);
        let system_class = make_core_class(heap, "System", object_class, metaclass_class, class_class);

        CoreClasses {
            object_class,
            class_class,
            metaclass_class,
            number_class,
            string_class,
            nil_class,
            bool_class,
            method_class,
            symbol_class,
            module_class,
            system_class,
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

        let class_cls = vm.universe.classes.class_class;
        primitive!(vm, class_cls, "superclass", SignatureKind::Getter, class_superclass);
        primitive!(vm, class_cls, "superclass", SignatureKind::Setter, class_set_superclass);
        primitive!(vm, class_cls, "+", SignatureKind::Method(1), class_add);
        primitive!(vm, class_cls, "new", SignatureKind::Method(0), class_new);

        let number_cls = vm.universe.classes.number_class;
        primitive!(vm, number_cls, "+", SignatureKind::Method(1), number_add);
        primitive!(vm, number_cls, "/", SignatureKind::Method(1), number_div);
        primitive_static!(vm, number_cls, "new", SignatureKind::Method(0), number_class_new);
        primitive_static!(vm, number_cls, "new", SignatureKind::Method(1), number_class_new);

        let string_cls = vm.universe.classes.string_class;
        primitive!(vm, string_cls, "+", SignatureKind::Method(1), string_add);
        primitive_static!(vm, string_cls, "new", SignatureKind::Method(0), string_class_new);
        primitive_static!(vm, string_cls, "new", SignatureKind::Method(1), string_class_new);

        let bool_cls = vm.universe.classes.bool_class;
        primitive_static!(vm, bool_cls, "new", SignatureKind::Method(0), bool_class_new);
        primitive_static!(vm, bool_cls, "new", SignatureKind::Method(1), bool_class_new);

        let symbol_cls = vm.universe.classes.symbol_class;
        primitive!(vm, symbol_cls, "toString", SignatureKind::Getter, symbol_tostring);
        primitive_static!(vm, symbol_cls, "new", SignatureKind::Method(1), symbol_class_new);

        let nil_cls = vm.universe.classes.nil_class;
        primitive_static!(vm, nil_cls, "new", SignatureKind::Method(0), nil_class_new);

        let bool_cls = vm.universe.classes.bool_class;
        primitive_static!(vm, bool_cls, "new", SignatureKind::Method(1), nil_class_new);

        let method_cls = vm.universe.classes.method_class;
        primitive_static!(vm, method_cls, "new", SignatureKind::Method(1), method_class_new);

        let system_cls = vm.universe.classes.system_class;
        primitive_static!(vm, system_cls, "print", SignatureKind::Method(1), system_class_print);
        primitive_static!(vm, system_cls, "new", SignatureKind::Method(0), system_class_new);

        let module_cls = vm.universe.classes.module_class;
        primitive_static!(vm, module_cls, "new", SignatureKind::Method(0), module_class_new);
    }
}

/// Allocates a core class `name` (with its own metaclass) and wires it.
///
/// The metaclass `"{name}.class"` is an instance of `metaclass_class` with
/// superclass `class_class`; the class itself is an instance of that metaclass
/// with the given `superclass`. This reproduces the original
/// `create_core_class` wiring exactly (F2 preserved).
fn make_core_class(heap: &mut Heap, name: &str, superclass: ClassId, metaclass_class: ClassId, class_class: ClassId) -> ClassId {
    let metaclass = heap.alloc_class(crate::class::ClassObject::bare(&format!("{name}.class")));
    {
        let meta = heap.class_mut(metaclass);
        meta.class = metaclass_class;
        meta.superclass = Some(class_class);
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
    /// `Class`, the class of all classes' metaclasses' superclass chain.
    pub class_class: ClassId,
    /// `Metaclass`, instance of itself.
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
    /// `Symbol`.
    pub symbol_class: ClassId,
    /// `Module`.
    pub module_class: ClassId,
    /// `System`.
    pub system_class: ClassId,
}
