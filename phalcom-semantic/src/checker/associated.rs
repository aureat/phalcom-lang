//! Associated resolution and family application models (Part 3).

use crate::associated::{AssociatedFamilyKind, AssociatedMemberId};
use crate::checker::context::CheckingContext;
use crate::checker::expected::ExpectedType;
use crate::checker::typed_expr::TypedExpression;
use crate::diagnostic::{DiagnosticCode, SemanticDiagnostic};
use crate::identity::{AssociatedFamilyId, DeclarationId, ExpressionId, InvocationTargetId};
use crate::types::denotation::SemanticDenotation;
use crate::types::environment::TypeView;
use crate::types::family::FamilyOperationShape;
use crate::types::id::{KindId, TypeId};
use crate::types::relation::is_subtype;
use crate::types::store::{CallableParameterType, CallableType, TypeData, TypeStore};
use phalcom_common::range::SourceRange;
use phalcom_common::selector::{Selector, SelectorBase, SelectorKind, SelectorPattern, SelectorSlot};
use std::collections::{BTreeMap, HashSet};

/// A specialized member belonging to an associated family or lookup outcome.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SpecializedAssociatedMember {
    pub member: AssociatedMemberId,
    pub operation: FamilyOperationShape,
    pub value_type: TypeId,
    pub target: Option<InvocationTargetId>,
}

/// Resolved semantic outcome for an associated lookup or direct invocation expression.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AssociatedResolution {
    pub owner_form: TypeId,
    pub lookup_owner: DeclarationId,
    /// `Some` only when lookup won in the declaration-owned associated
    /// namespace. Ordinary `::` behavior has no associated-family identity.
    pub family: Option<AssociatedFamilyId>,
    pub kind: AssociatedResolutionKind,
}

/// Specific variant of an associated resolution.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AssociatedResolutionKind {
    ExactValue {
        member: AssociatedMemberId,
        value_type: TypeId,
    },
    ExactCallable {
        member: AssociatedMemberId,
        target: InvocationTargetId,
        callable_type: TypeId,
    },
    Family {
        family_type: TypeId,
        members: Box<[SpecializedAssociatedMember]>,
    },
    StaticInvoke {
        member: AssociatedMemberId,
        target: InvocationTargetId,
        result_type: TypeId,
    },
    DynamicInvoke {
        candidates: Box<[SpecializedAssociatedMember]>,
        result_type: Option<TypeId>,
    },
    /// Ordinary receiver-bound `::` family/reference resolution. Kept as a
    /// distinct variant so lowering never infers namespace ownership from a
    /// callable side or from an `AssociatedMemberId`.
    BoundBehavioralFamily {
        family_type: TypeId,
        spec: BehavioralFamilySpec,
        members: Box<[BoundBehavioralMember]>,
    },
    /// Ordinary receiver-bound direct `::` invocation.
    BoundBehavioralInvoke {
        target: InvocationTargetId,
        result_type: TypeId,
    },
}

/// Source-independent selector specification for an ordinary bound family.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum BehavioralFamilySpec {
    Exact(Selector),
    Pattern(SelectorPattern),
}

/// One statically known ordinary behavior candidate retained by a bound
/// family. Runtime still dispatches on the captured receiver.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct BoundBehavioralMember {
    pub operation: FamilyOperationShape,
    pub member_kind: crate::types::family::FamilyMemberTypeKind,
    pub target: InvocationTargetId,
    pub callable_type: TypeId,
}

/// Semantic resolution product for an ordinary invocation on a first-class family value.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FamilyApplicationResolution {
    pub family_type: TypeId,
    pub selection: FamilyApplicationSelection,
}

/// Specific member selection for an ordinary family-value invocation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FamilyApplicationSelection {
    Static {
        operation: FamilyOperationShape,
        target: Option<InvocationTargetId>,
        callable_type: TypeId,
        result_type: TypeId,
    },
    Dynamic {
        candidates: Box<[FamilyApplicationCandidate]>,
        result_type: Option<TypeId>,
    },
}

/// Candidate operation/member for deferred/dynamic shape selection on a family value.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FamilyApplicationCandidate {
    pub operation: FamilyOperationShape,
    pub target: Option<InvocationTargetId>,
    pub callable_type: TypeId,
}

/// Body-local index of associated syntax resolutions.
pub type AssociatedResolutionIndex = BTreeMap<ExpressionId, AssociatedResolution>;

