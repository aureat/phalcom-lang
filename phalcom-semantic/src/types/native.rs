//! Normalization of rich native metadata specifications into canonical semantic types.

use super::application::TypeApplicationError;
use super::evidence::{EvidenceAuthority, TypeEvidence, TypeKnowledge, UnknownReason};
use super::id::TypeId;
use super::store::{TupleTypeElement, TypeStore};
use crate::declarations::DeclarationTypeTable;
use crate::identity::DeclarationId;
use phalcom_native_meta::types::TypeExprSpec;
use phalcom_native_meta::universe::UniverseKey;
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
            if let Some(form) = declarations.form(&decl) {
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
            let origin_form = resolve_native_type_form(store, declarations, parameters, universe_resolver, origin)?;
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

/// Registers declaration surfaces and dispatch signatures dynamically from the canonical native surface catalog.
pub fn register_native_surfaces(
    store: &mut TypeStore,
    declarations: &DeclarationTypeTable,
    resolver: &dyn crate::types::annotation::TypeResolver,
    current_module: &crate::identity::ModuleId,
    dispatch: &mut crate::dispatch::SurfaceDispatchResolver,
) {
    use crate::dispatch::{CallableParameter, CallableSignature};
    use crate::surface::DeclarationSurface;
    use phalcom_common::selector::Selector;
    use phalcom_native_surface::NATIVE_SURFACES;

    let universe_resolver = |key: UniverseKey| -> DeclarationId {
        resolver
            .resolve_type_name(current_module, key.name(), &[])
            .unwrap_or_else(|| DeclarationId::new(crate::identity::ModuleId::core(), key.name().into()))
    };

    let empty_params = HashMap::new();
    let mut surfaces_by_decl: HashMap<DeclarationId, DeclarationSurface> = HashMap::new();

    for record in NATIVE_SURFACES {
        let owner_name = record.owner().name();
        let decl = match resolver.resolve_type_name(current_module, owner_name, &[]) {
            Some(d) => d,
            None => DeclarationId::new(crate::identity::ModuleId::core(), owner_name.into()),
        };

        if let Some(t_self) = declarations.form(&decl) {
            dispatch.register_type(t_self, decl.clone());
        }

        let side = match record.side() {
            phalcom_native_meta::NativeDispatch::Instance => crate::identity::DispatchSide::Instance,
            phalcom_native_meta::NativeDispatch::Class => crate::identity::DispatchSide::Class,
        };

        let Ok(selector) = Selector::try_decode_exact(record.selector()) else {
            continue;
        };

        // Lower parameters
        let mut params = Vec::new();
        for (i, p_spec) in record.params().positional.iter().enumerate() {
            let p_knowledge = normalize_native_type(store, declarations, &empty_params, &universe_resolver, p_spec);
            let name = if i == 0 { "other" } else { "arg" };
            params.push(CallableParameter::new(name, p_knowledge));
        }

        // Lower return type
        let ret_knowledge = match record.flow() {
            phalcom_native_meta::ReturnFlowSpec::Receiver => {
                if let Some(t_self) = declarations.form(&decl) {
                    TypeKnowledge::known(t_self, EvidenceAuthority::TrustedNative)
                } else {
                    normalize_native_type(store, declarations, &empty_params, &universe_resolver, record.returns())
                }
            }
            phalcom_native_meta::ReturnFlowSpec::Never => TypeKnowledge::known(store.never(), EvidenceAuthority::TrustedNative),
            _ => normalize_native_type(store, declarations, &empty_params, &universe_resolver, record.returns()),
        };

        let sig = CallableSignature::new(selector, params, ret_knowledge);
        surfaces_by_decl
            .entry(decl)
            .or_insert_with(|| DeclarationSurface::new(None))
            .add_callable(side, sig);
    }

    for (decl, surface) in surfaces_by_decl {
        dispatch.register_surface(decl, surface);
    }
}

/// Registers standard declaration surfaces and dispatch signatures for core primitive types.
#[inline]
pub fn register_standard_surfaces(
    store: &mut TypeStore,
    declarations: &DeclarationTypeTable,
    resolver: &dyn crate::types::annotation::TypeResolver,
    current_module: &crate::identity::ModuleId,
    dispatch: &mut crate::dispatch::SurfaceDispatchResolver,
) {
    register_native_surfaces(store, declarations, resolver, current_module, dispatch);
}
