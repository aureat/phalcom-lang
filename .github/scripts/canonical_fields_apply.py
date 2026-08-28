from pathlib import Path


def replace_once(text: str, old: str, new: str, label: str) -> str:
    if old in text:
        return text.replace(old, new, 1)
    if new in text:
        return text
    raise SystemExit(f"{label} shape changed")


def replace_exact_count(text: str, old: str, new: str, count: int, label: str) -> str:
    old_count = text.count(old)
    new_count = text.count(new)
    if old_count == count:
        return text.replace(old, new)
    if old_count == 0 and new_count >= count:
        return text
    raise SystemExit(f"{label} shape changed: {old_count=}, {new_count=}")


# 1. Formal field lookup consumes canonical FieldSignatureTable. Structural
# DeclarationSurface dependency remains temporarily for invalidation until the
# next slice gives FieldSignature its own DB query key.
path = Path("phalcom-semantic/src/checker/context.rs")
text = path.read_text()
text = replace_once(
    text,
    "use crate::surface::DeclarationSurface;\n",
    "use crate::signature::FieldSignatureTable;\nuse crate::surface::DeclarationSurface;\n",
    "field signature table import",
)
text = replace_once(
    text,
    "    semantic_dependencies: SharedSemanticDependencies,\n    pub dispatch: DispatchAccess<'a>,\n",
    "    semantic_dependencies: SharedSemanticDependencies,\n    field_signatures: Option<&'a FieldSignatureTable>,\n    pub dispatch: DispatchAccess<'a>,\n",
    "checking context field signatures",
)
text = replace_exact_count(
    text,
    "            semantic_dependencies,\n            dispatch:",
    "            semantic_dependencies,\n            field_signatures: None,\n            dispatch:",
    2,
    "checking context constructors",
)
text = replace_once(
    text,
    "            semantic_dependencies: self.semantic_dependencies.clone(),\n            dispatch: DispatchAccess::Borrowed(self.dispatch.get()),\n",
    "            semantic_dependencies: self.semantic_dependencies.clone(),\n            field_signatures: self.field_signatures,\n            dispatch: DispatchAccess::Borrowed(self.dispatch.get()),\n",
    "resolver subcontext field signatures",
)
anchor = '''    /// Returns the dispatch resolver currently visible to this context.
    pub fn dispatch_ref(&self) -> &SurfaceDispatchResolver {
'''
insert = '''    /// Attaches compiler-owned canonical field declaration knowledge.
    pub fn attach_field_signatures(&mut self, field_signatures: &'a FieldSignatureTable) {
        self.field_signatures = Some(field_signatures);
    }

''' + anchor
text = replace_once(text, anchor, insert, "field signature attachment")
old = '''    pub fn get_field(&self, decl: &DeclarationId, side: DispatchSide, name: &str) -> Option<TypeKnowledge> {
        record_declaration_surface_dependency(&self.semantic_dependencies, decl);
        self.dispatch.get().get_surface(decl).and_then(|s| s.get_field(side, name)).cloned()
    }

    pub(crate) fn resolve_field_contract(&self, owner: &DeclarationId, side: DispatchSide, name: &str) -> Option<(crate::identity::FieldId, TypeKnowledge)> {
        record_declaration_surface_dependency(&self.semantic_dependencies, owner);
        let surface = self.dispatch.get().get_surface(owner)?;
        let field = surface.get_field_id(side, name)?.clone();
        let contract = surface.get_field(side, name)?.clone();
        Some((field, contract))
    }
'''
new = '''    pub fn get_field(&self, decl: &DeclarationId, side: DispatchSide, name: &str) -> Option<TypeKnowledge> {
        // Structural invalidation is intentionally retained until FieldSignature
        // becomes its own query product. Type authority already belongs solely
        // to canonical declaration knowledge.
        record_declaration_surface_dependency(&self.semantic_dependencies, decl);
        let field = crate::identity::FieldId::new(decl.clone(), name, side);
        let signature = self.field_signatures?.get(&field)?;
        Some(signature.declared_type.to_knowledge())
    }

    pub(crate) fn resolve_field_contract(&self, owner: &DeclarationId, side: DispatchSide, name: &str) -> Option<(crate::identity::FieldId, TypeKnowledge)> {
        record_declaration_surface_dependency(&self.semantic_dependencies, owner);
        let field = crate::identity::FieldId::new(owner.clone(), name, side);
        let signature = self.field_signatures?.get(&field)?;
        Some((field, signature.declared_type.to_knowledge()))
    }
'''
text = replace_once(text, old, new, "formal field authority")
path.write_text(text)

