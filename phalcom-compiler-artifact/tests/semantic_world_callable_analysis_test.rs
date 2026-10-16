use phalcom_compiler_artifact::module_world::SpanV1;
use phalcom_compiler_artifact::semantic::aggregates::{StructuralPrimaryContractsV1, VisibilityV1};
use phalcom_compiler_artifact::semantic::static_value::StaticValueGraphV1;
use phalcom_compiler_artifact::semantic::type_store::TypeGraphV1;
use phalcom_compiler_artifact::semantic::world::{
    CallableAnalysisRecordV1, CallableAnalysisSourceOwnerV1, CallableAnalysisStatusV1, CallableAnalysisUnsupportedFamilyCountsV1,
    CallableBindingAuthorityV1, CallableBindingConsistencyV1,
    CallableBindingRowV1, CallableCallSpecializationV1, CallableCausalInvalidityV1, CallableConformanceEvidenceNodeV1, CallableExactConformanceEvidenceV1,
    CallableExpressionRowV1, CallableFlowEdgeKindV1, CallableFlowEdgeV1, CallableFlowGraphV1, CallableFlowNodeKindV1, CallableFlowNodeV1,
    CallableFlowStateSummaryV1,
    CallableGenericCallConformanceBindingV1, CallableGenericCallConformanceSourceV1, CallableGenericConformanceProofRefV1, CallableInitializationStateV1,
    CallableTypeEnvironmentV1, ConstructorSuperInitializerV1, EvidenceOriginV1, EvidenceSetV1, EvidenceStatusV1, GenericConformanceKeyV1, SemanticWorldV3,
    ReturnContractValidationV1, TypeKnowledgeV1,
};
use phalcom_type_meta::identity::{
    StableCallableOwnerRef, StableCallableRef, StableDeclarationRef, StableDispatchSide, StableImplRef, StableModuleRef, StableProjectRef,
};

fn callable() -> StableCallableRef {
    StableCallableRef {
        owner: StableCallableOwnerRef::Declaration(StableDeclarationRef {
            module: StableModuleRef {
                project: StableProjectRef::Builtin {
                    namespace: "universe".into(),
                    version: "0.1.0".into(),
                },
                path: vec!["test".into()].into_boxed_slice(),
            },
            path: vec!["Owner".into()].into_boxed_slice(),
        }),
        side: StableDispatchSide::Instance,
        selector: "method".into(),
    }
}

fn owner_ref() -> StableDeclarationRef {
    StableDeclarationRef {
        module: StableModuleRef {
            project: StableProjectRef::Builtin {
                namespace: "universe".into(),
                version: "0.1.0".into(),
            },
            path: vec!["test".into()].into_boxed_slice(),
        },
        path: vec!["Owner".into()].into_boxed_slice(),
    }
}

fn expression_row(ordinal: u32) -> CallableExpressionRowV1 {
    CallableExpressionRowV1 {
        ordinal,
        range: SpanV1 { start: 0, end: 0 },
        knowledge: TypeKnowledgeV1::Unknown {
            reason: phalcom_compiler_artifact::semantic::aggregates::UnknownReasonV1::NoTypeEvidence,
        },
        callable: None,
        conditional_dispatch: None,
        conditional_capability: None,
        trait_dispatch: None,
        causal_invalidity: CallableCausalInvalidityV1::Clean,
        explanation: None,
        binding_read_safety: None,
        binding_read: None,
        binding_write: None,
        status: phalcom_compiler_artifact::semantic::world::CallableExpressionStatusV1::Ready,
        denotation: None,
        call_specialization: None,
    }
}

fn generic_conformance_key() -> GenericConformanceKeyV1 {
    GenericConformanceKeyV1 {
        subject: phalcom_compiler_artifact::semantic::aggregates::TypeTermV1::Canonical {
            ty: phalcom_compiler_artifact::semantic::type_store::ArtifactTypeId(0),
        },
        trait_ref: phalcom_compiler_artifact::semantic::aggregates::TraitRefV1 {
            declaration: owner_ref(),
            arguments: vec![phalcom_compiler_artifact::semantic::type_store::ArtifactTypeId(0)],
        },
    }
}

#[test]
fn callable_expression_binding_links_must_close_over_the_owner() {
    let mut valid = world(CallableAnalysisStatusV1::Complete);
    let mut row = expression_row(0);
    row.binding_read = Some(0);
    row.binding_write = Some(0);
    valid.callable_analyses[0].expressions.push(row);
    valid
        .validate(&Default::default())
        .expect("expression binding links resolve within their callable owner");

    let mut absent_binding = valid.clone();
    absent_binding.callable_analyses[0].expressions[0].binding_write = Some(1);
    assert!(absent_binding.validate(&Default::default()).is_err());

    let mut inverted_range = valid;
    inverted_range.callable_analyses[0].expressions[0].range = SpanV1 { start: 2, end: 1 };
    assert!(inverted_range.validate(&Default::default()).is_err());
}

#[test]
fn callable_conformance_references_are_kind_checked_and_graph_cycles_fail_closed() {
    let key = generic_conformance_key();
    let mut wrong_kind = world(CallableAnalysisStatusV1::Complete);
    wrong_kind.callable_analyses[0].conformance_evidence = vec![CallableConformanceEvidenceNodeV1::Assumed {
        key: key.clone(),
        owner: phalcom_compiler_artifact::semantic::type_store::ParameterOwnerV1::Callable { owner: callable() },
    }];
    let mut expression = expression_row(0);
    expression.call_specialization = Some(CallableCallSpecializationV1 {
        callable: callable(),
        bindings: vec![],
        conformances: vec![CallableGenericCallConformanceBindingV1 {
            callee_conformance: key.clone(),
            source: CallableGenericCallConformanceSourceV1::Exact { evidence: 0 },
        }],
    });
    wrong_kind.callable_analyses[0].expressions.push(expression);
    assert!(wrong_kind.validate(&Default::default()).is_err(), "Exact sources must reference Exact nodes");

    let mut cyclic = world(CallableAnalysisStatusV1::Complete);
    cyclic.callable_analyses[0].conformance_evidence = vec![CallableConformanceEvidenceNodeV1::Exact {
        evidence: CallableExactConformanceEvidenceV1 {
            source_impl: StableImplRef {
                module: owner_ref().module,
                local_id: 0,
            },
            exact_target: phalcom_compiler_artifact::semantic::type_store::ArtifactTypeId(0),
            exact_trait_ref: key.trait_ref.clone(),
            impl_environment: CallableTypeEnvironmentV1 {
                bindings: vec![],
                self_binding: None,
            },
            requirement_views: vec![],
            requirements: vec![],
            associated_types: vec![],
            trait_header_contract: vec![],
            trait_header_conformance_proofs: Some(vec![CallableGenericConformanceProofRefV1 { key, evidence: 0 }]),
        },
    }];
    assert!(cyclic.validate(&Default::default()).is_err(), "self-referential evidence is not a finite proof");
}

