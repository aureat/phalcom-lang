use phalcom_common::selector::Selector;
use phalcom_modules::identity::{ModuleComponent, ModuleId, ModulePath, ResolvedProjectId};
use phalcom_modules::interface::{LinkedExport, LinkedExportTarget, LinkedModuleInterface};
use phalcom_modules::linker::{GlobalBindingId, ImportBindingId, LinkedModule, LinkedProgram, LinkedReadSpec, ModuleBindingLayout, SymbolId};
use phalcom_modules::metadata::ModuleMetadata;
use phalcom_modules::project::ProjectUniverse;
use phalcom_modules::source::ModuleKind;
use phalcom_semantic::db::QueryKey;
use phalcom_semantic::identity::{CallableId, CallableOwnerId, DeclarationId, DispatchSide};
use phalcom_semantic::impls::{CallableDefinitionOrigin, ConformanceTarget};
use phalcom_semantic::session::{SemanticWorkspaceSession, SemanticWorkspaceUpdate};
use phalcom_semantic::source::ParsedModuleUnit;
use phalcom_semantic::traits::TraitRef;
use phalcom_semantic::workspace::SemanticWorkspaceInput;
use std::collections::BTreeMap;
use std::sync::Arc;

fn test_module() -> ModuleId {
    ModuleId::resolved(ResolvedProjectId::from_raw(42), ModulePath::root())
}

fn named_module(name: &str) -> ModuleId {
    ModuleId::resolved(
        ResolvedProjectId::from_raw(42),
        ModulePath::from_components(vec![ModuleComponent::from_identifier(name).expect("valid test module name")]),
    )
}

fn single_module_input(module: ModuleId, source: &str) -> SemanticWorkspaceInput {
    let parsed = phalcom_ast::parse(source, 0);
    assert!(parsed.errors.is_empty(), "parse errors: {:?}", parsed.errors);

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
        Arc::new(ParsedModuleUnit::new(
            module,
            ModuleKind::Module,
            None,
            Arc::from(source),
            Arc::new(parsed.program),
        )),
    );
    SemanticWorkspaceInput::new(linked, sources, 1)
}

fn analyze_impl(source: &str, selector: Selector) -> (SemanticWorkspaceSession, SemanticWorkspaceUpdate, CallableId) {
    let module = test_module();
    let callable = CallableId::new(DeclarationId::new(module.clone(), "User".into()), selector, DispatchSide::Instance);
    let mut session = SemanticWorkspaceSession::new();
    let output = session.update(single_module_input(module.clone(), source));
    assert!(!output.snapshot.has_errors(), "diagnostics: {:?}", output.snapshot.diagnostics);
    (session, output, callable)
}

fn owner(name: &str) -> DeclarationId {
    DeclarationId::new(test_module(), name.into())
}

fn impl_callable(owner_name: &str, selector: Selector) -> CallableId {
    CallableId::new(owner(owner_name), selector, DispatchSide::Instance)
}

fn surface_fingerprint(output: &SemanticWorkspaceUpdate, declaration: &DeclarationId) -> u64 {
    phalcom_semantic::db::fingerprint::declaration_surface_product_fingerprint(output.snapshot.surfaces().get(declaration).expect("declaration surface")).raw()
}

#[test]
fn explicit_conformance_is_indexed_without_polluting_inherent_surface() {
    let module = test_module();
    let source = "trait Tagged { tag -> String }\nclass User {}\nimpl Tagged for User { tag -> String { \"tagged\" } }\n";
    let mut session = SemanticWorkspaceSession::new();
    let output = session.update(single_module_input(module.clone(), source));
    assert!(!output.snapshot.has_errors(), "diagnostics: {:?}", output.snapshot.diagnostics);

    let trait_ref = TraitRef::new(
        DeclarationId::new(module.clone(), "Tagged".into()),
        Vec::<phalcom_semantic::TypeId>::new().into_boxed_slice(),
    );
    let user = output
        .snapshot
        .declarations
        .form(&DeclarationId::new(module.clone(), "User".into()))
        .expect("User type");
    let mut store = (*output.snapshot.store).clone();
    let matches = output.snapshot.conformance_index.query_exact(&mut store, user, &trait_ref);
    assert_eq!(matches.len(), 1, "expected one exact conformance head");
    assert_eq!(matches[0].impl_id.module, module);
    assert!(
        !output
            .snapshot
            .surfaces()
            .get(&DeclarationId::new(test_module(), "User".into()))
            .unwrap()
            .instance
            .callable_signatures
            .contains_key(&Selector::getter("tag").unwrap())
    );
}

#[test]
fn generic_conformance_head_matches_exact_target_specialization() {
    let module = test_module();
    let source = "trait Tagged {}\nclass Value<T> {}\nclass Marker {}\nimpl<T> Tagged for Value<T> {}\n";
    let mut session = SemanticWorkspaceSession::new();
    let output = session.update(single_module_input(module.clone(), source));
    assert!(!output.snapshot.has_errors(), "diagnostics: {:?}", output.snapshot.diagnostics);

    let value = DeclarationId::new(module.clone(), "Value".into());
    let marker = DeclarationId::new(module.clone(), "Marker".into());
    let value_form = output.snapshot.declarations.form(&value).expect("Value form");
    let marker_form = output.snapshot.declarations.form(&marker).expect("Marker form");
    let mut store = (*output.snapshot.store).clone();
    let specialized = store.apply_type_form(value_form, &[marker_form]).expect("specialized Value form");
    let trait_ref = TraitRef::new(
        DeclarationId::new(module, "Tagged".into()),
        Vec::<phalcom_semantic::TypeId>::new().into_boxed_slice(),
    );
    let matches = output.snapshot.conformance_index.query_exact(&mut store, specialized, &trait_ref);
    assert_eq!(matches.len(), 1, "generic conformance should match one exact specialization");
    assert_eq!(matches[0].impl_bindings.len(), 1, "impl generic parameter must be specialized");
    assert_eq!(matches[0].exact_target, specialized);
}

