//! Reusable persistent hash map engine with structural sharing and callback-safe preflight/commit split.
//!
//! Realizes the private persistent hash substrate for Bag and Bijection (INV-04..INV-14).
//! All user callbacks occur during Phase A (preflight); Phase B (commit) executes without callbacks.

use crate::error::PhResult;
use crate::heap::persistent_hash::{child_bit, compact_index, fragment, PersistentHashNode, MAX_SHIFT};
use crate::heap::{Heap, ObjRef};
use crate::primitive::ordered_hash::{hash_key, keys_equal};
use crate::value::Value;
use crate::vm::VM;

/// A frame along the path from root down to an update point in a persistent trie.
#[derive(Debug, Clone, Copy)]
pub struct PathFrame {
    pub branch_ref: ObjRef,
    pub bitmap: u32,
    pub bit: u32,
    pub compact_index: usize,
}

/// The terminal action determined during insert preflight.
#[derive(Debug)]
pub enum InsertTerminal {
    /// Trie was empty; allocate a single Leaf root.
    FreshLeaf,
    /// An absent child at an existing branch; insert new Leaf into that branch.
    NewChildLeaf,
    /// Target key matches an existing Leaf key; replace its payload.
    ReplaceLeaf,
    /// Hash collision with an existing Leaf; convert to a 2-entry Collision node.
    CollisionWithLeaf {
        existing_key: Value,
        existing_payload: Value,
    },
    /// Hashes diverge at this shift; build branch chain separating existing and new nodes.
    DivergentBranch {
        existing_ref: ObjRef,
        existing_hash: i64,
        shift: u32,
    },
    /// Target key matches an entry in an existing Collision node; replace that entry.
    ReplaceCollisionEntry {
        existing_ref: ObjRef,
        index: usize,
    },
    /// Full hash matches an existing Collision node but key is distinct; append entry.
    AppendCollisionEntry {
        existing_ref: ObjRef,
    },
}

/// Preflight plan for persistent insertion.
#[derive(Debug)]
pub struct InsertPlan {
    pub root: Option<ObjRef>,
    pub key: Value,
    pub payload: Value,
    pub hash: i64,
    pub frames: Vec<PathFrame>,
    pub terminal: InsertTerminal,
    pub old_payload: Option<Value>,
}

/// Preflight plan for persistent removal.
#[derive(Debug)]
pub struct RemovePlan {
    pub root: Option<ObjRef>,
    pub key: Value,
    pub hash: i64,
    pub frames: Vec<PathFrame>,
    pub terminal: RemoveTerminal,
    pub removed_payload: Option<Value>,
}

/// The terminal action determined during removal preflight.
#[derive(Debug)]
pub enum RemoveTerminal {
    /// Key was not found; no update required.
    None,
    /// Target key was in a Leaf; remove the leaf from its parent branch.
    RemoveLeaf,
    /// Target key was in a Collision node; remove entry and normalize if 1 remains.
    RemoveCollisionEntry {
        existing_ref: ObjRef,
        index: usize,
    },
}

