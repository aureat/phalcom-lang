# U2 — Work order: metaclass tower + `Behavior` + `verify_invariants()`

_Self-contained implementation plan for **one** `phalcom-implementer` agent. Corrects the metaclass
tower to the parallel rule, introduces the `Behavior` kernel class, and makes the object model
self-checking. Fixes audit findings **F2 / F5 / F6**. Load-bearing unit → independent
`phalcom-reviewer` gate afterward. Grounded in **ADR-0002** (parallel rule) + **ADR-0003** (`Behavior`),
built on the **post-U1 handle/arena heap** (ADR-0009/0010/0011). STATE.md ADR mapping is authoritative._

---

## 0. Mission (one sentence)
Wire the metaclass tower to the parallel rule `(X class).superclass == (X.superclass) class`, introduce
`Behavior` as the shared superclass of `Class`/`Metaclass`, materialize the full apex (`Object class`,
`Behavior class`, `Class class`, `Metaclass class`), and add a `verify_invariants()` regression guard
that asserts every object-model §5 rule after bootstrap — un-ignoring and correcting the invariant
harness so it encodes the *target* tower, not today's collapsed one.

## 1. Hard guardrails (read before writing any code)
- **This is a correctness fix to the kernel wiring, not a feature.** No new surface syntax; no allocation
  or `construct` changes (that is U7). No `Value`/`Heap` representation changes (that was U1).
