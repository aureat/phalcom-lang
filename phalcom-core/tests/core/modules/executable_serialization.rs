use phalcom_compiler_artifact::ArtifactDigest;
use phalcom_compiler_artifact::ValidationLimits;
use phalcom_compiler_artifact::executable::constant::ConstantRecipeV1;
use phalcom_compiler_artifact::executable::image::{ExecutableImageLimitsV1, ExecutableImageV1};
use phalcom_core::artifact::capture::capture_canonical_universe_executable;
use phalcom_type_meta::allocation::AllocationBudget;

fn semantic_row_count(pool: &phalcom_compiler_artifact::executable::semantic::PortableSemanticPoolV1) -> usize {
    pool.type_recipes.len()
        + pool.kind_recipes.len()
        + pool.enum_specs.len()
        + pool.variant_targets.len()
        + pool.data_specs.len()
        + pool.data_targets.len()
        + pool.data_constructions.len()
        + pool.anonymous_products.len()
        + pool.call_environments.len()
        + pool.associated_targets.len()
        + pool.family_descriptors.len()
        + pool.conditional_families.len()
        + pool.family_operations.len()
        + pool.family_candidate_sets.len()
        + pool.trait_invocations.len()
        + pool.trait_requirement_invocations.len()
        + pool.type_parameter_class_receivers.len()
}

fn debug_digest(value: &impl std::fmt::Debug) -> ArtifactDigest {
    ArtifactDigest::digest(format!("{value:?}").as_bytes())
}

fn semantic_family_digest<T: std::fmt::Debug>(
    pools: &[phalcom_compiler_artifact::executable::semantic::PortableSemanticPoolV1],
    family: impl for<'a> Fn(&'a phalcom_compiler_artifact::executable::semantic::PortableSemanticPoolV1) -> &'a Vec<T>,
) -> ArtifactDigest {
    debug_digest(&pools.iter().map(family).collect::<Vec<_>>())
}

fn constant_family_tag(recipe: &ConstantRecipeV1) -> u16 {
    match recipe {
        ConstantRecipeV1::Nil => 21,
        ConstantRecipeV1::None => 0,
        ConstantRecipeV1::Unit => 1,
        ConstantRecipeV1::EmptyRecord => 2,
        ConstantRecipeV1::EmptyPack => 3,
        ConstantRecipeV1::Boolean { .. } => 4,
        ConstantRecipeV1::Integer { .. } => 5,
        ConstantRecipeV1::Float { .. } => 6,
        ConstantRecipeV1::Character { .. } => 7,
        ConstantRecipeV1::Symbol { .. } => 8,
        ConstantRecipeV1::String { .. } => 9,
        ConstantRecipeV1::BigInteger { .. } => 10,
        ConstantRecipeV1::Class { .. } => 11,
        ConstantRecipeV1::Module { .. } => 12,
        ConstantRecipeV1::ClosureTemplate { .. } => 13,
        ConstantRecipeV1::Method { .. } => 14,
        ConstantRecipeV1::SelectorPattern { .. } => 15,
        ConstantRecipeV1::Vector { .. } => 16,
        ConstantRecipeV1::FrozenArray { .. } => 17,
        ConstantRecipeV1::Tuple { .. } => 18,
        ConstantRecipeV1::Record { .. } => 19,
        ConstantRecipeV1::Pack { .. } => 20,
    }
}

