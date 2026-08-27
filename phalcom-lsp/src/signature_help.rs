//! Read-only signature-help recovery and presentation.

use phalcom_ast::ast::RestMode;
use phalcom_common::range::SourceRange;
use tower_lsp::lsp_types::{ParameterInformation, ParameterLabel, SignatureHelp, SignatureInformation};

/// Syntax-only call context recovered from the pinned editor text.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CallSite {
    /// Callee spelling without receiver.
    pub name: String,
    /// Byte range of callee spelling.
    pub name_range: SourceRange,
    /// Receiver expression range for a dotted send.
    pub receiver_range: Option<SourceRange>,
    /// Canonical selector candidate from written argument labels/arity.
    pub selector: String,
    /// Zero-based argument under the cursor.
    pub active_parameter: usize,
}

/// Recovers the innermost call whose argument list contains `offset`.
pub fn call_site_at(text: &str, offset: usize) -> Option<CallSite> {
    let offset = offset.min(text.len());
    let open = find_call_open(text, offset)?;
    let prefix_end = trim_end(text, open);
    let name_end = prefix_end;
    let name_start = scan_identifier_start(text, name_end)?;
    let name = text.get(name_start..name_end)?.to_string();

    let before_name = trim_start(text, name_start);
    let receiver_range = if before_name > 0 && text.as_bytes().get(before_name - 1) == Some(&b'.') {
        let receiver_end = trim_end(text, before_name - 1);
        let receiver_start = scan_receiver_start(text, receiver_end);
        Some(SourceRange {
            start: receiver_start,
            end: receiver_end,
        })
    } else {
        None
    };

    let args = text.get(open + 1..offset).unwrap_or_default();
    let segments = split_top_level(args);
    let active_parameter = top_level_comma_count(args);
    let slots = segments.iter().filter_map(|segment| selector_slot(segment)).collect::<Vec<_>>();

    Some(CallSite {
        name: name.clone(),
        name_range: SourceRange {
            start: name_start,
            end: name_end,
        },
        receiver_range,
        selector: format!("{}({})", name, slots.join(",")),
        active_parameter,
    })
}

/// Renders one canonical compiler callable into LSP signature help.
///
/// This adapter owns no semantic state and performs no resolution. It only
/// projects the pinned compiler signature, using advisory shapes when the
/// formal term is not yet displayable.
pub fn render_signature_help(
    signature: &phalcom_semantic::CallableSemanticSignature,
    store: &phalcom_semantic::TypeStore,
    advisory: Option<&phalcom_semantic::AdvisoryCallableSummary>,
    active_parameter: usize,
) -> SignatureHelp {
    let presenter = phalcom_semantic::TypePresenter::new(store);
    let mut parameters = Vec::with_capacity(signature.parameters.len());
    for (index, parameter) in signature.parameters.iter().enumerate() {
        let type_text = compiler_term_text(&parameter.ty, &presenter)
            .or_else(|| {
                advisory.and_then(|summary| {
                    summary
                        .parameters
                        .iter()
                        .find(|(slot, _)| slot.index as usize == index)
                        .map(|(_, fact)| phalcom_semantic::AdvisoryPresenter::present_shape(&fact.shape))
                })
            })
            .unwrap_or_else(|| "Unknown".to_string());
        let rest = match parameter.rest {
            RestMode::None => "",
            RestMode::Positional => "*",
            RestMode::Labeled => "**",
            RestMode::Complete => "*",
        };
        let label = parameter
            .external_label
            .as_deref()
            .map(|label| format!("{label}: {}{rest}: {type_text}", parameter.local_name))
            .unwrap_or_else(|| format!("{}{rest}: {type_text}", parameter.local_name));
        parameters.push(ParameterInformation {
            label: ParameterLabel::Simple(label),
            documentation: None,
        });
    }

    let return_text = compiler_term_text(&signature.return_type, &presenter)
        .or_else(|| advisory.map(|summary| phalcom_semantic::AdvisoryPresenter::present_shape(&summary.return_fact.shape)))
        .unwrap_or_else(|| "Unknown".to_string());
    let active_parameter = (active_parameter < parameters.len()).then_some(active_parameter as u32);
    SignatureHelp {
        signatures: vec![SignatureInformation {
            label: format!("{} -> {return_text}", signature.selector.encode()),
            documentation: None,
            parameters: Some(parameters),
            active_parameter,
        }],
        active_signature: Some(0),
        active_parameter,
    }
}

