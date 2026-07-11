# Forge — Plan (building)

_Status: Phase 1 audit + 1b verification COMPLETE. Architect (Phase 2) pending. This file holds the VERIFIED finding ledger; the architect turns the survivors into the dependency-ordered plan._

## Verified finding ledger (post Phase 1b)

| ID | Finding | Location | Verdict | Live today? | Disposition |
|----|---------|----------|---------|-------------|-------------|
| F1 | `Invoke` discards `call_method`'s `Result` → primitive errors silently swallowed (empirical: `Number.new("abc")` → no output, exit 0). Stack-desync latent. No `?` (rustc `#[must_use]` warns). | `vm.rs:506` | **CONFIRMED** (2 auditors + empirical) | ✅ yes | **Bug-fix wave.** One-line `?`. |
| F9 | `SyntaxError`'s `Display::fmt` is `todo!()` → **any** parse error panics instead of a diagnostic (empirical: 4/6 examples + any trailing-newline file crash). | `phalcom-ast/src/error.rs:13` | **CONFIRMED** (stabilizer, empirical) | ✅ yes | **Bug-fix wave.** Implement Display. |
| F10 | Parser rejects a trailing `\n` at EOF → almost every real `.ph` file panics (compounds F9). | parser grammar (EOF handling) | **CONFIRMED** (stabilizer, empirical) | ✅ yes | **Bug-fix wave.** Grammar fix. Pairs w/ F9. |
| F4 | `object_name` returns `receiver.class(vm).name()` → `Number name`/`toString` ⇒ `"Number.class"`; instance `n.name` ⇒ class name. | `primitive/object.rs:10-12`, `universe.rs:105,108` | **CONFIRMED** (empirical) | ✅ yes | **Bug-fix wave** — but needs a **DECISION** on `name`/`toString` object semantics (see below). |
| F2 | Metaclass tower **inconsistent**: core metaclasses → `Class` (`universe.rs:70`); user metaclasses → `Object.class` (`vm.rs:92`); neither builds the spec's parallel hierarchy (ADR-0002 rule 4). | `universe.rs:70`, `vm.rs:92` | wiring CONFIRMED; harmful-consequence **REFUTED** | ⚠️ inert (no subclass syntax; `compiler/lib.rs:269` TODO) | **Foundational unit** = spec step 4 (metaclass fix + `verify_invariants()`). Not a live bug. Fold F5/F6 here. |
| F5 | `MaybeWeak` cycle-breaker inert (every `set_class_owned` → `Strong`; `Metaclass.class` self-cycle) → kernel never freed, weak path dead. | `universe.rs:49,57,62` | CONFIRMED (object-model) | leak only | fold into **F2 unit** |
| F6 | Apex collapsed: no distinct `Class class`/`Metaclass class`; `Metaclass class` apex absent. | `universe.rs:49,62` | CONFIRMED (med conf) | inert | fold into **F2 unit** |
| F3 | `runtime_error` does `module_source.unwrap()` on an always-`None` field. | `vm.rs:233` | **REFUTED** on severity — `runtime_error` is **dead code** (live CLI uses `eprintln!`; `Interpreter::run_file` has 0 callers). | ❌ unreachable | **DEFERRED** (cleanup: wire pretty-printer or delete dead path). |
| F7 | `object` static `new()` registered `Method(1)` for 0-arg selector (metadata mismatch). | `universe.rs:109` | inert (both auditors agree) | ❌ | **DEFERRED** |
| F8 | `Greater` opcode interns malformed selector `">( _)"` (stray space). | `vm.rs:542` | inert (operator dispatch is Tier-3 greenfield) | ❌ | **DEFERRED** |

### Immediate bug-fix candidates (existing aligned code, cheap, high value)
**F1, F9, F10, F4** — all live, all empirically reproduced. Independent of the greenfield tiers; can land as a first wave before foundational work.

### Open DECISION (surfaced by F4 verification)
`object_name` is installed as both `name` and `toString` on `Object`. Fixing "class name is wrong" requires deciding what `name`/`toString` mean on a **general object**, not just classes — the spec (`object-model.md`/`classes.md`) must be consulted; if it doesn't pin this, it's a **BLOCKED-ON-DECISION** for the user.

