use phalcom_common::selector::Selector;
use phalcom_modules::identity::{ModuleId, ModulePath, ResolvedProjectId};
use phalcom_semantic::checker::{PatternSpace, VariantSpace};
use phalcom_semantic::identity::{DeclarationId, VariantId};
use phalcom_semantic::match_semantics::BranchProofEnvironment;
use phalcom_semantic::types::id::TypeId;
use phalcom_semantic::types::relation::MapTypeHierarchy;
use phalcom_semantic::types::store::TypeStore;

fn test_module() -> ModuleId {
    ModuleId::resolved(ResolvedProjectId::from_raw(42), ModulePath::root())
}

#[test]
fn pattern_space_normalization_flattens_and_removes_empty() {
    let space = PatternSpace::Union(Box::new([
        PatternSpace::Empty,
        PatternSpace::Union(Box::new([PatternSpace::Opaque(TypeId(1)), PatternSpace::Empty])),
        PatternSpace::Opaque(TypeId(2)),
    ]));

    let normalized = space.normalize();
    assert_eq!(
        normalized,
        PatternSpace::Union(Box::new([PatternSpace::Opaque(TypeId(1)), PatternSpace::Opaque(TypeId(2)),]))
    );
}

#[test]
fn disjoint_variants_intersect_to_empty() {
    let mut store = TypeStore::new();
    let hier = MapTypeHierarchy::new();
    let module = test_module();
    let opt_decl = DeclarationId::new(module, "Option".into());

    let some_var = VariantId::new(opt_decl.clone(), Selector::getter("Some").expect("selector"));
    let none_var = VariantId::new(opt_decl, Selector::getter("None").expect("selector"));

    let some_space = PatternSpace::Variant(VariantSpace {
        variant: some_var,
        exact_case: TypeId(10),
        fields: Box::new([PatternSpace::Opaque(TypeId(1))]),
        proof: BranchProofEnvironment::default(),
    });

    let none_space = PatternSpace::Variant(VariantSpace {
        variant: none_var,
        exact_case: TypeId(11),
        fields: Box::new([]),
        proof: BranchProofEnvironment::default(),
    });

    assert_eq!(some_space.intersect(&none_space, &mut store, &hier), PatternSpace::Empty);
}

#[test]
fn nested_variant_subtraction_computes_exact_residual() {
    let mut store = TypeStore::new();
    let hier = MapTypeHierarchy::new();
    let module = test_module();
    let opt_decl = DeclarationId::new(module.clone(), "Option".into());
    let res_decl = DeclarationId::new(module, "Result".into());

    let some_var = VariantId::new(opt_decl, Selector::getter("Some").expect("selector"));
    let ok_var = VariantId::new(res_decl.clone(), Selector::getter("Ok").expect("selector"));
    let err_var = VariantId::new(res_decl, Selector::getter("Error").expect("selector"));

    // Some(Ok(x) | Error(y))
    let full_payload = PatternSpace::Union(Box::new([
        PatternSpace::Variant(VariantSpace {
            variant: ok_var.clone(),
            exact_case: TypeId(20),
            fields: Box::new([PatternSpace::Opaque(TypeId(1))]),
            proof: BranchProofEnvironment::default(),
        }),
        PatternSpace::Variant(VariantSpace {
            variant: err_var.clone(),
            exact_case: TypeId(21),
            fields: Box::new([PatternSpace::Opaque(TypeId(2))]),
            proof: BranchProofEnvironment::default(),
        }),
    ]));

    let some_full = PatternSpace::Variant(VariantSpace {
        variant: some_var.clone(),
        exact_case: TypeId(10),
        fields: Box::new([full_payload]),
        proof: BranchProofEnvironment::default(),
    });

    // Pattern to subtract: Some(Ok(_))
    let some_ok = PatternSpace::Variant(VariantSpace {
        variant: some_var.clone(),
        exact_case: TypeId(10),
        fields: Box::new([PatternSpace::Variant(VariantSpace {
            variant: ok_var,
            exact_case: TypeId(20),
            fields: Box::new([PatternSpace::Opaque(TypeId(1))]),
            proof: BranchProofEnvironment::default(),
        })]),
        proof: BranchProofEnvironment::default(),
    });

    let residual = some_full.subtract(&some_ok, &mut store, &hier);

    // Residual should be exactly Some(Error(_))
    let expected_residual = PatternSpace::Variant(VariantSpace {
        variant: some_var,
        exact_case: TypeId(10),
        fields: Box::new([PatternSpace::Variant(VariantSpace {
            variant: err_var,
            exact_case: TypeId(21),
            fields: Box::new([PatternSpace::Opaque(TypeId(2))]),
            proof: BranchProofEnvironment::default(),
        })]),
        proof: BranchProofEnvironment::default(),
    });

    assert_eq!(residual, expected_residual);
}

#[test]
fn tuple_subtraction_computes_cartesian_difference() {
    let mut store = TypeStore::new();
    let hier = MapTypeHierarchy::new();

    let full_tuple = PatternSpace::Tuple(Box::new([PatternSpace::Opaque(TypeId(1)), PatternSpace::Opaque(TypeId(2))]));

    // Subtract identical tuple
    let sub1 = full_tuple.subtract(&full_tuple, &mut store, &hier);
    assert_eq!(sub1, PatternSpace::Empty);
}
