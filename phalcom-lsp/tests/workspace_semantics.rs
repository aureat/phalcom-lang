use crate::support::{MarkedSource, TestLsp, TestWorkspace, completion_labels};
use serde_json::json;

#[tokio::test]
async fn same_named_classes_in_different_modules_keep_distinct_identity() {
    let workspace = TestWorkspace::from_fixture_dir("workspace");
    let main_uri = workspace.file_uri("main.ph");
    let main = MarkedSource::parse(&workspace.read("main.ph"));

    let mut lsp = TestLsp::start().await;
    lsp.initialize(Some(&workspace.uri())).await;
    lsp.open_and_wait(&main_uri, &main.text).await;

    let a = completion_labels(&lsp.completion(&main_uri, main.position("a")).await);
    let b = completion_labels(&lsp.completion(&main_uri, main.position("b")).await);

    assert!(a.iter().any(|x| x == "aOnly()"), "A.User surface: {a:#?}");
    assert!(!a.iter().any(|x| x == "bOnly()"), "B.User leaked into A.User: {a:#?}");

    assert!(b.iter().any(|x| x == "bOnly()"), "B.User surface: {b:#?}");
    assert!(!b.iter().any(|x| x == "aOnly()"), "A.User leaked into B.User: {b:#?}");

    lsp.finish().await;
}

#[tokio::test]
async fn editing_an_imported_provider_invalidates_consumer_completion() {
    let workspace = TestWorkspace::from_fixture_dir("workspace");

    let before = workspace.read("provider_before.ph");
    let after = workspace.read("provider_after.ph");
    workspace.write("provider.ph", &before);

    let provider_uri = workspace.file_uri("provider.ph");
    let consumer_uri = workspace.file_uri("provider_consumer.ph");
    let consumer = MarkedSource::parse(&workspace.read("provider_consumer.ph"));

    let mut lsp = TestLsp::start().await;
    lsp.initialize(Some(&workspace.uri())).await;
    lsp.open_and_wait(&provider_uri, &before).await;
    lsp.open_and_wait(&consumer_uri, &consumer.text).await;

    let old_labels = completion_labels(&lsp.completion(&consumer_uri, consumer.position("product")).await);
    assert!(old_labels.iter().any(|x| x == "oldMethod()"), "{old_labels:#?}");

    let before_change = lsp.counter_snapshot();
    lsp.change(&provider_uri, &after).await;
    lsp.wait_for_semantic_publication_after(before_change).await;

    let new_labels = completion_labels(&lsp.completion(&consumer_uri, consumer.position("product")).await);
    assert!(new_labels.iter().any(|x| x == "newMethod()"), "{new_labels:#?}");
    assert!(
        !new_labels.iter().any(|x| x == "oldMethod()"),
        "stale provider semantic facts survived didChange: {new_labels:#?}"
    );

    lsp.finish().await;
}

#[tokio::test]
async fn parameter_facts_from_multiple_consumer_modules_join_instead_of_overwriting() {
    let workspace = TestWorkspace::from_fixture_dir("interprocedural_join");
    let a_uri = workspace.file_uri("consumer_a.ph");
    let b_uri = workspace.file_uri("consumer_b.ph");
    let a = MarkedSource::parse(&workspace.read("consumer_a.ph"));
    let b = MarkedSource::parse(&workspace.read("consumer_b.ph"));

    let mut lsp = TestLsp::start().await;
    lsp.initialize(Some(&workspace.uri())).await;
    lsp.open_and_wait(&a_uri, &a.text).await;
    lsp.open_and_wait(&b_uri, &b.text).await;

    let a_labels = completion_labels(&lsp.completion(&a_uri, a.position("result")).await);
    assert!(
        a_labels.iter().any(|label| label == "catOnly()"),
        "joined Service.consume return lost Cat: {a_labels:#?}"
    );
    assert!(
        a_labels.iter().any(|label| label == "dogOnly()"),
        "joined Service.consume return lost Dog: {a_labels:#?}"
    );

    let b_labels = completion_labels(&lsp.completion(&b_uri, b.position("result")).await);
    assert!(
        b_labels.iter().any(|label| label == "catOnly()"),
        "joined Service.consume return lost Cat: {b_labels:#?}"
    );
    assert!(
        b_labels.iter().any(|label| label == "dogOnly()"),
        "joined Service.consume return lost Dog: {b_labels:#?}"
    );

    lsp.finish().await;
}

#[tokio::test]
async fn inferred_parameter_facts_propagate_through_forwarding_calls() {
    let workspace = TestWorkspace::from_fixture_dir("interprocedural_forward");
    let consumer_uri = workspace.file_uri("consumer.ph");
    let consumer = MarkedSource::parse(&workspace.read("consumer.ph"));

    let mut lsp = TestLsp::start().await;
    lsp.initialize(Some(&workspace.uri())).await;
    lsp.open_and_wait(&consumer_uri, &consumer.text).await;

    let labels = completion_labels(&lsp.completion(&consumer_uri, consumer.position("result")).await);
    assert!(
        labels.iter().any(|label| label == "productOnly()"),
        "Relay.forward(value) -> sink(value) did not propagate Product through the parameter fixed point: {labels:#?}"
    );

    lsp.finish().await;
}