fn body_local_binding_row(ordinal: u32) -> CallableBindingRowV1 {
    CallableBindingRowV1 {
        ordinal,
        name: format!("binding_{ordinal}"),
        parameter: None,
        range: SpanV1 { start: 0, end: 1 },
        contract: None,
        current: TypeKnowledgeV1::Unknown {
            reason: phalcom_compiler_artifact::semantic::aggregates::UnknownReasonV1::NoTypeEvidence,
        },
        denotation: None,
        consistency: CallableBindingConsistencyV1::Unconstrained,
        causal_invalidity: CallableCausalInvalidityV1::Clean,
        mutable: false,
        initialization: CallableInitializationStateV1::DefinitelyInitialized,
        version: 0,
        explanation: None,
        authority: CallableBindingAuthorityV1::BodyLocalSource {
            site_owner: CallableAnalysisSourceOwnerV1::Module { module: owner_ref().module },
            site_local: ordinal,
            declaration_range: SpanV1 { start: 0, end: 1 },
        },
    }
}

fn world(status: CallableAnalysisStatusV1) -> SemanticWorldV3 {
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
            data_contracts: vec![],
            enum_contracts: vec![],
            variant_contracts: vec![],
        },
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
        static_definitions: vec![],
        static_constants: vec![],
        module_static_bindings: vec![],
        witness_plans: vec![],
        derived_conformance_rules: vec![],
        callable_signatures: vec![],
        callable_definitions: vec![],
        callable_analyses: vec![CallableAnalysisRecordV1 {
            callable: callable(),
            body_source_owner: None,
            body_range: SpanV1 { start: 0, end: 8 },
            status,
            exits_unreachable: true,
            flow_graph: CallableFlowGraphV1 {
                nodes: vec![],
                edges: vec![],
                entry: None,
                exits: vec![],
            },
            constructor_super_initializer: Some(ConstructorSuperInitializerV1 {
                call_range: SpanV1 { start: 7, end: 9 },
                initializer: callable(),
            }),
            expressions: vec![],
            conformance_evidence: vec![],
            unsupported_family_counts: CallableAnalysisUnsupportedFamilyCountsV1 {
                trait_dispatch_candidates: 0,
                contextual_realizations: 0,
                positional_expansions: 0,
                known_invalid_positional_expansions: 0,
                family_applications: 0,
            },
            for_iteration_lanes: vec![],
            sequence_literals: vec![],
            associated_resolutions: vec![],
            callable_reference_resolutions: vec![],
            entry_flow: CallableFlowStateSummaryV1 {
                bindings: vec![],
                fields: vec![],
                fact_count: 0,
            },
            normal_returns: vec![],
            throws: vec![],
            return_validation: ReturnContractValidationV1::NotApplicable,
            bindings: vec![CallableBindingRowV1 {
                ordinal: 0,
                name: "shared".into(),
                parameter: None,
                range: SpanV1 { start: 0, end: 1 },
                contract: None,
                current: TypeKnowledgeV1::Unknown {
                    reason: phalcom_compiler_artifact::semantic::aggregates::UnknownReasonV1::NoTypeEvidence,
                },
                denotation: None,
                consistency: CallableBindingConsistencyV1::Unconstrained,
                causal_invalidity: CallableCausalInvalidityV1::Clean,
                mutable: false,
                initialization: CallableInitializationStateV1::DefinitelyInitialized,
                version: 0,
                explanation: None,
                authority: CallableBindingAuthorityV1::ModuleValue {
                    module: StableModuleRef {
                        project: StableProjectRef::Builtin {
                            namespace: "universe".into(),
                            version: "0.1.0".into(),
                        },
                        path: vec!["test".into()].into_boxed_slice(),
                    },
                    name: "shared".into(),
                    import_binding: Some(2),
                },
            }],
        }],
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
        enum_requirements: vec![],
    }
}

#[test]
fn callable_analysis_status_is_preserved_by_semantic_world_codec() {
    for status in [
        CallableAnalysisStatusV1::Complete,
        CallableAnalysisStatusV1::Partial,
        CallableAnalysisStatusV1::Blocked,
        CallableAnalysisStatusV1::Cancelled,
        CallableAnalysisStatusV1::BudgetExceeded,
    ] {
        let encoded = world(status.clone()).encode().expect("valid semantic world encodes");
        let decoded = SemanticWorldV3::decode(&encoded, &Default::default()).expect("semantic world decodes");
        assert_eq!(decoded.callable_analyses[0].status, status);
        assert_eq!(decoded.callable_analyses[0].callable, callable());
        assert!(decoded.callable_analyses[0].exits_unreachable);
        assert!(matches!(
            decoded.callable_analyses[0].bindings[0].authority,
            CallableBindingAuthorityV1::ModuleValue { import_binding: Some(2), .. }
        ));
        assert_eq!(
            decoded.callable_analyses[0]
                .constructor_super_initializer
                .as_ref()
                .expect("constructor attachment is preserved")
                .call_range,
            SpanV1 { start: 7, end: 9 }
        );
    }
}

