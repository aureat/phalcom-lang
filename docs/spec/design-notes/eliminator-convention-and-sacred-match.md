# Eliminator convention + sacred-`match` extension (2026-07-14)

**Status: FINDINGS + PROPOSAL.** Grounded via `language-design` skill (pattern-matching,
closures-control, vm axes) against the live tree at this commit. Two doc bugs found and
noted below; the ADR-writing and inliner-amendment work itself is not yet done — this
file is the spec for that work, not a record of it landing.

Origin: user question — "conditionals/switch nest ugly, how does Phalcom avoid it by
design/convention." Answer is not a syntax feature. It is a doctrine plus one perf gap.

---

## 1. The four causes of nested conditionals — and Phalcom's cure per cause

Nesting isn't one problem. Each cause has a different fix, and Phalcom already commits
to three of the four:

| Root cause | Cure | Phalcom status |
|---|---|---|
| Branch on type/tag | Dispatch — add a class | Committed: ADR-0007 (`Option`), ADR-0004 (`Bool` tower), `doesNotUnderstand` |
| Branch on a closed sum | **Eliminator method** `match(a:, b:, …)` | Exists in practice (`Option`, `Result`); not yet written down as a general convention |
| Sequential validation / arrow code | Guard clause + non-local `return` from a block | Committed: ADR-0013 |
| Unrelated predicates chained | `else if` keyword sugar | Parser already supports it: `parser.rs:2211` |

The gap is not missing language capability. It's an undocumented convention (row 2) and
one under-extended optimization (§3).

## 2. Why no `match`/`switch` *syntax* should be added

This was evaluated and rejected, not just "not requested":

- ADR-0011 makes instance fields **private and non-inherited**. So the payload-destructure
  half of ML/Rust-style structural matching (`Point(x, 0)`) is already precluded by the
  object model — there is no way to read another object's slots except through its
  accessor protocol. Any `match` syntax could only desugar to protocol sends, which is
  strictly worse than calling the eliminator directly (extra grammar, no totality gain,
  needs its own fallback-arm/`MatchError` story the eliminator's selector identity
  already gives away for free).
- Hazard: **adding `match` to a message language ⊗ open classes**
  (`pattern-matching.md:122`) — a real `match` in a language with runtime class addition
  gets no exhaustiveness guarantee and degrades to sugar over `isKindOf:` chains. Squeak
  shipped `caseOf:` as exactly this; Pharo dropped it. Buys ergonomics, not the safety
  that motivates `match` in ML-family languages.
- Hazard: **view/active pattern ⊗ exhaustiveness** (`pattern-matching.md:29`) — the only
  `match` shape ADR-0011 leaves open (a call into a protocol method) is precisely the
  shape a checker cannot see through.

Conclusion: no new grammar. The eliminator convention below is the switch-replacement,
and it gets exhaustiveness *for free* from a mechanism Phalcom already has — see §4.

## 3. The eliminator convention (proposed doctrine, not yet an ADR)

**Rule:** ship `match(label:, label:, …)` on an abstract root **only where the variant
set is closed and forever-fixed, and the operation set over it is open and growing.**
Every subclass implements its own arm by ordinary override; the eliminator body is
"dispatch," not "test."

This is *not* "any abstract root gets a `match`." That was my first draft and it's
wrong on the expression problem:

| | Add a variant | Add an operation |
|---|---|---|
| Plain polymorphism (subclass + override) | cheap | expensive — touch every subclass |
| Eliminator `match(a:, b:)` (Church encoding / visitor) | **expensive** — new selector breaks every call site | cheap — one method on the root, written over `match` |

`Option` (`Some`/`None`) and `Result` (`Ok`/`Err`) qualify: exactly two variants, fixed
by the type's own definition, dozens of combinators (`map`, `orElse`, `unwrapOr`, …)
layered on top. A hierarchy like `Shape`/`Node`/`Error` does **not** qualify — those grow
variants over time and should stay plain dispatch. "Nested `ifTrue` chain" is a smell for
*either* "missing eliminator" or "missing class" — the table above is how to tell which.

### Convention is not internally consistent today — pick one shape

