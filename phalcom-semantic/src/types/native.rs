//! Normalization of rich native metadata specifications into canonical semantic types.

use super::application::TypeApplicationError;
use super::evidence::{EvidenceOrigin, TypeKnowledge, UnknownReason};
use super::id::TypeId;
use super::store::{TupleTypeElement, TypeStore};
use crate::declarations::DeclarationTypeTable;
use crate::identity::DeclarationId;
use crate::types::parameter::{GenericConstraint, GenericSignature, TypeParameterData, TypeParameterOwner, TypeTerm};
use phalcom_common::selector::Selector;
use phalcom_native_meta::types::TypeExprSpec;
use phalcom_native_meta::universe::UniverseKey;
use phalcom_native_meta::{GenericConstraintSpec, NativeDispatch, PrimitiveKey, ReturnFlowSpec};
use phalcom_native_surface::{NATIVE_SURFACES, NativeCatalogFingerprint, NativeSurfaceId, NativeSurfaceRecord, catalog_fingerprint, catalog_fingerprint_for};
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
                        let object_decl = universe_resolver(UniverseKey::Object);
                        let Some(top) = declarations.form(&object_decl) else {
                            return Err(NativeTypeResolutionError::MissingDeclaration(object_decl));
                        };
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
        Ok(form) if store.is_proper_type(form) => TypeKnowledge::established(form, EvidenceOrigin::NativeSignature),
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
    if matches!(spec, TypeExprSpec::SelfType) {
        let decl = universe_resolver(key.owner);
        let side = match key.side {
            NativeDispatch::Instance => crate::identity::DispatchSide::Instance,
            NativeDispatch::Class => crate::identity::DispatchSide::Class,
        };
        let self_ty = store.self_type(crate::types::parameter::SelfTypeTerm {
            owner: decl,
            side,
            role: crate::types::parameter::SelfRole::InstanceType,
        });
        return Ok(TypeKnowledge::established(self_ty, EvidenceOrigin::NativeSignature));
    }
    if matches!(spec, TypeExprSpec::Unknown) {
        return Ok(TypeKnowledge::Unknown(UnknownReason::OpaqueNative));
    }
    let form = resolve_native_type_form(store, declarations, parameters, universe_resolver, spec)
        .map_err(|source| NativeSurfaceImportError::TypeLowering { key, source })?;
    if !store.is_proper_type(form) {
        return Err(NativeSurfaceImportError::TypeLowering {
            key,
            source: NativeTypeResolutionError::Unsupported,
        });
    }
    Ok(TypeKnowledge::established(form, EvidenceOrigin::NativeSignature))
}