#[test]
fn callable_flow_graph_round_trips_owner_local_nodes_edges_and_entry_exit() {
    let mut source = world(CallableAnalysisStatusV1::Complete);
    source.callable_analyses[0].flow_graph.nodes = vec![
        CallableFlowNodeV1 {
            kind: CallableFlowNodeKindV1::Entry,
            range: SpanV1 { start: 0, end: 0 },
        },
        CallableFlowNodeV1 {
            kind: CallableFlowNodeKindV1::Exit,
            range: SpanV1 { start: 4, end: 4 },
        },
    ];
    source.callable_analyses[0].flow_graph.edges = vec![CallableFlowEdgeV1 {
        source: 0,
        target: 1,
        kind: CallableFlowEdgeKindV1::Return,
    }];
    source.callable_analyses[0].flow_graph.entry = Some(0);
    source.callable_analyses[0].flow_graph.exits = vec![1];
    let expected = source.callable_analyses[0].flow_graph.clone();

    let encoded = source.encode().expect("well-formed owner-local graph encodes");
    let decoded = SemanticWorldV3::decode(&encoded, &Default::default()).expect("owner-local graph decodes");
    assert_eq!(decoded.callable_analyses[0].flow_graph, expected);

    let mut malformed = source.clone();
    malformed.callable_analyses[0].flow_graph.edges[0].target = 2;
    let error = malformed.encode().expect_err("dangling endpoint must fail closed");
    assert!(error.to_string().contains("dangling node reference"));

    let mut malformed = source;
    malformed.callable_analyses[0].body_range = SpanV1 { start: 9, end: 8 };
    let error = malformed.encode().expect_err("inverted body range must fail closed");
    assert!(error.to_string().contains("body source range is inverted"));
}

#[test]
fn callable_analysis_owner_and_constructor_range_are_validated() {
    let mut duplicate_owner = world(CallableAnalysisStatusV1::Complete);
    duplicate_owner.callable_analyses.push(duplicate_owner.callable_analyses[0].clone());
    assert!(duplicate_owner.validate(&Default::default()).is_err());

    let mut unsorted_owners = world(CallableAnalysisStatusV1::Complete);
    let mut earlier = unsorted_owners.callable_analyses[0].clone();
    earlier.callable.selector = "aaa".into();
    unsorted_owners.callable_analyses.push(earlier);
    assert!(unsorted_owners.validate(&Default::default()).is_err());

    let mut inverted_range = world(CallableAnalysisStatusV1::Complete);
    inverted_range.callable_analyses[0]
        .constructor_super_initializer
        .as_mut()
        .expect("constructor attachment is present")
        .call_range = SpanV1 { start: 9, end: 7 };
    assert!(inverted_range.validate(&Default::default()).is_err());

    let mut invalid_conditional_side = world(CallableAnalysisStatusV1::Complete);
    let mut expression = expression_row(0);
    expression.conditional_dispatch = Some(phalcom_compiler_artifact::semantic::world::ConditionalDispatchSelectionV1 {
        impl_id: StableImplRef {
            module: owner_ref().module,
            local_id: 0,
        },
        callable: callable(),
        declaring_owner: owner_ref(),
        side: 2,
    });
    invalid_conditional_side.callable_analyses[0].expressions = vec![expression];
    let error = invalid_conditional_side
        .validate(&Default::default())
        .expect_err("an expression conditional selection must use a valid dispatch side");
    assert!(error.to_string().contains("conditional dispatch selection has an invalid or inconsistent side"));
}

#[test]
fn callable_analysis_owner_local_families_obey_individual_and_cumulative_limits() {
    let mut oversized_expressions = world(CallableAnalysisStatusV1::Complete);
    oversized_expressions.callable_analyses[0].expressions = vec![expression_row(0), expression_row(1)];
    let mut zero_expression_limit = phalcom_compiler_artifact::ValidationLimits::default();
    zero_expression_limit.max_semantic_entries_per_family = 1;
    let error = oversized_expressions
        .validate(&zero_expression_limit)
        .expect_err("dense expression rows must be bounded before ordinal use");
    assert!(error.to_string().contains("callable expression or binding rows exceed their owner-local limit"));

    let mut oversized_total = world(CallableAnalysisStatusV1::Complete);
    let analysis = &mut oversized_total.callable_analyses[0];
    analysis.flow_graph.nodes = vec![
        CallableFlowNodeV1 {
            kind: CallableFlowNodeKindV1::Entry,
            range: SpanV1 { start: 0, end: 0 },
        },
        CallableFlowNodeV1 {
            kind: CallableFlowNodeKindV1::Exit,
            range: SpanV1 { start: 8, end: 8 },
        },
    ];
    analysis.flow_graph.edges = vec![CallableFlowEdgeV1 {
        source: 0,
        target: 1,
        kind: CallableFlowEdgeKindV1::Return,
    }];
    analysis.flow_graph.entry = Some(0);
    analysis.flow_graph.exits = vec![1];

    let mut limits = phalcom_compiler_artifact::ValidationLimits::default();
    limits.max_semantic_entries_per_family = 4;
    let error = oversized_total
        .validate(&limits)
        .expect_err("nested families must share the callable owner budget");
    assert!(error.to_string().contains("callable nested row budget exceeded"));
}

