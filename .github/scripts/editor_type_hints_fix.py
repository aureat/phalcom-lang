from pathlib import Path

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

assert ".editor().type_hints(" in text
assert "matches!(hint.formal," not in text