#[test]
fn overlapping_generic_and_specialized_conformances_are_rejected() {
    let module = test_module();
    let source = "trait Tagged {}\nclass Value<T> {}\nclass Marker {}\nimpl<T> Tagged for Value<T> {}\nimpl Tagged for Value<Marker> {}\n";
    let mut session = SemanticWorkspaceSession::new();
    let output = session.update(single_module_input(module.clone(), source));
    let diagnostics = output.snapshot.diagnostics_for(&module).expect("module diagnostics");
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == phalcom_semantic::DiagnosticCode::ImplConformanceOverlap),
        "expected overlap diagnostic: {diagnostics:?}"
    );
}

#[test]
fn incremental_conformance_add_edit_delete_updates_snapshot_index() {
    let module = test_module();
    let mut session = SemanticWorkspaceSession::new();
    let source_v1 = "trait Tagged {}\nclass User {}\nimpl Tagged for User {}\n";
    let first = session.update(single_module_input(module.clone(), source_v1));
    assert!(!first.snapshot.has_errors(), "diagnostics: {:?}", first.snapshot.diagnostics);
    assert_eq!(first.snapshot.conformance_index.iter().count(), 1);

    let source_v2 = "trait Tagged {}\nclass User {}\n";
    let second = session.update(single_module_input(module.clone(), source_v2));
    assert!(!second.snapshot.has_errors(), "diagnostics: {:?}", second.snapshot.diagnostics);
    assert_eq!(
        second.snapshot.conformance_index.iter().count(),
        0,
        "deleted conformance must not remain published"
    );

    let source_v3 = "trait Tagged {}\nclass User {}\nimpl Tagged for User {}\n";
    let third = session.update(single_module_input(module, source_v3));
    assert!(!third.snapshot.has_errors(), "diagnostics: {:?}", third.snapshot.diagnostics);
    assert_eq!(third.snapshot.conformance_index.iter().count(), 1, "re-added conformance must be published");
}

#[test]
fn incremental_and_cold_conformance_publication_have_the_same_head_identity() {
    let module = test_module();
    let source_v1 = "trait Tagged {}\nclass User {}\n";
    let source_v2 = "trait Tagged {}\nclass User {}\nimpl Tagged for User {}\n";
    let mut incremental = SemanticWorkspaceSession::new();
    let _ = incremental.update(single_module_input(module.clone(), source_v1));
    let incremental_output = incremental.update(single_module_input(module.clone(), source_v2));
    let mut cold = SemanticWorkspaceSession::new();
    let cold_output = cold.update(single_module_input(module.clone(), source_v2));

    let user = incremental_output
        .snapshot
        .declarations
        .form(&DeclarationId::new(module.clone(), "User".into()))
        .expect("User type");
    let trait_ref = TraitRef::new(DeclarationId::new(module, "Tagged".into()), Vec::new().into_boxed_slice());
    let mut incremental_store = (*incremental_output.snapshot.store).clone();
    let mut cold_store = (*cold_output.snapshot.store).clone();
    let incremental_matches = incremental_output
        .snapshot
        .conformance_index
        .query_exact(&mut incremental_store, user, &trait_ref);
    let cold_user = cold_output
        .snapshot
        .declarations
        .form(&DeclarationId::new(test_module(), "User".into()))
        .expect("cold User type");
    let cold_matches = cold_output.snapshot.conformance_index.query_exact(&mut cold_store, cold_user, &trait_ref);
    assert_eq!(incremental_matches.len(), 1);
    assert_eq!(cold_matches.len(), 1);
    assert_eq!(incremental_matches[0].impl_id, cold_matches[0].impl_id);
}

#[test]
fn distinct_trait_reference_arguments_remain_distinct_conformance_domains() {
    let module = test_module();
    let source = "trait Converter<T> {}\nclass User {}\nclass Int {}\nclass Text {}\nimpl Converter<Int> for User {}\nimpl Converter<Text> for User {}\n";
    let mut session = SemanticWorkspaceSession::new();
    let output = session.update(single_module_input(module.clone(), source));
    assert!(!output.snapshot.has_errors(), "diagnostics: {:?}", output.snapshot.diagnostics);
    assert_eq!(output.snapshot.conformance_index.iter().count(), 2);

    let mut store = (*output.snapshot.store).clone();
    let user = output
        .snapshot
        .declarations
        .form(&DeclarationId::new(module.clone(), "User".into()))
        .expect("User form");
    let int = output
        .snapshot
        .declarations
        .form(&DeclarationId::new(module.clone(), "Int".into()))
        .expect("Int form");
    let text = output
        .snapshot
        .declarations
        .form(&DeclarationId::new(module.clone(), "Text".into()))
        .expect("Text form");
    let trait_decl = DeclarationId::new(module, "Converter".into());
    let int_matches = output
        .snapshot
        .conformance_index
        .query_exact(&mut store, user, &TraitRef::new(trait_decl.clone(), vec![int].into_boxed_slice()));
    let text_matches = output
        .snapshot
        .conformance_index
        .query_exact(&mut store, user, &TraitRef::new(trait_decl, vec![text].into_boxed_slice()));
    assert_eq!(int_matches.len(), 1);
    assert_eq!(text_matches.len(), 1);
    assert_ne!(int_matches[0].impl_id, text_matches[0].impl_id);
}

