use phalcom_compiler_artifact::module_world::SpanV1;
use phalcom_compiler_artifact::semantic::aggregates::{
    DeclaredTypeBasisV1, DeclaredTypeFactV1, DeclaredTypeStateV1, EnumInfoV1, PayloadKindV1, StructuralPrimaryContractsV1, UnknownReasonV1,
    VariantInfoV1, VariantShapeV1, VariantVisibilityV1, VisibilityV1,
};
use phalcom_compiler_artifact::semantic::operational::CheckedStaticDefinitionsV1;
use phalcom_compiler_artifact::semantic::static_value::StaticValueGraphV1;
use phalcom_compiler_artifact::semantic::type_store::{ArtifactTypeId, ArtifactVariantTypeId, TypeGraphV1};
use phalcom_compiler_artifact::semantic::world::{
    CallableParameterV1, CallableSignatureRecordV1, CaseRequirementResultV1, CaseRequirementStatusV1, EffectSpecV1, EnumRequirementOwnerV1,
    EnumRequirementRecordV1, EnumRequirementSourceV1, ImplementationKindV1, NativeLifecycleSpecV1, RaisesSpecV1, ReturnContractValidationV1,
    ReturnFlowV1, SemanticWorldV2,
};
use phalcom_type_meta::identity::{StableCallableOwnerRef, StableCallableRef, StableDeclarationRef, StableDispatchSide, StableModuleRef, StableProjectRef, StableVariantRef};

fn declaration() -> StableDeclarationRef {
    StableDeclarationRef {
        module: StableModuleRef {
            project: StableProjectRef::Builtin {
                namespace: "universe".into(),
                version: "0.1.0".into(),
            },
            path: vec!["test".into()].into_boxed_slice(),
        },
        path: vec!["Shape".into()].into_boxed_slice(),
    }
}

fn variant(name: &str) -> StableVariantRef {
    StableVariantRef {
        owner: declaration(),
        selector: name.into(),
    }
}

fn implementation(variant: StableVariantRef, selector: &str) -> StableCallableRef {
    StableCallableRef {
        owner: StableCallableOwnerRef::Variant(variant),
        side: StableDispatchSide::Instance,
        selector: selector.into(),
    }
}

fn root_requirement(selector: &str) -> StableCallableRef {
    StableCallableRef {
        owner: StableCallableOwnerRef::Declaration(declaration()),
        side: StableDispatchSide::Instance,
        selector: selector.into(),
    }
}

fn callable_signature(callable: StableCallableRef) -> CallableSignatureRecordV1 {
    CallableSignatureRecordV1 {
        owner: declaration(),
        side: 0,
        selector: callable.selector.clone(),
        callable,
        generics: None,
        parameters: Vec::<CallableParameterV1>::new(),
        declared_return: DeclaredTypeFactV1 {
            state: DeclaredTypeStateV1::Unknown {
                reason: UnknownReasonV1::NoTypeEvidence,
            },
            basis: DeclaredTypeBasisV1::Unspecified,
        },
        return_validation: ReturnContractValidationV1::NotApplicable,
        inferred_return: None,
        source: None,
        implementation: ImplementationKindV1::Phalcom,
        native_id: None,
        effects: EffectSpecV1::Unknown,
        raises: RaisesSpecV1::Unknown,
        flow: ReturnFlowV1::Value,
        lifecycle: NativeLifecycleSpecV1 {
            since: None,
            deprecated_since: None,
            replacement: None,
        },
    }
}

