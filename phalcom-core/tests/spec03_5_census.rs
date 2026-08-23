//! Four-way native surface census. Keep output deterministic while migration
//! rows still exist in the compatibility installer.

use phalcom_common::selector::Selector;
use phalcom_core::native::PRIMITIVES;
use phalcom_core::vm::VM;
use phalcom_native_surface::{NATIVE_MEMBERS, NATIVE_SURFACES};
use std::collections::BTreeSet;

type Key = (phalcom_native_meta::UniverseKey, phalcom_native_meta::NativeDispatch, String);

fn owner_side_counts(keys: &BTreeSet<Key>) -> BTreeSet<(phalcom_native_meta::UniverseKey, phalcom_native_meta::NativeDispatch, usize)> {
    let mut counts = BTreeSet::new();
    let mut current = None;
    let mut count = 0;
    for (owner, side, _) in keys {
        let lane = (*owner, *side);
        if current == Some(lane) {
            count += 1;
        } else {
            if let Some((previous_owner, previous_side)) = current {
                counts.insert((previous_owner, previous_side, count));
            }
            current = Some(lane);
            count = 1;
        }
    }
    if let Some((owner, side)) = current {
        counts.insert((owner, side, count));
    }
    counts
}

fn surface_keys() -> BTreeSet<Key> {
    NATIVE_SURFACES
        .iter()
        .map(|record| (record.owner(), record.side(), record.selector().to_owned()))
        .collect()
}

fn descriptor_keys() -> BTreeSet<Key> {
    PRIMITIVES
        .iter()
        .map(|descriptor| {
            (
                descriptor.surface.key.owner,
                descriptor.surface.key.side,
                descriptor.surface.key.selector.to_owned(),
            )
        })
        .collect()
}

#[test]
fn canonical_surface_census_is_unique_and_actionable() {
    let generated = surface_keys();
    let descriptors = descriptor_keys();
    let legacy = NATIVE_MEMBERS
        .iter()
        .filter_map(|member| {
            Some((
                phalcom_native_meta::UniverseKey::from_name(member.class)?,
                member.side,
                member.selector.to_owned(),
            ))
        })
        .collect::<BTreeSet<_>>();

    let generated_only = generated.difference(&descriptors).collect::<Vec<_>>();
    let descriptor_only = descriptors.difference(&generated).collect::<Vec<_>>();
    let legacy_only = legacy.difference(&descriptors).collect::<Vec<_>>();

    for (_, _, selector) in generated.iter().chain(descriptors.iter()).chain(legacy.iter()) {
        assert!(Selector::try_decode_exact(selector).is_ok(), "noncanonical census selector {selector}");
    }

    eprintln!(
        "native census: legacy={} generated={} descriptors={} generated-only={generated_only:?} descriptor-only={descriptor_only:?} legacy-only={legacy_only:?}",
        legacy.len(),
        generated.len(),
        descriptors.len(),
    );
    eprintln!("native census per owner/side: generated={:?}", owner_side_counts(&generated));
    assert!(!generated.is_empty());
    assert!(!descriptors.is_empty());
}

#[test]
fn canonical_bootstrap_relations_have_one_row_per_class() {
    let mut classes = BTreeSet::new();
    for relation in phalcom_native_meta::UNIVERSE_CLASS_RELATIONS {
        assert!(classes.insert(relation.class), "duplicate bootstrap relation for {:?}", relation.class);
        if let Some(superclass) = relation.superclass {
            assert_ne!(relation.class, superclass, "self-superclass relation for {:?}", relation.class);
        }
    }
    assert!(classes.contains(&phalcom_native_meta::UniverseKey::Object));
}

#[test]
fn runtime_bootstrap_matches_canonical_relations() {
    let vm = VM::new();
    for relation in phalcom_native_meta::UNIVERSE_CLASS_RELATIONS {
        let class = vm.universe.classes.resolve(relation.class);
        let actual = vm.heap.class(class).superclass.map(|superclass| vm.heap.class(superclass).name.clone());
        let expected = relation.superclass.map(|superclass| superclass.name().to_string());
        assert_eq!(actual, expected, "runtime relation drift for {}", relation.class.name());
    }
}