/// Look up `key` in the persistent trie rooted at `root`.
///
/// Executes user `hash` and `==` callbacks safely without holding live heap borrows.
pub fn lookup(vm: &mut VM, root: Option<ObjRef>, key: Value) -> PhResult<Option<Value>> {
    let Some(root_ref) = root else {
        return Ok(None);
    };

    let depth = vm.temp_root_depth();
    vm.push_temp_root(Value::obj(root_ref));
    vm.push_temp_root(key);

    let res = (|| -> PhResult<Option<Value>> {
        let hash = hash_key(vm, key)?;
        let mut curr = root_ref;
        let mut shift = 0;

        loop {
            match vm.heap.persistent_hash_node(curr) {
                PersistentHashNode::Branch { bitmap, children } => {
                    let bitmap = *bitmap;
                    let frag = fragment(hash, shift);
                    let bit = child_bit(frag);
                    if (bitmap & bit) == 0 {
                        return Ok(None);
                    }
                    let idx = compact_index(bitmap, bit);
                    curr = children[idx];
                    shift += 5;
                }
                PersistentHashNode::Leaf { hash: leaf_hash, key: leaf_key, payload } => {
                    let leaf_hash = *leaf_hash;
                    let leaf_key = *leaf_key;
                    let payload = *payload;
                    if leaf_hash != hash {
                        return Ok(None);
                    }
                    if keys_equal(vm, leaf_key, key)? {
                        return Ok(Some(payload));
                    } else {
                        return Ok(None);
                    }
                }
                PersistentHashNode::Collision { hash: col_hash, entries } => {
                    let col_hash = *col_hash;
                    if col_hash != hash {
                        return Ok(None);
                    }
                    let entries_vec: Vec<(Value, Value)> = entries.to_vec();
                    for (cand_k, cand_v) in entries_vec {
                        if keys_equal(vm, cand_k, key)? {
                            return Ok(Some(cand_v));
                        }
                    }
                    return Ok(None);
                }
            }
        }
    })();

    vm.truncate_temp_roots(depth);
    res
}

/// Prepares an insertion plan (Phase A: Inspect / Preflight).
///
/// Invokes user hash/equality callbacks as necessary. Protects all live handles with VM temp roots.
pub fn prepare_insert(
    vm: &mut VM,
    root: Option<ObjRef>,
    key: Value,
    payload: Value,
) -> PhResult<InsertPlan> {
    let depth = vm.temp_root_depth();
    if let Some(r) = root {
        vm.push_temp_root(Value::obj(r));
    }
    vm.push_temp_root(key);
    vm.push_temp_root(payload);

    let res = (|| -> PhResult<InsertPlan> {
        let hash = hash_key(vm, key)?;

        let Some(root_ref) = root else {
            return Ok(InsertPlan {
                root: None,
                key,
                payload,
                hash,
                frames: Vec::new(),
                terminal: InsertTerminal::FreshLeaf,
                old_payload: None,
            });
        };

        let mut curr = root_ref;
        let mut shift = 0;
        let mut frames = Vec::new();

        loop {
            match vm.heap.persistent_hash_node(curr) {
                PersistentHashNode::Branch { bitmap, children } => {
                    let bitmap = *bitmap;
                    let frag = fragment(hash, shift);
                    let bit = child_bit(frag);
                    let idx = compact_index(bitmap, bit);
                    frames.push(PathFrame {
                        branch_ref: curr,
                        bitmap,
                        bit,
                        compact_index: idx,
                    });
                    if (bitmap & bit) == 0 {
                        return Ok(InsertPlan {
                            root,
                            key,
                            payload,
                            hash,
                            frames,
                            terminal: InsertTerminal::NewChildLeaf,
                            old_payload: None,
                        });
                    }
                    curr = children[idx];
                    shift += 5;
                }
                PersistentHashNode::Leaf { hash: leaf_hash, key: leaf_key, payload: leaf_payload } => {
                    let leaf_hash = *leaf_hash;
                    let leaf_key = *leaf_key;
                    let leaf_payload = *leaf_payload;
                    if leaf_hash == hash {
                        if keys_equal(vm, leaf_key, key)? {
                            return Ok(InsertPlan {
                                root,
                                key,
                                payload,
                                hash,
                                frames,
                                terminal: InsertTerminal::ReplaceLeaf,
                                old_payload: Some(leaf_payload),
                            });
                        } else {
                            return Ok(InsertPlan {
                                root,
                                key,
                                payload,
                                hash,
                                frames,
                                terminal: InsertTerminal::CollisionWithLeaf {
                                    existing_key: leaf_key,
                                    existing_payload: leaf_payload,
                                },
                                old_payload: None,
                            });
                        }
                    } else {
                        return Ok(InsertPlan {
                            root,
                            key,
                            payload,
                            hash,
                            frames,
                            terminal: InsertTerminal::DivergentBranch {
                                existing_ref: curr,
                                existing_hash: leaf_hash,
                                shift,
                            },
                            old_payload: None,
                        });
                    }
                }
                PersistentHashNode::Collision { hash: col_hash, entries } => {
                    let col_hash = *col_hash;
                    if col_hash == hash {
                        let entries_vec = entries.to_vec();
                        for (i, &(cand_k, cand_v)) in entries_vec.iter().enumerate() {
                            if keys_equal(vm, cand_k, key)? {
                                return Ok(InsertPlan {
                                    root,
                                    key,
                                    payload,
                                    hash,
                                    frames,
                                    terminal: InsertTerminal::ReplaceCollisionEntry {
                                        existing_ref: curr,
                                        index: i,
                                    },
                                    old_payload: Some(cand_v),
                                });
                            }
                        }
                        return Ok(InsertPlan {
                            root,
                            key,
                            payload,
                            hash,
                            frames,
                            terminal: InsertTerminal::AppendCollisionEntry {
                                existing_ref: curr,
                            },
                            old_payload: None,
                        });
                    } else {
                        return Ok(InsertPlan {
                            root,
                            key,
                            payload,
                            hash,
                            frames,
                            terminal: InsertTerminal::DivergentBranch {
                                existing_ref: curr,
                                existing_hash: col_hash,
                                shift,
                            },
                            old_payload: None,
                        });
                    }
                }
            }
        }
    })();

    vm.truncate_temp_roots(depth);
    res
}

