use phalcom_semantic::canonical_image::{CanonicalUniverseFormalImage, canonical_universe_formal_image};

#[test]
fn canonical_formal_image_is_complete_and_process_shared() {
    let first = canonical_universe_formal_image().expect("canonical Universe formal image");
    let second = canonical_universe_formal_image().expect("canonical Universe formal image");

    assert!(std::ptr::eq(first, second));
    assert_eq!(phalcom_semantic::canonical_image::canonical_universe_formal_image_build_count(), 1);
    assert_eq!(first.sources().len(), first.interfaces().len());
    assert_eq!(first.sources().len(), first.linked().modules.len());
    assert!(first.sources().keys().all(|module| module.project.is_universe()));
    assert!(first.sources().keys().all(|module| first.snapshot().sources.contains_key(module)));
    assert!(first.query_image().len() > first.sources().len());
    assert!(
        first
            .query_image()
            .entries()
            .all(|(key, entry)| entry.dependencies.iter().all(|dependency| dependency.dependent == *key))
    );
}

#[test]
fn canonical_formal_image_is_send_and_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<CanonicalUniverseFormalImage>();
}
