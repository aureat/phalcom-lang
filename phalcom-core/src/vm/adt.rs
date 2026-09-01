//! VM execution and registration support for ADTs and variants (Part 4 & Part 5 primitives).

use crate::adt::{CaseDiscriminant, RuntimeVariantId, RuntimeVariantShape};
use crate::error::RuntimeError;
use crate::heap::{ClassId, ClassObject, Object};
use crate::modules::semantic_lowering::EnumLoweringSpec;
use crate::value::Value;
use crate::vm::VM;
use phalcom_modules::DeclarationId;
use phalcom_semantic::core_surface::CoreDeclarationIds;
use phalcom_semantic::enum_semantics::VariantShape;
use phalcom_semantic::identity::VariantId;
use std::collections::BTreeMap;

struct RuntimeEnumClassBinding {
    root: ClassId,
    variants: BTreeMap<VariantId, ClassId>,
}

impl VM {
    fn canonical_universe_enum_root(&self, owner: &DeclarationId) -> Option<ClassId> {
        let ids = CoreDeclarationIds::default();
        let key = if ids.is_option(owner) {
            phalcom_native_meta::UniverseKey::Option
        } else if ids.is_result(owner) {
            phalcom_native_meta::UniverseKey::Result
        } else if ids.is_ordering(owner) {
            phalcom_native_meta::UniverseKey::Ordering
        } else {
            return None;
        };

        Some(self.universe.classes.resolve(key))
    }

    fn bind_native_option_classes(
        &mut self,
        spec: &EnumLoweringSpec,
    ) -> Result<RuntimeEnumClassBinding, RuntimeError> {
        let expected_some = VariantId::new(
            spec.owner.clone(),
            phalcom_common::selector::Selector::method(
                "Some",
                vec![phalcom_common::selector::SelectorSlot::Positional],
            )
            .map_err(|error| RuntimeError::Internal(error.to_string()))?,
        );

        let expected_none = VariantId::new(
            spec.owner.clone(),
            phalcom_common::selector::Selector::getter("None")
                .map_err(|error| RuntimeError::Internal(error.to_string()))?,
        );

        let some_spec = spec
            .variants
            .iter()
            .find(|v| v.id == expected_some)
            .ok_or_else(|| RuntimeError::Internal("missing Option::Some variant in spec".into()))?;
        if some_spec.shape != VariantShape::Constructor || some_spec.payload_fields.len() != 1 {
            return Err(RuntimeError::Internal("invalid Option::Some variant shape or fields".into()));
        }

        let none_spec = spec
            .variants
            .iter()
            .find(|v| v.id == expected_none)
            .ok_or_else(|| RuntimeError::Internal("missing Option::None variant in spec".into()))?;
        if none_spec.shape != VariantShape::Singleton || !none_spec.payload_fields.is_empty() {
            return Err(RuntimeError::Internal("invalid Option::None variant shape or fields".into()));
        }

        if spec.variants.len() != 2 {
            return Err(RuntimeError::Internal("unexpected variant count for Option".into()));
        }

        let mut variants = BTreeMap::new();
        variants.insert(expected_some, self.universe.classes.some_class);
        variants.insert(expected_none, self.universe.classes.none_class);

        Ok(RuntimeEnumClassBinding {
            root: self.universe.classes.option_class,
            variants,
        })
    }

    fn allocate_general_variant_classes(
        &mut self,
        spec: &EnumLoweringSpec,
        root_class_id: ClassId,
    ) -> Result<BTreeMap<VariantId, ClassId>, RuntimeError> {
        let mut variants = BTreeMap::new();
        for var_spec in spec.variants.iter() {
            let case_class_name = format!("{}::{}", spec.owner.name, var_spec.id.selector);
            let mut case_class = ClassObject::bare(&case_class_name);
            case_class.class = self.universe.classes.class_class;
            case_class.superclass = Some(root_class_id);
            let case_class_id = self.heap.alloc_class(case_class);
            variants.insert(var_spec.id.clone(), case_class_id);
        }
        Ok(variants)
    }

    fn allocate_general_enum_classes(
        &mut self,
        spec: &EnumLoweringSpec,
    ) -> Result<RuntimeEnumClassBinding, RuntimeError> {
        let mut root_class = ClassObject::bare(&spec.owner.name);
        root_class.class = self.universe.classes.class_class;
        root_class.superclass = Some(self.universe.classes.object_class);
        let root_class_id = self.heap.alloc_class(root_class);
        let variants = self.allocate_general_variant_classes(spec, root_class_id)?;

        Ok(RuntimeEnumClassBinding {
            root: root_class_id,
            variants,
        })
    }

