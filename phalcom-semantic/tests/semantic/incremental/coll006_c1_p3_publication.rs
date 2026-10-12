use super::support::{multi_module_input, single_module_input};
use phalcom_common::selector::Selector;
use phalcom_modules::identity::{ModuleComponent, ModuleId, ModulePath, ResolvedProjectId};
use phalcom_modules::linker::SymbolId;
use phalcom_semantic::checker::analysis::SequenceLiteralFamily;
use phalcom_semantic::identity::{CallableId, DeclarationId, DispatchSide};
use phalcom_semantic::materialization::{MaterializationDescriptor, SequenceMaterializationKind};
use phalcom_semantic::module_bindings::ModuleBindingPhase;
use phalcom_semantic::session::SemanticWorkspaceSession;
use phalcom_semantic::static_constant::{StaticConstantId, StaticConstantProduct};
use phalcom_semantic::static_value::{StaticInt, StaticSequenceFamily, StaticValueData};
use std::sync::Arc;

fn module(name: &str) -> ModuleId {
    ModuleId::resolved(
        ResolvedProjectId::from_raw(606),
        ModulePath::from_components(vec![ModuleComponent::from_identifier(name).unwrap()]),
    )
}

fn run_id(module: &ModuleId) -> CallableId {
    CallableId::new(
        DeclarationId::new(module.clone(), "Consumer".into()),
        Selector::method("run", []).unwrap(),
        DispatchSide::Class,
    )
}

fn sequence_meanings(
    snapshot: &phalcom_semantic::snapshot::SemanticSnapshot,
    callable: &CallableId,
) -> Vec<(SequenceLiteralFamily, String, String, Option<String>)> {
    let analysis = snapshot.callable_analyses.get(callable).expect("run analysis");
    analysis
        .sequence_literals
        .values()
        .map(|resolution| {
            (
                resolution.family,
                snapshot.store.format_type(resolution.result_type),
                snapshot.store.format_type(resolution.element_type),
                resolution.extent.map(|extent| snapshot.store.format_type(extent)),
            )
        })
        .collect()
}

fn source(failing: bool) -> String {
    let constraint = if failing { "String" } else { "Int" };
    format!(
        r#"
class Consumer {{
  @class reject<T>(_ values: Vector<T, 1>) -> Int where T == {constraint} {{ 0 }}
  @class run() {{
    let independent = <Int, 1>[9]
    let rejected = Consumer.reject(<Int, 1>[1])
  }}
}}
"#
    )
}

fn static_sequence_meaning(
    snapshot: &phalcom_semantic::snapshot::SemanticSnapshot,
    id: &StaticConstantId,
) -> Option<(StaticSequenceFamily, Vec<StaticValueData>, String)> {
    let product = snapshot.static_constant(id)?;
    let value = product.value()?;
    let StaticValueData::Sequence(sequence_id) = snapshot.static_values().get(value).ok()? else {
        return None;
    };
    let sequence = snapshot.static_values().get_sequence(*sequence_id).ok()?;
    let elements = sequence
        .elements()
        .iter()
        .map(|element| snapshot.static_values().get(*element).ok().cloned())
        .collect::<Option<Vec<_>>>()?;
    Some((sequence.family(), elements, snapshot.store.format_type(sequence.result_type())))
}

fn static_ints(values: &[i64]) -> Vec<StaticValueData> {
    values.iter().copied().map(|value| StaticValueData::Int(StaticInt::Small(value))).collect()
}

#[test]
fn publication_invalid_valid_invalid_matches_cold_and_reuses_unrelated_callable() {
    let main = module("publication");
    let other = module("other");
    let other_source = "class Other { @class stable() -> Int { 1 } }";
    let mut incremental = SemanticWorkspaceSession::new();

    let first = incremental.update(multi_module_input(vec![(main.clone(), source(true)), (other.clone(), other_source.into())], 1)).expect("semantic test update should succeed");
    let run = run_id(&main);
    let other_callable = CallableId::new(
        DeclarationId::new(other.clone(), "Other".into()),
        Selector::method("stable", []).unwrap(),
        DispatchSide::Class,
    );
    assert!(first.snapshot.has_errors());
    assert_eq!(sequence_meanings(&first.snapshot, &run).len(), 1);
    let other_first = first.snapshot.callable_analyses.get(&other_callable).expect("other callable").clone();

    let second = incremental.update(multi_module_input(vec![(main.clone(), source(false)), (other.clone(), other_source.into())], 2)).expect("semantic test update should succeed");
    assert!(!second.snapshot.has_errors(), "diagnostics: {:?}", second.snapshot.diagnostics);
    assert_eq!(sequence_meanings(&second.snapshot, &run).len(), 2);
    assert!(std::sync::Arc::ptr_eq(
        &other_first,
        second.snapshot.callable_analyses.get(&other_callable).expect("reused other callable"),
    ));

    let third = incremental.update(multi_module_input(vec![(main.clone(), source(true)), (other.clone(), other_source.into())], 3)).expect("semantic test update should succeed");
    assert!(third.snapshot.has_errors());
    assert_eq!(sequence_meanings(&third.snapshot, &run).len(), 1);

    for (generation, text, update) in [(1, source(true), &first), (2, source(false), &second), (3, source(true), &third)] {
        let mut cold = SemanticWorkspaceSession::new();
        let cold_update = cold.update(single_module_input(main.clone(), &text, generation)).expect("semantic test update should succeed");
        assert_eq!(update.snapshot.has_errors(), cold_update.snapshot.has_errors(), "generation {generation}");
        assert_eq!(
            sequence_meanings(&update.snapshot, &run),
            sequence_meanings(&cold_update.snapshot, &run),
            "generation {generation}"
        );
    }
}

