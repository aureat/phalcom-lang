//! Canonical artifact-local numbering for the captured executable graph.

use phalcom_compiler_artifact::executable::constant::{ConstantRecipeV1, MethodBodyV1};
use phalcom_compiler_artifact::executable::image::ExecutableImageV1;
use phalcom_compiler_artifact::executable::prototype::{ConstantId, PortablePrototypeV1, PrototypeId, PrototypeOwnerV1, SemanticPoolId};
use phalcom_type_meta::identity::{StableCallableRef, StableModuleRef};

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum PrototypeRootV1 {
    Module(StableModuleRef),
    Callable(StableCallableRef),
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct PrototypeKeyV1 {
    root: PrototypeRootV1,
    nested_ordinals: Vec<u32>,
}

/// Renumbers graph tables from stable ownership and ordered graph edges.
pub(crate) fn canonicalize_graph_ids(image: &mut ExecutableImageV1) -> Result<(), String> {
    image.detached_methods.sort_by(|left, right| left.callable.cmp(&right.callable));
    promote_detached_callable_owners(image)?;

    let mut keys = vec![None; image.prototypes.len()];
    let mut visiting = vec![false; image.prototypes.len()];
    for index in 0..image.prototypes.len() {
        prototype_key(index, &image.prototypes, &mut keys, &mut visiting)?;
    }
    let mut order = (0..image.prototypes.len()).collect::<Vec<_>>();
    order.sort_by_key(|index| keys[*index].as_ref().expect("key computed").clone());
    if order.windows(2).any(|pair| keys[pair[0]] == keys[pair[1]]) {
        return Err("duplicate canonical executable prototype owner".into());
    }

    let mut prototype_remap = vec![PrototypeId(u32::MAX); image.prototypes.len()];
    for (new, old) in order.iter().copied().enumerate() {
        prototype_remap[old] = PrototypeId(u32::try_from(new).map_err(|_| "prototype index exceeds u32")?);
    }

    let old_prototypes = std::mem::take(&mut image.prototypes).into_iter().map(Some).collect::<Vec<_>>();
    let mut old_semantic_pools = std::mem::take(&mut image.semantic_pools).into_iter().map(Some).collect::<Vec<_>>();
    let mut semantic_remap = vec![None; old_semantic_pools.len()];
    let mut prototypes = Vec::with_capacity(old_prototypes.len());
    let mut semantic_pools = Vec::with_capacity(old_semantic_pools.len());
    let mut old_prototypes = old_prototypes;

    for old in order {
        let mut prototype = old_prototypes[old].take().expect("prototype order contains unique rows");
        if let PrototypeOwnerV1::Nested { parent, ordinal } = &prototype.owner {
            let parent = remap_prototype(*parent, &prototype_remap)?;
            prototype.owner = PrototypeOwnerV1::Nested { parent, ordinal: *ordinal };
        }
        let old_semantic = prototype.semantics.0 as usize;
        let new_semantic = if let Some(existing) = semantic_remap.get(old_semantic).copied().flatten() {
            existing
        } else {
            let pool = old_semantic_pools
                .get_mut(old_semantic)
                .and_then(Option::take)
                .ok_or_else(|| "prototype semantic pool is absent or multiply assigned".to_owned())?;
            let new = SemanticPoolId(u32::try_from(semantic_pools.len()).map_err(|_| "semantic pool index exceeds u32")?);
            *semantic_remap
                .get_mut(old_semantic)
                .ok_or_else(|| "prototype semantic pool index is out of bounds".to_owned())? = Some(new);
            semantic_pools.push(pool);
            new
        };
        prototype.semantics = new_semantic;
        prototypes.push(prototype);
    }
    if old_prototypes.iter().any(Option::is_some) || old_semantic_pools.iter().any(Option::is_some) {
        return Err("captured executable table contains an unowned prototype or semantic pool".into());
    }
    image.prototypes = prototypes;
    image.semantic_pools = semantic_pools;

    for module in &mut image.modules {
        module.initializer = remap_prototype(module.initializer, &prototype_remap)?;
    }
    for module in &mut image.bootstrap.eager_modules {
        module.initializer = remap_prototype(module.initializer, &prototype_remap)?;
    }
    for constant in &mut image.constants {
        if let ConstantRecipeV1::ClosureTemplate { prototype, .. } = constant {
            *prototype = remap_prototype(*prototype, &prototype_remap)?;
        }
    }

    canonicalize_constant_ids(image)
}

fn promote_detached_callable_owners(image: &mut ExecutableImageV1) -> Result<(), String> {
    let mut promotions = Vec::new();
    for row in &image.detached_methods {
        let Some(ConstantRecipeV1::Method { recipe }) = image.constants.get(row.method_constant.0 as usize) else {
            return Err("detached method references an absent method recipe".into());
        };
        let MethodBodyV1::Closure { template } = &recipe.body else {
            continue;
        };
        let Some(ConstantRecipeV1::ClosureTemplate { prototype, .. }) = image.constants.get(template.0 as usize) else {
            return Err("detached method closure references an absent template recipe".into());
        };
        promotions.push((*prototype, row.callable.clone()));
    }
    for (prototype, callable) in promotions {
        let row = image
            .prototypes
            .get_mut(prototype.0 as usize)
            .ok_or_else(|| "detached callable refers to an absent prototype".to_owned())?;
        match &row.owner {
            PrototypeOwnerV1::Callable { callable: existing } if existing != &callable => {
                return Err("one prototype has multiple stable callable owners".into());
            }
            PrototypeOwnerV1::ModuleInitializer { .. } => {
                return Err("detached callable aliases a module initializer prototype".into());
            }
            _ => row.owner = PrototypeOwnerV1::Callable { callable },
        }
    }
    Ok(())
}

fn prototype_key(
    index: usize,
    prototypes: &[PortablePrototypeV1],
    keys: &mut [Option<PrototypeKeyV1>],
    visiting: &mut [bool],
) -> Result<PrototypeKeyV1, String> {
    if let Some(key) = keys.get(index).and_then(Option::as_ref) {
        return Ok(key.clone());
    }
    if visiting.get(index).copied().unwrap_or(false) {
        return Err("prototype owner graph contains a cycle".into());
    }
    let owner = &prototypes
        .get(index)
        .ok_or_else(|| "prototype owner graph references an absent row".to_owned())?
        .owner;
    visiting[index] = true;
    let key = match owner {
        PrototypeOwnerV1::ModuleInitializer { module } => PrototypeKeyV1 {
            root: PrototypeRootV1::Module(module.clone()),
            nested_ordinals: Vec::new(),
        },
        PrototypeOwnerV1::Callable { callable } => PrototypeKeyV1 {
            root: PrototypeRootV1::Callable(callable.clone()),
            nested_ordinals: Vec::new(),
        },
        PrototypeOwnerV1::Nested { parent, ordinal } => {
            let mut parent_key = prototype_key(parent.0 as usize, prototypes, keys, visiting)?;
            parent_key.nested_ordinals.push(*ordinal);
            parent_key
        }
    };
    visiting[index] = false;
    keys[index] = Some(key.clone());
    Ok(key)
}

fn remap_prototype(id: PrototypeId, remap: &[PrototypeId]) -> Result<PrototypeId, String> {
    remap
        .get(id.0 as usize)
        .copied()
        .ok_or_else(|| "executable graph references an absent prototype".into())
}

fn canonicalize_constant_ids(image: &mut ExecutableImageV1) -> Result<(), String> {
    let mut states = vec![0u8; image.constants.len()];
    let mut order = Vec::with_capacity(image.constants.len());
    for prototype in &image.prototypes {
        for constant in &prototype.constants {
            visit_constant(*constant, &image.constants, &mut states, &mut order)?;
        }
        let pool = image
            .semantic_pools
            .get(prototype.semantics.0 as usize)
            .ok_or_else(|| "prototype semantic pool is absent during constant numbering".to_owned())?;
        for family in &pool.conditional_families {
            for entry in &family.entries {
                visit_constant(entry.fallback_method, &image.constants, &mut states, &mut order)?;
            }
        }
    }
    for method in &image.detached_methods {
        visit_constant(method.method_constant, &image.constants, &mut states, &mut order)?;
    }
    if order.len() != image.constants.len() {
        return Err("captured executable constant is unreachable from canonical graph roots".into());
    }

    let mut remap = vec![ConstantId(u32::MAX); image.constants.len()];
    for (new, old) in order.iter().copied().enumerate() {
        remap[old] = ConstantId(u32::try_from(new).map_err(|_| "constant index exceeds u32")?);
    }
    let old_constants = std::mem::take(&mut image.constants);
    let mut constants = Vec::with_capacity(old_constants.len());
    for old in order {
        let mut recipe = old_constants[old].clone();
        remap_constant_references(&mut recipe, &remap)?;
        constants.push(recipe);
    }
    image.constants = constants;

    for prototype in &mut image.prototypes {
        for constant in &mut prototype.constants {
            *constant = remap_constant(*constant, &remap)?;
        }
    }
    for pool in &mut image.semantic_pools {
        for family in &mut pool.conditional_families {
            for entry in &mut family.entries {
                entry.fallback_method = remap_constant(entry.fallback_method, &remap)?;
            }
        }
    }
    for method in &mut image.detached_methods {
        method.method_constant = remap_constant(method.method_constant, &remap)?;
    }
    Ok(())
}

fn visit_constant(id: ConstantId, constants: &[ConstantRecipeV1], states: &mut [u8], order: &mut Vec<usize>) -> Result<(), String> {
    let index = id.0 as usize;
    let state = states
        .get_mut(index)
        .ok_or_else(|| "executable graph references an absent constant".to_owned())?;
    match *state {
        1 => return Err("constant recipe graph contains a cycle".into()),
        2 => return Ok(()),
        _ => *state = 1,
    }
    let recipe = constants
        .get(index)
        .ok_or_else(|| "executable graph references an absent constant".to_owned())?;
    for dependency in constant_dependencies(recipe) {
        visit_constant(dependency, constants, states, order)?;
    }
    states[index] = 2;
    order.push(index);
    Ok(())
}

fn constant_dependencies(recipe: &ConstantRecipeV1) -> Vec<ConstantId> {
    let mut dependencies = Vec::new();
    match recipe {
        ConstantRecipeV1::Method { recipe } => {
            if let MethodBodyV1::Closure { template } = recipe.body {
                dependencies.push(template);
            }
            if let Some(contracts) = &recipe.contracts {
                dependencies.extend(contracts.iter().map(|contract| contract.value));
            }
            dependencies.extend(recipe.attributes.iter().copied());
        }
        ConstantRecipeV1::Vector { elements } | ConstantRecipeV1::FrozenArray { elements } => dependencies.extend(elements.iter().copied()),
        ConstantRecipeV1::Tuple {
            positionals, labeled_values, ..
        }
        | ConstantRecipeV1::Pack {
            positionals, labeled_values, ..
        } => {
            dependencies.extend(positionals.iter().copied());
            dependencies.extend(labeled_values.iter().copied());
        }
        ConstantRecipeV1::Record { values, .. } => dependencies.extend(values.iter().copied()),
        ConstantRecipeV1::None
        | ConstantRecipeV1::Unit
        | ConstantRecipeV1::EmptyRecord
        | ConstantRecipeV1::EmptyPack
        | ConstantRecipeV1::Boolean { .. }
        | ConstantRecipeV1::Integer { .. }
        | ConstantRecipeV1::Float { .. }
        | ConstantRecipeV1::Character { .. }
        | ConstantRecipeV1::Symbol { .. }
        | ConstantRecipeV1::String { .. }
        | ConstantRecipeV1::BigInteger { .. }
        | ConstantRecipeV1::Class { .. }
        | ConstantRecipeV1::Module { .. }
        | ConstantRecipeV1::ClosureTemplate { .. }
        | ConstantRecipeV1::SelectorPattern { .. }
        | ConstantRecipeV1::Nil => {}
    }
    dependencies
}

fn remap_constant_references(recipe: &mut ConstantRecipeV1, remap: &[ConstantId]) -> Result<(), String> {
    match recipe {
        ConstantRecipeV1::Method { recipe } => {
            if let MethodBodyV1::Closure { template } = &mut recipe.body {
                *template = remap_constant(*template, remap)?;
            }
            if let Some(contracts) = &mut recipe.contracts {
                for contract in contracts {
                    contract.value = remap_constant(contract.value, remap)?;
                }
            }
            for attribute in &mut recipe.attributes {
                *attribute = remap_constant(*attribute, remap)?;
            }
        }
        ConstantRecipeV1::Vector { elements } | ConstantRecipeV1::FrozenArray { elements } => {
            for element in elements {
                *element = remap_constant(*element, remap)?;
            }
        }
        ConstantRecipeV1::Tuple {
            positionals, labeled_values, ..
        }
        | ConstantRecipeV1::Pack {
            positionals, labeled_values, ..
        } => {
            for element in positionals.iter_mut().chain(labeled_values) {
                *element = remap_constant(*element, remap)?;
            }
        }
        ConstantRecipeV1::Record { values, .. } => {
            for value in values {
                *value = remap_constant(*value, remap)?;
            }
        }
        ConstantRecipeV1::None
        | ConstantRecipeV1::Unit
        | ConstantRecipeV1::EmptyRecord
        | ConstantRecipeV1::EmptyPack
        | ConstantRecipeV1::Boolean { .. }
        | ConstantRecipeV1::Integer { .. }
        | ConstantRecipeV1::Float { .. }
        | ConstantRecipeV1::Character { .. }
        | ConstantRecipeV1::Symbol { .. }
        | ConstantRecipeV1::String { .. }
        | ConstantRecipeV1::BigInteger { .. }
        | ConstantRecipeV1::Class { .. }
        | ConstantRecipeV1::Module { .. }
        | ConstantRecipeV1::ClosureTemplate { .. }
        | ConstantRecipeV1::SelectorPattern { .. }
        | ConstantRecipeV1::Nil => {}
    }
    Ok(())
}

fn remap_constant(id: ConstantId, remap: &[ConstantId]) -> Result<ConstantId, String> {
    remap
        .get(id.0 as usize)
        .copied()
        .ok_or_else(|| "executable graph references an absent constant".into())
}
