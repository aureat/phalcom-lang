use phalcom_modules::ModuleId;
use phalcom_modules::identity::{ModuleComponent, ModulePath, ResolvedProjectId};
use phalcom_modules::interface::LinkedModuleInterface;
use phalcom_modules::linker::{LinkedModule, LinkedProgram, ModuleBindingLayout};
use phalcom_modules::metadata::ModuleMetadata;
use phalcom_modules::source::ModuleKind;
use phalcom_semantic::identity::{CallableId, DeclarationId, DispatchSide};
use phalcom_semantic::session::SemanticWorkspaceSession;
use phalcom_semantic::source::ParsedModuleUnit;
use phalcom_semantic::workspace::SemanticWorkspaceInput;
use std::collections::BTreeMap;
use std::sync::Arc;

fn module_id() -> ModuleId {
    ModuleId::resolved(
        ResolvedProjectId::from_raw(31),
        ModulePath::from_components(vec![ModuleComponent::from_identifier("main").unwrap()]),
    )
}

fn input(module: ModuleId, source: &str, generation: u64) -> SemanticWorkspaceInput {
    let parsed = phalcom_ast::parse(source, 0);
    assert!(parsed.errors.is_empty(), "parse errors: {:?}", parsed.errors);
    let program = Arc::new(parsed.program);
    let linked_module = LinkedModule {
        interface: LinkedModuleInterface {
            module: module.clone(),
            kind: ModuleKind::Module,
            exports: BTreeMap::new(),
            metadata: ModuleMetadata::default(),
        },
        bindings: ModuleBindingLayout::default(),
        linked_reads: Vec::new(),
        runtime_dependencies: Vec::new(),
    };
    let mut modules = BTreeMap::new();
    modules.insert(module.clone(), linked_module);
    let linked = Arc::new(LinkedProgram {
        universe: Arc::new(phalcom_modules::project::ProjectUniverse::new()),
        modules,
        graphs: phalcom_modules::graph::ModuleGraphs::default(),
        entry: module.clone(),
        initialization_order: vec![module.clone()],
    });
    let mut sources = BTreeMap::new();
    sources.insert(
        module.clone(),
        Arc::new(ParsedModuleUnit::new(module, ModuleKind::Module, None, Arc::from(source), program)),
    );
    SemanticWorkspaceInput::new(linked, sources, generation)
}

fn callable(module: &ModuleId, selector: &str, slots: usize) -> CallableId {
    let selector = match slots {
        0 => phalcom_common::selector::Selector::method(selector, []).unwrap(),
        1 => phalcom_common::selector::Selector::method(selector, [phalcom_common::selector::SelectorSlot::Positional]).unwrap(),
        _ => panic!("unsupported test selector arity"),
    };
    CallableId::new(DeclarationId::new(module.clone(), "Display".into()), selector, DispatchSide::Instance)
}

#[test]
fn trait_body_edit_reuses_surface_and_unrelated_default_but_refreshes_changed_body() {
    let module = module_id();
    let source_a = r#"
trait Display {
  render(_ value: Int) -> Int
  fallback(_ value: Int) -> Int { self.render(value) }
  unrelated() -> Int { 1 }
}
"#;
    let source_b = source_a.replace("self.render(value)", "self.render(1)");
    let mut session = SemanticWorkspaceSession::new();
    let first = session
        .update_with_budget_and_cancel(
            input(module.clone(), source_a, 1),
            phalcom_semantic::db::QueryBudget::default(),
            &phalcom_semantic::db::CancellationToken::new(),
        )
        .unwrap_or_else(|error| panic!("initial update failed: {error:?}"));
    assert!(!first.snapshot.has_errors(), "initial diagnostics: {:?}", first.snapshot.diagnostics);
    let declaration = DeclarationId::new(module.clone(), "Display".into());
    let first_surface = first.snapshot.trait_surfaces.get(&declaration).expect("initial trait surface").clone();
    let fallback = callable(&module, "fallback", 1);
    let unrelated = callable(&module, "unrelated", 0);
    let first_fallback = first.snapshot.callable_analyses.get(&fallback).expect("initial fallback analysis").clone();
    let first_unrelated = first.snapshot.callable_analyses.get(&unrelated).expect("initial unrelated analysis").clone();
    let header_key = phalcom_semantic::db::QueryKey::TraitHeader(declaration.clone());
    let surface_key = phalcom_semantic::db::QueryKey::TraitSurface(declaration.clone());
    let body_key = phalcom_semantic::db::QueryKey::CallableBody(fallback.clone());
    assert!(
        session
            .db()
            .index()
            .dependencies_of(&surface_key)
            .is_some_and(|edges| edges.iter().any(|edge| edge.dependency == header_key)),
        "trait surface must depend on its DB-owned header"
    );
    assert!(
        session
            .db()
            .index()
            .dependencies_of(&body_key)
            .is_some_and(|edges| edges.iter().any(|edge| edge.dependency == surface_key)),
        "trait default body must depend on its DB-owned surface"
    );

    let second = session
        .update(input(module.clone(), &source_b, 2))
        .expect("semantic test update should succeed");
    assert!(!second.snapshot.has_errors(), "updated diagnostics: {:?}", second.snapshot.diagnostics);
    let second_surface = second.snapshot.trait_surfaces.get(&declaration).expect("updated trait surface").clone();
    assert_eq!(
        phalcom_semantic::db::fingerprint::trait_surface_product_fingerprint(&first_surface),
        phalcom_semantic::db::fingerprint::trait_surface_product_fingerprint(&second_surface),
        "body-only edit must preserve the complete trait contract fingerprint",
    );
    let second_fallback = second.snapshot.callable_analyses.get(&fallback).expect("updated fallback analysis").clone();
    let second_unrelated = second.snapshot.callable_analyses.get(&unrelated).expect("updated unrelated analysis").clone();
    assert!(
        !Arc::ptr_eq(&first_fallback, &second_fallback),
        "changed default body must refresh its analysis"
    );
    assert!(Arc::ptr_eq(&first_unrelated, &second_unrelated), "unrelated default must remain reusable");
}

