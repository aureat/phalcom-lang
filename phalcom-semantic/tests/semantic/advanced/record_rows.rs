use phalcom_modules::identity::ModuleId;
use phalcom_semantic::identity::DeclarationId;
use phalcom_semantic::types::id::KindId;
use phalcom_semantic::types::outcome::{CancellationToken, QueryBudget};
use phalcom_semantic::types::parameter::{TypeParameterData, TypeParameterOwner};
use phalcom_semantic::types::relation::{MapTypeHierarchy, is_subtype};
use phalcom_semantic::types::row::{DuplicateFieldError, RecordRowData, RecordRowField, RecordRowFormationError, RecordRowTail};
use phalcom_semantic::types::row_solver::{RecordRowFailure, RecordRowSolver, RecordRowTerm, RecordRowTermTail};
use phalcom_semantic::types::store::TypeStore;

fn test_decl(name: &str) -> DeclarationId {
    let module = ModuleId::universe_root();
    DeclarationId::new(module, name.into())
}

#[test]
fn test_permutation_equivalence() {
    let mut store = TypeStore::new();
    let int_ty = store.nominal(test_decl("Int"));
    let str_ty = store.nominal(test_decl("String"));

    let f1 = vec![RecordRowField { name: "a".into(), ty: int_ty }, RecordRowField { name: "b".into(), ty: str_ty }];
    let f2 = vec![RecordRowField { name: "b".into(), ty: str_ty }, RecordRowField { name: "a".into(), ty: int_ty }];

    let rec1 = store.record(f1.into_boxed_slice());
    let rec2 = store.record(f2.into_boxed_slice());
    assert_eq!(rec1, rec2, "Closed record types are permutation equivalent");
}

#[test]
fn test_duplicate_fields_rejected() {
    let mut store = TypeStore::new();
    let int_ty = store.nominal(test_decl("Int"));

    let fields = vec![RecordRowField { name: "x".into(), ty: int_ty }, RecordRowField { name: "x".into(), ty: int_ty }];

    let res = RecordRowData::new_closed(fields);
    assert!(matches!(res, Err(DuplicateFieldError(name)) if name.as_ref() == "x"));
}

#[test]
fn test_row_subtraction_solves() {
    let mut store = TypeStore::new();
    let str_ty = store.nominal(test_decl("String"));

    let empty_row = store.record_row_checked(Vec::new(), RecordRowTail::Closed).unwrap();

    let single_row = store
        .record_row_checked(
            vec![RecordRowField {
                name: "name".into(),
                ty: str_ty,
            }],
            RecordRowTail::Closed,
        )
        .unwrap();

    let mut solver = RecordRowSolver::new();
    let r_var = solver.fresh_var();

    // #{ name: String } = #{ name: String | R }
    let left = RecordRowTerm::from_canonical(&store, single_row);
    let right = RecordRowTerm {
        fields: Box::new([RecordRowField {
            name: "name".into(),
            ty: str_ty,
        }]),
        tail: RecordRowTermTail::Var(r_var),
    };

    let mut budget = QueryBudget::default();
    let result = solver.solve(&left, &right, &store, &mut budget, &CancellationToken::new());
    assert!(matches!(result, phalcom_semantic::types::row_solver::RecordRowSolveResult::Solved(sol) if {
        sol.term_for(r_var) == Some(&RecordRowTerm { fields: Box::new([]), tail: RecordRowTermTail::Closed })
    }));
    assert_eq!(store.record_row_count(), 2, "solver must not intern speculative remainders");
    assert!(empty_row.index() < single_row.index());
}

#[test]
fn test_lacks_constraint_blocks_duplicate_extension() {
    let mut store = TypeStore::new();
    let str_ty = store.nominal(test_decl("String"));

    let row = store
        .record_row_checked(
            vec![RecordRowField {
                name: "name".into(),
                ty: str_ty,
            }],
            RecordRowTail::Closed,
        )
        .unwrap();

    let mut solver = RecordRowSolver::new();
    let r_var = solver.fresh_var();
    solver.add_lacks(r_var, "name".into()).unwrap();

    let left = RecordRowTerm {
        fields: Box::new([]),
        tail: RecordRowTermTail::Var(r_var),
    };
    let right = RecordRowTerm::from_canonical(&store, row);

    let mut budget = QueryBudget::default();
    let result = solver.solve(&left, &right, &store, &mut budget, &CancellationToken::new());
    assert!(matches!(
        result,
        phalcom_semantic::types::row_solver::RecordRowSolveResult::Rejected(RecordRowFailure::LacksViolation { .. })
    ));
}