#[tokio::test]
async fn inherited_hover_reports_the_defining_owner_not_the_receiver_class() {
    let workspace = TestWorkspace::from_fixture_dir("hover_identity");
    let uri = workspace.file_uri("inherited.ph");
    let source = MarkedSource::parse(&workspace.read("inherited.ph"));

    let mut lsp = TestLsp::start().await;
    lsp.initialize(Some(&workspace.uri())).await;
    lsp.open_and_wait(&uri, &source.text).await;

    let response = lsp
        .request(
            "textDocument/hover",
            json!({
                "textDocument": { "uri": uri },
                "position": source.position("speak")
            }),
        )
        .await;
    let value = response["result"]["contents"]["value"].as_str().expect("hover markdown");
    assert!(value.contains("on Animal"), "inherited member must report defining owner Animal: {value:?}");
    assert!(
        !value.contains("on Dog"),
        "receiver class Dog was incorrectly presented as defining owner: {value:?}"
    );
    assert!(
        value.contains("Animal speech documentation."),
        "hover must harvest docs from the defining member: {value:?}"
    );

    lsp.finish().await;
}

#[tokio::test]
async fn phaldoc_is_attached_to_the_resolved_declaration_not_the_first_matching_selector() {
    let workspace = TestWorkspace::from_fixture_dir("hover_identity");
    let uri = workspace.file_uri("duplicate_docs.ph");
    let source = MarkedSource::parse(&workspace.read("duplicate_docs.ph"));

    let mut lsp = TestLsp::start().await;
    lsp.initialize(Some(&workspace.uri())).await;
    lsp.open_and_wait(&uri, &source.text).await;

    let response = lsp
        .request(
            "textDocument/hover",
            json!({
                "textDocument": { "uri": uri },
                "position": source.position("b_ping")
            }),
        )
        .await;
    let value = response["result"]["contents"]["value"].as_str().expect("hover markdown");
    assert!(value.contains("B ping documentation."), "B.ping() must carry B's Phaldoc: {value:?}");
    assert!(
        !value.contains("A ping documentation."),
        "selector-only harvesting leaked A.ping() docs into B.ping(): {value:?}"
    );

    lsp.finish().await;
}

#[tokio::test]
async fn class_hover_surfaces_adjacent_class_phaldoc() {
    let workspace = TestWorkspace::from_fixture_dir("hover_identity");
    let uri = workspace.file_uri("class_docs.ph");
    let source = MarkedSource::parse(&workspace.read("class_docs.ph"));

    let mut lsp = TestLsp::start().await;
    lsp.initialize(Some(&workspace.uri())).await;
    lsp.open_and_wait(&uri, &source.text).await;

    let response = lsp
        .request(
            "textDocument/hover",
            json!({
                "textDocument": { "uri": uri },
                "position": source.position("widget")
            }),
        )
        .await;
    let value = response["result"]["contents"]["value"].as_str().expect("class hover markdown");
    assert!(value.contains("Widget"), "class hover must identify Widget: {value:?}");
    assert!(value.contains("Widget class documentation."), "class Phaldoc missing from hover: {value:?}");

    lsp.finish().await;
}

#[tokio::test]
async fn receiver_qualified_definition_does_not_fall_back_to_every_global_selector_match() {
    let workspace = TestWorkspace::from_fixture_dir("hover_identity");
    let uri = workspace.file_uri("unknown_definition.ph");
    let source = MarkedSource::parse(&workspace.read("unknown_definition.ph"));

    let mut lsp = TestLsp::start().await;
    lsp.initialize(Some(&workspace.uri())).await;
    lsp.open_and_wait(&uri, &source.text).await;

    let response = lsp
        .request(
            "textDocument/definition",
            json!({
                "textDocument": { "uri": uri },
                "position": source.position("ping")
            }),
        )
        .await;
    assert!(
        response["result"].is_null(),
        "unknown receiver must not jump to all A.ping()/B.ping() definitions: {response:#?}"
    );

    lsp.finish().await;
}

#[tokio::test]
async fn unimported_workspace_class_is_not_semantic_authority_for_hover() {
    let workspace = TestWorkspace::from_fixture_dir("module_visibility");
    let uri = workspace.file_uri("consumer.ph");
    let source = MarkedSource::parse(&workspace.read("consumer.ph"));

    let mut lsp = TestLsp::start().await;
    lsp.initialize(Some(&workspace.uri())).await;
    lsp.open_and_wait(&uri, &source.text).await;

    let response = lsp
        .request(
            "textDocument/hover",
            json!({
                "textDocument": { "uri": uri },
                "position": source.position("product_only")
            }),
        )
        .await;
    assert!(
        response["result"].is_null(),
        "consumer did not import provider.ph, so unique workspace class Product must not resolve semantically: {response:#?}"
    );

    lsp.finish().await;
}
