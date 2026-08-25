//! Shared semantic presentation policy for ordinary IDE surfaces.

/// Formats an inferred type hint using familiar Phalcom syntax.
pub fn inlay_type_label(type_text: &str, return_hint: bool) -> String {
    if return_hint { format!(" -> {type_text}") } else { format!(": {type_text}") }
}

/// Formats contextual advisory evidence without exposing analyzer taxonomy.
pub fn advisory_tooltip(type_text: &str, subject: &str) -> String {
    format!("Inferred {subject}: `{type_text}`\n\nDerived from current semantic evidence.")
}

/// Formats a compiler-owned formal hint tooltip.
pub fn formal_tooltip(type_text: &str) -> String {
    format!("Formal type: `{type_text}`")
}

/// Formats contextual advisory hover information.
pub fn advisory_hover(label: &str, type_text: &str) -> String {
    format!("**{label}:** `{type_text}`\n\nDerived from current value flow.")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ordinary_type_labels_have_no_epistemic_decoration() {
        let values = [
            inlay_type_label("String", false),
            inlay_type_label("Result", true),
            advisory_hover("Observed type", "String"),
        ];
        let joined = values.join("\n");
        assert!(!joined.contains('≈'));
        assert!(!joined.contains("Confidence:"));
        assert!(!joined.contains("Observed type: ≈"));
    }
}