/// Recursively builds branch nodes separating two associations whose hashes differ (Appendix A.1).
fn build_divergent_branches(
    heap: &mut Heap,
    existing_ref: ObjRef,
    existing_hash: i64,
    new_leaf_ref: ObjRef,
    new_hash: i64,
    shift: u32,
) -> ObjRef {
    debug_assert!(shift <= MAX_SHIFT, "divergent hashes must differ within 64 bits");
    let f_existing = fragment(existing_hash, shift);
    let f_new = fragment(new_hash, shift);
    if f_existing != f_new {
        let bit_existing = child_bit(f_existing);
        let bit_new = child_bit(f_new);
        let bitmap = bit_existing | bit_new;
        let children = if f_existing < f_new {
            Box::new([existing_ref, new_leaf_ref])
        } else {
            Box::new([new_leaf_ref, existing_ref])
        };
        heap.alloc_persistent_hash_node(PersistentHashNode::new_branch(bitmap, children))
    } else {
        let child = build_divergent_branches(heap, existing_ref, existing_hash, new_leaf_ref, new_hash, shift + 5);
        let bitmap = child_bit(f_existing);
        heap.alloc_persistent_hash_node(PersistentHashNode::new_branch(bitmap, Box::new([child])))
    }
}

/// Commits an insertion plan without invoking user code (Phase B: Build / Commit).
pub fn commit_insert(heap: &mut Heap, plan: InsertPlan) -> (ObjRef, Option<Value>) {
    let leaf = PersistentHashNode::new_leaf(plan.hash, plan.key, plan.payload);
    let new_leaf_ref = heap.alloc_persistent_hash_node(leaf);

    let mut curr_ref = match plan.terminal {
        InsertTerminal::FreshLeaf | InsertTerminal::NewChildLeaf => new_leaf_ref,
        InsertTerminal::ReplaceLeaf => new_leaf_ref,
        InsertTerminal::CollisionWithLeaf { existing_key, existing_payload } => {
            let col = PersistentHashNode::new_collision(
                plan.hash,
                Box::new([(existing_key, existing_payload), (plan.key, plan.payload)]),
            );
            heap.alloc_persistent_hash_node(col)
        }
        InsertTerminal::DivergentBranch { existing_ref, existing_hash, shift } => {
            build_divergent_branches(heap, existing_ref, existing_hash, new_leaf_ref, plan.hash, shift)
        }
        InsertTerminal::ReplaceCollisionEntry { existing_ref, index } => {
            let (hash, mut entries) = match heap.persistent_hash_node(existing_ref) {
                PersistentHashNode::Collision { hash, entries } => (*hash, entries.to_vec()),
                _ => unreachable!(),
            };
            entries[index] = (plan.key, plan.payload);
            let col = PersistentHashNode::new_collision(hash, entries.into_boxed_slice());
            heap.alloc_persistent_hash_node(col)
        }
        InsertTerminal::AppendCollisionEntry { existing_ref } => {
            let (hash, mut entries) = match heap.persistent_hash_node(existing_ref) {
                PersistentHashNode::Collision { hash, entries } => (*hash, entries.to_vec()),
                _ => unreachable!(),
            };
            entries.push((plan.key, plan.payload));
            let col = PersistentHashNode::new_collision(hash, entries.into_boxed_slice());
            heap.alloc_persistent_hash_node(col)
        }
    };

    // Path-copy ancestors from bottom to top
    for frame in plan.frames.into_iter().rev() {
        let (bitmap, children) = match heap.persistent_hash_node(frame.branch_ref) {
            PersistentHashNode::Branch { bitmap, children } => (*bitmap, children.clone()),
            _ => unreachable!(),
        };
        let mut new_children = children.into_vec();
        let new_bitmap = if (bitmap & frame.bit) == 0 {
            new_children.insert(frame.compact_index, curr_ref);
            bitmap | frame.bit
        } else {
            new_children[frame.compact_index] = curr_ref;
            bitmap
        };
        curr_ref = heap.alloc_persistent_hash_node(PersistentHashNode::new_branch(
            new_bitmap,
            new_children.into_boxed_slice(),
        ));
    }

    (curr_ref, plan.old_payload)
}

