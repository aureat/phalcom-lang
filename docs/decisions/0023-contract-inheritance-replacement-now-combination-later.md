# PDR-0023 — Contract inheritance: lexical replacement now (documented, Liskov-unsound), runtime combination later behind the metadata gate

- Status: Proposed
- Date: 2026-07-20
- Related: [`contracts.md`](../spec/decorator/contracts.md) (flagged the gap),
  [`requires.md`](../spec/v0.2/decorators/requires.md) /
  [`ensures.md`](../spec/v0.2/decorators/ensures.md) /
  [`invariant.md`](../spec/v0.2/decorators/invariant.md) (as-built weaves),
  DEF-11 (contract metadata never emitted — the gate),
  [PDR-0024](0024-metaobject-gate-floor-amendment.md) (the runtime-check
  route's other prerequisite),
  [ADR-0052](../adr/accepted/0052-invariant-reentrancy-scope-and-layout-confined-decorator-state.md).

## Context

Contracts are **lexically woven** into the bodies of the class that declares
them, at that class's compile. Two consequences nobody ruled:

1. **Overrides replace.** An overriding method's own `@requires`/`@ensures`
   apply; the parent's are never consulted. Under Liskov, an override may
   only *weaken* preconditions and *strengthen* postconditions; free
   replacement lets a subclass strengthen a precondition, so a polymorphic
   call site that satisfied the parent's contract can raise
   `PreconditionError` through an override. Unsound, silent.
2. **Subclass invariants do not guard inherited methods.** Dispatch walks to
   the parent's dictionary; the parent's woven body runs against the
   subclass receiver, carrying only the parent's invariant. A subclass
   `@invariant` is checked only in members the subclass itself declares.

Both follow from the architecture, not oversight: bodies compile once, in
one class, and classes are closed (PDR-0001) — there is no re-weaving a
parent's compiled method per subclass, and no post-hoc weave into an
already-defined class. Eiffel's `require else`/`ensure then` combination is
a *runtime* discipline: it needs the parent's predicates available as values
at check time. Phalcom's predicates currently exist only as woven bytecode —
DEF-11 (metadata emission) is precisely the missing reification.

## Decision

1. **v0.2 semantics: replacement, made explicit.** An override's contracts
   fully replace the parent's; a subclass `@invariant` guards only
   subclass-declared members. Both facts move from "consequence you discover"
   to normative text in contracts.md and the two as-built files, each with
   the one-line warning: *contracts on polymorphic surfaces are
   per-declaration, not per-hierarchy; an override may observably strengthen
   a precondition.*
2. **No enforcement theater.** Rejected: forbidding contracts on overrides
   (punishes the legitimate case — an override documenting its own, correct
   contract), and warn-on-override-contract (no warning tier exists; a
   hard error would be worse than the disease).
3. **The combination design is named now, built later.** When DEF-11's
   metadata (predicates retained on `MethodObject::contracts`) and
   [PDR-0024](0024-metaobject-gate-floor-amendment.md)'s reflective surface
   both exist, contract checking can move from pure lexical weave to a
   hierarchy-aware check: effective precondition = declaration-chain
   disjunction (`require else`), effective postcondition/invariant =
   conjunction (`ensure then`), computed per selector at class-definition
   time (chains are frozen — PDR-0001 — so the combined predicate list is
   stable and cacheable; no per-send chain walk). That is Eiffel's rule
   arriving on Phalcom's own terms: definition-time composition, runtime
   evaluation, zero cost for uncontracted hierarchies.
4. **Trigger, not schedule.** Ruling 3 is design direction, not a scheduled
   unit; it activates when DEF-11 lands, as its own record with the
   measured cost of the combined checks.

## Consequences

- v0.2 contracts remain what they are — single-class runtime checks with a
  purity floor — and now say so where a polymorphism user will read it.
- The one behavior this record *changes* is documentation truth (DEF-class
  fix), not semantics: no weave changes, no new errors, nothing to migrate.
- Racket's boundary-contract precedent is recorded as the alternative
  considered for the combination era: contracts attached at dispatch
  boundaries rather than declarations sidestep inheritance entirely, but
  they contract *call sites*, a different feature than DbC on methods; noted
  so the future design weighs it once, deliberately.

## What this precludes

- Advertising v0.2 contracts as Liskov-respecting, in any doc or example.
- A lexical fix (re-weaving parent bodies per subclass, or weaving subclass
  invariants into inherited methods) — forbidden by closed classes and
  single compilation; the combination route is the only sanctioned path.
