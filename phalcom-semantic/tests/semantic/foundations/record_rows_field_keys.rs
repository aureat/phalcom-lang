use num_bigint::BigInt;
use phalcom_modules::identity::ModuleId;
use phalcom_semantic::field_key::FieldKey;
use phalcom_semantic::identity::DeclarationId;
use phalcom_semantic::scalar::StaticInt;
use phalcom_semantic::types::evidence::UnknownReason;
use phalcom_semantic::types::id::KindId;
use phalcom_semantic::types::id::TypeId;
use phalcom_semantic::types::outcome::{BlockReason, RelationOutcome};
use phalcom_semantic::types::outcome::{CancellationToken, QueryBudget};
use phalcom_semantic::types::parameter::{TypeParameterData, TypeParameterOwner};
use phalcom_semantic::types::relation::{
    MapTypeHierarchy, UnorderedFieldLookup, check_field_key_domain_membership, check_subtype_bounded, lookup_unordered_field,
};
use phalcom_semantic::types::row::{AnonymousRecordRemainder, RecordRowField, RecordRowFormationError, RecordRowTail, StructuralMemberPresence};
use phalcom_semantic::types::row_solver::{RecordRowFailure, RecordRowShapeSolveResult, RecordRowSolver, RecordRowTerm, RecordRowTermTail};
use phalcom_semantic::types::store::TypeStore;

fn declaration(name: &str) -> DeclarationId {
    DeclarationId::new(ModuleId::universe_root(), name.into())
}

#[test]
fn canonical_row_identity_is_order_independent_for_mixed_field_keys() {
    let mut store = TypeStore::new();
    let int_type = store.nominal(declaration("Int"));
    let large_int = StaticInt::new(BigInt::parse_bytes(b"1234567890123456789012345678901234567890", 10).unwrap());
    let fields = vec![
        RecordRowField::optional(FieldKey::bool(true), int_type),
        RecordRowField::required(FieldKey::int(large_int.clone()), int_type),
        RecordRowField::required(FieldKey::string("same"), int_type),
        RecordRowField::optional(FieldKey::symbol("same"), int_type),
    ];
    let reversed = fields.iter().cloned().rev().collect();

    let first = store.record_row_checked(fields, RecordRowTail::Closed).expect("unique mixed-key row");
    let second = store.record_row_checked(reversed, RecordRowTail::Closed).expect("reordered mixed-key row");
    assert_eq!(first, second, "source order must not affect canonical RecordRowId");

    let row = store.record_row(first);
    assert_eq!(
        row.fields.iter().map(|field| field.key.clone()).collect::<Vec<_>>(),
        vec![
            FieldKey::symbol("same"),
            FieldKey::string("same"),
            FieldKey::int(large_int.clone()),
            FieldKey::bool(true),
        ]
    );
    assert_ne!(row.fields[0].key, row.fields[1].key, "same-content Symbol and String keys stay distinct");
    assert!(matches!(&row.fields[2].key, FieldKey::Int(value) if value == &large_int));
    assert_eq!(row.fields[0].presence, StructuralMemberPresence::Optional);
    assert_eq!(row.fields[1].presence, StructuralMemberPresence::Required);
    assert_eq!(row.fields[3].presence, StructuralMemberPresence::Optional);
}

#[test]
fn canonical_row_rejects_exact_duplicates_for_every_field_key_kind() {
    let mut store = TypeStore::new();
    let int_type = store.nominal(declaration("Int"));
    let keys = [
        FieldKey::symbol("duplicate"),
        FieldKey::string("duplicate"),
        FieldKey::int(StaticInt::from(7)),
        FieldKey::bool(false),
    ];

    for key in keys {
        let result = store.record_row_checked(
            vec![RecordRowField::required(key.clone(), int_type), RecordRowField::optional(key.clone(), int_type)],
            RecordRowTail::Closed,
        );
        assert!(matches!(result, Err(RecordRowFormationError::DuplicateField(duplicate)) if duplicate == key));
    }
}