## Plan-of-record (spec recommended order — unchanged target)
1. Selector redesign (#1) · 2. Blocks (#2) · 3. operators→sends (#3) + nil→Option (#4) · 4. Metaclass tower fix + `verify_invariants()` · 5. Features #5–#10.

_Note: F1/F3/F4 are **bugs in existing aligned code** — cheap, high-value pre-work that should land before/independent of the greenfield tiers. F2/F5/F6 are the metaclass-tower-fix unit (step 4) with concrete sites now pinned._

---

# Phase 2 — Implementation Plan

_Philosophy (per orchestrator course-correction): the spec (`docs/spec/` + ADRs) is the
design source of truth. We build the **right** architecture to realize it — we are NOT
constrained to keep the current Wren/clox substrate (arity dispatch, operators-as-opcodes,
dynamic field maps, first-class `nil`, `Rc<RefCell>` + inert `MaybeWeak`). Where the current
code already is the best-practice target we keep it and say so; where it fights the spec we
design the replacement rather than patch onto a substrate the spec wants gone._

**Kept as-is (already best-practice target):**
- The `phalcom-ast` front end: lexer + LALRPOP grammar (`parser.lalrpop`) + AST. Only the two
  live front-end defects (F9/F10) are fixed; the pipeline shape stays.
- **Symbol interning** (`interner.rs`) — the substrate for label-encoded selectors.
- The **dot-send → `Invoke(selector_const)`** shape (`MethodCall`/`GetProperty`/`SetProperty`
  lower to `Invoke` with a selector *constant index*). The label channel slots into that
  constant; we do not redesign the call shape, only what the selector symbol encodes.
- `static` flag end-to-end and getter≠method / setter `name=(_:)` modeling.

**Bug disposition under the redesign:** F1, F4, F7, F8 are defects in code the redesign
*replaces outright*, so they are **folded into the unit that rewrites that code**, not
scheduled as standalone patches (noted per unit). **F9 + F10 stay as genuine front-end
fixes** because the front end is kept and *nothing can run through the golden corpus until
the CLI stops panicking on ordinary input* (trailing-newline file → F10 panic → F9 `todo!()`
Display). This is the one true pre-work wave.

## Target architecture (the decisions the whole design rests on)

Every later unit is an instance of one of these. Best-practice grounding is for a
Smalltalk-style bytecode VM in Rust (avoid the `RefCell` double-borrow panic and the
`Rc`-cycle leak the audit found in F5).

| # | Decision | Best-practice choice | Spec / ADR | New ADR? |
|---|----------|---------------------|-----------|----------|
| TA-1 | **Object-graph / heap ownership** | The load-bearing fork. Current `Rc<RefCell<T>>` + `MaybeWeak` is inert (F5) and leaks the kernel; Smalltalk semantics (cycles, mutable `superclass=` per open-Q4, `System.gc` per `system.md`) ultimately need a real collector. Recommend a **handle/arena heap**: objects live in a central `Heap`, referenced by a `Copy` integer handle (`ObjRef`/`ClassId`). Kills Rc-cycles, removes `RefCell` borrow-panic surface, is cache- and inline-cache-friendly (handles are IC keys), and can host a tracing GC later. | object-model §6; `system.md` (gc); open-Q4 | **ADR-0008** — **BLOCKED-ON-DECISION BD-1** |
| TA-2 | **Value representation** | Tagged `enum Value` (`Number(f64)`, `Bool(bool)`, `Obj(ObjRef)`, private `Nil` sentinel, interned `Symbol`). NaN-boxing is a *later* optimization behind the same API — **deferred register**, not the critical path. Clarity/safety first. | object-model §3; ADR-0005 (Number=f64) | **ADR-0009** (Value repr; ADR-0005 only covers Number) |
| TA-3 | **Selector / Signature model** | Selector = interned symbol encoding **name + labels** (`move(to:duration:)` ≠ `move(_:_:)`); one hashmap probe. `Signature { selector: Symbol, kind, positional_arity, variadic }`. `Invoke` keeps its selector-constant operand. Dispatch built **inline-cache-ready** (monomorphic slot per call site) even if the IC itself is deferred. | messages-and-selectors §2–3; method-lookup §1 | **ADR-0012** (encoding + Invoke + IC-ready dispatch) |
| TA-4 | **Instance layout** | **Static slot vector**: `InstanceObject { class, slots: Box<[Value]> }` indexed by a compile-time slot offset from a per-class field table. Replaces the dynamic `IndexMap<Symbol,Value>`. Fields private + non-inherited → offsets are stable → fragile-base-class problem gone. Unassigned slot reads `None`. | classes.md §2 | **ADR-0010** (static slot layout) |
| TA-5 | **Block / closure model** | One `ClosureObject` shared by `Block` and `Method` (siblings under abstract `Function`). **Lua-style open/closed upvalues** for capture; **frame token** (frame ptr + generation counter) for non-local return, raising `DeadFrameError` on a dead frame. | blocks.md §1–7; ADR-0006 | **ADR-0011** (upvalue + frame-token) — **BD-3** |
| TA-6 | **Control-flow-as-message + inliner** | `if`/`while`/`for`/`and`/`or`/operators desugar to sends; compiler inlines the *sacred selectors* on literal-block call sites to jump opcodes, guarded by a receiver-type check that **deopts to a real send**. Zero closure alloc on the hot path. | control-flow.md §1–3 | covered |
| TA-7 | **Absence = Option** | No surface `nil`. Abstract `Option` + `Some`(`_value`) + singleton `None`; combinators are per-subclass methods (dispatch replaces branching). Private `Value::Nil` never leaks into a `Some`. `if (opt)` is a compile error (no truthiness). | values-and-absence.md; ADR-0007 | covered |
| TA-8 | **Metaclass tower** | Parallel rule `(X class).superclass == (X.superclass) class`, anchored `(Object class).superclass == Class`; `Behavior` is the shared kernel superclass of `Class`/`Metaclass`; `verify_invariants()` runs after bootstrap. | object-model §5–6; ADR-0002/0003 | covered |

> **ADR gate:** ADR-0008..0012 are load-bearing and not yet recorded. Draft them with the
> `documentation-and-adrs` skill. BD-1 (ADR-0008) and BD-3 (ADR-0011) additionally need a
> **user decision** before their units can start (see BLOCKED-ON-DECISION list).

## Wave breakdown

Reality check: nearly every VM-core unit must touch `vm.rs`, `compiler/`, and `bytecode.rs`,
so the foundational spine **genuinely serializes** — do not try to fan it out. True parallel
capacity appears only after dispatch + blocks are stable, and only for units with disjoint
write-sets (`core.ph` stdlib authoring, lexer-only surface features, `frame.rs`-local work).

| Wave | Units | Parallel? | Gate |
|------|-------|-----------|------|
| 0 | **U0** front-end stabilization (F9+F10) | 1 lane | none — start now |
| — | *DECISION GATE* | — | resolve **BD-1, BD-3** (also BD-2, BD-4) before Wave 1 |
| 1 | **U1** object-graph & Value repr (TA-1/2/4 base) | alone | U0 + BD-1 |
| 2 | **U2** metaclass tower + Behavior + `verify_invariants()` (⊇ F2/F5/F6, F4-`name`/`toString`) | alone | U1 (+ BD-2) |
| 3 | **U3** selector/Signature + Invoke dispatch (⊇ F1/F7/F8) | alone | U2 |
| 4 | **U4** blocks/closures + `=>`/trailing-block (TA-5) | alone | U3 + BD-3 |
| 5 | **U5** control-flow-as-message + inliner | alone | U4 |
| 6 | **U6** absence→Option | alone (compiler-heavy) | U5 |
| 7 | **U7** static fields + `construct` | alone (compiler-heavy) | U2, U3, U6 |
| F | **U8** dNU/`Message`/`perform`/`SEND_DYNAMIC` ‖ **U10** non-local return ‖ **U-LEX** Tier-B surface ‖ **U-STD** `core.ph` stdlib | fan-out (see disjointness note) | U4/U6/U7 |
| F+1 | **U9** variadics + spread ‖ **U11** refinements (Bool True/False) | after U8 | U8 |

**Fan-out disjointness (Wave F).** `U-LEX` (lexer.rs/token.rs/parser.lalrpop/ast.rs) and
`U-STD` (`core.ph` only) are truly disjoint from each other and from the Rust VM units and
may run fully parallel. `U8` and `U10` both touch `vm.rs` → **sequence them** (U8 then U10),
or split `vm.rs`'s dispatch-miss path (U8) from `frame.rs`/return (U10) if kept surgically
disjoint. Schedule as: {U-LEX ‖ U-STD ‖ U8}, then U10, then U9.

## Critical path

`U0 → [BD gate] → U1 → U2 → U3 → U4 → U5 → U6 → U7 → U8 → {U9, U10}`.
The spine U1→U7 is irreducibly serial (shared `vm.rs`/`compiler/`/`bytecode.rs`). Speed items
(inline caches, NaN-boxing) and refinements (Bool→True/False, Int/Float tower) are in the
**deferred register**, off the critical path.

---

## Units

### U0 — Front-end stabilization (F9 + F10)
- **Goal.** Make parse errors render a diagnostic instead of panicking, and accept a trailing
  `\n` at EOF, so the CLI + golden corpus can run at all.
- **Spec/ADR.** lexical-structure §1 (newline handling); this is a correctness fix, no ADR.
- **Write-set.** `phalcom-ast/src/error.rs` (implement `Display for SyntaxError`),
  `phalcom-ast/src/parser.lalrpop` (+ `lib.rs` glue) for EOF/trailing-newline.
- **Depends on.** nothing. **Subsumes.** F9, F10.
- **Design.** Implement `Display` off the existing `SyntaxErrorKind` `#[error(...)]` messages +
  `format_expected`; surface the `range` via `phalcom-common`. Allow optional terminal newline
  in the top-level grammar rule.
- **Risk.** LALRPOP regen churn in `lib.rs`; ensure the newline fix doesn't make statement
  separation ambiguous (newline-as-terminator interacts with lexical-structure §1 later).
- **Test.** `phalcom-ast/tests/parser.rs` snapshot: a trailing-newline file now yields a
  readable diagnostic, not exit-101. Add its `.ph` fixtures back into `tests/golden.rs` (the
  golden runner explicitly waits on this). `./scripts/verify.sh` green.
- **Must-not-preclude.** Don't hardcode a newline-insensitive grammar that blocks the
  lexical-structure §1 newline-suppression state machine (U-LEX).

### U1 — Object-graph & Value representation  (TA-1, TA-2, TA-4 base)
- **Goal.** Establish the heap-ownership model, `Value`, and the base `ClassObject` /
  `InstanceObject` types the whole VM builds on. Removes the inert `MaybeWeak` (F5 wiring).
- **Spec/ADR.** object-model §3, §6; ADR-0005; **new ADR-0008/0009/0010**.
- **Write-set.** `phalcom-common/src/refs.rs`, `phalcom-core/src/value.rs`, `class.rs`,
  `instance.rs`, and the ownership plumbing in `universe.rs`/`vm.rs` (base types only — tower
  wiring is U2).
- **Depends on.** U0; **BD-1** (ownership model).
- **Design.** Per BD-1: handle/arena `Heap` with `Copy` `ObjRef`/`ClassId` (recommended), or
  the fallback `Rc<RefCell>` + intentional-kernel-cycle model. `Value` = tagged enum with a
  **private** `Nil` sentinel (TA-2). `InstanceObject.slots: Box<[Value]>` scaffold (TA-4);
  dynamic `IndexMap` field map deleted.
- **Risk.** This is the borrow-model fragility epicenter. If BD-1 picks `Rc<RefCell>`, encode
  a borrow discipline (no `borrow_mut` held across a send) or the F1-class double-borrow panic
  returns. Handle model must not hand out dangling handles across GC/compaction (none yet, but
  design the API for it).
- **Test.** `tests/invariants.rs` still builds/passes on the new types (pointer-identity
  helpers rewritten in terms of handles). No behavior change asserted here — this is a
  representation swap; green `verify.sh` is the gate.
- **Must-not-preclude.** Handle API must not foreclose a future tracing GC (TA-1 option C) or
  NaN-boxing (TA-2). Slot layout must not assume inheritance-visible fields (classes.md §2).

### U2 — Metaclass tower + `Behavior` + `verify_invariants()`  (⊇ F2, F5, F6; folds F4)
- **Goal.** Wire the parallel metaclass tower correctly, introduce `Behavior`, and make the
  bootstrap self-checking. Install the corrected universal `Object` protocol.
- **Spec/ADR.** object-model §5–6, §8; ADR-0002, ADR-0003.
- **Write-set.** `phalcom-core/src/universe.rs`, `class.rs`, `vm.rs` (`create_class` metaclass
  wiring at ~:92), `primitive/object.rs`, `primitive/class.rs`, `tests/invariants.rs`
  (un-`#[ignore]` the parallel-rule cases).
- **Depends on.** U1. (+ **BD-2** for the `toString` default.)
- **Design.** Allocate-then-wire bootstrap (object-model §6 steps 1–7). Parallel rule
  `(X class).superclass == (X.superclass) class`, anchor `(Object class).superclass == Class`,
  close `Metaclass.class == Metaclass class` / `(Metaclass class).class == Metaclass`.
  **F4 fix, spec-pinned part:** `name` is **not** universal `Object` protocol — it is
  `Behavior`-side, returning the receiver's *own* name (`Number.name → "Number"`,
  `(Number class).name → "Number class"`); `anInstance.name` is simply not understood (→ dNU
  once U8; method-not-found until then). The current `object_name` returning
  `receiver.class().name()` is deleted. `toString` stays universal on `Object` (display repr,
  BD-2); a class's `toString` is its own name.
