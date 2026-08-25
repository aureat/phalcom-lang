//! Canonical provenance for advisory runtime-shape facts.

use crate::identity::{CallableId, FieldId, SourceSiteId};
use crate::presentation::FormalFactRef;

/// Origin of one advisory fact, keyed by compiler-owned identity.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum AdvisoryOrigin {
    /// Exact literal or syntax evidence.
    Syntax(SourceSiteId),
    /// Binding initializer or reassignment evidence.
    Binding(SourceSiteId),
    /// Callable summary evidence.
    Callable(CallableId),
    /// Resolved call-site evidence.
    CallSite(SourceSiteId),
    /// Structural use-site constraint.
    Constraint(SourceSiteId),
    /// Field observation.
    Field(FieldId),
    /// One-way formal-to-advisory seed.
    FormalFact(FormalFactRef),
}