#[test]
fn callable_local_ordinals_are_scoped_per_owner_and_duplicates_fail() {
    use phalcom_compiler_artifact::semantic::world::CallableCallSpecializationV1;

    let mut two_owners = world(CallableAnalysisStatusV1::Complete);
    two_owners.callable_analyses[0].expressions = vec![CallableExpressionRowV1 {
        ordinal: 0,
        range: phalcom_compiler_artifact::module_world::SpanV1 { start: 0, end: 0 },
        knowledge: phalcom_compiler_artifact::semantic::world::TypeKnowledgeV1::Unknown {
            reason: phalcom_compiler_artifact::semantic::aggregates::UnknownReasonV1::NoTypeEvidence,
        },
        callable: None,
        conditional_dispatch: None,
        causal_invalidity: phalcom_compiler_artifact::semantic::world::CallableCausalInvalidityV1::Clean,
        binding_read_safety: None,
        binding_read: None,
        binding_write: None,
        status: phalcom_compiler_artifact::semantic::world::CallableExpressionStatusV1::Ready,
        denotation: None,
        call_specialization: Some(CallableCallSpecializationV1 {
            callable: callable(),
            bindings: vec![],
            conformances: vec![],
        }),
    }];
    let mut earlier_owner = two_owners.callable_analyses[0].clone();
    earlier_owner.callable.selector = "aaa".into();
    two_owners.callable_analyses.insert(0, earlier_owner);
    two_owners
        .validate(&Default::default())
        .expect("ordinal zero belongs independently to each callable record");

    let mut duplicate_expression_ordinal = world(CallableAnalysisStatusV1::Complete);
    duplicate_expression_ordinal.callable_analyses[0].expressions = vec![
        CallableExpressionRowV1 {
            ordinal: 0,
            range: phalcom_compiler_artifact::module_world::SpanV1 { start: 0, end: 0 },
            knowledge: phalcom_compiler_artifact::semantic::world::TypeKnowledgeV1::Unknown {
                reason: phalcom_compiler_artifact::semantic::aggregates::UnknownReasonV1::NoTypeEvidence,
            },
            callable: None,
            conditional_dispatch: None,
            causal_invalidity: phalcom_compiler_artifact::semantic::world::CallableCausalInvalidityV1::Clean,
            binding_read_safety: None,
            binding_read: None,
            binding_write: None,
            status: phalcom_compiler_artifact::semantic::world::CallableExpressionStatusV1::Ready,
            denotation: None,
            call_specialization: Some(CallableCallSpecializationV1 {
                callable: callable(),
                bindings: vec![],
                conformances: vec![],
            }),
        },
        CallableExpressionRowV1 {
            ordinal: 0,
            range: phalcom_compiler_artifact::module_world::SpanV1 { start: 0, end: 0 },
            knowledge: phalcom_compiler_artifact::semantic::world::TypeKnowledgeV1::Unknown {
                reason: phalcom_compiler_artifact::semantic::aggregates::UnknownReasonV1::NoTypeEvidence,
            },
            callable: None,
            conditional_dispatch: None,
            causal_invalidity: phalcom_compiler_artifact::semantic::world::CallableCausalInvalidityV1::Clean,
            binding_read_safety: None,
            binding_read: None,
            binding_write: None,
            status: phalcom_compiler_artifact::semantic::world::CallableExpressionStatusV1::Ready,
            denotation: None,
            call_specialization: Some(CallableCallSpecializationV1 {
                callable: callable(),
                bindings: vec![],
                conformances: vec![],
            }),
        },
    ];
    assert!(
        duplicate_expression_ordinal.validate(&Default::default()).is_err(),
        "duplicate ordinals in one callable family must be rejected"
    );

    let mut wrong_family = world(CallableAnalysisStatusV1::Complete);
    wrong_family.callable_analyses[0].expressions = (0..4).map(expression_row).collect();
    wrong_family.callable_analyses[0].bindings = vec![body_local_binding_row(0), body_local_binding_row(2)];
    assert!(
        wrong_family.validate(&Default::default()).is_err(),
        "a valid expression ordinal cannot stand in for an absent binding ordinal"
    );
}

#[test]
fn explicit_dynamic_call_argument_round_trips_without_a_type_node() {
    use phalcom_compiler_artifact::semantic::type_store::{
        ArtifactKindId, ArtifactTypeParameterId, KindV1, ParameterModeV1, ParameterOwnerV1, ParameterV1, StaticArgumentV1, VarianceV1,
    };
    use phalcom_compiler_artifact::semantic::world::{CallableCallSpecializationV1, CallableStaticArgumentBindingV1};

    let mut image = world(CallableAnalysisStatusV1::Complete);
    image.type_graph.kinds = vec![KindV1::Type];
    image.type_graph.parameters = vec![ParameterV1 {
        owner: ParameterOwnerV1::Callable { owner: callable() },
        index: 0,
        name: "T".into(),
        kind: ArtifactKindId(0),
        variance: VarianceV1::Invariant,
        mode: ParameterModeV1::Single,
    }];
    image.callable_analyses[0].expressions = vec![CallableExpressionRowV1 {
        ordinal: 0,
        range: phalcom_compiler_artifact::module_world::SpanV1 { start: 0, end: 0 },
        knowledge: phalcom_compiler_artifact::semantic::world::TypeKnowledgeV1::Unknown {
            reason: phalcom_compiler_artifact::semantic::aggregates::UnknownReasonV1::NoTypeEvidence,
        },
        callable: None,
        conditional_dispatch: None,
        causal_invalidity: phalcom_compiler_artifact::semantic::world::CallableCausalInvalidityV1::Clean,
        binding_read_safety: None,
        binding_read: None,
        binding_write: None,
        status: phalcom_compiler_artifact::semantic::world::CallableExpressionStatusV1::Ready,
        denotation: None,
        call_specialization: Some(CallableCallSpecializationV1 {
            callable: callable(),
            bindings: vec![CallableStaticArgumentBindingV1 {
                parameter: ArtifactTypeParameterId(0),
                argument: StaticArgumentV1::DynamicType,
            }],
            conformances: vec![],
        }),
    }];

    let encoded = image.encode().expect("explicit Dynamic is a valid call argument");
    let decoded = SemanticWorldV3::decode(&encoded, &Default::default()).expect("call argument decodes");
    assert_eq!(decoded.type_graph.types.len(), 0, "Dynamic has no proper type node");
    assert!(matches!(
        decoded.callable_analyses[0].expressions[0]
            .call_specialization
            .as_ref()
            .expect("call specialization is nested under its expression")
            .bindings[0]
            .argument,
        StaticArgumentV1::DynamicType
    ));
}