#[test]
fn static_sequence_incremental_edits_match_cold_and_drop_stale_products() {
    let main_module = module("static_sequence_incremental");
    let other = module("static_sequence_other");
    let other_source = "const stable: Int = 7";
    let values = StaticConstantId::new(main_module.clone(), "values");
    let symbol = SymbolId {
        module: main_module.clone(),
        name: "values".into(),
    };
    let stable = StaticConstantId::new(other.clone(), "stable");
    let revisions = [
        (
            1,
            "const values: Array<Int> = <Int>[1, 2]\n",
            Some((StaticSequenceFamily::Array, static_ints(&[1, 2]), "Array<Int>".to_owned())),
        ),
        (
            2,
            "const values: Array<Int> = <Int>[1, 9]\n",
            Some((StaticSequenceFamily::Array, static_ints(&[1, 9]), "Array<Int>".to_owned())),
        ),
        (
            3,
            "const values = [1, 2, 3]\n",
            Some((StaticSequenceFamily::Vector, static_ints(&[1, 2, 3]), "Vector<Int, 3#const>".to_owned())),
        ),
        (4, "let values = [1, 2, 3]\n", None),
        (
            5,
            "const values = [1, 2, 3, 4]\n",
            Some((StaticSequenceFamily::Vector, static_ints(&[1, 2, 3, 4]), "Vector<Int, 4#const>".to_owned())),
        ),
    ];
    let mut incremental = SemanticWorkspaceSession::new();
    let mut previous_values: Option<Arc<StaticConstantProduct>> = None;
    let mut stable_product: Option<Arc<StaticConstantProduct>> = None;
    let mut previous_fingerprint = None;

    for (generation, source, expected) in revisions {
        let update = incremental.update(multi_module_input(
            vec![(main_module.clone(), source.to_owned()), (other.clone(), other_source.to_owned())],
            generation,
        )).expect("semantic test update should succeed");
        assert_eq!(static_sequence_meaning(&update.snapshot, &values), expected, "generation {generation}");
        let binding = update.snapshot.module_static_binding(&symbol).expect("current module binding");
        let (expected_phase, expected_type) = match expected.as_ref() {
            Some((_, _, ty)) => (ModuleBindingPhase::StaticConstant, ty.as_str()),
            None => (ModuleBindingPhase::Runtime, "Array<Int>"),
        };
        assert_eq!(binding.phase, expected_phase, "binding phase at generation {generation}");
        assert_eq!(
            update.snapshot.store.format_type(binding.state.current.ty().expect("solved binding type")),
            expected_type,
            "binding family at generation {generation}"
        );
        let product = update.snapshot.static_constant(&values).cloned();
        assert_eq!(
            product.as_ref().and_then(|product| product.value()).is_some(),
            expected.is_some(),
            "current value publication at generation {generation}"
        );
        if let (Some(previous), Some(product)) = (previous_values.as_ref(), product.as_ref()) {
            assert!(
                !Arc::ptr_eq(previous, product),
                "changed sequence input must not reuse its previous product at generation {generation}"
            );
        } else if expected.is_none() {
            assert!(
                product.is_none(),
                "runtime binding must not retain a stale static product at generation {generation}"
            );
        }
        if let Some(product) = product.as_ref() {
            let descriptor = product
                .materialization_descriptor(update.snapshot.static_values())
                .expect("reduced sequence has a self-contained descriptor");
            let expected_kind = match expected.as_ref().map(|value| value.0) {
                Some(StaticSequenceFamily::Array) => SequenceMaterializationKind::FrozenArray,
                Some(StaticSequenceFamily::Vector) => SequenceMaterializationKind::Vector,
                None => unreachable!("a product with a value has sequence family evidence"),
            };
            assert!(matches!(descriptor, MaterializationDescriptor::Sequence { kind, .. } if kind == expected_kind));
            let fingerprint = product
                .value()
                .and_then(|value| update.snapshot.static_values().content_fingerprint(value).ok())
                .expect("reduced sequence fingerprint");
            if let Some(previous) = previous_fingerprint {
                assert_ne!(
                    previous, fingerprint,
                    "changed sequence content or family must invalidate its fingerprint at generation {generation}"
                );
            }
            previous_fingerprint = Some(fingerprint);
        } else {
            previous_fingerprint = None;
        }
        previous_values = product;

        let current_stable = update.snapshot.static_constant(&stable).expect("unrelated stable product").clone();
        if let Some(first) = stable_product.as_ref() {
            assert!(
                Arc::ptr_eq(first, &current_stable),
                "unaffected constant should remain reusable at generation {generation}"
            );
        } else {
            stable_product = Some(current_stable);
        }

        let cold = SemanticWorkspaceSession::new().update(multi_module_input(
            vec![(main_module.clone(), source.to_owned()), (other.clone(), other_source.to_owned())],
            1,
        )).expect("semantic test update should succeed");
        assert_eq!(
            update.snapshot.diagnostics.get(&main_module),
            cold.snapshot.diagnostics.get(&main_module),
            "diagnostics at generation {generation}"
        );
        let cold_binding = cold.snapshot.module_static_binding(&symbol).expect("cold module binding");
        assert_eq!(binding.phase, cold_binding.phase);
        assert_eq!(
            update.snapshot.store.format_type(binding.state.current.ty().unwrap()),
            cold.snapshot.store.format_type(cold_binding.state.current.ty().unwrap()),
            "cold/incremental binding parity at generation {generation}"
        );
        assert_eq!(
            static_sequence_meaning(&update.snapshot, &values),
            static_sequence_meaning(&cold.snapshot, &values),
            "cold product at generation {generation}"
        );
    }
}

