from pathlib import Path


def replace_once(path: str, old: str, new: str, label: str) -> None:
    p = Path(path)
    text = p.read_text()
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{label} anchor count: {count}")
    p.write_text(text.replace(old, new, 1))

# Preserve exact trait-dispatch terminal identity at the ordinary-dispatch boundary.
replace_once(
    "phalcom-semantic/src/trait_dispatch.rs",
    '''pub enum TraitDispatchResolution {
    Found(TraitDispatchSelection),
    Ambiguous(Box<[TraitEvidencedMemberCandidate]>),
    Missing,
    Incomplete(ImplId),
    Unknown(UnknownReason),
    Blocked(BlockReason),
    Dynamic(DynamicBoundaryObligation),
    Cancelled,
    BudgetExceeded(BudgetReport),
    InternalFailure(Box<str>),
}
''',
    '''pub enum TraitDispatchResolution {
    Found(TraitDispatchSelection),
    Ambiguous(Box<[TraitEvidencedMemberCandidate]>),
    Missing,
    Incomplete(ImplId),
    Unknown(UnknownReason),
    Blocked(BlockReason),
    Dynamic(DynamicBoundaryObligation),
    Cancelled,
    BudgetExceeded(BudgetReport),
    InternalFailure(Box<str>),
}

/// Terminal proof state retained when ordinary member discovery found a trait
/// candidate but could not produce executable conformance evidence. This is
/// intentionally distinct from runtime-dynamic dispatch.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TraitDispatchTerminal {
    Incomplete(ImplId),
    Unknown(UnknownReason),
    Blocked(BlockReason),
    Dynamic(DynamicBoundaryObligation),
    Cancelled,
    BudgetExceeded(BudgetReport),
    InternalFailure(Box<str>),
}
''',
    "trait terminal model",
)

replace_once(
    "phalcom-semantic/src/dispatch.rs",
    '''pub enum ResolvedDispatchResult {
    Found(Box<ResolvedDispatch>),
    Ambiguous(Vec<ResolvedDispatch>),
    Missing { visited_owners: Box<[DeclarationId]> },
    Dynamic,
}
''',
    '''pub enum ResolvedDispatchResult {
    Found(Box<ResolvedDispatch>),
    Ambiguous(Vec<ResolvedDispatch>),
    Missing { visited_owners: Box<[DeclarationId]> },
    /// A trait candidate existed, but its exact conformance proof terminated
    /// without an executable selection. Never reinterpret this as ordinary
    /// runtime-dynamic dispatch.
    TraitTerminal(crate::trait_dispatch::TraitDispatchTerminal),
    Dynamic,
}
''',
    "resolved dispatch terminal variant",
)
replace_once(
    "phalcom-semantic/src/dispatch.rs",
    '''            ResolvedDispatchResult::Missing { .. } => DispatchResult::Missing,
            ResolvedDispatchResult::Dynamic => DispatchResult::Dynamic,
''',
    '''            ResolvedDispatchResult::Missing { .. } => DispatchResult::Missing,
            ResolvedDispatchResult::TraitTerminal(_) | ResolvedDispatchResult::Dynamic => DispatchResult::Dynamic,
''',
    "surface resolver compatibility projection",
)