#[test]
fn exact_enum_case_conformance_retains_variant_identity() {
    let module = test_module();
    let source = "trait Tagged {}\nenum Result { Ok }\nimpl Tagged for Result::Ok {}\n";
    let mut session = SemanticWorkspaceSession::new();
    let output = session.update(single_module_input(module.clone(), source));
    assert!(!output.snapshot.has_errors(), "diagnostics: {:?}", output.snapshot.diagnostics);
    let (_, contribution) = output.snapshot.conformance_index.iter().next().expect("conformance contribution");
    let ConformanceTarget::ExactEnumCase(variant) = &contribution.target else {
        panic!("expected exact enum-case target");
    };
    assert_eq!(variant.owner.name.as_ref(), "Result");
    assert_eq!(variant.selector, Selector::getter("Ok").unwrap());
    let variant_target = phalcom_semantic::identity::SemanticTargetId::Variant(variant.clone());
    assert!(
        output.snapshot.source_index.occurrences_for_target(&variant_target).is_some(),
        "exact target source site must retain variant identity"
    );
}

#[test]
fn linked_modules_publish_canonical_ownership_through_reexports() {
    let traits = named_module("traits");
    let models = named_module("models");
    let facade = named_module("facade");
    let foreign = named_module("foreign");
    let trait_symbol = SymbolId {
        module: traits.clone(),
        name: "Tagged".into(),
    };
    let user_symbol = SymbolId {
        module: models.clone(),
        name: "User".into(),
    };
    let module_specs = [
        (
            traits.clone(),
            "trait Tagged {}\nexport Tagged\n",
            LinkedModule {
                interface: LinkedModuleInterface {
                    module: traits.clone(),
                    kind: ModuleKind::Module,
                    exports: BTreeMap::from([(
                        "Tagged".into(),
                        LinkedExport {
                            public_name: "Tagged".into(),
                            target: LinkedExportTarget::Binding(trait_symbol.clone()),
                            range: Default::default(),
                        },
                    )]),
                    metadata: ModuleMetadata::default(),
                },
                bindings: ModuleBindingLayout {
                    local_globals: BTreeMap::from([("Tagged".into(), GlobalBindingId(0))]),
                    imports: BTreeMap::new(),
                },
                linked_reads: Vec::new(),
                runtime_dependencies: Vec::new(),
            },
        ),
        (
            models.clone(),
            "from facade import Tagged\nclass User {}\nimpl Tagged for User {}\nexport User\n",
            LinkedModule {
                interface: LinkedModuleInterface {
                    module: models.clone(),
                    kind: ModuleKind::Module,
                    exports: BTreeMap::from([(
                        "User".into(),
                        LinkedExport {
                            public_name: "User".into(),
                            target: LinkedExportTarget::Binding(user_symbol.clone()),
                            range: Default::default(),
                        },
                    )]),
                    metadata: ModuleMetadata::default(),
                },
                bindings: ModuleBindingLayout {
                    local_globals: BTreeMap::from([("User".into(), GlobalBindingId(0))]),
                    imports: BTreeMap::from([("Tagged".into(), ImportBindingId(0))]),
                },
                linked_reads: vec![LinkedReadSpec::Binding(trait_symbol.clone())],
                runtime_dependencies: vec![facade.clone()],
            },
        ),
        (
            facade.clone(),
            "from traits import Tagged\nexport Tagged\n",
            LinkedModule {
                interface: LinkedModuleInterface {
                    module: facade.clone(),
                    kind: ModuleKind::Module,
                    exports: BTreeMap::from([(
                        "Tagged".into(),
                        LinkedExport {
                            public_name: "Tagged".into(),
                            target: LinkedExportTarget::Binding(trait_symbol.clone()),
                            range: Default::default(),
                        },
                    )]),
                    metadata: ModuleMetadata::default(),
                },
                bindings: ModuleBindingLayout {
                    local_globals: BTreeMap::new(),
                    imports: BTreeMap::from([("Tagged".into(), ImportBindingId(0))]),
                },
                linked_reads: vec![LinkedReadSpec::Binding(trait_symbol.clone())],
                runtime_dependencies: vec![traits.clone()],
            },
        ),
        (
            foreign.clone(),
            "from facade import Tagged\nfrom models import User\nimpl Tagged for User {}\n",
            LinkedModule {
                interface: LinkedModuleInterface {
                    module: foreign.clone(),
                    kind: ModuleKind::Module,
                    exports: BTreeMap::new(),
                    metadata: ModuleMetadata::default(),
                },
                bindings: ModuleBindingLayout {
                    local_globals: BTreeMap::new(),
                    imports: BTreeMap::from([("Tagged".into(), ImportBindingId(0)), ("User".into(), ImportBindingId(1))]),
                },
                linked_reads: vec![LinkedReadSpec::Binding(trait_symbol), LinkedReadSpec::Binding(user_symbol)],
                runtime_dependencies: vec![facade.clone(), models.clone()],
            },
        ),
    ];
    let mut sources = BTreeMap::new();
    let mut linked_modules = BTreeMap::new();
    for (module, source, linked_module) in module_specs {
        let parsed = phalcom_ast::parse(source, 0);
        assert!(parsed.errors.is_empty(), "parse errors in {module}: {:?}", parsed.errors);
        sources.insert(
            module.clone(),
            Arc::new(ParsedModuleUnit::new(
                module.clone(),
                ModuleKind::Module,
                None,
                Arc::from(source),
                Arc::new(parsed.program),
            )),
        );
        linked_modules.insert(module, linked_module);
    }
    let linked = Arc::new(LinkedProgram {
        universe: Arc::new(ProjectUniverse::new()),
        modules: linked_modules,
        graphs: Default::default(),
        entry: models.clone(),
        initialization_order: vec![traits.clone(), facade.clone(), models.clone(), foreign.clone()],
    });
    let mut session = SemanticWorkspaceSession::new();
    let output = session.update(SemanticWorkspaceInput::new(linked, sources, 1));
    let trait_decl = DeclarationId::new(traits, "Tagged".into());
    let user_decl = DeclarationId::new(models.clone(), "User".into());
    let user = output.snapshot.declarations.form(&user_decl).expect("User type");
    let mut store = (*output.snapshot.store).clone();
    let matches = output
        .snapshot
        .conformance_index
        .query_exact(&mut store, user, &TraitRef::new(trait_decl, Vec::new().into_boxed_slice()));
    assert_eq!(matches.len(), 1, "only the target-owner conformance is eligible");
    assert_eq!(matches[0].impl_id.module, models);
    assert!(
        output.snapshot.diagnostics_for(&foreign).is_some_and(|diagnostics| diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == phalcom_semantic::DiagnosticCode::ImplForeignTarget)),
        "third-party conformance must be rejected even when both names arrive through linked imports"
    );
}

