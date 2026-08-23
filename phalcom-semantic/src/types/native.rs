//! Normalization of rich native metadata specifications into canonical semantic types.

use super::application::TypeApplicationError;
use super::evidence::{EvidenceAuthority, TypeEvidence, TypeKnowledge, UnknownReason};
use super::id::TypeId;
use super::store::{TupleTypeElement, TypeStore};
use crate::declarations::DeclarationTypeTable;
use crate::identity::DeclarationId;
use phalcom_common::selector::Selector;
use phalcom_native_meta::types::TypeExprSpec;
use phalcom_native_meta::universe::UniverseKey;
use phalcom_native_meta::{NativeDispatch, PrimitiveKey, ReturnFlowSpec};
use phalcom_native_surface::{NATIVE_SURFACES, NativeCatalogFingerprint, NativeSurfaceId, catalog_fingerprint};
use std::collections::HashMap;

#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum NativeTypeResolutionError {
    #[error("unknown type parameter: {0}")]
    UnknownParameter(String),
    #[error("missing declaration type info for: {0:?}")]
    MissingDeclaration(DeclarationId),
    #[error("type application error: {0}")]
    Application(#[from] TypeApplicationError),
    #[error("unsupported native type expression")]
    Unsupported,
}

/// Structured failure while importing generated native metadata.
#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum NativeSurfaceImportError {
    #[error("native surface {key:?} has invalid selector: {details}")]
    InvalidSelector { key: PrimitiveKey, details: String },
    #[error("native surface {key:?} owner is missing from semantic declarations")]
    OwnerMissing { key: PrimitiveKey },
    #[error("native surface {key:?} selector/callable arity mismatch: {details}")]
    SelectorArityMismatch { key: PrimitiveKey, details: String },
    #[error("native surface {key:?} type lowering failed: {source}")]
    TypeLowering { key: PrimitiveKey, source: NativeTypeResolutionError },
    #[error("native surface {key:?} has unsupported metadata: {reason}")]
    UnsupportedMetadata { key: PrimitiveKey, reason: String },
}

/// Result of importing the VM-free native catalog into semantic dispatch.
#[derive(Clone, Debug)]
pub struct NativeSurfaceImportReport {
    pub imported_keys: Vec<NativeSurfaceId>,
    pub callable_signatures: Vec<(crate::identity::CallableId, crate::signature::CallableSemanticSignature)>,
    pub failures: Vec<NativeSurfaceImportError>,
    pub fingerprint: NativeCatalogFingerprint,
}

impl Default for NativeSurfaceImportReport {
    fn default() -> Self {
        Self {
            imported_keys: Vec::new(),
            callable_signatures: Vec::new(),
            failures: Vec::new(),
            fingerprint: catalog_fingerprint(),
        }
    }
}

