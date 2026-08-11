use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

use tower_lsp::lsp_types::Position;

#[derive(Debug, Clone)]
pub struct MarkedSource {
    pub text: String,
    positions: HashMap<String, Position>,
}

impl MarkedSource {
    pub fn parse(input: &str) -> Self {
        let mut output = String::with_capacity(input.len());
        let mut positions = HashMap::new();
        let mut rest = input;

        loop {
            let Some(start) = rest.find("/*@") else {
                output.push_str(rest);
                break;
            };

            output.push_str(&rest[..start]);
            let marker_tail = &rest[start + 3..];
            let end = marker_tail
                .find("*/")
                .unwrap_or_else(|| panic!("unterminated fixture marker near {marker_tail:?}"));
            let name = &marker_tail[..end];

            assert!(!name.is_empty(), "fixture marker name must not be empty");
            let old = positions.insert(name.to_string(), position_of(&output));
            assert!(old.is_none(), "duplicate fixture marker {name:?}");

            rest = &marker_tail[end + 2..];
        }

        Self { text: output, positions }
    }

    pub fn position(&self, name: &str) -> Position {
        *self
            .positions
            .get(name)
            .unwrap_or_else(|| panic!("fixture has no marker named {name:?}; markers={:?}", self.positions.keys()))
    }
}

pub fn fixture_path(relative: impl AsRef<Path>) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests").join("fixtures").join(relative)
}

pub fn load_fixture(relative: impl AsRef<Path>) -> MarkedSource {
    let path = fixture_path(relative);
    let text = fs::read_to_string(&path).unwrap_or_else(|err| panic!("failed to read fixture {}: {err}", path.display()));
    MarkedSource::parse(&text)
}

fn position_of(text: &str) -> Position {
    let line_start = text.rfind('\n').map_or(0, |idx| idx + 1);
    let line = text[..line_start].chars().filter(|c| *c == '\n').count() as u32;
    let character = text[line_start..].encode_utf16().count() as u32;
    Position { line, character }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_markers_and_records_utf16_positions() {
        let fixture = MarkedSource::parse("α./*@x*/beta()\n");
        assert_eq!(fixture.text, "α.beta()\n");
        assert_eq!(fixture.position("x"), Position { line: 0, character: 2 });
    }
}