- **Risk.** The bootstrap circularity (`Metaclass` instance-of itself) is exactly where
  ownership (U1) bites — allocate-uninit-then-wire must not trip the handle/`Rc` invariants.
- **Test.** `tests/invariants.rs`: all `#[ignore]`d parallel-rule tests now pass, plus
  `Number.class.superclass == Object.class`; add a `verify_invariants()` unit test asserting
  every object-model §5 check. This unit is the permanent regression guard for the tower.
- **Must-not-preclude.** Don't special-case `Metaclass` in a way that blocks user-defined
  metaclass methods; keep open-Q4 (runtime `superclass=`) *possible* (don't bake immutable
  offsets into the tower that a later mutability decision would have to unwind).

### U3 — Selector / Signature model + Invoke dispatch  (⊇ F1, F7, F8)
- **Goal.** Replace arity-only dispatch with label-encoded selectors; make failed/errored
  sends propagate correctly.
- **Spec/ADR.** messages-and-selectors §2–3; method-lookup §1; **new ADR-0012**.
- **Write-set.** `phalcom-ast/src/ast.rs` (labels on `MethodDef`/call args),
  `phalcom-ast/src/parser.lalrpop` (label `to:` syntax), `phalcom-core/src/signature.rs`,
  `method.rs`, `bytecode.rs` (Invoke operand semantics), `compiler/lib.rs` + `compiler/mod.rs`
  (selector construction), `vm.rs` (dispatch + the `call_method` `?`), `interner.rs` if needed.