#[test]
fn class_side_requirement_edit_invalidates_fingerprint() {
    use phalcom_semantic::db::QueryKey;
    use phalcom_semantic::impls::ConformanceCompleteness;

    let module = module_id();
    let source_instance = r#"
trait Contract { foo() -> Int }
data Subject(_ value: Int)
impl Contract for Subject { foo() -> Int { self.value } }
"#;
    let source_class = source_instance.replace("trait Contract { foo()", "trait Contract { @class foo()");
    let declaration = DeclarationId::new(module.clone(), "Contract".into());
    let mut session = SemanticWorkspaceSession::new();
    let first = session
        .update(input(module.clone(), source_instance, 1))
        .expect("semantic test update should succeed");
    assert!(!first.snapshot.has_errors(), "initial diagnostics: {:?}", first.snapshot.diagnostics);
    let first_surface = first.snapshot.trait_surfaces.get(&declaration).expect("initial trait surface");
    assert!(first_surface.iter().any(|(requirement, _)| requirement.side == DispatchSide::Instance));
    let first_surface_fingerprint = phalcom_semantic::db::fingerprint::trait_surface_product_fingerprint(first_surface);
    let (impl_id, first_plan) = first.snapshot.conformance_witness_plans.iter().next().expect("initial conformance plan");
    assert!(matches!(first_plan.completeness, ConformanceCompleteness::Complete));
    let dispatch_key = QueryKey::ConformanceDispatch(impl_id.clone());
    let surface_key = QueryKey::TraitSurface(declaration.clone());
    assert!(session.db().index().dependencies_of(&dispatch_key).is_some_and(|edges| {
        edges
            .iter()
            .any(|edge| edge.dependency == surface_key && edge.observed_fingerprint == first_surface_fingerprint)
    }));

    let second = session
        .update_with_budget_and_cancel(
            input(module.clone(), &source_class, 2),
            phalcom_semantic::db::QueryBudget::default(),
            &phalcom_semantic::db::CancellationToken::new(),
        )
        .unwrap_or_else(|error| panic!("updated analysis failed: {error:?}"));
    let shard = session.semantic_structure_shards().get(&module).expect("updated semantic structure shard");
    assert_eq!(shard.source.text.as_ref(), source_class);
    let phalcom_ast::ast::Statement::Trait(trait_def) = &shard.source.program.statements[0] else {
        panic!("updated source must retain its trait declaration")
    };
    assert!(
        trait_def.members[0].behavior().is_some_and(|member| matches!(
            member,
            phalcom_ast::ast::BehaviorMember::Method(method)
                if method.attributes.iter().any(|attribute| attribute.name == "class")
        )),
        "parsed class-side requirement must carry Class side"
    );
    let second_surface = second.snapshot.trait_surfaces.get(&declaration).expect("updated trait surface");
    assert!(
        second_surface.iter().any(|(requirement, _)| requirement.side == DispatchSide::Class),
        "updated source: {source_class}; surface requirements: {:?}",
        second_surface.members.keys().collect::<Vec<_>>()
    );
    let second_surface_fingerprint = phalcom_semantic::db::fingerprint::trait_surface_product_fingerprint(second_surface);
    assert_ne!(
        first_surface_fingerprint, second_surface_fingerprint,
        "changing requirement side changes the trait contract fingerprint"
    );
    let second_plan = second.snapshot.conformance_witness_plans.get(impl_id).expect("updated conformance plan");
    assert!(!Arc::ptr_eq(first_plan, second_plan), "dependent conformance analysis must be recomputed");
    assert_ne!(
        first_plan.fingerprint, second_plan.fingerprint,
        "conformance plan records the changed requirement side"
    );
    assert!(matches!(second_plan.completeness, ConformanceCompleteness::Incomplete { .. }));
    assert!(
        second
            .snapshot
            .all_diagnostics()
            .any(|diagnostic| diagnostic.code == phalcom_semantic::diagnostic::DiagnosticCode::ImplConformanceIncomplete),
        "the instance-only witness must no longer satisfy the class-side requirement"
    );
    let stale_witness = first
        .snapshot
        .callable_definitions
        .values()
        .find(|definition| {
            matches!(
                &definition.origin,
                phalcom_semantic::impls::CallableDefinitionOrigin::ConformanceWitness(owner) if owner == impl_id
            )
        })
        .expect("initial accepted instance witness")
        .callable
        .clone();
    assert!(
        !second.snapshot.callable_definitions.contains_key(&stale_witness),
        "rejected witness definition must be retired"
    );
    assert!(
        second.snapshot.callable_signatures.get(&stale_witness).is_none(),
        "rejected witness signature must be retired"
    );
    assert!(
        !second.snapshot.callable_analyses.contains_key(&stale_witness),
        "rejected witness analysis must be retired"
    );
    assert!(session.db().product(&QueryKey::CallableDefinition(stale_witness.clone())).is_none());
    assert!(session.db().product(&QueryKey::CallableSignature(stale_witness.clone())).is_none());
    assert!(session.db().product(&QueryKey::CallableBody(stale_witness.clone())).is_none());

    let mut cold = SemanticWorkspaceSession::new();
    let cold_result = cold.update(input(module, &source_class, 1)).expect("semantic test update should succeed");
    let cold_plan = cold_result.snapshot.conformance_witness_plans.get(impl_id).expect("cold conformance plan");
    assert_eq!(
        second_plan.fingerprint, cold_plan.fingerprint,
        "incremental conformance plan must match cold analysis"
    );
    assert_eq!(
        second.snapshot.diagnostics, cold_result.snapshot.diagnostics,
        "incremental diagnostics must match cold analysis"
    );
}

