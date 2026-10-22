//! Relocation of a validated portable executable into one fresh VM heap.

use crate::callable::{Callable, UpvalueDescriptor};
use crate::chunk::Chunk;
use crate::error::{PhError, PhResult, RuntimeError, VmBootstrapError};
use crate::heap::{ClosureObject, Object};
use crate::method::{MemberVisibility, MethodKind, MethodObject, RestLayout, RestMode, RuntimeAccessOrigin, Signature, SignatureKind};
use crate::parameters::{ParameterShape, RestKind};
use crate::value::{Value, normalize_bigint};
use crate::vm::{ClassKey, ClassLayout, NativeInstallMode, VM};
use phalcom_common::range::SourceRange;
use phalcom_compiler_artifact::executable::constant::{
    ConstantRecipeV1, MethodBodyV1, MethodRecipeV1, MethodSignatureKindV1, MethodSignatureV1, RestModeV1, StableClassRefV1, VisibilityV1,
};
use phalcom_compiler_artifact::executable::module::PortableClassAllocationV1;
use phalcom_compiler_artifact::executable::prototype::{ConstantId, PrototypeId, RestKindV1, UpvalueCaptureV1};
use phalcom_type_meta::identity::StableDispatchSide;
use std::rc::Rc;
use std::sync::Arc;

use super::validate::ValidatedExecutableImage;

fn internal(message: impl Into<String>) -> PhError {
    RuntimeError::Internal(message.into()).into()
}

pub(crate) fn bootstrap_vm(
    artifact: &crate::modules::canonical_artifact::LoadedUniverseArtifact,
) -> Result<VM, VmBootstrapError> {
    let executable = artifact
        .executable()
        .ok_or_else(|| VmBootstrapError::Invariant("Universe artifact has no validated executable image".into()))?;
    let mut vm = VM::new_kernel();
    vm.materialize_universe_artifact_topology(&artifact.formal_image)
        .map_err(VmBootstrapError::Runtime)?;
    VM::install_native_runtime_on_artifact_topology(&mut vm, NativeInstallMode::DescriptorOnly)?;
    vm.install_semantic_metadata_bundle(Arc::clone(&artifact.metadata))
        .map_err(VmBootstrapError::Runtime)?;
    vm.materialize_universe_artifact_linkage(&artifact.formal_image)
        .map_err(VmBootstrapError::Runtime)?;
    ExecutableMaterializer::new(&mut vm, executable).install().map_err(VmBootstrapError::Runtime)?;
    vm.bind_native_error_constructors();
    vm.bind_canonical_variant_aliases()?;
    vm.sync_universe_class_aliases()?;
    vm.universe.mark_leaf_tostring_pristine();
    vm.universe.verify_invariants(&vm.heap).map_err(VmBootstrapError::Invariant)?;
    Ok(vm)
}

struct ExecutableMaterializer<'a> {
    vm: &'a mut VM,
    image: &'a phalcom_compiler_artifact::executable::image::ExecutableImageV1,
    constants: Vec<Option<Value>>,
    callables: Vec<Option<Rc<Callable>>>,
}

impl<'a> ExecutableMaterializer<'a> {
    fn new(vm: &'a mut VM, executable: &'a ValidatedExecutableImage) -> Self {
        let image = executable.image();
        Self {
            vm,
            image,
            constants: vec![None; image.constants.len()],
            callables: vec![None; image.prototypes.len()],
        }
    }

    fn install(mut self) -> PhResult<()> {
        self.install_class_authority()?;
        let eager = self.image.bootstrap.eager_modules.clone();
        for entry in eager {
            let module_id = phalcom_semantic::metadata::stable_identity::from_stable_module(&entry.module).map_err(internal)?;
            let module = self
                .vm
                .module_registry
                .get(&module_id)
                .ok_or_else(|| internal(format!("artifact module {module_id} is not materialized")))?
                .object;
            let closure = self.materialize_closure(entry.initializer, module, None, None)?;
            self.vm.heap.module_mut(module).closure = Some(closure);
            self.vm.run_in_module(module, closure)?;
            self.vm
                .module_registry
                .get_mut(&module_id)
                .expect("artifact module remains registered")
                .state = crate::modules::registry::ModuleState::Initialized;
            self.vm.bind_canonical_variant_aliases().map_err(|error| internal(error.to_string()))?;
        }
        Ok(())
    }

