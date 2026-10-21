use phalcom_common::selector::Selector;
use phalcom_modules::ProjectUniverse;
use phalcom_modules::graph::{DependencyPhase, RuntimeDependencyReason};
use phalcom_modules::identity::{ModuleComponent, ModuleId, ModulePath, ResolvedProjectId};
use phalcom_modules::interface::InterfaceBuilder;
use phalcom_modules::linker::{ImportPhaseSummary, LinkedReadSpec, ModuleLinker, RuntimeDependencyRequirement, SemanticLinkedProgram, SymbolId};
use phalcom_modules::source::{ModuleKind, ParsedModuleUnit};
use phalcom_semantic::checker::body::{BodyAnalysisContext, CallableBodyRequest};
use phalcom_semantic::checker::{CallableAnalysis, analyze_callable_body};
use phalcom_semantic::db::{CancellationToken, QueryBudget};
use phalcom_semantic::declarations::{DeclarationTypeTable, NominalDeclarationHeader};
use phalcom_semantic::dispatch::SurfaceDispatchResolver;
use phalcom_semantic::identity::{CallableId, DeclarationId, DispatchSide};
use phalcom_semantic::phase_evidence::compute_module_import_phase_products;
use phalcom_semantic::resolver::LinkedTypeResolver;
use phalcom_semantic::session::SemanticWorkspaceSession;
use phalcom_semantic::source::ParsedModuleUnit as SemanticParsedModuleUnit;
use phalcom_semantic::source_index::{SourceIndexContext, SourceSemanticIndex, build_source_scope_index};
use phalcom_semantic::types::relation::MapTypeHierarchy;
use phalcom_semantic::types::store::TypeStore;
use phalcom_semantic::workspace::SemanticWorkspaceInput;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::Arc;

fn module(name: &str) -> ModuleId {
    ModuleId::resolved(
        ResolvedProjectId::from_raw(808),
        ModulePath::from_components(vec![ModuleComponent::from_identifier(name).expect("valid module name")]),
    )
}

struct PhaseFixture {
    semantic: Arc<SemanticLinkedProgram>,
    sources: BTreeMap<ModuleId, Arc<SemanticParsedModuleUnit>>,
    source_index: SourceSemanticIndex,
    provider: ModuleId,
    consumer: ModuleId,
    unrelated: ModuleId,
}

fn phase_fixture(provider_source: &str, consumer_source: &str, unrelated_source: &str) -> PhaseFixture {
    let provider = module("provider");
    let consumer = module("consumer");
    let unrelated = module("unrelated");
    let source_texts = [
        (provider.clone(), provider_source),
        (consumer.clone(), consumer_source),
        (unrelated.clone(), unrelated_source),
    ];
    let mut interfaces = BTreeMap::new();
    let mut sources = BTreeMap::new();
    for (id, source) in source_texts {
        let parsed = phalcom_ast::parse(source, 0);
        assert!(parsed.errors.is_empty(), "parse errors in {id}: {:#?}", parsed.errors);
        let program = Arc::new(parsed.program);
        interfaces.insert(
            id.clone(),
            InterfaceBuilder::build(id.clone(), ModuleKind::Module, &program).expect("interface builds"),
        );
        sources.insert(
            id.clone(),
            Arc::new(ParsedModuleUnit::new(id, ModuleKind::Module, None, Arc::from(source), program)),
        );
    }

    let linker = ModuleLinker::new(Arc::new(ProjectUniverse::new()), interfaces);
    let semantic = Arc::new(
        linker
            .link_semantic(
                consumer.clone(),
                &BTreeMap::from([((consumer.clone(), ".provider".to_owned()), provider.clone())]),
            )
            .expect("symbolic link succeeds"),
    );
    let scopes = sources
        .iter()
        .map(|(id, source)| {
            (
                id.clone(),
                build_source_scope_index(id.clone(), &source.program, &SourceIndexContext::default()),
            )
        })
        .collect();
    let source_index = SourceSemanticIndex::from_scope_indices_with_programs(scopes, &sources);
    PhaseFixture {
        semantic,
        sources,
        source_index,
        provider,
        consumer,
        unrelated,
    }
}

fn imported_phase(fixture: &PhaseFixture, products: &BTreeMap<ModuleId, phalcom_modules::linker::ModuleImportPhaseProduct>) -> ImportPhaseSummary {
    let binding = fixture.semantic.modules[&fixture.consumer].imports.by_local_name["Imported"].clone();
    products[&fixture.consumer].bindings[&binding].clone()
}

fn runtime_value_analysis(fixture: &PhaseFixture) -> CallableAnalysis {
    analyze_body(fixture, "Imported").0
}

fn analyze_body(fixture: &PhaseFixture, body_source: &str) -> (CallableAnalysis, TypeStore) {
    let imported = DeclarationId::new(fixture.provider.clone(), "Imported".into());
    let resolver = LinkedTypeResolver::from_semantic(fixture.semantic.clone(), HashSet::from([imported.clone()]));
    let mut store = TypeStore::new();
    let mut declarations = DeclarationTypeTable::new();
    declarations.insert(NominalDeclarationHeader::from_signature(&mut store, imported, None).into_type_info(None));
    let hierarchy = MapTypeHierarchy::new();
    let dispatch = SurfaceDispatchResolver::new();
    let callable = CallableId::new(
        DeclarationId::new(fixture.consumer.clone(), "Consumer".into()),
        Selector::method("read", []).expect("method selector"),
        DispatchSide::Class,
    );
    let parsed = phalcom_ast::parse(body_source, 0);
    assert!(parsed.errors.is_empty(), "body parse errors: {:#?}", parsed.errors);
    let body = parsed.program.statements.as_slice();
    let cancel = CancellationToken::new();

    let analysis = analyze_callable_body(
        BodyAnalysisContext {
            store: &mut store,
            hierarchy: &hierarchy,
            resolver: &resolver,
            declarations: &declarations,
            dispatch: &dispatch,
            trait_surface: None,
            conformance_semantics: None,
            module: fixture.consumer.clone(),
            caller_package: None,
            access_topology: None,
            module_values: None,
            linked_module: None,
            semantic_linked_module: Some(&fixture.semantic.modules[&fixture.consumer]),
        },
        CallableBodyRequest {
            callable,
            body,
            body_range: Default::default(),
            owner_generic_signature: None,
            receiver_value_override: None,
            instance_self_override: None,
            declared_signature: None,
            budget: QueryBudget::default(),
            cancel: &cancel,
            field_signatures: None,
            field_lifecycle: None,
            constructor_initializations: None,
            callable_signatures: None,
            enum_semantics: None,
            data_semantics: None,
            associated_families: None,
            conformance_semantics: None,
        },
    );
    (analysis, store)
}

