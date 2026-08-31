use phalcom_ast::ast::{Pattern, VariantPattern, VariantPatternMode};
use phalcom_ast::selector::{selector_from_exact_variant_pattern, selector_pattern_from_variant_pattern};
use phalcom_common::range::SourceRange;
use phalcom_common::selector::{Selector, SelectorBase};
use std::collections::{BTreeMap, BTreeSet};

use crate::checker::context::CheckingContext;
use crate::checker::pattern_space::{PatternSpace, VariantSpace};
use crate::enum_semantics::VariantShape;
use crate::identity::{BindingId, DeclarationId, VariantFamilyId};
use crate::match_semantics::{
    PatternBindingResolution, PatternResolution, ResolvedFieldPattern, ResolvedListPattern, ResolvedOrPattern, ResolvedVariantCandidate,
    ResolvedVariantPattern, VariantSelectorConstraint,
};
use crate::types::denotation::ValueSemanticFact;
use crate::types::evidence::{EvidenceOrigin, TypeKnowledge, UnknownReason};
use crate::types::id::TypeId;
use crate::types::store::TypeData;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum BindingMode {
    Live,
    Detached,
}

/// Resolves an AST pattern against an expected type and value space.
///
/// The returned semantic product contains canonical branch bindings. Candidate
/// and or-pattern alternatives are analyzed with detached temporary identities,
/// then joined and committed exactly once to the surrounding branch scope.
pub fn resolve_pattern(
    ctx: &mut CheckingContext<'_>,
    pattern: &Pattern,
    expected_ty: TypeId,
    expected_space: &PatternSpace,
    bindings: &mut Vec<PatternBindingResolution>,
) -> (PatternResolution, PatternSpace) {
    resolve_pattern_with_mode(ctx, pattern, expected_ty, expected_space, bindings, BindingMode::Live)
}

