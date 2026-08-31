//! VM execution and registration support for ADTs and variants (Part 4 & Part 5 primitives).

use crate::adt::{CaseDiscriminant, RuntimeVariantId, RuntimeVariantShape};
use crate::error::RuntimeError;
use crate::heap::{ClassId, ClassObject, Object};
use crate::modules::semantic_lowering::EnumLoweringSpec;
use crate::value::Value;
use crate::vm::VM;
use phalcom_semantic::enum_semantics::VariantShape;

impl VM {
    /// Materializes and registers an enum declaration and its hidden case behavior classes.
    pub fn register_enum_from_spec(&mut self, spec: &EnumLoweringSpec) -> ClassId {
        if let Some(enum_id) = self.adt_registry.enum_by_declaration(&spec.owner) {
            if let Some(desc) = self.adt_registry.enum_descriptor(enum_id) {
                return desc.root_class;
            }
        }

        // 1. Create root class (superclass = Object)
        let mut root_class = ClassObject::bare(&spec.owner.name);
        root_class.class = self.universe.classes.class_class;
        root_class.superclass = Some(self.universe.classes.object_class);
        let root_class_id = self.heap.alloc_class(root_class);
        let enum_id = self.adt_registry.register_enum(spec.owner.clone(), root_class_id);

        // 2. Create hidden case behavior classes for each variant
        for (idx, var_spec) in spec.variants.iter().enumerate() {
            let case_class_name = format!("{}::{}", spec.owner.name, var_spec.id.selector);
            let mut case_class = ClassObject::bare(&case_class_name);
            case_class.class = self.universe.classes.class_class;
            case_class.superclass = Some(root_class_id);
            let case_class_id = self.heap.alloc_class(case_class);
            let discriminant = CaseDiscriminant(idx as u32);
            let shape = match var_spec.shape {
                VariantShape::Singleton => RuntimeVariantShape::Singleton,
                VariantShape::Constructor => RuntimeVariantShape::Constructor,
            };
            let payload_arity = var_spec.payload_fields.len() as u16;

            let runtime_var_id = self
                .adt_registry
                .register_variant(var_spec.id.clone(), enum_id, discriminant, shape, payload_arity, case_class_id, None);

            // Register payload field getters on the case behavior class
            let module = self.entry_module().or_else(|| self.core_module()).unwrap();
            for field in var_spec.payload_fields.iter() {
                let slot = field.slot as usize;
                let getter_name = &field.local_name;
                let sig_str = crate::method::make_signature(getter_name, crate::method::SignatureKind::Getter);
                let selector_sym = self.interner.intern(&sig_str);

                let mut chunk = crate::chunk::Chunk::new();
                chunk.add_instruction(crate::bytecode::Bytecode::GetLocal(0), phalcom_common::range::EmptySourceRange);
                chunk.add_instruction(
                    crate::bytecode::Bytecode::GetVariantPayload(slot as u16),
                    phalcom_common::range::EmptySourceRange,
                );
                chunk.add_instruction(crate::bytecode::Bytecode::Return, phalcom_common::range::EmptySourceRange);

                let callable = std::rc::Rc::new(crate::callable::Callable {
                    chunk,
                    max_slots: 1,
                    num_upvalues: 0,
                    upvalues: Vec::new(),
                    arity: 0,
                    parameter_shape: crate::parameters::ParameterShape::closure(0, false),
                    name_sym: selector_sym,
                    local_names: vec![self.interner.intern("self")],
                });

                let closure_ref = self.heap.alloc(Object::Closure(Box::new(crate::heap::ClosureObject {
                    callable,
                    module,
                    upvalues: Vec::new(),
                    lexical_class: Some(case_class_id),
                    foreign_receiver_guard: None,
                })));

                let method = crate::method::MethodObject::new(
                    selector_sym,
                    crate::method::SignatureKind::Getter,
                    crate::method::MethodKind::Closure(closure_ref),
                    Some(case_class_id),
                );
                let method_ref = self.heap.alloc(Object::Method(Box::new(method)));
                self.heap.class_mut(case_class_id).methods.insert(selector_sym, method_ref);
            }

            if shape == RuntimeVariantShape::Singleton {
                let singleton_val = Value::adt_singleton(runtime_var_id);
                if let Some(desc_mut) = self.adt_registry.variant_descriptor_mut(runtime_var_id) {
                    desc_mut.singleton = Some(singleton_val);
                }
            }
        }

        root_class_id
    }

    /// Part-5 runtime primitive: returns the RuntimeVariantId of an ADT value if any.
    pub fn runtime_variant_of(&self, value: Value) -> Option<RuntimeVariantId> {
        if let Some(rid) = value.as_adt_singleton() {
            return Some(rid);
        }
        if let Some(obj_ref) = value.as_obj() {
            if let Object::AdtCase(case) = self.heap.get(obj_ref) {
                return Some(case.variant);
            }
        }
        None
    }

    /// Part-5 runtime primitive: tests if a value is a specific variant.
    pub fn value_is_variant(&self, value: Value, expected: RuntimeVariantId) -> bool {
        self.runtime_variant_of(value) == Some(expected)
    }

    /// Part-5 runtime primitive: returns the payload length of an ADT case value.
    pub fn case_payload_len(&self, value: Value) -> Option<usize> {
        if value.is_adt_singleton() {
            return Some(0);
        }
        if let Some(obj_ref) = value.as_obj() {
            if let Object::AdtCase(case) = self.heap.get(obj_ref) {
                return Some(case.payload.len());
            }
        }
        None
    }

    /// Part-5 runtime primitive: extracts a payload slot from an ADT case value.
    pub fn case_payload_at(&self, value: Value, index: usize) -> Result<Value, RuntimeError> {
        if value.is_adt_singleton() {
            return Err(RuntimeError::InvalidVariantPayloadSlot { slot: index, len: 0 });
        }
        if let Some(obj_ref) = value.as_obj() {
            if let Object::AdtCase(case) = self.heap.get(obj_ref) {
                return case.payload.get(index).copied().ok_or(RuntimeError::InvalidVariantPayloadSlot {
                    slot: index,
                    len: case.payload.len(),
                });
            }
        }
        Err(RuntimeError::InvalidVariantPayloadSlot { slot: index, len: 0 })
    }

    /// Returns the hidden case behavior class for an ADT value.
    pub fn case_behavior_class(&self, value: Value) -> Option<ClassId> {
        let rid = self.runtime_variant_of(value)?;
        self.adt_registry.variant_descriptor(rid).map(|d| d.behavior_class)
    }
}
