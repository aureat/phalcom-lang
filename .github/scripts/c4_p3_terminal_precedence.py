from pathlib import Path

path = Path('phalcom-semantic/src/trait_dispatch.rs')
text = path.read_text()
old = '''fn retain_terminal(slot: &mut Option<TraitDispatchResolution>, candidate: TraitDispatchResolution) {
    if slot.is_none() {
        *slot = Some(candidate);
    }
}
'''
new = '''fn retain_terminal(slot: &mut Option<TraitDispatchResolution>, candidate: TraitDispatchResolution) {
    let replace = slot
        .as_ref()
        .is_none_or(|current| trait_dispatch_terminal_rank(&candidate) > trait_dispatch_terminal_rank(current));
    if replace {
        *slot = Some(candidate);
    }
}

/// Stable proof-state precedence for a bucket that produced no executable
/// trait candidate. A proven invalid/incomplete conformance must never be
/// hidden by an unrelated unresolved candidate, and control/solver terminal
/// states must remain stronger than ordinary lack of evidence.
fn trait_dispatch_terminal_rank(resolution: &TraitDispatchResolution) -> u8 {
    match resolution {
        TraitDispatchResolution::Incomplete(_) => 100,
        TraitDispatchResolution::InternalFailure(_) => 90,
        TraitDispatchResolution::Cancelled => 80,
        TraitDispatchResolution::BudgetExceeded(_) => 70,
        TraitDispatchResolution::Blocked(_) => 60,
        TraitDispatchResolution::Dynamic(_) => 50,
        TraitDispatchResolution::Unknown(_) => 40,
        TraitDispatchResolution::Found(_)
        | TraitDispatchResolution::Ambiguous(_)
        | TraitDispatchResolution::Missing => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::{retain_terminal, TraitDispatchResolution};
    use crate::identity::{ImplId, ImplLocalId};
    use crate::types::evidence::UnknownReason;
    use phalcom_modules::identity::{ModuleId, ModulePath, ResolvedProjectId};

    fn impl_id(local: u32) -> ImplId {
        ImplId::new(
            ModuleId::resolved(ResolvedProjectId::from_raw(77), ModulePath::root()),
            ImplLocalId(local),
        )
    }

    #[test]
    fn proven_incomplete_terminal_outranks_unknown_regardless_of_candidate_order() {
        let mut unknown_first = None;
        retain_terminal(
            &mut unknown_first,
            TraitDispatchResolution::Unknown(UnknownReason::NoTypeEvidence),
        );
        retain_terminal(
            &mut unknown_first,
            TraitDispatchResolution::Incomplete(impl_id(1)),
        );
        assert!(matches!(unknown_first, Some(TraitDispatchResolution::Incomplete(_))));

        let mut incomplete_first = None;
        retain_terminal(
            &mut incomplete_first,
            TraitDispatchResolution::Incomplete(impl_id(2)),
        );
        retain_terminal(
            &mut incomplete_first,
            TraitDispatchResolution::Unknown(UnknownReason::NoTypeEvidence),
        );
        assert!(matches!(incomplete_first, Some(TraitDispatchResolution::Incomplete(_))));
    }

    #[test]
    fn internal_failure_terminal_outranks_plain_unknown() {
        let mut terminal = None;
        retain_terminal(
            &mut terminal,
            TraitDispatchResolution::Unknown(UnknownReason::NoTypeEvidence),
        );
        retain_terminal(
            &mut terminal,
            TraitDispatchResolution::InternalFailure("broken evidence product".into()),
        );
        assert!(matches!(terminal, Some(TraitDispatchResolution::InternalFailure(_))));
    }
}
'''
if text.count(old) != 1:
    raise SystemExit(f'retain_terminal anchor count: {text.count(old)}')
path.write_text(text.replace(old, new, 1))
