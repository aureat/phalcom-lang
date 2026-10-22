//! Source-free relocation of validated portable executable semantic side tables.

use crate::chunk::{
    ConditionalFamilyDescriptor, ConditionalFamilyEntry, ExecutableSemanticPool, ExecutableTraitInvocationSpec,
    ExecutableTraitSelectionTarget,
};
use crate::error::{PhError, PhResult, RuntimeError};
use crate::modules::semantic_lowering::{
    AnonymousProductConstructionKind, AnonymousProductConstructionLoweringSpec, DataComponentLoweringSpec, DataConstructionLoweringSpec,
    DataDeclarationLoweringSpec, EnumLoweringSpec, ExecutableFamilyCandidate, ExecutableFamilyCandidateSet, ExecutableFamilyDescriptor,
    ExecutableFamilyEntry, ExecutableFamilyTarget, ExecutableInvocationTarget, ExecutableRestMode, TraitRequirementInvocationSpec,
    VariantFieldLoweringSpec, VariantLoweringSpec,
};
use crate::product::{ProductComponentSpec, ProductLayoutSpec, ProductSlotRepr};
use crate::typing::{
    RuntimeCallEnvironmentRecipe, RuntimeConformanceSlotRecipe, RuntimeKindRef, RuntimeOverlayKindNode, RuntimeOverlayTypeNode,
    RuntimeRecordField, RuntimeRecordRow, RuntimeStaticArgument, RuntimeTupleElement, RuntimeCallableParameter,
    RuntimeOverlayTypeParameter, RuntimeTypeRecipe, RuntimeTypeRef,
};
use crate::vm::VM;
use phalcom_common::selector::{Selector, SelectorBase, SelectorKind, SelectorSlot};
use phalcom_compiler_artifact::executable::constant::{
    SelectorBaseV1, SelectorKindPatternV1, SelectorKindV1, SelectorPatternV1, SelectorSlotV1,
};
use phalcom_compiler_artifact::executable::prototype::ConstantId;
use phalcom_compiler_artifact::executable::semantic as portable;
use phalcom_semantic::enum_semantics::VariantShape;
use phalcom_semantic::identity::{DataComponentId, DataConstructorId, VariantFieldId};
use phalcom_semantic::metadata::stable_identity::{
    from_stable_callable, from_stable_declaration, from_stable_dispatch_side, from_stable_variant,
};
use phalcom_semantic::traits::TraitRequirementId;
use phalcom_semantic::types::family::{FamilyMemberTypeKind, FamilyOperationShape};
use phalcom_type_meta::fingerprint::Fingerprint128;
use phalcom_type_meta::generic::{StableTypeParameterOwnerRef, StableTypeParameterRef};
use std::cell::Cell;
use std::sync::Arc;

fn internal(message: impl Into<String>) -> PhError {
    RuntimeError::Internal(message.into()).into()
}

pub(crate) fn materialize_semantic_pool(
    vm: &mut VM,
    source: &portable::PortableSemanticPoolV1,
    local_constants: &[ConstantId],
) -> PhResult<ExecutableSemanticPool> {
    SemanticPoolRelocator::new(vm, source, local_constants).materialize()
}

struct SemanticPoolRelocator<'a> {
    vm: &'a mut VM,
    source: &'a portable::PortableSemanticPoolV1,
    local_constants: &'a [ConstantId],
    type_memo: Vec<Option<RuntimeTypeRef>>,
    type_active: Vec<bool>,
    kind_memo: Vec<Option<RuntimeKindRef>>,
    kind_active: Vec<bool>,
    data_constructions: Vec<Arc<DataConstructionLoweringSpec>>,
    call_environments: Vec<Arc<RuntimeCallEnvironmentRecipe>>,
}

impl<'a> SemanticPoolRelocator<'a> {
    fn new(vm: &'a mut VM, source: &'a portable::PortableSemanticPoolV1, local_constants: &'a [ConstantId]) -> Self {
        Self {
            vm,
            source,
            local_constants,
            type_memo: vec![None; source.type_recipes.len()],
            type_active: vec![false; source.type_recipes.len()],
            kind_memo: vec![None; source.kind_recipes.len()],
            kind_active: vec![false; source.kind_recipes.len()],
            data_constructions: Vec::with_capacity(source.data_constructions.len()),
            call_environments: Vec::with_capacity(source.call_environments.len()),
        }
    }

