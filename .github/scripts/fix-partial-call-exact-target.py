from pathlib import Path

p = Path("phalcom-lsp/src/backend.rs")
text = p.read_text()
old = '''    let exact = compiler
        .formal_fact_at(module, site.name_range.start)
        .and_then(|fact| match &fact.fact {
            phalcom_semantic::FormalFactRef::Callable(callable) | phalcom_semantic::FormalFactRef::Expression { callable, .. } => Some(callable.clone()),
            phalcom_semantic::FormalFactRef::Binding { .. } => None,
        })
        .or_else(|| match compiler.editor().target_at(module, site.name_range.start) {
            Some(phalcom_semantic::SemanticTargetId::Callable(callable)) => Some(callable),
            _ => None,
        });
'''
new = '''    let exact = match compiler.editor().target_at(module, site.name_range.start) {
        Some(phalcom_semantic::SemanticTargetId::Callable(callable)) => Some(callable),
        _ => compiler.formal_fact_at(module, site.name_range.start).and_then(|fact| match &fact.fact {
            phalcom_semantic::FormalFactRef::Expression { callable, .. } => Some(callable.clone()),
            phalcom_semantic::FormalFactRef::Callable(_) | phalcom_semantic::FormalFactRef::Binding { .. } => None,
        }),
    };
'''
if old not in text:
    raise SystemExit("partial-call exact target anchor missing")
p.write_text(text.replace(old, new, 1))
print("partial call exact-target fix applied")
