//! Portable projection of the executable semantic side tables attached by the emitter.

use crate::chunk::ExecutableSemanticPool;
use crate::modules::semantic_lowering::{DataConstructionLoweringSpec, ExecutableFamilyTarget, ExecutableInvocationTarget, ExecutableRestMode};
use crate::typing::{RuntimeCallEnvironmentRecipe, RuntimeConformanceSlotRecipe, RuntimeTypeRecipe, RuntimeTypeRef};
use phalcom_compiler_artifact::executable::constant::{SelectorPatternV1, StableClassRefV1};
use phalcom_compiler_artifact::executable::prototype::ConstantId;
use phalcom_compiler_artifact::executable::semantic::*;
use phalcom_semantic::impls::RequirementSelectionTemplate;
use phalcom_semantic::metadata::stable_identity::{to_stable_callable, to_stable_declaration, to_stable_impl, to_stable_variant};
use phalcom_semantic::snapshot::SemanticSnapshot;
use phalcom_type_meta::fingerprint::Fingerprint128;
use phalcom_type_meta::generic::StableTypeParameterOwnerRef;
use std::sync::Arc;

#[derive(Debug)]
pub(crate) struct SemanticProjectionError {
    pub family: &'static str,
    pub reason: String,
}

type ProjectionResult<T> = Result<T, SemanticProjectionError>;

fn fail<T>(family: &'static str, reason: impl Into<String>) -> ProjectionResult<T> {
    Err(SemanticProjectionError { family, reason: reason.into() })
}

pub(crate) fn project_semantic_pool(
    vm: &crate::vm::VM,
    snapshot: &SemanticSnapshot,
    source: &ExecutableSemanticPool,
    constants: &[ConstantId],
) -> ProjectionResult<PortableSemanticPoolV1> {
    // Keep this exhaustive so every emitter-pool field gets an explicit
    // portable, reconstructed, or runtime-only disposition.
    let ExecutableSemanticPool {
        enum_specs: _,
        variant_targets: _,
        variant_target_index: _,
        data_specs: _,
        data_targets: _,
        data_target_index: _,
        data_constructions: _,
        data_construction_caches: _,
        anonymous_product_specs: _,
        call_environment_recipes: _,
        associated_targets: _,
        associated_target_caches: _,
        family_descriptors: _,
        conditional_family_descriptors: _,
        family_operations: _,
        family_candidate_sets: _,
        trait_invocations: _,
        runtime_trait_invocations: _,
        trait_requirement_invocations: _,
        type_parameter_class_receivers: _,
    } = source;
    let mut project = SemanticPoolProjector {
        vm,
        snapshot,
        source_constants: constants,
        output: empty_pool(),
        construction_specs: source.data_constructions.clone(),
        active_types: std::collections::HashSet::new(),
        active_kinds: std::collections::HashSet::new(),
    };

    project.output.variant_targets = source.variant_targets.iter().map(to_stable_variant).collect();
    project.output.data_targets = source.data_targets.iter().map(|target| to_stable_declaration(&target.owner)).collect();
    project.output.type_parameter_class_receivers = source.type_parameter_class_receivers.iter().map(portable_type_parameter).collect();

    for spec in source.enum_specs.iter() {
        project.output.enum_specs.push(project.enum_spec(spec)?);
    }
    for spec in source.data_specs.iter() {
        project.output.data_specs.push(project.data_spec(spec)?);
    }
    for spec in &source.anonymous_product_specs {
        let product = project.anonymous_product(spec)?;
        project.output.anonymous_products.push(product);
    }
    // Data-construction operands are positional bytecode references. Preserve
    // every source slot, including structurally equal recipes in distinct
    // slots; nested family targets may reuse this prefix when their recipe is
    // semantically identical.
    for construction in &source.data_constructions {
        let row = project.data_construction(construction)?;
        project.output.data_constructions.push(row);
    }

    // Bytecode operands address source slots, including equal recipes in
    // distinct slots. Only recipes reached through nested products may reuse
    // an existing entry after this source-indexed prefix is complete.
    for recipe in &source.call_environment_recipes {
        let row = project.call_environment(recipe)?;
        project.output.call_environments.push(row);
    }
    for target in &source.associated_targets {
        let target = project.invocation_target(target)?;
        project.output.associated_targets.push(target);
    }
    for descriptor in &source.family_descriptors {
        let mut entries = Vec::with_capacity(descriptor.entries.len());
        for entry in descriptor.entries.iter() {
            entries.push(PortableFamilyEntryV1 {
                operation: portable_operation(&entry.operation),
                member_kind: match entry.member_kind {
                    phalcom_semantic::types::family::FamilyMemberTypeKind::Value => PortableFamilyMemberKindV1::Value,
                    phalcom_semantic::types::family::FamilyMemberTypeKind::Callable => PortableFamilyMemberKindV1::Callable,
                },
                target: project.family_target(&entry.target)?,
            });
        }
        project.output.family_descriptors.push(PortableFamilyDescriptorV1 { entries });
    }
    for descriptor in &source.conditional_family_descriptors {
        let mut entries = Vec::with_capacity(descriptor.entries.len());
        for entry in descriptor.entries.iter() {
            let fallback_method = project
                .source_constants
                .get(entry.fallback_method as usize)
                .copied()
                .ok_or_else(|| SemanticProjectionError {
                    family: "conditional_family_descriptors",
                    reason: format!("fallback method constant {} is out of bounds", entry.fallback_method),
                })?;
            let call_environment = entry
                .call_environment_recipe
                .as_ref()
                .map(|recipe| project.ensure_call_environment(recipe))
                .transpose()?;
            entries.push(PortableConditionalFamilyEntryV1 {
                operation: portable_operation(&entry.operation),
                selector: project.vm.resolve_symbol(entry.selector).to_owned(),
                declaring_owner: to_stable_declaration(&entry.declaring_owner),
                side: portable_dispatch_side(entry.side),
                fallback_method,
                call_environment,
            });
        }
        project.output.conditional_families.push(PortableConditionalFamilyV1 { entries });
    }
    project.output.family_operations = source.family_operations.iter().map(portable_operation).collect();
    for candidates in &source.family_candidate_sets {
        let mut portable = Vec::with_capacity(candidates.candidates.len());
        for candidate in candidates.candidates.iter() {
            portable.push(PortableFamilyCandidateV1 {
                operation: portable_operation(&candidate.operation),
                target: candidate.target.as_ref().map(|target| project.invocation_target(target)).transpose()?,
            });
        }
        project.output.family_candidate_sets.push(PortableFamilyCandidateSetV1 { candidates: portable });
    }
    for invocation in &source.trait_invocations {
        let portable = project.trait_invocation(invocation)?;
        project.output.trait_invocations.push(portable);
    }
    for invocation in &source.trait_requirement_invocations {
        project.output.trait_requirement_invocations.push(PortableTraitRequirementInvocationV1 {
            requirement: portable_trait_requirement(&invocation.requirement),
            requirement_slot: invocation.requirement_slot,
        });
    }
    let mut output = project.output;
    canonicalize_type_recipe_ids(&mut output)?;
    Ok(output)
}

struct SemanticPoolProjector<'a> {
    vm: &'a crate::vm::VM,
    snapshot: &'a SemanticSnapshot,
    source_constants: &'a [ConstantId],
    output: PortableSemanticPoolV1,
    construction_specs: Vec<Arc<DataConstructionLoweringSpec>>,
    active_types: std::collections::HashSet<RuntimeTypeRef>,
    active_kinds: std::collections::HashSet<crate::typing::handle::RuntimeKindRef>,
}

fn append_recipe_key_part(output: &mut Vec<u8>, bytes: &[u8]) {
    output.extend_from_slice(&(bytes.len() as u64).to_be_bytes());
    output.extend_from_slice(bytes);
}

fn append_recipe_key_debug(output: &mut Vec<u8>, value: &impl std::fmt::Debug) {
    // Callers pass stable non-recipe payloads; recipe IDs are expanded recursively first.
    append_recipe_key_part(output, format!("{value:?}").as_bytes());
}

fn append_recipe_key_children(output: &mut Vec<u8>, children: impl IntoIterator<Item = Vec<u8>>) {
    let children = children.into_iter().collect::<Vec<_>>();
    output.extend_from_slice(&(children.len() as u64).to_be_bytes());
    for child in children {
        append_recipe_key_part(output, &child);
    }
}

