//! Source-language, compiler, ADT, and vertical integration tests.
//!
//! This is a semantic submodule of the unified `core` integration binary.
//! Files are grouped by language responsibility while retaining focused test
//! filtering through their module paths.

#[path = "../../support/mod.rs"]
mod support;

#[path = "../../support/vm.rs"]
mod vm_support;

#[path = "corpus.rs"]
mod corpus;

#[path = "algebraic_data/associated_gc.rs"]
mod algebraic_data_associated_gc;
#[path = "algebraic_data/associated_reification.rs"]
mod algebraic_data_associated_reification;
#[path = "algebraic_data/associated_runtime.rs"]
mod algebraic_data_associated_runtime;
#[path = "algebraic_data/behavior.rs"]
mod algebraic_data_behavior;
#[path = "algebraic_data/conformance.rs"]
mod algebraic_data_conformance;
#[path = "algebraic_data/construction_primitives.rs"]
mod algebraic_data_construction_primitives;
#[path = "algebraic_data/execution.rs"]
mod algebraic_data_execution;
#[path = "algebraic_data/gc.rs"]
mod algebraic_data_gc;
#[path = "algebraic_data/gc_scenarios.rs"]
mod algebraic_data_gc_scenarios;
#[path = "algebraic_data/pattern_context.rs"]
mod algebraic_data_pattern_context;
#[path = "algebraic_data/scenarios.rs"]
mod algebraic_data_scenarios;
#[path = "compiler/associated_lowering.rs"]
mod compiler_associated_lowering;
#[path = "compiler/lowering.rs"]
mod compiler_lowering;
#[path = "compiler/lowering_scenarios.rs"]
mod compiler_lowering_scenarios;

#[path = "golden.rs"]
mod golden;
#[path = "numeric_diagnostics.rs"]
mod numeric_diagnostics;
#[path = "numeric_values.rs"]
mod numeric_values;
#[path = "option.rs"]
mod option;