#[test]
fn call_specialization_rejects_unowned_ordinals_duplicate_bindings_and_wrong_dynamic_kind() {
    use phalcom_compiler_artifact::semantic::type_store::{
        ArtifactKindId, ArtifactTypeParameterId, KindV1, ParameterModeV1, ParameterOwnerV1, ParameterV1, StaticArgumentV1, VarianceV1,
    };
    use phalcom_compiler_artifact::semantic::world::{CallableCallSpecializationV1, CallableStaticArgumentBindingV1};

    let mut image = world(CallableAnalysisStatusV1::Complete);
    image.type_graph.kinds = vec![KindV1::Type];
    image.type_graph.parameters = vec![ParameterV1 {
        owner: ParameterOwnerV1::Callable { owner: callable() },
        index: 0,
        name: "T".into(),
        kind: ArtifactKindId(0),
        variance: VarianceV1::Invariant,
        mode: ParameterModeV1::Single,
    }];
    image.callable_analyses[0].expressions = vec![CallableExpressionRowV1 {
        ordinal: 0,
        range: phalcom_compiler_artifact::module_world::SpanV1 { start: 0, end: 0 },
        knowledge: phalcom_compiler_artifact::semantic::world::TypeKnowledgeV1::Unknown {
            reason: phalcom_compiler_artifact::semantic::aggregates::UnknownReasonV1::NoTypeEvidence,
        },
        callable: None,
        conditional_dispatch: None,
        causal_invalidity: phalcom_compiler_artifact::semantic::world::CallableCausalInvalidityV1::Clean,
        binding_read_safety: None,
        binding_read: None,
        binding_write: None,
        status: phalcom_compiler_artifact::semantic::world::CallableExpressionStatusV1::Ready,
        denotation: None,
        call_specialization: Some(CallableCallSpecializationV1 {
            callable: callable(),
            bindings: vec![
                CallableStaticArgumentBindingV1 {
                    parameter: ArtifactTypeParameterId(0),
                    argument: StaticArgumentV1::DynamicType,
                },
                CallableStaticArgumentBindingV1 {
                    parameter: ArtifactTypeParameterId(0),
                    argument: StaticArgumentV1::DynamicType,
                },
            ],
            conformances: vec![],
        }),
    }];
    assert!(image.validate(&Default::default()).is_err(), "duplicate parameter bindings must fail");

    let mut wrong_kind = world(CallableAnalysisStatusV1::Complete);
    wrong_kind.type_graph.kinds = vec![KindV1::RecordRow];
    wrong_kind.type_graph.parameters = vec![ParameterV1 {
        owner: ParameterOwnerV1::Callable { owner: callable() },
        index: 0,
        name: "R".into(),
        kind: ArtifactKindId(0),
        variance: VarianceV1::Invariant,
        mode: ParameterModeV1::Single,
    }];
    wrong_kind.callable_analyses[0].expressions = vec![CallableExpressionRowV1 {
        ordinal: 0,
        range: phalcom_compiler_artifact::module_world::SpanV1 { start: 0, end: 0 },
        knowledge: phalcom_compiler_artifact::semantic::world::TypeKnowledgeV1::Unknown {
            reason: phalcom_compiler_artifact::semantic::aggregates::UnknownReasonV1::NoTypeEvidence,
        },
        callable: None,
        conditional_dispatch: None,
        causal_invalidity: phalcom_compiler_artifact::semantic::world::CallableCausalInvalidityV1::Clean,
        binding_read_safety: None,
        binding_read: None,
        binding_write: None,
        status: phalcom_compiler_artifact::semantic::world::CallableExpressionStatusV1::Ready,
        denotation: None,
        call_specialization: Some(CallableCallSpecializationV1 {
            callable: callable(),
            bindings: vec![CallableStaticArgumentBindingV1 {
                parameter: ArtifactTypeParameterId(0),
                argument: StaticArgumentV1::DynamicType,
            }],
            conformances: vec![],
        }),
    }];
    assert!(wrong_kind.validate(&Default::default()).is_err(), "Dynamic requires a proper-type parameter");

    let mut bad_ordinal = world(CallableAnalysisStatusV1::Complete);
    bad_ordinal.callable_analyses[0].expressions = vec![expression_row(1)];
    assert!(
        bad_ordinal.validate(&Default::default()).is_err(),
        "expression rows must form the dense owner-local ordinal range"
    );
}

#[test]
fn associated_denotation_rejects_member_from_wrong_declaration_owner() {
    use phalcom_compiler_artifact::semantic::type_store::{ArtifactKindId, KindV1, TypeEntryV1, TypeV1};
    use phalcom_compiler_artifact::semantic::world::{
        AssociatedFamilyIdentityV1, AssociatedFamilyKindV1, AssociatedFamilyRecordV1, AssociatedMemberV1, AssociatedSurfaceRecordV1,
        AssociatedValueDenotationV1, FamilyOperationShapeV1, SelectorKindV1, SemanticDenotationV1,
    };

    let lookup_owner = owner_ref();
    let foreign_owner = StableDeclarationRef {
        module: lookup_owner.module.clone(),
        path: vec!["Other".into()].into_boxed_slice(),
    };
    let foreign_variant = phalcom_type_meta::identity::StableVariantRef {
        owner: foreign_owner,
        selector: "Ready".into(),
    };
    let member = AssociatedMemberV1::Variant { variant: foreign_variant };

    let mut image = world(CallableAnalysisStatusV1::Complete);
    image.type_graph.kinds = vec![KindV1::Type];
    image.type_graph.types = vec![TypeEntryV1 {
        kind: ArtifactKindId(0),
        form: TypeV1::Nominal {
            declaration: lookup_owner.clone(),
        },
    }];
    image.associated_surfaces = vec![AssociatedSurfaceRecordV1 {
        owner: lookup_owner.clone(),
        families: vec![AssociatedFamilyRecordV1 {
            selector: phalcom_compiler_artifact::semantic::type_store::TypeMemberSelectorV1::Named { name: "State".into() },
            kind: AssociatedFamilyKindV1::Variant,
            members: vec![member.clone()],
        }],
    }];
    image.callable_analyses[0].expressions = vec![CallableExpressionRowV1 {
        ordinal: 0,
        range: phalcom_compiler_artifact::module_world::SpanV1 { start: 0, end: 0 },
        knowledge: phalcom_compiler_artifact::semantic::world::TypeKnowledgeV1::Unknown {
            reason: phalcom_compiler_artifact::semantic::aggregates::UnknownReasonV1::NoTypeEvidence,
        },
        callable: None,
        conditional_dispatch: None,
        causal_invalidity: phalcom_compiler_artifact::semantic::world::CallableCausalInvalidityV1::Clean,
        binding_read_safety: None,
        binding_read: None,
        binding_write: None,
        status: phalcom_compiler_artifact::semantic::world::CallableExpressionStatusV1::Ready,
        denotation: Some(SemanticDenotationV1::AssociatedValue {
            value: AssociatedValueDenotationV1::Family {
                owner_form: phalcom_compiler_artifact::semantic::type_store::ArtifactTypeId(0),
                lookup_owner: lookup_owner.clone(),
                family: AssociatedFamilyIdentityV1 {
                    owner: lookup_owner.clone(),
                    selector: phalcom_compiler_artifact::semantic::type_store::TypeMemberSelectorV1::Named { name: "State".into() },
                },
                members: vec![phalcom_compiler_artifact::semantic::world::CapturedAssociatedMemberV1 {
                    operation: FamilyOperationShapeV1 {
                        kind: SelectorKindV1::Getter,
                        slots: vec![],
                    },
                    member,
                    target: None,
                }],
            },
        }),
        call_specialization: None,
    }];

    let error = image
        .validate(&Default::default())
        .expect_err("a denotation cannot use a foreign member even if the surface lists that identity");
    assert!(error.to_string().contains("owner disagrees"));
}