- **Depends on.** U2 (dispatch walks the corrected tower). **Subsumes.** F1 (the missing `?`
  is folded into the rewritten `Invoke` handler), F7 (correct 0-arg metadata), F8 (correct
  operator selector encoding, no stray space).
- **Design.** Intern the label-encoded string (`add(_:_:)`, `move(to:duration:)`, `name=(_:)`,
  `+(_:)`) → `Symbol`. `Signature { selector, kind, positional_arity, variadic:false }`.
  Method dict keyed by selector symbol; one probe. Dispatch structured **IC-ready** (a
  per-call-site monomorphic cache slot), IC population deferred.
- **Risk.** Selector-string canonicalization must be the single source of truth shared by
  compiler *and* any runtime `SEND_DYNAMIC`/`perform` builder (U8/U9) or they diverge (F8 was
  exactly a divergent encoder). Extract one `encode_selector(name, labels, kind)` helper.
- **Test.** New `tests/` golden `.ph`: `move(to:duration:)` and `move(_:_:)` resolve to
  distinct methods; a primitive that errors (e.g. bad arg) now surfaces the error instead of
  exit-0 (F1 regression). `verify.sh` green.
- **Must-not-preclude.** **BD-5 / open-Q3:** give `Signature`/param a *separate internal-binding
  field* so `move(to target:)` (external≠internal) can be added later **without** changing
  selector identity. Reserve the variadic flag now (U9). Keep IC slot shape generic enough for
  polymorphic/megamorphic later.

