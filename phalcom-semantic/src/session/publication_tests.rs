use super::{SemanticPublicationFailpoint, SemanticUpdateFailure, SemanticWorkspaceSession, retire_blocked_declaration_shell_queries};
use crate::db::{InputFingerprint, ProductFingerprint, QueryKey, QueryState, QueryValue, SemanticDb};
use crate::diagnostic::SemanticSourceSpan;
use crate::identity::{DeclarationId, WorkspaceId};
use crate::snapshot::SemanticSnapshot;
use crate::type_alias::{TypeAliasInfo, TypeAliasTable};
use crate::types::id::{KindId, TypeId};
use phalcom_modules::identity::{ModuleId, SourceId, SourceLocation};
use phalcom_modules::session::{SourceRevision, WorkspaceSourceBatchMutation};
use std::collections::BTreeSet;
use std::path::PathBuf;
use std::sync::Arc;

#[test]
fn blocked_nominal_shell_is_retired_while_alias_shell_remains_authoritative() {
    let module = ModuleId::universe_root();
    let nominal_owner = DeclarationId::new(module.clone(), "Nominal".into());
    let alias_owner = DeclarationId::new(module.clone(), "Alias".into());
    let nominal_key = QueryKey::DeclarationShell(nominal_owner.clone());
    let alias_key = QueryKey::DeclarationShell(alias_owner.clone());
    let mut db = SemanticDb::with_workspace(WorkspaceId::from_raw(153));
    let revision = db.revision();
    let mut type_aliases = TypeAliasTable::new();
    type_aliases.insert(TypeAliasInfo {
        declaration: alias_owner.clone(),
        kind: KindId::TYPE,
        kind_shape: "Type".into(),
        generic_signature: None,
        form: TypeId::DUMMY,
        structural_form: "Type".into(),
        dependencies: Box::new([]),
        source: SemanticSourceSpan::new(module, phalcom_common::range::SourceRange::default()),
    });

    for (key, fingerprint) in [(&nominal_key, 1), (&alias_key, 2)] {
        db.set_state(
            key.clone(),
            QueryState::Ready {
                revision,
                validated_revision: revision,
                input_fingerprint: InputFingerprint(fingerprint),
                product_fingerprint: ProductFingerprint(fingerprint),
                value: QueryValue::from_bytes([]),
            },
        );
    }

    retire_blocked_declaration_shell_queries(&mut db, &BTreeSet::from([nominal_owner, alias_owner]), &type_aliases);

    assert!(db.query_state(&nominal_key).is_none());
    assert!(db.query_state(&alias_key).is_some());
}

fn set_overlay(path: &str, revision: u64, text: &str) -> WorkspaceSourceBatchMutation {
    let display_path = PathBuf::from(path);
    WorkspaceSourceBatchMutation::SetOverlay {
        source: SourceLocation {
            source_id: SourceId(path.into()),
            display_path,
        },
        text: Arc::from(text),
        revision: SourceRevision(revision),
        recovered_program: None,
    }
}

fn remove_source(path: &str) -> WorkspaceSourceBatchMutation {
    WorkspaceSourceBatchMutation::RemoveSourceAt {
        source: SourceLocation {
            source_id: SourceId(path.into()),
            display_path: PathBuf::from(path),
        },
    }
}

#[derive(Debug, Eq, PartialEq)]
struct PublishedProjection {
    sources: Vec<(crate::identity::ModuleId, String)>,
    declarations: Vec<(crate::identity::DeclarationId, String)>,
    signatures: Vec<(crate::identity::CallableId, Vec<String>, String)>,
    diagnostics: Vec<(crate::identity::ModuleId, Vec<crate::diagnostic::SemanticDiagnostic>)>,
    formal_facts: Vec<crate::presentation::FormalFactSite>,
    source_sites: Vec<(
        crate::identity::SourceSiteId,
        phalcom_common::range::SourceRange,
        crate::source_index::SourceSiteKind,
    )>,
    static_values: Vec<(crate::static_constant::StaticConstantId, Option<u64>)>,
    linked_modules: Vec<crate::identity::ModuleId>,
}