fn portable_kind_recipe_key(
    recipes: &[PortableKindRecipeV1],
    id: PortableKindRecipeId,
    memo: &mut [Option<Vec<u8>>],
    active: &mut std::collections::BTreeSet<u32>,
    depth: usize,
) -> ProjectionResult<Vec<u8>> {
    if depth > 128 {
        return fail("kind_recipes", "portable kind key exceeds maximum structural depth");
    }
    let index = id.0 as usize;
    if let Some(key) = memo.get(index).and_then(Option::as_ref) {
        return Ok(key.clone());
    }
    if !active.insert(id.0) {
        return fail("kind_recipes", "portable kind recipe graph contains a cycle");
    }
    let recipe = recipes.get(index).ok_or_else(|| SemanticProjectionError {
        family: "kind_recipes",
        reason: format!("portable kind recipe {} is out of bounds", id.0),
    })?;
    let mut key = Vec::new();
    match recipe {
        PortableKindRecipeV1::Canonical { digest } => {
            key.push(0);
            key.extend_from_slice(digest);
        }
        PortableKindRecipeV1::Type => key.push(1),
        PortableKindRecipeV1::Arrow { parameters, result } => {
            key.push(2);
            let parameters = parameters
                .iter()
                .map(|parameter| portable_kind_recipe_key(recipes, *parameter, memo, active, depth + 1))
                .collect::<ProjectionResult<Vec<_>>>()?;
            append_recipe_key_children(&mut key, parameters);
            append_recipe_key_part(&mut key, &portable_kind_recipe_key(recipes, *result, memo, active, depth + 1)?);
        }
    }
    active.remove(&id.0);
    memo[index] = Some(key.clone());
    Ok(key)
}

fn portable_type_recipe_key(
    recipes: &[PortableTypeRecipeV1],
    kind_recipes: &[PortableKindRecipeV1],
    id: PortableTypeRecipeId,
    memo: &mut [Option<Vec<u8>>],
    active: &mut std::collections::BTreeSet<u32>,
    kind_memo: &mut [Option<Vec<u8>>],
    active_kinds: &mut std::collections::BTreeSet<u32>,
    depth: usize,
) -> ProjectionResult<Vec<u8>> {
    if depth > 128 {
        return fail("type_recipes", "portable type key exceeds maximum structural depth");
    }
    let index = id.0 as usize;
    if let Some(key) = memo.get(index).and_then(Option::as_ref) {
        return Ok(key.clone());
    }
    if !active.insert(id.0) {
        return fail("type_recipes", "portable type recipe graph contains a cycle");
    }
    let recipe = recipes.get(index).ok_or_else(|| SemanticProjectionError {
        family: "type_recipes",
        reason: format!("portable type recipe {} is out of bounds", id.0),
    })?;
    let mut key = Vec::new();
    match recipe {
        PortableTypeRecipeV1::Canonical { digest } => {
            key.push(0);
            key.extend_from_slice(digest);
        }
        PortableTypeRecipeV1::Parameter { parameter } => {
            key.push(1);
            append_recipe_key_debug(&mut key, parameter);
        }
        PortableTypeRecipeV1::Applied { origin, arguments } => {
            key.push(2);
            append_recipe_key_part(
                &mut key,
                &portable_type_recipe_key(recipes, kind_recipes, *origin, memo, active, kind_memo, active_kinds, depth + 1)?,
            );
            let arguments = arguments
                .iter()
                .map(|argument| portable_type_recipe_key(recipes, kind_recipes, *argument, memo, active, kind_memo, active_kinds, depth + 1))
                .collect::<ProjectionResult<Vec<_>>>()?;
            append_recipe_key_children(&mut key, arguments);
        }
        PortableTypeRecipeV1::Tuple { elements } => {
            key.push(3);
            let elements = elements
                .iter()
                .map(|element| portable_type_recipe_key(recipes, kind_recipes, *element, memo, active, kind_memo, active_kinds, depth + 1))
                .collect::<ProjectionResult<Vec<_>>>()?;
            append_recipe_key_children(&mut key, elements);
        }
        PortableTypeRecipeV1::Record { labels, values } => {
            key.push(4);
            append_recipe_key_debug(&mut key, labels);
            let values = values
                .iter()
                .map(|value| portable_type_recipe_key(recipes, kind_recipes, *value, memo, active, kind_memo, active_kinds, depth + 1))
                .collect::<ProjectionResult<Vec<_>>>()?;
            append_recipe_key_children(&mut key, values);
        }
        PortableTypeRecipeV1::Pack { positionals, labels, labeled } => {
            key.push(5);
            let positionals = positionals
                .iter()
                .map(|value| portable_type_recipe_key(recipes, kind_recipes, *value, memo, active, kind_memo, active_kinds, depth + 1))
                .collect::<ProjectionResult<Vec<_>>>()?;
            append_recipe_key_children(&mut key, positionals);
            append_recipe_key_debug(&mut key, labels);
            let labeled = labeled
                .iter()
                .map(|value| portable_type_recipe_key(recipes, kind_recipes, *value, memo, active, kind_memo, active_kinds, depth + 1))
                .collect::<ProjectionResult<Vec<_>>>()?;
            append_recipe_key_children(&mut key, labeled);
        }
        PortableTypeRecipeV1::Union { members } => {
            key.push(6);
            let mut members = members
                .iter()
                .map(|member| portable_type_recipe_key(recipes, kind_recipes, *member, memo, active, kind_memo, active_kinds, depth + 1))
                .collect::<ProjectionResult<Vec<_>>>()?;
            members.sort_unstable();
            append_recipe_key_children(&mut key, members);
        }
        PortableTypeRecipeV1::Dynamic => key.push(7),
        PortableTypeRecipeV1::Nominal { class } => {
            key.push(8);
            append_recipe_key_debug(&mut key, class);
        }
        PortableTypeRecipeV1::TupleLabeled { elements } => {
            key.push(9);
            key.extend_from_slice(&(elements.len() as u64).to_be_bytes());
            for element in elements {
                append_recipe_key_debug(&mut key, &element.label);
                append_recipe_key_part(
                    &mut key,
                    &portable_type_recipe_key(recipes, kind_recipes, element.ty, memo, active, kind_memo, active_kinds, depth + 1)?,
                );
            }
        }
        PortableTypeRecipeV1::RecordKeys { fields } => {
            key.push(10);
            key.extend_from_slice(&(fields.len() as u64).to_be_bytes());
            for field in fields {
                append_recipe_key_debug(&mut key, &field.key);
                append_recipe_key_part(
                    &mut key,
                    &portable_type_recipe_key(recipes, kind_recipes, field.ty, memo, active, kind_memo, active_kinds, depth + 1)?,
                );
            }
        }
        PortableTypeRecipeV1::Callable { parameters, return_type } => {
            key.push(11);
            key.extend_from_slice(&(parameters.len() as u64).to_be_bytes());
            for parameter in parameters {
                append_recipe_key_debug(&mut key, &parameter.label);
                append_recipe_key_debug(&mut key, &parameter.rest);
                append_recipe_key_part(
                    &mut key,
                    &portable_type_recipe_key(recipes, kind_recipes, parameter.ty, memo, active, kind_memo, active_kinds, depth + 1)?,
                );
            }
            append_recipe_key_part(
                &mut key,
                &portable_type_recipe_key(recipes, kind_recipes, *return_type, memo, active, kind_memo, active_kinds, depth + 1)?,
            );
        }
        PortableTypeRecipeV1::TypeLambda { parameters, body } => {
            key.push(12);
            key.extend_from_slice(&(parameters.len() as u64).to_be_bytes());
            for parameter in parameters {
                append_recipe_key_debug(&mut key, &parameter.name);
                append_recipe_key_part(
                    &mut key,
                    &portable_kind_recipe_key(kind_recipes, parameter.kind, kind_memo, active_kinds, depth + 1)?,
                );
            }
            append_recipe_key_part(
                &mut key,
                &portable_type_recipe_key(recipes, kind_recipes, *body, memo, active, kind_memo, active_kinds, depth + 1)?,
            );
        }
        PortableTypeRecipeV1::Special { name } => {
            key.push(13);
            append_recipe_key_debug(&mut key, name);
        }
        PortableTypeRecipeV1::SelfType { class } => {
            key.push(14);
            append_recipe_key_debug(&mut key, class);
        }
        PortableTypeRecipeV1::AppliedStatic { origin, arguments } => {
            key.push(15);
            append_recipe_key_part(
                &mut key,
                &portable_type_recipe_key(recipes, kind_recipes, *origin, memo, active, kind_memo, active_kinds, depth + 1)?,
            );
            key.extend_from_slice(&(arguments.len() as u64).to_be_bytes());
            for argument in arguments {
                match argument {
                    PortableStaticTypeArgumentV1::Term { ty } => {
                        key.push(0);
                        append_recipe_key_part(
                            &mut key,
                            &portable_type_recipe_key(recipes, kind_recipes, *ty, memo, active, kind_memo, active_kinds, depth + 1)?,
                        );
                    }
                    PortableStaticTypeArgumentV1::RecordRow { fields, tail } => {
                        key.push(1);
                        key.extend_from_slice(&(fields.len() as u64).to_be_bytes());
                        for field in fields {
                            append_recipe_key_debug(&mut key, &field.key);
                            append_recipe_key_part(
                                &mut key,
                                &portable_type_recipe_key(recipes, kind_recipes, field.ty, memo, active, kind_memo, active_kinds, depth + 1)?,
                            );
                        }
                        append_recipe_key_debug(&mut key, tail);
                    }
                    PortableStaticTypeArgumentV1::DynamicType => key.push(2),
                }
            }
        }
    }
    active.remove(&id.0);
    memo[index] = Some(key.clone());
    Ok(key)
}

