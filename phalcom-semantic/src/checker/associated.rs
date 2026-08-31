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
use crate::types::relation::{TypeHierarchy, is_subtype};
use crate::types::store::{CallableParameterType, CallableType, TypeData, TypeStore};
use phalcom_common::range::SourceRange;
use phalcom_common::selector::{Selector, SelectorBase, SelectorKind, SelectorSlot};
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
    pub family: AssociatedFamilyId,
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
pub fn resolve_associated_owner(ctx: &mut CheckingContext<'_>, typed_owner: &TypedExpression, range: SourceRange) -> Result<AssociatedOwnerResolution, ()> {
    let Some(SemanticDenotation::TypeForm(owner_form)) = typed_owner.denotation.clone() else {
        ctx.emit_diagnostic(SemanticDiagnostic::error_in(
            ctx.current_module.clone(),
            DiagnosticCode::AssociatedOwnerNotTypeForm,
            "associated lookup receiver is not a type form",
            range,
        ));
        return Err(());
    };

    let Some((lookup_owner, supplied_arguments)) = ctx.store.applied_nominal_parts(owner_form) else {
        ctx.emit_diagnostic(SemanticDiagnostic::error_in(
            ctx.current_module.clone(),
            DiagnosticCode::AssociatedOwnerNotDeclarationBacked,
            "associated lookup owner is not a declaration-backed type form",
            range,
        ));
        return Err(());
    };

    let residual_kind = ctx.store.kind_of(owner_form);
    Ok(AssociatedOwnerResolution {
        owner_form,
        lookup_owner,
        supplied_arguments,
        residual_kind,
    })
}

/// Resolves the effective associated family for a given base from the lookup owner,
/// including direct variant families and statically inherited class-side behavior.
pub fn resolve_effective_associated_family(
    ctx: &mut CheckingContext<'_>,
    owner: &AssociatedOwnerResolution,
    base: &SelectorBase,
    range: SourceRange,
) -> Result<EffectiveAssociatedFamily, ()> {
    // 1. Check direct surface on lookup owner (enum variants win directly)
    if let Some(surface) = ctx.associated_surface(&owner.lookup_owner) {
        if let Some(family) = surface.families.get(base) {
            if family.kind == AssociatedFamilyKind::Variant {
                return Ok(EffectiveAssociatedFamily {
                    id: family.id.clone(),
                    lookup_owner: owner.lookup_owner.clone(),
                    kind: family.kind,
                    members: family.members.to_vec(),
                });
            }
        }
    }

    // 2. Multi-hop supertype walk for behavioral class hierarchy
    let mut current_decl = Some(owner.lookup_owner.clone());
    let mut visited = HashSet::new();
    let mut collected_members: Vec<AssociatedMemberId> = Vec::new();
    let mut seen_selectors: HashSet<Selector> = HashSet::new();

    while let Some(decl) = current_decl {
        if !visited.insert(decl.clone()) {
            break;
        }

        // Check associated surface of `decl`
        if let Some(surface) = ctx.associated_surface(&decl) {
            if let Some(family) = surface.families.get(base) {
                for member in family.members.iter() {
                    match member {
                        AssociatedMemberId::Behavioral(callable_id) => {
                            if seen_selectors.insert(callable_id.selector.clone()) {
                                collected_members.push(member.clone());
                            }
                        }
                        AssociatedMemberId::Variant(variant_id) => {
                            if decl == owner.lookup_owner {
                                if seen_selectors.insert(variant_id.selector.clone()) {
                                    collected_members.push(member.clone());
                                }
                            }
                        }
                    }
                }
            }
        }

        // Also check dispatch surfaces for class-side and instance-side methods
        if let Some(surface) = ctx.dispatch.get().get_surface(&decl) {
            for side in [crate::identity::DispatchSide::Class, crate::identity::DispatchSide::Instance] {
                let side_surface = surface.surface(side);
                for (selector, _) in &side_surface.callable_signatures {
                    if &selector.base == base && seen_selectors.insert(selector.clone()) {
                        let callable_id = crate::identity::CallableId::new(decl.clone(), selector.clone(), side);
                        collected_members.push(AssociatedMemberId::Behavioral(callable_id));
                    }
                }
            }
        }

        current_decl = ctx.hierarchy.superclass(&decl).cloned();
    }

    if collected_members.is_empty() {
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
        return Err(());
    }

    let family_id = AssociatedFamilyId::new(owner.lookup_owner.clone(), base.clone());
    Ok(EffectiveAssociatedFamily {
        id: family_id,
        lookup_owner: owner.lookup_owner.clone(),
        kind: AssociatedFamilyKind::Behavioral,
        members: collected_members,
    })
}

/// Projects generic type arguments across class inheritance hops from child to ancestor.
pub fn project_supertype_arguments(
    store: &mut TypeStore,
    hierarchy: &dyn TypeHierarchy,
    start_decl: &DeclarationId,
    start_args: &[TypeId],
    target_decl: &DeclarationId,
) -> Vec<TypeId> {
    if start_decl == target_decl {
        return start_args.to_vec();
    }

    let mut current_decl = start_decl.clone();
    let mut current_args = start_args.to_vec();
    let mut visited = HashSet::new();

    while visited.insert(current_decl.clone()) {
        if &current_decl == target_decl {
            return current_args;
        }
        let Some(template) = hierarchy.supertype_template(&current_decl) else {
            break;
        };

        let mut env = crate::types::environment::TypeEnvironment::new();
        for (idx, &arg) in current_args.iter().enumerate() {
            if let Some(param_id) = store.find_type_parameter_id(&crate::types::parameter::TypeParameterOwner::Declaration(current_decl.clone()), idx as u32) {
                env.bind_param(param_id, arg);
            }
        }

        let specialized_super = TypeView::new(template.supertype, env).materialize(store);
        let Some((next_decl, next_args)) = store.applied_nominal_parts(specialized_super) else {
            break;
        };

        current_decl = next_decl;
        current_args = next_args;
    }

    Vec::new()
}