- `Option#match(some:, none:)` — **one Rust primitive** installed on the abstract root
  (`universe/primitives.rs:184`, `primitive/nil.rs:75`); it tag-tests `Some`/`None`
  internally in Rust.
- `Result#match(ok:, err:)` — **pure `.ph`, per-subclass**: `Ok#match` and `Err#match`
  each just call their own arm (`core/core.ph:272`, `:278`). No test anywhere — dispatch
  does the work.

Both are individually sound, but an ADR needs to pick the one to document as *the*
pattern other library authors copy. Recommend documenting **Result's per-subclass
`.ph` shape** as the canonical form; `Option`'s native-primitive shape is a bootstrap-era
concession (`Option`/`Some`/`None` are VM-blessed per ADR-0007), not the exemplar.

### User decision (2026-07-14, overrides the recommendation above on scope)

**Both `Option` and `Result` are to be primitives** — not just `Option`. This changes
§5 below: the sacred-`match` inline cut is in scope for **both** hierarchies, not Option
alone. Everywhere below that said "Result stays `.ph`-only, out of scope" is superseded
by this decision. If `Result`/`Ok`/`Err` move to native/VM-blessed status, that is a
change to ADR-0008 (error handling) and interacts with `Result`'s "deliberately not
reusing `Option`'s native eliminator, so a future `Option`→`.ph` migration stays
symmetric" note at `core/core.ph:216` — that note's premise (Result stays pure `.ph`)
no longer holds and the comment needs updating when this lands.

## 4. Why the eliminator gets exhaustiveness for free, with no checker

The strong claim, previously unstated: Phalcom's dispatch-key design
(name+arity+kind selector identity, ADR-0012) gives eliminator totality **without** a
Maranget-style usefulness algorithm. `match(ok:)` and `match(ok:, err:)` are different
selectors — a caller who forgets an arm doesn't get silent fallthrough, they get a
missed method lookup and `doesNotUnderstand`. This lands close to ML's compile-time
totality with zero static/flow analysis, because arity+labels are already load-bearing
in the dispatch key (the same property that makes default args unsound per open-Q12
here pays off as free exhaustiveness).

Precedent, with cost, for why this is worth stating explicitly:
- **Ruby** `case/in` + `deconstruct` — dynamic, no exhaustiveness ever; partial matches
  are silent until an input hits the missing arm (`NoMatchingPatternError` at runtime).
- **Scala** `unapply` — exhaustiveness only over `sealed`; extractor arms are opaque to
  the checker, so `MatchError` at runtime is still possible even in "checked" code.
- **Rust/OCaml/Haskell** — fully total, but closed-world: adding a variant is a breaking
  edit to every existing match (the classic expression-problem cost).
- **Smalltalk** — nothing native; `caseOf:` was a Squeak-only compiler hack, later
  dropped. You nest, and it's ugly — this is the status quo Phalcom is avoiding.

## 5. The perf gap: sacred set doesn't cover the eliminator

The sacred-selector inliner (`compiler/inliner.rs:140`, ADR-0018 — see doc-bug note
below) recognizes exactly six shapes: `ifTrue(_)`, `ifFalse(_)`, `ifTrue(_, ifFalse:_)`,
`and(_)`, `or(_)`, `whileTrue(_)`. **`match(some:, none:)` / `match(ok:, err:)` are not
on it.** So every `Option#orElse`/`isSome`/`map`/`okOr` and every `Result#isOk`/`map`/
`mapErr` call allocates two closures (one per labeled block arg) plus pays a full
`Invoke` send — exactly the allocation-dominated cost the U-PRIM-ABI measurement
(perf-log, `arith_send` −41.5%) already showed matters most.