fn remap_type_recipe_id(id: &mut PortableTypeRecipeId, remap: &[PortableTypeRecipeId]) -> ProjectionResult<()> {
    *id = *remap.get(id.0 as usize).ok_or_else(|| SemanticProjectionError {
        family: "type_recipes",
        reason: format!("portable type recipe reference {} is out of bounds", id.0),
    })?;
    Ok(())
}

fn rewrite_type_recipe_refs(
    recipe: &mut PortableTypeRecipeV1,
    keys: &[Vec<u8>],
    remap: &[PortableTypeRecipeId],
    kind_remap: &[PortableKindRecipeId],
) -> ProjectionResult<()> {
    match recipe {
        PortableTypeRecipeV1::Canonical { .. }
        | PortableTypeRecipeV1::Parameter { .. }
        | PortableTypeRecipeV1::Dynamic
        | PortableTypeRecipeV1::Nominal { .. }
        | PortableTypeRecipeV1::Special { .. }
        | PortableTypeRecipeV1::SelfType { .. } => {}
        PortableTypeRecipeV1::Applied { origin, arguments } => {
            remap_type_recipe_id(origin, remap)?;
            for argument in arguments {
                remap_type_recipe_id(argument, remap)?;
            }
        }
        PortableTypeRecipeV1::Tuple { elements } => {
            for element in elements {
                remap_type_recipe_id(element, remap)?;
            }
        }
        PortableTypeRecipeV1::Record { values, .. } => {
            for value in values {
                remap_type_recipe_id(value, remap)?;
            }
        }
        PortableTypeRecipeV1::Pack { positionals, labeled, .. } => {
            for value in positionals.iter_mut().chain(labeled.iter_mut()) {
                remap_type_recipe_id(value, remap)?;
            }
        }
        PortableTypeRecipeV1::Union { members } => {
            members.sort_by(|left, right| keys[left.0 as usize].cmp(&keys[right.0 as usize]));
            for member in members {
                remap_type_recipe_id(member, remap)?;
            }
        }
        PortableTypeRecipeV1::TupleLabeled { elements } => {
            for element in elements {
                remap_type_recipe_id(&mut element.ty, remap)?;
            }
        }
        PortableTypeRecipeV1::RecordKeys { fields } => {
            for field in fields {
                remap_type_recipe_id(&mut field.ty, remap)?;
            }
        }
        PortableTypeRecipeV1::Callable { parameters, return_type } => {
            for parameter in parameters {
                remap_type_recipe_id(&mut parameter.ty, remap)?;
            }
            remap_type_recipe_id(return_type, remap)?;
        }
        PortableTypeRecipeV1::TypeLambda { parameters, body } => {
            for parameter in parameters {
                remap_kind_recipe_id(&mut parameter.kind, kind_remap)?;
            }
            remap_type_recipe_id(body, remap)?;
        }
        PortableTypeRecipeV1::AppliedStatic { origin, arguments } => {
            remap_type_recipe_id(origin, remap)?;
            for argument in arguments {
                match argument {
                    PortableStaticTypeArgumentV1::Term { ty } => remap_type_recipe_id(ty, remap)?,
                    PortableStaticTypeArgumentV1::RecordRow { fields, .. } => {
                        for field in fields {
                            remap_type_recipe_id(&mut field.ty, remap)?;
                        }
                    }
                    PortableStaticTypeArgumentV1::DynamicType => {}
                }
            }
        }
    }
    Ok(())
}

fn remap_kind_recipe_id(id: &mut PortableKindRecipeId, remap: &[PortableKindRecipeId]) -> ProjectionResult<()> {
    *id = *remap.get(id.0 as usize).ok_or_else(|| SemanticProjectionError {
        family: "kind_recipes",
        reason: format!("portable kind recipe reference {} is out of bounds", id.0),
    })?;
    Ok(())
}

fn rewrite_kind_recipe_refs(recipe: &mut PortableKindRecipeV1, remap: &[PortableKindRecipeId]) -> ProjectionResult<()> {
    if let PortableKindRecipeV1::Arrow { parameters, result } = recipe {
        for parameter in parameters {
            remap_kind_recipe_id(parameter, remap)?;
        }
        remap_kind_recipe_id(result, remap)?;
    }
    Ok(())
}

fn remap_runtime_type_recipe(recipe: &mut PortableRuntimeTypeRecipeV1, remap: &[PortableTypeRecipeId]) -> ProjectionResult<()> {
    match recipe {
        PortableRuntimeTypeRecipeV1::Closed { ty } | PortableRuntimeTypeRecipeV1::Template { ty } => remap_type_recipe_id(ty, remap),
    }
}

fn remap_type_binding(binding: &mut PortableTypeBindingV1, remap: &[PortableTypeRecipeId]) -> ProjectionResult<()> {
    remap_runtime_type_recipe(&mut binding.recipe, remap)
}

fn remap_conformance_slot(slot: &mut PortableConformanceSlotV1, remap: &[PortableTypeRecipeId]) -> ProjectionResult<()> {
    match slot {
        PortableConformanceSlotV1::Selection { nested, .. } => {
            for entry in nested {
                remap_conformance_slot(&mut entry.recipe, remap)?;
            }
        }
        PortableConformanceSlotV1::Forwarded { .. } => {}
        PortableConformanceSlotV1::DerivedConditional { bindings, nested, .. } => {
            for binding in bindings {
                remap_type_binding(binding, remap)?;
            }
            for entry in nested {
                remap_conformance_slot(&mut entry.recipe, remap)?;
            }
        }
    }
    Ok(())
}

fn remap_trait_selection_target(target: &mut PortableTraitSelectionTargetV1, remap: &[PortableTypeRecipeId]) -> ProjectionResult<()> {
    if let PortableTraitSelectionTargetV1::DataComponent { specialized_type, .. } = target {
        remap_type_recipe_id(specialized_type, remap)?;
    }
    Ok(())
}

fn canonicalize_type_recipe_ids(pool: &mut PortableSemanticPoolV1) -> ProjectionResult<()> {
    let original_kinds = pool.kind_recipes.clone();
    let kind_remap = canonicalize_kind_recipe_ids(&mut pool.kind_recipes)?;
    let original = pool.type_recipes.clone();
    let mut type_memo = vec![None; original.len()];
    let mut kind_memo = vec![None; original_kinds.len()];
    for index in 0..original.len() {
        portable_type_recipe_key(
            &original,
            &original_kinds,
            PortableTypeRecipeId(u32::try_from(index).map_err(|_| SemanticProjectionError {
                family: "type_recipes",
                reason: "portable type recipe count exceeds u32".into(),
            })?),
            &mut type_memo,
            &mut std::collections::BTreeSet::new(),
            &mut kind_memo,
            &mut std::collections::BTreeSet::new(),
            0,
        )?;
    }
    let keys = type_memo
        .into_iter()
        .map(|key| {
            key.ok_or_else(|| SemanticProjectionError {
                family: "type_recipes",
                reason: "portable type recipe key was not computed".into(),
            })
        })
        .collect::<ProjectionResult<Vec<_>>>()?;

    let mut representative_by_key = std::collections::BTreeMap::<Vec<u8>, usize>::new();
    for (index, key) in keys.iter().enumerate() {
        representative_by_key.entry(key.clone()).or_insert(index);
    }
    let mut remap = vec![PortableTypeRecipeId(0); original.len()];
    let mut key_to_id = std::collections::BTreeMap::<Vec<u8>, PortableTypeRecipeId>::new();
    for (index, key) in representative_by_key.keys().enumerate() {
        let id = PortableTypeRecipeId(u32::try_from(index).map_err(|_| SemanticProjectionError {
            family: "type_recipes",
            reason: "canonical portable type recipe count exceeds u32".into(),
        })?);
        key_to_id.insert(key.clone(), id);
    }
    for (index, key) in keys.iter().enumerate() {
        remap[index] = *key_to_id.get(key).ok_or_else(|| SemanticProjectionError {
            family: "type_recipes",
            reason: "canonical portable type recipe key has no assigned ID".into(),
        })?;
    }
    // Rebuild canonical rows now that every old coordinate has a remap.
    let mut canonical_recipes = Vec::with_capacity(representative_by_key.len());
    for representative in representative_by_key.values() {
        let mut recipe = original[*representative].clone();
        rewrite_type_recipe_refs(&mut recipe, &keys, &remap, &kind_remap)?;
        canonical_recipes.push(recipe);
    }
    pool.type_recipes = canonical_recipes;

    for construction in &mut pool.data_constructions {
        remap_runtime_type_recipe(&mut construction.type_recipe, &remap)?;
    }
    for product in &mut pool.anonymous_products {
        remap_runtime_type_recipe(&mut product.type_recipe, &remap)?;
    }
    for environment in &mut pool.call_environments {
        for binding in &mut environment.bindings {
            remap_type_binding(binding, &remap)?;
        }
        for slot in &mut environment.conformance_slots {
            remap_conformance_slot(slot, &remap)?;
        }
    }
    for invocation in &mut pool.trait_invocations {
        remap_trait_selection_target(&mut invocation.selection, &remap)?;
        for target in &mut invocation.requirement_targets {
            remap_trait_selection_target(&mut target.target, &remap)?;
        }
        for antecedent in &mut invocation.antecedent_slots {
            remap_trait_selection_target(&mut antecedent.target, &remap)?;
        }
        for binding in &mut invocation.type_bindings {
            remap_type_binding(binding, &remap)?;
        }
    }
    Ok(())
}

