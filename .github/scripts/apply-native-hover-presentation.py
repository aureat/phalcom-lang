from pathlib import Path

editor = Path("phalcom-semantic/src/editor.rs")
text = editor.read_text()
anchor = '''/// Read-only editor query facade over one immutable semantic snapshot.\n#[derive(Clone, Copy, Debug)]\npub struct EditorSemanticQuery<'a> {\n'''
insert = '''/// Compiler-owned presentation metadata for a canonical native callable.\n///\n/// This deliberately projects only protocol-neutral documentation metadata;\n/// clients do not need direct access to the native surface catalog.\n#[derive(Clone, Copy, Debug, Eq, PartialEq)]\npub struct NativeCallablePresentation {\n    pub documentation: Option<&'static str>,\n    pub conceptual: Option<&'static str>,\n}\n\n/// Read-only editor query facade over one immutable semantic snapshot.\n#[derive(Clone, Copy, Debug)]\npub struct EditorSemanticQuery<'a> {\n'''
if anchor not in text:
    raise SystemExit("editor struct anchor not found")
text = text.replace(anchor, insert, 1)
method_anchor = '''    /// Returns exact canonical target at a source position.\n    pub fn target_at(&self, module: &ModuleId, offset: usize) -> Option<SemanticTargetId> {\n'''
method = '''    /// Returns compiler-owned native presentation metadata for one callable.\n    pub fn native_callable_presentation(&self, callable: &CallableId) -> Option<NativeCallablePresentation> {\n        let signature = self.snapshot.callable_signatures.get(callable)?;\n        let native_id = signature.native_id?;\n        let record = phalcom_native_surface::NATIVE_SURFACE_CATALOG.find(native_id.0)?;\n        Some(NativeCallablePresentation {\n            documentation: record.docs(),\n            conceptual: record.conceptual(),\n        })\n    }\n\n    /// Returns exact canonical target at a source position.\n    pub fn target_at(&self, module: &ModuleId, offset: usize) -> Option<SemanticTargetId> {\n'''
if method_anchor not in text:
    raise SystemExit("editor target anchor not found")
text = text.replace(method_anchor, method, 1)
editor.write_text(text)

lib = Path("phalcom-semantic/src/lib.rs")
text = lib.read_text()
old = '''    AccessContext, EditorMember, EditorMemberTarget, EditorSemanticQuery, PartialCallPattern, ReceiverAlternative, ReceiverMode, ResolvedReceiver,\n    VisibleSymbol,\n'''
new = '''    AccessContext, EditorMember, EditorMemberTarget, EditorSemanticQuery, NativeCallablePresentation, PartialCallPattern, ReceiverAlternative, ReceiverMode,\n    ResolvedReceiver, VisibleSymbol,\n'''
if old not in text:
    raise SystemExit("lib editor export not found")
lib.write_text(text.replace(old, new, 1))

hover = Path("phalcom-lsp/src/hover.rs")
text = hover.read_text()
anchor = '''/// Renders one lexical binding or parameter with formal type knowledge or advisory value.\n/// One place a selector is declared/known, as [`render_selector_hover`]\n'''
helper = '''/// Renders implementation/documentation details supplied by the compiler for\n/// one native callable. The semantic layer chooses the metadata; the LSP owns\n/// the Markdown presentation.\npub fn render_native_callable_details(documentation: Option<&str>) -> String {\n    let mut sections = vec!["native primitive".to_string()];\n    if let Some(documentation) = documentation.filter(|text| !text.trim().is_empty()) {\n        sections.push(documentation.to_string());\n    }\n    sections.join("\\n\\n---\\n\\n")\n}\n\n/// Renders one lexical binding or parameter with formal type knowledge or advisory value.\n/// One place a selector is declared/known, as [`render_selector_hover`]\n'''
if anchor not in text:
    raise SystemExit("hover helper anchor not found")
hover.write_text(text.replace(anchor, helper, 1))

