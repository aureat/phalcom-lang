//! Byte-offset to LSP `(line, character)` position mapping.
//!
//! `phalcom-ast` reports every [`phalcom_ast::error::SyntaxError`] as a **byte**
//! half-open [`std::ops::Range`] into the source string. LSP positions are
//! `(line, character)` pairs where `character` is a **UTF-16 code-unit**
//! offset from the start of the line (the LSP default `positionEncoding`,
//! ADR-0056 §5, DEC-LSP-C). [`LineIndex`] is the single place that
//! byte-offset↔position conversion happens in `phalcom-lsp`; nothing else in
//! the crate should do this math ad hoc.
//!
//! This is called out in ADR-0056 as the standing correctness hotspot: an
//! off-by-one or byte-vs-code-unit bug here misplaces *every* diagnostic and
//! future jump target. Hence exhaustive table-driven tests over ASCII,
//! multibyte, and line-ending edge cases below.

use tower_lsp::lsp_types::Position;

/// Maps byte offsets in a source string to LSP `(line, character)` positions.
///
/// Built once per document version from its full text ([`LineIndex::new`])
/// and re-built whenever the text changes (`phalcom-lsp` uses
/// `textDocumentSync=Full`, so every change replaces the whole document and
/// its `LineIndex`). Lines are delimited by `\n`; a preceding `\r` (CRLF) is
/// treated as ordinary line content for column-counting purposes, matching
/// the common LSP server convention of splitting only on `\n`.
///
/// `character` is counted in **UTF-16 code units**, per the LSP spec default
/// encoding and this server's negotiated `positionEncoding` (ADR-0056 §5).
#[derive(Debug, Clone)]
pub struct LineIndex {
    /// The full document text this index was built from.
    ///
    /// Retained so [`LineIndex::position`] can count UTF-16 code units
    /// between a line's start and the requested offset without the caller
    /// re-supplying the text.
    text: String,
    /// Byte offset of the start of each line, in ascending order.
    ///
    /// `line_starts[0]` is always `0`. `line_starts[i]` is the byte
    /// immediately after the `i`-th `\n` in the text (i.e. the first byte of
    /// line `i`).
    line_starts: Vec<usize>,
}

impl LineIndex {
    /// Builds a [`LineIndex`] over `text`, scanning once for line breaks.
    ///
    /// Line breaks are recognized as `\n` (LF); a `\r` immediately preceding
    /// a `\n` (CRLF) is left as ordinary content on the preceding line, so a
    /// byte offset landing exactly on the `\r` still maps to a valid
    /// in-line column rather than being folded into the newline.
    pub fn new(text: &str) -> Self {
        let mut line_starts = vec![0];
        for (i, b) in text.bytes().enumerate() {
            if b == b'\n' {
                line_starts.push(i + 1);
            }
        }
        Self {
            text: text.to_string(),
            line_starts,
        }
    }

    /// Converts a byte offset into `text` into a 0-based LSP [`Position`].
    ///
    /// `offset` is clamped to `text.len()` if it overshoots (e.g. an
    /// end-of-file [`phalcom_ast::error::SyntaxError`] whose zero-width range
    /// sits exactly at the end of the source). The offset need not fall on a
    /// UTF-8 char boundary defensively — `phalcom-ast` always produces
    /// boundary-aligned spans — but this function does not panic if it
    /// doesn't: it counts whole chars up to (and including, if it lands
    /// mid-char) the offset.
    pub fn position(&self, offset: usize) -> Position {
        let offset = offset.min(self.text.len());
        let line = self.line_of(offset);
        let line_start = self.line_starts[line];
        // Count UTF-16 code units from the start of the line up to `offset`,
        // stopping at (not past) any char whose byte range would cross
        // `offset` — i.e. only whole chars fully before `offset` count.
        let character: usize = self.text[line_start..]
            .char_indices()
            .take_while(|(i, c)| line_start + i + c.len_utf8() <= offset)
            .map(|(_, c)| c.len_utf16())
            .sum();
        Position {
            line: line as u32,
            character: character as u32,
        }
    }