### U4 — Blocks / closures  (TA-5)
- **Goal.** First-class blocks as the shared method substrate: `Value::Block`, `ClosureObject`,
  upvalues, closure/call/jump opcodes, `=>` and trailing-block sugar, the `Function`/`Block`/
  `Method` tower.
- **Spec/ADR.** blocks.md §1–7; ADR-0006; **new ADR-0011**.
- **Write-set.** `phalcom-ast/src/ast.rs` (block node), `parser.lalrpop` (`=>`, braced/unbraced
  forms, trailing-block), `phalcom-core/src/closure.rs`, `callable.rs`, `bytecode.rs`
  (`MakeClosure`/`Call`/`GetUpvalue`/`SetUpvalue`/jumps), `frame.rs`, `vm.rs`, `compiler/*`,
  `value.rs` (Block arm), `universe.rs`/`class.rs` (`Function`/`Block`/`Method` classes).
- **Depends on.** U3; **BD-3** (upvalue model).
- **Design.** One `ClosureObject` for blocks and methods (ADR-0006 siblings under `Function`).
  Lua-style open/closed upvalues (BD-3 rec.). Unbraced `n => e` is single-param expression-only
  (blocks §2–3); braced `{ a, b => … }` multi-param. Trailing block = final argument, selector
  unchanged (blocks §4). Non-local return **mechanism** deferred to U10 but the frame-token slot
  is allocated here.
- **Risk.** Upvalue lifetime vs the U1 heap model — escaping blocks (blocks §5) outlive their
  frame; open→closed upvalue promotion must interact correctly with handle/`Rc` ownership.
  Trailing-block grammar ambiguity with map/set literals (`{ … }`) — coordinate with U-LEX.
- **Test.** Golden `.ph`: `[1,2,3].map { n => n*2 }`, `5.times { … }`, `blk.call(…)`, `blk(…)`
  sugar, `blk.arity`. Snapshot the closure/jump disassembly.
- **Must-not-preclude.** Don't let unbraced `=>` ever carry a `return`/statement body (blocks §2)
  — that's what makes non-local return safe by construction; U10 depends on it. Keep one closure
  repr so `Fiber`/`Future` (concurrency.md) can take any `Function`.

### U5 — Control-flow-as-message + inliner  (⊇ operators→sends)
- **Goal.** `if`/`while`/`for`/`and`/`or`/operators desugar to sends; sacred-selector inliner
  emits guarded jumps.
- **Spec/ADR.** control-flow.md §1–3; object-model Invariant (send is the only primitive).
- **Write-set.** `phalcom-core/src/compiler/*`, `bytecode.rs` (remove hardwired
  `Add/And/Or/Negate/Greater/…` opcodes; keep jumps), `vm.rs`, `primitive/number.rs`,
  `primitive/boolean.rs`.
- **Depends on.** U4 (laziness needs blocks; inliner needs literal-block call sites).
- **Design.** `a+b` → send `+(_:)`; `a and b` → `a.and { b }`. Compiler inlines the sacred set
  (`ifTrue(_:)`, `ifFalse(_:)`, `ifTrue(_:)ifFalse(_:)`, `and(_:)`, `or(_:)`, `whileTrue(_:)`,
  `repeat(_:)`) to jumps on literal-block sites, guarded by a receiver-type check that **deopts
  to a real send**. `and`/`or`/operators become ordinary overridable `Bool`/`Number` methods.
- **Risk.** Deopt guard correctness — an inlined `ifTrue` on a non-`Bool` must fall back, not
  miscompile. This must land early (control-flow §3, Invariant 5): if blocks/branches are slow
  the whole spec unravels. Interacts with BD-2/ADR-0005 (flat Number) for arithmetic primitives.