/// Body-local index of ordinary family value application resolutions.
pub type FamilyApplicationResolutionIndex = BTreeMap<ExpressionId, FamilyApplicationResolution>;

/// Failure after a diagnostic has been emitted for an associated lookup.
/// Keeping this as a concrete error preserves the fail-closed API without
/// returning a unit error that loses the semantic failure category.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AssociatedResolutionError;

/// Normalized resolution of an associated owner type form.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AssociatedOwnerResolution {
    pub owner_form: TypeId,
    pub lookup_owner: DeclarationId,
    pub supplied_arguments: Vec<TypeId>,
    pub residual_kind: KindId,
}

/// A statically resolved effective associated family.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EffectiveAssociatedFamily {
    pub id: AssociatedFamilyId,
    pub lookup_owner: DeclarationId,
    pub kind: AssociatedFamilyKind,
    pub members: Vec<AssociatedMemberId>,
}

/// Resolves an associated owner expression, validating its type-form denotation
/// and recovering its root declaration, supplied arguments, and residual kind.
pub fn resolve_associated_owner(
    ctx: &mut CheckingContext<'_>,
    typed_owner: &TypedExpression,
    range: SourceRange,
) -> Result<AssociatedOwnerResolution, AssociatedResolutionError> {
    let Some(SemanticDenotation::TypeForm(owner_form)) = typed_owner.denotation.clone() else {
        ctx.emit_diagnostic(SemanticDiagnostic::error_in(
            ctx.current_module.clone(),
            DiagnosticCode::AssociatedOwnerNotTypeForm,
            "associated lookup receiver is not a type form",
            range,
        ));
        return Err(AssociatedResolutionError);
    };

    let Some((lookup_owner, supplied_arguments)) = ctx.store.applied_nominal_parts(owner_form) else {
        ctx.emit_diagnostic(SemanticDiagnostic::error_in(
            ctx.current_module.clone(),
            DiagnosticCode::AssociatedOwnerNotDeclarationBacked,
            "associated lookup owner is not a declaration-backed type form",
            range,
        ));
        return Err(AssociatedResolutionError);
    };

    let residual_kind = ctx.store.kind_of(owner_form);
    Ok(AssociatedOwnerResolution {
        owner_form,
        lookup_owner,
        supplied_arguments,
        residual_kind,
    })
}

/// Resolves the effective associated family for a given base from the lookup owner.
pub fn resolve_effective_associated_family(
    ctx: &mut CheckingContext<'_>,
    owner: &AssociatedOwnerResolution,
    base: &SelectorBase,
    range: SourceRange,
) -> Result<EffectiveAssociatedFamily, AssociatedResolutionError> {
    if let Some(surface) = ctx.associated_surface(&owner.lookup_owner) {
        if let Some(family) = surface.families.get(base) {
            return Ok(EffectiveAssociatedFamily {
                id: family.id.clone(),
                lookup_owner: owner.lookup_owner.clone(),
                kind: family.kind,
                members: family.members.to_vec(),
            });
        }
    }

    let base_str = match base {
        SelectorBase::Named(name) => name.as_str(),
        SelectorBase::Subscript => "[]",
    };
    ctx.emit_diagnostic(SemanticDiagnostic::error_in(
        ctx.current_module.clone(),
        DiagnosticCode::AssociatedFamilyMissing,
        format!("no associated family for base `{}` on `{}`", base_str, owner.lookup_owner.name),
        range,
    ));
    Err(AssociatedResolutionError)
}

