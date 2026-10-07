//! Runtime materialization of canonical checked static-constant descriptors.

use crate::error::{PhResult, RuntimeError};
use crate::heap::Object;
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

    /// Materializes one checked constant from its canonical semantic descriptor.
    pub(crate) fn materialize_static_constant(&mut self, spec: &StaticConstantLoweringSpec) -> PhResult<Value> {
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
}