    fn install_class_authority(&mut self) -> PhResult<()> {
        use indexmap::IndexMap;
        use std::collections::HashSet;

        for recipe in &self.image.classes {
            let declaration = recipe.class.declaration.clone();
            let declaration_id = phalcom_semantic::metadata::stable_identity::from_stable_declaration(&declaration).map_err(internal)?;
            let name = declaration_id.name.to_string();
            self.vm
                .typing_registry
                .register_declaration_identity(declaration_id.module.clone(), name.clone().into_boxed_str(), declaration.clone());

            if let PortableClassAllocationV1::Primordial { role } = &recipe.allocation {
                let key_name = role
                    .strip_prefix("universe:")
                    .ok_or_else(|| internal(format!("invalid primordial role {role}")))?;
                let key = phalcom_native_meta::UniverseKey::from_name(key_name)
                    .ok_or_else(|| internal(format!("unknown primordial role {role}")))?;
                let class = self.vm.universe.classes.resolve(key);
                let target = match recipe.class.side {
                    StableDispatchSide::Instance => class,
                    StableDispatchSide::Class => self.vm.heap.class(class).class,
                };
                self.vm.typing_registry.register_nominal_binding(declaration, target);
                continue;
            }

            if recipe.class.side != StableDispatchSide::Instance {
                continue;
            }
            let module = self
                .vm
                .module_registry
                .get(&declaration_id.module)
                .ok_or_else(|| internal(format!("class owner module {} is absent", declaration_id.module)))?
                .object;
            let name_symbol = self.vm.interner.intern(&name);
            let class_side = self
                .image
                .classes
                .iter()
                .find(|candidate| candidate.class.declaration == recipe.class.declaration && candidate.class.side == StableDispatchSide::Class);
            let mut field_slots = IndexMap::new();
            for field in &recipe.fields {
                field_slots.insert(
                    self.vm.interner.intern(&field.name),
                    u16::try_from(field.slot).map_err(|_| internal("artifact instance field slot exceeds u16"))?,
                );
            }
            let mut static_field_slots = IndexMap::new();
            if let Some(class_side) = class_side {
                for field in &class_side.fields {
                    static_field_slots.insert(
                        self.vm.interner.intern(&field.name),
                        u16::try_from(field.slot).map_err(|_| internal("artifact static field slot exceeds u16"))?,
                    );
                }
            }
            let immutable_instance_slots = recipe
                .immutable_slots
                .iter()
                .map(|slot| u16::try_from(*slot).map_err(|_| internal("artifact immutable instance slot exceeds u16")))
                .collect::<Result<HashSet<_>, _>>()?;
            let immutable_static_slots = class_side
                .into_iter()
                .flat_map(|row| row.immutable_slots.iter())
                .map(|slot| u16::try_from(*slot).map_err(|_| internal("artifact immutable static slot exceeds u16")))
                .collect::<Result<HashSet<_>, _>>()?;
            self.vm.field_layouts.insert(
                ClassKey { module, name: name_symbol },
                ClassLayout {
                    name: name_symbol,
                    field_slots,
                    field_count: u16::try_from(recipe.field_count).map_err(|_| internal("artifact instance field count exceeds u16"))?,
                    static_field_slots,
                    static_field_count: u16::try_from(class_side.map_or(0, |row| row.field_count))
                        .map_err(|_| internal("artifact static field count exceeds u16"))?,
                    immutable_instance_fields: HashSet::new(),
                    immutable_static_fields: HashSet::new(),
                    immutable_instance_slots,
                    immutable_static_slots,
                    abstract_members: HashSet::new(),
                    is_abstract: recipe.is_abstract,
                    declared_at: SourceRange::default(),
                },
            );
        }
        Ok(())
    }