#[test]
fn static_sequence_incremental_invalid_valid_invalid_and_spread_exactness_have_no_stale_value() {
    let main_module = module("static_sequence_validity");
    let values = StaticConstantId::new(main_module.clone(), "values");
    let source_id = StaticConstantId::new(main_module.clone(), "source");
    let mut incremental = SemanticWorkspaceSession::new();

    let invalid_empty = incremental.update(single_module_input(main_module.clone(), "const values = []\n", 1)).expect("semantic test update should succeed");
    assert!(invalid_empty.snapshot.has_errors());
    assert!(static_sequence_meaning(&invalid_empty.snapshot, &values).is_none());
    assert!(
        invalid_empty
            .snapshot
            .static_constant(&values)
            .is_some_and(|product| { product.value().is_none() && product.materialization_descriptor(invalid_empty.snapshot.static_values()).is_none() })
    );

    let valid = incremental.update(single_module_input(main_module.clone(), "const values = [1, 2]\n", 2)).expect("semantic test update should succeed");
    assert!(!valid.snapshot.has_errors(), "diagnostics: {:?}", valid.snapshot.diagnostics);
    assert!(static_sequence_meaning(&valid.snapshot, &values).is_some());

    let invalid_extent = incremental.update(single_module_input(main_module.clone(), "const values: Vector<Int, 2> = [1]\n", 3)).expect("semantic test update should succeed");
    assert!(invalid_extent.snapshot.has_errors());
    assert!(static_sequence_meaning(&invalid_extent.snapshot, &values).is_none());
    assert!(
        invalid_extent
            .snapshot
            .static_constant(&values)
            .is_some_and(|product| { product.value().is_none() && product.materialization_descriptor(invalid_extent.snapshot.static_values()).is_none() })
    );

    let exact_spread = incremental.update(single_module_input(
        main_module.clone(),
        "const source: Vector<Int, 2> = <Int, 2>[1, 2]\nconst values: Vector<Int, 2> = [*source]\n",
        4,
    )).expect("semantic test update should succeed");
    assert!(!exact_spread.snapshot.has_errors(), "diagnostics: {:?}", exact_spread.snapshot.diagnostics);
    assert_eq!(static_sequence_meaning(&exact_spread.snapshot, &values).unwrap().1.len(), 2);
    assert!(static_sequence_meaning(&exact_spread.snapshot, &source_id).is_some());

    let runtime_spread = incremental.update(single_module_input(
        main_module.clone(),
        "let source = <Int>[1, 2]\nconst values: Vector<Int, 2> = [*source]\n",
        5,
    )).expect("semantic test update should succeed");
    assert!(runtime_spread.snapshot.has_errors());
    assert!(static_sequence_meaning(&runtime_spread.snapshot, &values).is_none());
    assert!(
        runtime_spread
            .snapshot
            .static_constant(&values)
            .is_some_and(|product| { product.value().is_none() && product.materialization_descriptor(runtime_spread.snapshot.static_values()).is_none() })
    );

    let cold = SemanticWorkspaceSession::new().update(single_module_input(
        main_module,
        "let source = <Int>[1, 2]\nconst values: Vector<Int, 2> = [*source]\n",
        1,
    )).expect("semantic test update should succeed");
    assert_eq!(runtime_spread.snapshot.diagnostics, cold.snapshot.diagnostics);
    assert_eq!(
        static_sequence_meaning(&runtime_spread.snapshot, &values),
        static_sequence_meaning(&cold.snapshot, &values)
    );
}

