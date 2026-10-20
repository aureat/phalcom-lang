use num_bigint::BigInt;
use phalcom_semantic::StaticInt;
use phalcom_semantic::types::store::{TypeData, TypeStore};
use phalcom_type_meta::encode::{decode_metadata_json, encode_metadata_json};
use phalcom_type_meta::header::MetadataProfile;
use phalcom_type_meta::type_node::TypeNode;
use std::sync::Arc;

fn declaration(name: &str) -> phalcom_semantic::DeclarationId {
    phalcom_semantic::DeclarationId::new(phalcom_semantic::ModuleId::universe_root(), name.into())
}

fn huge() -> BigInt {
    BigInt::from(1_u8) << 200
}

#[test]
fn type_store_preserves_exact_large_static_integer_identity() {
    let positive = BigInt::parse_bytes(b"9223372036854775808", 10).unwrap();
    let negative = BigInt::parse_bytes(b"-9223372036854775809", 10).unwrap();
    let mut store = TypeStore::new();

    let positive_id = store.const_int(positive.clone());
    let positive_again = store.const_int(StaticInt::from(positive));
    let negative_id = store.const_int(negative.clone());

    assert_eq!(positive_id, positive_again);
    assert_ne!(positive_id, negative_id);
    assert_eq!(
        store.get(positive_id),
        &TypeData::ConstInt(StaticInt::from(BigInt::parse_bytes(b"9223372036854775808", 10).unwrap()))
    );
    assert_eq!(store.get(negative_id), &TypeData::ConstInt(StaticInt::from(negative)));
}

#[test]
fn metadata_round_trip_preserves_large_static_integer_type_identity() {
    let mut store = TypeStore::new();
    let positive = store.const_int(BigInt::parse_bytes(b"9223372036854775808", 10).unwrap());
    let negative = store.const_int(BigInt::parse_bytes(b"-9223372036854775809", 10).unwrap());
    let mut exporter = phalcom_semantic::metadata::MetadataExporter::new(&store, None, None, None, MetadataProfile::RuntimePublic);
    let positive_id = exporter.export_type_form(positive).unwrap();
    let negative_id = exporter.export_type_form(negative).unwrap();
    let bundle = exporter.build_bundle(&[]).unwrap();

    let json = encode_metadata_json(&bundle).unwrap();
    let decoded = decode_metadata_json(&json, &Default::default()).unwrap();
    assert_eq!(
        decoded.types[positive_id.0 as usize].form,
        TypeNode::ConstInt(StaticInt::from(BigInt::parse_bytes(b"9223372036854775808", 10).unwrap()))
    );
    assert_eq!(
        decoded.types[negative_id.0 as usize].form,
        TypeNode::ConstInt(StaticInt::from(BigInt::parse_bytes(b"-9223372036854775809", 10).unwrap()))
    );
}

#[test]
fn small_and_large_inputs_intern_one_normalized_static_term() {
    let mut store = TypeStore::new();
    for value in [i64::MIN, -1, 0, 1, i64::MAX] {
        let small = store.const_int(value);
        let large_input = StaticInt::Large(Arc::new(BigInt::from(value)));
        let from_large = store.const_int(large_input);
        assert_eq!(small, from_large);
        assert!(matches!(store.get(from_large), TypeData::ConstInt(StaticInt::Small(actual)) if *actual == value));
    }

    let mut fresh = TypeStore::new();
    let kind = fresh.static_domain_kind(fresh.const_int_domain());
    let large_first = fresh.intern_with_kind(TypeData::ConstInt(StaticInt::Large(Arc::new(BigInt::from(7)))), kind);
    assert!(matches!(fresh.get(large_first), TypeData::ConstInt(StaticInt::Small(7))));
    assert_eq!(large_first, fresh.const_int(7));
}

