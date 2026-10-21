//! Core-owned static validation that turns portable executable staging data
//! into runtime authority tied to the already-loaded P7 compiler image.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use phalcom_compiler_artifact::ArtifactError;
use phalcom_compiler_artifact::executable::compatibility::{CompilationProfileV1, PortableNativeAnchorV1, PrimordialRoleV1};
use phalcom_compiler_artifact::executable::constant::{ConstantRecipeV1, MethodSignatureV1, RestLayoutV1, RestModeV1, StableClassRefV1};
use phalcom_compiler_artifact::executable::image::{ExecutableImageLimitsV1, ExecutableImageV1};
use phalcom_compiler_artifact::executable::instruction::{OPCODE_MANIFEST_V1, OperandDomainV1, PortableOperandValueV1};
use phalcom_compiler_artifact::executable::module::{PortableClassAllocationV1, PortableLinkedReadV1};
use phalcom_compiler_artifact::executable::prototype::{PortablePrototypeV1, PrototypeOwnerV1, RestKindV1, UpvalueCaptureV1};
use phalcom_compiler_artifact::executable::semantic::{
    PortableConformanceSlotEntryV1, PortableConformanceSlotV1, PortableFamilyTargetV1, PortableInvocationTargetV1, PortableKindRecipeV1,
    PortableRuntimeTypeRecipeV1, PortableSemanticPoolV1, PortableTraitRequirementV1, PortableTraitSelectionTargetV1, PortableTypeParameterOwnerV1,
    PortableTypeParameterRefV1, PortableTypeRecipeV1,
};
use phalcom_modules::linker::{LinkedModule, LinkedReadSpec};
use phalcom_semantic::metadata::stable_identity::{
    to_stable_data_component, to_stable_declaration, to_stable_impl, to_stable_module, to_stable_variant, to_stable_variant_field,
};
use phalcom_type_meta::fingerprint::Fingerprint128;
use phalcom_type_meta::generic::{StableTypeParameterOwnerRef, StableTypeParameterRef};
use phalcom_type_meta::identity::{
    StableCallableOwnerRef, StableCallableRef, StableDataComponentRef, StableDeclarationRef, StableDispatchSide, StableFieldRef, StableImplRef,
    StableModuleRef, StableStaticConstantRef, StableVariantFieldRef, StableVariantRef,
};

use crate::modules::canonical_artifact::LoadedUniverseArtifact;