fn canonicalize_kind_recipe_ids(recipes: &mut Vec<PortableKindRecipeV1>) -> ProjectionResult<Vec<PortableKindRecipeId>> {
    let mut memo = vec![None; recipes.len()];
    for index in 0..recipes.len() {
        portable_kind_recipe_key(
            recipes,
            PortableKindRecipeId(u32::try_from(index).map_err(|_| SemanticProjectionError {
                family: "kind_recipes",
                reason: "portable kind recipe count exceeds u32".into(),
            })?),
            &mut memo,
            &mut std::collections::BTreeSet::new(),
            0,
        )?;
    }
    let keys = memo
        .into_iter()
        .map(|key| {
            key.ok_or_else(|| SemanticProjectionError {
                family: "kind_recipes",
                reason: "portable kind recipe key was not computed".into(),
            })
        })
        .collect::<ProjectionResult<Vec<_>>>()?;
    let mut representative_by_key = std::collections::BTreeMap::<Vec<u8>, usize>::new();
    for (index, key) in keys.iter().enumerate() {
        representative_by_key.entry(key.clone()).or_insert(index);
    }
    let mut key_to_id = std::collections::BTreeMap::<Vec<u8>, PortableKindRecipeId>::new();
    for (index, key) in representative_by_key.keys().enumerate() {
        key_to_id.insert(
            key.clone(),
            PortableKindRecipeId(u32::try_from(index).map_err(|_| SemanticProjectionError {
                family: "kind_recipes",
                reason: "canonical portable kind recipe count exceeds u32".into(),
            })?),
        );
    }
    let remap = keys
        .iter()
        .map(|key| {
            key_to_id.get(key).copied().ok_or_else(|| SemanticProjectionError {
                family: "kind_recipes",
                reason: "canonical portable kind recipe key has no assigned ID".into(),
            })
        })
        .collect::<ProjectionResult<Vec<_>>>()?;
    let mut canonical = Vec::with_capacity(representative_by_key.len());
    for representative in representative_by_key.values() {
        let mut recipe = recipes[*representative].clone();
        rewrite_kind_recipe_refs(&mut recipe, &remap)?;
        canonical.push(recipe);
    }
    *recipes = canonical;
    Ok(remap)
}

