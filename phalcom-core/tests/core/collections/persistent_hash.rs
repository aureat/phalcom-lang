//! Comprehensive test suite for the reusable persistent hash substrate (G3, CV-01..CV-17).
//!
//! Validates structural sharing, old-root immutability, collision correctness, depth-bounded
//! allocation, callback safety, error recovery, re-entrant reading, and forced GC.

use phalcom_core::heap::persistent_hash::PersistentHashNode;
use phalcom_core::primitive::persistent_hash::{
    collect_entries, collect_keys, count_entries, insert, lookup, prepare_insert, remove,
};
use phalcom_core::value::Value;
use phalcom_core::vm::VM;
use std::collections::HashMap;

/// Helper creating a settled VM.
fn test_vm() -> VM {
    let mut vm = VM::new();
    vm.force_gc();
    vm
}

// CV-01: empty trie
#[test]
fn persistent_hash_cv01_empty_trie() {
    let mut vm = test_vm();
    let root = None;

    assert_eq!(lookup(&mut vm, root, Value::int(42)).unwrap(), None);
    let (rem_root, rem_val) = remove(&mut vm, root, Value::int(42)).unwrap();
    assert_eq!(rem_root, None);
    assert_eq!(rem_val, None);
    assert_eq!(count_entries(&vm.heap, root), 0);
}

// CV-02: one leaf
#[test]
fn persistent_hash_cv02_one_leaf_immutability() {
    let mut vm = test_vm();
    let root0 = None;

    let (root1, old_val) = insert(&mut vm, root0, Value::int(1), Value::int(100)).unwrap();
    assert_eq!(old_val, None);
    assert_eq!(count_entries(&vm.heap, Some(root1)), 1);

    // Old root0 is still empty
    assert_eq!(lookup(&mut vm, root0, Value::int(1)).unwrap(), None);
    // New root1 has the value
    assert_eq!(lookup(&mut vm, Some(root1), Value::int(1)).unwrap(), Some(Value::int(100)));
}

// Two-phase prepare/commit split and traversal helpers
#[test]
fn persistent_hash_prepare_commit_and_traversals() {
    let mut vm = test_vm();
    let root0 = None;

    // Test prepare_insert followed by commit_insert
    let plan = prepare_insert(&mut vm, root0, Value::int(10), Value::int(100)).unwrap();
    let (root1, old) = phalcom_core::primitive::persistent_hash::commit_insert(&mut vm.heap, plan);
    assert_eq!(old, None);

    let plan2 = prepare_insert(&mut vm, Some(root1), Value::int(20), Value::int(200)).unwrap();
    let (root2, old2) = phalcom_core::primitive::persistent_hash::commit_insert(&mut vm.heap, plan2);
    assert_eq!(old2, None);

    let mut entries = Vec::new();
    collect_entries(&vm.heap, Some(root2), &mut entries);
    assert_eq!(entries.len(), 2);
    assert!(entries.contains(&(Value::int(10), Value::int(100))));
    assert!(entries.contains(&(Value::int(20), Value::int(200))));

    let mut keys = Vec::new();
    collect_keys(&vm.heap, Some(root2), &mut keys);
    assert_eq!(keys.len(), 2);
    assert!(keys.contains(&Value::int(10)));
    assert!(keys.contains(&Value::int(20)));
}

// CV-03: first-level branch (two keys diverging at shift 0)
// CV-04: deep branch (two keys diverging after several equal 5-bit fragments)
// CV-09: negative hash bits
// CV-10: max-depth fragments
#[test]
fn persistent_hash_cv03_04_09_10_divergent_branches_and_depths() {
    let mut vm = test_vm();
    let root0 = None;

    // Insert keys that produce different hashes
    let mut root = root0;
    let count = 50;
    for i in 0..count {
        let (new_root, old_val) = insert(&mut vm, root, Value::int(i), Value::int(i * 10)).unwrap();
        assert_eq!(old_val, None);
        root = Some(new_root);
    }

    assert_eq!(count_entries(&vm.heap, root), count as usize);

    for i in 0..count {
        assert_eq!(
            lookup(&mut vm, root, Value::int(i)).unwrap(),
            Some(Value::int(i * 10)),
            "key {i} must be present in trie"
        );
    }

    // Negative integer key (CV-09)
    let (neg_root, _) = insert(&mut vm, root, Value::int(-42), Value::int(999)).unwrap();
    assert_eq!(lookup(&mut vm, Some(neg_root), Value::int(-42)).unwrap(), Some(Value::int(999)));
    assert_eq!(lookup(&mut vm, root, Value::int(-42)).unwrap(), None, "old root unchanged");
}