/// Resolves an ordinary receiver-bound `::` family without consulting the
/// declaration-owned associated namespace. Candidate discovery is static, but
/// each target remains a behavioral selector dispatched against the captured
/// receiver at runtime.
pub fn resolve_bound_behavioral_family(
    ctx: &mut CheckingContext<'_>,
    receiver_type: TypeId,
    lookup: crate::dispatch::DispatchLookup,
    spec: BehavioralFamilySpec,
    range: SourceRange,
) -> Result<(DeclarationId, TypeId, Vec<BoundBehavioralMember>), AssociatedResolutionError> {
    let Some((lookup_owner, side)) = ctx.dispatch_owner_for_lookup(receiver_type, lookup.clone()) else {
        ctx.emit_diagnostic(SemanticDiagnostic::error_in(
            ctx.current_module.clone(),
            DiagnosticCode::AssociatedOwnerUnresolved,
            "bound `::` receiver has no dispatch owner",
            range,
        ));
        return Err(AssociatedResolutionError);
    };

    let mut members = Vec::new();
    let mut seen = HashSet::new();

    match &spec {
        BehavioralFamilySpec::Exact(selector) => {
            if let crate::dispatch::ResolvedDispatchResult::Found(resolved) = ctx.resolve_dispatch_target(receiver_type, selector, lookup) {
                if let Some(callable_type) = callable_type_from_signature(ctx, &resolved.signature) {
                    let member_kind = if selector.kind == SelectorKind::Getter {
                        crate::types::family::FamilyMemberTypeKind::Value
                    } else {
                        crate::types::family::FamilyMemberTypeKind::Callable
                    };
                    members.push(BoundBehavioralMember {
                        operation: FamilyOperationShape::new(selector.kind, selector.slots.clone()),
                        member_kind,
                        target: InvocationTargetId::Behavioral(resolved.callable),
                        callable_type,
                    });
                }
            }
        }
        BehavioralFamilySpec::Pattern(SelectorPattern { base, .. }) => {
            for owner in ctx.dispatch_ref().dispatch_owners(ctx.hierarchy.inner(), &lookup_owner, side) {
                let candidates = ctx.get_surface(&owner.declaration).map(|surface| {
                    surface
                        .surface(owner.side)
                        .callable_signatures
                        .iter()
                        .filter_map(|(selector, signature)| {
                            surface
                                .get_callable_id(owner.side, selector)
                                .cloned()
                                .map(|callable| (selector.clone(), signature.clone(), callable))
                        })
                        .collect::<Vec<_>>()
                });
                let Some(candidates) = candidates else {
                    continue;
                };
                for (selector, signature, callable) in candidates {
                    if selector.base != *base || !matches!(selector.kind, SelectorKind::Getter | SelectorKind::Setter | SelectorKind::Method) {
                        continue;
                    }
                    if !seen.insert(selector.clone()) {
                        continue;
                    }
                    let Some(callable_type) = callable_type_from_signature(ctx, &signature) else {
                        // A family may still be constructed when declaration
                        // type knowledge is incomplete. Omit only the static
                        // candidate; runtime keeps the live receiver-bound
                        // selector pattern authoritative.
                        continue;
                    };
                    let member_kind = if selector.kind == SelectorKind::Getter {
                        crate::types::family::FamilyMemberTypeKind::Value
                    } else {
                        crate::types::family::FamilyMemberTypeKind::Callable
                    };
                    members.push(BoundBehavioralMember {
                        operation: FamilyOperationShape::new(selector.kind, selector.slots.clone()),
                        member_kind,
                        target: InvocationTargetId::Behavioral(callable),
                        callable_type,
                    });
                }
            }
        }
    }

    let family_members = members.iter().map(|member| match member.member_kind {
        crate::types::family::FamilyMemberTypeKind::Value => crate::types::family::FamilyMemberType::value(member.operation.clone(), member.callable_type),
        crate::types::family::FamilyMemberTypeKind::Callable => {
            crate::types::family::FamilyMemberType::callable(member.operation.clone(), member.callable_type)
        }
    });
    let family_type = ctx.store.family_type(family_members).map_err(|_| AssociatedResolutionError)?;
    Ok((lookup_owner, family_type, members))
}

fn callable_type_from_signature(ctx: &mut CheckingContext<'_>, signature: &crate::dispatch::CallableSignature) -> Option<TypeId> {
    let return_type = signature.return_type.ty()?;
    let parameters = signature
        .parameters
        .iter()
        .map(|parameter| {
            Some(CallableParameterType {
                label: parameter.external_label.clone().map(Into::into),
                ty: parameter.ty.ty()?,
                rest: parameter.rest,
            })
        })
        .collect::<Option<Vec<_>>>()?;
    Some(ctx.store.callable(CallableType {
        parameters: parameters.into_boxed_slice(),
        return_type,
    }))
}