#[test]
fn class_side_requirement_edit_rebuilds_reused_conformance_module() {
    use phalcom_common::selector::{Selector, SelectorSlot};
    use phalcom_modules::{SourceId, SourceLocation, SourceRevision, WorkspaceSourceBatchMutation};

    let root = tempfile::tempdir().expect("temporary workspace");
    std::fs::write(root.path().join("package.ph"), "").expect("package marker");
    let provider_path = root.path().join("provider.ph");
    let impl_path = root.path().join("impl.ph");
    let location = |path: &std::path::Path| SourceLocation {
        source_id: SourceId(path.to_string_lossy().into()),
        display_path: path.to_path_buf(),
    };
    let provider = location(&provider_path);
    let implementation = location(&impl_path);
    let provider_source = "trait Contract { foo() -> Int }\nexport Contract\n";
    let provider_class = provider_source.replace("foo()", "@class foo()");
    let impl_source = "from .provider import Contract\ndata Subject(_ value: Int)\nimpl Contract for Subject { foo() -> Int { self.value } }\nclass Caller { run(_ subject: Subject) -> Int { subject.foo() } }\nexport Subject\n";
    let mut session = SemanticWorkspaceSession::new();
    let first = session
        .apply_module_mutations([
            WorkspaceSourceBatchMutation::SetOverlay {
                source: provider.clone(),
                text: Arc::from(provider_source),
                revision: SourceRevision(1),
                recovered_program: None,
            },
            WorkspaceSourceBatchMutation::SetOverlay {
                source: implementation.clone(),
                text: Arc::from(impl_source),
                revision: SourceRevision(1),
                recovered_program: None,
            },
        ])
        .expect("initial two-module publication");
    assert!(!first.snapshot.has_errors(), "initial diagnostics: {:?}", first.snapshot.diagnostics);
    let impl_module = first.snapshot.module_for_display_path(&impl_path).expect("implementation module").clone();
    let impl_id = first
        .snapshot
        .conformance_witness_plans
        .keys()
        .find(|impl_id| impl_id.module == impl_module)
        .expect("implementation conformance")
        .clone();
    let stale_witness = first
        .snapshot
        .callable_definitions
        .values()
        .find(|definition| {
            matches!(
                &definition.origin,
                phalcom_semantic::impls::CallableDefinitionOrigin::ConformanceWitness(owner) if owner == &impl_id
            )
        })
        .expect("initial accepted instance witness")
        .callable
        .clone();
    let caller = CallableId::new(
        DeclarationId::new(impl_module.clone(), "Caller".into()),
        Selector::method("run", [SelectorSlot::Positional]).unwrap(),
        DispatchSide::Instance,
    );
    let first_caller_analysis = first
        .snapshot
        .callable_analyses
        .get(&caller)
        .expect("initial dependent caller analysis")
        .clone();
    let first_impl_shard = session
        .semantic_structure_shards()
        .get(&impl_module)
        .expect("initial implementation shard")
        .clone();

    let second = session
        .apply_module_mutations([WorkspaceSourceBatchMutation::SetOverlay {
            source: provider.clone(),
            text: Arc::from(provider_class.as_str()),
            revision: SourceRevision(2),
            recovered_program: None,
        }])
        .expect("trait-side update");
    assert!(
        Arc::ptr_eq(
            &first_impl_shard,
            session.semantic_structure_shards().get(&impl_module).expect("retained implementation shard")
        ),
        "the conformance source shard should remain structurally reusable"
    );
    let second_plan = second.snapshot.conformance_witness_plans.get(&impl_id).expect("updated conformance plan");
    assert!(matches!(
        second_plan.completeness,
        phalcom_semantic::impls::ConformanceCompleteness::Incomplete { .. }
    ));
    assert!(!second.snapshot.callable_definitions.contains_key(&stale_witness));
    assert!(second.snapshot.callable_signatures.get(&stale_witness).is_none());
    assert!(!second.snapshot.callable_analyses.contains_key(&stale_witness));
    let second_caller_analysis = second.snapshot.callable_analyses.get(&caller).expect("refreshed dependent caller analysis");
    assert!(
        !Arc::ptr_eq(&first_caller_analysis, second_caller_analysis),
        "witness retirement must refresh dependent clients"
    );

    let mut cold = SemanticWorkspaceSession::new();
    let cold_result = cold
        .apply_module_mutations([
            WorkspaceSourceBatchMutation::SetOverlay {
                source: provider,
                text: Arc::from(provider_class.as_str()),
                revision: SourceRevision(1),
                recovered_program: None,
            },
            WorkspaceSourceBatchMutation::SetOverlay {
                source: implementation,
                text: Arc::from(impl_source),
                revision: SourceRevision(1),
                recovered_program: None,
            },
        ])
        .expect("cold final publication");
    let cold_plan = cold_result.snapshot.conformance_witness_plans.get(&impl_id).expect("cold conformance plan");
    assert_eq!(second_plan.fingerprint, cold_plan.fingerprint);
    assert_eq!(second.snapshot.diagnostics, cold_result.snapshot.diagnostics);
}