#[test]
fn test_occurs_check_rejects() {
    let mut store = TypeStore::new();
    let int_ty = store.nominal(test_decl("Int"));

    let mut solver = RecordRowSolver::new();
    let r_var = solver.fresh_var();

    // R = #{ next: Int | R }
    let left = RecordRowTerm {
        fields: Box::new([]),
        tail: RecordRowTermTail::Var(r_var),
    };
    let right = RecordRowTerm {
        fields: Box::new([RecordRowField {
            name: "next".into(),
            ty: int_ty,
        }]),
        tail: RecordRowTermTail::Var(r_var),
    };

    let mut budget = QueryBudget::default();
    let result = solver.solve(&left, &right, &store, &mut budget, &CancellationToken::new());
    assert!(matches!(
        result,
        phalcom_semantic::types::row_solver::RecordRowSolveResult::Rejected(RecordRowFailure::OccursCheckFailed { .. })
    ));
}

#[test]
#[should_panic(expected = "RecordRow-kinded type parameters must never produce TypeData::Parameter")]
fn test_domain_safety_record_row_type_parameter_never_produces_type_data_parameter() {
    let mut store = TypeStore::new();
    let decl = test_decl("Container");
    let param_id = store.intern_type_parameter(TypeParameterData::new(TypeParameterOwner::Declaration(decl), 0, "R", KindId::RECORD_ROW));

    // Attempting to create TypeData::Parameter with RecordRow kind must panic / be rejected
    store.parameter_form(param_id);
}

#[test]
fn test_record_subtyping_immutable_width() {
    let mut store = TypeStore::new();
    let hier = MapTypeHierarchy::new();
    let int_ty = store.nominal(test_decl("Int"));
    let str_ty = store.nominal(test_decl("String"));

    // sub: #{ a: Int, b: String }
    let sub = store.record(Box::new([
        RecordRowField { name: "a".into(), ty: int_ty },
        RecordRowField { name: "b".into(), ty: str_ty },
    ]));

    // sup: #{ a: Int }
    let sup = store.record(Box::new([RecordRowField { name: "a".into(), ty: int_ty }]));

    assert!(is_subtype(&mut store, &hier, sub, sup), "Width subtyping allowed for ReadOnly");
    assert!(!is_subtype(&mut store, &hier, sup, sub), "Narrow cannot subtype wide");
}

#[test]
fn checked_row_rejects_non_row_tail_parameter() {
    let mut store = TypeStore::new();
    let owner = TypeParameterOwner::Declaration(test_decl("Owner"));
    let parameter = store.intern_type_parameter(TypeParameterData::new(owner, 0, "T", KindId::TYPE));
    let int_ty = store.nominal(test_decl("Int"));
    let result = store.record_row_type_checked(
        vec![RecordRowField {
            name: "value".into(),
            ty: int_ty,
        }],
        RecordRowTail::Parameter(parameter),
    );
    assert!(matches!(
        result,
        Err(RecordRowFormationError::TailParameterWrongKind { actual: KindId::TYPE, .. })
    ));
}

#[test]
fn row_solution_is_history_independent() {
    let mut first = TypeStore::new();
    let string_ty = first.nominal(test_decl("String"));
    let mut solver = RecordRowSolver::new();
    let variable = solver.fresh_var();
    let left = RecordRowTerm {
        fields: Box::new([RecordRowField {
            name: "name".into(),
            ty: string_ty,
        }]),
        tail: RecordRowTermTail::Closed,
    };
    let right = RecordRowTerm {
        fields: Box::new([]),
        tail: RecordRowTermTail::Var(variable),
    };
    let mut budget = QueryBudget::default();
    let result = solver.solve(&left, &right, &first, &mut budget, &CancellationToken::new());
    assert!(matches!(result, phalcom_semantic::types::row_solver::RecordRowSolveResult::Solved(_)));

    let mut second = TypeStore::new();
    let string_ty = second.nominal(test_decl("String"));
    let _unrelated = second.record(Box::new([RecordRowField {
        name: "unrelated".into(),
        ty: string_ty,
    }]));
    let mut solver = RecordRowSolver::new();
    let variable = solver.fresh_var();
    let left = RecordRowTerm {
        fields: Box::new([RecordRowField {
            name: "name".into(),
            ty: string_ty,
        }]),
        tail: RecordRowTermTail::Closed,
    };
    let right = RecordRowTerm {
        fields: Box::new([]),
        tail: RecordRowTermTail::Var(variable),
    };
    let mut budget = QueryBudget::default();
    let result = solver.solve(&left, &right, &second, &mut budget, &CancellationToken::new());
    assert!(matches!(result, phalcom_semantic::types::row_solver::RecordRowSolveResult::Solved(_)));
}

