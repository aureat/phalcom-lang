from pathlib import Path

modules = Path("phalcom-modules/src/session.rs")
text = modules.read_text()
replacements = [
    (
        "    linked: Option<Arc<LinkedProgram>>,\n    generation: u64,\n",
        "    linked: Option<Arc<LinkedProgram>>,\n    resolved_imports: BTreeMap<(ModuleId, String), ModuleId>,\n    generation: u64,\n",
    ),
    (
        "            linked: None,\n            generation: 0,\n",
        "            linked: None,\n            resolved_imports: BTreeMap::new(),\n            generation: 0,\n",
    ),
    (
        "    pub fn linked(&self) -> Option<&Arc<LinkedProgram>> {\n        self.linked.as_ref()\n    }\n\n",
        "    pub fn linked(&self) -> Option<&Arc<LinkedProgram>> {\n        self.linked.as_ref()\n    }\n\n    /// Canonical resolver results keyed by importer and the written logical import path.\n    pub fn resolved_imports(&self) -> &BTreeMap<(ModuleId, String), ModuleId> {\n        &self.resolved_imports\n    }\n\n",
    ),
    (
        "            linked: self.linked.clone(),\n            generation: self.generation,\n",
        "            linked: self.linked.clone(),\n            resolved_imports: self.resolved_imports.clone(),\n            generation: self.generation,\n",
    ),
    (
        "        self.linked = staged.linked;\n        self.generation = staged.generation;\n",
        "        self.linked = staged.linked;\n        self.resolved_imports = staged.resolved_imports;\n        self.generation = staged.generation;\n",
    ),
    (
        "        self.linked = Some(linked.clone());\n        Ok(WorkspaceModuleUpdate {\n",
        "        self.resolved_imports = resolved;\n        self.linked = Some(linked.clone());\n        Ok(WorkspaceModuleUpdate {\n",
    ),
]
for old, new in replacements:
    if old not in text:
        raise SystemExit(f"missing phalcom-modules anchor:\n{old}")
    text = text.replace(old, new, 1)
modules.write_text(text)

semantic = Path("phalcom-semantic/src/session.rs")
text = semantic.read_text()
old = "        let mut resolved_imports_map = BTreeMap::new();\n"
new = "        let mut resolved_imports_map = self.module_session.resolved_imports().clone();\n"
if old not in text:
    raise SystemExit("missing semantic resolved_imports_map anchor")
semantic.write_text(text.replace(old, new, 1))