/// Resolves a native [`TypeExprSpec`] into a canonical [`TypeId`] form within the given store.
pub fn resolve_native_type_form(
    store: &mut TypeStore,
    declarations: &DeclarationTypeTable,
    parameters: &HashMap<&str, TypeId>,
    universe_resolver: &dyn Fn(UniverseKey) -> DeclarationId,
    spec: &TypeExprSpec,
) -> Result<TypeId, NativeTypeResolutionError> {
    match spec {
        TypeExprSpec::Never => Ok(store.never()),
        TypeExprSpec::Universe(UniverseKey::Unit) => Ok(store.unit()),
        TypeExprSpec::Universe(key) => {
            let decl = universe_resolver(*key);
            if let Some(mut form) = declarations.form(&decl) {
                while !store.is_proper_type(form) {
                    let kind_id = store.kind_of(form);
                    if let crate::types::kind::KindData::Arrow { parameters: ref params, .. } = store.get_kind(kind_id).clone() {
                        let top = declarations.form(&universe_resolver(UniverseKey::Object)).unwrap_or_else(|| store.never());
                        let args = vec![top; params.len()];
                        if let Ok(applied) = store.apply_type_form(form, &args) {
                            form = applied;
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                }
                Ok(form)
            } else {
                Err(NativeTypeResolutionError::MissingDeclaration(decl))
            }
        }
        TypeExprSpec::Parameter(name) => {
            if let Some(&ty) = parameters.get(name) {
                Ok(ty)
            } else {
                Err(NativeTypeResolutionError::UnknownParameter((*name).into()))
            }
        }
        TypeExprSpec::Applied { origin, arguments } => {
            let origin_form = match origin {
                TypeExprSpec::Universe(key) => {
                    let decl = universe_resolver(*key);
                    declarations.form(&decl).ok_or(NativeTypeResolutionError::MissingDeclaration(decl))?
                }
                _ => resolve_native_type_form(store, declarations, parameters, universe_resolver, origin)?,
            };
            let mut arg_forms = Vec::new();
            for arg in *arguments {
                arg_forms.push(resolve_native_type_form(store, declarations, parameters, universe_resolver, arg)?);
            }
            store.apply_type_form(origin_form, &arg_forms).map_err(NativeTypeResolutionError::Application)
        }
        TypeExprSpec::Union(members) => {
            let mut member_forms = Vec::new();
            for m in *members {
                let f = resolve_native_type_form(store, declarations, parameters, universe_resolver, m)?;
                if store.is_proper_type(f) {
                    member_forms.push(f);
                } else {
                    return Err(NativeTypeResolutionError::Unsupported);
                }
            }
            Ok(store.union(&member_forms))
        }
        TypeExprSpec::Tuple(tuple_spec) => {
            let mut elements = Vec::new();
            for pos in tuple_spec.positional {
                let f = resolve_native_type_form(store, declarations, parameters, universe_resolver, pos)?;
                if !store.is_proper_type(f) {
                    return Err(NativeTypeResolutionError::Unsupported);
                }
                elements.push(TupleTypeElement { label: None, ty: f });
            }
            for labeled in tuple_spec.labeled {
                let f = resolve_native_type_form(store, declarations, parameters, universe_resolver, labeled.ty)?;
                if !store.is_proper_type(f) {
                    return Err(NativeTypeResolutionError::Unsupported);
                }
                elements.push(TupleTypeElement {
                    label: Some(labeled.label.into()),
                    ty: f,
                });
            }
            Ok(store.tuple(elements.into_boxed_slice()))
        }
        TypeExprSpec::Unknown | TypeExprSpec::SelfType => Err(NativeTypeResolutionError::Unsupported),
    }
}

/// Normalizes a native [`TypeExprSpec`] into canonical [`TypeKnowledge`].
pub fn normalize_native_type(
    store: &mut TypeStore,
    declarations: &DeclarationTypeTable,
    parameters: &HashMap<&str, TypeId>,
    universe_resolver: &dyn Fn(UniverseKey) -> DeclarationId,
    spec: &TypeExprSpec,
) -> TypeKnowledge {
    match resolve_native_type_form(store, declarations, parameters, universe_resolver, spec) {
        Ok(form) if store.is_proper_type(form) => TypeKnowledge::Known(TypeEvidence {
            ty: form,
            authority: EvidenceAuthority::TrustedNative,
            provenance: Default::default(),
        }),
        _ => TypeKnowledge::Unknown(UnknownReason::OpaqueNative),
    }
}

fn import_native_type(
    store: &mut TypeStore,
    declarations: &DeclarationTypeTable,
    parameters: &HashMap<&str, TypeId>,
    universe_resolver: &dyn Fn(UniverseKey) -> DeclarationId,
    key: PrimitiveKey,
    spec: &TypeExprSpec,
) -> Result<TypeKnowledge, NativeSurfaceImportError> {
    if matches!(spec, TypeExprSpec::Unknown | TypeExprSpec::SelfType) {
        return Ok(normalize_native_type(store, declarations, parameters, universe_resolver, spec));
    }
    let form = resolve_native_type_form(store, declarations, parameters, universe_resolver, spec)
        .map_err(|source| NativeSurfaceImportError::TypeLowering { key, source })?;
    if !store.is_proper_type(form) {
        return Err(NativeSurfaceImportError::TypeLowering {
            key,
            source: NativeTypeResolutionError::Unsupported,
        });
    }
    Ok(TypeKnowledge::known(form, EvidenceAuthority::TrustedNative))
}

/// Registers declaration surfaces and dispatch signatures dynamically from the canonical native surface catalog.
pub fn register_native_surfaces(
    store: &mut TypeStore,
    declarations: &DeclarationTypeTable,
    resolver: &dyn crate::types::annotation::TypeResolver,
    current_module: &crate::identity::ModuleId,
    dispatch: &mut crate::dispatch::SurfaceDispatchResolver,
) -> Result<NativeSurfaceImportReport, NativeSurfaceImportError> {
    use crate::dispatch::{CallableParameter, CallableSignature};
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
        };

        // Lower parameters
        let mut params = Vec::new();
        for (i, p_spec) in record.params().positional.iter().enumerate() {
            let p_knowledge = import_native_type(store, declarations, &empty_params, &universe_resolver, record.surface.key, p_spec)?;
            let name = if i == 0 { "other" } else { "arg" };
            params.push(CallableParameter::new(name, p_knowledge));
        }
        for labeled in record.params().labeled {
            let p_knowledge = import_native_type(store, declarations, &empty_params, &universe_resolver, record.surface.key, labeled.ty)?;
            params.push(CallableParameter::new(labeled.label, p_knowledge).with_label(labeled.label));
        }
        if let Some(rest) = record.params().rest {
            let rest_knowledge = rest
                .ty
                .map(|ty| import_native_type(store, declarations, &empty_params, &universe_resolver, record.surface.key, ty))
                .transpose()?
                .unwrap_or(TypeKnowledge::Unknown(UnknownReason::OpaqueNative));
            params.push(CallableParameter::new("rest", rest_knowledge).with_rest(true));
        }

        // Lower return type
        let ret_knowledge = match record.flow() {
            ReturnFlowSpec::Receiver => {
                if let Some(t_self) = declarations.form(&decl) {
                    TypeKnowledge::known(t_self, EvidenceAuthority::TrustedNative)
                } else {
                    return Err(NativeSurfaceImportError::TypeLowering {
                        key: record.surface.key,
                        source: NativeTypeResolutionError::MissingDeclaration(decl.clone()),
                    });
                }
            }
            ReturnFlowSpec::Never => TypeKnowledge::known(store.never(), EvidenceAuthority::TrustedNative),
            ReturnFlowSpec::Argument(index) => {
                params
                    .get(index)
                    .map(|param| param.ty.clone())
                    .ok_or_else(|| NativeSurfaceImportError::UnsupportedMetadata {
                        key: record.surface.key,
                        reason: format!("return flow references missing parameter {index}"),
                    })?
            }
            _ => import_native_type(store, declarations, &empty_params, &universe_resolver, record.surface.key, record.returns())?,
        };

        let sig = CallableSignature::new(selector, params, ret_knowledge);
        let callable_id = crate::identity::CallableId::new(decl.clone(), sig.selector.clone(), side);
        report.imported_keys.push(record.id());
        surfaces_by_decl
            .entry(decl.clone())
            .or_insert_with(|| DeclarationSurface::new(Some(decl.clone())))
            .add_callable(side, sig);

        // The legacy dispatch surface remains the live query adapter. The
        // canonical identity is retained in the import report for clients
        // that publish the richer signature table.
        if let Some(surface) = surfaces_by_decl.get(&callable_id.owner) {
            if let Some(signature) = surface.get_callable(side, &callable_id.selector) {
                let parameters = signature
                    .parameters
                    .iter()
                    .enumerate()
                    .filter_map(|(index, parameter)| {
                        let ty = parameter.ty.ty()?;
                        Some(crate::signature::CallableParameterSemantic::new(
                            index as u32,
                            parameter.local_name.clone(),
                            ty.into(),
                        ))
                    })
                    .collect::<Vec<_>>()
                    .into_boxed_slice();
                if let Some(return_type) = signature.return_type.ty() {
                    let callable_owner = callable_id.owner.clone();
                    report.callable_signatures.push((
                        callable_id.clone(),
                        crate::signature::CallableSemanticSignature {
                            callable: callable_id.clone(),
                            owner: callable_owner,
                            side,
                            selector: signature.selector.clone(),
                            generics: None,
                            parameters,
                            return_type: return_type.into(),
                            source: None,
                            implementation: phalcom_native_meta::ImplementationKind::NativePrimitive,
                            native_id: Some(record.id()),
                            effects: record.effects(),
                            raises: record.raises(),
                            flow: record.flow(),
                            lifecycle: record.lifecycle(),
                        },
                    ));
                }
            }
        }
    }

    for (decl, surface) in surfaces_by_decl {
        dispatch.register_surface(decl, surface);
    }

    Ok(report)
}
