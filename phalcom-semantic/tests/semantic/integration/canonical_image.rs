use phalcom_semantic::canonical_image::{CanonicalUniverseFormalImage, canonical_universe_formal_image};

#[test]
fn canonical_formal_image_is_complete_and_process_shared() {
    let start = std::time::Instant::now();
    let barrier = std::sync::Arc::new(std::sync::Barrier::new(8));
    let callers = (0..8)
        .map(|_| {
            let barrier = barrier.clone();
            std::thread::spawn(move || {
                barrier.wait();
                canonical_universe_formal_image().expect("concurrent first access").clone()
            })
        })
        .collect::<Vec<_>>();
    let images = callers.into_iter().map(|caller| caller.join().unwrap()).collect::<Vec<_>>();
    assert!(images.iter().all(|image| std::sync::Arc::ptr_eq(image, &images[0])));
    let first = canonical_universe_formal_image().expect("canonical Universe formal image");
    let cold = start.elapsed();
    let start = std::time::Instant::now();
    for _ in 0..10 {
        assert!(std::sync::Arc::ptr_eq(first, canonical_universe_formal_image().unwrap()));
        let reader = first.reader().unwrap();
        let key = first.query_image().entries().next().unwrap().0;
        assert!(reader.product(key).is_some());
    }
    eprintln!(
        "canonical image: cold concurrent {:?}; ten warm readers {:?}; sources {}; queries {}; rejected {}; final types {}; incidents {}",
        cold,
        start.elapsed(),
        first.sources().len(),
        first.query_image().len(),
        first.query_image().census().rejections.len(),
        first.snapshot().store.len(),
        first.snapshot().internal_incidents.len()
    );
    assert_eq!(phalcom_semantic::canonical_image::canonical_universe_complete_update_count(), 1);
    assert!(first.snapshot().status.is_complete());
    let fresh = phalcom_semantic::SemanticWorkspaceSession::new();
    assert!(fresh.last_snapshot().is_none());
    assert!(fresh.last_known_good_snapshot().is_none());
    let second = canonical_universe_formal_image().expect("canonical Universe formal image");

    assert!(std::ptr::eq(first, second));
    assert_eq!(phalcom_semantic::canonical_image::canonical_universe_formal_image_build_count(), 1);
    assert_eq!(first.sources().len(), first.interfaces().len());
    assert_eq!(first.sources().len(), first.linked().modules.len());
    assert!(first.sources().keys().all(|module| module.project.is_universe()));
    assert!(first.sources().keys().all(|module| first.snapshot().sources.contains_key(module)));
    assert!(first.query_image().len() > first.sources().len());
    assert!(
        first
            .query_image()
            .entries()
            .all(|(key, entry)| entry.dependencies.iter().all(|dependency| dependency.dependent == *key))
    );
}

#[test]
fn canonical_formal_image_is_send_and_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<CanonicalUniverseFormalImage>();
}

