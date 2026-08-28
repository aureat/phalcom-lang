from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def replace_once(path: str, old: str, new: str) -> None:
    target = ROOT / path
    text = target.read_text()
    if old not in text:
        if new in text:
            return
        raise SystemExit(f"anchor missing in {path}: {old[:120]!r}")
    if text.count(old) != 1:
        raise SystemExit(f"anchor not unique in {path}: {text.count(old)}")
    target.write_text(text.replace(old, new, 1))


# Signature convenience keeps declaration requirement distinct from body inference.
replace_once(
    "phalcom-semantic/src/signature.rs",
    "use super::types::parameter::GenericSignature;",
    "use super::types::parameter::{GenericSignature, TypeTerm};",
)
replace_once(
    "phalcom-semantic/src/signature.rs",
    '''    pub fn published_return_knowledge(&self) -> TypeKnowledge {\n        self.inferred_return\n            .clone()\n            .unwrap_or_else(|| self.declared_return.to_knowledge())\n    }\n\n    pub fn is_complete(&self) -> bool {''',
    '''    pub fn published_return_knowledge(&self) -> TypeKnowledge {\n        self.inferred_return\n            .clone()\n            .unwrap_or_else(|| self.declared_return.to_knowledge())\n    }\n\n    pub fn published_return_term(&self) -> Option<TypeTerm> {\n        self.inferred_return\n            .as_ref()\n            .and_then(TypeKnowledge::ty)\n            .map(TypeTerm::Canonical)\n            .or_else(|| self.declared_return.known_term().cloned())\n    }\n\n    pub fn is_complete(&self) -> bool {''',
)

# Remove the now-unused complete-signature gate import.
replace_once(
    "phalcom-semantic/src/db/query.rs",
    "use crate::types::evidence::UnknownReason;\n",
    "",
)

# Metadata exports partial declaration states instead of treating absence as an incompatible model.
replace_once(
    "phalcom-semantic/src/metadata/export.rs",
    "use crate::declarations::DeclarationTypeTable;\n",
    "use crate::declaration_type::{DeclaredTypeBasis, DeclaredTypeFact, DeclaredTypeState};\nuse crate::declarations::DeclarationTypeTable;\n",
)
replace_once(
    "phalcom-semantic/src/metadata/export.rs",
    '''    CallableParameterRecord, CallableSemanticRecord, DeclarationTypeFlags, DeclarationTypeRecord, FieldMutabilityRef, FieldSemanticRecord,\n    MetadataUnavailableReason, PublishedTypeAuthority, PublishedTypeSlot, RestModeRef,\n''',
    '''    CallableParameterRecord, CallableSemanticRecord, DeclarationTypeFlags, DeclarationTypeRecord, DynamicReasonRef, FieldMutabilityRef, FieldSemanticRecord,\n    MetadataUnavailableReason, PublishedTypeAuthority, PublishedTypeSlot, RestModeRef, UnknownReasonRef,\n''',
)

# Add exporter helper immediately before bundle emission.
exporter = ROOT / "phalcom-semantic/src/metadata/export.rs"
text = exporter.read_text()
if "fn export_declared_type_fact(" not in text:
    anchor = "    pub fn build_bundle("
    pos = text.index(anchor)
    helper = '''    fn export_declared_type_fact(&mut self, fact: &DeclaredTypeFact) -> PublishedTypeSlot {\n        match &fact.state {\n            DeclaredTypeState::Known(term) => match self.export_type_term(term) {\n                Ok(form) => {\n                    let authority = match fact.basis {\n                        DeclaredTypeBasis::SourceAnnotation => PublishedTypeAuthority::DeclaredAnnotation,\n                        DeclaredTypeBasis::NativeSignature => PublishedTypeAuthority::TrustedNative,\n                        DeclaredTypeBasis::DeclarationSemantics | DeclaredTypeBasis::ConstructorSemantics => {\n                            PublishedTypeAuthority::GeneratedDeclaration\n                        }\n                        DeclaredTypeBasis::InitializerInference\n                        | DeclaredTypeBasis::BodyInference\n                        | DeclaredTypeBasis::ContextualTyping\n                        | DeclaredTypeBasis::PatternDecomposition\n                        | DeclaredTypeBasis::Unspecified => PublishedTypeAuthority::CompilerInferred,\n                    };\n                    PublishedTypeSlot::Known { form, authority }\n                }\n                Err(_) => PublishedTypeSlot::Unavailable {\n                    reason: MetadataUnavailableReason::IncompatibleModel,\n                },\n            },\n            DeclaredTypeState::Dynamic(reason) => PublishedTypeSlot::Dynamic {\n                reason: match reason {\n                    crate::types::evidence::DynamicReason::ExplicitEscape => DynamicReasonRef::ExplicitEscape,\n                    crate::types::evidence::DynamicReason::DynamicRestPack | crate::types::evidence::DynamicReason::RuntimeReflection => {\n                        DynamicReasonRef::UncheckedBoundary\n                    }\n                },\n            },\n            DeclaredTypeState::Unknown(reason) => PublishedTypeSlot::Unknown {\n                reason: match reason {\n                    crate::types::evidence::UnknownReason::UnannotatedDeclaration\n                    | crate::types::evidence::UnknownReason::NoTypeEvidence\n                    | crate::types::evidence::UnknownReason::MissingInitializer => UnknownReasonRef::UnannotatedDeclaration,\n                    crate::types::evidence::UnknownReason::OpaqueNative => UnknownReasonRef::OpaqueNative,\n                    _ => UnknownReasonRef::InferenceFailed,\n                },\n            },\n        }\n    }\n\n'''
    exporter.write_text(text[:pos] + helper + text[pos:])

