//! VM execution and registration support for data types (LANG005).

use crate::data::RuntimeDataDescriptorId;
use crate::error::RuntimeError;
use crate::heap::{ClassId, Object};
use crate::modules::semantic_lowering::DataDeclarationLoweringSpec;
use crate::product::ProductStorage;
use crate::value::Value;
use crate::vm::VM;

impl VM {
    /// Materializes and registers a data declaration and its behavior class.
    pub fn register_data_from_spec(&mut self, spec: &DataDeclarationLoweringSpec) -> Result<ClassId, RuntimeError> {
        if let Some(class) = self.data_registry.behavior_class_for_declaration(&spec.owner) {
            return Ok(class);
        }

        let module = self
            .module_registry
            .get(&spec.owner.module)
            .map(|record| record.object)
            .or_else(|| self.entry_module())
            .or_else(|| self.universe_module())
            .ok_or_else(|| RuntimeError::Internal(format!("data declaration module {} is not materialized", spec.owner.module)))?;
        let class_id = self.create_class(module, &spec.owner.name, Some(self.universe.classes.object_class));
        self.heap.class_mut(class_id).native_repr = true;
        let class_id = self.data_registry.register_behavior_class(spec.owner.clone(), class_id);

        if let Some(layout_spec) = &spec.layout {
            let layout = layout_spec.build_layout().map_err(|e| RuntimeError::Internal(e.to_string()))?;
            let layout_id = self.heap.product_layouts.register(layout);
            let component_slots = (0..spec.components.len()).map(|index| index as u32).collect::<Vec<_>>().into_boxed_slice();
            self.data_registry
                .register_with_component_slots(spec.owner.clone(), class_id, None, layout_id, Some(component_slots));
        }

        for comp in spec.components.iter() {
            let logical_index = comp.logical_index;
            let getter_name = &comp.local_name;
            let sig_str = crate::method::make_signature(getter_name, crate::method::SignatureKind::Getter);
            let selector_sym = self.interner.intern(&sig_str);

            let mut chunk = crate::chunk::Chunk::new();
            chunk.add_instruction(crate::bytecode::Bytecode::GetLocal(0), phalcom_common::range::EmptySourceRange);
            chunk.add_instruction(
                crate::bytecode::Bytecode::GetDataComponent(logical_index as u16),
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
                lexical_class: Some(class_id),
                super_anchor: Some(class_id),
                foreign_receiver_guard: None,
            })));

            let method = crate::method::MethodObject::new(
                crate::method::Signature::new(selector_sym, crate::method::SignatureKind::Getter, 0),
                crate::method::MethodKind::Closure(closure_ref),
                Some(class_id),
            );
            let method_ref = self.heap.alloc(Object::Method(Box::new(method)));
            self.heap.class_mut(class_id).methods.insert(selector_sym, method_ref);
        }

        Ok(class_id)
    }

    /// Resolves and interns a RuntimeDataDescriptorId from a lowering spec.
    pub fn get_or_bind_data_descriptor(
        &mut self,
        spec: &crate::modules::semantic_lowering::DataConstructionLoweringSpec,
    ) -> Result<RuntimeDataDescriptorId, RuntimeError> {
        use crate::typing::handle::{MetadataPoolId, RuntimeTypeRef};
        let base = self
            .data_registry
            .behavior_class_for_declaration(&spec.constructor.owner)
            .ok_or_else(|| RuntimeError::Internal("unregistered data declaration".into()))?;
        let type_recipe = if let Some(exact_type_metadata) = spec.exact_type.as_ref() {
            // Source-compiled lowering carries a private exact metadata bundle.
            // Install/reuse it to recover the bundle-local runtime root before
            // entering the shared runtime recipe path.
            let existing_pool = self
                .typing_registry
                .pools()
                .iter()
                .find(|pool| std::sync::Arc::ptr_eq(&pool.bundle, exact_type_metadata) || pool.bundle.as_ref() == exact_type_metadata.as_ref())
                .map(|pool| pool.id);
            let pool = match existing_pool {
                Some(id) => id,
                None => {
                    let id = MetadataPoolId(self.typing_registry.pool_count() as u32);
                    let loaded = crate::typing::loader::load_metadata_bundle(id, exact_type_metadata.clone(), &Default::default())
                        .map_err(|error| RuntimeError::Internal(error.to_string()))?;
                    self.typing_registry.register_pool(loaded)
                }
            };
            let node = exact_type_metadata
                .runtime_roots
                .first()
                .ok_or_else(|| RuntimeError::Internal("missing exact data type".into()))?
                .form;
            let base_type = RuntimeTypeRef::Base { pool, node };
            match spec.type_recipe {
                crate::typing::RuntimeTypeRecipe::Closed(_) => crate::typing::RuntimeTypeRecipe::Closed(base_type),
                crate::typing::RuntimeTypeRecipe::Template(_) => crate::typing::RuntimeTypeRecipe::Template(base_type),
            }
        } else {
            // Source-free executable materialization has already relocated the
            // validated portable type recipe against installed P7 metadata.
            spec.type_recipe
        };
        let env_id = self
            .frames
            .last()
            .map(|frame| frame.type_environment)
            .unwrap_or(crate::typing::environment::RuntimeTypeEnvironmentId::EMPTY);
        let env = self
            .runtime_type_environments
            .get(env_id)
            .cloned()
            .ok_or_else(|| RuntimeError::Internal("missing active runtime type environment".into()))?;
        let exact_type = crate::typing::instantiate_type_recipe(type_recipe, &env, &mut self.runtime_typing_context, &self.typing_registry)
            .ok_or_else(|| RuntimeError::Internal("generic data type could not be instantiated".into()))?;
        let layout = spec.layout.build_layout().map_err(|e| RuntimeError::Internal(e.into()))?;
        let layout_id = self.heap.product_layouts.register(layout);
        let base_descriptor = self
            .data_registry
            .descriptor_by_declaration(&spec.constructor.owner)
            .and_then(|id| self.data_registry.descriptor(id))
            .cloned();
        let component_slots = Some(spec.component_to_logical.clone());
        let is_nominal_base_type = match exact_type {
            RuntimeTypeRef::Base { pool, node } => self
                .typing_registry
                .get_pool(pool)
                .and_then(|loaded| loaded.bundle.types.get(node.0 as usize))
                .is_some_and(|ty| matches!(ty.form, phalcom_type_meta::type_node::TypeNode::Nominal { .. })),
            RuntimeTypeRef::Overlay(_) => false,
        };
        let descriptor = if is_nominal_base_type && let Some(base_descriptor) = base_descriptor {
            self.data_registry.bind_nominal_type(base_descriptor.runtime_id, exact_type);
            base_descriptor.runtime_id
        } else {
            self.data_registry
                .intern_exact_with_component_slots(
                    crate::data::RuntimeDataSpecialization {
                        semantic_owner: spec.constructor.owner.clone(),
                        behavior_class: base,
                        exact_type,
                        layout: layout_id,
                        component_slots,
                    },
                    &self.runtime_typing_context,
                    &self.typing_registry,
                )
                .map_err(|message| RuntimeError::Internal(message.into()))?
        };
        Ok(descriptor)
    }

    /// Constructs a runtime Value for a data instance using raw arguments and an argument-to-component mapping.
    pub fn construct_data_value_with_mapping(
        &mut self,
        descriptor_id: RuntimeDataDescriptorId,
        argument_to_component: &[u32],
        arguments: Vec<Value>,
    ) -> Result<Value, RuntimeError> {
        if arguments.len() != argument_to_component.len() {
            return Err(RuntimeError::Internal("data argument mapping arity mismatch".into()));
        }
        let mut components = vec![Value::nil(); arguments.len()];
        let mut seen = vec![false; arguments.len()];
        for (argument, &index) in arguments.into_iter().zip(argument_to_component.iter()) {
            let slot = index as usize;
            if slot >= components.len() || seen[slot] {
                return Err(RuntimeError::Internal("invalid data argument mapping".into()));
            }
            seen[slot] = true;
            components[slot] = argument;
        }
        self.construct_data_value(descriptor_id, components)
    }

    /// Binds a canonical, exported exact data type to a VM-local descriptor and constructs the value.
    pub fn construct_data_from_spec(
        &mut self,
        spec: &crate::modules::semantic_lowering::DataConstructionLoweringSpec,
        arguments: Vec<Value>,
    ) -> Result<Value, RuntimeError> {
        let descriptor = self.get_or_bind_data_descriptor(spec)?;
        self.construct_data_value_with_mapping(descriptor, &spec.argument_to_component, arguments)
    }

    /// Constructs a runtime Value for a data instance.
    pub fn construct_data_value(&mut self, descriptor_id: RuntimeDataDescriptorId, components: Vec<Value>) -> Result<Value, RuntimeError> {
        let desc = self
            .data_registry
            .descriptor(descriptor_id)
            .ok_or_else(|| RuntimeError::Internal("unknown data descriptor".into()))?;
        let layout = self
            .heap
            .product_layouts
            .get(desc.layout)
            .ok_or_else(|| RuntimeError::Internal("unknown product layout".into()))?;

        if components.len() != layout.components.len() {
            return Err(RuntimeError::Internal("data component count does not match layout".into()));
        }
        if layout.is_nullary() {
            return Ok(Value::data_singleton(descriptor_id));
        }
        let storage = ProductStorage::from_values(desc.layout, layout, &components).map_err(|e| RuntimeError::Internal(e.into()))?;

        let data_obj = crate::heap::DataObject::new(descriptor_id, storage);
        let data_ref = self.heap.alloc(Object::Data(Box::new(data_obj)));
        Ok(Value::obj(data_ref))
    }

    /// Loads a component from a DataObject or DataSingleton.
    pub fn get_data_component(&self, receiver: Value, logical_index: u32) -> Result<Value, RuntimeError> {
        if let Some(desc_id) = receiver.as_data_singleton() {
            let _ = desc_id;
            return Err(RuntimeError::Internal("data singleton has no components".into()));
        }

        if let Some(obj_ref) = receiver.as_obj() {
            if let Object::Data(data_obj) = self.heap.get(obj_ref) {
                let desc = self
                    .data_registry
                    .descriptor(data_obj.descriptor)
                    .ok_or_else(|| RuntimeError::Internal("unknown data descriptor".into()))?;
                let layout = self
                    .heap
                    .product_layouts
                    .get(desc.layout)
                    .ok_or_else(|| RuntimeError::Internal("unknown product layout".into()))?;
                let component_index = desc
                    .component_slots
                    .as_ref()
                    .and_then(|slots| slots.get(logical_index as usize).copied())
                    .or_else(|| desc.component_slots.is_none().then_some(logical_index))
                    .ok_or_else(|| RuntimeError::Internal("unknown fixed data component".into()))?;
                return data_obj
                    .storage
                    .load_component(layout, component_index)
                    .map_err(|e| RuntimeError::Internal(e.into()));
            }
        }

        Err(RuntimeError::Type {
            expected: "data object",
            found: receiver.type_name(),
        })
    }

    /// Loads an exact specialization-local product slot without interpreting
    /// its index as a declaration component ordinal.
    pub fn get_data_product_slot(&self, receiver: Value, logical_index: u32) -> Result<Value, RuntimeError> {
        if receiver.as_data_singleton().is_some() {
            return Err(RuntimeError::Internal("data singleton has no components".into()));
        }

        if let Some(obj_ref) = receiver.as_obj()
            && let Object::Data(data_obj) = self.heap.get(obj_ref)
        {
            let desc = self
                .data_registry
                .descriptor(data_obj.descriptor)
                .ok_or_else(|| RuntimeError::Internal("unknown data descriptor".into()))?;
            let layout = self
                .heap
                .product_layouts
                .get(desc.layout)
                .ok_or_else(|| RuntimeError::Internal("unknown product layout".into()))?;
            if layout.component(logical_index).is_none() {
                return Err(RuntimeError::Internal("unknown exact data product slot".into()));
            }
            return data_obj
                .storage
                .load_component(layout, logical_index)
                .map_err(|e| RuntimeError::Internal(e.into()));
        }

        Err(RuntimeError::Type {
            expected: "data object",
            found: receiver.type_name(),
        })
    }
}