# Context records existing checker terminal statuses, while returning the exact
# trait terminal to expression analysis instead of flattening it to Dynamic.
replace_once(
    "phalcom-semantic/src/checker/context.rs",
    '''            ResolvedDispatchResult::Missing { visited_owners } => visited_owners.clone(),
            ResolvedDispatchResult::Dynamic => Box::new([]),
''',
    '''            ResolvedDispatchResult::Missing { visited_owners } => visited_owners.clone(),
            ResolvedDispatchResult::TraitTerminal(_) | ResolvedDispatchResult::Dynamic => Box::new([]),
''',
    "visited owners terminal arm",
)
replace_once(
    "phalcom-semantic/src/checker/context.rs",
    '''                        crate::trait_dispatch::TraitDispatchResolution::Missing => {}
                        crate::trait_dispatch::TraitDispatchResolution::Incomplete(_)
                        | crate::trait_dispatch::TraitDispatchResolution::Unknown(_)
                        | crate::trait_dispatch::TraitDispatchResolution::Blocked(_)
                        | crate::trait_dispatch::TraitDispatchResolution::Dynamic(_)
                        | crate::trait_dispatch::TraitDispatchResolution::Cancelled
                        | crate::trait_dispatch::TraitDispatchResolution::BudgetExceeded(_)
                        | crate::trait_dispatch::TraitDispatchResolution::InternalFailure(_) => {
                            return ResolvedDispatchResult::Dynamic;
                        }
''',
    '''                        crate::trait_dispatch::TraitDispatchResolution::Missing => {}
                        crate::trait_dispatch::TraitDispatchResolution::Incomplete(impl_id) => {
                            return ResolvedDispatchResult::TraitTerminal(crate::trait_dispatch::TraitDispatchTerminal::Incomplete(impl_id));
                        }
                        crate::trait_dispatch::TraitDispatchResolution::Unknown(reason) => {
                            return ResolvedDispatchResult::TraitTerminal(crate::trait_dispatch::TraitDispatchTerminal::Unknown(reason));
                        }
                        crate::trait_dispatch::TraitDispatchResolution::Blocked(reason) => {
                            self.record_call_status(AnalysisStatus::Blocked(reason.clone()));
                            return ResolvedDispatchResult::TraitTerminal(crate::trait_dispatch::TraitDispatchTerminal::Blocked(reason));
                        }
                        crate::trait_dispatch::TraitDispatchResolution::Dynamic(obligation) => {
                            self.record_call_status(AnalysisStatus::DynamicBoundary(crate::types::evidence::DynamicReason::RuntimeReflection));
                            return ResolvedDispatchResult::TraitTerminal(crate::trait_dispatch::TraitDispatchTerminal::Dynamic(obligation));
                        }
                        crate::trait_dispatch::TraitDispatchResolution::Cancelled => {
                            self.record_call_status(AnalysisStatus::Cancelled);
                            return ResolvedDispatchResult::TraitTerminal(crate::trait_dispatch::TraitDispatchTerminal::Cancelled);
                        }
                        crate::trait_dispatch::TraitDispatchResolution::BudgetExceeded(report) => {
                            self.record_call_status(AnalysisStatus::BudgetExceeded(report.clone()));
                            return ResolvedDispatchResult::TraitTerminal(crate::trait_dispatch::TraitDispatchTerminal::BudgetExceeded(report));
                        }
                        crate::trait_dispatch::TraitDispatchResolution::InternalFailure(message) => {
                            let incident = self.record_internal_incident(
                                InternalSemanticIncidentKind::DatabaseInvariantViolation,
                                InternalSemanticIncidentDetails::Message { message: message.clone() },
                                None,
                            );
                            self.record_call_status(AnalysisStatus::InternalFailure(incident));
                            return ResolvedDispatchResult::TraitTerminal(crate::trait_dispatch::TraitDispatchTerminal::InternalFailure(message));
                        }
''',
    "trait terminal preservation",
)
replace_once(
    "phalcom-semantic/src/checker/context.rs",
    '''            ResolvedDispatchResult::Dynamic => ResolvedDispatchResult::Dynamic,
        }
    }

    pub fn resolve_dispatch(&mut self, receiver: TypeId, selector: &Selector, lookup: crate::dispatch::DispatchLookup) -> DispatchResult {
''',
    '''            ResolvedDispatchResult::TraitTerminal(terminal) => ResolvedDispatchResult::TraitTerminal(terminal),
            ResolvedDispatchResult::Dynamic => ResolvedDispatchResult::Dynamic,
        }
    }

    pub fn resolve_dispatch(&mut self, receiver: TypeId, selector: &Selector, lookup: crate::dispatch::DispatchLookup) -> DispatchResult {
''',
    "resolved terminal passthrough",
)
replace_once(
    "phalcom-semantic/src/checker/context.rs",
    '''            ResolvedDispatchResult::Missing { .. } => DispatchResult::Missing,
            ResolvedDispatchResult::Dynamic => DispatchResult::Dynamic,
''',
    '''            ResolvedDispatchResult::Missing { .. } => DispatchResult::Missing,
            ResolvedDispatchResult::TraitTerminal(_) | ResolvedDispatchResult::Dynamic => DispatchResult::Dynamic,
''',
    "context compatibility projection",
)