    fn materialize(mut self) -> PhResult<ExecutableSemanticPool> {
        let mut output = ExecutableSemanticPool::new();

        for row in &self.source.enum_specs {
            output.enum_specs.push(Arc::new(self.enum_spec(row)?));
        }

        for row in &self.source.variant_targets {
            let variant = from_stable_variant(row).map_err(internal)?;
            let index = u16::try_from(output.variant_targets.len()).map_err(|_| internal("artifact variant target count exceeds u16"))?;
            if output.variant_target_index.insert(variant.clone(), index).is_some() {
                return Err(internal("validated artifact contains duplicate runtime variant target"));
            }
            output.variant_targets.push(variant);
        }

        for row in &self.source.data_specs {
            output.data_specs.push(Arc::new(self.data_spec(row)?));
        }

        for row in &self.source.data_targets {
            let owner = from_stable_declaration(row).map_err(internal)?;
            let constructor = DataConstructorId::new(owner);
            let index = u16::try_from(output.data_targets.len()).map_err(|_| internal("artifact data target count exceeds u16"))?;
            if output.data_target_index.insert(constructor.clone(), index).is_some() {
                return Err(internal("validated artifact contains duplicate runtime data target"));
            }
            output.data_targets.push(constructor);
        }

        for row in self.source.data_constructions.clone() {
            let construction = Arc::new(self.data_construction(&row)?);
            self.data_constructions.push(Arc::clone(&construction));
            output.data_constructions.push(construction);
            output.data_construction_caches.push(Cell::new(None));
        }

        for row in self.source.anonymous_products.clone() {
            output.anonymous_product_specs.push(Arc::new(self.anonymous_product(&row)?));
        }

        for row in self.source.call_environments.clone() {
            let recipe = Arc::new(self.call_environment(&row)?);
            self.call_environments.push(Arc::clone(&recipe));
            output.call_environment_recipes.push(recipe);
        }

        for row in self.source.associated_targets.clone() {
            output.associated_targets.push(self.invocation_target(&row)?);
            output.associated_target_caches.push(Cell::new(None));
        }

        for row in self.source.family_descriptors.clone() {
            let entries = row
                .entries
                .iter()
                .map(|entry| {
                    Ok(ExecutableFamilyEntry {
                        operation: family_operation(&entry.operation),
                        member_kind: match &entry.member_kind {
                            portable::PortableFamilyMemberKindV1::Value => FamilyMemberTypeKind::Value,
                            portable::PortableFamilyMemberKindV1::Callable => FamilyMemberTypeKind::Callable,
                        },
                        target: self.family_target(&entry.target)?,
                    })
                })
                .collect::<PhResult<Vec<_>>>()?;
            output.family_descriptors.push(Arc::new(ExecutableFamilyDescriptor {
                entries: entries.into_boxed_slice(),
            }));
        }

        for row in self.source.conditional_families.clone() {
            let entries = row
                .entries
                .iter()
                .map(|entry| self.conditional_family_entry(entry))
                .collect::<PhResult<Vec<_>>>()?;
            output.conditional_family_descriptors.push(ConditionalFamilyDescriptor {
                entries: entries.into_boxed_slice(),
            });
        }

        output.family_operations = self.source.family_operations.iter().map(family_operation).collect();

        for row in self.source.family_candidate_sets.clone() {
            let candidates = row
                .candidates
                .iter()
                .map(|candidate| {
                    Ok(ExecutableFamilyCandidate {
                        operation: family_operation(&candidate.operation),
                        target: candidate.target.as_ref().map(|target| self.invocation_target(target)).transpose()?,
                    })
                })
                .collect::<PhResult<Vec<_>>>()?;
            output.family_candidate_sets.push(ExecutableFamilyCandidateSet {
                candidates: candidates.into_boxed_slice(),
            });
        }

        for row in self.source.trait_invocations.clone() {
            output.runtime_trait_invocations.push(self.trait_invocation(&row)?);
        }

        for row in &self.source.trait_requirement_invocations {
            output.trait_requirement_invocations.push(TraitRequirementInvocationSpec {
                requirement: trait_requirement(&row.requirement)?,
                requirement_slot: row.requirement_slot,
            });
        }

        output.type_parameter_class_receivers = self
            .source
            .type_parameter_class_receivers
            .iter()
            .map(type_parameter)
            .collect::<PhResult<Vec<_>>>()?;

        // Artifact chunks carry only runtime trait evidence. The semantic proof
        // rows are a source-compiler projection input and are intentionally not
        // reconstructed from durable executable data.
        output.trait_invocations = Vec::new();

        Ok(output)
    }

    fn enum_spec(&self, row: &portable::PortableEnumSpecV1) -> PhResult<EnumLoweringSpec> {
        let owner = from_stable_declaration(&row.owner).map_err(internal)?;
        let representation = match &row.representation {
            portable::PortableAdtRepresentationV1::OptionImmediate => crate::adt::RuntimeAdtRepresentation::NativeOption,
            portable::PortableAdtRepresentationV1::TaggedHeap => crate::adt::RuntimeAdtRepresentation::General,
        };
        let variants = row
            .variants
            .iter()
            .map(|variant| {
                let id = from_stable_variant(&variant.variant).map_err(internal)?;
                let shape = match &variant.shape {
                    portable::PortableVariantShapeV1::Singleton => VariantShape::Singleton,
                    portable::PortableVariantShapeV1::Positional { .. } => VariantShape::Constructor,
                    portable::PortableVariantShapeV1::Named { .. } => {
                        return Err(internal("portable named enum payload has no current executable lowering representation"));
                    }
                };
                let payload_fields = variant
                    .fields
                    .iter()
                    .map(|field| VariantFieldLoweringSpec {
                        id: VariantFieldId::new(id.clone(), field.index),
                        local_name: field.local_name.clone().into_boxed_str(),
                        slot: field.slot,
                    })
                    .collect::<Vec<_>>()
                    .into_boxed_slice();
                Ok(VariantLoweringSpec {
                    id,
                    shape,
                    payload_fields,
                    layout: variant.layout.as_ref().map(product_layout),
                })
            })
            .collect::<PhResult<Vec<_>>>()?;
        Ok(EnumLoweringSpec {
            owner,
            representation,
            variants: variants.into_boxed_slice(),
        })
    }

