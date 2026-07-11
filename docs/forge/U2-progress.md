# U2 — metaclass tower + `Behavior` kernel + `verify_invariants()` — progress log

_Executed per `docs/forge/remaining-work-handoff.md`'s U2→U4→U5 handoff, but **directly on
`main`, with no `phalcom-architect`/`phalcom-reviewer` gate this pass** (user instruction:
"no reviewer or architect yet, just pure coding and implementation"). The reconciliation
deltas that would normally come from an architect pass were pre-baked into the plan the
implementation followed; see below for exactly what changed and what remains unverified._

## What landed

All three slices from the plan landed in one pass (no fresh-subagent handoff needed — the
change fit comfortably in one context):

1. **Kernel allocation + wiring** (`phalcom-core/src/universe.rs`):
   - `create_core_classes` rewritten to the `object-model.md` §6 seven-step order: allocate
     all 8 apex rows bare (`Object`, `Behavior`, `Class`, `Metaclass`, `Object class`,
     `Behavior class`, `Class class`, `Metaclass class`), wire instance-of, wire
     instance-side superclasses, then wire metaclass-side superclasses by the parallel rule
     (ADR-0002).
   - `CoreClasses` gained a `behavior_class: ClassId` field.
   - `make_core_class` (renamed reference: the actual function, not `create_core_class`) now
     computes `(name class).superclass = superclass.class` instead of the old hardcoded
     `class_class` — fixes F2 at the core-bootstrap site.
   - `superclass`/`superclass=` primitives moved from `class_class` to `behavior_class` in
     `install_primitives`; `Class` and `Metaclass` now inherit them through the superclass
     chain instead of each having their own copy.
   - `Universe::verify_invariants(&self, heap: &Heap) -> Result<(), String>` added, asserting
     the full §5 apex table plus the four sanity checks (closed metaclass loop, parallel rule
     on an ordinary core class, bounded superclass-chain-terminates walk, `Object.superclass
     == None`).
2. **Bootstrap wiring + user-class F2 fix** (`phalcom-core/src/vm.rs`):
   - `VM::new()` calls `vm.universe.verify_invariants(&vm.heap).expect(...)` right after
     `Universe::install_primitives` — panics on a malformed kernel, per plan.
   - `VM::create_class` (used by the `Bytecode::Class` handler) fixed to the parallel rule:
     the new metaclass's `.class` is now `metaclass_class` (was incorrectly `class_class`),
     and its superclass is `superclass.class` (falling back to `class_class` if `superclass`
     is `None`) instead of always `Object`'s metaclass.
   - `install_core`'s `add_class!` macro list gained `behavior_class` so `Behavior` is exposed
     as a core-module global alongside `Class`/`Metaclass`.
3. **Test rewrite + ADR fold-in** (`phalcom-core/tests/invariants.rs`, ADRs):
   - Un-ignored `metaclass_superclass_parallels_instance_superclass` and
     `behavior_class_exists_in_tower` (rewritten to assert real wiring, not `unimplemented!()`).
   - Rewrote the 3 tests that encoded the collapsed apex
     (`metaclass_is_its_own_class_closing_the_loop` → `metaclass_is_instance_of_metaclass_class_closing_the_loop`,
     `class_class_is_metaclass` → `class_is_instance_of_class_class_not_metaclass_directly`,
     `metaclass_superclass_is_class` → covered by `behavior_class_exists_in_tower`) to assert
     the §5 apex table's real 8-row shape instead of the old 3-way-collapsed wiring.
   - Added `verify_invariants_holds_after_bootstrap` and
     `metaclass_responds_to_superclass_via_behavior` (asserts `Behavior` defines the
     `superclass` getter directly and both `Class`/`Metaclass` inherit it via
     `lookup_method_in_hierarchy`, not their own copies).
   - `docs/adr/0002-metaclass-tower-parallel-rule.md`: added a "Superseded (U2)" pointer note
     folding in ADR-0009's allocate-then-patch mechanism.
   - `docs/adr/0003-introduce-behavior-kernel-class.md`: replaced the "Status note — Open
     question" section with an "Implementation note (U2)" confirming Q11 resolved and citing
     the actual landing site.
   - `docs/forge/DEFERRED.md` F4 entry (already present from the architect pass) confirmed,
     not duplicated.

## Build/test bar (per plan — not full `verify.sh`)

- `cargo build --workspace` — clean.
- `cargo test -p phalcom-core --test invariants` — 10/10 pass (was 5 passing + 2 ignored + 3
  green-but-wrong before this unit).
- `cargo test -p phalcom-core` (full suite, to catch regressions beyond `invariants.rs`) —
  golden (6), invariants (10), lang (9 passed + 7 pre-existing `#[ignore]`d PENDING) all green.
  The `lang.rs::metaclass` test stays `#[ignore]`d — it targets the surface-syntax `.ph`
  golden-corpus lane (`tests/lang/metaclass/`), which doesn't exist yet and is out of this
  unit's file list.
- `cargo doc --workspace --no-deps` — clean (fixed one `rustdoc::private_intra_doc_links`
  warning on a doc-comment reference to the private `make_core_class`).
- Ran `cargo run -p phalcom-core --bin phalcom examples/simple.ph` post-change: interpreter
  boots and runs end to end (see below).

## Deferred verification risk (explicit, per handoff)

- **No independent `phalcom-reviewer` gate ran this pass.** STATE.md's review policy lists
  U2 as reviewer-ON (load-bearing, can corrupt the object model); that gate was explicitly
  skipped here per user instruction. Recommend a follow-up review pass before this is
  considered fully forge-verified.
- **`cargo clippy` and the golden-corpus byte-identical ceremony were not run.** Golden tests
  did pass as part of the full `cargo test -p phalcom-core` run above, but the stricter
  "byte-identical to a captured baseline" check from `verify.sh` was not exercised.
- **No worktree isolation was used** — implemented directly on `main` per explicit user
  instruction ("continue on the main branch"), diverging from the plan's worktree-based
  working model. There is no `feat/u2-metaclass` merge step as a result; the stale
  `../phalcom.worktrees/u2` worktree/branch pair (created but never used — still at the
  pre-U2 HEAD) should be cleaned up by the user or a follow-up session.

## Notable implementation choices not spelled out verbatim in the plan

- Apex row display names use a space (`"Object class"`, matching the spec's diagram/table
  literally) rather than a dot (`"Object.class"`, the old collapsed-apex naming and still
  what `VM::create_class` uses for *user*-defined classes). These are two different naming
  conventions for two different code paths (kernel bootstrap vs. runtime user classes) and
  were not unified — doing so was out of this unit's scope.
- `VM::create_class`'s `metaclass_superclass` falls back to `class_class` when `superclass`
  is `None`, mirroring the apex table's `Object class ← Class` rule for the one case where a
  user class could theoretically have no superclass. In practice the only caller
  (`Bytecode::Class` handler) always supplies `Some(Object)` at minimum today (no explicit
  superclass syntax yet — tracked separately, not a U2 concern).