/// Convenience entry point: prepares and commits an insertion in one step.
pub fn insert(
    vm: &mut VM,
    root: Option<ObjRef>,
    key: Value,
    payload: Value,
) -> PhResult<(ObjRef, Option<Value>)> {
    let plan = prepare_insert(vm, root, key, payload)?;
    Ok(commit_insert(&mut vm.heap, plan))
}

/// Prepares a removal plan (Phase A: Inspect / Preflight).
pub fn prepare_remove(vm: &mut VM, root: Option<ObjRef>, key: Value) -> PhResult<RemovePlan> {
    let Some(root_ref) = root else {
        return Ok(RemovePlan {
            root: None,
            key,
            hash: 0,
            frames: Vec::new(),
            terminal: RemoveTerminal::None,
            removed_payload: None,
        });
    };

    let depth = vm.temp_root_depth();
    vm.push_temp_root(Value::obj(root_ref));
    vm.push_temp_root(key);

    let res = (|| -> PhResult<RemovePlan> {
        let hash = hash_key(vm, key)?;
        let mut curr = root_ref;
        let mut shift = 0;
        let mut frames = Vec::new();

        loop {
            match vm.heap.persistent_hash_node(curr) {
                PersistentHashNode::Branch { bitmap, children } => {
                    let bitmap = *bitmap;
                    let frag = fragment(hash, shift);
                    let bit = child_bit(frag);
                    if (bitmap & bit) == 0 {
                        return Ok(RemovePlan {
                            root,
                            key,
                            hash,
                            frames: Vec::new(),
                            terminal: RemoveTerminal::None,
                            removed_payload: None,
                        });
                    }
                    let idx = compact_index(bitmap, bit);
                    frames.push(PathFrame {
                        branch_ref: curr,
                        bitmap,
                        bit,
                        compact_index: idx,
                    });
                    curr = children[idx];
                    shift += 5;
                }
                PersistentHashNode::Leaf { hash: leaf_hash, key: leaf_key, payload } => {
                    let leaf_hash = *leaf_hash;
                    let leaf_key = *leaf_key;
                    let payload = *payload;
                    if leaf_hash == hash && keys_equal(vm, leaf_key, key)? {
                        return Ok(RemovePlan {
                            root,
                            key,
                            hash,
                            frames,
                            terminal: RemoveTerminal::RemoveLeaf,
                            removed_payload: Some(payload),
                        });
                    } else {
                        return Ok(RemovePlan {
                            root,
                            key,
                            hash,
                            frames: Vec::new(),
                            terminal: RemoveTerminal::None,
                            removed_payload: None,
                        });
                    }
                }
                PersistentHashNode::Collision { hash: col_hash, entries } => {
                    let col_hash = *col_hash;
                    if col_hash == hash {
                        let entries_vec = entries.to_vec();
                        for (i, &(cand_k, cand_v)) in entries_vec.iter().enumerate() {
                            if keys_equal(vm, cand_k, key)? {
                                return Ok(RemovePlan {
                                    root,
                                    key,
                                    hash,
                                    frames,
                                    terminal: RemoveTerminal::RemoveCollisionEntry {
                                        existing_ref: curr,
                                        index: i,
                                    },
                                    removed_payload: Some(cand_v),
                                });
                            }
                        }
                    }
                    return Ok(RemovePlan {
                        root,
                        key,
                        hash,
                        frames: Vec::new(),
                        terminal: RemoveTerminal::None,
                        removed_payload: None,
                    });
                }
            }
        }
    })();

    vm.truncate_temp_roots(depth);
    res
}

