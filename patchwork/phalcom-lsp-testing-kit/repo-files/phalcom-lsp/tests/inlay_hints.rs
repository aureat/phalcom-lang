use tower_lsp::lsp_types::Url;

use crate::support::{fixture_path, hint_labels, load_fixture, TestLsp};

#[tokio::test]
async fn stable_runtime_shapes_are_exposed_as_inlay_hints() {
    let relative = "semantic/inlay_basic.ph";
    let fixture = load_fixture(relative);
    let uri = Url::from_file_path(fixture_path(relative))
        .unwrap()
        .to_string();

    let mut lsp = TestLsp::start().await;
    let init = lsp.initialize(None).await;

    assert!(
        init["result"]["capabilities"]["inlayHintProvider"].is_boolean()
            || init["result"]["capabilities"]["inlayHintProvider"].is_object(),
        "server must advertise inlay hints: {init:#?}"
    );

    lsp.open(&uri, &fixture.text).await;

    let response = lsp.inlay_hints(&uri, 100).await;
    let labels = hint_labels(&response);

    assert!(labels.iter().any(|x| x.contains("Int")), "{labels:#?}");
    assert!(
        labels.iter().any(|x| x.contains("Person")),
        "{labels:#?}"
    );
    assert!(
        labels.iter().all(|x| !x.contains("Unknown")),
        "Unknown must not be displayed as a useful type hint: {labels:#?}"
    );

    lsp.finish().await;
}

#[tokio::test]
async fn unknown_parameter_does_not_get_a_fake_precise_hint() {
    let relative = "semantic/unknown_receiver.ph";
    let fixture = load_fixture(relative);
    let uri = Url::from_file_path(fixture_path(relative))
        .unwrap()
        .to_string();

    let mut lsp = TestLsp::start().await;
    lsp.initialize(None).await;
    lsp.open(&uri, &fixture.text).await;

    let labels = hint_labels(&lsp.inlay_hints(&uri, 100).await);

    assert!(
        labels.iter().all(|x| !x.contains("Unknown")),
        "Unknown is analysis absence, not a source-level type: {labels:#?}"
    );

    lsp.finish().await;
}