    fn bind_canonical_universe_enum_classes(
        &mut self,
        spec: &EnumLoweringSpec,
    ) -> Result<Option<RuntimeEnumClassBinding>, RuntimeError> {
        let Some(root) = self.canonical_universe_enum_root(&spec.owner) else {
            return Ok(None);
        };

        let ids = CoreDeclarationIds::default();
        if ids.is_option(&spec.owner) {
            if spec.representation != crate::adt::RuntimeAdtRepresentation::NativeOption {
                return Err(RuntimeError::Internal(
                    "canonical Universe Option must use NativeOption representation".into(),
                ));
            }
            return self.bind_native_option_classes(spec).map(Some);
        }

        if spec.representation != crate::adt::RuntimeAdtRepresentation::General {
            return Err(RuntimeError::Internal(format!(
                "canonical Universe enum `{}` must use General representation",
                spec.owner.name
            )));
        }

        let variants = self.allocate_general_variant_classes(spec, root)?;
        Ok(Some(RuntimeEnumClassBinding { root, variants }))
    }

    fn class_binding_for_enum(
        &mut self,
        spec: &EnumLoweringSpec,
    ) -> Result<RuntimeEnumClassBinding, RuntimeError> {
        if let Some(binding) = self.bind_canonical_universe_enum_classes(spec)? {
            return Ok(binding);
        }

        match spec.representation {
            crate::adt::RuntimeAdtRepresentation::NativeOption => Err(RuntimeError::Internal(
                "NativeOption representation is reserved for canonical Universe Option".into(),
            )),
            crate::adt::RuntimeAdtRepresentation::General => self.allocate_general_enum_classes(spec),
        }
    }

