use crate::heap::ArgumentPackBuilderObject;
use crate::heap::BlockObject;
use crate::heap::BytesObject;
use crate::heap::ClassObject;
use crate::heap::ClosureObject;
use crate::heap::InstanceObject;
use crate::heap::ListObject;
use crate::heap::MapObject;
use crate::heap::ModuleObject;
use crate::heap::RangeObject;
use crate::heap::RecordLiteralBuilderObject;
use crate::heap::RecordObject;
use crate::heap::StringObject;
use crate::heap::TupleObject;
use crate::heap::Upvalue;
use crate::method::MethodObject;

use super::object::{BoundMethodObject, FamilyObject};
use super::{ClassId, FiberObject, Heap, ObjRef, Object};

impl Heap {
    pub fn record_literal_builder(&self, id: ObjRef) -> &RecordLiteralBuilderObject {
        match self.get(id) {
            Object::RecordLiteralBuilder(builder) => builder,
            _ => panic!("ObjRef {id:?} is not a RecordLiteralBuilderObject"),
        }
    }

    pub fn record_literal_builder_mut(&mut self, id: ObjRef) -> &mut RecordLiteralBuilderObject {
        match self.get_mut(id) {
            Object::RecordLiteralBuilder(builder) => builder,
            _ => panic!("ObjRef {id:?} is not a RecordLiteralBuilderObject"),
        }
    }

    pub fn as_record_literal_builder(&self, id: ObjRef) -> Option<&RecordLiteralBuilderObject> {
        match self.objects.get(id) {
            Some(Object::RecordLiteralBuilder(builder)) => Some(builder),
            _ => None,
        }
    }