fn world(enum_requirements: Vec<EnumRequirementOwnerV1>) -> SemanticWorldV2 {
    let mut callable_refs = Vec::new();
    for requirement_owner in &enum_requirements {
        callable_refs.extend(requirement_owner.requirements.iter().map(|requirement| requirement.requirement.clone()));
        for result in &requirement_owner.case_statuses {
            if let CaseRequirementStatusV1::Satisfied { implementation } | CaseRequirementStatusV1::Incompatible { implementation } = &result.status {
                callable_refs.push(implementation.clone());
            }
        }
    }
    callable_refs.sort_unstable();
    callable_refs.dedup();
    let callable_signatures = callable_refs.into_iter().map(callable_signature).collect();
    let variants = vec![variant("Circle"), variant("Square")];
    let structural_contracts = StructuralPrimaryContractsV1 {
        declaration_shells: vec![],
        hierarchy_and_aliases: vec![],
        generic_signatures: vec![],
        data_contracts: vec![],
        enum_contracts: vec![EnumInfoV1 {
            owner: declaration(),
            root_form: ArtifactTypeId(0),
            generic_signature: None,
            static_data: None,
            default_result_type: ArtifactTypeId(0),
            variants: variants.clone(),
        }],
        variant_contracts: variants
            .iter()
            .enumerate()
            .map(|(index, variant)| VariantInfoV1 {
                variant: variant.clone(),
                variant_type: ArtifactVariantTypeId(index as u32),
                shape: VariantShapeV1::Singleton,
                payload_kind: PayloadKindV1::Singleton,
                fields: vec![],
                result_type: ArtifactTypeId(0),
                exact_case_type: ArtifactTypeId(0),
                case_environment: phalcom_compiler_artifact::semantic::aggregates::CaseEnvironmentV1 {
                    bindings: vec![],
                    equalities: vec![],
                },
                constructor: None,
                visibility: VariantVisibilityV1 {
                    name: VisibilityV1::Public,
                    construct: VisibilityV1::Public,
                    payload: VisibilityV1::Public,
                },
            })
            .collect(),
    };
    SemanticWorldV2 {
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
        structural_contracts,
        static_values: StaticValueGraphV1 {
            values: vec![],
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
        checked_definitions: CheckedStaticDefinitionsV1 { definitions: vec![] },
        witness_plans: vec![],
        callable_signatures,
        callable_analyses: vec![],
        trait_surfaces: vec![],
        dispatch_surfaces: vec![],
        conformance_heads: vec![],
        constructor_initializations: vec![],
        type_member_declarations: vec![],
        associated_static_domains: vec![],
        associated_surfaces: vec![],
        enum_requirements,
    }
}

fn enum_requirement_owner() -> EnumRequirementOwnerV1 {
    let source_module = declaration().module;
    let circle = variant("Circle");
    let square = variant("Square");
    EnumRequirementOwnerV1 {
        owner: declaration(),
        // Wire rows are selector-ordered; source_order preserves declaration order.
        requirements: vec![
            EnumRequirementRecordV1 {
                requirement: root_requirement("area()"),
                signature: callable_signature(root_requirement("area()")),
                source_order: 1,
                source: None,
            },
            EnumRequirementRecordV1 {
                requirement: root_requirement("width()"),
                signature: callable_signature(root_requirement("width()")),
                source_order: 0,
                source: Some(EnumRequirementSourceV1 {
                    module: source_module,
                    range: SpanV1 { start: 11, end: 19 },
                }),
            },
        ],
        // Status row order is semantic source order; requirement_index addresses
        // the selector-ordered requirement vector above.
        case_statuses: vec![
            CaseRequirementResultV1 {
                variant: circle.clone(),
                requirement: root_requirement("width()"),
                requirement_order: 0,
                variant_order: 0,
                status: CaseRequirementStatusV1::Satisfied {
                    implementation: implementation(circle.clone(), "width()"),
                },
            },
            CaseRequirementResultV1 {
                variant: square.clone(),
                requirement: root_requirement("width()"),
                requirement_order: 0,
                variant_order: 1,
                status: CaseRequirementStatusV1::Missing,
            },
            CaseRequirementResultV1 {
                variant: circle.clone(),
                requirement: root_requirement("area()"),
                requirement_order: 1,
                variant_order: 0,
                status: CaseRequirementStatusV1::Incompatible {
                    implementation: implementation(circle, "area()"),
                },
            },
            CaseRequirementResultV1 {
                variant: square,
                requirement: root_requirement("area()"),
                requirement_order: 1,
                variant_order: 1,
                status: CaseRequirementStatusV1::Blocked,
            },
        ],
    }
}

#[test]
fn enum_requirement_rows_round_trip_all_statuses_and_source_order() {
    let encoded = world(vec![enum_requirement_owner()]).encode().expect("valid enum requirement records encode");
    let decoded = SemanticWorldV2::decode(&encoded, &Default::default()).expect("enum requirement records decode");
    assert_eq!(decoded.enum_requirements, vec![enum_requirement_owner()]);
}

#[test]
fn semantic_world_schema_one_is_rejected() {
    let mut encoded = world(vec![enum_requirement_owner()]).encode().expect("valid semantic world encodes");
    encoded[..4].copy_from_slice(&1u32.to_le_bytes());
    assert!(SemanticWorldV2::decode(&encoded, &Default::default()).is_err());
}

#[test]
fn enum_requirement_rows_reject_invalid_ranges_owners_and_requirement_indices() {
    let mut inverted_range = world(vec![enum_requirement_owner()]);
    inverted_range.enum_requirements[0].requirements[1]
        .source
        .as_mut()
        .expect("one requirement has a source span")
        .range = SpanV1 { start: 19, end: 11 };
    assert!(inverted_range.validate(&Default::default()).is_err());

    let mut foreign_variant = world(vec![enum_requirement_owner()]);
    foreign_variant.enum_requirements[0].case_statuses[0].variant.owner.path = vec!["Other".into()].into_boxed_slice();
    assert!(foreign_variant.validate(&Default::default()).is_err());

    let mut missing_requirement = world(vec![enum_requirement_owner()]);
    missing_requirement.enum_requirements[0].case_statuses[0].requirement_order = 2;
    assert!(missing_requirement.validate(&Default::default()).is_err());

    let mut duplicate_selector = world(vec![enum_requirement_owner()]);
    duplicate_selector.enum_requirements[0].requirements[1].requirement.selector = "area()".into();
    assert!(duplicate_selector.validate(&Default::default()).is_err());
}
