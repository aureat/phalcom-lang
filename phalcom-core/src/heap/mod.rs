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

mod accessors;
mod block;
mod class;
mod closure;
mod fiber;
mod instance;
mod list;
mod map;
mod module;
mod object;
mod range;
mod string;
mod tuple;
mod upvalue;

pub use block::BlockObject;
pub use class::{lookup_method_in_hierarchy, ClassObject};
pub use closure::ClosureObject;
pub use fiber::{FiberObject, FiberResumeMode, FiberStatus};
pub use instance::InstanceObject;
pub use list::ListObject;
pub use map::MapObject;
pub use module::{next_module_id, ModuleObject, ModuleId, CORE_MODULE_NAME, MAIN_MODULE_NAME, MAX_GLOBALS};
pub use object::{BoundMethodObject, FamilyObject, Object};
pub use range::RangeObject;
pub use string::StringObject;
pub use tuple::TupleObject;
pub use upvalue::Upvalue;

use slotmap::{new_key_type, SlotMap};

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

    /// Allocates an empty [`Object::Map`] and returns its [`ObjRef`].
    pub fn alloc_map(&mut self) -> ObjRef {
        self.objects.insert(Object::Map(MapObject::new()))
    }

    /// Allocates an empty [`Object::Set`] and returns its [`ObjRef`].
    pub fn alloc_set(&mut self) -> ObjRef {
        self.objects.insert(Object::Set(MapObject::new()))
    }

    /// Allocates an [`Object::Tuple`] from a fixed `elements` slice and
    /// returns its [`ObjRef`].
    pub fn alloc_tuple(&mut self, elements: Box<[crate::value::Value]>) -> ObjRef {
        self.objects.insert(Object::Tuple(TupleObject::new(elements)))
    }

    /// Allocates an [`Object::Range`] from its three bound fields and returns
    /// its [`ObjRef`].
    pub fn alloc_range(&mut self, start: crate::value::Value, end: crate::value::Value, inclusive: bool) -> ObjRef {
        self.objects.insert(Object::Range(RangeObject::new(start, end, inclusive)))
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
}
