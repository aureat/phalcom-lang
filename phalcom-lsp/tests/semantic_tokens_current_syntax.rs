use serde_json::Value;
use tower_lsp::lsp_types::Url;

use crate::support::{fixture_path, load_fixture, TestLsp};

#[tokio::test]
async fn current_syntax_uses_readable_semantic_token_expectations() {
    let relative = "highlighting/current_syntax.ph";
    let fixture = load_fixture(relative);
    let uri = Url::from_file_path(fixture_path(relative))
        .unwrap()
        .to_string();

    let mut lsp = TestLsp::start().await;
    let init = lsp.initialize(None).await;
    lsp.open(&uri, &fixture.text).await;

    let response = lsp.semantic_tokens_full(&uri).await;
    let decoded = decode(&fixture.text, &init, &response);

    assert_pair(&decoded, "class", "keyword");
    assert_pair(&decoded, "Widget", "class");
    assert_pair(&decoded, "new", "method");
    assert_pair(&decoded, "value", "method");
    assert_pair(&decoded, "#move(_,to)", "selector");
    assert_pair(&decoded, "42", "number");

    lsp.finish().await;
}

fn assert_pair(decoded: &[(String, String)], text: &str, kind: &str) {
    assert!(
        decoded
            .iter()
            .any(|(token_text, token_kind)| token_text == text && token_kind == kind),
        "missing ({text:?}, {kind:?}); decoded={decoded:#?}"
    );
}

fn decode(
    text: &str,
    init: &Value,
    response: &Value,
) -> Vec<(String, String)> {
    let legend =
        init["result"]["capabilities"]["semanticTokensProvider"]["legend"]
            ["tokenTypes"]
            .as_array()
            .expect("semantic token legend")
            .iter()
            .map(|x| x.as_str().expect("token type string"))
            .collect::<Vec<_>>();

    let data = response["result"]["data"]
        .as_array()
        .expect("semantic token data array");

    assert_eq!(
        data.len() % 5,
        0,
        "semantic token data uses 5 integers/token"
    );

    let lines = text.lines().collect::<Vec<_>>();
    let mut out = Vec::new();
    let mut line = 0u32;
    let mut start = 0u32;

    for token in data.chunks(5) {
        let delta_line = token[0].as_u64().unwrap() as u32;
        let delta_start = token[1].as_u64().unwrap() as u32;
        let length = token[2].as_u64().unwrap() as u32;
        let token_type = token[3].as_u64().unwrap() as usize;

        if delta_line == 0 {
            start += delta_start;
        } else {
            line += delta_line;
            start = delta_start;
        }

        let line_text = lines.get(line as usize).copied().unwrap_or("");
        let token_text = utf16_slice(line_text, start, length);
        let kind = legend
            .get(token_type)
            .unwrap_or_else(|| {
                panic!("token type index {token_type} out of legend")
            })
            .to_string();

        out.push((token_text, kind));
    }

    out
}

fn utf16_slice(text: &str, start: u32, len: u32) -> String {
    let units = text.encode_utf16().collect::<Vec<_>>();
    let start = start as usize;
    let end = start + len as usize;

    String::from_utf16(&units[start..end])
        .expect("semantic token lands on valid UTF-16 boundary")
}
