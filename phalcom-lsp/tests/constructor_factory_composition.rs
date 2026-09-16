use serde_json::{Value, json};
use tower_lsp::lsp_types::Url;

use crate::support::{MarkedSource, TestLsp, completion_labels};

#[tokio::test]
async fn constructor_factory_inference_is_authoritative_across_lsp_features() {
    let source = MarkedSource::parse(
        r#"
class CellNum {
  _raw: Int

  @constructor
  /*@constructor_decl*/new(_ raw: Int) {
    _raw = raw
  }

  @class
  /*@factory_decl*/of(_ raw: Int) {
    CellNum./*@constructor_call*/new(raw)
  }

  value() -> Int {
    _raw
  }
}

const /*@binding_decl*/x: /*@annotation*/Int =
  CellNum./*@factory_call*/of(42)

class Probe {
  run() {
    /*@binding_use*/x./*@member_completion*/value()
  }
}
"#,
    );

    let path = std::env::temp_dir().join(format!("phalcom-constructor-factory-{}.ph", std::process::id()));

    std::fs::write(&path, &source.text).unwrap();

    let uri = Url::from_file_path(&path).unwrap().to_string();

    let mut lsp = TestLsp::start().await;
    lsp.initialize(None).await;
    lsp.open_and_wait(&uri, &source.text).await;

    //
    // 1. The factory itself must be inferred as returning CellNum.
    //
    // This is the central semantic fact:
    //
    //     CellNum.new(raw)       -> CellNum   (@constructor)
    //     CellNum.of(raw) body   -> CellNum
    //     CellNum.of(42)         -> CellNum
    //
    let factory_hover = lsp.hover(&uri, source.position("factory_call")).await;

    let factory_hover_text = hover_text(&factory_hover);

    assert!(
        factory_hover_text.contains("CellNum"),
        "factory hover must expose inferred CellNum return type:\n{factory_hover:#?}"
    );

    //
    // 2. The constructor call inside `of` must resolve to the actual
    //    @constructor declaration.
    //
    let constructor_def = lsp
        .request(
            "textDocument/definition",
            json!({
                "textDocument": { "uri": uri },
                "position": source.position("constructor_call"),
            }),
        )
        .await;

    assert_same_file_position(
        &constructor_def,
        &uri,
        source.position("constructor_decl"),
        "CellNum.new inside CellNum.of must resolve to the constructor",
    );

    //
    // 3. CellNum.of at the binding initializer must resolve to the
    //    class-side factory declaration.
    //
    let factory_def = lsp
        .request(
            "textDocument/definition",
            json!({
                "textDocument": { "uri": uri },
                "position": source.position("factory_call"),
            }),
        )
        .await;

    assert_same_file_position(
        &factory_def,
        &uri,
        source.position("factory_decl"),
        "CellNum.of must resolve to its class-side declaration",
    );

    //
    // 4. CRITICAL:
    //
    //      const x: Int = CellNum.of(42)
    //
    // The annotation is programmer input. It must not rewrite the
    // authoritative inferred value type.
    //
    // Even though `Int` is written here, x's value is known to be CellNum.
    //
    let binding_hover = lsp.hover(&uri, source.position("binding_use")).await;
    let binding_hover_text = hover_text(&binding_hover);

    assert!(
        binding_hover_text.contains("CellNum"),
        "x must retain the authoritative inferred CellNum type; the incorrect `: Int` annotation must not poison semantic inference:\n{binding_hover:#?}",
    );

    //
    // 5. Completion must consume the SAME semantic fact.
    //
    // If hover knows x is CellNum but completion treats x as Int, the
    // semantic presentation/query layers disagree.
    //
    let completion = lsp.completion(&uri, source.position("member_completion")).await;

    let labels = completion_labels(&completion);

    assert!(
        labels.iter().any(|label| label == "value()"),
        "completion on x must use CellNum's surface despite the bad Int annotation; got {labels:#?}",
    );

    //
    // 6. Navigation on x must still point to the binding declaration.
    //
    // Type mismatch must not damage identity/reference information.
    //
    let binding_def = lsp
        .request(
            "textDocument/definition",
            json!({
                "textDocument": { "uri": uri },
                "position": source.position("binding_use"),
            }),
        )
        .await;

    assert_same_file_position(
        &binding_def,
        &uri,
        source.position("binding_decl"),
        "x usage must resolve to its declaration despite its invalid annotation",
    );

    //
    // 7. The annotation itself must resolve independently to the real Int
    //    declaration.
    //
    // This proves that:
    //
    //     annotation identity = Int
    //     inferred value type = CellNum
    //
    // are simultaneously represented rather than one overwriting the other.
    //
    let int_def = lsp
        .request(
            "textDocument/definition",
            json!({
                "textDocument": { "uri": uri },
                "position": source.position("annotation"),
            }),
        )
        .await;

    let int_locations = int_def["result"].as_array().expect("Int definition must return locations");

    assert_eq!(int_locations.len(), 1, "Int annotation should resolve to exactly one canonical declaration");

    let int_uri = int_locations[0]["uri"].as_str().expect("Int definition URI");

    assert_ne!(int_uri, uri, "Int must resolve to its universe/core declaration, not local source");

    //
    // 8. Follow semantic navigation all the way to source provenance.
    //
    // If the core declaration is virtual, the URI returned by goto-definition
    // must also be consumable by phalcom/sourceText.
    //
    if int_uri.starts_with("phalcom://") {
        let source_response = lsp
            .request(
                "phalcom/sourceText",
                json!({
                    "uri": int_uri,
                }),
            )
            .await;

        let int_source = source_response["result"]
            .as_str()
            .expect("virtual Int definition must expose canonical source text");

        assert!(
            int_source.contains("class Int"),
            "Int definition URI must resolve to canonical Int source:\n{int_source}"
        );
    }

    lsp.finish().await;

    let _ = std::fs::remove_file(path);
}

fn hover_text(response: &Value) -> String {
    let contents = &response["result"]["contents"];

    if let Some(value) = contents["value"].as_str() {
        return value.to_owned();
    }

    if let Some(value) = contents.as_str() {
        return value.to_owned();
    }

    panic!("unexpected hover representation: {response:#?}");
}

fn assert_same_file_position(response: &Value, expected_uri: &str, expected: tower_lsp::lsp_types::Position, message: &str) {
    let locations = response["result"]
        .as_array()
        .unwrap_or_else(|| panic!("{message}: expected location array: {response:#?}"));

    assert_eq!(locations.len(), 1, "{message}: expected exactly one location: {locations:#?}");

    assert_eq!(locations[0]["uri"].as_str(), Some(expected_uri), "{message}: wrong target file");

    assert_eq!(
        locations[0]["range"]["start"]["line"].as_u64(),
        Some(expected.line as u64),
        "{message}: wrong target line"
    );

    assert_eq!(
        locations[0]["range"]["start"]["character"].as_u64(),
        Some(expected.character as u64),
        "{message}: wrong target character"
    );
}