fn published_projection(snapshot: &SemanticSnapshot) -> PublishedProjection {
    let mut sources = snapshot
        .sources
        .iter()
        .map(|(module, source)| (module.clone(), source.text.to_string()))
        .collect::<Vec<_>>();
    sources.sort_by_key(|(module, _)| module.clone());

    let mut declarations = snapshot
        .declarations
        .iter()
        .map(|(declaration, info)| (declaration.clone(), snapshot.store.format_type(info.form)))
        .collect::<Vec<_>>();
    declarations.sort_by_key(|(declaration, _)| declaration.clone());

    let mut signatures = snapshot
        .callable_signatures
        .iter()
        .map(|(callable, signature)| {
            let parameters = signature
                .parameters
                .iter()
                .map(|parameter| {
                    parameter
                        .declared_type
                        .canonical_type()
                        .map(|ty| snapshot.store.format_type(ty))
                        .unwrap_or_else(|| format!("{:?}", parameter.declared_type))
                })
                .collect();
            let result = signature
                .declared_return
                .canonical_type()
                .map(|ty| snapshot.store.format_type(ty))
                .unwrap_or_else(|| format!("{:?}", signature.declared_return));
            (callable.clone(), parameters, result)
        })
        .collect::<Vec<_>>();
    signatures.sort_by_key(|(callable, _, _)| callable.clone());

    let mut diagnostics = snapshot
        .diagnostics
        .iter()
        .map(|(module, diagnostics)| (module.clone(), diagnostics.to_vec()))
        .collect::<Vec<_>>();
    diagnostics.sort_by_key(|(module, _)| module.clone());

    let mut formal_facts = snapshot
        .formal_projection()
        .modules()
        .flat_map(|(_, module)| module.sites().iter().cloned())
        .collect::<Vec<_>>();
    formal_facts.sort_by_key(|fact| (fact.module.clone(), fact.range.start, fact.range.len(), fact.fact.clone()));

    let mut source_sites = snapshot
        .source_index()
        .modules()
        .flat_map(|(_, module)| module.structure.sites.values().map(|site| (site.id.clone(), site.range, site.kind.clone())))
        .collect::<Vec<_>>();
    source_sites.sort_by_key(|(site, _, _)| site.clone());

    let mut static_values = snapshot
        .static_constants
        .iter()
        .map(|(id, product)| {
            let fingerprint = product.value().and_then(|value| snapshot.static_values().content_fingerprint(value).ok());
            (id.clone(), fingerprint)
        })
        .collect::<Vec<_>>();
    static_values.sort_by_key(|(id, _)| id.clone());

    let mut linked_modules = snapshot.module_products.linked.keys().cloned().collect::<Vec<_>>();
    linked_modules.sort();

    PublishedProjection {
        sources,
        declarations,
        signatures,
        diagnostics,
        formal_facts,
        source_sites,
        static_values,
        linked_modules,
    }
}

#[test]
fn failed_fresh_publication_is_not_returned_as_authoritative_success() {
    let mut session = SemanticWorkspaceSession::new();
    *session.test_publication_failpoint.lock().unwrap() = Some(SemanticPublicationFailpoint::AfterRevisionBegin);

    let failed_attempt = session.apply_module_mutations([set_overlay("/typeaud001/fresh.ph", 1, "const value = 1\n")]);
    assert!(failed_attempt.is_err(), "a failed first semantic attempt must be returned as an explicit error");
    assert!(
        session.last_snapshot().is_none(),
        "a failed first semantic attempt must not create an authoritative snapshot"
    );
}

