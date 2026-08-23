//! Interprocedural effect SCC fixpoint inference.

use super::infer::infer_intraprocedural_effects;
use super::summary::EffectKnowledge;
use crate::checker::analysis::CallableAnalysis;
use crate::identity::CallableId;
use std::collections::HashMap;

/// Computes the interprocedural effect knowledge for a set of analyzed callables.
pub fn infer_interprocedural_effects_scc(analyses: &HashMap<CallableId, CallableAnalysis>) -> HashMap<CallableId, EffectKnowledge> {
    let mut summaries: HashMap<CallableId, EffectKnowledge> = HashMap::new();

    // Seed intraprocedural effects
    for (id, analysis) in analyses {
        let intra = infer_intraprocedural_effects(analysis);
        summaries.insert(id.clone(), intra);
    }

    // Fixpoint propagation over call graph dependencies
    let mut changed = true;
    let mut iterations = 0;
    let max_iterations = 1000;

    while changed && iterations < max_iterations {
        changed = false;
        iterations += 1;

        for (caller_id, analysis) in analyses {
            let mut current = summaries.get(caller_id).cloned().unwrap_or(EffectKnowledge::PURE);

            for dep_id in analysis.dependencies.iter() {
                if let Some(dep_effect) = summaries.get(dep_id) {
                    let joined = current.join(dep_effect);
                    if joined != current {
                        current = joined;
                        changed = true;
                    }
                }
            }

            summaries.insert(caller_id.clone(), current);
        }
    }

    summaries
}