#[derive(Debug, thiserror::Error)]
pub(crate) enum ExecutableValidationError {
    #[error("executable structural validation failed: {0}")]
    Structural(#[from] ArtifactError),
    #[error("unsupported Universe executable profile {0:?}; expected Debug")]
    UnsupportedProfile(CompilationProfileV1),
    #[error("loaded P7 authority contains duplicate stable module identity {0:?}")]
    DuplicateP7Module(StableModuleRef),
    #[error("executable is missing P7 module {0:?}")]
    MissingP7Module(StableModuleRef),
    #[error("executable contains module absent from P7 authority: {0:?}")]
    UnknownP7Module(StableModuleRef),
    #[error("executable linked reads for module {module:?} differ from P7 at index {index}")]
    LinkedReadMismatch { module: StableModuleRef, index: usize },
    #[error("executable bootstrap order places {module:?} before eager dependency {dependency:?}")]
    BootstrapDependencyOrder { module: StableModuleRef, dependency: StableModuleRef },
    #[error("executable references P7 module absent from the loaded module world: {0:?}")]
    MissingLinkedDependency(StableModuleRef),
    #[error("executable native anchor count is {found}, but the host catalog requires {expected}")]
    NativeAnchorCount { found: usize, expected: usize },
    #[error("native anchor {binding_key:?} does not match the current host catalog field {field}")]
    NativeAnchorMismatch { binding_key: String, field: &'static str },
    #[error("host native descriptor {binding_key:?} is invalid: {reason}")]
    InvalidHostNativeDescriptor { binding_key: String, reason: String },
    #[error("executable primordial role count is {found}, but the host runtime requires {expected}")]
    PrimordialRoleCount { found: usize, expected: usize },
    #[error("executable primordial role {role:?} does not match the host runtime binding")]
    PrimordialRoleMismatch { role: String },
    #[error("primordial role {role:?} has no class recipe")]
    PrimordialClassMissing { role: String },
    #[error("class recipe references declaration absent from P7 metadata: {0:?}")]
    UnknownP7Declaration(StableDeclarationRef),
    #[error("class recipe references module absent from P7 authority: {0:?}")]
    UnknownClassModule(StableModuleRef),
    #[error("executable class reference has no class recipe: {0:?}")]
    MissingClassRecipe(StableClassRefV1),
    #[error("executable class inheritance contains a cycle at {0:?}")]
    SuperclassCycle(StableClassRefV1),
    #[error("executable prototype callable is absent from P7 metadata: {0:?}")]
    UnknownP7Callable(phalcom_type_meta::identity::StableCallableRef),
    #[error("executable prototype callable references a module absent from P7 authority: {0:?}")]
    UnknownCallableModule(StableModuleRef),
    #[error("bootstrap-required declaration is absent from P7 metadata: {0:?}")]
    UnknownRequiredDeclaration(StableDeclarationRef),
    #[error("semantic pool {pool} type recipe {recipe} names an unknown canonical type")]
    UnknownCanonicalType { pool: usize, recipe: usize },
    #[error("semantic pool {pool} kind recipe {recipe} names an unknown canonical kind")]
    UnknownCanonicalKind { pool: usize, recipe: usize },
    #[error("semantic pool {pool} type recipe {recipe} references an absent class")]
    UnknownSemanticClass { pool: usize, recipe: usize },
    #[error("semantic pool {pool} type recipe {recipe} references an absent P7 declaration")]
    UnknownSemanticDeclaration { pool: usize, recipe: usize },
    #[error("semantic pool {pool} type recipe {recipe} references an absent P7 callable")]
    UnknownSemanticCallable { pool: usize, recipe: usize },
    #[error("semantic pool {pool} type recipe {recipe} references an absent P7 module")]
    UnknownSemanticModule { pool: usize, recipe: usize },
    #[error("semantic pool {pool} {family} row {row} references an identity absent from P7 authority")]
    UnknownSemanticIdentity { pool: usize, family: &'static str, row: usize },
    #[error("semantic pool {pool} {family} row {row} is inconsistent with P7 authority: {detail}")]
    InvalidSemanticProduct {
        pool: usize,
        family: &'static str,
        row: usize,
        detail: String,
    },
    #[error("prototype {prototype} instruction {instruction} has invalid {operand}: {detail}")]
    InvalidInstruction {
        prototype: usize,
        instruction: usize,
        operand: &'static str,
        detail: String,
    },
    #[error("prototype {prototype} instruction {instruction} targets invalid instruction index {target}")]
    InvalidBranchTarget { prototype: usize, instruction: usize, target: i64 },
    #[error("prototype {prototype} has invalid executable frame or parameter layout: {detail}")]
    InvalidPrototypeLayout { prototype: usize, detail: String },
    #[error("nested prototype {child} has invalid capture from parent {parent}: {detail}")]
    InvalidClosureCapture { child: usize, parent: usize, detail: String },
    #[error("prototype owner {owner:?} is claimed by more than one executable prototype")]
    DuplicatePrototypeOwner { owner: PrototypeOwnerV1 },
    #[error("class recipe {class:?} has an invalid runtime field layout: {detail}")]
    InvalidClassLayout { class: StableClassRefV1, detail: String },
    #[error("prototype {prototype} uses field slot {slot} without a P7-owned class layout")]
    UnknownFieldLayout { prototype: usize, slot: u16 },
    #[error("executable {table} row {index} is invalid: {detail}")]
    InvalidExecutableReference { table: &'static str, index: usize, detail: String },
    #[error("prototype {prototype} instruction {instruction} uses opcode tag {tag} absent from the manifest")]
    UnknownOpcodeManifestTag { prototype: usize, instruction: usize, tag: u16 },
}

/// An executable image that has passed every core-owned static authorization
/// check. Its private fields prevent callers from manufacturing this authority
/// from a decoded DTO.
pub(crate) struct ValidatedExecutableImage {
    image: Arc<ExecutableImageV1>,
}

impl ValidatedExecutableImage {
    pub(crate) fn image(&self) -> &ExecutableImageV1 {
        &self.image
    }
}

struct ExecutableValidationContext<'a> {
    linked_modules: BTreeMap<StableModuleRef, &'a LinkedModule>,
    declarations: BTreeSet<StableDeclarationRef>,
    class_declarations: BTreeSet<StableDeclarationRef>,
    callables: BTreeSet<phalcom_type_meta::identity::StableCallableRef>,
    fields: BTreeSet<StableFieldRef>,
    implementations: BTreeSet<StableImplRef>,
    type_parameters: BTreeSet<StableTypeParameterRef>,
    variants: BTreeSet<StableVariantRef>,
    variant_fields: BTreeSet<StableVariantFieldRef>,
    variant_field_counts: BTreeMap<StableVariantRef, usize>,
    variant_field_names: BTreeMap<StableVariantFieldRef, String>,
    enum_variants: BTreeMap<StableDeclarationRef, Vec<StableVariantRef>>,
    data_declarations: BTreeSet<StableDeclarationRef>,
    data_components: BTreeSet<StableDataComponentRef>,
    data_component_names: BTreeMap<StableDataComponentRef, String>,
    static_constants: BTreeSet<StableStaticConstantRef>,
    canonical_type_digests: BTreeSet<[u8; 32]>,
    canonical_kind_digests: BTreeSet<[u8; 32]>,
}

impl<'a> ExecutableValidationContext<'a> {
    fn new(loaded: &'a LoadedUniverseArtifact) -> Result<Self, ExecutableValidationError> {
        let mut linked_modules = BTreeMap::new();
        for (module, linked) in loaded.formal_image.linked().modules.iter() {
            let stable = to_stable_module(module);
            if linked_modules.insert(stable.clone(), linked).is_some() {
                return Err(ExecutableValidationError::DuplicateP7Module(stable));
            }
        }

        let metadata = &loaded.metadata;
        let declarations = metadata
            .declarations
            .iter()
            .map(|record| record.declaration.clone())
            .chain(metadata.aliases.iter().map(|record| record.declaration.clone()))
            .collect();
        let class_declarations = metadata
            .declarations
            .iter()
            .filter(|record| !record.flags.is_trait)
            .map(|record| record.declaration.clone())
            .collect();
        let callables = metadata.callables.iter().map(|record| record.callable.clone()).collect();
        let fields = metadata.fields.iter().map(|record| record.field.clone()).collect();
        let mut implementations = BTreeSet::new();
        let snapshot = loaded.formal_image.snapshot();
        implementations.extend(snapshot.conformance_index.iter().map(|(implementation, _)| to_stable_impl(implementation)));
        implementations.extend(
            snapshot
                .conformance_witness_plans
                .iter()
                .map(|(implementation, _)| to_stable_impl(implementation)),
        );
        implementations.extend(snapshot.derived_conformance_rules.iter().map(|rule| to_stable_impl(&rule.impl_id)));
        let type_parameters = metadata.parameters.iter().map(|parameter| parameter.id.clone()).collect();
        let mut variants = BTreeSet::new();
        let mut variant_fields = BTreeSet::new();
        let mut variant_field_counts = BTreeMap::new();
        let mut variant_field_names = BTreeMap::new();
        let mut enum_variants = BTreeMap::new();
        let mut data_declarations = BTreeSet::new();
        let mut data_components = BTreeSet::new();
        let mut data_component_names = BTreeMap::new();
        for (module, _) in loaded.formal_image.linked().modules.iter() {
            for (_, data) in snapshot.data_semantics.iter_module(module) {
                let owner = to_stable_declaration(&data.owner);
                data_declarations.insert(owner);
                for component in &data.components {
                    let stable = to_stable_data_component(&component.id);
                    data_component_names.insert(stable.clone(), component.local_name.to_string());
                    data_components.insert(stable);
                }
            }
            for (_, enum_info) in snapshot.enum_semantics.iter_module(module) {
                let mut stable_variants = Vec::with_capacity(enum_info.variants.len());
                for variant_id in enum_info.variants.iter() {
                    let Some(variant) = snapshot.enum_semantics.variant_info(variant_id) else {
                        continue;
                    };
                    let stable_variant = to_stable_variant(&variant.id);
                    variant_field_counts.insert(stable_variant.clone(), variant.fields.len());
                    variants.insert(stable_variant.clone());
                    stable_variants.push(stable_variant);
                    for field in variant.fields.iter() {
                        let stable_field = to_stable_variant_field(&field.id);
                        variant_field_names.insert(stable_field.clone(), field.local_name.to_string());
                        variant_fields.insert(stable_field);
                    }
                }
                enum_variants.insert(to_stable_declaration(&enum_info.owner), stable_variants);
            }
        }
        implementations.extend(metadata.parameters.iter().filter_map(|parameter| match &parameter.id.owner {
            StableTypeParameterOwnerRef::Impl(implementation) => Some(implementation.clone()),
            _ => None,
        }));
        let static_constants = metadata.static_constants.iter().map(|constant| constant.id.clone()).collect();
        let canonical_type_digests = metadata
            .types
            .iter()
            .map(|entry| stable_semantic_digest(entry.structural_fingerprint))
            .collect();
        let canonical_kind_digests = metadata
            .kinds
            .iter()
            .map(|entry| stable_semantic_digest(entry.structural_fingerprint))
            .collect();

        Ok(Self {
            linked_modules,
            declarations,
            class_declarations,
            callables,
            fields,
            implementations,
            type_parameters,
            variants,
            variant_fields,
            variant_field_counts,
            variant_field_names,
            enum_variants,
            data_declarations,
            data_components,
            data_component_names,
            static_constants,
            canonical_type_digests,
            canonical_kind_digests,
        })
    }

    fn has_p7_module(&self, module: &StableModuleRef) -> bool {
        self.linked_modules.contains_key(module)
    }

    fn has_p7_declaration(&self, declaration: &StableDeclarationRef) -> bool {
        self.declarations.contains(declaration)
    }

    fn has_p7_callable(&self, callable: &phalcom_type_meta::identity::StableCallableRef) -> bool {
        self.callables.contains(callable)
    }

    fn has_p7_class(&self, class: &StableClassRefV1) -> bool {
        self.class_declarations.contains(&class.declaration)
    }

    fn has_p7_variant(&self, variant: &StableVariantRef) -> bool {
        self.variants.contains(variant)
    }

    fn has_p7_data_component(&self, component: &StableDataComponentRef) -> bool {
        self.data_components.contains(component)
    }

    fn has_p7_variant_field(&self, field: &StableVariantFieldRef) -> bool {
        self.variant_fields.contains(field)
    }

    fn has_p7_type_parameter(&self, parameter: &StableTypeParameterRef) -> bool {
        self.type_parameters.contains(parameter)
    }

    fn has_p7_implementation(&self, implementation: &StableImplRef) -> bool {
        self.implementations.contains(implementation)
    }

    fn has_p7_field(&self, field: &StableFieldRef) -> bool {
        self.fields.contains(field)
    }

    fn has_p7_static_constant(&self, constant: &StableStaticConstantRef) -> bool {
        self.static_constants.contains(constant)
    }
}

/// Checks the executable's initial compatibility and P7-reference boundary.
/// This remains a staging check, not execution authority: the function returns
/// no VM-usable product until prototype, control-flow, and semantic-pool
/// validation are complete.
pub(crate) fn validate_executable_image(
    image: &ExecutableImageV1,
    loaded: &LoadedUniverseArtifact,
    limits: &ExecutableImageLimitsV1,
) -> Result<(), ExecutableValidationError> {
    image.validate_structure(limits)?;
    image.validate_native_abi_digest()?;
    if image.compatibility.compilation_profile != CompilationProfileV1::Debug {
        return Err(ExecutableValidationError::UnsupportedProfile(image.compatibility.compilation_profile.clone()));
    }

    let context = ExecutableValidationContext::new(loaded)?;
    validate_module_coverage(image, &context)?;
    validate_bootstrap_dependencies(image, &context)?;
    validate_native_anchors(image)?;
    validate_primordial_roles(image)?;
    validate_classes(image, &context)?;
    validate_prototype_owners(image, &context)?;
    validate_prototypes(image, &context)?;
    validate_required_declarations(image, &context)?;
    validate_canonical_semantic_roots(image, &context)?;
    validate_semantic_pools(image, &context)?;
    Ok(())
}

fn validate_module_coverage(image: &ExecutableImageV1, context: &ExecutableValidationContext<'_>) -> Result<(), ExecutableValidationError> {
    let actual_modules = image.modules.iter().map(|module| module.identity.clone()).collect::<BTreeSet<_>>();
    for module in context.linked_modules.keys() {
        if !actual_modules.contains(module) {
            return Err(ExecutableValidationError::MissingP7Module(module.clone()));
        }
    }
    for module in &actual_modules {
        if !context.has_p7_module(module) {
            return Err(ExecutableValidationError::UnknownP7Module(module.clone()));
        }
    }

    for module in &image.modules {
        let Some(linked) = context.linked_modules.get(&module.identity) else {
            return Err(ExecutableValidationError::UnknownP7Module(module.identity.clone()));
        };
        if module.linked_reads.len() != linked.linked_reads.len() {
            return Err(ExecutableValidationError::LinkedReadMismatch {
                module: module.identity.clone(),
                index: module.linked_reads.len().min(linked.linked_reads.len()),
            });
        }
        for (index, (actual, expected)) in module.linked_reads.iter().zip(&linked.linked_reads).enumerate() {
            let matches = match (actual, expected) {
                (PortableLinkedReadV1::Module { module: actual }, LinkedReadSpec::Module(expected)) => actual == &to_stable_module(expected),
                (PortableLinkedReadV1::Binding { module, name }, LinkedReadSpec::Binding(expected)) => {
                    module == &to_stable_module(&expected.module) && name == expected.name.as_ref()
                }
                (PortableLinkedReadV1::StaticConstant { module, name }, LinkedReadSpec::Binding(expected)) => {
                    module == &to_stable_module(&expected.module)
                        && name == expected.name.as_ref()
                        && context.has_p7_static_constant(&StableStaticConstantRef {
                            module: module.clone(),
                            name: name.clone().into_boxed_str(),
                        })
                }
                _ => false,
            };
            if !matches {
                return Err(ExecutableValidationError::LinkedReadMismatch {
                    module: module.identity.clone(),
                    index,
                });
            }
        }
        for dependency in &linked.runtime_dependencies {
            let stable = to_stable_module(dependency);
            if !context.has_p7_module(&stable) {
                return Err(ExecutableValidationError::MissingLinkedDependency(stable));
            }
        }
    }
    Ok(())
}

fn validate_bootstrap_dependencies(image: &ExecutableImageV1, context: &ExecutableValidationContext<'_>) -> Result<(), ExecutableValidationError> {
    let eager_positions = image
        .bootstrap
        .eager_modules
        .iter()
        .enumerate()
        .map(|(index, row)| (row.module.clone(), index))
        .collect::<BTreeMap<_, _>>();

    for (index, row) in image.bootstrap.eager_modules.iter().enumerate() {
        let Some(linked) = context.linked_modules.get(&row.module) else {
            return Err(ExecutableValidationError::UnknownP7Module(row.module.clone()));
        };
        for dependency in &linked.runtime_dependencies {
            let dependency = to_stable_module(dependency);
            let Some(dependency_index) = eager_positions.get(&dependency) else {
                return Err(ExecutableValidationError::BootstrapDependencyOrder {
                    module: row.module.clone(),
                    dependency,
                });
            };
            if *dependency_index >= index {
                return Err(ExecutableValidationError::BootstrapDependencyOrder {
                    module: row.module.clone(),
                    dependency,
                });
            }
        }
    }
    Ok(())
}

fn validate_native_anchors(image: &ExecutableImageV1) -> Result<(), ExecutableValidationError> {
    let catalog_fingerprint = phalcom_native_surface::catalog_fingerprint().0;
    let mut expected = crate::native::PRIMITIVES
        .iter()
        .map(|descriptor| expected_native_anchor(descriptor, &catalog_fingerprint))
        .collect::<Result<Vec<_>, _>>()?;
    expected.sort_by(|left, right| left.binding_key.cmp(&right.binding_key));

    if image.native_anchors.len() != expected.len() {
        return Err(ExecutableValidationError::NativeAnchorCount {
            found: image.native_anchors.len(),
            expected: expected.len(),
        });
    }
    for (actual, expected) in image.native_anchors.iter().zip(expected) {
        if actual.binding_key != expected.binding_key {
            return Err(ExecutableValidationError::NativeAnchorMismatch {
                binding_key: actual.binding_key.clone(),
                field: "binding key",
            });
        }
        if actual.owner != expected.owner {
            return Err(ExecutableValidationError::NativeAnchorMismatch {
                binding_key: actual.binding_key.clone(),
                field: "owner",
            });
        }
        if actual.signature != expected.signature {
            return Err(ExecutableValidationError::NativeAnchorMismatch {
                binding_key: actual.binding_key.clone(),
                field: "selector or signature shape",
            });
        }
        if actual.contract_identity != expected.contract_identity {
            return Err(ExecutableValidationError::NativeAnchorMismatch {
                binding_key: actual.binding_key.clone(),
                field: "native ABI contract identity",
            });
        }
    }
    Ok(())
}

fn expected_native_anchor(
    descriptor: &crate::native::PrimitiveDescriptor,
    catalog_fingerprint: &[u8; 16],
) -> Result<PortableNativeAnchorV1, ExecutableValidationError> {
    use phalcom_common::selector::{Selector, SelectorKind, SelectorSlot};
    use phalcom_compiler_artifact::executable::constant::MethodSignatureKindV1;

    let key = descriptor.surface.key;
    let side_name = match key.side {
        phalcom_native_meta::NativeDispatch::Instance => "instance",
        phalcom_native_meta::NativeDispatch::Class => "class",
    };
    let binding_key = format!(
        "universe:{}:{}/{}/{}:{}",
        key.owner.name().len(),
        key.owner.name(),
        side_name,
        key.selector.len(),
        key.selector
    );
    let selector = Selector::try_decode_exact(key.selector).map_err(|error| ExecutableValidationError::InvalidHostNativeDescriptor {
        binding_key: binding_key.clone(),
        reason: error.to_string(),
    })?;
    let has_rest = key.selector.contains("***");
    let fixed_slots = if has_rest {
        &selector.slots[..selector.slots.len().saturating_sub(1)]
    } else {
        &selector.slots
    };
    let fixed_positionals = u8::try_from(fixed_slots.iter().filter(|slot| matches!(slot, SelectorSlot::Positional)).count()).map_err(|_| {
        ExecutableValidationError::InvalidHostNativeDescriptor {
            binding_key: binding_key.clone(),
            reason: "fixed positional count exceeds the executable ABI".into(),
        }
    })?;
    let setter_value = matches!(selector.kind, SelectorKind::Setter | SelectorKind::SubscriptSet);
    let positional_arity = fixed_positionals
        .checked_add(u8::from(setter_value))
        .ok_or_else(|| ExecutableValidationError::InvalidHostNativeDescriptor {
            binding_key: binding_key.clone(),
            reason: "positional arity exceeds the executable ABI".into(),
        })?;
    let slot_count = if selector.kind == SelectorKind::Method && has_rest {
        selector.slots.len().saturating_sub(1)
    } else {
        selector.slots.len()
    };
    let slot_count = u8::try_from(slot_count).map_err(|_| ExecutableValidationError::InvalidHostNativeDescriptor {
        binding_key: binding_key.clone(),
        reason: "selector slot count exceeds the executable ABI".into(),
    })?;
    let kind = match selector.kind {
        SelectorKind::Getter => MethodSignatureKindV1::Getter,
        SelectorKind::Setter => MethodSignatureKindV1::Setter,
        SelectorKind::Method => MethodSignatureKindV1::Method { slots: slot_count },
        SelectorKind::SubscriptGet => MethodSignatureKindV1::SubscriptGet { slots: slot_count },
        SelectorKind::SubscriptSet => MethodSignatureKindV1::SubscriptSet { slots: slot_count },
    };
    let rest = has_rest.then(|| RestLayoutV1 {
        fixed_positionals,
        fixed_labels: fixed_slots
            .iter()
            .filter_map(|slot| match slot {
                SelectorSlot::Label(label) => Some(label.clone()),
                SelectorSlot::Positional => None,
            })
            .collect(),
        mode: RestModeV1::Complete { param_index: 0 },
    });
    let owner = StableClassRefV1 {
        declaration: to_stable_declaration(&phalcom_semantic::core_surface::universe_declaration(key.owner)),
        side: match key.side {
            phalcom_native_meta::NativeDispatch::Instance => StableDispatchSide::Instance,
            phalcom_native_meta::NativeDispatch::Class => StableDispatchSide::Class,
        },
    };
    let abi = match descriptor.abi {
        phalcom_native_meta::PrimitiveAbi::Value => "value",
        phalcom_native_meta::PrimitiveAbi::Shape => "shape",
    };
    let contract_identity = format!("native-surface-catalog-v1:{}:primitive-abi:{abi}", hex_digest(catalog_fingerprint));

    Ok(PortableNativeAnchorV1 {
        binding_key,
        owner,
        signature: MethodSignatureV1 {
            selector: key.selector.to_owned(),
            kind,
            positional_arity,
            rest,
        },
        contract_identity: Some(contract_identity),
    })
}

fn validate_primordial_roles(image: &ExecutableImageV1) -> Result<(), ExecutableValidationError> {
    let mut expected = phalcom_native_meta::UNIVERSE_BINDINGS
        .iter()
        .map(|binding| PrimordialRoleV1 {
            role: format!("universe:{}", binding.key.name()),
            class: StableClassRefV1 {
                declaration: to_stable_declaration(&phalcom_semantic::core_surface::universe_declaration(binding.key)),
                side: StableDispatchSide::Instance,
            },
        })
        .collect::<Vec<_>>();
    expected.sort_by(|left, right| left.role.cmp(&right.role));

    if image.primordial_roles.len() != expected.len() {
        return Err(ExecutableValidationError::PrimordialRoleCount {
            found: image.primordial_roles.len(),
            expected: expected.len(),
        });
    }
    for (actual, expected) in image.primordial_roles.iter().zip(&expected) {
        if actual != expected {
            return Err(ExecutableValidationError::PrimordialRoleMismatch { role: actual.role.clone() });
        }
    }

    let expected_runtime_roles = expected.iter().map(|role| role.role.clone()).collect::<Vec<_>>();
    if image.bootstrap.native_runtime_roles != expected_runtime_roles {
        return Err(ExecutableValidationError::PrimordialRoleMismatch {
            role: "bootstrap native runtime role list".into(),
        });
    }
    for role in &expected {
        let Some(class) = image.classes.iter().find(|row| row.class == role.class) else {
            return Err(ExecutableValidationError::PrimordialClassMissing { role: role.role.clone() });
        };
        if !matches!(&class.allocation, PortableClassAllocationV1::Primordial { role: class_role } if class_role == &role.role) {
            return Err(ExecutableValidationError::PrimordialRoleMismatch { role: role.role.clone() });
        }
    }
    Ok(())
}

fn validate_classes(image: &ExecutableImageV1, context: &ExecutableValidationContext<'_>) -> Result<(), ExecutableValidationError> {
    let class_index = image
        .classes
        .iter()
        .enumerate()
        .map(|(index, row)| ((row.class.declaration.clone(), row.class.side), index))
        .collect::<BTreeMap<_, _>>();

    for row in &image.classes {
        if !context.has_p7_module(&row.class.declaration.module) {
            return Err(ExecutableValidationError::UnknownClassModule(row.class.declaration.module.clone()));
        }
        if !context.has_p7_declaration(&row.class.declaration) {
            return Err(ExecutableValidationError::UnknownP7Declaration(row.class.declaration.clone()));
        }
        if !context.has_p7_class(&row.class) {
            return Err(ExecutableValidationError::UnknownP7Declaration(row.class.declaration.clone()));
        }
        for reference in row.superclass.iter().chain(row.metaclass.iter()) {
            if !class_index.contains_key(&(reference.declaration.clone(), reference.side)) {
                return Err(ExecutableValidationError::MissingClassRecipe(reference.clone()));
            }
        }
        if row.field_count > u32::from(u16::MAX) + 1 {
            return Err(ExecutableValidationError::InvalidClassLayout {
                class: row.class.clone(),
                detail: "field count exceeds the portable field-slot address space".to_owned(),
            });
        }
        let mut field_slots = BTreeSet::new();
        for field in &row.fields {
            if field.slot >= row.field_count || !field_slots.insert(field.slot) {
                return Err(ExecutableValidationError::InvalidClassLayout {
                    class: row.class.clone(),
                    detail: format!("field {} has a duplicate or out-of-range slot {}", field.name, field.slot),
                });
            }
        }
        let mut immutable_slots = BTreeSet::new();
        for slot in &row.immutable_slots {
            if *slot >= row.field_count || !immutable_slots.insert(*slot) {
                return Err(ExecutableValidationError::InvalidClassLayout {
                    class: row.class.clone(),
                    detail: format!("immutable slot {slot} is duplicated or out of range"),
                });
            }
        }
        for field in context
            .fields
            .iter()
            .filter(|field| field.owner == row.class.declaration && field.side == row.class.side)
        {
            if !row.fields.iter().any(|layout| layout.name == field.name.as_ref()) {
                return Err(ExecutableValidationError::InvalidClassLayout {
                    class: row.class.clone(),
                    detail: format!("P7 field {} is absent from the runtime layout", field.name),
                });
            }
        }
    }

    let mut state = vec![0u8; image.classes.len()];
    for start in 0..image.classes.len() {
        if state[start] != 0 {
            continue;
        }
        let mut path = Vec::new();
        let mut current = Some(start);
        while let Some(index) = current {
            match state[index] {
                1 => return Err(ExecutableValidationError::SuperclassCycle(image.classes[index].class.clone())),
                2 => break,
                _ => {
                    state[index] = 1;
                    path.push(index);
                    current = image.classes[index]
                        .superclass
                        .as_ref()
                        .and_then(|superclass| class_index.get(&(superclass.declaration.clone(), superclass.side)).copied());
                }
            }
        }
        for index in path {
            state[index] = 2;
        }
    }
    Ok(())
}

fn validate_prototype_owners(image: &ExecutableImageV1, context: &ExecutableValidationContext<'_>) -> Result<(), ExecutableValidationError> {
    let mut initializer_owners = BTreeSet::new();
    let mut callable_owners = BTreeSet::new();
    let mut nested_owners = BTreeSet::new();
    for (index, prototype) in image.prototypes.iter().enumerate() {
        match &prototype.owner {
            PrototypeOwnerV1::ModuleInitializer { module } => {
                if !context.has_p7_module(module) {
                    return Err(ExecutableValidationError::UnknownP7Module(module.clone()));
                }
                if !initializer_owners.insert(module.clone()) {
                    return Err(ExecutableValidationError::InvalidExecutableReference {
                        table: "module initializer prototypes",
                        index,
                        detail: format!("duplicate initializer for {module:?}"),
                    });
                }
            }
            PrototypeOwnerV1::Callable { callable } => {
                let module = callable_module(callable);
                if !context.has_p7_module(module) {
                    return Err(ExecutableValidationError::UnknownCallableModule(module.clone()));
                }
                if !context.has_p7_callable(callable) {
                    return Err(ExecutableValidationError::UnknownP7Callable(callable.clone()));
                }
                if !callable_owners.insert(callable.clone()) {
                    return Err(ExecutableValidationError::InvalidExecutableReference {
                        table: "callable prototypes",
                        index,
                        detail: format!("duplicate prototype for {callable:?}"),
                    });
                }
            }
            PrototypeOwnerV1::Nested { parent, ordinal } => {
                let parent_index = parent.0 as usize;
                if parent_index >= index {
                    return Err(ExecutableValidationError::InvalidExecutableReference {
                        table: "nested prototype owners",
                        index,
                        detail: format!("parent prototype {} is not earlier than its child", parent.0),
                    });
                }
                if !nested_owners.insert((parent.0, *ordinal)) {
                    return Err(ExecutableValidationError::InvalidExecutableReference {
                        table: "nested prototype owners",
                        index,
                        detail: format!("duplicate child ordinal {ordinal} for parent {}", parent.0),
                    });
                }
                let parent = &image.prototypes[parent_index];
                let expected_module = prototype_module(image, parent_index)?;
                let actual_module = prototype_module(image, index)?;
                if expected_module != actual_module {
                    return Err(ExecutableValidationError::InvalidExecutableReference {
                        table: "nested prototype owners",
                        index,
                        detail: "nested prototype changed its canonical module owner".to_owned(),
                    });
                }
                for (capture_index, capture) in prototype.upvalues.iter().enumerate() {
                    let valid = match capture {
                        UpvalueCaptureV1::Local(slot) => *slot < parent.max_slots,
                        UpvalueCaptureV1::Upvalue(slot) => (*slot as usize) < parent.upvalues.len(),
                    };
                    if !valid {
                        return Err(ExecutableValidationError::InvalidClosureCapture {
                            child: index,
                            parent: parent_index,
                            detail: format!("capture {capture_index} refers to an absent parent slot"),
                        });
                    }
                }
            }
        }
    }
    Ok(())
}

fn validate_prototypes(image: &ExecutableImageV1, context: &ExecutableValidationContext<'_>) -> Result<(), ExecutableValidationError> {
    validate_closure_template_constants(image, context)?;
    for (prototype_index, prototype) in image.prototypes.iter().enumerate() {
        let fixed_label_count = prototype.parameters.fixed_labels.len();
        let mut labels = BTreeSet::new();
        if prototype.parameters.fixed_labels.iter().any(|label| label.is_empty() || !labels.insert(label)) {
            return Err(ExecutableValidationError::InvalidPrototypeLayout {
                prototype: prototype_index,
                detail: "parameter labels must be nonempty and unique".to_owned(),
            });
        }
        let rest_slots = match prototype.parameters.rest {
            Some(RestKindV1::Positional | RestKindV1::Labeled | RestKindV1::Complete) => 1,
            Some(RestKindV1::Split) => 1,
            None => 0,
        };
        let fixed_slots = (prototype.parameters.fixed_positionals as usize)
            .checked_add(fixed_label_count)
            .and_then(|slots| slots.checked_add(rest_slots));
        if fixed_slots.is_none_or(|slots| slots > prototype.max_slots as usize) {
            return Err(ExecutableValidationError::InvalidPrototypeLayout {
                prototype: prototype_index,
                detail: "frame is smaller than its declared parameter slots".to_owned(),
            });
        }

        validate_prototype_operands(image, context, prototype_index, prototype)?;
        validate_prototype_control_flow(image, context, prototype_index, prototype)?;
    }
    Ok(())
}

fn validate_closure_template_constants(image: &ExecutableImageV1, context: &ExecutableValidationContext<'_>) -> Result<(), ExecutableValidationError> {
    for (constant_index, constant) in image.constants.iter().enumerate() {
        let ConstantRecipeV1::ClosureTemplate {
            prototype: target,
            module,
            lexical_class,
            super_anchor,
        } = constant
        else {
            continue;
        };
        let target_index = target.0 as usize;
        if target_index >= image.prototypes.len() {
            return Err(ExecutableValidationError::InvalidExecutableReference {
                table: "closure-template constants",
                index: constant_index,
                detail: format!("target prototype {} is absent", target.0),
            });
        }
        if !context.has_p7_module(module) || prototype_module(image, target_index)? != module {
            return Err(ExecutableValidationError::InvalidExecutableReference {
                table: "closure-template constants",
                index: constant_index,
                detail: "closure template module does not match its P7-owned prototype".to_owned(),
            });
        }
        for class in lexical_class.iter().chain(super_anchor.iter()) {
            if !context.has_p7_class(class)
                || !image
                    .classes
                    .iter()
                    .any(|row| row.class.declaration == class.declaration && row.class.side == class.side)
            {
                return Err(ExecutableValidationError::InvalidExecutableReference {
                    table: "closure-template class anchors",
                    index: constant_index,
                    detail: format!("class anchor {class:?} is not present in P7 and the executable image"),
                });
            }
        }
    }
    Ok(())
}

fn validate_prototype_operands(
    image: &ExecutableImageV1,
    context: &ExecutableValidationContext<'_>,
    prototype_index: usize,
    prototype: &PortablePrototypeV1,
) -> Result<(), ExecutableValidationError> {
    for (instruction_index, instruction) in prototype.code.iter().enumerate() {
        let mut invalid = None;
        instruction.visit_operands(|domain, value| {
            if invalid.is_none() {
                invalid = validate_instruction_operand(image, context, prototype_index, prototype, instruction_index, domain, value).err();
            }
        });
        if let Some(error) = invalid {
            return Err(error);
        }
    }
    Ok(())
}

fn validate_instruction_operand(
    image: &ExecutableImageV1,
    context: &ExecutableValidationContext<'_>,
    prototype_index: usize,
    prototype: &PortablePrototypeV1,
    instruction_index: usize,
    domain: OperandDomainV1,
    value: PortableOperandValueV1,
) -> Result<(), ExecutableValidationError> {
    let invalid = |detail: String| ExecutableValidationError::InvalidInstruction {
        prototype: prototype_index,
        instruction: instruction_index,
        operand: operand_domain_name(domain),
        detail,
    };
    match domain {
        OperandDomainV1::Constant
        | OperandDomainV1::SymbolConstant
        | OperandDomainV1::ClosureTemplateConstant
        | OperandDomainV1::MethodConstant
        | OperandDomainV1::ModuleConstant
        | OperandDomainV1::SelectorSpecification => {
            let local_index = unsigned_operand(value).and_then(|value| usize::try_from(value).ok());
            let Some(local_index) = local_index else {
                return Err(invalid("operand is not an unsigned constant-pool index".to_owned()));
            };
            let Some(constant_id) = prototype.constants.get(local_index) else {
                return Err(invalid(format!("local constant index {local_index} is absent")));
            };
            let Some(constant) = image.constants.get(constant_id.0 as usize) else {
                return Err(invalid(format!("constant recipe {} is absent", constant_id.0)));
            };
            let domain_matches = match (domain, constant) {
                (OperandDomainV1::SymbolConstant, ConstantRecipeV1::Symbol { .. }) => true,
                (OperandDomainV1::ClosureTemplateConstant, ConstantRecipeV1::ClosureTemplate { .. }) => true,
                (OperandDomainV1::MethodConstant, ConstantRecipeV1::Method { .. }) => true,
                (OperandDomainV1::ModuleConstant, ConstantRecipeV1::Module { module }) => context.has_p7_module(module),
                (OperandDomainV1::SelectorSpecification, ConstantRecipeV1::Symbol { .. } | ConstantRecipeV1::SelectorPattern { .. }) => true,
                (OperandDomainV1::Constant, _) => true,
                _ => false,
            };
            if !domain_matches {
                return Err(invalid(format!("constant recipe {constant:?} does not satisfy its declared domain")));
            }
        }
        OperandDomainV1::LinkedRead => {
            let Some(index) = unsigned_operand(value).and_then(|value| usize::try_from(value).ok()) else {
                return Err(invalid("operand is not an unsigned linked-read index".to_owned()));
            };
            let module = prototype_module(image, prototype_index)?;
            let Some(module) = image.modules.iter().find(|row| row.identity == *module) else {
                return Err(invalid(format!("prototype module {module:?} has no executable row")));
            };
            if index >= module.linked_reads.len() {
                return Err(invalid(format!("linked-read index {index} is absent from the owning module")));
            }
        }
        OperandDomainV1::Local => {
            let Some(slot) = unsigned_operand(value) else {
                return Err(invalid("operand is not an unsigned frame-slot index".to_owned()));
            };
            if slot >= prototype.max_slots {
                return Err(invalid(format!("frame slot {slot} exceeds max_slots {}", prototype.max_slots)));
            }
        }
        OperandDomainV1::Upvalue => {
            let Some(slot) = unsigned_operand(value) else {
                return Err(invalid("operand is not an unsigned upvalue index".to_owned()));
            };
            if slot as usize >= prototype.upvalues.len() {
                return Err(invalid(format!("upvalue slot {slot} exceeds {} captures", prototype.upvalues.len())));
            }
        }
        OperandDomainV1::Field => {
            let Some(slot) = unsigned_operand(value).and_then(|slot| u16::try_from(slot).ok()) else {
                return Err(invalid("operand is not a portable field-slot index".to_owned()));
            };
            let Some(class) = prototype_class(image, prototype_index)? else {
                return Err(ExecutableValidationError::UnknownFieldLayout {
                    prototype: prototype_index,
                    slot,
                });
            };
            let Some(layout) = image.classes.iter().find(|row| row.class == class) else {
                return Err(ExecutableValidationError::UnknownFieldLayout {
                    prototype: prototype_index,
                    slot,
                });
            };
            if u32::from(slot) >= layout.field_count {
                return Err(invalid(format!("field slot {slot} exceeds class field count {}", layout.field_count)));
            }
        }
        OperandDomainV1::RelativeJump => {
            let PortableOperandValueV1::Signed(offset) = value else {
                return Err(invalid("operand is not a signed relative branch offset".to_owned()));
            };
            validate_branch_target(prototype_index, instruction_index, prototype.code.len(), offset)?;
        }
        OperandDomainV1::AnonymousProduct
        | OperandDomainV1::AssociatedTarget
        | OperandDomainV1::CallEnvironment
        | OperandDomainV1::ConditionalFamily
        | OperandDomainV1::DataConstruction
        | OperandDomainV1::DataDeclaration
        | OperandDomainV1::DataTarget
        | OperandDomainV1::EnumDeclaration
        | OperandDomainV1::FamilyCandidates
        | OperandDomainV1::FamilyDescriptor
        | OperandDomainV1::FamilyOperation
        | OperandDomainV1::TraitInvocation
        | OperandDomainV1::TraitRequirementInvocation
        | OperandDomainV1::TypeParameterReceiver
        | OperandDomainV1::VariantTarget => {
            let Some(index) = unsigned_operand(value).and_then(|value| usize::try_from(value).ok()) else {
                return Err(invalid("operand is not an unsigned semantic-pool index".to_owned()));
            };
            let Some(pool) = image.semantic_pools.get(prototype.semantics.0 as usize) else {
                return Err(invalid(format!("semantic pool {} is absent", prototype.semantics.0)));
            };
            let table_len = semantic_operand_table_len(pool, domain);
            if index >= table_len {
                return Err(invalid(format!("semantic-pool index {index} exceeds table length {table_len}")));
            }
        }
        OperandDomainV1::Component => {
            // This operand is a runtime logical component ordinal, not a
            // semantic-pool entry. Its VM bounds depend on the selected value's
            // runtime shape, so it is intentionally not remapped as a pool ID.
            if unsigned_operand(value).is_none() {
                return Err(invalid("component ordinal is not unsigned".to_owned()));
            }
        }
        OperandDomainV1::Access | OperandDomainV1::FamilyKind | OperandDomainV1::SendKind => {
            if !matches!(value, PortableOperandValueV1::Tag(_)) {
                return Err(invalid("operand is not a validated portable tag".to_owned()));
            }
        }
        OperandDomainV1::Flag => {
            if !matches!(value, PortableOperandValueV1::Boolean(_)) {
                return Err(invalid("operand is not a canonical boolean".to_owned()));
            }
        }
        OperandDomainV1::Arity | OperandDomainV1::Count => {
            if unsigned_operand(value).is_none() {
                return Err(invalid("operand is not an unsigned arity/count".to_owned()));
            }
        }
    }
    Ok(())
}

fn validate_prototype_control_flow(
    _image: &ExecutableImageV1,
    _context: &ExecutableValidationContext<'_>,
    prototype_index: usize,
    prototype: &PortablePrototypeV1,
) -> Result<(), ExecutableValidationError> {
    let code_len = prototype.code.len();
    let mut reachable = vec![false; code_len];
    let mut pending = vec![0usize];
    while let Some(instruction_index) = pending.pop() {
        if instruction_index >= code_len {
            return Err(ExecutableValidationError::InvalidBranchTarget {
                prototype: prototype_index,
                instruction: instruction_index.saturating_sub(1),
                target: i64::try_from(instruction_index).unwrap_or(i64::MAX),
            });
        }
        if reachable[instruction_index] {
            continue;
        }
        reachable[instruction_index] = true;
        let instruction = &prototype.code[instruction_index];
        let Some(opcode) = OPCODE_MANIFEST_V1.iter().find(|row| row.tag == instruction.opcode_tag()) else {
            return Err(ExecutableValidationError::UnknownOpcodeManifestTag {
                prototype: prototype_index,
                instruction: instruction_index,
                tag: instruction.opcode_tag(),
            });
        };
        let mut target = None;
        instruction.visit_operands(|domain, value| {
            if domain == OperandDomainV1::RelativeJump {
                if let PortableOperandValueV1::Signed(offset) = value {
                    target = branch_target(instruction_index, offset);
                }
            }
        });
        match opcode.name {
            "Return" | "ReturnNonLocal" => {}
            "Jump" | "Loop" => {
                let target = target.ok_or(ExecutableValidationError::InvalidBranchTarget {
                    prototype: prototype_index,
                    instruction: instruction_index,
                    target: i64::MAX,
                })?;
                if target >= code_len {
                    return Err(ExecutableValidationError::InvalidBranchTarget {
                        prototype: prototype_index,
                        instruction: instruction_index,
                        target: target as i64,
                    });
                }
                pending.push(target);
            }
            "JumpIfFalse" | "JumpIfNone" | "GuardBool" | "GuardBlock" | "JumpIfUnsupported" | "TryInvokeExact" => {
                let target = target.ok_or(ExecutableValidationError::InvalidBranchTarget {
                    prototype: prototype_index,
                    instruction: instruction_index,
                    target: i64::MAX,
                })?;
                if target >= code_len {
                    return Err(ExecutableValidationError::InvalidBranchTarget {
                        prototype: prototype_index,
                        instruction: instruction_index,
                        target: target as i64,
                    });
                }
                pending.push(target);
                pending.push(instruction_index + 1);
            }
            _ => pending.push(instruction_index + 1),
        }
    }
    Ok(())
}

fn validate_branch_target(prototype: usize, instruction: usize, code_len: usize, offset: i32) -> Result<(), ExecutableValidationError> {
    let Some(target) = branch_target(instruction, offset) else {
        return Err(ExecutableValidationError::InvalidBranchTarget {
            prototype,
            instruction,
            target: instruction as i64 + 1 + i64::from(offset),
        });
    };
    if target >= code_len {
        return Err(ExecutableValidationError::InvalidBranchTarget {
            prototype,
            instruction,
            target: target as i64,
        });
    }
    Ok(())
}

fn branch_target(instruction: usize, offset: i32) -> Option<usize> {
    let target = i64::try_from(instruction).ok()?.checked_add(1)?.checked_add(i64::from(offset))?;
    usize::try_from(target).ok()
}

fn semantic_operand_table_len(pool: &phalcom_compiler_artifact::executable::semantic::PortableSemanticPoolV1, domain: OperandDomainV1) -> usize {
    match domain {
        OperandDomainV1::AnonymousProduct => pool.anonymous_products.len(),
        OperandDomainV1::AssociatedTarget => pool.associated_targets.len(),
        OperandDomainV1::CallEnvironment => pool.call_environments.len(),
        OperandDomainV1::ConditionalFamily => pool.conditional_families.len(),
        OperandDomainV1::DataConstruction => pool.data_constructions.len(),
        OperandDomainV1::DataDeclaration => pool.data_specs.len(),
        OperandDomainV1::DataTarget => pool.data_targets.len(),
        OperandDomainV1::EnumDeclaration => pool.enum_specs.len(),
        OperandDomainV1::FamilyCandidates => pool.family_candidate_sets.len(),
        OperandDomainV1::FamilyDescriptor => pool.family_descriptors.len(),
        OperandDomainV1::FamilyOperation => pool.family_operations.len(),
        OperandDomainV1::TraitInvocation => pool.trait_invocations.len(),
        OperandDomainV1::TraitRequirementInvocation => pool.trait_requirement_invocations.len(),
        OperandDomainV1::TypeParameterReceiver => pool.type_parameter_class_receivers.len(),
        OperandDomainV1::VariantTarget => pool.variant_targets.len(),
        OperandDomainV1::Access
        | OperandDomainV1::Arity
        | OperandDomainV1::Component
        | OperandDomainV1::Constant
        | OperandDomainV1::Count
        | OperandDomainV1::FamilyKind
        | OperandDomainV1::Field
        | OperandDomainV1::Flag
        | OperandDomainV1::LinkedRead
        | OperandDomainV1::Local
        | OperandDomainV1::MethodConstant
        | OperandDomainV1::ModuleConstant
        | OperandDomainV1::RelativeJump
        | OperandDomainV1::SelectorSpecification
        | OperandDomainV1::SendKind
        | OperandDomainV1::SymbolConstant
        | OperandDomainV1::Upvalue
        | OperandDomainV1::ClosureTemplateConstant => 0,
    }
}

fn operand_domain_name(domain: OperandDomainV1) -> &'static str {
    match domain {
        OperandDomainV1::Access => "access operand",
        OperandDomainV1::AnonymousProduct => "anonymous-product reference",
        OperandDomainV1::Arity => "arity operand",
        OperandDomainV1::AssociatedTarget => "associated-target reference",
        OperandDomainV1::CallEnvironment => "call-environment reference",
        OperandDomainV1::ClosureTemplateConstant => "closure-template constant",
        OperandDomainV1::Component => "component ordinal",
        OperandDomainV1::ConditionalFamily => "conditional-family reference",
        OperandDomainV1::Constant => "constant-pool reference",
        OperandDomainV1::Count => "count operand",
        OperandDomainV1::DataConstruction => "data-construction reference",
        OperandDomainV1::DataDeclaration => "data-declaration reference",
        OperandDomainV1::DataTarget => "data-target reference",
        OperandDomainV1::EnumDeclaration => "enum-declaration reference",
        OperandDomainV1::FamilyCandidates => "family-candidate reference",
        OperandDomainV1::FamilyDescriptor => "family-descriptor reference",
        OperandDomainV1::FamilyKind => "family-kind operand",
        OperandDomainV1::FamilyOperation => "family-operation reference",
        OperandDomainV1::Field => "field-slot operand",
        OperandDomainV1::Flag => "flag operand",
        OperandDomainV1::LinkedRead => "linked-read reference",
        OperandDomainV1::Local => "frame-slot operand",
        OperandDomainV1::MethodConstant => "method constant",
        OperandDomainV1::ModuleConstant => "module constant",
        OperandDomainV1::RelativeJump => "relative branch operand",
        OperandDomainV1::SelectorSpecification => "selector specification",
        OperandDomainV1::SendKind => "send-kind operand",
        OperandDomainV1::SymbolConstant => "symbol constant",
        OperandDomainV1::TraitInvocation => "trait invocation reference",
        OperandDomainV1::TraitRequirementInvocation => "trait-requirement invocation reference",
        OperandDomainV1::TypeParameterReceiver => "type-parameter receiver reference",
        OperandDomainV1::Upvalue => "upvalue operand",
        OperandDomainV1::VariantTarget => "variant target reference",
    }
}

fn unsigned_operand(value: PortableOperandValueV1) -> Option<u32> {
    match value {
        PortableOperandValueV1::Unsigned(value) => Some(value),
        PortableOperandValueV1::Signed(_) | PortableOperandValueV1::Boolean(_) | PortableOperandValueV1::Tag(_) => None,
    }
}

fn prototype_module(image: &ExecutableImageV1, index: usize) -> Result<&StableModuleRef, ExecutableValidationError> {
    let Some(prototype) = image.prototypes.get(index) else {
        return Err(ExecutableValidationError::InvalidExecutableReference {
            table: "prototypes",
            index,
            detail: "prototype index is absent".to_owned(),
        });
    };
    match &prototype.owner {
        PrototypeOwnerV1::ModuleInitializer { module } => Ok(module),
        PrototypeOwnerV1::Callable { callable } => Ok(callable_module(callable)),
        PrototypeOwnerV1::Nested { parent, .. } => prototype_module(image, parent.0 as usize),
    }
}

fn prototype_class(image: &ExecutableImageV1, index: usize) -> Result<Option<StableClassRefV1>, ExecutableValidationError> {
    let Some(prototype) = image.prototypes.get(index) else {
        return Err(ExecutableValidationError::InvalidExecutableReference {
            table: "prototypes",
            index,
            detail: "prototype index is absent".to_owned(),
        });
    };
    match &prototype.owner {
        PrototypeOwnerV1::ModuleInitializer { .. } => Ok(None),
        PrototypeOwnerV1::Callable { callable } => {
            let declaration = match &callable.owner {
                StableCallableOwnerRef::Declaration(declaration) => Some(declaration.clone()),
                StableCallableOwnerRef::Variant(variant) => Some(variant.owner.clone()),
                StableCallableOwnerRef::Conformance(_) => None,
            };
            Ok(declaration.map(|declaration| StableClassRefV1 {
                declaration,
                side: callable.side,
            }))
        }
        PrototypeOwnerV1::Nested { parent, .. } => prototype_class(image, parent.0 as usize),
    }
}

fn validate_required_declarations(image: &ExecutableImageV1, context: &ExecutableValidationContext<'_>) -> Result<(), ExecutableValidationError> {
    for declaration in &image.bootstrap.required_declarations {
        if !context.has_p7_module(&declaration.module) || !context.has_p7_declaration(declaration) {
            return Err(ExecutableValidationError::UnknownRequiredDeclaration(declaration.clone()));
        }
    }
    Ok(())
}

fn validate_canonical_semantic_roots(image: &ExecutableImageV1, context: &ExecutableValidationContext<'_>) -> Result<(), ExecutableValidationError> {
    let class_index = image
        .classes
        .iter()
        .map(|row| ((row.class.declaration.clone(), row.class.side), ()))
        .collect::<BTreeMap<_, _>>();

    for (pool_index, pool) in image.semantic_pools.iter().enumerate() {
        for (recipe_index, recipe) in pool.type_recipes.iter().enumerate() {
            match recipe {
                PortableTypeRecipeV1::Canonical { digest } if !context.canonical_type_digests.contains(digest) => {
                    return Err(ExecutableValidationError::UnknownCanonicalType {
                        pool: pool_index,
                        recipe: recipe_index,
                    });
                }
                PortableTypeRecipeV1::Nominal { class } | PortableTypeRecipeV1::SelfType { class: Some(class) } => {
                    if !class_index.contains_key(&(class.declaration.clone(), class.side)) {
                        return Err(ExecutableValidationError::UnknownSemanticClass {
                            pool: pool_index,
                            recipe: recipe_index,
                        });
                    }
                    if !context.has_p7_module(&class.declaration.module) || !context.has_p7_declaration(&class.declaration) {
                        return Err(ExecutableValidationError::UnknownSemanticDeclaration {
                            pool: pool_index,
                            recipe: recipe_index,
                        });
                    }
                }
                PortableTypeRecipeV1::Parameter { parameter } => match &parameter.owner {
                    PortableTypeParameterOwnerV1::Declaration { owner } => {
                        if !context.has_p7_module(&owner.module) {
                            return Err(ExecutableValidationError::UnknownSemanticModule {
                                pool: pool_index,
                                recipe: recipe_index,
                            });
                        }
                        if !context.has_p7_declaration(owner) {
                            return Err(ExecutableValidationError::UnknownSemanticDeclaration {
                                pool: pool_index,
                                recipe: recipe_index,
                            });
                        }
                    }
                    PortableTypeParameterOwnerV1::Callable { owner } => {
                        if !context.has_p7_module(callable_module(owner)) {
                            return Err(ExecutableValidationError::UnknownSemanticModule {
                                pool: pool_index,
                                recipe: recipe_index,
                            });
                        }
                        if !context.has_p7_callable(owner) {
                            return Err(ExecutableValidationError::UnknownSemanticCallable {
                                pool: pool_index,
                                recipe: recipe_index,
                            });
                        }
                    }
                    PortableTypeParameterOwnerV1::Implementation { owner } => {
                        if !context.has_p7_module(&owner.module) {
                            return Err(ExecutableValidationError::UnknownSemanticModule {
                                pool: pool_index,
                                recipe: recipe_index,
                            });
                        }
                    }
                },
                _ => {}
            }
        }
        for (recipe_index, recipe) in pool.kind_recipes.iter().enumerate() {
            if let PortableKindRecipeV1::Canonical { digest } = recipe
                && !context.canonical_kind_digests.contains(digest)
            {
                return Err(ExecutableValidationError::UnknownCanonicalKind {
                    pool: pool_index,
                    recipe: recipe_index,
                });
            }
        }
    }
    Ok(())
}

fn validate_semantic_pools(image: &ExecutableImageV1, context: &ExecutableValidationContext<'_>) -> Result<(), ExecutableValidationError> {
    for (pool_index, pool) in image.semantic_pools.iter().enumerate() {
        for (row, recipe) in pool.type_recipes.iter().enumerate() {
            if let PortableTypeRecipeV1::Parameter { parameter } = recipe {
                validate_type_parameter_identity(context, pool_index, "type recipe", row, parameter)?;
            }
        }

        for (row, spec) in pool.enum_specs.iter().enumerate() {
            let Some(expected_variants) = context.enum_variants.get(&spec.owner) else {
                return Err(ExecutableValidationError::UnknownSemanticIdentity {
                    pool: pool_index,
                    family: "enum specification owner",
                    row,
                });
            };
            if spec.variants.len() != expected_variants.len() {
                return Err(ExecutableValidationError::InvalidSemanticProduct {
                    pool: pool_index,
                    family: "enum specification",
                    row,
                    detail: format!("contains {} variants but P7 declares {}", spec.variants.len(), expected_variants.len()),
                });
            }
            for (variant_index, (variant, expected)) in spec.variants.iter().zip(expected_variants).enumerate() {
                if &variant.variant != expected {
                    return Err(ExecutableValidationError::InvalidSemanticProduct {
                        pool: pool_index,
                        family: "enum specification",
                        row,
                        detail: format!("variant {variant_index} differs from P7 declaration order"),
                    });
                }
                let expected_fields = context.variant_field_counts.get(expected).copied().unwrap_or_default();
                if variant.fields.len() != expected_fields {
                    return Err(ExecutableValidationError::InvalidSemanticProduct {
                        pool: pool_index,
                        family: "variant specification",
                        row,
                        detail: format!("variant {expected:?} has the wrong field count"),
                    });
                }
                let mut slots = BTreeSet::new();
                for (field_index, field) in variant.fields.iter().enumerate() {
                    let stable = StableVariantFieldRef {
                        variant: expected.clone(),
                        index: field.index,
                    };
                    if field.index as usize != field_index || !context.has_p7_variant_field(&stable) {
                        return Err(ExecutableValidationError::UnknownSemanticIdentity {
                            pool: pool_index,
                            family: "variant field",
                            row,
                        });
                    }
                    if context.variant_field_names.get(&stable).is_none_or(|name| name != &field.local_name) {
                        return Err(ExecutableValidationError::InvalidSemanticProduct {
                            pool: pool_index,
                            family: "variant field",
                            row,
                            detail: format!("field {field_index} name does not match P7"),
                        });
                    }
                    if !slots.insert(field.slot) {
                        return Err(ExecutableValidationError::InvalidSemanticProduct {
                            pool: pool_index,
                            family: "variant field",
                            row,
                            detail: format!("variant {expected:?} assigns physical slot {} more than once", field.slot),
                        });
                    }
                }
            }
        }

        for (row, variant) in pool.variant_targets.iter().enumerate() {
            require_p7_variant(context, pool_index, "variant target", row, variant)?;
        }

        for (row, spec) in pool.data_specs.iter().enumerate() {
            if !context.data_declarations.contains(&spec.owner) {
                return Err(ExecutableValidationError::UnknownSemanticIdentity {
                    pool: pool_index,
                    family: "data specification owner",
                    row,
                });
            }
            let expected_components = context
                .data_components
                .iter()
                .filter(|component| component.owner == spec.owner)
                .collect::<Vec<_>>();
            if spec.components.len() != expected_components.len() {
                return Err(ExecutableValidationError::InvalidSemanticProduct {
                    pool: pool_index,
                    family: "data specification",
                    row,
                    detail: format!("contains {} components but P7 declares {}", spec.components.len(), expected_components.len()),
                });
            }
            for component in &spec.components {
                let stable = StableDataComponentRef {
                    owner: spec.owner.clone(),
                    index: component.index,
                };
                if !context.has_p7_data_component(&stable) {
                    return Err(ExecutableValidationError::UnknownSemanticIdentity {
                        pool: pool_index,
                        family: "data component",
                        row,
                    });
                }
                if context.data_component_names.get(&stable).is_none_or(|name| name != &component.local_name) {
                    return Err(ExecutableValidationError::InvalidSemanticProduct {
                        pool: pool_index,
                        family: "data component",
                        row,
                        detail: format!("component {} name does not match P7", component.index),
                    });
                }
            }
        }

        for (row, target) in pool.data_targets.iter().enumerate() {
            if !context.data_declarations.contains(target) {
                return Err(ExecutableValidationError::UnknownSemanticIdentity {
                    pool: pool_index,
                    family: "data target",
                    row,
                });
            }
        }
        for (row, construction) in pool.data_constructions.iter().enumerate() {
            if !context.data_declarations.contains(&construction.constructor) {
                return Err(ExecutableValidationError::UnknownSemanticIdentity {
                    pool: pool_index,
                    family: "data construction",
                    row,
                });
            }
            validate_runtime_type_recipe(pool_index, "data construction", row, pool, &construction.type_recipe)?;
        }
        for (row, product) in pool.anonymous_products.iter().enumerate() {
            validate_runtime_type_recipe(pool_index, "anonymous product", row, pool, &product.type_recipe)?;
        }

        for (row, environment) in pool.call_environments.iter().enumerate() {
            require_p7_callable(context, pool_index, "call environment", row, &environment.callable)?;
            for binding in &environment.bindings {
                validate_type_parameter_identity(context, pool_index, "call-environment binding", row, &binding.parameter)?;
            }
            validate_conformance_slots(pool_index, row, pool, context, &environment.conformance_slots)?;
        }

        for (row, target) in pool.associated_targets.iter().enumerate() {
            validate_invocation_target(pool_index, "associated target", row, pool, context, target)?;
        }
        for (row, descriptor) in pool.family_descriptors.iter().enumerate() {
            for entry in &descriptor.entries {
                validate_family_target(pool_index, "family descriptor", row, pool, context, &entry.target)?;
            }
        }
        for (row, candidates) in pool.family_candidate_sets.iter().enumerate() {
            for candidate in &candidates.candidates {
                if let Some(target) = &candidate.target {
                    validate_invocation_target(pool_index, "family candidate", row, pool, context, target)?;
                }
            }
        }
        for (row, family) in pool.conditional_families.iter().enumerate() {
            for entry in &family.entries {
                require_p7_declaration(context, pool_index, "conditional-family owner", row, &entry.declaring_owner)?;
                if !matches!(image.constants.get(entry.fallback_method.0 as usize), Some(ConstantRecipeV1::Method { .. })) {
                    return Err(ExecutableValidationError::InvalidSemanticProduct {
                        pool: pool_index,
                        family: "conditional family",
                        row,
                        detail: "fallback does not reference a method recipe".to_owned(),
                    });
                }
                validate_call_environment_index(pool_index, row, pool, entry.call_environment.map(|id| id.0), "conditional family")?;
            }
        }
        for (row, invocation) in pool.trait_invocations.iter().enumerate() {
            if !context.has_p7_implementation(&invocation.source_impl) {
                return Err(ExecutableValidationError::UnknownSemanticIdentity {
                    pool: pool_index,
                    family: "trait invocation implementation",
                    row,
                });
            }
            validate_trait_selection(pool_index, "trait invocation selection", row, context, &invocation.selection)?;
            validate_trait_requirement(pool_index, "trait invocation requirement", row, context, &invocation.requirement)?;
            let mut requirement_slots = BTreeSet::new();
            for slot in &invocation.requirement_slots {
                validate_trait_requirement(pool_index, "trait requirement slot", row, context, &slot.requirement)?;
                if !requirement_slots.insert(slot.slot) {
                    return Err(ExecutableValidationError::InvalidSemanticProduct {
                        pool: pool_index,
                        family: "trait invocation",
                        row,
                        detail: format!("requirement slot {} is duplicated", slot.slot),
                    });
                }
            }
            if !invocation
                .requirement_slots
                .iter()
                .any(|slot| slot.slot == invocation.requirement_slot && slot.requirement == invocation.requirement)
            {
                return Err(ExecutableValidationError::InvalidSemanticProduct {
                    pool: pool_index,
                    family: "trait invocation",
                    row,
                    detail: "selected requirement slot does not identify the invocation requirement".to_owned(),
                });
            }
            for target in &invocation.requirement_targets {
                validate_trait_requirement(pool_index, "trait requirement target", row, context, &target.requirement)?;
                validate_trait_selection(pool_index, "trait requirement target", row, context, &target.target)?;
            }
            for antecedent in &invocation.antecedent_slots {
                validate_trait_selection(pool_index, "trait antecedent", row, context, &antecedent.target)?;
            }
            for binding in &invocation.type_bindings {
                validate_type_parameter_identity(context, pool_index, "trait invocation binding", row, &binding.parameter)?;
            }
            validate_call_environment_index(pool_index, row, pool, invocation.call_environment.map(|id| id.0), "trait invocation")?;
        }
        for (row, invocation) in pool.trait_requirement_invocations.iter().enumerate() {
            validate_trait_requirement(pool_index, "trait requirement invocation", row, context, &invocation.requirement)?;
        }
        for (row, parameter) in pool.type_parameter_class_receivers.iter().enumerate() {
            validate_type_parameter_identity(context, pool_index, "type-parameter class receiver", row, parameter)?;
        }
    }
    Ok(())
}

fn validate_type_parameter_identity(
    context: &ExecutableValidationContext<'_>,
    pool: usize,
    family: &'static str,
    row: usize,
    parameter: &PortableTypeParameterRefV1,
) -> Result<(), ExecutableValidationError> {
    let owner = match &parameter.owner {
        PortableTypeParameterOwnerV1::Declaration { owner } => StableTypeParameterOwnerRef::Declaration(owner.clone()),
        PortableTypeParameterOwnerV1::Callable { owner } => StableTypeParameterOwnerRef::Callable(owner.clone()),
        PortableTypeParameterOwnerV1::Implementation { owner } => StableTypeParameterOwnerRef::Impl(owner.clone()),
    };
    let stable = StableTypeParameterRef { owner, index: parameter.index };
    if !context.has_p7_type_parameter(&stable) {
        return Err(ExecutableValidationError::UnknownSemanticIdentity { pool, family, row });
    }
    Ok(())
}

fn require_p7_declaration(
    context: &ExecutableValidationContext<'_>,
    pool: usize,
    family: &'static str,
    row: usize,
    declaration: &StableDeclarationRef,
) -> Result<(), ExecutableValidationError> {
    if !context.has_p7_declaration(declaration) {
        return Err(ExecutableValidationError::UnknownSemanticIdentity { pool, family, row });
    }
    Ok(())
}

fn require_p7_callable(
    context: &ExecutableValidationContext<'_>,
    pool: usize,
    family: &'static str,
    row: usize,
    callable: &StableCallableRef,
) -> Result<(), ExecutableValidationError> {
    if !context.has_p7_module(callable_module(callable)) || !context.has_p7_callable(callable) {
        return Err(ExecutableValidationError::UnknownSemanticIdentity { pool, family, row });
    }
    Ok(())
}

fn require_p7_variant(
    context: &ExecutableValidationContext<'_>,
    pool: usize,
    family: &'static str,
    row: usize,
    variant: &StableVariantRef,
) -> Result<(), ExecutableValidationError> {
    if !context.has_p7_variant(variant) {
        return Err(ExecutableValidationError::UnknownSemanticIdentity { pool, family, row });
    }
    Ok(())
}

fn validate_trait_requirement(
    pool: usize,
    family: &'static str,
    row: usize,
    context: &ExecutableValidationContext<'_>,
    requirement: &PortableTraitRequirementV1,
) -> Result<(), ExecutableValidationError> {
    require_p7_declaration(context, pool, family, row, &requirement.owner)?;
    let callable = StableCallableRef {
        owner: StableCallableOwnerRef::Declaration(requirement.owner.clone()),
        side: requirement.side,
        selector: requirement.selector.clone().into_boxed_str(),
    };
    require_p7_callable(context, pool, family, row, &callable)
}

fn validate_trait_selection(
    pool: usize,
    family: &'static str,
    row: usize,
    context: &ExecutableValidationContext<'_>,
    selection: &PortableTraitSelectionTargetV1,
) -> Result<(), ExecutableValidationError> {
    match selection {
        PortableTraitSelectionTargetV1::Callable { callable } => require_p7_callable(context, pool, family, row, callable),
        PortableTraitSelectionTargetV1::DataComponent { component, .. } => {
            let stable = StableDataComponentRef {
                owner: component.owner.clone(),
                index: component.index,
            };
            if context.has_p7_data_component(&stable) {
                Ok(())
            } else {
                Err(ExecutableValidationError::UnknownSemanticIdentity { pool, family, row })
            }
        }
    }
}

fn validate_invocation_target(
    pool_index: usize,
    family: &'static str,
    row: usize,
    pool: &PortableSemanticPoolV1,
    context: &ExecutableValidationContext<'_>,
    target: &PortableInvocationTargetV1,
) -> Result<(), ExecutableValidationError> {
    match target {
        PortableInvocationTargetV1::Behavioral { lookup_owner, callable, .. } => {
            require_p7_declaration(context, pool_index, family, row, lookup_owner)?;
            require_p7_callable(context, pool_index, family, row, callable)
        }
        PortableInvocationTargetV1::VariantConstructor { variant } => require_p7_variant(context, pool_index, family, row, variant),
        PortableInvocationTargetV1::DataConstructor { constructor, construction } => {
            validate_data_construction_reference(pool_index, family, row, pool, context, constructor, *construction)
        }
    }
}

fn validate_family_target(
    pool_index: usize,
    family: &'static str,
    row: usize,
    pool: &PortableSemanticPoolV1,
    context: &ExecutableValidationContext<'_>,
    target: &PortableFamilyTargetV1,
) -> Result<(), ExecutableValidationError> {
    match target {
        PortableFamilyTargetV1::Singleton { variant } | PortableFamilyTargetV1::VariantConstructor { variant } => {
            require_p7_variant(context, pool_index, family, row, variant)
        }
        PortableFamilyTargetV1::Behavioral { target } => validate_invocation_target(pool_index, family, row, pool, context, target),
        PortableFamilyTargetV1::DataConstructor { constructor, construction } => {
            validate_data_construction_reference(pool_index, family, row, pool, context, constructor, *construction)
        }
    }
}

fn validate_data_construction_reference(
    pool_index: usize,
    family: &'static str,
    row: usize,
    pool: &PortableSemanticPoolV1,
    context: &ExecutableValidationContext<'_>,
    constructor: &StableDeclarationRef,
    construction: u32,
) -> Result<(), ExecutableValidationError> {
    if !context.data_declarations.contains(constructor) {
        return Err(ExecutableValidationError::UnknownSemanticIdentity { pool: pool_index, family, row });
    }
    if pool
        .data_constructions
        .get(construction as usize)
        .is_none_or(|recipe| &recipe.constructor != constructor)
    {
        return Err(ExecutableValidationError::InvalidSemanticProduct {
            pool: pool_index,
            family,
            row,
            detail: format!("construction index {construction} does not describe {constructor:?}"),
        });
    }
    Ok(())
}

fn validate_runtime_type_recipe(
    pool_index: usize,
    family: &'static str,
    row: usize,
    pool: &PortableSemanticPoolV1,
    recipe: &PortableRuntimeTypeRecipeV1,
) -> Result<(), ExecutableValidationError> {
    let type_id = match recipe {
        PortableRuntimeTypeRecipeV1::Closed { ty } | PortableRuntimeTypeRecipeV1::Template { ty } => ty,
    };
    if type_id.0 as usize >= pool.type_recipes.len() {
        return Err(ExecutableValidationError::InvalidSemanticProduct {
            pool: pool_index,
            family,
            row,
            detail: format!("type recipe {} is absent", type_id.0),
        });
    }
    Ok(())
}

fn validate_call_environment_index(
    pool_index: usize,
    row: usize,
    pool: &PortableSemanticPoolV1,
    call_environment: Option<u32>,
    family: &'static str,
) -> Result<(), ExecutableValidationError> {
    if call_environment.is_some_and(|index| index as usize >= pool.call_environments.len()) {
        return Err(ExecutableValidationError::InvalidSemanticProduct {
            pool: pool_index,
            family,
            row,
            detail: "call-environment reference is absent".to_owned(),
        });
    }
    Ok(())
}

fn validate_conformance_slots(
    pool_index: usize,
    row: usize,
    pool: &PortableSemanticPoolV1,
    context: &ExecutableValidationContext<'_>,
    slots: &[PortableConformanceSlotV1],
) -> Result<(), ExecutableValidationError> {
    for slot in slots {
        validate_conformance_slot(pool_index, row, pool, context, slot, slots.len())?;
    }
    Ok(())
}

fn validate_conformance_slot(
    pool_index: usize,
    row: usize,
    pool: &PortableSemanticPoolV1,
    context: &ExecutableValidationContext<'_>,
    slot: &PortableConformanceSlotV1,
    forward_limit: usize,
) -> Result<(), ExecutableValidationError> {
    match slot {
        PortableConformanceSlotV1::Selection { callable, nested, .. } => {
            require_p7_callable(context, pool_index, "conformance selection", row, callable)?;
            validate_conformance_slot_entries(pool_index, row, pool, context, nested, forward_limit)
        }
        PortableConformanceSlotV1::Forwarded { slot } => {
            if *slot as usize >= forward_limit {
                return Err(ExecutableValidationError::InvalidSemanticProduct {
                    pool: pool_index,
                    family: "forwarded conformance slot",
                    row,
                    detail: format!("slot {slot} is outside the enclosing call environment"),
                });
            }
            Ok(())
        }
        PortableConformanceSlotV1::DerivedConditional {
            callable, bindings, nested, ..
        } => {
            require_p7_callable(context, pool_index, "derived conformance selection", row, callable)?;
            for binding in bindings {
                validate_type_parameter_identity(context, pool_index, "conformance binding", row, &binding.parameter)?;
            }
            validate_conformance_slot_entries(pool_index, row, pool, context, nested, forward_limit)
        }
    }
}

fn validate_conformance_slot_entries(
    pool_index: usize,
    row: usize,
    pool: &PortableSemanticPoolV1,
    context: &ExecutableValidationContext<'_>,
    entries: &[PortableConformanceSlotEntryV1],
    forward_limit: usize,
) -> Result<(), ExecutableValidationError> {
    let mut indices = BTreeSet::new();
    for entry in entries {
        if !indices.insert(entry.slot) {
            return Err(ExecutableValidationError::InvalidSemanticProduct {
                pool: pool_index,
                family: "nested conformance slots",
                row,
                detail: format!("nested slot {} is duplicated", entry.slot),
            });
        }
        validate_conformance_slot(pool_index, row, pool, context, &entry.recipe, forward_limit)?;
    }
    Ok(())
}

fn callable_module(callable: &phalcom_type_meta::identity::StableCallableRef) -> &StableModuleRef {
    use phalcom_type_meta::identity::StableCallableOwnerRef;

    match &callable.owner {
        StableCallableOwnerRef::Declaration(declaration) => &declaration.module,
        StableCallableOwnerRef::Variant(variant) => &variant.owner.module,
        StableCallableOwnerRef::Conformance(implementation) => &implementation.module,
    }
}

fn stable_semantic_digest(fingerprint: Fingerprint128) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new_derive_key("phalcom.telosc.executable.semantic-type.v1");
    hasher.update(fingerprint.as_bytes());
    *hasher.finalize().as_bytes()
}

fn hex_digest(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut encoded = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        encoded.push(char::from(HEX[usize::from(byte >> 4)]));
        encoded.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    encoded
}