fn compiler_term_text(term: &phalcom_semantic::types::TypeTerm, presenter: &phalcom_semantic::TypePresenter<'_>) -> Option<String> {
    match term {
        phalcom_semantic::types::TypeTerm::Canonical(ty) => Some(presenter.present_type(*ty)),
        phalcom_semantic::types::TypeTerm::SelfType(_) | phalcom_semantic::types::TypeTerm::Infer(_) => None,
    }
}

fn find_call_open(text: &str, offset: usize) -> Option<usize> {
    let bytes = text.as_bytes();
    let mut closers = Vec::new();
    for index in (0..offset).rev() {
        match bytes[index] {
            b')' => closers.push(b'('),
            b']' => closers.push(b'['),
            b'}' => closers.push(b'{'),
            b'(' | b'[' | b'{' => {
                if closers.last().copied() == Some(bytes[index]) {
                    closers.pop();
                } else if bytes[index] == b'(' && closers.is_empty() {
                    return Some(index);
                }
            }
            _ => {}
        }
    }
    None
}

fn trim_end(text: &str, mut end: usize) -> usize {
    while end > 0 && text.as_bytes()[end - 1].is_ascii_whitespace() {
        end -= 1;
    }
    end
}

fn trim_start(text: &str, mut start: usize) -> usize {
    while start > 0 && text.as_bytes()[start - 1].is_ascii_whitespace() {
        start -= 1;
    }
    start
}

fn scan_identifier_start(text: &str, end: usize) -> Option<usize> {
    if end == 0 || !is_identifier_byte(text.as_bytes()[end - 1]) {
        return None;
    }
    let mut start = end;
    while start > 0 && is_identifier_byte(text.as_bytes()[start - 1]) {
        start -= 1;
    }
    Some(start)
}

fn scan_receiver_start(text: &str, end: usize) -> usize {
    let bytes = text.as_bytes();
    let mut start = end;
    while start > 0 {
        let byte = bytes[start - 1];
        if is_identifier_byte(byte) || byte == b'.' {
            start -= 1;
        } else {
            break;
        }
    }
    start
}

fn split_top_level(text: &str) -> Vec<&str> {
    let mut segments = Vec::new();
    let mut start = 0;
    let mut depth = 0usize;
    for (index, byte) in text.bytes().enumerate() {
        match byte {
            b'(' | b'[' | b'{' => depth += 1,
            b')' | b']' | b'}' => depth = depth.saturating_sub(1),
            b',' if depth == 0 => {
                if !text[start..index].trim().is_empty() {
                    segments.push(&text[start..index]);
                }
                start = index + 1;
            }
            _ => {}
        }
    }
    if !text[start..].trim().is_empty() {
        segments.push(&text[start..]);
    }
    segments
}

fn top_level_comma_count(text: &str) -> usize {
    let mut count = 0;
    let mut depth = 0usize;
    for byte in text.bytes() {
        match byte {
            b'(' | b'[' | b'{' => depth += 1,
            b')' | b']' | b'}' => depth = depth.saturating_sub(1),
            b',' if depth == 0 => count += 1,
            _ => {}
        }
    }
    count
}

fn selector_slot(segment: &str) -> Option<String> {
    let segment = segment.trim();
    if segment.is_empty() {
        return None;
    }
    let mut depth = 0usize;
    for (index, byte) in segment.bytes().enumerate() {
        match byte {
            b'(' | b'[' | b'{' => depth += 1,
            b')' | b']' | b'}' => depth = depth.saturating_sub(1),
            b':' if depth == 0 => {
                let label = segment[..index].trim();
                if !label.is_empty() && label.bytes().all(is_identifier_byte) {
                    return Some(label.to_string());
                }
                return Some("_".to_string());
            }
            _ => {}
        }
    }
    Some("_".to_string())
}

fn is_identifier_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_'
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn call_site_tracks_receiver_labels_and_active_parameter() {
        let source = "service.compute(1, label: value";
        let site = call_site_at(source, source.len()).expect("call site");
        assert_eq!(site.name, "compute");
        assert_eq!(site.name_range, SourceRange { start: 8, end: 15 });
        assert_eq!(site.receiver_range, Some(SourceRange { start: 0, end: 7 }));
        assert_eq!(site.selector, "compute(_,label)");
        assert_eq!(site.active_parameter, 1);
    }

    #[test]
    fn call_site_handles_incomplete_unqualified_call() {
        let source = "compute(";
        let site = call_site_at(source, source.len()).expect("call site");
        assert_eq!(site.name, "compute");
        assert_eq!(site.selector, "compute()");
        assert_eq!(site.active_parameter, 0);
    }
}