#[test]
fn p1_generic_specialization_matrix_and_iterable_head_stay_at_head_level() {
    let module = test_module();
    let source = "trait Tagged { tag -> String }\ntrait Iterable<Item, Cursor> {\n  iterate(_ cursor: Cursor) -> Cursor\n  iteratorValue(_ cursor: Cursor) -> Item\n}\nclass Value<T> {}\nclass Text {}\nclass Number {}\nclass Boolish {}\nclass Countdown {}\nimpl Tagged for Value<Text> { tag -> String { \"text\" } }\nimpl Tagged for Value<Number> { tag -> String { \"number\" } }\nimpl Iterable<Countdown, Countdown> for Countdown {}\n";
    let mut session = SemanticWorkspaceSession::new();
    let output = session.update(single_module_input(module.clone(), source));
    assert!(!output.snapshot.has_errors(), "diagnostics: {:?}", output.snapshot.diagnostics);

    let value = DeclarationId::new(module.clone(), "Value".into());
    let text = DeclarationId::new(module.clone(), "Text".into());
    let number = DeclarationId::new(module.clone(), "Number".into());
    let boolish = DeclarationId::new(module.clone(), "Boolish".into());
    let countdown = DeclarationId::new(module.clone(), "Countdown".into());
    let value_form = output.snapshot.declarations.form(&value).expect("Value form");
    let text_form = output.snapshot.declarations.form(&text).expect("Text form");
    let number_form = output.snapshot.declarations.form(&number).expect("Number form");
    let boolish_form = output.snapshot.declarations.form(&boolish).expect("Boolish form");
    let countdown_form = output.snapshot.declarations.form(&countdown).expect("Countdown form");
    let mut store = (*output.snapshot.store).clone();
    let value_text = store.apply_type_form(value_form, &[text_form]).expect("Value<Text>");
    let value_number = store.apply_type_form(value_form, &[number_form]).expect("Value<Number>");
    let value_boolish = store.apply_type_form(value_form, &[boolish_form]).expect("Value<Boolish>");
    let tagged = TraitRef::new(DeclarationId::new(module.clone(), "Tagged".into()), Vec::new().into_boxed_slice());
    let text_match = output.snapshot.conformance_index.query_exact(&mut store, value_text, &tagged);
    let number_match = output.snapshot.conformance_index.query_exact(&mut store, value_number, &tagged);
    let boolish_match = output.snapshot.conformance_index.query_exact(&mut store, value_boolish, &tagged);
    assert_eq!(text_match.len(), 1);
    assert_eq!(number_match.len(), 1);
    assert!(boolish_match.is_empty(), "unlisted Value<Boolish> must have no P1 candidate");
    assert_ne!(text_match[0].impl_id, number_match[0].impl_id);
    assert_eq!(text_match[0].exact_target, value_text);
    assert_eq!(number_match[0].exact_target, value_number);

    let iterable = TraitRef::new(
        DeclarationId::new(module, "Iterable".into()),
        vec![countdown_form, countdown_form].into_boxed_slice(),
    );
    let iterable_match = output.snapshot.conformance_index.query_exact(&mut store, countdown_form, &iterable);
    assert_eq!(iterable_match.len(), 1, "generic trait reference must remain an exact indexed domain");
}