fn aggregate(
    fixture: &PhaseFixture,
    analyses: &HashMap<CallableId, Arc<CallableAnalysis>>,
) -> (
    BTreeMap<ModuleId, phalcom_modules::linker::ModuleImportPhaseProduct>,
    Vec<RuntimeDependencyRequirement>,
) {
    compute_module_import_phase_products(&fixture.semantic, &fixture.sources, &fixture.source_index, analyses, None, None, true)
}

fn session_with_private_numbers_child_exposed(workspace: phalcom_semantic::identity::WorkspaceId) -> SemanticWorkspaceSession {
    let universe_provider = phalcom_modules::UniverseSourceProvider::new();
    let numbers = ModuleId::universe(ModulePath::from_components(vec![
        ModuleComponent::from_identifier("numbers").expect("valid module name"),
    ]));
    let numbers_source = universe_provider.source_text(&numbers).expect("canonical numbers source");
    let compatible_numbers_source = numbers_source.replacen("from .capabilities import", "expose .capabilities\nfrom .capabilities import", 1);
    assert_ne!(
        compatible_numbers_source,
        numbers_source.as_ref(),
        "test fixture must amend the private child exposure"
    );
    let universe_provider = phalcom_modules::UniverseSourceProvider::with_test_source_override(numbers, Arc::from(compatible_numbers_source))
        .expect("test-only Universe source override is valid");
    SemanticWorkspaceSession::with_universe_source_provider(workspace, universe_provider)
}

#[test]
fn imported_value_use_is_classified_runtime() {
    let fixture = phase_fixture(
        "class Imported {}\nexport Imported\n",
        "from .provider import Imported\nclass Consumer { read() { Imported } }\n",
        "class Unchanged {}\n",
    );
    let analysis = runtime_value_analysis(&fixture);
    let analyses = HashMap::from([(analysis.callable.clone(), Arc::new(analysis))]);
    let (products, _) = aggregate(&fixture, &analyses);

    let phase = imported_phase(&fixture, &products);
    assert_eq!(phase.phase, DependencyPhase::Runtime);
    assert!(phase.observed, "runtime expression use must be retained as observed evidence");
}

#[test]
fn associated_and_family_selections_emit_external_callable_owner_requirements() {
    let fixture = phase_fixture(
        "class Imported {}\nexport Imported\n",
        "from .provider import Imported\nclass Consumer { read() { Imported } }\n",
        "class ExternalExtension {}\n",
    );
    let (mut analysis, _) = analyze_body(&fixture, "Imported");
    let expression_id = *analysis.expressions.keys().next().expect("analyzed source expression");
    let external_owner = DeclarationId::new(fixture.unrelated.clone(), "ExternalExtension".into());
    let external_callable = CallableId::new(
        external_owner.clone(),
        Selector::method("invoke", []).expect("callable selector"),
        DispatchSide::Class,
    );
    let invocation_target = phalcom_semantic::identity::InvocationTargetId::Behavioral(external_callable);

    analysis.associated_resolutions = Arc::new(
        BTreeMap::from([(
            expression_id,
            phalcom_semantic::checker::associated::AssociatedResolution {
                owner_form: phalcom_semantic::types::id::TypeId(0),
                lookup_owner: DeclarationId::new(fixture.provider.clone(), "Imported".into()),
                family: None,
                kind: phalcom_semantic::checker::associated::AssociatedResolutionKind::ExactCallable {
                    member: phalcom_semantic::associated::AssociatedMemberId::DataConstructor(phalcom_semantic::identity::DataConstructorId::new(
                        external_owner.clone(),
                    )),
                    target: invocation_target.clone(),
                    callable_type: phalcom_semantic::types::id::TypeId(0),
                },
            },
        )])
        .into(),
    );
    analysis.family_applications = Arc::new(
        BTreeMap::from([(
            expression_id,
            phalcom_semantic::checker::associated::FamilyApplicationResolution {
                family_type: phalcom_semantic::types::id::TypeId(0),
                kind: phalcom_semantic::checker::associated::FamilyApplicationKind::Associated,
                selection: phalcom_semantic::checker::associated::FamilyApplicationSelection::Static {
                    operation: phalcom_semantic::types::family::FamilyOperationShape::getter(),
                    target: Some(invocation_target),
                    callable_type: phalcom_semantic::types::id::TypeId(0),
                    result_type: phalcom_semantic::types::id::TypeId(0),
                },
            },
        )])
        .into(),
    );

    let analyses = HashMap::from([(analysis.callable.clone(), Arc::new(analysis))]);
    let (_, requirements) = aggregate(&fixture, &analyses);
    assert!(requirements.iter().any(|requirement| {
        requirement.importer == fixture.consumer
            && requirement.dependency == fixture.unrelated
            && requirement.provenance.producer == phalcom_modules::RuntimeRequirementProducer::CallableOwner
            && requirement.provenance.target == phalcom_modules::RuntimeRequirementTarget::Module(fixture.unrelated.clone())
            && requirement.read_target.is_none()
    }));
}