    fn materialize_closure(
        &mut self,
        prototype: PrototypeId,
        module: crate::heap::ObjRef,
        lexical_class: Option<crate::heap::ClassId>,
        super_anchor: Option<crate::heap::ClassId>,
    ) -> PhResult<crate::heap::ObjRef> {
        let callable = self.materialize_callable(prototype, module)?;
        Ok(self.vm.heap.alloc(Object::Closure(Box::new(ClosureObject {
            callable,
            module,
            upvalues: Vec::new(),
            lexical_class,
            super_anchor,
            foreign_receiver_guard: None,
        }))))
    }

    fn materialize_callable(&mut self, id: PrototypeId, module: crate::heap::ObjRef) -> PhResult<Rc<Callable>> {
        let index = id.0 as usize;
        if let Some(callable) = self.callables.get(index).and_then(Clone::clone) {
            return Ok(callable);
        }
        let row = self
            .image
            .prototypes
            .get(index)
            .cloned()
            .ok_or_else(|| internal(format!("artifact prototype {} is absent", id.0)))?;
        let mut chunk = Chunk::new();
        for constant in row.constants.iter().copied() {
            chunk.constants.push(self.materialize_constant(constant)?);
        }
        let semantic_pool = self
            .image
            .semantic_pools
            .get(row.semantics.0 as usize)
            .ok_or_else(|| internal(format!("artifact semantic pool {} is absent", row.semantics.0)))?;
        chunk.executable_semantics =
            super::semantic_materialize::materialize_semantic_pool(self.vm, semantic_pool, &row.constants)?;
        for (index, instruction) in row.code.into_iter().enumerate() {
            let span = row.spans.get(index).map_or_else(SourceRange::default, |span| {
                SourceRange::new(usize::try_from(span.start).unwrap_or(usize::MAX), usize::try_from(span.end).unwrap_or(usize::MAX))
            });
            chunk.add_instruction(instruction.into(), span);
        }
        if self.vm.heap.module(module).sources.is_empty() {
            self.vm.heap.module_mut(module).push_source(Arc::new(String::new()));
        }
        chunk.source_id = 0;
        let callable = Rc::new(Callable {
            chunk,
            max_slots: row.max_slots as usize,
            num_upvalues: row.upvalues.len(),
            upvalues: row
                .upvalues
                .into_iter()
                .map(|capture| match capture {
                    UpvalueCaptureV1::Local(index) => UpvalueDescriptor { is_local: true, index: index as usize },
                    UpvalueCaptureV1::Upvalue(index) => UpvalueDescriptor { is_local: false, index: index as usize },
                })
                .collect(),
            arity: row.arity as usize,
            parameter_shape: ParameterShape {
                fixed_positionals: row.parameters.fixed_positionals as usize,
                fixed_labels: row.parameters.fixed_labels.iter().map(|label| self.vm.interner.intern(label)).collect(),
                rest: row.parameters.rest.map(|rest| match rest {
                    RestKindV1::Positional => RestKind::Positional,
                    RestKindV1::Labeled => RestKind::Labeled,
                    RestKindV1::Split => RestKind::Split,
                    RestKindV1::Complete => RestKind::Complete,
                }),
            },
            name_sym: self.vm.interner.intern(&row.name),
            local_names: row.local_names.iter().map(|name| self.vm.interner.intern(name)).collect(),
        });
        self.callables[index] = Some(Rc::clone(&callable));
        Ok(callable)
    }