    fn data_spec(&self, row: &portable::PortableDataSpecV1) -> PhResult<DataDeclarationLoweringSpec> {
        let owner = from_stable_declaration(&row.owner).map_err(internal)?;
        let components = row
            .components
            .iter()
            .map(|component| DataComponentLoweringSpec {
                id: DataComponentId::new(owner.clone(), component.index),
                local_name: component.local_name.clone().into_boxed_str(),
                logical_index: component.logical_index,
            })
            .collect::<Vec<_>>()
            .into_boxed_slice();
        Ok(DataDeclarationLoweringSpec {
            owner,
            layout: row.layout.as_ref().map(product_layout),
            components,
        })
    }

    fn data_construction(&mut self, row: &portable::PortableDataConstructionV1) -> PhResult<DataConstructionLoweringSpec> {
        let owner = from_stable_declaration(&row.constructor).map_err(internal)?;
        Ok(DataConstructionLoweringSpec {
            constructor: DataConstructorId::new(owner),
            exact_type: None,
            type_recipe: self.runtime_type_recipe(&row.type_recipe)?,
            layout: product_layout(&row.layout),
            argument_to_component: row.argument_to_component.clone().into_boxed_slice(),
            component_to_logical: row.component_to_logical.clone().into_boxed_slice(),
        })
    }

    fn anonymous_product(&mut self, row: &portable::PortableAnonymousProductV1) -> PhResult<AnonymousProductConstructionLoweringSpec> {
        let kind = match &row.kind {
            portable::PortableAnonymousProductKindV1::Tuple { positional_len, labels } => AnonymousProductConstructionKind::Tuple {
                positional_len: *positional_len,
                labels: labels.iter().cloned().map(String::into_boxed_str).collect::<Vec<_>>().into_boxed_slice(),
            },
            portable::PortableAnonymousProductKindV1::Record {
                logical_keys,
                source_to_logical,
            } => AnonymousProductConstructionKind::Record {
                logical_keys: logical_keys.iter().map(field_key).collect::<PhResult<Vec<_>>>()?.into_boxed_slice(),
                source_to_logical: source_to_logical.clone().into_boxed_slice(),
            },
            portable::PortableAnonymousProductKindV1::Pack { positional_len, labels } => AnonymousProductConstructionKind::Pack {
                positional_len: *positional_len,
                labels: labels.iter().cloned().map(String::into_boxed_str).collect::<Vec<_>>().into_boxed_slice(),
            },
        };
        Ok(AnonymousProductConstructionLoweringSpec {
            kind,
            layout: product_layout(&row.layout),
            type_recipe: self.runtime_type_recipe(&row.type_recipe)?,
        })
    }

    fn call_environment(&mut self, row: &portable::PortableCallEnvironmentV1) -> PhResult<RuntimeCallEnvironmentRecipe> {
        Ok(RuntimeCallEnvironmentRecipe {
            callable: row.callable.clone(),
            bindings: row
                .bindings
                .iter()
                .map(|binding| Ok((type_parameter(&binding.parameter)?, self.runtime_type_recipe(&binding.recipe)?)))
                .collect::<PhResult<Vec<_>>>()?
                .into_boxed_slice(),
            conformance_slots: row
                .conformance_slots
                .iter()
                .map(|slot| self.conformance_slot(slot))
                .collect::<PhResult<Vec<_>>>()?
                .into_boxed_slice(),
        })
    }

    fn conformance_slot(&mut self, row: &portable::PortableConformanceSlotV1) -> PhResult<RuntimeConformanceSlotRecipe> {
        Ok(match row {
            portable::PortableConformanceSlotV1::Selection { callable, selector, nested } => RuntimeConformanceSlotRecipe::Selection {
                callable: from_stable_callable(callable).map_err(internal)?,
                selector: exact_selector_pattern(selector)?,
                nested_slots: nested
                    .iter()
                    .map(|entry| Ok((entry.slot, self.conformance_slot(&entry.recipe)?)))
                    .collect::<PhResult<Vec<_>>>()?
                    .into_boxed_slice(),
            },
            portable::PortableConformanceSlotV1::Forwarded { slot } => RuntimeConformanceSlotRecipe::Forwarded(*slot),
            portable::PortableConformanceSlotV1::DerivedConditional {
                callable,
                selector,
                bindings,
                nested,
            } => RuntimeConformanceSlotRecipe::DerivedConditional {
                callable: from_stable_callable(callable).map_err(internal)?,
                selector: exact_selector_pattern(selector)?,
                type_bindings: bindings
                    .iter()
                    .map(|binding| Ok((type_parameter(&binding.parameter)?, self.runtime_type_recipe(&binding.recipe)?)))
                    .collect::<PhResult<Vec<_>>>()?
                    .into_boxed_slice(),
                nested_slots: nested
                    .iter()
                    .map(|entry| Ok((entry.slot, self.conformance_slot(&entry.recipe)?)))
                    .collect::<PhResult<Vec<_>>>()?
                    .into_boxed_slice(),
            },
        })
    }

