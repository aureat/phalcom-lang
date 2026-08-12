use tower_lsp::lsp_types::Url;

use crate::support::{
    completion_labels, fixture_path, hint_labels, load_fixture, TestLsp,
};

#[tokio::test]
async fn completion_and_inlay_hints_share_the_same_semantic_fact() {
    let relative = "semantic/direct_instance.ph";
    let fixture = load_fixture(relative);
    let uri = Url::from_file_path(fixture_path(relative))
        .unwrap()
        .to_string();

    let mut lsp = TestLsp::start().await;
    lsp.initialize(None).await;
    lsp.open(&uri, &fixture.text).await;

    let completion = lsp
        .completion(&uri, fixture.position("completion"))
        .await;
    let labels = completion_labels(&completion);
    assert!(labels.iter().any(|x| x == "greet()"), "{labels:#?}");

    let hints = hint_labels(&lsp.inlay_hints(&uri, 100).await);
    assert!(
        hints.iter().any(|x| x.contains("Person")),
        "completion knows Person's surface but inlay hints do not expose Person: {hints:#?}"
    );

    lsp.finish().await;
}