/// Specializes an associated member against the owner's supplied type arguments,
/// verifying GADT owner compatibility and producing specialized types/signatures.
pub fn specialize_associated_member(
    ctx: &mut CheckingContext<'_>,
    owner: &AssociatedOwnerResolution,
    member_id: &AssociatedMemberId,
    range: SourceRange,
) -> Result<SpecializedAssociatedMember, AssociatedResolutionError> {
    match member_id {
        AssociatedMemberId::Variant(variant_id) => {
            let Some(variant_info) = ctx.variant_info(variant_id).cloned() else {
                ctx.emit_diagnostic(SemanticDiagnostic::error_in(
                    ctx.current_module.clone(),
                    DiagnosticCode::AssociatedMemberMissing,
                    format!("variant member `{}` not found", variant_id.selector),
                    range,
                ));
                return Err(AssociatedResolutionError);
            };

            let env =
                crate::types::specialization::specialize_receiver_to_owner(ctx.store, &ctx.hierarchy, owner.owner_form, &owner.lookup_owner, &ctx.control)
                    .map_err(|failure| {
                        ctx.emit_diagnostic(SemanticDiagnostic::error_in(
                            ctx.current_module.clone(),
                            DiagnosticCode::AssociatedOwnerUnresolved,
                            format!("cannot specialize associated owner: {failure:?}"),
                            range,
                        ));
                        AssociatedResolutionError
                    })?
                    .environment;

            // GADT verification: check if owner supplied arguments conflict with variant GADT constraints
            for (param_id, &constrained_ty) in &variant_info.case_environment.bindings {
                if let Some(supplied) = env.get_param(*param_id) {
                    let matches =
                        is_subtype(ctx.store, &ctx.hierarchy, supplied, constrained_ty) && is_subtype(ctx.store, &ctx.hierarchy, constrained_ty, supplied);

                    if !matches {
                        ctx.emit_diagnostic(SemanticDiagnostic::error_in(
                            ctx.current_module.clone(),
                            DiagnosticCode::AssociatedGadtOwnerConflict,
                            format!(
                                "GADT variant `{}` requires type parameter to be `{}` but owner specified `{}`",
                                variant_id.selector,
                                ctx.store.format_type(constrained_ty),
                                ctx.store.format_type(supplied)
                            ),
                            range,
                        ));
                        return Err(AssociatedResolutionError);
                    }
                }
            }

            if variant_info.shape == crate::enum_semantics::VariantShape::Singleton {
                let value_type = TypeView::new(variant_info.exact_case_template, env).materialize(ctx.store);
                let operation = FamilyOperationShape::getter();
                Ok(SpecializedAssociatedMember {
                    member: member_id.clone(),
                    operation,
                    value_type,
                    target: None,
                })
            } else if let Some(constructor) = &variant_info.constructor {
                let constructor_result = TypeView::new(constructor.exact_case_template, env.clone()).materialize(ctx.store);
                let mut parameters = Vec::with_capacity(constructor.parameters.len());
                for parameter in &constructor.parameters {
                    let Some(parameter_type) = parameter.declared_type.canonical_type() else {
                        ctx.emit_diagnostic(SemanticDiagnostic::error_in(
                            ctx.current_module.clone(),
                            DiagnosticCode::AssociatedMemberMissing,
                            format!(
                                "variant `{}` has unresolved constructor parameter `{}`",
                                variant_id.selector, parameter.local_name
                            ),
                            range,
                        ));
                        return Err(AssociatedResolutionError);
                    };
                    let ty = TypeView::new(parameter_type, env.clone()).materialize(ctx.store);
                    parameters.push(CallableParameterType {
                        label: parameter.external_label.clone(),
                        ty,
                        rest: phalcom_ast::ast::RestMode::None,
                    });
                }

                let callable_ty = ctx.store.callable(CallableType {
                    parameters: parameters.into_boxed_slice(),
                    return_type: constructor_result,
                });

                let slots: Vec<SelectorSlot> = constructor
                    .parameters
                    .iter()
                    .map(|p| match &p.external_label {
                        Some(label) => SelectorSlot::Label(label.to_string()),
                        None => SelectorSlot::Positional,
                    })
                    .collect();

                let operation = FamilyOperationShape::new(SelectorKind::Method, slots.into_boxed_slice());

                Ok(SpecializedAssociatedMember {
                    member: member_id.clone(),
                    operation,
                    value_type: callable_ty,
                    target: Some(InvocationTargetId::VariantConstructor(constructor.constructor.clone())),
                })
            } else {
                ctx.emit_diagnostic(SemanticDiagnostic::error_in(
                    ctx.current_module.clone(),
                    DiagnosticCode::AssociatedMemberMissing,
                    format!("variant constructor for `{}` is missing", variant_id.selector),
                    range,
                ));
                Err(AssociatedResolutionError)
            }
        }
    }
}