#[test]
fn tail_key_and_value_domains_are_retained_and_must_be_proper_types() {
    let mut store = TypeStore::new();
    let string_type = store.nominal(declaration("String"));
    let int_type = store.nominal(declaration("Int"));

    let unconstrained = store
        .record_row_checked(
            Vec::new(),
            RecordRowTail::Anonymous(AnonymousRecordRemainder::Unconstrained { key_type: string_type }),
        )
        .expect("proper key-domain type");
    assert_eq!(
        store.record_row(unconstrained).tail,
        RecordRowTail::Anonymous(AnonymousRecordRemainder::Unconstrained { key_type: string_type })
    );

    let typed = store
        .record_row_checked(
            Vec::new(),
            RecordRowTail::Anonymous(AnonymousRecordRemainder::Typed {
                key_type: int_type,
                value_type: string_type,
            }),
        )
        .expect("proper key and value domain types");
    assert_eq!(
        store.record_row(typed).tail,
        RecordRowTail::Anonymous(AnonymousRecordRemainder::Typed {
            key_type: int_type,
            value_type: string_type,
        })
    );

    let non_type = store.const_int(1);
    assert!(matches!(
        store.record_row_checked(
            Vec::new(),
            RecordRowTail::Anonymous(AnonymousRecordRemainder::Unconstrained { key_type: non_type }),
        ),
        Err(RecordRowFormationError::TailKeyDomainInvalid(found)) if found == non_type
    ));
}

#[test]
fn solver_merges_mixed_canonical_prefixes_and_keeps_same_text_keys_distinct() {
    let mut store = TypeStore::new();
    let int_type = store.nominal(declaration("Int"));
    let symbol_x = FieldKey::symbol("x");
    let string_x = FieldKey::string("x");
    let int_one = FieldKey::int(StaticInt::from(1));
    let bool_false = FieldKey::bool(false);
    let left = RecordRowTerm::new(
        vec![
            RecordRowField::required(symbol_x.clone(), int_type),
            RecordRowField::required(int_one.clone(), int_type),
            RecordRowField::required(bool_false.clone(), int_type),
        ],
        RecordRowTermTail::Closed,
    )
    .unwrap();
    let right = RecordRowTerm::new(
        vec![
            RecordRowField::required(string_x.clone(), int_type),
            RecordRowField::required(int_one.clone(), int_type),
            RecordRowField::required(bool_false.clone(), int_type),
        ],
        RecordRowTermTail::Closed,
    )
    .unwrap();

    let mut solver = RecordRowSolver::<TypeId>::new();
    let left_tail = solver.fresh_var();
    let right_tail = solver.fresh_var();
    let left = RecordRowTerm::new(left.fields.into_vec(), RecordRowTermTail::Var(left_tail)).unwrap();
    let right = RecordRowTerm::new(right.fields.into_vec(), RecordRowTermTail::Var(right_tail)).unwrap();
    solver.unify(&left, &right, &store).expect("distinct keys can be solved through open tails");
    let normalized_left = solver.normalize_term(&RecordRowTerm::new(Vec::new(), RecordRowTermTail::Var(left_tail)).unwrap());
    let normalized_right = solver.normalize_term(&RecordRowTerm::new(Vec::new(), RecordRowTermTail::Var(right_tail)).unwrap());
    assert_eq!(normalized_left.fields.len(), 1);
    assert_eq!(normalized_right.fields.len(), 1);
    assert_eq!(normalized_left.fields[0].key, string_x);
    assert_eq!(normalized_right.fields[0].key, symbol_x);
}

