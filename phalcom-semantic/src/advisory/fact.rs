//! Advisory fact metadata and bounded provenance.

use std::cmp::Ordering;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use crate::advisory::{AdvisoryOrigin, ValueShape};
use crate::db::ProductFingerprint;

/// Small literal payload retained beside an advisory shape.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum AdvisoryLiteral {
    /// Known boolean literal.
    Bool(bool),
}

/// Trust level for an advisory prediction. This is separate from formal
/// `EvidenceStatus` and cannot upgrade or repair formal facts.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum AdvisoryConfidence {
    /// Direct syntax or exact semantic guarantee.
    Exact,
    /// Local flow or binding propagation.
    Flow,
    /// Cross-call or cross-file propagation.
    Interprocedural,
    /// Structural use-site heuristic.
    Heuristic,
}

impl AdvisoryConfidence {
    /// Joins confidence conservatively, retaining weakest evidence.
    pub fn join(self, other: Self) -> Self {
        self.max(other)
    }
}

/// Runtime-shape fact with bounded canonical provenance.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdvisoryFact {
    /// Advisory runtime shape.
    pub shape: ValueShape,
    /// Optional literal refinement.
    pub literal: Option<AdvisoryLiteral>,
    /// Advisory evidence strength.
    pub confidence: AdvisoryConfidence,
    /// At most four deterministic representative origins.
    pub provenance: Vec<AdvisoryOrigin>,
}

impl AdvisoryFact {
    /// Computes a deterministic semantic fingerprint for this advisory fact.
    pub fn fingerprint(&self) -> ProductFingerprint {
        let mut hasher = DefaultHasher::new();
        self.shape.hash(&mut hasher);
        self.literal.hash(&mut hasher);
        self.confidence.hash(&mut hasher);
        self.provenance.hash(&mut hasher);
        ProductFingerprint::new(hasher.finish())
    }

    /// Creates a fact without assuming that a source site is available.
    pub fn new(shape: ValueShape, confidence: AdvisoryConfidence) -> Self {
        Self {
            shape: shape.canonicalize(),
            literal: None,
            confidence,
            provenance: Vec::new(),
        }
    }

    /// Creates an unknown heuristic fact.
    pub fn unknown() -> Self {
        Self {
            shape: ValueShape::Unknown,
            literal: None,
            confidence: AdvisoryConfidence::Heuristic,
            provenance: Vec::new(),
        }
    }

    /// Creates an exact syntax fact.
    pub fn exact(shape: ValueShape, origin: AdvisoryOrigin) -> Self {
        Self {
            shape: shape.canonicalize(),
            literal: None,
            confidence: AdvisoryConfidence::Exact,
            provenance: vec![origin],
        }
    }

    /// Creates an exact boolean syntax fact.
    pub fn exact_boolean(value: bool, shape: ValueShape, origin: AdvisoryOrigin) -> Self {
        Self {
            shape: shape.canonicalize(),
            literal: Some(AdvisoryLiteral::Bool(value)),
            confidence: AdvisoryConfidence::Exact,
            provenance: vec![origin],
        }
    }

    /// Creates a local-flow fact.
    pub fn flow(shape: ValueShape, origin: AdvisoryOrigin) -> Self {
        Self {
            shape: shape.canonicalize(),
            literal: None,
            confidence: AdvisoryConfidence::Flow,
            provenance: vec![origin],
        }
    }

    /// Creates an interprocedural fact.
    pub fn interprocedural(shape: ValueShape, origin: AdvisoryOrigin) -> Self {
        Self {
            shape: shape.canonicalize(),
            literal: None,
            confidence: AdvisoryConfidence::Interprocedural,
            provenance: vec![origin],
        }
    }

    /// Joins two facts while retaining a deterministic provenance sample.
    pub fn join(&self, other: &Self) -> Self {
        let mut provenance = self.provenance.iter().chain(other.provenance.iter()).cloned().collect::<Vec<_>>();
        provenance.sort_by(compare_origins);
        provenance.dedup();
        provenance.truncate(4);
        Self {
            shape: self.shape.join(&other.shape),
            literal: match (self.literal, other.literal) {
                (Some(left), Some(right)) if left == right => Some(left),
                _ => None,
            },
            confidence: self.confidence.join(other.confidence),
            provenance,
        }
    }

    /// Adds one causal origin and weakens confidence to the requested level.
    pub fn derive(mut self, confidence: AdvisoryConfidence, origin: AdvisoryOrigin) -> Self {
        self.confidence = self.confidence.join(confidence);
        self.provenance.push(origin);
        self.provenance.sort_by(compare_origins);
        self.provenance.dedup();
        self.provenance.truncate(4);
        self
    }
}

fn compare_origins(left: &AdvisoryOrigin, right: &AdvisoryOrigin) -> Ordering {
    fn site(origin: &AdvisoryOrigin) -> Option<(&crate::identity::SourceSiteId, u8)> {
        match origin {
            AdvisoryOrigin::Syntax(site) => Some((site, 0)),
            AdvisoryOrigin::Binding(site) => Some((site, 1)),
            AdvisoryOrigin::CallSite(site) => Some((site, 2)),
            AdvisoryOrigin::Constraint(site) => Some((site, 3)),
            _ => None,
        }
    }

    match (site(left), site(right)) {
        (Some((left_site, left_kind)), Some((right_site, right_kind))) => left_site.cmp(right_site).then(left_kind.cmp(&right_kind)),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => left.cmp(right),
    }
}
