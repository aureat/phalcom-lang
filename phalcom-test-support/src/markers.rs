use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MarkerPosition {
    pub byte_offset: usize,
    pub line: u32,
    pub utf16_character: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MarkedSource {
    pub text: String,
    positions: BTreeMap<String, MarkerPosition>,
}

impl MarkedSource {
    pub fn parse(input: &str) -> Self {
        let mut output = String::with_capacity(input.len());
        let mut positions = BTreeMap::new();
        let mut rest = input;

        loop {
            let Some(start) = rest.find("/*@") else {
                output.push_str(rest);
                break;
            };

            output.push_str(&rest[..start]);
            let tail = &rest[start + 3..];
            let end = tail.find("*/").unwrap_or_else(|| panic!("unterminated golden marker near {tail:?}"));
            let name = &tail[..end];
            assert!(!name.is_empty(), "golden marker name must not be empty");
            let position = position_of(&output);
            let old = positions.insert(name.to_string(), position);
            assert!(old.is_none(), "duplicate golden marker {name:?}");
            rest = &tail[end + 2..];
        }

        Self { text: output, positions }
    }

    pub fn position(&self, name: &str) -> MarkerPosition {
        *self.positions.get(name).unwrap_or_else(|| {
            panic!("golden source has no marker named {name:?}; markers={:?}", self.positions.keys())
        })
    }

    pub fn markers(&self) -> impl Iterator<Item = (&str, MarkerPosition)> {
        self.positions.iter().map(|(name, position)| (name.as_str(), *position))
    }
}

fn position_of(text: &str) -> MarkerPosition {
    let line_start = text.rfind('\n').map_or(0, |idx| idx + 1);
    let line = text[..line_start].bytes().filter(|byte| *byte == b'\n').count() as u32;
    let utf16_character = text[line_start..].encode_utf16().count() as u32;
    MarkerPosition { byte_offset: text.len(), line, utf16_character }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_markers_and_records_utf16_positions() {
        let source = MarkedSource::parse("α./*@completion*/beta()\n");
        assert_eq!(source.text, "α.beta()\n");
        assert_eq!(
            source.position("completion"),
            MarkerPosition { byte_offset: 3, line: 0, utf16_character: 2 }
        );
    }
}