# 2. Callable body checking receives the canonical field table explicitly.
path = Path("phalcom-semantic/src/checker/body.rs")
text = path.read_text()
text = replace_once(
    text,
    "        cancel,\n        None,\n    )\n}\n\npub fn analyze_callable_body_with_fields(",
    "        cancel,\n        None,\n        None,\n    )\n}\n\npub fn analyze_callable_body_with_fields(",
    "compat body field signature argument",
)
text = replace_once(
    text,
    "    cancel: &CancellationToken,\n    field_lifecycle: Option<&crate::checker::field_lifecycle::FieldLifecycleTable>,\n) -> CallableAnalysis {\n    let control = CheckerControl::new(budget, cancel);\n    let mut ctx = CheckingContext::new_with_dispatch_ref_and_control(store, hierarchy, resolver, declarations, dispatch, module, control);\n",
    "    cancel: &CancellationToken,\n    field_signatures: Option<&crate::signature::FieldSignatureTable>,\n    field_lifecycle: Option<&crate::checker::field_lifecycle::FieldLifecycleTable>,\n) -> CallableAnalysis {\n    let control = CheckerControl::new(budget, cancel);\n    let mut ctx = CheckingContext::new_with_dispatch_ref_and_control(store, hierarchy, resolver, declarations, dispatch, module, control);\n    if let Some(field_signatures) = field_signatures {\n        ctx.attach_field_signatures(field_signatures);\n    }\n",
    "body field signature input",
)
path.write_text(text)

# 3. Formal DB body queries pass the current canonical table without making it
# part of the coarse body input fingerprint. Fine-grained field query ownership
# lands in the following slice.
path = Path("phalcom-semantic/src/db/query.rs")
text = path.read_text()
text = replace_once(
    text,
    "    pub declarations: &'a DeclarationTypeTable,\n    pub field_lifecycle: Option<&'a crate::checker::field_lifecycle::FieldLifecycleTable>,\n",
    "    pub declarations: &'a DeclarationTypeTable,\n    pub field_signatures: Option<&'a crate::signature::FieldSignatureTable>,\n    pub field_lifecycle: Option<&'a crate::checker::field_lifecycle::FieldLifecycleTable>,\n",
    "formal query field signatures",
)
text = replace_once(
    text,
    "        cancel,\n        formal_inputs.and_then(|inputs| inputs.field_lifecycle),\n    );\n",
    "        cancel,\n        formal_inputs.and_then(|inputs| inputs.field_signatures),\n        formal_inputs.and_then(|inputs| inputs.field_lifecycle),\n    );\n",
    "body query field signatures",
)
path.write_text(text)

# 4. Workspace formal analysis attaches the single canonical field table to all
# consumers, including default initializers and fixed-point body rechecks.
path = Path("phalcom-semantic/src/session.rs")
text = path.read_text()
text = replace_once(
    text,
    "            let mut ctx = CheckingContext::new_with_dispatch_ref(&mut self.store, &hierarchy, &resolver, &declarations, &dispatch, module_id.clone());\n            for stmt in &parsed_unit.program.statements {\n",
    "            let mut ctx = CheckingContext::new_with_dispatch_ref(&mut self.store, &hierarchy, &resolver, &declarations, &dispatch, module_id.clone());\n            ctx.attach_field_signatures(&field_signatures);\n            for stmt in &parsed_unit.program.statements {\n",
    "default field context attachment",
)
text = replace_once(
    text,
    "                                    declarations: &declarations,\n                                    field_lifecycle: Some(&field_lifecycle),\n",
    "                                    declarations: &declarations,\n                                    field_signatures: Some(&field_signatures),\n                                    field_lifecycle: Some(&field_lifecycle),\n",
    "formal body input field signatures",
)
# The second direct context is the top-level/field-initializer checker.
needle = "            let mut ctx = CheckingContext::new_with_dispatch_ref(&mut self.store, &hierarchy, &resolver, &declarations, &dispatch, module_id.clone());\n\n            for stmt in &parsed_unit.program.statements {\n"
replacement = "            let mut ctx = CheckingContext::new_with_dispatch_ref(&mut self.store, &hierarchy, &resolver, &declarations, &dispatch, module_id.clone());\n            ctx.attach_field_signatures(&field_signatures);\n\n            for stmt in &parsed_unit.program.statements {\n"
text = replace_once(text, needle, replacement, "top-level field context attachment")
text = replace_once(
    text,
    "            previous_snapshot.as_ref().map(|snapshot| snapshot.callable_analyses.as_ref()),\n            &field_lifecycle,\n",
    "            previous_snapshot.as_ref().map(|snapshot| snapshot.callable_analyses.as_ref()),\n            &field_signatures,\n            &field_lifecycle,\n",
    "refresh field signature call",
)
text = replace_once(
    text,
    "    previous_callable_analyses: Option<&HashMap<crate::identity::CallableId, Arc<crate::checker::CallableAnalysis>>>,\n    field_lifecycle: &crate::checker::field_lifecycle::FieldLifecycleTable,\n",
    "    previous_callable_analyses: Option<&HashMap<crate::identity::CallableId, Arc<crate::checker::CallableAnalysis>>>,\n    field_signatures: &FieldSignatureTable,\n    field_lifecycle: &crate::checker::field_lifecycle::FieldLifecycleTable,\n",
    "refresh field signature parameter",
)
text = replace_once(
    text,
    "                        cancel,\n                        Some(field_lifecycle),\n                    );\n",
    "                        cancel,\n                        Some(field_signatures),\n                        Some(field_lifecycle),\n                    );\n",
    "refresh body field signatures",
)
path.write_text(text)

# Architecture guard: canonical field type retrieval must not read dispatch.
context = Path("phalcom-semantic/src/checker/context.rs").read_text()
if "surface.get_field(side, name)" in context:
    raise SystemExit("formal field typing still reads DeclarationSurface")
if "field_signatures" not in context:
    raise SystemExit("checker context does not consume canonical field signatures")
