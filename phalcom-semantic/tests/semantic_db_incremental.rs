use phalcom_modules::ModuleId;
use phalcom_semantic::db::{
    DependencyRecorder, InputFingerprint, ProductFingerprint, QueryKey, QueryState, QueryValue, SemanticDb,
};

fn mod_a() -> ModuleId {
    ModuleId::core()
}

#[test]
fn test_generic_reuse_validation_matrix() {
    let mut db = SemanticDb::new();
    let rev1 = db.revision();

    let key_leaf = QueryKey::ParsedModule(mod_a());
    let key_mid = QueryKey::UnlinkedInterface(mod_a());
    let key_root = QueryKey::LinkedInterface(mod_a());

    // Publish leaf: input_fp = 10, prod_fp = 100
    db.publish_ready(
        key_leaf.clone(),
        rev1,
        InputFingerprint::new(10),
        ProductFingerprint::new(100),
        QueryValue::from_bytes(b"leaf_v1"),
        [],
    )
    .unwrap();

    // Use record_dependency helper to record dependency leaf -> mid
    let mut mid_rec = DependencyRecorder::new(key_mid.clone());
    db.record_dependency(&mut mid_rec, key_leaf.clone())
        .expect("leaf is Ready so record_dependency must succeed");

    // Publish mid: input_fp = 20, prod_fp = 200, depends on leaf (observed 100)
    db.publish_ready(
        key_mid.clone(),
        rev1,
        InputFingerprint::new(20),
        ProductFingerprint::new(200),
        QueryValue::from_bytes(b"mid_v1"),
        mid_rec.finish(),
    )
    .unwrap();

    // Record mid -> root
    let mut root_rec = DependencyRecorder::new(key_root.clone());
    db.record_dependency(&mut root_rec, key_mid.clone())
        .expect("mid is Ready so record_dependency must succeed");

    // Publish root: input_fp = 30, prod_fp = 300, depends on mid (observed 200)
    db.publish_ready(
        key_root.clone(),
        rev1,
        InputFingerprint::new(30),
        ProductFingerprint::new(300),
        QueryValue::from_bytes(b"root_v1"),
        root_rec.finish(),
    )
    .unwrap();

    // 1. Same input + same dependency products => reusable
    assert!(db.is_reusable(&key_leaf, InputFingerprint::new(10)));
    assert!(db.is_reusable(&key_mid, InputFingerprint::new(20)));
    assert!(db.is_reusable(&key_root, InputFingerprint::new(30)));

    // 2. Changed input + same dependencies => not reusable
    assert!(!db.is_reusable(&key_leaf, InputFingerprint::new(11)));
    assert!(!db.is_reusable(&key_mid, InputFingerprint::new(21)));
    assert!(!db.is_reusable(&key_root, InputFingerprint::new(31)));

    // 3. Same input + changed dependency product fingerprint => not reusable
    let rev2 = db.begin_revision();

    // Re-publish leaf with new product fingerprint 101
    db.publish_ready(
        key_leaf.clone(),
        rev2,
        InputFingerprint::new(12),
        ProductFingerprint::new(101),
        QueryValue::from_bytes(b"leaf_v2"),
        [],
    )
    .unwrap();

    // Leaf itself is reusable for input 12
    assert!(db.is_reusable(&key_leaf, InputFingerprint::new(12)));
    // Mid depends on leaf with observed prod_fp 100, but current leaf prod_fp is 101 => mid NOT reusable!
    assert!(!db.is_reusable(&key_mid, InputFingerprint::new(20)));
    // Root depends on mid with observed prod_fp 200, mid is still 200, so root is still reusable until mid changes
    assert!(db.is_reusable(&key_root, InputFingerprint::new(30)));

    // 4. Product fingerprint can remain stable across newer revision and still be reused
    let mut mid_rec2 = DependencyRecorder::new(key_mid.clone());
    db.record_dependency(&mut mid_rec2, key_leaf.clone())
        .expect("leaf is Ready");

    // Mid computes from new leaf but produces SAME product fingerprint 200!
    db.publish_ready(
        key_mid.clone(),
        rev2,
        InputFingerprint::new(22),
        ProductFingerprint::new(200), // stable product fingerprint!
        QueryValue::from_bytes(b"mid_v2"),
        mid_rec2.finish(),
    )
    .unwrap();

    assert!(db.is_reusable(&key_mid, InputFingerprint::new(22)));
    // Root's observed dependency on mid (200) is satisfied because mid produced 200!
    assert!(db.is_reusable(&key_root, InputFingerprint::new(30)));

    // 5. Dependency cancelled/blocked/missing => not reusable
    let rev3 = db.begin_revision();
    db.set_state(key_leaf.clone(), QueryState::Cancelled { revision: rev3 });
    assert!(!db.is_reusable(&key_mid, InputFingerprint::new(22)));
}

#[test]
fn test_record_dependency_fails_on_non_ready() {
    let mut db = SemanticDb::new();
    let key_a = QueryKey::ParsedModule(mod_a());
    let key_b = QueryKey::UnlinkedInterface(mod_a());

    let mut recorder = DependencyRecorder::new(key_b);
    let err = db.record_dependency(&mut recorder, key_a.clone());
    assert!(err.is_err(), "cannot record dependency on vacant query");

    db.set_state(key_a.clone(), QueryState::Cancelled { revision: db.revision() });
    let err = db.record_dependency(&mut recorder, key_a);
    assert!(err.is_err(), "cannot record dependency on cancelled query");
}
