//! Match-space elimination loop, exhaustiveness prover, and missing-case witness generator (Part 05.1).

use crate::checker::context::CheckingContext;
use crate::checker::pattern_space::{PatternSpace, VariantSpace};
use crate::diagnostic::{DiagnosticCode, SemanticDiagnostic};
use crate::match_semantics::{CoverageWitness, ExhaustivenessResult, PatternUsefulness};
use crate::types::id::TypeId;
use crate::types::store::TypeData;
use phalcom_common::range::SourceRange;

/// Computes the initial pattern value-space for a given scrutinee type.
pub fn build_initial_pattern_space(
    ctx: &mut CheckingContext<'_>,
    scrutinee_ty: TypeId,
) -> PatternSpace {
    let store_type = ctx.store.get(scrutinee_ty).clone();
    match store_type {
        TypeData::ExactCase { variant, .. } => {
            let variant_id = ctx.store.variant_identity(variant).clone();
            let var_info = ctx.enum_table.and_then(|t| t.variants.get(&variant_id).cloned());
            if let Some(info) = var_info {
                let mut field_spaces = Vec::with_capacity(info.fields.len());
                for f in info.fields.iter() {
                    let f_ty = f.declared_type.canonical_type().unwrap_or(ctx.store.unit());
                    field_spaces.push(build_initial_pattern_space(ctx, f_ty));
                }
                PatternSpace::Variant(VariantSpace {
                    variant: variant_id,
                    exact_case: scrutinee_ty,
                    fields: field_spaces.into_boxed_slice(),
                    proof: crate::match_semantics::BranchProofEnvironment::default(),
                })
            } else {
                PatternSpace::Opaque(scrutinee_ty)
            }
        }
        TypeData::Tuple(elements) => {
            let mut element_spaces = Vec::with_capacity(elements.len());
            for elem in elements.iter() {
                element_spaces.push(build_initial_pattern_space(ctx, elem.ty));
            }
            PatternSpace::Tuple(element_spaces.into_boxed_slice())
        }
        _ => {
            if let Some(origin) = ctx.store.nominal_origin_declaration(scrutinee_ty).cloned() {
                if let Some(enum_info) = ctx.enum_table.and_then(|t| t.enums.get(&origin).cloned()) {
                    let mut variant_spaces = Vec::new();
                    for variant_id in enum_info.variants.iter() {
                        let Some(v_info) = ctx.enum_table.and_then(|t| t.variants.get(variant_id).cloned()) else { continue; };
                        let gadt_res = crate::checker::gadt_proof::solve_gadt_branch_proof(
                            ctx.store,
                            &ctx.hierarchy,
                            &origin,
                            &v_info,
                            scrutinee_ty,
                        );
                        let (proof, exact_case) = match gadt_res {
                            crate::checker::gadt_proof::GadtProofResult::Reachable { proof, exact_case } => (proof, exact_case),
                            crate::checker::gadt_proof::GadtProofResult::Refuted => continue,
                        };

                        let mut field_spaces = Vec::with_capacity(v_info.fields.len());
                        for f in v_info.fields.iter() {
                            let f_ty = f.declared_type.canonical_type().unwrap_or(ctx.store.unit());
                            field_spaces.push(build_initial_pattern_space(ctx, f_ty));
                        }

                        variant_spaces.push(PatternSpace::Variant(VariantSpace {
                            variant: variant_id.clone(),
                            exact_case,
                            fields: field_spaces.into_boxed_slice(),
                            proof,
                        }));
                    }

                    if variant_spaces.is_empty() {
                        PatternSpace::Empty
                    } else if variant_spaces.len() == 1 {
                        variant_spaces.pop().unwrap().normalize()
                    } else {
                        PatternSpace::Union(variant_spaces.into_boxed_slice()).normalize()
                    }
                } else {
                    PatternSpace::Opaque(scrutinee_ty)
                }
            } else {
                PatternSpace::Opaque(scrutinee_ty)
            }
        }
    }
}

/// Evaluates exhaustiveness and arm usefulness across match branches.
pub fn evaluate_match_exhaustiveness(
    ctx: &mut CheckingContext<'_>,
    initial_space: &PatternSpace,
    arm_spaces: &[PatternSpace],
    arm_ranges: &[SourceRange],
    match_range: SourceRange,
) -> (ExhaustivenessResult, Vec<(PatternSpace, PatternSpace, PatternUsefulness)>) {
    let mut current_space = initial_space.clone();
    let mut arm_results = Vec::with_capacity(arm_spaces.len());

    for (i, arm_space) in arm_spaces.iter().enumerate() {
        let reachable = current_space.intersect(arm_space, ctx.store, &ctx.hierarchy);
        let usefulness = if reachable.is_empty() {
            ctx.emit_diagnostic(SemanticDiagnostic::warning_in(
                ctx.current_module.clone(),
                DiagnosticCode::MatchArmRedundant,
                "unreachable match arm: pattern value space has already been fully covered",
                arm_ranges[i],
            ));
            PatternUsefulness::Redundant
        } else {
            PatternUsefulness::Useful
        };

        let residual = current_space.subtract(arm_space, ctx.store, &ctx.hierarchy);
        arm_results.push((reachable, residual.clone(), usefulness));
        current_space = residual;
    }

    let exhaustiveness = if current_space.is_empty() {
        ExhaustivenessResult::Proven
    } else {
        let witnesses = generate_coverage_witnesses(&current_space);
        ctx.emit_diagnostic(SemanticDiagnostic::error_in(
            ctx.current_module.clone(),
            DiagnosticCode::MatchNonExhaustive,
            format!("non-exhaustive match expression: missing cases {:?}", witnesses),
            match_range,
        ));
        ExhaustivenessResult::Missing(witnesses.into_boxed_slice())
    };

    (exhaustiveness, arm_results)
}

fn generate_coverage_witnesses(space: &PatternSpace) -> Vec<CoverageWitness> {
    match space {
        PatternSpace::Empty => Vec::new(),
        PatternSpace::Opaque(ty) => vec![CoverageWitness::Opaque(*ty)],
        PatternSpace::Union(members) => {
            let mut witnesses = Vec::new();
            for m in members.iter() {
                witnesses.extend(generate_coverage_witnesses(m));
            }
            witnesses
        }
        PatternSpace::Variant(v) => {
            let mut field_witnesses = Vec::with_capacity(v.fields.len());
            for f in v.fields.iter() {
                let w = generate_coverage_witnesses(f).into_iter().next().unwrap_or(CoverageWitness::Wildcard);
                field_witnesses.push(w);
            }
            vec![CoverageWitness::Variant {
                variant: v.variant.clone(),
                exact_case: v.exact_case,
                fields: field_witnesses.into_boxed_slice(),
            }]
        }
        PatternSpace::Tuple(elements) => {
            let mut elem_witnesses = Vec::with_capacity(elements.len());
            for e in elements.iter() {
                let w = generate_coverage_witnesses(e).into_iter().next().unwrap_or(CoverageWitness::Wildcard);
                elem_witnesses.push(w);
            }
            vec![CoverageWitness::Tuple(elem_witnesses.into_boxed_slice())]
        }
        PatternSpace::List(_) => vec![CoverageWitness::List(Box::new([]))],
    }
}
