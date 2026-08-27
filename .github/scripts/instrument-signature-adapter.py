from pathlib import Path

p = Path("phalcom-lsp/src/backend.rs")
text = p.read_text()
text = text.replace(
    "        let site = signature_help::call_site_at(&request.document.text, offset)?;\n",
    "        let site = signature_help::call_site_at(&request.document.text, offset)?;\n        eprintln!(\"SIGHELP site={site:?} source_match={:?} module={:?} compiler={}\", request.source_match, request.compiler_module(), request.compiler.is_some());\n",
    1,
)
text = text.replace(
    "    if let Some(callable) = exact {\n",
    "    eprintln!(\"SIGHELP exact={exact:?}\");\n    if let Some(callable) = exact {\n",
    1,
)
text = text.replace(
    "    let receiver_range = site.receiver_range?;\n    let receiver = compiler.editor().resolve_receiver_at(module, receiver_range)?;\n",
    "    let receiver_range = match site.receiver_range { Some(range) => range, None => { eprintln!(\"SIGHELP no receiver range\"); return None; } };\n    let receiver = match compiler.editor().resolve_receiver_at(module, receiver_range) { Some(receiver) => receiver, None => { eprintln!(\"SIGHELP receiver unresolved range={receiver_range:?}\"); return None; } };\n    eprintln!(\"SIGHELP receiver={receiver:?}\");\n",
    1,
)
text = text.replace(
    "    let candidates = compiler.editor().callable_candidates(&receiver, &pattern, &access);\n",
    "    let candidates = compiler.editor().callable_candidates(&receiver, &pattern, &access);\n    eprintln!(\"SIGHELP selector={} pattern={pattern:?} candidates={candidates:?}\", site.selector);\n",
    1,
)
p.write_text(text)
print("signature adapter instrumentation applied")