impl SemanticPoolProjector<'_> {
    fn enum_spec(&self, spec: &crate::modules::semantic_lowering::EnumLoweringSpec) -> ProjectionResult<PortableEnumSpecV1> {
        let mut variants = Vec::with_capacity(spec.variants.len());
        for variant in spec.variants.iter() {
            let shape = match variant.shape {
                phalcom_semantic::enum_semantics::VariantShape::Singleton => PortableVariantShapeV1::Singleton,
                phalcom_semantic::enum_semantics::VariantShape::Constructor => PortableVariantShapeV1::Positional {
                    arity: u32::try_from(variant.payload_fields.len()).map_err(|_| SemanticProjectionError {
                        family: "enum_specs",
                        reason: "variant payload count exceeds u32".into(),
                    })?,
                },
            };
            let fields = variant
                .payload_fields
                .iter()
                .map(|field| PortableVariantFieldV1 {
                    index: field.id.index,
                    local_name: field.local_name.to_string(),
                    slot: field.slot,
                })
                .collect();
            variants.push(PortableVariantSpecV1 {
                variant: to_stable_variant(&variant.id),
                shape,
                fields,
                layout: variant.layout.as_ref().map(portable_layout),
            });
        }
        Ok(PortableEnumSpecV1 {
            owner: to_stable_declaration(&spec.owner),
            representation: match spec.representation {
                crate::adt::RuntimeAdtRepresentation::NativeOption => PortableAdtRepresentationV1::OptionImmediate,
                crate::adt::RuntimeAdtRepresentation::General => PortableAdtRepresentationV1::TaggedHeap,
            },
            variants,
        })
    }

    fn data_spec(&self, spec: &crate::modules::semantic_lowering::DataDeclarationLoweringSpec) -> ProjectionResult<PortableDataSpecV1> {
        Ok(PortableDataSpecV1 {
            owner: to_stable_declaration(&spec.owner),
            layout: spec.layout.as_ref().map(portable_layout),
            components: spec
                .components
                .iter()
                .map(|component| PortableDataComponentV1 {
                    index: component.id.index,
                    local_name: component.local_name.to_string(),
                    logical_index: component.logical_index,
                })
                .collect(),
        })
    }

    fn ensure_data_construction(&mut self, spec: &Arc<DataConstructionLoweringSpec>) -> ProjectionResult<u32> {
        let index = if let Some(index) = self.construction_specs.iter().position(|existing| existing.as_ref() == spec.as_ref()) {
            index
        } else {
            self.construction_specs.push(spec.clone());
            self.construction_specs.len() - 1
        };
        while self.output.data_constructions.len() <= index {
            let row_index = self.output.data_constructions.len();
            let row_spec = self.construction_specs[row_index].clone();
            let row = self.data_construction(&row_spec)?;
            self.output.data_constructions.push(row);
        }
        u32::try_from(index).map_err(|_| SemanticProjectionError {
            family: "data_constructions",
            reason: "data construction index exceeds u32".into(),
        })
    }

    fn data_construction(&mut self, spec: &DataConstructionLoweringSpec) -> ProjectionResult<PortableDataConstructionV1> {
        let root = spec.exact_type.runtime_roots.first().ok_or_else(|| SemanticProjectionError {
            family: "data_constructions",
            reason: "exact type metadata has no runtime root".into(),
        })?;
        let entry = spec.exact_type.types.get(root.form.0 as usize).ok_or_else(|| SemanticProjectionError {
            family: "data_constructions",
            reason: "exact type metadata runtime root is outside its type table".into(),
        })?;
        let (template, recipe) = data_construction_type_recipe(spec.type_recipe, root.form, entry)?;
        let recipe_id = self.intern_type_recipe(recipe)?;
        let type_recipe = if template {
            PortableRuntimeTypeRecipeV1::Template { ty: recipe_id }
        } else {
            PortableRuntimeTypeRecipeV1::Closed { ty: recipe_id }
        };
        Ok(PortableDataConstructionV1 {
            constructor: to_stable_declaration(&spec.constructor.owner),
            exact_type_digest: exact_metadata_digest(&spec.exact_type)?,
            type_recipe,
            layout: portable_layout(&spec.layout),
            argument_to_component: spec.argument_to_component.to_vec(),
            component_to_logical: spec.component_to_logical.to_vec(),
        })
    }

    fn family_target(&mut self, target: &ExecutableFamilyTarget) -> ProjectionResult<PortableFamilyTargetV1> {
        Ok(match target {
            ExecutableFamilyTarget::Singleton { variant } => PortableFamilyTargetV1::Singleton {
                variant: to_stable_variant(variant),
            },
            ExecutableFamilyTarget::Behavioral { target } => PortableFamilyTargetV1::Behavioral {
                target: self.invocation_target(target)?,
            },
            ExecutableFamilyTarget::VariantConstructor { variant } => PortableFamilyTargetV1::VariantConstructor {
                variant: to_stable_variant(variant),
            },
            ExecutableFamilyTarget::DataConstructor { constructor, construction } => PortableFamilyTargetV1::DataConstructor {
                constructor: to_stable_declaration(&constructor.owner),
                construction: self.ensure_data_construction(construction)?,
            },
        })
    }

    fn invocation_target(&mut self, target: &ExecutableInvocationTarget) -> ProjectionResult<PortableInvocationTargetV1> {
        Ok(match target {
            ExecutableInvocationTarget::Behavioral {
                lookup_owner,
                callable,
                operation,
                rest_mode,
            } => PortableInvocationTargetV1::Behavioral {
                lookup_owner: to_stable_declaration(lookup_owner),
                callable: to_stable_callable(callable),
                operation: portable_operation(operation),
                rest_mode: portable_rest_mode(*rest_mode),
            },
            ExecutableInvocationTarget::VariantConstructor { variant } => PortableInvocationTargetV1::VariantConstructor {
                variant: to_stable_variant(variant),
            },
            ExecutableInvocationTarget::DataConstructor { constructor, construction } => PortableInvocationTargetV1::DataConstructor {
                constructor: to_stable_declaration(&constructor.owner),
                construction: self.ensure_data_construction(construction)?,
            },
        })
    }

    fn ensure_call_environment(&mut self, recipe: &Arc<RuntimeCallEnvironmentRecipe>) -> ProjectionResult<PortableCallEnvironmentId> {
        let candidate = self.call_environment(recipe)?;
        if let Some(index) = self.output.call_environments.iter().position(|existing| existing == &candidate) {
            return Ok(PortableCallEnvironmentId(u32::try_from(index).map_err(|_| SemanticProjectionError {
                family: "call_environment_recipes",
                reason: "call environment index exceeds u32".into(),
            })?));
        }
        let id = PortableCallEnvironmentId(u32::try_from(self.output.call_environments.len()).map_err(|_| SemanticProjectionError {
            family: "call_environment_recipes",
            reason: "call environment index exceeds u32".into(),
        })?);
        self.output.call_environments.push(candidate);
        Ok(id)
    }

    fn call_environment(&mut self, recipe: &RuntimeCallEnvironmentRecipe) -> ProjectionResult<PortableCallEnvironmentV1> {
        let mut source_bindings = recipe.bindings.iter().collect::<Vec<_>>();
        source_bindings.sort_by(|(left, _), (right, _)| left.cmp(right));
        let bindings = source_bindings
            .into_iter()
            .map(|(parameter, ty)| {
                Ok(PortableTypeBindingV1 {
                    parameter: portable_type_parameter(parameter),
                    recipe: self.runtime_type_recipe(*ty)?,
                })
            })
            .collect::<ProjectionResult<Vec<_>>>()?;
        let conformance_slots = recipe
            .conformance_slots
            .iter()
            .map(|slot| self.conformance_slot(slot))
            .collect::<ProjectionResult<Vec<_>>>()?;
        Ok(PortableCallEnvironmentV1 {
            callable: recipe.callable.clone(),
            bindings,
            conformance_slots,
        })
    }

    fn conformance_slot(&mut self, recipe: &RuntimeConformanceSlotRecipe) -> ProjectionResult<PortableConformanceSlotV1> {
        Ok(match recipe {
            RuntimeConformanceSlotRecipe::Selection {
                callable,
                selector,
                nested_slots,
            } => PortableConformanceSlotV1::Selection {
                callable: to_stable_callable(callable),
                selector: exact_selector_pattern(selector),
                nested: self.conformance_entries(nested_slots)?,
            },
            RuntimeConformanceSlotRecipe::Forwarded(slot) => PortableConformanceSlotV1::Forwarded { slot: *slot },
            RuntimeConformanceSlotRecipe::DerivedConditional {
                callable,
                selector,
                type_bindings,
                nested_slots,
            } => {
                let bindings = type_bindings
                    .iter()
                    .map(|(parameter, ty)| {
                        Ok(PortableTypeBindingV1 {
                            parameter: portable_type_parameter(parameter),
                            recipe: self.runtime_type_recipe(*ty)?,
                        })
                    })
                    .collect::<ProjectionResult<Vec<_>>>()?;
                PortableConformanceSlotV1::DerivedConditional {
                    callable: to_stable_callable(callable),
                    selector: exact_selector_pattern(selector),
                    bindings,
                    nested: self.conformance_entries(nested_slots)?,
                }
            }
        })
    }

    fn conformance_entries(&mut self, slots: &[(u16, RuntimeConformanceSlotRecipe)]) -> ProjectionResult<Vec<PortableConformanceSlotEntryV1>> {
        let mut entries = slots
            .iter()
            .map(|(slot, recipe)| {
                Ok(PortableConformanceSlotEntryV1 {
                    slot: *slot,
                    recipe: self.conformance_slot(recipe)?,
                })
            })
            .collect::<ProjectionResult<Vec<_>>>()?;
        entries.sort_by_key(|entry| entry.slot);
        Ok(entries)
    }

    fn trait_invocation(&mut self, spec: &crate::modules::semantic_lowering::TraitInvocationSpec) -> ProjectionResult<PortableTraitInvocationV1> {
        let selection = self.trait_selection_target(&spec.selection.selection)?;
        let mut requirement_targets = spec
            .selection
            .requirement_targets
            .iter()
            .map(|(requirement, target)| {
                Ok(PortableTraitRequirementTargetV1 {
                    requirement: portable_trait_requirement(requirement),
                    target: self.trait_selection_target(target)?,
                })
            })
            .collect::<ProjectionResult<Vec<_>>>()?;
        requirement_targets.sort_by(|left, right| trait_requirement_order(&left.requirement, &right.requirement));
        let antecedent_slots = spec
            .antecedent_slots
            .iter()
            .map(|(slot, target, selector)| {
                Ok(PortableTraitAntecedentSlotV1 {
                    slot: *slot,
                    target: self.trait_selection_target(target)?,
                    selector: selector.encode(),
                })
            })
            .collect::<ProjectionResult<Vec<_>>>()?;
        let type_bindings = spec
            .type_bindings
            .iter()
            .map(|(parameter, ty)| {
                Ok(PortableTypeBindingV1 {
                    parameter: portable_type_parameter(parameter),
                    recipe: self.runtime_type_recipe(*ty)?,
                })
            })
            .collect::<ProjectionResult<Vec<_>>>()?;
        Ok(PortableTraitInvocationV1 {
            source_impl: to_stable_impl(&spec.selection.source_impl),
            selection,
            selection_is_trait_default: matches!(
                spec.selection.selection,
                phalcom_semantic::impls::RequirementSelectionTemplate::TraitDefault { .. }
            ),
            requirement: portable_trait_requirement(&spec.selection.requirement),
            requirement_slot: spec.requirement_slot,
            requirement_slots: spec
                .requirement_slots
                .iter()
                .map(|(requirement, slot)| PortableTraitRequirementSlotV1 {
                    requirement: portable_trait_requirement(requirement),
                    slot: *slot,
                })
                .collect(),
            requirement_targets,
            antecedent_slots,
            type_bindings,
            call_environment: spec
                .call_environment_recipe
                .as_ref()
                .map(|recipe| self.ensure_call_environment(recipe))
                .transpose()?,
        })
    }

    fn trait_selection_target(&mut self, target: &RequirementSelectionTemplate) -> ProjectionResult<PortableTraitSelectionTargetV1> {
        Ok(match target {
            RequirementSelectionTemplate::ConformanceCallable { callable }
            | RequirementSelectionTemplate::InherentCallable { callable, .. }
            | RequirementSelectionTemplate::TraitDefault { callable } => PortableTraitSelectionTargetV1::Callable {
                callable: to_stable_callable(callable),
            },
            RequirementSelectionTemplate::ConditionalInherent { candidate, .. } => PortableTraitSelectionTargetV1::Callable {
                callable: to_stable_callable(&candidate.callable),
            },
            RequirementSelectionTemplate::DataComponent { component, specialized_type } => PortableTraitSelectionTargetV1::DataComponent {
                component: PortableDataComponentRefV1 {
                    owner: to_stable_declaration(&component.owner),
                    index: component.index,
                },
                specialized_type: self.semantic_type_id(*specialized_type)?,
            },
        })
    }

    fn runtime_type_recipe(&mut self, recipe: RuntimeTypeRecipe) -> ProjectionResult<PortableRuntimeTypeRecipeV1> {
        Ok(match recipe {
            RuntimeTypeRecipe::Closed(ty) => PortableRuntimeTypeRecipeV1::Closed { ty: self.type_recipe(ty, 0)? },
            RuntimeTypeRecipe::Template(ty) => PortableRuntimeTypeRecipeV1::Template { ty: self.type_recipe(ty, 0)? },
        })
    }

    fn semantic_type_id(&mut self, ty: phalcom_semantic::types::id::TypeId) -> ProjectionResult<PortableTypeRecipeId> {
        let mut exporter = phalcom_semantic::metadata::export::MetadataExporter::new(
            &self.snapshot.store,
            None,
            None,
            None,
            phalcom_type_meta::header::MetadataProfile::RuntimePublic,
        );
        let fingerprint = exporter.type_fingerprint(ty).map_err(|error| SemanticProjectionError {
            family: "trait_invocations",
            reason: format!("canonical semantic type identity is unavailable: {error}"),
        })?;
        self.intern_type_recipe(PortableTypeRecipeV1::Canonical {
            digest: stable_type_digest(fingerprint),
        })
    }

    fn type_recipe(&mut self, ty: RuntimeTypeRef, depth: usize) -> ProjectionResult<PortableTypeRecipeId> {
        if depth > 128 {
            return fail("type_recipes", "runtime type recipe exceeds maximum structural depth");
        }
        if !self.active_types.insert(ty) {
            return fail("type_recipes", "runtime type recipe contains a cycle");
        }
        let result = match ty {
            RuntimeTypeRef::Base { pool, node } => {
                let fingerprint = self
                    .vm
                    .typing_registry
                    .get_pool(pool)
                    .and_then(|loaded| loaded.bundle.types.get(node.0 as usize))
                    .map(|entry| entry.structural_fingerprint)
                    .ok_or_else(|| SemanticProjectionError {
                        family: "type_recipes",
                        reason: format!("canonical runtime type reference pool={} node={} is unresolved", pool.0, node.0),
                    })?;
                self.intern_type_recipe(PortableTypeRecipeV1::Canonical {
                    digest: stable_type_digest(fingerprint),
                })
            }
            RuntimeTypeRef::Overlay(id) => {
                let node = self
                    .vm
                    .runtime_typing_context
                    .overlay
                    .type_node(id)
                    .cloned()
                    .ok_or_else(|| SemanticProjectionError {
                        family: "type_recipes",
                        reason: format!("runtime overlay type {} is unresolved", id.0),
                    })?;
                let recipe = match node {
                    crate::typing::overlay::RuntimeOverlayTypeNode::Nominal { class } => PortableTypeRecipeV1::Nominal {
                        class: stable_class(self.vm, class)?,
                    },
                    crate::typing::overlay::RuntimeOverlayTypeNode::Applied { origin, arguments } => {
                        let mut portable_args = Vec::with_capacity(arguments.len());
                        let mut all_terms = true;
                        for argument in arguments.iter() {
                            match argument {
                                crate::typing::overlay::RuntimeStaticArgument::Term(ty) => {
                                    portable_args.push(PortableStaticTypeArgumentV1::Term {
                                        ty: self.type_recipe(*ty, depth + 1)?,
                                    });
                                }
                                crate::typing::overlay::RuntimeStaticArgument::RecordRow(row) => {
                                    all_terms = false;
                                    let fields = row
                                        .fields
                                        .iter()
                                        .map(|field| {
                                            Ok(PortableRecordTypeFieldV1 {
                                                key: portable_field_key(&field.key),
                                                ty: self.type_recipe(field.ty, depth + 1)?,
                                            })
                                        })
                                        .collect::<ProjectionResult<Vec<_>>>()?;
                                    portable_args.push(PortableStaticTypeArgumentV1::RecordRow {
                                        fields,
                                        tail: row.tail.as_ref().map(portable_type_parameter),
                                    });
                                }
                                crate::typing::overlay::RuntimeStaticArgument::DynamicType => {
                                    all_terms = false;
                                    portable_args.push(PortableStaticTypeArgumentV1::DynamicType);
                                }
                            }
                        }
                        let origin = self.type_recipe(origin, depth + 1)?;
                        if all_terms {
                            PortableTypeRecipeV1::Applied {
                                origin,
                                arguments: portable_args
                                    .into_iter()
                                    .map(|argument| match argument {
                                        PortableStaticTypeArgumentV1::Term { ty } => ty,
                                        _ => unreachable!("all_terms excludes heterogeneous type arguments"),
                                    })
                                    .collect(),
                            }
                        } else {
                            PortableTypeRecipeV1::AppliedStatic {
                                origin,
                                arguments: portable_args,
                            }
                        }
                    }
                    crate::typing::overlay::RuntimeOverlayTypeNode::Union(members) => PortableTypeRecipeV1::Union {
                        members: members
                            .iter()
                            .map(|member| self.type_recipe(*member, depth + 1))
                            .collect::<ProjectionResult<Vec<_>>>()?,
                    },
                    crate::typing::overlay::RuntimeOverlayTypeNode::Tuple(elements) => PortableTypeRecipeV1::TupleLabeled {
                        elements: elements
                            .iter()
                            .map(|element| {
                                Ok(PortableTupleTypeElementV1 {
                                    label: element.label.as_ref().map(ToString::to_string),
                                    ty: self.type_recipe(element.ty, depth + 1)?,
                                })
                            })
                            .collect::<ProjectionResult<Vec<_>>>()?,
                    },
                    crate::typing::overlay::RuntimeOverlayTypeNode::Record(fields) => PortableTypeRecipeV1::RecordKeys {
                        fields: fields
                            .iter()
                            .map(|field| {
                                Ok(PortableRecordTypeFieldV1 {
                                    key: portable_field_key(&field.key),
                                    ty: self.type_recipe(field.ty, depth + 1)?,
                                })
                            })
                            .collect::<ProjectionResult<Vec<_>>>()?,
                    },
                    crate::typing::overlay::RuntimeOverlayTypeNode::Callable { parameters, return_type } => PortableTypeRecipeV1::Callable {
                        parameters: parameters
                            .iter()
                            .map(|parameter| {
                                Ok(PortableCallableTypeParameterV1 {
                                    label: parameter.label.as_ref().map(ToString::to_string),
                                    ty: self.type_recipe(parameter.ty, depth + 1)?,
                                    rest: parameter.rest,
                                })
                            })
                            .collect::<ProjectionResult<Vec<_>>>()?,
                        return_type: self.type_recipe(return_type, depth + 1)?,
                    },
                    crate::typing::overlay::RuntimeOverlayTypeNode::TypeLambda { parameters, body } => {
                        let mut portable_parameters = Vec::with_capacity(parameters.len());
                        for parameter in parameters.iter() {
                            portable_parameters.push(PortableTypeLambdaParameterV1 {
                                name: parameter.name.to_string(),
                                kind: self.kind_recipe(parameter.kind, depth + 1)?,
                            });
                        }
                        PortableTypeRecipeV1::TypeLambda {
                            parameters: portable_parameters,
                            body: self.type_recipe(body, depth + 1)?,
                        }
                    }
                    crate::typing::overlay::RuntimeOverlayTypeNode::Special(name) => PortableTypeRecipeV1::Special { name: name.to_string() },
                    crate::typing::overlay::RuntimeOverlayTypeNode::SelfType(class) => PortableTypeRecipeV1::SelfType {
                        class: class.map(|class| stable_class(self.vm, class)).transpose()?,
                    },
                };
                self.intern_type_recipe(recipe)
            }
        };
        self.active_types.remove(&ty);
        result
    }

    fn kind_recipe(&mut self, kind: crate::typing::handle::RuntimeKindRef, depth: usize) -> ProjectionResult<PortableKindRecipeId> {
        if depth > 128 {
            return fail("kind_recipes", "runtime kind recipe exceeds maximum structural depth");
        }
        if !self.active_kinds.insert(kind) {
            return fail("kind_recipes", "runtime kind recipe contains a cycle");
        }
        let recipe = match kind {
            crate::typing::handle::RuntimeKindRef::Base { pool, node } => {
                let fingerprint = self
                    .vm
                    .typing_registry
                    .get_pool(pool)
                    .and_then(|loaded| loaded.bundle.kinds.get(node.0 as usize))
                    .map(|entry| entry.structural_fingerprint)
                    .ok_or_else(|| SemanticProjectionError {
                        family: "kind_recipes",
                        reason: format!("canonical runtime kind reference pool={} node={} is unresolved", pool.0, node.0),
                    })?;
                PortableKindRecipeV1::Canonical {
                    digest: stable_type_digest(fingerprint),
                }
            }
            crate::typing::handle::RuntimeKindRef::Overlay(id) => {
                match self
                    .vm
                    .runtime_typing_context
                    .overlay
                    .kind_node(id)
                    .cloned()
                    .ok_or_else(|| SemanticProjectionError {
                        family: "kind_recipes",
                        reason: format!("runtime overlay kind {} is unresolved", id.0),
                    })? {
                    crate::typing::overlay::RuntimeOverlayKindNode::Type => PortableKindRecipeV1::Type,
                    crate::typing::overlay::RuntimeOverlayKindNode::Arrow { parameters, result } => PortableKindRecipeV1::Arrow {
                        parameters: parameters
                            .iter()
                            .map(|parameter| self.kind_recipe(*parameter, depth + 1))
                            .collect::<ProjectionResult<Vec<_>>>()?,
                        result: self.kind_recipe(*result, depth + 1)?,
                    },
                }
            }
        };
        self.active_kinds.remove(&kind);
        self.intern_kind_recipe(recipe)
    }

    fn intern_type_recipe(&mut self, recipe: PortableTypeRecipeV1) -> ProjectionResult<PortableTypeRecipeId> {
        if let Some(index) = self.output.type_recipes.iter().position(|existing| existing == &recipe) {
            return Ok(PortableTypeRecipeId(u32::try_from(index).map_err(|_| SemanticProjectionError {
                family: "type_recipes",
                reason: "type recipe index exceeds u32".into(),
            })?));
        }
        let id = PortableTypeRecipeId(u32::try_from(self.output.type_recipes.len()).map_err(|_| SemanticProjectionError {
            family: "type_recipes",
            reason: "type recipe index exceeds u32".into(),
        })?);
        self.output.type_recipes.push(recipe);
        Ok(id)
    }

    fn intern_kind_recipe(&mut self, recipe: PortableKindRecipeV1) -> ProjectionResult<PortableKindRecipeId> {
        if let Some(index) = self.output.kind_recipes.iter().position(|existing| existing == &recipe) {
            return Ok(PortableKindRecipeId(u32::try_from(index).map_err(|_| SemanticProjectionError {
                family: "kind_recipes",
                reason: "kind recipe index exceeds u32".into(),
            })?));
        }
        let id = PortableKindRecipeId(u32::try_from(self.output.kind_recipes.len()).map_err(|_| SemanticProjectionError {
            family: "kind_recipes",
            reason: "kind recipe index exceeds u32".into(),
        })?);
        self.output.kind_recipes.push(recipe);
        Ok(id)
    }
}

