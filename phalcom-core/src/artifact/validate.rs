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
    to_stable_callable, to_stable_data_component, to_stable_declaration, to_stable_impl, to_stable_module, to_stable_variant, to_stable_variant_field,
};
use phalcom_type_meta::SemanticMetadataBundle;
use phalcom_type_meta::fingerprint::Fingerprint128;
use phalcom_type_meta::generic::{StableTypeParameterOwnerRef, StableTypeParameterRef};
use phalcom_type_meta::identity::{
    StableCallableOwnerRef, StableCallableRef, StableDataComponentRef, StableDeclarationRef, StableDispatchSide, StableFieldRef, StableImplRef,
    StableModuleRef, StableStaticConstantRef, StableVariantFieldRef, StableVariantRef,
};

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct InstructionSite {
    prototype: usize,
    instruction: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum FieldLayoutAuthority {
    Class(StableClassRefV1),
    Variant(StableVariantRef),
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum ExecutableValidationError {
    #[error("executable structural validation failed: {0}")]
    Structural(#[from] ArtifactError),
    #[error("unsupported Universe executable profile {0:?}; expected Debug")]
    UnsupportedProfile(CompilationProfileV1),
    #[error("package executable authority violation: {0}")]
    PackageAuthority(String),
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
    StackUnderflow {
        prototype: usize,
        instruction: usize,
        required: usize,
        available: usize,
    },
    #[error("prototype {prototype} instruction {instruction} references local slot {slot} outside the live frame range 0..{limit}")]
    InvalidLocalSlot {
        prototype: usize,
        instruction: usize,
        slot: usize,
        limit: usize,
    },
    #[error("prototype {prototype} instruction {instruction} joins with incompatible stack heights {existing} and {incoming}")]
    IncompatibleControlFlowState {
        prototype: usize,
        instruction: usize,
        existing: usize,
        incoming: usize,
    },
    #[error("prototype {prototype} instruction {instruction} stack height overflowed")]
    StackDepthOverflow { prototype: usize, instruction: usize },
    #[error("prototype {prototype} instruction {instruction} uses non-local return without exclusive block activation (owner {owner:?})")]
    InvalidNonLocalReturn {
        prototype: usize,
        instruction: usize,
        owner: PrototypeOwnerV1,
    },
    #[error("prototype {prototype} has invalid executable frame or parameter layout: {detail}")]
    InvalidPrototypeLayout { prototype: usize, detail: String },
    #[error("nested prototype {child} has invalid capture from parent {parent}: {detail}")]
    InvalidClosureCapture { child: usize, parent: usize, detail: String },
    #[error("prototype owner {owner:?} is claimed by more than one executable prototype")]
    DuplicatePrototypeOwner { owner: PrototypeOwnerV1 },
    #[error("class recipe {class:?} has an invalid runtime field layout: {detail}")]
    InvalidClassLayout { class: StableClassRefV1, detail: String },
    #[error("method installation at prototype {installer_prototype} instruction {instruction} has unknown method provenance")]
    UnknownMethodInstallation { installer_prototype: usize, instruction: usize },
    #[error("method installation at prototype {installer_prototype} instruction {instruction} has no exact P7 class receiver for body {body_prototype:?}: {detail}")]
    UnknownMethodInstallationReceiver {
        installer_prototype: usize,
        instruction: usize,
        body_prototype: Option<usize>,
        detail: String,
    },
    #[error("method installations for body {body_prototype:?} conflict between {first_source:?} and {second_source:?}")]
    ConflictingMethodInstallation {
        body_prototype: Option<usize>,
        first_source: InstallationSource,
        second_source: InstallationSource,
    },
    #[error(
        "method installation at prototype {installer_prototype} instruction {instruction} has conflicting class receiver provenance for body {body_prototype:?}"
    )]
    ConflictingMethodInstallationReceiver {
        installer_prototype: usize,
        instruction: usize,
        body_prototype: Option<usize>,
    },
    #[error("prototype {prototype} instruction {instruction} has unknown receiver provenance for field slot {slot}: {detail}")]
    UnknownFieldReceiver {
        prototype: usize,
        instruction: usize,
        slot: u16,
        detail: String,
    },
    #[error("prototype {prototype} instruction {instruction} has conflicting field authority for slot {slot}")]
    ConflictingFieldAuthority { prototype: usize, instruction: usize, slot: u16 },
    #[error("prototype {prototype} instruction {instruction} uses field slot {slot} outside {authority:?} field count {field_count}")]
    FieldSlotOutOfRange {
        prototype: usize,
        instruction: usize,
        authority: FieldLayoutAuthority,
        slot: u16,
        field_count: usize,
    },
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

#[cfg_attr(test, derive(Default))]
struct ExecutableValidationContext<'a> {
    snapshot: Option<&'a phalcom_semantic::SemanticSnapshot>,
    linked_modules: BTreeMap<StableModuleRef, &'a LinkedModule>,
    declarations: BTreeSet<StableDeclarationRef>,
    class_declarations: BTreeSet<StableDeclarationRef>,
    callables: BTreeSet<phalcom_type_meta::identity::StableCallableRef>,
    trait_requirements: BTreeSet<StableCallableRef>,
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
    fn new(
        linked_program: &'a phalcom_modules::LinkedProgram,
        formal_image: &'a CanonicalUniverseFormalImage,
        metadata: &SemanticMetadataBundle,
    ) -> Result<Self, ExecutableValidationError> {
        let mut linked_modules = BTreeMap::new();
        for (module, linked) in linked_program.modules.iter() {
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
        // Abstract requirements are canonical trait-surface identities, not
        // necessarily executable callables in RuntimePublic metadata.
        let trait_requirements = snapshot
            .trait_surfaces
            .iter()
            .flat_map(|(_, surface)| surface.iter().map(|(_, member)| to_stable_callable(&member.callable)))
            .collect();
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
            snapshot: Some(snapshot),
            linked_modules,
            declarations,
            class_declarations,
            callables,
            trait_requirements,
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
            || self.enum_variants.contains_key(&class.declaration)
            || self.data_declarations.contains(&class.declaration)
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
/// Validates a package generation's executable image.
///
/// Enforces the structural, ABI and profile contract and the package trust
/// boundary: the image owns exactly `scope`, claims no native anchors or
/// primordial roles (those belong to Universe), and every eager module is
/// package-owned. Every module and class reference owned by an upstream
/// project must resolve to a row of `prefix`, the Universe image the package
/// composes over; semantic owner resolution is checked at installation.
pub(crate) fn validate_package_executable_image(
    image: Arc<ExecutableImageV1>,
    scope: &std::collections::BTreeSet<phalcom_type_meta::identity::StableModuleRef>,
    prefix: &ExecutableImageV1,
    limits: &ExecutableImageLimitsV1,
) -> Result<ValidatedExecutableImage, ExecutableValidationError> {
    image.validate_structure(limits)?;
    image.validate_upstream_over(prefix)?;
    image.validate_native_abi_digest()?;
    if image.compatibility.compilation_profile != CompilationProfileV1::Debug {
        return Err(ExecutableValidationError::UnsupportedProfile(image.compatibility.compilation_profile.clone()));
    }
    let owned = image.modules.iter().map(|module| module.identity.clone()).collect::<std::collections::BTreeSet<_>>();
    if &owned != scope || owned.len() != image.modules.len() {
        return Err(ExecutableValidationError::PackageAuthority("package executable module coverage differs from its compiler scope".into()));
    }
    if !image.native_anchors.is_empty() || !image.primordial_roles.is_empty() {
        return Err(ExecutableValidationError::PackageAuthority("package executable claims Universe native or primordial authority".into()));
    }
    if image.bootstrap.eager_modules.iter().any(|entry| !scope.contains(&entry.module)) {
        return Err(ExecutableValidationError::PackageAuthority("package executable schedules a foreign module".into()));
    }
    Ok(ValidatedExecutableImage { image })
}

pub(crate) fn validate_executable_image(
    image: Arc<ExecutableImageV1>,
    linked_program: &phalcom_modules::LinkedProgram,
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

    let context = ExecutableValidationContext::new(linked_program, formal_image, metadata)?;
    validate_module_coverage(&image, &context)?;
    validate_bootstrap_dependencies(&image, &context)?;
    validate_native_anchors(&image)?;
    validate_primordial_roles(&image)?;
    validate_classes(&image, &context)?;
    validate_prototype_owners(&image, &context)?;
    validate_prototypes(&image, &context)?;
    validate_method_recipes(&image)?;
    validate_detached_methods(&image, &context)?;
    validate_executable_provenance(&image, &context)?;
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
        let Some(metaclass) = &row.metaclass else {
            return Err(ExecutableValidationError::InvalidClassLayout {
                class: row.class.clone(),
                detail: "class recipe has no metaclass".to_owned(),
            });
        };
        let expected_metaclass_side = match row.class.side {
            StableDispatchSide::Instance => StableDispatchSide::Class,
            StableDispatchSide::Class => StableDispatchSide::Instance,
        };
        if metaclass.side != expected_metaclass_side {
            return Err(ExecutableValidationError::InvalidClassLayout {
                class: row.class.clone(),
                detail: format!("metaclass reference is not on the expected {expected_metaclass_side:?} dispatch side"),
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
    use phalcom_common::selector::{Selector, SelectorKind, SelectorSlot};
    use phalcom_compiler_artifact::executable::constant::{MethodBodyV1, MethodSignatureKindV1, RestModeV1};

    for (index, constant) in image.constants.iter().enumerate() {
        let ConstantRecipeV1::Method { recipe } = constant else {
            continue;
        };

        let invalid = |detail: String| ExecutableValidationError::InvalidExecutableReference {
            table: "method recipes",
            index,
            detail,
        };
        let selector =
            Selector::try_decode_exact(&recipe.signature.selector).map_err(|error| invalid(format!("method recipe selector is invalid: {error}")))?;
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
            return Err(invalid(
                "method signature kind or positional arity disagrees with its exact selector".to_owned(),
            ));
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
                if rest.fixed_positionals == fixed_positionals && rest.fixed_labels.iter().map(String::as_str).eq(fixed_labels.iter().copied()) => {}
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
                // Method dispatch validates selector labels and rest arguments
                // before entering the body. The compiler then flattens those
                // bound values into positional frame slots (including rest
                // bindings); a method-body prototype is not itself an
                // externally callable closure with the method's selector
                // shape.
                let expected_body_parameters = if let Some(rest) = &recipe.signature.rest {
                    let rest_parameters = if matches!(rest.mode, RestModeV1::Split { .. }) { 2 } else { 1 };
                    usize::from(rest.fixed_positionals)
                        .checked_add(rest.fixed_labels.len())
                        .and_then(|count| count.checked_add(rest_parameters))
                        .ok_or_else(|| invalid("method body parameter count overflow".to_owned()))?
                } else {
                    fixed_slots
                        .len()
                        .checked_add(usize::from(setter_value))
                        .ok_or_else(|| invalid("method body parameter count overflow".to_owned()))?
                };
                if prototype.parameters.fixed_positionals as usize != expected_body_parameters
                    || !prototype.parameters.fixed_labels.is_empty()
                    || prototype.parameters.rest.is_some()
                {
                    let found = &prototype.parameters;
                    return Err(invalid(format!(
                        "closure prototype {prototype_index} frame parameters disagree with its method binding layout: expected {expected_body_parameters} positional slots, found {} positional slots, {} labeled slots, rest {:?}",
                        found.fixed_positionals,
                        found.fixed_labels.len(),
                        found.rest,
                    )));
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
            let Some(_slot) = unsigned_operand(value).and_then(|slot| u16::try_from(slot).ok()) else {
                return Err(invalid("operand is not a portable field-slot index".to_owned()));
            };
            // Field-layout authority is flow-sensitive and checked after
            // method installation and closure activation provenance exists.
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
    fn adjust(self, prototype: usize, instruction: usize, pop: usize, push: usize) -> Result<Self, ExecutableValidationError> {
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
            .ok_or(ExecutableValidationError::StackDepthOverflow { prototype, instruction })?;
        Ok(Self { height, floor: self.floor })
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

fn require_stack_values(state: ReachableStackState, prototype: usize, instruction: usize, required: usize) -> Result<(), ExecutableValidationError> {
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
        .ok_or(ExecutableValidationError::StackDepthOverflow { prototype, instruction })?;
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

fn activation_closure_target(image: &ExecutableImageV1, constant_id: u32, table: &'static str, index: usize) -> Result<usize, ExecutableValidationError> {
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

fn classify_prototype_activation_roles(image: &ExecutableImageV1) -> Result<Vec<PrototypeActivationRoles>, ExecutableValidationError> {
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

fn prototype_entry_slots(prototype_index: usize, prototype: &PortablePrototypeV1) -> Result<usize, ExecutableValidationError> {
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
        (AnonymousProductShape::Tuple, K::Tuple { positional_len, labels }) | (AnonymousProductShape::Pack, K::Pack { positional_len, labels }) => {
            usize::try_from(*positional_len)
                .ok()
                .and_then(|positionals| positionals.checked_add(labels.len()))
                .ok_or(ExecutableValidationError::StackDepthOverflow {
                    prototype: prototype_index,
                    instruction: instruction_index,
                })?
        }
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
            let operands = usize::from(arity).checked_add(1).ok_or(ExecutableValidationError::StackDepthOverflow {
                prototype,
                instruction: instruction_index,
            })?;
            require_stack_values(state, prototype, instruction_index, operands)?;
            Ok(linear(operands, 1))
        }
        I::InvokeCompilerInternal { operand0: arity, .. } | I::InvokeSpecialized { operand0: arity, .. } => {
            let operands = usize::from(arity).checked_add(1).ok_or(ExecutableValidationError::StackDepthOverflow {
                prototype,
                instruction: instruction_index,
            })?;
            require_stack_values(state, prototype, instruction_index, operands)?;
            Ok(linear(operands, 1))
        }
        I::InvokeLocal {
            operand0: slot,
            operand1: arity,
            ..
        } => {
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
            let operands = usize::from(arity).checked_add(1).ok_or(ExecutableValidationError::StackDepthOverflow {
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
            let arguments = usize::from(arity).checked_add(1).ok_or(ExecutableValidationError::StackDepthOverflow {
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
            let labeled_values = usize::from(labeled).checked_mul(2).ok_or(ExecutableValidationError::StackDepthOverflow {
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
            let values = usize::from(fields).checked_mul(2).ok_or(ExecutableValidationError::StackDepthOverflow {
                prototype,
                instruction: instruction_index,
            })?;
            Ok(linear(values, 1))
        }
        I::BuildPack { positional, labeled } => {
            let labeled_values = usize::from(labeled).checked_mul(2).ok_or(ExecutableValidationError::StackDepthOverflow {
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
        I::BuildRange { has_lower, has_upper, .. } => Ok(linear(usize::from(has_lower) + usize::from(has_upper), 1)),
        I::BuildList { operand0: count } | I::BuildArray { operand0: count } | I::BuildVector { operand0: count } => Ok(linear(usize::from(count), 1)),
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
        let transfer = match instruction_stack_transfer(
            image,
            prototype_index,
            prototype,
            instruction_index,
            instruction,
            &prototype.owner,
            activation_roles,
            state,
        ) {
            Ok(transfer) => transfer,
            Err(error @ ExecutableValidationError::InvalidLocalSlot { .. }) => {
                let start = instruction_index.saturating_sub(6);
                let end = (instruction_index + 7).min(prototype.code.len());
                eprintln!(
                    "invalid-live-local prototype={prototype_index} owner={:?} name={:?} instruction={instruction_index} state={state:?} locals={:?} context={:#?}",
                    prototype.owner,
                    prototype.name,
                    prototype.local_names,
                    &prototype.code[start..end],
                );
                return Err(error);
            }
            Err(error) => return Err(error),
        };

        match control_flow {
            ControlFlowKindV1::Fallthrough => {
                let StackTransfer::Linear(delta) = transfer else {
                    return Err(invalid_stack_transfer_classification(prototype_index, instruction_index, control_flow));
                };
                let outgoing = apply_stack_delta(state, delta, prototype_index, instruction_index)?;
                let target = instruction_index
                    .checked_add(fallthrough_width(instruction))
                    .filter(|target| *target < code_len)
                    .ok_or(ExecutableValidationError::InvalidFallthrough {
                        prototype: prototype_index,
                        instruction: instruction_index,
                        target: instruction_index
                            .checked_add(fallthrough_width(instruction))
                            .and_then(|target| i64::try_from(target).ok())
                            .unwrap_or(i64::MAX),
                    })?;
                enqueue_stack_state(&mut pending, &mut states, prototype_index, instruction_index, target, outgoing, prototype)?;
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
                enqueue_stack_state(&mut pending, &mut states, prototype_index, instruction_index, target, outgoing, prototype)?;
            }
            ControlFlowKindV1::ConditionalBranch => {
                let offset = relative_jump_offset(instruction).ok_or(ExecutableValidationError::InvalidBranchTarget {
                    prototype: prototype_index,
                    instruction: instruction_index,
                    target: i64::MAX,
                })?;
                let target = validate_branch_target(prototype_index, instruction_index, code_len, offset)?;
                let fallthrough =
                    instruction_index
                        .checked_add(1)
                        .filter(|target| *target < code_len)
                        .ok_or(ExecutableValidationError::InvalidFallthrough {
                            prototype: prototype_index,
                            instruction: instruction_index,
                            target: instruction_index
                                .checked_add(1)
                                .and_then(|target| i64::try_from(target).ok())
                                .unwrap_or(i64::MAX),
                        })?;
                let (taken_delta, fallthrough_delta) = match transfer {
                    StackTransfer::Linear(delta) => (delta, delta),
                    StackTransfer::Conditional { taken, fallthrough } => (taken, fallthrough),
                };
                let taken_state = apply_stack_delta(state, taken_delta, prototype_index, instruction_index)?;
                let fallthrough_state = apply_stack_delta(state, fallthrough_delta, prototype_index, instruction_index)?;
                enqueue_stack_state(&mut pending, &mut states, prototype_index, instruction_index, target, taken_state, prototype)?;
                enqueue_stack_state(
                    &mut pending,
                    &mut states,
                    prototype_index,
                    instruction_index,
                    fallthrough,
                    fallthrough_state,
                    prototype,
                )?;
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

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum AllocationIdentity {
    Instruction(InstructionSite),
    Constant(usize),
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum ReceiverOrigin {
    Method {
        method_constant: usize,
        allocation: AllocationIdentity,
    },
    VariantMethod {
        method_constant: usize,
        variant: StableVariantRef,
    },
    DetachedCallable {
        method_constant: usize,
        callable: phalcom_type_meta::identity::StableCallableRef,
    },
    Instance {
        allocation: AllocationIdentity,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ActivationAuthority {
    layout: FieldLayoutAuthority,
    origin: ReceiverOrigin,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum RuntimeValueProvenance {
    Method {
        constant: usize,
    },
    ClassObject {
        declaration: StableDeclarationRef,
        allocation: AllocationIdentity,
    },
    Receiver(ActivationAuthority),
    ClosureTemplate {
        constant: usize,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum ProvenanceFact<T> {
    Unknown,
    Exact(T),
    Conflict,
}

impl<T> ProvenanceFact<T> {
    fn map<U>(self, f: impl FnOnce(T) -> U) -> ProvenanceFact<U> {
        match self {
            Self::Unknown => ProvenanceFact::Unknown,
            Self::Exact(value) => ProvenanceFact::Exact(f(value)),
            Self::Conflict => ProvenanceFact::Conflict,
        }
    }
}

fn join_provenance<T: Clone + Eq>(left: &ProvenanceFact<T>, right: &ProvenanceFact<T>) -> ProvenanceFact<T> {
    match (left, right) {
        (ProvenanceFact::Conflict, _) | (_, ProvenanceFact::Conflict) => ProvenanceFact::Conflict,
        (ProvenanceFact::Unknown, _) | (_, ProvenanceFact::Unknown) => ProvenanceFact::Unknown,
        (ProvenanceFact::Exact(left), ProvenanceFact::Exact(right)) if left == right => ProvenanceFact::Exact(left.clone()),
        (ProvenanceFact::Exact(_), ProvenanceFact::Exact(_)) => ProvenanceFact::Conflict,
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ProvenanceState {
    values: Vec<ProvenanceFact<RuntimeValueProvenance>>,
}

/// Receiver authority and the actual values of the closure's captured cells
/// are separate proofs. Lexical method ownership never authorizes an upvalue.
#[derive(Clone, Debug, Eq, PartialEq)]
struct PrototypeActivationProvenance {
    receiver: ProvenanceFact<ActivationAuthority>,
    conformance_caller: ProvenanceFact<Option<StableCallableRef>>,
    upvalues: Vec<ProvenanceFact<RuntimeValueProvenance>>,
}

fn activation_with_unknown_captures(
    prototype: &PortablePrototypeV1,
    receiver: ProvenanceFact<ActivationAuthority>,
) -> PrototypeActivationProvenance {
    PrototypeActivationProvenance {
        receiver,
        conformance_caller: ProvenanceFact::Unknown,
        upvalues: vec![ProvenanceFact::Unknown; prototype.upvalues.len()],
    }
}

/// Check writes reachable *after* closure creation by CFG, not textual
/// instruction position. A branch may jump backward to an earlier assignment
/// and exit without re-entering the closure creation instruction.
fn captured_local_has_reachable_write(
    image: &ExecutableImageV1,
    site: InstructionSite,
    slot: u32,
) -> bool {
    use PortableInstructionV1 as I;

    let Some(prototype) = image.prototypes.get(site.prototype) else {
        return true;
    };
    let mut pending = vec![site.instruction];
    let mut visited = BTreeSet::new();
    while let Some(index) = pending.pop() {
        if !visited.insert(index) {
            continue;
        }
        let Some(instruction) = prototype.code.get(index) else {
            return true;
        };
        match instruction {
            I::SetLocal { operand0 } if u32::from(*operand0) == slot => return true,
            I::ReserveScratchLocal { operand0 } | I::ReleaseScratchLocal { operand0 }
                if u32::from(*operand0) <= slot => return true,
            _ => {}
        }

        let fallthrough = || index.checked_add(fallthrough_width(instruction)).filter(|target| *target < prototype.code.len());
        let branch = || relative_jump_offset(instruction)
            .and_then(|offset| branch_target(index, offset))
            .filter(|target| *target < prototype.code.len());

        match instruction.control_flow_kind() {
            ControlFlowKindV1::Fallthrough => {
                let Some(next) = fallthrough() else { return true };
                pending.push(next);
            }
            ControlFlowKindV1::UnconditionalBranch => {
                let Some(next) = branch() else { return true };
                pending.push(next);
            }
            ControlFlowKindV1::ConditionalBranch => {
                let (Some(next), Some(target)) = (index.checked_add(1).filter(|target| *target < prototype.code.len()), branch()) else {
                    return true;
                };
                pending.push(next);
                pending.push(target);
            }
            ControlFlowKindV1::FunctionExit | ControlFlowKindV1::NonLocalExit | ControlFlowKindV1::ExceptionalTermination => {}
        }
    }
    false
}

/// A captured local is an aliased VM cell, not an immutable value snapshot.
/// Only propagate the value at a closure instruction if neither its creating
/// frame nor any nested closure capable of sharing the cell can overwrite it.
/// Conservative false positives become Unknown and fail closed at field uses.
fn captured_cell_may_change(
    image: &ExecutableImageV1,
    parent: usize,
    creation_instruction: usize,
    capture: UpvalueCaptureV1,
) -> bool {
    use PortableInstructionV1 as I;

    if image.prototypes.get(parent).is_none() {
        return true;
    }
    let mut aliases = Vec::new();
    match capture {
        UpvalueCaptureV1::Local(slot) => {
            // Looping over a closure site is not itself evidence that a captured
            // cell changes. The provenance dataflow already joins every incoming
            // iteration at the creation site, while this CFG walk proves whether
            // the aliased local can actually be overwritten after any creation.
            // Reject the mutation, not the cycle.
            if captured_local_has_reachable_write(image, InstructionSite {
                prototype: parent,
                instruction: creation_instruction,
            }, slot) {
                return true;
            }
            for (child_index, child) in image.prototypes.iter().enumerate() {
                if matches!(child.owner, PrototypeOwnerV1::Nested { parent: owner, .. } if owner.0 as usize == parent) {
                    for (slot_index, descriptor) in child.upvalues.iter().enumerate() {
                        if *descriptor == UpvalueCaptureV1::Local(slot) {
                            aliases.push((child_index, slot_index));
                        }
                    }
                }
            }
        }
        UpvalueCaptureV1::Upvalue(slot) => {
            aliases.push((parent, slot as usize));
        }
    }

    let mut visited = BTreeSet::new();
    while let Some((owner_index, slot)) = aliases.pop() {
        if !visited.insert((owner_index, slot)) {
            continue;
        }
        let Some(owner) = image.prototypes.get(owner_index) else {
            return true;
        };
        if owner.code.iter().any(|opcode| {
            matches!(opcode, I::SetUpvalue { operand0 } if usize::from(*operand0) == slot)
        }) {
            return true;
        }
        for (child_index, child) in image.prototypes.iter().enumerate() {
            if matches!(child.owner, PrototypeOwnerV1::Nested { parent: owner, .. } if owner.0 as usize == owner_index) {
                for (child_slot, descriptor) in child.upvalues.iter().enumerate() {
                    if *descriptor == UpvalueCaptureV1::Upvalue(slot as u32) {
                        aliases.push((child_index, child_slot));
                    }
                }
            }
        }
    }
    false
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum InstallationTarget {
    Class { allocation: AllocationIdentity, layout: StableClassRefV1 },
    Variant(StableVariantRef),
    DetachedCallable { callable: phalcom_type_meta::identity::StableCallableRef },
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct InstallationRoute {
    method_constant: usize,
    selector: String,
    target: InstallationTarget,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum InstallationSource {
    Bytecode(InstructionSite),
    DetachedMethod { row: usize },
}

#[derive(Clone, Debug)]
struct RecordedInstallation {
    route: InstallationRoute,
    first_source: InstallationSource,
    body_prototype: Option<usize>,
}

#[derive(Default)]
struct PrototypeProvenanceEffects {
    activations: Vec<(usize, PrototypeActivationProvenance)>,
    installations: Vec<(InstallationRoute, Option<usize>, InstructionSite, ProvenanceFact<ActivationAuthority>)>,
    class_allocations: Vec<(StableDeclarationRef, InstructionSite)>,
}

fn validate_executable_provenance(image: &ExecutableImageV1, context: &ExecutableValidationContext<'_>) -> Result<(), ExecutableValidationError> {
    let roles = classify_prototype_activation_roles(image)?;
    let mut activations = vec![None::<PrototypeActivationProvenance>; image.prototypes.len()];
    let mut pending = std::collections::VecDeque::new();
    for (index, prototype) in image.prototypes.iter().enumerate() {
        if matches!(prototype.owner, PrototypeOwnerV1::ModuleInitializer { .. }) {
            let mut activation = activation_with_unknown_captures(prototype, ProvenanceFact::Unknown);
            activation.conformance_caller = ProvenanceFact::Exact(None);
            activations[index] = Some(activation);
            pending.push_back(index);
        }
    }

    let mut installs_by_method = BTreeMap::<usize, RecordedInstallation>::new();
    let mut installs_by_body = BTreeMap::<usize, RecordedInstallation>::new();
    let mut class_allocations = BTreeMap::<StableDeclarationRef, BTreeSet<InstructionSite>>::new();

    // Callable ownership is not field authority by itself. A detached row
    // supplies the exact callable <-> method constant <-> body relation; only
    // that validated relation seeds this separate activation route.
    for (row, detached) in image.detached_methods.iter().enumerate() {
        let method_constant = detached.method_constant.0 as usize;
        let Some(ConstantRecipeV1::Method { recipe }) = image.constants.get(method_constant) else {
            continue; // validate_detached_methods has already rejected this.
        };
        let (body, _) = method_body_prototype(image, method_constant, &recipe.body)?;
        let layout = match &detached.callable.owner {
            StableCallableOwnerRef::Declaration(declaration) => {
                let class = StableClassRefV1 {
                    declaration: declaration.clone(),
                    side: detached.callable.side,
                };
                (context.has_p7_class(&class) && image.classes.iter().any(|entry| entry.class == class)).then_some(FieldLayoutAuthority::Class(class))
            }
            StableCallableOwnerRef::Variant(variant) => context.has_p7_variant(variant).then_some(FieldLayoutAuthority::Variant(variant.clone())),
            StableCallableOwnerRef::Conformance(_) => None,
        };
        let authority = layout.map_or(ProvenanceFact::Unknown, |layout| {
            ProvenanceFact::Exact(ActivationAuthority {
                layout,
                origin: ReceiverOrigin::DetachedCallable {
                    method_constant,
                    callable: detached.callable.clone(),
                },
            })
        });
        let route = InstallationRoute {
            method_constant,
            selector: recipe.signature.selector.clone(),
            target: InstallationTarget::DetachedCallable {
                callable: detached.callable.clone(),
            },
        };
        let record = RecordedInstallation {
            route: route.clone(),
            first_source: InstallationSource::DetachedMethod { row },
            body_prototype: body,
        };
        if let Some(previous) = installs_by_method.get(&method_constant)
            && previous.route != route
        {
            return Err(ExecutableValidationError::ConflictingMethodInstallation {
                body_prototype: body.or(previous.body_prototype),
                first_source: previous.first_source,
                second_source: InstallationSource::DetachedMethod { row },
            });
        }
        installs_by_method.entry(method_constant).or_insert_with(|| record.clone());
        if let Some(body) = body {
            if let Some(previous) = installs_by_body.get(&body)
                && previous.route != route
            {
                return Err(ExecutableValidationError::ConflictingMethodInstallation {
                    body_prototype: Some(body),
                    first_source: previous.first_source,
                    second_source: InstallationSource::DetachedMethod { row },
                });
            }
            installs_by_body.entry(body).or_insert(record);
            let mut activation = activation_with_unknown_captures(&image.prototypes[body], authority);
            activation.conformance_caller = ProvenanceFact::Exact(Some(installation_callable(&route)));
            merge_activation(&mut activations, &mut pending, body, activation)?;
        }
    }

    while let Some(prototype_index) = pending.pop_front() {
        let Some(prototype) = image.prototypes.get(prototype_index) else {
            continue;
        };
        let activation = activations[prototype_index]
            .clone()
            .unwrap_or_else(|| activation_with_unknown_captures(prototype, ProvenanceFact::Unknown));
        let effects = analyze_prototype_provenance(image, context, prototype_index, prototype, &activation, roles[prototype_index])?;

        for (declaration, site) in effects.class_allocations {
            class_allocations.entry(declaration).or_default().insert(site);
        }
        for (route, body, site, authority) in effects.installations {
            let record = RecordedInstallation {
                route: route.clone(),
                first_source: InstallationSource::Bytecode(site),
                body_prototype: body,
            };
            if let Some(previous) = installs_by_method.get(&route.method_constant)
                && previous.route != route
            {
                return Err(ExecutableValidationError::ConflictingMethodInstallation {
                    body_prototype: body.or(previous.body_prototype),
                    first_source: previous.first_source,
                    second_source: InstallationSource::Bytecode(site),
                });
            }
            installs_by_method.entry(route.method_constant).or_insert_with(|| record.clone());

            if let Some(body_prototype) = body {
                if let Some(previous) = installs_by_body.get(&body_prototype)
                    && previous.route != route
                {
                    return Err(ExecutableValidationError::ConflictingMethodInstallation {
                        body_prototype: Some(body_prototype),
                        first_source: previous.first_source,
                        second_source: InstallationSource::Bytecode(site),
                    });
                }
                installs_by_body.entry(body_prototype).or_insert(record);
                let mut activation = activation_with_unknown_captures(&image.prototypes[body_prototype], authority);
                activation.conformance_caller = ProvenanceFact::Exact(Some(installation_callable(&route)));
                merge_activation(&mut activations, &mut pending, body_prototype, activation)?;
            }
        }
        for (child, incoming) in effects.activations {
            merge_activation(&mut activations, &mut pending, child, incoming)?;
        }
    }

    validate_forwarded_conformance_authority(image, context, &activations)?;

    for installation in installs_by_method.values() {
        if let InstallationTarget::Class {
            allocation: AllocationIdentity::Instruction(site),
            layout,
        } = &installation.route.target
        {
            let sites = class_allocations.get(&layout.declaration);
            if sites.is_none_or(|sites| sites.len() != 1 || !sites.contains(site)) || instruction_site_is_cyclic(image, *site)? {
                return Err(ExecutableValidationError::UnknownMethodInstallationReceiver {
                    installer_prototype: match installation.first_source {
                        InstallationSource::Bytecode(site) => site.prototype,
                        InstallationSource::DetachedMethod { row } => row,
                    },
                    instruction: match installation.first_source {
                        InstallationSource::Bytecode(site) => site.instruction,
                        InstallationSource::DetachedMethod { .. } => 0,
                    },
                    body_prototype: installation.body_prototype,
                detail: format!(
                    "class {:?} allocation {:?} is not unique/non-cyclic; observed allocations {:?}, cyclic={}",
                    layout.declaration,
                    site,
                    sites,
                    instruction_site_is_cyclic(image, *site)?,
                ),
                });
            }
        }
    }

    Ok(())
}

fn installation_callable(route: &InstallationRoute) -> StableCallableRef {
    let (owner, side) = match &route.target {
        InstallationTarget::DetachedCallable { callable } => return callable.clone(),
        InstallationTarget::Class { layout, .. } => (StableCallableOwnerRef::Declaration(layout.declaration.clone()), layout.side),
        InstallationTarget::Variant(variant) => (StableCallableOwnerRef::Variant(variant.clone()), StableDispatchSide::Instance),
    };
    StableCallableRef {
        owner,
        side,
        selector: route.selector.clone().into_boxed_str(),
    }
}

fn conformance_slot_has_forwarding(slot: &PortableConformanceSlotV1) -> bool {
    match slot {
        PortableConformanceSlotV1::Forwarded { .. } => true,
        PortableConformanceSlotV1::Selection { nested, .. } | PortableConformanceSlotV1::DerivedConditional { nested, .. } => {
            nested.iter().any(|entry| conformance_slot_has_forwarding(&entry.recipe))
        }
    }
}

fn validate_forwarded_conformance_authority(
    image: &ExecutableImageV1,
    context: &ExecutableValidationContext<'_>,
    activations: &[Option<PrototypeActivationProvenance>],
) -> Result<(), ExecutableValidationError> {
    let mut checked = BTreeSet::new();
    let mut limits = BTreeMap::<StableCallableRef, usize>::new();
    for (prototype_index, prototype) in image.prototypes.iter().enumerate() {
        let pool_index = prototype.semantics.0 as usize;
        let Some(pool) = image.semantic_pools.get(pool_index) else { continue };
        for (row, environment) in pool.call_environments.iter().enumerate() {
            if !environment.conformance_slots.iter().any(conformance_slot_has_forwarding) {
                continue;
            }
            let invalid = |detail: String| ExecutableValidationError::InvalidSemanticProduct {
                pool: pool_index,
                family: "forwarded conformance slot",
                row,
                detail,
            };
            let caller = activations
                .get(prototype_index)
                .and_then(Option::as_ref)
                .map(|activation| &activation.conformance_caller);
            let limit = match caller {
                Some(ProvenanceFact::Exact(None)) => 0,
                Some(ProvenanceFact::Exact(Some(callable))) => {
                    require_p7_callable(context, pool_index, "forwarding caller", row, callable)?;
                    if let Some(limit) = limits.get(callable) {
                        *limit
                    } else {
                        let snapshot = context.snapshot.ok_or_else(|| invalid("caller has no imported semantic authority".into()))?;
                        let matches = snapshot
                            .callable_signatures
                            .iter()
                            .filter(|(id, _)| to_stable_callable(id) == *callable)
                            .map(|(id, _)| id)
                            .collect::<Vec<_>>();
                        if matches.len() != 1 {
                            return Err(invalid(format!("caller {callable:?} has no unique canonical signature")));
                        }
                        let evidence = crate::modules::compile::caller_generic_evidence(snapshot, matches[0])
                            .map_err(|error| invalid(format!("caller evidence layout failed: {error}")))?;
                        let limit = evidence.len();
                        limits.insert(callable.clone(), limit);
                        limit
                    }
                }
                _ => return Err(invalid(format!("prototype {prototype_index} has no exact caller conformance authority"))),
            };
            validate_conformance_slots(pool_index, row, pool, context, &environment.conformance_slots, Some(limit)).map_err(|error| match error {
                ExecutableValidationError::InvalidSemanticProduct {
                    pool,
                    family: "forwarded conformance slot",
                    row,
                    detail,
                } => ExecutableValidationError::InvalidSemanticProduct {
                    pool,
                    family: "forwarded conformance slot",
                    row,
                    detail: format!("{detail}; prototype {prototype_index}, authority {caller:?}, caller slot count {limit}"),
                },
                error => error,
            })?;
            checked.insert((pool_index, row));
        }
    }
    for (pool_index, pool) in image.semantic_pools.iter().enumerate() {
        for (row, environment) in pool.call_environments.iter().enumerate() {
            if environment.conformance_slots.iter().any(conformance_slot_has_forwarding) && !checked.contains(&(pool_index, row)) {
                return Err(ExecutableValidationError::InvalidSemanticProduct {
                    pool: pool_index,
                    family: "forwarded conformance slot",
                    row,
                    detail: "forwarding recipe has no proven consuming activation".into(),
                });
            }
        }
    }
    Ok(())
}

fn merge_activation(
    activations: &mut [Option<PrototypeActivationProvenance>],
    pending: &mut std::collections::VecDeque<usize>,
    child: usize,
    incoming: PrototypeActivationProvenance,
) -> Result<(), ExecutableValidationError> {
    let Some(current) = activations.get_mut(child) else {
        return Err(ExecutableValidationError::InvalidExecutableReference {
            table: "closure activations",
            index: child,
            detail: "activation targets an absent prototype".to_owned(),
        });
    };
    match current {
        None => {
            *current = Some(incoming);
            pending.push_back(child);
        }
        Some(existing) => {
            if existing.upvalues.len() != incoming.upvalues.len() {
                return Err(ExecutableValidationError::InvalidClosureCapture {
                    child,
                    parent: child,
                    detail: "activation routes disagree on captured-cell count".to_owned(),
                });
            }
            let merged = PrototypeActivationProvenance {
                receiver: join_provenance(&existing.receiver, &incoming.receiver),
                conformance_caller: join_provenance(&existing.conformance_caller, &incoming.conformance_caller),
                upvalues: existing.upvalues.iter().zip(&incoming.upvalues)
                    .map(|(left, right)| join_provenance(left, right))
                    .collect(),
            };
            if &merged != existing {
                *existing = merged;
                pending.push_back(child);
            }
        }
    }
    Ok(())
}

fn analyze_prototype_provenance(
    image: &ExecutableImageV1,
    context: &ExecutableValidationContext<'_>,
    prototype_index: usize,
    prototype: &PortablePrototypeV1,
    activation: &PrototypeActivationProvenance,
    activation_roles: PrototypeActivationRoles,
) -> Result<PrototypeProvenanceEffects, ExecutableValidationError> {
    use PortableInstructionV1 as I;

    let entry_height = prototype_entry_slots(prototype_index, prototype)?;
    if prototype.code.is_empty() {
        return Ok(PrototypeProvenanceEffects::default());
    }
    let mut entry_values = vec![ProvenanceFact::Unknown; entry_height];
    if let Some(local_self) = entry_values.first_mut() {
        *local_self = activation.receiver.clone().map(RuntimeValueProvenance::Receiver);
    }
    let mut states = vec![None::<ProvenanceState>; prototype.code.len()];
    let mut pending = vec![0usize];
    states[0] = Some(ProvenanceState { values: entry_values });
    let mut effects = PrototypeProvenanceEffects::default();

    while let Some(instruction_index) = pending.pop() {
        let state = states[instruction_index].clone().ok_or(ExecutableValidationError::InvalidInstruction {
            prototype: prototype_index,
            instruction: instruction_index,
            operand: "reachable provenance state",
            detail: "reachable instruction has no incoming provenance state".to_owned(),
        })?;
        let instruction = &prototype.code[instruction_index];
        let height = state.values.len();
        let stack_state = ReachableStackState { height, floor: entry_height };
        let transfer = instruction_stack_transfer(
            image,
            prototype_index,
            prototype,
            instruction_index,
            instruction,
            &prototype.owner,
            activation_roles,
            stack_state,
        )?;

        let site = InstructionSite {
            prototype: prototype_index,
            instruction: instruction_index,
        };
        let mut special_push = None::<ProvenanceFact<RuntimeValueProvenance>>;
        let mut state_after_local_write = state.clone();

        match *instruction {
            I::Constant { operand0 } => {
                let constant = local_constant_index(prototype, operand0);
                special_push = Some(constant.map_or(ProvenanceFact::Unknown, |constant| constant_value_provenance(image, context, constant)));
            }
            I::Closure { operand0 } => {
                if let Some(constant) = local_constant_index(prototype, operand0) {
                    special_push = Some(ProvenanceFact::Exact(RuntimeValueProvenance::ClosureTemplate { constant }));
                    if let Some(ConstantRecipeV1::ClosureTemplate {
                        prototype: child,
                        lexical_class,
                        super_anchor,
                        ..
                    }) = image.constants.get(constant)
                    {
                        let child = child.0 as usize;
                        let child_prototype = image.prototypes.get(child).ok_or_else(|| ExecutableValidationError::InvalidExecutableReference {
                            table: "closure creation",
                            index: constant,
                            detail: format!("closure template refers to missing prototype {child}"),
                        })?;
                        // The VM resolves Local and Upvalue against the *executing*
                        // parent frame. A forged template aimed at another parent's
                        // nested prototype must not inherit this frame's values.
                        if !matches!(child_prototype.owner, PrototypeOwnerV1::Nested { parent, .. } if parent.0 as usize == prototype_index) {
                            return Err(ExecutableValidationError::InvalidClosureCapture {
                                child,
                                parent: prototype_index,
                                detail: "closure template is not nested under its creating frame".to_owned(),
                            });
                        }
                        let child_authority = explicit_template_authority(image, context, constant, lexical_class.as_ref(), super_anchor.as_ref(), &activation.receiver);
                        let captures = child_prototype.upvalues.iter().map(|descriptor| {
                            if captured_cell_may_change(image, prototype_index, instruction_index, *descriptor) {
                                return ProvenanceFact::Unknown;
                            }
                            match descriptor {
                                UpvalueCaptureV1::Local(slot) =>
                                    state.values.get(*slot as usize).cloned().unwrap_or(ProvenanceFact::Unknown),
                                UpvalueCaptureV1::Upvalue(slot) =>
                                    activation.upvalues.get(*slot as usize).cloned().unwrap_or(ProvenanceFact::Unknown),
                            }
                        }).collect();
                        effects.activations.push((child, PrototypeActivationProvenance {
                            receiver: child_authority,
                            // Block frames do not capture the creator's conformance environment.
                            conformance_caller: ProvenanceFact::Exact(None),
                            upvalues: captures,
                        }));
                    }
                }
            }
            I::Class { operand0 } => {
                let declaration = local_symbol(prototype, image, operand0)
                    .and_then(|name| source_declared_class(image, prototype_module(image, prototype_index).ok()?, name));
                if let Some(declaration) = declaration {
                    effects.class_allocations.push((declaration.clone(), site));
                    special_push = Some(ProvenanceFact::Exact(RuntimeValueProvenance::ClassObject {
                        declaration,
                        allocation: AllocationIdentity::Instruction(site),
                    }));
                }
            }
            I::Enum { operand0 } => {
                // Enum creates a fresh root class object at runtime. Its exact
                // declaration comes from the bytecode-addressed semantic row,
                // while the instruction site preserves runtime allocation
                // identity for the same uniqueness and cycle checks as Class.
                let pool_index = prototype.semantics.0 as usize;
                let declaration = image
                    .semantic_pools
                    .get(pool_index)
                    .and_then(|pool| pool.enum_specs.get(usize::from(operand0)))
                    .map(|spec| spec.owner.clone());
                let declaration = declaration.ok_or_else(|| ExecutableValidationError::InvalidInstruction {
                    prototype: prototype_index,
                    instruction: instruction_index,
                    operand: "enum declaration provenance",
                    detail: format!(
                        "semantic pool {pool_index} has no enum specification at slot {operand0}"
                    ),
                })?;
                let class = StableClassRefV1 {
                    declaration: declaration.clone(),
                    side: StableDispatchSide::Instance,
                };
                if !context.has_p7_class(&class) {
                    return Err(ExecutableValidationError::UnknownSemanticIdentity {
                        pool: pool_index,
                        family: "enum root class owner",
                        row: usize::from(operand0),
                    });
                }
                if !image.classes.iter().any(|row| row.class == class) {
                    return Err(ExecutableValidationError::MissingClassRecipe(class));
                }
                effects.class_allocations.push((declaration.clone(), site));
                special_push = Some(ProvenanceFact::Exact(RuntimeValueProvenance::ClassObject {
                    declaration,
                    allocation: AllocationIdentity::Instruction(site),
                }));
            }
            I::Data { operand0 } => {
                let pool_index = prototype.semantics.0 as usize;
                let declaration = image
                    .semantic_pools
                    .get(pool_index)
                    .and_then(|pool| pool.data_specs.get(usize::from(operand0)))
                    .map(|spec| spec.owner.clone())
                    .ok_or_else(|| ExecutableValidationError::InvalidInstruction {
                        prototype: prototype_index,
                        instruction: instruction_index,
                        operand: "data declaration provenance",
                        detail: format!("semantic pool {pool_index} has no data specification at slot {operand0}"),
                    })?;
                let class = StableClassRefV1 {
                    declaration: declaration.clone(),
                    side: StableDispatchSide::Instance,
                };
                if !context.has_p7_class(&class) {
                    return Err(ExecutableValidationError::UnknownSemanticIdentity {
                        pool: pool_index,
                        family: "data root class owner",
                        row: usize::from(operand0),
                    });
                }
                if !image.classes.iter().any(|row| row.class == class) {
                    return Err(ExecutableValidationError::MissingClassRecipe(class));
                }
                effects.class_allocations.push((declaration.clone(), site));
                special_push = Some(ProvenanceFact::Exact(RuntimeValueProvenance::ClassObject {
                    declaration,
                    allocation: AllocationIdentity::Instruction(site),
                }));
            }
            I::NewInstance {} => {
                special_push = Some(class_receiver_from_value(state.values.last()));
            }
            I::GetSelf {} => {
                special_push = Some(activation.receiver.clone().map(RuntimeValueProvenance::Receiver));
            }
            I::GetUpvalue { operand0 } | I::GetUpvalueChecked { operand0, .. } => {
                special_push = Some(activation.upvalues.get(usize::from(operand0))
                    .cloned().unwrap_or(ProvenanceFact::Unknown));
            }
            I::GetLocal { operand0 } | I::GetLocalChecked { operand0, .. } => {
                special_push = Some(state.values.get(usize::from(operand0)).cloned().unwrap_or(ProvenanceFact::Unknown));
            }
            I::SetLocal { operand0 } => {
                if let (Some(target), Some(value)) = (state_after_local_write.values.get_mut(usize::from(operand0)), state.values.last()) {
                    *target = value.clone();
                }
            }
            I::Dup {} => {
                special_push = Some(state.values.last().cloned().unwrap_or(ProvenanceFact::Unknown));
            }
            I::Method { operand0, operand1 } => {
                let selector = local_symbol(prototype, image, operand0).unwrap_or_default();
                let method_value = state.values.last();
                let class_value = state.values.get(height.saturating_sub(2));
                let (route, body, authority) = prove_method_installation(
                    image,
                    context,
                    prototype_index,
                    instruction_index,
                    method_value,
                    class_value,
                    selector,
                    operand1,
                )?;
                effects.installations.push((route, body, site, authority));
            }
            I::VariantMethod { variant, selector } => {
                let selector = local_symbol(prototype, image, selector).unwrap_or_default();
                let method_value = state.values.last();
                let (route, body, authority) =
                    prove_variant_method_installation(image, context, prototype_index, instruction_index, method_value, variant, selector)?;
                effects.installations.push((route, body, site, authority));
            }
            I::GetField { operand0 } | I::GetFieldChecked { operand0, .. } => {
                validate_field_receiver(image, context, prototype_index, instruction_index, operand0, state.values.last())?;
            }
            I::SetField { operand0 } => {
                validate_field_receiver(
                    image,
                    context,
                    prototype_index,
                    instruction_index,
                    operand0,
                    state.values.get(height.saturating_sub(2)),
                )?;
            }
            _ => {}
        }

        let control_flow = instruction.control_flow_kind();
        match control_flow {
            ControlFlowKindV1::Fallthrough => {
                let StackTransfer::Linear(delta) = transfer else {
                    return Err(invalid_stack_transfer_classification(prototype_index, instruction_index, control_flow));
                };
                let outgoing = apply_provenance_delta(&state_after_local_write, delta, special_push, instruction);
                let target = instruction_index
                    .checked_add(fallthrough_width(instruction))
                    .filter(|target| *target < prototype.code.len())
                    .ok_or(ExecutableValidationError::InvalidFallthrough {
                        prototype: prototype_index,
                        instruction: instruction_index,
                        target: instruction_index
                            .checked_add(fallthrough_width(instruction))
                            .and_then(|target| i64::try_from(target).ok())
                            .unwrap_or(i64::MAX),
                    })?;
                enqueue_provenance_state(&mut pending, &mut states, prototype_index, instruction_index, target, outgoing, prototype)?;
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
                let target = validate_branch_target(prototype_index, instruction_index, prototype.code.len(), offset)?;
                let outgoing = apply_provenance_delta(&state_after_local_write, delta, special_push, instruction);
                enqueue_provenance_state(&mut pending, &mut states, prototype_index, instruction_index, target, outgoing, prototype)?;
            }
            ControlFlowKindV1::ConditionalBranch => {
                let offset = relative_jump_offset(instruction).ok_or(ExecutableValidationError::InvalidBranchTarget {
                    prototype: prototype_index,
                    instruction: instruction_index,
                    target: i64::MAX,
                })?;
                let target = validate_branch_target(prototype_index, instruction_index, prototype.code.len(), offset)?;
                let fallthrough =
                    instruction_index
                        .checked_add(1)
                        .filter(|target| *target < prototype.code.len())
                        .ok_or(ExecutableValidationError::InvalidFallthrough {
                            prototype: prototype_index,
                            instruction: instruction_index,
                            target: instruction_index
                                .checked_add(1)
                                .and_then(|target| i64::try_from(target).ok())
                                .unwrap_or(i64::MAX),
                        })?;
                let (taken, normal) = match transfer {
                    StackTransfer::Linear(delta) => (delta, delta),
                    StackTransfer::Conditional { taken, fallthrough } => (taken, fallthrough),
                };
                enqueue_provenance_state(
                    &mut pending,
                    &mut states,
                    prototype_index,
                    instruction_index,
                    target,
                    apply_provenance_delta(&state_after_local_write, taken, special_push.clone(), instruction),
                    prototype,
                )?;
                enqueue_provenance_state(
                    &mut pending,
                    &mut states,
                    prototype_index,
                    instruction_index,
                    fallthrough,
                    apply_provenance_delta(&state_after_local_write, normal, special_push, instruction),
                    prototype,
                )?;
            }
            ControlFlowKindV1::FunctionExit | ControlFlowKindV1::NonLocalExit | ControlFlowKindV1::ExceptionalTermination => {}
        }
    }
    Ok(effects)
}

fn apply_provenance_delta(
    state: &ProvenanceState,
    delta: StackDelta,
    special_push: Option<ProvenanceFact<RuntimeValueProvenance>>,
    instruction: &PortableInstructionV1,
) -> ProvenanceState {
    match *instruction {
        PortableInstructionV1::ReserveScratchLocal { operand0 } => {
            let mut values = state.values.clone();
            values.insert(usize::from(operand0), ProvenanceFact::Unknown);
            return ProvenanceState { values };
        }
        PortableInstructionV1::ReleaseScratchLocal { operand0 } => {
            let mut values = state.values.clone();
            values.remove(usize::from(operand0));
            return ProvenanceState { values };
        }
        _ => {}
    }
    let mut values = state.values.clone();
    values.truncate(values.len().saturating_sub(delta.pop));
    for index in 0..delta.push {
        if index == 0 {
            values.push(special_push.clone().unwrap_or(ProvenanceFact::Unknown));
        } else {
            values.push(ProvenanceFact::Unknown);
        }
    }
    ProvenanceState { values }
}

fn enqueue_provenance_state(
    pending: &mut Vec<usize>,
    states: &mut [Option<ProvenanceState>],
    prototype: usize,
    instruction: usize,
    target: usize,
    incoming: ProvenanceState,
    prototype_data: &PortablePrototypeV1,
) -> Result<(), ExecutableValidationError> {
    let Some(existing) = states.get_mut(target) else {
        return Err(ExecutableValidationError::InvalidBranchTarget {
            prototype,
            instruction,
            target: i64::try_from(target).unwrap_or(i64::MAX),
        });
    };
    let Some(existing_state) = existing else {
        *existing = Some(incoming);
        pending.push(target);
        return Ok(());
    };
    if existing_state.values.len() != incoming.values.len() {
        eprintln!(
            "provenance-height mismatch prototype={prototype} owner={:?} name={:?} predecessor={instruction}:{:?} target={target}:{:?} existing={} incoming={}",
            prototype_data.owner,
            prototype_data.name,
            prototype_data.code.get(instruction),
            prototype_data.code.get(target),
            existing_state.values.len(),
            incoming.values.len(),
        );
        return Err(ExecutableValidationError::IncompatibleControlFlowState {
            prototype,
            instruction: target,
            existing: existing_state.values.len(),
            incoming: incoming.values.len(),
        });
    }
    let merged = existing_state
        .values
        .iter()
        .zip(&incoming.values)
        .map(|(left, right)| join_provenance(left, right))
        .collect::<Vec<_>>();
    if merged != existing_state.values {
        existing_state.values = merged;
        pending.push(target);
    }
    Ok(())
}

fn local_constant_index(prototype: &PortablePrototypeV1, local: u16) -> Option<usize> {
    prototype.constants.get(usize::from(local)).map(|constant| constant.0 as usize)
}

fn local_symbol<'a>(prototype: &PortablePrototypeV1, image: &'a ExecutableImageV1, local: u16) -> Option<&'a str> {
    let constant = local_constant_index(prototype, local)?;
    match image.constants.get(constant)? {
        ConstantRecipeV1::Symbol { text } => Some(text),
        _ => None,
    }
}

fn constant_value_provenance(image: &ExecutableImageV1, context: &ExecutableValidationContext<'_>, constant: usize) -> ProvenanceFact<RuntimeValueProvenance> {
    match image.constants.get(constant) {
        Some(ConstantRecipeV1::Method { .. }) => ProvenanceFact::Exact(RuntimeValueProvenance::Method { constant }),
        Some(ConstantRecipeV1::Class { reference })
            if reference.side == StableDispatchSide::Instance && context.has_p7_class(reference) && image.classes.iter().any(|row| row.class == *reference) =>
        {
            ProvenanceFact::Exact(RuntimeValueProvenance::ClassObject {
                declaration: reference.declaration.clone(),
                allocation: AllocationIdentity::Constant(constant),
            })
        }
        _ => ProvenanceFact::Unknown,
    }
}

fn class_receiver_from_value(value: Option<&ProvenanceFact<RuntimeValueProvenance>>) -> ProvenanceFact<RuntimeValueProvenance> {
    match value {
        Some(ProvenanceFact::Exact(RuntimeValueProvenance::ClassObject { declaration, allocation })) => {
            ProvenanceFact::Exact(RuntimeValueProvenance::Receiver(ActivationAuthority {
                layout: FieldLayoutAuthority::Class(StableClassRefV1 {
                    declaration: declaration.clone(),
                    side: StableDispatchSide::Instance,
                }),
                origin: ReceiverOrigin::Instance {
                    allocation: allocation.clone(),
                },
            }))
        }
        Some(ProvenanceFact::Conflict) => ProvenanceFact::Conflict,
        _ => ProvenanceFact::Unknown,
    }
}

fn source_declared_class(image: &ExecutableImageV1, module: &StableModuleRef, name: &str) -> Option<StableDeclarationRef> {
    let mut matches = image.classes.iter().filter_map(|row| {
        if row.class.side != StableDispatchSide::Instance
            || row.class.declaration.module != *module
            || row.class.declaration.path.len() != 1
            || row.class.declaration.path[0].as_ref() != name
            || !matches!(&row.allocation, PortableClassAllocationV1::SourceDeclared)
        {
            return None;
        }
        Some(row.class.declaration.clone())
    });
    let declaration = matches.next()?;
    matches.next().is_none().then_some(declaration)
}

fn explicit_template_authority(
    image: &ExecutableImageV1,
    context: &ExecutableValidationContext<'_>,
    _constant: usize,
    lexical_class: Option<&StableClassRefV1>,
    super_anchor: Option<&StableClassRefV1>,
    inherited: &ProvenanceFact<ActivationAuthority>,
) -> ProvenanceFact<ActivationAuthority> {
    if let (Some(lexical), Some(anchor)) = (lexical_class, super_anchor) {
        if lexical.declaration != anchor.declaration || !context.has_p7_class(anchor) || !image.classes.iter().any(|row| row.class == *anchor) {
            return ProvenanceFact::Conflict;
        }
        return match inherited {
            ProvenanceFact::Exact(authority) if authority.layout == FieldLayoutAuthority::Class(anchor.clone()) => inherited.clone(),
            ProvenanceFact::Unknown => ProvenanceFact::Unknown,
            ProvenanceFact::Conflict => ProvenanceFact::Conflict,
            ProvenanceFact::Exact(_) => ProvenanceFact::Conflict,
        };
    }
    inherited.clone()
}

fn prove_method_installation(
    image: &ExecutableImageV1,
    context: &ExecutableValidationContext<'_>,
    installer_prototype: usize,
    instruction: usize,
    method_value: Option<&ProvenanceFact<RuntimeValueProvenance>>,
    class_value: Option<&ProvenanceFact<RuntimeValueProvenance>>,
    selector: &str,
    is_static: bool,
) -> Result<(InstallationRoute, Option<usize>, ProvenanceFact<ActivationAuthority>), ExecutableValidationError> {
    let site = InstructionSite {
        prototype: installer_prototype,
        instruction,
    };
    let Some(ProvenanceFact::Exact(RuntimeValueProvenance::Method { constant })) = method_value else {
        return Err(ExecutableValidationError::UnknownMethodInstallation {
            installer_prototype,
            instruction,
        });
    };
    let Some(ConstantRecipeV1::Method { recipe }) = image.constants.get(*constant) else {
        return Err(ExecutableValidationError::InvalidInstruction {
            prototype: installer_prototype,
            instruction,
            operand: "method provenance",
            detail: format!("exact method value points to non-method constant {constant}"),
        });
    };
    if recipe.signature.selector != selector {
        return Err(ExecutableValidationError::InvalidInstruction {
            prototype: installer_prototype,
            instruction,
            operand: "method selector",
            detail: format!("installation selector {selector:?} differs from method recipe {:?}", recipe.signature.selector),
        });
    }
    let (body, template) = method_body_prototype(image, *constant, &recipe.body)?;
    let (declaration, allocation) = match class_value {
        Some(ProvenanceFact::Exact(RuntimeValueProvenance::ClassObject { declaration, allocation })) => (declaration.clone(), allocation.clone()),
        Some(ProvenanceFact::Conflict) => {
            return Err(ExecutableValidationError::ConflictingMethodInstallationReceiver {
                installer_prototype,
                instruction,
                body_prototype: body,
            });
        }
        _ => {
            let detail = image.prototypes.get(installer_prototype).map_or_else(
                || format!("class operand provenance is {class_value:?}; installer prototype is absent"),
                |prototype| {
                    let start = instruction.saturating_sub(6);
                    let end = (instruction + 1).min(prototype.code.len());
                    format!(
                        "class operand provenance is {class_value:?}; installer {:?} {:?}, code before installation {:?}",
                        prototype.owner,
                        prototype.name,
                        &prototype.code[start..end],
                    )
                },
            );
            return Err(ExecutableValidationError::UnknownMethodInstallationReceiver {
                installer_prototype,
                instruction,
                body_prototype: body,
                detail,
            });
        }
    };
    let layout = StableClassRefV1 {
        declaration: declaration.clone(),
        side: if is_static { StableDispatchSide::Class } else { StableDispatchSide::Instance },
    };
    if !context.has_p7_class(&layout) || !image.classes.iter().any(|row| row.class == layout) {
        return Err(ExecutableValidationError::UnknownMethodInstallationReceiver {
            installer_prototype,
            instruction,
            body_prototype: body,
            detail: format!("class {declaration:?} has no exact P7-owned {layout:?} field layout"),
        });
    }
    let route = InstallationRoute {
        method_constant: *constant,
        selector: selector.to_owned(),
        target: InstallationTarget::Class {
            allocation: allocation.clone(),
            layout: layout.clone(),
        },
    };
    let authority = ProvenanceFact::Exact(ActivationAuthority {
        layout: FieldLayoutAuthority::Class(layout),
        origin: ReceiverOrigin::Method {
            method_constant: *constant,
            allocation,
        },
    });
    let _ = (site, template);
    Ok((route, body, authority))
}

fn prove_variant_method_installation(
    image: &ExecutableImageV1,
    context: &ExecutableValidationContext<'_>,
    installer_prototype: usize,
    instruction: usize,
    method_value: Option<&ProvenanceFact<RuntimeValueProvenance>>,
    variant_index: u16,
    selector: &str,
) -> Result<(InstallationRoute, Option<usize>, ProvenanceFact<ActivationAuthority>), ExecutableValidationError> {
    let Some(ProvenanceFact::Exact(RuntimeValueProvenance::Method { constant })) = method_value else {
        return Err(ExecutableValidationError::UnknownMethodInstallation {
            installer_prototype,
            instruction,
        });
    };
    let Some(ConstantRecipeV1::Method { recipe }) = image.constants.get(*constant) else {
        return Err(ExecutableValidationError::InvalidInstruction {
            prototype: installer_prototype,
            instruction,
            operand: "variant method provenance",
            detail: format!("exact method value points to non-method constant {constant}"),
        });
    };
    if recipe.signature.selector != selector {
        return Err(ExecutableValidationError::InvalidInstruction {
            prototype: installer_prototype,
            instruction,
            operand: "variant method selector",
            detail: format!("installation selector {selector:?} differs from method recipe {:?}", recipe.signature.selector),
        });
    }
    let pool_index = image.prototypes[installer_prototype].semantics.0 as usize;
    let variant = image
        .semantic_pools
        .get(pool_index)
        .and_then(|pool| pool.variant_targets.get(usize::from(variant_index)))
        .filter(|variant| context.has_p7_variant(variant))
        .cloned()
        .ok_or(ExecutableValidationError::InvalidExecutableReference {
            table: "variant method installation targets",
            index: instruction,
            detail: format!("variant target {variant_index} is not an exact P7 variant"),
        })?;
    let (body, _) = method_body_prototype(image, *constant, &recipe.body)?;
    let authority = ProvenanceFact::Exact(ActivationAuthority {
        layout: FieldLayoutAuthority::Variant(variant.clone()),
        origin: ReceiverOrigin::VariantMethod {
            method_constant: *constant,
            variant: variant.clone(),
        },
    });
    Ok((
        InstallationRoute {
            method_constant: *constant,
            selector: selector.to_owned(),
            target: InstallationTarget::Variant(variant),
        },
        body,
        authority,
    ))
}

fn method_body_prototype(
    image: &ExecutableImageV1,
    method_constant: usize,
    body: &phalcom_compiler_artifact::executable::constant::MethodBodyV1,
) -> Result<(Option<usize>, Option<usize>), ExecutableValidationError> {
    use phalcom_compiler_artifact::executable::constant::MethodBodyV1;
    let MethodBodyV1::Closure { template } = body else {
        return Ok((None, None));
    };
    let template_index = template.0 as usize;
    let Some(ConstantRecipeV1::ClosureTemplate { prototype, .. }) = image.constants.get(template_index) else {
        return Err(ExecutableValidationError::InvalidExecutableReference {
            table: "method installations",
            index: method_constant,
            detail: format!("method body template {template_index} is absent or not a closure template"),
        });
    };
    let body = prototype.0 as usize;
    if body >= image.prototypes.len() {
        return Err(ExecutableValidationError::InvalidExecutableReference {
            table: "method installations",
            index: method_constant,
            detail: format!("method body prototype {body} is absent"),
        });
    }
    Ok((Some(body), Some(template_index)))
}

fn validate_field_receiver(
    image: &ExecutableImageV1,
    context: &ExecutableValidationContext<'_>,
    prototype: usize,
    instruction: usize,
    slot: u16,
    receiver: Option<&ProvenanceFact<RuntimeValueProvenance>>,
) -> Result<(), ExecutableValidationError> {
    let authority = match receiver {
        Some(ProvenanceFact::Exact(RuntimeValueProvenance::Receiver(activation))) => activation.layout.clone(),
        Some(ProvenanceFact::Exact(RuntimeValueProvenance::ClassObject { declaration, .. })) => FieldLayoutAuthority::Class(StableClassRefV1 {
            declaration: declaration.clone(),
            side: StableDispatchSide::Class,
        }),
        Some(ProvenanceFact::Conflict) => return Err(ExecutableValidationError::ConflictingFieldAuthority { prototype, instruction, slot }),
        _ => return Err(unknown_field_receiver(image, prototype, instruction, slot, receiver)),
    };
    let field_count = match &authority {
        FieldLayoutAuthority::Class(class) => {
            if !context.has_p7_class(class) {
                return Err(unknown_field_receiver(image, prototype, instruction, slot, receiver));
            }
            image.classes.iter().find(|row| row.class == *class).map(|row| row.field_count as usize)
        }
        FieldLayoutAuthority::Variant(variant) => context.variant_field_counts.get(variant).copied(),
    }
    .ok_or_else(|| unknown_field_receiver(image, prototype, instruction, slot, receiver))?;
    if usize::from(slot) >= field_count {
        return Err(ExecutableValidationError::FieldSlotOutOfRange {
            prototype,
            instruction,
            authority,
            slot,
            field_count,
        });
    }
    Ok(())
}

fn unknown_field_receiver(
    image: &ExecutableImageV1,
    prototype: usize,
    instruction: usize,
    slot: u16,
    receiver: Option<&ProvenanceFact<RuntimeValueProvenance>>,
) -> ExecutableValidationError {
    let detail = image.prototypes.get(prototype).map_or_else(
        || format!("receiver provenance is {receiver:?}; prototype is absent"),
        |prototype_data| {
            let start = instruction.saturating_sub(6);
            let end = (instruction + 1).min(prototype_data.code.len());
            let nested = match prototype_data.owner {
                PrototypeOwnerV1::Nested { parent, .. } => image.prototypes.get(parent.0 as usize).map_or_else(
                    || format!("; captures {:?}; parent prototype {} is absent", prototype_data.upvalues, parent.0),
                    |parent_data| {
                        format!(
                            "; captures {:?}; parent {} owner {:?}, name {:?}, code {:?}",
                            prototype_data.upvalues,
                            parent.0,
                            parent_data.owner,
                            parent_data.name,
                            parent_data.code
                        )
                    },
                ),
                _ => String::new(),
            };
            format!(
                "receiver provenance is {receiver:?}; owner {:?}, name {:?}, code around access {:?}{nested}",
                prototype_data.owner,
                prototype_data.name,
                &prototype_data.code[start..end],
            )
        },
    );
    ExecutableValidationError::UnknownFieldReceiver {
        prototype,
        instruction,
        slot,
        detail,
    }
}

fn instruction_site_is_cyclic(image: &ExecutableImageV1, site: InstructionSite) -> Result<bool, ExecutableValidationError> {
    let Some(prototype) = image.prototypes.get(site.prototype) else {
        return Ok(true);
    };
    if site.instruction >= prototype.code.len() {
        return Ok(true);
    }
    let mut pending = vec![site.instruction];
    let mut visited = BTreeSet::new();
    while let Some(instruction_index) = pending.pop() {
        if !visited.insert(instruction_index) {
            continue;
        }
        let instruction = &prototype.code[instruction_index];
        let successors = match instruction.control_flow_kind() {
            ControlFlowKindV1::Fallthrough => instruction_index
                .checked_add(fallthrough_width(instruction))
                .filter(|target| *target < prototype.code.len())
                .into_iter()
                .collect::<Vec<_>>(),
            ControlFlowKindV1::UnconditionalBranch => {
                let offset = relative_jump_offset(instruction).ok_or(ExecutableValidationError::InvalidBranchTarget {
                    prototype: site.prototype,
                    instruction: instruction_index,
                    target: i64::MAX,
                })?;
                vec![validate_branch_target(site.prototype, instruction_index, prototype.code.len(), offset)?]
            }
            ControlFlowKindV1::ConditionalBranch => {
                let offset = relative_jump_offset(instruction).ok_or(ExecutableValidationError::InvalidBranchTarget {
                    prototype: site.prototype,
                    instruction: instruction_index,
                    target: i64::MAX,
                })?;
                let mut targets = vec![validate_branch_target(site.prototype, instruction_index, prototype.code.len(), offset)?];
                if let Some(next) = instruction_index.checked_add(1).filter(|target| *target < prototype.code.len()) {
                    targets.push(next);
                }
                targets
            }
            ControlFlowKindV1::FunctionExit | ControlFlowKindV1::NonLocalExit | ControlFlowKindV1::ExceptionalTermination => vec![],
        };
        for successor in successors {
            if successor == site.instruction {
                return Ok(true);
            }
            pending.push(successor);
        }
    }
    Ok(false)
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
    prototype_data: &PortablePrototypeV1,
) -> Result<(), ExecutableValidationError> {
    let debug_heights = (prototype == 56).then(|| {
        states
            .iter()
            .enumerate()
            .filter_map(|(index, state)| state.map(|state| (index, state.height, &prototype_data.code[index])))
            .collect::<Vec<_>>()
    });
    let Some(existing) = states.get_mut(target) else {
        return Err(ExecutableValidationError::InvalidBranchTarget {
            prototype,
            instruction,
            target: i64::try_from(target).unwrap_or(i64::MAX),
        });
    };
    match existing {
        Some(existing) if existing.height != incoming.height => {
            eprintln!(
                "stack-height mismatch prototype={prototype} owner={:?} name={:?} predecessor={instruction}:{:?} target={target}:{:?} existing={} incoming={}",
                prototype_data.owner,
                prototype_data.name,
                prototype_data.code.get(instruction),
                prototype_data.code.get(target),
                existing.height,
                incoming.height,
            );
            eprintln!("prototype {prototype} locals: {:#?}", prototype_data.local_names);
            eprintln!("prototype {prototype} reachable stack heights: {debug_heights:#?}");
            eprintln!("prototype {prototype} full code: {:#?}", prototype_data.code);
            Err(ExecutableValidationError::IncompatibleControlFlowState {
                prototype,
                instruction: target,
                existing: existing.height,
                incoming: incoming.height,
            })
        }
        Some(_) => Ok(()),
        None => {
            *existing = Some(incoming);
            pending.push(target);
            Ok(())
        }
    }
}

fn invalid_stack_transfer_classification(prototype: usize, instruction: usize, flow: ControlFlowKindV1) -> ExecutableValidationError {
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
            ControlFlowKindV1::Fallthrough | ControlFlowKindV1::FunctionExit | ControlFlowKindV1::NonLocalExit | ControlFlowKindV1::ExceptionalTermination => {
                false
            }
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
                enqueue_fallthrough(&mut pending, prototype_index, instruction_index, code_len, fallthrough_width(instruction))?;
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
                enqueue_fallthrough(&mut pending, prototype_index, instruction_index, code_len, 1)?;
            }
            ControlFlowKindV1::FunctionExit | ControlFlowKindV1::NonLocalExit | ControlFlowKindV1::ExceptionalTermination => {}
        }
    }
    Ok(())
}

fn enqueue_fallthrough(pending: &mut Vec<usize>, prototype: usize, instruction: usize, code_len: usize, width: usize) -> Result<(), ExecutableValidationError> {
    let target = instruction.checked_add(width);
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

fn fallthrough_width(instruction: &PortableInstructionV1) -> usize {
    match instruction {
        PortableInstructionV1::InvokeLocal { .. } | PortableInstructionV1::InvokeConst { .. } => 2,
        _ => 1,
    }
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
            validate_conformance_slots(pool_index, row, pool, context, &environment.conformance_slots, None)?;
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
            if invocation.selection_is_trait_default != trait_selection_identity_proves_default(invocation) {
                return Err(ExecutableValidationError::InvalidSemanticProduct {
                    pool: pool_index,
                    family: "trait invocation",
                    row,
                    detail: "trait-default execution mode disagrees with stable selected-callable identity".to_owned(),
                });
            }
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
    let callable = StableCallableRef {
        owner: StableCallableOwnerRef::Declaration(requirement.owner.clone()),
        side: requirement.side,
        selector: requirement.selector.clone().into_boxed_str(),
    };
    if !context.has_p7_module(&requirement.owner.module) || !context.trait_requirements.contains(&callable) {
        return Err(ExecutableValidationError::UnknownSemanticIdentity { pool, family, row });
    }
    Ok(())
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

fn trait_selection_identity_proves_default(
    invocation: &phalcom_compiler_artifact::executable::semantic::PortableTraitInvocationV1,
) -> bool {
    match &invocation.selection {
        PortableTraitSelectionTargetV1::Callable { callable } => {
            matches!(
                &callable.owner,
                StableCallableOwnerRef::Declaration(owner)
                    if owner == &invocation.requirement.owner
                        && callable.selector.as_ref() == invocation.requirement.selector
                        && callable.side == invocation.requirement.side
            )
        }
        PortableTraitSelectionTargetV1::DataComponent { .. } => false,
    }
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
    forward_limit: Option<usize>,
) -> Result<(), ExecutableValidationError> {
    for slot in slots {
        validate_conformance_slot(pool_index, row, pool, context, slot, forward_limit)?;
    }
    Ok(())
}

fn validate_conformance_slot(
    pool_index: usize,
    row: usize,
    pool: &PortableSemanticPoolV1,
    context: &ExecutableValidationContext<'_>,
    slot: &PortableConformanceSlotV1,
    forward_limit: Option<usize>,
) -> Result<(), ExecutableValidationError> {
    match slot {
        PortableConformanceSlotV1::Selection { callable, nested, .. } => {
            require_p7_callable(context, pool_index, "conformance selection", row, callable)?;
            validate_conformance_slot_entries(pool_index, row, pool, context, nested, forward_limit)
        }
        PortableConformanceSlotV1::Forwarded { slot } => {
            if forward_limit.is_some_and(|limit| *slot as usize >= limit) {
                return Err(ExecutableValidationError::InvalidSemanticProduct {
                    pool: pool_index,
                    family: "forwarded conformance slot",
                    row,
                    detail: format!("slot {slot} is outside the proven caller conformance environment"),
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
    forward_limit: Option<usize>,
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

    fn field_method_fixture(
        class_names: &[&str],
        is_static: bool,
        field_slot: u16,
        field_count: u32,
    ) -> (ExecutableImageV1, ExecutableValidationContext<'static>) {
        use phalcom_compiler_artifact::executable::constant::{
            MethodBodyV1, MethodOriginV1, MethodRecipeV1, MethodSignatureKindV1, MethodSignatureV1, VisibilityV1,
        };
        use phalcom_compiler_artifact::executable::instruction::PortableInstructionV1 as I;
        use phalcom_compiler_artifact::executable::module::{PortableClassRecipeV1, PortableFieldLayoutV1};
        use phalcom_compiler_artifact::executable::prototype::{ConstantId, ParameterShapeV1, PrototypeId, PrototypeOwnerV1, SemanticPoolId, SourceMapId};

        let module = test_module();
        let method_constant = class_names.len();
        let template_constant = method_constant + 1;
        let child_template_constant = method_constant + 2;
        let selector_constant = method_constant + 3;
        let mut constants = class_names
            .iter()
            .map(|name| ConstantRecipeV1::Symbol { text: (*name).to_owned() })
            .collect::<Vec<_>>();
        constants.push(ConstantRecipeV1::Method {
            recipe: MethodRecipeV1 {
                body: MethodBodyV1::Closure {
                    template: ConstantId(template_constant as u32),
                },
                signature: MethodSignatureV1 {
                    selector: "field".to_owned(),
                    kind: MethodSignatureKindV1::Getter,
                    positional_arity: 0,
                    rest: None,
                },
                holder: None,
                access_owner: None,
                visibility: VisibilityV1::Public,
                origin: MethodOriginV1 {
                    module: Some(module.clone()),
                    package: None,
                    project: None,
                },
                contracts: None,
                attributes: vec![],
                attributes_frozen: false,
            },
        });
        constants.push(ConstantRecipeV1::ClosureTemplate {
            prototype: PrototypeId(1),
            module: module.clone(),
            lexical_class: None,
            super_anchor: None,
        });
        constants.push(ConstantRecipeV1::ClosureTemplate {
            prototype: PrototypeId(2),
            module: module.clone(),
            lexical_class: None,
            super_anchor: None,
        });
        constants.push(ConstantRecipeV1::Symbol { text: "field".to_owned() });

        let mut root_constants = (0..class_names.len()).map(|index| ConstantId(index as u32)).collect::<Vec<_>>();
        root_constants.push(ConstantId(method_constant as u32));
        root_constants.push(ConstantId(selector_constant as u32));
        let selector_local = class_names.len() + 1;
        let method_local = class_names.len();
        let mut root_code = Vec::new();
        for (index, _) in class_names.iter().enumerate() {
            root_code.push(I::Nil {});
            root_code.push(I::Class { operand0: index as u16 });
            root_code.push(I::Constant { operand0: method_local as u16 });
            root_code.push(I::Method {
                operand0: selector_local as u16,
                operand1: is_static,
            });
            if index + 1 != class_names.len() {
                root_code.push(I::Pop {});
            }
        }
        root_code.push(I::Return {});

        let body = PortablePrototypeV1 {
            owner: PrototypeOwnerV1::Nested {
                parent: PrototypeId(0),
                ordinal: 17,
            },
            code: vec![
                I::Closure { operand0: 0 },
                I::Pop {},
                I::GetSelf {},
                I::GetField { operand0: field_slot },
                I::Return {},
            ],
            constants: vec![ConstantId(child_template_constant as u32)],
            max_slots: 1,
            arity: 0,
            upvalues: vec![],
            parameters: ParameterShapeV1 {
                fixed_positionals: 0,
                fixed_labels: vec![],
                rest: None,
            },
            name: "field".to_owned(),
            local_names: vec![],
            source: SourceMapId(0),
            semantics: SemanticPoolId(0),
            spans: vec![],
        };
        let root = PortablePrototypeV1 {
            owner: PrototypeOwnerV1::ModuleInitializer { module: module.clone() },
            code: root_code,
            constants: root_constants,
            max_slots: 0,
            arity: 0,
            upvalues: vec![],
            parameters: ParameterShapeV1 {
                fixed_positionals: 0,
                fixed_labels: vec![],
                rest: None,
            },
            name: "<module>".to_owned(),
            local_names: vec![],
            source: SourceMapId(0),
            semantics: SemanticPoolId(0),
            spans: vec![],
        };
        let child = PortablePrototypeV1 {
            owner: PrototypeOwnerV1::Nested {
                parent: PrototypeId(1),
                ordinal: 0,
            },
            code: vec![I::GetSelf {}, I::GetField { operand0: field_slot }, I::Return {}],
            constants: vec![],
            max_slots: 1,
            arity: 0,
            upvalues: vec![],
            parameters: ParameterShapeV1 {
                fixed_positionals: 0,
                fixed_labels: vec![],
                rest: None,
            },
            name: "field block".to_owned(),
            local_names: vec![],
            source: SourceMapId(0),
            semantics: SemanticPoolId(0),
            spans: vec![],
        };
        let mut image = empty_image();
        image.prototypes = vec![root, body, child];
        image.constants = constants;
        let mut context = ExecutableValidationContext::default();
        for name in class_names {
            let declaration = StableDeclarationRef {
                module: module.clone(),
                path: vec![(*name).into()].into(),
            };
            context.class_declarations.insert(declaration.clone());
            for side in [StableDispatchSide::Instance, StableDispatchSide::Class] {
                let count = if (side == StableDispatchSide::Class) == is_static { field_count } else { 0 };
                image.classes.push(PortableClassRecipeV1 {
                    class: StableClassRefV1 {
                        declaration: declaration.clone(),
                        side,
                    },
                    allocation: PortableClassAllocationV1::SourceDeclared,
                    superclass: None,
                    metaclass: None,
                    fields: if count == 0 {
                        vec![]
                    } else {
                        vec![PortableFieldLayoutV1 {
                            name: "field".to_owned(),
                            slot: 0,
                        }]
                    },
                    field_count: count,
                    immutable_slots: vec![],
                    native_repr: false,
                    sealed: false,
                    is_abstract: false,
                });
            }
        }
        (image, context)
    }

    #[test]
    fn method_recipe_validates_flattened_body_parameter_slots() {
        use phalcom_compiler_artifact::executable::constant::{ConstantRecipeV1, MethodSignatureKindV1};

        let (mut image, _) = field_method_fixture(&[], false, 0, 0);
        let ConstantRecipeV1::Method { recipe } = &mut image.constants[0] else {
            panic!("fixture method recipe");
        };
        recipe.signature.selector = "combine(left,right)".to_owned();
        recipe.signature.kind = MethodSignatureKindV1::Method { slots: 2 };
        recipe.signature.positional_arity = 0;
        image.prototypes[1].parameters.fixed_positionals = 2;

        assert!(validate_method_recipes(&image).is_ok());

        image.prototypes[1].parameters.fixed_positionals = 1;
        assert!(matches!(
            validate_method_recipes(&image),
            Err(ExecutableValidationError::InvalidExecutableReference {
                table: "method recipes",
                detail,
                ..
            }) if detail.contains("frame parameters disagree")
        ));
    }

    #[test]
    fn method_installation_authorizes_nested_field_receiver() {
        let (image, context) = field_method_fixture(&["Foo"], false, 0, 1);
        validate_executable_provenance(&image, &context).unwrap();
    }

    #[test]
    fn class_side_method_uses_static_portable_field_layout() {
        let (image, context) = field_method_fixture(&["Foo"], true, 0, 1);
        validate_executable_provenance(&image, &context).unwrap();
    }

    #[test]
    fn enum_instruction_proves_exact_class_receiver_for_method_install() {
        use phalcom_compiler_artifact::executable::instruction::PortableInstructionV1 as I;
        use phalcom_compiler_artifact::executable::prototype::SemanticPoolId;
        use phalcom_compiler_artifact::executable::semantic::{
            PortableAdtRepresentationV1, PortableSemanticPoolV1, PortableEnumSpecV1,
        };

        let (mut image, mut context) = field_method_fixture(&["Foo"], true, 0, 1);
        let declaration = image.classes[0].class.declaration.clone();
        context.class_declarations.remove(&declaration);
        context.enum_variants.insert(declaration.clone(), vec![]);
        image.prototypes[0].code = vec![
            I::Enum { operand0: 0 },
            I::Constant { operand0: 1 },
            I::Method {
                operand0: 2,
                operand1: true,
            },
            I::Return {},
        ];
        image.prototypes[0].semantics = SemanticPoolId(0);
        image.semantic_pools = vec![PortableSemanticPoolV1 {
            type_recipes: vec![],
            kind_recipes: vec![],
            enum_specs: vec![PortableEnumSpecV1 {
                owner: declaration,
                representation: PortableAdtRepresentationV1::TaggedHeap,
                variants: vec![],
            }],
            variant_targets: vec![],
            data_specs: vec![],
            data_targets: vec![],
            data_constructions: vec![],
            anonymous_products: vec![],
            call_environments: vec![],
            associated_targets: vec![],
            family_descriptors: vec![],
            conditional_families: vec![],
            family_operations: vec![],
            family_candidate_sets: vec![],
            trait_invocations: vec![],
            trait_requirement_invocations: vec![],
            type_parameter_class_receivers: vec![],
            provenance: None,
        }];

        validate_executable_provenance(&image, &context).unwrap();
    }

    #[test]
    fn data_instruction_proves_exact_class_receiver_for_method_install() {
        use phalcom_compiler_artifact::executable::instruction::PortableInstructionV1 as I;
        use phalcom_compiler_artifact::executable::prototype::SemanticPoolId;
        use phalcom_compiler_artifact::executable::semantic::{PortableDataSpecV1, PortableSemanticPoolV1};

        let (mut image, mut context) = field_method_fixture(&["Foo"], true, 0, 1);
        let declaration = image.classes[0].class.declaration.clone();
        context.class_declarations.remove(&declaration);
        context.data_declarations.insert(declaration.clone());
        image.prototypes[0].code = vec![
            I::Data { operand0: 0 },
            I::Constant { operand0: 1 },
            I::Method {
                operand0: 2,
                operand1: true,
            },
            I::Return {},
        ];
        image.prototypes[0].semantics = SemanticPoolId(0);
        image.semantic_pools = vec![PortableSemanticPoolV1 {
            type_recipes: vec![],
            kind_recipes: vec![],
            enum_specs: vec![],
            variant_targets: vec![],
            data_specs: vec![PortableDataSpecV1 {
                owner: declaration,
                layout: None,
                components: vec![],
            }],
            data_targets: vec![],
            data_constructions: vec![],
            anonymous_products: vec![],
            call_environments: vec![],
            associated_targets: vec![],
            family_descriptors: vec![],
            conditional_families: vec![],
            family_operations: vec![],
            family_candidate_sets: vec![],
            trait_invocations: vec![],
            trait_requirement_invocations: vec![],
            type_parameter_class_receivers: vec![],
            provenance: None,
        }];

        validate_executable_provenance(&image, &context).unwrap();
    }

    #[test]
    fn scratch_local_release_preserves_exact_class_receiver_provenance() {
        use phalcom_compiler_artifact::executable::instruction::PortableInstructionV1 as I;

        let (mut image, context) = field_method_fixture(&["Foo"], false, 0, 1);
        image.prototypes[0].code = vec![
            I::Nil {},
            I::Class { operand0: 0 },
            I::ReserveScratchLocal { operand0: 0 },
            I::ReleaseScratchLocal { operand0: 0 },
            I::Constant { operand0: 1 },
            I::Method {
                operand0: 2,
                operand1: false,
            },
            I::Return {},
        ];

        validate_executable_provenance(&image, &context).unwrap();
    }

    #[test]
    fn unknown_method_installation_receiver_rejects_field_authority() {
        let (mut image, context) = field_method_fixture(&["Missing"], false, 0, 1);
        image.constants[0] = ConstantRecipeV1::Symbol { text: "Absent".to_owned() };
        assert!(matches!(
            validate_executable_provenance(&image, &context),
            Err(ExecutableValidationError::UnknownMethodInstallationReceiver { body_prototype: Some(1), .. })
        ));
    }

    #[test]
    fn conflicting_method_installations_reject_shared_body() {
        let (image, context) = field_method_fixture(&["Foo", "Bar"], false, 0, 1);
        assert!(matches!(
            validate_executable_provenance(&image, &context),
            Err(ExecutableValidationError::ConflictingMethodInstallation { body_prototype: Some(1), .. })
        ));
    }

    #[test]
    fn field_slot_equal_to_field_count_is_rejected() {
        let (image, context) = field_method_fixture(&["Foo"], false, 1, 1);
        let result = validate_executable_provenance(&image, &context);
        assert!(
            matches!(result, Err(ExecutableValidationError::FieldSlotOutOfRange { slot: 1, field_count: 1, .. })),
            "unexpected result: {result:?}"
        );
    }

    #[test]
    fn forged_closure_creation_parent_cannot_borrow_another_frames_captures() {
        use phalcom_compiler_artifact::executable::instruction::PortableInstructionV1 as I;
        use phalcom_compiler_artifact::executable::prototype::PrototypeId;

        let (mut image, context) = field_method_fixture(&["Foo"], false, 0, 1);
        image.prototypes[2].owner = PrototypeOwnerV1::Nested { parent: PrototypeId(0), ordinal: 0 };
        image.prototypes[2].upvalues = vec![UpvalueCaptureV1::Local(0)];
        image.prototypes[2].code = vec![I::GetUpvalue { operand0: 0 }, I::GetField { operand0: 0 }, I::Return {}];

        assert!(matches!(
            validate_executable_provenance(&image, &context),
            Err(ExecutableValidationError::InvalidClosureCapture { child: 2, parent: 1, .. })
        ));
    }

    #[test]
    fn closure_capture_proves_exact_instance_receiver_upvalue() {
        use phalcom_compiler_artifact::executable::instruction::PortableInstructionV1 as I;

        let (mut image, context) = field_method_fixture(&["Foo"], false, 0, 1);
        image.prototypes[2].upvalues = vec![UpvalueCaptureV1::Local(0)];
        image.prototypes[2].code = vec![I::GetUpvalue { operand0: 0 }, I::GetField { operand0: 0 }, I::Return {}];
        validate_executable_provenance(&image, &context).unwrap();
    }


    #[test]
    fn cyclic_closure_creation_preserves_immutable_exact_receiver_capture() {
        use phalcom_compiler_artifact::executable::instruction::PortableInstructionV1 as I;

        let (mut image, context) = field_method_fixture(&["Foo"], false, 0, 1);
        image.prototypes[1].code = vec![
            I::Closure { operand0: 0 },
            I::Pop {},
            I::True {},
            I::JumpIfFalse { operand0: 1 },
            I::Jump { operand0: -5 },
            I::GetSelf {},
            I::GetField { operand0: 0 },
            I::Return {},
        ];
        image.prototypes[2].upvalues = vec![UpvalueCaptureV1::Local(0)];
        image.prototypes[2].code = vec![I::GetUpvalue { operand0: 0 }, I::GetField { operand0: 0 }, I::Return {}];

        validate_executable_provenance(&image, &context).unwrap();
    }

    #[test]
    fn closure_capture_proves_exact_class_side_receiver_upvalue() {
        use phalcom_compiler_artifact::executable::instruction::PortableInstructionV1 as I;

        let (mut image, context) = field_method_fixture(&["Foo"], true, 0, 1);
        image.prototypes[2].upvalues = vec![UpvalueCaptureV1::Local(0)];
        // Checked reads name their binding through the child's local
        // constant table; keep the fixture valid at that structural boundary.
        let name = *image.prototypes[0].constants.last().unwrap();
        image.prototypes[2].constants = vec![name];
        image.prototypes[2].code = vec![I::GetUpvalueChecked { operand0: 0, operand1: 0 }, I::GetField { operand0: 0 }, I::Return {}];
        validate_executable_provenance(&image, &context).unwrap();
    }

    #[test]
    fn nested_forwarded_upvalue_preserves_exact_receiver() {
        use phalcom_compiler_artifact::executable::instruction::PortableInstructionV1 as I;
        use phalcom_compiler_artifact::executable::prototype::{ConstantId, PrototypeId};

        let (mut image, context) = field_method_fixture(&["Foo"], false, 0, 1);
        image.prototypes[2].upvalues = vec![UpvalueCaptureV1::Local(0)];
        let grandchild_constant = ConstantId(image.constants.len() as u32);
        image.constants.push(ConstantRecipeV1::ClosureTemplate {
            prototype: PrototypeId(3),
            module: test_module(),
            lexical_class: None,
            super_anchor: None,
        });
        image.prototypes[2].constants = vec![grandchild_constant];
        image.prototypes[2].code = vec![
            I::Closure { operand0: 0 },
            I::Pop {},
            I::GetUpvalue { operand0: 0 },
            I::GetField { operand0: 0 },
            I::Return {},
        ];
        let mut grandchild = image.prototypes[2].clone();
        grandchild.owner = PrototypeOwnerV1::Nested { parent: PrototypeId(2), ordinal: 0 };
        grandchild.upvalues = vec![UpvalueCaptureV1::Upvalue(0)];
        grandchild.constants.clear();
        grandchild.code = vec![I::GetUpvalue { operand0: 0 }, I::GetField { operand0: 0 }, I::Return {}];
        image.prototypes.push(grandchild);

        validate_executable_provenance(&image, &context).unwrap();
    }

    #[test]
    fn closure_capture_rejects_unknown_local_receiver() {
        use phalcom_compiler_artifact::executable::instruction::PortableInstructionV1 as I;

        let (mut image, context) = field_method_fixture(&["Foo"], false, 0, 1);
        image.prototypes[1].code = vec![
            I::Nil {},
            I::SetLocal { operand0: 0 },
            I::Closure { operand0: 0 },
            I::Pop {},
            I::GetSelf {},
            I::GetField { operand0: 0 },
            I::Return {},
        ];
        image.prototypes[2].upvalues = vec![UpvalueCaptureV1::Local(0)];
        image.prototypes[2].code = vec![I::GetUpvalue { operand0: 0 }, I::GetField { operand0: 0 }, I::Return {}];

        assert!(matches!(
            validate_executable_provenance(&image, &context),
            Err(ExecutableValidationError::UnknownFieldReceiver { prototype: 2, .. })
        ));
    }

    #[test]
    fn closure_capture_rejects_parent_write_after_creation() {
        use phalcom_compiler_artifact::executable::instruction::PortableInstructionV1 as I;

        let (mut image, context) = field_method_fixture(&["Foo"], false, 0, 1);
        image.prototypes[1].code = vec![
            I::Closure { operand0: 0 },
            I::Pop {},
            I::Nil {},
            I::SetLocal { operand0: 0 },
            I::GetSelf {},
            I::GetField { operand0: 0 },
            I::Return {},
        ];
        image.prototypes[2].upvalues = vec![UpvalueCaptureV1::Local(0)];
        image.prototypes[2].code = vec![I::GetUpvalue { operand0: 0 }, I::GetField { operand0: 0 }, I::Return {}];

        assert!(matches!(
            validate_executable_provenance(&image, &context),
            Err(ExecutableValidationError::UnknownFieldReceiver { prototype: 2, .. })
        ));
    }

    #[test]
    fn backward_branch_cannot_hide_post_capture_local_write() {
        use phalcom_compiler_artifact::executable::instruction::PortableInstructionV1 as I;

        let (mut image, context) = field_method_fixture(&["Foo"], false, 0, 1);
        // The write is textually before Closure, but control flow reaches it
        // afterwards without repeating the creation site.
        image.prototypes[1].code = vec![
            I::Jump { operand0: 3 },
            I::Nil {},
            I::SetLocal { operand0: 0 },
            I::Return {},
            I::Closure { operand0: 0 },
            I::Pop {},
            I::Jump { operand0: -6 },
        ];
        image.prototypes[2].upvalues = vec![UpvalueCaptureV1::Local(0)];
        image.prototypes[2].code = vec![I::GetUpvalue { operand0: 0 }, I::GetField { operand0: 0 }, I::Return {}];

        assert!(matches!(
            validate_executable_provenance(&image, &context),
            Err(ExecutableValidationError::UnknownFieldReceiver { prototype: 2, .. })
        ));
    }

    #[test]
    fn closure_capture_rejects_mutating_shared_upvalue_cell() {
        use phalcom_compiler_artifact::executable::instruction::PortableInstructionV1 as I;

        let (mut image, context) = field_method_fixture(&["Foo"], false, 0, 1);
        image.prototypes[2].upvalues = vec![UpvalueCaptureV1::Local(0)];
        image.prototypes[2].code = vec![
            I::Nil {},
            I::SetUpvalue { operand0: 0 },
            I::GetUpvalue { operand0: 0 },
            I::GetField { operand0: 0 },
            I::Return {},
        ];

        assert!(matches!(
            validate_executable_provenance(&image, &context),
            Err(ExecutableValidationError::UnknownFieldReceiver { prototype: 2, .. })
        ));
    }

    #[test]
    fn conflicting_captured_values_do_not_select_first_activation() {
        use phalcom_compiler_artifact::executable::prototype::UpvalueCaptureV1;
        let (mut image, _) = field_method_fixture(&["Foo", "Bar"], false, 0, 1);
        image.prototypes[2].upvalues = vec![UpvalueCaptureV1::Local(0)];
        let mut activations = vec![None::<PrototypeActivationProvenance>; image.prototypes.len()];
        let mut pending = std::collections::VecDeque::new();
        for (index, declaration) in image.classes.iter().filter(|row| row.class.side == StableDispatchSide::Instance).enumerate() {
            let incoming = PrototypeActivationProvenance {
                receiver: ProvenanceFact::Unknown,
                conformance_caller: ProvenanceFact::Unknown,
                upvalues: vec![ProvenanceFact::Exact(RuntimeValueProvenance::ClassObject {
                    declaration: declaration.class.declaration.clone(),
                    allocation: AllocationIdentity::Constant(index),
                })],
            };
            merge_activation(&mut activations, &mut pending, 2, incoming).unwrap();
        }
        assert!(matches!(
            activations[2].as_ref().unwrap().upvalues[0],
            ProvenanceFact::Conflict
        ));
    }

    #[test]
    fn closure_capture_field_slot_must_fit_proven_layout() {
        use phalcom_compiler_artifact::executable::instruction::PortableInstructionV1 as I;

        let (mut image, context) = field_method_fixture(&["Foo"], false, 0, 1);
        image.prototypes[2].upvalues = vec![UpvalueCaptureV1::Local(0)];
        image.prototypes[2].code = vec![I::GetUpvalue { operand0: 0 }, I::GetField { operand0: 1 }, I::Return {}];

        assert!(matches!(
            validate_executable_provenance(&image, &context),
            Err(ExecutableValidationError::FieldSlotOutOfRange {
                prototype: 2,
                slot: 1,
                field_count: 1,
                ..
            })
        ));
    }

    fn empty_image() -> ExecutableImageV1 {
        use phalcom_compiler_artifact::executable::{compatibility::ExecutableCompatibilityV1, module::PortableBootstrapPlanV1};
        ExecutableImageV1 {
            compatibility: ExecutableCompatibilityV1::current(CompilationProfileV1::Debug, &[], &[]).unwrap(),
            modules: vec![],
            prototypes: vec![],
            constants: vec![],
            semantic_pools: vec![],
            classes: vec![],
            native_anchors: vec![],
            primordial_roles: vec![],
            source_maps: vec![],
            detached_methods: vec![],
            bootstrap: PortableBootstrapPlanV1 {
                native_runtime_roles: vec![],
                eager_modules: vec![],
                deferred_modules: vec![],
                required_declarations: vec![],
            },
        }
    }

    fn test_module() -> StableModuleRef {
        StableModuleRef {
            project: phalcom_type_meta::identity::StableProjectRef::Builtin {
                namespace: "universe".into(),
                version: "0.1.0".into(),
            },
            path: vec!["object".into()].into(),
        }
    }

    #[test]
    fn fused_invokes_skip_dead_invoke_in_both_validator_traversals() {
        use PortableInstructionV1 as I;
        use phalcom_compiler_artifact::executable::prototype::{ParameterShapeV1, PrototypeId, SemanticPoolId, SourceMapId};
        let image = empty_image();
        let context = ExecutableValidationContext::default();
        for fused in [
            I::InvokeConst {
                operand0: 0,
                operand1: 0,
                operand2: 0,
            },
            I::InvokeLocal {
                operand0: 0,
                operand1: 0,
                operand2: 0,
            },
        ] {
            let mut prototype = PortablePrototypeV1 {
                owner: PrototypeOwnerV1::Nested {
                    parent: PrototypeId(0),
                    ordinal: 0,
                },
                code: vec![fused, I::Invoke { operand0: 255, operand1: 0 }, I::Return {}],
                constants: vec![],
                max_slots: 2,
                arity: 0,
                upvalues: vec![],
                parameters: ParameterShapeV1 {
                    fixed_positionals: 0,
                    fixed_labels: vec![],
                    rest: None,
                },
                name: "fused".into(),
                local_names: vec![],
                source: SourceMapId(0),
                semantics: SemanticPoolId(0),
                spans: vec![],
            };
            validate_prototype_control_flow(&image, &context, 0, &prototype).unwrap();
            validate_prototype_stack(&image, 0, &prototype, PrototypeActivationRoles::default()).unwrap();
            prototype.code[2] = I::Pop {};
            prototype.code.push(I::Pop {});
            prototype.code.push(I::Return {});
            assert!(matches!(
                validate_prototype_stack(&image, 0, &prototype, PrototypeActivationRoles::default()),
                Err(ExecutableValidationError::StackUnderflow { instruction: 3, .. })
            ));
            prototype.code.truncate(2);
            assert!(matches!(
                validate_prototype_control_flow(&image, &context, 0, &prototype),
                Err(ExecutableValidationError::InvalidFallthrough { instruction: 0, target: 2, .. })
            ));
        }
    }

    #[test]
    fn class_tower_accepts_metaclass_instances_and_object_class_superclass() {
        use phalcom_compiler_artifact::executable::module::PortableClassRecipeV1;
        use phalcom_modules::{interface::LinkedModuleInterface, linker::ModuleBindingLayout, source::ModuleKind};
        let module = test_module();
        let linked = LinkedModule {
            interface: LinkedModuleInterface {
                module: phalcom_semantic::metadata::stable_identity::from_stable_module(&module).unwrap(),
                kind: ModuleKind::Module,
                exports: Default::default(),
                metadata: Default::default(),
            },
            bindings: ModuleBindingLayout::default(),
            linked_reads: vec![],
            runtime_dependencies: vec![],
        };
        let mut context = ExecutableValidationContext::default();
        context.linked_modules.insert(module.clone(), &linked);
        let reference = |name: &str, side| StableClassRefV1 {
            declaration: StableDeclarationRef {
                module: module.clone(),
                path: vec![name.into()].into(),
            },
            side,
        };
        let object = reference("Object", StableDispatchSide::Instance);
        let object_class = reference("Object", StableDispatchSide::Class);
        let class = reference("Class", StableDispatchSide::Instance);
        let class_class = reference("Class", StableDispatchSide::Class);
        let meta = reference("Metaclass", StableDispatchSide::Instance);
        let meta_class = reference("Metaclass", StableDispatchSide::Class);
        let mut image = empty_image();
        for (class_ref, superclass, metaclass) in [
            (object.clone(), None, object_class.clone()),
            (object_class, Some(class.clone()), meta.clone()),
            (class.clone(), Some(object.clone()), class_class.clone()),
            (class_class, None, meta.clone()),
            (meta.clone(), Some(class), meta_class.clone()),
            (meta_class, None, meta),
        ] {
            context.declarations.insert(class_ref.declaration.clone());
            context.class_declarations.insert(class_ref.declaration.clone());
            image.classes.push(PortableClassRecipeV1 {
                class: class_ref,
                allocation: PortableClassAllocationV1::SourceDeclared,
                superclass,
                metaclass: Some(metaclass),
                fields: vec![],
                field_count: 0,
                immutable_slots: vec![],
                native_repr: false,
                sealed: false,
                is_abstract: false,
            });
        }
        validate_classes(&image, &context).unwrap();
        image.classes[1].metaclass.as_mut().unwrap().side = StableDispatchSide::Class;
        assert!(matches!(
            validate_classes(&image, &context),
            Err(ExecutableValidationError::InvalidClassLayout { .. })
        ));
        image.classes[1].metaclass.as_mut().unwrap().side = StableDispatchSide::Instance;
        image.classes[0].superclass = Some(object);
        assert!(matches!(validate_classes(&image, &context), Err(ExecutableValidationError::SuperclassCycle(_))));
    }

    #[test]
    fn validator_contract_uses_unpublished_p2_authority_and_returns_private_authority() {
        let _validator: fn(
            Arc<ExecutableImageV1>,
            &phalcom_modules::LinkedProgram,
            &phalcom_semantic::canonical_image::CanonicalUniverseFormalImage,
            &phalcom_type_meta::SemanticMetadataBundle,
            &ExecutableImageLimitsV1,
        ) -> Result<ValidatedExecutableImage, ExecutableValidationError> = validate_executable_image;
    }
    fn empty_semantic_pool() -> PortableSemanticPoolV1 {
        PortableSemanticPoolV1 {
            type_recipes: vec![],
            kind_recipes: vec![],
            enum_specs: vec![],
            variant_targets: vec![],
            data_specs: vec![],
            data_targets: vec![],
            data_constructions: vec![],
            anonymous_products: vec![],
            call_environments: vec![],
            associated_targets: vec![],
            family_descriptors: vec![],
            conditional_families: vec![],
            family_operations: vec![],
            family_candidate_sets: vec![],
            trait_invocations: vec![],
            trait_requirement_invocations: vec![],
            type_parameter_class_receivers: vec![],
            provenance: None,
        }
    }

    #[test]
    fn forwarded_slots_use_caller_bounds_including_nested_recipes() {
        let pool = empty_semantic_pool();
        let context = ExecutableValidationContext::default();
        let slots = [PortableConformanceSlotV1::Forwarded { slot: 26 }];
        // One callee output slot can legally forward caller slot 26.
        assert!(validate_conformance_slots(1446, 0, &pool, &context, &slots, Some(27)).is_ok());
        assert!(validate_conformance_slots(1446, 0, &pool, &context, &slots, Some(26)).is_err());
        assert!(validate_conformance_slots(1446, 0, &pool, &context, &slots, Some(0)).is_err());
        let nested = [PortableConformanceSlotEntryV1 {
            slot: 0,
            recipe: PortableConformanceSlotV1::Forwarded { slot: 26 },
        }];
        assert!(validate_conformance_slot_entries(1446, 0, &pool, &context, &nested, Some(27)).is_ok());
        assert!(validate_conformance_slot_entries(1446, 0, &pool, &context, &nested, Some(26)).is_err());
    }

    #[test]
    fn forwarding_requires_proven_consuming_activation() {
        use phalcom_compiler_artifact::executable::semantic::PortableCallEnvironmentV1;
        let (mut image, context) = field_method_fixture(&["Foo"], false, 0, 1);
        let pool = image.prototypes[0].semantics.0 as usize;
        image.semantic_pools.push(empty_semantic_pool());
        image.semantic_pools[pool].call_environments.push(PortableCallEnvironmentV1 {
            callable: StableCallableRef {
                owner: StableCallableOwnerRef::Declaration(image.classes[0].class.declaration.clone()),
                side: StableDispatchSide::Instance,
                selector: "callee".into(),
            },
            bindings: vec![],
            conformance_slots: vec![PortableConformanceSlotV1::Forwarded { slot: 0 }],
        });
        let mut activations = vec![None; image.prototypes.len()];
        assert!(validate_forwarded_conformance_authority(&image, &context, &activations).is_err());
        let mut activation = activation_with_unknown_captures(&image.prototypes[0], ProvenanceFact::Unknown);
        activation.conformance_caller = ProvenanceFact::Exact(None);
        activations[0] = Some(activation);
        assert!(validate_forwarded_conformance_authority(&image, &context, &activations).is_err());
        // No prototype may lend authority to an orphan semantic pool.
        image.prototypes.clear();
        assert!(validate_forwarded_conformance_authority(&image, &context, &[]).is_err());
    }

    #[test]
    fn conflicting_caller_activations_cannot_lend_forwarding_authority() {
        let (image, _) = field_method_fixture(&["Foo"], false, 0, 1);
        let mut activations = vec![None; image.prototypes.len()];
        let mut pending = std::collections::VecDeque::new();
        let mut method = activation_with_unknown_captures(&image.prototypes[1], ProvenanceFact::Unknown);
        method.conformance_caller = ProvenanceFact::Exact(Some(StableCallableRef {
            owner: StableCallableOwnerRef::Declaration(image.classes[0].class.declaration.clone()),
            side: StableDispatchSide::Instance,
            selector: "field".into(),
        }));
        let mut block = activation_with_unknown_captures(&image.prototypes[1], ProvenanceFact::Unknown);
        block.conformance_caller = ProvenanceFact::Exact(None);
        merge_activation(&mut activations, &mut pending, 1, method).unwrap();
        merge_activation(&mut activations, &mut pending, 1, block).unwrap();
        assert!(matches!(activations[1].as_ref().unwrap().conformance_caller, ProvenanceFact::Conflict));
    }

    #[test]
    fn trait_requirement_identity_uses_canonical_surface_authority() {
        let module = test_module();
        let owner = StableDeclarationRef {
            module: module.clone(),
            path: Box::new(["Iterable".into()]),
        };
        let requirement = PortableTraitRequirementV1 {
            owner: owner.clone(),
            selector: "iterator".into(),
            side: StableDispatchSide::Instance,
        };
        let identity = StableCallableRef {
            owner: StableCallableOwnerRef::Declaration(owner),
            side: requirement.side,
            selector: requirement.selector.clone().into_boxed_str(),
        };
        let mut context = ExecutableValidationContext::default();
        let linked = LinkedModule {
            interface: phalcom_modules::interface::LinkedModuleInterface {
                module: phalcom_semantic::metadata::stable_identity::from_stable_module(&module).unwrap(),
                kind: phalcom_modules::source::ModuleKind::Module,
                exports: Default::default(),
                metadata: Default::default(),
            },
            bindings: Default::default(),
            linked_reads: vec![],
            runtime_dependencies: vec![],
        };
        context.linked_modules.insert(module, &linked);
        context.trait_requirements.insert(identity.clone());
        // A persisted abstract requirement needs no runtime declaration or body.
        assert!(validate_trait_requirement(72, "trait requirement invocation", 0, &context, &requirement).is_ok());
        assert!(require_p7_callable(&context, 72, "call target", 0, &identity).is_err());

        let mut forged = requirement.clone();
        forged.selector = "next".into();
        assert!(validate_trait_requirement(72, "trait requirement invocation", 0, &context, &forged).is_err());
        forged = requirement.clone();
        forged.side = StableDispatchSide::Class;
        assert!(validate_trait_requirement(72, "trait requirement invocation", 0, &context, &forged).is_err());
        forged = requirement.clone();
        forged.owner.path = Box::new(["OtherTrait".into()]);
        assert!(validate_trait_requirement(72, "trait requirement invocation", 0, &context, &forged).is_err());

        context.linked_modules.clear();
        assert!(validate_trait_requirement(72, "trait requirement invocation", 0, &context, &requirement).is_err());
        context.linked_modules.insert(requirement.owner.module.clone(), &linked);

        // A callable record alone cannot authorize a trait requirement.
        context.trait_requirements.clear();
        context.callables.insert(identity);
        context.declarations.insert(requirement.owner.clone());
        assert!(validate_trait_requirement(72, "trait requirement invocation", 0, &context, &requirement).is_err());
    }

    #[test]
    fn trait_default_mode_is_derived_from_exact_stable_callable_identity() {
        use phalcom_compiler_artifact::executable::semantic::{
            PortableDataComponentRefV1, PortableTraitInvocationV1, PortableTraitRequirementV1, PortableTraitSelectionTargetV1,
        };
        use phalcom_type_meta::identity::{
            StableCallableOwnerRef, StableCallableRef, StableDeclarationRef, StableDispatchSide, StableImplRef, StableModuleRef, StableProjectRef,
        };

        let module = StableModuleRef {
            project: StableProjectRef::Builtin {
                namespace: "universe".into(),
                version: "0.1.0".into(),
            },
            path: Box::new(["traits".into()]),
        };
        let owner = StableDeclarationRef {
            module: module.clone(),
            path: Box::new(["Iterable".into()]),
        };
        let requirement = PortableTraitRequirementV1 {
            owner: owner.clone(),
            selector: "iterator".into(),
            side: StableDispatchSide::Instance,
        };
        let exact = StableCallableRef {
            owner: StableCallableOwnerRef::Declaration(owner.clone()),
            side: StableDispatchSide::Instance,
            selector: "iterator".into(),
        };
        let invocation = |selection| PortableTraitInvocationV1 {
            source_impl: StableImplRef {
                module: module.clone(),
                local_id: 0,
            },
            selection,
            selection_is_trait_default: true,
            requirement: requirement.clone(),
            requirement_slot: 0,
            requirement_slots: vec![],
            requirement_targets: vec![],
            antecedent_slots: vec![],
            type_bindings: vec![],
            call_environment: None,
        };

        assert!(trait_selection_identity_proves_default(&invocation(
            PortableTraitSelectionTargetV1::Callable { callable: exact.clone() }
        )));

        let mut wrong_selector = exact.clone();
        wrong_selector.selector = "next".into();
        assert!(!trait_selection_identity_proves_default(&invocation(
            PortableTraitSelectionTargetV1::Callable { callable: wrong_selector }
        )));

        let mut wrong_side = exact;
        wrong_side.side = StableDispatchSide::Class;
        assert!(!trait_selection_identity_proves_default(&invocation(
            PortableTraitSelectionTargetV1::Callable { callable: wrong_side }
        )));

        assert!(!trait_selection_identity_proves_default(&invocation(
            PortableTraitSelectionTargetV1::DataComponent {
                component: PortableDataComponentRefV1 {
                    owner,
                    index: 0,
                },
                specialized_type: phalcom_compiler_artifact::executable::semantic::PortableTypeRecipeId(0),
            }
        )));
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
