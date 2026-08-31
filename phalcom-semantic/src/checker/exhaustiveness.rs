//! Match-space construction, ordered usefulness, exhaustiveness, and witness generation (Part 05.1).

use crate::checker::context::CheckingContext;
use crate::checker::pattern_space::{PatternSpace, VariantSpace};
use crate::diagnostic::{DiagnosticCode, SemanticDiagnostic};
use crate::match_semantics::{CoverageWitness, ExhaustivenessResult, PatternUsefulness};
use crate::types::id::TypeId;
use crate::types::store::TypeData;
use std::collections::BTreeSet;

const MAX_COVERAGE_WITNESSES: usize = 8;

/// Computes the reachable pattern value-space for a scrutinee type.
///
/// Closed ADTs, exact cases, tuples, and union members are expanded recursively.
/// Recursive type cycles fall back to an opaque member instead of recursing
/// indefinitely; that fallback is conservative and therefore cannot manufacture
/// a false exhaustiveness proof.
pub fn build_initial_pattern_space(ctx: &mut CheckingContext<'_>, scrutinee_ty: TypeId) -> PatternSpace {
    let mut visiting = BTreeSet::new();
    build_initial_pattern_space_inner(ctx, scrutinee_ty, &mut visiting).normalize()
}

fn build_initial_pattern_space_inner(ctx: &mut CheckingContext<'_>, scrutinee_ty: TypeId, visiting: &mut BTreeSet<TypeId>) -> PatternSpace {
    if !visiting.insert(scrutinee_ty) {
        return PatternSpace::Opaque(scrutinee_ty);
    }

    let store_type = ctx.store.get(scrutinee_ty).clone();
    let result = match store_type {
        TypeData::Never => PatternSpace::Empty,
        TypeData::Union(members) => {
            let spaces = members
                .iter()
                .map(|&member| build_initial_pattern_space_inner(ctx, member, visiting))
                .collect::<Vec<_>>();
            PatternSpace::Union(spaces.into_boxed_slice()).normalize()
        }
        TypeData::ExactCase { variant, enum_type } => {
            let variant_id = ctx.store.variant_identity(variant).clone();
            let var_info = ctx.enum_table.and_then(|table| table.variants.get(&variant_id)).cloned();
            if let Some(info) = var_info {
                ctx.record_semantic_dependency(crate::checker::analysis::SemanticDependency::EnumDeclaration(variant_id.owner.clone()));
                match crate::checker::gadt_proof::solve_gadt_branch_proof(ctx.store, &ctx.hierarchy, &variant_id.owner, &info, enum_type) {
                    crate::checker::gadt_proof::GadtProofResult::Reachable { proof, exact_case } => {
                        let declaration_substitution = crate::types::substitution::substitution_for_applied(ctx.declarations, ctx.store, enum_type);
                        let fields = info
                            .fields
                            .iter()
                            .map(|field| {
                                let raw = field.declared_type.canonical_type().unwrap_or(enum_type);
                                let declaration_specialized = declaration_substitution
                                    .as_ref()
                                    .map(|substitution| substitution.apply(ctx.store, raw))
                                    .unwrap_or(raw);
                                let branch_specialized = crate::checker::gadt_proof::apply_branch_proof(ctx.store, &proof, declaration_specialized);
                                build_initial_pattern_space_inner(ctx, branch_specialized, visiting)
                            })
                            .collect::<Vec<_>>();
                        PatternSpace::Variant(VariantSpace {
                            variant: variant_id,
                            exact_case,
                            fields: fields.into_boxed_slice(),
                            proof,
                        })
                    }
                    crate::checker::gadt_proof::GadtProofResult::Refuted => PatternSpace::Empty,
                }
            } else {
                PatternSpace::Opaque(scrutinee_ty)
            }
        }
        TypeData::Tuple(elements) => {
            let spaces = elements
                .iter()
                .map(|element| build_initial_pattern_space_inner(ctx, element.ty, visiting))
                .collect::<Vec<_>>();
            PatternSpace::Tuple(spaces.into_boxed_slice()).normalize()
        }
        _ => build_enum_or_opaque_space(ctx, scrutinee_ty, visiting),
    };

    visiting.remove(&scrutinee_ty);
    result.normalize()
}