    fn materialize_constant(&mut self, id: ConstantId) -> PhResult<Value> {
        let index = id.0 as usize;
        if let Some(value) = self.constants.get(index).and_then(|value| *value) {
            return Ok(value);
        }
        let recipe = self
            .image
            .constants
            .get(index)
            .cloned()
            .ok_or_else(|| internal(format!("artifact constant {} is absent", id.0)))?;
        let value = match recipe {
            ConstantRecipeV1::None => Value::none(),
            ConstantRecipeV1::Unit => Value::unit(),
            ConstantRecipeV1::EmptyRecord => Value::empty_record(),
            ConstantRecipeV1::EmptyPack => Value::empty_pack(),
            ConstantRecipeV1::Boolean { value } => Value::bool(value),
            ConstantRecipeV1::Integer { bits } => Value::int(i64::from_le_bytes(bits.to_le_bytes())),
            ConstantRecipeV1::Float { bits } => Value::float(f64::from_bits(bits)),
            ConstantRecipeV1::Character { scalar } => Value::char(char::from_u32(scalar).ok_or_else(|| internal("invalid artifact character"))?),
            ConstantRecipeV1::Symbol { text } => Value::symbol(self.vm.interner.intern(&text)),
            ConstantRecipeV1::String { text } => self.vm.alloc_string_value(text),
            ConstantRecipeV1::BigInteger { negative, magnitude } => {
                let sign = if negative { num_bigint::Sign::Minus } else { num_bigint::Sign::Plus };
                normalize_bigint(num_bigint::BigInt::from_bytes_le(sign, &magnitude), &mut self.vm.heap)
            }
            ConstantRecipeV1::Class { reference } => Value::obj(self.resolve_class(&reference)?),
            ConstantRecipeV1::Module { module } => {
                let id = phalcom_semantic::metadata::stable_identity::from_stable_module(&module).map_err(internal)?;
                Value::obj(self.vm.module_registry.get(&id).ok_or_else(|| internal(format!("artifact module {id} is absent")))?.object)
            }
            ConstantRecipeV1::ClosureTemplate {
                prototype,
                module,
                lexical_class,
                super_anchor,
            } => {
                let module_id = phalcom_semantic::metadata::stable_identity::from_stable_module(&module).map_err(internal)?;
                let module = self.vm.module_registry.get(&module_id).ok_or_else(|| internal(format!("artifact module {module_id} is absent")))?.object;
                let lexical_class = lexical_class.as_ref().map(|class| self.resolve_class(class)).transpose()?;
                let super_anchor = super_anchor.as_ref().map(|class| self.resolve_class(class)).transpose()?;
                Value::obj(self.materialize_closure(prototype, module, lexical_class, super_anchor)?)
            }
            ConstantRecipeV1::Method { recipe } => Value::obj(self.materialize_method(recipe)?),
            ConstantRecipeV1::SelectorPattern { pattern } => Value::obj(self.vm.alloc_selector_pattern(portable_selector_pattern(pattern)?)),
            ConstantRecipeV1::Vector { elements } => {
                let values = elements.into_iter().map(|id| self.materialize_constant(id)).collect::<PhResult<Vec<_>>>()?;
                Value::obj(self.vm.heap.alloc_vector(values))
            }
            ConstantRecipeV1::FrozenArray { elements } => {
                let values = elements.into_iter().map(|id| self.materialize_constant(id)).collect::<PhResult<Vec<_>>>()?;
                Value::obj(self.vm.heap.alloc(Object::Array(crate::heap::ArrayObject::from_vec_frozen(values))))
            }
            ConstantRecipeV1::Tuple {
                positionals,
                labels,
                labeled_values,
            } => {
                let positionals = positionals
                    .into_iter()
                    .map(|id| self.materialize_constant(id))
                    .collect::<PhResult<Vec<_>>>()?;
                let labeled_values = labeled_values
                    .into_iter()
                    .map(|id| self.materialize_constant(id))
                    .collect::<PhResult<Vec<_>>>()?;
                let labeled = labels
                    .into_iter()
                    .zip(labeled_values)
                    .map(|(label, value)| (self.vm.interner.intern(&label), value))
                    .collect();
                crate::product::finish_tuple_value(self.vm, positionals, labeled)?
            }
            ConstantRecipeV1::Record { labels, values } => {
                let values = values
                    .into_iter()
                    .map(|id| self.materialize_constant(id))
                    .collect::<PhResult<Vec<_>>>()?;
                crate::product::finish_record(
                    self.vm,
                    labels
                        .into_iter()
                        .zip(values)
                        .map(|(label, value)| (phalcom_semantic::FieldKey::string(label), value))
                        .collect(),
                )
                .map_err(|error| internal(format!("artifact record constant is invalid: {error:?}")))?
            }
            ConstantRecipeV1::Pack {
                positionals,
                labels,
                labeled_values,
            } => {
                let positionals = positionals
                    .into_iter()
                    .map(|id| self.materialize_constant(id))
                    .collect::<PhResult<Vec<_>>>()?;
                let labeled_values = labeled_values
                    .into_iter()
                    .map(|id| self.materialize_constant(id))
                    .collect::<PhResult<Vec<_>>>()?;
                let labeled = labels
                    .into_iter()
                    .zip(labeled_values)
                    .map(|(label, value)| (self.vm.interner.intern(&label), value))
                    .collect();
                crate::product::finish_pack_value(self.vm, positionals, labeled)?
            }
            ConstantRecipeV1::Nil => Value::nil(),
        };
        self.constants[index] = Some(value);
        Ok(value)
    }