- **Test.** Golden `.ph`: `if`/`while`/`for` produce identical output to their desugared send
  forms; an overridden `Bool>>and` is honored on the non-inlined path; snapshot the jump
  bytecode to prove zero closure alloc on the hot path.
- **Must-not-preclude.** Don't hardwire two-operand numeric assumptions that block the open-Q2
  Int/Float tower (ADR-0005 keeps flat *for now* but the split is reserved).

### U6 — Absence → Option  (TA-7)
- **Goal.** Remove surface `nil`; introduce `Option`/`Some`/`None`; wire `??`/`?.`; forbid
  truthiness.
- **Spec/ADR.** values-and-absence.md §1–6; ADR-0007.
- **Write-set.** `phalcom-core/src/nil.rs` → `option.rs`, `value.rs` (keep private sentinel,
  no surface literal), `universe.rs` (`Option`/`Some`/`None` classes), `primitive/*`,
  `core/core.ph` (combinators), `compiler/*` (`?.`/`??` desugar, remove `Expr::Nil`/`Nil`
  literal, `if (opt)` compile error), `phalcom-ast` (`?.`/`??` tokens, drop `nil` keyword).
- **Depends on.** U4 (combinators/`ifSome` take blocks; `ifTrue` returns `Option`).
- **Design.** ADR-0007: abstract `Option` + `Some`(`_value`) + singleton `None`; per-subclass
  combinators (dispatch replaces tag tests). `a ?? b ≡ a.orElse { b }`, `opt?.foo ≡
  opt.map { x => x.foo }`. `if (opt)` → compile error (no truthiness, §3.5). Private
  `Value::Nil` must never enter a `Some`.
- **Risk.** Every place today that produces surface `nil` (`Return` default popped `Value::Nil`
  at `vm.rs`, uninitialized reads) must be audited so the sentinel cannot leak to user code
  (Invariant 4). Coordinate with U7 (unassigned field reads `None`).
- **Test.** Golden `.ph`: `Some(42).map { … }`, `None.unwrapOr(0)`, `a ?? b`, `opt?.foo` chain
  short-circuit; a `.ph` using `if (opt)` must be a **compile error** (negative golden).
- **Must-not-preclude.** Reserve the `Result` bridge vocabulary (values-and-absence §3.7,
  open-Q9): `map`/`flatMap`/`unwrapOr` names must carry identically to a future `Result`.

### U7 — Static fields + `construct`  (TA-4 full)
- **Goal.** Static per-class slot layout with implicit field declaration, read-before-write
  compile error, and `construct` on the metaclass.
- **Spec/ADR.** classes.md §1–2; object-model §5; **ADR-0010**.
- **Write-set.** `phalcom-core/src/instance.rs`, `class.rs` (per-class field table),
  `compiler/*` (field collection + slot assignment + read-before-write check + `construct`
  lowering), `phalcom-ast/src/ast.rs` + `parser.lalrpop` (`construct` keyword/node, `_field`
  token per lexical §3), `primitive/*`.
- **Depends on.** U2 (metaclass for `construct`), U3 (selector for `construct new(name:)`),
  U6 (unassigned field reads `None`).
- **Design.** Collect fields assigned anywhere in the class body → fix slot offsets at
  class-definition time; `GetField/SetField(slot)` index the `Box<[Value]>`. Read-before-write
  (a field never assigned in *any* method) is a **compile error** (classes §2). Fields private +
  non-inherited (subclass writing `_name` gets its own slot). `construct` = alloc + body with
  implicit `self` + implicit `return self`; declared on the metaclass; no user-visible allocator.
- **Risk.** Slot-offset stability vs open-Q4 (runtime hierarchy mutability): the spec makes
  fields non-inherited precisely to keep offsets static — don't design a layout that a later
  mutability decision forces to renumber. Read-before-write analysis must be whole-class, not
  per-method-local.
- **Test.** Golden `.ph`: `Person` with `construct new(name:age:)`, getters/setters, unassigned
  field reads `None`; a negative golden where `_naem` (typo) is a **compile error**.
- **Must-not-preclude.** Keep the field table shape open to open-Q7 destructuring binds and to a
  future inheritance-visible-accessor convention.

### U8 — `doesNotUnderstand` / `Message` / `perform` / `SEND_DYNAMIC`
- **Goal.** Failed sends reify a `Message` and re-dispatch through `doesNotUnderstand(_:)`;
  `perform` and spread share one dynamic-send primitive.
- **Spec/ADR.** method-lookup §2–3; messages-and-selectors §5; object-model §4/§8.
- **Write-set.** `phalcom-core/src/vm.rs` (miss path + `SEND_DYNAMIC`), `value.rs`/new
  `message.rs` (`Message`), `universe.rs` (`Message`, `Error`, `MessageNotUnderstood`),
  `primitive/object.rs` (`perform`, `respondsTo`, default `doesNotUnderstand`).
