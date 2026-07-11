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

use crate::block::BlockObject;
use crate::class::ClassObject;
use crate::closure::ClosureObject;
use crate::instance::InstanceObject;
use crate::method::MethodObject;
use crate::module::ModuleObject;
use crate::string::StringObject;
use crate::upvalue::Upvalue;

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
    /// A heap-allocated upvalue cell ([`Upvalue`]).
    Upvalue(Upvalue),
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
