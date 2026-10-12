use crate::semantic::support::{Fixture, TypeExpectation, applied, assert_refuted, assert_validated, known, nominal, tuple};
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
            assert_eq!(elements.elements.len(), 2);
            assert_eq!(elements.elements[0].ty, int_ty);
            f.assert_tuple_types(elements.elements[1].ty, &[string_ty, cell_num]);
        }
        other => panic!("expected nested tuple, got {other:?}"),
    }
}

/// LAW: broad tuple contract validates each precise product component.
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
    let declared = x.declared_type().expect("declared tuple contract");
    f.assert_tuple_types(declared, &[number, option]);
    let current = x.current.ty().expect("specific tuple knowledge");
    f.assert_tuple_types(current, &[int_ty, some]);
    assert_validated(x);
    f.assert_no_error_diagnostics();
}

/// LAW: product refutation retains actual component facts and owns one diagnostic.
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
    let declared = x.declared_type().expect("declared tuple");
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

/// LAW: heterogeneous collection literals retain the exact element union.
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
            f.assert_union_members(arguments[0].as_type_id().expect("type-only collection argument"), &[int_ty, string_ty]);
        }
        other => panic!("expected List<...>, got {other:?}"),
    }
}

/// LAW: contextual product components guide nested empty literals without
/// laundering contextual selection into established syntax evidence.
#[test]
fn contextual_product_components_propagate_to_nested_literals() {
    let f = Fixture::new(
        r#"
class Probe {
  @class
  run() {
    let pair: (List<Int>, Map<Symbol, List<Int>>) = ([], Map {key: []})
  }
}
"#,
    );
    let run = f.callable("Probe", "run", DispatchSide::Class);
    let pair = f.expression(run, "([], Map {key: []})");
    let pair_ty = pair.knowledge.ty().expect("contextual tuple result");
    f.assert_type(
        pair_ty,
        tuple([
            applied("List", [nominal("Int")]),
            applied("Map", [nominal("Symbol"), applied("List", [nominal("Int")])]),
        ]),
    );
    assert_eq!(pair.knowledge.status(), Some(EvidenceStatus::Assumed));
}

/// LAW: ordinary membership is an RHS-owned `contains(_)` send and keeps Bool
/// result typing for both polarities.
#[test]
fn membership_is_owned_by_rhs_contains_protocol() {
    let f = Fixture::new(
        r#"
class Container {
  contains(_ value: Int) -> Bool { true }
}
class Probe {
  @class
  run(_ container: Container) {
    let present = 1 in container
    let absent = 1 not in container
  }
}
"#,
    );
    let run = f.callable("Probe", "run", DispatchSide::Class);
    assert_eq!(f.expression(run, "1 in container").knowledge.ty(), Some(f.ty("Bool")));
    assert_eq!(f.expression(run, "1 not in container").knowledge.ty(), Some(f.ty("Bool")));
}

/// LAW: lifted type membership is a Bool result and does not turn its
/// candidate expression into a value-type fact.
#[test]
fn lifted_type_membership_publishes_bool() {
    let f = Fixture::new(
        r#"
class Probe {
  @class
  run(_ value: Object) {
    let result = value is in [Object]
  }
}
"#,
    );
    let run = f.callable("Probe", "run", DispatchSide::Class);
    assert_eq!(f.expression(run, "value is in [Object]").knowledge.ty(), Some(f.ty("Bool")));
}

/// LAW: record literals publish a closed structural row with the exact field
/// names and leaf types, not merely an opaque `Record` category.
#[test]
fn record_literal_preserves_structural_field_types() {
    let f = Fixture::new(
        r#"
class Probe {
  @class
  run() {
    let user = {name: "A", age: 42}
  }
}
"#,
    );
    let string_ty = f.ty("String");
    let int_ty = f.ty("Int");
    let run = f.callable("Probe", "run", DispatchSide::Class);
    let user = f.binding(run, "user");
    let record_ty = user.current.ty().expect("record type");
    f.assert_type(
        record_ty,
        TypeExpectation::Record(vec![("name".into(), string_ty.into()), ("age".into(), int_ty.into())]),
    );
    assert_eq!(user.current.status(), Some(EvidenceStatus::Established));
    assert_eq!(user.current.origin(), Some(phalcom_semantic::EvidenceOrigin::Syntax));
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
    assert_eq!(x.current.origin(), Some(phalcom_semantic::EvidenceOrigin::PatternDecomposition));
    assert_eq!(y.current.origin(), Some(phalcom_semantic::EvidenceOrigin::PatternDecomposition));
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
    assert_eq!(
        f.binding(run, "x").current.origin(),
        Some(phalcom_semantic::EvidenceOrigin::PatternDecomposition)
    );
    assert_eq!(
        f.binding(run, "y").current.origin(),
        Some(phalcom_semantic::EvidenceOrigin::PatternDecomposition)
    );
}