#[test]
fn trait_bodyless_to_bodyful_preserves_requirement_and_source_callable_identity() {
    let module = module_id();
    let source_a = "trait Display { render(_ value: Int) -> Int }\n";
    let source_b = "trait Display { render(_ value: Int) -> Int { value } }\n";
    let mut session = SemanticWorkspaceSession::new();
    let first = session.update(input(module.clone(), source_a, 1)).expect("semantic test update should succeed");
    let second = session.update(input(module.clone(), source_b, 2)).expect("semantic test update should succeed");
    assert!(!first.snapshot.has_errors(), "initial diagnostics: {:?}", first.snapshot.diagnostics);
    assert!(!second.snapshot.has_errors(), "updated diagnostics: {:?}", second.snapshot.diagnostics);
    let declaration = DeclarationId::new(module.clone(), "Display".into());
    let selector = phalcom_common::selector::Selector::method("render", [phalcom_common::selector::SelectorSlot::Positional]).unwrap();
    let first_member = first
        .snapshot
        .trait_surfaces
        .get(&declaration)
        .unwrap()
        .get_by_selector(&selector, DispatchSide::Instance)
        .unwrap();
    let second_member = second
        .snapshot
        .trait_surfaces
        .get(&declaration)
        .unwrap()
        .get_by_selector(&selector, DispatchSide::Instance)
        .unwrap();
    assert_eq!(first_member.requirement, second_member.requirement);
    assert_eq!(first_member.callable, second_member.callable);
    assert!(!first_member.default_present);
    assert!(second_member.default_present);
    assert!(second.snapshot.callable_analyses.contains_key(&second_member.callable));
}