#[test]
fn test_query_callable_signature_for_impl_method() {
    let (session, output, callable) = analyze_impl(
        "class User {}\nimpl User {\n  name() -> String { \"hello\" }\n}\n",
        Selector::method("name", []).unwrap(),
    );

    let signature = session
        .db()
        .product(&QueryKey::CallableSignature(callable.clone()))
        .and_then(|product| product.as_callable_signature())
        .expect("workspace query should publish the impl signature");
    assert_eq!(signature.callable, callable);
    assert_eq!(signature.owner, DeclarationId::new(test_module(), "User".into()));
    assert_eq!(signature.side, DispatchSide::Instance);
    assert!(output.snapshot.callable_signatures.get(&signature.callable).is_some());
    let definition = output
        .snapshot
        .callable_definitions
        .get(&callable)
        .expect("snapshot must retain accepted impl provenance for lowering");
    assert!(matches!(definition.origin, CallableDefinitionOrigin::InherentImpl(_)));
}

#[test]
fn test_query_callable_body_for_impl_method() {
    let (session, output, callable) = analyze_impl(
        "class User {}\nimpl User {\n  greet() -> String { \"hello world\" }\n}\n",
        Selector::method("greet", []).unwrap(),
    );

    let analysis = session
        .db()
        .product(&QueryKey::CallableBody(callable.clone()))
        .and_then(|product| product.as_callable_body())
        .expect("workspace query should publish the impl body");
    assert_eq!(analysis.callable, callable);
    assert!(analysis.diagnostics.is_empty());
    assert!(output.snapshot.callable_analyses.contains_key(&analysis.callable));
}

#[test]
fn test_rejected_duplicate_impl_definition_is_not_published() {
    let module = test_module();
    let callable = CallableId::new(
        DeclarationId::new(module.clone(), "User".into()),
        Selector::method("name", []).unwrap(),
        DispatchSide::Instance,
    );
    let mut session = SemanticWorkspaceSession::new();
    let output = session.update(single_module_input(
        module,
        "class User {}\nimpl User { name() -> String { \"first\" } }\nimpl User { name() -> String { \"second\" } }\n",
    ));
    assert!(output.snapshot.has_errors(), "duplicate impl member must diagnose");

    let definition = session
        .db()
        .product(&QueryKey::CallableDefinition(callable.clone()))
        .and_then(|product| product.as_callable_definition())
        .expect("accepted impl definition");
    assert!(matches!(definition.origin, CallableDefinitionOrigin::InherentImpl(ref id) if id.local.0 == 1));
    assert_eq!(definition.source_member_index, 0);
    let snapshot_definition = output
        .snapshot
        .callable_definitions
        .get(&callable)
        .expect("only the accepted duplicate candidate reaches lowering");
    assert!(matches!(snapshot_definition.origin, CallableDefinitionOrigin::InherentImpl(ref id) if id.local.0 == 1));
}

#[test]
fn incremental_impl_edit_matches_cold_effective_surfaces() {
    let source_v1 = "class Alpha {}\nclass Beta {}\nimpl Alpha { ping() -> String { \"a\" } }\nimpl Beta { ping() -> Int { 1 } }\n";
    let source_v2 = "class Alpha {}\nclass Beta {}\nimpl Alpha { ping() -> String { \"changed\" } }\nimpl Beta { ping() -> Int { 2 } }\n";
    let module = test_module();
    let mut session = SemanticWorkspaceSession::new();
    let first = session.update(single_module_input(module.clone(), source_v1));
    assert!(!first.snapshot.has_errors(), "diagnostics: {:?}", first.snapshot.diagnostics);
    let second = session.update(single_module_input(module.clone(), source_v2));
    assert!(!second.snapshot.has_errors(), "diagnostics: {:?}", second.snapshot.diagnostics);

    let mut cold_session = SemanticWorkspaceSession::new();
    let cold = cold_session.update(single_module_input(module, source_v2));
    assert!(!cold.snapshot.has_errors(), "cold diagnostics: {:?}", cold.snapshot.diagnostics);

    for name in ["Alpha", "Beta"] {
        let declaration = owner(name);
        assert_eq!(surface_fingerprint(&second, &declaration), surface_fingerprint(&cold, &declaration));
    }
    assert_eq!(second.snapshot.callable_definitions, cold.snapshot.callable_definitions);
}

#[test]
fn impl_body_only_edit_preserves_surface_and_reuses_unaffected_callable() {
    let module = test_module();
    let callable = impl_callable("User", Selector::method("greet", []).unwrap());
    let mut session = SemanticWorkspaceSession::new();
    let first = session.update(single_module_input(
        module.clone(),
        "class User {}\nimpl User { greet() -> String { \"first\" } }\nclass Other { keep() -> Bool { true } }\n",
    ));
    assert!(!first.snapshot.has_errors(), "diagnostics: {:?}", first.snapshot.diagnostics);
    let first_surface = surface_fingerprint(&first, &owner("User"));
    let first_signature = first.snapshot.callable_signatures.get(&callable).expect("impl signature").clone();
    let first_analysis = first.snapshot.callable_analyses.get(&callable).cloned().expect("impl body");

    let second = session.update(single_module_input(
        module,
        "class User {}\nimpl User { greet() -> String { \"second\" } }\nclass Other { keep() -> Bool { false } }\n",
    ));
    assert!(!second.snapshot.has_errors(), "diagnostics: {:?}", second.snapshot.diagnostics);
    assert_eq!(
        first_surface,
        surface_fingerprint(&second, &owner("User")),
        "impl body bytes must not enter effective surface identity"
    );
    assert_eq!(
        first_signature,
        *second.snapshot.callable_signatures.get(&callable).expect("reused impl signature")
    );
    assert!(!Arc::ptr_eq(
        &first_analysis,
        second.snapshot.callable_analyses.get(&callable).expect("refreshed impl body")
    ));
    assert!(second.stats.callable_signatures_reused > 0, "signature product should remain reusable");
}