    pub fn pack_builder(&self, id: ObjRef) -> &ArgumentPackBuilderObject {
        match self.get(id) {
            Object::PackBuilder(builder) => builder,
            _ => panic!("ObjRef {id:?} is not an ArgumentPackBuilderObject"),
        }
    }
    pub fn pack_builder_mut(&mut self, id: ObjRef) -> &mut ArgumentPackBuilderObject {
        match self.get_mut(id) {
            Object::PackBuilder(builder) => builder,
            _ => panic!("ObjRef {id:?} is not an ArgumentPackBuilderObject"),
        }
    }
    pub fn as_pack_builder(&self, id: ObjRef) -> Option<&ArgumentPackBuilderObject> {
        match self.objects.get(id) {
            Some(Object::PackBuilder(builder)) => Some(builder),
            _ => None,
        }
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

    /// Mutably borrows the [`ClosureObject`] behind `id`.
    pub fn closure_mut(&mut self, id: ObjRef) -> &mut ClosureObject {
        match self.get_mut(id) {
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

    /// Borrows the [`BytesObject`] behind `id`.
    ///
    /// # Panics
    ///
    /// Panics if `id` is stale or does not refer to an [`Object::Bytes`].
    pub fn bytes(&self, id: ObjRef) -> &BytesObject {
        match self.get(id) {
            Object::Bytes(bytes) => bytes,
            _ => panic!("ObjRef {id:?} is not a BytesObject"),
        }
    }

    /// Mutably borrows the [`BytesObject`] behind `id`.
    ///
    /// # Panics
    ///
    /// Panics if `id` is stale or does not refer to an [`Object::Bytes`].
    pub fn bytes_mut(&mut self, id: ObjRef) -> &mut BytesObject {
        match self.get_mut(id) {
            Object::Bytes(bytes) => bytes,
            _ => panic!("ObjRef {id:?} is not a BytesObject"),
        }
    }

    /// Returns the [`BytesObject`] behind `id`, or `None` if it is not one.
    pub fn as_bytes(&self, id: ObjRef) -> Option<&BytesObject> {
        match self.objects.get(id) {
            Some(Object::Bytes(bytes)) => Some(bytes),
            _ => None,
        }
    }

    /// Borrows the [`MapObject`] behind `id`.
    ///
    /// # Panics
    ///
    /// Panics if `id` is stale or does not refer to an [`Object::Map`].
    pub fn map(&self, id: ObjRef) -> &MapObject {
        match self.get(id) {
            Object::Map(map) => map,
            _ => panic!("ObjRef {id:?} is not an Object::Map"),
        }
    }

    /// Mutably borrows the [`MapObject`] behind `id`.
    ///
    /// # Panics
    ///
    /// Panics if `id` is stale or does not refer to an [`Object::Map`].
    pub fn map_mut(&mut self, id: ObjRef) -> &mut MapObject {
        match self.get_mut(id) {
            Object::Map(map) => map,
            _ => panic!("ObjRef {id:?} is not an Object::Map"),
        }
    }

    /// Returns the [`MapObject`] behind `id`, or `None` if it is not an
    /// [`Object::Map`].
    pub fn as_map(&self, id: ObjRef) -> Option<&MapObject> {
        match self.objects.get(id) {
            Some(Object::Map(map)) => Some(map),
            _ => None,
        }
    }

    /// Borrows the [`MapObject`] behind `id`, treating it as an
    /// [`Object::Set`]'s keys-only backing store.
    ///
    /// # Panics
    ///
    /// Panics if `id` is stale or does not refer to an [`Object::Set`].
    pub fn set(&self, id: ObjRef) -> &MapObject {
        match self.get(id) {
            Object::Set(set) => set,
            _ => panic!("ObjRef {id:?} is not an Object::Set"),
        }
    }

    /// Mutably borrows the [`MapObject`] behind `id`, treating it as an
    /// [`Object::Set`]'s keys-only backing store.
    ///
    /// # Panics
    ///
    /// Panics if `id` is stale or does not refer to an [`Object::Set`].
    pub fn set_mut(&mut self, id: ObjRef) -> &mut MapObject {
        match self.get_mut(id) {
            Object::Set(set) => set,
            _ => panic!("ObjRef {id:?} is not an Object::Set"),
        }
    }

    /// Returns the [`MapObject`] behind `id`, or `None` if it is not an
    /// [`Object::Set`].
    pub fn as_set(&self, id: ObjRef) -> Option<&MapObject> {
        match self.objects.get(id) {
            Some(Object::Set(set)) => Some(set),
            _ => None,
        }
    }

    /// Borrows the [`TupleObject`] behind `id`.
    ///
    /// # Panics
    ///
    /// Panics if `id` is stale or does not refer to an [`Object::Tuple`].
    pub fn tuple(&self, id: ObjRef) -> &TupleObject {
        match self.get(id) {
            Object::Tuple(tuple) => tuple,
            _ => panic!("ObjRef {id:?} is not an Object::Tuple"),
        }
    }

    /// Returns the [`TupleObject`] behind `id`, or `None` if it is not an
    /// [`Object::Tuple`].
    ///
    /// There is deliberately no `tuple_mut` — `Tuple` is immutable by
    /// representation (`docs/spec/v0.2/core/tuple-and-range.md` §1); a mutable
    /// accessor here would be the one way a later diff could accidentally
    /// reintroduce mutation.
    pub fn as_tuple(&self, id: ObjRef) -> Option<&TupleObject> {
        match self.objects.get(id) {
            Some(Object::Tuple(tuple)) => Some(tuple),
            _ => None,
        }
    }

    /// Borrows the immutable [`RecordObject`] behind `id`.
    pub fn record(&self, id: ObjRef) -> &RecordObject {
        match self.get(id) {
            Object::Record(record) => record,
            _ => panic!("ObjRef {id:?} is not an Object::Record"),
        }
    }

    /// Returns the immutable [`RecordObject`] behind `id`, if present.
    /// There is deliberately no `record_mut`.
    pub fn as_record(&self, id: ObjRef) -> Option<&RecordObject> {
        match self.objects.get(id) {
            Some(Object::Record(record)) => Some(record),
            _ => None,
        }
    }

    /// Borrows the [`RangeObject`] behind `id`.
    ///
    /// # Panics
    ///
    /// Panics if `id` is stale or does not refer to an [`Object::Range`].
    pub fn range(&self, id: ObjRef) -> &RangeObject {
        match self.get(id) {
            Object::Range(range) => range,
            _ => panic!("ObjRef {id:?} is not an Object::Range"),
        }
    }

    /// Returns the [`RangeObject`] behind `id`, or `None` if it is not an
    /// [`Object::Range`].
    ///
    /// There is deliberately no `range_mut`: bound descriptors are immutable.
    pub fn as_range(&self, id: ObjRef) -> Option<&RangeObject> {
        match self.objects.get(id) {
            Some(Object::Range(range)) => Some(range),
            _ => None,
        }
    }

    /// Borrows the [`FiberObject`] behind `id`
    /// ([ADR-0030](../../../docs/adr/accepted/0030-fibers-and-futures-cooperative-concurrency.md) §2).
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
    /// ([ADR-0030](../../../docs/adr/accepted/0030-fibers-and-futures-cooperative-concurrency.md) §2).
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
    /// ([ADR-0030](../../../docs/adr/accepted/0030-fibers-and-futures-cooperative-concurrency.md) §2).
    pub fn as_fiber(&self, id: ObjRef) -> Option<&FiberObject> {
        match self.objects.get(id) {
            Some(Object::Fiber(fiber)) => Some(fiber),
            _ => None,
        }
    }

    /// Borrows the [`FamilyObject`] behind `id` (selectors.md §3, U16-Open,
    /// U16-Pinned).
    ///
    /// # Panics
    ///
    /// Panics if `id` is stale or does not refer to an [`Object::Family`].
    pub fn family(&self, id: ObjRef) -> &FamilyObject {
        match self.get(id) {
            Object::Family(family) => family,
            _ => panic!("ObjRef {id:?} is not an Object::Family"),
        }
    }

    /// Returns the [`FamilyObject`] behind `id`, or `None` if it is not one.
    ///
    /// There is deliberately no `family_mut` — a `Family` is immutable once
    /// constructed (all fields are `Copy`, set once at [`crate::bytecode::Bytecode::MakeFamily`]).
    pub fn as_family(&self, id: ObjRef) -> Option<&FamilyObject> {
        match self.objects.get(id) {
            Some(Object::Family(family)) => Some(family),
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

    /// Returns the [`ClosureObject`] behind `id`, or `None` if it is not one.
    pub fn as_closure(&self, id: ObjRef) -> Option<&ClosureObject> {
        match self.objects.get(id) {
            Some(Object::Closure(closure)) => Some(closure),
            _ => None,
        }
    }

    /// Borrows the [`num_bigint::BigInt`] behind `id`.
    ///
    /// # Panics
    ///
    /// Panics if `id` is stale or does not refer to an [`Object::LargeInt`].
    pub fn large_int(&self, id: ObjRef) -> &num_bigint::BigInt {
        match self.get(id) {
            Object::LargeInt(val) => val,
            _ => panic!("ObjRef {id:?} is not a LargeInt"),
        }
    }

    /// Returns the [`num_bigint::BigInt`] behind `id`, or `None` if it is not one.
    pub fn as_large_int(&self, id: ObjRef) -> Option<&num_bigint::BigInt> {
        match self.objects.get(id) {
            Some(Object::LargeInt(val)) => Some(val),
            _ => None,
        }
    }
}