#[test]
fn canonical_formal_image_source_parity_and_provider_isolation() {
    use phalcom_modules::{ModuleComponent, ModuleId, ModulePath, UniverseSourceProvider};
    use phalcom_semantic::canonical_image::canonical_universe_formal_image_with_provider;
    use std::sync::Arc;
    let image = canonical_universe_formal_image().unwrap();
    let input = phalcom_semantic::SemanticWorkspaceInput::new(image.linked().clone(), image.sources().clone(), 0)
        .with_interfaces(image.interfaces().as_source().expect("source image interfaces").clone());
    let reference = phalcom_semantic::analyze_workspace(input).unwrap().snapshot;
    assert_eq!(reference.status, image.snapshot().status);
    assert_eq!(reference.callable_signatures.len(), image.snapshot().callable_signatures.len());
    assert_eq!(reference.callable_analyses.len(), image.snapshot().callable_analyses.len());
    assert_eq!(reference.conformance_witness_plans.len(), image.snapshot().conformance_witness_plans.len());
    // Store IDs and allocation order may differ across equivalent cold analyses.
    for (id, declaration) in reference.declarations.iter() {
        let expected = image.snapshot().declarations.get(id).expect("same declaration identity");
        assert_eq!(reference.store.format_type(declaration.form), image.snapshot().store.format_type(expected.form));
        assert_eq!(reference.store.format_kind(declaration.kind), image.snapshot().store.format_kind(expected.kind));
    }
    for (id, signature) in reference.callable_signatures.iter() {
        let expected = image.snapshot().callable_signatures.get(id).expect("same callable identity");
        assert_eq!(signature.selector, expected.selector);
        assert_eq!(signature.side, expected.side);
        assert_eq!(signature.parameters.len(), expected.parameters.len());
        assert_eq!(signature.return_validation, expected.return_validation);
        assert_eq!(
            signature.declared_return.canonical_type().map(|ty| reference.store.format_type(ty)),
            expected.declared_return.canonical_type().map(|ty| image.snapshot().store.format_type(ty))
        );
    }
    for (module, diagnostics) in reference.diagnostics.iter() {
        let expected = &image.snapshot().diagnostics[module];
        assert_eq!(
            diagnostics.iter().map(|d| (&d.code, &d.message)).collect::<Vec<_>>(),
            expected.iter().map(|d| (&d.code, &d.message)).collect::<Vec<_>>()
        );
    }
    let module = ModuleId::universe(ModulePath::from_components(vec![
        ModuleComponent::from_identifier("math").unwrap(),
        ModuleComponent::from_identifier("numerical_errors").unwrap(),
    ]));
    let provider = UniverseSourceProvider::new();
    let text = format!("{}\nclass ImageOnlyMarker {{}}\n", provider.source_text(&module).unwrap());
    let override_provider = UniverseSourceProvider::with_test_source_override(module.clone(), Arc::from(text)).unwrap();
    let private = canonical_universe_formal_image_with_provider(&override_provider).unwrap();
    assert_ne!(private.world(), image.world());
    assert!(!Arc::ptr_eq(&private, image));
    let marker = phalcom_semantic::identity::DeclarationId::new(module.clone(), "ImageOnlyMarker".into());
    assert!(private.snapshot().declarations.get(&marker).is_some());
    assert!(image.snapshot().declarations.get(&marker).is_none());
    let broken = UniverseSourceProvider::with_test_source_override(module, Arc::from("class {")).unwrap();
    assert!(canonical_universe_formal_image_with_provider(&broken).is_err());
    assert!(Arc::ptr_eq(image, canonical_universe_formal_image().unwrap()));
    assert_eq!(phalcom_semantic::canonical_image::canonical_universe_formal_image_build_count(), 1);
}