fn empty_pool() -> PortableSemanticPoolV1 {
    PortableSemanticPoolV1 {
        type_recipes: Vec::new(),
        kind_recipes: Vec::new(),
        enum_specs: Vec::new(),
        variant_targets: Vec::new(),
        data_specs: Vec::new(),
        data_targets: Vec::new(),
        data_constructions: Vec::new(),
        anonymous_products: Vec::new(),
        call_environments: Vec::new(),
        associated_targets: Vec::new(),
        family_descriptors: Vec::new(),
        conditional_families: Vec::new(),
        family_operations: Vec::new(),
        family_candidate_sets: Vec::new(),
        trait_invocations: Vec::new(),
        trait_requirement_invocations: Vec::new(),
        type_parameter_class_receivers: Vec::new(),
        provenance: None,
    }
}

#[cfg(test)]
mod type_recipe_canonicalization_tests {
    use super::*;

    #[test]
    fn projection_preserves_equal_call_environment_slots_addressed_by_bytecode() {
        use phalcom_semantic::identity::{SemanticRevision, WorkspaceId};
        use phalcom_type_meta::identity::{StableCallableOwnerRef, StableCallableRef, StableDeclarationRef, StableDispatchSide};
        let snapshot = SemanticSnapshot::new(
            WorkspaceId(0),
            SemanticRevision(0),
            0,
            Default::default(),
            Default::default(),
            Default::default(),
            Default::default(),
            Default::default(),
            Default::default(),
            Default::default(),
            Default::default(),
            Default::default(),
        );
        let vm = crate::vm::VM::new_kernel();
        let recipe = Arc::new(RuntimeCallEnvironmentRecipe {
            callable: StableCallableRef {
                owner: StableCallableOwnerRef::Declaration(StableDeclarationRef {
                    module: phalcom_semantic::metadata::stable_identity::to_stable_module(&phalcom_modules::ModuleId::universe_root()),
                    path: vec!["Probe".into()].into(),
                }),
                side: StableDispatchSide::Instance,
                selector: "probe".into(),
            },
            bindings: Box::new([]),
            conformance_slots: Box::new([]),
        });
        let mut source = ExecutableSemanticPool::new();
        let span = phalcom_common::range::SourceRange::default();
        let first = source.add_call_environment_recipe(recipe.clone(), span).unwrap();
        let second = source.add_call_environment_recipe(recipe, span).unwrap();
        assert_ne!(first, second);
        let projected = project_semantic_pool(&vm, &snapshot, &source, &[]).unwrap();
        assert_eq!(projected.call_environments.len(), 2);
        assert_eq!(projected.call_environments[first as usize], projected.call_environments[second as usize]);
    }