    fn invocation_target(&mut self, row: &portable::PortableInvocationTargetV1) -> PhResult<ExecutableInvocationTarget> {
        Ok(match row {
            portable::PortableInvocationTargetV1::Behavioral {
                lookup_owner,
                callable,
                operation,
                rest_mode,
            } => ExecutableInvocationTarget::Behavioral {
                lookup_owner: from_stable_declaration(lookup_owner).map_err(internal)?,
                callable: from_stable_callable(callable).map_err(internal)?,
                operation: family_operation(operation),
                rest_mode: rest_mode_value(rest_mode),
            },
            portable::PortableInvocationTargetV1::VariantConstructor { variant } => ExecutableInvocationTarget::VariantConstructor {
                variant: from_stable_variant(variant).map_err(internal)?,
            },
            portable::PortableInvocationTargetV1::DataConstructor {
                constructor,
                construction,
            } => {
                let owner = from_stable_declaration(constructor).map_err(internal)?;
                let construction = self
                    .data_constructions
                    .get(*construction as usize)
                    .cloned()
                    .ok_or_else(|| internal(format!("artifact data construction {} is absent", construction)))?;
                ExecutableInvocationTarget::DataConstructor {
                    constructor: DataConstructorId::new(owner),
                    construction,
                }
            }
        })
    }

    fn family_target(&mut self, row: &portable::PortableFamilyTargetV1) -> PhResult<ExecutableFamilyTarget> {
        Ok(match row {
            portable::PortableFamilyTargetV1::Singleton { variant } => ExecutableFamilyTarget::Singleton {
                variant: from_stable_variant(variant).map_err(internal)?,
            },
            portable::PortableFamilyTargetV1::Behavioral { target } => ExecutableFamilyTarget::Behavioral {
                target: self.invocation_target(target)?,
            },
            portable::PortableFamilyTargetV1::VariantConstructor { variant } => ExecutableFamilyTarget::VariantConstructor {
                variant: from_stable_variant(variant).map_err(internal)?,
            },
            portable::PortableFamilyTargetV1::DataConstructor {
                constructor,
                construction,
            } => {
                let owner = from_stable_declaration(constructor).map_err(internal)?;
                let construction = self
                    .data_constructions
                    .get(*construction as usize)
                    .cloned()
                    .ok_or_else(|| internal(format!("artifact family data construction {} is absent", construction)))?;
                ExecutableFamilyTarget::DataConstructor {
                    constructor: DataConstructorId::new(owner),
                    construction,
                }
            }
        })
    }

    fn conditional_family_entry(&mut self, row: &portable::PortableConditionalFamilyEntryV1) -> PhResult<ConditionalFamilyEntry> {
        let local = self
            .local_constants
            .iter()
            .position(|candidate| *candidate == row.fallback_method)
            .ok_or_else(|| internal(format!("conditional-family fallback constant {} is not local to its prototype", row.fallback_method.0)))?;
        let fallback_method = u16::try_from(local).map_err(|_| internal("conditional-family fallback local constant exceeds u16"))?;
        let call_environment_recipe = row
            .call_environment
            .map(|id| {
                self.call_environments
                    .get(id.0 as usize)
                    .cloned()
                    .ok_or_else(|| internal(format!("conditional-family call environment {} is absent", id.0)))
            })
            .transpose()?;
        Ok(ConditionalFamilyEntry {
            operation: family_operation(&row.operation),
            selector: self.vm.interner.intern(&row.selector),
            declaring_owner: from_stable_declaration(&row.declaring_owner).map_err(internal)?,
            side: from_stable_dispatch_side(row.side),
            fallback_method,
            call_environment_recipe,
        })
    }

    fn trait_invocation(&mut self, row: &portable::PortableTraitInvocationV1) -> PhResult<ExecutableTraitInvocationSpec> {
        let mut requirement_targets = row
            .requirement_targets
            .iter()
            .map(|target| Ok((trait_requirement(&target.requirement)?, self.trait_target(&target.target)?)))
            .collect::<PhResult<Vec<_>>>()?;
        requirement_targets.sort_by(|(left, _), (right, _)| left.cmp(right));

        Ok(ExecutableTraitInvocationSpec {
            selection: self.trait_target(&row.selection)?,
            selection_is_trait_default: row.selection_is_trait_default,
            requirement: trait_requirement(&row.requirement)?,
            requirement_slot: row.requirement_slot,
            requirement_slots: row
                .requirement_slots
                .iter()
                .map(|slot| Ok((trait_requirement(&slot.requirement)?, slot.slot)))
                .collect::<PhResult<Vec<_>>>()?
                .into_boxed_slice(),
            requirement_targets: requirement_targets.into_boxed_slice(),
            antecedent_slots: row
                .antecedent_slots
                .iter()
                .map(|slot| {
                    Ok((
                        slot.slot,
                        self.trait_target(&slot.target)?,
                        Selector::try_decode_exact(&slot.selector).map_err(|error| internal(error.to_string()))?,
                    ))
                })
                .collect::<PhResult<Vec<_>>>()?
                .into_boxed_slice(),
            type_bindings: row
                .type_bindings
                .iter()
                .map(|binding| Ok((type_parameter(&binding.parameter)?, self.runtime_type_recipe(&binding.recipe)?)))
                .collect::<PhResult<Vec<_>>>()?
                .into_boxed_slice(),
            call_environment_recipe: row
                .call_environment
                .map(|id| {
                    self.call_environments
                        .get(id.0 as usize)
                        .cloned()
                        .ok_or_else(|| internal(format!("trait call environment {} is absent", id.0)))
                })
                .transpose()?,
        })
    }

