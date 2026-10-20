use phalcom_compiler_artifact::limits::ValidationLimits;
use phalcom_core::modules::canonical_artifact::{build_universe_telosc, load_universe_telosc, load_universe_telosc_from_bytes, write_universe_telosc};
use phalcom_core::modules::canonical_universe::canonical_universe_program;
use phalcom_core::modules::compile::{EntrySelection, ProgramCompileError, ProgramCompiler};
use phalcom_core::modules::parity::{assert_compile_error_parity, assert_compiled_program_parity};
use phalcom_semantic::canonical_image::canonical_universe_formal_image;
use phalcom_semantic::diagnostic::DiagnosticCode;
use tempfile::tempdir;

#[test]
fn build_and_load_universe_telosc_compiles_source_free_inline_user_entry() {
    let semantic_image = phalcom_semantic::canonical_image::canonical_universe_formal_image().expect("canonical semantic image");
    let semantic_snapshot = semantic_image.snapshot();
    for (callable, analysis) in semantic_snapshot.callable_analyses.iter().filter(|(callable, analysis)| {
        callable.module().to_string() == "universe:numbers"
            && analysis.body_source_owner == Some(phalcom_semantic::identity::SourceOwner::Module(callable.module().clone()))
    }) {
        let imported_i = analysis
            .bindings
            .values()
            .find(|binding| binding.name == "i")
            .expect("numbers i import binding");
        assert!(
            matches!(
                analysis.binding_origins.get(&imported_i.binding),
                Some(phalcom_semantic::checker::analysis::CallableBindingOrigin::ModuleValue {
                    symbol,
                    import: Some(_),
                }) if symbol.name.as_ref() == "i"
            ),
            "top-level imported i must retain its canonical module storage identity: {callable:?}, binding={imported_i:?}, origin={:?}",
            analysis.binding_origins.get(&imported_i.binding),
        );
        let imported_source = semantic_snapshot
            .source_index
            .module(callable.module())
            .expect("numbers source shard")
            .structure
            .bindings
            .values()
            .find(|binding| binding.name.as_ref() == "i" && binding.kind == phalcom_semantic::SourceBindingKind::Import)
            .expect("local import declaration site");
        assert_eq!(
            imported_source.declaration_site.owner,
            phalcom_semantic::identity::SourceOwner::Module(callable.module().clone())
        );
    }
    let internal_incidents = semantic_snapshot
        .callable_analyses
        .iter()
        .filter(|(_, analysis)| !analysis.internal_incidents.is_empty())
        .map(|(callable, analysis)| (callable.clone(), analysis.internal_incidents.clone(), analysis.status.clone()))
        .collect::<Vec<_>>();
    assert!(internal_incidents.is_empty(), "canonical callable internal incidents: {internal_incidents:#?}");
    let callable_authority_counts = [
        (
            "expressions",
            semantic_snapshot.callable_analyses.iter().map(|(_, analysis)| analysis.expressions.len()).sum(),
        ),
        (
            "expression.call",
            semantic_snapshot
                .callable_analyses
                .iter()
                .flat_map(|(_, analysis)| analysis.expressions.values())
                .filter(|expression| expression.call.is_some())
                .count(),
        ),
        (
            "expression.conditional_capability",
            semantic_snapshot
                .callable_analyses
                .iter()
                .flat_map(|(_, analysis)| analysis.expressions.values())
                .filter(|expression| expression.conditional_capability.is_some())
                .count(),
        ),
        (
            "expression.trait_dispatch",
            semantic_snapshot
                .callable_analyses
                .iter()
                .flat_map(|(_, analysis)| analysis.expressions.values())
                .filter(|expression| expression.trait_dispatch.is_some())
                .count(),
        ),
        (
            "expression.trait_dispatch_candidates",
            semantic_snapshot
                .callable_analyses
                .iter()
                .flat_map(|(_, analysis)| analysis.expressions.values())
                .filter(|expression| expression.trait_dispatch_candidates.is_some())
                .count(),
        ),
        (
            "expression.contextual_realization",
            semantic_snapshot
                .callable_analyses
                .iter()
                .flat_map(|(_, analysis)| analysis.expressions.values())
                .filter(|expression| expression.contextual_realization.is_some())
                .count(),
        ),
        (
            "expression.explanation",
            semantic_snapshot
                .callable_analyses
                .iter()
                .flat_map(|(_, analysis)| analysis.expressions.values())
                .filter(|expression| expression.explanation.is_some())
                .count(),
        ),
        (
            "for_iteration_lanes",
            semantic_snapshot
                .callable_analyses
                .iter()
                .map(|(_, analysis)| analysis.for_iteration_lanes.len())
                .sum(),
        ),
        (
            "positional_expansions",
            semantic_snapshot
                .callable_analyses
                .iter()
                .map(|(_, analysis)| analysis.positional_expansions.len())
                .sum(),
        ),
        (
            "known_invalid_positional_expansions",
            semantic_snapshot
                .callable_analyses
                .iter()
                .map(|(_, analysis)| analysis.known_invalid_positional_expansions.len())
                .sum(),
        ),
        (
            "sequence_literals",
            semantic_snapshot
                .callable_analyses
                .iter()
                .map(|(_, analysis)| analysis.sequence_literals.len())
                .sum(),
        ),
        (
            "associated_resolutions",
            semantic_snapshot
                .callable_analyses
                .iter()
                .map(|(_, analysis)| analysis.associated_resolutions.len())
                .sum(),
        ),
        (
            "callable_reference_resolutions",
            semantic_snapshot
                .callable_analyses
                .iter()
                .map(|(_, analysis)| analysis.callable_reference_resolutions.len())
                .sum(),
        ),
        (
            "family_applications",
            semantic_snapshot
                .callable_analyses
                .iter()
                .map(|(_, analysis)| analysis.family_applications.len())
                .sum(),
        ),
        (
            "match_resolutions",
            semantic_snapshot
                .callable_analyses
                .iter()
                .map(|(_, analysis)| analysis.match_resolutions.len())
                .sum(),
        ),
        (
            "pattern_site_resolutions",
            semantic_snapshot
                .callable_analyses
                .iter()
                .map(|(_, analysis)| analysis.pattern_site_resolutions.len())
                .sum(),
        ),
        (
            "flow_graph.edge.predicate",
            semantic_snapshot
                .callable_analyses
                .iter()
                .flat_map(|(_, analysis)| analysis.flow_graph.edge_entries())
                .filter(|(_, edge)| edge.predicate.is_some())
                .count(),
        ),
        (
            "return_explanation",
            semantic_snapshot
                .callable_analyses
                .iter()
                .filter(|(_, analysis)| analysis.return_explanation.is_some())
                .count(),
        ),
        (
            "EvidenceRef::CallResolution",
            semantic_snapshot
                .callable_analyses
                .iter()
                .flat_map(|(_, analysis)| analysis.explanations.iter())
                .flat_map(|(_, node)| node.evidence.iter())
                .filter(|evidence| matches!(evidence, phalcom_semantic::explain::node::EvidenceRef::CallResolution(_)))
                .count(),
        ),
        (
            "entry_flow_facts",
            semantic_snapshot
                .callable_analyses
                .iter()
                .map(|(_, analysis)| analysis.entry_flow.fact_count)
                .sum(),
        ),
        (
            "normal_returns",
            semantic_snapshot
                .callable_analyses
                .iter()
                .map(|(_, analysis)| analysis.exits.normal_returns.len())
                .sum(),
        ),
        (
            "throws",
            semantic_snapshot
                .callable_analyses
                .iter()
                .map(|(_, analysis)| analysis.exits.throws.len())
                .sum(),
        ),
        (
            "diagnostics",
            semantic_snapshot.callable_analyses.iter().map(|(_, analysis)| analysis.diagnostics.len()).sum(),
        ),
        (
            "explanations",
            semantic_snapshot
                .callable_analyses
                .iter()
                .map(|(_, analysis)| analysis.explanations.len())
                .sum(),
        ),
        (
            "dependencies",
            semantic_snapshot
                .callable_analyses
                .iter()
                .map(|(_, analysis)| analysis.dependencies.len())
                .sum(),
        ),
        (
            "semantic_dependencies",
            semantic_snapshot
                .callable_analyses
                .iter()
                .map(|(_, analysis)| analysis.semantic_dependencies.len())
                .sum(),
        ),
    ];
    for family in [
        "expressions",
        "match_resolutions",
        "pattern_site_resolutions",
        "explanations",
        "semantic_dependencies",
    ] {
        assert!(
            callable_authority_counts.iter().any(|(name, count)| *name == family && *count > 0),
            "Universe must exercise {family}"
        );
    }
    for scaffold in ["expression.call", "flow_graph.edge.predicate", "EvidenceRef::CallResolution"] {
        let count = callable_authority_counts
            .iter()
            .find(|(name, _)| *name == scaffold)
            .expect("unused scaffold has an explicit census entry")
            .1;
        assert_eq!(count, 0, "canonical Universe must not publish unowned {scaffold} handles");
    }
    let mut associated_kinds = std::collections::BTreeMap::<&'static str, usize>::new();
    let mut callable_reference_kinds = std::collections::BTreeMap::<&'static str, usize>::new();
    let mut trait_dispatch_kinds = std::collections::BTreeMap::<&'static str, usize>::new();
    let mut explanation_steps = std::collections::BTreeMap::<&'static str, usize>::new();
    let mut explanation_rules = std::collections::BTreeMap::<&'static str, usize>::new();
    let mut explanation_evidence = std::collections::BTreeMap::<&'static str, usize>::new();
    let mut pattern_kinds = std::collections::BTreeMap::<&'static str, usize>::new();
    let mut pattern_space_kinds = std::collections::BTreeMap::<&'static str, usize>::new();
    let mut exhaustiveness_kinds = std::collections::BTreeMap::<&'static str, usize>::new();
    let mut branch_proof_counts = [0usize; 4];
    fn count_pattern<R, S, K>(
        pattern: &phalcom_semantic::match_semantics::PatternResolution<R, S, K>,
        counts: &mut std::collections::BTreeMap<&'static str, usize>,
    ) {
        use phalcom_semantic::match_semantics::PatternResolution;
        let kind = match pattern {
            PatternResolution::Wildcard => "wildcard",
            PatternResolution::Char(_) => "char",
            PatternResolution::Binding { .. } => "binding",
            PatternResolution::Variant(variant) => {
                for candidate in variant.candidates.iter() {
                    for field in candidate.fields.iter() {
                        count_pattern(&field.child, counts);
                    }
                }
                "variant"
            }
            PatternResolution::Or(pattern) => {
                for alternative in pattern.alternatives.iter() {
                    count_pattern(alternative, counts);
                }
                "or"
            }
            PatternResolution::Tuple(items) => {
                for item in items.iter() {
                    count_pattern(item, counts);
                }
                "tuple"
            }
            PatternResolution::List(list) => {
                for item in list.prefix.iter() {
                    count_pattern(item, counts);
                }
                if let Some(rest) = &list.rest {
                    count_pattern(rest, counts);
                }
                "list"
            }
            PatternResolution::Record(fields) => {
                for field in fields.iter() {
                    count_pattern(&field.child, counts);
                }
                "record"
            }
            PatternResolution::Map(entries) => {
                for entry in entries.iter() {
                    count_pattern(&entry.child, counts);
                }
                "map"
            }
        };
        *counts.entry(kind).or_default() += 1;
    }
    fn count_space(space: &phalcom_semantic::match_semantics::PatternSpaceSummary, counts: &mut std::collections::BTreeMap<&'static str, usize>) {
        use phalcom_semantic::match_semantics::PatternSpaceSummary;
        let kind = match space {
            PatternSpaceSummary::Empty => "empty",
            PatternSpaceSummary::Char(_) => "char",
            PatternSpaceSummary::Blocked(_) => "blocked",
            PatternSpaceSummary::Opaque(_) => "opaque",
            PatternSpaceSummary::Union(items) => {
                for item in items.iter() {
                    count_space(item, counts);
                }
                "union"
            }
            PatternSpaceSummary::Variant { fields, .. } => {
                for field in fields.iter() {
                    count_space(field, counts);
                }
                "variant"
            }
            PatternSpaceSummary::Tuple(items) => {
                for item in items.iter() {
                    count_space(item, counts);
                }
                "tuple"
            }
            PatternSpaceSummary::List => "list",
        };
        *counts.entry(kind).or_default() += 1;
    }
    for (_, analysis) in semantic_snapshot.callable_analyses.iter() {
        analysis
            .local_proof_graph
            .validate()
            .expect("canonical callable has a closed acyclic proof graph");
        for resolution in analysis.associated_resolutions.values() {
            let kind = match &resolution.kind {
                phalcom_semantic::checker::associated::AssociatedResolutionKind::ExactValue { .. } => "exact_value",
                phalcom_semantic::checker::associated::AssociatedResolutionKind::DataExpansionField { .. } => "data_expansion_field",
                phalcom_semantic::checker::associated::AssociatedResolutionKind::ExactCallable { .. } => "exact_callable",
                phalcom_semantic::checker::associated::AssociatedResolutionKind::Family { .. } => "family",
                phalcom_semantic::checker::associated::AssociatedResolutionKind::StaticInvoke { .. } => "static_invoke",
                phalcom_semantic::checker::associated::AssociatedResolutionKind::StaticDomain { .. } => "static_domain",
                phalcom_semantic::checker::associated::AssociatedResolutionKind::StaticTerm { .. } => "static_term",
                phalcom_semantic::checker::associated::AssociatedResolutionKind::DynamicInvoke { .. } => "dynamic_invoke",
            };
            *associated_kinds.entry(kind).or_default() += 1;
        }
        for resolution in analysis.callable_reference_resolutions.values() {
            let kind = match &resolution.kind {
                phalcom_semantic::checker::associated::CallableReferenceResolutionKind::BoundFamily { .. } => "bound_family",
                phalcom_semantic::checker::associated::CallableReferenceResolutionKind::Associated(_) => "associated",
            };
            *callable_reference_kinds.entry(kind).or_default() += 1;
        }
        for expression in analysis.expressions.values() {
            if let Some(site) = &expression.trait_dispatch {
                let kind = match site {
                    phalcom_semantic::trait_dispatch::TraitDispatchSite::AbstractRequirement { .. } => "abstract",
                    phalcom_semantic::trait_dispatch::TraitDispatchSite::Evidenced(_) => "evidenced",
                    phalcom_semantic::trait_dispatch::TraitDispatchSite::Assumed(_) => "assumed",
                };
                *trait_dispatch_kinds.entry(kind).or_default() += 1;
            }
        }
        for (_, node) in analysis.explanations.iter() {
            let step_kind = match &node.step {
                phalcom_semantic::explain::node::ExplanationStep::Literal { .. } => "Literal",
                phalcom_semantic::explain::node::ExplanationStep::ExpressionResult { .. } => "ExpressionResult",
                phalcom_semantic::explain::node::ExplanationStep::BindingRead { .. } => "BindingRead",
                phalcom_semantic::explain::node::ExplanationStep::Declared { .. } => "Declared",
                phalcom_semantic::explain::node::ExplanationStep::BindingContract { .. } => "BindingContract",
                phalcom_semantic::explain::node::ExplanationStep::TypeRequirement { .. } => "TypeRequirement",
                phalcom_semantic::explain::node::ExplanationStep::TypeRelation { .. } => "TypeRelation",
                phalcom_semantic::explain::node::ExplanationStep::CallableSelection { .. } => "CallableSelection",
                phalcom_semantic::explain::node::ExplanationStep::UnionArm { .. } => "UnionArm",
                phalcom_semantic::explain::node::ExplanationStep::CallableKind { .. } => "CallableKind",
                phalcom_semantic::explain::node::ExplanationStep::CallableReturn { .. } => "CallableReturn",
                phalcom_semantic::explain::node::ExplanationStep::SelfTypeSpecialization { .. } => "SelfTypeSpecialization",
                phalcom_semantic::explain::node::ExplanationStep::ArgumentCheck { .. } => "ArgumentCheck",
                phalcom_semantic::explain::node::ExplanationStep::CallShape { .. } => "CallShape",
                phalcom_semantic::explain::node::ExplanationStep::MethodCall { .. } => "MethodCall",
                phalcom_semantic::explain::node::ExplanationStep::UnresolvedCall { .. } => "UnresolvedCall",
                phalcom_semantic::explain::node::ExplanationStep::GenericConstraint { .. } => "GenericConstraint",
                phalcom_semantic::explain::node::ExplanationStep::GenericSolution { .. } => "GenericSolution",
                phalcom_semantic::explain::node::ExplanationStep::GenericConflict { .. } => "GenericConflict",
                phalcom_semantic::explain::node::ExplanationStep::CollectionSynthesis { .. } => "CollectionSynthesis",
                phalcom_semantic::explain::node::ExplanationStep::ProductComponent { .. } => "ProductComponent",
                phalcom_semantic::explain::node::ExplanationStep::FlowRefinement { .. } => "FlowRefinement",
                phalcom_semantic::explain::node::ExplanationStep::BranchJoin { .. } => "BranchJoin",
                phalcom_semantic::explain::node::ExplanationStep::LoopJoin { .. } => "LoopJoin",
                phalcom_semantic::explain::node::ExplanationStep::IterationElement { .. } => "IterationElement",
                phalcom_semantic::explain::node::ExplanationStep::ReturnCheck { .. } => "ReturnCheck",
                phalcom_semantic::explain::node::ExplanationStep::CallableReturnSummary { .. } => "CallableReturnSummary",
                phalcom_semantic::explain::node::ExplanationStep::UnknownBoundary { .. } => "UnknownBoundary",
                phalcom_semantic::explain::node::ExplanationStep::DynamicBoundary { .. } => "DynamicBoundary",
                phalcom_semantic::explain::node::ExplanationStep::InternalBlocked { .. } => "InternalBlocked",
                phalcom_semantic::explain::node::ExplanationStep::Subtyping { .. } => "Subtyping",
            };
            *explanation_steps.entry(step_kind).or_default() += 1;
            *explanation_rules.entry(node.rule.as_str()).or_default() += 1;
            for evidence in &node.evidence {
                let kind = match evidence {
                    phalcom_semantic::explain::node::EvidenceRef::SourceSpan(_) => "source_span",
                    phalcom_semantic::explain::node::EvidenceRef::TypeId(_) => "type",
                    phalcom_semantic::explain::node::EvidenceRef::CallResolution(_) => "call_resolution",
                    phalcom_semantic::explain::node::EvidenceRef::BindingVersion { .. } => "binding_version",
                    phalcom_semantic::explain::node::EvidenceRef::Suppressed { .. } => "suppressed",
                };
                *explanation_evidence.entry(kind).or_default() += 1;
            }
        }
        for resolution in analysis.match_resolutions.values() {
            count_space(&resolution.initial_space, &mut pattern_space_kinds);
            for arm in resolution.arms.iter() {
                count_pattern(&arm.pattern, &mut pattern_kinds);
                count_space(&arm.reachable_space, &mut pattern_space_kinds);
                count_space(&arm.residual_after, &mut pattern_space_kinds);
                branch_proof_counts[0] += arm.proof.bindings.len();
                branch_proof_counts[1] += arm.proof.equalities.len();
                branch_proof_counts[2] += arm.proof.local_bindings.len();
                branch_proof_counts[3] += arm.proof.local_equalities.len();
            }
            let kind = match &resolution.exhaustiveness {
                phalcom_semantic::match_semantics::ExhaustivenessResult::Proven => "proven",
                phalcom_semantic::match_semantics::ExhaustivenessResult::Missing(_) => "missing",
                phalcom_semantic::match_semantics::ExhaustivenessResult::Blocked(_) => "blocked",
            };
            *exhaustiveness_kinds.entry(kind).or_default() += 1;
        }
        for resolution in analysis.pattern_site_resolutions.values() {
            count_pattern(&resolution.pattern, &mut pattern_kinds);
        }
    }
    eprintln!("canonical callable authority counts: {callable_authority_counts:#?}");
    eprintln!("canonical explanation rules: {explanation_rules:#?}");
    eprintln!("canonical explanation steps: {explanation_steps:#?}");
    eprintln!("canonical explanation evidence: {explanation_evidence:#?}");
    assert!(associated_kinds.get("static_invoke").copied().unwrap_or(0) > 0);
    assert!(callable_reference_kinds.get("bound_family").copied().unwrap_or(0) > 0);
    assert!(trait_dispatch_kinds.get("evidenced").copied().unwrap_or(0) > 0);
    assert!(explanation_rules.get("generic_solution").copied().unwrap_or(0) > 0);
    assert!(explanation_evidence.get("type").copied().unwrap_or(0) > 0);
    assert!(pattern_kinds.get("variant").copied().unwrap_or(0) > 0);
    assert!(pattern_space_kinds.get("blocked").copied().unwrap_or(0) > 0);
    assert_eq!(
        exhaustiveness_kinds.get("proven").copied().unwrap_or(0),
        callable_authority_counts.iter().find(|(name, _)| *name == "match_resolutions").unwrap().1
    );
    assert!(
        branch_proof_counts[2] > 0 && branch_proof_counts[3] > 0,
        "Universe must retain local branch bindings and equalities"
    );
    let mut missing_surface_signatures = Vec::new();
    let mut missing_signature_modules = std::collections::BTreeSet::new();
    for (owner, surface) in semantic_snapshot.surfaces.iter() {
        for member_surface in [&surface.instance, &surface.class] {
            for callable in member_surface.callables.keys() {
                if semantic_snapshot.callable_signatures.get(callable).is_some() {
                    continue;
                }
                missing_signature_modules.insert(callable.module().clone());
                let retained_kind = member_surface.callable_signatures.get(&callable.selector).map(|signature| &signature.kind);
                let conditional_members = semantic_snapshot
                    .dispatch
                    .get_conditional_members(&phalcom_semantic::impls::InherentImplTarget::Declaration(owner.clone()))
                    .map(|set| {
                        set.members
                            .iter()
                            .filter(|member| &member.callable == callable)
                            .map(|member| (member.signature_template.native_id.clone(), member.is_requirement, member.source_member))
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default();
                missing_surface_signatures.push((
                    callable.clone(),
                    retained_kind,
                    semantic_snapshot.callable_definitions.get(callable).is_some(),
                    conditional_members,
                ));
            }
        }
    }
    missing_surface_signatures.sort_by(|left, right| left.0.cmp(&right.0));
    eprintln!(
        "canonical surface callables missing signatures (callable, retained kind, definition present, conditional native/requirement/source): {missing_surface_signatures:#?}"
    );
    for module in missing_signature_modules {
        if let Some(diagnostics) = semantic_snapshot.diagnostics.get(&module) {
            for diagnostic in diagnostics.iter() {
                eprintln!("canonical missing-signature module {module}: {:?}: {}", diagnostic.code, diagnostic.message);
            }
        }
    }
    let program = canonical_universe_program().expect("canonical universe program must compile");
    let view = program.compiler_image_view().expect("compiler image view must be present");
    let limits = ValidationLimits::default();

    // 1. Build deterministic universe.telosc artifact:
    let built = build_universe_telosc(&view, &limits).expect("build_universe_telosc must succeed");
    assert!(!built.bytes.is_empty());
    assert!(!built.summary.capabilities.portable_executable);
    assert!(built.summary.capabilities.module_world);
    assert!(built.summary.capabilities.semantic_world);
    assert!(built.summary.capabilities.query_image);
    assert!(built.summary.capabilities.runtime_public_metadata);
    assert!(built.summary.module_world_bytes > 0);
    assert!(built.summary.semantic_world_bytes > 0);
    assert!(built.summary.query_image_bytes > 0);
    assert!(built.summary.metadata_bytes > 0);

    // 2. Determinism check: building again from the same canonical program produces byte-identical output:
    let built2 = build_universe_telosc(&view, &limits).expect("second build must succeed");
    assert_eq!(built.bytes, built2.bytes, "build_universe_telosc must be bitwise deterministic");
    assert_eq!(built.summary.artifact_digest, built2.summary.artifact_digest);

    // 3. Atomic publication:
    let dir = tempdir().unwrap();
    let path = dir.path().join("universe.telosc");
    write_universe_telosc(&path, &built).expect("write_universe_telosc must succeed");
    assert!(path.is_file());

    // 4. Source-free loader from disk:
    let loaded_from_disk = load_universe_telosc(&path, &limits).expect("load_universe_telosc must succeed");
    assert!(loaded_from_disk.formal_image.sources().is_empty(), "loaded image must not retain source units");
    assert!(
        loaded_from_disk.formal_image.snapshot().is_source_free_artifact_complete(),
        "loaded snapshot must be source-free complete"
    );

    // 5. Source-free loader from in-memory bytes:
    let loaded = load_universe_telosc_from_bytes(&built.bytes, &limits).expect("load_universe_telosc_from_bytes must succeed");
    assert!(loaded.formal_image.sources().is_empty());
    assert!(loaded.formal_image.snapshot().is_source_free_artifact_complete());
    assert!(loaded.formal_image.reader().is_ok(), "loaded formal image must remain query-reader compatible");

    let snapshot = loaded.formal_image.snapshot();
    assert!(!snapshot.field_signatures.is_empty(), "field signatures must be restored");
    assert!(snapshot.trait_headers.len() > 0, "trait headers must be restored");
    assert!(snapshot.trait_surfaces.iter().next().is_some(), "trait surfaces must be restored");
    let sample_surface = snapshot.trait_surfaces.iter().find(|(_, s)| !s.members.is_empty()).map(|(_, s)| s);
    assert!(sample_surface.is_some(), "trait surface members must be populated");
    assert!(snapshot.constructor_initializations.len() > 0, "constructor initializations must be restored");
    assert!(snapshot.conformance_index.iter().next().is_some(), "conformance index must be restored");
    let (_, sample_head) = snapshot.conformance_index.iter().next().unwrap();
    assert!(sample_head.authorized, "conformance head must preserve authorized flag");
    assert!(!snapshot.conformance_witness_plans.is_empty(), "witness plans must be restored");
    let sample_witness = snapshot
        .conformance_witness_plans
        .iter()
        .find(|(_, w)| !w.requirements.is_empty())
        .map(|(_, w)| w);
    assert!(sample_witness.is_some(), "witness plan requirements must be restored");
    assert!(matches!(
        sample_witness.unwrap().completeness,
        phalcom_semantic::impls::ConformanceCompleteness::Complete
    ));

    // 6. User compilation consuming the source-free loaded image:
    let compiled = loaded
        .compile_user_entry(EntrySelection::Inline("print(42);".into()))
        .expect("compiling user program against loaded universe image must succeed");
    assert!(!compiled.modules.is_empty());
    assert!(compiled.entry.path.is_root());

    // Also compile user code exercising Universe prelude types (Option, String):
    let compiled_user = loaded
        .compile_user_entry(EntrySelection::Inline("let s = \"hello\"; let opt = Option.some(s); print(opt);".into()))
        .expect("compiling user program exercising Universe prelude types must succeed");
    assert!(!compiled_user.modules.is_empty());

    // Also compile a multi-file dependent user package:
    let pkg_dir = tempdir().unwrap();
    std::fs::write(pkg_dir.path().join("package.ph"), "expose .main\n").expect("write package.ph");
    std::fs::write(
        pkg_dir.path().join("main.ph"),
        "let message = \"package user program\";\nlet opt = Option.some(message);\nprint(opt);\n",
    )
    .expect("write main.ph");
    let compiled_pkg = loaded
        .compile_user_entry(EntrySelection::Package(pkg_dir.path().to_path_buf()))
        .expect("compiling user package against loaded universe image must succeed");
    assert!(!compiled_pkg.modules.is_empty());
}

#[test]
fn fresh_process_source_bootstrap_negative_control_runner() {
    if std::env::var("PHALCOM_TEST_NEGATIVE_CONTROL").is_err() {
        return;
    }
    // Attempt canonical source bootstrap; must panic because PHALCOM_DENY_SOURCE_BOOTSTRAP=1 is set.
    let _ = canonical_universe_formal_image();
}

#[test]
fn fresh_process_target_runner() {
    let Ok(telosc_path) = std::env::var("PHALCOM_TEST_TELOSC_PATH") else {
        return;
    };
    let pkg_path = std::env::var("PHALCOM_TEST_PACKAGE_PATH").expect("package path");
    let limits = ValidationLimits::default();

    // Verify 0 source bootstrap attempts prior to loading
    assert_eq!(
        phalcom_semantic::canonical_image::canonical_universe_formal_image_build_count(),
        0,
        "source bootstrap build count before loading must be 0"
    );

    let loaded = load_universe_telosc(&std::path::PathBuf::from(telosc_path), &limits).expect("load_universe_telosc in fresh process must succeed");
    assert!(loaded.formal_image.sources().is_empty(), "fresh process loaded image has no sources");
    assert!(
        loaded.formal_image.snapshot().is_source_free_artifact_complete(),
        "fresh process loaded snapshot is source-free complete"
    );

    // Verify 0 source bootstrap attempts after loading artifact
    assert_eq!(
        phalcom_semantic::canonical_image::canonical_universe_formal_image_build_count(),
        0,
        "source bootstrap build count after loading must be 0"
    );

    let compiled = loaded
        .compile_user_entry(EntrySelection::Package(std::path::PathBuf::from(pkg_path)))
        .expect("compile package in fresh process against loaded artifact must succeed");
    assert!(!compiled.modules.is_empty(), "compiled modules must be non-empty");

    // SlotMap compilation is mandatory in fresh source-denied process:
    let slotmap_path = std::env::var("PHALCOM_TEST_SLOTMAP_PATH").expect("PHALCOM_TEST_SLOTMAP_PATH must be set; slotmap compilation is mandatory");
    let compiled_slotmap = loaded
        .compile_user_entry(EntrySelection::Project(std::path::PathBuf::from(slotmap_path)))
        .expect("compile slotmap project in fresh process against loaded artifact must succeed");
    assert_eq!(compiled_slotmap.modules.len(), 4, "slotmap project modules count must be 4");

    // Verify 0 source bootstrap attempts after package compilation
    assert_eq!(
        phalcom_semantic::canonical_image::canonical_universe_formal_image_build_count(),
        0,
        "source bootstrap build count after package compilation must be 0"
    );
}

/// Proves that compiling against a loaded Universe artifact operates with zero
/// source bootstrap in a fresh process under `PHALCOM_DENY_SOURCE_BOOTSTRAP=1`.
///
/// Denial instrumentation behavior:
/// - `canonical_universe_program()` checks `PHALCOM_DENY_SOURCE_BOOTSTRAP` and panics.
/// - `canonical_universe_formal_image()` checks `PHALCOM_DENY_SOURCE_BOOTSTRAP` and panics.
/// - `UniverseSourceProvider::read_source()` checks `PHALCOM_DENY_SOURCE_BOOTSTRAP` and panics.
/// - `PreludeTypeMap::canonical_universe()` accesses `UniverseSourceProvider::new()`,
///   so invoking it while source bootstrap is denied will panic in `UniverseSourceProvider`.
/// - `PreludeTypeMap::initialize_from_snapshot()` does NOT access `UniverseSourceProvider`
///   or parse sources; it initializes the prelude directly from the artifact snapshot.
#[test]
fn fresh_process_source_denied_artifact_compilation() {
    let (_dir, telosc_path) = if let Ok(path) = std::env::var("PHALCOM_UNIVERSE_TELOSC_FOR_TEST").map(std::path::PathBuf::from) {
        if path.is_file() {
            (None, path)
        } else {
            let program = canonical_universe_program().expect("canonical universe program");
            let view = program.compiler_image_view().expect("compiler image view");
            let limits = ValidationLimits::default();
            let built = build_universe_telosc(&view, &limits).expect("build_universe_telosc must succeed");
            let dir = tempdir().unwrap();
            let telosc_path = dir.path().join("universe.telosc");
            write_universe_telosc(&telosc_path, &built).expect("write_universe_telosc must succeed");
            (Some(dir), telosc_path)
        }
    } else {
        let program = canonical_universe_program().expect("canonical universe program");
        let view = program.compiler_image_view().expect("compiler image view");
        let limits = ValidationLimits::default();
        let built = build_universe_telosc(&view, &limits).expect("build_universe_telosc must succeed");
        let dir = tempdir().unwrap();
        let telosc_path = dir.path().join("universe.telosc");
        write_universe_telosc(&telosc_path, &built).expect("write_universe_telosc must succeed");
        (Some(dir), telosc_path)
    };

    let pkg_dir = tempdir().unwrap();
    std::fs::write(pkg_dir.path().join("package.ph"), "expose .main\n").expect("write package.ph");
    std::fs::write(
        pkg_dir.path().join("main.ph"),
        "let message = \"fresh process verification\";\nlet opt = Option.some(message);\nprint(opt);\n",
    )
    .expect("write main.ph");

    let slotmap_pkg = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().join("packages/slotmap");
    assert!(slotmap_pkg.exists(), "packages/slotmap must exist at {:?}", slotmap_pkg);

    let current_exe = std::env::current_exe().expect("current test executable");

    // 1. Negative control: attempt canonical source bootstrap under PHALCOM_DENY_SOURCE_BOOTSTRAP=1 and assert it fails
    let mut neg_cmd = std::process::Command::new(&current_exe);
    neg_cmd
        .arg("--exact")
        .arg("modules_canonical_artifact::fresh_process_source_bootstrap_negative_control_runner")
        .arg("--nocapture")
        .env("PHALCOM_DENY_SOURCE_BOOTSTRAP", "1")
        .env("PHALCOM_TEST_NEGATIVE_CONTROL", "1");
    let neg_output = neg_cmd.output().expect("spawn negative control runner");
    assert!(
        !neg_output.status.success(),
        "Canonical source bootstrap must fail under PHALCOM_DENY_SOURCE_BOOTSTRAP=1"
    );
    let neg_stderr = String::from_utf8_lossy(&neg_output.stderr);
    assert!(
        neg_stderr.contains("PHALCOM_DENY_SOURCE_BOOTSTRAP"),
        "Negative control failure must cite PHALCOM_DENY_SOURCE_BOOTSTRAP, got stderr: {}",
        neg_stderr
    );

    // 2. Positive proof: loaded artifact compiles packages under PHALCOM_DENY_SOURCE_BOOTSTRAP=1 with zero source reads
    let mut cmd = std::process::Command::new(current_exe);
    cmd.arg("--exact")
        .arg("modules_canonical_artifact::fresh_process_target_runner")
        .arg("--nocapture")
        .env("PHALCOM_DENY_SOURCE_BOOTSTRAP", "1")
        .env("PHALCOM_TEST_TELOSC_PATH", &telosc_path)
        .env("PHALCOM_TEST_PACKAGE_PATH", pkg_dir.path())
        .env("PHALCOM_TEST_SLOTMAP_PATH", slotmap_pkg);

    let output = cmd.output().expect("spawn fresh process target runner");

    if !output.status.success() {
        panic!(
            "Fresh process failed with code {:?}:\nStdout: {}\nStderr: {}",
            output.status.code(),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

#[test]
fn source_backed_and_artifact_backed_compiler_program_parity() {
    let program = canonical_universe_program().expect("canonical universe program");
    let view = program.compiler_image_view().expect("compiler image view");
    let limits = ValidationLimits::default();
    let built = build_universe_telosc(&view, &limits).expect("build");
    let loaded = load_universe_telosc_from_bytes(&built.bytes, &limits).expect("load");
    let source_image = canonical_universe_formal_image().expect("canonical universe formal image");

    // 1. Real package: packages/slotmap (4 modules)
    let slotmap_pkg = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().join("packages/slotmap");
    assert!(slotmap_pkg.exists(), "packages/slotmap must exist");
    let entry = EntrySelection::Project(slotmap_pkg);
    let src_slotmap = ProgramCompiler::compile_entry_selection_with_image(entry.clone(), source_image.clone()).expect("source compilation of packages/slotmap");
    let art_slotmap = loaded.compile_user_entry(entry).expect("artifact compilation of packages/slotmap");
    assert_eq!(src_slotmap.modules.len(), 4);
    assert_compiled_program_parity(&src_slotmap, &art_slotmap);

    // 2. Real package: packages/slotmap/tests/runtime (8 modules)
    let slotmap_tests_pkg = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("packages/slotmap/tests/runtime");
    assert!(slotmap_tests_pkg.exists(), "packages/slotmap/tests/runtime must exist");
    let entry = EntrySelection::Project(slotmap_tests_pkg);
    let src_tests =
        ProgramCompiler::compile_entry_selection_with_image(entry.clone(), source_image.clone()).expect("source compilation of slotmap runtime tests");
    let art_tests = loaded.compile_user_entry(entry).expect("artifact compilation of slotmap runtime tests");
    assert_eq!(src_tests.modules.len(), 8);
    assert_compiled_program_parity(&src_tests, &art_tests);
}

#[test]
fn artifact_backed_slotmap_runtime_tests_bootstrap_vm_with_generic_intrinsic_adts() {
    let program = canonical_universe_program().expect("canonical universe program");
    let view = program.compiler_image_view().expect("compiler image view");
    let limits = ValidationLimits::default();
    let built = build_universe_telosc(&view, &limits).expect("build");
    let loaded = load_universe_telosc_from_bytes(&built.bytes, &limits).expect("load");

    let slotmap_tests_pkg = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("packages/slotmap/tests/runtime");
    assert!(slotmap_tests_pkg.exists(), "packages/slotmap/tests/runtime must exist");
    let entry = EntrySelection::Project(slotmap_tests_pkg);
    let art_tests = loaded.compile_user_entry(entry).expect("artifact compilation of slotmap runtime tests");
    assert_eq!(art_tests.modules.len(), 8);

    let _vm = phalcom_core::vm::VM::new();
}

#[test]
fn source_backed_and_artifact_backed_negative_diagnostic_parity() {
    let program = canonical_universe_program().expect("canonical universe program");
    let view = program.compiler_image_view().expect("compiler image view");
    let limits = ValidationLimits::default();
    let built = build_universe_telosc(&view, &limits).expect("build");
    let loaded = load_universe_telosc_from_bytes(&built.bytes, &limits).expect("load");
    let source_image = canonical_universe_formal_image().expect("canonical universe formal image");

    // Invalid program exercising canonical semantic authority:
    // Calling List.withCapacity with incompatible argument type String instead of Int
    let invalid_entry = EntrySelection::Inline(
        r#"
let list = List.withCapacity("not an int");
"#
        .into(),
    );

    let src_res = ProgramCompiler::compile_entry_selection_with_image(invalid_entry.clone(), source_image.clone());
    let art_res = loaded.compile_user_entry(invalid_entry);

    let src_err = src_res.expect_err("source compilation must fail on type mismatch with canonical Option");
    let art_err = art_res.expect_err("artifact compilation must fail on type mismatch with canonical Option");

    match (&src_err, &art_err) {
        (ProgramCompileError::Semantic(src_diags), ProgramCompileError::Semantic(art_diags)) => {
            assert!(!src_diags.by_module.is_empty(), "must have semantic diagnostics");
            assert_eq!(src_diags.by_module.len(), art_diags.by_module.len());
        }
        _ => panic!("Expected semantic errors, got src: {:?}, art: {:?}", src_err, art_err),
    }

    assert_compile_error_parity(&src_err, &art_err);
}

fn assert_negative_diagnostic_parity_for_source(source: &str, expected_code: DiagnosticCode) {
    let program = canonical_universe_program().expect("canonical universe program");
    let view = program.compiler_image_view().expect("compiler image view");
    let limits = ValidationLimits::default();
    let built = build_universe_telosc(&view, &limits).expect("build");
    let loaded = load_universe_telosc_from_bytes(&built.bytes, &limits).expect("load");
    let source_image = canonical_universe_formal_image().expect("canonical universe formal image");

    let invalid_entry = EntrySelection::Inline(source.into());
    let src_res = ProgramCompiler::compile_entry_selection_with_image(invalid_entry.clone(), source_image.clone());
    let art_res = loaded.compile_user_entry(invalid_entry);

    let src_err = src_res.expect_err("source compilation must fail");
    let art_err = art_res.expect_err("artifact compilation must fail");

    match (&src_err, &art_err) {
        (ProgramCompileError::Semantic(src_diags), ProgramCompileError::Semantic(art_diags)) => {
            assert!(!src_diags.by_module.is_empty(), "must have semantic diagnostics");
            assert_eq!(src_diags.by_module.len(), art_diags.by_module.len());
            let src_has_code = src_diags.by_module.values().any(|diags| diags.iter().any(|d| d.code == expected_code));
            assert!(
                src_has_code,
                "source diagnostics must contain expected code {:?}, got: {:?}",
                expected_code, src_diags
            );
            let art_has_code = art_diags.by_module.values().any(|diags| diags.iter().any(|d| d.code == expected_code));
            assert!(
                art_has_code,
                "artifact diagnostics must contain expected code {:?}, got: {:?}",
                expected_code, art_diags
            );
        }
        _ => panic!("Expected semantic errors, got src: {:?}, art: {:?}", src_err, art_err),
    }

    assert_compile_error_parity(&src_err, &art_err);
}

#[test]
fn negative_parity_constructor_resolution_failure() {
    let source = r#"
class Parent {
  @constructor new() {}
}
class Child is Parent {
  @constructor new() {
    super.new(42)
  }
}
"#;
    assert_negative_diagnostic_parity_for_source(source, DiagnosticCode::ConstructorSuperInvalid);
}

#[test]
fn negative_parity_inaccessible_declaration() {
    let source = r#"
data Secret private(_ value: Int)
let s = Secret(42);
"#;
    assert_negative_diagnostic_parity_for_source(source, DiagnosticCode::MemberInaccessible);
}

#[test]
fn negative_parity_invalid_pattern_matching() {
    let source = r#"
class Probe {
  test(_ opt: Option<Int>) {
    match opt {
      Option::Some(a, b) => 1
      _ => 2
    }
  }
}
"#;
    assert_negative_diagnostic_parity_for_source(source, DiagnosticCode::MatchPatternArityMismatch);
}

#[test]
fn negative_parity_trait_resolution_failure() {
    let source = r#"
trait Printable {
  printMe() -> Unit
}
class Host {
  @class display<T>(_ item: T) where T |= Printable {
    item.printMe()
  }
}
class NotPrintable {}
let x = Host.display(NotPrintable());
"#;
    assert_negative_diagnostic_parity_for_source(source, DiagnosticCode::GenericConstraintUnsatisfied);
}

#[test]
fn load_universe_telosc_rejects_corrupted_artifact() {
    let program = canonical_universe_program().expect("canonical universe program");
    let view = program.compiler_image_view().expect("compiler image view");
    let limits = ValidationLimits::default();
    let mut built = build_universe_telosc(&view, &limits).expect("build");

    // Corrupt one byte in the container payload:
    let mid = built.bytes.len() / 2;
    built.bytes[mid] ^= 0xFF;

    let res = load_universe_telosc_from_bytes(&built.bytes, &limits);
    assert!(res.is_err(), "corrupted container bytes must be rejected");
}

#[test]
fn foreign_native_catalog_fingerprint_is_rejected_before_section_materialization() {
    use phalcom_compiler_artifact::{
        ArtifactDigest,
        container::TeloscContainerBuilder,
        header::{TELOSC_CONTAINER_SCHEMA_V2, TELOSC_MIN_READER_SCHEMA_V2, TeloscCapabilities, TeloscHeaderV2},
        identity::TeloscPackageIdentity,
        section::{SectionFlags, SectionKind},
    };

    let mut foreign_catalog = phalcom_native_surface::catalog_fingerprint().0;
    foreign_catalog[0] ^= 0xff;
    let header = TeloscHeaderV2 {
        container_schema: TELOSC_CONTAINER_SCHEMA_V2,
        minimum_reader_schema: TELOSC_MIN_READER_SCHEMA_V2,
        package_identity: TeloscPackageIdentity::universe(),
        semantic_model_version: phalcom_type_meta::SEMANTIC_MODEL_VERSION,
        stable_identity_schema: 1,
        module_semantic_abi: 1,
        semantic_artifact_abi: 1,
        query_fingerprint_abi: 1,
        native_surface_schema_version: phalcom_native_meta::NATIVE_SURFACE_SCHEMA_VERSION,
        native_catalog_fingerprint: foreign_catalog,
        type_metadata_schema_version: phalcom_type_meta::TYPE_METADATA_SCHEMA_VERSION,
        capabilities: TeloscCapabilities::CANONICAL_COMPILER_IMAGE_V1 | TeloscCapabilities::RUNTIME_PUBLIC_METADATA,
        source_digest: ArtifactDigest([1; 32]),
        interface_digest: ArtifactDigest([2; 32]),
        semantic_digest: ArtifactDigest([3; 32]),
        query_digest: ArtifactDigest([4; 32]),
        metadata_digest: ArtifactDigest([5; 32]),
        artifact_digest: ArtifactDigest::ZERO,
        section_count: 0,
    };
    let mut builder = TeloscContainerBuilder::new(header);
    for kind in [
        SectionKind::ModuleWorld,
        SectionKind::SemanticWorld,
        SectionKind::QueryImage,
        SectionKind::Metadata,
    ] {
        let schema = if kind == SectionKind::SemanticWorld {
            phalcom_compiler_artifact::semantic::world::SEMANTIC_WORLD_SCHEMA_V3
        } else {
            1
        };
        builder.add_section(kind, schema, SectionFlags::REQUIRED, vec![0xff]);
    }
    let bytes = builder.build().expect("container integrity should be valid");

    assert!(matches!(
        load_universe_telosc_from_bytes(&bytes, &ValidationLimits::default()),
        Err(phalcom_core::modules::canonical_artifact::TeloscArtifactError::Artifact(
            phalcom_compiler_artifact::ArtifactError::Validation(phalcom_compiler_artifact::ValidationError::NativeCatalogFingerprintMismatch { .. })
        ))
    ));
}

#[test]
fn write_universe_telosc_rejects_portable_executable_capability() {
    let program = canonical_universe_program().expect("canonical universe program");
    let view = program.compiler_image_view().expect("compiler image view");
    let limits = ValidationLimits::default();
    let mut built = build_universe_telosc(&view, &limits).expect("build");

    // P7 compiler-only writer must refuse PORTABLE_EXECUTABLE:
    built.summary.capabilities.portable_executable = true;
    let dir = tempdir().unwrap();
    let path = dir.path().join("universe.telosc");
    assert!(write_universe_telosc(&path, &built).is_err());
    assert!(!path.exists());
}

#[test]
fn load_universe_telosc_enforces_budget_limits_and_records_timings() {
    let program = canonical_universe_program().expect("canonical universe program");
    let view = program.compiler_image_view().expect("compiler image view");
    let limits = ValidationLimits::default();
    let built = build_universe_telosc(&view, &limits).expect("build");

    // Reject when decoded section byte limit is too small:
    let tight_limits = ValidationLimits {
        max_decoded_section_bytes: 100,
        ..ValidationLimits::default()
    };
    let res = load_universe_telosc_from_bytes(&built.bytes, &tight_limits);
    assert!(res.is_err());

    // Successful load records load timings:
    let loaded = load_universe_telosc_from_bytes(&built.bytes, &limits).expect("load");
    assert!(loaded.timings.section_validation.as_nanos() > 0);
    assert!(loaded.timings.relocation.as_nanos() > 0);
}

#[test]
fn load_universe_telosc_enforces_cumulative_transaction_budget() {
    let program = canonical_universe_program().expect("canonical universe program");
    let view = program.compiler_image_view().expect("compiler image view");
    let limits = ValidationLimits::default();
    let built = build_universe_telosc(&view, &limits).expect("build");

    let max_single_section = built
        .summary
        .module_world_bytes
        .max(built.summary.semantic_world_bytes)
        .max(built.summary.query_image_bytes)
        .max(built.summary.metadata_bytes);

    let cumulative_insufficient = (max_single_section + 1024) as u64;
    let budget_limits = ValidationLimits {
        max_total_allocation_bytes: cumulative_insufficient,
        ..ValidationLimits::default()
    };
    let res = load_universe_telosc_from_bytes(&built.bytes, &budget_limits);
    assert!(
        res.is_err(),
        "cumulative budget must fail when total section allocations exceed transaction budget"
    );
}

#[test]
fn bounded_file_acquisition_rejects_oversized_file() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("oversized.telosc");
    let file = std::fs::File::create(&path).unwrap();
    file.set_len(1024 * 1024 * 1024).unwrap();
    let limits = ValidationLimits {
        max_artifact_bytes: 16,
        ..Default::default()
    };
    assert!(matches!(
        load_universe_telosc(&path, &limits),
        Err(phalcom_core::modules::canonical_artifact::TeloscArtifactError::BudgetExceeded { .. })
    ));
}

#[test]
fn writer_preserves_previous_output_when_bytes_are_invalid() {
    use phalcom_core::modules::canonical_artifact::{BuiltUniverseTelosc, TeloscBuildSummary, TeloscCapabilities};
    let dir = tempdir().unwrap();
    let path = dir.path().join("universe.telosc");
    std::fs::write(&path, b"previous output").unwrap();
    let built = BuiltUniverseTelosc {
        bytes: vec![1, 2, 3],
        summary: TeloscBuildSummary {
            capabilities: TeloscCapabilities::compiler_only_v1(),
            module_world_bytes: 0,
            semantic_world_bytes: 0,
            query_image_bytes: 0,
            metadata_bytes: 0,
            artifact_digest: [0; 32],
        },
    };
    assert!(write_universe_telosc(&path, &built).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), b"previous output");
    assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 1);
}

#[test]
fn loader_rejects_rehashed_incompatible_header_fields() {
    use phalcom_compiler_artifact::{
        container::{TeloscContainerBuilder, TeloscContainerView},
        header::TeloscCapabilities,
        section::{SectionFlags, SectionKind},
    };
    let program = canonical_universe_program().unwrap();
    let view = program.compiler_image_view().unwrap();
    let limits = ValidationLimits::default();
    let built = build_universe_telosc(&view, &limits).unwrap();
    let parsed = TeloscContainerView::parse(&built.bytes, &limits).unwrap();
    for field in 0..12 {
        let mut header = parsed.header().clone();
        match field {
            0 => header.semantic_model_version += 1,
            1 => header.stable_identity_schema += 1,
            2 => header.module_semantic_abi += 1,
            3 => header.semantic_artifact_abi += 1,
            4 => header.query_fingerprint_abi += 1,
            5 => header.native_surface_schema_version += 1,
            6 => header.type_metadata_schema_version += 1,
            7 => {
                header.package_identity = phalcom_compiler_artifact::identity::TeloscPackageIdentity::Builtin {
                    namespace: "other".into(),
                    version: "0.1.0".into(),
                }
            }
            8 => header.capabilities.remove(TeloscCapabilities::CANONICAL_COMPILER_IMAGE_V1),
            9 => header.capabilities.insert(TeloscCapabilities::PRESENTATION),
            10 => header.native_catalog_fingerprint[0] ^= 0xff,
            11 => {
                header.package_identity = phalcom_compiler_artifact::identity::TeloscPackageIdentity::Builtin {
                    namespace: "universe".into(),
                    version: "0.2.0".into(),
                }
            }
            _ => unreachable!(),
        }
        let mut builder = TeloscContainerBuilder::new(header);
        for kind in [
            SectionKind::ModuleWorld,
            SectionKind::SemanticWorld,
            SectionKind::QueryImage,
            SectionKind::Metadata,
        ] {
            let schema = if kind == SectionKind::SemanticWorld {
                phalcom_compiler_artifact::semantic::world::SEMANTIC_WORLD_SCHEMA_V3
            } else {
                1
            };
            builder.add_section(kind, schema, SectionFlags::REQUIRED, parsed.require_section(kind).unwrap().to_vec());
        }
        let bytes = builder.build().unwrap();
        // Integrity remains valid; the reader must reject compatibility itself.
        TeloscContainerView::parse(&bytes, &limits).unwrap();
        let result = load_universe_telosc_from_bytes(&bytes, &limits);
        if field == 10 {
            assert!(
                matches!(
                    result,
                    Err(phalcom_core::modules::canonical_artifact::TeloscArtifactError::Artifact(
                        phalcom_compiler_artifact::ArtifactError::Validation(
                            phalcom_compiler_artifact::ValidationError::NativeCatalogFingerprintMismatch { .. }
                        )
                    ))
                ),
                "foreign native catalog fingerprint must fail before section materialization"
            );
        } else {
            assert!(result.is_err(), "header field {field}");
        }
    }
}