#[test]
fn nested_heterogeneous_application_infers_and_substitutes_exact_signed_terms() {
    use phalcom_semantic::checker::inference::{ConstraintOrigin, InferenceRelation, InferenceSession, InferenceStaticArgument, InferenceTerm};
    use phalcom_semantic::types::store::StaticArgument;
    use phalcom_semantic::{KindId, MapTypeHierarchy, TupleTypeElement, TypeParameterData, TypeParameterOwner, TypeSubstitution};

    for value in [huge(), -huge()] {
        let mut store = TypeStore::new();
        let integer = store.const_int(value.clone());
        let integer_kind = store.kind_of(integer);
        assert!(store.proper_type(integer).is_err());
        let carrier_kind = store.arrow_kind(Box::new([KindId::TYPE, integer_kind]), KindId::TYPE);
        let carrier = store.nominal_form(declaration("SizedEvidence"), carrier_kind);
        let element = store.nominal_type(declaration("Element"));
        let concrete = store
            .apply_static_form(carrier, &[StaticArgument::Term(element), StaticArgument::Term(integer)])
            .unwrap();
        let partial = store.apply_static_form(carrier, &[StaticArgument::Term(element)]).unwrap();
        assert_eq!(concrete, store.apply_static_form(partial, &[StaticArgument::Term(integer)]).unwrap());
        assert!(
            store
                .apply_static_form(carrier, &[StaticArgument::Term(integer), StaticArgument::Term(element)])
                .is_err()
        );

        let nested = store.tuple(Box::new([
            TupleTypeElement::unlabeled(concrete),
            TupleTypeElement::labeled("same".into(), concrete),
        ]));
        let owner = TypeParameterOwner::Declaration(declaration("GenericEvidence"));
        let t = store.intern_type_parameter(TypeParameterData::new(owner.clone(), 0, "T", KindId::TYPE));
        let n = store.intern_type_parameter(TypeParameterData::new(owner, 1, "N", integer_kind));
        let t_form = store.parameter_form(t);
        let n_form = store.parameter_form(n);
        let symbolic = store
            .apply_static_form(carrier, &[StaticArgument::Term(t_form), StaticArgument::Term(n_form)])
            .unwrap();
        let symbolic_nested = store.tuple(Box::new([
            TupleTypeElement::unlabeled(symbolic),
            TupleTypeElement::labeled("same".into(), symbolic),
        ]));
        let mut substitution = TypeSubstitution::new();
        substitution.bind(t, element);
        substitution.bind(n, integer);
        assert_eq!(substitution.apply(&mut store, symbolic_nested), nested);

        let mut inference = InferenceSession::new();
        let inferred_t = inference.fresh_variable(KindId::TYPE);
        let inferred_n = inference.fresh_variable(integer_kind);
        let term = InferenceTerm::Applied {
            origin: Box::new(InferenceTerm::Canonical(carrier)),
            arguments: Box::new([
                InferenceStaticArgument::Term(InferenceTerm::Var(inferred_t)),
                InferenceStaticArgument::Term(InferenceTerm::Var(inferred_n)),
            ]),
        };
        inference.add_constraint(
            InferenceRelation::Equivalent(term.clone(), InferenceTerm::Canonical(concrete)),
            ConstraintOrigin::Explicit,
            None,
        );
        assert!(inference.solve(&mut store, &MapTypeHierarchy::new()).is_solved());
        assert_eq!(inference.materialize(&term, &mut store), Ok(concrete));
        assert_eq!(inference.materialize(&InferenceTerm::Var(inferred_n), &mut store), Ok(integer));

        let different = store.const_int(value + (BigInt::from(1_u8) << 64));
        let mut contradiction = InferenceSession::new();
        contradiction.add_constraint(
            InferenceRelation::Equivalent(InferenceTerm::Canonical(integer), InferenceTerm::Canonical(different)),
            ConstraintOrigin::Explicit,
            None,
        );
        assert!(!contradiction.solve(&mut store, &MapTypeHierarchy::new()).is_solved());
    }
}

#[test]
fn exact_integer_fingerprints_are_sign_sensitive_width_sensitive_and_store_independent() {
    use phalcom_semantic::KindId;
    use phalcom_semantic::metadata::MetadataExporter;
    use phalcom_semantic::types::store::StaticArgument;

    let build = |noise: bool| {
        let mut store = TypeStore::new();
        if noise {
            store.const_int(42);
        }
        let values = [huge(), -huge(), huge() + (BigInt::from(1_u8) << 64)];
        let terms = values.map(|value| store.const_int(value));
        let integer_kind = store.kind_of(terms[0]);
        let constructor_kind = store.arrow_kind(Box::new([integer_kind]), KindId::TYPE);
        let constructor = store.nominal_form(declaration("StaticEvidence"), constructor_kind);
        let applications = terms.map(|term| store.apply_static_form(constructor, &[StaticArgument::Term(term)]).unwrap());
        let mut exporter = MetadataExporter::new(&store, None, None, None, MetadataProfile::RuntimePublic);
        let nodes = applications.map(|application| exporter.export_type_form(application).unwrap());
        let bundle = exporter.build_bundle(&[]).unwrap();
        nodes.map(|node| bundle.types[node.0 as usize].structural_fingerprint)
    };
    let fingerprints = build(false);
    assert_eq!(fingerprints, build(true));
    assert_ne!(fingerprints[0], fingerprints[1]);
    assert_ne!(fingerprints[0], fingerprints[2]);
}