    fn trait_target(&mut self, row: &portable::PortableTraitSelectionTargetV1) -> PhResult<ExecutableTraitSelectionTarget> {
        Ok(match row {
            portable::PortableTraitSelectionTargetV1::Callable { callable } => {
                ExecutableTraitSelectionTarget::Callable(from_stable_callable(callable).map_err(internal)?)
            }
            portable::PortableTraitSelectionTargetV1::DataComponent { component, .. } => {
                ExecutableTraitSelectionTarget::DataComponent(DataComponentId::new(
                    from_stable_declaration(&component.owner).map_err(internal)?,
                    component.index,
                ))
            }
        })
    }

    fn runtime_type_recipe(&mut self, row: &portable::PortableRuntimeTypeRecipeV1) -> PhResult<RuntimeTypeRecipe> {
        Ok(match row {
            portable::PortableRuntimeTypeRecipeV1::Closed { ty } => RuntimeTypeRecipe::Closed(self.type_recipe(*ty)?),
            portable::PortableRuntimeTypeRecipeV1::Template { ty } => RuntimeTypeRecipe::Template(self.type_recipe(*ty)?),
        })
    }

    fn type_recipe(&mut self, id: portable::PortableTypeRecipeId) -> PhResult<RuntimeTypeRef> {
        let index = id.0 as usize;
        if let Some(value) = self.type_memo.get(index).and_then(|value| *value) {
            return Ok(value);
        }
        let Some(active) = self.type_active.get_mut(index) else {
            return Err(internal(format!("artifact type recipe {} is absent", id.0)));
        };
        if *active {
            return Err(internal(format!("artifact type recipe {} is cyclic", id.0)));
        }
        *active = true;
        let row = self
            .source
            .type_recipes
            .get(index)
            .cloned()
            .ok_or_else(|| internal(format!("artifact type recipe {} is absent", id.0)))?;

        let result = match row {
            portable::PortableTypeRecipeV1::Canonical { digest } => self.canonical_type(&digest)?,
            portable::PortableTypeRecipeV1::Parameter { parameter } => self.parameter_type(&type_parameter(&parameter)?)?,
            portable::PortableTypeRecipeV1::Applied { origin, arguments } => {
                let origin = self.type_recipe(origin)?;
                let arguments = arguments
                    .into_iter()
                    .map(|argument| self.type_recipe(argument).map(RuntimeStaticArgument::Term))
                    .collect::<PhResult<Vec<_>>>()?;
                self.vm.runtime_typing_context.overlay.type_ref(RuntimeOverlayTypeNode::Applied {
                    origin,
                    arguments: arguments.into_boxed_slice(),
                })
            }
            portable::PortableTypeRecipeV1::Tuple { elements } => {
                let elements = elements
                    .into_iter()
                    .map(|ty| {
                        Ok(RuntimeTupleElement {
                            label: None,
                            ty: self.type_recipe(ty)?,
                        })
                    })
                    .collect::<PhResult<Vec<_>>>()?;
                self.vm
                    .runtime_typing_context
                    .overlay
                    .type_ref(RuntimeOverlayTypeNode::Tuple(elements.into_boxed_slice()))
            }
            portable::PortableTypeRecipeV1::Record { labels, values } => {
                if labels.len() != values.len() {
                    return Err(internal("artifact record type recipe label/value arity mismatch"));
                }
                let fields = labels
                    .into_iter()
                    .zip(values)
                    .map(|(label, ty)| {
                        Ok(RuntimeRecordField {
                            key: phalcom_semantic::FieldKey::string(label),
                            ty: self.type_recipe(ty)?,
                        })
                    })
                    .collect::<PhResult<Vec<_>>>()?;
                self.vm
                    .runtime_typing_context
                    .overlay
                    .type_ref(RuntimeOverlayTypeNode::Record(fields.into_boxed_slice()))
            }
            portable::PortableTypeRecipeV1::Pack { .. } => {
                return Err(internal("portable Pack type recipe has no current RuntimeTypingOverlay representation"));
            }
            portable::PortableTypeRecipeV1::Union { members } => {
                let members = members.into_iter().map(|member| self.type_recipe(member)).collect::<PhResult<Vec<_>>>()?;
                self.vm
                    .runtime_typing_context
                    .overlay
                    .type_ref(RuntimeOverlayTypeNode::Union(members.into_boxed_slice()))
            }
            portable::PortableTypeRecipeV1::Dynamic => self
                .vm
                .runtime_typing_context
                .overlay
                .type_ref(RuntimeOverlayTypeNode::Special("Dynamic".into())),
            portable::PortableTypeRecipeV1::Nominal { class } => {
                let class = resolve_class(self.vm, &class)?;
                self.vm.runtime_typing_context.overlay.type_ref(RuntimeOverlayTypeNode::Nominal { class })
            }
            portable::PortableTypeRecipeV1::TupleLabeled { elements } => {
                let elements = elements
                    .into_iter()
                    .map(|element| {
                        Ok(RuntimeTupleElement {
                            label: element.label.map(String::into_boxed_str),
                            ty: self.type_recipe(element.ty)?,
                        })
                    })
                    .collect::<PhResult<Vec<_>>>()?;
                self.vm
                    .runtime_typing_context
                    .overlay
                    .type_ref(RuntimeOverlayTypeNode::Tuple(elements.into_boxed_slice()))
            }
            portable::PortableTypeRecipeV1::RecordKeys { fields } => {
                let fields = fields
                    .into_iter()
                    .map(|field| {
                        Ok(RuntimeRecordField {
                            key: field_key(&field.key)?,
                            ty: self.type_recipe(field.ty)?,
                        })
                    })
                    .collect::<PhResult<Vec<_>>>()?;
                self.vm
                    .runtime_typing_context
                    .overlay
                    .type_ref(RuntimeOverlayTypeNode::Record(fields.into_boxed_slice()))
            }
            portable::PortableTypeRecipeV1::Callable {
                parameters,
                return_type,
            } => {
                let parameters = parameters
                    .into_iter()
                    .map(|parameter| {
                        Ok(RuntimeCallableParameter {
                            label: parameter.label.map(String::into_boxed_str),
                            ty: self.type_recipe(parameter.ty)?,
                            rest: parameter.rest,
                        })
                    })
                    .collect::<PhResult<Vec<_>>>()?;
                let return_type = self.type_recipe(return_type)?;
                self.vm.runtime_typing_context.overlay.type_ref(RuntimeOverlayTypeNode::Callable {
                    parameters: parameters.into_boxed_slice(),
                    return_type,
                })
            }
            portable::PortableTypeRecipeV1::TypeLambda { parameters, body } => {
                let parameters = parameters
                    .into_iter()
                    .map(|parameter| {
                        Ok(RuntimeOverlayTypeParameter {
                            name: parameter.name.into_boxed_str(),
                            kind: self.kind_recipe(parameter.kind)?,
                        })
                    })
                    .collect::<PhResult<Vec<_>>>()?;
                let body = self.type_recipe(body)?;
                self.vm.runtime_typing_context.overlay.type_ref(RuntimeOverlayTypeNode::TypeLambda {
                    parameters: parameters.into_boxed_slice(),
                    body,
                })
            }
            portable::PortableTypeRecipeV1::Special { name } => self
                .vm
                .runtime_typing_context
                .overlay
                .type_ref(RuntimeOverlayTypeNode::Special(name.into_boxed_str())),
            portable::PortableTypeRecipeV1::SelfType { class } => {
                let class = class.as_ref().map(|class| resolve_class(self.vm, class)).transpose()?;
                self.vm.runtime_typing_context.overlay.type_ref(RuntimeOverlayTypeNode::SelfType(class))
            }
            portable::PortableTypeRecipeV1::AppliedStatic { origin, arguments } => {
                let origin = self.type_recipe(origin)?;
                let arguments = arguments
                    .into_iter()
                    .map(|argument| self.static_argument(argument))
                    .collect::<PhResult<Vec<_>>>()?;
                self.vm.runtime_typing_context.overlay.type_ref(RuntimeOverlayTypeNode::Applied {
                    origin,
                    arguments: arguments.into_boxed_slice(),
                })
            }
        };

        self.type_active[index] = false;
        self.type_memo[index] = Some(result);
        Ok(result)
    }