fn import_native_generic_signature(
    store: &mut TypeStore,
    declarations: &DeclarationTypeTable,
    universe_resolver: &dyn Fn(UniverseKey) -> DeclarationId,
    key: PrimitiveKey,
    callable: &crate::identity::CallableId,
    spec: &phalcom_native_meta::CallableTypeSpec,
) -> Result<(Option<GenericSignature>, HashMap<&'static str, TypeId>), NativeSurfaceImportError> {
    if spec.type_params.is_empty() && spec.constraints.is_empty() {
        return Ok((None, HashMap::new()));
    }

    let owner = TypeParameterOwner::Callable(callable.clone());
    let mut parameter_ids = Vec::with_capacity(spec.type_params.len());
    let mut parameter_forms = HashMap::with_capacity(spec.type_params.len());
    let mut parameter_kinds = Vec::with_capacity(spec.type_params.len());
    for (index, parameter) in spec.type_params.iter().enumerate() {
        let kind = crate::declarations::lower_kind_spec(store, &parameter.kind);
        let parameter_id = store.intern_type_parameter(TypeParameterData::new(owner.clone(), index as u32, parameter.name, kind));
        let form = store.parameter_form(parameter_id);
        parameter_ids.push(parameter_id);
        parameter_kinds.push(kind);
        parameter_forms.insert(parameter.name, form);
    }

    let mut constraints = Vec::with_capacity(spec.constraints.len());
    for constraint in spec.constraints {
        let (relation, left, right) = match constraint {
            GenericConstraintSpec::Subtype { lower, upper } => (0_u8, *lower, *upper),
            GenericConstraintSpec::Equivalent { left, right } => (1_u8, *left, *right),
        };
        let lower = resolve_native_type_form(store, declarations, &parameter_forms, universe_resolver, left)
            .map_err(|source| NativeSurfaceImportError::TypeLowering { key, source })?;
        let upper = resolve_native_type_form(store, declarations, &parameter_forms, universe_resolver, right)
            .map_err(|source| NativeSurfaceImportError::TypeLowering { key, source })?;
        constraints.push(if relation == 0 {
            GenericConstraint::Subtype {
                lower: TypeTerm::Canonical(lower),
                upper: TypeTerm::Canonical(upper),
            }
        } else {
            GenericConstraint::Equivalent {
                left: TypeTerm::Canonical(lower),
                right: TypeTerm::Canonical(upper),
            }
        });
    }

    let constraint_shapes = constraints
        .iter()
        .map(|constraint| match constraint {
            GenericConstraint::Subtype { lower, upper } => format!("Subtype({}, {})", format_native_term(store, lower), format_native_term(store, upper)),
            GenericConstraint::Equivalent { left, right } => {
                format!("Equivalent({}, {})", format_native_term(store, left), format_native_term(store, right))
            }
        })
        .map(String::into_boxed_str)
        .collect::<Vec<_>>()
        .into_boxed_slice();
    let parameter_kind_shapes = parameter_kinds
        .iter()
        .map(|&kind| store.format_kind(kind).into_boxed_str())
        .collect::<Vec<_>>()
        .into_boxed_slice();
    let signature = GenericSignature::with_constraints(owner, parameter_ids.into_boxed_slice(), constraints.into_boxed_slice())
        .with_parameter_metadata(
            parameter_kinds.into_boxed_slice(),
            vec![crate::types::variance::Variance::Invariant; spec.type_params.len()].into_boxed_slice(),
        )
        .with_parameter_kind_shapes(parameter_kind_shapes)
        .with_constraint_shapes(constraint_shapes);
    signature
        .validate_publishable(store)
        .map_err(|error| NativeSurfaceImportError::UnsupportedMetadata {
            key,
            reason: format!("generic callable signature is not publishable: {error:?}"),
        })?;

    Ok((Some(signature), parameter_forms))
}

fn format_native_term(store: &TypeStore, term: &TypeTerm) -> String {
    match term {
        TypeTerm::Canonical(ty) => store.format_type(*ty),
        TypeTerm::SelfType(self_term) => format!("Self<{:?}:{:?}:{:?}>", self_term.owner, self_term.side, self_term.role),
        TypeTerm::Infer(variable) => format!("Infer<{variable:?}>"),
    }
}

/// Registers declaration surfaces and dispatch signatures dynamically from the canonical native surface catalog.
pub fn register_native_surfaces(
    store: &mut TypeStore,
    declarations: &DeclarationTypeTable,
    _resolver: &dyn crate::types::annotation::TypeResolver,
    _current_module: &crate::identity::ModuleId,
    dispatch: &mut crate::dispatch::SurfaceDispatchResolver,
) -> Result<NativeSurfaceImportReport, NativeSurfaceImportError> {
    register_native_surfaces_from_records(NATIVE_SURFACES, store, declarations, dispatch)
}

