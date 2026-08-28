from pathlib import Path

# The formal checker records a local binding's enclosing declaration statement
# range while the source index deliberately stores the binding-name range.
# Attachment may use range containment plus the name to connect those already
# canonical identities, but it must still fail closed if that source attachment
# is ambiguous.
path = Path("phalcom-semantic/src/source_index/mod.rs")
text = path.read_text()
old = '''                .filter(|binding| {
                    binding.name.as_ref() == state.name
                        && binding.declaration_range == state.range
                        && binding.declaration_site.owner == crate::identity::SourceOwner::Callable(callable.clone())
                })
'''
new = '''                .filter(|binding| {
                    binding.name.as_ref() == state.name
                        && state.range.start <= binding.declaration_range.start
                        && binding.declaration_range.end <= state.range.end
                        && binding.declaration_site.owner == crate::identity::SourceOwner::Callable(callable.clone())
                })
'''
if old in text:
    text = text.replace(old, new, 1)
elif new not in text:
    raise SystemExit("local formal attachment shape changed")
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