fn analyze_universe_client(source: &str) -> (phalcom_semantic::SemanticAnalysis, phalcom_modules::ModuleId) {
    let root = tempfile::tempdir().expect("workspace tempdir");
    std::fs::write(root.path().join("package.ph"), "").expect("workspace package marker");
    let path = root.path().join("main.ph");
    let location = phalcom_modules::SourceLocation {
        source_id: phalcom_modules::SourceId(path.to_string_lossy().into_owned().into()),
        display_path: path,
    };
    let source: std::sync::Arc<str> = std::sync::Arc::from(source);
    let publication = phalcom_semantic::session::SemanticWorkspaceSession::new()
        .apply_module_mutations([phalcom_modules::WorkspaceSourceBatchMutation::SetOverlay {
            source: location.clone(),
            text: std::sync::Arc::clone(&source),
            revision: phalcom_modules::SourceRevision(1),
            recovered_program: None,
        }])
        .expect("canonical Universe workspace analysis");
    let module = publication
        .snapshot
        .module_for_source(&location.source_id)
        .expect("user module identity")
        .clone();
    (
        phalcom_semantic::SemanticAnalysis {
            snapshot: publication.snapshot,
        },
        module,
    )
}

/// LAW: explicit implementation of canonical Product is rejected as compiler-derived.
#[test]
fn explicit_impl_of_canonical_product_is_rejected() {
    let (f, module) = analyze_universe_client(
        r#"
from universe.reflection.structural import Product

class Foo {}
impl Product for Foo {}
"#,
    );
    let diags = f.snapshot.diagnostics_for(&module).unwrap_or(&[]);
    assert_eq!(
        diags.iter().map(|d| d.code).collect::<Vec<_>>(),
        vec![DiagnosticCode::ConformanceCompilerDerivedTrait]
    );
}

/// LAW: explicit implementation of canonical Sum is rejected as compiler-derived.
#[test]
fn explicit_impl_of_canonical_sum_is_rejected() {
    let (f, module) = analyze_universe_client(
        r#"
from universe.reflection.structural import Sum

class Foo {}
impl Sum for Foo {}
"#,
    );
    let diags = f.snapshot.diagnostics_for(&module).unwrap_or(&[]);
    assert_eq!(
        diags.iter().map(|d| d.code).collect::<Vec<_>>(),
        vec![DiagnosticCode::ConformanceCompilerDerivedTrait]
    );
}

/// LAW: user trait named Product is not rejected by canonical structural capability sealing.
#[test]
fn user_trait_named_product_is_allowed() {
    let f = Fixture::new(
        r#"
trait Product {}

class Foo {}
impl Product for Foo {}
"#,
    );
    f.assert_no_error_diagnostics();
}