#[test]
fn solver_lacks_preserve_exact_identity_for_all_field_key_kinds() {
    let mut store = TypeStore::new();
    let int_type = store.nominal(declaration("Int"));
    let keys = [
        FieldKey::symbol("x"),
        FieldKey::string("x"),
        FieldKey::int(StaticInt::from(3)),
        FieldKey::bool(true),
    ];
    for key in keys {
        let mut solver = RecordRowSolver::<TypeId>::new();
        let variable = solver.fresh_var();
        solver.add_lacks(variable, key.clone()).unwrap();
        let unresolved = RecordRowTerm::new(Vec::new(), RecordRowTermTail::Var(variable)).unwrap();
        let concrete = RecordRowTerm::closed(vec![RecordRowField::required(key.clone(), int_type)]).unwrap();
        let mut budget = QueryBudget::default();
        let result = solver.solve_shapes_for_roots(&[(unresolved, concrete)], &[variable], &mut budget, &CancellationToken::new());
        assert!(matches!(
            result,
            RecordRowShapeSolveResult::Rejected(RecordRowFailure::LacksViolation { field, .. }) if field == key
        ));
    }

    let mut solver = RecordRowSolver::<TypeId>::new();
    let variable = solver.fresh_var();
    solver.add_lacks(variable, FieldKey::symbol("same")).unwrap();
    let unresolved = RecordRowTerm::new(Vec::new(), RecordRowTermTail::Var(variable)).unwrap();
    let string_field = RecordRowTerm::closed(vec![RecordRowField::required(FieldKey::string("same"), int_type)]).unwrap();
    let mut budget = QueryBudget::default();
    assert!(matches!(
        solver.solve_shapes_for_roots(&[(unresolved, string_field)], &[variable], &mut budget, &CancellationToken::new(),),
        RecordRowShapeSolveResult::Solved { .. }
    ));
}

#[test]
fn solver_correspondence_handles_ten_thousand_field_prefixes() {
    let mut store = TypeStore::new();
    let int_type = store.nominal(declaration("Int"));
    let fields = (0..10_000)
        .map(|index| RecordRowField::required(FieldKey::int(StaticInt::from(index)), int_type))
        .collect::<Vec<_>>();
    let left = RecordRowTerm::closed(fields.clone()).expect("unique left row");
    let right = RecordRowTerm::closed(fields).expect("unique right row");
    let mut solver = RecordRowSolver::<TypeId>::new();
    let mut budget = QueryBudget::default();
    let cancellation = CancellationToken::new();

    let result = solver.solve_shapes_for_roots(&[(left, right)], &[], &mut budget, &cancellation);
    let RecordRowShapeSolveResult::Solved { field_equations, .. } = result else {
        panic!("expected canonical prefixes to solve, got {result:?}");
    };
    assert_eq!(field_equations.len(), 10_000);
}

