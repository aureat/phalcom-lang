from pathlib import Path
import re

path = Path("phalcom-semantic/src/types/native.rs")
text = path.read_text()

start = text.index("pub fn register_native_surfaces(")
# This function is the final item in the module today. Replace it as one unit so
# the native import cannot retain a hidden dispatch->semantic reconstruction path.
replacement = r'''pub fn register_native_surfaces(
    store: &mut TypeStore,
    declarations: &DeclarationTypeTable,
    resolver: &dyn crate::types::annotation::TypeResolver,
    current_module: &crate::identity::ModuleId,
    dispatch: &mut crate::dispatch::SurfaceDispatchResolver,
) -> Result<NativeSurfaceImportReport, NativeSurfaceImportError> {
    use crate::surface::DeclarationSurface;

    let universe_resolver = |key: UniverseKey| -> DeclarationId { DeclarationId::new(crate::identity::ModuleId::core(), key.name().into()) };

    let empty_params = HashMap::new();
    let mut surfaces_by_decl: HashMap<DeclarationId, DeclarationSurface> = HashMap::new();
    let mut report = NativeSurfaceImportReport::default();
    let mut records: Vec<_> = NATIVE_SURFACES.iter().collect();
    records.sort_by_key(|record| record.surface.key.sort_key());

    for record in records {
        let owner_name = record.owner().name();
        let decl = match resolver.resolve_type_name(current_module, owner_name, &[]) {
            Some(d) => d,
            None => {
                let fallback = DeclarationId::new(crate::identity::ModuleId::core(), owner_name.into());
                if declarations.form(&fallback).is_none() {
                    return Err(NativeSurfaceImportError::OwnerMissing { key: record.surface.key });
                }
                fallback
            }
        };

        if let Some(t_self) = declarations.form(&decl) {
            dispatch.register_type(t_self, decl.clone());
        }

        let side = match record.side() {
            NativeDispatch::Instance => crate::identity::DispatchSide::Instance,
            NativeDispatch::Class => crate::identity::DispatchSide::Class,
        };

        let selector = Selector::try_decode_exact(record.selector()).map_err(|error| NativeSurfaceImportError::InvalidSelector {
            key: record.surface.key,
            details: error.to_string(),
        })?;

        let declared_arity = record.params().positional.len() + record.params().labeled.len();
        let selector_arity = match selector.kind {
            phalcom_common::selector::SelectorKind::Getter => 0,
            phalcom_common::selector::SelectorKind::Setter => 1,
            phalcom_common::selector::SelectorKind::Method | phalcom_common::selector::SelectorKind::SubscriptGet => selector.slots.len(),
            phalcom_common::selector::SelectorKind::SubscriptSet => selector.slots.len() + 1,
        };
        if declared_arity != selector_arity && record.params().rest.is_none() {
            return Err(NativeSurfaceImportError::SelectorArityMismatch {
                key: record.surface.key,
                details: format!("selector has {selector_arity} slots but metadata has {declared_arity} parameters"),
            });
        }

        let callable_id = crate::identity::CallableId::new(decl.clone(), selector.clone(), side);
        let mut parameters = Vec::new();

        for (index, p_spec) in record.params().positional.iter().enumerate() {
            let knowledge = import_native_type(store, declarations, &empty_params, &universe_resolver, record.surface.key, p_spec)?;
            let name = if index == 0 { "other" } else { "arg" };
            parameters.push(crate::signature::CallableParameterSemantic::new(
                crate::identity::CallableParameterId::new(callable_id.clone(), index as u32),
                name,
                crate::declaration_type::DeclaredTypeFact::from_knowledge_with_basis(
                    &knowledge,
                    crate::declaration_type::DeclaredTypeBasis::NativeSignature,
                ),
            ));
        }

        for labeled in record.params().labeled {
            let knowledge = import_native_type(store, declarations, &empty_params, &universe_resolver, record.surface.key, labeled.ty)?;
            let index = parameters.len() as u32;
            parameters.push(
                crate::signature::CallableParameterSemantic::new(
                    crate::identity::CallableParameterId::new(callable_id.clone(), index),
                    labeled.label,
                    crate::declaration_type::DeclaredTypeFact::from_knowledge_with_basis(
                        &knowledge,
                        crate::declaration_type::DeclaredTypeBasis::NativeSignature,
                    ),
                )
                .with_external_label(Some(labeled.label.into())),
            );
        }

        if let Some(rest) = record.params().rest {
            let knowledge = rest
                .ty
                .map(|ty| import_native_type(store, declarations, &empty_params, &universe_resolver, record.surface.key, ty))
                .transpose()?
                .unwrap_or(TypeKnowledge::Unknown(UnknownReason::OpaqueNative));
            let index = parameters.len() as u32;
            parameters.push(
                crate::signature::CallableParameterSemantic::new(
                    crate::identity::CallableParameterId::new(callable_id.clone(), index),
                    "rest",
                    crate::declaration_type::DeclaredTypeFact::from_knowledge_with_basis(
                        &knowledge,
                        crate::declaration_type::DeclaredTypeBasis::NativeSignature,
                    ),
                )
                .with_rest(phalcom_ast::ast::RestMode::Complete),
            );
        }

        let return_knowledge = match record.flow() {
            ReturnFlowSpec::Receiver => {
                let self_ty = store.self_type(crate::types::parameter::SelfTypeTerm {
                    owner: decl.clone(),
                    side,
                    role: crate::types::parameter::SelfRole::InstanceType,
                });
                TypeKnowledge::established(self_ty, EvidenceOrigin::NativeSignature)
            }
            ReturnFlowSpec::Never => TypeKnowledge::established(store.never(), EvidenceOrigin::NativeSignature),
            ReturnFlowSpec::Argument(index) => parameters
                .get(index)
                .map(|parameter| parameter.declared_type.to_knowledge())
                .ok_or_else(|| NativeSurfaceImportError::UnsupportedMetadata {
                    key: record.surface.key,
                    reason: format!("return flow references missing parameter {index}"),
                })?,
            _ => import_native_type(store, declarations, &empty_params, &universe_resolver, record.surface.key, record.returns())?,
        };

        let canonical_signature = crate::signature::CallableSemanticSignature {
            callable: callable_id.clone(),
            owner: decl.clone(),
            side,
            selector: selector.clone(),
            generics: None,
            parameters: parameters.into_boxed_slice(),
            declared_return: crate::declaration_type::DeclaredTypeFact::from_knowledge_with_basis(
                &return_knowledge,
                crate::declaration_type::DeclaredTypeBasis::NativeSignature,
            ),
            inferred_return: None,
            source: None,
            implementation: phalcom_native_meta::ImplementationKind::NativePrimitive,
            native_id: Some(record.id()),
            effects: record.effects(),
            raises: record.raises(),
            flow: record.flow(),
            lifecycle: record.lifecycle(),
        };

        let projection = crate::checker::declaration_signature::project_semantic_signature(&canonical_signature);
        surfaces_by_decl
            .entry(decl.clone())
            .or_insert_with(|| DeclarationSurface::new(Some(decl.clone())))
            .add_callable(side, projection);
        report.imported_keys.push(record.id());
        report.callable_signatures.push((callable_id, canonical_signature));
    }

    for (decl, surface) in surfaces_by_decl {
        dispatch.register_surface(decl, surface);
    }

    Ok(report)
}
'''

text = text[:start] + replacement

for forbidden in (
    "surface.get_callable(side, &callable_id.selector)",
    "use crate::dispatch::{CallableParameter, CallableSignature};",
):
    if forbidden in text:
        raise SystemExit(f"native reverse authority remains: {forbidden}")
if "project_semantic_signature(&canonical_signature)" not in text:
    raise SystemExit("native dispatch is not projected from canonical signature")

path.write_text(text)