    fn static_argument(&mut self, row: portable::PortableStaticTypeArgumentV1) -> PhResult<RuntimeStaticArgument> {
        Ok(match row {
            portable::PortableStaticTypeArgumentV1::Term { ty } => RuntimeStaticArgument::Term(self.type_recipe(ty)?),
            portable::PortableStaticTypeArgumentV1::RecordRow { fields, tail } => RuntimeStaticArgument::RecordRow(RuntimeRecordRow {
                fields: fields
                    .into_iter()
                    .map(|field| {
                        Ok(RuntimeRecordField {
                            key: field_key(&field.key)?,
                            ty: self.type_recipe(field.ty)?,
                        })
                    })
                    .collect::<PhResult<Vec<_>>>()?
                    .into_boxed_slice(),
                tail: tail.as_ref().map(type_parameter).transpose()?,
            }),
            portable::PortableStaticTypeArgumentV1::DynamicType => RuntimeStaticArgument::DynamicType,
        })
    }

    fn kind_recipe(&mut self, id: portable::PortableKindRecipeId) -> PhResult<RuntimeKindRef> {
        let index = id.0 as usize;
        if let Some(value) = self.kind_memo.get(index).and_then(|value| *value) {
            return Ok(value);
        }
        let Some(active) = self.kind_active.get_mut(index) else {
            return Err(internal(format!("artifact kind recipe {} is absent", id.0)));
        };
        if *active {
            return Err(internal(format!("artifact kind recipe {} is cyclic", id.0)));
        }
        *active = true;
        let row = self
            .source
            .kind_recipes
            .get(index)
            .cloned()
            .ok_or_else(|| internal(format!("artifact kind recipe {} is absent", id.0)))?;
        let result = match row {
            portable::PortableKindRecipeV1::Canonical { digest } => self.canonical_kind(&digest)?,
            portable::PortableKindRecipeV1::Type => self.vm.runtime_typing_context.overlay.kind_ref(RuntimeOverlayKindNode::Type),
            portable::PortableKindRecipeV1::Arrow { parameters, result } => {
                let parameters = parameters
                    .into_iter()
                    .map(|parameter| self.kind_recipe(parameter))
                    .collect::<PhResult<Vec<_>>>()?;
                let result = self.kind_recipe(result)?;
                self.vm.runtime_typing_context.overlay.kind_ref(RuntimeOverlayKindNode::Arrow {
                    parameters: parameters.into_boxed_slice(),
                    result: Box::new(result),
                })
            }
        };
        self.kind_active[index] = false;
        self.kind_memo[index] = Some(result);
        Ok(result)
    }

