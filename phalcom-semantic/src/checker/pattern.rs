use phalcom_ast::ast::{Pattern, VariantPattern, VariantPatternMode};
use phalcom_ast::selector::{selector_from_exact_variant_pattern, selector_pattern_from_variant_pattern};
use phalcom_common::range::SourceRange;
use phalcom_common::selector::{Selector, SelectorBase};

use crate::checker::context::CheckingContext;
use crate::checker::pattern_space::{PatternSpace, VariantSpace};
use crate::enum_semantics::VariantShape;
use crate::identity::{DeclarationId, VariantFamilyId};
use crate::match_semantics::{
    BranchProofEnvironment, PatternBindingResolution, PatternResolution, ResolvedFieldPattern,
    ResolvedListPattern, ResolvedOrPattern, ResolvedVariantCandidate,
    ResolvedVariantPattern, VariantSelectorConstraint,
};
use crate::types::denotation::ValueSemanticFact;
use crate::types::evidence::{EvidenceOrigin, TypeKnowledge, UnknownReason};
use crate::types::id::TypeId;
use crate::types::store::TypeData;

/// Resolves an AST pattern against an expected type and value space.
pub fn resolve_pattern(
    ctx: &mut CheckingContext<'_>,
    pattern: &Pattern,
    expected_ty: TypeId,
    expected_space: &PatternSpace,
    bindings: &mut Vec<PatternBindingResolution>,
) -> (PatternResolution, PatternSpace) {
    match pattern {
        Pattern::Wildcard { .. } => (PatternResolution::Wildcard, expected_space.clone()),
        Pattern::Name { name, range } => {
            // Check if bare identifier matches a contextual singleton variant in expected space
            if let Some((var_res, var_space)) = try_resolve_contextual_singleton(ctx, name, *range, expected_ty, expected_space) {
                (PatternResolution::Variant(var_res), var_space)
            } else {
                let bind_res = ctx.bind_pattern_binding_with_causal(
                    name.clone(),
                    ValueSemanticFact::new(TypeKnowledge::established(expected_ty, EvidenceOrigin::Flow)),
                    *range,
                    crate::checker::causal::CausalInvalidity::Clean,
                );
                let binding_id = match bind_res {
                    crate::checker::binding::BindingDeclarationResult::Inserted(b) => b,
                    crate::checker::binding::BindingDeclarationResult::Redeclared(b) => b,
                };
                let resolution = PatternBindingResolution {
                    binding: binding_id,
                    name: name.as_str().into(),
                    knowledge: TypeKnowledge::established(expected_ty, EvidenceOrigin::Flow),
                    source: *range,
                };
                bindings.push(resolution);
                (
                    PatternResolution::Binding {
                        binding: binding_id,
                        name: name.as_str().into(),
                        knowledge: TypeKnowledge::established(expected_ty, EvidenceOrigin::Flow),
                    },
                    expected_space.clone(),
                )
            }
        }
        Pattern::Variant(variant_pat) => {
            let (res, space) = resolve_variant_pattern(ctx, variant_pat, expected_ty, expected_space, bindings);
            (PatternResolution::Variant(res), space)
        }
        Pattern::Or { alternatives, .. } => {
            let mut resolved_alts = Vec::with_capacity(alternatives.len());
            let mut alt_space = PatternSpace::Empty;

            for alt in alternatives {
                let (alt_res, s) = resolve_pattern(ctx, alt, expected_ty, expected_space, bindings);
                alt_space = alt_space.union(&s);
                resolved_alts.push(alt_res);
            }

            (
                PatternResolution::Or(ResolvedOrPattern {
                    alternatives: resolved_alts.into_boxed_slice(),
                }),
                alt_space.normalize(),
            )
        }
        Pattern::Tuple { elements, .. } => {
            let mut tuple_res = Vec::with_capacity(elements.len());
            let mut element_spaces = Vec::with_capacity(elements.len());

            for (i, elem) in elements.iter().enumerate() {
                let elem_ty = match ctx.store.get(expected_ty) {
                    TypeData::Tuple(elems) => elems.get(i).map(|e| e.ty).unwrap_or(expected_ty),
                    _ => ctx.core_type(&ctx.core_ids.object.clone()).unwrap_or(expected_ty),
                };
                let elem_expected_space = PatternSpace::Opaque(elem_ty);
                let (elem_res, elem_space) = resolve_pattern(ctx, elem, elem_ty, &elem_expected_space, bindings);
                tuple_res.push(elem_res);
                element_spaces.push(elem_space);
            }

            (
                PatternResolution::Tuple(tuple_res.into_boxed_slice()),
                PatternSpace::Tuple(element_spaces.into_boxed_slice()).normalize(),
            )
        }
        Pattern::List { elements, rest, .. } => {
            let elem_ty = ctx.core_type(&ctx.core_ids.object.clone()).unwrap_or(expected_ty);
            let mut prefix_res = Vec::with_capacity(elements.len());
            for elem in elements {
                let elem_expected_space = PatternSpace::Opaque(elem_ty);
                let (elem_res, _) = resolve_pattern(ctx, elem, elem_ty, &elem_expected_space, bindings);
                prefix_res.push(elem_res);
            }
            let rest_res = rest.as_ref().map(|r| {
                let elem_expected_space = PatternSpace::Opaque(elem_ty);
                let (r_res, _) = resolve_pattern(ctx, r, elem_ty, &elem_expected_space, bindings);
                Box::new(r_res)
            });

            (
                PatternResolution::List(ResolvedListPattern {
                    prefix: prefix_res.into_boxed_slice(),
                    rest: rest_res,
                }),
                expected_space.clone(),
            )
        }
        _ => (PatternResolution::Wildcard, expected_space.clone()),
    }
}

