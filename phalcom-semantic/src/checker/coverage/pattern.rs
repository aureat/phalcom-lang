//! Query-local coverage pattern arena for finite symbolic elimination.

use crate::identity::VariantId;
use crate::types::id::TypeId;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) struct CoveragePatternId(pub(crate) u32);

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum CoveragePattern {
    Wildcard,

    Variant {
        candidates: Box<[VariantId]>,
        exact_cases: Box<[TypeId]>,
        fields: Box<[CoveragePatternId]>,
    },

    Or(Box<[CoveragePatternId]>),

    Tuple(Box<[CoveragePatternId]>),

    List {
        prefix: Box<[CoveragePatternId]>,
        rest: Option<CoveragePatternId>,
    },

    RecordPredicate,
    MapPredicate,
}

#[derive(Clone, Debug)]
pub(crate) struct CoveragePatternArena {
    nodes: Vec<CoveragePattern>,
    wildcard: CoveragePatternId,
}

impl Default for CoveragePatternArena {
    fn default() -> Self {
        let mut nodes = Vec::with_capacity(16);
        nodes.push(CoveragePattern::Wildcard);
        Self {
            nodes,
            wildcard: CoveragePatternId(0),
        }
    }
}

impl CoveragePatternArena {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn wildcard(&self) -> CoveragePatternId {
        self.wildcard
    }

    pub(crate) fn alloc(&mut self, pat: CoveragePattern) -> CoveragePatternId {
        let id = CoveragePatternId(self.nodes.len() as u32);
        self.nodes.push(pat);
        id
    }

    pub(crate) fn get(&self, id: CoveragePatternId) -> &CoveragePattern {
        &self.nodes[id.0 as usize]
    }
}
