//! Runtime materialization of canonical checked static-constant descriptors.

use crate::error::{PhResult, RuntimeError};
use crate::heap::{ArrayObject, Object, VectorObject};
use crate::modules::linkage::RuntimeStaticConstantRef;
use crate::modules::semantic_lowering::StaticConstantLoweringSpec;
use crate::value::Value;
use crate::vm::VM;
use phalcom_semantic::materialization::{MaterializationDescriptor, SequenceMaterializationKind};
use phalcom_semantic::static_value::StaticInt;

impl VM {
    pub(crate) fn static_constant_spec(&self, module: &phalcom_modules::ModuleId, name: &str) -> Option<StaticConstantLoweringSpec> {
        self.module_registry
            .get(module)
            .and_then(|record| self.heap.module(record.object).lowering.as_ref())
            .and_then(|lowering| lowering.static_constants.get(name))
            .cloned()
    }

    fn materialize_static_constant_value(&mut self, spec: &StaticConstantLoweringSpec) -> PhResult<Value> {
        self.materialize_descriptor(&spec.descriptor, &spec.id)
    }

    fn materialize_descriptor(
        &mut self,
        descriptor: &MaterializationDescriptor,
        owner: &phalcom_semantic::static_constant::StaticConstantId,
    ) -> PhResult<Value> {
        let value = match descriptor {
            MaterializationDescriptor::Unit => Value::unit(),
            MaterializationDescriptor::Bool(value) => Value::bool(*value),
            MaterializationDescriptor::Char(value) => Value::char(*value),
            MaterializationDescriptor::Int(StaticInt::Small(value)) => Value::int(*value),
            MaterializationDescriptor::Int(StaticInt::Large(value)) => Value::obj(self.heap.alloc(Object::LargeInt(value.as_ref().clone()))),
            MaterializationDescriptor::Float(value) => Value::float(value.to_f64()),
            MaterializationDescriptor::String(value) => {
                let mut string = String::new();
                string.try_reserve_exact(value.len()).map_err(|_| RuntimeError::ResourceLimit)?;
                string.push_str(value);
                self.alloc_string_value(string)
            }
            MaterializationDescriptor::Symbol(value) => Value::symbol(self.interner.intern(value.as_ref())),
            MaterializationDescriptor::Sequence { kind, elements } => {
                let mut values = Vec::new();
                values.try_reserve_exact(elements.len()).map_err(|_| RuntimeError::ResourceLimit)?;
                for element in elements.iter() {
                    values.push(self.materialize_descriptor(element, owner)?);
                }
                let object = match kind {
                    SequenceMaterializationKind::Vector => Object::Vector(VectorObject::new(values.into_boxed_slice())),
                    SequenceMaterializationKind::FrozenArray => Object::Array(ArrayObject::from_vec_frozen(values)),
                };
                Value::obj(self.heap.alloc(object))
            }
            MaterializationDescriptor::DeferredStructural(_) | MaterializationDescriptor::Unsupported(_) => {
                return Err(RuntimeError::Internal(format!("static constant {owner} has no supported runtime materialization")).into());
            }
        };
        // Keep every heap child alive while the enclosing descriptor is still
        // being assembled. The caller brackets one complete owner
        // materialization with a temp-root depth and releases these roots only
        // after the canonical module cache has taken ownership.
        self.push_temp_root(value);
        Ok(value)
    }

    /// Ensures a checked constant has one materialized value in its canonical
    /// owner module and returns its VM-local source identity.
    pub(crate) fn ensure_static_constant_materialized(&mut self, spec: &StaticConstantLoweringSpec) -> PhResult<RuntimeStaticConstantRef> {
        let owner_id = spec.id.module();
        let owner = self
            .module_registry
            .get(owner_id)
            .map(|record| record.object)
            .ok_or_else(|| RuntimeError::Internal(format!("static constant {} has no registered owner module", spec.id)))?;
        let name = self.interner.intern(spec.id.name());
        if self.heap.module(owner).static_constants.contains_key(&name) {
            return Ok(RuntimeStaticConstantRef { module: owner, name });
        }

        let root_depth = self.temp_root_depth();
        let value = match self.materialize_static_constant_value(spec) {
            Ok(value) => value,
            Err(error) => {
                self.truncate_temp_roots(root_depth);
                return Err(error);
            }
        };
        self.heap.module_mut(owner).static_constants.insert(name, value);
        self.truncate_temp_roots(root_depth);
        Ok(RuntimeStaticConstantRef { module: owner, name })
    }

    /// Resolves a canonical static-constant reference to its owner-held value.
    pub(crate) fn resolve_static_constant(&self, reference: RuntimeStaticConstantRef) -> PhResult<Value> {
        let name = self.resolve_symbol(reference.name);
        let Some(Object::Module(module)) = self.heap.try_get(reference.module) else {
            return Err(RuntimeError::Internal(format!("static constant {name} has no live owner module at {:?}", reference.module)).into());
        };
        module
            .static_constants
            .get(&reference.name)
            .copied()
            .ok_or_else(|| RuntimeError::Internal(format!("static constant {}::{name} is not materialized", module.id)).into())
    }

