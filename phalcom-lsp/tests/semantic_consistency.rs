use tower_lsp::lsp_types::Url;

use crate::support::{TestLsp, completion_labels, fixture_path, hint_labels, load_fixture};

#[tokio::test]
async fn completion_and_inlay_hints_share_the_same_semantic_fact() {
    let relative = "semantic/direct_instance.ph";
    let fixture = load_fixture(relative);
    let uri = Url::from_file_path(fixture_path(relative)).unwrap().to_string();

    let mut lsp = TestLsp::start().await;
    lsp.initialize(None).await;
    lsp.open_and_wait(&uri, &fixture.text).await;

    let completion = lsp.completion(&uri, fixture.position("completion")).await;
    let labels = completion_labels(&completion);
    assert!(labels.iter().any(|x| x == "greet()"), "{labels:#?}");

    let hints = hint_labels(&lsp.inlay_hints(&uri, 100).await);
    assert!(
        hints.iter().any(|x| x.contains("Person")),
        "completion knows Person's surface but inlay hints do not expose Person: {hints:#?}"
    );

    lsp.finish().await;
}

#[tokio::test]
async fn trusted_system_print_return_never_surfaces_as_option_in_ide_hints() {
    let uri = "file:///tmp/phalcom-system-print-return.ph";
    let source = "class Demo {\n  run() {\n    System.print(\"hello\")\n  }\n}\n";

    let mut lsp = TestLsp::start().await;
    lsp.initialize(None).await;
    lsp.open_and_wait(uri, source).await;

    let hover = lsp.hover(uri, tower_lsp::lsp_types::Position::new(2, 12)).await;
    assert!(
        !hover.to_string().contains("Option"),
        "trusted System.print hover must not report Option: {hover:#?}"
    );

    let hints = hint_labels(&lsp.inlay_hints(uri, 100).await);
    assert!(
        !hints.iter().any(|label| label.contains("Option")),
        "trusted System.print inlay must not report Option: {hints:#?}"
    );

    lsp.finish().await;
}

#[tokio::test]
async fn local_binding_definition_and_references_are_precise() {
    let relative = "semantic/binding_goto_def.ph";
    let fixture = load_fixture(relative);
    let uri = Url::from_file_path(fixture_path(relative)).unwrap().to_string();

    let mut lsp = TestLsp::start().await;
    lsp.initialize(None).await;
    lsp.open_and_wait(&uri, &fixture.text).await;

    // Go to definition of "value" reference
    let def_response = lsp
        .request(
            "textDocument/definition",
            serde_json::json!({
                "textDocument": { "uri": uri },
                "position": fixture.position("use")
            }),
        )
        .await;

    let def_locs = def_response["result"].as_array().expect("array of locations");
    assert_eq!(def_locs.len(), 1);
    let def_pos = &def_locs[0]["range"]["start"];
    let decl_pos = fixture.position("decl");
    assert_eq!(def_pos["line"], decl_pos.line);
    assert_eq!(def_pos["character"], decl_pos.character);

    // References to "value"
    let ref_response = lsp
        .request(
            "textDocument/references",
            serde_json::json!({
                "textDocument": { "uri": uri },
                "position": fixture.position("use"),
                "context": { "includeDeclaration": true }
            }),
        )
        .await;

    let ref_locs = ref_response["result"].as_array().expect("array of references");
    assert_eq!(ref_locs.len(), 2); // declaration + use

    lsp.finish().await;
}
