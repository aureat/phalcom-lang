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
- [x] **Track 2 (Option/Some/None sealing)** — `8d401f4` `Option`/`Some`/`None` registered in `VM::sealed_classes` at bootstrap, reusing `attr.sealed_violation` enforcement (`class_decl.rs`); user subclassing now a compile error. `None`'s singleton global binding untouched (no `.ph` reopen — would clobber the global back to the class object). 3 `compile-errors` fixtures added (`absence_{option,some,none}_sealed_violation.ph`), `compile_errors` test green. Answers ADR-0044's open subclass-compatibility question for niche-encoding by ruling the fallback-tier question moot — subclassing is no longer possible at all.

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
- [x] **U-ITERABLE** — [plan](units/U-ITERABLE/plan.md) — bare-cursor Route B (raw index cursor + `None` end-sentinel, no per-step `Some` allocation) + kernel `Iterable` root (`core.ph:309`) hoisting `each`/`map`/`filter`/`reduce` (ADR-0048). Golden suite rebaselined off the pre-Route-B Option-wrapped-cursor protocol: `list_wren_iterate_cursor_protocol`, `map_wren_cursor_roundtrip`, `range_cursor_protocol_direct` (bare `0`/`1`/… + `None` sentinel, no `Some.new(_)`/`.unwrapOr`/`.isSome`), plus 3 negative fixtures (`map_wren_iterate_not_int/not_num`, `range_iterate_wrong_cursor_type`) updated from the old `does not understand 'map(_)'` dNU to the new arithmetic rejection `Expected String, got number` — `cursor + 1` on a non-numeric cursor is the natural error now, not a `.map(_)` dispatch on a non-`Option`. `./scripts/verify.sh` green for all of these. Gate-clean for Route B itself — **unrelated** pre-existing gap remains in `indexing`/`indexing_negative` (bracket-selector method-definition syntax `[i] { ... }` and empty-bracket calls `xs[]` don't parse at all; predates this unit, `26747d0`) — tracked separately, not a U-ITERABLE regression.
- [ ] **U-SEQ** — [plan](units/U-SEQ/plan.md) · [spec](units/U-SEQ/implementation-spec.md) — unblocked, `U-ITERABLE`'s golden-suite regression is fixed. Sequence-breadth combinators (`all`/`any`/`count`/`find`/`join`) + lazy views (`MapView`/`WhereView`/…).
- [ ] **U-STRING** — [plan](units/U-STRING/plan.md) — `rawByteCount`/`rawByteAt`/`rawSlice` + `System.rawWrite` funnel (ADR-0019 floor amendment, ADR-0049 draft). Independent of Iterable/Seq — can dispatch in parallel with either.

## 6. Concurrency (`Fiber` / `Future` / scheduler)

- [x] **U-FIBER** — [plan](units/U-FIBER/plan.md) — `5334774`→`a26b05b` cooperative bare `Fiber` (`new`/`call`/`try`/`yield`/`current`/`abort`) on the restricted re-entrant loop (ADR-0030).
- [x] **U-FUTURE (Slice A)** — [plan](units/U-FUTURE/plan.md) — `f0d128a` settle-once `Future` state machine, pure `.ph`, no scheduler dependency.
- [x] **U-FIBER-FIX** *(follow-on, no own unit folder)* — `1451f62`/`a3e23e8` root-abort guard, resume-gate message, cross-fiber non-local-return → `DeadFrameError` (found via Fiber adversarial testing, root-caused into the reopen mechanism — see `U-REOPEN-FIX` in §3).
- [x] **U-SCHED-FIBER** — [`U-FIBER-REFLECT`](units/U-SCHED-FIBER/U-FIBER-REFLECT/plan.md) · [`U-SCHED`](units/U-SCHED-FIBER/U-SCHED/plan.md) — `34246a8` `Fiber#isDone`/`#error` reads + native `VM::ready_queue`, `System.schedule`/`nextScheduled`, root-drive pump. Both are `U-FUTURE` Slice B's preconditions and are now satisfied.
- [x] **U-FUTURE (Slice B)** — [plan §6.3/§9](units/U-FUTURE/plan.md) — `06432bd` `async`/`await` in `.ph`: `Future.async(action)` runs action in fresh fiber, `#await` suspends current fiber or pumps `System.runScheduled()` on root, `then`/`map`/`catch` register continuations on pending futures. Native fix alongside it: `block_on` (`primitive/block.rs`) now wraps VM/native-raised errors (`CannotYieldAcrossNativeFrame`, `NotAllowed`), not just user `Raise`, into reified `Error` so `.attempt()`/`try...catch` can see them. `concurrency_future_async_await.ph` graduated from `pending/`; `concurrency_future_slice_b.ph` added covering suspending continuations, top-level await, and the native-frame-yield error path. Golden suite green.
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

- [x] **U-INDEX** — [plan](units/U-INDEX/plan.md) — landed 2026-07-14, superseding `47b0b22`'s original ADR-0055 draft (`xs[i]` desugared straight to `.at(i)`). Now implements ADR-0060's ratified design: `[idx] {}`/`[idx, put:] {}` is a dedicated bracket-subscript class member — no separate name token, params live inside the brackets (`Parser::parse_index_member`, dispatched from `parse_class_member`, not `parse_method_name`) — compiling to a direct `SignatureKind::Subscript` send (`[_]`/`[_,put]`/`[]`/`[put]`), never `at`/`at(_,put:)` lowering. `[...]` at the call site is arg-list-shaped (`xs[i,j]`, empty `xs[]` cleanly short-circuits) via `parse_arg_list()`. Collapsed the pre-existing, unwired `SignatureKind::SubscriptGet`/`SubscriptSet` (a third, incompatible spelling) into one `Subscript(u8)` matching ADR-0060. `List`/`Map` get `[i] {}`/`[i,put:] {}` wrappers in `core.ph`, `Tuple` gets `[i] {}` only (immutable). `indexing`/`indexing_negative` golden lanes both green. `./scripts/verify.sh` all-green, `cargo clippy`/`cargo doc` unchanged vs. baseline. Plan doc corrected in the same pass — its own Design section had picked the spelling ADR-0060 later rejected; see plan.md's top-of-file correction note. Perf-delta re-measurement (`phalcom-perf --bench-only`) not run this pass — follow-up, don't assume unchanged.

## 9. Annotations, contracts & layout

- [x] **U-ANNOT-CONTRACTS** — [plan](units/U-ANNOT-CONTRACTS/plan.md) — `dc01b07`/`44d277f` `@` core annotation mechanism + `@requires`/`@ensures`/`@invariant` contract weaving (`compiler/attributes.rs`, `AttributeExpander`, `VM::checking` fiber-safe guard set per ADR-0052).
- [x] **U-ANNOT-LAYOUT** — [plan](units/U-ANNOT-LAYOUT/plan.md) — `9f1e31e`/`60db152` `FieldDef` + `@data`/`@sealed`/`@variant` layout-derive tier + `@get`/`@set` accessor derive (collision-checked against hand-written accessors).

## 10. Tooling (`vsphalcom`)

- [x] **U-VSPHALCOM** — [plan](units/U-VSPHALCOM/plan.md) — extension modernization for spec v0.2: `db190f4` move in-tree + grammar rewrite, `20036eb`/`206f0b9` `phalcom check` CLI + diagnostics wiring, `122fd40`/`a568ab6` autocomplete, `9daeb43` hover (keyword docs, selectors, Phaldoc, contract stub) — commit message notes hover is "complete pending dev-host verification."

## 11. Performance, GC & benchmarking

Tiered strategy per `docs/spec/v0.2/performance.md` ("measure first") — `U-BENCH` gates
everything else in this group; the rest can reorder based on what the baseline shows.

- [x] **U-BENCH** — [plan](units/U-BENCH/plan.md) — `ebe9d97` Tier 0: Wren-suite reference programs + plan landed (perf harness binary itself still pending). Blocks every later perf tier.
- [x] **U-PRIM-ABI** — [plan](units/U-PRIM-ABI/plan.md) · [perf-log 001](../forge/perf-log/001-prim-abi-inline-args.md) — `37f31c9` Tier 2: on-stack `[Value;8]` arg buffer replaces the per-send heap `Vec` in the primitive dispatch path. Measured: `arith_send` −41.5%, `bare_send` −33.8%, zero golden diff. **DEC-PRIM-B resolved:** allocation cut alone won ~41% on arithmetic, so the guarded arithmetic superinstruction and the full ~70-primitive window-status ABI migration were deliberately not pursued — deferred to `U-IC`.
- [x] **U-TRACE** — [plan](units/U-TRACE/plan.md) · [perf-log 003](../forge/perf-log/003-vm-trace-feature-gate.md) — Tier 1: `vm-trace` Cargo feature (default **off**) compiles the dispatch loop's per-opcode `vm_opcode` span + three `debug!`s out entirely. Measured `arith_send` (5M, whole-process) **−16.7%**, zero golden diff, 239 tests green. Existed only as a one-line "next candidate" note in perf-log README until now. **Two mechanisms falsified on the way (finding F9):** the subscriber-config fix bought −0.4% (`main.rs` untouched), and the cost is *not* span-specific — it splits evenly with the `debug!`s, so all four callsites had to be gated; gating the span alone would have won half. `tracing` stays a hard dep (compiler + `vm/api.rs` are cold paths).
- [ ] **U-IC** — [plan](units/U-IC/plan.md) — Tier 3: selector-only interner + monomorphic inline cache + superinstructions, populates ADR-0012's reserved IC seam (currently a comment stub, `vm/dispatch.rs`). Not started — preconditions unmet per perf-log finding F4: `Symbol` is one mixed namespace (no `SelectorId` yet), `ClassObject` has no epoch/`world_version` field. Also carries the arithmetic fast path deferred from U-PRIM-ABI.
- [ ] **U-HOTPATH** — [plan](units/U-HOTPATH/plan.md) — dispatch-loop hot-path optimizations (register-hoisted interpreter state, behavior-invariant); natural follow-on to `U-IC`'s dispatch-loop work. Not started, blocked on `U-IC`.
- [x] **U-GC** — [plan](units/U-GC/plan.md) · [impl spec steps 3–5](units/U-GC/IMPL-SPEC-steps-3-5.md) — non-moving mark-sweep collector (ADR-0050). Steps 0–4 landed `94b6bbf`: roots/edge table, Win A (`Box` fat variants, 280B→40B), `trace_object`/`collect`/`force_gc`, `System.gc`, safepoint latch (Invariant L) — 14 GC tests green. **Step 5 (fiber-stack pool) landed `496912b`, gated behind `fiber-pool` Cargo feature (off by default alongside `vm-trace`, `phalcom-core/Cargo.toml:26-32`)** — code present in `heap/fiber.rs`, `primitive/fiber.rs`, `vm/{mod,bootstrap,dispatch,gc}.rs`, all `#[cfg(feature = "fiber-pool")]`; builds clean both with and without the flag (verified 2026-07-14). Kept opt-in rather than deleted because the A/B measurement showed net-negative (pool bookkeeping cost exceeds allocations avoided — post-sweep the pool competes against malloc, not a leak); flag exists so the experiment can be re-run/re-measured later without reconstructing it. **Correction:** earlier notes in this file claimed step 5 was reverted and only in `git stash@{0}` (`U-GC-pool-implementation-null-result`) — that was wrong, the stash is a stale duplicate of what's now properly landed behind the flag; safe to drop the stash. `DEFERRED.md` M-RUNTIME temp-root note, formal perf numbers under the flag, miri lane, and reviewer gate (`heap/`/`vm/` spine files) still open before the unit is fully closed.
- [ ] **U-COMPILE** — [plan](units/U-COMPILE/plan.md) — Tier 5: compile-time/startup optimization (constant dedup, cache `core.ph` compile). Last — needs the dispatch/heap shape from the tiers above settled first.

## 12. Naming conventions (opportunistic, no urgency)

- [x] **U-NATIVE-MARKER** — [plan](units/U-NATIVE-MARKER/plan.md) — `3e362e2` mechanical rename, native/private primitive `raw*` prefix → trailing `_` suffix (Wren convention).

---

## Suggested next dispatch (given current state)

1. **Close out `U-GC`** (§11) — steps 0–4 landed, step 5 done as a second null result (stashed, not shipped); still needs `DEFERRED.md` M-RUNTIME note, perf numbers, miri lane, and `phalcom-reviewer` sign-off (spine files).
2. **`U-SEQ`** (§5) — unblocked, `U-ITERABLE`'s golden-suite regression is fixed.
3. **ADR-0061 ratification** (§9-adjacent, prefix reservation) — spec written and citation-verified (`51ab8eb`), needs user ruling before `_`/`_$`/`__` prefix enforcement work can start in lexer/parser/compiler.
4. **`U-INDEX` perf re-measurement** — `phalcom-perf --bench-only` wasn't run after landing; record the expected small `[]`-vs-`.at` overhead delta (§8, plan.md Perf section) rather than assuming unchanged.
5. **`U-STRING`** (§5) — independent of Iterable/Seq, can dispatch in parallel with `U-SEQ`.
6. Walk the remaining perf tiers (§11) as time allows — lowest urgency, gated on wanting perf work at all.