#[test]
fn unrelated_class_body_edit_reuses_impl_callable_and_surface() {
    let module = test_module();
    let callable = impl_callable("User", Selector::method("greet", []).unwrap());
    let mut session = SemanticWorkspaceSession::new();
    let first = session.update(single_module_input(
        module.clone(),
        "class User {}\nimpl User { greet() -> String { \"hi\" } }\nclass Other { keep() -> Bool { true } }\n",
    ));
    assert!(!first.snapshot.has_errors(), "diagnostics: {:?}", first.snapshot.diagnostics);
    let first_surface = surface_fingerprint(&first, &owner("User"));
    let first_analysis = first.snapshot.callable_analyses.get(&callable).cloned().expect("impl body");

    let second = session.update(single_module_input(
        module,
        "class User {}\nimpl User { greet() -> String { \"hi\" } }\nclass Other { keep() -> Bool { false } }\n",
    ));
    assert!(!second.snapshot.has_errors(), "diagnostics: {:?}", second.snapshot.diagnostics);
    assert_eq!(first_surface, surface_fingerprint(&second, &owner("User")));
    assert!(Arc::ptr_eq(
        &first_analysis,
        second.snapshot.callable_analyses.get(&callable).expect("unrelated edit must retain impl body")
    ));
}

#[test]
fn impl_return_annotation_and_member_addition_change_only_affected_surfaces() {
    let module = test_module();
    let ping = impl_callable("Alpha", Selector::method("ping", []).unwrap());
    let mut session = SemanticWorkspaceSession::new();
    let first = session.update(single_module_input(
        module.clone(),
        "class Alpha {}\nclass Beta {}\nclass Caller { run(value: Alpha) { value.ping() } }\nimpl Alpha { ping() -> String { \"a\" } }\nimpl Beta { keep() -> Bool { true } }\n",
    ));
    assert!(!first.snapshot.has_errors(), "diagnostics: {:?}", first.snapshot.diagnostics);
    let caller = impl_callable(
        "Caller",
        Selector::method("run", [phalcom_common::selector::SelectorSlot::Label("value".into())]).unwrap(),
    );
    let first_caller = first.snapshot.callable_analyses.get(&caller).cloned().expect("caller body");

    let second = session.update(single_module_input(
        module.clone(),
        "class Alpha {}\nclass Beta {}\nclass Caller { run(value: Alpha) { value.ping() } }\nimpl Alpha { ping() -> Int { 1 } extra() -> Bool { true } }\nimpl Beta { keep() -> Bool { true } }\n",
    ));
    assert!(!second.snapshot.has_errors(), "diagnostics: {:?}", second.snapshot.diagnostics);
    assert_ne!(
        first.snapshot.callable_signatures.get(&ping),
        second.snapshot.callable_signatures.get(&ping),
        "return annotation changes the accepted target signature"
    );
    assert_ne!(surface_fingerprint(&first, &owner("Alpha")), surface_fingerprint(&second, &owner("Alpha")));
    assert!(second.snapshot.surfaces().get(&owner("Alpha")).is_some_and(|surface| {
        surface
            .instance
            .callable_signatures
            .keys()
            .any(|selector| selector.encode().starts_with("extra"))
    }));
    assert!(
        !Arc::ptr_eq(&first_caller, second.snapshot.callable_analyses.get(&caller).expect("caller body")),
        "target signature change invalidates dependent caller"
    );
}

#[test]
fn adding_impl_member_does_not_invalidate_another_target_or_its_caller() {
    let module = test_module();
    let caller = impl_callable(
        "Caller",
        Selector::method("run", [phalcom_common::selector::SelectorSlot::Label("value".into())]).unwrap(),
    );
    let mut session = SemanticWorkspaceSession::new();
    let first = session.update(single_module_input(
        module.clone(),
        "class Alpha {}\nclass Beta {}\nclass Caller { run(value: Alpha) { value.ping() } }\nimpl Alpha { ping() -> Int { 1 } }\nimpl Beta { keep() -> Bool { true } }\n",
    ));
    assert!(!first.snapshot.has_errors(), "diagnostics: {:?}", first.snapshot.diagnostics);
    let alpha_surface = surface_fingerprint(&first, &owner("Alpha"));
    let caller_v1 = first.snapshot.callable_analyses.get(&caller).cloned().expect("caller body");

    let second = session.update(single_module_input(
        module,
        "class Alpha {}\nclass Beta {}\nclass Caller { run(value: Alpha) { value.ping() } }\nimpl Alpha { ping() -> Int { 1 } }\nimpl Beta { keep() -> Bool { true } extra() -> Int { 2 } }\n",
    ));
    assert!(!second.snapshot.has_errors(), "diagnostics: {:?}", second.snapshot.diagnostics);
    assert_eq!(alpha_surface, surface_fingerprint(&second, &owner("Alpha")));
    assert!(Arc::ptr_eq(
        &caller_v1,
        second.snapshot.callable_analyses.get(&caller).expect("unchanged caller body")
    ));
    assert_ne!(surface_fingerprint(&first, &owner("Beta")), surface_fingerprint(&second, &owner("Beta")));
}

