//! Canonical integration-test binary for `phalcom-semantic`.

#[path = "semantic/mod.rs"]
mod semantic;

#[path = "constructor_factory_probe.rs"]
mod constructor_factory_probe;

#[path = "module_query_provenance.rs"]
mod module_query_provenance;

#[path = "canonical_parameter_advisory.rs"]
mod canonical_parameter_advisory;

#[path = "explanation_derivation.rs"]
mod explanation_derivation;

#[path = "export_invalidation.rs"]
mod export_invalidation;

#[path = "audit_probes/mod.rs"]
mod audit_probes;

#[test]
#[cfg(feature = "test-timings")]
fn ten_sessions_share_one_baseline_build() {
    let _ = phalcom_semantic::SemanticWorkspaceSession::new();
    for _ in 0..10 {
        let _ = phalcom_semantic::SemanticWorkspaceSession::new();
    }
    assert_eq!(phalcom_semantic::semantic_baseline_build_count(), 1);
    assert!(phalcom_semantic::semantic_session_instantiation_count() >= 11);
}

#[test]
fn forked_stores_have_distinct_ids_and_identical_prefix() {
    let mut baseline_store = phalcom_semantic::TypeStore::new();
    let declaration = |name: &str| phalcom_semantic::DeclarationId::new(phalcom_semantic::ModuleId::universe_root(), name.into());
    let num_id = baseline_store.intern_with_kind(
        phalcom_semantic::TypeData::Nominal {
            declaration: declaration("Num"),
        },
        phalcom_semantic::KindId::TYPE,
    );
    let row_id = baseline_store
        .record_row_checked(
            vec![phalcom_semantic::types::row::RecordRowField::required("base".into(), num_id)],
            phalcom_semantic::types::row::RecordRowTail::Closed,
        )
        .expect("baseline row should be valid");
    let scoped_id = baseline_store
        .arena_mut()
        .intern_scoped(phalcom_semantic::types::type_lambda::ScopedTypeData::Free(num_id));
    let lambda_type = baseline_store.lambda(
        vec![phalcom_semantic::KindId::TYPE].into_boxed_slice(),
        scoped_id,
        phalcom_semantic::KindId::TYPE,
    );
    let phalcom_semantic::TypeData::Lambda(lambda_id) = baseline_store.get(lambda_type) else {
        panic!("expected baseline type lambda");
    };
    let lambda_id = *lambda_id;
    let static_domain = baseline_store.nominal_static_domain(declaration("BaselineStatic"));

    let baseline_type_count = baseline_store.type_count();
    let baseline_row_count = baseline_store.record_row_count();
    let baseline_static_domain_count = baseline_store.static_domain_ids().count();
    let mut fork_a = baseline_store.fork_from_baseline();
    let mut fork_b = baseline_store.fork_from_baseline();

    assert_ne!(baseline_store.id(), fork_a.id());
    assert_ne!(fork_a.id(), fork_b.id());

    assert_eq!(fork_a.get(num_id), baseline_store.get(num_id));
    assert_eq!(fork_b.get(num_id), baseline_store.get(num_id));
    assert_eq!(fork_a.get(num_id), fork_b.get(num_id));
    assert_eq!(fork_a.record_row(row_id), baseline_store.record_row(row_id));
    assert_eq!(fork_b.record_row(row_id), baseline_store.record_row(row_id));
    assert_eq!(fork_a.arena().get_lambda(lambda_id), baseline_store.arena().get_lambda(lambda_id));
    assert_eq!(fork_b.arena().get_lambda(lambda_id), baseline_store.arena().get_lambda(lambda_id));
    assert_eq!(fork_a.static_domain(static_domain), baseline_store.static_domain(static_domain));
    assert_eq!(fork_b.static_domain(static_domain), baseline_store.static_domain(static_domain));

    let type_a_declaration = declaration("ForkAOnly");
    let type_a = fork_a.nominal_type(type_a_declaration.clone());
    let row_a = fork_a
        .record_row_checked(
            vec![phalcom_semantic::types::row::RecordRowField::required("a".into(), type_a)],
            phalcom_semantic::types::row::RecordRowTail::Closed,
        )
        .expect("fork A row should be valid");
    let static_a_declaration = declaration("ForkAStatic");
    let static_a = fork_a.nominal_static_domain(static_a_declaration.clone());
    let scoped_a = fork_a
        .arena_mut()
        .intern_scoped(phalcom_semantic::types::type_lambda::ScopedTypeData::Free(type_a));
    let lambda_a_type = fork_a.lambda(
        vec![phalcom_semantic::KindId::TYPE].into_boxed_slice(),
        scoped_a,
        phalcom_semantic::KindId::TYPE,
    );
    let phalcom_semantic::TypeData::Lambda(lambda_a) = fork_a.get(lambda_a_type) else {
        panic!("expected fork A type lambda");
    };
    let lambda_a = *lambda_a;

    let type_b_declaration = declaration("ForkBOnly");
    let type_b = fork_b.nominal_type(type_b_declaration.clone());
    let row_b = fork_b
        .record_row_checked(
            vec![phalcom_semantic::types::row::RecordRowField::required("b".into(), type_b)],
            phalcom_semantic::types::row::RecordRowTail::Closed,
        )
        .expect("fork B row should be valid");
    let static_b_declaration = declaration("ForkBStatic");
    let static_b = fork_b.nominal_static_domain(static_b_declaration.clone());
    let scoped_b = fork_b
        .arena_mut()
        .intern_scoped(phalcom_semantic::types::type_lambda::ScopedTypeData::Bound { depth: 0, index: 0 });
    let lambda_b_type = fork_b.lambda(
        vec![phalcom_semantic::KindId::TYPE].into_boxed_slice(),
        scoped_b,
        phalcom_semantic::KindId::TYPE,
    );
    let phalcom_semantic::TypeData::Lambda(lambda_b) = fork_b.get(lambda_b_type) else {
        panic!("expected fork B type lambda");
    };
    let lambda_b = *lambda_b;

    assert_eq!(type_a.index(), baseline_type_count);
    assert_eq!(type_b.index(), baseline_type_count);
    assert_eq!(
        fork_a.get(type_a),
        &phalcom_semantic::TypeData::Nominal {
            declaration: type_a_declaration.clone()
        }
    );
    assert_eq!(
        fork_b.get(type_b),
        &phalcom_semantic::TypeData::Nominal {
            declaration: type_b_declaration.clone()
        }
    );

    assert_eq!(row_a.index(), baseline_row_count);
    assert_eq!(row_b.index(), baseline_row_count);
    assert_eq!(fork_a.record_row(row_a).fields[0].name.as_ref(), "a");
    assert_eq!(fork_b.record_row(row_b).fields[0].name.as_ref(), "b");
    assert!(fork_a.find_record_row(fork_b.record_row(row_b)).is_none());

    assert_eq!(static_a.index(), baseline_static_domain_count);
    assert_eq!(static_b.index(), baseline_static_domain_count);
    assert_eq!(
        fork_a.static_domain(static_a),
        Some(&phalcom_semantic::types::store::StaticDomainData::Nominal {
            declaration: static_a_declaration,
        })
    );
    assert_eq!(
        fork_b.static_domain(static_b),
        Some(&phalcom_semantic::types::store::StaticDomainData::Nominal {
            declaration: static_b_declaration,
        })
    );

    assert_eq!(scoped_a.index(), scoped_id.index() + 1);
    assert_eq!(scoped_b.index(), scoped_id.index() + 1);
    assert_eq!(
        fork_a.arena().get_scoped(scoped_a),
        &phalcom_semantic::types::type_lambda::ScopedTypeData::Free(type_a)
    );
    assert_eq!(
        fork_b.arena().get_scoped(scoped_b),
        &phalcom_semantic::types::type_lambda::ScopedTypeData::Bound { depth: 0, index: 0 }
    );
    assert_eq!(lambda_a.index(), lambda_id.index() + 1);
    assert_eq!(lambda_b.index(), lambda_id.index() + 1);
    assert_eq!(fork_a.arena().get_lambda(lambda_a).body, scoped_a);
    assert_eq!(fork_b.arena().get_lambda(lambda_b).body, scoped_b);

    assert_eq!(baseline_store.type_count(), baseline_type_count);
    assert_eq!(baseline_store.record_row_count(), baseline_row_count);
    assert_eq!(baseline_store.static_domain_ids().count(), baseline_static_domain_count);
}