#[test]
fn generated_delegation_definition_retains_body_authority_without_analysis() {
    let mut session = SemanticWorkspaceSession::new();
    let update = session
        .apply_module_mutations([set_overlay(
            "/typeaud001/delegation-provenance.ph",
            1,
            "class Counter { mut _count: Int }\nimpl Counter { mut count via _count }\n",
        )])
        .expect("delegation source should publish");
    assert!(!update.snapshot.has_errors(), "diagnostics: {:?}", update.snapshot.diagnostics);

    let generated = update
        .snapshot
        .callable_definitions
        .iter()
        .filter(|(_, definition)| {
            matches!(
                definition.body_provenance,
                crate::impls::CallableDefinitionBodyProvenance::GeneratedDelegation { .. }
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(generated.len(), 2, "delegated property publishes its getter and setter definitions");
    for (callable, definition) in generated {
        assert!(matches!(
            &definition.body_provenance,
            crate::impls::CallableDefinitionBodyProvenance::GeneratedDelegation { target_field }
                if target_field.as_ref() == "_count"
        ));
        assert!(
            !update.snapshot.callable_analyses.contains_key(callable),
            "body provenance authorizes generated accessors without a CallableAnalysis"
        );
    }
}

#[test]
fn generated_trait_property_accessors_share_source_member_index() {
    let mut session = SemanticWorkspaceSession::new();
    let update = session
        .apply_module_mutations([set_overlay(
            "/typeaud001/trait-property-source-ordinal.ph",
            1,
            "trait PropertySourceOrdinal { mut count: Int }\ntrait DiagnosticSurface {\n  item: Int\n  item: Int\n}\n",
        )])
        .expect("trait property source should publish");

    let (declaration, surface) = update
        .snapshot
        .trait_surfaces
        .iter_local()
        .find(|(declaration, _)| declaration.name.as_ref() == "PropertySourceOrdinal")
        .expect("trait surface should be published");
    assert_eq!(surface.declaration, *declaration);
    assert_eq!(surface.members.len(), 2, "mutable property publishes getter and setter requirements");
    assert!(
        surface.members.values().all(|member| member.source_member_index == 0),
        "both generated accessors belong to the property's single source member"
    );
    assert!(surface.diagnostics.is_empty());

    let diagnostic_surface = update
        .snapshot
        .trait_surfaces
        .iter_local()
        .find(|(declaration, _)| declaration.name.as_ref() == "DiagnosticSurface")
        .map(|(_, surface)| surface)
        .expect("diagnostic trait surface should be published");
    assert!(diagnostic_surface.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == crate::diagnostic::DiagnosticCode::TraitMemberConflict && diagnostic.message.contains("duplicate trait member selector")
    }));
}

#[test]
fn failed_module_update_retains_its_pending_semantic_delta() {
    let mut session = SemanticWorkspaceSession::new();
    session
        .apply_module_mutations([
            set_overlay("/typeaud001/retry-a.ph", 1, "const value = 1\n"),
            set_overlay("/typeaud001/retry-b.ph", 1, "const value = 10\n"),
        ])
        .expect("initial module and semantic publication should succeed");

    *session.test_publication_failpoint.lock().unwrap() = Some(SemanticPublicationFailpoint::AfterSourceFingerprints);
    let first_failed_attempt = session.apply_module_mutations([set_overlay("/typeaud001/retry-a.ph", 2, "const value = 2\n")]);

    *session.test_publication_failpoint.lock().unwrap() = Some(SemanticPublicationFailpoint::AfterSourceFingerprints);
    let second_failed_attempt = session.apply_module_mutations([set_overlay("/typeaud001/retry-b.ph", 2, "const value = 20\n")]);

    assert!(
        session.pending_semantic_delta.as_ref().is_some_and(|delta| delta.changed_modules.len() == 2),
        "both accepted module changes must remain pending across repeated failures"
    );
    assert!(first_failed_attempt.is_err(), "semantic failure must be returned to the caller");
    assert!(second_failed_attempt.is_err(), "second semantic failure must also be returned to the caller");

    let retried = session
        .apply_module_mutations([])
        .expect("an empty module batch retries the accumulated semantic work");
    assert!(
        session.pending_semantic_delta.is_none(),
        "pending deltas clear only after successful publication"
    );
    assert_eq!(retried.snapshot.sources.len(), 2);
    assert!(retried.snapshot.sources.values().any(|source| source.text.contains("const value = 2")));
    assert!(retried.snapshot.sources.values().any(|source| source.text.contains("const value = 20")));
}

#[test]
fn budget_exhaustion_retains_all_module_changes_for_retry() {
    use crate::db::{CancellationToken, QueryBudget};

    let provider_path = "/typeaud001/budget-provider.ph";
    let consumer_path = "/typeaud001/budget-consumer.ph";
    let mut session = SemanticWorkspaceSession::new();
    session
        .apply_module_mutations([
            set_overlay(provider_path, 1, "class Provider { value() -> Int { 1 } }\n"),
            set_overlay(consumer_path, 1, "class Consumer { value() -> Int { 2 } }\n"),
        ])
        .expect("initial modules should publish");

    let update = session
        .module_session_mut()
        .apply_batch([
            set_overlay(provider_path, 2, "class Provider { value() -> Int { 10 } }\n"),
            set_overlay(consumer_path, 2, "class Consumer { value() -> Int { 20 } }\n"),
        ])
        .expect("module sources commit before semantic publication");
    let failed = session.update_module_workspace_with_budget_and_cancel(
        update,
        QueryBudget::new(0),
        crate::attribute_analysis::AttributeExecutionMode::Debug,
        &CancellationToken::new(),
    );

    assert!(matches!(failed, Err(SemanticUpdateFailure::BudgetExceeded(_))));
    assert!(session.pending_semantic_delta.as_ref().is_some_and(|delta| delta.changed_modules.len() == 2));

    let retried = session
        .apply_module_mutations([])
        .expect("default-budget empty retry publishes both accepted edits");
    assert!(session.pending_semantic_delta.is_none());
    assert!(retried.snapshot.sources.values().any(|source| source.text.contains("value() -> Int { 10 }")));
    assert!(retried.snapshot.sources.values().any(|source| source.text.contains("value() -> Int { 20 }")));
}

#[test]
fn operational_query_failures_keep_distinct_public_error_categories() {
    use crate::db::{BudgetKind, BudgetReport, QueryOutcome};
    use crate::types::outcome::BlockReason;

    assert_eq!(SemanticUpdateFailure::from(QueryOutcome::Cancelled), SemanticUpdateFailure::Cancelled);
    assert_eq!(
        SemanticUpdateFailure::from(QueryOutcome::BudgetExceeded(BudgetReport::new(BudgetKind::Steps, 1, 2))),
        SemanticUpdateFailure::BudgetExceeded(BudgetReport::new(BudgetKind::Steps, 1, 2)),
    );
    assert_eq!(
        SemanticUpdateFailure::from(QueryOutcome::Blocked(BlockReason::RecursiveFixpoint)),
        SemanticUpdateFailure::Blocked(Box::new(BlockReason::RecursiveFixpoint)),
    );
    assert_eq!(
        SemanticUpdateFailure::from(QueryOutcome::Failed("fixture failure".into())),
        SemanticUpdateFailure::Failed("fixture failure".into()),
    );
}

#[test]
fn failure_before_commit_is_returned_and_retains_last_published_snapshot() {
    let mut session = SemanticWorkspaceSession::new();
    let initial = session
        .apply_module_mutations([set_overlay("/typeaud001/precommit.ph", 1, "const value = 1\n")])
        .expect("initial module and semantic publication should succeed");

    *session.test_publication_failpoint.lock().unwrap() = Some(SemanticPublicationFailpoint::BeforeCommit);
    let failed_attempt = session.apply_module_mutations([set_overlay("/typeaud001/precommit.ph", 2, "const value = 2\n")]);

    assert!(failed_attempt.is_err(), "failure before the semantic commit must not return success");
    assert!(
        session.last_snapshot().is_some_and(|snapshot| Arc::ptr_eq(snapshot, &initial.snapshot)),
        "the last authoritative snapshot must remain the one from the successful generation"
    );
}

#[test]
fn cancellation_at_the_commit_boundary_keeps_the_previous_publication() {
    use crate::db::CancellationToken;

    let path = "/typeaud001/cancel-before-commit.ph";
    let mut session = SemanticWorkspaceSession::new();
    let initial = session
        .apply_module_mutations([set_overlay(path, 1, "const value = 1\n")])
        .expect("initial module and semantic publication should succeed");
    let update = session
        .module_session_mut()
        .apply_batch([set_overlay(path, 2, "const value = 2\n")])
        .expect("source update commits before semantic publication");
    let cancel = CancellationToken::new();
    *session.test_publication_failpoint.lock().unwrap() = Some(SemanticPublicationFailpoint::CancelBeforeCommit);

    let failed = session.update_module_workspace_with_cancel(update, &cancel);

    assert!(matches!(failed, Err(SemanticUpdateFailure::Cancelled)));
    assert!(cancel.is_cancelled());
    assert!(session.last_snapshot().is_some_and(|snapshot| Arc::ptr_eq(snapshot, &initial.snapshot)));
    assert!(session.pending_semantic_delta.as_ref().is_some_and(|delta| !delta.changed_modules.is_empty()));
}

#[test]
fn failure_after_revision_begin_is_returned_without_replacing_last_snapshot() {
    let mut session = SemanticWorkspaceSession::new();
    let initial = session
        .apply_module_mutations([set_overlay("/typeaud001/revision.ph", 1, "const value = 1\n")])
        .expect("initial module and semantic publication should succeed");

    *session.test_publication_failpoint.lock().unwrap() = Some(SemanticPublicationFailpoint::AfterRevisionBegin);
    let failed_attempt = session.apply_module_mutations([set_overlay("/typeaud001/revision.ph", 2, "const value = 2\n")]);

    assert!(failed_attempt.is_err(), "the injected DB revision failure must be returned");
    assert!(
        session.last_snapshot().is_some_and(|snapshot| Arc::ptr_eq(snapshot, &initial.snapshot)),
        "a DB revision begun by a failed attempt must not replace the last published snapshot"
    );
}

#[test]
fn candidate_type_parameter_interning_preserves_retained_snapshot_provenance() {
    let mut session = SemanticWorkspaceSession::new();
    let initial = session
        .apply_module_mutations([set_overlay("/typeaud001/provenance.ph", 1, "class Holder<T> {}\n")])
        .expect("initial generic module should publish");
    let module = initial.snapshot.sources.keys().next().expect("module source exists").clone();
    let declaration = crate::identity::DeclarationId::new(module.clone(), "Holder".into());
    let parameter = initial
        .snapshot
        .declarations
        .generic_signature(&declaration)
        .expect("generic signature exists")
        .parameters[0];
    let original_source = session.store.type_parameter(parameter).source.clone().expect("generic parameter source exists");
    let old_store = initial.snapshot.store.clone();
    let candidate_source = crate::diagnostic::SemanticSourceSpan::new(module.clone(), phalcom_common::range::SourceRange::new(31, 32));
    let mut candidate_parameter = session.store.type_parameter(parameter).clone();
    candidate_parameter.source = Some(candidate_source.clone());
    *session.test_candidate_type_parameter.lock().unwrap() = Some(candidate_parameter.clone());

    *session.test_publication_failpoint.lock().unwrap() = Some(SemanticPublicationFailpoint::BeforeCommit);
    let failed = session.apply_module_mutations([set_overlay("/typeaud001/provenance.ph", 2, "class Holder<T> {}\n")]);

    assert!(failed.is_err(), "the candidate must fail before committing its provenance refresh");
    assert!(Arc::ptr_eq(&session.store, &old_store), "failure restores the committed store root");
    assert_eq!(session.store.type_parameter(parameter).source.as_ref(), Some(&original_source));
    assert_eq!(initial.snapshot.store.type_parameter(parameter).source.as_ref(), Some(&original_source));

    let committed_source = crate::diagnostic::SemanticSourceSpan::new(module, phalcom_common::range::SourceRange::new(51, 52));
    candidate_parameter.source = Some(committed_source.clone());
    *session.test_candidate_type_parameter.lock().unwrap() = Some(candidate_parameter);
    let updated = session
        .apply_module_mutations([set_overlay("/typeaud001/provenance.ph", 3, "class Holder<T> {}\n")])
        .expect("a later candidate should commit");
    let updated_parameter = updated
        .snapshot
        .declarations
        .generic_signature(&declaration)
        .expect("updated generic signature exists")
        .parameters[0];
    let updated_source = updated
        .snapshot
        .store
        .type_parameter(updated_parameter)
        .source
        .as_ref()
        .expect("updated source exists");

    assert_eq!(updated_parameter, parameter, "source-only movement retains the canonical parameter ID");
    assert_eq!(updated_source, &committed_source, "committed candidate provenance is installed");
    assert_ne!(candidate_source.range, original_source.range);
    assert_eq!(initial.snapshot.store.type_parameter(parameter).source.as_ref(), Some(&original_source));
}

#[test]
fn failed_static_value_candidate_restores_the_committed_store_root() {
    let mut session = SemanticWorkspaceSession::new();
    let initial = session
        .apply_module_mutations([set_overlay("/typeaud001/static-root.ph", 1, "const value = 1\n")])
        .expect("initial static constant should publish");
    let committed_values = initial.snapshot.static_values.clone();
    let committed_len = committed_values.len();

    *session.test_publication_failpoint.lock().unwrap() = Some(SemanticPublicationFailpoint::AfterStaticValueMutation);
    let failed = session.apply_module_mutations([set_overlay("/typeaud001/static-root.ph", 2, "const value = 2\n")]);

    assert!(failed.is_err(), "the post-evaluation failure must be returned");
    assert!(
        Arc::ptr_eq(&session.static_values, &committed_values),
        "failure restores the committed static-value root"
    );
    assert_eq!(session.static_values.len(), committed_len);
    assert!(session.last_snapshot().is_some_and(|snapshot| Arc::ptr_eq(snapshot, &initial.snapshot)));
}

#[test]
fn publication_phase_failures_restore_all_authoritative_session_roots() {
    let mut session = SemanticWorkspaceSession::new();
    let initial = session
        .apply_module_mutations([set_overlay("/typeaud001/phase-roots.ph", 1, "class Holder<T> {}\nconst value = 1\n")])
        .expect("initial module should publish");
    let module = initial.snapshot.sources.keys().next().expect("module source exists").clone();
    let old_shard = session.semantic_structure_shards.get(&module).expect("source shard exists").clone();
    let old_store = initial.snapshot.store.clone();
    let old_static_values = initial.snapshot.static_values.clone();
    let old_revision = session.db.revision();
    let old_source = session.sources[&module].text.clone();
    let old_fingerprint = session.source_fingerprints[&module];
    let old_query_states = session
        .db
        .query_keys()
        .map(|key| (key.clone(), session.db.query_state(key).expect("listed query has state").clone()))
        .collect::<std::collections::BTreeMap<_, _>>();
    let phases = [
        SemanticPublicationFailpoint::AfterStructuralShards,
        SemanticPublicationFailpoint::AfterQueryRetirement,
        SemanticPublicationFailpoint::AfterSourceFingerprints,
        SemanticPublicationFailpoint::AfterStaticValueMutation,
        SemanticPublicationFailpoint::AfterReadyQueryPublish,
        SemanticPublicationFailpoint::BeforeCommit,
    ];

    for (index, phase) in phases.into_iter().enumerate() {
        let revision = index as u64 + 2;
        *session.test_publication_failpoint.lock().unwrap() = Some(phase);
        let failed = session.apply_module_mutations([set_overlay("/typeaud001/phase-roots.ph", revision, &format!("const value = {}\n", revision))]);

        assert!(failed.is_err(), "{phase:?} must abort publication");
        assert!(session.last_snapshot().is_some_and(|snapshot| Arc::ptr_eq(snapshot, &initial.snapshot)));
        assert_eq!(session.db.revision(), old_revision, "{phase:?} must restore the DB revision");
        assert!(Arc::ptr_eq(&session.store, &old_store), "{phase:?} must restore the TypeStore root");
        assert!(
            Arc::ptr_eq(&session.static_values, &old_static_values),
            "{phase:?} must restore the static-value root"
        );
        assert!(Arc::ptr_eq(
            session.semantic_structure_shards.get(&module).expect("old shard restored"),
            &old_shard
        ));
        assert_eq!(session.sources[&module].text, old_source);
        assert_eq!(session.source_fingerprints[&module], old_fingerprint);
        let query_states = session
            .db
            .query_keys()
            .map(|key| (key.clone(), session.db.query_state(key).expect("listed query has state").clone()))
            .collect::<std::collections::BTreeMap<_, _>>();
        assert_eq!(query_states, old_query_states, "{phase:?} must restore query authority");
    }
}

#[test]
fn removed_module_is_not_resurrected_after_failed_remove_and_readd() {
    let path = "/typeaud001/remove-readd.ph";
    let mut session = SemanticWorkspaceSession::new();
    let initial = session
        .apply_module_mutations([set_overlay(path, 1, "class Old {}\n")])
        .expect("initial module should publish");
    let old_module = initial.snapshot.sources.keys().next().expect("initial source exists").clone();
    let old_declaration = crate::identity::DeclarationId::new(old_module.clone(), "Old".into());
    let old_query = crate::db::QueryKey::DeclarationShell(old_declaration);

    *session.test_publication_failpoint.lock().unwrap() = Some(SemanticPublicationFailpoint::AfterQueryRetirement);
    let failed_remove = session.apply_module_mutations([remove_source(path)]);
    assert!(failed_remove.is_err(), "module removal must remain uncommitted semantically on failure");
    assert!(
        session
            .pending_semantic_delta
            .as_ref()
            .is_some_and(|delta| delta.removed_modules.contains(&old_module))
    );

    let readded = session
        .apply_module_mutations([set_overlay(path, 2, "class Reborn {}\n")])
        .expect("re-added module should publish the accumulated lifecycle delta");

    assert!(session.pending_semantic_delta.is_none());
    assert_eq!(readded.snapshot.sources.len(), 1);
    assert!(readded.snapshot.sources.values().any(|source| source.text.contains("class Reborn")));
    assert!(
        session.db.last_known_good_product(&old_query).is_none(),
        "removed declaration LKG must not reappear"
    );
    assert!(
        readded
            .snapshot
            .declarations
            .get(&crate::identity::DeclarationId::new(old_module, "Old".into()))
            .is_none()
    );
}

#[test]
fn repeated_failed_module_updates_retry_to_cold_equivalent_publication() {
    let provider_path = "/typeaud001/cold-provider.ph";
    let consumer_path = "/typeaud001/cold-consumer.ph";
    let mut incremental = SemanticWorkspaceSession::new();
    let initial = incremental
        .apply_module_mutations([
            set_overlay(provider_path, 1, "class Provider { @class value() -> Int { 1 } }\nconst providerValue = 1\n"),
            set_overlay(consumer_path, 1, "class Consumer { run() -> Int { 2 } }\nconst consumerValue = 2\n"),
        ])
        .expect("initial provider and consumer sources should publish");

    *incremental.test_publication_failpoint.lock().unwrap() = Some(SemanticPublicationFailpoint::AfterReadyQueryPublish);
    let first = incremental.apply_module_mutations([set_overlay(
        provider_path,
        2,
        "class Provider { @class value() -> Int { 10 } }\nconst providerValue = 10\n",
    )]);
    *incremental.test_publication_failpoint.lock().unwrap() = Some(SemanticPublicationFailpoint::AfterSourceFingerprints);
    let second = incremental.apply_module_mutations([set_overlay(
        consumer_path,
        2,
        "class Consumer { run() -> Int { 20 } }\nconst consumerValue = 20\n",
    )]);

    assert!(first.is_err() && second.is_err(), "both candidate publications must fail explicitly");
    assert!(incremental.last_snapshot().is_some_and(|snapshot| Arc::ptr_eq(snapshot, &initial.snapshot)));

    let retried = incremental
        .apply_module_mutations([])
        .expect("an empty module batch must publish all accumulated edits");
    assert!(incremental.pending_semantic_delta.is_none());

    let cold_mutations = incremental
        .module_session()
        .sources()
        .values()
        .map(|source| WorkspaceSourceBatchMutation::SetOverlayForModule {
            module: source.module.clone(),
            source: source.location.clone(),
            text: source.text.clone(),
            revision: source.revision,
            recovered_program: Some(source.parsed.program.clone()),
        })
        .collect::<Vec<_>>();
    let mut cold = SemanticWorkspaceSession::new();
    let cold_publication = cold.apply_module_mutations(cold_mutations).expect("fresh analysis of latest accepted sources");

    assert_eq!(
        published_projection(&retried.snapshot),
        published_projection(&cold_publication.snapshot),
        "retry must match a fresh analysis of the latest accepted module sources",
    );
}

#[test]
fn deep_analysis_frontier_survives_failure_and_is_cleared_after_retry() {
    let path = "/typeaud001/deep-frontier.ph";
    let mut session = SemanticWorkspaceSession::new();
    let initial = session
        .apply_module_mutations([set_overlay(path, 1, "class Worker { run() -> Int { 1 } }\n")])
        .expect("initial module should publish");
    let module = initial.snapshot.sources.keys().next().expect("source module exists").clone();
    *session.test_publication_failpoint.lock().unwrap() = Some(SemanticPublicationFailpoint::BeforeCommit);

    let failed = session.apply_module_mutations_with_deep_modules(
        [set_overlay(path, 2, "class Worker { run() -> Int { 2 } }\n")],
        std::collections::BTreeSet::from([module.clone()]),
    );

    assert!(failed.is_err());
    assert_eq!(session.requested_deep_modules, Some(std::collections::BTreeSet::from([module])));
    let retried = session.apply_module_mutations([]).expect("empty-batch retry preserves requested deep work");
    assert!(session.requested_deep_modules.is_none());
    assert_eq!(
        retried.snapshot.sources.values().next().unwrap().text.as_ref(),
        "class Worker { run() -> Int { 2 } }\n"
    );
}
