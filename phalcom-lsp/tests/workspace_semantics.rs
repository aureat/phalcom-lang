use crate::support::{MarkedSource, TestLsp, TestWorkspace, completion_labels};

#[tokio::test]
async fn same_named_classes_in_different_modules_keep_distinct_identity() {
    let workspace = TestWorkspace::from_fixture_dir("workspace");
    let main_uri = workspace.file_uri("main.ph");
    let main = MarkedSource::parse(&workspace.read("main.ph"));

    let mut lsp = TestLsp::start().await;
    lsp.initialize(Some(&workspace.uri())).await;
    lsp.open(&main_uri, &main.text).await;

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
    lsp.open(&provider_uri, &before).await;
    lsp.open(&consumer_uri, &consumer.text).await;

    let old_labels = completion_labels(&lsp.completion(&consumer_uri, consumer.position("product")).await);
    assert!(old_labels.iter().any(|x| x == "oldMethod()"), "{old_labels:#?}");

    lsp.change(&provider_uri, &after).await;

    let new_labels = completion_labels(&lsp.completion(&consumer_uri, consumer.position("product")).await);
    assert!(new_labels.iter().any(|x| x == "newMethod()"), "{new_labels:#?}");
    assert!(
        !new_labels.iter().any(|x| x == "oldMethod()"),
        "stale provider semantic facts survived didChange: {new_labels:#?}"
    );

    lsp.finish().await;
}