#[test]
fn ordinary_image_session_keeps_universe_external_across_replacement() {
    use phalcom_modules::{ModuleId, ModulePath, SourceId, SourceLocation, SourceRevision, SyntheticProjectIdAllocator, WorkspaceSourceBatchMutation};
    use phalcom_semantic::{SemanticWorkspaceSession, identity::WorkspaceId};
    use std::sync::Arc;
    let image = canonical_universe_formal_image().unwrap();
    let mut ids = SyntheticProjectIdAllocator;
    let local = ModuleId::synthetic(ids.allocate(), ModulePath::root());
    let mut session = SemanticWorkspaceSession::from_image(WorkspaceId::from_raw(733), image.clone());
    let assert_canonical_queries_external = |session: &SemanticWorkspaceSession| {
        for (key, _) in image.query_image().entries() {
            assert!(
                session.db().local_query_state(key).is_none(),
                "canonical query {key:?} must not be copied into local session state"
            );
            assert_eq!(
                session.db().product_origin(key),
                Some(phalcom_semantic::db::SemanticProductOrigin::Canonical(image.world().clone())),
                "canonical query {key:?} must retain image ownership"
            );
        }
    };
    assert_canonical_queries_external(&session);
    assert!(Arc::ptr_eq(session.last_snapshot().expect("initial image snapshot"), image.snapshot()));
    assert!(Arc::ptr_eq(
        session.last_known_good_snapshot().expect("initial image last-known-good snapshot"),
        image.snapshot()
    ));
    assert!(image.snapshot().type_aliases.iter().next().is_some());
    assert!(image.snapshot().static_definitions.iter().next().is_some());
    assert!(image.snapshot().static_constants.iter().next().is_some());
    assert!(image.snapshot().callable_definitions.iter().next().is_some());
    assert!(image.snapshot().module_static_bindings.iter().next().is_some());
    assert!(image.snapshot().constructor_initializations.iter().next().is_some());
    assert!(image.snapshot().trait_headers.iter().next().is_some());
    assert!(image.snapshot().trait_surfaces.iter().next().is_some());
    assert!(image.snapshot().derived_conformance_rules.iter().next().is_some());
    assert!(image.snapshot().conformance_index.iter().next().is_some());
    for (revision, source) in [
        "from universe.random import Xoshiro256PlusPlus\nlet value = Xoshiro256PlusPlus.seed(42).nextU64\n",
        "from universe.random import Xoshiro256PlusPlus\nlet value = Xoshiro256PlusPlus.seed(7).nextU64\n",
        "let replacement = 2\n",
    ]
    .into_iter()
    .enumerate()
    {
        let started = std::time::Instant::now();
        let update = session
            .apply_module_mutations([WorkspaceSourceBatchMutation::SetOverlayForModule {
                module: local.clone(),
                source: SourceLocation {
                    source_id: SourceId("image-overlay.ph".into()),
                    display_path: "/tmp/image-overlay.ph".into(),
                },
                text: Arc::from(source),
                revision: SourceRevision(revision as u64 + 1),
                recovered_program: None,
            }])
            .unwrap();
        eprintln!("local image update {revision}: {:?}; stats {:?}", started.elapsed(), update.stats);
        assert_canonical_queries_external(&session);
        assert_eq!(update.snapshot.sources.len(), 1);
        assert!(update.snapshot.sources.contains_key(&local));
        assert!(update.snapshot.semantic_structure_shards.keys().all(|module| !module.project.is_universe()));
        assert!(session.module_session().sources().keys().all(|module| !module.project.is_universe()));
        assert!(session.module_session().linked_modules().keys().all(|module| !module.project.is_universe()));
        assert!(
            update.snapshot.diagnostics.get(&local).is_none_or(|diagnostics| diagnostics.is_empty()),
            "local overlay diagnostics: {:?}",
            update.snapshot.diagnostics.get(&local)
        );
        assert!(update.snapshot.internal_incidents.is_empty(), "{:?}", update.snapshot.internal_incidents);
        assert_eq!(
            update.snapshot.conformance_witness_plans.len(),
            image.snapshot().conformance_witness_plans.len()
        );
        // Canonical conformance heads remain in the same immutable parent
        // throughout all three local publication revisions.
        assert_eq!(
            update.snapshot.derived_conformance_rules.local_len(),
            0,
            "unexpected local rules: {:?}",
            update
                .snapshot
                .derived_conformance_rules
                .iter()
                .map(|rule| (
                    &rule.impl_id,
                    &rule.source_module,
                    &rule.trait_ref.declaration,
                    rule.target_parameter,
                    rule.target_template,
                    image.snapshot().derived_conformance_rules.get_rule(&rule.impl_id).is_some(),
                ))
                .collect::<Vec<_>>()
        );
        assert_eq!(
            update.snapshot.derived_conformance_rules.parent_len(),
            image.snapshot().derived_conformance_rules.len()
        );
        for rule in image.snapshot().derived_conformance_rules.iter() {
            assert!(std::ptr::eq(
                update
                    .snapshot
                    .derived_conformance_rules
                    .get_rule(&rule.impl_id)
                    .expect("canonical derived rule"),
                rule
            ));
        }
        assert_eq!(
            update.snapshot.conformance_index.iter().count(),
            image.snapshot().conformance_index.iter().count()
        );
        assert_eq!(
            update.snapshot.conformance_index.iter().count(),
            image.snapshot().conformance_index.iter().count()
        );
        for (impl_id, contribution) in image.snapshot().conformance_index.iter() {
            assert!(std::ptr::eq(
                update.snapshot.conformance_index.get(impl_id).expect("canonical conformance head"),
                contribution
            ));
        }
        assert_eq!(update.snapshot.surfaces.local_len(), 0);
        assert_eq!(update.snapshot.data_semantics.data_decls.local_len(), 0);
        assert_eq!(update.snapshot.enum_semantics.enums.local_len(), 0);
        assert_eq!(update.snapshot.associated_surfaces.surfaces.local_len(), 0);
        assert_eq!(
            update.snapshot.enum_requirements.requirements.len(),
            image.snapshot().enum_requirements.requirements.len()
        );
        assert_eq!(update.snapshot.type_member_declarations.local_len(), 0);
        assert_eq!(update.snapshot.type_aliases.local_len(), 0);
        assert_eq!(update.snapshot.static_definitions.local_len(), 0);
        assert_eq!(update.snapshot.callable_definitions.local_len(), 0);
        assert_eq!(update.snapshot.module_static_bindings.local_len(), 1);
        assert_eq!(
            update.snapshot.module_static_bindings.parent_len(),
            image.snapshot().module_static_bindings.len()
        );
        assert_eq!(update.snapshot.constructor_initializations.local_len(), 0);
        assert_eq!(
            update.snapshot.constructor_initializations.parent_len(),
            image.snapshot().constructor_initializations.iter().count()
        );
        assert_eq!(update.snapshot.trait_headers.local_len(), 0);
        assert_eq!(update.snapshot.trait_headers.parent_len(), image.snapshot().trait_headers.iter().count());
        assert_eq!(update.snapshot.trait_surfaces.local_len(), 0);
        assert_eq!(update.snapshot.trait_surfaces.parent_len(), image.snapshot().trait_surfaces.iter().count());
        assert_eq!(
            update.snapshot.static_constants.local_len(),
            update
                .snapshot
                .static_constants
                .iter()
                .filter(|(id, _)| !id.module().project.is_universe())
                .count()
        );
        for (id, surface) in image.snapshot().dispatch.surfaces().iter() {
            assert!(std::ptr::eq(update.snapshot.dispatch.get_surface(id).unwrap(), surface));
            assert!(std::ptr::eq(update.snapshot.surfaces.get(id).unwrap(), surface));
        }
        for (id, alias) in image.snapshot().type_aliases.iter() {
            assert!(std::ptr::eq(update.snapshot.type_aliases.get(id).unwrap(), alias));
        }
        for (id, definition) in image.snapshot().static_definitions.iter() {
            assert!(std::ptr::eq(update.snapshot.static_definitions.get(id).unwrap(), definition));
        }
        for (id, constant) in image.snapshot().static_constants.iter() {
            assert!(std::ptr::eq(update.snapshot.static_constants.get(id).unwrap(), constant));
        }
        for (id, definition) in image.snapshot().callable_definitions.iter() {
            assert!(std::ptr::eq(update.snapshot.callable_definitions.get(id).unwrap(), definition));
        }
        for (id, bindings) in image.snapshot().module_static_bindings.iter() {
            assert!(std::ptr::eq(update.snapshot.module_static_bindings.get(id).unwrap(), bindings));
        }
        for (id, initialization) in image.snapshot().constructor_initializations.iter() {
            assert!(std::ptr::eq(update.snapshot.constructor_initializations.get(id).unwrap(), initialization));
        }
        for (id, header) in image.snapshot().trait_headers.iter() {
            assert!(std::ptr::eq(update.snapshot.trait_headers.get(id).unwrap(), header));
        }
        for (id, surface) in image.snapshot().trait_surfaces.iter() {
            assert!(std::ptr::eq(update.snapshot.trait_surfaces.get(id).unwrap(), surface));
        }
        assert!(update.snapshot.store.extends_canonical_prefix(&image.snapshot().store));
        assert!(update.snapshot.static_values.extends_canonical_prefix(&image.snapshot().static_values));
    }

    // Local removals and insertions must never shadow or erase canonical facts.
    let (alias_id, alias) = image.snapshot().type_aliases.iter().next().unwrap();
    let mut aliases = phalcom_semantic::type_alias::TypeAliasTable::with_parent(image.snapshot().type_aliases.clone());
    aliases.remove(alias_id);
    assert!(std::ptr::eq(aliases.get(alias_id).unwrap(), alias));
    assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| aliases.insert(alias.clone()))).is_err());

    let (definition_id, definition) = image.snapshot().static_definitions.iter().next().unwrap();
    let mut definitions = phalcom_semantic::static_definition::StaticDefinitionTable::with_parent(image.snapshot().static_definitions.clone());
    definitions.remove(definition_id);
    assert!(std::ptr::eq(definitions.get(definition_id).unwrap(), definition));
    assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| definitions.insert(definition.clone()))).is_err());

    let (constant_id, constant) = image.snapshot().static_constants.iter().next().unwrap();
    let mut constants = phalcom_semantic::static_constant::StaticConstantTable::with_parent(image.snapshot().static_constants.clone());
    constants.remove_module(constant_id.module());
    assert!(std::ptr::eq(constants.get(constant_id).unwrap(), constant));
    assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| constants.insert(constant.clone()))).is_err());
}