fn try_resolve_contextual_singleton(
    ctx: &mut CheckingContext<'_>,
    name: &str,
    _range: SourceRange,
    expected_ty: TypeId,
    _expected_space: &PatternSpace,
) -> Option<(ResolvedVariantPattern, PatternSpace)> {
    let enum_table = ctx.enum_table?;
    let owner = ctx.store.nominal_origin_declaration(expected_ty)?.clone();
    let enum_info = enum_table.enums.get(&owner)?;

    let target_selector = Selector::getter(name).ok()?;
    let variant_id = enum_info.variants.iter().find(|v| v.selector == target_selector)?;
    let variant_info = enum_table.variants.get(variant_id)?;

    if variant_info.shape != VariantShape::Singleton {
        return None;
    }

    let exact_case = ctx.store.exact_case_type(variant_id, expected_ty).unwrap_or(variant_info.exact_case_template);
    let family_id = variant_info.family.clone().unwrap_or_else(|| VariantFamilyId::new(owner.clone(), name));

    let candidate = ResolvedVariantCandidate {
        variant: variant_id.clone(),
        exact_case,
        fields: Box::new([]),
        proof: BranchProofEnvironment::default(),
    };

    let resolution = ResolvedVariantPattern {
        owner,
        family: family_id,
        selector: VariantSelectorConstraint::Exact(target_selector),
        candidates: Box::new([candidate]),
    };

    let space = PatternSpace::Variant(VariantSpace {
        variant: variant_id.clone(),
        exact_case,
        fields: Box::new([]),
        proof: BranchProofEnvironment::default(),
    });

    Some((resolution, space))
}