    fn canonical_type(&self, digest: &[u8; 32]) -> PhResult<RuntimeTypeRef> {
        for pool in self.vm.typing_registry.pools() {
            for (index, entry) in pool.bundle.types.iter().enumerate() {
                if stable_semantic_digest(entry.structural_fingerprint) == *digest {
                    return Ok(RuntimeTypeRef::Base {
                        pool: pool.id,
                        node: phalcom_type_meta::type_node::TypeNodeId(index as u32),
                    });
                }
            }
        }
        Err(internal("validated artifact canonical type digest is absent from the installed typing registry"))
    }

    fn canonical_kind(&self, digest: &[u8; 32]) -> PhResult<RuntimeKindRef> {
        for pool in self.vm.typing_registry.pools() {
            for (index, entry) in pool.bundle.kinds.iter().enumerate() {
                if stable_semantic_digest(entry.structural_fingerprint) == *digest {
                    return Ok(RuntimeKindRef::Base {
                        pool: pool.id,
                        node: phalcom_type_meta::kind::KindNodeId(index as u32),
                    });
                }
            }
        }
        Err(internal("validated artifact canonical kind digest is absent from the installed typing registry"))
    }

    fn parameter_type(&self, parameter: &StableTypeParameterRef) -> PhResult<RuntimeTypeRef> {
        for pool in self.vm.typing_registry.pools() {
            for (index, entry) in pool.bundle.types.iter().enumerate() {
                if matches!(&entry.form, phalcom_type_meta::type_node::TypeNode::Parameter(candidate) if candidate == parameter) {
                    return Ok(RuntimeTypeRef::Base {
                        pool: pool.id,
                        node: phalcom_type_meta::type_node::TypeNodeId(index as u32),
                    });
                }
            }
        }
        Err(internal(format!("validated artifact type parameter {parameter:?} has no installed runtime type node")))
    }
}

fn resolve_class(vm: &VM, reference: &phalcom_compiler_artifact::executable::constant::StableClassRefV1) -> PhResult<crate::heap::ClassId> {
    let class = vm
        .typing_registry
        .resolve_nominal(&reference.declaration)
        .ok_or_else(|| internal(format!("artifact class {:?} is not materialized", reference.declaration)))?;
    Ok(match reference.side {
        phalcom_type_meta::identity::StableDispatchSide::Instance => class,
        phalcom_type_meta::identity::StableDispatchSide::Class => vm.heap.class(class).class,
    })
}

fn product_layout(row: &portable::PortableProductLayoutV1) -> ProductLayoutSpec {
    ProductLayoutSpec::new(
        row.components
            .iter()
            .map(|component| ProductComponentSpec {
                logical_index: component.logical_index,
                repr: match &component.repr {
                    portable::PortableProductSlotReprV1::Int64 => ProductSlotRepr::Int64,
                    portable::PortableProductSlotReprV1::Float64 => ProductSlotRepr::Float64,
                    portable::PortableProductSlotReprV1::Bool => ProductSlotRepr::Bool,
                    portable::PortableProductSlotReprV1::Symbol => ProductSlotRepr::Symbol,
                    portable::PortableProductSlotReprV1::Value => ProductSlotRepr::Value,
                },
            })
            .collect(),
    )
}