#[test]
fn ordinary_image_session_local_edit_reuses_unrelated_local_queries_and_retains_universe_identity() {
    use phalcom_modules::{
        ModuleId, ModulePath, ProjectIdentity, SourceId, SourceLocation, SourceRevision, SyntheticProjectIdAllocator, WorkspaceSourceBatchMutation,
    };
    use phalcom_semantic::{SemanticWorkspaceSession, db::QueryKey, identity::WorkspaceId};
    use std::collections::BTreeSet;
    use std::sync::Arc;

    let image = canonical_universe_formal_image().unwrap();
    let mut ids = SyntheticProjectIdAllocator;
    let edited = ModuleId::synthetic(ids.allocate(), ModulePath::root());
    let stable = ModuleId::synthetic(ids.allocate(), ModulePath::root());
    assert!(matches!(edited.project, ProjectIdentity::Synthetic(_)));
    assert!(matches!(stable.project, ProjectIdentity::Synthetic(_)));

    let mut session = SemanticWorkspaceSession::from_image(WorkspaceId::from_raw(736), image.clone());
    let initial = session
        .apply_module_mutations([
            WorkspaceSourceBatchMutation::SetOverlayForModule {
                module: edited.clone(),
                source: SourceLocation {
                    source_id: SourceId("image-edited.ph".into()),
                    display_path: "/tmp/image-edited.ph".into(),
                },
                text: Arc::from("from universe.random import Xoshiro256PlusPlus\nlet edited = Xoshiro256PlusPlus.seed(42).nextU64\n"),
                revision: SourceRevision(1),
                recovered_program: None,
            },
            WorkspaceSourceBatchMutation::SetOverlayForModule {
                module: stable.clone(),
                source: SourceLocation {
                    source_id: SourceId("image-stable.ph".into()),
                    display_path: "/tmp/image-stable.ph".into(),
                },
                text: Arc::from("7\n"),
                revision: SourceRevision(1),
                recovered_program: None,
            },
        ])
        .expect("initial local image overlay should publish");
    for module in [&edited, &stable] {
        assert!(
            initial.snapshot.diagnostics.get(module).is_none_or(|diagnostics| diagnostics.is_empty()),
            "initial local diagnostics for {module:?}: {:?}",
            initial.snapshot.diagnostics.get(module)
        );
    }

    let stable_key = QueryKey::ModuleStaticBindings(stable.clone());
    let edited_key = QueryKey::ModuleStaticBindings(edited.clone());
    let stable_product = session.db().product(&stable_key).expect("stable module query product").clone();
    assert!(session.db().local_query_state(&stable_key).is_some());
    assert!(session.db().local_query_state(&edited_key).is_some());

    let initial_universe_targets = session
        .module_session()
        .resolved_imports()
        .iter()
        .filter(|((importer, written), _)| importer == &edited && written.starts_with("universe"))
        .map(|(_, target)| target.clone())
        .collect::<Vec<_>>();
    assert!(
        !initial_universe_targets.is_empty(),
        "local import must resolve through external Universe authority"
    );
    assert!(
        initial_universe_targets.iter().all(|target| target.project == ProjectIdentity::Universe),
        "Universe imports must retain the builtin project identity"
    );

    let canonical_keys = image.query_image().entries().map(|(key, _)| key.clone()).collect::<BTreeSet<_>>();
    let update = session
        .apply_module_mutations([WorkspaceSourceBatchMutation::SetOverlayForModule {
            module: edited.clone(),
            source: SourceLocation {
                source_id: SourceId("image-edited.ph".into()),
                display_path: "/tmp/image-edited.ph".into(),
            },
            text: Arc::from("from universe.random import Xoshiro256PlusPlus\nlet edited = Xoshiro256PlusPlus.seed(7).nextU64\n"),
            revision: SourceRevision(2),
            recovered_program: None,
        }])
        .expect("edited local image overlay should republish");
    assert!(
        update.snapshot.diagnostics.get(&edited).is_none_or(|diagnostics| diagnostics.is_empty()),
        "edited local diagnostics: {:?}",
        update.snapshot.diagnostics.get(&edited)
    );
    assert!(
        update.stats.query_products_recomputed > 0,
        "the edited local source must recompute query products"
    );
    assert!(
        !update.invalidated.is_empty(),
        "the edited local source must invalidate prior local query state"
    );
    assert!(session.db().local_query_state(&edited_key).is_some(), "edited binding query remains local");
    assert!(session.db().is_current(&edited_key), "edited binding query must be current after republishing");
    assert!(
        update.invalidated.iter().all(|key| !canonical_keys.contains(key)),
        "canonical query roots must never enter the local invalidation set"
    );
    assert!(
        update.recomputed.iter().all(|key| !canonical_keys.contains(key)),
        "canonical query roots must never enter the local recomputation set"
    );
    assert!(
        !update.invalidated.iter().any(|key| key == &stable_key),
        "unrelated local query must not be invalidated"
    );
    assert!(
        !update.recomputed.iter().any(|key| key == &stable_key),
        "unrelated local query must not be recomputed"
    );
    let retained_stable = session.db().product(&stable_key).expect("stable module query product remains published");
    assert!(
        Arc::ptr_eq(retained_stable, &stable_product),
        "unrelated local query product must retain Arc identity"
    );

    let edited_universe_targets = session
        .module_session()
        .resolved_imports()
        .iter()
        .filter(|((importer, written), _)| importer == &edited && written.starts_with("universe"))
        .map(|(_, target)| target.clone())
        .collect::<Vec<_>>();
    assert!(
        !edited_universe_targets.is_empty(),
        "edited import must still resolve through external Universe authority"
    );
    assert!(
        edited_universe_targets.iter().all(|target| target.project == ProjectIdentity::Universe),
        "editing a local module must not rebase imported Universe identities"
    );
    assert!(session.module_session().sources().keys().all(|module| !module.project.is_universe()));
}

