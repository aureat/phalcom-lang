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
            let origin_form = resolve_native_type_form(
                store,
                declarations,
                parameters,
                universe_resolver,
                origin,
            )?;
            let mut arg_forms = Vec::new();
            for arg in *arguments {
                arg_forms.push(resolve_native_type_form(
                    store,
                    declarations,
                    parameters,
                    universe_resolver,
                    arg,
                )?);
            }
            store
                .apply_type_form(origin_form, &arg_forms)
                .map_err(NativeTypeResolutionError::Application)
        }
        TypeExprSpec::Union(members) => {
            let mut member_forms = Vec::new();
            for m in *members {
                let f = resolve_native_type_form(
                    store,
                    declarations,
                    parameters,
                    universe_resolver,
                    m,
                )?;
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
                let f = resolve_native_type_form(
                    store,
                    declarations,
                    parameters,
                    universe_resolver,
                    pos,
                )?;
                if !store.is_proper_type(f) {
                    return Err(NativeTypeResolutionError::Unsupported);
                }
                elements.push(TupleTypeElement {
                    label: None,
                    ty: f,
                });
            }
            for labeled in tuple_spec.labeled {
                let f = resolve_native_type_form(
                    store,
                    declarations,
                    parameters,
                    universe_resolver,
                    labeled.ty,
                )?;
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
        TypeExprSpec::Unknown | TypeExprSpec::SelfType => {
            Err(NativeTypeResolutionError::Unsupported)
        }
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

/// Registers standard declaration surfaces and dispatch signatures for core primitive types.
pub fn register_standard_surfaces(
    store: &mut TypeStore,
    declarations: &DeclarationTypeTable,
    resolver: &dyn crate::types::annotation::TypeResolver,
    current_module: &crate::identity::ModuleId,
    dispatch: &mut crate::dispatch::SurfaceDispatchResolver,
) {
    use crate::dispatch::{CallableParameter, CallableSignature};
    use crate::surface::DeclarationSurface;
    use phalcom_common::selector::{Selector, SelectorSlot};

    let int_decl = resolver.resolve_type_name(current_module, "Int", &[]);
    let float_decl = resolver.resolve_type_name(current_module, "Float", &[]);
    let string_decl = resolver.resolve_type_name(current_module, "String", &[]);
    let bool_decl = resolver.resolve_type_name(current_module, "Bool", &[]);
    let list_decl = resolver.resolve_type_name(current_module, "List", &[]);
    let map_decl = resolver.resolve_type_name(current_module, "Map", &[]);
    let set_decl = resolver.resolve_type_name(current_module, "Set", &[]);
    let obj_decl = resolver.resolve_type_name(current_module, "Object", &[]);

    let t_int = int_decl.as_ref().and_then(|d| declarations.form(d));
    let t_float = float_decl.as_ref().and_then(|d| declarations.form(d));
    let t_string = string_decl.as_ref().and_then(|d| declarations.form(d));
    let t_bool = bool_decl.as_ref().and_then(|d| declarations.form(d));
    let t_unit = store.unit();

    let k_int = t_int.map(|t| TypeKnowledge::known(t, EvidenceAuthority::TrustedNative));
    let k_float = t_float.map(|t| TypeKnowledge::known(t, EvidenceAuthority::TrustedNative));
    let k_string = t_string.map(|t| TypeKnowledge::known(t, EvidenceAuthority::TrustedNative));
    let k_bool = t_bool.map(|t| TypeKnowledge::known(t, EvidenceAuthority::TrustedNative));
    let k_unit = TypeKnowledge::known(t_unit, EvidenceAuthority::TrustedNative);

    let make_binary_sig =
        |op: &str, param_k: TypeKnowledge, ret_k: TypeKnowledge| -> CallableSignature {
            let sel = Selector::method(op, vec![SelectorSlot::Positional]).unwrap();
            let param = CallableParameter::new("other", param_k);
            CallableSignature::new(sel, vec![param], ret_k)
        };

    let make_getter_sig = |name: &str, ret_k: TypeKnowledge| -> CallableSignature {
        let sel = Selector::getter(name).unwrap();
        CallableSignature::new(sel, Vec::new(), ret_k)
    };

    // Int surface
    if let (Some(decl), Some(t_self), Some(k_self)) = (int_decl, t_int, k_int.clone()) {
        let mut surface = DeclarationSurface::new(Some(decl.clone()));
        dispatch.register_type(t_self, decl.clone());

        for op in ["+", "-", "*", "//", "%", "**", "<<", ">>", "&", "|", "^"] {
            surface.add_callable(make_binary_sig(op, k_self.clone(), k_self.clone()));
        }
        if let Some(ref kf) = k_float {
            surface.add_callable(make_binary_sig("/", k_self.clone(), kf.clone()));
        }
        if let Some(ref kb) = k_bool {
            for op in ["==", "!=", "<", "<=", ">", ">="] {
                surface.add_callable(make_binary_sig(op, k_self.clone(), kb.clone()));
            }
        }
        for op in ["+", "-", "~"] {
            surface.add_callable(make_getter_sig(op, k_self.clone()));
        }
        if let Some(ref ks) = k_string {
            surface.add_callable(make_getter_sig("toString", ks.clone()));
        }

        dispatch.register_surface(decl, surface);
    }

    // Float surface
    if let (Some(decl), Some(t_self), Some(k_self)) = (float_decl, t_float, k_float.clone()) {
        let mut surface = DeclarationSurface::new(Some(decl.clone()));
        dispatch.register_type(t_self, decl.clone());

        for op in ["+", "-", "*", "/", "**"] {
            surface.add_callable(make_binary_sig(op, k_self.clone(), k_self.clone()));
        }
        if let Some(ref kb) = k_bool {
            for op in ["==", "!=", "<", "<=", ">", ">="] {
                surface.add_callable(make_binary_sig(op, k_self.clone(), kb.clone()));
            }
        }
        for op in ["+", "-"] {
            surface.add_callable(make_getter_sig(op, k_self.clone()));
        }
        if let Some(ref ks) = k_string {
            surface.add_callable(make_getter_sig("toString", ks.clone()));
        }

        dispatch.register_surface(decl, surface);
    }

    // String surface
    if let (Some(decl), Some(t_self), Some(k_self)) = (string_decl, t_string, k_string.clone()) {
        let mut surface = DeclarationSurface::new(Some(decl.clone()));
        dispatch.register_type(t_self, decl.clone());

        surface.add_callable(make_binary_sig("+", k_self.clone(), k_self.clone()));
        if let Some(ref kb) = k_bool {
            for op in ["==", "!=", "<", "<=", ">", ">="] {
                surface.add_callable(make_binary_sig(op, k_self.clone(), kb.clone()));
            }
        }
        if let Some(ref ki) = k_int {
            surface.add_callable(make_getter_sig("length", ki.clone()));
            surface.add_callable(make_getter_sig("size", ki.clone()));
            let sel = Selector::subscript_get(vec![SelectorSlot::Positional]).unwrap();
            let param = CallableParameter::new("index", ki.clone());
            surface.add_callable(CallableSignature::new(sel, vec![param], k_self.clone()));
        }

        dispatch.register_surface(decl, surface);
    }

    // Bool surface
    if let (Some(decl), Some(t_self), Some(k_self)) = (bool_decl, t_bool, k_bool.clone()) {
        let mut surface = DeclarationSurface::new(Some(decl.clone()));
        dispatch.register_type(t_self, decl.clone());

        surface.add_callable(make_getter_sig("not", k_self.clone()));
        for op in ["==", "!=", "and", "or", "&", "|", "^"] {
            surface.add_callable(make_binary_sig(op, k_self.clone(), k_self.clone()));
        }

        dispatch.register_surface(decl, surface);
    }

    // List surface
    if let Some(decl) = list_decl {
        let mut surface = DeclarationSurface::new(Some(decl.clone()));

        if let Some(ref ki) = k_int {
            surface.add_callable(make_getter_sig("length", ki.clone()));
            surface.add_callable(make_getter_sig("size", ki.clone()));
        }

        let elem_param_k = if let Some(sig) = declarations.generic_signature(&decl) {
            if !sig.parameters.is_empty() {
                let p_form = store.parameter_form(sig.parameters[0]);
                TypeKnowledge::known(p_form, EvidenceAuthority::TrustedNative)
            } else {
                TypeKnowledge::Dynamic(crate::types::evidence::DynamicReason::ExplicitEscape)
            }
        } else {
            TypeKnowledge::Dynamic(crate::types::evidence::DynamicReason::ExplicitEscape)
        };

        let add_sel = Selector::method("add", vec![SelectorSlot::Positional]).unwrap();
        let add_param = CallableParameter::new("elem", elem_param_k);
        surface.add_callable(CallableSignature::new(add_sel, vec![add_param], k_unit.clone()));

        dispatch.register_surface(decl, surface);
    }

    // Map surface
    if let Some(decl) = map_decl {
        let mut surface = DeclarationSurface::new(Some(decl.clone()));

        if let Some(ref ki) = k_int {
            surface.add_callable(make_getter_sig("length", ki.clone()));
            surface.add_callable(make_getter_sig("size", ki.clone()));
        }

        dispatch.register_surface(decl, surface);
    }

    // Set surface
    if let Some(decl) = set_decl {
        let mut surface = DeclarationSurface::new(Some(decl.clone()));

        if let Some(ref ki) = k_int {
            surface.add_callable(make_getter_sig("length", ki.clone()));
            surface.add_callable(make_getter_sig("size", ki.clone()));
        }

        dispatch.register_surface(decl, surface);
    }

    // Object surface
    if let Some(decl) = obj_decl {
        let mut surface = DeclarationSurface::new(Some(decl.clone()));
        if let Some(t_obj) = declarations.form(&decl) {
            dispatch.register_type(t_obj, decl.clone());
        }

        if let Some(ref kb) = k_bool {
            surface.add_callable(make_binary_sig(
                "==",
                TypeKnowledge::Dynamic(crate::types::evidence::DynamicReason::ExplicitEscape),
                kb.clone(),
            ));
            surface.add_callable(make_binary_sig(
                "!=",
                TypeKnowledge::Dynamic(crate::types::evidence::DynamicReason::ExplicitEscape),
                kb.clone(),
            ));
        }
        if let Some(ref ks) = k_string {
            surface.add_callable(make_getter_sig("toString", ks.clone()));
        }

        dispatch.register_surface(decl, surface);
    }
}
