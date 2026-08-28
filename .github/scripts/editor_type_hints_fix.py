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