#[test]
fn inferred_trait_result_replaces_conformance_plans_and_rechecks_clients() {
    use phalcom_common::selector::{Selector, SelectorSlot};
    use phalcom_semantic::core_surface::CoreDeclarationIds;
    use phalcom_semantic::db::QueryKey;
    use phalcom_semantic::diagnostic::DiagnosticCode;
    use phalcom_semantic::impls::ConformanceCompleteness;

    let source_string = r#"trait Contract {
  value() { "text" }
  explicit() -> Bool { true }
  bodyless() -> String
}
class Good { bodyless() -> String { "good" } }
impl Contract for Good {}
class Text {
  value() -> String { "witness" }
  bodyless() -> String { "witness bodyless" }
}
impl Contract for Text {}
class Bad {
  value() -> Int { 1 }
  bodyless() -> String { "bad" }
}
impl Contract for Bad {}
class Caller { invoke(_ value: Good) { value.value() } }
"#;
    let source_int = source_string
        .replace("value() { \"text\" }", "value() { 1 }")
        .replace("value() -> String { \"witness\" }", "value() -> Int { 1 }");
    let source_same_result = source_string.replace("\"text\"", "\"other text\"");
    let module = module_id();
    let stable_module = ModuleId::resolved(
        ResolvedProjectId::from_raw(31),
        ModulePath::from_components(vec![ModuleComponent::from_identifier("stable").unwrap()]),
    );
    let workspace_input = |main_source: &str, generation| {
        let main_input = input(module.clone(), main_source, generation);
        let stable_source = "trait Stable { stable() -> String }\nclass Untouched { stable() -> String { \"stable\" } }\nimpl Stable for Untouched {}\n";
        let parsed = phalcom_ast::parse(stable_source, 0);
        assert!(parsed.errors.is_empty(), "stable-module parse errors: {:?}", parsed.errors);
        let mut linked = (*main_input.linked).clone();
        linked.modules.insert(
            stable_module.clone(),
            LinkedModule {
                interface: LinkedModuleInterface {
                    module: stable_module.clone(),
                    kind: ModuleKind::Module,
                    exports: BTreeMap::new(),
                    metadata: ModuleMetadata::default(),
                },
                bindings: ModuleBindingLayout::default(),
                linked_reads: Vec::new(),
                runtime_dependencies: Vec::new(),
            },
        );
        linked.initialization_order.push(stable_module.clone());
        let mut sources = main_input.sources.clone();
        sources.insert(
            stable_module.clone(),
            Arc::new(ParsedModuleUnit::new(
                stable_module.clone(),
                ModuleKind::Module,
                None,
                Arc::from(stable_source),
                Arc::new(parsed.program),
            )),
        );
        SemanticWorkspaceInput::new(Arc::new(linked), sources, generation)
    };
    let mut session = SemanticWorkspaceSession::new();

    let first = session
        .update_with_budget_and_cancel(
            workspace_input(source_string, 1),
            phalcom_semantic::db::QueryBudget::default(),
            &phalcom_semantic::db::CancellationToken::new(),
        )
        .unwrap_or_else(|error| panic!("initial semantic update failed: {error:?}"));
    let good_impl = first
        .snapshot
        .conformance_index
        .iter()
        .find(|(_, contribution)| contribution.target.declaration().name.as_ref() == "Good")
        .expect("Good conformance")
        .1
        .impl_id
        .clone();
    let bad_impl = first
        .snapshot
        .conformance_index
        .iter()
        .find(|(_, contribution)| contribution.target.declaration().name.as_ref() == "Bad")
        .expect("Bad conformance")
        .1
        .impl_id
        .clone();
    let text_impl = first
        .snapshot
        .conformance_index
        .iter()
        .find(|(_, contribution)| contribution.target.declaration().name.as_ref() == "Text")
        .expect("Text conformance")
        .1
        .impl_id
        .clone();
    let stable_impl = first
        .snapshot
        .conformance_index
        .iter()
        .find(|(impl_id, _)| impl_id.module == stable_module)
        .expect("unrelated Stable conformance")
        .1
        .impl_id
        .clone();

    let contract = DeclarationId::new(module.clone(), "Contract".into());
    let surface = first.snapshot.trait_surfaces.get(&contract).expect("Contract surface");
    let requirement = |name: &str| {
        surface
            .get_by_selector(&Selector::method(name, []).expect("requirement selector"), DispatchSide::Instance)
            .expect("trait requirement")
            .requirement
            .clone()
    };
    let inferred_requirement = requirement("value");
    let explicit_requirement = requirement("explicit");
    let bodyless_requirement = requirement("bodyless");
    let string_type = first.snapshot.declarations.form(&CoreDeclarationIds::default().string).expect("String type");
    let bool_type = first.snapshot.declarations.form(&CoreDeclarationIds::default().bool_).expect("Bool type");
    let int_type = first.snapshot.declarations.form(&CoreDeclarationIds::default().int).expect("Int type");

    let default_callable = surface
        .get_by_selector(&Selector::method("value", []).unwrap(), DispatchSide::Instance)
        .expect("default member")
        .callable
        .clone();
    let canonical_default = first.snapshot.callable_signatures.get(&default_callable).expect("canonical default signature");
    assert_eq!(canonical_default.inferred_return.as_ref().and_then(|result| result.ty()), Some(string_type));
    let first_good_plan = first.snapshot.conformance_witness_plans.get(&good_impl).expect("Good plan");
    let first_text_plan = first.snapshot.conformance_witness_plans.get(&text_impl).expect("Text plan");
    let first_bad_plan = first.snapshot.conformance_witness_plans.get(&bad_impl).expect("Bad plan");
    let first_stable_plan = first.snapshot.conformance_witness_plans.get(&stable_impl).expect("Stable plan");
    assert!(matches!(first_good_plan.completeness, ConformanceCompleteness::Complete));
    assert!(
        matches!(first_text_plan.completeness, ConformanceCompleteness::Complete),
        "Text plan: {:?}",
        first_text_plan.completeness
    );
    assert!(matches!(first_bad_plan.completeness, ConformanceCompleteness::Incomplete { .. }));
    assert_eq!(
        first_good_plan.requirement_views[&inferred_requirement]
            .signature
            .published_return_knowledge()
            .ty(),
        Some(string_type),
        "conformance projection must consume the canonical inferred result"
    );
    assert!(first_good_plan.requirement_views[&inferred_requirement].signature.declared_return.is_unknown());
    assert_eq!(
        first_good_plan.requirement_views[&explicit_requirement]
            .signature
            .declared_return
            .canonical_type(),
        Some(bool_type),
        "an explicit result remains the requirement authority"
    );
    assert_eq!(
        first_good_plan.requirement_views[&bodyless_requirement]
            .signature
            .declared_return
            .canonical_type(),
        Some(string_type),
        "bodyless explicit requirements remain unchanged"
    );
    let first_incomplete_count = first.snapshot.diagnostics.get(&module).map_or(0, |diagnostics| {
        diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.code == DiagnosticCode::ImplConformanceIncomplete)
            .count()
    });
    assert_eq!(first_incomplete_count, 1, "only the incompatible Bad witness should be rejected");
    assert_eq!(first_good_plan.source_module, module);

    let caller = CallableId::new(
        DeclarationId::new(module.clone(), "Caller".into()),
        phalcom_common::selector::Selector::method("invoke", [SelectorSlot::Positional]).unwrap(),
        DispatchSide::Instance,
    );
    let body_key = QueryKey::CallableBody(caller.clone());
    let dispatch_key = QueryKey::ConformanceDispatch(good_impl.clone());
    let signature_key = QueryKey::CallableSignature(default_callable.clone());
    let first_client_analysis = first.snapshot.callable_analyses.get(&caller).expect("initial client analysis").clone();
    let first_call = first_client_analysis
        .expressions
        .values()
        .find(|expression| source_string.get(expression.range.start..expression.range.end) == Some("value.value()"))
        .expect("initial trait dispatch call");
    assert_eq!(first_call.knowledge.ty(), Some(string_type));
    let first_plan = first.snapshot.conformance_witness_plans.get(&good_impl).expect("initial Good plan");
    let first_plan_state = session.db().query_state(&dispatch_key).expect("published plan query");
    assert_eq!(first_plan_state.product_fingerprint(), Some(first_plan.fingerprint));
    let signature_fingerprint = phalcom_semantic::db::fingerprint::callable_signature_product_fingerprint(canonical_default);
    assert!(session.db().index().dependencies_of(&dispatch_key).is_some_and(|edges| {
        edges
            .iter()
            .any(|edge| edge.dependency == signature_key && edge.observed_fingerprint == signature_fingerprint)
    }));
    let surface_key = QueryKey::TraitSurface(contract.clone());
    let surface_fingerprint = session.db().ready_product_fingerprint(&surface_key).expect("published trait surface");
    assert!(session.db().index().dependencies_of(&dispatch_key).is_some_and(|edges| {
        edges
            .iter()
            .any(|edge| edge.dependency == surface_key && edge.observed_fingerprint == surface_fingerprint)
    }));
    assert!(session.db().index().dependencies_of(&body_key).is_some_and(|edges| {
        edges
            .iter()
            .any(|edge| edge.dependency == dispatch_key && edge.observed_fingerprint == first_plan.fingerprint)
    }));

    // A body edit with the same inferred result must not replace the plan or
    // force dispatch clients to observe a different semantic fingerprint.
    let second = session
        .update(workspace_input(&source_same_result, 2))
        .expect("semantic test update should succeed");
    let second_good_plan = second.snapshot.conformance_witness_plans.get(&good_impl).expect("same-result Good plan");
    let second_text_plan = second.snapshot.conformance_witness_plans.get(&text_impl).expect("same-result Text plan");
    let second_bad_plan = second.snapshot.conformance_witness_plans.get(&bad_impl).expect("same-result Bad plan");
    let second_stable_plan = second.snapshot.conformance_witness_plans.get(&stable_impl).expect("same-result Stable plan");
    assert_eq!(first_plan.fingerprint, second_good_plan.fingerprint);
    assert_eq!(
        first_text_plan.fingerprint, second_text_plan.fingerprint,
        "a compatible explicit witness plan is also unaffected by same-result edits"
    );
    assert_eq!(
        first.snapshot.conformance_witness_plans[&bad_impl].fingerprint, second_bad_plan.fingerprint,
        "same-result source edits must not churn any affected plan's semantic fingerprint"
    );
    assert!(
        Arc::ptr_eq(first_stable_plan, second_stable_plan),
        "unaffected conformance plan should be reused"
    );
    let second_client_analysis = second.snapshot.callable_analyses.get(&caller).expect("same-result client analysis").clone();
    assert_eq!(
        second.snapshot.diagnostics.get(&module).map_or(0, |diagnostics| {
            diagnostics
                .iter()
                .filter(|diagnostic| diagnostic.code == DiagnosticCode::ImplConformanceIncomplete)
                .count()
        }),
        1,
        "same-result refresh must republish the current incomplete-plan diagnostic exactly once"
    );
    let second_call = second_client_analysis
        .expressions
        .values()
        .find(|expression| source_same_result.get(expression.range.start..expression.range.end) == Some("value.value()"))
        .expect("same-result dispatch call");
    assert_eq!(second_call.knowledge.ty(), Some(string_type));

    // A result-changing provider edit replaces the exact plan, refreshes the
    // already-analyzed client, and retracts the now-obsolete Bad mismatch.
    let third = session.update(workspace_input(&source_int, 3)).expect("semantic test update should succeed");
    let third_good_plan = third.snapshot.conformance_witness_plans.get(&good_impl).expect("updated Good plan");
    let third_text_plan = third.snapshot.conformance_witness_plans.get(&text_impl).expect("updated Text plan");
    let third_bad_plan = third.snapshot.conformance_witness_plans.get(&bad_impl).expect("updated Bad plan");
    let third_stable_plan = third.snapshot.conformance_witness_plans.get(&stable_impl).expect("unaffected Stable plan");
    assert!(
        !Arc::ptr_eq(second_good_plan, third_good_plan),
        "changed inferred requirement replaces the plan"
    );
    assert_ne!(second_good_plan.fingerprint, third_good_plan.fingerprint);
    assert!(matches!(third_good_plan.completeness, ConformanceCompleteness::Complete));
    assert!(matches!(third_text_plan.completeness, ConformanceCompleteness::Complete));
    assert!(matches!(third_bad_plan.completeness, ConformanceCompleteness::Complete));
    assert!(
        Arc::ptr_eq(second_stable_plan, third_stable_plan),
        "unaffected conformance plan must remain reused across a changed result"
    );
    assert_eq!(
        third_good_plan.requirement_views[&inferred_requirement]
            .signature
            .published_return_knowledge()
            .ty(),
        Some(int_type),
        "the replacement plan carries the canonical changed result"
    );
    assert!(!third.snapshot.diagnostics.get(&module).is_some_and(|diagnostics| {
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == DiagnosticCode::ImplConformanceIncomplete)
    }));

    let third_client_analysis = third.snapshot.callable_analyses.get(&caller).expect("updated client analysis");
    assert!(
        !Arc::ptr_eq(&second_client_analysis, third_client_analysis),
        "a changed plan must invalidate and reanalyze clients that consumed the old dispatch result"
    );
    let third_call = third_client_analysis
        .expressions
        .values()
        .find(|expression| source_int.get(expression.range.start..expression.range.end) == Some("value.value()"))
        .expect("updated trait dispatch call");
    assert_eq!(third_call.knowledge.ty(), Some(int_type));
    let current_revision = session.db().revision();
    let current_plan_state = session.db().query_state(&dispatch_key).expect("updated plan query");
    assert_eq!(current_plan_state.product_fingerprint(), Some(third_good_plan.fingerprint));
    assert_eq!(current_plan_state.validated_revision(), Some(current_revision));
    let current_body_state = session.db().query_state(&body_key).expect("updated client query");
    assert_eq!(current_body_state.revision(), Some(current_revision));
    assert!(session.db().index().dependencies_of(&body_key).is_some_and(|edges| {
        edges
            .iter()
            .any(|edge| edge.dependency == dispatch_key && edge.observed_fingerprint == third_good_plan.fingerprint)
    }));

    let mut cold_session = SemanticWorkspaceSession::new();
    let cold = cold_session
        .update(workspace_input(&source_int, 1))
        .expect("semantic test update should succeed");
    assert_eq!(
        third.snapshot.diagnostics, cold.snapshot.diagnostics,
        "changed-result diagnostics must match cold analysis"
    );
    assert_eq!(
        third.snapshot.conformance_witness_plans[&good_impl].fingerprint, cold.snapshot.conformance_witness_plans[&good_impl].fingerprint,
        "changed-result plan must match cold analysis"
    );
    assert_eq!(
        third.snapshot.conformance_witness_plans[&text_impl].fingerprint, cold.snapshot.conformance_witness_plans[&text_impl].fingerprint,
        "compatible explicit witness plan must match cold analysis"
    );
    let cold_client = cold.snapshot.callable_analyses.get(&caller).expect("cold client analysis");
    let cold_call = cold_client
        .expressions
        .values()
        .find(|expression| source_int.get(expression.range.start..expression.range.end) == Some("value.value()"))
        .expect("cold trait dispatch call");
    assert_eq!(cold_call.knowledge.ty(), Some(int_type));
}