#[test]
fn type_member_compatibility_inputs_canonicalize_exact_integers_and_survive_metadata() {
    use phalcom_semantic::metadata::MetadataExporter;
    use phalcom_semantic::types::type_member::{ConstTerm, ProjectionSubject, TypeLevelProjection, TypeMemberArgument, TypeMemberSelector};
    use phalcom_semantic::{KindId, TypeSubstitution};
    use phalcom_type_meta::type_node::TypeMemberArgumentRef;

    for value in [huge(), -huge()] {
        let mut store = TypeStore::new();
        let subject = store.nominal_type(declaration("Evidence"));
        let integer = store.const_int(value.clone());
        let projection = |argument| TypeLevelProjection {
            subject: ProjectionSubject::Type(subject),
            selector: TypeMemberSelector::named("Witness"),
            arguments: Box::new([argument]),
            source_declaration: None,
        };
        let compatibility = store.type_member_projection(
            projection(TypeMemberArgument::Const(ConstTerm::Int(StaticInt::new(value.clone())))),
            KindId::TYPE,
        );
        let canonical = store.type_member_projection(projection(TypeMemberArgument::Type(integer)), KindId::TYPE);
        assert_eq!(compatibility, canonical);
        assert_eq!(TypeSubstitution::new().apply(&mut store, compatibility), canonical);
        let mut exporter = MetadataExporter::new(&store, None, None, None, MetadataProfile::RuntimePublic);
        let node = exporter.export_type_form(compatibility).unwrap();
        let mut bundle = exporter.build_bundle(&[]).unwrap();
        let TypeNode::TypeMemberProjection(projected) = &bundle.types[node.0 as usize].form else {
            panic!("projection expected");
        };
        let [TypeMemberArgumentRef::Type(integer_node)] = projected.arguments.as_ref() else {
            panic!("canonical integer term expected");
        };
        assert_eq!(bundle.types[integer_node.0 as usize].form, TypeNode::ConstInt(StaticInt::new(value.clone())));

        // Legacy direct member-argument transport uses the same exact scalar.
        let TypeNode::TypeMemberProjection(projected) = &mut bundle.types[node.0 as usize].form else {
            unreachable!();
        };
        projected.arguments = Box::new([TypeMemberArgumentRef::ConstInt(StaticInt::new(value))]);
        let decoded = decode_metadata_json(&encode_metadata_json(&bundle).unwrap(), &Default::default()).unwrap();
        assert_eq!(decoded, bundle);
    }
}

#[test]
fn tuple_projection_reports_exact_huge_indices_out_of_bounds() {
    use phalcom_semantic::types::projection::{ProjectionNormalizationContext, ProjectionNormalizationMode, ProjectionNormalizationResult, normalize_type};
    use phalcom_semantic::types::type_member::{ProjectionSubject, TypeLevelProjection, TypeMemberArgument, TypeMemberSelector};
    use phalcom_semantic::{CancellationToken, DeclarationTypeTable, KindId, MapTypeHierarchy, QueryBudget, TupleTypeElement};

    let mut store = TypeStore::new();
    let tuple = store.tuple(Box::new([TupleTypeElement::unlabeled(store.unit())]));
    for value in [huge(), -huge()] {
        let integer = store.const_int(value.clone());
        let projection = store.type_member_projection(
            TypeLevelProjection {
                subject: ProjectionSubject::Type(tuple),
                selector: TypeMemberSelector::Subscript,
                arguments: Box::new([TypeMemberArgument::Type(integer)]),
                source_declaration: None,
            },
            KindId::TYPE,
        );
        let conformance_index = Default::default();
        let witnesses = Default::default();
        let traits = Default::default();
        let declarations = DeclarationTypeTable::new();
        let hierarchy = MapTypeHierarchy::new();
        let cancel = CancellationToken::new();
        let mut budget = QueryBudget::new(100);
        let mode = ProjectionNormalizationMode::Exact {
            conformance_index: &conformance_index,
            witness_plans: &witnesses,
            trait_surfaces: &traits,
            derived_rules: None,
            declarations: &declarations,
            hierarchy: &hierarchy,
        };
        let result = normalize_type(&mut store, projection, &mut ProjectionNormalizationContext::new(mode, &mut budget, &cancel));
        assert!(
            matches!(result, ProjectionNormalizationResult::InternalFailure(message) if message.contains(&value.to_string()) && message.contains("out of bounds"))
        );
    }
}