fn resolve_variant_pattern(
    ctx: &mut CheckingContext<'_>,
    variant_pat: &VariantPattern,
    expected_ty: TypeId,
    _expected_space: &PatternSpace,
    bindings: &mut Vec<PatternBindingResolution>,
) -> (ResolvedVariantPattern, PatternSpace) {
    let owner_decl = if let Some(ref owner_ref) = variant_pat.owner {
        DeclarationId::new(ctx.current_module.clone(), owner_ref.root.clone().into())
    } else {
        ctx.store.nominal_origin_declaration(expected_ty).cloned().unwrap_or_else(|| {
            DeclarationId::new(ctx.current_module.clone(), variant_pat.base.clone().into())
        })
    };

    let enum_table = ctx.enum_table.cloned();
    let enum_info = enum_table.as_ref().and_then(|t| t.enums.get(&owner_decl).cloned());

    let constraint = match &variant_pat.mode {
        VariantPatternMode::WholeFamily { .. } => VariantSelectorConstraint::WholeFamily,
        VariantPatternMode::Singleton => {
            let sel = selector_from_exact_variant_pattern(variant_pat)
                .unwrap_or_else(|_| Selector::getter(&variant_pat.base).unwrap());
            VariantSelectorConstraint::Exact(sel)
        }
        VariantPatternMode::ExactCall { .. } => {
            let sel = selector_from_exact_variant_pattern(variant_pat)
                .unwrap_or_else(|_| Selector::method(&variant_pat.base, vec![]).unwrap());
            VariantSelectorConstraint::Exact(sel)
        }
        VariantPatternMode::CallablePattern { .. } => {
            let pat = selector_pattern_from_variant_pattern(variant_pat)
                .unwrap_or_else(|_| phalcom_common::selector::SelectorPattern::named(
                    &variant_pat.base,
                    phalcom_common::selector::SelectorKindPattern::AnyNamed,
                    vec![],
                    vec![],
                    true,
                ).unwrap());
            VariantSelectorConstraint::Pattern(pat)
        }
    };

    let mut candidate_resolutions = Vec::new();
    let mut candidate_spaces = Vec::new();

    if let (Some(table), Some(info)) = (&enum_table, &enum_info) {
        for variant_id in info.variants.iter() {
            let Some(v_info) = table.variants.get(variant_id) else { continue; };
            if !matches_selector_constraint(&v_info.id.selector, &variant_pat.base, &constraint) {
                continue;
            }

            let gadt_res = crate::checker::gadt_proof::solve_gadt_branch_proof(
                ctx.store,
                &ctx.hierarchy,
                &owner_decl,
                v_info,
                expected_ty,
            );

            let (proof, exact_case) = match gadt_res {
                crate::checker::gadt_proof::GadtProofResult::Reachable { proof, exact_case } => (proof, exact_case),
                crate::checker::gadt_proof::GadtProofResult::Refuted => {
                    continue;
                }
            };

            let subst = crate::types::substitution::substitution_for_applied(ctx.declarations, ctx.store, expected_ty);
            let mut resolved_fields = Vec::new();
            let mut field_spaces = Vec::new();

            // Match pattern arguments to variant payload fields
            match &variant_pat.mode {
                VariantPatternMode::ExactCall { arguments } => {
                    for (i, arg) in arguments.iter().enumerate() {
                        let field_semantic = if let Some(ref label) = arg.label {
                            v_info.fields.iter().find(|f| f.external_label.as_deref() == Some(label))
                        } else {
                            v_info.fields.get(i)
                        };

                        let (field_id, field_type) = if let Some(f) = field_semantic {
                            let f_raw = f.declared_type.canonical_type().unwrap_or(expected_ty);
                            let f_ty = if let Some(ref s) = subst { s.apply(ctx.store, f_raw) } else { f_raw };
                            (f.id.clone(), TypeKnowledge::established(f_ty, EvidenceOrigin::Flow))
                        } else {
                            (
                                crate::identity::VariantFieldId::new(variant_id.clone(), i as u32),
                                TypeKnowledge::Unknown(UnknownReason::NoTypeEvidence),
                            )
                        };

                        let f_expected_ty = field_type.ty().unwrap_or(expected_ty);
                        let f_expected_space = PatternSpace::Opaque(f_expected_ty);
                        let (child_res, f_space) = resolve_pattern(ctx, &arg.pattern, f_expected_ty, &f_expected_space, bindings);

                        resolved_fields.push(ResolvedFieldPattern {
                            field: field_id,
                            field_type,
                            child: Box::new(child_res),
                        });
                        field_spaces.push(f_space);
                    }
                }
                VariantPatternMode::CallablePattern { prefix, suffix, .. } => {
                    // Initialize field_spaces for all variant fields with opaque wildcard spaces
                    for f in v_info.fields.iter() {
                        let f_raw = f.declared_type.canonical_type().unwrap_or(expected_ty);
                        let f_ty = if let Some(ref s) = subst { s.apply(ctx.store, f_raw) } else { f_raw };
                        field_spaces.push(PatternSpace::Opaque(f_ty));
                    }

                    for (i, arg) in prefix.iter().enumerate() {
                        let (field_idx, field_semantic) = if let Some(ref label) = arg.label {
                            v_info.fields.iter().enumerate().find(|(_, f)| f.external_label.as_deref() == Some(label))
                                .map(|(idx, f)| (idx, Some(f)))
                                .unwrap_or((i, None))
                        } else {
                            (i, v_info.fields.get(i))
                        };

                        let (field_id, field_type) = if let Some(f) = field_semantic {
                            let f_raw = f.declared_type.canonical_type().unwrap_or(expected_ty);
                            let f_ty = if let Some(ref s) = subst { s.apply(ctx.store, f_raw) } else { f_raw };
                            (f.id.clone(), TypeKnowledge::established(f_ty, EvidenceOrigin::Flow))
                        } else {
                            (
                                crate::identity::VariantFieldId::new(variant_id.clone(), field_idx as u32),
                                TypeKnowledge::Unknown(UnknownReason::NoTypeEvidence),
                            )
                        };

                        let f_expected_ty = field_type.ty().unwrap_or(expected_ty);
                        let f_expected_space = PatternSpace::Opaque(f_expected_ty);
                        let (child_res, f_space) = resolve_pattern(ctx, &arg.pattern, f_expected_ty, &f_expected_space, bindings);
                        resolved_fields.push(ResolvedFieldPattern {
                            field: field_id,
                            field_type,
                            child: Box::new(child_res),
                        });
                        if field_idx < field_spaces.len() {
                            field_spaces[field_idx] = f_space;
                        }
                    }

                    for (s_idx, arg) in suffix.iter().enumerate() {
                        let (field_idx, field_semantic) = if let Some(ref label) = arg.label {
                            v_info.fields.iter().enumerate().find(|(_, f)| f.external_label.as_deref() == Some(label))
                                .map(|(idx, f)| (idx, Some(f)))
                                .unwrap_or((v_info.fields.len().saturating_sub(suffix.len() - s_idx), None))
                        } else {
                            let idx = v_info.fields.len().saturating_sub(suffix.len() - s_idx);
                            (idx, v_info.fields.get(idx))
                        };

                        let (field_id, field_type) = if let Some(f) = field_semantic {
                            let f_raw = f.declared_type.canonical_type().unwrap_or(expected_ty);
                            let f_ty = if let Some(ref s) = subst { s.apply(ctx.store, f_raw) } else { f_raw };
                            (f.id.clone(), TypeKnowledge::established(f_ty, EvidenceOrigin::Flow))
                        } else {
                            (
                                crate::identity::VariantFieldId::new(variant_id.clone(), field_idx as u32),
                                TypeKnowledge::Unknown(UnknownReason::NoTypeEvidence),
                            )
                        };

                        let f_expected_ty = field_type.ty().unwrap_or(expected_ty);
                        let f_expected_space = PatternSpace::Opaque(f_expected_ty);
                        let (child_res, f_space) = resolve_pattern(ctx, &arg.pattern, f_expected_ty, &f_expected_space, bindings);
                        resolved_fields.push(ResolvedFieldPattern {
                            field: field_id,
                            field_type,
                            child: Box::new(child_res),
                        });
                        if field_idx < field_spaces.len() {
                            field_spaces[field_idx] = f_space;
                        }
                    }
                }
                VariantPatternMode::Singleton | VariantPatternMode::WholeFamily { .. } => {
                    for (_i, f) in v_info.fields.iter().enumerate() {
                        let f_raw = f.declared_type.canonical_type().unwrap_or(expected_ty);
                        let f_ty = if let Some(ref s) = subst { s.apply(ctx.store, f_raw) } else { f_raw };
                        field_spaces.push(PatternSpace::Opaque(f_ty));
                    }
                }
            }

            candidate_resolutions.push(ResolvedVariantCandidate {
                variant: variant_id.clone(),
                exact_case,
                fields: resolved_fields.into_boxed_slice(),
                proof: proof.clone(),
            });

            candidate_spaces.push(PatternSpace::Variant(VariantSpace {
                variant: variant_id.clone(),
                exact_case,
                fields: field_spaces.into_boxed_slice(),
                proof,
            }));
        }
    }

    let family_id = VariantFamilyId::new(owner_decl.clone(), variant_pat.base.clone());
    let pattern_space = if candidate_spaces.is_empty() {
        PatternSpace::Empty
    } else if candidate_spaces.len() == 1 {
        candidate_spaces.pop().unwrap().normalize()
    } else {
        PatternSpace::Union(candidate_spaces.into_boxed_slice()).normalize()
    };

    let resolution = ResolvedVariantPattern {
        owner: owner_decl,
        family: family_id,
        selector: constraint,
        candidates: candidate_resolutions.into_boxed_slice(),
    };

    (resolution, pattern_space)
}

fn matches_selector_constraint(
    selector: &Selector,
    base_name: &str,
    constraint: &VariantSelectorConstraint,
) -> bool {
    let matches_base = match &selector.base {
        SelectorBase::Named(name) => name == base_name,
        _ => false,
    };
    if !matches_base {
        return false;
    }
    match constraint {
        VariantSelectorConstraint::WholeFamily => true,
        VariantSelectorConstraint::Exact(exact) => selector == exact,
        VariantSelectorConstraint::Pattern(pattern) => pattern.matches(selector),
    }
}
