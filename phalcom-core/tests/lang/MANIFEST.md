# Phalcom Language Corpus Manifest

Spec-conformance corpus for end-to-end CLI validation, organized by label
directory. Each case is a `<name>.ph` plus a sibling `<name>.expected`.

## Model

- **PASS** — feature works today; `.expected` is the exact verified stdout. Runs by
  default (`check_pass`).
- **NEGATIVE** — malformed input; must exit non-zero with the `.expected` diagnostic
  as a substring, never panic. Runs by default (`check_negative`).
- **PENDING** — a spec target the language does not implement yet. Lives in
  `<label>/pending/`; `.expected` pins the *intended* spec output, not today's
  behavior. Wired as `#[ignore]` `check_pending`, so it stays out of the green run and
  graduates to PASS when the feature lands.

## Summary

- **Labels:** absence, arithmetic, bindings, blocks, booleans, classes, collections,
  compile-errors, concurrency, control-flow, dispatch, errors, functions, inheritance,
  iteration, lexical, list, messages, metaclass, runtime-errors, syntax-errors, system.
- **Case counts:** PASS 163 · NEGATIVE 34 · PENDING 32 · **total 229** (U13
  hierarchy-stability policy, DEC-U13a=A: +1 NEGATIVE —
  `runtime-errors/runtime_error_superclass_reparent_rejected.ph` pins the
  sealed-hierarchy reject, `Can't set superclass of a class`, as a catchable
  runtime error rather than a panic, never a mutation of the class graph.
  Prior to that, +3 PASS —
  U-FUTURE Slice A (pure `.ph` settle-once `Future`; `docs/forge/units/U-FUTURE/plan.md`):
  `concurrency/concurrency_future_value_error_isready.ph` (C-FUT-1 settled half,
  C-FUT-8), `concurrency/concurrency_future_settle_once.ph` (C-FUT-3), and
  `concurrency/concurrency_future_then_map_catch_settled.ph` (C-FUT-4 settled-only
  half); the suspending half — `async`/`await`, pending-continuation drain — is
  Slice B, gated on DEC-FUT-SCHED and unowned `U-SCHED`, and stays out of this
  corpus for now (see `concurrency/pending/concurrency_future_async_await.ph`);
  the U-ITER deferred item 5 fiber x generator fixtures `iteration/for_generator_suspends.ph`
  (C-ITER-8) and `concurrency/each_generator_raises.ph`, cut from U-ITER and graduated now
  that U-FIBER has landed, over the `U-COLLTYPES` `{k:v}`-map-literal wiring, landed after
  Phase 3; +1 PASS /
  -1 NEGATIVE — `negative/map_literal_pending.ph` retired (the map literal no
  longer raises a "pending" diagnostic) and replaced by
  `map_literal_construction.ph`. Phase 3 (`Range`) was +1 PASS over Phase 2
  (new `range_basics.ph`) — Phase 2 was +2 PASS / -1 PENDING over Phase 1
  (`literal_tuple.ph` graduated `pending/` → PASS plus a new `tuple_basics.ph`)
  — Phase 1 was +1 PASS / +1 NEGATIVE over the `U-COLL` collection-literal
  landing, itself +4 PASS / +2 NEGATIVE / +2 PENDING over the
  `U-INH`/`U-ITER`/`U-FIBER` three-worktree consolidation).
- Active suites (`cargo test -p phalcom-core --test lang`) are green; PENDING run only
  under `-- --ignored` and are expected to fail until their feature is implemented.
- Baseline recorded 2026-07-11 against `./target/debug/phalcom` at commit `037da3d`; the
  `absence` lane was reconciled at the U6 landing (`51f56e4`) — +7 PASS cases graduated
  (empty/value-less block & method bodies, false `ifTrue` branch, `print` result, root
  superclass, empty `match` none-branch, empty block call → all `<None instance>`).

## Label matrix