/// Commits a removal plan without invoking user code (Phase B: Build / Commit).
pub fn commit_remove(heap: &mut Heap, plan: RemovePlan) -> (Option<ObjRef>, Option<Value>) {
    let Some(removed_payload) = plan.removed_payload else {
        return (plan.root, None);
    };

    let mut curr_ref: Option<ObjRef> = match plan.terminal {
        RemoveTerminal::None => unreachable!(),
        RemoveTerminal::RemoveLeaf => None,
        RemoveTerminal::RemoveCollisionEntry { existing_ref, index } => {
            let (hash, mut entries) = match heap.persistent_hash_node(existing_ref) {
                PersistentHashNode::Collision { hash, entries } => (*hash, entries.to_vec()),
                _ => unreachable!(),
            };
            entries.remove(index);
            if entries.len() == 1 {
                let (k, v) = entries[0];
                let leaf = PersistentHashNode::new_leaf(hash, k, v);
                Some(heap.alloc_persistent_hash_node(leaf))
            } else {
                let col = PersistentHashNode::new_collision(hash, entries.into_boxed_slice());
                Some(heap.alloc_persistent_hash_node(col))
            }
        }
    };

    for frame in plan.frames.into_iter().rev() {
        let (bitmap, children) = match heap.persistent_hash_node(frame.branch_ref) {
            PersistentHashNode::Branch { bitmap, children } => (*bitmap, children.clone()),
            _ => unreachable!(),
        };
        let mut new_children = children.into_vec();
        match curr_ref {
            Some(replacement) => {
                new_children[frame.compact_index] = replacement;
                curr_ref = Some(heap.alloc_persistent_hash_node(PersistentHashNode::new_branch(
                    bitmap,
                    new_children.into_boxed_slice(),
                )));
            }
            None => {
                new_children.remove(frame.compact_index);
                let new_bitmap = bitmap & !frame.bit;
                if new_bitmap == 0 {
                    curr_ref = None;
                } else {
                    curr_ref = Some(heap.alloc_persistent_hash_node(PersistentHashNode::new_branch(
                        new_bitmap,
                        new_children.into_boxed_slice(),
                    )));
                }
            }
        }
    }

    (curr_ref, Some(removed_payload))
}

