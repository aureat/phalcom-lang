# Handoff — Core library: U-CORE-0 spec → implementation specifications

A self-contained prompt for a fresh agent/session to carry the core-library work
from the current state (U-CORE-0 landed & re-baselined) through to a
dispatch-ready implementation specification per U-CORE-N unit. Paste the block
below verbatim.

---

```
HANDOFF — Phalcom core library: from U-CORE-0 spec → implementation specifications

ROLE
Continue the core-library specification for Phalcom (Smalltalk-style bytecode VM in
Rust). You are picking up mid-stream. Work is spec/docs, grounded in real code.

ORIENT FIRST (graphify-first, then read source)
- Read: /CLAUDE.md, docs/spec/core/{README,floor-census,bootstrap-phases,catalog-delta}.md,
  docs/spec/object-model.md §4-6, docs/forge/{STATE,PHASE2-INDEX}.md.
- graphify query/explain BEFORE reading raw files (repo hook enforces this).
- Ground every claim in phalcom-core/src/universe.rs (install_primitives / create_core_classes)
  and vm.rs (VM::new / install_core) — NOT the graph. The graph orients; source is truth.

CURRENT STATE
- U-CORE-0 deliverables #1–#4 landed & committed: floor census, bootstrap phases,
  sacred-selector set (folded into census §5), catalog delta.
- Re-baselined to HEAD through U9 (code commit c9805d0): 73 floor bindings, 57 distinct
  native fns, 19 named kernel classes. Baseline-pin policy is in README — keep it current.
- U-CORE-2 Bool + core Option combinators LANDED (commit 0da64d6): half-Option closed,
  catalog-delta §4.2 resolved, ifNone/orElse/isSome/isNone added in .ph. Remaining Option
  transform combinators (ifSome/map/unwrapOr) are deferred to U-STD per catalog-delta §2.2.
- SCOPE CHANGE: U8 already shipped perform/respondsTo/doesNotUnderstand/Message, so U-CORE-1
  shrank to: Object#hash, Object#isA(_), and Behavior/Class reflection (name, method-dict).

REMAINING STEPS (in order)

A. Finish U-CORE-0 (three docs under docs/spec/core/):
   #5 pending-retirement.md — map each tests/lang.rs `_pending()` twin → the protocol +
      owning unit that flips it. NOTE: U8/U9 already retired several (dispatch/messages/
      variadics) — verify against current tests, don't assume the pre-U8 list.
   #6 invariant-requirements.md — per-unit assertions to add to verify_invariants /
      tests/invariants.rs. Must include: floor-census audit test (assert 73 bindings),
      parallel-rule for ALL ordinary rows (today only Number), absence-non-surfacing at boot,
      Some + Message fixed-slot layout checks.
   #7 forward-compat.md — "must not preclude" checklist vs Fiber/Future (concurrency.md),
      Error mechanism (ADR-0008), modules/imports, integer/float split (open-questions.md).
   Re-verify against HEAD before writing each (drift policy).

B. Close the gating decisions (route each to a short ADR or a ruling):
   Q1 is Object#hash a floor primitive? (ADR-0019 amendment if yes) — blocks Map/Set.
   Q2 error mechanism — largely pre-decided by ADR-0008; confirm, don't redesign.
   Q4 prelude/global model; Q5 collection mutability+equality.
   §4.1 Method superclass: catalog says <Function, code says <Object — re-parent or amend catalog.
   §4.4 per-type toString (Number/String/Symbol/Bool/Option) — U-CORE-4.

C. Author implementation specifications — one dispatch-ready plan per unit, via the
   phalcom-architect agent (forge Phase 2), in dependency order:
     U-CORE-1 kernel reflection (hash, isA, Behavior/Class)
     U-CORE-2 absence + Boolean  (Bool + core Option combinators landed 0da64d6; only U-STD transforms remain)
     U-CORE-3 callables/Block    (HARD PREREQ: must precede any iteration method below)
     U-CORE-4 value classes      (incl. per-type toString overrides)
     U-CORE-5 collection protocol contract (the shared interface; NOT new collection classes — ADR-0020)
     U-CORE-6 Error root + wire dNU → MessageNotUnderstood (per ADR-0008)
   Each plan must cite spec §/ADR, state the native-vs-.ph split, name the `_pending` tests it
   flips (from #5), the invariants it adds (from #6), and pass a "must not preclude" check (#7).

NON-NEGOTIABLES (do not regress)
- ADR-0019 floor is FROZEN: a new native primitive requires an ADR-0019 amendment, not a commit.
- No truthiness (ADR-0021); absence is Option/Some/None, surface `nil` is unreachable (Invariant 4).
- Doc selector notation = human `_` form (`+(_)`, `match(some, none)`); the interned heap form is
  `_:` (`+(_:)`, `match(some:none:)`). Don't "correct" the docs to the heap form.
- Sacred selectors are compiler-coupled (ADR-0018): inlined fast path ≡ primitive deopt path.
- Commit per green checkpoint; scope commits to docs/spec/core/ ONLY — the tree has concurrent
  forge sessions; never sweep others' uncommitted changes (STATE.md, .agents/, etc.).
- The tree moves under you (U8/U9 landed mid-authoring). Re-baseline + bump the README pin
  before each unit.

DONE WHEN
U-CORE-0 is 7/7, Q1/Q2/Q4/Q5 + the two catalog divergences are ruled, and there is a
dispatch-ready implementation spec for each of U-CORE-1…6 that a phalcom-implementer can execute.
```