#[test]
fn ordinary_image_sessions_share_canonical_products_but_isolate_workspace_state() {
    use phalcom_modules::{ModuleId, ModulePath, SourceId, SourceLocation, SourceRevision, SyntheticProjectIdAllocator, WorkspaceSourceBatchMutation};
    use phalcom_semantic::{SemanticWorkspaceSession, db::QueryKey, identity::WorkspaceId};
    use std::sync::Arc;

    let image = canonical_universe_formal_image().unwrap();
    let mut left = SemanticWorkspaceSession::from_image(WorkspaceId::from_raw(737), image.clone());
    let right = SemanticWorkspaceSession::from_image(WorkspaceId::from_raw(738), image.clone());
    assert_ne!(left.workspace(), right.workspace());
    assert!(Arc::ptr_eq(left.last_snapshot().expect("left initial snapshot"), image.snapshot()));
    assert!(Arc::ptr_eq(right.last_snapshot().expect("right initial snapshot"), image.snapshot()));

    let canonical_key = image.query_image().entries().next().expect("canonical query root").0.clone();
    let left_canonical = left.db().product(&canonical_key).expect("left canonical product").clone();
    let right_canonical = right.db().product(&canonical_key).expect("right canonical product").clone();
    assert!(
        Arc::ptr_eq(&left_canonical, &right_canonical),
        "sessions must share the exact immutable canonical product"
    );
    assert!(left.db().local_query_state(&canonical_key).is_none());
    assert!(right.db().local_query_state(&canonical_key).is_none());

    let mut ids = SyntheticProjectIdAllocator;
    let local = ModuleId::synthetic(ids.allocate(), ModulePath::root());
    let published = left
        .apply_module_mutations([WorkspaceSourceBatchMutation::SetOverlayForModule {
            module: local.clone(),
            source: SourceLocation {
                source_id: SourceId("image-session-left.ph".into()),
                display_path: "/tmp/image-session-left.ph".into(),
            },
            text: Arc::from("1\n"),
            revision: SourceRevision(1),
            recovered_program: None,
        }])
        .expect("left session local source should publish");
    assert!(
        published.snapshot.diagnostics.get(&local).is_none_or(|diagnostics| diagnostics.is_empty()),
        "left-session local diagnostics: {:?}",
        published.snapshot.diagnostics.get(&local)
    );

    let local_key = QueryKey::ModuleStaticBindings(local.clone());
    assert!(
        left.db().local_query_state(&local_key).is_some(),
        "left session must publish ordinary local query state"
    );
    assert!(left.db().product(&local_key).is_some(), "left session must own the local query product");
    assert!(
        right.db().local_query_state(&local_key).is_none(),
        "right session must not observe left local query state"
    );
    assert!(
        right.db().product(&local_key).is_none(),
        "right session must not observe left local query products"
    );
    assert!(
        right.module_session().sources().is_empty(),
        "right session source ownership must remain isolated"
    );
    assert!(Arc::ptr_eq(right.last_snapshot().expect("right snapshot remains canonical"), image.snapshot()));
    assert!(right.last_known_good_snapshot().is_some_and(|snapshot| Arc::ptr_eq(snapshot, image.snapshot())));

    let left_after = left.db().product(&canonical_key).expect("left canonical product remains readable");
    let right_after = right.db().product(&canonical_key).expect("right canonical product remains readable");
    assert!(Arc::ptr_eq(left_after, &left_canonical));
    assert!(Arc::ptr_eq(right_after, &right_canonical));
    assert!(Arc::ptr_eq(left_after, right_after));
    assert!(left.db().local_query_state(&canonical_key).is_none());
    assert!(right.db().local_query_state(&canonical_key).is_none());
}