/// Specializes an associated member against the owner's supplied type arguments,
/// verifying GADT owner compatibility and producing specialized types/signatures.
pub fn specialize_associated_member(
    ctx: &mut CheckingContext<'_>,
    owner: &AssociatedOwnerResolution,
    member_id: &AssociatedMemberId,
    range: SourceRange,
) -> Result<SpecializedAssociatedMember, ()> {
    match member_id {
        AssociatedMemberId::Variant(variant_id) => {
            let Some(variant_info) = ctx.variant_info(variant_id).cloned() else {
                ctx.emit_diagnostic(SemanticDiagnostic::error_in(
                    ctx.current_module.clone(),
                    DiagnosticCode::AssociatedMemberMissing,
                    format!("variant member `{}` not found", variant_id.selector),
                    range,
                ));
                return Err(());
            };

            // Build TypeEnvironment from owner.supplied_arguments
            let mut env = crate::types::environment::TypeEnvironment::new();
            for (idx, &arg) in owner.supplied_arguments.iter().enumerate() {
                if let Some(param_id) = ctx.store.find_type_parameter_id(
                    &crate::types::parameter::TypeParameterOwner::Declaration(owner.lookup_owner.clone()),
                    idx as u32,
                ) {
                    env.bind_param(param_id, arg);
                }
            }

            // GADT verification: check if owner supplied arguments conflict with variant GADT constraints
            for (param_id, &constrained_ty) in &variant_info.case_environment.bindings {
                if let Some(supplied) = env.get_param(*param_id) {
                    let matches = is_subtype(ctx.store, ctx.hierarchy.inner(), supplied, constrained_ty)
                        && is_subtype(ctx.store, ctx.hierarchy.inner(), constrained_ty, supplied);

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
                        return Err(());
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
                let parameters: Vec<CallableParameterType> = constructor
                    .parameters
                    .iter()
                    .map(|p| {
                        let p_ty = p.declared_type.canonical_type().unwrap_or_else(|| ctx.store.unit());
                        let ty = TypeView::new(p_ty, env.clone()).materialize(ctx.store);
                        CallableParameterType {
                            label: p.external_label.clone(),
                            ty,
                            rest: phalcom_ast::ast::RestMode::None,
                        }
                    })
                    .collect();

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
                Err(())
            }
        }
        AssociatedMemberId::Behavioral(callable_id) => {
            let defining_decl = callable_id.declaration_owner();
            let defining_args = project_supertype_arguments(ctx.store, ctx.hierarchy.inner(), &owner.lookup_owner, &owner.supplied_arguments, defining_decl);

            let mut env = crate::types::environment::TypeEnvironment::new();
            for (idx, &arg) in defining_args.iter().enumerate() {
                if let Some(param_id) = ctx
                    .store
                    .find_type_parameter_id(&crate::types::parameter::TypeParameterOwner::Declaration(defining_decl.clone()), idx as u32)
                {
                    env.bind_param(param_id, arg);
                }
            }

            let Some(surface) = ctx.dispatch.get().get_surface(defining_decl).cloned() else {
                ctx.emit_diagnostic(SemanticDiagnostic::error_in(
                    ctx.current_module.clone(),
                    DiagnosticCode::AssociatedMemberMissing,
                    format!("callable `{}` surface missing", callable_id.selector),
                    range,
                ));
                return Err(());
            };
            let class_surface = surface.surface(callable_id.side);
            let Some(signature) = class_surface.callable_signatures.get(&callable_id.selector).cloned() else {
                ctx.emit_diagnostic(SemanticDiagnostic::error_in(
                    ctx.current_module.clone(),
                    DiagnosticCode::AssociatedMemberMissing,
                    format!("callable `{}` signature missing", callable_id.selector),
                    range,
                ));
                return Err(());
            };

            let object_decl = crate::identity::DeclarationId::new(crate::identity::ModuleId::core(), "Object".into());
            let object_ty = ctx.store.nominal(object_decl);
            let return_ty = signature.return_type.ty().unwrap_or(object_ty);
            let specialized_return = TypeView::new(return_ty, env.clone()).materialize(ctx.store);

            let parameters: Vec<CallableParameterType> = signature
                .parameters
                .iter()
                .map(|p| {
                    let param_ty = p.ty.ty().unwrap_or(object_ty);
                    let specialized_param = TypeView::new(param_ty, env.clone()).materialize(ctx.store);
                    CallableParameterType {
                        label: p.external_label.clone().map(Into::into),
                        ty: specialized_param,
                        rest: p.rest,
                    }
                })
                .collect();

            let callable_ty = ctx.store.callable(CallableType {
                parameters: parameters.into_boxed_slice(),
                return_type: specialized_return,
            });

            let slots: Vec<SelectorSlot> = signature
                .parameters
                .iter()
                .map(|p| match &p.external_label {
                    Some(label) => SelectorSlot::Label(label.clone()),
                    None => SelectorSlot::Positional,
                })
                .collect();

            let operation = FamilyOperationShape::new(callable_id.selector.kind, slots.into_boxed_slice());

            Ok(SpecializedAssociatedMember {
                member: member_id.clone(),
                operation,
                value_type: callable_ty,
                target: Some(InvocationTargetId::Behavioral(callable_id.clone())),
            })
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
) -> Result<TypeId, ()> {
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
        return Err(());
    }

    Ok(resolved_type)
}
