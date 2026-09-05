pub(crate) mod bindings;
pub(crate) mod conformance;
pub(crate) mod diagnostics;
pub(crate) mod exhaustiveness;
pub(crate) mod flow;
pub(crate) mod gadt_refinement;
pub(crate) mod patterns;
pub(crate) mod recursive_coverage;
pub(crate) mod resolution;

// Flow-specific tests are intentionally kept in their own responsibility
// module even before the source fixture matrix is complete.