| Label | PASS | NEG | PEND | Harness | Spec anchor |
|---|---:|---:|---:|---|---|
| arithmetic | 12 | – | – | `check_pass` | values-and-absence.md; messages-and-selectors.md; control-flow.md |
| lexical | 10 | – | 7 | `check_pass` + `check_pending` | lexical-structure.md; values-and-absence.md; selectors.md |
| classes | 10 | – | 10 | `check_pass` + `check_pending` | classes.md; object-model.md; ADR-0011; ADR-0017 |
| inheritance | 8 | – | – | `check_pass` | object-model.md §5.1; method-lookup.md §1.14; ADR-0002; ADR-0040 |
| messages | 7 | – | 2 | `check_pass` + `check_pending` | messages-and-selectors.md; selectors.md; object-model.md |
| system | 4 | – | 2 | `check_pass` + `check_pending` | system.md |
| bindings | 3 | – | 2 | `check_pass` + `check_pending` | values-and-absence.md; open-questions.md; ADR-0014 |
| control-flow | 3 | – | 5 | `check_pass` + `check_pending` | control-flow.md; blocks.md |
| dispatch | 3 | – | 5 | `check_pass` + `check_pending` | messages-and-selectors.md; method-lookup.md; object-model.md |
| metaclass | 2 | – | 1 | `check_pass` + `check_pending` | object-model.md |
| list | 4 | – | – | `check_pass` | U-LIST-plan.md; ADR-0019; ADR-0020 |
| collections | 14 | 2 | 1 | `check_pass` (+ `check_negative`, `check_pending`) | U-CORE-5 as-built.md; U-COLL: lexical-structure.md §4/§6/§7/§8; ADR-0029; ADR-0032; U-COLLTYPES: map-and-set.md; tuple-and-range.md; ADR-0039 |
| iteration | 9 | – | 2 | `check_pass` (+ `iteration_disasm`, `check_pending`) | ADR-0035; iteration.md; U-ITER specification |
| syntax-errors | – | 5 | – | `check_negative` | lexical-structure.md; implementation-status.md |
| runtime-errors | – | 11 | – | `check_negative` | messages-and-selectors.md; method-lookup.md; U-LIST-plan.md §3; ADR-0026; ADR-0041 |
| compile-errors | – | 12 | – | `check_negative` | values-and-absence.md; ADR-0014; ADR-0007; ADR-0021; object-model.md §5.1; ADR-0035 (break/continue outside loop) |
| absence | 10 | – | 5 | `check_pass` + `check_pending` | values-and-absence.md; ADR-0007; ADR-0021; selectors.md |
| blocks | – | – | 3 | `check_pending` | blocks.md; functions.md |
| booleans | – | – | 2 | `check_pending` | control-flow.md |
| concurrency | 9 | – | 1 | `check_pass` + `check_pending` | concurrency.md; ADR-0030 |
| errors | – | – | 2 | `check_pending` | error-handling.md |
| functions | – | – | 2 | `check_pending` | functions.md; selectors.md |

## Spec coverage

Every document in `docs/spec/` maps to at least one label:

| Spec doc | Labels |
|---|---|
| lexical-structure.md | lexical, syntax-errors |
| values-and-absence.md | arithmetic, bindings, absence, compile-errors |
| object-model.md | metaclass, dispatch, classes, inheritance |
| blocks.md | blocks, control-flow |
| functions.md | functions |
| messages-and-selectors.md | messages, dispatch, runtime-errors |
| selectors.md | messages/pending, lexical/pending, absence/pending, functions/pending |
| classes.md | classes |
| method-lookup.md | dispatch, runtime-errors, inheritance |
| control-flow.md | control-flow, booleans |
| error-handling.md | errors |
| concurrency.md | concurrency |
| system.md | system |
| U-LIST-plan.md | list, runtime-errors |
| lexical-structure.md §4/§6/§7/§8 (collection literals) | collections |

## Running

```sh
cargo test -p phalcom-core --test lang               # active PASS/NEGATIVE (green)
cargo test -p phalcom-core --test lang classes       # one label
cargo test -p phalcom-core --test lang -- --ignored  # PENDING spec targets (expected to fail)
```

## Notes

- The corpus deliberately mixes working regression guards with pending spec targets;
  the pending set is the executable to-do list for the remaining spec surface.
- Numbers are `f64`: `7/2` → `3.5`, whole results print without a decimal, `1/0` → `inf`,
  `0/0` → `NaN`.
- Three known bugs are pinned as pending spec targets: setter parameter name hardcoded
  to `value`; keyword-argument calls build selectors no method definition can match;
  user-defined `==(other)` never dispatched for instances (identity fallback).
- Adding a case: drop `<name>.ph` + `<name>.expected` in the label dir (or `pending/`);
  create the label dir first if new (a missing dir panics `collect_cases`), and wire the
  label in `../lang.rs` if it has no `check_*` test yet.
- U6 (absence → `Option` + `let`/`var`): surface `nil` was removed, so the old
  `lexical_nil_prints` / `system_print_nil` PASS cases became the single
  `compile-errors/compile_error_surface_nil` NEGATIVE (they were byte-identical), and the
  former `binding_let_reassignment` PASS became `compile_error_let_reassignment` (reassigning
  a `let` is now a compile error). The `compile-errors` lane holds compile-time semantic
  diagnostics (surface `nil`, `let` no-initializer, `let` reassignment, `Option` truthiness).
  The `absence`/`bindings` `pending/` cases stay pending: they pin the final surface (a pretty
  `None` printString and `Some(x)` sugar) that U-STD/later units deliver, not U6's substrate
  output (`<None instance>`, `Some.new(_)`).