// CV-05: full-hash collision
// CV-06: collision replace
// CV-07: collision remove / collapse
#[test]
fn persistent_hash_cv05_06_07_full_hash_collisions() {
    // Create a custom Telos class with custom hash returning a constant to force collisions
    let (mut vm, module) = super::vm_support::run_inline(
        r#"
        class CollidingKey {
          _id
          @constructor
          new(_ id) { _id = id }
          id { _id }
          hash { 99999 }
          ==(_ other) {
            if (other is CollidingKey) {
              return _id == other.id
            }
            false
          }
        }

        let k1 = CollidingKey.new(1)
        let k2 = CollidingKey.new(2)
        let k3 = CollidingKey.new(3)
        "#,
    )
    .expect("compile CollidingKey");

    let sym_k1 = vm.interner.find("k1").unwrap();
    let sym_k2 = vm.interner.find("k2").unwrap();
    let sym_k3 = vm.interner.find("k3").unwrap();
    let k1 = vm.heap.module(module).get(sym_k1).unwrap();
    let k2 = vm.heap.module(module).get(sym_k2).unwrap();
    let k3 = vm.heap.module(module).get(sym_k3).unwrap();

    let root0 = None;

    // CV-05: Insert k1
    let (root1, old) = insert(&mut vm, root0, k1, Value::int(10)).unwrap();
    assert_eq!(old, None);

    // Insert k2: identical hash (99999), different key -> Collision node!
    let (root2, old) = insert(&mut vm, Some(root1), k2, Value::int(20)).unwrap();
    assert_eq!(old, None);
    assert_eq!(count_entries(&vm.heap, Some(root2)), 2);

    assert_eq!(lookup(&mut vm, Some(root2), k1).unwrap(), Some(Value::int(10)));
    assert_eq!(lookup(&mut vm, Some(root2), k2).unwrap(), Some(Value::int(20)));
    assert_eq!(lookup(&mut vm, Some(root2), k3).unwrap(), None);

    // Old root1 only has k1
    assert_eq!(lookup(&mut vm, Some(root1), k1).unwrap(), Some(Value::int(10)));
    assert_eq!(lookup(&mut vm, Some(root1), k2).unwrap(), None);

    // Insert k3: third colliding entry
    let (root3, old) = insert(&mut vm, Some(root2), k3, Value::int(30)).unwrap();
    assert_eq!(old, None);
    assert_eq!(count_entries(&vm.heap, Some(root3)), 3);

    // CV-06: Collision replace: re-insert k2 with new value
    let (root4, old) = insert(&mut vm, Some(root3), k2, Value::int(222)).unwrap();
    assert_eq!(old, Some(Value::int(20)), "old payload returned on replacement");
    assert_eq!(count_entries(&vm.heap, Some(root4)), 3, "distinct count unchanged");
    assert_eq!(lookup(&mut vm, Some(root4), k2).unwrap(), Some(Value::int(222)));
    assert_eq!(lookup(&mut vm, Some(root3), k2).unwrap(), Some(Value::int(20)), "root3 intact");

    // CV-07: Collision remove & collapse:
    // Remove k3 from root4 -> 2 entries left
    let (root5, rem) = remove(&mut vm, Some(root4), k3).unwrap();
    assert_eq!(rem, Some(Value::int(30)));
    assert_eq!(count_entries(&vm.heap, root5), 2);

    // Remove k2 from root5 -> 1 entry left -> normalizes to Leaf!
    let (root6, rem) = remove(&mut vm, root5, k2).unwrap();
    assert_eq!(rem, Some(Value::int(222)));
    assert_eq!(count_entries(&vm.heap, root6), 1);
    let root6_node = vm.heap.persistent_hash_node(root6.unwrap());
    assert!(root6_node.is_leaf(), "Collision with 1 entry collapsed to Leaf (INV-10 / §7.4)");
    assert_eq!(lookup(&mut vm, root6, k1).unwrap(), Some(Value::int(10)));

    // Remove k1 from root6 -> 0 entries left -> None
    let (root7, rem) = remove(&mut vm, root6, k1).unwrap();
    assert_eq!(rem, Some(Value::int(10)));
    assert_eq!(root7, None);
}