    /// Converts a 0-based LSP [`Position`] back into a byte offset into
    /// `text` — the inverse of [`LineIndex::position`].
    ///
    /// Used by `textDocument/definition` and `textDocument/references`
    /// (`backend.rs`) to turn the cursor's client-reported position into the
    /// byte offset [`crate::index::selector_at_offset`] needs.
    ///
    /// `position.line` is clamped to the last known line if it overshoots;
    /// `position.character` (UTF-16 code units) is clamped to the end of
    /// that line if it overshoots. Both clamps make this total rather than
    /// panicking on a stale or out-of-range client position.
    pub fn offset(&self, position: Position) -> usize {
        let line = (position.line as usize).min(self.line_starts.len() - 1);
        let line_start = self.line_starts[line];
        let line_end = self.line_starts.get(line + 1).copied().unwrap_or(self.text.len());
        // Exclude the line's own trailing `\n` from the content scanned: a
        // character offset clamps to just before the newline, not past it
        // (mirrors `position`, which never reports a character position
        // inside the terminator).
        let content_end = if line_end > line_start && self.text.as_bytes()[line_end - 1] == b'\n' {
            line_end - 1
        } else {
            line_end
        };

        let mut remaining_units = position.character as usize;
        let mut offset = line_start;
        for c in self.text[line_start..content_end].chars() {
            if remaining_units == 0 {
                break;
            }
            let units = c.len_utf16();
            if units > remaining_units {
                break;
            }
            remaining_units -= units;
            offset += c.len_utf8();
        }
        offset.min(self.text.len())
    }

    /// Converts a byte half-open [`std::ops::Range`] into an LSP
    /// [`tower_lsp::lsp_types::Range`] by mapping both endpoints via
    /// [`LineIndex::position`].
    pub fn range(&self, range: std::ops::Range<usize>) -> tower_lsp::lsp_types::Range {
        tower_lsp::lsp_types::Range {
            start: self.position(range.start),
            end: self.position(range.end),
        }
    }