- **Depends on.** U3 (selector encode helper), U4 (Message `args` as values/list).
- **Design.** On lookup exhaustion, build a `Message { selector, name, labels, args }`, cache
  the resolved handler per receiver class, re-send `doesNotUnderstand(_:)`. Default raises
  `MessageNotUnderstood`. `SEND_DYNAMIC` builds the selector at runtime from materialized arg
  count/labels — one primitive for spread, `perform`, and dNU forwarding.
- **Risk.** Uses the *same* `encode_selector` as U3 or proxies silently misroute. Error raising
  couples to open-Q9 — keep the raise mechanism, defer `try/catch` surface.
- **Test.** Golden `.ph`: a `Proxy` forwarding via `doesNotUnderstand`; `obj.perform(:foo, [])`;
  `respondsTo(:bar)`. Assert `MessageNotUnderstood` is raised (not a Rust panic).
- **Must-not-preclude.** `Message` shape and `Error` hierarchy must not foreclose open-Q9
  (`throw/try/catch` vs `Result`) — model `Error`+`raise` now, leave the surface open.

### U9 — Variadics + spread
- **Goal.** Rest parameters, the variadic table, and spread call sites.
- **Spec/ADR.** messages-and-selectors §4–5.
- **Write-set.** `phalcom-ast/src/ast.rs`/`parser.lalrpop` (`*p` rest, `*args` spread),
  `phalcom-core/src/compiler/*`, `vm.rs` (variadic-table probe), `method.rs`/`signature.rs`
  (variadic flag from U3).
- **Depends on.** U3 (Signature variadic flag), U8 (`SEND_DYNAMIC`).
- **Design.** Variadic interns as `sum(_...)`; collect trailing positionals into a `List`; must
  be last; positional-only. Lookup: exact probe → variadic table `(name, min_arity)` → dNU.
  Spread emits `SEND_DYNAMIC`. Table built once at class-definition; warm sites never hit step 2.
- **Risk.** Interaction with labels (a labelled param cannot be variadic) must be a
  compile-time reject, not silent.
- **Test.** Golden `.ph`: `sum(1,2,3)`, `f(*args)`, `[1,*rest]`. Assert the variadic-table
  fallback resolves.
- **Must-not-preclude.** No `**kwargs` (labels are selector identity) — don't add a kwargs table.

### U10 — Non-local return + `DeadFrameError`
- **Goal.** `return` inside a block unwinds to its home method frame; escaped blocks raise
  `DeadFrameError`.
- **Spec/ADR.** blocks.md §5; object-model §4 (`DeadFrameError`).
- **Write-set.** `phalcom-core/src/frame.rs`, `closure.rs` (frame token), `vm.rs` (unwind +
  generation check), `universe.rs` (`DeadFrameError`).
- **Depends on.** U4 (frame-token slot allocated there).
- **Design.** Frame token = frame ptr + generation counter (blocks §5). On non-local return,
  compare token to live frame; mismatch → raise `DeadFrameError` (cheap integer compare turns a
  memory-safety hazard into a clean error).
- **Risk.** Shares `vm.rs` with U8 — **sequence after U8** or keep the unwind path surgically
  separate. Generation counter must advance on every frame reuse or a stale token aliases a live
  frame.
- **Test.** Golden `.ph`: `findNegative` early-exits via block `return`; a stored escaped block
  invoked after its home returns raises `DeadFrameError` (not UB/panic).
- **Must-not-preclude.** Keep the unwind compatible with `Fiber` boundaries (concurrency.md) —
  a non-local return must not cross a fiber.

### U-LEX — Tier-B surface features
- **Goal.** String interpolation, numeric separators, newline-suppression state machine,
  collection literals (tuple/list/map/set) + brace disambiguation.
- **Spec/ADR.** lexical-structure §1/§4/§5/§6; open-Q5/Q6 (interp/set-literal syntax).
- **Write-set.** `phalcom-ast/src/lexer.rs`, `token.rs`, `parser.lalrpop`, `ast.rs` (+ small
  `compiler/*` for literal lowering). Disjoint from all Rust VM units except the shared
  `parser.lalrpop`/`ast.rs` — **sequence after U4/U7** (which also edit the grammar).
- **Depends on.** U0 (grammar stable); U4 (brace disambiguation vs trailing block).
- **Design.** `"{name}"` interpolation (open-Q5 assumes `{}`), `1_000_000` separators,
  `{ }` map/set vs block disambiguation, `(a,b)`/`[…]`/`Set(…)`/`a..b`.
- **Risk.** `{ }` ambiguity (block vs map/set) is the crux — must agree with U4's trailing-block
  grammar. BLOCKED-adjacent: open-Q5/Q6 pick surface syntax.
- **Test.** Lexer/parser insta snapshots; golden `.ph` printing interpolated strings.
- **Must-not-preclude.** Leave open-Q5 (`${}`/`\()`)and open-Q6 (`#{}` set) reversible — don't
  bake one interpolation delimiter irreversibly.

### U-STD — `core.ph` standard library authoring
- **Goal.** Author the Phalcom-side core methods (Option combinators, Bool logic, Number,
  collection protocols) once their host classes exist.