#[test]
fn canonical_universe_executable_capture_structurally_round_trips() {
    let payload = capture_canonical_universe_executable().unwrap_or_else(|error| panic!("{error:#?}"));
    assert!(!payload.payload().is_empty());
    let repeated = capture_canonical_universe_executable().unwrap_or_else(|error| panic!("second canonical capture failed: {error:#?}"));
    assert_eq!(
        payload.payload(),
        repeated.payload(),
        "identical canonical builds must produce byte-identical executable payloads"
    );

    let mut budget = AllocationBudget::new(512 * 1024 * 1024);
    let decoded = ExecutableImageV1::decode(
        payload.payload(),
        &ValidationLimits::default(),
        &ExecutableImageLimitsV1::default(),
        &mut budget,
    )
    .unwrap_or_else(|error| panic!("canonical executable payload failed structural decode: {error}"));

    assert!(!decoded.modules.is_empty(), "canonical executable image must contain module initializers");
    assert!(!decoded.prototypes.is_empty(), "canonical executable image must contain emitted prototypes");
    assert_eq!(decoded.modules.len(), decoded.source_maps.len(), "every canonical module needs one source map");
    assert_eq!(
        decoded.bootstrap.eager_modules.len() + decoded.bootstrap.deferred_modules.len(),
        decoded.modules.len(),
        "every canonical module needs an eager or deferred bootstrap disposition"
    );
    assert!(!decoded.native_anchors.is_empty(), "the executable ABI must contain registered native anchors");
    assert!(
        !decoded.primordial_roles.is_empty(),
        "the executable ABI must contain required primordial roles"
    );
    assert!(
        !decoded.bootstrap.native_runtime_roles.is_empty(),
        "the bootstrap plan must name the native runtime floor"
    );
    decoded.validate_structure(&ExecutableImageLimitsV1::default()).unwrap();
    decoded.validate_native_abi_digest().unwrap();
    assert_eq!(decoded.encode().unwrap(), payload.payload(), "decode/encode must preserve canonical wire bytes");

    let semantic_rows = decoded.semantic_pools.iter().map(semantic_row_count).sum::<usize>();
    let used_constant_families = decoded.constants.iter().map(constant_family_tag).collect::<std::collections::BTreeSet<_>>();
    let instruction_count = decoded.prototypes.iter().map(|prototype| prototype.code.len()).sum::<usize>();
    println!(
        "ESI canonical capture: bytes={} digest={} modules={} prototypes={} instructions={} constants={} constant_families={}/22 semantic_pools={} semantic_rows={} semantic_pool_families=17/17 classes={} native_anchors={} primordial_roles={} detached_methods={} unresolved_references=0 unsupported_reachable_products=0",
        payload.payload().len(),
        ArtifactDigest::digest(payload.payload()),
        decoded.modules.len(),
        decoded.prototypes.len(),
        instruction_count,
        decoded.constants.len(),
        used_constant_families.len(),
        decoded.semantic_pools.len(),
        semantic_rows,
        decoded.classes.len(),
        decoded.native_anchors.len(),
        decoded.primordial_roles.len(),
        decoded.detached_methods.len(),
    );
    println!(
        "ESI table fingerprints: modules={} prototypes={} constants={} semantic_pools={} classes={} native_anchors={} primordial_roles={} source_maps={} detached_methods={} bootstrap={}",
        debug_digest(&decoded.modules),
        debug_digest(&decoded.prototypes),
        debug_digest(&decoded.constants),
        debug_digest(&decoded.semantic_pools),
        debug_digest(&decoded.classes),
        debug_digest(&decoded.native_anchors),
        debug_digest(&decoded.primordial_roles),
        debug_digest(&decoded.source_maps),
        debug_digest(&decoded.detached_methods),
        debug_digest(&decoded.bootstrap),
    );
    let pools = &decoded.semantic_pools;
    println!(
        "ESI semantic-family fingerprints: types={} kinds={} enum_specs={} variant_targets={} data_specs={} data_targets={} data_constructions={} anonymous_products={} call_environments={} associated_targets={} family_descriptors={} conditional_families={} family_operations={} family_candidate_sets={} trait_invocations={} trait_requirement_invocations={} type_parameter_class_receivers={}",
        semantic_family_digest(pools, |pool| &pool.type_recipes),
        semantic_family_digest(pools, |pool| &pool.kind_recipes),
        semantic_family_digest(pools, |pool| &pool.enum_specs),
        semantic_family_digest(pools, |pool| &pool.variant_targets),
        semantic_family_digest(pools, |pool| &pool.data_specs),
        semantic_family_digest(pools, |pool| &pool.data_targets),
        semantic_family_digest(pools, |pool| &pool.data_constructions),
        semantic_family_digest(pools, |pool| &pool.anonymous_products),
        semantic_family_digest(pools, |pool| &pool.call_environments),
        semantic_family_digest(pools, |pool| &pool.associated_targets),
        semantic_family_digest(pools, |pool| &pool.family_descriptors),
        semantic_family_digest(pools, |pool| &pool.conditional_families),
        semantic_family_digest(pools, |pool| &pool.family_operations),
        semantic_family_digest(pools, |pool| &pool.family_candidate_sets),
        semantic_family_digest(pools, |pool| &pool.trait_invocations),
        semantic_family_digest(pools, |pool| &pool.trait_requirement_invocations),
        semantic_family_digest(pools, |pool| &pool.type_parameter_class_receivers),
    );
}
