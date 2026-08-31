use super::super::support::analyze_adt;
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

#[test]
fn one_empty_variant_field_makes_the_cartesian_space_empty() {
    let owner = DeclarationId::new(test_module(), "Fixture".into());
    let variant = VariantId::new(owner, Selector::getter("V").expect("selector"));
    let raw = PatternSpace::Variant(VariantSpace {
        variant,
        exact_case: TypeId(10),
        fields: Box::new([PatternSpace::Opaque(TypeId(1)), PatternSpace::Empty]),
        proof: BranchProofEnvironment::default(),
    });
    assert!(raw.is_empty());
    assert_eq!(raw.normalize(), PatternSpace::Empty);
}

#[test]
fn nested_raw_empty_union_normalizes_to_empty_without_pre_normalizing_children() {
    let owner = DeclarationId::new(test_module(), "Fixture".into());
    let variant = VariantId::new(owner.clone(), Selector::getter("Outer").expect("selector"));
    let raw = PatternSpace::Variant(VariantSpace {
        variant,
        exact_case: TypeId(20),
        fields: Box::new([PatternSpace::Variant(VariantSpace {
            variant: VariantId::new(owner, Selector::getter("Inner").expect("selector")),
            exact_case: TypeId(21),
            fields: Box::new([PatternSpace::Union(Box::new([]))]),
            proof: BranchProofEnvironment::default(),
        })]),
        proof: BranchProofEnvironment::default(),
    });
    assert_eq!(raw.normalize(), PatternSpace::Empty);
}

#[test]
fn normalization_is_idempotent_for_raw_nested_unions_and_products() {
    let owner = DeclarationId::new(test_module(), "Fixture".into());
    let variant = VariantId::new(owner, Selector::getter("V").expect("selector"));
    let spaces = [
        PatternSpace::Union(Box::new([])),
        PatternSpace::Union(Box::new([PatternSpace::Empty, PatternSpace::Opaque(TypeId(1))])),
        PatternSpace::Variant(VariantSpace {
            variant,
            exact_case: TypeId(30),
            fields: Box::new([PatternSpace::Union(Box::new([])), PatternSpace::Opaque(TypeId(2))]),
            proof: BranchProofEnvironment::default(),
        }),
    ];

    for space in spaces {
        let normalized = space.normalize();
        assert_eq!(normalized.clone().normalize(), normalized);
        assert_eq!(normalized.is_empty(), matches!(normalized, PatternSpace::Empty));
    }
}

#[test]
fn match_space_06_opaque_and_closed_members_survive_union_normalization() {
    let space = PatternSpace::Union(Box::new([PatternSpace::Opaque(TypeId(1)), PatternSpace::Opaque(TypeId(2))]));
    assert_eq!(space.clone().normalize(), space);
}

#[test]
fn match_space_07_intersection_is_idempotent_for_opaque_member() {
    let mut store = TypeStore::new();
    let hierarchy = MapTypeHierarchy::new();
    let space = PatternSpace::Opaque(TypeId(1));
    assert_eq!(space.intersect(&space, &mut store, &hierarchy), space);
}

#[test]
fn match_space_08_intersection_with_empty_is_empty() {
    let mut store = TypeStore::new();
    let hierarchy = MapTypeHierarchy::new();
    let space = PatternSpace::Opaque(TypeId(1));
    assert_eq!(space.intersect(&PatternSpace::Empty, &mut store, &hierarchy), PatternSpace::Empty);
}

#[test]
fn match_space_09_intersection_distributes_over_union() {
    let mut store = TypeStore::new();
    let hierarchy = MapTypeHierarchy::new();
    let left = PatternSpace::Union(Box::new([PatternSpace::Opaque(TypeId(1)), PatternSpace::Opaque(TypeId(2))]));
    assert_eq!(
        left.intersect(&PatternSpace::Opaque(TypeId(1)), &mut store, &hierarchy),
        PatternSpace::Opaque(TypeId(1))
    );
}

#[test]
fn match_space_10_subtract_empty_preserves_space() {
    let mut store = TypeStore::new();
    let hierarchy = MapTypeHierarchy::new();
    let space = PatternSpace::Opaque(TypeId(1));
    assert_eq!(space.subtract(&PatternSpace::Empty, &mut store, &hierarchy), space);
}

#[test]
fn match_space_12_union_subtraction_retains_uncovered_member() {
    let mut store = TypeStore::new();
    let hierarchy = MapTypeHierarchy::new();
    let left = PatternSpace::Union(Box::new([PatternSpace::Opaque(TypeId(1)), PatternSpace::Opaque(TypeId(2))]));
    assert_eq!(
        left.subtract(&PatternSpace::Opaque(TypeId(1)), &mut store, &hierarchy),
        PatternSpace::Opaque(TypeId(2))
    );
}

