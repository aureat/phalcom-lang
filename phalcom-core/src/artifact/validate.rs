//! Core-owned static validation that turns portable executable staging data
//! into runtime authority tied to the already-loaded P7 compiler image.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use phalcom_compiler_artifact::ArtifactError;
use phalcom_compiler_artifact::executable::compatibility::{CompilationProfileV1, PortableNativeAnchorV1, PrimordialRoleV1};
use phalcom_compiler_artifact::executable::constant::{ConstantRecipeV1, MethodSignatureV1, RestLayoutV1, RestModeV1, StableClassRefV1};
use phalcom_compiler_artifact::executable::image::{ExecutableImageLimitsV1, ExecutableImageV1};
use phalcom_compiler_artifact::executable::instruction::{
    ControlFlowKindV1, OPCODE_MANIFEST_V1, OperandDomainV1, PortableInstructionV1, PortableOperandValueV1,
};
use phalcom_compiler_artifact::executable::module::{PortableClassAllocationV1, PortableLinkedReadV1};
use phalcom_compiler_artifact::executable::prototype::{PortablePrototypeV1, PrototypeOwnerV1, UpvalueCaptureV1};
use phalcom_compiler_artifact::executable::semantic::{
    PortableConformanceSlotEntryV1, PortableConformanceSlotV1, PortableFamilyTargetV1, PortableInvocationTargetV1, PortableKindRecipeV1,
    PortableRuntimeTypeRecipeV1, PortableSemanticPoolV1, PortableTraitRequirementV1, PortableTraitSelectionTargetV1, PortableTypeParameterOwnerV1,
    PortableTypeParameterRefV1, PortableTypeRecipeV1,
};
use phalcom_modules::linker::{LinkedModule, LinkedReadSpec};
use phalcom_semantic::canonical_image::CanonicalUniverseFormalImage;
use phalcom_semantic::metadata::stable_identity::{
    to_stable_data_component, to_stable_declaration, to_stable_impl, to_stable_module, to_stable_variant, to_stable_variant_field,
};
use phalcom_type_meta::SemanticMetadataBundle;
use phalcom_type_meta::fingerprint::Fingerprint128;
use phalcom_type_meta::generic::{StableTypeParameterOwnerRef, StableTypeParameterRef};
use phalcom_type_meta::identity::{
    StableCallableOwnerRef, StableCallableRef, StableDataComponentRef, StableDeclarationRef, StableDispatchSide, StableFieldRef, StableImplRef,
    StableModuleRef, StableStaticConstantRef, StableVariantFieldRef, StableVariantRef,
};

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
    #[error("prototype {prototype} instruction {instruction} falls through to invalid instruction index {target}")]
    InvalidFallthrough { prototype: usize, instruction: usize, target: i64 },
    #[error("prototype {prototype} instruction {instruction} needs {required} stack values but only {available} are available")]
    StackUnderflow { prototype: usize, instruction: usize, required: usize, available: usize },
    #[error("prototype {prototype} instruction {instruction} references local slot {slot} outside the live frame range 0..{limit}")]
    InvalidLocalSlot { prototype: usize, instruction: usize, slot: usize, limit: usize },
    #[error("prototype {prototype} instruction {instruction} joins with incompatible stack heights {existing} and {incoming}")]
    IncompatibleControlFlowState { prototype: usize, instruction: usize, existing: usize, incoming: usize },
    #[error("prototype {prototype} instruction {instruction} stack height overflowed")]
    StackDepthOverflow { prototype: usize, instruction: usize },
    #[error("prototype {prototype} instruction {instruction} uses non-local return without exclusive block activation (owner {owner:?})")]
    InvalidNonLocalReturn { prototype: usize, instruction: usize, owner: PrototypeOwnerV1 },
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
    fn new(formal_image: &'a CanonicalUniverseFormalImage, metadata: &SemanticMetadataBundle) -> Result<Self, ExecutableValidationError> {
        let mut linked_modules = BTreeMap::new();
        for (module, linked) in formal_image.linked().modules.iter() {
            let stable = to_stable_module(module);
            if linked_modules.insert(stable.clone(), linked).is_some() {
                return Err(ExecutableValidationError::DuplicateP7Module(stable));
            }
        }

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
        let snapshot = formal_image.snapshot();
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
        for (module, _) in formal_image.linked().modules.iter() {
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

/// Authorizes a decoded executable only after all core-owned compatibility,
/// control-flow, runtime-object, and P2 semantic-reference checks pass.
pub(crate) fn validate_executable_image(
    image: Arc<ExecutableImageV1>,
    formal_image: &CanonicalUniverseFormalImage,
    metadata: &SemanticMetadataBundle,
    limits: &ExecutableImageLimitsV1,
) -> Result<ValidatedExecutableImage, ExecutableValidationError> {
    // The artifact layer owns fixed wire compatibility and bounded structural
    // accounting; core adds host policy and loaded runtime/P7 authority checks.
    image.validate_structure(limits)?;
    image.validate_native_abi_digest()?;
    if image.compatibility.compilation_profile != CompilationProfileV1::Debug {
        return Err(ExecutableValidationError::UnsupportedProfile(image.compatibility.compilation_profile.clone()));
    }

    let context = ExecutableValidationContext::new(formal_image, metadata)?;
    validate_module_coverage(&image, &context)?;
    validate_bootstrap_dependencies(&image, &context)?;
    validate_native_anchors(&image)?;
    validate_primordial_roles(&image)?;
    validate_classes(&image, &context)?;
    validate_prototype_owners(&image, &context)?;
    validate_prototypes(&image, &context)?;
    validate_method_recipes(&image)?;
    validate_detached_methods(&image, &context)?;
    validate_required_declarations(&image, &context)?;
    validate_canonical_semantic_roots(&image, &context)?;
    validate_semantic_pools(&image, &context)?;
    Ok(ValidatedExecutableImage { image })
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
    for class in &image.classes {
        if let PortableClassAllocationV1::Primordial { role } = &class.allocation {
            let Some(expected_role) = expected.iter().find(|expected| expected.role == *role) else {
                return Err(ExecutableValidationError::PrimordialRoleMismatch { role: role.clone() });
            };
            if expected_role.class != class.class {
                return Err(ExecutableValidationError::PrimordialRoleMismatch { role: role.clone() });
            }
        }
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
    let mut class_index = BTreeMap::new();
    for (index, row) in image.classes.iter().enumerate() {
        if class_index.insert((row.class.declaration.clone(), row.class.side), index).is_some() {
            return Err(ExecutableValidationError::InvalidClassLayout {
                class: row.class.clone(),
                detail: "duplicate class recipe".to_owned(),
            });
        }
    }

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
        if let Some(superclass) = &row.superclass
            && superclass.side != row.class.side
        {
            return Err(ExecutableValidationError::InvalidClassLayout {
                class: row.class.clone(),
                detail: "superclass dispatch side differs from the class recipe".to_owned(),
            });
        }
        let Some(metaclass) = &row.metaclass else {
            return Err(ExecutableValidationError::InvalidClassLayout {
                class: row.class.clone(),
                detail: "class recipe has no metaclass".to_owned(),
            });
        };
        if metaclass.side != StableDispatchSide::Class {
            return Err(ExecutableValidationError::InvalidClassLayout {
                class: row.class.clone(),
                detail: "metaclass reference is not on the class dispatch side".to_owned(),
            });
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
        let mut field_names = BTreeSet::new();
        for field in &row.fields {
            if field.slot >= row.field_count || !field_slots.insert(field.slot) || !field_names.insert(&field.name) {
                return Err(ExecutableValidationError::InvalidClassLayout {
                    class: row.class.clone(),
                    detail: format!("field {} has a duplicate name or duplicate/out-of-range slot {}", field.name, field.slot),
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

fn validate_method_recipes(image: &ExecutableImageV1) -> Result<(), ExecutableValidationError> {
    use phalcom_compiler_artifact::executable::constant::{MethodBodyV1, MethodSignatureKindV1, RestModeV1};
    use phalcom_compiler_artifact::executable::prototype::RestKindV1;
    use phalcom_common::selector::{Selector, SelectorKind, SelectorSlot};

    for (index, constant) in image.constants.iter().enumerate() {
        let ConstantRecipeV1::Method { recipe } = constant else {
            continue;
        };

        let invalid = |detail: String| ExecutableValidationError::InvalidExecutableReference {
            table: "method recipes",
            index,
            detail,
        };
        let selector = Selector::try_decode_exact(&recipe.signature.selector)
            .map_err(|error| invalid(format!("method recipe selector is invalid: {error}")))?;
        let has_rest = recipe.signature.selector.contains("***");
        let fixed_slots = if has_rest {
            &selector.slots[..selector.slots.len().saturating_sub(1)]
        } else {
            &selector.slots
        };
        let fixed_positionals = u8::try_from(fixed_slots.iter().filter(|slot| matches!(slot, SelectorSlot::Positional)).count())
            .map_err(|_| invalid("fixed positional count exceeds the executable ABI".to_owned()))?;
        let setter_value = matches!(selector.kind, SelectorKind::Setter | SelectorKind::SubscriptSet);
        let positional_arity = fixed_positionals
            .checked_add(u8::from(setter_value))
            .ok_or_else(|| invalid("positional arity exceeds the executable ABI".to_owned()))?;
        let slot_count = if selector.kind == SelectorKind::Method && has_rest {
            selector.slots.len().saturating_sub(1)
        } else {
            selector.slots.len()
        };
        let slot_count = u8::try_from(slot_count).map_err(|_| invalid("selector slot count exceeds the executable ABI".to_owned()))?;
        let expected_kind = match selector.kind {
            SelectorKind::Getter => MethodSignatureKindV1::Getter,
            SelectorKind::Setter => MethodSignatureKindV1::Setter,
            SelectorKind::Method => MethodSignatureKindV1::Method { slots: slot_count },
            SelectorKind::SubscriptGet => MethodSignatureKindV1::SubscriptGet { slots: slot_count },
            SelectorKind::SubscriptSet => MethodSignatureKindV1::SubscriptSet { slots: slot_count },
        };
        if recipe.signature.kind != expected_kind || recipe.signature.positional_arity != positional_arity {
            return Err(invalid("method signature kind or positional arity disagrees with its exact selector".to_owned()));
        }

        let fixed_labels = fixed_slots
            .iter()
            .filter_map(|slot| match slot {
                SelectorSlot::Label(label) => Some(label.as_str()),
                SelectorSlot::Positional => None,
            })
            .collect::<Vec<_>>();
        match (&recipe.signature.rest, has_rest) {
            (Some(rest), true)
                if rest.fixed_positionals == fixed_positionals
                    && rest.fixed_labels.iter().map(String::as_str).eq(fixed_labels.iter().copied()) => {}
            (None, false) => {}
            _ => return Err(invalid("rest signature layout disagrees with the exact selector".to_owned())),
        }
        if let Some(rest) = &recipe.signature.rest {
            let rest_parameters = if matches!(rest.mode, RestModeV1::Split { .. }) { 2 } else { 1 };
            let parameter_count = usize::from(rest.fixed_positionals)
                .checked_add(rest.fixed_labels.len())
                .and_then(|count| count.checked_add(rest_parameters))
                .ok_or_else(|| invalid("rest parameter count overflow".to_owned()))?;
            let valid_indices = match &rest.mode {
                RestModeV1::Positional { param_index } | RestModeV1::Labeled { param_index } | RestModeV1::Complete { param_index } => {
                    usize::from(*param_index) < parameter_count
                }
                RestModeV1::Split {
                    positional_param_index,
                    labeled_param_index,
                } => {
                    positional_param_index != labeled_param_index
                        && usize::from(*positional_param_index) < parameter_count
                        && usize::from(*labeled_param_index) < parameter_count
                }
            };
            if !matches!(recipe.signature.kind, MethodSignatureKindV1::Method { .. }) || !valid_indices {
                return Err(invalid("rest mode does not fit the method's parameter slots".to_owned()));
            }
        }

        match &recipe.body {
            MethodBodyV1::Native { anchor } => {
                let anchor_index = anchor.0 as usize;
                let Some(native_anchor) = image.native_anchors.get(anchor_index) else {
                    return Err(invalid(format!("native body references absent anchor {anchor_index}")));
                };
                let rest_shape_matches = match (&recipe.signature.rest, &native_anchor.signature.rest) {
                    (None, None) => true,
                    (Some(recipe_rest), Some(anchor_rest)) => {
                        recipe_rest.fixed_positionals == anchor_rest.fixed_positionals && recipe_rest.fixed_labels == anchor_rest.fixed_labels
                    }
                    _ => false,
                };
                if recipe.signature.selector != native_anchor.signature.selector
                    || recipe.signature.kind != native_anchor.signature.kind
                    || recipe.signature.positional_arity != native_anchor.signature.positional_arity
                    || !rest_shape_matches
                {
                    return Err(invalid(format!("native body signature shape disagrees with anchor {anchor_index}")));
                }
            }
            MethodBodyV1::Closure { template } => {
                let template_index = template.0 as usize;
                let Some(ConstantRecipeV1::ClosureTemplate { prototype: target, .. }) = image.constants.get(template_index) else {
                    return Err(invalid(format!("closure body references absent or non-template constant {template_index}")));
                };
                let prototype_index = target.0 as usize;
                let Some(prototype) = image.prototypes.get(prototype_index) else {
                    return Err(invalid(format!("closure body references absent prototype {prototype_index}")));
                };
                let expected_rest = recipe.signature.rest.as_ref().map(|rest| match rest.mode {
                    phalcom_compiler_artifact::executable::constant::RestModeV1::Positional { .. } => RestKindV1::Positional,
                    phalcom_compiler_artifact::executable::constant::RestModeV1::Labeled { .. } => RestKindV1::Labeled,
                    phalcom_compiler_artifact::executable::constant::RestModeV1::Split { .. } => RestKindV1::Split,
                    phalcom_compiler_artifact::executable::constant::RestModeV1::Complete { .. } => RestKindV1::Complete,
                });
                let expected_labels = recipe
                    .signature
                    .rest
                    .as_ref()
                    .map(|rest| rest.fixed_labels.iter().map(String::as_str).collect::<Vec<_>>())
                    .unwrap_or_else(|| fixed_labels.clone());
                if prototype.parameters.fixed_positionals != u32::from(recipe.signature.positional_arity)
                    || prototype.parameters.fixed_labels.iter().map(String::as_str).ne(expected_labels.iter().copied())
                    || prototype.parameters.rest != expected_rest
                {
                    return Err(invalid(format!("closure prototype {prototype_index} parameter layout disagrees with its method signature")));
                }
            }
        }
    }
    Ok(())
}

fn validate_detached_methods(image: &ExecutableImageV1, context: &ExecutableValidationContext<'_>) -> Result<(), ExecutableValidationError> {
    for (index, detached) in image.detached_methods.iter().enumerate() {
        if !context.has_p7_callable(&detached.callable) {
            return Err(ExecutableValidationError::UnknownP7Callable(detached.callable.clone()));
        }
        let method_constant = detached.method_constant.0 as usize;
        let Some(ConstantRecipeV1::Method { recipe }) = image.constants.get(method_constant) else {
            return Err(ExecutableValidationError::InvalidExecutableReference {
                table: "detached methods",
                index,
                detail: format!("method constant {method_constant} is absent or is not a method recipe"),
            });
        };
        if recipe.signature.selector != detached.callable.selector.as_ref() {
            return Err(ExecutableValidationError::InvalidExecutableReference {
                table: "detached methods",
                index,
                detail: "method recipe selector does not match its P7 callable identity".to_owned(),
            });
        }
        if let phalcom_compiler_artifact::executable::constant::MethodBodyV1::Closure { template } = &recipe.body {
            let template_index = template.0 as usize;
            let Some(ConstantRecipeV1::ClosureTemplate { prototype, .. }) = image.constants.get(template_index) else {
                return Err(ExecutableValidationError::InvalidExecutableReference {
                    table: "detached methods",
                    index,
                    detail: format!("closure body references absent or non-template constant {template_index}"),
                });
            };
            let prototype_index = prototype.0 as usize;
            let Some(target) = image.prototypes.get(prototype_index) else {
                return Err(ExecutableValidationError::InvalidExecutableReference {
                    table: "detached methods",
                    index,
                    detail: format!("closure body references absent prototype {prototype_index}"),
                });
            };
            if !matches!(&target.owner, PrototypeOwnerV1::Callable { callable } if callable == &detached.callable) {
                return Err(ExecutableValidationError::InvalidExecutableReference {
                    table: "detached methods",
                    index,
                    detail: format!("closure target prototype {prototype_index} is not owned by the detached P7 callable"),
                });
            }
        }
    }
    Ok(())
}

fn validate_prototypes(image: &ExecutableImageV1, context: &ExecutableValidationContext<'_>) -> Result<(), ExecutableValidationError> {
    validate_closure_template_constants(image, context)?;
    let activation_roles = classify_prototype_activation_roles(image)?;
    for (prototype_index, prototype) in image.prototypes.iter().enumerate() {
        let source_map_index = prototype.source.0 as usize;
        let Some(source_map) = image.source_maps.get(source_map_index) else {
            return Err(ExecutableValidationError::InvalidExecutableReference {
                table: "prototype source maps",
                index: prototype_index,
                detail: format!("source map {source_map_index} is absent"),
            });
        };
        let owner_module = prototype_module(image, prototype_index)?;
        if &source_map.module != owner_module {
            return Err(ExecutableValidationError::InvalidExecutableReference {
                table: "prototype source maps",
                index: prototype_index,
                detail: "source map module does not match prototype owner".to_owned(),
            });
        }
        let mut labels = BTreeSet::new();
        if prototype.parameters.fixed_labels.iter().any(|label| label.is_empty() || !labels.insert(label)) {
            return Err(ExecutableValidationError::InvalidPrototypeLayout {
                prototype: prototype_index,
                detail: "parameter labels must be nonempty and unique".to_owned(),
            });
        }
        let entry_slots = prototype_entry_slots(prototype_index, prototype)?;
        if entry_slots > prototype.max_slots as usize {
            return Err(ExecutableValidationError::InvalidPrototypeLayout {
                prototype: prototype_index,
                detail: format!("frame has {} slots but requires {entry_slots} entry slots", prototype.max_slots),
            });
        }

        validate_prototype_operands(image, context, prototype_index, prototype)?;
        validate_prototype_control_flow(image, context, prototype_index, prototype)?;
        validate_prototype_stack(image, prototype_index, prototype, activation_roles[prototype_index])?;
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
            if domain == OperandDomainV1::ClosureTemplateConstant {
                let ConstantRecipeV1::ClosureTemplate { prototype: child, .. } = constant else {
                    return Err(invalid("closure-template operand does not resolve to a closure template".to_owned()));
                };
                let Some(child_prototype) = image.prototypes.get(child.0 as usize) else {
                    return Err(invalid(format!("closure target prototype {} is absent", child.0)));
                };
                if let PrototypeOwnerV1::Nested { parent, .. } = &child_prototype.owner
                    && parent.0 as usize != prototype_index
                {
                    return Err(invalid(format!(
                        "nested closure target {} belongs to parent {}, not this prototype",
                        child.0, parent.0
                    )));
                }
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ReachableStackState {
    /// Total values above `CallFrame::stack_offset`, including receiver and parameters.
    height: usize,
    /// The receiver/parameter window established when this frame is entered.
    floor: usize,
}

impl ReachableStackState {
    fn adjust(
        self,
        prototype: usize,
        instruction: usize,
        pop: usize,
        push: usize,
    ) -> Result<Self, ExecutableValidationError> {
        let Some(available) = self.height.checked_sub(self.floor) else {
            return Err(ExecutableValidationError::StackUnderflow {
                prototype,
                instruction,
                required: pop,
                available: 0,
            });
        };
        if pop > available {
            return Err(ExecutableValidationError::StackUnderflow {
                prototype,
                instruction,
                required: pop,
                available,
            });
        }
        let height = self
            .height
            .checked_sub(pop)
            .and_then(|height| height.checked_add(push))
            .ok_or(ExecutableValidationError::StackDepthOverflow {
                prototype,
                instruction,
            })?;
        Ok(Self {
            height,
            floor: self.floor,
        })
    }
}

#[derive(Clone, Copy, Debug)]
struct StackDelta {
    pop: usize,
    push: usize,
}

#[derive(Clone, Copy, Debug)]
enum StackTransfer {
    Linear(StackDelta),
    Conditional { taken: StackDelta, fallthrough: StackDelta },
}

fn apply_stack_delta(
    state: ReachableStackState,
    delta: StackDelta,
    prototype: usize,
    instruction: usize,
) -> Result<ReachableStackState, ExecutableValidationError> {
    state.adjust(prototype, instruction, delta.pop, delta.push)
}

fn require_stack_values(
    state: ReachableStackState,
    prototype: usize,
    instruction: usize,
    required: usize,
) -> Result<(), ExecutableValidationError> {
    let Some(available) = state.height.checked_sub(state.floor) else {
        return Err(ExecutableValidationError::StackUnderflow {
            prototype,
            instruction,
            required,
            available: 0,
        });
    };
    if available < required {
        return Err(ExecutableValidationError::StackUnderflow {
            prototype,
            instruction,
            required,
            available,
        });
    }
    Ok(())
}

fn require_live_local_slot(
    state: ReachableStackState,
    prototype: usize,
    instruction: usize,
    slot: usize,
    insertion: bool,
) -> Result<(), ExecutableValidationError> {
    let limit = state
        .height
        .checked_add(usize::from(insertion))
        .ok_or(ExecutableValidationError::StackDepthOverflow {
            prototype,
            instruction,
        })?;
    if slot >= limit {
        return Err(ExecutableValidationError::InvalidLocalSlot {
            prototype,
            instruction,
            slot,
            limit,
        });
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, Default)]
struct PrototypeActivationRoles {
    block_template: bool,
    non_block: bool,
}

impl PrototypeActivationRoles {
    fn guarantees_home_frame_token(self) -> bool {
        self.block_template && !self.non_block
    }
}

fn activation_closure_target(
    image: &ExecutableImageV1,
    constant_id: u32,
    table: &'static str,
    index: usize,
) -> Result<usize, ExecutableValidationError> {
    let Some(ConstantRecipeV1::ClosureTemplate { prototype, .. }) = image.constants.get(constant_id as usize) else {
        return Err(ExecutableValidationError::InvalidExecutableReference {
            table,
            index,
            detail: format!("constant {constant_id} is absent or is not a closure template"),
        });
    };
    let target = prototype.0 as usize;
    if target >= image.prototypes.len() {
        return Err(ExecutableValidationError::InvalidExecutableReference {
            table,
            index,
            detail: format!("closure template targets absent prototype {}", prototype.0),
        });
    }
    Ok(target)
}

fn classify_prototype_activation_roles(
    image: &ExecutableImageV1,
) -> Result<Vec<PrototypeActivationRoles>, ExecutableValidationError> {
    let mut roles = vec![PrototypeActivationRoles::default(); image.prototypes.len()];
    for (index, prototype) in image.prototypes.iter().enumerate() {
        if matches!(prototype.owner, PrototypeOwnerV1::ModuleInitializer { .. } | PrototypeOwnerV1::Callable { .. }) {
            roles[index].non_block = true;
        }
        for (instruction_index, instruction) in prototype.code.iter().enumerate() {
            let PortableInstructionV1::Closure { operand0 } = instruction else {
                continue;
            };
            let Some(constant_id) = prototype.constants.get(usize::from(*operand0)) else {
                return Err(ExecutableValidationError::InvalidExecutableReference {
                    table: "block activation graph",
                    index: instruction_index,
                    detail: format!("prototype {index} closure instruction references absent constant slot {operand0}"),
                });
            };
            let target = activation_closure_target(image, constant_id.0, "block activation graph", instruction_index)?;
            roles[target].block_template = true;
        }
    }

    for (constant_index, constant) in image.constants.iter().enumerate() {
        let ConstantRecipeV1::Method { recipe } = constant else {
            continue;
        };
        let phalcom_compiler_artifact::executable::constant::MethodBodyV1::Closure { template } = &recipe.body else {
            continue;
        };
        let target = activation_closure_target(image, template.0, "method-body activation graph", constant_index)?;
        roles[target].non_block = true;
    }

    Ok(roles)
}

fn prototype_entry_slots(
    prototype_index: usize,
    prototype: &PortablePrototypeV1,
) -> Result<usize, ExecutableValidationError> {
    match prototype.owner {
        PrototypeOwnerV1::ModuleInitializer { .. } => Ok(0),
        PrototypeOwnerV1::Callable { .. } | PrototypeOwnerV1::Nested { .. } => {
            let rest_slots = usize::from(prototype.parameters.rest.is_some());
            (prototype.parameters.fixed_positionals as usize)
                .checked_add(prototype.parameters.fixed_labels.len())
                .and_then(|slots| slots.checked_add(rest_slots))
                .and_then(|slots| slots.checked_add(1))
                .ok_or_else(|| ExecutableValidationError::InvalidPrototypeLayout {
                    prototype: prototype_index,
                    detail: "receiver and parameter entry-slot count overflowed".to_owned(),
                })
        }
    }
}

#[derive(Clone, Copy)]
enum AnonymousProductShape {
    Tuple,
    Record,
    Pack,
}

fn anonymous_product_stack_values(
    image: &ExecutableImageV1,
    prototype_index: usize,
    prototype: &PortablePrototypeV1,
    instruction_index: usize,
    spec: u16,
    expected: AnonymousProductShape,
) -> Result<usize, ExecutableValidationError> {
    let invalid = |detail: String| ExecutableValidationError::InvalidInstruction {
        prototype: prototype_index,
        instruction: instruction_index,
        operand: "anonymous-product specification",
        detail,
    };
    let pool_index = prototype.semantics.0 as usize;
    let product = image
        .semantic_pools
        .get(pool_index)
        .and_then(|pool| pool.anonymous_products.get(usize::from(spec)))
        .ok_or_else(|| invalid(format!("semantic product {} is absent from pool {pool_index}", spec)))?;
    use phalcom_compiler_artifact::executable::semantic::PortableAnonymousProductKindV1 as K;
    let count = match (expected, &product.kind) {
        (AnonymousProductShape::Tuple, K::Tuple { positional_len, labels })
        | (AnonymousProductShape::Pack, K::Pack { positional_len, labels }) => usize::try_from(*positional_len)
            .ok()
            .and_then(|positionals| positionals.checked_add(labels.len()))
            .ok_or(ExecutableValidationError::StackDepthOverflow {
                prototype: prototype_index,
                instruction: instruction_index,
            })?,
        (AnonymousProductShape::Record, K::Record { logical_keys, .. }) => logical_keys.len(),
        (AnonymousProductShape::Tuple, _) => return Err(invalid("static Tuple instruction references a non-Tuple product".to_owned())),
        (AnonymousProductShape::Record, _) => return Err(invalid("static Record instruction references a non-Record product".to_owned())),
        (AnonymousProductShape::Pack, _) => return Err(invalid("static Pack instruction references a non-Pack product".to_owned())),
    };
    Ok(count)
}

fn instruction_stack_transfer(
    image: &ExecutableImageV1,
    prototype: usize,
    prototype_data: &PortablePrototypeV1,
    instruction_index: usize,
    instruction: &PortableInstructionV1,
    owner: &PrototypeOwnerV1,
    activation_roles: PrototypeActivationRoles,
    state: ReachableStackState,
) -> Result<StackTransfer, ExecutableValidationError> {
    use PortableInstructionV1 as I;

    let linear = |pop, push| StackTransfer::Linear(StackDelta { pop, push });
    match *instruction {
        I::Constant { .. }
        | I::Nil {}
        | I::True {}
        | I::False {}
        | I::Closure { .. }
        | I::GetEllipsis {}
        | I::GetGlobal { .. }
        | I::GetGlobalChecked { .. }
        | I::UninitializedBinding {}
        | I::GetLinked { .. }
        | I::LoadTypeParameterClass { .. } => Ok(linear(0, 1)),
        I::Pop {} => Ok(linear(1, 0)),
        I::GetLocal { operand0 } | I::GetLocalChecked { operand0, .. } => {
            require_live_local_slot(state, prototype, instruction_index, usize::from(operand0), false)?;
            Ok(linear(0, 1))
        }
        I::SetLocal { operand0 } => {
            require_live_local_slot(state, prototype, instruction_index, usize::from(operand0), false)?;
            require_stack_values(state, prototype, instruction_index, 1)?;
            Ok(linear(0, 0))
        }
        I::DefineGlobal { .. } => Ok(linear(1, 0)),
        I::SetGlobal { .. } => {
            require_stack_values(state, prototype, instruction_index, 1)?;
            Ok(linear(0, 0))
        }
        I::GetSelf {} => {
            require_live_local_slot(state, prototype, instruction_index, 0, false)?;
            Ok(linear(0, 1))
        }
        I::Class { .. } => {
            require_stack_values(state, prototype, instruction_index, 1)?;
            Ok(linear(1, 1))
        }
        I::Method { .. } => {
            require_stack_values(state, prototype, instruction_index, 2)?;
            Ok(linear(1, 0))
        }
        I::FinalizeClass {} => {
            require_stack_values(state, prototype, instruction_index, 1)?;
            Ok(linear(0, 0))
        }
        I::MakeFamily { .. } | I::MakeConditionalFamily { .. } => {
            require_stack_values(state, prototype, instruction_index, 1)?;
            Ok(linear(1, 1))
        }
        I::MakeResolvedBoundMethod { .. } | I::MakeAssociatedFamily { .. } => Ok(linear(0, 1)),
        I::MakeTraitBoundMethod { .. } => {
            require_stack_values(state, prototype, instruction_index, 1)?;
            Ok(linear(1, 1))
        }
        I::GetField { .. } | I::GetFieldChecked { .. } => {
            require_stack_values(state, prototype, instruction_index, 1)?;
            Ok(linear(1, 1))
        }
        I::SetField { .. } => {
            require_stack_values(state, prototype, instruction_index, 2)?;
            Ok(linear(2, 1))
        }
        I::Invoke { operand0: arity, .. } | I::SuperSend { operand0: arity, .. } => {
            let operands = usize::from(arity)
                .checked_add(1)
                .ok_or(ExecutableValidationError::StackDepthOverflow {
                    prototype,
                    instruction: instruction_index,
                })?;
            require_stack_values(state, prototype, instruction_index, operands)?;
            Ok(linear(operands, 1))
        }
        I::InvokeCompilerInternal { operand0: arity, .. } | I::InvokeSpecialized { operand0: arity, .. } => {
            let operands = usize::from(arity)
                .checked_add(1)
                .ok_or(ExecutableValidationError::StackDepthOverflow {
                    prototype,
                    instruction: instruction_index,
                })?;
            require_stack_values(state, prototype, instruction_index, operands)?;
            Ok(linear(operands, 1))
        }
        I::InvokeLocal { operand0: slot, operand1: arity, .. } => {
            require_live_local_slot(state, prototype, instruction_index, usize::from(slot), false)?;
            let arguments = usize::from(arity);
            require_stack_values(state, prototype, instruction_index, arguments)?;
            Ok(linear(arguments, 1))
        }
        I::InvokeConst { operand1: arity, .. } => {
            let arguments = usize::from(arity);
            require_stack_values(state, prototype, instruction_index, arguments)?;
            Ok(linear(arguments, 1))
        }
        I::InvokePack { .. } | I::SuperSendPack { .. } => {
            require_stack_values(state, prototype, instruction_index, 2)?;
            Ok(linear(1, 0))
        }
        I::InvokeSubscriptSetPack { .. } => {
            require_stack_values(state, prototype, instruction_index, 3)?;
            Ok(linear(2, 0))
        }
        I::InvokeResolvedAssociated { arity, .. } => {
            let arguments = usize::from(arity);
            require_stack_values(state, prototype, instruction_index, arguments)?;
            Ok(linear(arguments, 1))
        }
        I::InvokeAssociatedFamilyStatic { arity, .. }
        | I::InvokeConditional { arity, .. }
        | I::InvokeTraitSelected { arity, .. }
        | I::InvokeTraitRequirement { arity, .. } => {
            let operands = usize::from(arity)
                .checked_add(1)
                .ok_or(ExecutableValidationError::StackDepthOverflow {
                    prototype,
                    instruction: instruction_index,
                })?;
            require_stack_values(state, prototype, instruction_index, operands)?;
            Ok(linear(operands, 1))
        }
        I::InvokeAssociatedFamilyPack { .. } => {
            require_stack_values(state, prototype, instruction_index, 2)?;
            Ok(linear(1, 0))
        }
        I::GetUpvalue { .. } | I::GetUpvalueChecked { .. } => Ok(linear(0, 1)),
        I::SetUpvalue { .. } => {
            require_stack_values(state, prototype, instruction_index, 1)?;
            Ok(linear(0, 0))
        }
        I::CloseUpvalue { operand0 } => {
            require_live_local_slot(state, prototype, instruction_index, usize::from(operand0), false)?;
            Ok(linear(0, 0))
        }
        I::Jump { .. } | I::Loop { .. } => Ok(linear(0, 0)),
        I::Return {} => Ok(linear(1, 0)),
        I::ReturnNonLocal {} => {
            if !activation_roles.guarantees_home_frame_token() {
                return Err(ExecutableValidationError::InvalidNonLocalReturn {
                    prototype,
                    instruction: instruction_index,
                    owner: owner.clone(),
                });
            }
            Ok(linear(1, 0))
        }
        I::JumpIfFalse { .. } | I::JumpIfNone { .. } => Ok(linear(1, 0)),
        I::GuardBool { .. } => {
            require_stack_values(state, prototype, instruction_index, 1)?;
            Ok(linear(0, 0))
        }
        I::GuardBlock { .. } => Ok(linear(0, 0)),
        I::NewInstance {} => {
            require_stack_values(state, prototype, instruction_index, 1)?;
            Ok(linear(1, 1))
        }
        I::Dup {} => {
            require_stack_values(state, prototype, instruction_index, 1)?;
            Ok(linear(0, 1))
        }
        I::WrapSome {} => Ok(linear(1, 1)),
        I::GuardSymbol {} => {
            require_stack_values(state, prototype, instruction_index, 1)?;
            Ok(linear(0, 0))
        }
        I::BilateralPreferReflected { .. } => {
            require_stack_values(state, prototype, instruction_index, 2)?;
            Ok(linear(0, 1))
        }
        I::ValidateOrdering { .. } => {
            require_stack_values(state, prototype, instruction_index, 1)?;
            Ok(linear(1, 1))
        }
        I::Same {} => {
            require_stack_values(state, prototype, instruction_index, 2)?;
            Ok(linear(2, 1))
        }
        I::RaiseUnsupported { .. } => {
            require_stack_values(state, prototype, instruction_index, 2)?;
            Ok(linear(2, 0))
        }
        I::RaiseNonIterableStarOperand {} => {
            require_stack_values(state, prototype, instruction_index, 1)?;
            Ok(linear(1, 0))
        }
        I::ReserveScratchLocal { operand0 } => {
            require_live_local_slot(state, prototype, instruction_index, usize::from(operand0), true)?;
            Ok(linear(0, 1))
        }
        I::ReleaseScratchLocal { operand0 } => {
            require_live_local_slot(state, prototype, instruction_index, usize::from(operand0), false)?;
            Ok(linear(1, 0))
        }
        I::TryInvokeExact { arity, .. } => {
            let arguments = usize::from(arity)
                .checked_add(1)
                .ok_or(ExecutableValidationError::StackDepthOverflow {
                    prototype,
                    instruction: instruction_index,
                })?;
            require_stack_values(state, prototype, instruction_index, arguments)?;
            Ok(StackTransfer::Conditional {
                taken: StackDelta { pop: arguments, push: 0 },
                fallthrough: StackDelta { pop: arguments, push: 1 },
            })
        }
        I::JumpIfUnsupported { .. } => {
            require_stack_values(state, prototype, instruction_index, 1)?;
            Ok(StackTransfer::Conditional {
                taken: StackDelta { pop: 1, push: 0 },
                fallthrough: StackDelta { pop: 0, push: 0 },
            })
        }
        I::BuildTuple { positional, labeled } => {
            let labeled_values = usize::from(labeled)
                .checked_mul(2)
                .ok_or(ExecutableValidationError::StackDepthOverflow {
                    prototype,
                    instruction: instruction_index,
                })?;
            let values = usize::from(positional)
                .checked_add(labeled_values)
                .ok_or(ExecutableValidationError::StackDepthOverflow {
                    prototype,
                    instruction: instruction_index,
                })?;
            Ok(linear(values, 1))
        }
        I::BuildRecord { fields } => {
            let values = usize::from(fields)
                .checked_mul(2)
                .ok_or(ExecutableValidationError::StackDepthOverflow {
                    prototype,
                    instruction: instruction_index,
                })?;
            Ok(linear(values, 1))
        }
        I::BuildPack { positional, labeled } => {
            let labeled_values = usize::from(labeled)
                .checked_mul(2)
                .ok_or(ExecutableValidationError::StackDepthOverflow {
                    prototype,
                    instruction: instruction_index,
                })?;
            let values = usize::from(positional)
                .checked_add(labeled_values)
                .ok_or(ExecutableValidationError::StackDepthOverflow {
                    prototype,
                    instruction: instruction_index,
                })?;
            Ok(linear(values, 1))
        }
        I::BuildStaticTuple { spec } => {
            let values = anonymous_product_stack_values(image, prototype, prototype_data, instruction_index, spec, AnonymousProductShape::Tuple)?;
            require_stack_values(state, prototype, instruction_index, values)?;
            Ok(linear(values, 1))
        }
        I::BuildStaticRecord { spec } => {
            let values = anonymous_product_stack_values(image, prototype, prototype_data, instruction_index, spec, AnonymousProductShape::Record)?;
            require_stack_values(state, prototype, instruction_index, values)?;
            Ok(linear(values, 1))
        }
        I::BuildStaticPack { spec } => {
            let values = anonymous_product_stack_values(image, prototype, prototype_data, instruction_index, spec, AnonymousProductShape::Pack)?;
            require_stack_values(state, prototype, instruction_index, values)?;
            Ok(linear(values, 1))
        }
        I::BuildRange { has_lower, has_upper, .. } => {
            Ok(linear(usize::from(has_lower) + usize::from(has_upper), 1))
        }
        I::BuildList { operand0: count }
        | I::BuildArray { operand0: count }
        | I::BuildVector { operand0: count } => Ok(linear(usize::from(count), 1)),
        I::BeginListLiteral {}
        | I::BeginSetLiteral {}
        | I::BeginMapLiteral {}
        | I::BeginArrayLiteral {}
        | I::BeginVectorLiteral {}
        | I::NewRecordLiteralBuilder {} => Ok(linear(0, 1)),
        I::ListLiteralAppend {} | I::ArrayLiteralAppend {} => {
            require_stack_values(state, prototype, instruction_index, 2)?;
            Ok(linear(2, 1))
        }
        I::ListTryExpandTuplePositionals {} | I::ArrayTryExpandTuplePositionals {} => {
            require_stack_values(state, prototype, instruction_index, 2)?;
            Ok(linear(2, 1))
        }
        I::FinishListLiteral {} => {
            require_stack_values(state, prototype, instruction_index, 1)?;
            Ok(linear(0, 0))
        }
        I::FinishSetLiteral {} | I::FinishMapLiteral {} => Ok(linear(0, 0)),
        I::SetLiteralAdd {} => {
            require_stack_values(state, prototype, instruction_index, 2)?;
            Ok(linear(1, 0))
        }
        I::MapLiteralInsertUnique {} => {
            require_stack_values(state, prototype, instruction_index, 3)?;
            Ok(linear(2, 0))
        }
        I::MapLiteralExpandLabels {} => {
            require_stack_values(state, prototype, instruction_index, 2)?;
            Ok(linear(1, 0))
        }
        I::RecordLiteralAppend {} => {
            require_stack_values(state, prototype, instruction_index, 3)?;
            Ok(linear(3, 0))
        }
        I::RecordLiteralExpandLabels {} => {
            require_stack_values(state, prototype, instruction_index, 2)?;
            Ok(linear(2, 0))
        }
        I::FinishRecordLiteral {} | I::FinishArrayLiteral {} | I::FinishVectorLiteral { .. } => {
            require_stack_values(state, prototype, instruction_index, 1)?;
            Ok(linear(1, 1))
        }
        I::VectorExpandExactPositionals { .. } => {
            require_stack_values(state, prototype, instruction_index, 2)?;
            Ok(linear(2, 1))
        }
        I::CopyVector {} => Ok(linear(1, 1)),
        I::Enum { .. } | I::Data { .. } | I::LoadVariantSingleton { .. } | I::LoadDataSingleton { .. } => Ok(linear(0, 1)),
        I::FinalizeEnum { .. } | I::FinalizeData { .. } => Ok(linear(0, 0)),
        I::VariantMethod { .. } => {
            require_stack_values(state, prototype, instruction_index, 1)?;
            Ok(linear(1, 0))
        }
        I::ConstructVariant { arity, .. } | I::ConstructData { arity, .. } => {
            let arity = usize::from(arity);
            require_stack_values(state, prototype, instruction_index, arity)?;
            Ok(linear(arity, 1))
        }
        I::GetVariantPayload { .. } | I::GetDataComponent { .. } | I::GetDataProductSlot { .. } | I::IsVariant { .. } => {
            require_stack_values(state, prototype, instruction_index, 1)?;
            Ok(linear(1, 1))
        }
        I::MatchInvariantFailure {} => Ok(linear(0, 0)),
        I::NewArgumentPack {} => Ok(linear(0, 1)),
        I::PackPushPositional {} => {
            require_stack_values(state, prototype, instruction_index, 2)?;
            Ok(linear(2, 0))
        }
        I::PackReserveStaticLabel { .. } => {
            require_stack_values(state, prototype, instruction_index, 1)?;
            Ok(linear(1, 0))
        }
        I::PackReserveComputedLabel {} | I::PackFillReservedLabel {} => {
            require_stack_values(state, prototype, instruction_index, 2)?;
            Ok(linear(2, 0))
        }
        I::PackExpandLabels {} | I::PackExpandComplete {} => {
            require_stack_values(state, prototype, instruction_index, 2)?;
            Ok(linear(2, 0))
        }
        I::PackTryExpandTuplePositionals {} => {
            require_stack_values(state, prototype, instruction_index, 2)?;
            Ok(linear(2, 1))
        }
        I::FinishTuplePack {} | I::FinishPack {} => {
            require_stack_values(state, prototype, instruction_index, 1)?;
            Ok(linear(1, 1))
        }
        I::GuardFieldKey {} => {
            require_stack_values(state, prototype, instruction_index, 1)?;
            Ok(linear(0, 0))
        }
        I::RecordLiteralCheckKey {} => {
            require_stack_values(state, prototype, instruction_index, 2)?;
            Ok(linear(0, 0))
        }
    }
}

fn validate_prototype_stack(
    image: &ExecutableImageV1,
    prototype_index: usize,
    prototype: &PortablePrototypeV1,
    activation_roles: PrototypeActivationRoles,
) -> Result<(), ExecutableValidationError> {
    let entry_height = prototype_entry_slots(prototype_index, prototype)?;

    let code_len = prototype.code.len();
    if code_len == 0 {
        return Err(ExecutableValidationError::InvalidBranchTarget {
            prototype: prototype_index,
            instruction: 0,
            target: 0,
        });
    }
    let mut states = vec![None; code_len];
    let mut pending = vec![0usize];
    states[0] = Some(ReachableStackState {
        height: entry_height,
        floor: entry_height,
    });

    while let Some(instruction_index) = pending.pop() {
        let state = states[instruction_index].ok_or(ExecutableValidationError::InvalidInstruction {
            prototype: prototype_index,
            instruction: instruction_index,
            operand: "reachable control-flow state",
            detail: "reachable instruction has no incoming stack state".to_owned(),
        })?;
        let instruction = &prototype.code[instruction_index];
        let control_flow = instruction.control_flow_kind();
        let transfer = instruction_stack_transfer(
            image,
            prototype_index,
            prototype,
            instruction_index,
            instruction,
            &prototype.owner,
            activation_roles,
            state,
        )?;

        match control_flow {
            ControlFlowKindV1::Fallthrough => {
                let StackTransfer::Linear(delta) = transfer else {
                    return Err(invalid_stack_transfer_classification(prototype_index, instruction_index, control_flow));
                };
                let outgoing = apply_stack_delta(state, delta, prototype_index, instruction_index)?;
                let target = instruction_index.checked_add(1).filter(|target| *target < code_len).ok_or(
                    ExecutableValidationError::InvalidFallthrough {
                        prototype: prototype_index,
                        instruction: instruction_index,
                        target: instruction_index
                            .checked_add(1)
                            .and_then(|target| i64::try_from(target).ok())
                            .unwrap_or(i64::MAX),
                    },
                )?;
                enqueue_stack_state(&mut pending, &mut states, prototype_index, instruction_index, target, outgoing)?;
            }
            ControlFlowKindV1::UnconditionalBranch => {
                let StackTransfer::Linear(delta) = transfer else {
                    return Err(invalid_stack_transfer_classification(prototype_index, instruction_index, control_flow));
                };
                let offset = relative_jump_offset(instruction).ok_or(ExecutableValidationError::InvalidBranchTarget {
                    prototype: prototype_index,
                    instruction: instruction_index,
                    target: i64::MAX,
                })?;
                let target = validate_branch_target(prototype_index, instruction_index, code_len, offset)?;
                let outgoing = apply_stack_delta(state, delta, prototype_index, instruction_index)?;
                enqueue_stack_state(&mut pending, &mut states, prototype_index, instruction_index, target, outgoing)?;
            }
            ControlFlowKindV1::ConditionalBranch => {
                let offset = relative_jump_offset(instruction).ok_or(ExecutableValidationError::InvalidBranchTarget {
                    prototype: prototype_index,
                    instruction: instruction_index,
                    target: i64::MAX,
                })?;
                let target = validate_branch_target(prototype_index, instruction_index, code_len, offset)?;
                let fallthrough = instruction_index.checked_add(1).filter(|target| *target < code_len).ok_or(
                    ExecutableValidationError::InvalidFallthrough {
                        prototype: prototype_index,
                        instruction: instruction_index,
                        target: instruction_index
                            .checked_add(1)
                            .and_then(|target| i64::try_from(target).ok())
                            .unwrap_or(i64::MAX),
                    },
                )?;
                let (taken_delta, fallthrough_delta) = match transfer {
                    StackTransfer::Linear(delta) => (delta, delta),
                    StackTransfer::Conditional { taken, fallthrough } => (taken, fallthrough),
                };
                let taken_state = apply_stack_delta(state, taken_delta, prototype_index, instruction_index)?;
                let fallthrough_state = apply_stack_delta(state, fallthrough_delta, prototype_index, instruction_index)?;
                enqueue_stack_state(&mut pending, &mut states, prototype_index, instruction_index, target, taken_state)?;
                enqueue_stack_state(&mut pending, &mut states, prototype_index, instruction_index, fallthrough, fallthrough_state)?;
            }
            ControlFlowKindV1::FunctionExit | ControlFlowKindV1::NonLocalExit | ControlFlowKindV1::ExceptionalTermination => {
                let StackTransfer::Linear(delta) = transfer else {
                    return Err(invalid_stack_transfer_classification(prototype_index, instruction_index, control_flow));
                };
                let _terminal_state = apply_stack_delta(state, delta, prototype_index, instruction_index)?;
            }
        }
    }

    Ok(())
}

fn relative_jump_offset(instruction: &PortableInstructionV1) -> Option<i32> {
    let mut offset = None;
    instruction.visit_operands(|domain, value| {
        if domain == OperandDomainV1::RelativeJump {
            if let PortableOperandValueV1::Signed(value) = value {
                offset = i32::try_from(value).ok();
            }
        }
    });
    offset
}

fn enqueue_stack_state(
    pending: &mut Vec<usize>,
    states: &mut [Option<ReachableStackState>],
    prototype: usize,
    instruction: usize,
    target: usize,
    incoming: ReachableStackState,
) -> Result<(), ExecutableValidationError> {
    let Some(existing) = states.get_mut(target) else {
        return Err(ExecutableValidationError::InvalidBranchTarget {
            prototype,
            instruction,
            target: i64::try_from(target).unwrap_or(i64::MAX),
        });
    };
    match existing {
        Some(existing) if existing.height != incoming.height => Err(ExecutableValidationError::IncompatibleControlFlowState {
            prototype,
            instruction: target,
            existing: existing.height,
            incoming: incoming.height,
        }),
        Some(_) => Ok(()),
        None => {
            *existing = Some(incoming);
            pending.push(target);
            Ok(())
        }
    }
}

fn invalid_stack_transfer_classification(
    prototype: usize,
    instruction: usize,
    flow: ControlFlowKindV1,
) -> ExecutableValidationError {
    ExecutableValidationError::InvalidInstruction {
        prototype,
        instruction,
        operand: "stack-transfer metadata",
        detail: format!("conditional stack transfer is incompatible with {flow:?}"),
    }
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
        let control_flow = instruction.control_flow_kind();
        if opcode.control_flow != control_flow {
            return Err(ExecutableValidationError::InvalidInstruction {
                prototype: prototype_index,
                instruction: instruction_index,
                operand: "control-flow metadata",
                detail: "instruction and opcode manifest classifications disagree".to_owned(),
            });
        }

        let mut relative_jump_count = 0;
        let mut relative_jump_offset = None;
        instruction.visit_operands(|domain, value| {
            if domain == OperandDomainV1::RelativeJump {
                relative_jump_count += 1;
                if let PortableOperandValueV1::Signed(offset) = value {
                    relative_jump_offset = Some(offset);
                }
            }
        });

        let expects_relative_jump = match control_flow {
            ControlFlowKindV1::UnconditionalBranch | ControlFlowKindV1::ConditionalBranch => true,
            ControlFlowKindV1::Fallthrough
            | ControlFlowKindV1::FunctionExit
            | ControlFlowKindV1::NonLocalExit
            | ControlFlowKindV1::ExceptionalTermination => false,
        };
        if relative_jump_count != usize::from(expects_relative_jump) {
            return Err(ExecutableValidationError::InvalidInstruction {
                prototype: prototype_index,
                instruction: instruction_index,
                operand: "control-flow metadata",
                detail: format!(
                    "{control_flow:?} requires {} relative jump operand(s), found {relative_jump_count}",
                    usize::from(expects_relative_jump),
                ),
            });
        }

        match control_flow {
            ControlFlowKindV1::Fallthrough => {
                enqueue_fallthrough(&mut pending, prototype_index, instruction_index, code_len)?;
            }
            ControlFlowKindV1::UnconditionalBranch => {
                let offset = relative_jump_offset.ok_or(ExecutableValidationError::InvalidBranchTarget {
                    prototype: prototype_index,
                    instruction: instruction_index,
                    target: i64::MAX,
                })?;
                let target = validate_branch_target(prototype_index, instruction_index, code_len, offset)?;
                pending.push(target);
            }
            ControlFlowKindV1::ConditionalBranch => {
                let offset = relative_jump_offset.ok_or(ExecutableValidationError::InvalidBranchTarget {
                    prototype: prototype_index,
                    instruction: instruction_index,
                    target: i64::MAX,
                })?;
                let target = validate_branch_target(prototype_index, instruction_index, code_len, offset)?;
                pending.push(target);
                enqueue_fallthrough(&mut pending, prototype_index, instruction_index, code_len)?;
            }
            ControlFlowKindV1::FunctionExit
            | ControlFlowKindV1::NonLocalExit
            | ControlFlowKindV1::ExceptionalTermination => {}
        }
    }
    Ok(())
}

fn enqueue_fallthrough(
    pending: &mut Vec<usize>,
    prototype: usize,
    instruction: usize,
    code_len: usize,
) -> Result<(), ExecutableValidationError> {
    let target = instruction.checked_add(1);
    if let Some(target) = target.filter(|target| *target < code_len) {
        pending.push(target);
        return Ok(());
    }
    Err(ExecutableValidationError::InvalidFallthrough {
        prototype,
        instruction,
        target: target.and_then(|target| i64::try_from(target).ok()).unwrap_or(i64::MAX),
    })
}

fn validate_branch_target(prototype: usize, instruction: usize, code_len: usize, offset: i32) -> Result<usize, ExecutableValidationError> {
    let Some(target) = branch_target(instruction, offset) else {
        return Err(ExecutableValidationError::InvalidBranchTarget {
            prototype,
            instruction,
            target: i64::try_from(instruction)
                .ok()
                .and_then(|instruction| instruction.checked_add(1))
                .and_then(|instruction| instruction.checked_add(i64::from(offset)))
                .unwrap_or(i64::MAX),
        });
    };
    if target >= code_len {
        return Err(ExecutableValidationError::InvalidBranchTarget {
            prototype,
            instruction,
            target: target as i64,
        });
    }
    Ok(target)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validator_contract_uses_unpublished_p2_authority_and_returns_private_authority() {
        let _validator: fn(
            Arc<ExecutableImageV1>,
            &phalcom_semantic::canonical_image::CanonicalUniverseFormalImage,
            &phalcom_type_meta::SemanticMetadataBundle,
            &ExecutableImageLimitsV1,
        ) -> Result<ValidatedExecutableImage, ExecutableValidationError> = validate_executable_image;
    }
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
