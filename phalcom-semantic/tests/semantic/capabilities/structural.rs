use crate::semantic::support::{Fixture, assert_refuted, assert_validated, known, tuple};
use phalcom_semantic::diagnostic::DiagnosticCode;
use phalcom_semantic::identity::DispatchSide;
use phalcom_semantic::types::evidence::EvidenceStatus;

/// LAW: nested products preserve exact leaf types.
#[test]
fn nested_tuple_composes_exact_constituent_facts() {
    let f = Fixture::new(
        r#"
class CellNum { @constructor new() {} }
class Probe {
  @class
  run() {
    let x = (1, ("hello", CellNum.new()))
  }
}
"#,
    );
    let int_ty = f.ty("Int");
    let string_ty = f.ty("String");
    let cell_num = f.ty("CellNum");
    let run = f.callable("Probe", "run", DispatchSide::Class);
    f.assert_knowledge(
        &f.binding(run, "x").current,
        &known(tuple([int_ty.into(), tuple([string_ty.into(), cell_num.into()])]))
            .established()
            .origin(phalcom_semantic::EvidenceOrigin::Syntax),
    );
    let outer = f.binding(run, "x").current.ty().expect("tuple type");
    match f.analysis.snapshot.store.get(outer) {
        phalcom_semantic::types::store::TypeData::Tuple(elements) => {
            assert_eq!(elements.len(), 2);
            assert_eq!(elements[0].ty, int_ty);
            f.assert_tuple_types(elements[1].ty, &[string_ty, cell_num]);
        }
        other => panic!("expected nested tuple, got {other:?}"),
    }
}

/// LAW: broad tuple contract validates each precise product component.
/// LAW: heterogeneous literals retain a precise element union.
#[test]
fn tuple_supertype_annotation_preserves_specific_product_fact() {
    let f = Fixture::new(
        r#"
class Choice {}
class Present is Choice { @constructor new() {} }
class Probe {
  @class
  run() {
    let x: (Number, Choice) = (1, Present.new())
  }
}
"#,
    );
    let number = f.ty("Number");
    let option = f.ty("Choice");
    let int_ty = f.ty("Int");
    let some = f.ty("Present");
    let run = f.callable("Probe", "run", DispatchSide::Class);
    let x = f.binding(run, "x");
    let declared = x.declared.expect("declared tuple contract");
    f.assert_tuple_types(declared, &[number, option]);
    let current = x.current.ty().expect("specific tuple knowledge");
    f.assert_tuple_types(current, &[int_ty, some]);
    assert_validated(x);
    f.assert_no_error_diagnostics();
}

/// LAW: product refutation retains actual component facts and owns one diagnostic.
/// LAW: record literals retain named structural field facts.
#[test]
fn tuple_component_refutation_preserves_actual_product_fact() {
    let f = Fixture::new(
        r#"
class Choice {}
class Present is Choice { @constructor new() {} }
class Probe {
  @class
  run() {
    let x: (Number, Choice) = ("bad", Present.new())
  }
}
"#,
    );
    let number = f.ty("Number");
    let option = f.ty("Choice");
    let string_ty = f.ty("String");
    let some = f.ty("Present");
    let run = f.callable("Probe", "run", DispatchSide::Class);
    let x = f.binding(run, "x");
    let declared = x.declared.expect("declared tuple");
    f.assert_tuple_types(declared, &[number, option]);
    let actual = x.current.ty().expect("actual tuple");
    f.assert_tuple_types(actual, &[string_ty, some]);
    assert_refuted(x, actual, declared);
    assert_eq!(f.diagnostics(DiagnosticCode::BindingInitializerMismatch).len(), 1);
    f.assert_only_error_codes(&[DiagnosticCode::BindingInitializerMismatch]);
}

/// LAW: branch product results retain constituent precision.
#[test]
fn branch_product_results_preserve_component_precision() {
    let f = Fixture::new(
        r#"
class Probe {
  @class
  run(_ flag: Bool) {
    let x = if flag {
      (1, "a")
    } else {
      (2, "b")
    }
  }
}
"#,
    );
    let int_ty = f.ty("Int");
    let string_ty = f.ty("String");
    let run = f.callable("Probe", "run", DispatchSide::Class);
    let x = f.binding(run, "x");
    f.assert_tuple_types(x.current.ty().expect("joined product type"), &[int_ty, string_ty]);
    f.assert_no_error_diagnostics();
}

#[test]
fn heterogeneous_collection_infers_union_element_type() {
    let f = Fixture::new(
        r#"
class Probe {
  @class
  run() {
    let xs = [1, "hello"]
  }
}
"#,
    );
    let int_ty = f.ty("Int");
    let string_ty = f.ty("String");
    let run = f.callable("Probe", "run", DispatchSide::Class);
    let list_ty = f.binding(run, "xs").current.ty().expect("list type");
    match f.analysis.snapshot.store.get(list_ty) {
        phalcom_semantic::types::store::TypeData::Applied { arguments, .. } => {
            assert_eq!(arguments.len(), 1);
            f.assert_union_members(arguments[0], &[int_ty, string_ty]);
        }
        other => panic!("expected List<...>, got {other:?}"),
    }
}

#[test]
fn record_literal_preserves_structural_field_types() {
    let f = Fixture::new(
        r#"
class Probe {
  @class
  run() {
    let user = #{name: "A", age: 42}
  }
}
"#,
    );
    let run = f.callable("Probe", "run", DispatchSide::Class);
    let user = f.binding(run, "user");
    let record_ty = user.current.ty().expect("record type");
    f.assert_record(record_ty);
    assert_eq!(user.current.status(), Some(EvidenceStatus::Established));
    f.assert_no_error_diagnostics();
}

/// LAW: tuple decomposition creates independent component bindings.
#[test]
fn tuple_destructuring_establishes_independent_component_bindings() {
    let f = Fixture::new(
        r#"
class Probe {
  @class
  run() {
    let (x, y) = (1, "hello")
  }
}
"#,
    );
    let int_ty = f.ty("Int");
    let string_ty = f.ty("String");
    let run = f.callable("Probe", "run", DispatchSide::Class);
    let x = f.binding(run, "x");
    let y = f.binding(run, "y");
    assert_ne!(x.binding, y.binding);
    assert_eq!(x.current.ty(), Some(int_ty));
    assert_eq!(y.current.ty(), Some(string_ty));
    assert_eq!(x.current.status(), Some(EvidenceStatus::Established));
    assert_eq!(y.current.status(), Some(EvidenceStatus::Established));
    f.assert_no_error_diagnostics();
}

/// LAW: destructuring reads current product components, not broad declaration components.
#[test]
fn tuple_destructuring_with_broad_contract_keeps_specific_components() {
    let f = Fixture::new(
        r#"
class Choice {}
class Present is Choice { @constructor new() {} }
class Probe {
  @class
  run() {
    let pair: (Number, Choice) = (1, Present.new())
    let (x, y) = pair
  }
}
"#,
    );
    let int_ty = f.ty("Int");
    let some = f.ty("Present");
    let run = f.callable("Probe", "run", DispatchSide::Class);
    f.assert_binding_type(run, "x", int_ty);
    f.assert_binding_type(run, "y", some);
}
