//! Normalization of rich native metadata specifications into canonical semantic types.

use super::evidence::{EvidenceAuthority, TypeEvidence, TypeKnowledge, UnknownReason};
use super::store::TypeStore;
use crate::identity::DeclarationId;
use phalcom_native_meta::types::TypeExprSpec;
use phalcom_native_meta::universe::UniverseKey;

/// Normalizes a native [`TypeExprSpec`] into canonical [`TypeKnowledge`].
pub fn normalize_native_type(store: &mut TypeStore, universe_resolver: &dyn Fn(UniverseKey) -> DeclarationId, spec: &TypeExprSpec) -> TypeKnowledge {
    match spec {
        TypeExprSpec::Never => TypeKnowledge::Known(TypeEvidence {
            ty: store.never(),
            authority: EvidenceAuthority::TrustedNative,
            provenance: Default::default(),
        }),
        TypeExprSpec::Universe(UniverseKey::Unit) => TypeKnowledge::Known(TypeEvidence {
            ty: store.unit(),
            authority: EvidenceAuthority::TrustedNative,
            provenance: Default::default(),
        }),
        TypeExprSpec::Universe(key) => {
            let decl = universe_resolver(*key);
            let ty = store.nominal(decl);
            TypeKnowledge::Known(TypeEvidence {
                ty,
                authority: EvidenceAuthority::TrustedNative,
                provenance: Default::default(),
            })
        }
        TypeExprSpec::Union(members) => {
            let mut tys = Vec::new();
            for m in *members {
                let k = normalize_native_type(store, universe_resolver, m);
                if let Some(ty) = k.ty() {
                    tys.push(ty);
                } else {
                    return TypeKnowledge::Unknown(UnknownReason::OpaqueNative);
                }
            }
            let union_ty = store.union(&tys);
            TypeKnowledge::Known(TypeEvidence {
                ty: union_ty,
                authority: EvidenceAuthority::TrustedNative,
                provenance: Default::default(),
            })
        }
        TypeExprSpec::Unknown => TypeKnowledge::Unknown(UnknownReason::OpaqueNative),
        _ => TypeKnowledge::Unknown(UnknownReason::OpaqueNative),
    }
}

/// Registers standard declaration surfaces and dispatch signatures for core primitive types.
pub fn register_standard_surfaces(
    store: &mut TypeStore,
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

    let t_int = int_decl.as_ref().map(|d| store.nominal(d.clone()));
    let t_float = float_decl.as_ref().map(|d| store.nominal(d.clone()));
    let t_string = string_decl.as_ref().map(|d| store.nominal(d.clone()));
    let t_bool = bool_decl.as_ref().map(|d| store.nominal(d.clone()));
    let t_unit = store.unit();

    let k_int = t_int.map(|t| TypeKnowledge::known(t, EvidenceAuthority::TrustedNative));
    let k_float = t_float.map(|t| TypeKnowledge::known(t, EvidenceAuthority::TrustedNative));
    let k_string = t_string.map(|t| TypeKnowledge::known(t, EvidenceAuthority::TrustedNative));
    let k_bool = t_bool.map(|t| TypeKnowledge::known(t, EvidenceAuthority::TrustedNative));
    let k_unit = TypeKnowledge::known(t_unit, EvidenceAuthority::TrustedNative);

    let make_binary_sig = |op: &str, param_k: TypeKnowledge, ret_k: TypeKnowledge| -> CallableSignature {
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
        let t_list = store.nominal(decl.clone());
        dispatch.register_type(t_list, decl.clone());

        if let Some(ref ki) = k_int {
            surface.add_callable(make_getter_sig("length", ki.clone()));
            surface.add_callable(make_getter_sig("size", ki.clone()));
        }
        let add_sel = Selector::method("add", vec![SelectorSlot::Positional]).unwrap();
        let add_param = CallableParameter::new("elem", TypeKnowledge::Dynamic(crate::types::evidence::DynamicReason::ExplicitEscape));
        surface.add_callable(CallableSignature::new(add_sel, vec![add_param], k_unit.clone()));

        dispatch.register_surface(decl, surface);
    }

    // Map surface
    if let Some(decl) = map_decl {
        let mut surface = DeclarationSurface::new(Some(decl.clone()));
        let t_map = store.nominal(decl.clone());
        dispatch.register_type(t_map, decl.clone());

        if let Some(ref ki) = k_int {
            surface.add_callable(make_getter_sig("length", ki.clone()));
            surface.add_callable(make_getter_sig("size", ki.clone()));
        }

        dispatch.register_surface(decl, surface);
    }

    // Set surface
    if let Some(decl) = set_decl {
        let mut surface = DeclarationSurface::new(Some(decl.clone()));
        let t_set = store.nominal(decl.clone());
        dispatch.register_type(t_set, decl.clone());

        if let Some(ref ki) = k_int {
            surface.add_callable(make_getter_sig("length", ki.clone()));
            surface.add_callable(make_getter_sig("size", ki.clone()));
        }

        dispatch.register_surface(decl, surface);
    }

    // Object surface
    if let Some(decl) = obj_decl {
        let mut surface = DeclarationSurface::new(Some(decl.clone()));
        let t_obj = store.nominal(decl.clone());
        dispatch.register_type(t_obj, decl.clone());

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
