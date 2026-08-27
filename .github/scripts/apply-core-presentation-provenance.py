from pathlib import Path


def replace_once(path: str, old: str, new: str) -> None:
    p = Path(path)
    text = p.read_text()
    if old not in text:
        raise SystemExit(f"missing patch anchor in {path}: {old[:180]!r}")
    p.write_text(text.replace(old, new, 1))


presentation = "phalcom-semantic/src/core_surface/presentation.rs"
replace_once(
    presentation,
    "use phalcom_native_surface::NativeSurfaceId;\n",
    "use phalcom_native_surface::NativeSurfaceId;\nuse std::sync::Arc;\n",
)
with Path(presentation).open("a") as f:
    f.write(
        r'''

/// Renders the stable read-only source document used to present canonical
/// builtin declaration provenance. This text is a presentation product only:
/// it is never linked, type checked, or executed.
pub fn render_canonical_core_source() -> Arc<str> {
    let mut bindings = phalcom_native_meta::universe::UNIVERSE_BINDINGS.iter().collect::<Vec<_>>();
    bindings.sort_by(|left, right| left.name.cmp(right.name));

    let mut out = String::from("// Generated Canonical Core Surface — Read Only\n");
    out.push_str("// Semantic identities and runtime behavior remain compiler-owned.\n\n");
    for binding in bindings {
        out.push_str("class ");
        out.push_str(binding.name);
        out.push_str(" {}\n\n");
    }
    Arc::from(out)
}
'''
    )

snapshot = "phalcom-semantic/src/snapshot.rs"
replace_once(
    snapshot,
    "    pub sources: Arc<BTreeMap<ModuleId, Arc<ParsedModuleUnit>>>,\n",
    "    pub sources: Arc<BTreeMap<ModuleId, Arc<ParsedModuleUnit>>>,\n    /// Read-only compiler-generated source documents used solely for semantic\n    /// provenance and editor presentation. They are never analysis inputs.\n    pub presentation_sources: Arc<BTreeMap<ModuleId, Arc<str>>>,\n",
)
replace_once(
    snapshot,
    "            sources,\n            surfaces,\n",
    "            sources,\n            presentation_sources: Arc::new(BTreeMap::new()),\n            surfaces,\n",
)
replace_once(
    snapshot,
    "            sources,\n            surfaces,\n",
    "            sources,\n            presentation_sources: Arc::new(BTreeMap::new()),\n            surfaces,\n",
)
replace_once(
    snapshot,
    "    /// Attaches one immutable compiler-owned source semantic index.\n    pub fn with_source_index",
    "    /// Attaches read-only compiler presentation sources. These documents\n    /// provide source coordinates for canonical declaration provenance but do\n    /// not participate in linking or semantic analysis.\n    pub fn with_presentation_sources(mut self, sources: Arc<BTreeMap<ModuleId, Arc<str>>>) -> Self {\n        self.presentation_sources = sources;\n        self\n    }\n\n    /// Returns exact compiler-owned presentation text for one virtual source.\n    pub fn presentation_source(&self, module: &ModuleId) -> Option<&str> {\n        self.presentation_sources.get(module).map(AsRef::as_ref)\n    }\n\n    /// Attaches one immutable compiler-owned source semantic index.\n    pub fn with_source_index",
)

session = "phalcom-semantic/src/session.rs"
replace_once(
    session,
    "use crate::diagnostic::{DiagnosticCode, SemanticDiagnostic};\n",
    "use crate::diagnostic::{DiagnosticCode, SemanticDiagnostic};\nuse crate::core_surface::render_canonical_core_source;\n",
)
replace_once(
    session,
    "        let mut source_index = build_source_semantic_index(&input.sources, &callable_analyses, &resolved_imports_map, input.linked.as_ref(), &resolver);\n",
    "        let (mut source_index, presentation_sources) =\n            build_source_semantic_index(&input.sources, &callable_analyses, &resolved_imports_map, input.linked.as_ref(), &resolver);\n",
)
replace_once(
    session,
    "        snapshot_obj = snapshot_obj.with_source_index(Arc::new(source_index));\n",
    "        snapshot_obj = snapshot_obj.with_presentation_sources(Arc::new(presentation_sources));\n        snapshot_obj = snapshot_obj.with_source_index(Arc::new(source_index));\n",
)
replace_once(
    session,
    "    type_resolver: &dyn TypeResolver,\n) -> SourceSemanticIndex {\n",
    "    type_resolver: &dyn TypeResolver,\n) -> (SourceSemanticIndex, BTreeMap<ModuleId, Arc<str>>) {\n",
)
replace_once(
    session,
    "    for analysis in callable_analyses.values() {\n        let module = &analysis.callable.owner.module;\n        if index.module(module).is_some() {\n            let _ = index.attach_formal_analysis(module, analysis);\n        }\n    }\n    index\n}\n",
    "    for analysis in callable_analyses.values() {\n        let module = &analysis.callable.owner.module;\n        if index.module(module).is_some() {\n            let _ = index.attach_formal_analysis(module, analysis);\n        }\n    }\n\n    let mut presentation_sources = BTreeMap::new();\n    let core = ModuleId::core();\n    if !sources.contains_key(&core) {\n        let text = render_canonical_core_source();\n        let parsed = phalcom_ast::parse(&text, 0);\n        assert!(\n            parsed.errors.is_empty(),\n            \"compiler-owned canonical core presentation must parse: {:#?}\",\n            parsed.errors\n        );\n        let structure = build_source_scope_index(core.clone(), &parsed.program, &SourceIndexContext::default());\n        let mut core_index = SourceSemanticIndex::from_scope_indices(BTreeMap::from([(core.clone(), structure)]));\n        let shard = core_index.modules.remove(&core).expect(\"canonical core presentation shard\");\n        index.modules.insert(core.clone(), shard);\n        index.rebuild_target_occurrences();\n        presentation_sources.insert(core, text);\n    }\n\n    (index, presentation_sources)\n}\n",
)
