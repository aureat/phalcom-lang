use phalcom_compiler_artifact::ValidationLimits;
use phalcom_compiler_artifact::semantic::static_value::{
    ArtifactStaticSymbolId, ArtifactStaticValueId, StaticPackBackingV1, StaticValueGraphV1, StaticValueV1,
};

#[test]
fn static_graph_rejects_duplicate_relocated_pack_labels() {
    let graph = StaticValueGraphV1 {
        values: vec![StaticValueV1::Unit, StaticValueV1::Unit],
        strings: vec![],
        symbols: vec!["label".into()],
        tuples: vec![],
        sequences: vec![],
        records: vec![],
        data_values: vec![],
        variant_values: vec![],
        pack_backings: vec![StaticPackBackingV1 {
            positionals: vec![],
            labels: vec![ArtifactStaticSymbolId(0), ArtifactStaticSymbolId(0)],
            labeled_values: vec![ArtifactStaticValueId(0), ArtifactStaticValueId(1)],
        }],
        packs: vec![],
    };

    let error = graph
        .validate(&ValidationLimits::default())
        .expect_err("duplicate static labels must be rejected");
    assert!(error.to_string().contains("duplicate static pack label"), "unexpected error: {error}");
}
