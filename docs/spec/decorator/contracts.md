# Contract decorators — `@requires`, `@ensures`, `@invariant`

- Status: **Built (U-ANNOT-CONTRACTS)** with two live defects (DEF-1, DEF-2)
  and one unbuilt substrate (DEF-11). As-built truth:
  [requires.md](../v0.2/decorators/requires.md),
  [ensures.md](../v0.2/decorators/ensures.md),
  [invariant.md](../v0.2/decorators/invariant.md); semantics base:
  annotations-contract-semantics.md; re-entrancy ruling: ADR-0052.
- Tier: Compile / weave. Eiffel nesting `invariant → post → pre` produced by
  the fixed phase order (invariant weave runs last, over woven bodies).

## Verified design summary

| | `@requires` | `@ensures` | `@invariant` |
|---|---|---|---|
| Target | method/getter/setter | method/getter/setter | class |
| Check shape | `pred.ifFalse { PreconditionError.new(msg).raise() }` prologue | exit-path checks + `old()` hoist + return rewrite | entry+exit on every public member; exit-only on constructors |
| Debug | woven | woven | woven |
| Release | **woven** (Meyer's demand-side default) | stripped | stripped |
| Unchecked | stripped | stripped | stripped |
| Purity floor | `contract.impure_predicate`, validated in **every** mode | same, plus `contract.old_in_precondition` / `contract.old_on_mutable` | same |

The purity check running even when the guard strips is the right call and
worth naming: a predicate that only compiles in the mode that never runs it
would rot invisibly. This mirrors the ADR-0021 truthiness posture — a
syntactic floor, honestly incomplete, never presented as proof.

Philosophy check passes: checks lower to ordinary sends on `Bool`
(no truthiness — the predicate must *be* a `Bool` or `GuardBool` raises),
errors are ordinary `Error` subclasses (ADR-0008), stripping is
member-granular and bytecode-identical (the erasure rule), and the whole
family is `runtime: false`.

## Defects and fix plans

### Plan §1 — the `result` binding (DEF-1)

`@ensures(result > 0)` is the documented surface; the weave binds `__result`
and never puts either name in the predicate's scope — the documented form
cannot evaluate. Fix in `EnsuresExpander`'s check synthesis: substitute
occurrences of the identifier `result` in the predicate AST with the internal
`__result` var at weave time (pure AST substitution, same machinery as the
`old()` rewrite; no scoping change, no new binding visible to user code).
Reject a predicate that *assigns* `result` via the existing purity floor.
Fixture: `contracts_postcondition_result.ph` positive + a negative proving
`result` outside `@ensures` still resolves normally (it must remain an
ordinary identifier everywhere else — this is a predicate-scoped substitution,
not a reserved word).

### Plan §2 — the missing re-entrancy guard (DEF-2)

ADR-0052 Fix 1 is ratified: per-fiber receiver-identity set (`checking:
Set<ObjRef>`), ownership captured before insert, `ensure`-based unwind-safe
release, state saved/restored on fiber switch. The weave on HEAD emits none of
it — a self-send inside a guarded method re-checks the invariant mid-mutation
(false positives on any temporarily-broken invariant, the exact bug class
invariants exist to allow within a method body). Plan:

1. Fiber state: add the `checking` set to `FiberObject` alongside
   `stack`/`frames`/`open_upvalues`, mirrored by a `VM` pointer on switch —
   the ADR's stated shape. This is VM plumbing, not floor surface (no new
   `.ph`-callable primitive).
2. Weave: prologue captures `__invariant_owner`, inserts, entry-checks iff
   owner; epilogue via the same `Block#ensure(_)` lowering the exit check
   already uses, exit-checking and removing iff owner.
3. Regressions from the ADR's own test list:
   `contracts_invariant_cross_receiver.ph` (B still checked inside A's call),
   `contracts_invariant_survives_throw.ph` (no permanently-inflated guard),
   and the pinned `contracts_invariant_fiber_yield.ph` limitation
   (yield under the woven `ensure` raises `CannotYieldAcrossNativeFrame` —
   **accepted**, not to be "fixed" en passant; both alternatives were
   evaluated and rejected in the ADR's amendment).
4. Until this lands, invariant.md's status line should say "guard absent" in
   its first paragraph, not only in a doc-comment citation — a ratified-ADR
   citation on unimplemented code is how the tree grows provenance lies.

### Plan §3 — contract metadata emission (DEF-11)

`strip_metadata` plumbing exists; nothing emits. Needed by the Phaldoc
contract view (metadata-and-docs.md §"contract view") and `Method>>contracts`
reflection. Emit at weave time into `MethodObject::contracts` (source-printed
predicate + kind + mode-availability), retained in Debug/Release, stripped in
Unchecked by default. Gate: this is reflection surface — coordinate with the
same object-model §8 gate that holds `Method` reflection generally (D-2), do
not ship a one-off.

## Evaluated extensions

- **`@pure`** — a declared-purity marker the contract validator could trust
  (and the memoizer could require). **Parked.** Without an effect system it is
  an unverifiable claim that *weakens* the existing floor (the syntactic
  checker would be obliged to trust it past what it can see). Revisit only if
  a real analysis lands; precedent: D's `pure` works because the compiler
  checks it — an unchecked `pure` is documentation cosplaying as semantics.
- **`old()` beyond `@ensures`** — no. `contract.old_in_precondition` already
  rejects it; the operator is meaningful only against a pre-state snapshot,
  which only the postcondition weave establishes.
- **Contract inheritance (Eiffel's weaken-pre/strengthen-post)** — not
  designed anywhere in the corpus. Recorded as the family's one genuine open
  question: what happens when an overriding method's class carries its own
  `@requires`? Today: the override's own contracts apply; the parent's are
  not consulted (each body is woven independently). That is *covariant
  contract replacement* — unsound by Liskov, allowed by construction.
  Needs a ruling before contracts are advertised for polymorphic use;
  options (inherit-and-OR preconditions per Eiffel, forbid contracts on
  overrides, document-and-accept) belong to a future PDR. Flagged here so the
  gap is owned, not discovered.

## Hazards

- The `Fiber.yield`-under-`ensure` limitation (ADR-0052 Bug 3) will be
  re-reported as a bug forever; the negative fixture and this paragraph are
  the defense. It is a consequence of the guard being *unwind-safe*, which is
  the property that matters more.
- Contract checks are ordinary sends — a predicate calling the method it
  guards recurses. The re-entrancy guard bounds invariant recursion only;
  `@requires(self.valid)` where `valid` is itself guarded is user-visible
  infinite recursion and stays their error (floor, not proof).

## What this precludes

- Contracts as a static verification story. These are runtime checks with a
  syntactic purity floor; nothing here builds toward SMT-backed verification,
  and no spec should imply it.
- A `Contract` module (`Contract.require(...)`) — the woven-send shape is the
  committed one; a parallel functional surface would be a second mechanism.