    fn canonical(digest: u8) -> PortableTypeRecipeV1 {
        PortableTypeRecipeV1::Canonical { digest: [digest; 32] }
    }

    fn tuple_product(type_recipe: u32) -> PortableAnonymousProductV1 {
        PortableAnonymousProductV1 {
            kind: PortableAnonymousProductKindV1::Tuple {
                positional_len: 0,
                labels: Vec::new(),
            },
            layout: PortableProductLayoutV1 { components: Vec::new() },
            type_recipe: PortableRuntimeTypeRecipeV1::Template {
                ty: PortableTypeRecipeId(type_recipe),
            },
        }
    }

    #[test]
    fn recipe_ids_and_global_union_members_ignore_discovery_order() {
        let mut first = empty_pool();
        first.type_recipes = vec![
            canonical(2),
            canonical(1),
            PortableTypeRecipeV1::Union {
                members: vec![PortableTypeRecipeId(0), PortableTypeRecipeId(1), PortableTypeRecipeId(0)],
            },
            PortableTypeRecipeV1::Tuple {
                elements: vec![PortableTypeRecipeId(2), PortableTypeRecipeId(1)],
            },
        ];
        first.anonymous_products.push(tuple_product(3));

        let mut second = empty_pool();
        second.type_recipes = vec![
            canonical(1),
            canonical(2),
            PortableTypeRecipeV1::Tuple {
                elements: vec![PortableTypeRecipeId(3), PortableTypeRecipeId(0)],
            },
            PortableTypeRecipeV1::Union {
                members: vec![PortableTypeRecipeId(1), PortableTypeRecipeId(0), PortableTypeRecipeId(1)],
            },
        ];
        second.anonymous_products.push(tuple_product(2));

        canonicalize_type_recipe_ids(&mut first).expect("first recipe graph canonicalizes");
        canonicalize_type_recipe_ids(&mut second).expect("second recipe graph canonicalizes");

        assert_eq!(first.type_recipes, second.type_recipes);
        assert_eq!(first.anonymous_products, second.anonymous_products);
        let union = first
            .type_recipes
            .iter()
            .find_map(|recipe| match recipe {
                PortableTypeRecipeV1::Union { members } => Some(members),
                _ => None,
            })
            .expect("union recipe remains present");
        assert_eq!(union.len(), 3, "canonicalization preserves union cardinality");
        assert_eq!(union[1], union[2], "canonicalization preserves repeated contributions");
    }

    #[test]
    fn ordered_applied_arguments_remain_ordered() {
        let mut first = empty_pool();
        first.type_recipes = vec![
            canonical(1),
            canonical(2),
            PortableTypeRecipeV1::Applied {
                origin: PortableTypeRecipeId(0),
                arguments: vec![PortableTypeRecipeId(0), PortableTypeRecipeId(1)],
            },
        ];
        let mut second = empty_pool();
        second.type_recipes = vec![
            canonical(1),
            canonical(2),
            PortableTypeRecipeV1::Applied {
                origin: PortableTypeRecipeId(0),
                arguments: vec![PortableTypeRecipeId(1), PortableTypeRecipeId(0)],
            },
        ];

        canonicalize_type_recipe_ids(&mut first).expect("first recipe graph canonicalizes");
        canonicalize_type_recipe_ids(&mut second).expect("second recipe graph canonicalizes");

        assert_ne!(first.type_recipes, second.type_recipes);
    }

    #[test]
    fn kind_recipe_ids_and_type_lambda_references_ignore_discovery_order() {
        let mut first = empty_pool();
        first.kind_recipes = vec![
            PortableKindRecipeV1::Canonical { digest: [2; 32] },
            PortableKindRecipeV1::Type,
            PortableKindRecipeV1::Arrow {
                parameters: vec![PortableKindRecipeId(0)],
                result: PortableKindRecipeId(1),
            },
            PortableKindRecipeV1::Type,
            PortableKindRecipeV1::Arrow {
                parameters: vec![PortableKindRecipeId(0)],
                result: PortableKindRecipeId(3),
            },
        ];
        first.type_recipes = vec![
            canonical(9),
            PortableTypeRecipeV1::TypeLambda {
                parameters: vec![PortableTypeLambdaParameterV1 {
                    name: "T".into(),
                    kind: PortableKindRecipeId(2),
                }],
                body: PortableTypeRecipeId(0),
            },
        ];
        if let PortableTypeRecipeV1::TypeLambda { parameters, .. } = &mut first.type_recipes[1] {
            parameters.push(PortableTypeLambdaParameterV1 {
                name: "U".into(),
                kind: PortableKindRecipeId(4),
            });
        }

        let mut second = empty_pool();
        second.kind_recipes = vec![
            PortableKindRecipeV1::Type,
            PortableKindRecipeV1::Canonical { digest: [2; 32] },
            PortableKindRecipeV1::Type,
            PortableKindRecipeV1::Arrow {
                parameters: vec![PortableKindRecipeId(1)],
                result: PortableKindRecipeId(0),
            },
            PortableKindRecipeV1::Arrow {
                parameters: vec![PortableKindRecipeId(1)],
                result: PortableKindRecipeId(2),
            },
        ];
        second.type_recipes = vec![
            PortableTypeRecipeV1::TypeLambda {
                parameters: vec![PortableTypeLambdaParameterV1 {
                    name: "T".into(),
                    kind: PortableKindRecipeId(4),
                }],
                body: PortableTypeRecipeId(1),
            },
            canonical(9),
        ];
        if let PortableTypeRecipeV1::TypeLambda { parameters, .. } = &mut second.type_recipes[0] {
            parameters.push(PortableTypeLambdaParameterV1 {
                name: "U".into(),
                kind: PortableKindRecipeId(3),
            });
        }

        canonicalize_type_recipe_ids(&mut first).expect("first recipe graph canonicalizes");
        canonicalize_type_recipe_ids(&mut second).expect("second recipe graph canonicalizes");

        assert_eq!(first.kind_recipes, second.kind_recipes);
        assert_eq!(first.kind_recipes.len(), 3, "equivalent kind recipes are interned once");
        assert_eq!(first.type_recipes, second.type_recipes);
    }
}

fn portable_layout(layout: &crate::product::ProductLayoutSpec) -> PortableProductLayoutV1 {
    PortableProductLayoutV1 {
        components: layout
            .components
            .iter()
            .map(|component| PortableProductComponentV1 {
                logical_index: component.logical_index,
                repr: match component.repr {
                    crate::product::ProductSlotRepr::Int64 => PortableProductSlotReprV1::Int64,
                    crate::product::ProductSlotRepr::Float64 => PortableProductSlotReprV1::Float64,
                    crate::product::ProductSlotRepr::Bool => PortableProductSlotReprV1::Bool,
                    crate::product::ProductSlotRepr::Symbol => PortableProductSlotReprV1::Symbol,
                    crate::product::ProductSlotRepr::Value => PortableProductSlotReprV1::Value,
                },
            })
            .collect(),
    }
}