/// LAW: compiler-derived structural capabilities conform according to the structural matrix.
#[test]
fn compiler_derived_structural_capabilities_conform_to_structural_matrix() {
    let f = Fixture::new(
        r#"
import universe.reflection.structural.Product
import universe.reflection.structural.OrderedProduct
import universe.reflection.structural.UnorderedProduct
import universe.reflection.structural.Sum

class Probe {
  @class
  run() {
    let u = ()
    let t = (1, "a")
    let r = Record {a: 1}
  }
}
"#,
    );
    let run = f.callable("Probe", "run", DispatchSide::Class);
    let unit_ty = f.binding(run, "u").current.ty().unwrap();
    let tuple_ty = f.binding(run, "t").current.ty().unwrap();
    let record_ty = f.binding(run, "r").current.ty().unwrap();
    let int_ty = f.ty("Int");
    let mut store = (*f.analysis.snapshot.store).clone();
    let pack_ty = store
        .pack(Box::new([
            phalcom_semantic::types::store::PackTypeElement { label: None, ty: int_ty },
        ]))
        .unwrap();

    let product_decl = f.decl("Product");
    let ordered_decl = f.decl("OrderedProduct");
    let unordered_decl = f.decl("UnorderedProduct");
    let sum_decl = f.decl("Sum");

    let product_ref = phalcom_semantic::traits::TraitRef::new(product_decl, Vec::new());
    let ordered_ref = phalcom_semantic::traits::TraitRef::new(ordered_decl, Vec::new());
    let unordered_ref = phalcom_semantic::traits::TraitRef::new(unordered_decl, Vec::new());
    let sum_ref = phalcom_semantic::traits::TraitRef::new(sum_decl, Vec::new());

    let check_proof = |subject: phalcom_semantic::types::id::TypeId, trait_ref: &phalcom_semantic::traits::TraitRef, store: &mut phalcom_semantic::types::store::TypeStore| {
        let term = phalcom_semantic::types::parameter::TypeTerm::Canonical(subject);
        phalcom_semantic::generic_evidence::query_generic_conformance_proof(
            &term,
            trait_ref,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Some(store),
        )
    };

    // Unit: Product=Proven, OrderedProduct=Proven, UnorderedProduct=Missing, Sum=Missing
    assert!(matches!(check_proof(unit_ty, &product_ref, &mut store), phalcom_semantic::generic_evidence::GenericConformanceProof::Proven(_)));
    assert!(matches!(check_proof(unit_ty, &ordered_ref, &mut store), phalcom_semantic::generic_evidence::GenericConformanceProof::Proven(_)));
    assert!(matches!(check_proof(unit_ty, &unordered_ref, &mut store), phalcom_semantic::generic_evidence::GenericConformanceProof::Missing));
    assert!(matches!(check_proof(unit_ty, &sum_ref, &mut store), phalcom_semantic::generic_evidence::GenericConformanceProof::Missing));

    // Tuple: Product=Proven, OrderedProduct=Proven, UnorderedProduct=Missing, Sum=Missing
    assert!(matches!(check_proof(tuple_ty, &product_ref, &mut store), phalcom_semantic::generic_evidence::GenericConformanceProof::Proven(_)));
    assert!(matches!(check_proof(tuple_ty, &ordered_ref, &mut store), phalcom_semantic::generic_evidence::GenericConformanceProof::Proven(_)));
    assert!(matches!(check_proof(tuple_ty, &unordered_ref, &mut store), phalcom_semantic::generic_evidence::GenericConformanceProof::Missing));
    assert!(matches!(check_proof(tuple_ty, &sum_ref, &mut store), phalcom_semantic::generic_evidence::GenericConformanceProof::Missing));

    // Pack: Product=Proven, OrderedProduct=Proven, UnorderedProduct=Missing, Sum=Missing
    assert!(matches!(check_proof(pack_ty, &product_ref, &mut store), phalcom_semantic::generic_evidence::GenericConformanceProof::Proven(_)));
    assert!(matches!(check_proof(pack_ty, &ordered_ref, &mut store), phalcom_semantic::generic_evidence::GenericConformanceProof::Proven(_)));
    assert!(matches!(check_proof(pack_ty, &unordered_ref, &mut store), phalcom_semantic::generic_evidence::GenericConformanceProof::Missing));
    assert!(matches!(check_proof(pack_ty, &sum_ref, &mut store), phalcom_semantic::generic_evidence::GenericConformanceProof::Missing));

    // Record: Product=Proven, OrderedProduct=Missing, UnorderedProduct=Proven, Sum=Missing
    assert!(matches!(check_proof(record_ty, &product_ref, &mut store), phalcom_semantic::generic_evidence::GenericConformanceProof::Proven(_)));
    assert!(matches!(check_proof(record_ty, &ordered_ref, &mut store), phalcom_semantic::generic_evidence::GenericConformanceProof::Missing));
    assert!(matches!(check_proof(record_ty, &unordered_ref, &mut store), phalcom_semantic::generic_evidence::GenericConformanceProof::Proven(_)));
    assert!(matches!(check_proof(record_ty, &sum_ref, &mut store), phalcom_semantic::generic_evidence::GenericConformanceProof::Missing));

    // Int: Product=Missing, OrderedProduct=Missing, UnorderedProduct=Missing, Sum=Missing
    assert!(matches!(check_proof(int_ty, &product_ref, &mut store), phalcom_semantic::generic_evidence::GenericConformanceProof::Missing));
    assert!(matches!(check_proof(int_ty, &ordered_ref, &mut store), phalcom_semantic::generic_evidence::GenericConformanceProof::Missing));
    assert!(matches!(check_proof(int_ty, &unordered_ref, &mut store), phalcom_semantic::generic_evidence::GenericConformanceProof::Missing));
    assert!(matches!(check_proof(int_ty, &sum_ref, &mut store), phalcom_semantic::generic_evidence::GenericConformanceProof::Missing));
}

