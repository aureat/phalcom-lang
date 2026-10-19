//! Allocation-free JSON syntax and content accounting.

use super::{MetadataPreflightLimits, MetadataSectionError};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct MetadataPreflightCounts {
    pub tokens: u32,
    pub containers: u32,
    pub nesting: u16,
}

struct Scanner<'a> {
    bytes: &'a [u8],
    cursor: usize,
    limits: MetadataPreflightLimits,
    counts: MetadataPreflightCounts,
}

impl Scanner<'_> {
    fn invalid<T>(&self, reason: &'static str) -> Result<T, MetadataSectionError> {
        Err(MetadataSectionError::InvalidEncodedBundle(reason.into()))
    }

    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.cursor).copied()
    }

    fn bump(&mut self) -> Option<u8> {
        let value = self.peek()?;
        self.cursor += 1;
        Some(value)
    }

    fn whitespace(&mut self) {
        while matches!(self.peek(), Some(b' ' | b'\n' | b'\r' | b'\t')) {
            self.cursor += 1;
        }
    }

    fn token(&mut self) -> Result<(), MetadataSectionError> {
        let actual = self.counts.tokens.checked_add(1).ok_or(MetadataSectionError::TokenBudgetExceeded {
            limit: self.limits.max_token_count,
            actual: u32::MAX,
        })?;
        if actual > self.limits.max_token_count {
            return Err(MetadataSectionError::TokenBudgetExceeded {
                limit: self.limits.max_token_count,
                actual,
            });
        }
        self.counts.tokens = actual;
        Ok(())
    }

    fn container(&mut self, depth: u16) -> Result<(), MetadataSectionError> {
        let actual = self.counts.containers.checked_add(1).ok_or(MetadataSectionError::ContainerBudgetExceeded {
            limit: self.limits.max_container_count,
            actual: u32::MAX,
        })?;
        if actual > self.limits.max_container_count {
            return Err(MetadataSectionError::ContainerBudgetExceeded {
                limit: self.limits.max_container_count,
                actual,
            });
        }
        if depth > self.limits.max_nesting_depth {
            return Err(MetadataSectionError::NestingBudgetExceeded {
                limit: self.limits.max_nesting_depth,
                actual: depth,
            });
        }
        self.counts.containers = actual;
        self.counts.nesting = self.counts.nesting.max(depth);
        Ok(())
    }

    fn parse_value(&mut self, parent_depth: u16) -> Result<(), MetadataSectionError> {
        self.whitespace();
        self.token()?;
        match self.peek() {
            Some(b'{') => self.parse_object(parent_depth),
            Some(b'[') => self.parse_array(parent_depth),
            Some(b'"') => self.parse_string().map(|_| ()),
            Some(b't') => self.parse_literal(b"true"),
            Some(b'f') => self.parse_literal(b"false"),
            Some(b'n') => self.parse_literal(b"null"),
            Some(b'-' | b'0'..=b'9') => self.parse_number(),
            Some(_) => self.invalid("invalid JSON value"),
            None => self.invalid("unexpected end of JSON value"),
        }
    }

    fn enter_container(&mut self, parent_depth: u16) -> Result<u16, MetadataSectionError> {
        let depth = parent_depth.checked_add(1).ok_or(MetadataSectionError::NestingBudgetExceeded {
            limit: self.limits.max_nesting_depth,
            actual: u16::MAX,
        })?;
        self.container(depth)?;
        Ok(depth)
    }

    fn parse_object(&mut self, parent_depth: u16) -> Result<(), MetadataSectionError> {
        self.cursor += 1; // '{'
        let depth = self.enter_container(parent_depth)?;
        self.whitespace();
        if self.peek() == Some(b'}') {
            self.cursor += 1;
            return Ok(());
        }
        loop {
            self.whitespace();
            if self.peek() != Some(b'"') {
                return self.invalid("JSON object key must be a string");
            }
            self.token()?; // Object keys are tokens in the metadata envelope.
            self.parse_string()?;
            self.whitespace();
            if self.bump() != Some(b':') {
                return self.invalid("JSON object key is missing ':'");
            }
            self.parse_value(depth)?;
            self.whitespace();
            match self.bump() {
                Some(b',') => {}
                Some(b'}') => return Ok(()),
                _ => return self.invalid("JSON object is missing ',' or '}'"),
            }
        }
    }

    fn parse_array(&mut self, parent_depth: u16) -> Result<(), MetadataSectionError> {
        self.cursor += 1; // '['
        let depth = self.enter_container(parent_depth)?;
        self.whitespace();
        if self.peek() == Some(b']') {
            self.cursor += 1;
            return Ok(());
        }
        loop {
            self.parse_value(depth)?;
            self.whitespace();
            match self.bump() {
                Some(b',') => {}
                Some(b']') => return Ok(()),
                _ => return self.invalid("JSON array is missing ',' or ']'"),
            }
        }
    }

    fn parse_string(&mut self) -> Result<(), MetadataSectionError> {
        if self.bump() != Some(b'"') {
            return self.invalid("expected JSON string");
        }
        let content_start = self.cursor;
        loop {
            match self.bump() {
                Some(b'"') => {
                    // Validate raw UTF-8 without creating a String. Escape
                    // bytes are ASCII and are checked as the scanner advances.
                    let content_end = self.cursor - 1;
                    if std::str::from_utf8(&self.bytes[content_start..content_end]).is_err() {
                        return self.invalid("JSON string is not UTF-8");
                    }
                    return Ok(());
                }
                Some(0x00..=0x1f) => return self.invalid("unescaped control byte in JSON string"),
                Some(b'\\') => self.parse_escape()?,
                Some(_) => {}
                None => return self.invalid("unterminated JSON string"),
            }
        }
    }

    fn parse_escape(&mut self) -> Result<(), MetadataSectionError> {
        match self.bump() {
            Some(b'"' | b'\\' | b'/' | b'b' | b'f' | b'n' | b'r' | b't') => Ok(()),
            Some(b'u') => {
                let first = self.parse_hex_quad()?;
                match first {
                    0xd800..=0xdbff => {
                        if self.bump() != Some(b'\\') || self.bump() != Some(b'u') {
                            return self.invalid("high surrogate is missing its low surrogate");
                        }
                        if !matches!(self.parse_hex_quad()?, 0xdc00..=0xdfff) {
                            return self.invalid("invalid low surrogate in JSON string");
                        }
                        Ok(())
                    }
                    0xdc00..=0xdfff => self.invalid("unpaired low surrogate in JSON string"),
                    _ => Ok(()),
                }
            }
            _ => self.invalid("invalid JSON string escape"),
        }
    }

    fn parse_hex_quad(&mut self) -> Result<u16, MetadataSectionError> {
        let mut value = 0_u16;
        for _ in 0..4 {
            let digit = match self.bump() {
                Some(byte @ b'0'..=b'9') => u16::from(byte - b'0'),
                Some(byte @ b'a'..=b'f') => u16::from(byte - b'a' + 10),
                Some(byte @ b'A'..=b'F') => u16::from(byte - b'A' + 10),
                _ => return self.invalid("invalid hexadecimal JSON escape"),
            };
            value = (value << 4) | digit;
        }
        Ok(value)
    }

    fn parse_literal(&mut self, literal: &[u8]) -> Result<(), MetadataSectionError> {
        let end = self
            .cursor
            .checked_add(literal.len())
            .ok_or_else(|| MetadataSectionError::InvalidEncodedBundle("JSON literal offset overflow".into()))?;
        if self.bytes.get(self.cursor..end) != Some(literal) {
            return self.invalid("invalid JSON literal");
        }
        self.cursor = end;
        Ok(())
    }

    fn parse_number(&mut self) -> Result<(), MetadataSectionError> {
        if self.peek() == Some(b'-') {
            self.cursor += 1;
        }
        match self.bump() {
            Some(b'0') => {
                if matches!(self.peek(), Some(b'0'..=b'9')) {
                    return self.invalid("leading zero in JSON number");
                }
            }
            Some(b'1'..=b'9') => {
                while matches!(self.peek(), Some(b'0'..=b'9')) {
                    self.cursor += 1;
                }
            }
            _ => return self.invalid("invalid JSON number integer part"),
        }
        if self.peek() == Some(b'.') {
            self.cursor += 1;
            if !matches!(self.bump(), Some(b'0'..=b'9')) {
                return self.invalid("invalid JSON number fraction");
            }
            while matches!(self.peek(), Some(b'0'..=b'9')) {
                self.cursor += 1;
            }
        }
        if matches!(self.peek(), Some(b'e' | b'E')) {
            self.cursor += 1;
            if matches!(self.peek(), Some(b'+' | b'-')) {
                self.cursor += 1;
            }
            if !matches!(self.bump(), Some(b'0'..=b'9')) {
                return self.invalid("invalid JSON number exponent");
            }
            while matches!(self.peek(), Some(b'0'..=b'9')) {
                self.cursor += 1;
            }
        }
        Ok(())
    }
}

/// Validates JSON syntax and actual content counts before decoding allocated
/// tables. It does not decode strings, build a serde value tree, or allocate
/// scanner scratch; counts include object keys and all values.
pub fn inspect_metadata_json(bytes: &[u8], limits: &MetadataPreflightLimits) -> Result<MetadataPreflightCounts, MetadataSectionError> {
    if bytes.len() > limits.max_metadata_bytes {
        return Err(MetadataSectionError::ByteBudgetExceeded {
            limit: limits.max_metadata_bytes,
            actual: bytes.len(),
        });
    }
    let mut scanner = Scanner {
        bytes,
        cursor: 0,
        limits: *limits,
        counts: MetadataPreflightCounts::default(),
    };
    scanner.whitespace();
    scanner.parse_value(0)?;
    scanner.whitespace();
    if scanner.cursor != bytes.len() {
        return Err(MetadataSectionError::InvalidEncodedBundle("trailing JSON content".into()));
    }
    Ok(scanner.counts)
}