**Proposed cut:** extend the ADR-0018 sacred set with `match(some:, none:)` and, per the
user decision in §3, `match(ok:, err:)` too — both hierarchies, once `Result` is
VM-blessed. Mechanically this is the same recognizer shape as the existing
`IfTrueIfFalse` arm (`inliner.rs:149`): two labeled literal-block arguments, receiver
`ClassId` known at bootstrap, override-epoch deopt guard already exists and is reused
unchanged (ADR-0018's guard mechanism, not a new one).

### Rubric self-score (per skill's mandatory step-5 check)

1. **Soundness** — sound only *with* the existing override-epoch guard. Failure state
   without it: a class extends/reopens `Option`/`Result` and overrides `match`; the
   inlined path would ignore the override. Guard: receiver `ClassId` ∈ known set + epoch
   check; deopt materializes both blocks and falls back to an ordinary send. This
   mechanism is already implemented for `ifTrue`/`ifFalse`/`and`/`or`/`whileTrue` — no
   new guard machinery needed, only a new recognizer arm.
2. **Dispatch impact** — none on selector identity; recognition is purely syntactic
   (two labeled literal-block args), same shape class as `IfTrueIfFalse`.
3. **Representation impact** — removes two closure allocations per eliminator call.
   That is the entire point of the cut.
4. **Preclusion** — bakes selector-shape knowledge of `Option`/`Result` into the
   compiler, which requires both to be **VM-blessed / native**, not stdlib `.ph`
   classes. This is exactly what the user's decision in §3 grants for both hierarchies,
   so the preclusion is intentional and pre-authorized, not a hazard to flag later. Note
   for whoever writes the `Result` ADR-0008 amendment: this inliner dependency should be
   cited as a *reason* Result moves native, not discovered as a side effect afterward.
5. **Precedent** — Smalltalk's sacred set is exactly this kind of move (inline what you
   know can't be redefined out from under you); nobody else sacred-inlines a *library*
   eliminator because nobody else has one blessed at bootstrap the way Phalcom's
   `Option`/(pending) `Result` are.
6. **Spec reconciliation** — requires an ADR-0018 amendment (not a new ADR), plus the
   `Result`-native change requires an ADR-0008 amendment (error-handling model) since
   ADR-0008 currently describes `Result`/`Ok`/`Err` as pure `.ph` with zero floor
   delta (`core/core.ph:216`'s "net floor delta for this whole file: 0" comment becomes
   false the moment `Result` goes native).

## 6. Doc bugs found while grounding this (fix before/alongside the ADR work)

- **`.claude/skills/language-design/phalcom/overlay.md` lines 29 and 86** cite
  `ADR-0017 (drafting, U5)` for the sacred-selector inliner. Wrong on two counts: the
  inliner is **ADR-0018** (`docs/adr/accepted/0018-sacred-selector-inliner-and-override-guard.md`,
  Status: Accepted), and ADR-0017 is a *different*, unrelated, already-shipped ADR
  (class-side stored static fields — the overlay correctly cites ADR-0017 for that at
  its own line 75). `compiler/inliner.rs:1` already cites ADR-0018 correctly; only the
  overlay is stale. Fix: overlay lines 29/86 → `ADR-0018`, status `Accepted` not
  `drafting`.
- **`VM::sealed_classes` naming risk** (`vm/mod.rs:186`): this is a
  `HashMap<Symbol, ObjRef>` mapping class name → owning module, used only as a
  cross-compilation-unit reopen guard (`compiler/lib/class_decl.rs:334`). It is **not**
  a Scala/Kotlin-style sealed-variant-set mechanism and provides no route to a
  checker-verifiable closed hierarchy. Flagging so nobody reads the field name and
  assumes Phalcom has sealed-hierarchy exhaustiveness machinery already — it doesn't;
  §4's free exhaustiveness comes from selector identity, not from this field.

## 7. Concrete next actions (not yet done)

1. Fix the two ADR-0017→0018 citations in the overlay (mechanical).
2. Write the eliminator-convention ADR: closed-variants/open-operations rule (§3),
   naming `Result`'s per-subclass `.ph` shape as canonical, `Option`'s native shape as
   the bootstrap exception — updated per §3's user decision once `Result` also goes
   native.
3. Draft the `Result`-goes-native change as an ADR-0008 amendment, citing the sacred-
   inline dependency (§5.4) as a motivating reason, and update `core/core.ph:216`'s
   stale "net floor delta: 0" comment when it lands.
4. Amend ADR-0018 to add the `match(some:, none:)` / `match(ok:, err:)` recognizer arms
   to the sacred set, reusing the existing override-epoch guard.

None of 2–4 have been implemented yet — this file is the plan, grounded against the
live tree, not a changelog.