    /// Finds the 0-based line index containing byte `offset`.
    ///
    /// Binary-searches [`Self::line_starts`] for the greatest start `<=
    /// offset`.
    fn line_of(&self, offset: usize) -> usize {
        match self.line_starts.binary_search(&offset) {
            Ok(i) => i,
            Err(i) => i - 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// One (name, text, offset, expected `(line, character)`) case.
    struct Case {
        name: &'static str,
        text: &'static str,
        offset: usize,
        expected: (u32, u32),
    }

    #[test]
    fn table_driven_positions() {
        let cases = [
            Case {
                name: "ascii start of file",
                text: "let x = 1\nlet y = 2\n",
                offset: 0,
                expected: (0, 0),
            },
            Case {
                name: "ascii mid first line",
                text: "let x = 1\nlet y = 2\n",
                offset: 4,
                expected: (0, 4),
            },
            Case {
                name: "ascii start of second line",
                text: "let x = 1\nlet y = 2\n",
                offset: 10,
                expected: (1, 0),
            },
            Case {
                name: "ascii mid second line",
                text: "let x = 1\nlet y = 2\n",
                offset: 14,
                expected: (1, 4),
            },
            Case {
                name: "multibyte precedes offset — é is 2 UTF-8 bytes, 1 UTF-16 unit",
                // "café" — 'é' is U+00E9: 2 bytes in UTF-8, 1 code unit in UTF-16.
                text: "café = 1",
                // byte offset of the '=' : c(1) a(1) f(1) é(2) space(1) = 6 bytes in.
                offset: 6,
                // UTF-16 chars before '=': c,a,f,é,' ' = 5 code units.
                expected: (0, 5),
            },
            Case {
                name: "emoji precedes offset — U+1F600 is 4 UTF-8 bytes, 2 UTF-16 units (surrogate pair)",
                text: "😀x",
                // offset 4 = start of 'x' (emoji is 4 bytes in UTF-8).
                offset: 4,
                // emoji is 2 UTF-16 code units (surrogate pair).
                expected: (0, 2),
            },
            Case {
                name: "offset inside emoji byte sequence still counts 0 whole chars",
                text: "😀x",
                offset: 2,
                expected: (0, 0),
            },
            Case {
                name: "CRLF: offset right after \\r on line 0 is still line 0",
                text: "abc\r\ndef",
                offset: 4, // the '\r' is at byte 3; offset 4 is right after it, before '\n'
                expected: (0, 4),
            },
            Case {
                name: "CRLF: offset at start of line 1 (after \\n)",
                text: "abc\r\ndef",
                offset: 5,
                expected: (1, 0),
            },
            Case {
                name: "CRLF: offset mid line 1",
                text: "abc\r\ndef",
                offset: 7,
                expected: (1, 2),
            },
            Case {
                name: "EOF zero-width range at very end of file",
                text: "abc",
                offset: 3,
                expected: (0, 3),
            },
            Case {
                name: "EOF zero-width range on empty file",
                text: "",
                offset: 0,
                expected: (0, 0),
            },
            Case {
                name: "offset past end of text clamps to EOF position",
                text: "abc",
                offset: 100,
                expected: (0, 3),
            },
            Case {
                name: "trailing newline: offset at EOF after final \\n is a fresh empty line",
                text: "abc\n",
                offset: 4,
                expected: (1, 0),
            },
            Case {
                name: "multiple blank lines",
                text: "a\n\n\nb",
                offset: 4, // 'b'
                expected: (3, 0),
            },
        ];

        for case in cases {
            let index = LineIndex::new(case.text);
            let pos = index.position(case.offset);
            assert_eq!(
                (pos.line, pos.character),
                case.expected,
                "case {:?} failed: text={:?} offset={}",
                case.name,
                case.text,
                case.offset
            );
        }
    }

    #[test]
    fn offset_round_trips_position_over_ascii_multibyte_and_emoji() {
        let cases: &[(&str, usize)] = &[
            ("let x = 1\nlet y = 2\n", 0),
            ("let x = 1\nlet y = 2\n", 4),
            ("let x = 1\nlet y = 2\n", 10),
            ("let x = 1\nlet y = 2\n", 14),
            ("café = 1", 6),
            ("😀x", 4),
        ];
        for (text, byte_offset) in cases {
            let index = LineIndex::new(text);
            let pos = index.position(*byte_offset);
            assert_eq!(index.offset(pos), *byte_offset, "round-trip failed for text={text:?} offset={byte_offset}");
        }
    }

    #[test]
    fn offset_clamps_character_past_end_of_line() {
        let index = LineIndex::new("abc\ndef");
        // Line 0 is 3 chars long; asking for character 100 clamps to the
        // line's end (byte 3, right before the '\n').
        let offset = index.offset(Position { line: 0, character: 100 });
        assert_eq!(offset, 3);
    }

    #[test]
    fn offset_clamps_line_past_end_of_file() {
        let index = LineIndex::new("abc\ndef");
        let offset = index.offset(Position { line: 100, character: 0 });
        assert_eq!(offset, 4); // start of (only) line 1
    }

    #[test]
    fn offset_at_emoji_surrogate_pair_boundary() {
        // Position (0, 2) is right after the emoji (2 UTF-16 units); must
        // land on the byte offset where 'x' starts (byte 4).
        let index = LineIndex::new("😀x");
        assert_eq!(index.offset(Position { line: 0, character: 2 }), 4);
    }

    #[test]
    fn range_maps_both_endpoints() {
        let index = LineIndex::new("let x = 1\nlet y = 2\n");
        let r = index.range(4..9);
        assert_eq!(r.start, Position { line: 0, character: 4 });
        assert_eq!(r.end, Position { line: 0, character: 9 });
    }

    #[test]
    fn range_spanning_lines() {
        let index = LineIndex::new("aaa\nbbb\nccc");
        let r = index.range(2..9);
        assert_eq!(r.start, Position { line: 0, character: 2 });
        assert_eq!(r.end, Position { line: 2, character: 1 });
    }
}