#[test]
fn analyzed_attribute_call_uses_canonical_external_callable_evidence() {
    let first_fixture = phase_fixture(
        "class Factory {\n  @class build() -> Int { 23 }\n}\nexport Factory\n",
        "from .provider import Factory\n@Factory(1)\nclass Consumer {}\n",
        "class Unchanged {}\n",
    );
    let changed_fixture = phase_fixture(
        "class Factory {\n  @class build() -> Int { 23 }\n}\nexport Factory\n",
        "from .provider import Factory\n@Factory(Factory.build())\nclass Consumer {}\n",
        "class Unchanged {}\n",
    );
    assert_eq!(first_fixture.consumer, changed_fixture.consumer);
    assert_eq!(first_fixture.provider, changed_fixture.provider);
    let mut session = session_with_private_numbers_child_exposed(phalcom_semantic::identity::WorkspaceId::from_raw(910));
    let first = session
        .update_with_budget_and_cancel(
            SemanticWorkspaceInput::new(first_fixture.semantic.to_unfinalized_program(), first_fixture.sources.clone(), 1)
                .with_semantic_linked(first_fixture.semantic.clone()),
            QueryBudget::default(),
            &CancellationToken::new(),
        )
        .expect("initial attribute analysis update succeeds")
        .snapshot;
    assert!(!first.generated_runtime_requirements.iter().any(|requirement| {
        requirement.importer == first_fixture.consumer
            && requirement.dependency == first_fixture.provider
            && requirement.provenance.producer == phalcom_modules::RuntimeRequirementProducer::CallableOwner
    }));

    let snapshot = session
        .update_with_budget_and_cancel(
            SemanticWorkspaceInput::new(changed_fixture.semantic.to_unfinalized_program(), changed_fixture.sources.clone(), 2)
                .with_semantic_linked(changed_fixture.semantic.clone()),
            QueryBudget::default(),
            &CancellationToken::new(),
        )
        .expect("changed attribute analysis update succeeds")
        .snapshot;
    let import_id = changed_fixture.semantic.modules[&changed_fixture.consumer].imports.by_local_name["Factory"].clone();
    assert_eq!(
        snapshot.import_phase_products[&changed_fixture.consumer].coverage,
        phalcom_modules::linker::PhaseEvidenceCoverage::Complete,
        "a fully checked supported attribute argument should close coverage"
    );
    assert_eq!(
        snapshot.import_phase_products[&changed_fixture.consumer].bindings[&import_id].phase,
        DependencyPhase::Runtime,
        "the attribute's selected static callable consumes the imported owner at runtime"
    );
    assert!(snapshot.generated_runtime_requirements.iter().any(|requirement| {
        requirement.importer == changed_fixture.consumer
            && requirement.dependency == changed_fixture.provider
            && requirement.provenance.producer == phalcom_modules::RuntimeRequirementProducer::CallableOwner
            && requirement.provenance.target == phalcom_modules::RuntimeRequirementTarget::Module(changed_fixture.provider.clone())
    }));

    let mut cold_session = session_with_private_numbers_child_exposed(phalcom_semantic::identity::WorkspaceId::from_raw(911));
    let cold = cold_session
        .update_with_budget_and_cancel(
            SemanticWorkspaceInput::new(changed_fixture.semantic.to_unfinalized_program(), changed_fixture.sources.clone(), 1)
                .with_semantic_linked(changed_fixture.semantic.clone()),
            QueryBudget::default(),
            &CancellationToken::new(),
        )
        .expect("cold analysis of changed attribute succeeds")
        .snapshot;
    assert_eq!(
        snapshot.generated_runtime_requirements.as_ref(),
        cold.generated_runtime_requirements.as_ref(),
        "incremental generated requirements must match a cold analysis after editing an attribute argument"
    );
    assert_eq!(
        snapshot.import_phase_products[&changed_fixture.consumer], cold.import_phase_products[&changed_fixture.consumer],
        "incremental attribute phase evidence must match a cold analysis"
    );
}

#[test]
fn selected_external_trait_witness_emits_its_runtime_owner_requirement() {
    let fixture = phase_fixture(
        "trait Tagged { tag -> String }\nclass User {}\nimpl Tagged for User { tag -> String { \"user\" } }\nexport Tagged, User\n",
        "from .provider import Tagged, User\nclass Caller { read(user: User) -> String { user.tag } }\n",
        "class Unchanged {}\n",
    );
    let mut session = session_with_private_numbers_child_exposed(phalcom_semantic::identity::WorkspaceId::from_raw(913));
    let snapshot = session
        .update_with_budget_and_cancel(
            SemanticWorkspaceInput::new(fixture.semantic.to_unfinalized_program(), fixture.sources.clone(), 1).with_semantic_linked(fixture.semantic.clone()),
            QueryBudget::default(),
            &CancellationToken::new(),
        )
        .expect("trait witness analysis succeeds")
        .snapshot;

    assert!(
        snapshot.generated_runtime_requirements.iter().any(|requirement| {
            requirement.importer == fixture.consumer
                && requirement.dependency == fixture.provider
                && requirement.provenance.producer == phalcom_modules::RuntimeRequirementProducer::CallableOwner
                && requirement.provenance.target == phalcom_modules::RuntimeRequirementTarget::Module(fixture.provider.clone())
        }),
        "selected witness implementation ownership must become a runtime requirement"
    );
    let user_import = fixture.semantic.modules[&fixture.consumer].imports.by_local_name["User"].clone();
    assert_eq!(
        snapshot.import_phase_products[&fixture.consumer].bindings[&user_import].phase,
        DependencyPhase::InterfaceOnly,
        "the parameter's nominal type stays interface-only; executable witness ownership is carried separately"
    );
}

#[test]
fn contract_attribute_analysis_retains_receiver_and_owner_callable_generics() {
    let fixture = phase_fixture(
        "class Provider {}\nexport Provider\n",
        "class Box<T> {\n  @requires(self is T and self is U)\n  read<U>(input: U) { input }\n}\n",
        "class Unchanged {}\n",
    );
    let mut session = session_with_private_numbers_child_exposed(phalcom_semantic::identity::WorkspaceId::from_raw(912));
    let snapshot = session
        .update_with_budget_and_cancel(
            SemanticWorkspaceInput::new(fixture.semantic.to_unfinalized_program(), fixture.sources.clone(), 1).with_semantic_linked(fixture.semantic.clone()),
            QueryBudget::default(),
            &CancellationToken::new(),
        )
        .expect("generic contract attribute analysis succeeds")
        .snapshot;

    assert_eq!(
        snapshot.import_phase_products[&fixture.consumer].coverage,
        phalcom_modules::linker::PhaseEvidenceCoverage::Complete,
        "receiver contracts should be complete when both declaration and callable binders resolve"
    );
    assert!(
        !snapshot.all_diagnostics().any(|diagnostic| diagnostic.primary.module == fixture.consumer),
        "contract analysis must resolve `self`, the class binder `T`, and the method binder `U`"
    );
}

