from pathlib import Path


def replace_once(text: str, old: str, new: str, label: str) -> str:
    if old in text:
        return text.replace(old, new, 1)
    if new in text:
        return text
    raise SystemExit(f"{label} shape changed")


# 1. Source fields are lowered into canonical FieldSemanticSignature first.
path = Path("phalcom-semantic/src/checker/declaration_signature.rs")
text = path.read_text()
text = replace_once(
    text,
    "use crate::identity::{CallableId, CallableParameterId, DeclarationId, DispatchSide};\nuse crate::signature::{CallableParameterSemantic, CallableSemanticSignature};\n",
    "use crate::identity::{CallableId, CallableParameterId, DeclarationId, DispatchSide, FieldId};\nuse crate::signature::{CallableParameterSemantic, CallableSemanticSignature, FieldSemanticSignature};\n",
    "field signature imports",
)
anchor = "pub(crate) fn semantic_signature_for_member(ctx: &mut CheckingContext<'_>, owner: &DeclarationId, member: &ClassMember) -> Option<CallableSemanticSignature> {\n"
helper = '''pub(crate) fn semantic_field_signature_for_member(
    ctx: &mut CheckingContext<'_>,
    owner: &DeclarationId,
    member: &ClassMember,
) -> Option<FieldSemanticSignature> {
    let ClassMember::Field(field) = member else {
        return None;
    };
    let side = super::declaration::member_side(member);
    let declaration_type_parameters = ctx
        .declaration_generic_signature(owner)
        .map(|signature| {
            signature
                .parameters
                .iter()
                .map(|&parameter_id| {
                    let name = ctx.store.type_parameter(parameter_id).name.to_string();
                    let form = ctx.store.parameter_form(parameter_id);
                    (name, form)
                })
                .collect()
        })
        .unwrap_or_default();
    let parent_resolver = ctx.resolver.clone();
    let declaration_resolver = crate::types::annotation::ScopedTypeResolver {
        parent: &parent_resolver,
        type_parameters: declaration_type_parameters,
    };
    let declared_type = annotation_fact(
        ctx,
        &declaration_resolver,
        field.annotation.as_ref(),
        UnknownReason::UnannotatedDeclaration,
    );
    let field_id = FieldId::new(owner.clone(), field.name.clone(), side);
    Some(FieldSemanticSignature {
        field: field_id,
        owner: owner.clone(),
        side,
        name: field.name.clone().into(),
        mutable: field.mutable,
        declared_type,
        source: None,
    })
}

pub(crate) fn project_field_signature(signature: &FieldSemanticSignature) -> TypeKnowledge {
    signature.declared_type.to_knowledge()
}

''' + anchor
text = replace_once(text, anchor, helper, "canonical field helper")
path.write_text(text)

# 2. DeclarationSurface becomes a projection for fields just as for callables.
path = Path("phalcom-semantic/src/checker/declaration.rs")
text = path.read_text()
old = '''    // Fields are still projected directly in this phase. Callable members go
    // through the declaration-owned semantic signature builder first.
    let type_params_map = if let Some(sig) = ctx.declaration_generic_signature(&decl_id) {
        sig.parameters
            .iter()
            .map(|&param_id| {
                let name = ctx.store.type_parameter(param_id).name.to_string();
                let param_form = ctx.store.parameter_form(param_id);
                (name, param_form)
            })
            .collect()
    } else {
        std::collections::HashMap::new()
    };
    let parent_resolver = ctx.resolver.clone();
    let field_resolver = crate::types::annotation::ScopedTypeResolver {
        parent: &parent_resolver,
        type_parameters: type_params_map,
    };

'''
text = replace_once(text, old, "", "direct field resolver")
old = '''            ClassMember::Field(field) => {
                let side = member_side(member);
                let declared = field
                    .annotation
                    .as_ref()
                    .map(|annotation| ctx.resolve_type_annotation(&field_resolver, annotation).0)
                    .unwrap_or_else(|| TypeKnowledge::Unknown(UnknownReason::UnannotatedDeclaration));
                surface.add_field_with_visibility(side, &field.name, declared, visibility);
            }
'''
new = '''            ClassMember::Field(_) => {
                let Some(signature) = super::declaration_signature::semantic_field_signature_for_member(ctx, &decl_id, member) else {
                    continue;
                };
                surface.add_field_with_visibility(
                    signature.side,
                    &signature.name,
                    super::declaration_signature::project_field_signature(&signature),
                    visibility,
                );
            }
'''
text = replace_once(text, old, new, "field surface projection")
text = text.replace("use crate::types::evidence::{EvidenceOrigin, TypeKnowledge, UnknownReason};", "use crate::types::evidence::{EvidenceOrigin, TypeKnowledge};")
path.write_text(text)

