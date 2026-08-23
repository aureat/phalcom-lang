//! Ranking measures for loop / recursion termination.

use super::TerminationEvidence;
use crate::identity::BindingId;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RankingMeasure {
    IntegerDecreasing(BindingId),
    CollectionDecreasing(BindingId),
    StructuralDecreasing(BindingId),
}

impl RankingMeasure {
    pub fn to_evidence(&self) -> TerminationEvidence {
        match self {
            Self::IntegerDecreasing(b) => TerminationEvidence::IntegerDecreasing(*b),
            Self::CollectionDecreasing(b) => TerminationEvidence::CollectionDecreasing(*b),
            Self::StructuralDecreasing(b) => TerminationEvidence::StructuralDecreasing(*b),
        }
    }
}
