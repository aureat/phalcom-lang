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
    PhaseFixture {
        semantic,
        sources,
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
    compute_module_import_phase_products(&fixture.semantic, &fixture.sources, analyses, None, None, true)
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
fn cv_04_imported_value_use_is_classified_runtime() {
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
fn cv_05_superclass_use_emits_generated_runtime_requirement() {
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
    let expression = expressions.values_mut().next().expect("body has a type-form expression");
    expression.denotation = Some(phalcom_semantic::types::denotation::SemanticDenotation::TypeForm(applied));
    analysis.expressions = expressions;
    let analyses = HashMap::from([(analysis.callable.clone(), Arc::new(analysis))]);
    let (products, requirements) = compute_module_import_phase_products(&fixture.semantic, &fixture.sources, &analyses, Some(&store), None, true);

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
