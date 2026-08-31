//! Unified core integration tests.
//!
//! Modules are grouped by semantic responsibility. Runtime tests consume
//! compiler output; they do not recreate semantic match proofs.

#[path = "language/mod.rs"]
mod language;

#[path = "../native_adt_runtime.rs"]
mod native_adt_runtime;

#[path = "compiler/semantic_boundary.rs"]
mod compiler_semantic_boundary;
#[path = "modules/compile.rs"]
mod modules_compile;
#[path = "modules/linking.rs"]
mod modules_linking;
#[path = "modules/package_info.rs"]
mod modules_package_info;
#[path = "modules/project_reflection.rs"]
mod modules_project_reflection;
#[path = "modules/reflection.rs"]
mod modules_reflection;
#[path = "modules/universe.rs"]
mod modules_universe;
#[path = "native/contracts.rs"]
mod native_surface_contracts;
#[path = "object_model/invariants.rs"]
mod object_model_invariants;
#[path = "observability/diagnostic_cli.rs"]
mod observability_diagnostic_cli;
#[path = "reflection/reflection.rs"]
mod reflection;
#[path = "reflection/census.rs"]
mod reflection_census;
#[path = "reflection/conformance.rs"]
mod reflection_conformance;
#[path = "reflection/type_metadata.rs"]
mod type_metadata_invariants;

#[path = "compiler/contract_metadata.rs"]
mod compiler_contract_metadata;
#[path = "compiler/declaration_dispatch.rs"]
mod compiler_declaration_dispatch;
#[path = "collections/contract.rs"]
mod core_collections;
#[path = "collections/outgoing_packs.rs"]
mod core_outgoing_packs;
#[path = "collections/outgoing_packs_completion.rs"]
mod core_outgoing_packs_completion;
#[path = "collections/symbol_selector.rs"]
mod core_symbol_selector;
#[path = "execution/depth_limits.rs"]
mod execution_depth_limits;
#[path = "execution/family_runtime.rs"]
mod execution_family_runtime;
#[path = "execution/repair_regressions.rs"]
mod execution_repair_regressions;
#[path = "execution/send_arity.rs"]
mod execution_send_arity;
#[path = "memory/gc.rs"]
mod memory_gc;
#[path = "memory/pack_gc.rs"]
mod memory_pack_gc;
#[path = "modules/runtime.rs"]
mod modules_runtime;
#[path = "observability/disassembly.rs"]
mod observability_disassembly;
#[path = "observability/exit_codes.rs"]
mod observability_exit_codes;
#[path = "observability/fiber_trace.rs"]
mod observability_fiber_trace;
#[path = "observability/super_disassembly.rs"]
mod observability_super_disassembly;
#[path = "observability/traceback.rs"]
mod observability_traceback;
#[path = "repl/immutability.rs"]
mod repl_immutability;
#[path = "repl/session.rs"]
mod repl_session;
#[path = "repl/source_maps.rs"]
mod repl_source_maps;
