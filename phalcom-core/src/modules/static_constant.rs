//! Runtime materialization of canonical checked static-constant descriptors.

use crate::error::{PhResult, RuntimeError};
use crate::heap::Object;
use crate::modules::linkage::RuntimeStaticConstantRef;
use crate::modules::semantic_lowering::StaticConstantLoweringSpec;
use crate::value::Value;
use crate::vm::VM;
use phalcom_semantic::materialization::MaterializationDescriptor;
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
        let value = match &spec.descriptor {
            MaterializationDescriptor::Unit => Value::unit(),
            MaterializationDescriptor::Bool(value) => Value::bool(*value),
            MaterializationDescriptor::Int(StaticInt::Small(value)) => Value::int(*value),
            MaterializationDescriptor::Int(StaticInt::Large(value)) => Value::obj(self.heap.alloc(Object::LargeInt(value.as_ref().clone()))),
            MaterializationDescriptor::Float(value) => Value::float(value.to_f64()),
            MaterializationDescriptor::String(value) => self.alloc_string_value(value.to_string()),
            MaterializationDescriptor::Symbol(value) => Value::symbol(self.interner.intern(value.as_ref())),
            MaterializationDescriptor::DeferredStructural(_) | MaterializationDescriptor::Unsupported(_) => {
                return Err(RuntimeError::Internal(format!("static constant {} has no supported runtime materialization", spec.id)).into());
            }
        };
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

        let value = self.materialize_static_constant_value(spec)?;
        self.heap.module_mut(owner).static_constants.insert(name, value);
        Ok(RuntimeStaticConstantRef { module: owner, name })
    }

    /// Resolves a canonical static-constant reference to its owner-held value.
    pub(crate) fn resolve_static_constant(&self, reference: RuntimeStaticConstantRef) -> PhResult<Value> {
        let name = self.resolve_symbol(reference.name);
        let Some(Object::Module(module)) = self.heap.try_get(reference.module) else {
            return Err(RuntimeError::Internal(format!(
                "static constant {name} has no live owner module at {:?}",
                reference.module
            ))
            .into());
        };
        module.static_constants.get(&reference.name).copied().ok_or_else(|| {
            RuntimeError::Internal(format!("static constant {}::{name} is not materialized", module.id)).into()
        })
    }

    /// Reconstructs the semantic symbol identity represented by a runtime
    /// constant reference, preserving the canonical owner across re-exports.
    pub(crate) fn runtime_static_constant_symbol(&self, reference: RuntimeStaticConstantRef) -> PhResult<phalcom_modules::SymbolId> {
        let name = self.resolve_symbol(reference.name);
        let Some(Object::Module(module)) = self.heap.try_get(reference.module) else {
            return Err(RuntimeError::Internal(format!(
                "static constant {name} has no live owner module at {:?}",
                reference.module
            ))
            .into());
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
    use phalcom_semantic::materialization::DeferredStructuralKind;
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
        let unsupported_error = vm.ensure_static_constant_materialized(&unsupported_spec).expect_err("deferred value fails closed");
        assert!(unsupported_error.to_string().contains(&unsupported_spec.id.to_string()));

        let missing_owner = ModuleId::universe(ModulePath::from_components(vec![ModuleComponent::from_identifier("missing_static_owner").unwrap()]));
        let missing_spec = StaticConstantLoweringSpec {
            id: StaticConstantId::new(missing_owner, "missing"),
            descriptor: MaterializationDescriptor::Bool(true),
        };
        let missing_error = vm.ensure_static_constant_materialized(&missing_spec).expect_err("missing owner returns an error");
        assert!(missing_error.to_string().contains(&missing_spec.id.to_string()));
    }
}