    /// Reconstructs the semantic symbol identity represented by a runtime
    /// constant reference, preserving the canonical owner across re-exports.
    pub(crate) fn runtime_static_constant_symbol(&self, reference: RuntimeStaticConstantRef) -> PhResult<phalcom_modules::SymbolId> {
        let name = self.resolve_symbol(reference.name);
        let Some(Object::Module(module)) = self.heap.try_get(reference.module) else {
            return Err(RuntimeError::Internal(format!("static constant {name} has no live owner module at {:?}", reference.module)).into());
        };
        Ok(phalcom_modules::SymbolId {
            module: module.id.clone(),
            name: name.into(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use phalcom_modules::{ModuleComponent, ModuleId, ModulePath};
    use phalcom_semantic::materialization::{DeferredStructuralKind, SequenceMaterializationKind};
    use phalcom_semantic::scalar::StaticInt;
    use phalcom_semantic::static_constant::StaticConstantId;
    use std::sync::Arc;

    #[test]
    fn canonical_materialization_reuses_values_and_fails_closed() {
        let mut vm = VM::new();
        let owner_id = vm.module_registry.iter().next().expect("bootstrapped owner module").0.clone();

        let string_spec = StaticConstantLoweringSpec {
            id: StaticConstantId::new(owner_id.clone(), "__static_test_string"),
            descriptor: MaterializationDescriptor::String(Arc::from("cached")),
        };
        let string_first = vm.ensure_static_constant_materialized(&string_spec).expect("first String materialization");
        let string_value_first = vm.resolve_static_constant(string_first).expect("first String read");
        let string_second = vm.ensure_static_constant_materialized(&string_spec).expect("reused String materialization");
        let string_value_second = vm.resolve_static_constant(string_second).expect("reused String read");
        assert_eq!(string_first, string_second);
        assert_eq!(string_value_first, string_value_second, "String storage is reused by object identity");

        let large_spec = StaticConstantLoweringSpec {
            id: StaticConstantId::new(owner_id.clone(), "__static_test_large_int"),
            descriptor: MaterializationDescriptor::Int(StaticInt::new("999999999999999999999999999999".parse().unwrap())),
        };
        let large_first = vm.ensure_static_constant_materialized(&large_spec).expect("first LargeInt materialization");
        let large_value_first = vm.resolve_static_constant(large_first).expect("first LargeInt read");
        let large_second = vm.ensure_static_constant_materialized(&large_spec).expect("reused LargeInt materialization");
        let large_value_second = vm.resolve_static_constant(large_second).expect("reused LargeInt read");
        assert_eq!(large_first, large_second);
        assert_eq!(large_value_first, large_value_second, "LargeInt storage is reused by object identity");

        let unsupported_spec = StaticConstantLoweringSpec {
            id: StaticConstantId::new(owner_id, "__static_test_deferred"),
            descriptor: MaterializationDescriptor::DeferredStructural(DeferredStructuralKind::Tuple),
        };
        let unsupported_error = vm
            .ensure_static_constant_materialized(&unsupported_spec)
            .expect_err("deferred value fails closed");
        assert!(unsupported_error.to_string().contains(&unsupported_spec.id.to_string()));

        let missing_owner = ModuleId::universe(ModulePath::from_components(vec![
            ModuleComponent::from_identifier("missing_static_owner").unwrap(),
        ]));
        let missing_spec = StaticConstantLoweringSpec {
            id: StaticConstantId::new(missing_owner, "missing"),
            descriptor: MaterializationDescriptor::Bool(true),
        };
        let missing_error = vm
            .ensure_static_constant_materialized(&missing_spec)
            .expect_err("missing owner returns an error");
        assert!(missing_error.to_string().contains(&missing_spec.id.to_string()));
    }

    #[test]
    fn sequence_materialization_builds_native_objects_and_roots_nested_children() {
        let mut vm = VM::new();
        let owner_id = vm.module_registry.iter().next().expect("bootstrapped owner module").0.clone();

        let vector_spec = StaticConstantLoweringSpec {
            id: StaticConstantId::new(owner_id.clone(), "__static_test_vector"),
            descriptor: MaterializationDescriptor::Sequence {
                kind: SequenceMaterializationKind::Vector,
                elements: Arc::from([
                    MaterializationDescriptor::Int(StaticInt::new(1.into())),
                    MaterializationDescriptor::Int(StaticInt::new(2.into())),
                ]),
            },
        };
        let vector_reference = vm.ensure_static_constant_materialized(&vector_spec).expect("Vector materialization");
        let vector_value = vm.resolve_static_constant(vector_reference).expect("Vector owner value");
        let vector_object = vector_value.as_obj().expect("heap Vector");
        assert_eq!(vm.heap.vector(vector_object).elements(), &[Value::int(1), Value::int(2)]);

        let array_spec = StaticConstantLoweringSpec {
            id: StaticConstantId::new(owner_id.clone(), "__static_test_frozen_array"),
            descriptor: MaterializationDescriptor::Sequence {
                kind: SequenceMaterializationKind::FrozenArray,
                elements: Arc::from([MaterializationDescriptor::Int(StaticInt::new(3.into()))]),
            },
        };
        let array_reference = vm.ensure_static_constant_materialized(&array_spec).expect("Array materialization");
        let array_value = vm.resolve_static_constant(array_reference).expect("Array owner value");
        let array_object = array_value.as_obj().expect("heap Array");
        assert!(vm.heap.array(array_object).is_frozen(), "const Array is frozen before owner publication");
        assert_eq!(vm.heap.array(array_object).get(0), Some(Value::int(3)));
        assert_eq!(
            vm.heap.array_mut(array_object).set(0, Value::int(4)),
            Err(crate::heap::ArrayMutationError::Frozen)
        );

        let large: num_bigint::BigInt = "999999999999999999999999999999999999".parse().unwrap();
        let nested_spec = StaticConstantLoweringSpec {
            id: StaticConstantId::new(owner_id.clone(), "__static_test_nested_sequence"),
            descriptor: MaterializationDescriptor::Sequence {
                kind: SequenceMaterializationKind::FrozenArray,
                elements: Arc::from([MaterializationDescriptor::Sequence {
                    kind: SequenceMaterializationKind::Vector,
                    elements: Arc::from([
                        MaterializationDescriptor::String(Arc::from("retained nested string")),
                        MaterializationDescriptor::Int(StaticInt::new(large.clone())),
                    ]),
                }]),
            },
        };
        let nested_reference = vm.ensure_static_constant_materialized(&nested_spec).expect("nested sequence materialization");
        let nested_value = vm.resolve_static_constant(nested_reference).expect("nested owner value");
        let nested_root = nested_value.as_obj().expect("heap Array root");
        let nested_vector = vm.heap.array(nested_root).get(0).unwrap().as_obj().expect("nested Vector");
        let nested_string = vm.heap.vector(nested_vector).get(0).unwrap().as_obj().expect("nested String");
        let nested_large = vm.heap.vector(nested_vector).get(1).unwrap().as_obj().expect("nested LargeInt");

        let repeated_reference = vm.ensure_static_constant_materialized(&nested_spec).expect("cached nested value");
        assert_eq!(repeated_reference, nested_reference);
        assert_eq!(vm.resolve_static_constant(repeated_reference).unwrap(), nested_value);

        vm.force_gc();
        assert!(vm.heap.as_array(nested_root).is_some(), "canonical owner cache roots the sequence root");
        assert!(vm.heap.as_vector(nested_vector).is_some(), "nested sequence traces its Vector child");
        assert!(matches!(vm.heap.get(nested_string), Object::Str(_)), "nested String survives forced GC");
        assert_eq!(vm.heap.as_large_int(nested_large), Some(&large), "nested LargeInt survives forced GC");
    }

    #[test]
    fn failed_sequence_materialization_does_not_publish_partial_owner_value() {
        let mut vm = VM::new();
        let owner_id = vm.module_registry.iter().next().expect("bootstrapped owner module").0.clone();
        let spec = StaticConstantLoweringSpec {
            id: StaticConstantId::new(owner_id.clone(), "__static_test_failed_sequence"),
            descriptor: MaterializationDescriptor::Sequence {
                kind: SequenceMaterializationKind::FrozenArray,
                elements: Arc::from([
                    MaterializationDescriptor::String(Arc::from("temporary child")),
                    MaterializationDescriptor::DeferredStructural(DeferredStructuralKind::Tuple),
                ]),
            },
        };
        let name = vm.get_or_intern(spec.id.name());
        let error = vm
            .ensure_static_constant_materialized(&spec)
            .expect_err("unsupported nested child fails the complete owner materialization");
        assert!(error.to_string().contains(&spec.id.to_string()));
        let owner = vm.module_registry.get(&owner_id).expect("owner module").object;
        assert!(
            !vm.heap.module(owner).static_constants.contains_key(&name),
            "partial roots never enter the owner cache"
        );
        assert_eq!(vm.temp_root_depth(), 0, "failed materialization restores its temporary-root depth");

        let valid = StaticConstantLoweringSpec {
            id: StaticConstantId::new(owner_id.clone(), "__static_test_after_failed_sequence"),
            descriptor: MaterializationDescriptor::Sequence {
                kind: SequenceMaterializationKind::FrozenArray,
                elements: Arc::from([MaterializationDescriptor::Int(StaticInt::new(7.into()))]),
            },
        };
        let valid_reference = vm.ensure_static_constant_materialized(&valid).expect("valid retry after failure");
        let valid_value = vm.resolve_static_constant(valid_reference).expect("valid owner value");
        let valid_object = valid_value.as_obj().expect("heap Array");
        assert!(matches!(vm.heap.get(valid_object), Object::Array(array) if array.is_frozen() && array.get(0) == Some(Value::int(7))));
        assert_eq!(vm.temp_root_depth(), 0, "successful retry also restores its temporary-root depth");
    }
}