#[test]
fn invariant_runtime_requirements_follow_the_selected_execution_mode() {
    let fixture = phase_fixture(
        "class Required {}\nclass Ensured {}\nclass InvariantValue {}\nclass BodyUsed {}\nexport Required, Ensured, InvariantValue, BodyUsed\n",
        "from .provider import Required, Ensured, InvariantValue, BodyUsed\nclass Consumer {\n  @requires(self is Required)\n  required() {}\n  @ensures(self is Ensured)\n  ensured() {}\n  @invariant(self is InvariantValue)\n  invariantChecked() {}\n  bodyUse() { BodyUsed }\n}\n",
        "class Unchanged {}\n",
    );
    let run = |mode, workspace| {
        let mut session = session_with_private_numbers_child_exposed(phalcom_semantic::identity::WorkspaceId::from_raw(workspace));
        session
            .update_with_budget_and_cancel(
                SemanticWorkspaceInput::new(fixture.semantic.to_unfinalized_program(), fixture.sources.clone(), 1)
                    .with_semantic_linked(fixture.semantic.clone())
                    .with_attribute_execution_mode(mode),
                QueryBudget::default(),
                &CancellationToken::new(),
            )
            .expect("invariant semantic analysis succeeds")
            .snapshot
    };
    let debug = run(phalcom_semantic::attribute_analysis::AttributeExecutionMode::Debug, 914);
    let release = run(phalcom_semantic::attribute_analysis::AttributeExecutionMode::Release, 915);
    let unchecked = run(phalcom_semantic::attribute_analysis::AttributeExecutionMode::Unchecked, 916);
    let has_read = |snapshot: &phalcom_semantic::SemanticSnapshot, name: &str| {
        let read_target = LinkedReadSpec::Binding(SymbolId {
            module: fixture.provider.clone(),
            name: name.into(),
        });
        snapshot
            .generated_runtime_requirements
            .iter()
            .any(|requirement| requirement.importer == fixture.consumer && requirement.read_target == Some(read_target.clone()))
    };
    let import_phase = |snapshot: &phalcom_semantic::SemanticSnapshot, name: &str| {
        let binding = fixture.semantic.modules[&fixture.consumer].imports.by_local_name[name].clone();
        snapshot.import_phase_products[&fixture.consumer].bindings[&binding].clone()
    };

    assert!(
        has_read(&debug, "Required") && has_read(&debug, "Ensured") && has_read(&debug, "InvariantValue"),
        "Debug emits runtime reads for all contract guards"
    );
    assert!(
        has_read(&release, "Required") && !has_read(&release, "Ensured") && !has_read(&release, "InvariantValue"),
        "Release retains preconditions and strips postconditions and invariants"
    );
    assert!(
        !has_read(&unchecked, "Required") && !has_read(&unchecked, "Ensured") && !has_read(&unchecked, "InvariantValue"),
        "Unchecked strips every contract guard"
    );
    assert_eq!(
        import_phase(&debug, "Ensured"),
        ImportPhaseSummary {
            phase: DependencyPhase::Runtime,
            observed: true
        }
    );
    assert_eq!(
        import_phase(&release, "Ensured"),
        ImportPhaseSummary {
            phase: DependencyPhase::InterfaceOnly,
            observed: true
        }
    );
    assert_eq!(
        import_phase(&release, "InvariantValue"),
        ImportPhaseSummary {
            phase: DependencyPhase::InterfaceOnly,
            observed: true
        }
    );
    assert_eq!(
        import_phase(&unchecked, "Required"),
        ImportPhaseSummary {
            phase: DependencyPhase::InterfaceOnly,
            observed: true
        }
    );
    assert_eq!(
        import_phase(&unchecked, "InvariantValue"),
        ImportPhaseSummary {
            phase: DependencyPhase::InterfaceOnly,
            observed: true
        }
    );
    for snapshot in [&debug, &release, &unchecked] {
        assert_eq!(
            import_phase(snapshot, "BodyUsed"),
            ImportPhaseSummary {
                phase: DependencyPhase::Runtime,
                observed: true
            },
            "an emitted body use remains Runtime in every compile mode"
        );
    }
    let finalize = |snapshot: &phalcom_semantic::SemanticSnapshot| {
        ModuleLinker::new(fixture.semantic.universe.clone(), BTreeMap::new())
            .finalize_executable_with_requirements(
                (*fixture.semantic).clone(),
                &snapshot.import_phase_products,
                &snapshot.generated_runtime_requirements,
            )
            .expect("mode-specific phase evidence finalizes")
    };
    // Reuse query products while changing only the projection policy. Both
    // stripping and restoring guards must agree with a cold analysis.
    let mut retained = session_with_private_numbers_child_exposed(phalcom_semantic::identity::WorkspaceId::from_raw(924));
    for (generation, mode, cold) in [
        (1, phalcom_semantic::attribute_analysis::AttributeExecutionMode::Debug, &debug),
        (2, phalcom_semantic::attribute_analysis::AttributeExecutionMode::Release, &release),
        (3, phalcom_semantic::attribute_analysis::AttributeExecutionMode::Unchecked, &unchecked),
        (4, phalcom_semantic::attribute_analysis::AttributeExecutionMode::Debug, &debug),
    ] {
        let incremental = retained
            .update_with_budget_and_cancel(
                SemanticWorkspaceInput::new(fixture.semantic.to_unfinalized_program(), fixture.sources.clone(), generation)
                    .with_semantic_linked(fixture.semantic.clone())
                    .with_attribute_execution_mode(mode),
                QueryBudget::default(),
                &CancellationToken::new(),
            )
            .expect("mode transition publishes")
            .snapshot;
        assert_eq!(
            incremental.import_phase_products, cold.import_phase_products,
            "phase evidence differs after {mode:?}"
        );
        assert_eq!(
            incremental.generated_runtime_requirements, cold.generated_runtime_requirements,
            "requirements differ after {mode:?}"
        );
        assert_eq!(
            finalize(&incremental).modules[&fixture.consumer].linked_reads,
            finalize(cold).modules[&fixture.consumer].linked_reads,
            "finalized read layout differs after {mode:?}",
        );
    }
    let debug_linked = finalize(&debug);
    let release_linked = finalize(&release);
    let unchecked_linked = finalize(&unchecked);
    let read_target = |name: &str| {
        LinkedReadSpec::Binding(SymbolId {
            module: fixture.provider.clone(),
            name: name.into(),
        })
    };
    assert!(debug_linked.runtime_read(&fixture.consumer, &read_target("Ensured")).is_some());
    assert!(debug_linked.runtime_read(&fixture.consumer, &read_target("BodyUsed")).is_some());
    assert!(release_linked.runtime_read(&fixture.consumer, &read_target("Required")).is_some());
    assert!(release_linked.runtime_read(&fixture.consumer, &read_target("BodyUsed")).is_some());
    assert!(release_linked.runtime_read(&fixture.consumer, &read_target("Ensured")).is_none());
    assert!(release_linked.runtime_read(&fixture.consumer, &read_target("InvariantValue")).is_none());
    assert!(unchecked_linked.runtime_read(&fixture.consumer, &read_target("Required")).is_none());
    assert!(unchecked_linked.runtime_read(&fixture.consumer, &read_target("BodyUsed")).is_some());
    assert!(unchecked_linked.runtime_read(&fixture.consumer, &read_target("Ensured")).is_none());
    assert!(unchecked_linked.runtime_read(&fixture.consumer, &read_target("InvariantValue")).is_none());
    assert_eq!(
        debug.all_diagnostics().count(),
        release.all_diagnostics().count(),
        "mode changes requirement projection only; semantic diagnostics remain mode-independent"
    );
    assert_eq!(
        debug.all_diagnostics().count(),
        unchecked.all_diagnostics().count(),
        "Unchecked retains the same semantic diagnostic coverage"
    );
}