#[test]
fn associated_denotation_rejects_member_without_primary_owner_authority() {
    use phalcom_compiler_artifact::semantic::type_store::{ArtifactKindId, KindV1, TypeEntryV1, TypeV1};
    use phalcom_compiler_artifact::semantic::world::{
        AssociatedFamilyKindV1, AssociatedFamilyRecordV1, AssociatedMemberV1, AssociatedSurfaceRecordV1, AssociatedValueDenotationV1, CallableExpressionRowV1,
        SemanticDenotationV1,
    };

    let owner = owner_ref();
    let mut image = world(CallableAnalysisStatusV1::Complete);
    image.type_graph.kinds = vec![KindV1::Type];
    image.type_graph.types = vec![TypeEntryV1 {
        kind: ArtifactKindId(0),
        form: TypeV1::Nominal { declaration: owner.clone() },
    }];
    image.associated_surfaces = vec![AssociatedSurfaceRecordV1 {
        owner: owner.clone(),
        families: vec![AssociatedFamilyRecordV1 {
            selector: phalcom_compiler_artifact::semantic::type_store::TypeMemberSelectorV1::Named { name: "State".into() },
            kind: AssociatedFamilyKindV1::Variant,
            members: vec![AssociatedMemberV1::Variant {
                variant: phalcom_type_meta::identity::StableVariantRef {
                    owner: owner.clone(),
                    selector: "Invented".into(),
                },
            }],
        }],
    }];
    image.callable_analyses[0].expressions = vec![CallableExpressionRowV1 {
        ordinal: 0,
        range: phalcom_compiler_artifact::module_world::SpanV1 { start: 0, end: 0 },
        knowledge: phalcom_compiler_artifact::semantic::world::TypeKnowledgeV1::Unknown {
            reason: phalcom_compiler_artifact::semantic::aggregates::UnknownReasonV1::NoTypeEvidence,
        },
        callable: None,
        conditional_dispatch: None,
        causal_invalidity: phalcom_compiler_artifact::semantic::world::CallableCausalInvalidityV1::Clean,
        binding_read_safety: None,
        binding_read: None,
        binding_write: None,
        status: phalcom_compiler_artifact::semantic::world::CallableExpressionStatusV1::Ready,
        denotation: Some(SemanticDenotationV1::AssociatedValue {
            value: AssociatedValueDenotationV1::Exact {
                owner_form: phalcom_compiler_artifact::semantic::type_store::ArtifactTypeId(0),
                lookup_owner: owner,
                member: AssociatedMemberV1::Variant {
                    variant: phalcom_type_meta::identity::StableVariantRef {
                        owner: owner_ref(),
                        selector: "Invented".into(),
                    },
                },
                target: None,
            },
        }),
        call_specialization: None,
    }];

    let error = image
        .validate(&Default::default())
        .expect_err("an associated surface cannot invent a primary variant declaration");
    assert!(error.to_string().contains("primary owner authority"));
}

