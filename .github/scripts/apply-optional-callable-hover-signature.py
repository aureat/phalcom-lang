from pathlib import Path

path = Path("phalcom-lsp/src/backend.rs")
text = path.read_text()
old = '''    fn compiler_callable_hover(&self, request: &RequestContext, callable: &phalcom_semantic::identity::CallableId) -> Option<CompilerCallableHover> {
        let compiler = request.compiler.as_deref()?;
        let signature = compiler.callable_signatures.get(callable)?;
        let source = compiler.source_index().callable_source(callable);
'''
new = '''    fn compiler_callable_hover(&self, request: &RequestContext, callable: &phalcom_semantic::identity::CallableId) -> Option<CompilerCallableHover> {
        let compiler = request.compiler.as_deref()?;
        let signature = compiler.callable_signatures.get(callable);
        let source = compiler.source_index().callable_source(callable);
        if signature.is_none() && source.is_none() {
            return None;
        }
'''
if old not in text:
    raise SystemExit("compiler_callable_hover header not found")
text = text.replace(old, new, 1)
old = '''        let presenter = phalcom_semantic::TypePresenter::new(&compiler.store);
        let formal = match &signature.return_type {
            phalcom_semantic::types::TypeTerm::Canonical(ty) => FormalPresentation::Known(presenter.present_type(*ty)),
            phalcom_semantic::types::TypeTerm::SelfType(_) | phalcom_semantic::types::TypeTerm::Infer(_) => FormalPresentation::Unknown,
        };
'''
new = '''        let presenter = phalcom_semantic::TypePresenter::new(&compiler.store);
        let formal = signature.map_or(FormalPresentation::Unknown, |signature| match &signature.return_type {
            phalcom_semantic::types::TypeTerm::Canonical(ty) => FormalPresentation::Known(presenter.present_type(*ty)),
            phalcom_semantic::types::TypeTerm::SelfType(_) | phalcom_semantic::types::TypeTerm::Infer(_) => FormalPresentation::Unknown,
        });
'''
if old not in text:
    raise SystemExit("formal return block not found")
text = text.replace(old, new, 1)
path.write_text(text)
print("optional callable hover signature applied")
