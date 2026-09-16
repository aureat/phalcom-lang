use crate::support::{MarkedSource, TestLsp};
use serde_json::json;

#[tokio::test]
async fn impl_target_and_member_navigation_use_canonical_definitions() {
    let uri = "file:///impl-navigation.ph";
    let source = "class User {}\nimpl User { greet() -> String { \"hi\" } }\nclass Caller {\n  run(user: User) -> String { user.greet() }\n}\n";
    let mut lsp = TestLsp::start().await;
    lsp.initialize(None).await;
    lsp.open_and_wait(uri, source).await;

    let target_definition = lsp
        .request(
            "textDocument/definition",
            json!({
                "textDocument": { "uri": uri },
                "position": { "line": 1, "character": 6 }
            }),
        )
        .await;
    let target_locations = target_definition["result"].as_array().expect("impl target definition locations");
    assert_eq!(target_locations.len(), 1, "{target_definition:#?}");
    assert_eq!(target_locations[0]["uri"], uri);
    assert_eq!(target_locations[0]["range"]["start"], json!({ "line": 0, "character": 6 }));

    let member_definition = lsp
        .request(
            "textDocument/definition",
            json!({
                "textDocument": { "uri": uri },
                "position": { "line": 3, "character": 35 }
            }),
        )
        .await;
    let member_locations = member_definition["result"].as_array().expect("impl member definition locations");
    assert_eq!(member_locations.len(), 1, "{member_definition:#?}");
    assert_eq!(member_locations[0]["uri"], uri);
    assert_eq!(member_locations[0]["range"]["start"], json!({ "line": 1, "character": 12 }));

    let references = lsp
        .request(
            "textDocument/references",
            json!({
                "textDocument": { "uri": uri },
                "position": { "line": 3, "character": 35 },
                "context": { "includeDeclaration": true }
            }),
        )
        .await;
    let reference_locations = references["result"].as_array().expect("impl member references");
    assert!(
        reference_locations
            .iter()
            .any(|location| location["range"]["start"] == json!({ "line": 1, "character": 12 })),
        "{references:#?}"
    );
    assert!(
        reference_locations
            .iter()
            .any(|location| location["range"]["start"] == json!({ "line": 3, "character": 35 })),
        "{references:#?}"
    );

    let symbols = lsp.request("workspace/symbol", json!({ "query": "impl" })).await;
    let symbols = symbols["result"].as_array().expect("workspace symbols");
    assert!(
        symbols.iter().all(|symbol| symbol["name"] != json!("impl")),
        "impl blocks are not symbols: {symbols:#?}"
    );

    lsp.finish().await;
}

#[tokio::test]
async fn enum_exact_case_and_root_impl_navigation() {
    let uri = "file:///enum-impl-navigation.ph";
    let source = "enum Expr {\n  Lit(val: Int)\n  Add(left: Expr, right: Expr)\n}\nimpl Expr {\n  eval() -> Int\n}\nimpl Expr::Lit(val: _) {\n  eval() -> Int { self.val }\n}\n";
    let mut lsp = TestLsp::start().await;
    lsp.initialize(None).await;
    lsp.open_and_wait(uri, source).await;

    // 1. Navigation on Expr in `impl Expr` -> points to `enum Expr` at line 0, char 5
    let root_target = lsp
        .request(
            "textDocument/definition",
            json!({
                "textDocument": { "uri": uri },
                "position": { "line": 4, "character": 6 }
            }),
        )
        .await;
    let root_locs = root_target["result"].as_array().expect("root target definition");
    assert_eq!(root_locs.len(), 1, "{root_target:#?}");
    assert_eq!(root_locs[0]["range"]["start"], json!({ "line": 0, "character": 5 }));

    // 2. Navigation on Lit in `impl Expr::Lit(val: _)` -> points to variant `Lit` at line 1, char 2
    let case_target = lsp
        .request(
            "textDocument/definition",
            json!({
                "textDocument": { "uri": uri },
                "position": { "line": 7, "character": 12 }
            }),
        )
        .await;
    let case_locs = case_target["result"].as_array().expect("case target definition");
    assert!(
        case_locs.iter().any(|loc| loc["range"]["start"] == json!({ "line": 1, "character": 2 })),
        "{case_target:#?}"
    );

    lsp.finish().await;
}

#[tokio::test]
async fn associated_type_navigation_and_hover_use_canonical_trait_owned_target() {
    let uri = "file:///associated-type-navigation.ph";
    let source = MarkedSource::parse(
        r#"trait Cell {
  type /*@declaration*/Item
  read -> Self::/*@projection*/Item
}
trait Other {
  type Item
}
class Box {}
impl Cell for Box {
  type /*@binding*/Item = Int
  read -> Int { 1 }
}
class Caller {
  run(_ box: Box) -> Int { box.read }
}
"#,
    );
    let mut lsp = TestLsp::start().await;
    lsp.initialize(None).await;
    lsp.open_and_wait(uri, &source.text).await;

    for marker in ["declaration", "binding", "projection"] {
        let definition = lsp
            .request(
                "textDocument/definition",
                json!({
                    "textDocument": { "uri": uri },
                    "position": source.position(marker)
                }),
            )
            .await;
        let locations = definition["result"].as_array().expect("associated type definition locations");
        assert_eq!(locations.len(), 1, "{marker}: {definition:#?}");
        assert_eq!(locations[0]["uri"], uri);
        assert_eq!(
            locations[0]["range"]["start"],
            json!({ "line": 1, "character": 7 }),
            "{marker} must resolve to Cell.Item, not a same-spelled local target"
        );
    }

    let hover = lsp
        .request(
            "textDocument/hover",
            json!({
                "textDocument": { "uri": uri },
                "position": source.position("projection")
            }),
        )
        .await;
    let hover_text = hover["result"]["contents"]["value"].as_str().expect("associated type hover markdown");
    assert!(hover_text.contains("Cell::Item"), "hover must include the canonical owner and associated name: {hover:#?}");
    assert!(hover_text.contains("associated type"), "hover must identify the target kind: {hover:#?}");

    lsp.finish().await;
}