fn type_parameter(row: &portable::PortableTypeParameterRefV1) -> PhResult<StableTypeParameterRef> {
    Ok(StableTypeParameterRef {
        owner: match &row.owner {
            portable::PortableTypeParameterOwnerV1::Declaration { owner } => StableTypeParameterOwnerRef::Declaration(owner.clone()),
            portable::PortableTypeParameterOwnerV1::Callable { owner } => StableTypeParameterOwnerRef::Callable(owner.clone()),
            portable::PortableTypeParameterOwnerV1::Implementation { owner } => StableTypeParameterOwnerRef::Impl(owner.clone()),
        },
        index: row.index,
    })
}

fn field_key(row: &portable::PortableFieldKeyV1) -> PhResult<phalcom_semantic::FieldKey> {
    Ok(match row {
        portable::PortableFieldKeyV1::Symbol { text } => phalcom_semantic::FieldKey::symbol(text.clone()),
        portable::PortableFieldKeyV1::String { text } => phalcom_semantic::FieldKey::string(text.clone()),
        portable::PortableFieldKeyV1::Integer { decimal } => {
            let value = decimal
                .parse::<num_bigint::BigInt>()
                .map_err(|error| internal(format!("invalid portable integer field key {decimal:?}: {error}")))?;
            phalcom_semantic::FieldKey::int(value.into())
        }
        portable::PortableFieldKeyV1::Boolean { value } => phalcom_semantic::FieldKey::bool(*value),
    })
}

fn family_operation(row: &portable::PortableFamilyOperationV1) -> FamilyOperationShape {
    FamilyOperationShape {
        kind: match &row.kind {
            portable::PortableSelectorKindV1::Getter => phalcom_common::selector::SelectorKind::Getter,
            portable::PortableSelectorKindV1::Setter => phalcom_common::selector::SelectorKind::Setter,
            portable::PortableSelectorKindV1::Method => phalcom_common::selector::SelectorKind::Method,
            portable::PortableSelectorKindV1::SubscriptGet => phalcom_common::selector::SelectorKind::SubscriptGet,
            portable::PortableSelectorKindV1::SubscriptSet => phalcom_common::selector::SelectorKind::SubscriptSet,
        },
        slots: row
            .slots
            .iter()
            .map(|slot| match slot {
                portable::PortableSelectorSlotV1::Positional => phalcom_common::selector::SelectorSlot::Positional,
                portable::PortableSelectorSlotV1::Label { label } => phalcom_common::selector::SelectorSlot::Label(label.clone()),
            })
            .collect::<Vec<_>>()
            .into_boxed_slice(),
    }
}

fn rest_mode_value(row: &portable::PortableRestModeV1) -> ExecutableRestMode {
    match row {
        portable::PortableRestModeV1::None => ExecutableRestMode::None,
        portable::PortableRestModeV1::Positional => ExecutableRestMode::Positional,
        portable::PortableRestModeV1::Labeled => ExecutableRestMode::Labeled,
        portable::PortableRestModeV1::Complete => ExecutableRestMode::Complete,
    }
}

fn trait_requirement(row: &portable::PortableTraitRequirementV1) -> PhResult<TraitRequirementId> {
    Ok(TraitRequirementId {
        owner: from_stable_declaration(&row.owner).map_err(internal)?,
        selector: Selector::try_decode_exact(&row.selector).map_err(|error| internal(error.to_string()))?,
        side: from_stable_dispatch_side(row.side),
    })
}

fn exact_selector_pattern(row: &SelectorPatternV1) -> PhResult<Selector> {
    if row.has_gap || !row.suffix.is_empty() {
        return Err(internal("runtime conformance selector must be an exact selector pattern"));
    }
    let kind = match &row.kind {
        SelectorKindPatternV1::Exact { kind } => match kind {
            SelectorKindV1::Getter => SelectorKind::Getter,
            SelectorKindV1::Setter => SelectorKind::Setter,
            SelectorKindV1::Method => SelectorKind::Method,
            SelectorKindV1::SubscriptGet => SelectorKind::SubscriptGet,
            SelectorKindV1::SubscriptSet => SelectorKind::SubscriptSet,
        },
        SelectorKindPatternV1::AnyNamed | SelectorKindPatternV1::NamedAccessors | SelectorKindPatternV1::AnySubscript => {
            return Err(internal("runtime conformance selector pattern is not exact"));
        }
    };
    let base = match &row.base {
        SelectorBaseV1::Named { name } => SelectorBase::Named(name.clone()),
        SelectorBaseV1::Subscript => SelectorBase::Subscript,
    };
    let slots = row
        .prefix
        .iter()
        .map(|slot| match slot {
            SelectorSlotV1::Positional => SelectorSlot::Positional,
            SelectorSlotV1::Label { label } => SelectorSlot::Label(label.clone()),
        })
        .collect::<Vec<_>>()
        .into_boxed_slice();
    Selector::new(base, kind, slots).map_err(|error| internal(error.to_string()))
}

fn stable_semantic_digest(fingerprint: Fingerprint128) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new_derive_key("phalcom.telosc.executable.semantic-type.v1");
    hasher.update(fingerprint.as_bytes());
    *hasher.finalize().as_bytes()
}