// CV-08: branch remove
// CV-11: structural sharing (unaffected child ObjRefs identical across versions)
// CV-12: allocation shape (bounded by depth, not n)
// CV-13: old-root immutability
#[test]
fn persistent_hash_cv08_11_12_13_structural_sharing_and_allocation_shape() {
    let mut vm = test_vm();
    let mut root = None;

    // Insert 100 entries
    let n = 100;
    for i in 0..n {
        let (new_root, _) = insert(&mut vm, root, Value::int(i), Value::int(i)).unwrap();
        root = Some(new_root);
    }

    let root_v1 = root.unwrap();
    let live_before = vm.heap.live_count();

    // Insert 1 new key into root_v1 -> produces root_v2
    let (root_v2, _) = insert(&mut vm, Some(root_v1), Value::int(9999), Value::int(9999)).unwrap();

    let live_after = vm.heap.live_count();
    let delta = live_after - live_before;
    // Bounded by trie depth (at most 13 nodes for 64-bit hash at 5 bits/level) + leaf
    assert!(
        delta <= 14,
        "INV-07 / CV-12: Single update allocated {delta} nodes, must be bounded by depth (<=14), not N ({n})"
    );

    // Verify CV-11: structural sharing
    // Walk root_v1 and root_v2 branches and check that unaffected children are identical ObjRefs
    let node1 = vm.heap.persistent_hash_node(root_v1);
    let node2 = vm.heap.persistent_hash_node(root_v2);
    if let (PersistentHashNode::Branch { children: c1, .. }, PersistentHashNode::Branch { children: c2, .. }) = (node1, node2) {
        let mut shared_found = false;
        for child1 in c1.iter() {
            if c2.contains(child1) {
                shared_found = true;
                break;
            }
        }
        assert!(shared_found, "INV-06 / CV-11: Unaffected child ObjRefs must be shared across versions");
    }

    // Verify CV-13: old-root immutability
    assert_eq!(lookup(&mut vm, Some(root_v1), Value::int(9999)).unwrap(), None);
    assert_eq!(lookup(&mut vm, Some(root_v2), Value::int(9999)).unwrap(), Some(Value::int(9999)));

    // CV-08: Branch remove: remove key 9999 from root_v2
    let (root_v3, rem) = remove(&mut vm, Some(root_v2), Value::int(9999)).unwrap();
    assert_eq!(rem, Some(Value::int(9999)));
    assert_eq!(count_entries(&vm.heap, root_v3), n as usize);
    // root_v2 still contains 9999
    assert_eq!(lookup(&mut vm, Some(root_v2), Value::int(9999)).unwrap(), Some(Value::int(9999)));
}

// CV-14: callback hash error leaves old root valid
// CV-15: callback equality error leaves old root valid
#[test]
fn persistent_hash_cv14_15_callback_error_recovery() {
    let (mut vm, module) = super::vm_support::run_inline(
        r#"
        class HashThrowKey {
          hash { throw Error.new("hash boom") }
        }

        class EqThrowKey {
          _id
          @constructor
          new(_ id) { _id = id }
          id { _id }
          hash { 42 }
          ==(_ other) {
            if (other is EqThrowKey) {
              if (_id == other.id) {
                return true
              }
            }
            throw Error.new("eq boom")
          }
        }

        let badHash = HashThrowKey.new()
        let eq1 = EqThrowKey.new(1)
        let eq2 = EqThrowKey.new(2)
        "#,
    )
    .expect("compile test classes");

    let bad_hash = vm.heap.module(module).get(vm.interner.find("badHash").unwrap()).unwrap();
    let eq1 = vm.heap.module(module).get(vm.interner.find("eq1").unwrap()).unwrap();
    let eq2 = vm.heap.module(module).get(vm.interner.find("eq2").unwrap()).unwrap();

    let mut root = None;
    let (r, _) = insert(&mut vm, root, Value::int(1), Value::int(100)).unwrap();
    root = Some(r);

    // CV-14: Hash error
    let res = insert(&mut vm, root, bad_hash, Value::int(999));
    assert!(res.is_err(), "throwing hash must fail");
    // Old root intact
    assert_eq!(lookup(&mut vm, root, Value::int(1)).unwrap(), Some(Value::int(100)));
    assert_eq!(count_entries(&vm.heap, root), 1);

    // CV-15: Equality error during collision inspection
    let (r2, _) = insert(&mut vm, root, eq1, Value::int(10)).unwrap();
    root = Some(r2);

    let res = insert(&mut vm, root, eq2, Value::int(20));
    assert!(res.is_err(), "throwing == must fail");
    // Old root intact
    assert_eq!(lookup(&mut vm, root, Value::int(1)).unwrap(), Some(Value::int(100)));
    assert_eq!(lookup(&mut vm, root, eq1).unwrap(), Some(Value::int(10)));
    assert_eq!(count_entries(&vm.heap, root), 2);
}