replace_once(
    "phalcom-semantic/src/metadata/export.rs",
    '''                    let ty_slot = match self.export_type_term(&p.ty) {\n                        Ok(t) => PublishedTypeSlot::Known {\n                            form: t,\n                            authority: PublishedTypeAuthority::DeclaredAnnotation,\n                        },\n                        Err(_) => PublishedTypeSlot::Unavailable {\n                            reason: MetadataUnavailableReason::IncompatibleModel,\n                        },\n                    };\n                    params.push(CallableParameterRecord {\n                        index: p.index,''',
    '''                    let ty_slot = self.export_declared_type_fact(&p.declared_type);\n                    params.push(CallableParameterRecord {\n                        index: p.index(),''',
)
replace_once(
    "phalcom-semantic/src/metadata/export.rs",
    '''                let return_slot = match self.export_type_term(&sig.return_type) {\n                    Ok(t) => PublishedTypeSlot::Known {\n                        form: t,\n                        authority: PublishedTypeAuthority::DeclaredAnnotation,\n                    },\n                    Err(_) => PublishedTypeSlot::Unavailable {\n                        reason: MetadataUnavailableReason::IncompatibleModel,\n                    },\n                };''',
    '''                let return_slot = if let Some(inferred) = sig.inferred_return.as_ref().filter(|knowledge| knowledge.is_known()) {\n                    match inferred.ty().and_then(|ty| self.export_type_term(&TypeTerm::Canonical(ty)).ok()) {\n                        Some(form) => PublishedTypeSlot::Known {\n                            form,\n                            authority: PublishedTypeAuthority::CompilerInferred,\n                        },\n                        None => self.export_declared_type_fact(&sig.declared_return),\n                    }\n                } else {\n                    self.export_declared_type_fact(&sig.declared_return)\n                };''',
)
replace_once(
    "phalcom-semantic/src/metadata/export.rs",
    '''                let ty_slot = match self.export_type_term(&sig.ty) {\n                    Ok(t) => PublishedTypeSlot::Known {\n                        form: t,\n                        authority: PublishedTypeAuthority::DeclaredAnnotation,\n                    },\n                    Err(_) => PublishedTypeSlot::Unavailable {\n                        reason: MetadataUnavailableReason::IncompatibleModel,\n                    },\n                };''',
    '''                let ty_slot = self.export_declared_type_fact(&sig.declared_type);''',
)

# Protocol-neutral presentation renders partial declaration state directly.
replace_once(
    "phalcom-semantic/src/presentation.rs",
    "                index: parameter.index,",
    "                index: parameter.index(),",
)
replace_once(
    "phalcom-semantic/src/presentation.rs",
    "                type_: present_type_term(&parameter.ty, presenter),",
    "                type_: present_declared_type(&parameter.declared_type, presenter),",
)
replace_once(
    "phalcom-semantic/src/presentation.rs",
    "            return_type: present_type_term(&signature.return_type, presenter),",
    '''            return_type: signature\n                .inferred_return\n                .as_ref()\n                .filter(|knowledge| knowledge.is_known() || knowledge.is_dynamic())\n                .map(|knowledge| presenter.present_knowledge(knowledge))\n                .unwrap_or_else(|| present_declared_type(&signature.declared_return, presenter)),''',
)
replace_once(
    "phalcom-semantic/src/presentation.rs",
    '''fn present_type_term(term: &crate::types::TypeTerm, presenter: &TypePresenter<'_>) -> FormalPresentation {''',
    '''fn present_declared_type(fact: &crate::declaration_type::DeclaredTypeFact, presenter: &TypePresenter<'_>) -> FormalPresentation {\n    match &fact.state {\n        crate::declaration_type::DeclaredTypeState::Known(term) => present_type_term(term, presenter),\n        crate::declaration_type::DeclaredTypeState::Dynamic(_) => FormalPresentation::Dynamic,\n        crate::declaration_type::DeclaredTypeState::Unknown(_) => FormalPresentation::Unknown,\n    }\n}\n\nfn present_type_term(term: &crate::types::TypeTerm, presenter: &TypePresenter<'_>) -> FormalPresentation {''',
)

# Keep the transitional dispatch-first inferred-return path compiling until the next authority cutover.
replace_once(
    "phalcom-semantic/src/session.rs",
    "            if !dispatch.update_callable_return_type(&callable, summary) {",
    "            if !dispatch.update_callable_return_type(&callable, summary.clone()) {",
)

print("phase A consumer fixes applied")
