use phalcom_compiler_artifact::query::{
    PortableQueryKeyV1, PortableQueryKindV1, PortableQueryOwnerKeyV1, QueryImageV3, QueryProductRefV2, QueryRootEntryV3,
};
use phalcom_compiler_artifact::semantic::aggregates::{DataInfoV1, DataShapeV1, StructuralPrimaryContractsV1, VisibilityV1};
use phalcom_compiler_artifact::semantic::static_value::{StaticValueGraphV1, StaticValueV1};
use phalcom_compiler_artifact::semantic::type_store::{ArtifactTypeId, TypeGraphV1};
use phalcom_compiler_artifact::semantic::world::SemanticWorldV3;
use phalcom_type_meta::fingerprint::Fingerprint128;
use phalcom_type_meta::identity::{StableDeclarationRef, StableModuleRef, StableProjectRef};

fn world_with_data(owner: StableDeclarationRef) -> SemanticWorldV3 {
    SemanticWorldV3 {
        type_graph: TypeGraphV1 {
            types: vec![],
            kinds: vec![],
            parameters: vec![],
            current_parameters: vec![],
            rows: vec![],
            families: vec![],
            variants: vec![],
            static_domains: vec![],
            static_constructors: vec![],
            scoped: vec![],
            lambdas: vec![],
            variance_overrides: vec![],
        },
        structural_contracts: StructuralPrimaryContractsV1 {
            declaration_shells: vec![],
            hierarchy_and_aliases: vec![],
            generic_signatures: vec![],
            data_contracts: vec![DataInfoV1 {
                owner,
                root_form: ArtifactTypeId(0),
                generic_signature: None,
                static_data: None,
                shape: DataShapeV1::Tuple,
                components: vec![],
                constructor_visibility: VisibilityV1::Public,
                constructor_parameters: vec![],
                constructor_result_type: ArtifactTypeId(0),
                shape_contributions: vec![],
            }],
            enum_contracts: vec![],
            variant_contracts: vec![],
        },
        enum_requirements: vec![],
        static_values: StaticValueGraphV1 {
            values: vec![StaticValueV1::Unit],
            strings: vec![],
            symbols: vec![],
            tuples: vec![],
            sequences: vec![],
            records: vec![],
            data_values: vec![],
            variant_values: vec![],
            pack_backings: vec![],
            packs: vec![],
        },
        static_definitions: vec![],
        static_constants: vec![],
        module_static_bindings: vec![],
        witness_plans: vec![],
        derived_conformance_rules: vec![],
        callable_signatures: vec![],
        callable_definitions: vec![],
        callable_analyses: vec![],
        trait_surfaces: vec![],
        dispatch_surfaces: vec![],
        conditional_inherent_sets: vec![],
        conformance_heads: vec![],
        constructor_initializations: vec![],
        type_member_declarations: vec![],
        associated_static_domains: vec![],
        static_case_catalogs: vec![],
        static_case_signatures: vec![],
        associated_surfaces: vec![],
    }
}

#[test]
fn structural_query_fingerprint_is_portable_stable_owner_evidence() {
    let project = StableProjectRef::Package {
        package: "portable".into(),
        version: "1.0.0".into(),
        artifact_fingerprint: Fingerprint128([0x33; 16]),
    };
    let owner = StableDeclarationRef {
        module: StableModuleRef {
            project,
            path: vec!["model".into()].into_boxed_slice(),
        },
        path: vec!["Payload".into()].into_boxed_slice(),
    };
    let mut world = world_with_data(owner.clone());
    let mut image = QueryImageV3 {
        supported_roots: vec![QueryRootEntryV3 {
            key: PortableQueryKeyV1 {
                kind: PortableQueryKindV1::DataDeclaration,
                owner: PortableQueryOwnerKeyV1::Declaration(owner),
            },
            product: QueryProductRefV2 {
                kind: PortableQueryKindV1::DataDeclaration,
                row_index: 0,
            },
            input_fingerprint: 0,
            product_fingerprint: 0,
            dependency_indices: vec![],
        }],
        dependency_evidence: vec![],
    };

    image
        .refresh_portable_structural_fingerprints(&world)
        .expect("producer fingerprint refresh");
    let root = &image.supported_roots[0];
    assert_ne!(root.input_fingerprint, 0);
    assert_ne!(root.product_fingerprint, 0);
    assert_ne!(root.input_fingerprint, root.product_fingerprint, "input and product domains are separated");
    image
        .validate_portable_structural_fingerprints(&world)
        .expect("fresh portable fingerprints validate");

    world.structural_contracts.data_contracts[0].constructor_visibility = VisibilityV1::Private;
    assert!(
        image.validate_portable_structural_fingerprints(&world).is_err(),
        "semantic-row changes must invalidate stored portable query evidence"
    );
}