/// Checks if a type contains any unspecialized declaration or local type parameters.
pub fn contains_any_type_parameter(store: &TypeStore, ty: TypeId) -> bool {
    match store.get(ty) {
        TypeData::Parameter(_) => true,
        TypeData::Applied { origin, arguments } => {
            contains_any_type_parameter(store, *origin) || arguments.iter().any(|&a| contains_any_type_parameter(store, a))
        }
        TypeData::Union(members) => members.iter().any(|&m| contains_any_type_parameter(store, m)),
        TypeData::Tuple(elems) => elems.iter().any(|e| contains_any_type_parameter(store, e.ty)),
        TypeData::Record(row_id) => {
            let row = store.record_row(*row_id);
            row.fields.iter().any(|f| contains_any_type_parameter(store, f.ty))
        }
        TypeData::Callable(call) => {
            call.parameters.iter().any(|p| contains_any_type_parameter(store, p.ty)) || contains_any_type_parameter(store, call.return_type)
        }
        TypeData::Family(fam_id) => {
            let fam = store.get_family(*fam_id);
            fam.members.iter().any(|m| contains_any_type_parameter(store, m.ty))
        }
        TypeData::ExactCase { enum_type, .. } => contains_any_type_parameter(store, *enum_type),
        _ => false,
    }
}

/// Enforces Part 3.5 Option A underconstraint checking for reified associated values.
///
/// If contextual expected type is available, attempts parameter specialization.
/// If unsolved type parameters remain in the value type, emits `AssociatedGenericUnderconstrained`.
pub fn check_reification_underconstrained(
    ctx: &mut CheckingContext<'_>,
    value_type: TypeId,
    expected: &ExpectedType,
    range: SourceRange,
) -> Result<TypeId, AssociatedResolutionError> {
    let mut resolved_type = value_type;
    if let Some(expected_ty) = expected.ty() {
        if let (TypeData::Callable(val_call), TypeData::Callable(exp_call)) = (ctx.store.get(value_type).clone(), ctx.store.get(expected_ty).clone()) {
            let mut env = crate::types::environment::TypeEnvironment::new();
            if let (
                TypeData::Applied {
                    origin: val_orig,
                    arguments: val_args,
                },
                TypeData::Applied {
                    origin: exp_orig,
                    arguments: exp_args,
                },
            ) = (ctx.store.get(val_call.return_type).clone(), ctx.store.get(exp_call.return_type).clone())
            {
                if val_orig == exp_orig && val_args.len() == exp_args.len() {
                    for (v, e) in val_args.iter().zip(exp_args.iter()) {
                        if let TypeData::Parameter(param_id) = ctx.store.get(*v) {
                            env.bind_param(*param_id, *e);
                        }
                    }
                }
            }
            for (vp, ep) in val_call.parameters.iter().zip(exp_call.parameters.iter()) {
                if let TypeData::Parameter(param_id) = ctx.store.get(vp.ty) {
                    env.bind_param(*param_id, ep.ty);
                }
            }
            resolved_type = TypeView::new(value_type, env).materialize(ctx.store);
        } else if let (
            TypeData::ExactCase {
                enum_type: val_enum,
                variant: val_var,
            },
            TypeData::ExactCase {
                enum_type: exp_enum,
                variant: exp_var,
            },
        ) = (ctx.store.get(value_type).clone(), ctx.store.get(expected_ty).clone())
        {
            if val_var == exp_var {
                if let (
                    TypeData::Applied {
                        origin: val_orig,
                        arguments: val_args,
                    },
                    TypeData::Applied {
                        origin: exp_orig,
                        arguments: exp_args,
                    },
                ) = (ctx.store.get(val_enum).clone(), ctx.store.get(exp_enum).clone())
                {
                    if val_orig == exp_orig && val_args.len() == exp_args.len() {
                        let mut env = crate::types::environment::TypeEnvironment::new();
                        for (v, e) in val_args.iter().zip(exp_args.iter()) {
                            if let TypeData::Parameter(param_id) = ctx.store.get(*v) {
                                env.bind_param(*param_id, *e);
                            }
                        }
                        resolved_type = TypeView::new(value_type, env).materialize(ctx.store);
                    }
                }
            }
        }
    }

    if contains_any_type_parameter(ctx.store, resolved_type) {
        ctx.emit_diagnostic(SemanticDiagnostic::error_in(
            ctx.current_module.clone(),
            DiagnosticCode::AssociatedGenericUnderconstrained,
            format!(
                "associated generic value `{}` is underconstrained; specify owner type arguments or provide contextual expected type",
                ctx.store.format_type(resolved_type)
            ),
            range,
        ));
        return Err(AssociatedResolutionError);
    }

    Ok(resolved_type)
}
