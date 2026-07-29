# Units tracker — by feature, oldest → newest

Cross-cutting index over [`units/`](units/), grouped by feature area instead of by unit
number. Complements, doesn't replace, the status of record (`STATE.md` +
as-built specs). Within each group, units are ordered by actual landing sequence
(oldest first); unchecked items are proposed dispatch order for future work, not fact.

Audited against `git log --oneline --all` + `git status --short` on `9daeb43` (2026-07-13).
**Re-audited against the tree at `de49d3a` (2026-07-19)** — the 2026-07-13 pass had gone
~60 commits stale and three rows were materially wrong: `U-SEQ` and `U-STRING` were marked
not-started but had fully landed, and `U-IC`/`U-HOTPATH` were marked not-started/blocked
while each is partially built. Every row in §1–§4 was re-verified clean (commits exist,
docs exist, features present in source). Rows now carry per-piece ✅/◐/❌ marks where a
unit is partial, so "landed" never again hides an unbuilt half.
Concurrent sessions are live on this branch — re-verify file/commit references before
trusting them if time has passed.

**Citation sweep 2026-07-20 against `e33e8e5`.** All 24 `file:line` citations in this file were
re-resolved by opening each cited symbol. **15 of 24 had drifted** — worst case `vm/mod.rs:101`
→ `:173` (~70 lines) and `dispatch.rs:870-885` → `:477-483` (the variadic-selector cache moved
~390 lines). The 9 that held: `interner.rs:10`, `heap/class.rs:25-40`, `value/mod.rs:121`,
`chunk.rs:10-18`, `phalcom-lsp/src/index.rs:147`, `lexer.rs:216`, `core.ph:84-360`,
`core.ph:694-777`, `findings.md:329`. The sweep also found **two rows whose *status* was false,
not merely their line numbers** — `U-CLASSNS` and `U-CLASSCLOSE` both read "design only, zero
code" while both had landed (`d3b6cd2`/`8b4465c`/`14cdfb9`, then `7c2cfab`). That is the same
failure this file's 2026-07-19 note already records for `U-SEQ`/`U-STRING`, recurring within
24 hours.

Both findings are filed as unowned items in
[`../deferred/doc-citation-integrity.md`](../deferred/doc-citation-integrity.md) — item 1 the
rot, item 2 the false-status asymmetry (rows always *understate* what exists, never overstate,
which points at the fix).

