pub(crate) mod domain;
pub(crate) mod pattern;
pub(crate) mod subject;
pub(crate) mod usefulness;

#[allow(unused_imports)]
pub(crate) use domain::{ConstructorCase, ConstructorHead, DomainDecomposition, OpenedVariantCase, decompose_domain, open_variant_case};
#[allow(unused_imports)]
pub(crate) use pattern::{CoveragePattern, CoveragePatternArena, CoveragePatternId};
#[allow(unused_imports)]
pub(crate) use subject::CoverageSubject;
#[allow(unused_imports)]
pub(crate) use usefulness::{CoverageEngine, summarize_pattern};