- **Build on the post-U1 substrate.** Classes live in the `Heap`, referenced by `Copy` `ClassId`/`ObjRef`
  handles; the bootstrap uses ADR-0009 **allocate-then-patch** (superseding ADR-0002's `Rc::new_cyclic`).
  There is **no `PhRef`/`MaybeWeak`/`RefCell`** — do not reintroduce them. Confirm the exact handle/heap
  API on HEAD before editing (§2).
- **`Behavior` is ratified** (ADR-0003; open-questions Q11 RESOLVED). Introduce it; do not re-litigate.
- **Scope is F2/F5/F6 only.** Do **NOT** fold in **F4** (`object_name`/instance `toString`) here — it is
  governed by ADR-0015 and reserved for its own unit. (This intentionally narrows PLAN.md §U2, which
  speculatively folded F4; keeping U2 tight makes it independently verifiable. Note it in DEFERRED.)
- **Do NOT touch other live bugs** (F1 swallowed `Result`, etc.). Stay inside the write-set (§3). If
  forced outside it, **STOP and report a conflict**; append out-of-scope ideas to
  [`DEFERRED.md`](DEFERRED.md). **Do not self-approve** — a `phalcom-reviewer` gates this unit.

## 2. Preconditions (verify first; do not assume)
- **U1 is merged to `main` and green.** This unit branches `feat/u2-metaclass` off `main` and runs in its
  **own git worktree** (`git worktree add ../phalcom.worktrees/u2 feat/u2-metaclass`), per the working model
  (one spine unit per worktree). Confirm `./scripts/verify.sh` is green on the branch point (baseline).
- **graphify-first:** the on-disk source predates U1. Re-derive the write-set on the *actual* post-U1 HEAD:
  `graphify explain "ClassObject"`, `graphify explain "CoreClasses"`, `graphify affected "create_core_classes"`,
  `graphify affected "install_primitives"`. Confirm the class handle type name (`ClassId`/`ObjRef`), how a
  class's `class`/`superclass` fields are stored and patched, and where the bootstrap now lives before reading raw source.
- Confirm the two `#[ignore]`d tests still exist in `tests/invariants.rs` and that U1 migrated the file to
  handle equality (it previously used `Rc::ptr_eq`; post-U1 it must compare `ClassId` by `==`).

## 3. Confirmed write-set (derive final form from `graphify affected` on post-U1 HEAD)
| File | Why it's in scope |
|---|---|
| `phalcom-core/src/universe.rs` | **Primary.** `create_core_classes` bootstrap (F2/F5/F6): materialize `Behavior` + the four apex metaclasses, wire the parallel rule; add `behavior_class` to `CoreClasses`; move the shared reflective protocol onto `Behavior` in `install_primitives`; house `verify_invariants()`. |
| `phalcom-core/src/vm.rs` | Bootstrap driver: call `verify_invariants()` after wiring; fix the **user-class** metaclass wiring (F2, ~`vm.rs:92` pre-U1 — confirm site) to the parallel rule so future subclass syntax inherits correctly. |
| `phalcom-core/tests/invariants.rs` | Un-ignore the 2 spec-target tests **and correct the currently-green apex tests** (see §4/§5 — several encode the collapsed F6 apex and become wrong under the fix); add a `verify_invariants()` test. |
| `phalcom-core/src/class.rs` | *Only if needed:* a small accessor/helper for parallel-superclass wiring or a `Behavior`-aware predicate. Keep minimal; prefer to add nothing. |
| `docs/adr/0002-metaclass-tower-parallel-rule.md` | **Fold-in (§6):** add a pointer note superseding the `Rc::new_cyclic` mechanism with ADR-0009 handle-patching (the *decision* stands). |
| `docs/adr/0003-introduce-behavior-kernel-class.md` | **Fold-in (§6):** flip the stale "Status note: Open question pending" → Accepted (open-questions Q11 is RESOLVED). |

> Keep `phalcom-core/src/primitive/*.rs` **out** of the write-set: the primitive fn bodies do not change;
> only *which class* they are installed on changes, and that lives in `universe.rs::install_primitives`.

## 4. Design decisions (ADR-0002 / ADR-0003 / object-model §5–6 — realize, don't re-litigate)
- **Parallel rule (ADR-0002 rule 4):** `(X class).superclass == (X.superclass) class`, anchored by the root
  rule `(Object class).superclass == Class`. The bootstrap helper that builds each core class's metaclass
  must set the metaclass's superclass to **the superclass's metaclass**, not to `Class` (today's F2 bug).
- **`Behavior` kernel (ADR-0003):** `Object <- Behavior <- {Class, Metaclass}`. `Behavior` is abstract (no
  live instances). Add a `behavior_class: ClassId` field to `CoreClasses`.
- **Full apex, no collapse (fixes F6):** materialize eight distinct kernel objects — `Object`, `Behavior`,
  `Class`, `Metaclass` **and** `Object class`, `Behavior class`, `Class class`, `Metaclass class` — wired
  exactly per the object-model §5 apex table:

  | object | `.class` | `.superclass` |
  |---|---|---|
  | `Object` | `Object class` | *(none)* |
  | `Behavior` | `Behavior class` | `Object` |
  | `Class` | `Class class` | `Behavior` |
  | `Metaclass` | `Metaclass class` | `Behavior` |
  | `Object class` | `Metaclass` | `Class` |
  | `Behavior class` | `Metaclass` | `Object class` |
  | `Class class` | `Metaclass` | `Behavior class` |
  | `Metaclass class` | `Metaclass` | `Behavior class` |

  Note the loop closer: `Metaclass.class == Metaclass class` and `(Metaclass class).class == Metaclass`
  (§5 rule 5) — **not** `Metaclass.class == Metaclass`. Today's bootstrap collapses these (F6); the fix
  distinguishes them.
- **Bootstrap ordering (object-model §6, ADR-0009 allocate-then-patch):** (1) allocate all eight kernel
  handles with fields unset; (2) patch instance-of (`.class`); (3) patch **instance-side** superclasses;
  (4) patch **metaclass-side** superclasses by rule 4 — *this step must read the already-patched
  instance-side superclasses*, so ordering is load-bearing; (5) build remaining core classes via the
  `(name, superclass)` helper such that `name.class == name class`, `(name class).class == Metaclass`,
  `(name class).superclass == superclass.class`; (6) install primitives; (7) run `verify_invariants()`.
- **`verify_invariants()` home + signature:** implement as `Universe::verify_invariants(&self, heap: &Heap)
  -> Result<(), String>` (or a small `InvariantError`), asserting every §5 rule + the sanity checks
  (lines 241–245: `Object.class.class == Metaclass`, `Metaclass.class.class == Metaclass`,
  `Number.class.superclass == Object.class`, and metaclass-chain termination at `Class → Behavior →
  Object → none`). Call it from the VM bootstrap after step 6; a broken kernel is unrecoverable, so the
  bootstrap **panics** on `Err` (`.expect("kernel invariants")`). Tests call the same function and assert `Ok`.
- **Shared reflective protocol → `Behavior` (realizes ADR-0003):** install the `superclass` getter/setter on
  `behavior_class` instead of duplicating on `class_class`, so both `Class` and `Metaclass` inherit it. This
  is low-risk (lookup still resolves via the deeper chain) and is verifiable by a new test asserting a
  metaclass responds to `superclass`. **Leave `new`/allocation per-class as today** (do not consolidate onto
  `Behavior` — that is U7's `construct` domain). `name`/`toString` stay on `Object` for now (F4, deferred).

## 5. Build order (land as one coherent, reviewable diff)
1. **`universe.rs` — kernel allocation + wiring.** Rewrite `create_core_classes` to the §6 ordering above:
   allocate the eight apex handles, patch instance-of, patch instance-side superclasses, then patch
   metaclass-side superclasses by the parallel rule. Add `behavior_class` to `CoreClasses`. Full rustdoc on
   the new field + the wiring steps; cite ADR-0002/0003/0009. **Watch F5:** with handles there is no weak
   path — the self-cycle (`Metaclass class`) is just a handle to an already-allocated slot; no leak.
2. **`universe.rs` — helper + primitives.** Update the `create_core_class(name, superclass)` helper to set
   `(name class).superclass = superclass.class`. Move `superclass` getter/setter onto `behavior_class` in
   `install_primitives`. Keep `new` per-class.
3. **`universe.rs`/`vm.rs` — `verify_invariants()`.** Implement the checker (§4); invoke it from the VM
   bootstrap after primitives; panic on `Err`.
4. **`vm.rs` — user-class parallel wiring.** Fix the user-metaclass creation site (F2) to the parallel rule
   so it matches the kernel path (inert today, correct for U7's subclassing). Confirm the exact site on HEAD.
5. **`tests/invariants.rs` — encode the *target* tower.** This is not just un-ignoring two tests:
   - **Un-ignore** `metaclass_superclass_parallels_instance_superclass` and `behavior_class_exists_in_tower`;
     implement the latter against the new `CoreClasses::behavior_class` and the §5 apex.
   - **Correct the currently-green tests that encode the collapsed apex (F6)** and would now be *wrong*:
     `metaclass_is_its_own_class_closing_the_loop` (→ `Metaclass.class == Metaclass class`, and
     `(Metaclass class).class == Metaclass`), `class_class_is_metaclass` (→ `Class.class == Class class`;
     `Class.class.class == Metaclass`), and `metaclass_superclass_is_class` (→ `Metaclass.superclass ==
     Behavior`). Rewrite each to the §5 apex table; keep the ones that remain true (`object_has_no_superclass`,
     `object_class_class_is_metaclass`, `core_classes_have_correct_metaclass_and_superclass`,
     `walking_metaclass_superclass_chain_terminates`).
   - **Add** a test that calls `Universe::verify_invariants(&heap)` and asserts `Ok`, plus a test that a
     metaclass responds to `superclass` (Behavior inheritance).
6. **Fold-in ADR notes (§6)** and re-run the green gate.

## 6. Fold-in cleanup (doc-only; U2 owns these ADRs)
- **ADR-0002:** add a Consequences/note line: the `PhRef::new_cyclic` cycle-break is **superseded by
  ADR-0009's handle allocate-then-patch**; the parallel-rule *decision* is unchanged, only the wiring
  mechanism. (STATE.md: "Add a pointer note on ADR-0002 when U2 lands.")
- **ADR-0003:** replace the trailing "Status note: Open question pending a go/no-go decision" with an
  Accepted status, citing open-questions Q11 (RESOLVED) and the STATE.md ADR mapping.

## 7. Risks (what can go subtly wrong)
- **Ordering hazard:** step 4 (metaclass-side superclasses) reads instance-side superclasses set in step 3.
  Reversing them silently mis-wires the tower and `verify_invariants` should catch it — make sure the checker
  actually runs before any `.expect` masks a panic.
- **Apex-collapse regressions in tests:** three *currently-green* invariants (§5) encode F6 and must be
  rewritten, not preserved. A reviewer must confirm they assert the §5 apex table, not the old collapsed one.
- **Deeper chains change lookup:** inserting `Behavior` between `Object` and `Class`/`Metaclass`, and routing
  metaclass superclasses through the parallel chain, lengthens lookup paths. Golden output must stay
  byte-identical — a class-side send that previously resolved on `Class` must still resolve (now possibly via
  the `Behavior` link). Verify via `./scripts/verify.sh` goldens.
- **Handle-before-patch reads:** with `Copy` handles an unset field may be a sentinel/default `ClassId`;
  reading it before patching yields a wrong-but-not-panicking wiring. Allocate all eight first, patch after.
- **Borrow-model fragility (standing risk):** even post-U1, threading `&mut Heap` through the wiring steps can
  fight the borrow checker if you hold a `&mut` to one class while reading another. Fetch handles first, then
  mutate one slot at a time.

## 8. Test strategy (what must assert green)
- `tests/invariants.rs`: full §5 apex table + sanity checks (lines 241–245), both previously-`#[ignore]`d
  tests passing, and a direct `verify_invariants() == Ok` test.
- `verify_invariants()` itself is the permanent in-VM regression guard (runs every bootstrap).
- Golden corpus (`examples/core_new.ph`, `person2.ph`, `tests/fixtures/golden/*`) + `tests/lang.rs`:
  **byte-identical** output — the tower change must be observationally invisible to running programs.
- `./scripts/verify.sh` exits 0 (build + test + clippy + golden + invariants).

## 9. Mandatory rules
- **Docs** ([`docs/rust-documentation-guidelines.md`](../rust-documentation-guidelines.md)): `//!` refreshed on
  every touched module; `///` on every new/changed public item (`CoreClasses::behavior_class`,
  `verify_invariants`, any helper) with `# Panics` where the bootstrap can panic, intra-doc links, and
  **ADR-0002/0003/0009 citations**. `cargo doc --workspace --no-deps` adds **no new warnings**.
- **Green gate:** `./scripts/verify.sh` exits 0; goldens byte-identical; don't add clippy warnings (fix
  pre-existing ones in any file you rewrite).
- **Best practices:** `rust-best-practices` skill. Do not reintroduce `Rc`/`RefCell`/`MaybeWeak` — the tower
  is now handle-wired.

## 10. Return contract (to the reviewer, not self-approval)
Report: the eight-object apex wiring (with a small table proving it matches §5) · where `verify_invariants`
lives + what it checks · the exact `tests/invariants.rs` deltas (which currently-green tests were corrected
and why) · confirmation goldens + `tests/lang.rs` stayed byte-identical (with `verify.sh` tail) · `cargo doc`
tail · both ADR fold-in notes applied · explicit confirmation **F4 was NOT touched** and allocation/`new`
were left per-class · any new `DEFERRED.md` entries. A `phalcom-reviewer` independently verifies the tower
against the §5 apex table, the invariant-harness correctness, and the green gate.