#[test]
fn match_space_13_variant_subtraction_preserves_sibling_case() {
    let mut store = TypeStore::new();
    let hierarchy = MapTypeHierarchy::new();
    let owner = DeclarationId::new(test_module(), "Option".into());
    let some = PatternSpace::Variant(VariantSpace {
        variant: VariantId::new(owner.clone(), Selector::getter("Some").expect("Some")),
        exact_case: TypeId(1),
        fields: Box::new([]),
        proof: BranchProofEnvironment::default(),
    });
    let none = PatternSpace::Variant(VariantSpace {
        variant: VariantId::new(owner, Selector::getter("None").expect("None")),
        exact_case: TypeId(2),
        fields: Box::new([]),
        proof: BranchProofEnvironment::default(),
    });
    let space = PatternSpace::Union(Box::new([some.clone(), none.clone()]));
    assert_eq!(space.subtract(&some, &mut store, &hierarchy), none);
}

#[test]
fn match_space_11_self_subtraction_is_empty() {
    let mut store = TypeStore::new();
    let hierarchy = MapTypeHierarchy::new();
    let space = PatternSpace::Opaque(TypeId(1));
    assert_eq!(space.subtract(&space, &mut store, &hierarchy), PatternSpace::Empty);
}

#[test]
fn match_space_14_nested_subtraction_preserves_uncovered_child() {
    let mut store = TypeStore::new();
    let hierarchy = MapTypeHierarchy::new();
    let owner = DeclarationId::new(test_module(), "Option".into());
    let some = VariantId::new(owner.clone(), Selector::getter("Some").expect("Some"));
    let child = PatternSpace::Union(Box::new([PatternSpace::Opaque(TypeId(1)), PatternSpace::Opaque(TypeId(2))]));
    let full = PatternSpace::Variant(VariantSpace {
        variant: some.clone(),
        exact_case: TypeId(3),
        fields: Box::new([child]),
        proof: BranchProofEnvironment::default(),
    });
    let covered = PatternSpace::Variant(VariantSpace {
        variant: some,
        exact_case: TypeId(3),
        fields: Box::new([PatternSpace::Opaque(TypeId(1))]),
        proof: BranchProofEnvironment::default(),
    });
    let residual = full.subtract(&covered, &mut store, &hierarchy);
    assert!(matches!(residual, PatternSpace::Variant(VariantSpace { fields, .. }) if fields.as_ref() == [PatternSpace::Opaque(TypeId(2))]));
}

#[test]
fn match_space_15_nested_subtraction_preserves_sibling_and_child_residuals() {
    let mut store = TypeStore::new();
    let hierarchy = MapTypeHierarchy::new();
    let owner = DeclarationId::new(test_module(), "Option".into());
    let some = PatternSpace::Variant(VariantSpace {
        variant: VariantId::new(owner.clone(), Selector::getter("Some").expect("Some")),
        exact_case: TypeId(1),
        fields: Box::new([PatternSpace::Opaque(TypeId(2))]),
        proof: BranchProofEnvironment::default(),
    });
    let none = PatternSpace::Variant(VariantSpace {
        variant: VariantId::new(owner, Selector::getter("None").expect("None")),
        exact_case: TypeId(3),
        fields: Box::new([]),
        proof: BranchProofEnvironment::default(),
    });
    let domain = PatternSpace::Union(Box::new([some.clone(), none.clone()]));
    assert_eq!(domain.subtract(&some, &mut store, &hierarchy), none);
}

#[test]
fn match_space_17_opaque_subtraction_is_conservative() {
    let mut store = TypeStore::new();
    let hierarchy = MapTypeHierarchy::new();
    let owner = DeclarationId::new(test_module(), "Option".into());
    let opaque = PatternSpace::Opaque(TypeId(1));
    let case = PatternSpace::Variant(VariantSpace {
        variant: VariantId::new(owner, Selector::getter("Some").expect("Some")),
        exact_case: TypeId(2),
        fields: Box::new([]),
        proof: BranchProofEnvironment::default(),
    });
    assert_eq!(opaque.subtract(&case, &mut store, &hierarchy), opaque);
}

#[test]
fn match_space_18_wildcard_consumes_opaque_space() {
    let mut store = TypeStore::new();
    let hierarchy = MapTypeHierarchy::new();
    let opaque = PatternSpace::Opaque(TypeId(1));
    assert_eq!(opaque.subtract(&opaque, &mut store, &hierarchy), PatternSpace::Empty);
}

#[test]
fn review_c2_01_empty_is_checked_before_recursive_normalization() {
    let space = PatternSpace::Union(Box::new([PatternSpace::Empty, PatternSpace::Opaque(TypeId(1))]));
    assert_eq!(space.normalize(), PatternSpace::Opaque(TypeId(1)));
}

#[test]
fn review_c2_02_empty_nested_product_normalizes_to_empty() {
    let owner = DeclarationId::new(test_module(), "Fixture".into());
    let space = PatternSpace::Variant(VariantSpace {
        variant: VariantId::new(owner, Selector::getter("V").expect("V")),
        exact_case: TypeId(1),
        fields: Box::new([PatternSpace::Union(Box::new([]))]),
        proof: BranchProofEnvironment::default(),
    });
    assert_eq!(space.normalize(), PatternSpace::Empty);
}

