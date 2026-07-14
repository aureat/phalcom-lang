# Units tracker — by feature, oldest → newest

Cross-cutting index over [`units/`](units/), grouped by feature area instead of by unit
number. Complements, doesn't replace, the status of record (`phase-next/STATE.md` +
as-built specs). Within each group, units are ordered by actual landing sequence
(oldest first); unchecked items are proposed dispatch order for future work, not fact.

Audited against `git log --oneline --all` + `git status --short` on `9daeb43` (2026-07-13).
Concurrent sessions are live on this branch — re-verify file/commit references before
trusting them if time has passed.

- [x] landed (git history has the commit(s))
- [x] *(uncommitted)* — implementation exists in the working tree but isn't committed yet
- [ ] proposed / not started

---

## 1. Spine & bootstrap (closed roster)

The frozen VM/compiler core every later track builds on. See [`units/README.md`](units/README.md#spine-track---the-landed-language-core-forge-roster-closed).

- [x] **U0** — [as-built](units/U0/as-built.md) — stand up `verify.sh`, golden corpus, invariant harness; every later unit gates on this.
- [x] **U1** — [as-built](units/U1/as-built.md) — slotmap handle/arena heap + tagged `Value` (ADR-0009/0010).
- [x] **U2** — [as-built](units/U2/as-built.md) — parallel metaclass rule + `Behavior` kernel + `verify_invariants()` (ADR-0002/0003).
- [x] **U3** — [as-built](units/U3/as-built.md) — label-encoded selector symbols replace arity-only dispatch (ADR-0012).
- [x] **U4** — [as-built](units/U4/as-built.md) — first-class blocks/closures, open/closed upvalues, frame-token infra (ADR-0013 groundwork).
- [x] **U5** — [as-built](units/U5/as-built.md) — control flow lowered to ordinary sends via the guarded sacred-selector inliner (ADR-0018).
- [x] **U6** — [as-built](units/U6/as-built.md) — absence → first-class `Option`, no surface nil, `let`/`var` (ADR-0007/0014/0021).
- [x] **U7** — [as-built](units/U7/as-built.md) — fixed per-class slot layout, `construct`, class-side stored static fields (ADR-0011/0017).
- [x] **U8** — [as-built](units/U8/as-built.md) — `doesNotUnderstand`/`perform`/`respondsTo` + `Message` reification (ADR-0012 amendment).
- [x] **U9** — [as-built](units/U9/as-built.md) — trailing `*name` rest params collapse extra args into a `List`.
- [x] **U10** — [as-built](units/U10/as-built.md) — non-local `return` inside a block via frame-token unwind (ADR-0013).
- [x] **U11** — [as-built](units/U11/as-built.md) — `Bool` split into abstract root + `True`/`False` singletons (ADR-0004).
- [x] **U-FE** — [as-built](units/U-FE/as-built.md) — hand-written lexer + recursive-descent/Pratt parser, replacing logos/LALRPOP (ADR-0016).
- [x] **U-LEX** — [as-built](units/U-LEX/as-built.md) — block comments, digit separators, newline suppression, `\(expr)` interpolation, `??`/`?.`.
- [x] **U-LIST** — [as-built](units/U-LIST/as-built.md) — native `Vec<Value>` kernel `List` + thin `.ph` protocol (ADR-0019/0020).
- [x] **U-STD** — [as-built](units/U-STD/as-built.md) — pure-`.ph` `Option`/`List` combinator layer over the frozen floor.
- [x] **U11-UCORE** — [handoff](units/U11-UCORE/handoff.md) — *(historical)* resume bridge from spine-close to U-CORE track; self-marked superseded by `U-CORE-3/handoff.md`, kept only for provenance.

## 2. Core-library reflection (U-CORE track)

The core library built in Phalcom itself, over the now-frozen primitive floor. Landing
order is non-numeric — `U-CORE-2` shipped before `U-CORE-1` (see commits below).

- [x] **U-CORE-2** — [as-built](units/U-CORE-2/as-built.md) — `0da64d6` Some-lift one-armed `ifTrue`/`ifFalse`, closes the half-`Option` divergence.
- [x] **U-CORE-1** — [as-built](units/U-CORE-1/as-built.md) — `03764e3` kernel reflection: `hash`, `isA`, `Behavior.name`/`.methods` + invariant substrate (floor 73→80).
- [x] **U-CORE-3** — [as-built](units/U-CORE-3/as-built.md) — `10ebd06` callables/`Block` reflection surface + `BoundMethod` value.
- [x] **U-CORE-4** — [as-built](units/U-CORE-4/as-built.md) — `2061795` per-type `toString` overrides, unified native print path.
- [x] **U-CORE-5** — [as-built](units/U-CORE-5/as-built.md) — `bc161fb` shared collection-protocol contract; `List` as reference impl; structural `#==`/`!=`.
- [x] **U-CORE-6** — [as-built](units/U-CORE-6/as-built.md) — `85c4e1d` `Error` root + `MessageNotUnderstood`, wired into the unified unwind.
- [x] **U12** — [plan](units/U12/plan.md) — *(decision closed, no code)* `f16b58a` DEC-U12=A: keep flat `f64` `Number`, Integer/Float split deferred.
- [x] **U17** — [plan](units/U17/plan.md) — *(decision closed, no code)* `0ff3239` ADR-0044: formalize `None`/`Option` bootstrap-cycle avoidance; niche-encoding deferred.

## 3. Object model & inheritance

- [x] **U-INH** — [plan](units/U-INH/plan.md) — `0e920ab`/`3248b05` single inheritance: `extends`, parallel-metaclass repair, `SuperSend` + super-construct chaining.
- [x] **ctor-inherit guard fix** *(follow-on to U-INH/U7, no own unit folder)* — `72f87dc` made the bare-allocator `new`-guard inheritance-aware via `VM::class_parents` chain-walk.
- [x] **U13** — [plan](units/U13/plan.md) — `5d84ad8` ADR-0041 hierarchy-stability policy: sealed, no runtime superclass reassignment; traits/MI ruled out.
- [x] **U-REOPEN-FIX** *(follow-on to U7/U13, no own unit folder)* — `e85f31a`/`a9e1eaf` class-reopen was dropping/corrupting methods and field layout; reopen now appends methods and rejects field-adding/superclass-changing reopens.

## 4. Error handling

- [x] **U-ERR** — [plan](units/U-ERR/plan.md) — `7c901cf` `throw`/`try`/`catch`/`on`/`ensure` + `Result`/`Ok`/`Err` (ADR-0008/0031/0038). Builds on `U-CORE-6`'s `Error` root.
- [x] **U-ERR-FIX** *(follow-on, no own unit folder)* — five test-wave bugs found after U-ERR landed: `e904b57` `System.print` routed through `toString`, `b5ac831` class-side `super` via metaclass chain, `2f8ccd9` operator selectors after `.`, `dd2e178` `not` as unary-prefix keyword, `69c1157` golden rebaseline.

## 5. Collections & iteration

- [x] **U-ITER** — [plan](units/U-ITER/plan.md) · [spec](units/U-ITER/specification.md) — cursor protocol (`iterate`/`iteratorValue`) + `for`/`break`/`continue` (ADR-0035). Landed in 4 steps, `aa964bb`→`0142c7a`.
- [x] **U-FIBER** — [plan](units/U-FIBER/plan.md) — *(see §6, lands interleaved with U-ITER)*
- [x] **U-COLL** — [plan](units/U-COLL/plan.md) — `1274504`/`5bc31e8`/`dc9eab0` literal sugar: `[…]`/`(a,b)`/`{k:v}` desugar to List/Tuple/Map construction sends (ADR-0029/0032).
- [x] **U-ITER-FIX** — [plan](units/U-ITER-FIX/plan.md) — four loop-control follow-ons: `9288ad5` materialized-block break/continue trap, `ac4f721` bare-`while` break/continue, `08a323b` fresh loop-var cell per iteration, `b566e6b` jump-helper dedup.
- [x] **U-COLLTYPES** — [plan](units/U-COLLTYPES/plan.md) — native `Map`/`Set` (`be8426e`), `Tuple` (`2d140f0`), `Range` (`f934cf1`), map-literal wire (`10e1715`) — ADR-0032/0039 floor amendment.
- [x] **U-STD** — *(see §1 — List#each migrated onto cursor protocol here, `c35171a`, alongside this track)*
- [ ] **U-ITERABLE** — [plan](units/U-ITERABLE/plan.md) — bare-cursor Route B (kill per-step `Some` allocation) + kernel `Iterable` root hoisting `each`/`map`/`filter`/`reduce` (ADR-0048, ratified, unimplemented). **Next up** — everything below in this group waits on it.
- [ ] **U-SEQ** — [plan](units/U-SEQ/plan.md) · [spec](units/U-SEQ/implementation-spec.md) — hard-blocked on `U-ITERABLE`. Sequence-breadth combinators (`all`/`any`/`count`/`find`/`join`) + lazy views (`MapView`/`WhereView`/…).
- [ ] **U-STRING** — [plan](units/U-STRING/plan.md) — `rawByteCount`/`rawByteAt`/`rawSlice` + `System.rawWrite` funnel (ADR-0019 floor amendment, ADR-0049 draft). Independent of Iterable/Seq — can dispatch in parallel with either.

## 6. Concurrency (`Fiber` / `Future` / scheduler)

- [x] **U-FIBER** — [plan](units/U-FIBER/plan.md) — `5334774`→`a26b05b` cooperative bare `Fiber` (`new`/`call`/`try`/`yield`/`current`/`abort`) on the restricted re-entrant loop (ADR-0030).
- [x] **U-FUTURE (Slice A)** — [plan](units/U-FUTURE/plan.md) — `f0d128a` settle-once `Future` state machine, pure `.ph`, no scheduler dependency.
- [x] **U-FIBER-FIX** *(follow-on, no own unit folder)* — `1451f62`/`a3e23e8` root-abort guard, resume-gate message, cross-fiber non-local-return → `DeadFrameError` (found via Fiber adversarial testing, root-caused into the reopen mechanism — see `U-REOPEN-FIX` in §3).
- [x] **U-SCHED-FIBER** — [`U-FIBER-REFLECT`](units/U-SCHED-FIBER/U-FIBER-REFLECT/plan.md) · [`U-SCHED`](units/U-SCHED-FIBER/U-SCHED/plan.md) — `34246a8` `Fiber#isDone`/`#error` reads + native `VM::ready_queue`, `System.schedule`/`nextScheduled`, root-drive pump. Both are `U-FUTURE` Slice B's preconditions and are now satisfied.
- [ ] **U-FUTURE (Slice B)** — [plan §6.3/§9](units/U-FUTURE/plan.md) — `async`/`await`, native pump wired through `System.runScheduled`. **Next up** — no remaining blocker; DEC-FUT-SCHED is ratified and `U-SCHED-FIBER` has landed.
- ~~**U20**~~ — [plan](units/U20/plan.md) — *(superseded, not future work)* speculative first-cut Fiber+Future sketch, authored before `U-FIBER`/`U-FUTURE` existed under those names; zero implementation, kept for provenance only.

## 7. Lexer & parser surface

- [x] **U-LEX-HASH** — [plan](units/U-LEX-HASH/plan.md) — `fac45ae` `#`-prefixed name-symbol and selector-symbol literals.
- [x] **U15** — [plan](units/U15/plan.md) — `6188973` `import`: relative-path resolution, whole-module binding, memoized + cyclic-import detection.
- [x] **U14** — [plan](units/U14/plan.md) — `0769316` destructuring `let`/`var`: `let (a,b) = point`, `let [first, *rest] = list`.
- [x] **U16** — [plan](units/U16/plan.md) — `dfb96ff` Open-form `::` method references, `71c703d` Pinned-form + Family introspection.
- [x] **U18** — [plan](units/U18/plan.md) — *(decision closed, no code)* `f16b58a` DEC-U18=A: no default arguments, keeps one-arity-per-selector dispatch.
- [x] **U-NEG** — [plan](units/U-NEG/plan.md) — `0b98ca9` unify boolean negation on `not`; retire prefix `!` (keep `!=`).
- [x] **U-IS** — [plan](units/U-IS/plan.md) — `03dbd09` `is`/`is!`/`is not` type-test operators desugaring to `is(_)`/`isExactly(_)`.

## 8. Indexing / subscript

- [x] **U-INDEX** — [plan](units/U-INDEX/plan.md) — `47b0b22` postfix `[]` read/write sugar over a dedicated `[](...)`/`[](...,put:)` operator selector.

## 9. Annotations, contracts & layout

- [x] **U-ANNOT-CONTRACTS** *(uncommitted)* — [plan](units/U-ANNOT-CONTRACTS/plan.md) — `@` core annotation mechanism + `@requires`/`@ensures`/`@invariant` contract weaving. Implemented (`compiler/attributes.rs`, `AttributeExpander`, `VM::checking` fiber-safe guard set per ADR-0052) but **not yet committed** — land this before touching it further; verify against current tree, concurrent sessions are active here.
- [ ] **U-ANNOT-LAYOUT** — [plan](units/U-ANNOT-LAYOUT/plan.md) — `FieldDef` + `@get`/`@set`/`@construct` + `@data`/`@sealed`/`@variant` layout-derive tier. Strictly depends on `U-ANNOT-CONTRACTS` landing (committed) first.

## 10. Tooling (`vsphalcom`)

- [x] **U-VSPHALCOM** — [plan](units/U-VSPHALCOM/plan.md) — extension modernization for spec v0.2: `db190f4` move in-tree + grammar rewrite, `20036eb`/`206f0b9` `phalcom check` CLI + diagnostics wiring, `122fd40`/`a568ab6` autocomplete, `9daeb43` hover (keyword docs, selectors, Phaldoc, contract stub) — commit message notes hover is "complete pending dev-host verification."

## 11. Performance, GC & benchmarking

Tiered strategy per `docs/spec/v0.2/performance.md` ("measure first") — `U-BENCH` gates
everything else in this group; the rest can reorder based on what the baseline shows.

- [x] **U-BENCH** — [plan](units/U-BENCH/plan.md) — `ebe9d97` Tier 0: Wren-suite reference programs + plan landed (perf harness binary itself still pending). Blocks every later perf tier.
- [ ] **U-PRIM-ABI** — [plan](units/U-PRIM-ABI/plan.md) — Tier 2: in-place primitive stack ABI + arithmetic fast path, cuts per-send allocation.
- [ ] **U-IC** — [plan](units/U-IC/plan.md) — Tier 3: selector-only interner + monomorphic inline cache + superinstructions, populates ADR-0012's reserved IC seam.
- [ ] **U-HOTPATH** — [plan](units/U-HOTPATH/plan.md) — dispatch-loop hot-path optimizations (register-hoisted interpreter state, behavior-invariant); natural follow-on to `U-IC`'s dispatch-loop work.
- [x] **U-GC** — [plan](units/U-GC/plan.md) · [impl spec steps 3–5](units/U-GC/IMPL-SPEC-steps-3-5.md) — non-moving mark-sweep collector (ADR-0050). Steps 0–4 landed `94b6bbf`: roots/edge table, Win A (`Box` fat variants, 280B→40B), `trace_object`/`collect`/`force_gc`, `System.gc`, safepoint latch (Invariant L) — 14 GC tests green. **Step 5 (fiber-stack pool re-measure) done, second null result** — rebuilt per the F5 design, A/B'd post-collector, **no measurable win** (post-sweep the pool competes against malloc, not a leak); reverted and kept in `git stash@{0}` (`U-GC-pool-implementation-null-result`), not landed. `DEFERRED.md` M-RUNTIME temp-root note, perf numbers, miri lane, and reviewer gate (`heap/`/`vm/` spine files) still open before the unit is fully closed.
- [ ] **U-COMPILE** — [plan](units/U-COMPILE/plan.md) — Tier 5: compile-time/startup optimization (constant dedup, cache `core.ph` compile). Last — needs the dispatch/heap shape from the tiers above settled first.

## 12. Naming conventions (opportunistic, no urgency)

- [x] **U-NATIVE-MARKER** — [plan](units/U-NATIVE-MARKER/plan.md) — `3e362e2` mechanical rename, native/private primitive `raw*` prefix → trailing `_` suffix (Wren convention).

---

## Suggested next dispatch (given current state)

1. **Commit `U-ANNOT-CONTRACTS`** (§9) — already implemented, sitting uncommitted; land it before anything else touches `compiler/`.
2. **Close out `U-GC`** (§11) — steps 0–4 landed, step 5 done as a second null result (stashed, not shipped); still needs `DEFERRED.md` M-RUNTIME note, perf numbers, miri lane, and `phalcom-reviewer` sign-off (spine files).
3. **`U-FUTURE` Slice B** (§6) — fully unblocked now, highest-value concurrency work left.
4. **`U-ITERABLE` → `U-SEQ`** (§5) — unblocks the collections track's remaining breadth.
5. Walk the remaining perf tiers (§11) as time allows — lowest urgency, gated on wanting perf work at all.