/// Registers an explicit native record slice. The catalog-backed wrapper is
/// used by production bootstrap; this entry point keeps import behavior
/// testable for native/generated parity without adding a second semantic
/// registration path.
pub fn register_native_surfaces_from_records(
    native_records: &[NativeSurfaceRecord],
    store: &mut TypeStore,
    declarations: &DeclarationTypeTable,
    dispatch: &mut crate::dispatch::SurfaceDispatchResolver,
) -> Result<NativeSurfaceImportReport, NativeSurfaceImportError> {
    use crate::surface::DeclarationSurface;

    let universe_resolver = |key: UniverseKey| -> DeclarationId { crate::core_surface::universe_declaration(key) };

    let mut surfaces_by_decl: HashMap<DeclarationId, DeclarationSurface> = HashMap::new();
    let mut report = NativeSurfaceImportReport {
        fingerprint: catalog_fingerprint_for(native_records),
        ..NativeSurfaceImportReport::default()
    };
    let mut records: Vec<_> = native_records.iter().collect();
    records.sort_by_key(|record| record.surface.key.sort_key());

    for record in records {
        if record.surface.anchor == phalcom_native_meta::NativeAnchorPolicy::Hidden {
            continue;
        }

        let decl = crate::core_surface::universe_declaration(record.owner());
        if declarations.form(&decl).is_none() {
            return Err(NativeSurfaceImportError::OwnerMissing { key: record.surface.key });
        }

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
        let callable_spec = record.callable();

        let declared_arity = callable_spec.params.positional.len() + callable_spec.params.labeled.len();
        let selector_arity = match selector.kind {
            phalcom_common::selector::SelectorKind::Getter => 0,
            phalcom_common::selector::SelectorKind::Setter => 1,
            phalcom_common::selector::SelectorKind::Method | phalcom_common::selector::SelectorKind::SubscriptGet => selector.slots.len(),
            phalcom_common::selector::SelectorKind::SubscriptSet => selector.slots.len() + 1,
        };
        if declared_arity != selector_arity && callable_spec.params.rest.is_none() {
            return Err(NativeSurfaceImportError::SelectorArityMismatch {
                key: record.surface.key,
                details: format!("selector has {selector_arity} slots but metadata has {declared_arity} parameters"),
            });
        }

        let callable_id = crate::identity::CallableId::new(decl.clone(), selector.clone(), side);
        let (generic_signature, parameter_forms) =
            import_native_generic_signature(store, declarations, &universe_resolver, record.surface.key, &callable_id, callable_spec)?;
        let mut parameters = Vec::new();

        for (index, p_spec) in callable_spec.params.positional.iter().enumerate() {
            let knowledge = import_native_type(store, declarations, &parameter_forms, &universe_resolver, record.surface.key, p_spec)?;
            let name = if index == 0 { "other" } else { "arg" };
            parameters.push(crate::signature::CallableParameterSemantic::new(
                crate::identity::CallableParameterId::new(callable_id.clone(), index as u32),
                name,
                crate::declaration_type::DeclaredTypeFact::from_knowledge_with_basis(&knowledge, crate::declaration_type::DeclaredTypeBasis::NativeSignature),
            ));
        }

        for labeled in callable_spec.params.labeled {
            let knowledge = import_native_type(store, declarations, &parameter_forms, &universe_resolver, record.surface.key, labeled.ty)?;
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
                .with_label(labeled.label),
            );
        }

        if let Some(rest) = callable_spec.params.rest {
            let knowledge = rest
                .ty
                .map(|ty| import_native_type(store, declarations, &parameter_forms, &universe_resolver, record.surface.key, ty))
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

        let return_knowledge =
            match record.flow() {
                ReturnFlowSpec::Receiver => {
                    let self_ty = store.self_type(crate::types::parameter::SelfTypeTerm {
                        owner: decl.clone(),
                        side,
                        role: crate::types::parameter::SelfRole::InstanceType,
                    });
                    TypeKnowledge::established(self_ty, EvidenceOrigin::NativeSignature)
                }
                ReturnFlowSpec::Never => TypeKnowledge::established(store.never(), EvidenceOrigin::NativeSignature),
                ReturnFlowSpec::Argument(index) => parameters.get(index).map(|parameter| parameter.declared_type.to_knowledge()).ok_or_else(|| {
                    NativeSurfaceImportError::UnsupportedMetadata {
                        key: record.surface.key,
                        reason: format!("return flow references missing parameter {index}"),
                    }
                })?,
                _ => import_native_type(
                    store,
                    declarations,
                    &parameter_forms,
                    &universe_resolver,
                    record.surface.key,
                    callable_spec.return_type,
                )?,
            };

        let canonical_signature = crate::signature::CallableSemanticSignature {
            callable: callable_id.clone(),
            owner: decl.clone(),
            side,
            selector: selector.clone(),
            generics: generic_signature,
            parameters: parameters.into_boxed_slice(),
            declared_return: crate::declaration_type::DeclaredTypeFact::from_knowledge_with_basis(
                &return_knowledge,
                crate::declaration_type::DeclaredTypeBasis::NativeSignature,
            ),
            return_validation: crate::signature::ReturnContractValidation::NotApplicable,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::declarations::bootstrap_universe_declarations;
    use crate::dispatch::SurfaceDispatchResolver;
    use crate::types::parameter::TypeParameterOwner;
    use crate::types::store::TypeStore;
    use phalcom_native_meta::{
        CallableTypeSpec, GenericConstraintSpec, KindSpec, NativeAnchorPolicy, NativeLifecycleSpec, NativeStability, NativeTrust, PrimitiveAbi,
        PrimitiveSurfaceSpec, TypeParameterSpec,
    };
    use phalcom_native_surface::{NativeMemberKind, NativeReturnShape, catalog_fingerprint_for};

    static GENERIC_PARAMETER_TYPE: TypeExprSpec = TypeExprSpec::Parameter("T");
    static OBJECT_TYPE: TypeExprSpec = TypeExprSpec::Universe(UniverseKey::Object);
    static GENERIC_TYPE_PARAMETER: TypeParameterSpec = TypeParameterSpec {
        name: "T",
        kind: KindSpec::Type,
    };
    static GENERIC_PARAMETERS: phalcom_native_meta::ParameterTupleSpec = phalcom_native_meta::ParameterTupleSpec {
        positional: &[GENERIC_PARAMETER_TYPE],
        labeled: &[],
        rest: None,
    };
    static GENERIC_CONSTRAINTS: [GenericConstraintSpec; 1] = [GenericConstraintSpec::Subtype {
        lower: &GENERIC_PARAMETER_TYPE,
        upper: &OBJECT_TYPE,
    }];
    static GENERIC_CALLABLE: CallableTypeSpec = CallableTypeSpec {
        type_params: &[GENERIC_TYPE_PARAMETER],
        params: &GENERIC_PARAMETERS,
        return_type: &GENERIC_PARAMETER_TYPE,
        constraints: &GENERIC_CONSTRAINTS,
    };
    static GENERIC_RECORD: NativeSurfaceRecord = NativeSurfaceRecord {
        surface: PrimitiveSurfaceSpec {
            key: PrimitiveKey {
                owner: UniverseKey::Object,
                side: NativeDispatch::Instance,
                selector: "identity(_)",
            },
            visibility: phalcom_native_meta::NativeVisibility::Public,
            stability: NativeStability::Stable,
            anchor: NativeAnchorPolicy::Required,
            params: &GENERIC_PARAMETERS,
            returns: &GENERIC_PARAMETER_TYPE,
            callable: &GENERIC_CALLABLE,
            raises: phalcom_native_meta::RaisesSpec::Unknown,
            effects: phalcom_native_meta::EffectSpec::Pure,
            flow: ReturnFlowSpec::Value,
            termination: phalcom_native_meta::TerminationSpec::Unknown,
            since: None,
            deprecated_since: None,
            replacement: None,
            lifecycle: NativeLifecycleSpec::UNKNOWN,
            intrinsic: None,
            trust: NativeTrust::Ordinary,
            docs: None,
            conceptual: None,
        },
        kind: NativeMemberKind::Method,
        abi: PrimitiveAbi::Value,
        return_shape: NativeReturnShape::Unknown,
    };

    #[test]
    fn generic_native_surface_imports_callable_owned_signature_and_constraints() {
        let mut store = TypeStore::new();
        let resolver = crate::core_surface::universe_declaration;
        let declarations = bootstrap_universe_declarations(&mut store, &resolver);
        let mut dispatch = SurfaceDispatchResolver::new();
        let report = register_native_surfaces_from_records(&[GENERIC_RECORD], &mut store, &declarations, &mut dispatch).expect("native import");

        let (callable, signature) = report.callable_signatures.first().expect("imported callable");
        let generics = signature.generics.as_ref().expect("generic native signature");
        assert_eq!(generics.owner, TypeParameterOwner::Callable(callable.clone()));
        assert_eq!(generics.parameters.len(), 1);
        assert_eq!(generics.constraints.len(), 1);
        assert_eq!(store.type_parameter(generics.parameters[0]).owner, generics.owner);
        assert_eq!(
            signature.parameters[0].declared_type.to_knowledge().ty(),
            signature.declared_return.to_knowledge().ty()
        );
        assert_eq!(report.fingerprint, catalog_fingerprint_for(&[GENERIC_RECORD]));
        assert_eq!(callable.owner, crate::core_surface::universe_declaration(UniverseKey::Object));
    }
}