#[test]
fn subtype_checks_fixed_keys_and_row_tails_against_key_domains() {
    let mut store = TypeStore::new();
    let core = phalcom_semantic::core_surface::CoreDeclarationIds::default();
    let symbol_type = store.nominal_type(core.symbol);
    let string_type = store.nominal_type(core.string);
    let int_type = store.nominal_type(core.int);
    let object_type = store.nominal_type(core.object);
    let field_key_type = store.field_key_type_from_core();
    let hierarchy = MapTypeHierarchy::new();
    let cancellation = CancellationToken::new();

    let mut budget = QueryBudget::default();
    assert!(check_field_key_domain_membership(&mut store, &hierarchy, &FieldKey::symbol("name"), symbol_type, &mut budget, &cancellation).is_proven());
    let mut budget = QueryBudget::default();
    assert!(matches!(
        check_field_key_domain_membership(&mut store, &hierarchy, &FieldKey::symbol("name"), string_type, &mut budget, &cancellation),
        RelationOutcome::Refuted(_)
    ));

    let source_tail = store
        .record_row_type_checked(
            Vec::new(),
            RecordRowTail::Anonymous(AnonymousRecordRemainder::Typed {
                key_type: string_type,
                value_type: int_type,
            }),
        )
        .unwrap();
    let target_tail = store
        .record_row_type_checked(
            Vec::new(),
            RecordRowTail::Anonymous(AnonymousRecordRemainder::Typed {
                key_type: field_key_type,
                value_type: object_type,
            }),
        )
        .unwrap();
    let mut budget = QueryBudget::default();
    assert!(check_subtype_bounded(&mut store, &hierarchy, source_tail, target_tail, &mut budget, &cancellation).is_proven());

    let reversed_tail = store
        .record_row_type_checked(
            Vec::new(),
            RecordRowTail::Anonymous(AnonymousRecordRemainder::Typed {
                key_type: field_key_type,
                value_type: int_type,
            }),
        )
        .unwrap();
    let strict_target = store
        .record_row_type_checked(
            Vec::new(),
            RecordRowTail::Anonymous(AnonymousRecordRemainder::Typed {
                key_type: symbol_type,
                value_type: int_type,
            }),
        )
        .unwrap();
    let mut budget = QueryBudget::default();
    assert!(matches!(
        check_subtype_bounded(&mut store, &hierarchy, reversed_tail, strict_target, &mut budget, &cancellation),
        RelationOutcome::Refuted(_)
    ));

    let unconstrained_field_key = store
        .record_row_type_checked(
            Vec::new(),
            RecordRowTail::Anonymous(AnonymousRecordRemainder::Unconstrained { key_type: field_key_type }),
        )
        .unwrap();
    let unconstrained_string = store
        .record_row_type_checked(
            Vec::new(),
            RecordRowTail::Anonymous(AnonymousRecordRemainder::Unconstrained { key_type: string_type }),
        )
        .unwrap();
    let typed_string = source_tail;
    let mut budget = QueryBudget::default();
    assert!(check_subtype_bounded(&mut store, &hierarchy, typed_string, unconstrained_field_key, &mut budget, &cancellation).is_proven());
    let mut budget = QueryBudget::default();
    assert!(
        check_subtype_bounded(
            &mut store,
            &hierarchy,
            unconstrained_string,
            unconstrained_field_key,
            &mut budget,
            &cancellation
        )
        .is_proven()
    );
    let mut budget = QueryBudget::default();
    assert!(matches!(
        check_subtype_bounded(
            &mut store,
            &hierarchy,
            unconstrained_field_key,
            unconstrained_string,
            &mut budget,
            &cancellation
        ),
        RelationOutcome::Refuted(_)
    ));

    let fixed_string = store
        .record_row_type_checked(vec![RecordRowField::required(FieldKey::string("extra"), int_type)], RecordRowTail::Closed)
        .unwrap();
    let fixed_symbol = store
        .record_row_type_checked(vec![RecordRowField::required(FieldKey::symbol("extra"), int_type)], RecordRowTail::Closed)
        .unwrap();
    let mut budget = QueryBudget::default();
    assert!(check_subtype_bounded(&mut store, &hierarchy, fixed_string, source_tail, &mut budget, &cancellation).is_proven());
    let mut budget = QueryBudget::default();
    assert!(matches!(
        check_subtype_bounded(&mut store, &hierarchy, fixed_symbol, source_tail, &mut budget, &cancellation),
        RelationOutcome::Refuted(_)
    ));
}

#[test]
fn key_domain_relation_preserves_query_budget_and_cancellation() {
    let mut store = TypeStore::new();
    let core = phalcom_semantic::core_surface::CoreDeclarationIds::default();
    let symbol_type = store.nominal_type(core.symbol);
    let hierarchy = MapTypeHierarchy::new();
    let mut budget = QueryBudget::new(0);
    assert!(matches!(
        check_field_key_domain_membership(
            &mut store,
            &hierarchy,
            &FieldKey::symbol("name"),
            symbol_type,
            &mut budget,
            &CancellationToken::new()
        ),
        RelationOutcome::BudgetExceeded(_)
    ));

    let cancellation = CancellationToken::new();
    cancellation.cancel();
    assert!(matches!(
        check_field_key_domain_membership(
            &mut store,
            &hierarchy,
            &FieldKey::symbol("name"),
            symbol_type,
            &mut QueryBudget::default(),
            &cancellation
        ),
        RelationOutcome::Cancelled
    ));
}