fn portable_operation(operation: &phalcom_semantic::types::family::FamilyOperationShape) -> PortableFamilyOperationV1 {
    use phalcom_common::selector::{SelectorKind, SelectorSlot};
    PortableFamilyOperationV1 {
        kind: match operation.kind {
            SelectorKind::Getter => PortableSelectorKindV1::Getter,
            SelectorKind::Setter => PortableSelectorKindV1::Setter,
            SelectorKind::Method => PortableSelectorKindV1::Method,
            SelectorKind::SubscriptGet => PortableSelectorKindV1::SubscriptGet,
            SelectorKind::SubscriptSet => PortableSelectorKindV1::SubscriptSet,
        },
        slots: operation
            .slots
            .iter()
            .map(|slot| match slot {
                SelectorSlot::Positional => PortableSelectorSlotV1::Positional,
                SelectorSlot::Label(label) => PortableSelectorSlotV1::Label { label: label.clone() },
            })
            .collect(),
    }
}

fn portable_rest_mode(mode: ExecutableRestMode) -> PortableRestModeV1 {
    match mode {
        ExecutableRestMode::None => PortableRestModeV1::None,
        ExecutableRestMode::Positional => PortableRestModeV1::Positional,
        ExecutableRestMode::Labeled => PortableRestModeV1::Labeled,
        ExecutableRestMode::Complete => PortableRestModeV1::Complete,
    }
}

fn portable_dispatch_side(side: phalcom_semantic::identity::DispatchSide) -> phalcom_type_meta::identity::StableDispatchSide {
    match side {
        phalcom_semantic::identity::DispatchSide::Instance => phalcom_type_meta::identity::StableDispatchSide::Instance,
        phalcom_semantic::identity::DispatchSide::Class => phalcom_type_meta::identity::StableDispatchSide::Class,
    }
}

fn portable_trait_requirement(requirement: &phalcom_semantic::traits::TraitRequirementId) -> PortableTraitRequirementV1 {
    PortableTraitRequirementV1 {
        owner: to_stable_declaration(&requirement.owner),
        selector: requirement.selector.encode(),
        side: portable_dispatch_side(requirement.side),
    }
}

fn trait_requirement_order(left: &PortableTraitRequirementV1, right: &PortableTraitRequirementV1) -> std::cmp::Ordering {
    (&left.owner, &left.selector, left.side).cmp(&(&right.owner, &right.selector, right.side))
}

fn portable_type_parameter(parameter: &phalcom_type_meta::StableTypeParameterRef) -> PortableTypeParameterRefV1 {
    PortableTypeParameterRefV1 {
        owner: match &parameter.owner {
            StableTypeParameterOwnerRef::Declaration(owner) => PortableTypeParameterOwnerV1::Declaration { owner: owner.clone() },
            StableTypeParameterOwnerRef::Callable(owner) => PortableTypeParameterOwnerV1::Callable { owner: owner.clone() },
            StableTypeParameterOwnerRef::Impl(owner) => PortableTypeParameterOwnerV1::Implementation { owner: owner.clone() },
        },
        index: parameter.index,
    }
}

fn exact_selector_pattern(selector: &phalcom_common::selector::Selector) -> SelectorPatternV1 {
    let pattern = phalcom_common::selector::SelectorPattern {
        base: selector.base.clone(),
        kind: phalcom_common::selector::SelectorKindPattern::Exact(selector.kind),
        prefix: selector.slots.clone(),
        suffix: Box::new([]),
        has_gap: false,
    };
    crate::artifact::project::project_selector_pattern(&pattern)
}

fn portable_field_key(key: &phalcom_semantic::field_key::FieldKey) -> PortableFieldKeyV1 {
    match key {
        phalcom_semantic::field_key::FieldKey::Symbol(text) => PortableFieldKeyV1::Symbol { text: text.to_string() },
        phalcom_semantic::field_key::FieldKey::String(text) => PortableFieldKeyV1::String { text: text.to_string() },
        phalcom_semantic::field_key::FieldKey::Int(value) => PortableFieldKeyV1::Integer {
            decimal: value.to_bigint().to_string(),
        },
        phalcom_semantic::field_key::FieldKey::Bool(value) => PortableFieldKeyV1::Boolean { value: *value },
    }
}

fn stable_class(vm: &crate::vm::VM, class: crate::heap::ClassId) -> ProjectionResult<StableClassRefV1> {
    crate::artifact::project::project_class_reference(vm, class).map_err(|error| SemanticProjectionError {
        family: "type_recipes",
        reason: format!("class type has no stable semantic owner: {error}"),
    })
}

fn stable_type_digest(fingerprint: Fingerprint128) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new_derive_key("phalcom.telosc.executable.semantic-type.v1");
    hasher.update(fingerprint.as_bytes());
    *hasher.finalize().as_bytes()
}

fn data_construction_type_recipe(
    recipe: RuntimeTypeRecipe,
    root: phalcom_type_meta::type_node::TypeNodeId,
    entry: &phalcom_type_meta::type_node::TypeNodeEntry,
) -> ProjectionResult<(bool, PortableTypeRecipeV1)> {
    let (template, runtime_type) = match recipe {
        RuntimeTypeRecipe::Closed(runtime_type) => (false, runtime_type),
        RuntimeTypeRecipe::Template(runtime_type) => (true, runtime_type),
    };
    match runtime_type {
        RuntimeTypeRef::Base { node, .. } if node == root => Ok((
            template,
            PortableTypeRecipeV1::Canonical {
                digest: stable_type_digest(entry.structural_fingerprint),
            },
        )),
        RuntimeTypeRef::Base { .. } => fail("data_constructions", "runtime type recipe does not reference its owning metadata root"),
        RuntimeTypeRef::Overlay(_) => fail("data_constructions", "exact data construction type must be owned by semantic metadata"),
    }
}

fn exact_metadata_digest(bundle: &phalcom_type_meta::SemanticMetadataBundle) -> ProjectionResult<[u8; 32]> {
    let encoded = serde_json::to_vec(bundle).map_err(|error| SemanticProjectionError {
        family: "data_constructions",
        reason: format!("exact type metadata could not be fingerprinted: {error}"),
    })?;
    let mut hasher = blake3::Hasher::new_derive_key("phalcom.telosc.executable.exact-type-metadata.v1");
    hasher.update(&encoded);
    Ok(*hasher.finalize().as_bytes())
}

#[cfg(test)]
mod data_construction_type_recipe_tests {
    use super::*;

    #[test]
    fn data_construction_recipe_uses_its_own_metadata_root() {
        let root = phalcom_type_meta::type_node::TypeNodeId(2);
        let entry = phalcom_type_meta::type_node::TypeNodeEntry {
            kind: phalcom_type_meta::kind::KindNodeId(0),
            form: phalcom_type_meta::type_node::TypeNode::Unit,
            structural_fingerprint: Fingerprint128::from_u128(0x51a7),
        };
        let recipe = RuntimeTypeRecipe::Template(RuntimeTypeRef::Base {
            pool: crate::typing::MetadataPoolId(97),
            node: root,
        });

        let (template, portable) = data_construction_type_recipe(recipe, root, &entry).expect("owned root recipe");
        assert!(template);
        assert_eq!(
            portable,
            PortableTypeRecipeV1::Canonical {
                digest: stable_type_digest(entry.structural_fingerprint),
            }
        );
        assert!(data_construction_type_recipe(recipe, phalcom_type_meta::type_node::TypeNodeId(3), &entry).is_err());
    }
}

impl SemanticPoolProjector<'_> {
    fn anonymous_product(
        &mut self,
        spec: &crate::modules::semantic_lowering::AnonymousProductConstructionLoweringSpec,
    ) -> ProjectionResult<PortableAnonymousProductV1> {
        let kind = match &spec.kind {
            crate::modules::semantic_lowering::AnonymousProductConstructionKind::Tuple { positional_len, labels } => PortableAnonymousProductKindV1::Tuple {
                positional_len: *positional_len,
                labels: labels.iter().map(ToString::to_string).collect(),
            },
            crate::modules::semantic_lowering::AnonymousProductConstructionKind::Record {
                logical_keys,
                source_to_logical,
            } => PortableAnonymousProductKindV1::Record {
                logical_keys: logical_keys.iter().map(portable_field_key).collect(),
                source_to_logical: source_to_logical.to_vec(),
            },
            crate::modules::semantic_lowering::AnonymousProductConstructionKind::Pack { positional_len, labels } => PortableAnonymousProductKindV1::Pack {
                positional_len: *positional_len,
                labels: labels.iter().map(ToString::to_string).collect(),
            },
        };
        Ok(PortableAnonymousProductV1 {
            kind,
            layout: portable_layout(&spec.layout),
            type_recipe: self.runtime_type_recipe(spec.type_recipe)?,
        })
    }
}