#[test]
fn impl_whitespace_and_range_edit_preserves_semantic_surface_fingerprint() {
    let module = test_module();
    let callable = impl_callable("User", Selector::method("greet", []).unwrap());
    let mut session = SemanticWorkspaceSession::new();
    let first = session.update(single_module_input(
        module.clone(),
        "class User {}\nimpl User { greet() -> String { \"hi\" } }\n",
    ));
    assert!(!first.snapshot.has_errors(), "diagnostics: {:?}", first.snapshot.diagnostics);
    let first_surface = surface_fingerprint(&first, &owner("User"));
    let first_signature =
        phalcom_semantic::db::fingerprint::callable_signature_product_fingerprint(first.snapshot.callable_signatures.get(&callable).expect("impl signature"));

    let second = session.update(single_module_input(
        module,
        "class User {}\n\nimpl User {\n  greet() -> String {\n    \"hi\"\n  }\n}\n",
    ));
    assert!(!second.snapshot.has_errors(), "diagnostics: {:?}", second.snapshot.diagnostics);
    assert_eq!(first_surface, surface_fingerprint(&second, &owner("User")));
    assert_eq!(
        first_signature,
        phalcom_semantic::db::fingerprint::callable_signature_product_fingerprint(second.snapshot.callable_signatures.get(&callable).expect("impl signature")),
        "range movement must not change the semantic signature product"
    );
}

#[test]
fn moving_impl_target_retires_old_callable_and_publishes_new_target() {
    let module = test_module();
    let alpha_ping = impl_callable("Alpha", Selector::method("ping", []).unwrap());
    let beta_ping = impl_callable("Beta", Selector::method("ping", []).unwrap());
    let mut session = SemanticWorkspaceSession::new();
    let first = session.update(single_module_input(
        module.clone(),
        "class Alpha {}\nclass Beta {}\nimpl Alpha { ping() -> Int { 1 } }\n",
    ));
    assert!(!first.snapshot.has_errors(), "diagnostics: {:?}", first.snapshot.diagnostics);
    assert!(first.snapshot.callable_definitions.contains_key(&alpha_ping));

    let second = session.update(single_module_input(
        module,
        "class Alpha {}\nclass Beta {}\nimpl Beta { ping() -> Int { 1 } }\n",
    ));
    assert!(
        !second.snapshot.callable_definitions.contains_key(&alpha_ping),
        "old target must lose the moved definition"
    );
    assert!(
        second.snapshot.callable_definitions.contains_key(&beta_ping),
        "new target must publish the moved definition"
    );
    assert!(
        !second
            .snapshot
            .surfaces()
            .get(&owner("Alpha"))
            .expect("Alpha surface")
            .instance
            .callable_signatures
            .keys()
            .any(|selector| selector.encode().starts_with("ping"))
    );
    assert!(
        second
            .snapshot
            .surfaces()
            .get(&owner("Beta"))
            .expect("Beta surface")
            .instance
            .callable_signatures
            .keys()
            .any(|selector| selector.encode().starts_with("ping"))
    );
}

#[test]
fn incremental_adding_variant_invalidates_closed_requirement_completeness() {
    let module = test_module();
    let mut session = SemanticWorkspaceSession::new();
    let source_v1 = "enum Expr {\n  Lit(val: Int)\n}\nimpl Expr {\n  eval() -> Int\n}\nimpl Expr::Lit(val: _) {\n  eval() -> Int { self.val }\n}\n";
    let first = session.update(single_module_input(module.clone(), source_v1));
    assert!(!first.snapshot.has_errors(), "v1 must have no diagnostics: {:?}", first.snapshot.diagnostics);

    // Adding Add variant without an exact case implementation for eval() must emit missing requirement diagnostic
    let source_v2 = "enum Expr {\n  Lit(val: Int)\n  Add(left: Expr, right: Expr)\n}\nimpl Expr {\n  eval() -> Int\n}\nimpl Expr::Lit(val: _) {\n  eval() -> Int { self.val }\n}\n";
    let second = session.update(single_module_input(module.clone(), source_v2));
    assert!(second.snapshot.has_errors(), "v2 must produce missing requirement error for Add");

    // Cold analysis must produce matching diagnostics
    let mut cold_session = SemanticWorkspaceSession::new();
    let cold = cold_session.update(single_module_input(module, source_v2));
    assert!(cold.snapshot.has_errors(), "cold v2 must produce missing requirement error");
    assert_eq!(second.snapshot.has_errors(), cold.snapshot.has_errors());
}

