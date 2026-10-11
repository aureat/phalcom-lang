#[test]
fn canonical_universe_semantic_baseline_builds() {
    let baseline = phalcom_semantic::CanonicalUniverseSemanticBaseline::build();
    assert!(!baseline.linked_program().modules.is_empty(), "canonical baseline must retain Universe modules");
}