/// G6, G8, G9, G10: Canonical compiler-derived structural reflection reduces
/// Product, OrderedProduct, UnorderedProduct, Sum, and UnorderedShape members.
#[test]
fn compiler_derived_structural_reflection_evaluates_shapes_and_variants() {
    let f = Fixture::new(
        r#"
data Color {
  @variant Red
  @variant Green(_ value: Int)
  @variant Blue(intensity: Int)
}

class Probe<T, S> {
  @class
  run(_ t_param: T, _ s_param: S) {
    let u = ()
    let t = (1, "a")
    let r = Record {a: 1}
  }
}
"#,
    );
    let run = f.callable("Probe", "run", DispatchSide::Class);
    let unit_ty = f.binding(run, "u").current.ty().unwrap();
    let tuple_ty = f.binding(run, "t").current.ty().unwrap();
    let record_ty = f.binding(run, "r").current.ty().unwrap();
    let t_param_ty = f.binding(run, "t_param").current.ty().unwrap();
    let s_param_ty = f.binding(run, "s_param").current.ty().unwrap();
    let color_decl = f.decl("Color");
    let color_ty = f.analysis.snapshot.declarations.form(&color_decl).unwrap();

    let mut store = (*f.analysis.snapshot.store).clone();
    let mut values = phalcom_semantic::static_value::StaticValueStore::new();

    let structural_ids = phalcom_semantic::core_surface::identity::StructuralUniverseIds::new();
    let classifier = phalcom_semantic::structural::StructuralClassifier::new(
        &f.analysis.snapshot.data_semantics,
        &f.analysis.snapshot.enum_semantics,
    );

    // Verify Sum conformance proof before creating projector
    let sum_decl = f.decl("Sum");
    let sum_ref = phalcom_semantic::traits::TraitRef::new(sum_decl, Vec::new());
    let term = phalcom_semantic::types::parameter::TypeTerm::Canonical(color_ty);
    let proof = phalcom_semantic::generic_evidence::query_generic_conformance_proof_with_structural(
        &term,
        &sum_ref,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        Some(&mut store),
        Some(phalcom_semantic::generic_evidence::StructuralConformanceContext {
            classifier: classifier.clone(),
            ids: &structural_ids,
        }),
    );
    assert!(matches!(proof, phalcom_semantic::generic_evidence::GenericConformanceProof::Proven(_)));

    let mut projector = phalcom_semantic::structural_reflection::StructuralReflectionProjector::new(
        classifier,
        &structural_ids,
        &mut store,
        &mut values,
        &f.analysis.snapshot.declarations,
        &f.analysis.snapshot.data_semantics,
        &f.analysis.snapshot.enum_semantics,
    );

    // G6: Ordered Product Reflection
    // Unit Product::Kind -> Ordered
    let unit_kind = projector.project_product_kind(unit_ty).expect("project unit kind");
    assert!(matches!(unit_kind, phalcom_semantic::member_projection::StaticMemberProjectionOutcome::ReducedValue(_)));
    if let phalcom_semantic::member_projection::StaticMemberProjectionOutcome::ReducedValue(val) = unit_kind {
        let phalcom_semantic::static_value::StaticValueData::Variant(var_id) = *projector.values.get(val).unwrap() else {
            panic!("expected variant for kind");
        };
        let var = projector.values.get_variant(var_id).unwrap();
        assert!(matches!(&var.variant.selector.base, phalcom_common::selector::SelectorBase::Named(n) if n == "Ordered"));
    }

    // Unit Product::Shape -> ProductShape::Ordered
    let unit_shape = projector.project_product_shape(unit_ty).expect("project unit shape");
    assert!(matches!(unit_shape, phalcom_semantic::member_projection::StaticMemberProjectionOutcome::ReducedValue(_)));

    // Tuple Product::Kind -> Ordered
    let tuple_kind = projector.project_product_kind(tuple_ty).expect("project tuple kind");
    assert!(matches!(tuple_kind, phalcom_semantic::member_projection::StaticMemberProjectionOutcome::ReducedValue(_)));
    if let phalcom_semantic::member_projection::StaticMemberProjectionOutcome::ReducedValue(val) = tuple_kind {
        let phalcom_semantic::static_value::StaticValueData::Variant(var_id) = *projector.values.get(val).unwrap() else {
            panic!("expected variant for kind");
        };
        let var = projector.values.get_variant(var_id).unwrap();
        assert!(matches!(&var.variant.selector.base, phalcom_common::selector::SelectorBase::Named(n) if n == "Ordered"));
    }

    // Tuple OrderedProduct::Entries -> Pack of 2 entries
    let tuple_entries = projector.project_ordered_product_member(tuple_ty, "Entries").expect("tuple entries");
    assert!(matches!(tuple_entries, phalcom_semantic::member_projection::StaticMemberProjectionOutcome::ReducedValue(_)));
    if let phalcom_semantic::member_projection::StaticMemberProjectionOutcome::ReducedValue(val) = tuple_entries {
        let phalcom_semantic::static_value::StaticValueData::Pack(pack_id) = *projector.values.get(val).unwrap() else {
            panic!("expected pack");
        };
        let pack = projector.values.get_pack(pack_id).unwrap();
        assert_eq!(pack.1.positionals.len(), 2);
    }

    // G8: Unordered Product Reflection
    // Record Product::Kind -> Unordered
    let rec_kind = projector.project_product_kind(record_ty).expect("project record kind");
    assert!(matches!(rec_kind, phalcom_semantic::member_projection::StaticMemberProjectionOutcome::ReducedValue(_)));
    if let phalcom_semantic::member_projection::StaticMemberProjectionOutcome::ReducedValue(val) = rec_kind {
        let phalcom_semantic::static_value::StaticValueData::Variant(var_id) = *projector.values.get(val).unwrap() else {
            panic!("expected variant for kind");
        };
        let var = projector.values.get_variant(var_id).unwrap();
        assert!(matches!(&var.variant.selector.base, phalcom_common::selector::SelectorBase::Named(n) if n == "Unordered"));
    }

    // Record Product::Shape -> ProductShape::Unordered(UnorderedShape)
    let rec_shape = projector.project_product_shape(record_ty).expect("project record shape");
    assert!(matches!(rec_shape, phalcom_semantic::member_projection::StaticMemberProjectionOutcome::ReducedValue(_)));

    // Record UnorderedProduct::UnorderedShape -> Data(UnorderedShape)
    let rec_u_shape = projector.project_unordered_product_member(record_ty, "UnorderedShape").expect("record unordered shape");
    assert!(matches!(rec_u_shape, phalcom_semantic::member_projection::StaticMemberProjectionOutcome::ReducedValue(_)));
    let phalcom_semantic::member_projection::StaticMemberProjectionOutcome::ReducedValue(shape_val) = rec_u_shape else { unreachable!() };

    // UnorderedShape::Lookup<'a'> -> Required(Int)
    let sym_a = projector.values.intern_symbol("a");
    let lookup_res = projector.project_unordered_shape_member(
        shape_val,
        phalcom_semantic::types::type_member::IntrinsicTypeOperation::UnorderedShapeLookup,
        &[phalcom_semantic::member_projection::MemberProjectionArgument::Value(sym_a)],
    ).expect("lookup a");
    assert!(matches!(lookup_res, phalcom_semantic::member_projection::StaticMemberProjectionOutcome::ReducedValue(_)));
    if let phalcom_semantic::member_projection::StaticMemberProjectionOutcome::ReducedValue(val) = lookup_res {
        let phalcom_semantic::static_value::StaticValueData::Variant(var_id) = *projector.values.get(val).unwrap() else {
            panic!("expected variant");
        };
        let var = projector.values.get_variant(var_id).unwrap();
        assert!(matches!(&var.variant.selector.base, phalcom_common::selector::SelectorBase::Named(n) if n == "Required"));
    }

    // UnorderedShape::Lookup<'b'> -> Absent
    let sym_b = projector.values.intern_symbol("b");
    let lookup_b = projector.project_unordered_shape_member(
        shape_val,
        phalcom_semantic::types::type_member::IntrinsicTypeOperation::UnorderedShapeLookup,
        &[phalcom_semantic::member_projection::MemberProjectionArgument::Value(sym_b)],
    ).expect("lookup b");
    assert!(matches!(lookup_b, phalcom_semantic::member_projection::StaticMemberProjectionOutcome::ReducedValue(_)));
    if let phalcom_semantic::member_projection::StaticMemberProjectionOutcome::ReducedValue(val) = lookup_b {
        let phalcom_semantic::static_value::StaticValueData::Variant(var_id) = *projector.values.get(val).unwrap() else {
            panic!("expected variant");
        };
        let var = projector.values.get_variant(var_id).unwrap();
        assert!(matches!(&var.variant.selector.base, phalcom_common::selector::SelectorBase::Named(n) if n == "Absent"));
    }

    // UnorderedShape::Tail -> Closed
    let tail_res = projector.project_unordered_shape_member(
        shape_val,
        phalcom_semantic::types::type_member::IntrinsicTypeOperation::UnorderedShapeTail,
        &[],
    ).expect("tail");
    assert!(matches!(tail_res, phalcom_semantic::member_projection::StaticMemberProjectionOutcome::ReducedValue(_)));
    if let phalcom_semantic::member_projection::StaticMemberProjectionOutcome::ReducedValue(val) = tail_res {
        let phalcom_semantic::static_value::StaticValueData::Variant(var_id) = *projector.values.get(val).unwrap() else {
            panic!("expected variant");
        };
        let var = projector.values.get_variant(var_id).unwrap();
        assert!(matches!(&var.variant.selector.base, phalcom_common::selector::SelectorBase::Named(n) if n == "Closed"));
    }

    // UnorderedShape::View -> Field
    let view_res = projector.project_unordered_shape_member(
        shape_val,
        phalcom_semantic::types::type_member::IntrinsicTypeOperation::UnorderedShapeView,
        &[],
    ).expect("view");
    assert!(matches!(view_res, phalcom_semantic::member_projection::StaticMemberProjectionOutcome::ReducedValue(_)));
    if let phalcom_semantic::member_projection::StaticMemberProjectionOutcome::ReducedValue(val) = view_res {
        let phalcom_semantic::static_value::StaticValueData::Variant(var_id) = *projector.values.get(val).unwrap() else {
            panic!("expected variant");
        };
        let var = projector.values.get_variant(var_id).unwrap();
        assert!(matches!(&var.variant.selector.base, phalcom_common::selector::SelectorBase::Named(n) if n == "Field"));
    }

    // G9: Sum / Variant Reflection
    let variants_res = projector.project_sum_variants(color_ty).expect("project sum variants");
    assert!(matches!(variants_res, phalcom_semantic::member_projection::StaticMemberProjectionOutcome::ReducedValue(_)));
    if let phalcom_semantic::member_projection::StaticMemberProjectionOutcome::ReducedValue(val) = variants_res {
        let phalcom_semantic::static_value::StaticValueData::Pack(pack_id) = *projector.values.get(val).unwrap() else {
            panic!("expected pack");
        };
        let pack = projector.values.get_pack(pack_id).unwrap();
        assert_eq!(pack.1.positionals.len(), 3);

        // Check variant names in order: Red, Green, Blue
        let expected_names = ["Red", "Green", "Blue"];
        for (i, &var_ref) in pack.1.positionals.iter().enumerate() {
            let phalcom_semantic::static_value::StaticValueData::Data(data_id) = *projector.values.get(var_ref).unwrap() else {
                panic!("expected Variant data");
            };
            let data = projector.values.get_data(data_id).unwrap();
            let name_val = data.components[0].1;
            let phalcom_semantic::static_value::StaticValueData::Symbol(sym) = projector.values.get(name_val).unwrap() else {
                panic!("expected symbol");
            };
            assert_eq!(projector.values.get_symbol(*sym).unwrap(), expected_names[i]);
        }
    }

    // G10: Generic / Stuck projections
    let stuck_kind = projector.project_product_kind(t_param_ty).expect("generic product kind");
    assert!(matches!(stuck_kind, phalcom_semantic::member_projection::StaticMemberProjectionOutcome::Stuck(_)));

    let stuck_sum = projector.project_sum_variants(s_param_ty).expect("generic sum variants");
    assert!(matches!(stuck_sum, phalcom_semantic::member_projection::StaticMemberProjectionOutcome::Stuck(_)));
}
