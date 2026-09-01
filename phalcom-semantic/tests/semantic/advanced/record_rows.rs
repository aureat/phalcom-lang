use phalcom_modules::identity::ModuleId;
use phalcom_semantic::identity::DeclarationId;
use phalcom_semantic::types::id::KindId;
use phalcom_semantic::types::parameter::{TypeParameterData, TypeParameterOwner};
use phalcom_semantic::types::relation::{MapTypeHierarchy, is_subtype};
use phalcom_semantic::types::row::{DuplicateFieldError, RecordRowData, RecordRowField, RecordRowTail};
use phalcom_semantic::types::row_solver::{RecordRowFailure, RecordRowSolver, RecordRowTerm};
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

    let empty_row = store.intern_record_row(RecordRowData {
        fields: Box::new([]),
        tail: RecordRowTail::Closed,
    });

    let single_row = store.intern_record_row(RecordRowData {
        fields: Box::new([RecordRowField {
            name: "name".into(),
            ty: str_ty,
        }]),
        tail: RecordRowTail::Closed,
    });

    let mut solver = RecordRowSolver::new(100);
    let r_var = solver.fresh_var();

    // #{ name: String } = #{ name: String | R }
    let left = RecordRowTerm::Canonical(single_row);
    let right = RecordRowTerm::Extend {
        fields: vec![RecordRowField {
            name: "name".into(),
            ty: str_ty,
        }],
        tail: Box::new(RecordRowTerm::Var(r_var)),
    };

    let result = solver.solve(&left, &right, &store);
    assert!(matches!(result, phalcom_semantic::types::row_solver::RecordRowSolveResult::Solved(sol) if {
        sol.substitutions.get(&r_var) == Some(&RecordRowTerm::Canonical(empty_row))
    }));
}

#[test]
fn test_lacks_constraint_blocks_duplicate_extension() {
    let mut store = TypeStore::new();
    let str_ty = store.nominal(test_decl("String"));

    let row = store.intern_record_row(RecordRowData {
        fields: Box::new([RecordRowField {
            name: "name".into(),
            ty: str_ty,
        }]),
        tail: RecordRowTail::Closed,
    });

    let mut solver = RecordRowSolver::new(100);
    let r_var = solver.fresh_var();
    solver.add_lacks(r_var, "name".into());

    let left = RecordRowTerm::Var(r_var);
    let right = RecordRowTerm::Canonical(row);

    let result = solver.solve(&left, &right, &store);
    assert!(matches!(
        result,
        phalcom_semantic::types::row_solver::RecordRowSolveResult::Rejected(RecordRowFailure::LacksViolation { .. })
    ));
}

#[test]
fn test_occurs_check_rejects() {
    let mut store = TypeStore::new();
    let int_ty = store.nominal(test_decl("Int"));

    let mut solver = RecordRowSolver::new(100);
    let r_var = solver.fresh_var();

    // R = #{ next: Int | R }
    let left = RecordRowTerm::Var(r_var);
    let right = RecordRowTerm::Extend {
        fields: vec![RecordRowField {
            name: "next".into(),
            ty: int_ty,
        }],
        tail: Box::new(RecordRowTerm::Var(r_var)),
    };

    let result = solver.solve(&left, &right, &store);
    assert!(matches!(
        result,
        phalcom_semantic::types::row_solver::RecordRowSolveResult::Rejected(RecordRowFailure::OccursCheckFailed { .. })
    ));
}

#[test]
fn test_domain_safety_record_row_type_parameter_never_produces_type_data_parameter() {
    let mut store = TypeStore::new();
    let decl = test_decl("Container");
    let param_id = store.intern_type_parameter(TypeParameterData::new(TypeParameterOwner::Declaration(decl), 0, "R", KindId::RECORD_ROW));

    // Attempting to create TypeData::Parameter with RecordRow kind must panic / be rejected
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let mut local_store = TypeStore::new();
        local_store.parameter_form(param_id);
    }));
    assert!(result.is_err(), "Must panic when constructing TypeData::Parameter with KindId::RECORD_ROW");
}

#[test]
fn test_record_subtyping_read_only_width() {
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