fn build_enum_or_opaque_space(ctx: &mut CheckingContext<'_>, scrutinee_ty: TypeId, visiting: &mut BTreeSet<TypeId>) -> PatternSpace {
    let Some(owner) = ctx.store.nominal_origin_declaration(scrutinee_ty).cloned() else {
        return PatternSpace::Opaque(scrutinee_ty);
    };
    let Some(enum_info) = ctx.enum_table.and_then(|table| table.enums.get(&owner)).cloned() else {
        return PatternSpace::Opaque(scrutinee_ty);
    };

    ctx.record_semantic_dependency(crate::checker::analysis::SemanticDependency::EnumDeclaration(owner.clone()));
    let declaration_substitution = crate::types::substitution::substitution_for_applied(ctx.declarations, ctx.store, scrutinee_ty);
    let variant_infos = enum_info
        .variants
        .iter()
        .filter_map(|variant| ctx.enum_table.and_then(|table| table.variants.get(variant)).cloned())
        .collect::<Vec<_>>();

    let mut variants = Vec::new();
    for info in variant_infos {
        let crate::checker::gadt_proof::GadtProofResult::Reachable { proof, exact_case } =
            crate::checker::gadt_proof::solve_gadt_branch_proof(ctx.store, &ctx.hierarchy, &owner, &info, scrutinee_ty)
        else {
            continue;
        };

        let mut fields = Vec::with_capacity(info.fields.len());
        for field in info.fields.iter() {
            let raw = field.declared_type.canonical_type().unwrap_or(scrutinee_ty);
            let declaration_specialized = declaration_substitution
                .as_ref()
                .map(|substitution| substitution.apply(ctx.store, raw))
                .unwrap_or(raw);
            let branch_specialized = crate::checker::gadt_proof::apply_branch_proof(ctx.store, &proof, declaration_specialized);
            fields.push(build_initial_pattern_space_inner(ctx, branch_specialized, visiting));
        }

        variants.push(PatternSpace::Variant(VariantSpace {
            variant: info.id.clone(),
            exact_case,
            fields: fields.into_boxed_slice(),
            proof,
        }));
    }

    match variants.len() {
        0 => PatternSpace::Empty,
        1 => variants.pop().expect("single variant space exists").normalize(),
        _ => PatternSpace::Union(variants.into_boxed_slice()).normalize(),
    }
}

/// Evaluates ordered match-arm usefulness, residual space, and final exhaustiveness.
///
/// `Impossible` is measured against the original scrutinee domain; `Redundant`
/// is measured against the residual after earlier arms. This distinction is
/// semantically significant for GADTs and exact-case unions.
pub fn evaluate_match_exhaustiveness(
    ctx: &mut CheckingContext<'_>,
    initial_space: &PatternSpace,
    arm_spaces: &[PatternSpace],
    arm_ranges: &[phalcom_common::range::SourceRange],
    match_range: phalcom_common::range::SourceRange,
) -> (ExhaustivenessResult, Vec<(PatternSpace, PatternSpace, PatternUsefulness)>) {
    let mut current_space = initial_space.clone().normalize();
    let mut arm_results = Vec::with_capacity(arm_spaces.len());

    for (index, arm_space) in arm_spaces.iter().enumerate() {
        let in_original_domain = initial_space.intersect(arm_space, ctx.store, &ctx.hierarchy).normalize();
        let reachable = current_space.intersect(arm_space, ctx.store, &ctx.hierarchy).normalize();
        let usefulness = if in_original_domain.is_empty() {
            PatternUsefulness::Impossible
        } else if reachable.is_empty() {
            let range = arm_ranges.get(index).copied().unwrap_or(match_range);
            ctx.emit_diagnostic(SemanticDiagnostic::error_in(
                ctx.current_module.clone(),
                DiagnosticCode::MatchArmRedundant,
                "redundant match arm: earlier patterns already cover every reachable value of this pattern",
                range,
            ));
            PatternUsefulness::Redundant
        } else {
            PatternUsefulness::Useful
        };

        current_space = current_space.subtract(arm_space, ctx.store, &ctx.hierarchy).normalize();
        arm_results.push((reachable, current_space.clone(), usefulness));
    }

    let exhaustiveness = if current_space.is_empty() {
        ExhaustivenessResult::Proven
    } else {
        let witnesses = generate_coverage_witnesses(&current_space, MAX_COVERAGE_WITNESSES);
        ctx.emit_diagnostic(
            SemanticDiagnostic::error_in(
                ctx.current_module.clone(),
                DiagnosticCode::MatchNonExhaustive,
                "non-exhaustive match: reachable values remain uncovered",
                match_range,
            )
            .with_note(format!("{} uncovered value-space witness(es) retained", witnesses.len())),
        );
        ExhaustivenessResult::Missing(witnesses.into_boxed_slice())
    };

    (exhaustiveness, arm_results)
}