#[test]
fn from_image_zero_canonical_entries_copied_cardinality_guard() {
    let image = canonical_universe_formal_image().unwrap();
    let recorder = ::telos_observability::PerfRecorder::new(::telos_observability::PerfRunMetadata::new(
        "p5-t11-cardinality",
        ::telos_observability::ObservabilityLevel::Summary,
    ));
    let context = recorder.context();
    let _guard = context.install_current_thread();

    let session = phalcom_semantic::SemanticWorkspaceSession::from_image(phalcom_semantic::identity::WorkspaceId::from_raw(999), image.clone());
    assert!(std::sync::Arc::ptr_eq(
        session.last_snapshot().expect("initial image snapshot"),
        image.snapshot()
    ));

    #[cfg(feature = "perf-observability")]
    {
        let report = recorder.report();
        assert_eq!(
            report
                .counters
                .get(&::telos_observability::PerfCounter::CanonicalAggregateEntriesCopied)
                .copied()
                .unwrap_or(0),
            0,
            "from_image must deep-copy zero canonical aggregate entries"
        );
        assert_eq!(
            report
                .counters
                .get(&::telos_observability::PerfCounter::CanonicalParentHandlesAttached)
                .copied(),
            Some(10),
            "from_image must attach all 10 canonical parent table handles"
        );
    }
}