#[test]
fn cv_06_mixed_interface_and_runtime_use_joins_to_runtime() {
    let fixture = phase_fixture(
        "class Imported {}\nexport Imported\n",
        "from .provider import Imported\nclass Consumer { read(value: Imported) -> Imported { Imported } }\n",
        "class Unchanged {}\n",
    );
    let analysis = runtime_value_analysis(&fixture);
    let analyses = HashMap::from([(analysis.callable.clone(), Arc::new(analysis))]);
    let (products, _) = aggregate(&fixture, &analyses);

    let phase = imported_phase(&fixture, &products);
    assert_eq!(
        phase.phase,
        DependencyPhase::Runtime,
        "runtime value use must dominate interface-only annotations"
    );
    assert!(phase.observed, "both phase-qualified uses must be retained as observed evidence");
}

#[test]
fn superclass_use_emits_generated_runtime_requirement() {
    let fixture = phase_fixture(
        "class Imported {}\nexport Imported\n",
        "from .provider import Imported\nclass Child is Imported {}\n",
        "class Unchanged {}\n",
    );
    let (products, requirements) = aggregate(&fixture, &HashMap::new());

    assert_eq!(imported_phase(&fixture, &products).phase, DependencyPhase::Runtime);
    assert!(
        requirements.iter().any(|requirement| {
            requirement.importer == fixture.consumer
                && requirement.dependency == fixture.provider
                && requirement.reason == RuntimeDependencyReason::RuntimeDeclarationReference
                && requirement.read_target
                    == Some(LinkedReadSpec::Binding(SymbolId {
                        module: fixture.provider.clone(),
                        name: "Imported".into(),
                    }))
        }),
        "superclass materialization must retain its canonical read target and generated runtime edge"
    );
}

#[test]
fn type_form_nominal_and_applied_roots_emit_canonical_class_read() {
    let fixture = phase_fixture(
        "class Imported<T> {}\nexport Imported\n",
        "from .provider import Imported\nclass Consumer { read() { Imported } }\n",
        "class Unchanged {}\n",
    );
    let (mut analysis, mut store) = analyze_body(&fixture, "Imported");
    let declaration = DeclarationId::new(fixture.provider.clone(), "Imported".into());
    let parameterized_kind = store.arrow_kind(Box::new([phalcom_semantic::types::id::KindId::TYPE]), phalcom_semantic::types::id::KindId::TYPE);
    let origin = store.nominal_form(declaration, parameterized_kind);
    let unit = store.unit();
    let applied = store.apply_type_form(origin, &[unit]).expect("well-kinded applied class form");
    assert!(matches!(store.get(applied), phalcom_semantic::types::store::TypeData::Applied { .. }));

    let mut expressions = analysis.expressions.clone();
    let expression_id = expressions.keys().next().copied().expect("body has a type-form expression");
    let expression = expressions
        .get_source_mut(&expression_id)
        .expect("source analysis map")
        .expect("type-form expression");
    expression.denotation = Some(phalcom_semantic::types::denotation::SemanticDenotation::TypeForm(applied));
    analysis.expressions = expressions;
    let analyses = HashMap::from([(analysis.callable.clone(), Arc::new(analysis))]);
    let (products, requirements) =
        compute_module_import_phase_products(&fixture.semantic, &fixture.sources, &fixture.source_index, &analyses, Some(&store), None, true);

    assert!(requirements.iter().any(|requirement| {
        requirement.importer == fixture.consumer
            && requirement.dependency == fixture.provider
            && requirement.read_target
                == Some(LinkedReadSpec::Binding(SymbolId {
                    module: fixture.provider.clone(),
                    name: "Imported".into(),
                }))
    }));
    assert_eq!(imported_phase(&fixture, &products).phase, DependencyPhase::Runtime);
}

#[test]
fn structural_pattern_requirements_include_error_class_read() {
    let fixture = phase_fixture(
        "class Imported {}\nexport Imported\n",
        "from .provider import Imported\nclass Consumer {}\n",
        "class Unchanged {}\n",
    );
    let (analysis, _) = analyze_body(&fixture, "if let (left, right) = (1, 2) { left }");
    assert!(!analysis.pattern_site_resolutions.is_empty(), "tuple destructuring must publish a pattern site");
    let analyses = HashMap::from([(analysis.callable.clone(), Arc::new(analysis))]);
    let (_, requirements) = aggregate(&fixture, &analyses);
    let prelude = phalcom_semantic::prelude::PreludeTypeMap::shared_canonical_universe();
    let error = prelude.get("Error").expect("canonical Error prelude declaration");

    assert!(requirements.iter().any(|requirement| {
        requirement.importer == fixture.consumer
            && requirement.dependency == error.module
            && requirement.read_target
                == Some(LinkedReadSpec::Binding(SymbolId {
                    module: error.module.clone(),
                    name: error.name.clone(),
                }))
    }));
}

#[test]
fn for_loop_lowering_projects_error_read_even_for_binding_pattern() {
    let fixture = phase_fixture(
        "class Imported {}\nexport Imported\n",
        "from .provider import Imported\nclass Consumer { read() { for value in [1] {} } }\n",
        "class Unchanged {}\n",
    );
    let (products, requirements) = aggregate(&fixture, &HashMap::new());
    let prelude = phalcom_semantic::prelude::PreludeTypeMap::shared_canonical_universe();
    let error = prelude.get("Error").expect("canonical Error prelude declaration");
    let expected = LinkedReadSpec::Binding(SymbolId {
        module: error.module.clone(),
        name: error.name.clone(),
    });

    assert!(
        requirements
            .iter()
            .any(|requirement| { requirement.importer == fixture.consumer && requirement.read_target == Some(expected.clone()) }),
        "for-pattern lowering emits Error.new even for a binding pattern"
    );
    assert!(
        !requirements.iter().any(|requirement| {
            matches!(
                requirement.read_target.as_ref(),
                Some(LinkedReadSpec::Binding(symbol)) if symbol.name.as_ref() == "ZipLengthMismatchError"
            )
        }),
        "single-lane loops do not emit strict-zip reads"
    );
    assert_eq!(products[&fixture.consumer].coverage, phalcom_modules::linker::PhaseEvidenceCoverage::Incomplete);
}

