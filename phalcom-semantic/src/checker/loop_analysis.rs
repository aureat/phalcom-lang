//! Cyclic flow topology and bounded loop fixed-point analysis.

use crate::checker::context::CheckingContext;
use crate::checker::flow::state::{FlowInvariantFailure, FlowState};

pub(crate) const MAX_LOOP_FIXPOINT_ITERATIONS: usize = 8;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum LoopConvergence {
    Stable { iterations: u8 },
    Exhausted { iterations: u8 },
}

#[derive(Clone, Debug, Default)]
pub(crate) struct LoopStepResult {
    pub normal_backedge: Option<FlowState>,
    pub continues: Vec<FlowState>,
    #[allow(dead_code)]
    pub breaks: Vec<FlowState>,
}

impl LoopStepResult {
    pub(crate) fn backedge_states(&self) -> impl Iterator<Item = &FlowState> {
        self.normal_backedge.as_ref().into_iter().chain(self.continues.iter())
    }
}

#[derive(Clone, Debug)]
pub(crate) struct LoopFixpoint {
    pub header: FlowState,
    #[allow(dead_code)]
    pub convergence: LoopConvergence,
}

/// Solves loop cyclic flow facts to a bounded semantic fixed point.
pub(crate) fn solve_loop_header(
    ctx: &mut CheckingContext<'_>,
    entry: &FlowState,
    probe_iteration: impl FnMut(&mut CheckingContext<'_>, &FlowState) -> LoopStepResult,
) -> Result<LoopFixpoint, FlowInvariantFailure> {
    solve_loop_header_with_limit(ctx, entry, MAX_LOOP_FIXPOINT_ITERATIONS, probe_iteration)
}

/// Bounded fixed-point solver with custom iteration limit (used in production & tests).
pub(crate) fn solve_loop_header_with_limit(
    ctx: &mut CheckingContext<'_>,
    entry: &FlowState,
    limit: usize,
    mut probe_iteration: impl FnMut(&mut CheckingContext<'_>, &FlowState) -> LoopStepResult,
) -> Result<LoopFixpoint, FlowInvariantFailure> {
    if !entry.is_reachable() {
        return Ok(LoopFixpoint {
            header: entry.clone(),
            convergence: LoopConvergence::Stable { iterations: 0 },
        });
    }

    let mut header = entry.clone();
    let mut iterations = 0;

    for iter_idx in 1..=limit {
        iterations = iter_idx as u8;
        let mut step_res = None;
        let _ = ctx.run_flow_probe(header.clone(), |probe_ctx| {
            step_res = Some(probe_iteration(probe_ctx, &header));
        });
        let step = step_res.expect("probe must execute");

        let reachable_backedges: Vec<FlowState> = step.backedge_states().filter(|s| s.is_reachable()).cloned().collect();

        if reachable_backedges.is_empty() {
            // No backedges reach the header; entry header is stable
            return Ok(LoopFixpoint {
                header,
                convergence: LoopConvergence::Stable { iterations },
            });
        }

        let mut candidate_states = vec![entry.clone()];
        candidate_states.extend(reachable_backedges);

        let candidate = FlowState::join_with_hierarchy(&candidate_states, ctx.store, &ctx.hierarchy)?;
        let next_header = FlowState::widen_loop_state_with_hierarchy(&header, &candidate, ctx.store, &ctx.hierarchy)?;

        if next_header.fixpoint_key() == header.fixpoint_key() {
            return Ok(LoopFixpoint {
                header: next_header,
                convergence: LoopConvergence::Stable { iterations },
            });
        }

        header = next_header;
    }

    // Limit exhausted: run one more probe and weaken unstable dimensions
    let mut step_res = None;
    let _ = ctx.run_flow_probe(header.clone(), |probe_ctx| {
        step_res = Some(probe_iteration(probe_ctx, &header));
    });
    let step = step_res.expect("probe must execute");
    let reachable_backedges: Vec<FlowState> = step.backedge_states().filter(|s| s.is_reachable()).cloned().collect();

    let next_header = if !reachable_backedges.is_empty() {
        let mut candidate_states = vec![entry.clone()];
        candidate_states.extend(reachable_backedges);
        let candidate = FlowState::join_with_hierarchy(&candidate_states, ctx.store, &ctx.hierarchy)?;
        FlowState::widen_loop_state_with_hierarchy(&header, &candidate, ctx.store, &ctx.hierarchy)?
    } else {
        header.clone()
    };

    let weakened = FlowState::weaken_unstable_fixpoint_facts(&header, &next_header);
    Ok(LoopFixpoint {
        header: weakened,
        convergence: LoopConvergence::Exhausted { iterations },
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::declarations::DeclarationTypeTable;
    use crate::identity::DeclarationId;
    use crate::types::annotation::SimpleTypeResolver;
    use crate::types::evidence::{EvidenceOrigin, TypeKnowledge};
    use crate::types::relation::MapTypeHierarchy;
    use crate::types::store::TypeStore;
    use phalcom_common::range::SourceRange;
    use phalcom_modules::identity::ModuleId;

    #[test]
    fn solver_converges_on_two_step_widening() {
        let mut store = TypeStore::new();
        let hierarchy = MapTypeHierarchy::default();
        let resolver = SimpleTypeResolver::new();
        let declarations = DeclarationTypeTable::new();
        let mut ctx = CheckingContext::new(&mut store, &hierarchy, &resolver, &declarations, ModuleId::universe_root());

        let int_ty = ctx.store.nominal_type(DeclarationId::new(ModuleId::universe_root(), "Int".into()));
        let string_ty = ctx.store.nominal_type(DeclarationId::new(ModuleId::universe_root(), "String".into()));

        let b_id = ctx.alloc_binding();
        let mut entry = FlowState::new();
        entry.declare(
            b_id,
            "x",
            SourceRange::default(),
            None,
            TypeKnowledge::established(int_ty, EvidenceOrigin::Flow),
            true,
        );

        let fixpoint = solve_loop_header(&mut ctx, &entry, |_probe_ctx, current_header| {
            let mut back = current_header.clone();
            back.write(
                b_id,
                TypeKnowledge::established(string_ty, EvidenceOrigin::Flow),
                None,
                crate::checker::BindingConsistency::Unconstrained,
                crate::checker::CausalInvalidity::Clean,
            );
            LoopStepResult {
                normal_backedge: Some(back),
                continues: Vec::new(),
                breaks: Vec::new(),
            }
        })
        .expect("solver succeeds");

        assert!(matches!(fixpoint.convergence, LoopConvergence::Stable { .. }));
        let final_ty = fixpoint.header.get_binding(b_id).unwrap().current.ty().expect("final type");
        assert_ne!(final_ty, int_ty);
        assert_ne!(final_ty, string_ty);
    }

    #[test]
    fn breaks_do_not_feed_header() {
        let mut store = TypeStore::new();
        let hierarchy = MapTypeHierarchy::default();
        let resolver = SimpleTypeResolver::new();
        let declarations = DeclarationTypeTable::new();
        let mut ctx = CheckingContext::new(&mut store, &hierarchy, &resolver, &declarations, ModuleId::universe_root());

        let int_ty = ctx.store.nominal_type(DeclarationId::new(ModuleId::universe_root(), "Int".into()));
        let string_ty = ctx.store.nominal_type(DeclarationId::new(ModuleId::universe_root(), "String".into()));

        let b_id = ctx.alloc_binding();
        let mut entry = FlowState::new();
        entry.declare(
            b_id,
            "x",
            SourceRange::default(),
            None,
            TypeKnowledge::established(int_ty, EvidenceOrigin::Flow),
            true,
        );

        let fixpoint = solve_loop_header(&mut ctx, &entry, |_probe_ctx, current_header| {
            let mut brk = current_header.clone();
            brk.write(
                b_id,
                TypeKnowledge::established(string_ty, EvidenceOrigin::Flow),
                None,
                crate::checker::BindingConsistency::Unconstrained,
                crate::checker::CausalInvalidity::Clean,
            );
            LoopStepResult {
                normal_backedge: None,
                continues: Vec::new(),
                breaks: vec![brk],
            }
        })
        .expect("solver succeeds");

        assert_eq!(fixpoint.header.get_binding(b_id).unwrap().current.ty(), Some(int_ty));
    }

    #[test]
    fn unchanged_backedge_is_not_reported_as_progress() {
        let mut store = TypeStore::new();
        let hierarchy = MapTypeHierarchy::default();
        let resolver = SimpleTypeResolver::new();
        let declarations = DeclarationTypeTable::new();
        let mut ctx = CheckingContext::new(&mut store, &hierarchy, &resolver, &declarations, ModuleId::universe_root());
        let entry = FlowState::new();
        let mut probes = 0;

        let fixpoint = solve_loop_header_with_limit(&mut ctx, &entry, 8, |_probe_ctx, current_header| {
            probes += 1;
            LoopStepResult {
                normal_backedge: Some(current_header.clone()),
                continues: Vec::new(),
                breaks: Vec::new(),
            }
        })
        .expect("solver succeeds");

        assert_eq!(probes, 1);
        assert_eq!(fixpoint.convergence, LoopConvergence::Stable { iterations: 1 });
    }
}