#[test]
fn invalidated_static_domain_trait_binder_retires_previous_header() {
    let module = module_id();
    let declaration = DeclarationId::new(module.clone(), "Tagged".into());
    let mut session = SemanticWorkspaceSession::new();
    let first = session
        .update(input(module.clone(), "trait Tagged<T> {}\n", 1))
        .expect("semantic test update should succeed");
    assert!(!first.snapshot.has_errors(), "initial diagnostics: {:?}", first.snapshot.diagnostics);
    assert!(first.snapshot.trait_headers.contains(&declaration));

    let second = session
        .update(input(module, "type data Shape { Scalar }\ntrait Tagged<S: Shape> {}\n", 2))
        .expect("semantic test update should succeed");
    assert!(
        second.snapshot.all_diagnostics().any(|diagnostic| {
            diagnostic.code == phalcom_semantic::diagnostic::DiagnosticCode::AnnotationUnsupported
                && diagnostic.message.contains("trait generic parameters over user-defined static domains")
        }),
        "expected targeted binder diagnostic: {:?}",
        second.snapshot.diagnostics
    );
    assert!(
        !second.snapshot.trait_headers.contains(&declaration),
        "an invalid edit must not retain the prior snapshot's generic header"
    );
}

#[test]
fn trait_incremental_signature_and_header_edits_match_cold_diagnostics() {
    let module = module_id();
    let source_a = r#"
trait Display<T> {
  render(_ value: T) -> T
  fallback(_ value: T) -> T { self.render(value) }
}
"#;
    let source_b = r#"
trait Display<T> where T <: Object {
  render(_ value: T) -> Int
  fallback(_ value: T) -> T { self.render(value) }
}
"#;
    let mut incremental = SemanticWorkspaceSession::new();
    let first = incremental
        .update(input(module.clone(), source_a, 1))
        .expect("semantic test update should succeed");
    assert!(!first.snapshot.has_errors(), "initial diagnostics: {:?}", first.snapshot.diagnostics);
    let updated = incremental
        .update(input(module.clone(), source_b, 2))
        .expect("semantic test update should succeed");
    let mut cold = SemanticWorkspaceSession::new();
    let cold_result = cold.update(input(module.clone(), source_b, 1)).expect("semantic test update should succeed");
    assert_eq!(updated.snapshot.diagnostics, cold_result.snapshot.diagnostics);
    let declaration = DeclarationId::new(module.clone(), "Display".into());
    let incremental_surface = updated.snapshot.trait_surfaces.get(&declaration).expect("incremental surface");
    let cold_surface = cold_result.snapshot.trait_surfaces.get(&declaration).expect("cold surface");
    assert_eq!(incremental_surface, cold_surface, "header/signature edits must preserve cold parity");
}