#[test]
fn lacks_constraint_survives_variable_alias() {
    let mut store = TypeStore::new();
    let string_ty = store.nominal(test_decl("String"));
    let row = store
        .record_row_checked(
            vec![RecordRowField {
                name: "name".into(),
                ty: string_ty,
            }],
            RecordRowTail::Closed,
        )
        .unwrap();
    let mut solver = RecordRowSolver::new();
    let first = solver.fresh_var();
    let second = solver.fresh_var();
    solver.add_lacks(first, "name".into()).unwrap();
    let mut budget = QueryBudget::default();
    let alias = solver.solve(
        &RecordRowTerm {
            fields: Box::new([]),
            tail: RecordRowTermTail::Var(first),
        },
        &RecordRowTerm {
            fields: Box::new([]),
            tail: RecordRowTermTail::Var(second),
        },
        &store,
        &mut budget,
        &CancellationToken::new(),
    );
    assert!(matches!(alias, phalcom_semantic::types::row_solver::RecordRowSolveResult::Underconstrained(_)));

    let mut budget = QueryBudget::default();
    let result = solver.solve(
        &RecordRowTerm {
            fields: Box::new([]),
            tail: RecordRowTermTail::Var(second),
        },
        &RecordRowTerm::from_canonical(&store, row),
        &store,
        &mut budget,
        &CancellationToken::new(),
    );
    assert!(matches!(
        result,
        phalcom_semantic::types::row_solver::RecordRowSolveResult::Rejected(RecordRowFailure::LacksViolation { .. })
    ));
}

#[test]
fn indirect_row_occurs_check_is_rejected() {
    let store = TypeStore::new();
    let mut solver = RecordRowSolver::new();
    let first = solver.fresh_var();
    let second = solver.fresh_var();
    let equations = [
        (
            RecordRowTerm {
                fields: Box::new([]),
                tail: RecordRowTermTail::Var(first),
            },
            RecordRowTerm {
                fields: Box::new([]),
                tail: RecordRowTermTail::Var(second),
            },
        ),
        (
            RecordRowTerm {
                fields: Box::new([]),
                tail: RecordRowTermTail::Var(second),
            },
            RecordRowTerm {
                fields: Box::new([RecordRowField {
                    name: "next".into(),
                    ty: store.unit(),
                }]),
                tail: RecordRowTermTail::Var(first),
            },
        ),
    ];
    let mut budget = QueryBudget::default();
    let result = solver.solve_many(&equations, &store, &mut budget, &CancellationToken::new());
    assert!(matches!(
        result,
        phalcom_semantic::types::row_solver::RecordRowSolveResult::Rejected(RecordRowFailure::OccursCheckFailed { .. })
    ));
}

#[test]
fn row_solver_preserves_budget_and_cancellation_terminal_states() {
    let store = TypeStore::new();
    let mut solver = RecordRowSolver::new();
    let variable = solver.fresh_var();
    let left = RecordRowTerm {
        fields: Box::new([]),
        tail: RecordRowTermTail::Var(variable),
    };
    let right = RecordRowTerm {
        fields: Box::new([]),
        tail: RecordRowTermTail::Closed,
    };
    let mut budget = QueryBudget::new(0);
    assert!(matches!(
        solver.solve(&left, &right, &store, &mut budget, &CancellationToken::new()),
        phalcom_semantic::types::row_solver::RecordRowSolveResult::BudgetExceeded(_)
    ));

    let token = CancellationToken::new();
    token.cancel();
    let mut budget = QueryBudget::default();
    assert!(matches!(
        solver.solve(&left, &right, &store, &mut budget, &token),
        phalcom_semantic::types::row_solver::RecordRowSolveResult::Cancelled
    ));
}