    fn resolve_class(&self, reference: &StableClassRefV1) -> PhResult<crate::heap::ClassId> {
        let class = self
            .vm
            .typing_registry
            .resolve_nominal(&reference.declaration)
            .ok_or_else(|| internal(format!("artifact class {:?} is not materialized", reference.declaration)))?;
        Ok(match reference.side {
            StableDispatchSide::Instance => class,
            StableDispatchSide::Class => self.vm.heap.class(class).class,
        })
    }

    fn materialize_method(&mut self, recipe: MethodRecipeV1) -> PhResult<crate::heap::ObjRef> {
        if let MethodBodyV1::Native { anchor } = recipe.body {
            let anchor = self.image.native_anchors.get(anchor.0 as usize).ok_or_else(|| internal("artifact native anchor is absent"))?;
            let owner = self.resolve_class(&anchor.owner)?;
            let selector = self.vm.interner.intern(&anchor.signature.selector);
            return self
                .vm
                .heap
                .class(owner)
                .get_method(selector)
                .ok_or_else(|| internal(format!("installed native method {} is absent", anchor.binding_key)));
        }
        let MethodBodyV1::Closure { template } = recipe.body else { unreachable!() };
        let closure = self.materialize_constant(template)?.as_obj().ok_or_else(|| internal("artifact method template is not an object"))?;
        let holder = recipe.holder.as_ref().map(|class| self.resolve_class(class)).transpose()?;
        let access_owner = recipe.access_owner.as_ref().map(|class| self.resolve_class(class)).transpose()?;
        let mut method = MethodObject::new(signature(self.vm, recipe.signature), MethodKind::Closure(closure), holder);
        method.visibility = match recipe.visibility {
            VisibilityV1::Public => MemberVisibility::Public,
            VisibilityV1::Private => MemberVisibility::Private,
            VisibilityV1::Protected => MemberVisibility::Protected,
            VisibilityV1::Module => MemberVisibility::Module,
            VisibilityV1::Package => MemberVisibility::Package,
            VisibilityV1::Internal => MemberVisibility::Internal,
        };
        method.access_owner = access_owner;
        method.origin = RuntimeAccessOrigin {
            module: recipe.origin.module.as_ref().map(|module| self.resolve_module(module)).transpose()?,
            package: recipe.origin.package.as_ref().map(|module| self.resolve_module(module)).transpose()?,
            project: recipe
                .origin
                .project
                .as_ref()
                .map(phalcom_semantic::metadata::stable_identity::from_stable_project)
                .transpose()
                .map_err(internal)?,
        };
        method.contracts = recipe
            .contracts
            .map(|contracts| {
                contracts
                    .into_iter()
                    .map(|contract| Ok((self.vm.interner.intern(&contract.predicate), self.materialize_constant(contract.value)?)))
                    .collect::<PhResult<Vec<_>>>()
            })
            .transpose()?;
        method.attributes = recipe
            .attributes
            .into_iter()
            .map(|attribute| self.materialize_constant(attribute))
            .collect::<PhResult<Vec<_>>>()?;
        method.attributes_frozen = recipe.attributes_frozen;
        Ok(self.vm.heap.alloc(Object::Method(Box::new(method))))
    }