    /// Materializes and registers an enum declaration and its hidden case behavior classes.
    pub fn register_enum_from_spec(&mut self, spec: &EnumLoweringSpec) -> Result<ClassId, RuntimeError> {
        if let Some(enum_id) = self.adt_registry.enum_by_declaration(&spec.owner) {
            if let Some(desc) = self.adt_registry.enum_descriptor(enum_id) {
                if desc.representation != spec.representation {
                    return Err(RuntimeError::Internal(format!(
                        "enum `{}` is already registered with representation {:?}, requested {:?}",
                        spec.owner.name, desc.representation, spec.representation
                    )));
                }
                if let Some(expected_root) = self.canonical_universe_enum_root(&spec.owner)
                    && desc.root_class != expected_root
                {
                    return Err(RuntimeError::Internal(format!(
                        "canonical Universe enum `{}` is registered with a non-canonical runtime root",
                        spec.owner.name
                    )));
                }
                return Ok(desc.root_class);
            }
        }

        let class_binding = self.class_binding_for_enum(spec)?;
        let root_class_id = class_binding.root;
        let enum_id = self
            .adt_registry
            .register_enum_with_representation(spec.owner.clone(), root_class_id, spec.representation);

        let mut some_runtime_opt = None;
        let mut none_runtime_opt = None;

        for (idx, var_spec) in spec.variants.iter().enumerate() {
            let case_class_id = *class_binding
                .variants
                .get(&var_spec.id)
                .ok_or_else(|| RuntimeError::Internal(
                    format!("missing runtime behavior class for variant `{}`", var_spec.id.selector)
                ))?;
            let discriminant =
                CaseDiscriminant(u32::try_from(idx).map_err(|_| RuntimeError::Message(format!("enum `{}` has too many variants", spec.owner.name)))?);
            let shape = match var_spec.shape {
                VariantShape::Singleton => RuntimeVariantShape::Singleton,
                VariantShape::Constructor => RuntimeVariantShape::Constructor,
            };
            let payload_arity = u16::try_from(var_spec.payload_fields.len())
                .map_err(|_| RuntimeError::Message(format!("variant `{}` has too many payload fields", var_spec.id.selector)))?;

            let runtime_var_id = self
                .adt_registry
                .register_variant(var_spec.id.clone(), enum_id, discriminant, shape, payload_arity, case_class_id, None);

            if spec.representation == crate::adt::RuntimeAdtRepresentation::NativeOption {
                if let phalcom_common::selector::SelectorBase::Named(name) = &var_spec.id.selector.base {
                    if name == "Some" {
                        some_runtime_opt = Some(runtime_var_id);
                    } else if name == "None" {
                        none_runtime_opt = Some(runtime_var_id);
                    }
                }
            }

            // Register payload field getters on the case behavior class
            if let Some(module) = self.entry_module().or_else(|| self.universe_module()) {
                for field in var_spec.payload_fields.iter() {
                    let slot = field.slot;
                    let getter_name = &field.local_name;
                    let sig_str = crate::method::make_signature(getter_name, crate::method::SignatureKind::Getter);
                    let selector_sym = self.interner.intern(&sig_str);

                    let mut chunk = crate::chunk::Chunk::new();
                    chunk.add_instruction(crate::bytecode::Bytecode::GetLocal(0), phalcom_common::range::EmptySourceRange);
                    chunk.add_instruction(crate::bytecode::Bytecode::GetVariantPayload(slot), phalcom_common::range::EmptySourceRange);
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
            }

            if shape == RuntimeVariantShape::Singleton {
                let singleton_val = match spec.representation {
                    crate::adt::RuntimeAdtRepresentation::NativeOption => Value::none(),
                    crate::adt::RuntimeAdtRepresentation::General => Value::adt_singleton(runtime_var_id),
                };
                if let Some(desc_mut) = self.adt_registry.variant_descriptor_mut(runtime_var_id) {
                    desc_mut.singleton = Some(singleton_val);
                }
            }
        }

        if spec.representation == crate::adt::RuntimeAdtRepresentation::NativeOption {
            if let (Some(some_rt), Some(none_rt)) = (some_runtime_opt, none_runtime_opt) {
                let _ = self.adt_registry.bind_native_option_variants(some_rt, none_rt);
            }
        }

        Ok(root_class_id)
    }

    /// Central variant value constructor.
    pub fn construct_variant_value(
        &mut self,
        variant: RuntimeVariantId,
        payload: Vec<Value>,
    ) -> Result<Value, RuntimeError> {
        let variant_desc = self
            .adt_registry
            .variant_descriptor(variant)
            .cloned()
            .ok_or_else(|| RuntimeError::Internal(format!("unknown runtime variant {}", variant.raw())))?;

        let enum_desc = self
            .adt_registry
            .enum_descriptor(variant_desc.enum_id)
            .cloned()
            .ok_or_else(|| RuntimeError::Internal(format!("unknown runtime enum {}", variant_desc.enum_id.raw())))?;

        match enum_desc.representation {
            crate::adt::RuntimeAdtRepresentation::NativeOption => {
                let ids = self
                    .adt_registry
                    .native_option_variants()
                    .ok_or_else(|| RuntimeError::Internal("native Option variants are not bound".into()))?;

                if variant == ids.none {
                    if !payload.is_empty() {
                        return Err(RuntimeError::Message(format!(
                            "variant None takes 0 arguments, got {}",
                            payload.len()
                        )));
                    }
                    return Ok(Value::none());
                }

                if variant == ids.some {
                    let [value] = payload.as_slice() else {
                        return Err(RuntimeError::Message(format!(
                            "variant Some takes 1 argument, got {}",
                            payload.len()
                        )));
                    };
                    return Ok(value.wrap_some()?);
                }

                Err(RuntimeError::Internal(
                    "non-Option variant registered under NativeOption representation".into(),
                ))
            }
            crate::adt::RuntimeAdtRepresentation::General => {
                let case_obj = crate::heap::AdtCaseObject {
                    variant,
                    payload: payload.into_boxed_slice(),
                };
                let obj_ref = self.heap.alloc(Object::AdtCase(Box::new(case_obj)));
                Ok(Value::obj(obj_ref))
            }
        }
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
        if value.is_option() {
            if let Some(variants) = self.adt_registry.native_option_variants() {
                return Some(if value.is_none() {
                    variants.none
                } else {
                    variants.some
                });
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
        if value.is_option() {
            return Some(if value.is_none() { 0 } else { 1 });
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
        if value.is_option() {
            if value.is_none() {
                return Err(RuntimeError::InvalidVariantPayloadSlot { slot: index, len: 0 });
            } else if value.is_some() {
                if index == 0 {
                    return Ok(value.with_some_depth(value.some_depth_raw() - 1));
                } else {
                    return Err(RuntimeError::InvalidVariantPayloadSlot { slot: index, len: 1 });
                }
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
