from pathlib import Path

# Local BindingState source metadata must point at the bound pattern name, not
# at the enclosing let/const statement. This preserves exact formal/source
# attachment instead of widening the historical name+range heuristic.
path = Path("phalcom-semantic/src/checker/statement.rs")
text = path.read_text()
old = '''        Pattern::Name { name, .. } => {
            ctx.declare_binding(BindingSeed {
                parameter: None,
                name: name.clone(),
                range,
'''
new = '''        Pattern::Name { name, range: name_range } => {
            ctx.declare_binding(BindingSeed {
                parameter: None,
                name: name.clone(),
                range: *name_range,
'''
if old in text:
    text = text.replace(old, new, 1)
elif new not in text:
    raise SystemExit("local binding source range shape changed")
path.write_text(text)

# Hover consumes the shared compiler-owned callable presentation instead of
# decoding the removed dispatch-era return_type field itself.
path = Path("phalcom-lsp/src/backend.rs")
text = path.read_text()
old = '''        let presenter = phalcom_semantic::TypePresenter::new(&compiler.store);
        let formal = signature.map_or(FormalPresentation::Unknown, |signature| match &signature.return_type {
            phalcom_semantic::types::TypeTerm::Canonical(ty) => FormalPresentation::Known(presenter.present_type(*ty)),
            phalcom_semantic::types::TypeTerm::SelfType(_) | phalcom_semantic::types::TypeTerm::Infer(_) => FormalPresentation::Unknown,
        });
'''
new = '''        let presenter = phalcom_semantic::TypePresenter::new(&compiler.store);
        let formal = signature.map_or(FormalPresentation::Unknown, |signature| {
            phalcom_semantic::CallablePresentation::from_signature(signature, source, &presenter).return_type
        });
'''
if old in text:
    text = text.replace(old, new, 1)
elif new not in text:
    raise SystemExit("backend callable hover presentation shape changed")
path.write_text(text)

# Signature help renders the canonical formal signature exactly. Advisory
# observations remain separate documentation and never replace Unknown in the
# formal parameter/return signature.
path = Path("phalcom-lsp/src/signature_help.rs")
text = path.read_text()
old_import = '''use tower_lsp::lsp_types::{ParameterInformation, ParameterLabel, SignatureHelp, SignatureInformation};
'''
new_import = '''use tower_lsp::lsp_types::{Documentation, ParameterInformation, ParameterLabel, SignatureHelp, SignatureInformation};
'''
if old_import in text:
    text = text.replace(old_import, new_import, 1)
elif new_import not in text:
    raise SystemExit("signature help import shape changed")
start = text.index('''pub fn render_signature_help(
''')
end = text.index('''fn find_call_open(''', start)
replacement = '''pub fn render_signature_help(
    signature: &phalcom_semantic::CallableSemanticSignature,
    store: &phalcom_semantic::TypeStore,
    advisory: Option<&phalcom_semantic::AdvisoryCallableSummary>,
    active_parameter: usize,
) -> SignatureHelp {
    let presenter = phalcom_semantic::TypePresenter::new(store);
    let presentation = phalcom_semantic::CallablePresentation::from_signature(signature, None, &presenter);
    let mut parameters = Vec::with_capacity(presentation.parameters.len());
    for parameter in presentation.parameters.iter() {
        let type_text = parameter.type_.text();
        let rest = match parameter.rest {
            RestMode::None => "",
            RestMode::Positional => "*",
            RestMode::Labeled => "**",
            RestMode::Complete => "*",
        };
        let label = parameter
            .external_label
            .as_deref()
            .map(|label| format!("{label}: {}{rest}: {type_text}", parameter.name))
            .unwrap_or_else(|| format!("{}{rest}: {type_text}", parameter.name));
        let documentation = matches!(parameter.type_, phalcom_semantic::FormalPresentation::Unknown)
            .then(|| {
                advisory.and_then(|summary| {
                    summary
                        .parameters
                        .iter()
                        .find(|(slot, _)| slot.index == parameter.index)
                        .filter(|(_, fact)| !matches!(fact.shape, phalcom_semantic::ValueShape::Unknown))
                        .map(|(_, fact)| {
                            Documentation::String(format!(
                                "Observed: {}",
                                phalcom_semantic::AdvisoryPresenter::present_shape(&fact.shape)
                            ))
                        })
                })
            })
            .flatten();
        parameters.push(ParameterInformation {
            label: ParameterLabel::Simple(label),
            documentation,
        });
    }

    let return_text = presentation.return_type.text();
    let documentation = matches!(presentation.return_type, phalcom_semantic::FormalPresentation::Unknown)
        .then(|| {
            advisory
                .map(|summary| &summary.return_fact)
                .filter(|fact| !matches!(fact.shape, phalcom_semantic::ValueShape::Unknown))
                .map(|fact| {
                    Documentation::String(format!(
                        "Observed return: {}",
                        phalcom_semantic::AdvisoryPresenter::present_shape(&fact.shape)
                    ))
                })
        })
        .flatten();
    let active_parameter = (active_parameter < parameters.len()).then_some(active_parameter as u32);
    SignatureHelp {
        signatures: vec![SignatureInformation {
            label: format!("{} -> {return_text}", signature.selector.encode()),
            documentation,
            parameters: Some(parameters),
            active_parameter,
        }],
        active_signature: Some(0),
        active_parameter,
    }
}

'''
text = text[:start] + replacement + text[end:]
path.write_text(text)

# Inlay rendering consumes compiler-owned hint eligibility. User display policy
# may choose whether to surface advisory evidence, but it never changes formal
# facts in the semantic result object.
path = Path("phalcom-lsp/src/inlay_hints.rs")
text = path.read_text()
text = text.replace(
    '''    let mut hints = snapshot
        .editor()
        .type_hints(module, SourceRange::new(visible_start, visible_end))
        .into_iter()
''',
    '''    let mut hints = snapshot.editor().type_hints(module, SourceRange::new(visible_start, visible_end)).into_iter()
''',
)
text = text.replace(
    '''                && !matches!(hint.formal, Some(FormalPresentation::Known(_) | FormalPresentation::Dynamic))
''',
    '''                && !matches!(hint.formal.as_ref(), Some(FormalPresentation::Known(_) | FormalPresentation::Dynamic))
''',
)
text = text.replace(
    '''        if hint.formal.is_some() && !matches!(hint.formal, Some(FormalPresentation::Unknown)) {
''',
    '''        if hint.formal.is_some() && !matches!(hint.formal.as_ref(), Some(FormalPresentation::Unknown)) {
''',
)
path.write_text(text)

backend = Path("phalcom-lsp/src/backend.rs").read_text()
signature_help = Path("phalcom-lsp/src/signature_help.rs").read_text()
assert "CallablePresentation::from_signature" in backend
assert "signature.return_type" not in backend
assert "CallablePresentation::from_signature" in signature_help
assert "signature.return_type" not in signature_help
assert "parameter.ty" not in signature_help
assert ".editor().type_hints(" in text
assert "matches!(hint.formal," not in text