- **Spec/ADR.** values-and-absence §3.3; classes.md; object-model §4.
- **Write-set.** `phalcom-core/core/core.ph` **only** — fully disjoint from every Rust unit,
  the one reliably parallel lane.
- **Depends on.** the class it targets (Option→U6, Bool→U5, etc.).
- **Test.** Golden `.ph` exercising each combinator.
- **Must-not-preclude.** Keep `Result`-bridge names reserved (open-Q9).

### U11 — Refinement: `Bool` as `True`/`False`
- **Goal.** Split `Bool` into abstract `Bool` + singleton `True`/`False` so boolean control flow
  is pure dispatch.
- **Spec/ADR.** ADR-0004.
- **Write-set.** `phalcom-core/src/boolean.rs`, `universe.rs`, `value.rs` (class selection from
  the `bool` payload — no new variant), `core/core.ph`.
- **Depends on.** U5 (inliner path must deopt to the right subclass method).
- **Design.** `Value::Bool(true).class == True`; per-subclass `and`/`or`/`ifTrue`. No new
  `Value` variant (ADR-0004).
- **Risk.** Must interact cleanly with U5's inliner deopt guard.
- **Test.** `verify_invariants` extended; golden dispatch on `True`/`False`.
- **Must-not-preclude.** Keep surface as one `Bool` class (users don't see `True`/`False` as a
  breaking split).

---

## Deferred register (off critical path — correctness first)
- Inline-cache population (polymorphic/megamorphic) — dispatch is built IC-*ready* in U3; the
  cache itself is a speed item.
- NaN-boxing of `Value` (TA-2 reserves the API).
- Int/Float numeric tower (open-Q2; ADR-0005 keeps flat).
- Tracing GC (TA-1 option C) — the handle heap is designed to host it later.
- F3 cleanup (dead `runtime_error` path: wire the pretty-printer or delete).
- open-Q7 destructuring, open-Q8 modules/imports, open-Q9 `try/catch` surface, open-Q10
  traits/mixins, concurrency.md `Fiber`/`Future`.

---

## BLOCKED-ON-DECISION (needs the user before the gated unit can start)

- **BD-1 — Object-graph / heap ownership model.** *Gates U1 and the entire VM spine.* The
  current `Rc<RefCell>` + inert `MaybeWeak` (F5) leaks the kernel and keeps a `RefCell`
  borrow-panic surface. Options: **(A)** `Rc<RefCell<T>>` + accept an intentional
  process-lifetime kernel cycle, `Weak` only where needed — simplest, matches substrate, still
  leaks user cycles + panic risk; **(B)** *handle/arena heap* with `Copy` `ObjRef`/`ClassId`
  and a central `Heap` — no Rc-cycles, no borrow-panic, IC- and cache-friendly, GC-ready;
  **(C)** tracing GC (`Gc<T>`) — most Smalltalk-faithful (cycles, `System.gc`, mutable
  `superclass=`), heaviest lift. **Recommendation: B now, designed to host C later.** Record as
  **ADR-0008**.
- **BD-2 — `Object>>toString` default for a *plain instance*.** *Gates U2.* Spec pins `name`
  (Behavior-side, own name) but leaves the instance display string open ("display
  representation"). Options: **(A)** `"a {ClassName}"` / `"an …"` (Smalltalk `printString`;
  needs vowel logic); **(B)** `"{ClassName} instance"` (deterministic, golden-stable —
  **recommended**); **(C)** `"<{ClassName}>"`. In all cases a *class*'s `toString` is its own
  name. `printString` is **not** a spec selector — do not introduce it.
- **BD-3 — Closure upvalue capture model.** *Gates U4.* Options: **(A)** Lua-style open/closed
  upvalues — best-practice, supports escaping blocks (blocks §5) with shared-mutable capture
  (**recommended**); **(B)** by-value capture snapshots — simpler but breaks shared mutation and
  fights non-local return. Record as **ADR-0011**.
- **BD-4 — `let` vs `var` (open-Q1).** *Gates U6/bindings.* Recommendation (adopt the spec's own
  proposal): `let` = immutable, `var` = mutable, `var x` with no initializer = `None`. Needs
  ratification; the lexer currently has only `let`.

### Design-guard (not hard blockers, but must be honored)
- **open-Q3 external/internal param names** — U3 must give `Signature` a separate
  internal-binding field so `move(to target:)` can be added later without changing selector
  identity. Recommend: bind label directly *now*, keep the field reserved.
- **open-Q5/Q6 interpolation / set-literal syntax** — U-LEX must keep the delimiter choices
  reversible.
- **open-Q9 error handling** — U8/U10 model `Error` + `raise` now; leave `throw/try/catch` vs
  `Result` surface open.

### New ADRs to draft (documentation-and-adrs skill)
ADR-0008 (heap ownership, BD-1) · ADR-0009 (Value representation) · ADR-0010 (static slot
layout) · ADR-0011 (closure/upvalue + frame-token, BD-3) · ADR-0012 (selector/Signature
encoding + IC-ready dispatch).