#[test]
fn incremental_exact_case_body_edit_preserves_unrelated_products() {
    let module = test_module();
    let mut session = SemanticWorkspaceSession::new();
    let source_v1 = "enum Expr {\n  Lit(val: Int)\n  Add(left: Expr, right: Expr)\n}\nimpl Expr {\n  eval() -> Int\n}\nimpl Expr::Lit(val: _) {\n  eval() -> Int { self.val }\n}\nimpl Expr::Add(left: _, right: _) {\n  eval() -> Int { self.left.eval() + self.right.eval() }\n}\n";
    let first = session.update(single_module_input(module.clone(), source_v1));
    assert!(!first.snapshot.has_errors(), "diagnostics: {:?}", first.snapshot.diagnostics);

    let lit_var = phalcom_semantic::identity::VariantId::new(
        owner("Expr"),
        phalcom_common::selector::Selector::method("Lit", [phalcom_common::selector::SelectorSlot::Label("val".into())]).unwrap(),
    );
    let lit_callable = CallableId::new(
        phalcom_semantic::identity::CallableOwnerId::Variant(lit_var),
        Selector::method("eval", []).unwrap(),
        DispatchSide::Instance,
    );
    let first_lit_analysis = first.snapshot.callable_analyses.get(&lit_callable).cloned().expect("lit body analysis");

    // Edit only Lit body
    let source_v2 = "enum Expr {\n  Lit(val: Int)\n  Add(left: Expr, right: Expr)\n}\nimpl Expr {\n  eval() -> Int\n}\nimpl Expr::Lit(val: _) {\n  eval() -> Int { self.val + 1 }\n}\nimpl Expr::Add(left: _, right: _) {\n  eval() -> Int { self.left.eval() + self.right.eval() }\n}\n";
    let second = session.update(single_module_input(module.clone(), source_v2));
    assert!(!second.snapshot.has_errors(), "diagnostics: {:?}", second.snapshot.diagnostics);

    let second_lit_analysis = second.snapshot.callable_analyses.get(&lit_callable).expect("updated lit body analysis");
    assert!(!Arc::ptr_eq(&first_lit_analysis, second_lit_analysis), "lit body analysis must be refreshed");

    let mut cold_session = SemanticWorkspaceSession::new();
    let cold = cold_session.update(single_module_input(module, source_v2));
    assert_eq!(second.snapshot.callable_definitions, cold.snapshot.callable_definitions);
}

#[test]
fn incremental_case_only_member_leaves_root_surface_unchanged() {
    let module = test_module();
    let mut session = SemanticWorkspaceSession::new();
    let source_v1 = "enum Choice {\n  A\n  B\n}\nimpl Choice::A {\n  aOnly() -> Int { 42 }\n}\n";
    let first = session.update(single_module_input(module.clone(), source_v1));
    assert!(!first.snapshot.has_errors(), "diagnostics: {:?}", first.snapshot.diagnostics);
    let root_surface_v1 = surface_fingerprint(&first, &owner("Choice"));
    let a_variant = phalcom_semantic::identity::VariantId::new(owner("Choice"), Selector::getter("A").unwrap());
    let a_target = phalcom_semantic::impls::InherentImplTarget::ExactEnumCase(a_variant);
    assert_eq!(first.snapshot.dispatch.get_conditional_members(&a_target).map(|set| set.members.len()), Some(1));

    // Add another case-only member to B
    let source_v2 = "enum Choice {\n  A\n  B\n}\nimpl Choice::A {\n  aOnly() -> Int { 42 }\n}\nimpl Choice::B {\n  bOnly() -> Int { 99 }\n}\n";
    let second = session.update(single_module_input(module.clone(), source_v2));
    assert!(!second.snapshot.has_errors(), "diagnostics: {:?}", second.snapshot.diagnostics);
    let root_surface_v2 = surface_fingerprint(&second, &owner("Choice"));
    let b_variant = phalcom_semantic::identity::VariantId::new(owner("Choice"), Selector::getter("B").unwrap());
    let b_target = phalcom_semantic::impls::InherentImplTarget::ExactEnumCase(b_variant);
    assert_eq!(
        second.snapshot.dispatch.get_conditional_members(&a_target).map(|set| set.members.len()),
        Some(1)
    );
    assert_eq!(
        second.snapshot.dispatch.get_conditional_members(&b_target).map(|set| set.members.len()),
        Some(1)
    );

    assert_eq!(
        root_surface_v1, root_surface_v2,
        "case-only members must not alter root enum declaration surface"
    );

    let third = session.update(single_module_input(module, "enum Choice {\n  A\n  B\n}\n"));
    assert!(
        !third.snapshot.has_errors(),
        "removal must not leave diagnostics: {:?}",
        third.snapshot.diagnostics
    );
    assert!(third.snapshot.dispatch.get_conditional_members(&a_target).is_none());
    assert!(third.snapshot.dispatch.get_conditional_members(&b_target).is_none());
}

#[test]
fn exact_case_conditional_precedence_shadows_root_requirement() {
    let module = test_module();
    let source = "enum Status { Ready Done }\nimpl Status { code() -> Int }\nimpl Status::Ready { code() -> Int { 1 } }\nimpl Status::Done { code() -> Int { 2 } }\nclass Probe { run() -> Int { Status::Ready.code() } }\n";
    let mut session = SemanticWorkspaceSession::new();
    let output = session.update(single_module_input(module.clone(), source));
    assert!(!output.snapshot.has_errors(), "diagnostics: {:?}", output.snapshot.diagnostics);

    let selection = output
        .snapshot
        .callable_analyses
        .values()
        .flat_map(|analysis| analysis.expressions.values())
        .find_map(|expression| expression.conditional_dispatch.as_ref())
        .expect("exact-case call must publish conditional dispatch selection");
    assert!(matches!(
        &selection.callable.owner,
        CallableOwnerId::Variant(variant) if variant.owner == DeclarationId::new(module, "Status".into())
    ));
    assert_eq!(selection.callable.selector, Selector::method("code", []).unwrap());
}