#[test]
fn key_domain_union_with_unresolved_parameter_does_not_claim_absence() {
    let mut store = TypeStore::new();
    let core = phalcom_semantic::core_surface::CoreDeclarationIds::default();
    let string_type = store.nominal_type(core.string);
    let int_type = store.nominal_type(core.int);
    let parameter = store.intern_type_parameter(TypeParameterData::new(
        TypeParameterOwner::Declaration(declaration("UnionKeyProbe")),
        0,
        "K",
        KindId::TYPE,
    ));
    let parameter_type = store.parameter_form(parameter);
    let key_domain = store.union(&[string_type, parameter_type]);
    let hierarchy = MapTypeHierarchy::new();
    let key = FieldKey::bool(false);
    let mut budget = QueryBudget::default();
    let outcome = check_field_key_domain_membership(&mut store, &hierarchy, &key, key_domain, &mut budget, &CancellationToken::new());
    assert!(matches!(
        outcome,
        RelationOutcome::Blocked(BlockReason::UnknownType(UnknownReason::InferenceBlocked))
    ));

    let row = store
        .record_row_checked(
            Vec::new(),
            RecordRowTail::Anonymous(AnonymousRecordRemainder::Typed {
                key_type: key_domain,
                value_type: int_type,
            }),
        )
        .unwrap();
    assert!(matches!(
        lookup_unordered_field(&mut store, &hierarchy, row, &key, &mut QueryBudget::default(), &CancellationToken::new(),),
        UnorderedFieldLookup::Unknown(RelationOutcome::Blocked(BlockReason::UnknownType(UnknownReason::InferenceBlocked)))
    ));
}

#[test]
fn subtype_optional_absence_uses_source_tail_key_membership() {
    let mut store = TypeStore::new();
    let core = phalcom_semantic::core_surface::CoreDeclarationIds::default();
    let symbol_type = store.nominal_type(core.symbol);
    let string_type = store.nominal_type(core.string);
    let int_type = store.nominal_type(core.int);
    let string_value_type = string_type;
    let hierarchy = MapTypeHierarchy::new();
    let cancellation = CancellationToken::new();

    let source_with_string_tail = store
        .record_row_type_checked(
            Vec::new(),
            RecordRowTail::Anonymous(AnonymousRecordRemainder::Typed {
                key_type: string_type,
                value_type: int_type,
            }),
        )
        .unwrap();
    let optional_symbol_target = store
        .record_row_type_checked(
            vec![RecordRowField::optional(FieldKey::symbol("name"), string_value_type)],
            RecordRowTail::Anonymous(AnonymousRecordRemainder::Typed {
                key_type: string_type,
                value_type: int_type,
            }),
        )
        .unwrap();
    let mut budget = QueryBudget::default();
    assert!(
        check_subtype_bounded(
            &mut store,
            &hierarchy,
            source_with_string_tail,
            optional_symbol_target,
            &mut budget,
            &cancellation
        )
        .is_proven()
    );

    let source_with_symbol_tail = store
        .record_row_type_checked(
            Vec::new(),
            RecordRowTail::Anonymous(AnonymousRecordRemainder::Typed {
                key_type: symbol_type,
                value_type: int_type,
            }),
        )
        .unwrap();
    let optional_symbol_target_with_symbol_tail = store
        .record_row_type_checked(
            vec![RecordRowField::optional(FieldKey::symbol("name"), string_value_type)],
            RecordRowTail::Anonymous(AnonymousRecordRemainder::Typed {
                key_type: symbol_type,
                value_type: int_type,
            }),
        )
        .unwrap();
    let mut budget = QueryBudget::default();
    assert!(matches!(
        check_subtype_bounded(
            &mut store,
            &hierarchy,
            source_with_symbol_tail,
            optional_symbol_target_with_symbol_tail,
            &mut budget,
            &cancellation
        ),
        RelationOutcome::Refuted(_)
    ));

    let unconstrained_string_tail = store
        .record_row_type_checked(
            Vec::new(),
            RecordRowTail::Anonymous(AnonymousRecordRemainder::Unconstrained { key_type: string_type }),
        )
        .unwrap();
    let optional_symbol_target_with_unconstrained_string_tail = store
        .record_row_type_checked(
            vec![RecordRowField::optional(FieldKey::symbol("name"), string_value_type)],
            RecordRowTail::Anonymous(AnonymousRecordRemainder::Unconstrained { key_type: string_type }),
        )
        .unwrap();
    let mut budget = QueryBudget::default();
    assert!(
        check_subtype_bounded(
            &mut store,
            &hierarchy,
            unconstrained_string_tail,
            optional_symbol_target_with_unconstrained_string_tail,
            &mut budget,
            &cancellation
        )
        .is_proven()
    );

    let unconstrained_symbol_tail = store
        .record_row_type_checked(
            Vec::new(),
            RecordRowTail::Anonymous(AnonymousRecordRemainder::Unconstrained { key_type: symbol_type }),
        )
        .unwrap();
    let optional_symbol_target_with_unconstrained_symbol_tail = store
        .record_row_type_checked(
            vec![RecordRowField::optional(FieldKey::symbol("name"), string_value_type)],
            RecordRowTail::Anonymous(AnonymousRecordRemainder::Unconstrained { key_type: symbol_type }),
        )
        .unwrap();
    let mut budget = QueryBudget::default();
    assert!(matches!(
        check_subtype_bounded(
            &mut store,
            &hierarchy,
            unconstrained_symbol_tail,
            optional_symbol_target_with_unconstrained_symbol_tail,
            &mut budget,
            &cancellation
        ),
        RelationOutcome::Refuted(_)
    ));
}