    fn resolve_module(&self, module: &phalcom_type_meta::identity::StableModuleRef) -> PhResult<crate::heap::ObjRef> {
        let id = phalcom_semantic::metadata::stable_identity::from_stable_module(module).map_err(internal)?;
        self.vm
            .module_registry
            .get(&id)
            .map(|record| record.object)
            .ok_or_else(|| internal(format!("artifact module {id} is absent")))
    }
}

fn signature(vm: &mut VM, signature: MethodSignatureV1) -> Signature {
    let selector = vm.interner.intern(&signature.selector);
    let kind = match signature.kind {
        MethodSignatureKindV1::Method { slots } => SignatureKind::Method(slots),
        MethodSignatureKindV1::Getter => SignatureKind::Getter,
        MethodSignatureKindV1::Setter => SignatureKind::Setter,
        MethodSignatureKindV1::SubscriptGet { slots } => SignatureKind::SubscriptGet(slots),
        MethodSignatureKindV1::SubscriptSet { slots } => SignatureKind::SubscriptSet(slots),
    };
    let rest = signature.rest.map(|rest| {
        let labels = rest.fixed_labels.iter().map(|label| vm.interner.intern(label)).collect::<Vec<_>>().into_boxed_slice();
        let mode = match rest.mode {
            RestModeV1::Positional { param_index } => RestMode::Positional { param_index },
            RestModeV1::Labeled { param_index } => RestMode::Labeled { param_index },
            RestModeV1::Split {
                positional_param_index,
                labeled_param_index,
            } => RestMode::Split {
                positional_param_index,
                labeled_param_index,
            },
            RestModeV1::Complete { param_index } => RestMode::Complete { param_index },
        };
        RestLayout::new(rest.fixed_positionals, labels, mode)
    });
    Signature::new_with_arity(selector, kind, signature.positional_arity, rest)
}

fn portable_selector_pattern(
    pattern: phalcom_compiler_artifact::executable::constant::SelectorPatternV1,
) -> PhResult<phalcom_common::selector::SelectorPattern> {
    use phalcom_common::selector::{SelectorBase, SelectorKind, SelectorKindPattern, SelectorPattern, SelectorSlot};
    use phalcom_compiler_artifact::executable::constant::{SelectorBaseV1, SelectorKindPatternV1, SelectorKindV1, SelectorSlotV1};
    let slot = |slot| match slot {
        SelectorSlotV1::Positional => SelectorSlot::Positional,
        SelectorSlotV1::Label { label } => SelectorSlot::Label(label),
    };
    let base = match pattern.base {
        SelectorBaseV1::Named { name } => SelectorBase::Named(name),
        SelectorBaseV1::Subscript => SelectorBase::Subscript,
    };
    let kind = match pattern.kind {
        SelectorKindPatternV1::AnyNamed => SelectorKindPattern::AnyNamed,
        SelectorKindPatternV1::NamedAccessors => SelectorKindPattern::NamedAccessors,
        SelectorKindPatternV1::AnySubscript => SelectorKindPattern::AnySubscript,
        SelectorKindPatternV1::Exact { kind } => SelectorKindPattern::Exact(match kind {
            SelectorKindV1::Getter => SelectorKind::Getter,
            SelectorKindV1::Setter => SelectorKind::Setter,
            SelectorKindV1::Method => SelectorKind::Method,
            SelectorKindV1::SubscriptGet => SelectorKind::SubscriptGet,
            SelectorKindV1::SubscriptSet => SelectorKind::SubscriptSet,
        }),
    };
    SelectorPattern::new(
        base,
        kind,
        pattern.prefix.into_iter().map(slot).collect::<Vec<_>>(),
        pattern.suffix.into_iter().map(slot).collect::<Vec<_>>(),
        pattern.has_gap,
    )
        .map_err(|error| internal(error.to_string()))
}