#[test]
fn strict_multi_lane_for_projects_canonical_zip_error_read() {
    let fixture = phase_fixture(
        "class Imported {}\nexport Imported\n",
        "from .provider import Imported\nclass Consumer { read() { for left in [1], right in [2] {} } }\n",
        "class Unchanged {}\n",
    );
    let (_, requirements) = aggregate(&fixture, &HashMap::new());
    let zip_error_module = ModuleId::universe(ModulePath::from_components(vec![
        ModuleComponent::from_identifier("errors").expect("canonical Universe component"),
        ModuleComponent::from_identifier("iteration").expect("canonical Universe component"),
    ]));
    let expected = LinkedReadSpec::Binding(SymbolId {
        module: zip_error_module.clone(),
        name: "ZipLengthMismatchError".into(),
    });

    assert!(
        requirements.iter().any(|requirement| {
            requirement.importer == fixture.consumer
                && requirement.dependency == zip_error_module
                && requirement.reason == RuntimeDependencyReason::RuntimeDeclarationReference
                && requirement.read_target == Some(expected.clone())
        }),
        "strict zip lowering emits a canonical ZipLengthMismatchError read"
    );
}

#[test]
fn loop_control_error_reads_match_materialized_trap_paths() {
    fn error_read_count(consumer_source: &str) -> usize {
        let fixture = phase_fixture("class Provider {}\nexport Provider\n", consumer_source, "class Unchanged {}\n");
        let (_, requirements) = aggregate(&fixture, &HashMap::new());
        let prelude = phalcom_semantic::prelude::PreludeTypeMap::shared_canonical_universe();
        let error = prelude.get("Error").expect("canonical Error prelude declaration");
        requirements
            .iter()
            .filter(|requirement| {
                requirement.importer == fixture.consumer
                    && requirement.read_target
                        == Some(LinkedReadSpec::Binding(SymbolId {
                            module: error.module.clone(),
                            name: error.name.clone(),
                        }))
            })
            .count()
    }

    let plain_while = error_read_count("while value { nil }\n");
    let while_break = error_read_count("while value { break }\n");
    assert_eq!(plain_while, 0, "a while without loop control does not need the trap's Error object");
    assert_eq!(while_break, 1, "the guarded while fallback materializes a trap for break");

    let nested_for_without_control = error_read_count("for a in [1] {\n  for b in [2] {\n    throw nil\n  }\n}\n");
    let nested_for_with_control = error_read_count("for a in [1] {\n  for b in [2] {\n    break\n  }\n}\n");
    assert_eq!(nested_for_without_control, 2, "each for loop has its own canonical Error read");
    assert_eq!(
        nested_for_with_control, nested_for_without_control,
        "direct nested-loop break adds no trap read"
    );

    let block_control = error_read_count("class Truthy { ifTrue(_ block) { return block.call() } }\nfor a in [1] {\n  Truthy.new().ifTrue || { break }\n}\n");
    assert_eq!(block_control, 2, "a break captured by a materialized block adds the trap's Error read");
}

#[test]
fn attribute_argument_roots_emit_exact_prelude_and_import_reads() {
    let fixture = phase_fixture(
        "class Imported {}\nexport Imported\n",
        "from .provider import Imported\n@custom(Imported, match (1, 2) { (left, right) => 1 })\nclass Consumer { @requires(Error) read() {} }\n",
        "class Unchanged {}\n",
    );
    let (products, requirements) = aggregate(&fixture, &HashMap::new());
    let prelude = phalcom_semantic::prelude::PreludeTypeMap::shared_canonical_universe();
    let error = prelude.get("Error").expect("canonical Error prelude declaration");
    let imported = LinkedReadSpec::Binding(SymbolId {
        module: fixture.provider.clone(),
        name: "Imported".into(),
    });

    assert!(requirements.iter().any(|requirement| requirement.read_target == Some(imported.clone())));
    assert!(requirements.iter().any(|requirement| {
        requirement.dependency == error.module
            && requirement.read_target
                == Some(LinkedReadSpec::Binding(SymbolId {
                    module: error.module.clone(),
                    name: error.name.clone(),
                }))
    }));
    let tuple = prelude.get("Tuple").expect("canonical Tuple prelude declaration");
    assert!(
        requirements.iter().any(|requirement| {
            requirement.dependency == tuple.module
                && requirement.read_target
                    == Some(LinkedReadSpec::Binding(SymbolId {
                        module: tuple.module.clone(),
                        name: tuple.name.clone(),
                    }))
        }),
        "AST match patterns must project their implicit Tuple class read"
    );
    assert_eq!(imported_phase(&fixture, &products).phase, DependencyPhase::Runtime);
    assert_eq!(products[&fixture.consumer].coverage, phalcom_modules::linker::PhaseEvidenceCoverage::Incomplete);
}

#[test]
fn tuple_binding_pattern_emits_tuple_and_error_runtime_reads() {
    let fixture = phase_fixture("class Imported {}\nexport Imported\n", "let (left, right) = (1, 2)\n", "class Unchanged {}\n");
    let (_, requirements) = aggregate(&fixture, &HashMap::new());
    let prelude = phalcom_semantic::prelude::PreludeTypeMap::shared_canonical_universe();

    for name in ["Tuple", "Error"] {
        let declaration = prelude.get(name).expect("canonical prelude declaration");
        let target = LinkedReadSpec::Binding(SymbolId {
            module: declaration.module.clone(),
            name: declaration.name.clone(),
        });
        assert!(
            requirements
                .iter()
                .any(|requirement| requirement.importer == fixture.consumer && requirement.read_target == Some(target.clone())),
            "tuple binding lowering requires a canonical runtime read for {name}: {requirements:#?}"
        );
    }
}

#[test]
fn tuple_binding_pattern_in_static_field_default_emits_tuple_and_error_runtime_reads() {
    let fixture = phase_fixture(
        "class Imported {}\nexport Imported\n",
        "class Consumer {\n  @class mut _value = if true {\n    let (left, right) = (1, 2)\n    left\n  }\n}\n",
        "class Unchanged {}\n",
    );
    let (_, requirements) = aggregate(&fixture, &HashMap::new());
    let prelude = phalcom_semantic::prelude::PreludeTypeMap::shared_canonical_universe();

    for name in ["Tuple", "Error"] {
        let declaration = prelude.get(name).expect("canonical prelude declaration");
        let target = LinkedReadSpec::Binding(SymbolId {
            module: declaration.module.clone(),
            name: declaration.name.clone(),
        });
        assert!(
            requirements
                .iter()
                .any(|requirement| requirement.importer == fixture.consumer && requirement.read_target == Some(target.clone())),
            "tuple field-default lowering requires a canonical runtime read for {name}: {requirements:#?}"
        );
    }
}