fn resolve_pattern_with_mode(
    ctx: &mut CheckingContext<'_>,
    pattern: &Pattern,
    expected_ty: TypeId,
    expected_space: &PatternSpace,
    bindings: &mut Vec<PatternBindingResolution>,
    binding_mode: BindingMode,
) -> (PatternResolution, PatternSpace) {
    match pattern {
        Pattern::Wildcard { .. } => (PatternResolution::Wildcard, expected_space.clone()),
        Pattern::Name { name, range } => {
            if let Some((var_res, var_space)) = try_resolve_contextual_singleton(ctx, name, *range, expected_ty, expected_space) {
                (PatternResolution::Variant(var_res), var_space)
            } else {
                bind_name_pattern(ctx, name, *range, expected_ty, expected_space, bindings, binding_mode)
            }
        }
        Pattern::Variant(variant_pat) => {
            let (res, space) = resolve_variant_pattern(ctx, variant_pat, expected_ty, expected_space, bindings, binding_mode);
            (PatternResolution::Variant(res), space)
        }
        Pattern::Or { alternatives, range } => {
            let mut resolved_alternatives = Vec::with_capacity(alternatives.len());
            let mut alternative_spaces = Vec::with_capacity(alternatives.len());
            let mut alternative_bindings = Vec::with_capacity(alternatives.len());
            let mut local_remaining = expected_space.clone().normalize();

            for alternative in alternatives {
                let mut local_bindings = Vec::new();
                let (resolution, space) = resolve_pattern_with_mode(ctx, alternative, expected_ty, expected_space, &mut local_bindings, BindingMode::Detached);

                let in_domain = expected_space.intersect(&space, ctx.store, &ctx.hierarchy).normalize();
                let reachable = local_remaining.intersect(&space, ctx.store, &ctx.hierarchy).normalize();
                if !in_domain.is_empty() && reachable.is_empty() {
                    ctx.emit_diagnostic(crate::diagnostic::SemanticDiagnostic::error_in(
                        ctx.current_module.clone(),
                        crate::diagnostic::DiagnosticCode::MatchPatternOrRedundant,
                        "redundant or-pattern alternative: earlier alternatives already cover its reachable value space",
                        *range,
                    ));
                }
                local_remaining = local_remaining.subtract(&space, ctx.store, &ctx.hierarchy).normalize();

                resolved_alternatives.push(resolution);
                alternative_spaces.push(space);
                alternative_bindings.push(local_bindings);
            }

            let replacements = commit_shared_bindings(
                ctx,
                &alternative_bindings,
                bindings,
                *range,
                binding_mode,
                crate::diagnostic::DiagnosticCode::MatchPatternOrBindingMismatch,
                "or-pattern alternatives must introduce the same binding names",
            );
            for resolution in &mut resolved_alternatives {
                remap_pattern_bindings(resolution, &replacements);
            }

            let mut covered = PatternSpace::Empty;
            for space in alternative_spaces {
                covered = covered.union(&space);
            }
            (
                PatternResolution::Or(ResolvedOrPattern {
                    alternatives: resolved_alternatives.into_boxed_slice(),
                }),
                covered.normalize(),
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
                let (elem_res, elem_space) = resolve_pattern_with_mode(ctx, elem, elem_ty, &elem_expected_space, bindings, binding_mode);
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
                let (elem_res, _) = resolve_pattern_with_mode(ctx, elem, elem_ty, &elem_expected_space, bindings, binding_mode);
                prefix_res.push(elem_res);
            }
            let rest_res = rest.as_ref().map(|rest_pattern| {
                let elem_expected_space = PatternSpace::Opaque(elem_ty);
                let (rest_resolution, _) = resolve_pattern_with_mode(ctx, rest_pattern, elem_ty, &elem_expected_space, bindings, binding_mode);
                Box::new(rest_resolution)
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

fn bind_name_pattern(
    ctx: &mut CheckingContext<'_>,
    name: &str,
    range: SourceRange,
    expected_ty: TypeId,
    expected_space: &PatternSpace,
    bindings: &mut Vec<PatternBindingResolution>,
    binding_mode: BindingMode,
) -> (PatternResolution, PatternSpace) {
    if let Some(existing) = bindings.iter().find(|binding| binding.name.as_ref() == name).cloned() {
        ctx.emit_diagnostic(
            crate::diagnostic::SemanticDiagnostic::error_in(
                ctx.current_module.clone(),
                crate::diagnostic::DiagnosticCode::MatchPatternDuplicateBinding,
                format!("pattern binds `{name}` more than once in the same alternative"),
                range,
            )
            .with_label(existing.source, "first binding in this pattern alternative"),
        );
        return (
            PatternResolution::Binding {
                binding: existing.binding,
                name: name.into(),
                knowledge: existing.knowledge,
            },
            expected_space.clone(),
        );
    }

    let knowledge = TypeKnowledge::established(expected_ty, EvidenceOrigin::PatternDecomposition);
    let binding_id = declare_pattern_binding(ctx, name, range, &knowledge, binding_mode);
    bindings.push(PatternBindingResolution {
        binding: binding_id,
        name: name.into(),
        knowledge: knowledge.clone(),
        source: range,
    });
    (
        PatternResolution::Binding {
            binding: binding_id,
            name: name.into(),
            knowledge,
        },
        expected_space.clone(),
    )
}

fn declare_pattern_binding(ctx: &mut CheckingContext<'_>, name: &str, range: SourceRange, knowledge: &TypeKnowledge, mode: BindingMode) -> BindingId {
    match mode {
        BindingMode::Detached => ctx.alloc_binding(),
        BindingMode::Live => {
            let result = ctx.bind_pattern_binding_with_causal(
                name.to_owned(),
                ValueSemanticFact::new(knowledge.clone()),
                range,
                crate::checker::causal::CausalInvalidity::Clean,
            );
            match result {
                crate::checker::binding::BindingDeclarationResult::Inserted(binding)
                | crate::checker::binding::BindingDeclarationResult::Redeclared(binding) => binding,
            }
        }
    }
}

fn commit_shared_bindings(
    ctx: &mut CheckingContext<'_>,
    alternatives: &[Vec<PatternBindingResolution>],
    output: &mut Vec<PatternBindingResolution>,
    range: SourceRange,
    mode: BindingMode,
    mismatch_code: crate::diagnostic::DiagnosticCode,
    mismatch_message: &str,
) -> BTreeMap<String, (BindingId, TypeKnowledge)> {
    let Some(first) = alternatives.first() else {
        return BTreeMap::new();
    };

    let first_names = first.iter().map(|binding| binding.name.to_string()).collect::<BTreeSet<_>>();
    let coherent = alternatives
        .iter()
        .all(|alternative| alternative.iter().map(|binding| binding.name.to_string()).collect::<BTreeSet<_>>() == first_names);
    if !coherent {
        ctx.emit_diagnostic(crate::diagnostic::SemanticDiagnostic::error_in(
            ctx.current_module.clone(),
            mismatch_code,
            mismatch_message,
            range,
        ));
        return BTreeMap::new();
    }

    let mut replacements = BTreeMap::new();
    for name in first_names {
        let matching = alternatives
            .iter()
            .filter_map(|alternative| alternative.iter().find(|binding| binding.name.as_ref() == name.as_str()))
            .collect::<Vec<_>>();
        if matching.len() != alternatives.len() {
            continue;
        }

        let knowledge = crate::types::evidence::join_type_knowledge(ctx.store, matching.iter().map(|binding| binding.knowledge.clone()).collect::<Vec<_>>());
        let source = matching.first().map(|binding| binding.source).unwrap_or(range);

        if let Some(existing) = output.iter().find(|binding| binding.name.as_ref() == name.as_str()).cloned() {
            ctx.emit_diagnostic(
                crate::diagnostic::SemanticDiagnostic::error_in(
                    ctx.current_module.clone(),
                    crate::diagnostic::DiagnosticCode::MatchPatternDuplicateBinding,
                    format!("pattern binds `{name}` more than once in the same alternative"),
                    source,
                )
                .with_label(existing.source, "first binding in this pattern alternative"),
            );
            replacements.insert(name, (existing.binding, existing.knowledge));
            continue;
        }

        let binding = declare_pattern_binding(ctx, &name, source, &knowledge, mode);
        output.push(PatternBindingResolution {
            binding,
            name: name.clone().into_boxed_str(),
            knowledge: knowledge.clone(),
            source,
        });
        replacements.insert(name, (binding, knowledge));
    }
    replacements
}

fn remap_pattern_bindings(resolution: &mut PatternResolution, replacements: &BTreeMap<String, (BindingId, TypeKnowledge)>) {
    match resolution {
        PatternResolution::Wildcard => {}
        PatternResolution::Binding { binding, name, knowledge } => {
            if let Some((replacement, joined)) = replacements.get(name.as_ref()) {
                *binding = *replacement;
                *knowledge = joined.clone();
            }
        }
        PatternResolution::Variant(variant) => {
            for candidate in variant.candidates.iter_mut() {
                for field in candidate.fields.iter_mut() {
                    remap_pattern_bindings(&mut field.child, replacements);
                }
            }
        }
        PatternResolution::Or(or_pattern) => {
            for alternative in or_pattern.alternatives.iter_mut() {
                remap_pattern_bindings(alternative, replacements);
            }
        }
        PatternResolution::Tuple(elements) => {
            for element in elements.iter_mut() {
                remap_pattern_bindings(element, replacements);
            }
        }
        PatternResolution::List(list) => {
            for element in list.prefix.iter_mut() {
                remap_pattern_bindings(element, replacements);
            }
            if let Some(rest) = list.rest.as_mut() {
                remap_pattern_bindings(rest, replacements);
            }
        }
        PatternResolution::Record(fields) => {
            for field in fields.iter_mut() {
                remap_pattern_bindings(&mut field.child, replacements);
            }
        }
        PatternResolution::Map(entries) => {
            for entry in entries.iter_mut() {
                remap_pattern_bindings(&mut entry.child, replacements);
            }
        }
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
    let variant_id = enum_info.variants.iter().find(|variant| variant.selector == target_selector)?;
    let variant_info = enum_table.variants.get(variant_id)?.clone();

    if variant_info.shape != VariantShape::Singleton {
        return None;
    }

    ctx.record_semantic_dependency(crate::checker::analysis::SemanticDependency::EnumDeclaration(owner.clone()));

    let (proof, exact_case) = match crate::checker::gadt_proof::solve_gadt_branch_proof(ctx.store, &ctx.hierarchy, &owner, &variant_info, expected_ty) {
        crate::checker::gadt_proof::GadtProofResult::Reachable { proof, exact_case } => (proof, exact_case),
        crate::checker::gadt_proof::GadtProofResult::Refuted => return None,
    };
    let family_id = variant_info.family.clone().unwrap_or_else(|| VariantFamilyId::new(owner.clone(), name));

    let candidate = ResolvedVariantCandidate {
        variant: variant_id.clone(),
        exact_case,
        fields: Box::new([]),
        proof: proof.clone(),
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
        proof,
    });

    Some((resolution, space))
}

fn resolve_variant_pattern(
    ctx: &mut CheckingContext<'_>,
    variant_pat: &VariantPattern,
    expected_ty: TypeId,
    _expected_space: &PatternSpace,
    bindings: &mut Vec<PatternBindingResolution>,
    binding_mode: BindingMode,
) -> (ResolvedVariantPattern, PatternSpace) {
    let expected_nominal_decl = ctx.store.nominal_origin_declaration(expected_ty).cloned();

    let owner_decl = if let Some(ref owner_ref) = variant_pat.owner {
        let decl = DeclarationId::new(ctx.current_module.clone(), owner_ref.root.clone().into());
        if let Some(ref exp_decl) = expected_nominal_decl {
            if &decl != exp_decl {
                ctx.emit_diagnostic(crate::diagnostic::SemanticDiagnostic::error_in(
                    ctx.current_module.clone(),
                    crate::diagnostic::DiagnosticCode::MatchPatternContradictory,
                    format!("pattern type `{}` cannot match scrutinee nominal type `{}`", decl.name, exp_decl.name),
                    variant_pat.range,
                ));
            }
        }
        decl
    } else {
        expected_nominal_decl.unwrap_or_else(|| DeclarationId::new(ctx.current_module.clone(), variant_pat.base.clone().into()))
    };

    ctx.record_semantic_dependency(crate::checker::analysis::SemanticDependency::EnumDeclaration(owner_decl.clone()));

    let enum_table = ctx.enum_table.cloned();
    let enum_info = enum_table.as_ref().and_then(|table| table.enums.get(&owner_decl).cloned());

    if enum_info.is_none() {
        ctx.emit_diagnostic(crate::diagnostic::SemanticDiagnostic::error_in(
            ctx.current_module.clone(),
            crate::diagnostic::DiagnosticCode::MatchPatternUnresolved,
            format!("type `{}` is not an enum or cannot be resolved", owner_decl.name),
            variant_pat.range,
        ));
    }

    let constraint = match &variant_pat.mode {
        VariantPatternMode::WholeFamily { .. } => VariantSelectorConstraint::WholeFamily,
        VariantPatternMode::Singleton => {
            let selector =
                selector_from_exact_variant_pattern(variant_pat).unwrap_or_else(|_| Selector::getter(&variant_pat.base).expect("variant getter selector"));
            VariantSelectorConstraint::Exact(selector)
        }
        VariantPatternMode::ExactCall { .. } => {
            let selector = selector_from_exact_variant_pattern(variant_pat)
                .unwrap_or_else(|_| Selector::method(&variant_pat.base, vec![]).expect("variant method selector"));
            VariantSelectorConstraint::Exact(selector)
        }
        VariantPatternMode::CallablePattern { .. } => {
            let pattern = selector_pattern_from_variant_pattern(variant_pat).unwrap_or_else(|_| {
                phalcom_common::selector::SelectorPattern::named(
                    &variant_pat.base,
                    phalcom_common::selector::SelectorKindPattern::AnyNamed,
                    vec![],
                    vec![],
                    true,
                )
                .expect("fallback selector pattern")
            });
            VariantSelectorConstraint::Pattern(pattern)
        }
    };

    let mut candidate_resolutions = Vec::new();
    let mut candidate_spaces = Vec::new();
    let mut candidate_bindings = Vec::new();

    if let (Some(table), Some(info)) = (&enum_table, &enum_info) {
        let matching_base_variants = info
            .variants
            .iter()
            .filter(|variant| match &variant.selector.base {
                SelectorBase::Named(name) => name == &variant_pat.base,
                _ => false,
            })
            .cloned()
            .collect::<Vec<_>>();

        if matching_base_variants.is_empty() {
            ctx.emit_diagnostic(crate::diagnostic::SemanticDiagnostic::error_in(
                ctx.current_module.clone(),
                crate::diagnostic::DiagnosticCode::MatchPatternUnresolved,
                format!("variant `{}` not found in enum `{}`", variant_pat.base, owner_decl.name),
                variant_pat.range,
            ));
        }

        let mut matched_any_variant = false;

        for variant_id in &matching_base_variants {
            let Some(v_info) = table.variants.get(variant_id) else {
                continue;
            };
            if !matches_selector_constraint(&v_info.id.selector, &variant_pat.base, &constraint) {
                continue;
            }
            matched_any_variant = true;

            let (proof, exact_case) = match crate::checker::gadt_proof::solve_gadt_branch_proof(ctx.store, &ctx.hierarchy, &owner_decl, v_info, expected_ty) {
                crate::checker::gadt_proof::GadtProofResult::Reachable { proof, exact_case } => (proof, exact_case),
                crate::checker::gadt_proof::GadtProofResult::Refuted => continue,
            };

            let substitution = crate::types::substitution::substitution_for_applied(ctx.declarations, ctx.store, expected_ty);
            let mut resolved_fields = Vec::new();
            let mut field_spaces = Vec::new();
            let mut local_bindings = Vec::new();

            let specialize_field = |ctx: &mut CheckingContext<'_>, raw: TypeId| {
                let declaration_specialized = substitution.as_ref().map(|substitution| substitution.apply(ctx.store, raw)).unwrap_or(raw);
                crate::checker::gadt_proof::apply_branch_proof(ctx.store, &proof, declaration_specialized)
            };

            match &variant_pat.mode {
                VariantPatternMode::ExactCall { arguments } => {
                    for (i, argument) in arguments.iter().enumerate() {
                        let field_semantic = if let Some(ref label) = argument.label {
                            v_info.fields.iter().find(|field| field.external_label.as_deref() == Some(label))
                        } else {
                            v_info.fields.get(i)
                        };

                        let (field_id, field_type) = if let Some(field) = field_semantic {
                            let raw = field.declared_type.canonical_type().unwrap_or(expected_ty);
                            let ty = specialize_field(ctx, raw);
                            (field.id.clone(), TypeKnowledge::established(ty, EvidenceOrigin::PatternDecomposition))
                        } else {
                            (
                                crate::identity::VariantFieldId::new(variant_id.clone(), i as u32),
                                TypeKnowledge::Unknown(UnknownReason::NoTypeEvidence),
                            )
                        };

                        let field_expected_ty = field_type.ty().unwrap_or(expected_ty);
                        let field_expected_space = PatternSpace::Opaque(field_expected_ty);
                        let (child, field_space) = resolve_pattern_with_mode(
                            ctx,
                            &argument.pattern,
                            field_expected_ty,
                            &field_expected_space,
                            &mut local_bindings,
                            BindingMode::Detached,
                        );
                        resolved_fields.push(ResolvedFieldPattern {
                            field: field_id,
                            field_type,
                            child: Box::new(child),
                        });
                        field_spaces.push(field_space);
                    }
                }
                VariantPatternMode::CallablePattern { prefix, suffix, .. } => {
                    for field in v_info.fields.iter() {
                        let raw = field.declared_type.canonical_type().unwrap_or(expected_ty);
                        field_spaces.push(PatternSpace::Opaque(specialize_field(ctx, raw)));
                    }

                    for (i, argument) in prefix.iter().enumerate() {
                        let (field_index, field_semantic) = if let Some(ref label) = argument.label {
                            v_info
                                .fields
                                .iter()
                                .enumerate()
                                .find(|(_, field)| field.external_label.as_deref() == Some(label))
                                .map(|(index, field)| (index, Some(field)))
                                .unwrap_or((i, None))
                        } else {
                            (i, v_info.fields.get(i))
                        };
                        let (field_id, field_type) = if let Some(field) = field_semantic {
                            let raw = field.declared_type.canonical_type().unwrap_or(expected_ty);
                            let ty = specialize_field(ctx, raw);
                            (field.id.clone(), TypeKnowledge::established(ty, EvidenceOrigin::PatternDecomposition))
                        } else {
                            (
                                crate::identity::VariantFieldId::new(variant_id.clone(), field_index as u32),
                                TypeKnowledge::Unknown(UnknownReason::NoTypeEvidence),
                            )
                        };
                        let field_expected_ty = field_type.ty().unwrap_or(expected_ty);
                        let field_expected_space = PatternSpace::Opaque(field_expected_ty);
                        let (child, field_space) = resolve_pattern_with_mode(
                            ctx,
                            &argument.pattern,
                            field_expected_ty,
                            &field_expected_space,
                            &mut local_bindings,
                            BindingMode::Detached,
                        );
                        resolved_fields.push(ResolvedFieldPattern {
                            field: field_id,
                            field_type,
                            child: Box::new(child),
                        });
                        if field_index < field_spaces.len() {
                            field_spaces[field_index] = field_space;
                        }
                    }

                    for (suffix_index, argument) in suffix.iter().enumerate() {
                        let (field_index, field_semantic) = if let Some(ref label) = argument.label {
                            v_info
                                .fields
                                .iter()
                                .enumerate()
                                .find(|(_, field)| field.external_label.as_deref() == Some(label))
                                .map(|(index, field)| (index, Some(field)))
                                .unwrap_or((v_info.fields.len().saturating_sub(suffix.len() - suffix_index), None))
                        } else {
                            let index = v_info.fields.len().saturating_sub(suffix.len() - suffix_index);
                            (index, v_info.fields.get(index))
                        };
                        let (field_id, field_type) = if let Some(field) = field_semantic {
                            let raw = field.declared_type.canonical_type().unwrap_or(expected_ty);
                            let ty = specialize_field(ctx, raw);
                            (field.id.clone(), TypeKnowledge::established(ty, EvidenceOrigin::PatternDecomposition))
                        } else {
                            (
                                crate::identity::VariantFieldId::new(variant_id.clone(), field_index as u32),
                                TypeKnowledge::Unknown(UnknownReason::NoTypeEvidence),
                            )
                        };
                        let field_expected_ty = field_type.ty().unwrap_or(expected_ty);
                        let field_expected_space = PatternSpace::Opaque(field_expected_ty);
                        let (child, field_space) = resolve_pattern_with_mode(
                            ctx,
                            &argument.pattern,
                            field_expected_ty,
                            &field_expected_space,
                            &mut local_bindings,
                            BindingMode::Detached,
                        );
                        resolved_fields.push(ResolvedFieldPattern {
                            field: field_id,
                            field_type,
                            child: Box::new(child),
                        });
                        if field_index < field_spaces.len() {
                            field_spaces[field_index] = field_space;
                        }
                    }
                }
                VariantPatternMode::Singleton | VariantPatternMode::WholeFamily { .. } => {
                    for field in v_info.fields.iter() {
                        let raw = field.declared_type.canonical_type().unwrap_or(expected_ty);
                        field_spaces.push(PatternSpace::Opaque(specialize_field(ctx, raw)));
                    }
                }
            }

            candidate_bindings.push(local_bindings);
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

        if !matching_base_variants.is_empty() && !matched_any_variant {
            for variant_id in &matching_base_variants {
                let Some(v_info) = table.variants.get(variant_id) else {
                    continue;
                };
                match &variant_pat.mode {
                    VariantPatternMode::ExactCall { arguments } => {
                        if v_info.fields.len() != arguments.len() {
                            ctx.emit_diagnostic(crate::diagnostic::SemanticDiagnostic::error_in(
                                ctx.current_module.clone(),
                                crate::diagnostic::DiagnosticCode::MatchPatternArityMismatch,
                                format!(
                                    "variant `{}` expects {} arguments, got {}",
                                    v_info.id.selector,
                                    v_info.fields.len(),
                                    arguments.len()
                                ),
                                variant_pat.range,
                            ));
                        }
                        for argument in arguments.iter() {
                            if let Some(ref label) = argument.label {
                                if !v_info.fields.iter().any(|field| field.external_label.as_deref() == Some(label)) {
                                    ctx.emit_diagnostic(crate::diagnostic::SemanticDiagnostic::error_in(
                                        ctx.current_module.clone(),
                                        crate::diagnostic::DiagnosticCode::MatchPatternFieldMismatch,
                                        format!("field label `{}` does not match any field of variant `{}`", label, v_info.id.selector),
                                        argument.range,
                                    ));
                                }
                            }
                        }
                    }
                    VariantPatternMode::CallablePattern { prefix, suffix, .. } => {
                        if prefix.len() + suffix.len() > v_info.fields.len() {
                            ctx.emit_diagnostic(crate::diagnostic::SemanticDiagnostic::error_in(
                                ctx.current_module.clone(),
                                crate::diagnostic::DiagnosticCode::MatchPatternArityMismatch,
                                format!("too many pattern arguments for variant `{}`", v_info.id.selector),
                                variant_pat.range,
                            ));
                        }
                        for argument in prefix.iter().chain(suffix.iter()) {
                            if let Some(ref label) = argument.label {
                                if !v_info.fields.iter().any(|field| field.external_label.as_deref() == Some(label)) {
                                    ctx.emit_diagnostic(crate::diagnostic::SemanticDiagnostic::error_in(
                                        ctx.current_module.clone(),
                                        crate::diagnostic::DiagnosticCode::MatchPatternFieldMismatch,
                                        format!("field label `{}` does not match any field of variant `{}`", label, v_info.id.selector),
                                        argument.range,
                                    ));
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        if !matching_base_variants.is_empty() && candidate_resolutions.is_empty() && matched_any_variant {
            ctx.emit_diagnostic(crate::diagnostic::SemanticDiagnostic::error_in(
                ctx.current_module.clone(),
                crate::diagnostic::DiagnosticCode::MatchPatternContradictory,
                format!("pattern `{}` is contradictory for scrutinee type", variant_pat.base),
                variant_pat.range,
            ));
        }
    }

    let replacements = commit_shared_bindings(
        ctx,
        &candidate_bindings,
        bindings,
        variant_pat.range,
        binding_mode,
        crate::diagnostic::DiagnosticCode::MatchPatternOrBindingMismatch,
        "variant-family candidates establish incompatible pattern bindings",
    );
    for candidate in &mut candidate_resolutions {
        for field in candidate.fields.iter_mut() {
            remap_pattern_bindings(&mut field.child, &replacements);
        }
    }

    let family_id = VariantFamilyId::new(owner_decl.clone(), variant_pat.base.clone());
    let pattern_space = match candidate_spaces.len() {
        0 => PatternSpace::Empty,
        1 => candidate_spaces.pop().expect("single candidate space exists").normalize(),
        _ => PatternSpace::Union(candidate_spaces.into_boxed_slice()).normalize(),
    };

    let resolution = ResolvedVariantPattern {
        owner: owner_decl,
        family: family_id,
        selector: constraint,
        candidates: candidate_resolutions.into_boxed_slice(),
    };

    (resolution, pattern_space)
}

fn matches_selector_constraint(selector: &Selector, base_name: &str, constraint: &VariantSelectorConstraint) -> bool {
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
