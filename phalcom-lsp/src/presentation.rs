//! Shared semantic presentation policy for ordinary IDE surfaces.

/// Formats an inferred type hint using familiar Phalcom syntax.
pub fn inlay_type_label(type_text: &str, return_hint: bool) -> String {
    if return_hint { format!(" -> {type_text}") } else { format!(": {type_text}") }
}

/// Formats contextual advisory evidence without exposing analyzer taxonomy.
pub fn advisory_tooltip(type_text: &str, subject: &str) -> String {
    let evidence = match subject {
        "return value" => "Inferred from call sites.",
        _ => "Inferred from local flow.",
    };
    format!("`{type_text}`\n\n{evidence}")
}

/// Formats a compiler-owned formal hint tooltip.
pub fn formal_tooltip(type_text: &str) -> String {
    format!("`{type_text}`")
}

/// Formats contextual advisory hover information.
pub fn advisory_hover(_label: &str, type_text: &str) -> String {
    format!("`{type_text}`\n\nInferred from local flow.")
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
        assert!(!joined.contains(char::from_u32(0x2248).expect("approximation sign")));
        assert!(!joined.contains(["Confidence", ":"].concat().as_str()));
        assert!(!joined.contains("Observed type"));
        assert!(joined.contains("Inferred from local flow."));
    }
}
