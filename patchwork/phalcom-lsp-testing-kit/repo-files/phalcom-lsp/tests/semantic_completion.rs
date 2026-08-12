use tower_lsp::lsp_types::Url;

use crate::support::{completion_labels, fixture_path, load_fixture, TestLsp};

fn file_uri(relative: &str) -> String {
    Url::from_file_path(fixture_path(relative))
        .expect("fixture path URL")
        .to_string()
}

async fn complete_fixture(relative: &str, marker: &str) -> Vec<String> {
    let fixture = load_fixture(relative);
    let uri = file_uri(relative);

    let mut lsp = TestLsp::start().await;
    lsp.initialize(None).await;
    lsp.open(&uri, &fixture.text).await;

    let response = lsp.completion(&uri, fixture.position(marker)).await;
    let labels = completion_labels(&response);

    lsp.finish().await;
    labels
}

#[tokio::test]
async fn user_instance_completion_uses_declared_surface() {
    let labels = complete_fixture("semantic/direct_instance.ph", "completion").await;

    assert!(labels.iter().any(|x| x == "greet()"), "{labels:#?}");
    assert!(labels.iter().any(|x| x == "name"), "{labels:#?}");
    assert!(labels.iter().any(|x| x == "rename(_)"), "{labels:#?}");
    assert!(!labels.iter().any(|x| x == "ifTrue(_)"), "{labels:#?}");
}

#[tokio::test]
async fn inherited_members_are_visible() {
    let labels = complete_fixture("semantic/inheritance.ph", "completion").await;

    assert!(labels.iter().any(|x| x == "bark()"), "{labels:#?}");
    assert!(labels.iter().any(|x| x == "move()"), "{labels:#?}");
    assert!(labels.iter().any(|x| x == "name"), "{labels:#?}");
}

#[tokio::test]
async fn overrides_do_not_duplicate_the_same_selector() {
    let labels = complete_fixture("semantic/override.ph", "completion").await;

    assert_eq!(
        labels.iter().filter(|x| x.as_str() == "run()").count(),
        1,
        "{labels:#?}"
    );
    assert!(labels.iter().any(|x| x == "parentOnly()"), "{labels:#?}");
    assert!(labels.iter().any(|x| x == "childOnly()"), "{labels:#?}");
}

#[tokio::test]
async fn super_completion_starts_at_the_lexical_superclass() {
    let labels = complete_fixture("semantic/super_send.ph", "super").await;

    assert!(labels.iter().any(|x| x == "parentOnly()"), "{labels:#?}");
    assert!(labels.iter().any(|x| x == "grandOnly()"), "{labels:#?}");
    assert!(labels.iter().any(|x| x == "shared()"), "{labels:#?}");
    assert!(
        !labels.iter().any(|x| x == "childOnly()"),
        "super must not use the child's ordinary instance surface: {labels:#?}"
    );
}

#[tokio::test]
async fn class_and_instance_surfaces_do_not_leak() {
    let class_labels = complete_fixture("semantic/class_instance_side.ph", "class").await;
    let instance_labels = complete_fixture("semantic/class_instance_side.ph", "instance").await;

    assert!(
        class_labels.iter().any(|x| x == "make()"),
        "{class_labels:#?}"
    );
    assert!(
        !class_labels.iter().any(|x| x == "render()"),
        "instance member leaked to class-side completion: {class_labels:#?}"
    );

    assert!(
        instance_labels.iter().any(|x| x == "render()"),
        "{instance_labels:#?}"
    );
    assert!(
        !instance_labels.iter().any(|x| x == "make()"),
        "class-side member leaked to instance completion: {instance_labels:#?}"
    );
}

#[tokio::test]
async fn chained_receiver_uses_method_return_summary() {
    let labels = complete_fixture("semantic/chained_receivers.ph", "completion").await;
    assert!(labels.iter().any(|x| x == "greet()"), "{labels:#?}");
}

#[tokio::test]
async fn field_receiver_uses_field_knowledge() {
    let labels = complete_fixture("semantic/fields.ph", "field").await;
    assert!(labels.iter().any(|x| x == "request()"), "{labels:#?}");
}

#[tokio::test]
async fn unknown_receiver_does_not_fabricate_a_fixture_class_surface() {
    let labels = complete_fixture("semantic/unknown_receiver.ph", "unknown").await;

    assert!(
        !labels.iter().any(|x| {
            x == "request()" || x == "greet()" || x == "bark()"
        }),
        "unknown receiver was treated as a known fixture class: {labels:#?}"
    );
}

#[tokio::test]
async fn completion_survives_a_trailing_dot_in_incomplete_source() {
    let labels = complete_fixture("incomplete/trailing_dot.ph", "completion").await;
    assert!(labels.iter().any(|x| x == "greet()"), "{labels:#?}");
}