#[test]
fn multi_hop_static_spread_dependencies_follow_extent_edits_and_cold_analysis() {
    let main = module("static_sequence_multi_hop");
    let base = StaticConstantId::new(main.clone(), "base");
    let middle = StaticConstantId::new(main.clone(), "middle");
    let outer = StaticConstantId::new(main.clone(), "outer");
    let stable = StaticConstantId::new(main.clone(), "stable");
    let mut incremental = SemanticWorkspaceSession::new();

    let initial_source = r#"
const base: Vector<Int, 2> = <Int, 2>[1, 2]
const middle: Vector<Int, 2> = [*base]
const outer: Vector<Int, 3> = [0, *middle]
const stable: Vector<Int, 1> = <Int, 1>[7]
"#;
    let initial = incremental.update(single_module_input(main.clone(), initial_source, 1)).expect("semantic test update should succeed");
    assert!(!initial.snapshot.has_errors(), "initial diagnostics: {:?}", initial.snapshot.diagnostics);
    let initial_middle = static_sequence_meaning(&initial.snapshot, &middle).expect("middle value");
    let initial_outer = static_sequence_meaning(&initial.snapshot, &outer).expect("outer value");
    assert_eq!(initial_middle.1, static_ints(&[1, 2]));
    assert_eq!(initial_outer.1, static_ints(&[0, 1, 2]));

    let content_edit = r#"
const base: Vector<Int, 2> = <Int, 2>[3, 4]
const middle: Vector<Int, 2> = [*base]
const outer: Vector<Int, 3> = [0, *middle]
const stable: Vector<Int, 1> = <Int, 1>[7]
"#;
    let changed = incremental.update(single_module_input(main.clone(), content_edit, 2)).expect("semantic test update should succeed");
    assert!(!changed.snapshot.has_errors(), "content edit diagnostics: {:?}", changed.snapshot.diagnostics);
    assert_eq!(static_sequence_meaning(&changed.snapshot, &middle).unwrap().1, static_ints(&[3, 4]));
    assert_eq!(static_sequence_meaning(&changed.snapshot, &outer).unwrap().1, static_ints(&[0, 3, 4]));
    assert_eq!(static_sequence_meaning(&changed.snapshot, &stable), static_sequence_meaning(&initial.snapshot, &stable));

    let extent_edit = r#"
const base: Vector<Int, 3> = <Int, 3>[5, 6, 7]
const middle: Vector<Int, 2> = [*base]
const outer: Vector<Int, 3> = [0, *middle]
const stable: Vector<Int, 1> = <Int, 1>[7]
"#;
    let invalid = incremental.update(single_module_input(main.clone(), extent_edit, 3)).expect("semantic test update should succeed");
    assert!(invalid.snapshot.has_errors(), "a stale exact spread extent must be rejected");
    assert!(static_sequence_meaning(&invalid.snapshot, &base).is_some(), "edited source is valid");
    for id in [&middle, &outer] {
        assert!(static_sequence_meaning(&invalid.snapshot, id).is_none(), "stale dependent value for {id}");
        let product = invalid.snapshot.static_constant(id).expect("current dependent product");
        assert!(product.value().is_none(), "stale dependent value survived for {id}");
        assert!(product.materialization_descriptor(invalid.snapshot.static_values()).is_none(), "stale descriptor survived for {id}");
    }
    assert_eq!(static_sequence_meaning(&invalid.snapshot, &stable), static_sequence_meaning(&changed.snapshot, &stable));

    let cold = SemanticWorkspaceSession::new().update(single_module_input(main, extent_edit, 1)).expect("semantic test update should succeed");
    assert_eq!(invalid.snapshot.diagnostics, cold.snapshot.diagnostics);
    for id in [&base, &middle, &outer, &stable] {
        assert_eq!(static_sequence_meaning(&invalid.snapshot, id), static_sequence_meaning(&cold.snapshot, id), "cold parity for {id}");
    }
}