/// Produces a bounded set of structured witnesses for an uncovered pattern space.
fn generate_coverage_witnesses(space: &PatternSpace, limit: usize) -> Vec<CoverageWitness> {
    let mut output = Vec::new();
    push_coverage_witnesses(space, limit, &mut output);
    output
}

fn push_coverage_witnesses(space: &PatternSpace, limit: usize, output: &mut Vec<CoverageWitness>) {
    if output.len() >= limit {
        return;
    }
    match space {
        PatternSpace::Empty => {}
        PatternSpace::Opaque(ty) => output.push(CoverageWitness::Opaque(*ty)),
        PatternSpace::Union(members) => {
            for member in members.iter() {
                if output.len() >= limit {
                    break;
                }
                push_coverage_witnesses(member, limit, output);
            }
        }
        PatternSpace::Variant(variant) => {
            let fields = variant.fields.iter().map(first_coverage_witness).collect::<Vec<_>>().into_boxed_slice();
            output.push(CoverageWitness::Variant {
                variant: variant.variant.clone(),
                exact_case: variant.exact_case,
                fields,
            });
        }
        PatternSpace::Tuple(elements) => output.push(CoverageWitness::Tuple(
            elements.iter().map(first_coverage_witness).collect::<Vec<_>>().into_boxed_slice(),
        )),
        PatternSpace::List(list) => {
            let mut elements = list.prefix.iter().map(first_coverage_witness).collect::<Vec<_>>();
            if let Some(rest) = &list.rest {
                elements.push(first_coverage_witness(rest));
            }
            output.push(CoverageWitness::List(elements.into_boxed_slice()));
        }
    }
}

fn first_coverage_witness(space: &PatternSpace) -> CoverageWitness {
    match space {
        PatternSpace::Empty => CoverageWitness::Wildcard,
        PatternSpace::Opaque(ty) => CoverageWitness::Opaque(*ty),
        PatternSpace::Union(members) => members.first().map(first_coverage_witness).unwrap_or(CoverageWitness::Wildcard),
        PatternSpace::Variant(variant) => CoverageWitness::Variant {
            variant: variant.variant.clone(),
            exact_case: variant.exact_case,
            fields: variant.fields.iter().map(first_coverage_witness).collect::<Vec<_>>().into_boxed_slice(),
        },
        PatternSpace::Tuple(elements) => CoverageWitness::Tuple(elements.iter().map(first_coverage_witness).collect::<Vec<_>>().into_boxed_slice()),
        PatternSpace::List(list) => {
            let mut elements = list.prefix.iter().map(first_coverage_witness).collect::<Vec<_>>();
            if let Some(rest) = &list.rest {
                elements.push(first_coverage_witness(rest));
            }
            CoverageWitness::List(elements.into_boxed_slice())
        }
    }
}