backend = Path("phalcom-lsp/src/backend.rs")
text = backend.read_text()
old = '''    FormalPresentation,\n    Option<phalcom_semantic::advisory::AdvisoryFact>,\n);\n'''
new = '''    FormalPresentation,\n    Option<phalcom_semantic::advisory::AdvisoryFact>,\n    Option<phalcom_semantic::NativeCallablePresentation>,\n);\n'''
if old not in text:
    raise SystemExit("CompilerCallableHover tuple not found")
text = text.replace(old, new, 1)
old = '''        Some((\n            callable.selector.encode(),\n            SelectorSite {\n                owner: callable.owner.clone(),\n                receiver: None,\n                kind,\n            },\n            phaldoc,\n            formal,\n            compiler.advisory_callable(callable).map(|summary| summary.return_fact.clone()),\n        ))\n'''
new = '''        Some((\n            callable.selector.encode(),\n            SelectorSite {\n                owner: callable.owner.clone(),\n                receiver: None,\n                kind,\n            },\n            phaldoc,\n            formal,\n            compiler.advisory_callable(callable).map(|summary| summary.return_fact.clone()),\n            compiler.editor().native_callable_presentation(callable),\n        ))\n'''
if old not in text:
    raise SystemExit("compiler callable hover return not found")
text = text.replace(old, new, 1)
old = '''                phalcom_semantic::SemanticTargetId::Callable(callable)\n                    if let Some((selector, site, phaldoc, formal, advisory)) = self.compiler_callable_hover(request, &callable)\n                        && let Some(contents) =\n                            hover::render_selector_hover_with_formal_value(&selector, &[site], phaldoc.as_ref(), Some(&formal), advisory.as_ref()) =>\n                {\n                    return Some(Hover {\n                        contents: markdown_contents(contents),\n                        range: Some(span),\n                    });\n                }\n'''
new = '''                phalcom_semantic::SemanticTargetId::Callable(callable)\n                    if let Some((selector, site, phaldoc, formal, advisory, native)) = self.compiler_callable_hover(request, &callable)\n                        && let Some(mut contents) =\n                            hover::render_selector_hover_with_formal_value(&selector, &[site], phaldoc.as_ref(), Some(&formal), advisory.as_ref()) =>\n                {\n                    if let Some(native) = native {\n                        contents.push_str("\\n\\n---\\n\\n");\n                        contents.push_str(&hover::render_native_callable_details(native.documentation));\n                    }\n                    return Some(Hover {\n                        contents: markdown_contents(contents),\n                        range: Some(span),\n                    });\n                }\n'''
if old not in text:
    raise SystemExit("callable hover render branch not found")
backend.write_text(text.replace(old, new, 1))

probe = Path("phalcom-semantic/tests/constructor_factory_probe.rs")
text = probe.read_text()
marker = "fn native_callable_presentation_is_compiler_owned()"
if marker not in text:
    text += r'''

#[test]
fn native_callable_presentation_is_compiler_owned() {
    let module = ModuleId::core();
    let source: Arc<str> = Arc::from("let x = true.ifTrue || { 1 };\n");
    let parsed = phalcom_ast::parse(&source, 0);
    assert!(parsed.errors.is_empty(), "parse errors: {:#?}", parsed.errors);
    let analysis = analyze_single_module(module.clone(), source.clone(), Arc::new(parsed.program));
    let snapshot = analysis.snapshot;
    let offset = source.find("ifTrue").expect("ifTrue call");
    let target = snapshot.editor().target_at(&module, offset).expect("native callable target");
    let SemanticTargetId::Callable(callable) = target else {
        panic!("expected callable target, got {target:#?}");
    };
    let native = snapshot
        .editor()
        .native_callable_presentation(&callable)
        .expect("native presentation metadata");
    assert_eq!(native.documentation, Some("Executes block if receiver is true."));
}
'''
probe.write_text(text)
print("native hover presentation applied")