# 3. Snapshot owns canonical field signatures, with backwards-compatible
# constructors defaulting to an empty table for lower-level callers.
path = Path("phalcom-semantic/src/snapshot.rs")
text = path.read_text()
text = replace_once(
    text,
    "use crate::signature::CallableSignatureTable;\n",
    "use crate::signature::{CallableSignatureTable, FieldSignatureTable};\n",
    "snapshot field signature import",
)
text = replace_once(
    text,
    "    pub callable_signatures: Arc<CallableSignatureTable>,\n    pub declarations: Arc<DeclarationTypeTable>,\n",
    "    pub callable_signatures: Arc<CallableSignatureTable>,\n    pub field_signatures: Arc<FieldSignatureTable>,\n    pub declarations: Arc<DeclarationTypeTable>,\n",
    "snapshot field signature storage",
)
# Both snapshot constructors initialize the new table.
needle = "            callable_signatures,\n            declarations,\n"
replacement = "            callable_signatures,\n            field_signatures: Arc::new(FieldSignatureTable::new()),\n            declarations,\n"
if text.count(needle) != 2 and text.count(replacement) != 2:
    raise SystemExit(f"snapshot constructor shape changed: {text.count(needle)=}, {text.count(replacement)=}")
text = text.replace(needle, replacement)
anchor = '''    pub fn with_callable_analyses(mut self, callable_analyses: Arc<HashMap<crate::identity::CallableId, Arc<crate::checker::CallableAnalysis>>>) -> Self {
'''
builder = '''    pub fn with_field_signatures(mut self, field_signatures: Arc<FieldSignatureTable>) -> Self {
        self.field_signatures = field_signatures;
        self
    }

''' + anchor
text = replace_once(text, anchor, builder, "snapshot field signature builder")
path.write_text(text)

# 4. Workspace session materializes source field signatures from the same
# canonical lowering helper before dispatch surfaces are consumed.
path = Path("phalcom-semantic/src/session.rs")
text = path.read_text()
text = replace_once(
    text,
    "use crate::signature::CallableSignatureTable;\n",
    "use crate::signature::{CallableSignatureTable, FieldSignatureTable};\n",
    "session field signature import",
)
text = replace_once(
    text,
    "        let mut dispatch = self.base_dispatch.clone();\n        let mut callable_signatures = self.base_callable_signatures.clone();\n",
    "        let mut dispatch = self.base_dispatch.clone();\n        let mut callable_signatures = self.base_callable_signatures.clone();\n        let mut field_signatures = FieldSignatureTable::new();\n",
    "field signature table creation",
)
anchor = '''                let decl_id = DeclarationId::new(module_id.clone(), class_def.name.clone().into());
                // Publish declaration-owned callable signatures first. Dispatch
'''
insert = '''                let decl_id = DeclarationId::new(module_id.clone(), class_def.name.clone().into());
                {
                    let mut context = CheckingContext::new(
                        &mut self.store,
                        &hierarchy,
                        &resolver,
                        &declarations,
                        module_id.clone(),
                    );
                    for member in &class_def.members {
                        if let Some(signature) = crate::checker::declaration_signature::semantic_field_signature_for_member(&mut context, &decl_id, member) {
                            field_signatures.insert(signature);
                        }
                    }
                    if !context.diagnostics.is_empty() {
                        diags_by_module.entry(module_id.clone()).or_default().extend(context.diagnostics);
                    }
                }
                // Publish declaration-owned callable signatures first. Dispatch
'''
text = replace_once(text, anchor, insert, "session field signature lowering")
text = replace_once(
    text,
    "        snapshot_obj = snapshot_obj.with_presentation_sources(Arc::new(presentation_sources));\n",
    "        snapshot_obj = snapshot_obj.with_field_signatures(Arc::new(field_signatures));\n        snapshot_obj = snapshot_obj.with_presentation_sources(Arc::new(presentation_sources));\n",
    "snapshot field signature publication",
)
old = '''            let previous_callables = previous.callable_signatures.iter().map(|(callable, _)| callable).collect::<BTreeSet<_>>();
            let current_callables = snapshot.callable_signatures.iter().map(|(callable, _)| callable).collect::<BTreeSet<_>>();
            !previous.surfaces.keys().eq(snapshot.surfaces.keys()) || previous_callables != current_callables
'''
new = '''            let previous_callables = previous.callable_signatures.iter().map(|(callable, _)| callable).collect::<BTreeSet<_>>();
            let current_callables = snapshot.callable_signatures.iter().map(|(callable, _)| callable).collect::<BTreeSet<_>>();
            let previous_fields = previous.field_signatures.iter().map(|(field, _)| field).collect::<BTreeSet<_>>();
            let current_fields = snapshot.field_signatures.iter().map(|(field, _)| field).collect::<BTreeSet<_>>();
            !previous.surfaces.keys().eq(snapshot.surfaces.keys()) || previous_callables != current_callables || previous_fields != current_fields
'''
text = replace_once(text, old, new, "declaration index field identity")
path.write_text(text)

# Architectural postconditions.
if "field_resolver" in Path("phalcom-semantic/src/checker/declaration.rs").read_text():
    raise SystemExit("field declaration still owns a parallel source->dispatch resolver")
if "field_signatures: Arc<FieldSignatureTable>" not in Path("phalcom-semantic/src/snapshot.rs").read_text():
    raise SystemExit("snapshot does not own canonical field signatures")
