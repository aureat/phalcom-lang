//! Canonical integration-test binary for `phalcom-semantic`.

#[path = "semantic/mod.rs"]
mod semantic;

#[path = "constructor_factory_probe.rs"]
mod constructor_factory_probe;

#[path = "module_query_provenance.rs"]
mod module_query_provenance;

#[path = "canonical_parameter_advisory.rs"]
mod canonical_parameter_advisory;

#[path = "explanation_derivation.rs"]
mod explanation_derivation;

#[path = "checkpoint_a4.rs"]
mod checkpoint_a4;
