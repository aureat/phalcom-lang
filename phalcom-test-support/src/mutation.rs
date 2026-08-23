use crate::MarkedSource;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Mutation {
    pub anchor: String,
    pub old: String,
    pub new: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MutationError {
    OldTextMismatch { anchor: String, expected: String, actual_prefix: String },
}

impl Mutation {
    pub fn apply(&self, source: &MarkedSource) -> Result<String, MutationError> {
        let position = source.position(&self.anchor);
        let tail = &source.text[position.byte_offset..];
        if !tail.starts_with(&self.old) {
            return Err(MutationError::OldTextMismatch {
                anchor: self.anchor.clone(),
                expected: self.old.clone(),
                actual_prefix: tail.chars().take(self.old.chars().count()).collect(),
            });
        }

        let mut result = source.text.clone();
        let end = position.byte_offset + self.old.len();
        result.replace_range(position.byte_offset..end, &self.new);
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mutation_is_guarded_by_expected_old_text() {
        let source = MarkedSource::parse("const x = /*@value*/42
");
        let mutation = Mutation { anchor: "value".into(), old: "42".into(), new: ""wrong"".into() };
        assert_eq!(mutation.apply(&source).unwrap(), "const x = "wrong"
");
    }
}
