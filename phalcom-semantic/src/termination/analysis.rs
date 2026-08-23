//! Complete termination analysis combining CFG acyclicity, ranking measures,
//! native trust, and dependency graphs.

use super::cfg::check_cfg_acyclicity;
use super::{TerminationBlockedReason, TerminationEvidence, TerminationKnowledge};
use crate::checker::analysis::CallableAnalysis;
use crate::checker::flow::graph::FlowGraph;
use phalcom_native_meta::primitive::TerminationSpec;

pub fn analyze_callable_termination(flow_graph: Option<&FlowGraph>, analysis: &CallableAnalysis, native_spec: Option<TerminationSpec>) -> TerminationKnowledge {
    if let Some(spec) = native_spec {
        return match spec {
            TerminationSpec::Terminates => TerminationKnowledge::Proven(TerminationEvidence::TrustedNative),
            TerminationSpec::MayDiverge => TerminationKnowledge::Blocked(TerminationBlockedReason::OpaqueNative),
            TerminationSpec::Unknown => TerminationKnowledge::Blocked(TerminationBlockedReason::OpaqueNative),
        };
    }

    if let Some(graph) = flow_graph {
        if let Some(evidence) = check_cfg_acyclicity(graph) {
            // If CFG is acyclic and not self-recursive
            let is_recursive = analysis.dependencies.contains(&analysis.callable);
            if !is_recursive {
                return TerminationKnowledge::Proven(evidence);
            } else {
                return TerminationKnowledge::Blocked(TerminationBlockedReason::UnsupportedRecursionPattern);
            }
        }
    }

    TerminationKnowledge::Blocked(TerminationBlockedReason::UnprovenLoop)
}