/// Updates the payload of a located entry without invoking user code (Phase B).
pub fn commit_replace_payload(heap: &mut Heap, plan: RemovePlan, new_payload: Value) -> Option<ObjRef> {
    if plan.removed_payload.is_none() {
        return plan.root;
    }

    let mut curr_ref: Option<ObjRef> = match plan.terminal {
        RemoveTerminal::None => unreachable!(),
        RemoveTerminal::RemoveLeaf => {
            let leaf = PersistentHashNode::new_leaf(plan.hash, plan.key, new_payload);
            Some(heap.alloc_persistent_hash_node(leaf))
        }
        RemoveTerminal::RemoveCollisionEntry { existing_ref, index } => {
            let (hash, mut entries) = match heap.persistent_hash_node(existing_ref) {
                PersistentHashNode::Collision { hash, entries } => (*hash, entries.to_vec()),
                _ => unreachable!(),
            };
            entries[index] = (plan.key, new_payload);
            let col = PersistentHashNode::new_collision(hash, entries.into_boxed_slice());
            Some(heap.alloc_persistent_hash_node(col))
        }
    };

    for frame in plan.frames.into_iter().rev() {
        let (bitmap, children) = match heap.persistent_hash_node(frame.branch_ref) {
            PersistentHashNode::Branch { bitmap, children } => (*bitmap, children.clone()),
            _ => unreachable!(),
        };
        let mut new_children = children.into_vec();
        match curr_ref {
            Some(replacement) => {
                new_children[frame.compact_index] = replacement;
                curr_ref = Some(heap.alloc_persistent_hash_node(PersistentHashNode::new_branch(
                    bitmap,
                    new_children.into_boxed_slice(),
                )));
            }
            None => unreachable!(),
        }
    }

    curr_ref
}

/// Convenience entry point: prepares and commits removal in one step.
pub fn remove(
    vm: &mut VM,
    root: Option<ObjRef>,
    key: Value,
) -> PhResult<(Option<ObjRef>, Option<Value>)> {
    let plan = prepare_remove(vm, root, key)?;
    Ok(commit_remove(&mut vm.heap, plan))
}

/// Collects all `(key, payload)` entries from the persistent trie in deterministic encounter order.
pub fn collect_entries(heap: &Heap, root: Option<ObjRef>, out: &mut Vec<(Value, Value)>) {
    let Some(root_ref) = root else {
        return;
    };
    let mut stack = vec![root_ref];
    while let Some(node_ref) = stack.pop() {
        match heap.persistent_hash_node(node_ref) {
            PersistentHashNode::Leaf { key, payload, .. } => {
                out.push((*key, *payload));
            }
            PersistentHashNode::Collision { entries, .. } => {
                out.extend_from_slice(entries);
            }
            PersistentHashNode::Branch { children, .. } => {
                for &child in children.iter().rev() {
                    stack.push(child);
                }
            }
        }
    }
}

/// Collects all distinct keys from the persistent trie in deterministic encounter order.
pub fn collect_keys(heap: &Heap, root: Option<ObjRef>, out: &mut Vec<Value>) {
    let Some(root_ref) = root else {
        return;
    };
    let mut stack = vec![root_ref];
    while let Some(node_ref) = stack.pop() {
        match heap.persistent_hash_node(node_ref) {
            PersistentHashNode::Leaf { key, .. } => {
                out.push(*key);
            }
            PersistentHashNode::Collision { entries, .. } => {
                for (k, _) in entries.iter() {
                    out.push(*k);
                }
            }
            PersistentHashNode::Branch { children, .. } => {
                for &child in children.iter().rev() {
                    stack.push(child);
                }
            }
        }
    }
}

/// Counts the number of logical entries in the persistent trie.
pub fn count_entries(heap: &Heap, root: Option<ObjRef>) -> usize {
    let Some(root_ref) = root else {
        return 0;
    };
    let mut count = 0;
    let mut stack = vec![root_ref];
    while let Some(node_ref) = stack.pop() {
        match heap.persistent_hash_node(node_ref) {
            PersistentHashNode::Leaf { .. } => count += 1,
            PersistentHashNode::Collision { entries, .. } => count += entries.len(),
            PersistentHashNode::Branch { children, .. } => {
                for &child in children.iter() {
                    stack.push(child);
                }
            }
        }
    }
    count
}