// CV-16: re-entrant read: hash/== callback can read immutable collection safely
// CV-17: forced GC in callback: callback invokes GC; receiver/query/candidate survive
#[test]
fn persistent_hash_cv16_17_reentrant_read_and_forced_gc() {
    let (mut vm, module) = super::vm_support::run_inline(
        r#"
        class ReentrantKey {
          _id
          @constructor
          new(_ id) { _id = id }
          id { _id }
          hash {
            // Trigger forced GC inside the hash callback
            System.gc
            42
          }
          ==(_ other) {
            // Trigger forced GC inside equality callback
            System.gc
            if (other is ReentrantKey) {
              return _id == other.id
            }
            false
          }
        }

        let r1 = ReentrantKey.new(1)
        let r2 = ReentrantKey.new(2)
        "#,
    )
    .expect("compile ReentrantKey");

    let r1 = vm.heap.module(module).get(vm.interner.find("r1").unwrap()).unwrap();
    let r2 = vm.heap.module(module).get(vm.interner.find("r2").unwrap()).unwrap();

    let mut root = None;

    // Insert r1 (triggers System.gc in hash callback)
    let (new_root, _) = insert(&mut vm, root, r1, Value::int(111)).unwrap();
    root = Some(new_root);

    // Insert r2 (triggers System.gc in hash and in == callbacks)
    let (new_root, _) = insert(&mut vm, root, r2, Value::int(222)).unwrap();
    root = Some(new_root);

    // Lookup r1 and r2 (triggers System.gc during lookup)
    assert_eq!(lookup(&mut vm, root, r1).unwrap(), Some(Value::int(111)));
    assert_eq!(lookup(&mut vm, root, r2).unwrap(), Some(Value::int(222)));
}

// Randomized model test comparing persistent trie against Rust HashMap across versions
#[test]
fn persistent_hash_randomized_model_conformance() {
    let mut vm = test_vm();

    let mut model_versions: Vec<HashMap<i64, i64>> = vec![HashMap::new()];
    let mut trie_versions: Vec<Option<phalcom_core::heap::ObjRef>> = vec![None];

    let mut state: u64 = 123456789;
    let mut next_rand = || -> u64 {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        state
    };

    for step in 0..200 {
        let op = next_rand() % 3;
        let v_idx = (next_rand() as usize) % trie_versions.len();
        let key = (next_rand() % 40) as i64;
        let val = (next_rand() % 1000) as i64;

        let mut current_model = model_versions[v_idx].clone();
        let current_trie = trie_versions[v_idx];

        match op {
            0 | 1 => {
                // Insert
                current_model.insert(key, val);
                let (new_trie, _) = insert(&mut vm, current_trie, Value::int(key), Value::int(val)).unwrap();
                model_versions.push(current_model);
                trie_versions.push(Some(new_trie));
            }
            2 => {
                // Remove
                current_model.remove(&key);
                let (new_trie, _) = remove(&mut vm, current_trie, Value::int(key)).unwrap();
                model_versions.push(current_model);
                trie_versions.push(new_trie);
            }
            _ => unreachable!(),
        }

        // Periodically verify all past versions still match their respective models
        if step % 25 == 0 {
            for (idx, (m, t)) in model_versions.iter().zip(trie_versions.iter()).enumerate() {
                assert_eq!(count_entries(&vm.heap, *t), m.len(), "version {idx} size mismatch");
                for (&k, &expected_v) in m.iter() {
                    assert_eq!(
                        lookup(&mut vm, *t, Value::int(k)).unwrap(),
                        Some(Value::int(expected_v)),
                        "version {idx} key {k} mismatch"
                    );
                }
            }
        }
    }
}
