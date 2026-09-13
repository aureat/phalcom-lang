//! VM execution and registration support for data types (LANG005).

use crate::data::RuntimeDataDescriptorId;
use crate::error::RuntimeError;
use crate::heap::{ClassId, ClassObject, Object};
use crate::modules::semantic_lowering::DataDeclarationLoweringSpec;
use crate::product::ProductStorage;
use crate::value::Value;
use crate::vm::VM;

impl VM {
    /// Materializes and registers a data declaration and its behavior class.
    pub fn register_data_from_spec(&mut self, spec: &DataDeclarationLoweringSpec) -> Result<ClassId, RuntimeError> {
        if let Some(desc_id) = self.data_registry.descriptor_by_declaration(&spec.owner) {
            if let Some(desc) = self.data_registry.descriptor(desc_id) {
                return Ok(desc.behavior_class);
            }
        }

        let mut class_obj = ClassObject::bare(&spec.owner.name);
        class_obj.native_repr = true;
        class_obj.class = self.universe.classes.class_class;
        class_obj.superclass = Some(self.universe.classes.object_class);
        let class_id = self.heap.alloc_class(class_obj);

        let layout_spec = spec.layout.build_layout().map_err(|e| RuntimeError::Internal(e.to_string()))?;
        let layout_id = self.heap.product_layouts.register(layout_spec);
        let _desc_id = self.data_registry.register(spec.owner.clone(), class_id, None, layout_id);

        if let Some(module) = self.entry_module().or_else(|| self.universe_module()) {
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
                    foreign_receiver_guard: None,
                })));

                let method = crate::method::MethodObject::new(
                    selector_sym,
                    crate::method::SignatureKind::Getter,
                    crate::method::MethodKind::Closure(closure_ref),
                    Some(class_id),
                );
                let method_ref = self.heap.alloc(Object::Method(Box::new(method)));
                self.heap.class_mut(class_id).methods.insert(selector_sym, method_ref);
            }
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
            .descriptor_by_declaration(&spec.constructor.owner)
            .and_then(|id| self.data_registry.descriptor(id))
            .cloned()
            .ok_or_else(|| RuntimeError::Internal("unregistered data declaration".into()))?;
        let existing_pool = self
            .typing_registry
            .pools()
            .iter()
            .find(|pool| pool.bundle == spec.exact_type)
            .map(|pool| pool.id);
        let pool = match existing_pool {
            Some(id) => id,
            None => {
                let id = MetadataPoolId(self.typing_registry.pool_count() as u32);
                let loaded = crate::typing::loader::load_metadata_bundle(id, spec.exact_type.clone(), &Default::default())
                    .map_err(|error| RuntimeError::Internal(error.to_string()))?;
                self.typing_registry.register_pool(loaded)
            }
        };
        let node = spec
            .exact_type
            .runtime_roots
            .first()
            .ok_or_else(|| RuntimeError::Internal("missing exact data type".into()))?
            .form;
        let exact_type = RuntimeTypeRef::Base { pool, node };
        let layout = spec.layout.build_layout().map_err(|e| RuntimeError::Internal(e.into()))?;
        let layout_id = self.heap.product_layouts.register(layout);
        let descriptor = if matches!(
            spec.exact_type.types[node.0 as usize].form,
            phalcom_type_meta::type_node::TypeNode::Nominal { .. }
        ) {
            self.data_registry.bind_nominal_type(base.runtime_id, exact_type);
            base.runtime_id
        } else {
            self.data_registry
                .register(spec.constructor.owner.clone(), base.behavior_class, Some(exact_type), layout_id)
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
                return data_obj
                    .storage
                    .load_component(layout, logical_index)
                    .map_err(|e| RuntimeError::Internal(e.into()));
            }
        }

        Err(RuntimeError::Type {
            expected: "data object",
            found: receiver.type_name(),
        })
    }
}
