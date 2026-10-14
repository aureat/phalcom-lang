# IMPLEMENTATION INCIDENT — PERF003.C2.P5 canonical trait evidence slot

## 1. Identity

- Program/checkpoint: `PERF003` / `PERF003.C2`.
- Plan/task: `PERF003.C2.P5`, canonical publication and P4-dependent metadata authority gate.
- Branch/worktree: `perf/universe-characterization` / `.worktrees/perf-universe-characterization`.
- Starting committed revision: `55452273`; PR-B source is uncommitted.

## 2. Escalation trigger

Generic-evidence ownership across semantic publication and executable lowering is
nonlocal. A missing slot cannot be repaired by fabricating one in lowering without
potentially strengthening evidence (AGENTS.md mandatory trigger 11; protocol
trigger 15). Stop on this issue until the canonical owner of the absent
conformance-owner constraints is determined.

## 3. Required invariant

P5 compiler completeness includes canonical trait/conformance evidence required
for new code and for canonical executable lowering (INV-P5-02). P4.T6 and
P5.T10 require actual compilable canonical `RuntimePublic` metadata, not merely
a well-formed metadata schema. Assumed conformance invocations must resolve an
existing checked generic-evidence slot; lowering must not invent proof facts.

## 4. Expected architecture

`canonical_universe_formal_image()` publishes source-backed trait surfaces,
conformance/derived rules and callable analyses. `ProgramCompiler` projects the
same snapshot. Lowering composes the conformance owner's generic constraints
with callable constraints to build a `GenericEvidenceEnvironment` and locates
the `Zero.zero` slot for `T.zero` inside a generic conformance method.

## 5. Observed behavior / minimal reproduction

```sh
RUSTFLAGS='' cargo test -p phalcom-core --lib canonical_universe_program_is_process_singleton
```

One selected test fails after ~41–48 s during lowering in
`universe:numbers.complex`: missing class-side `Zero.zero` (a reflection
integration run instead reported `One.one`). The earlier `FixedFields`
metadata failure is not reached. `RUSTFLAGS='' cargo test -p phalcom-semantic
--lib canonical_compiler_image_retains_executable_numeric_trait_requirements`
passes one selected test: both exact class-side requirements are present in
the canonical formal snapshot's trait surfaces.

## 6. Relevant path and evidence

```text
canonical_universe.rs::build_canonical_universe_program
  -> canonical_universe_formal_image (same snapshot retained)
  -> ProgramCompiler::compile_analyzed_for_canonical_bootstrap
  -> semantic_lowering.rs::project_module_lowering
     -> owner_sig for CallableOwnerId::Conformance
     -> GenericEvidenceEnvironment::try_from_constraints / try_compose
     -> TraitDispatchSite::Assumed -> generic_evidence.slot_for -> None
```

Temporary, subsequently removed, diagnostic instrumentation identified the
exact failing branch as `Assumed/evidence`, not the trait-surface requirement
slot map. At the failure the callable is the class-side `along(_,_)` of
`numbers.complex` impl local ID 8; `owner_sig` is `None`,
`generic_evidence.len() == 0`, snapshot witness-plan count is 0 and derived-rule
count is 1. The site requests class-side `Zero.zero`. The semantic canonical
surface test establishes that the requirement itself exists. No source
correction for this failure was attempted.

## 7. Scoped implementation diff

Uncommitted P4.PR-B changes extend Record metadata schema/export, contextual
legacy decode, runtime typing and tests. One P5 semantic discriminator test
asserts the `Zero`/`One` class-side surfaces. There is **no** lasting change
to `semantic_lowering.rs`. The failure occurs before metadata export and is
not explained by PR-B's schema or runtime consumer edits.

## 8. Causal hypothesis and alternative

The conformance callable loses the owning generic constraints in the canonical
snapshot: neither its witness-plan lookup nor the matching derived-rule lookup
supplies an owner signature, leaving a checked assumed `T |= Zero` call without
an executable slot. Possible causes include canonical witness-plan publication
coverage or a valid conformance category not represented by either current
lookup. An unrelated snapshot/analysis mismatch at this lowering site remains
possible; the surface result rules out simply missing `Zero` from the trait
surface. No inference-authority change is assumed.

## 9. Decision requested

Which canonical semantic product owns the generic constraints for this
`CallableOwnerId::Conformance` (`numbers.complex` impl local ID 8), and should
P5 extend the existing witness-plan/derived-rule publication or an existing
canonical conformance-owned view to provide it? Preserve checked evidence and
incremental/cold parity; avoid a lowering-only synthetic constraint source.

## 10. Root cause and resolution

In commit `c34f9c3fee75f33304a85c229dcf0a5133816479`, `publish_conformance_witness_plans`
in `session.rs` was updated to skip Universe conformances:
`if contribution.source_module.project.is_universe() { continue; }`.
This check was intended for overlay sessions that inherit the completed canonical
image, but it unconditionally skipped Universe conformances during the canonical
image build itself (`db.canonical_image().is_none()`). As a result, the canonical
image snapshot had 0 conformance witness plans.

Resolution:
Constrain the skip gate in `session.rs`:
`if db.canonical_image().is_some() && contribution.source_module.project.is_universe() { continue; }`.

During canonical image construction, all Universe conformance witness plans are
published and stored in the canonical image snapshot. In subsequent overlay
sessions, Universe witness plans are inherited without recomputation.

Verification:
- `canonical_universe_program_is_process_singleton`: PASS (1 selected, 43.19 s).
- `modules::canonical_universe::tests`: PASS (6 selected).
- `canonical_image::tests`: PASS (10 selected, 42.56 s).
- `semantic::integration::canonical_image`: PASS (4 selected, 130.56 s).
- `applied_record_row_metadata_display_preserves_field_types_and_tail`: PASS.