#[test]
fn associated_family_capture_round_trips_subset_and_optional_targets() {
    use phalcom_compiler_artifact::semantic::aggregates::{
        CaseEnvironmentV1, EnumInfoV1, PayloadKindV1, VariantConstructorV1, VariantInfoV1, VariantShapeV1, VariantVisibilityV1, VisibilityV1,
    };
    use phalcom_compiler_artifact::semantic::type_store::{ArtifactKindId, ArtifactTypeId, ArtifactVariantTypeId, KindV1, TypeEntryV1, TypeV1};
    use phalcom_compiler_artifact::semantic::world::{
        AssociatedFamilyIdentityV1, AssociatedFamilyKindV1, AssociatedFamilyRecordV1, AssociatedMemberV1, AssociatedSurfaceRecordV1,
        AssociatedValueDenotationV1, CallableExpressionRowV1, CapturedAssociatedMemberV1, EnumRequirementOwnerV1, FamilyOperationShapeV1, SelectorKindV1,
        SelectorSlotV1, SemanticDenotationV1,
    };

    let owner = owner_ref();
    let variant_ready = phalcom_type_meta::identity::StableVariantRef {
        owner: owner.clone(),
        selector: "Ready".into(),
    };
    let variant_failed = phalcom_type_meta::identity::StableVariantRef {
        owner: owner.clone(),
        selector: "Failed".into(),
    };
    let variant_unselected = phalcom_type_meta::identity::StableVariantRef {
        owner: owner.clone(),
        selector: "Other".into(),
    };
    let variant_member = AssociatedMemberV1::Variant {
        variant: variant_ready.clone(),
    };
    let mut image = world(CallableAnalysisStatusV1::Complete);
    image.type_graph.kinds = vec![KindV1::Type];
    image.type_graph.types = vec![TypeEntryV1 {
        kind: ArtifactKindId(0),
        form: TypeV1::Nominal { declaration: owner.clone() },
    }];
    image.structural_contracts.enum_contracts.push(EnumInfoV1 {
        owner: owner.clone(),
        root_form: ArtifactTypeId(0),
        generic_signature: None,
        static_data: None,
        default_result_type: ArtifactTypeId(0),
        variants: vec![variant_failed.clone(), variant_ready.clone(), variant_unselected.clone()],
    });
    image.enum_requirements.push(EnumRequirementOwnerV1 {
        owner: owner.clone(),
        requirements: vec![],
        case_statuses: vec![],
    });
    let variant_contract = |variant, shape, constructor| {
        let payload_kind = if shape == VariantShapeV1::Singleton {
            PayloadKindV1::Singleton
        } else {
            PayloadKindV1::ExplicitEmpty
        };
        VariantInfoV1 {
            variant,
            variant_type: ArtifactVariantTypeId(0),
            shape,
            payload_kind,
            fields: vec![],
            result_type: ArtifactTypeId(0),
            exact_case_type: ArtifactTypeId(0),
            case_environment: CaseEnvironmentV1 {
                bindings: vec![],
                equalities: vec![],
            },
            constructor,
            visibility: VariantVisibilityV1 {
                name: VisibilityV1::Public,
                construct: VisibilityV1::Public,
                payload: VisibilityV1::Public,
            },
        }
    };
    image.structural_contracts.variant_contracts = vec![
        variant_contract(variant_failed.clone(), VariantShapeV1::Singleton, None),
        variant_contract(
            variant_ready.clone(),
            VariantShapeV1::Constructor,
            Some(VariantConstructorV1 {
                generic_signature: None,
                parameters: vec![],
                result_type: ArtifactTypeId(0),
                exact_case_type: ArtifactTypeId(0),
            }),
        ),
        variant_contract(variant_unselected.clone(), VariantShapeV1::Singleton, None),
    ];
    image.type_graph.variants = vec![variant_failed.clone(), variant_ready.clone(), variant_unselected.clone()];
    image.associated_surfaces = vec![AssociatedSurfaceRecordV1 {
        owner: owner.clone(),
        families: vec![AssociatedFamilyRecordV1 {
            selector: phalcom_compiler_artifact::semantic::type_store::TypeMemberSelectorV1::Named { name: "State".into() },
            kind: AssociatedFamilyKindV1::Variant,
            members: vec![
                AssociatedMemberV1::Variant {
                    variant: variant_failed.clone(),
                },
                variant_member.clone(),
                AssociatedMemberV1::Variant { variant: variant_unselected },
            ],
        }],
    }];
    let source_denotation = SemanticDenotationV1::AssociatedValue {
        value: AssociatedValueDenotationV1::Family {
            owner_form: ArtifactTypeId(0),
            lookup_owner: owner.clone(),
            family: AssociatedFamilyIdentityV1 {
                owner,
                selector: phalcom_compiler_artifact::semantic::type_store::TypeMemberSelectorV1::Named { name: "State".into() },
            },
            // Preserve both absent and present targets and the checker-captured
            // order; the declaration contains an additional unselected case.
            members: vec![
                CapturedAssociatedMemberV1 {
                    operation: FamilyOperationShapeV1 {
                        kind: SelectorKindV1::Getter,
                        slots: vec![],
                    },
                    member: AssociatedMemberV1::Variant { variant: variant_failed },
                    target: None,
                },
                CapturedAssociatedMemberV1 {
                    operation: FamilyOperationShapeV1 {
                        kind: SelectorKindV1::Method,
                        slots: vec![SelectorSlotV1::Positional],
                    },
                    member: variant_member,
                    target: Some(phalcom_compiler_artifact::semantic::world::InvocationTargetV1::VariantConstructor { variant: variant_ready }),
                },
            ],
        },
    };
    image.callable_analyses[0].expressions = vec![CallableExpressionRowV1 {
        ordinal: 0,
        range: phalcom_compiler_artifact::module_world::SpanV1 { start: 0, end: 0 },
        knowledge: phalcom_compiler_artifact::semantic::world::TypeKnowledgeV1::Unknown {
            reason: phalcom_compiler_artifact::semantic::aggregates::UnknownReasonV1::NoTypeEvidence,
        },
        callable: None,
        conditional_dispatch: None,
        causal_invalidity: phalcom_compiler_artifact::semantic::world::CallableCausalInvalidityV1::Clean,
        binding_read_safety: None,
        binding_read: None,
        binding_write: None,
        status: phalcom_compiler_artifact::semantic::world::CallableExpressionStatusV1::Ready,
        denotation: Some(source_denotation.clone()),
        call_specialization: None,
    }];

    let encoded = image.encode().expect("valid checker-captured subset should encode");
    let decoded = SemanticWorldV3::decode(&encoded, &Default::default()).expect("captured subset should validate and decode");
    assert_eq!(decoded.callable_analyses[0].expressions[0].denotation.as_ref(), Some(&source_denotation));
}