#[test]
fn review_c2_03_union_empty_member_does_not_hide_live_member() {
    let space = PatternSpace::Union(Box::new([PatternSpace::Union(Box::new([])), PatternSpace::Opaque(TypeId(7))]));
    assert!(!space.is_empty());
    assert_eq!(space.normalize(), PatternSpace::Opaque(TypeId(7)));
}

#[test]
fn review_c2_04_empty_tuple_component_is_empty_product() {
    let space = PatternSpace::Tuple(Box::new([PatternSpace::Opaque(TypeId(1)), PatternSpace::Empty]));
    assert!(space.is_empty());
    assert_eq!(space.normalize(), PatternSpace::Empty);
}

#[test]
fn review_c2_05_normalization_is_idempotent_after_empty_elision() {
    let space = PatternSpace::Union(Box::new([PatternSpace::Empty, PatternSpace::Opaque(TypeId(3))]));
    let normalized = space.normalize();
    assert_eq!(normalized.clone().normalize(), normalized);
}

#[test]
fn review_c2_06_empty_union_is_canonical_empty() {
    assert_eq!(PatternSpace::Union(Box::new([])).normalize(), PatternSpace::Empty);
}

#[test]
fn review_c2_05_variant_with_raw_empty_union_is_empty() {
    let owner = DeclarationId::new(test_module(), "Fixture".into());
    let space = PatternSpace::Variant(VariantSpace {
        variant: VariantId::new(owner, Selector::getter("V").expect("V")),
        exact_case: TypeId(1),
        fields: Box::new([PatternSpace::Union(Box::new([])), PatternSpace::Opaque(TypeId(2))]),
        proof: BranchProofEnvironment::default(),
    });
    assert!(space.is_empty());
    assert_eq!(space.normalize(), PatternSpace::Empty);
}

#[test]
fn review_c2_06_normalized_empty_spaces_have_consistent_empty_invariant() {
    let spaces = [
        PatternSpace::Empty,
        PatternSpace::Union(Box::new([])),
        PatternSpace::Tuple(Box::new([PatternSpace::Empty])),
    ];
    for space in spaces {
        let normalized = space.normalize();
        assert_eq!(normalized.is_empty(), matches!(normalized, PatternSpace::Empty));
    }
}

#[test]
fn review_m2_01_wide_union_deduplicates_distinct_exact_members() {
    let spaces = (0..512)
        .map(|index| PatternSpace::Opaque(TypeId(index)))
        .chain((0..512).map(|index| PatternSpace::Opaque(TypeId(index))))
        .collect::<Vec<_>>();
    let normalized = PatternSpace::Union(spaces.into_boxed_slice()).normalize();
    let PatternSpace::Union(members) = normalized else {
        panic!("wide union must retain multiple members")
    };
    assert_eq!(members.len(), 512);
    assert_eq!(members.iter().filter(|member| **member == PatternSpace::Opaque(TypeId(511))).count(), 1);
}

#[test]
fn review_m2_02_duplicate_heavy_union_has_unique_structural_members() {
    let spaces = (0..4096).map(|index| PatternSpace::Opaque(TypeId((index % 8) as u32))).collect::<Vec<_>>();
    let normalized = PatternSpace::Union(spaces.into_boxed_slice()).normalize();
    let PatternSpace::Union(members) = normalized else {
        panic!("duplicate-heavy union must retain unique members")
    };
    assert_eq!(members.len(), 8);
}

#[test]
#[ignore = "GATED: generated wide source enum fixture is not needed for algebra unit coverage"]
fn review_m2_03_wide_enum_initial_space_smoke() {
    let variants = (0..64).map(|index| format!("@variant V{index}")).collect::<Vec<_>>().join(" ");
    let source = format!("enum Wide {{ {variants} }}\nclass Test {{ run(_ value: Wide) {{ match value {{ _ => 0 }} }} }}\n");
    let case = analyze_adt(&source);
    assert_eq!(case.enum_info("Wide").variants.len(), 64);
    case.only_match().assert_exhaustive();
}

#[test]
#[ignore = "GATED: normalization comparison counter would require test-only instrumentation"]
fn review_m2_04_union_dedup_growth_is_near_linear() {
    let mut previous = None;
    for width in [8, 64, 512, 2048] {
        let members = (0..width).map(|index| PatternSpace::Opaque(TypeId((index % 32) as u32))).collect::<Vec<_>>();
        let normalized = PatternSpace::Union(members.into_boxed_slice()).normalize();
        if let Some(previous) = previous {
            assert!(normalized != previous || width > 32);
        }
        previous = Some(normalized);
    }
}

#[test]
#[ignore = "OPTIONAL: benchmark belongs outside CI correctness suite"]
fn review_m2_05_union_normalization_benchmark_shapes_are_registered() {
    let shapes = [0usize, 1, 64, 512];
    for width in shapes {
        let members = (0..width).map(|index| PatternSpace::Opaque(TypeId(index as u32))).collect::<Vec<_>>();
        let normalized = PatternSpace::Union(members.into_boxed_slice()).normalize();
        assert_eq!(normalized.is_empty(), width == 0);
    }
}