#[test]
fn woven_precondition_emits_canonical_contract_error_runtime_read() {
    let fixture = phase_fixture(
        "class Imported {}\nexport Imported\n",
        "class Guarded {\n  @requires(value > 0)\n  check(_ value) { value }\n}\n",
        "class Unchanged {}\n",
    );
    let (_, requirements) = aggregate(&fixture, &HashMap::new());
    let contracts_module = ModuleId::universe(ModulePath::from_components(vec![
        ModuleComponent::from_identifier("errors").expect("valid module component"),
        ModuleComponent::from_identifier("contracts").expect("valid module component"),
    ]));
    let target = LinkedReadSpec::Binding(SymbolId {
        module: contracts_module.clone(),
        name: "PreconditionError".into(),
    });

    assert!(
        requirements.iter().any(|requirement| {
            requirement.importer == fixture.consumer && requirement.dependency == contracts_module && requirement.read_target.as_ref() == Some(&target)
        }),
        "a woven precondition guard requires a canonical PreconditionError read: {requirements:#?}"
    );
}

#[test]
fn attribute_argument_selected_callable_emits_external_owner_requirement() {
    let fixture = phase_fixture(
        "class Factory { @class produce() {} }\nexport Factory\n",
        "from .provider import Factory\nclass custom {}\n@custom(Factory.produce())\nclass Consumer {}\n",
        "class Unchanged {}\n",
    );
    let mut session = session_with_private_numbers_child_exposed(phalcom_semantic::identity::WorkspaceId::from_raw(918));
    let snapshot = session
        .update_with_budget_and_cancel(
            SemanticWorkspaceInput::new(fixture.semantic.to_unfinalized_program(), fixture.sources.clone(), 1).with_semantic_linked(fixture.semantic.clone()),
            QueryBudget::default(),
            &CancellationToken::new(),
        )
        .expect("attribute argument callable analysis succeeds")
        .snapshot;

    assert_eq!(
        snapshot.import_phase_products[&fixture.consumer].coverage,
        phalcom_modules::linker::PhaseEvidenceCoverage::Complete,
        "a complete user-attribute argument analysis closes import coverage"
    );
    let factory_import = fixture.semantic.modules[&fixture.consumer].imports.by_local_name["Factory"].clone();
    assert_eq!(
        snapshot.import_phase_products[&fixture.consumer].bindings[&factory_import].phase,
        DependencyPhase::Runtime,
        "the attribute argument's receiver is an executable import"
    );
    assert!(
        snapshot.generated_runtime_requirements.iter().any(|requirement| {
            requirement.importer == fixture.consumer
                && requirement.dependency == fixture.provider
                && requirement.reason == RuntimeDependencyReason::RuntimeDeclarationReference
                && requirement.read_target.is_none()
                && requirement.provenance.producer == phalcom_modules::RuntimeRequirementProducer::CallableOwner
                && requirement.provenance.target == phalcom_modules::RuntimeRequirementTarget::Module(fixture.provider.clone())
        }),
        "the selected external class callable's canonical owner is a runtime requirement: {:#?}",
        snapshot.generated_runtime_requirements
    );
}

#[test]
fn attribute_argument_dynamic_trait_send_falls_back_to_runtime_imports() {
    let consumer_source = "from .provider import Marked\nclass custom {}\nclass Local { @class make() -> Local { Local.new() } }\nimpl Marked for Local {}\n@custom(Local.make().marker())\nclass Consumer {}\n";
    let fixture = phase_fixture("trait Marked { marker -> Int { 7 } }\nexport Marked\n", consumer_source, "class Unchanged {}\n");
    let mut session = session_with_private_numbers_child_exposed(phalcom_semantic::identity::WorkspaceId::from_raw(919));
    let snapshot = session
        .update_with_budget_and_cancel(
            SemanticWorkspaceInput::new(fixture.semantic.to_unfinalized_program(), fixture.sources.clone(), 1).with_semantic_linked(fixture.semantic.clone()),
            QueryBudget::default(),
            &CancellationToken::new(),
        )
        .expect("attribute argument analysis succeeds with an unresolved dynamic send")
        .snapshot;

    let marked_import = fixture.semantic.modules[&fixture.consumer].imports.by_local_name["Marked"].clone();
    assert_eq!(
        snapshot.import_phase_products[&fixture.consumer].coverage,
        phalcom_modules::linker::PhaseEvidenceCoverage::Incomplete,
        "a Ready analysis with an unresolved dynamic send cannot close runtime dependency coverage"
    );
    assert_eq!(
        snapshot.import_phase_products[&fixture.consumer].bindings[&marked_import],
        phalcom_modules::linker::ImportPhaseSummary {
            phase: DependencyPhase::Runtime,
            observed: true,
        },
        "an unresolved dynamic trait send must retain the imported trait module at runtime"
    );
    let marked_target = LinkedReadSpec::Binding(SymbolId {
        module: fixture.provider.clone(),
        name: "Marked".into(),
    });
    assert!(
        snapshot.generated_runtime_requirements.iter().any(|requirement| {
            requirement.importer == fixture.consumer
                && requirement.dependency == fixture.provider
                && requirement.reason == RuntimeDependencyReason::RuntimeDeclarationReference
                && requirement.read_target == Some(marked_target.clone())
                && requirement.provenance.producer == phalcom_modules::RuntimeRequirementProducer::ImportRead
        }),
        "an unresolved trait send falls back to the exact imported interface binding read: {:#?}",
        snapshot.generated_runtime_requirements
    );
}

#[test]
fn conformance_trait_root_is_interface_only() {
    let fixture = phase_fixture(
        "trait Imported {}\nexport Imported\n",
        "from .provider import Imported\nclass Local {}\nimpl Imported for Local {}\n",
        "class Unchanged {}\n",
    );
    let (products, requirements) = aggregate(&fixture, &HashMap::new());

    let phase = imported_phase(&fixture, &products);
    assert_eq!(phase.phase, DependencyPhase::InterfaceOnly);
    assert_eq!(products[&fixture.consumer].coverage, phalcom_modules::linker::PhaseEvidenceCoverage::Complete);
    assert!(phase.observed, "conformance trait reference must be retained as observed evidence");
    assert!(
        requirements
            .iter()
            .all(|requirement| requirement.importer != fixture.consumer || requirement.dependency != fixture.provider)
    );
}