> **Line numbers in this file are the least durable thing in it.** They drift on any commit that
> touches the cited file, and nothing checks them. Prefer citing a *symbol* (`fn invoke_at`,
> `pub struct ClassKey`) — greppable and stable — and treat any bare `file:line` older than a few
> commits as a hint about *which file*, not *which line*. Re-grep before quoting one into a
> decision record.

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
- [x] **U-REOPEN-FIX** *(follow-on to U7/U13, no own unit folder)* — `e85f31a`/`a9e1eaf` class-reopen was dropping/corrupting methods and field layout; reopen now appends methods and rejects field-adding/superclass-changing reopens. **Superseded in intent by U-CLASSCLOSE below** — the feature it repaired is being removed.
- [x] **U-CLASSNS** — [plan](units/U-CLASSNS/plan.md) · [PDR-0001](../pdr/0001-classes-are-closed.md) — Unit A of two. **VM half LANDED; the "design only, zero code" claim this row previously carried was already false** (re-audited 2026-07-20 against `e33e8e5`): `d3b6cd2` (field_layouts + `ClassLayout.declared_at`), `8b4465c` (`vm.classes`), `14cdfb9` (sealed-check self-lookup must not fall back to core). ✅ Class identity re-keyed from name to `(module, name)` via `ClassKey { module: ObjRef, name: Symbol }` (`vm/mod.rs:74-79`); all four maps now `ClassKey`-keyed — `VM::classes`/`field_layouts`/`class_parents`/`sealed_classes` (`vm/mod.rs:173,238,274,296`). The **live silent bug is fixed**: two modules each declaring `class Point` no longer collapse into one `ClassId`. ✅ `SuperSend` (`dispatch.rs:877`) re-keyed — **shipped unmeasured**; measure current state before comparing against any pre-U-CLASSNS number ([deferred item 1](../deferred/class-sealing-followups.md)). ✅ **LSP half DONE 2026-07-20** — spec §8's collapse, not a re-key: `ClassMap` is now `DashMap<(Url, String), ClassEntry>` and the `Vec` is deleted (it only ever modelled cross-file reopening, which U-CLASSCLOSE removed). `class_members`/`class_parent`/`has_class` + the `collect_class_members` walk are file-scoped; `Url` is the module proxy (a file is a module, ADR-0045; this crate never resolves `import`). Fixes two live wrong answers — members unioned across files, and `parent()` returning the first entry that named *any* superclass. 6 fixtures, **each verified to fail against the old semantics** before the change; 113 tests green. Deliberate behavior change: cross-file inheritance completion stops resolving (it was resolving to an unrelated class). Write-up: [2026-07-20-u-classns-lsp-classmap-collapse.md](../logs/2026-07-20-u-classns-lsp-classmap-collapse.md). **U-CLASSNS is complete.** Follow-ups in [`class-sealing-followups.md`](../deferred/class-sealing-followups.md) (`8ed448c`); item 7 closed, item 8 (hover's separately name-keyed `DefinitionMetaMap`) newly opened and unaudited.
- [x] **U-CLASSCLOSE** — [plan](units/U-CLASSCLOSE/plan.md) · [PDR-0001](../pdr/0001-classes-are-closed.md) — **LANDED `7c2cfab`; the "design only, zero code" claim this row previously carried was already false** (re-audited 2026-07-20 against `e33e8e5`). Unit B of two, unblocked by U-CLASSNS. Retires ADR-0026 **Axis 1**; Axis 2 (reparenting sealed) kept. Class reopening is gone: ✅ redefinition-in-module is a compile error carrying both spans — `class.already_defined` (`compiler/lib/error.rs:194`), with `class.duplicate_member` for duplicates in one body (`:208`); ✅ the `Cannot reopen class X` wording retired with the feature — **zero hits in `phalcom-core/src/`**; ✅ the runtime reopen seam in `dispatch.rs` was **deleted, not gated** (the commit is net −82 lines there, removing the ADR-0018 "attaches methods, not a new class" block); ✅ all four `class_reopen_*` fixtures deleted — **zero remain** in `tests/lang/classes/`. Implementation log: [2026-07-20-u-classclose-two-issues-and-five-restored-tests.md](../logs/2026-07-20-u-classclose-two-issues-and-five-restored-tests.md). **Key finding that shrank this unit:** core.ph contains *zero* true reopens — every class is declared exactly once and `add_class!` never writes `field_layouts`, so its 22 "kernel reopens" are stub-completions on an already-separate code path. ⚠️ **Left a real hole:** five goldens that reopened a *kernel* class to flip an override-epoch flag or bust an inline cache (`Number#toString`, `Bool#and` ×2, `Block#whileTrue`, `Option#match`) no longer exercise the `.ph` override path — [deferred item 6](../deferred/class-sealing-followups.md). The nested-class ban shipped as a **syntax rule, not an invariant** ([item 5](../deferred/class-sealing-followups.md)).

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
- [x] **U-ITERABLE** — [plan](units/U-ITERABLE/plan.md) — bare-cursor Route B (raw index cursor + `None` end-sentinel, no per-step `Some` allocation) + kernel `Iterable` root (`core.ph:646`) hoisting `each`/`map`/`filter`/`reduce` (ADR-0048). Golden suite rebaselined off the pre-Route-B Option-wrapped-cursor protocol: `list_wren_iterate_cursor_protocol`, `map_wren_cursor_roundtrip`, `range_cursor_protocol_direct` (bare `0`/`1`/… + `None` sentinel, no `Some.new(_)`/`.unwrapOr`/`.isSome`), plus 3 negative fixtures (`map_wren_iterate_not_int/not_num`, `range_iterate_wrong_cursor_type`) updated from the old `does not understand 'map(_)'` dNU to the new arithmetic rejection `Expected String, got number` — `cursor + 1` on a non-numeric cursor is the natural error now, not a `.map(_)` dispatch on a non-`Option`. `./scripts/verify.sh` green for all of these. Gate-clean for Route B itself — **unrelated** pre-existing gap remains in `indexing`/`indexing_negative` (bracket-selector method-definition syntax `[i] { ... }` and empty-bracket calls `xs[]` don't parse at all; predates this unit, `26747d0`) — tracked separately, not a U-ITERABLE regression.
- [x] **U-SEQ** — [plan](units/U-SEQ/plan.md) · [spec](units/U-SEQ/implementation-spec.md) — `606829e` combinator breadth + lazy view classes (steps 1–2), `e6e767a` sugar wiring (step 3, **DEC-SEQ-A = Branch A**, BREAKING: `map` became lazy), `838244a` lazy-semantics comments. `Iterable` gains `all`/`any`/`count`/`count(_)`/`find`/`join`/`join(_)`/`toList`/`includes`/`isEmpty` + `where`/`skip`/`take` (`core.ph:694-777`); `MapView`/`WhereView`/`SkipView`/`TakeView` all `extends Iterable` (`core.ph:1366-1443`). Fixtures: `tests/lang/sequence/` — 21 positive + 2 negative, every fixture spec §4 names is present; `sequence` lane green (`cargo test -p phalcom-core --test lang` → 46 passed / 0 failed, verified 2026-07-19). **Open:** no `as-built.md`; spec §5's `all_generator_raises.ph` fiber fixture was never written (only `concurrency/each_generator_raises.ph` exists); Branch-A's `map`→lazy break was not swept outside `tests/` (benchmarks/, examples/).
- [x] **U-STRING** — [plan](units/U-STRING/plan.md) — `0bae56d` `ArgumentError` boundary-guard class · `3a37b19` raw primitives + `System.rawWrite` (ADR-0049) · `bd3f492` String protocol + write funnel (steps 3–6) · `18b7c9a` core rework (`split()` fix, UTF-8 `codePointAt`) · `3b2dd97`/`a21367d` `trimStart`/`trimEnd` charset iteration + stop-on-mismatch · `64de2bf` negative-lane guards + `bytes`/`codePoints` golden. `String` protocol at `core.ph:84-360`, `StringByteSequence`/`StringCodePointSequence` at `:363`/`:388`. `string`/`strings`/`strings_negative` lanes green. **Open:** no `as-built.md`.

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
- [x] **M-ATTR-ROOT** — [handoff](HANDOFF-M-ATTR-ROOT.md) *(no `units/` folder)* — *(row added 2026-07-19, previously untracked)* attribute storage infra on `Class`/`Method`/`Module` + `__attach`/`__attributes`/`__freezeAttributes` primitives (`3960036`), then `Attribute`/`@On`/tier-singleton retention + validation (`17226fd`), format/follow-ups `21636b1`/`d2d745d`. Surfaces in `core.ph` as `class Attribute`, `class On is Attribute`, the `Tier`/`Compile`/`Layout`/`Install`/`Dispatch`/`Runtime` singletons, and `Behavior`/`Method` `attributes`/`attributesOfType(_)`. **Ships the `__`-prefixed names that [ADR-0061](../adr/proposed/0061-underscore-prefix-reservation-fields-internals-reserved.md) would reserve** — that ADR is unratified, so this naming is provisional.
- [x] *(partial — 2 of 8)* **Decorators track** — [PLAN-DECORATORS.md](PLAN-DECORATORS.md) *(no `units/` folder)* — *(row added 2026-07-19, previously untracked)* design fully ratified, implementation barely begun. **Landed:** `8f6140e` Install-tier mechanism ADRs + 8 named decorators specced · `787d202` B-1/B-2/D-1/D-2/D-3 resolved (all option (a)) · `c3a0684` ADR-0054 Install/Dispatch/Runtime tiers ratified · `21811a6` decorator spec status headers synced to Accepted · `b222985`/`9f68a56` ADR-0057 decorator/proxy granularity split · `385fed9` `Tracer`/`OffBehavior`/`Backoff` core classes (pure-`.ph` library scaffolding, shipped **ahead of** the mechanism that would use them) · `2f13bfe`+`f874e6c` `@native`/`@ignore` specced and **registered** as subtractive Compile-tier no-ops in `expand_class_attributes` · `df259f3` plan updated after the M-ATTR-ROOT unblock. Reactivity groundwork: `47e4a58` reactivity.md ratified + ADR-0058, `c1c06eb` ADR-0059 reactive-tracking-context guard. **NOT built:** the other 6 named decorators, and the Install/Dispatch/Runtime mechanism itself. See `@native` is an LSP-only `.ph` anchor for a Rust impl; `@ignore` is the sanctioned ignore — they are LSP-only `.ph` anchors today and need a subset-invariant check or they drift.

## 10. Tooling (`vsphalcom`, `phalcom-lsp`)

- [x] **U-VSPHALCOM** — [plan](units/U-VSPHALCOM/plan.md) — extension modernization for spec v0.2: `db190f4` move in-tree + grammar rewrite, `20036eb`/`206f0b9` `phalcom check` CLI + diagnostics wiring, `122fd40`/`a568ab6` autocomplete, `9daeb43` hover (keyword docs, selectors, Phaldoc, contract stub) — commit message notes hover is "complete pending dev-host verification." **Six later commits were missing from this row** (added 2026-07-19): `b6562ca` shell-injection fix in `run.ts` quoting, `3e09670` LSP-vs-subprocess diagnostics made mutually exclusive, `9bf054b` Phaldoc hover decoupled from core-selector matching, `c49b28b` live snippet tab-stops + class/call color semantics, `a295562` manual-test fixtures + dev-host checklist, `268574b` Run Phalcom File button. Extension lives at `tools/vsphalcom`, not a repo-root `vsphalcom/`.
- [x] **U-LSP** — [plan](units/U-LSP/plan.md) — *(row added 2026-07-19; this unit had **no tracker presence at all** despite five landed stages and a workspace crate)* in-process `phalcom-lsp` crate. Stage 1 `d935575` diagnostics + client flag · Stage 2 `b1a3636` workspace symbol index, go-to-def, find-refs · Stage 3 `ba4bf25` receiver-aware completion (merge `fb15fd0`) · Stage 4 `42b1b7b` server-side hover (merge `a0c1c03`), accuracy follow-ons `086d4a7`/`becf7c9` · Stage 5 `5e7598d` `semanticTokens/full` via a flat lexer pass. Client migration completed by `268574b`. Tests under `phalcom-lsp/tests/` (e.g. `stage4_hover.rs`). **Note:** [ADR-0056](../adr/proposed/0056-phalcom-lsp-architecture.md) governing this crate is still filed **Proposed** — shipped-under-Proposed, same status/reality gap class as 0028/0036/0037/0040 before their fix.
- [ ] **U-REPL** — [plan](units/U-REPL/plan.md) — **Stages 0–5 landed; Stage 6 deferred.** §D2 (`16b3760`); §D7 parser half (`2fe6aba`); §S8 (`380461c`); Stages 0–2 (`3e118ab`); Phase B with multi-line continuation, snapshot oracle, refine-only highlighting, completer, and the `:reload` command namespace. ⚠️ **The earlier "fully landed (Stages 0–6) / all 6 load-bearing tests green" claim was wrong on both halves** and is corrected here. **Stage 6 (§S5 L2 highlighting, §D8's LSP-backed completion) is unbuilt** — `grep phalcom_lsp phalcom-repl/src/` returns zero hits — and is deferred pending ADR-0056's ratification ([PDR-0009](../pdr/0009-defer-lsp-backed-repl-surface.md)). §S6 debounce is also unbuilt. Of the 6 load-bearing tests, **2 were vacuous**: `trailing_backslash_joins_before_lexing` passed a Rust `\n` escape containing no backslash, and `value_echo_survives_raising_tostring` asserted `contains("BadString")`, which the native debug form also satisfies — both passed against a broken mechanism. Six defects found and fixed in `dcc4420` + `8466867` (echo never dispatched `toString`; parse, compile, and file-mode runtime errors all reported nothing; tracebacks were always empty; the read loop spun forever on non-tty stdin). Rulings: [PDR-0006](../pdr/0006-repl-completeness-is-a-parser-signal.md), [PDR-0008](../pdr/0008-cell-boundary-diagnostics-and-state-hygiene.md), [PDR-0009](../pdr/0009-defer-lsp-backed-repl-surface.md). Still owed: 5 specced completer/highlighter tests, and `:reset`/`:help` (deferred on purpose).

## 11. Performance, GC & benchmarking

Tiered strategy per `docs/spec/current/performance.md` ("measure first") — `U-BENCH` gates
everything else in this group; the rest can reorder based on what the baseline shows.

- [x] **U-BENCH** — [plan](units/U-BENCH/plan.md) — `ebe9d97` Tier 0: Wren-suite reference programs + plan landed (perf harness binary itself still pending). Blocks every later perf tier.
- [x] **U-PRIM-ABI** — [plan](units/U-PRIM-ABI/plan.md) · [perf-log 001](../forge/perf-log/001-prim-abi-inline-args.md) — `37f31c9` Tier 2: on-stack `[Value;8]` arg buffer replaces the per-send heap `Vec` in the primitive dispatch path. Measured: `arith_send` −41.5%, `bare_send` −33.8%, zero golden diff. **DEC-PRIM-B resolved:** allocation cut alone won ~41% on arithmetic, so the guarded arithmetic superinstruction and the full ~70-primitive window-status ABI migration were deliberately not pursued — deferred to `U-IC`.
- [x] **U-TRACE** — [plan](units/U-TRACE/plan.md) · [perf-log 003](../forge/perf-log/003-vm-trace-feature-gate.md) — Tier 1: `vm-trace` Cargo feature (default **off**) compiles the dispatch loop's per-opcode `vm_opcode` span + three `debug!`s out entirely. Measured `arith_send` (5M, whole-process) **−16.7%**, zero golden diff, 239 tests green. Existed only as a one-line "next candidate" note in perf-log README until now. **Two mechanisms falsified on the way (finding F9):** the subscriber-config fix bought −0.4% (`main.rs` untouched), and the cost is *not* span-specific — it splits evenly with the `debug!`s, so all four callsites had to be gated; gating the span alone would have won half. `tracing` stays a hard dep (compiler + `vm/api.rs` are cold paths).
- [x] *(partial)* **U-IC** — [plan](units/U-IC/plan.md) — Tier 3. **Two of four pieces landed** (audited 2026-07-19 against `de49d3a`; the "not started, preconditions unmet" claim this row previously carried was already false). ✅ **Inline cache**: `InlineCache { class, method, version }` (`chunk.rs:10-18`) in `Chunk.caches: Vec<Cell<Option<InlineCache>>>` (`chunk.rs:62`), genuinely probed and refilled on the send path in `invoke_at` (`vm/dispatch.rs:433`, probe `:445` / refill `:461`) — `49e38b6` (world_version) / `d030908` (struct + side table) / `f5e41f1` (probe + refill, 4 coherence tests). ✅ **Superinstructions**: `InvokeLocal`/`InvokeConst` fused opcodes (`bytecode.rs:352,362`) + in-place `Chunk::fuse_superinstructions` with jump-target-safety tests (`chunk.rs:129`, tests `:171-325`) — `1d2baea` (cut 008). ❌ **Selector-only interner**: not built — `interner.rs:10` still has only `Symbol(u32)`, one mixed namespace; zero `SelectorId` hits in any `.rs`. ❌ **Per-class epoch**: not built — invalidation is a **global** `VM::world_version` counter (`vm/mod.rs:203`) bumped at every method-install site; `ClassObject` (`heap/class.rs:25-40`) has no epoch field. Still carries the arithmetic fast path deferred from U-PRIM-ABI.
- [x] *(partial)* **U-HOTPATH** — [plan](units/U-HOTPATH/plan.md) — dispatch-loop hot-path optimizations. **Not blocked on U-IC and not unstarted** — three of the plan's changes are in, one is not (audited 2026-07-19). ◐ **Change 1 (register-hoist)**: chunk/`Rc<Callable>` half landed — `run_until_inner` hoists the frame's `Rc<Callable>` behind a single `closure_id` compare (`vm/dispatch.rs:543-569`), `1531070` + `5254586` (F14 S1a, cut 007). `ip`/`stack_offset` deliberately **not** hoisted — still read from `frames.last()` and written back every instruction (`dispatch.rs:542` read, `:561` comment); the code carries an explicit "Do not extend this guard to cover a hoisted `ip`" comment (see the guard is a closure identity, not a frame identity). ◐ **Change 2 (kill derived-selector String allocs)**: variadic site fixed via `VM::variadic_selector_cache` (`dispatch.rs:477-483`), `debadfa`. The second site the plan named (`init …` class-init fallback) no longer exists — superseded by compile-time `constructor_aliases`. **Dead code deleted 2026-07-20**: `VM::init_selector_cache` was declared, zero-initialized and never read — removed from `vm/mod.rs`, `vm/bootstrap.rs`, `vm/gc.rs`. ✅ **Change 3 (branch-free class-of), closed 2026-07-20**: the arm-reorder half was already tried and measured — SCOREBOARD's investigated-not-landed table records "no measurable change; LLVM already ordered the match", dropped pre-landing. Per the plan's own rule ("keep the change only if it measures or is a wash"), reordering stays as-is. The other half of the change, `#[inline]` on `Value::class`, was genuinely untouched and has now been added (`value/mod.rs:121`) — zero golden diff, `cargo build`/`cargo test --workspace` green.
- [x] **U-GC** — [plan](units/U-GC/plan.md) · [impl spec steps 3–5](units/U-GC/IMPL-SPEC-steps-3-5.md) — non-moving mark-sweep collector (ADR-0050). Steps 0–4 landed `94b6bbf`: roots/edge table, Win A (`Box` fat variants, 280B→40B), `trace_object`/`collect`/`force_gc`, `System.gc`, safepoint latch (Invariant L) — 14 GC tests green. **Step 5 (fiber-stack pool) landed `496912b`, gated behind `fiber-pool` Cargo feature (off by default alongside `vm-trace`, `phalcom-core/Cargo.toml:26-32`)** — code present in `heap/fiber.rs`, `primitive/fiber.rs`, `vm/{mod,bootstrap,dispatch,gc}.rs`, all `#[cfg(feature = "fiber-pool")]`; builds clean both with and without the flag (verified 2026-07-14). Kept opt-in rather than deleted because the A/B measurement showed net-negative (pool bookkeeping cost exceeds allocations avoided — post-sweep the pool competes against malloc, not a leak); flag exists so the experiment can be re-run/re-measured later without reconstructing it. **Correction:** earlier notes in this file claimed step 5 was reverted and only in `git stash@{0}` (`U-GC-pool-implementation-null-result`) — that was wrong, the stash is a stale duplicate of what's now properly landed behind the flag; safe to drop the stash. **Closeout re-audited 2026-07-19, three loose ends closed 2026-07-20:** ✅ `DEFERRED.md` M-RUNTIME temp-root note **written** (`DEFERRED.md` Open entries — flags the unbuilt `aroundSend`/`Invocation` Runtime decorator tier as the one forward obligation the landed `temp_roots` API must be wired against). ✅ **Formal perf numbers under the flag are DONE** — `perf-log/findings.md:329-402` (F10) carries the full same-machine A/B at 100k/500k/1M fibers (1M: user 0.62→0.85 s **+37%**, RSS 635→1090 MB **+72%**), landed `9207fac` on 2026-07-14. ✅ **miri lane wired into CI** — `.github/workflows/ci.yml` gained a `miri` job (nightly + `miri` component, `cargo +nightly miri test -p phalcom-ast`, mirrors `scripts/verify.sh --miri`'s scope; phalcom-core's `invariants` suite is out of scope per docs/forge/perf-log/011-attack-on-010.md §11 — it does not finish inside a CI timeout). ✅ **reviewer sign-off done 2026-07-20** — [docs/logs/2026-07-20-u-gc-spine-review.md](../logs/2026-07-20-u-gc-spine-review.md), reviewed against `7480d75~1..94b6bbf` (heap/vm spine) + `cdd2117` (temp_roots UAF fix). **Verdict: APPROVE**, no blocking finding; one robustness follow-up filed to `DEFERRED.md` (`block_ensure`'s temp-root catch-all is correct today but not future-proof against a new `RuntimeError` variant carrying a `Value`).
- [ ] **U-COMPILE** — [plan](units/U-COMPILE/plan.md) · [spec](units/U-COMPILE/implementation-spec.md) — Tier 5: compile-time/startup optimization (constant dedup, cache `core.ph` compile). **NOT STARTED — confirmed 2026-07-19**, only the doc-only commit `242daee` touches it. All four preconditions still describe HEAD verbatim: `core.ph` is re-`include_str!`'d and recompiled on every `VM::new` with no memoization (`vm/bootstrap.rs:171,179`); `Chunk::add_constant` pushes unconditionally, no dedup (`chunk.rs:98`); `scan_number` runs `slice.replace('_', "")` whether or not a separator is present (`phalcom-ast/src/lexer.rs:216`); no U-COMPILE commit touches `compiler/lib/scope.rs`. Last — needs the dispatch/heap shape from the tiers above settled first.

Landed perf cuts with **no unit of their own** (row added 2026-07-19; only cuts 001/002/003
were reachable from this file, via U-PRIM-ABI/U-GC/U-TRACE). Full detail in
[`perf-log/`](perf-log/); the scoreboard is the single source of record for numbers —
`perf-log/SCOREBOARD.md` is the only source of record for numbers.

- [x] **Cut 004** — `0274f10` stop inlining inside deopt-fallback copies (F13).
- [x] **Cut 006** — `916be0a` drop `spans[ip]` from the dispatch read-decode (F14 S2).
- [x] **Cut 007** — `5254586` hoist the frame's `Rc<Callable>` out of the dispatch loop (F14 S1a). Also counted under U-HOTPATH Change 1 above.
- [x] **Cut 008** — `1d2baea` fuse `(GetLocal|Constant)→Invoke` superinstructions. Also counted under U-IC above.
- [x] **F11** — `4f2eed8` yield-adaptive GC threshold (grow `next_gc` 4× when yield <10%). skynet −11.7% user, −8% RSS.
- [x] **F12** — `39d9042` per-callsite global slot cache, version-guarded; `3f22a70` locks shadow invalidation in the `ic` golden lane. skynet → 2.9× Wren.
- ~~**Cut 009**~~ — `18a57af` `GetSelf→GetField` fusion — **shipped and measured a LOSS** (F21/H16). Documented as a null result rather than reverted; do not re-propose without reading `perf-log/009-*`.
- [ ] **Cuts 010/011** — `54dae3a` pre-registration of the H16/H17/H13 batch; `011-attack-on-010.md` is **uncommitted in the working tree** and corrects both cut 010 and SCOREBOARD, which still assert the refuted L1i/128-byte claims (the L1i and 128-byte-init claims were refuted statically). Commit it or the wrong claims stay published.
- [ ] **H17 — the one un-timed live lever.** The `core.ph` cursor-protocol probe deletes −10.2% of `for.ph`'s instructions and 2 `.ph` frames/element, stdout byte-identical. **No wall-clock exists** — `ab-guarded.py` refused on machine load. Needs a quiet box, and needs kernel-collection sealing (the `8d401f4` `Option` precedent) before it can land at all. See `for` is a 4-frame-per-element `.ph` call chain.
- [x] **U-PERF** — [work-1](units/U-PERF/work-1.md) · [work-2](units/U-PERF/work-2.md) — session ledgers for the perf investigation track itself (`81d494c`, `54dae3a`), including the ideas that failed and why.

## 12. Naming conventions (opportunistic, no urgency)

- [x] **U-NATIVE-MARKER** — [plan](units/U-NATIVE-MARKER/plan.md) — `3e362e2` mechanical rename, native/private primitive `raw*` prefix → trailing `_` suffix (Wren convention).

## 13. Bindings & constructors — **ratified by ADR, zero code**

*(Section added 2026-07-19. Neither unit had any tracker presence, and both are among the
largest pieces of designed-but-unbuilt work in the repo. Dependency order is fixed:
U-BINDINGS lands **before** U-CTOR, because the field grammar is load-bearing for it.)*

- [ ] **U-BINDINGS** — [plan](units/U-BINDINGS/plan.md) · [ADR-0064](../adr/accepted/0064-let-const-bindings-and-field-mutability.md) — **design only, no implementation.** Only commit is the docs commit `47df92e`. Supersedes ADR-0014 on *spelling*: `var`→`let`, `let`→`const`, mutable fields take no keyword, and one genuinely new rule — `const` field writes are legal only inside a `@constructor` (syntactic, no flow analysis). Motivated by a measured hole: `let` on a *field* is unenforced today (`clobber(v) { _n = v }` on a `let _n` succeeds silently). **Main risk is the 1080-site codemod** (352 `var` + 728 `let`, 395 files): it is a *swap*, so a naive two-pass `sed` turns `var`→`const`; it is also position-dependent (class-body `var _x`→`_x` vs statement `var x`→`let x`). Must be single-pass and AST-driven.
- [ ] *(wip patch only)* **ctor** — [plan](../work/pending/ctor/plan.md) · [ADR-0063](../adr/accepted/0063-constructors-are-ordinary-class-side-methods.md) — archived implementation unit; its historical plan is superseded at surface level by [PDR-0028](../pdr/0028-class-and-constructor-decorator-canon.md). Preserve its implementation notes for gradual migration.

## 14. Confirmed-backlog fixes (CB-1…CB-6) — all landed

*(Section added 2026-07-19; these had no tracker row. Full evidence in [`DEFERRED.md`](DEFERRED.md) §Confirmed Backlog, which is the status of record. All six are verified-against-tree and fixed. Worth remembering that **4 of the 6 original prescriptions were wrong** and two would have broken the tree — see a verified diagnosis is not a verified fix.)*

- [x] **CB-1** — `22cc756` derive `toString` for `Map`/`Set`/`Tuple`/`Range`; `122ae3f` `\(…)` now sends `toString` (getter send, not a 0-arg call) + ADR-0022 prose amended.
- [x] **CB-2** — `0911656` floor census reconciled to the machine-checked 125, naming the test as source of record; `9f17ca9` writes the five missing amendment banners, closing the 125/110 chain.
- [x] **CB-3** — `3859974` `@variant` gate reads the union of both sealing sources; S-2 dissolved.
- [x] **CB-4** — `1610efa` retire `experimental/default-arguments.md`; ADR-0043 prose amended to Q12's fixed mechanism.
- [x] **CB-5** — `b285139` admit `Fiber`'s 11 primitives to the audited floor (125 → 136).
- [x] **CB-6** — `e65b5bd` render by sending `toString` (`System.print` had bypassed every container); `d9a3dee` leaf-`toString` fast path guarded by an override-epoch flag.

## 15. Documentation & examples (no unit, no plan)

- [x] **`docs/learn/` module series** — *(row added 2026-07-19)* a full pedagogical track with zero unit presence, authored per [`docs/learn/AUTHORING.md`](../learn/AUTHORING.md). Object-model track: `dda2ab0` upvalues, `5b06736` metaclass-tower. VM track: `24f867a` execution-loop (Doc 1), `edfacf8` compiled-artifact (Doc 2), `39e8e49` frames (Doc 3), `de49d3a` message-send (Doc 4). **Doc 5 (caches-and-fusion) is written but uncommitted** (`docs/learn/vm/caches-and-fusion.md` + its `docs/learn/caches-and-fusion/` working set); **Doc 6 (identity) not started**.
- [x] **`examples/sheetcalc/`** — spreadsheet-engine example app: `05b8141` spec + commentary, `7b652a1` phase 1 (support layer + value model), `7ee621d` L1/L2 smoke tests.
- [x] **`ea4c8f3`** — dispatch bug fix with no unit home: selector encoding moved from colon-form to ADR-0012 comma-form, fixing two live bugs (`send_eq` hardcoded colon-form; sacred-selector sets were colon-form).

---

## Suggested next dispatch (rewritten 2026-07-19 after the full re-audit)

The old list is deleted, not amended — items 2 and 5 (`U-SEQ`, `U-STRING`) had already
landed, and item 1's premise about `U-GC` step 5 was wrong.

**Blocked on a user ruling — nothing downstream can start:**

1. **ADR-0061** (underscore prefix reservation, `_`/`_$`/`__`) — designed, citations re-verified, zero code. Blocks prefix enforcement in lexer/parser/compiler *and* the M-ATTR-ROOT `__attach`/`__attributes` rename (§9), which ships those names provisionally today.
2. **ADR-0059** (reactive tracking context bound to the native-frame switch guard) — Proposed, needs ratification.
3. **ADR-0056** (`phalcom-lsp`) — Proposed while the crate has five landed stages (§10). Status/reality gap, needs a ruling not an implementation.
4. **Decorator spec status confirm** (soft) — ADR-0054 ratifies the mechanism/tiers; the four named-decorator specs still read "Proposed" though their own questions are resolved inline. Confirm the coverage or bump the headers.

**Ready to dispatch, no ruling needed:**

5. **`U-BINDINGS`** (§13) — the largest unbuilt unit, and U-CTOR is stuck behind it. AST-driven single-pass codemod, 1080 sites.
5b. **`U-CLASSNS` → `U-CLASSCLOSE`** (§3) — [PDR-0001](../pdr/0001-classes-are-closed.md), ruled 2026-07-19, both plans written, **no implementation specs yet** (ruled: dispatch on plans, revisit if the first one bites). Strictly ordered internally: namespacing first (its `(module, name)` re-key is what makes the redefinition error decidable), closing second. **Runs after U-BINDINGS** (ruled 2026-07-19) — U-BINDINGS is fully specced and blocks U-CTOR, so it goes first even though its §12C L-5 reopen exemption is throwaway work that U-CLASSCLOSE §3.3's core-module gate makes unnecessary. **No `core.ph` conflict in either order** — neither of these two units touches that file. Whichever lands second collapses the exemption into the gate rather than leaving two special cases.
6. **Close out `U-GC`** (§11) — three items left: write the `DEFERRED.md` M-RUNTIME temp-root note, put the miri lane in CI (it exists only as a `verify.sh` flag), get `phalcom-reviewer` sign-off on the `heap/`/`vm/` spine diff. The perf-numbers item is already done (F10).
7. **Finish `U-HOTPATH` Change 3** (§11) — branch-free class-of is the one untouched piece, and `VM::init_selector_cache` is dead code to delete while in there.
8. **Commit `011-attack-on-010.md`** (§11) — it is uncommitted and SCOREBOARD still publishes the claims it refutes.
9. **`U-SEQ`/`U-STRING` as-built docs** (§5) — both units landed with no as-built; U-SEQ also never got spec §5's `all_generator_raises.ph` fixture, and its Branch-A `map`→lazy break was not swept outside `tests/`.
10. **`U-INDEX` perf re-measurement** (§8) — `phalcom-perf --bench-only` still not run after landing.
11. **Doc 5 + Doc 6 of the VM learn track** (§15) — Doc 5 written but uncommitted, Doc 6 not started.

**Lower urgency:** the rest of §11's perf tiers (`U-IC`'s two unbuilt pieces, `U-COMPILE`), gated on wanting perf work at all; H17 additionally gated on a quiet box *and* kernel-collection sealing.