#[test]
fn lookup_unordered_field_distinguishes_fixed_tail_absent_and_unknown() {
    let mut store = TypeStore::new();
    let core = phalcom_semantic::core_surface::CoreDeclarationIds::default();
    let string_type = store.nominal_type(core.string);
    let int_type = store.nominal_type(core.int);
    let hierarchy = MapTypeHierarchy::new();
    let cancellation = CancellationToken::new();
    let row = store
        .record_row_checked(
            vec![
                RecordRowField::required(FieldKey::symbol("required"), int_type),
                RecordRowField::optional(FieldKey::symbol("optional"), string_type),
            ],
            RecordRowTail::Anonymous(AnonymousRecordRemainder::Typed {
                key_type: string_type,
                value_type: int_type,
            }),
        )
        .unwrap();

    assert_eq!(
        lookup_unordered_field(
            &mut store,
            &hierarchy,
            row,
            &FieldKey::symbol("required"),
            &mut QueryBudget::default(),
            &cancellation
        ),
        UnorderedFieldLookup::Required(int_type)
    );
    assert_eq!(
        lookup_unordered_field(
            &mut store,
            &hierarchy,
            row,
            &FieldKey::symbol("optional"),
            &mut QueryBudget::default(),
            &cancellation
        ),
        UnorderedFieldLookup::Optional(string_type)
    );
    assert_eq!(
        lookup_unordered_field(
            &mut store,
            &hierarchy,
            row,
            &FieldKey::symbol("missing"),
            &mut QueryBudget::default(),
            &cancellation
        ),
        UnorderedFieldLookup::Absent
    );
    assert_eq!(
        lookup_unordered_field(
            &mut store,
            &hierarchy,
            row,
            &FieldKey::string("extra"),
            &mut QueryBudget::default(),
            &cancellation
        ),
        UnorderedFieldLookup::Optional(int_type)
    );

    let unconstrained = store
        .record_row_checked(
            Vec::new(),
            RecordRowTail::Anonymous(AnonymousRecordRemainder::Unconstrained { key_type: string_type }),
        )
        .unwrap();
    assert!(matches!(
        lookup_unordered_field(
            &mut store,
            &hierarchy,
            unconstrained,
            &FieldKey::string("maybe"),
            &mut QueryBudget::default(),
            &cancellation
        ),
        UnorderedFieldLookup::Unknown(RelationOutcome::Blocked(BlockReason::UnknownType(UnknownReason::NoTypeEvidence)))
    ));

    let key_parameter = store.intern_type_parameter(TypeParameterData::new(
        TypeParameterOwner::Declaration(declaration("LookupProbe")),
        0,
        "K",
        KindId::TYPE,
    ));
    let unknown_key_domain = store.parameter_form(key_parameter);
    let generic_tail = store
        .record_row_checked(
            Vec::new(),
            RecordRowTail::Anonymous(AnonymousRecordRemainder::Typed {
                key_type: unknown_key_domain,
                value_type: int_type,
            }),
        )
        .unwrap();
    assert!(matches!(
        lookup_unordered_field(
            &mut store,
            &hierarchy,
            generic_tail,
            &FieldKey::symbol("possibly-present"),
            &mut QueryBudget::default(),
            &cancellation
        ),
        UnorderedFieldLookup::Unknown(RelationOutcome::Blocked(BlockReason::UnknownType(UnknownReason::InferenceBlocked)))
    ));
}