#[test]
fn class_signature_interface_import_keeps_complete_coverage() {
    let fixture = phase_fixture(
        "class Imported {}\nexport Imported\n",
        "from .provider import Imported\nclass Consumer { read(value: Imported) -> Imported { value } }\n",
        "class Unchanged {}\n",
    );
    let (products, _) = aggregate(&fixture, &HashMap::new());

    assert_eq!(imported_phase(&fixture, &products).phase, DependencyPhase::InterfaceOnly);
    assert_eq!(products[&fixture.consumer].coverage, phalcom_modules::linker::PhaseEvidenceCoverage::Complete);
}

#[test]
fn cv_07_incremental_phase_change_matches_cold_analysis() {
    let first_fixture = phase_fixture(
        "class Imported {}\nexport Imported\n",
        "from .provider import Imported\nclass Consumer { read(value: Imported) -> Imported { value } }\n",
        "class Unchanged {}\n",
    );
    let second_fixture = phase_fixture(
        "class Imported {}\nexport Imported\n",
        "from .provider import Imported\nclass Consumer { read(value: Imported) -> Imported { Imported } }\n",
        "class Unchanged {}\n",
    );
    assert_eq!(first_fixture.consumer, second_fixture.consumer);
    assert_eq!(first_fixture.provider, second_fixture.provider);
    assert_eq!(first_fixture.unrelated, second_fixture.unrelated);

    // The repository's canonical embedded Universe currently has an unrelated
    // private-child import inconsistency. The test provider exposes that child
    // only for this session fixture, leaving production sources untouched.
    let mut session = session_with_private_numbers_child_exposed(phalcom_semantic::identity::WorkspaceId::from_raw(908));
    let first = session
        .update_with_budget_and_cancel(
            SemanticWorkspaceInput::new(first_fixture.semantic.to_unfinalized_program(), first_fixture.sources.clone(), 1)
                .with_semantic_linked(first_fixture.semantic.clone()),
            QueryBudget::default(),
            &CancellationToken::new(),
        )
        .expect("first semantic update succeeds")
        .snapshot;
    let import_id = first_fixture.semantic.modules[&first_fixture.consumer].imports.by_local_name["Imported"].clone();
    let first_phase = &first.import_phase_products[&first_fixture.consumer].bindings[&import_id];
    assert_eq!(first_phase.phase, DependencyPhase::InterfaceOnly);
    assert!(first_phase.observed);

    let second = session
        .update_with_budget_and_cancel(
            SemanticWorkspaceInput::new(second_fixture.semantic.to_unfinalized_program(), second_fixture.sources.clone(), 2)
                .with_semantic_linked(second_fixture.semantic.clone()),
            QueryBudget::default(),
            &CancellationToken::new(),
        )
        .expect("incremental semantic update succeeds")
        .snapshot;
    let second_phase = &second.import_phase_products[&second_fixture.consumer].bindings[&import_id];
    assert_eq!(second_phase.phase, DependencyPhase::Runtime);
    assert!(second_phase.observed, "new value-use evidence must be published after the edit");

    let mut cold_session = session_with_private_numbers_child_exposed(phalcom_semantic::identity::WorkspaceId::from_raw(909));
    let cold = cold_session
        .update_with_budget_and_cancel(
            SemanticWorkspaceInput::new(second_fixture.semantic.to_unfinalized_program(), second_fixture.sources.clone(), 1)
                .with_semantic_linked(second_fixture.semantic.clone()),
            QueryBudget::default(),
            &CancellationToken::new(),
        )
        .expect("cold semantic update of edited workspace succeeds")
        .snapshot;
    assert_eq!(
        second.import_phase_products[&second_fixture.consumer], cold.import_phase_products[&second_fixture.consumer],
        "incremental edited consumer phase product must match cold analysis"
    );
    let diagnostic_signature = |snapshot: &phalcom_semantic::SemanticSnapshot| {
        let mut diagnostics = snapshot
            .all_diagnostics()
            .map(|diagnostic| {
                (
                    format!("{:?}", diagnostic.code),
                    diagnostic.message.clone(),
                    diagnostic.primary.module.clone(),
                    diagnostic.primary_range.start,
                    diagnostic.primary_range.end,
                )
            })
            .collect::<Vec<_>>();
        diagnostics.sort();
        diagnostics
    };
    assert_eq!(
        diagnostic_signature(&second),
        diagnostic_signature(&cold),
        "incremental diagnostics must match cold analysis"
    );
}

#[test]
fn missing_checked_static_initializer_keeps_import_projection_conservative() {
    use phalcom_modules::linker::PhaseEvidenceCoverage;
    use phalcom_semantic::static_constant::{StaticConstantId, StaticConstantProduct, StaticConstantTable};
    use phalcom_semantic::static_eval::StaticEvalOutcome;
    use phalcom_semantic::static_value::StaticValueStore;

    let fixture = phase_fixture(
        "class Imported {}\nexport Imported\n",
        "from .provider import Imported\nclass Consumer { _value: Imported }\n",
        "class Unchanged {}\n",
    );
    let (baseline, _) = aggregate(&fixture, &HashMap::new());
    assert_eq!(baseline[&fixture.consumer].coverage, PhaseEvidenceCoverage::Complete);
    let mut values = StaticValueStore::new();
    let value = values.intern_unit();
    let mut constants = StaticConstantTable::new();
    constants.insert(Arc::new(StaticConstantProduct {
        id: StaticConstantId::new(fixture.consumer.clone(), "Unchecked"),
        declaration_range: Default::default(),
        initializer_range: Default::default(),
        checked: None,
        outcome: StaticEvalOutcome::Reduced(value),
        residuals: None,
        value: Some(value),
        lkg_value: Some(value),
        diagnostics: Arc::new([]),
        imported: None,
        authority: None,
    }));
    let (products, requirements) = compute_module_import_phase_products(
        &fixture.semantic,
        &fixture.sources,
        &fixture.source_index,
        &HashMap::new(),
        None,
        Some(&constants),
        true,
    );
    assert_eq!(products[&fixture.consumer].coverage, PhaseEvidenceCoverage::Incomplete);
    let linker = ModuleLinker::new(
        Arc::new(ProjectUniverse::new()),
        fixture
            .sources
            .iter()
            .map(|(id, source)| {
                (
                    id.clone(),
                    InterfaceBuilder::build(id.clone(), ModuleKind::Module, &source.program).expect("fixture interface"),
                )
            })
            .collect(),
    );
    let executable = linker
        .finalize_executable_with_requirements((*fixture.semantic).clone(), &products, &requirements)
        .expect("conservative runtime projection remains valid");
    assert!(executable.modules[&fixture.consumer].bindings.imports.contains_key("Imported"));
}