#[test]
fn behavioral_family_capture_requires_conditional_member_authority() {
    use phalcom_compiler_artifact::semantic::aggregates::TraitRefV1;
    use phalcom_compiler_artifact::semantic::type_store::{ArtifactKindId, ArtifactTypeId, KindV1, TypeEntryV1, TypeMemberSelectorV1, TypeV1};
    use phalcom_compiler_artifact::semantic::world::{
        BehavioralFamilyDenotationV1, BehavioralFamilySpecV1, CallableExpressionRowV1, CapturedBehavioralMemberV1, ConditionalDispatchSelectionV1,
        ConformanceHeadRecordV1, ConformanceTargetV1, DeclarationSurfaceRecordV1, ExactSelectorV1, FamilyOperationShapeV1, SelectorKindV1, SelectorSlotV1,
        SemanticDenotationV1,
    };

    let owner = owner_ref();
    let callable = callable();
    let module = owner.module.clone();
    let impl_id = phalcom_type_meta::identity::StableImplRef {
        module: module.clone(),
        local_id: 7,
    };
    let operation = FamilyOperationShapeV1 {
        kind: SelectorKindV1::Method,
        slots: vec![SelectorSlotV1::Positional],
    };
    let target = phalcom_compiler_artifact::semantic::world::InvocationTargetV1::Behavioral { callable: callable.clone() };
    let selection = ConditionalDispatchSelectionV1 {
        impl_id: impl_id.clone(),
        callable: callable.clone(),
        declaring_owner: owner.clone(),
        side: 0,
    };
    let mut image = world(CallableAnalysisStatusV1::Complete);
    image.type_graph.kinds = vec![KindV1::Type];
    image.type_graph.types = vec![TypeEntryV1 {
        kind: ArtifactKindId(0),
        form: TypeV1::Nominal { declaration: owner.clone() },
    }];
    image.dispatch_surfaces = vec![DeclarationSurfaceRecordV1 {
        declaration: owner.clone(),
        instance_callables: vec![phalcom_compiler_artifact::semantic::world::DispatchCallableFactV1 {
            callable: callable.clone(),
            knowledge: TypeKnowledgeV1::Known {
                ty: ArtifactTypeId(0),
                status: EvidenceStatusV1::Established,
                origin: EvidenceOriginV1::CallableSignature,
                provenance: EvidenceSetV1 {
                    ranges: vec![],
                    descriptions: vec![],
                },
            },
            visibility: VisibilityV1::Public,
        }],
        class_callables: vec![],
        fields: vec![
            phalcom_compiler_artifact::semantic::world::FieldSignatureV1 {
                owner: owner.clone(),
                name: "field".into(),
                declared_type: Some(phalcom_compiler_artifact::semantic::aggregates::DeclaredTypeFactV1 {
                    state: phalcom_compiler_artifact::semantic::aggregates::DeclaredTypeStateV1::Unknown {
                        reason: phalcom_compiler_artifact::semantic::aggregates::UnknownReasonV1::NoTypeEvidence,
                    },
                    basis: phalcom_compiler_artifact::semantic::aggregates::DeclaredTypeBasisV1::Unspecified,
                }),
                knowledge: TypeKnowledgeV1::Known {
                    ty: ArtifactTypeId(0),
                    status: EvidenceStatusV1::Established,
                    origin: EvidenceOriginV1::DeclarationSemantics,
                    provenance: EvidenceSetV1 {
                        ranges: vec![],
                        descriptions: vec![],
                    },
                },
                mutable: Some(true),
                side: 0,
                visibility: VisibilityV1::Protected,
                source: Some(SpanV1 { start: 8, end: 13 }),
            },
            phalcom_compiler_artifact::semantic::world::FieldSignatureV1 {
                owner: owner.clone(),
                name: "native".into(),
                declared_type: None,
                knowledge: TypeKnowledgeV1::Unknown {
                    reason: phalcom_compiler_artifact::semantic::aggregates::UnknownReasonV1::NoTypeEvidence,
                },
                mutable: None,
                side: 0,
                visibility: VisibilityV1::Public,
                source: None,
            },
        ],
    }];
    image.callable_signatures = vec![phalcom_compiler_artifact::semantic::world::CallableSignatureRecordV1 {
        callable: callable.clone(),
        owner: owner.clone(),
        side: 0,
        selector: callable.selector.clone(),
        generics: None,
        parameters: vec![],
        declared_return: phalcom_compiler_artifact::semantic::aggregates::DeclaredTypeFactV1 {
            state: phalcom_compiler_artifact::semantic::aggregates::DeclaredTypeStateV1::Unknown {
                reason: phalcom_compiler_artifact::semantic::aggregates::UnknownReasonV1::NoTypeEvidence,
            },
            basis: phalcom_compiler_artifact::semantic::aggregates::DeclaredTypeBasisV1::Unspecified,
        },
        return_validation: phalcom_compiler_artifact::semantic::world::ReturnContractValidationV1::NotApplicable,
        inferred_return: None,
        source: None,
        implementation: phalcom_compiler_artifact::semantic::world::ImplementationKindV1::Phalcom,
        native_id: None,
        effects: phalcom_compiler_artifact::semantic::world::EffectSpecV1::Unknown,
        raises: phalcom_compiler_artifact::semantic::world::RaisesSpecV1::Unknown,
        flow: phalcom_compiler_artifact::semantic::world::ReturnFlowV1::Value,
        lifecycle: phalcom_compiler_artifact::semantic::world::NativeLifecycleSpecV1 {
            since: None,
            deprecated_since: None,
            replacement: None,
        },
    }];
    image.conformance_heads = vec![ConformanceHeadRecordV1 {
        impl_id: impl_id.clone(),
        source_module: module.clone(),
        target: ConformanceTargetV1::Declaration { declaration: owner.clone() },
        target_type: ArtifactTypeId(0),
        trait_ref: TraitRefV1 {
            declaration: owner.clone(),
            arguments: vec![],
        },
        generic_signature: None,
        authorized: true,
        eligible: true,
    }];
    image.conditional_inherent_sets = vec![phalcom_compiler_artifact::semantic::world::ConditionalInherentSetV1 {
        target: phalcom_compiler_artifact::semantic::world::InherentImplTargetV1::Declaration { declaration: owner.clone() },
        domains: vec![phalcom_compiler_artifact::semantic::world::InherentImplDomainV1 {
            impl_id: impl_id.clone(),
            target: phalcom_compiler_artifact::semantic::world::InherentImplTargetV1::Declaration { declaration: owner.clone() },
            head_type: ArtifactTypeId(0),
            generic_signature: None,
            constraints: vec![],
        }],
        members: vec![phalcom_compiler_artifact::semantic::world::ConditionalInherentMemberV1 {
            impl_id: impl_id.clone(),
            member_order: 0,
            is_conditionally_applicable: true,
            callable: callable.clone(),
            signature: image.callable_signatures[0].clone(),
            visibility: VisibilityV1::Public,
            source_member_index: 0,
            is_requirement: false,
        }],
    }];
    let denotation = SemanticDenotationV1::BehavioralFamily {
        family: BehavioralFamilyDenotationV1 {
            receiver_type: ArtifactTypeId(0),
            receiver_application: None,
            spec: BehavioralFamilySpecV1::Exact {
                selector: ExactSelectorV1 {
                    base: TypeMemberSelectorV1::Named { name: "run".into() },
                    kind: SelectorKindV1::Method,
                    slots: vec![SelectorSlotV1::Positional],
                },
            },
            members: vec![
                CapturedBehavioralMemberV1 {
                    operation: operation.clone(),
                    target: target.clone(),
                    conditional: None,
                },
                CapturedBehavioralMemberV1 {
                    operation,
                    target,
                    conditional: Some(selection),
                },
            ],
        },
    };
    image.callable_analyses[0].expressions = vec![CallableExpressionRowV1 {
        ordinal: 0,
        range: phalcom_compiler_artifact::module_world::SpanV1 { start: 0, end: 0 },
        knowledge: phalcom_compiler_artifact::semantic::world::TypeKnowledgeV1::Unknown {
            reason: phalcom_compiler_artifact::semantic::aggregates::UnknownReasonV1::NoTypeEvidence,
        },
        callable: None,
        conditional_dispatch: None,
        causal_invalidity: phalcom_compiler_artifact::semantic::world::CallableCausalInvalidityV1::Clean,
        binding_read_safety: None,
        binding_read: None,
        binding_write: None,
        status: phalcom_compiler_artifact::semantic::world::CallableExpressionStatusV1::Ready,
        denotation: Some(denotation.clone()),
        call_specialization: None,
    }];

    image
        .validate(&Default::default())
        .expect("serialized conditional member should authorize the captured selection");
    image.encode().expect("valid captured behavioral targets should encode");

    let mut missing_member_authority = image;
    missing_member_authority.conditional_inherent_sets.clear();
    assert!(missing_member_authority.validate(&Default::default()).is_err());
}
