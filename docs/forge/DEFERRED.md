# Deferred improvements register

Out-of-scope optimizations / DX / speed / security observations noticed while
landing a forge unit, but deliberately not implemented in that unit. Each
entry: file:line, category, one-line rationale.

_**Merged 2026-07-15.** This file absorbed the former `phase-next/DEFERRED.md`, which is deleted.
Its numbered entries (#1–#33) are carried forward **verbatim and with their numbers
intact** under [Numbered backlog](#numbered-backlog-merged-from-phase-next) —
~18 docs across `units/` cite them by number (`DEFERRED #30`, `#19`, `#9`, …) and
renumbering would silently break every one of those citations._

## Confirmed Backlog

**Read this section first.** Everything else in this file is a ledger of things noticed
in passing; entries age, and some are stale (several below are known-resolved and are
marked as such, but the file has never had a full triage pass — treat an unmarked entry
as *unverified*, not as *live*).

This section is different: each entry here has been **verified against the tree on the
date given**, with the evidence inline. An entry leaves this section only by being fixed
or by being disproved at a cited file:line — never by assumption.

### CB-1 · String interpolation bypasses `toString` overrides — **FIXED 2026-07-15**

_Verified 2026-07-15 by reading the tree. Supersedes and sharpens [#30](#numbered-backlog-merged-from-phase-next), filed 2026-07-12._

_**Fixed 2026-07-15** — option **C**, user-ruled. Reproduced live first
(`System.print(p)` → `<redacted>`, `"\(p)"` → `<Secret instance>`, same object), then:_

1. _**`Map`/`Set`/`Tuple`/`Range` gained derived `toString`s** (`22cc756`). They were the
   **only** classes with none — so they, and only they, would have regressed to
   `<Cls>` the moment the desugar started sending. Derived in `core.ph` over the existing
   floor, **not** admitted to it: the floor stays at 136 and no amendment was needed. Each
   mirrors `Value::to_string`'s native format exactly (verified both populated and empty),
   including `Range`'s counterintuitive `..` = inclusive / `...` = exclusive._
2. _**The desugar now emits `expr.toString`** — `Expr::GetProperty`, a **getter** send, not
   a zero-arg `MethodCall` (`toString` is bound `SignatureKind::Getter`; `toString()` is a
   different selector that would miss)._
3. _**ADR-0022 amended** — prose only, sigil untouched. The ADR had *pre-authorised* this
   exact revisit ("when U-CORE-4 lands a real content `toString`, the desugar target can be
   revisited"); U-CORE-4 landed in `2061795`, so the trigger had already fired. Guard:
   `tests/lang/strings/string_interp_sends_tostring.ph`._

_**This entry's own stated blocker was wrong.** It said "the signature difference is real
work, not a one-liner: `to_string(&self, vm: &VM)` is infallible; `to_display_string(&self,
vm: &mut VM) -> PhResult<String>` needs `&mut VM` and can raise." But `string_class_new`
**already** had `&mut VM` and **already** returned `PhResult` — every primitive does. At
that site it was a one-liner. The framing also missed that `to_display_string` is a
*hybrid* (native for the collections, sending only for what `to_string` botches), not a
"send `toString` to everything" path. Fourth CB entry whose analysis did not survive
contact._

_Residue: **CB-6** below — the same defect one level down._

**The defect.** [ADR-0022](../adr/accepted/0022-string-interpolation-backslash-paren-sigil.md) desugars
`"\(x)"` to `String.new(x)`. `string_class_new` (`phalcom-core/src/primitive/string.rs:58`)
calls `Value::to_string` — **not** `Value::to_display_string`. Only
`to_display_string` (`phalcom-core/src/value/render.rs:78`) sends the `toString`
message; `to_string` (`render.rs:19`) hardcodes native rendering for
`Str`/`List`/`Map`/`Set`/`Tuple`/`Range`/`None`/`Some` and falls through to `to_debug`
(`render.rs:98`) for everything else — including a plain user instance, a class, a
metaclass.

**Consequence: a user's `toString` override is silently bypassed by interpolation.**
`System.print(p)` and `"\(p)"` disagree for exactly the objects
[ADR-0015](../adr/accepted/0015-object-default-tostring.md) governs. This is the
un-fixed half of U-ERR-FIX's BUG-PRINT-TOSTRING (`dd2e178`), which routed
`system_class_print` through `to_display_string` and left the interpolation path alone —
even though interpolation is by far the more common stringify site.

**Why #30's stated precondition no longer applies.** #30 deferred this on "blocked on
U-CORE-4 landing a real content `toString`". **U-CORE-4 landed** (`2061795`, per
[STATE.md](STATE.md) and `docs/adr/STATUS.md`'s ADR-0036 row). The blocker is gone; the
work was never done. This is ripe, not blocked.

**Security dimension (why it is in this section and not just #30).** ADR-0015's default
`toString` (`<ClassName>`) is a redaction-safe default — a `SecretKey` renders as
`<SecretKey>`, not its contents. That property **does not hold through interpolation**,
so a class that overrides `toString` to redact is still un-redacted by `"key: \(k)"`.
Today the leak is *bounded*: `to_debug` renders an instance as `<ClassName instance>`
with no field contents, so nothing sensitive escapes yet. The risk is latent —
enriching `to_debug` to dump slots (an ordinary debug convenience someone will
eventually want) silently converts every interpolation site into a field-disclosure
bug. Fix the routing before that temptation arrives, not after.

**Fix.** Point the `\(…)` desugar at the `toString`-sending path. Note the signature
difference is real work, not a one-liner: `to_string(&self, vm: &VM) -> String` is
infallible; `to_display_string(&self, vm: &mut VM) -> PhResult<String>` sends a message,
so it needs `&mut VM` and can raise (a user `toString` that throws). Owning unit:
unassigned — touches `phalcom-ast/src/parser.rs::desugar_string_interp` (or the
`String.new` primitive itself; decide which).

### CB-2 · The floor census contradicts itself *and* the test that guards it — **FIXED 2026-07-15**

_Verified 2026-07-15 by running the invariant test and summing its constants._
_**Fixed 2026-07-15** (docs-only, no code change). `floor-census.md` §1.1 now reads **125**
bindings / **110** distinct fns, with a new **§1.3** naming
`invariants.rs::floor_census_matches_installed_bindings` as the source of record and the
standing rule "never quote a floor count from prose — including this file's". §7's
hardcoded 117 is gone. Kept below for the audit trail; see **CB-5** for the gap this fix
uncovered._

**Three numbers, no two alike.** The census is the document every ADR is explicitly told
to cite *instead of* quoting its own figure — and it is the one that is wrong:

| Source | Count |
|---|---|
| `docs/spec/v0.2/core/floor-census.md:36` §1.1 "Installed `(class, selector)` bindings" | **113** |
| `docs/spec/v0.2/core/floor-census.md:665` §7 (audit-hook prose) | **117** |
| `phalcom-core/tests/invariants.rs:631+` `floor_census_matches_installed_bindings` — **machine-checked, green** | **125** |

**The test is authoritative and it passes.** Its constants sum to exactly 125: `BASELINE`
73 + `NEW` 7 + `NEW_METHOD_REFLECTION` 5 + `NEW_VALUE_TOSTRING` 1 + `NEW_ERROR` 2 +
`NEW_MAP_SET` 14 + `NEW_TUPLE` 3 + `NEW_RANGE` 4 + `NEW_ON_ENSURE` 2 + `NEW_IMPORTS` 1 +
`NEW_FAMILY` 1 + `NEW_SCHED` 2 + `NEW_INVARIANT_GUARD` 2 + `NEW_ATTR_ROOT` 3 + `NEW_GC` 1 +
`NEW_STRING` 4 = **125**.

**The drift is precisely attributable**, which is what makes this fixable rather than
archaeology. 125 − 113 = 12 = `NEW_SCHED`(2) + `NEW_INVARIANT_GUARD`(2) +
`NEW_ATTR_ROOT`(3) + `NEW_GC`(1) + `NEW_STRING`(4) — the five amendments that landed
without updating §1.1. §7's 117 is 113 + `NEW_SCHED` + `NEW_INVARIANT_GUARD`, i.e. §7 was
brought forward two amendments and then abandoned. Also stale: §1.1's "distinct native
Rust functions = 98", and §8 points at `universe.rs`, which is now a directory.

**Why this is worse than an ordinary doc-drift row.** The overlay's *Known documentation
defects* #4 already recorded "floor census numbers don't chain" and concluded **"never
quote a floor number from an ADR — `floor-census.md` is authoritative."** That conclusion
is now itself wrong: the census is *also* not authoritative. The only authority is the
test. Until §1.1 is reconciled, the correct instruction is **"never quote a floor number
from any document — read `invariants.rs`."** Two independent agents this session were
sent to the census as the source of truth and both came back with different wrong
numbers.

**Fix.** Reconcile §1.1 and §7 to 125/N-fns from the test, and add a line to the census
naming `invariants.rs::floor_census_matches_installed_bindings` as the machine-checked
source of record — so the next drift is caught by the test rather than propagated by the
prose. Owning unit: unassigned. Cheap; do it before the next floor amendment adds a
sixteenth constant.

**Done.** All of the above, plus three findings the fix turned up:

1. **113 was not a typo — it was the terminus of an abandoned chain.** The census's
   per-amendment banners run 73 → 80 → … → 113 and *stop at U16-Open*. The five later
   amendments (`NEW_SCHED` 2, `NEW_INVARIANT_GUARD` 2, `NEW_ATTR_ROOT` 3, `NEW_GC` 1,
   `NEW_STRING` 4 = the missing 12) never got a banner. §1.1 was internally consistent
   with the chain; the chain was simply five amendments behind. §1.3's reconciliation note
   records the deltas; **the five banners themselves are still unwritten** (residue, filed
   as [#34](#34-write-the-five-missing-floor-census-amendment-banners)).
2. **§2's enumeration was near-complete** — only M-ATTR-ROOT's three (`__attributes`,
   `__attach(_)`, `__freezeAttributes()`) were absent. Added to §2.1. Note §2 is prose the
   test does **not** read: R-INV-0.1 compares its own 125-entry vec against a live VM, so a
   §2 omission is invisible to it. §2 completeness is still a manual property.
3. **§8's traceability table was 100% dead** — every row pointed at `universe.rs`, which is
   now the `universe/` directory, and `core.ph`'s List protocol had moved L53 → L779.
   Rewritten to lead with symbols; line numbers demoted to dated hints.

Distinct-fn count independently derived (126 macro lines − 2 loop lines + 10 loop
expansions + 2 hand-rolled = 136 installed; − 11 Fiber = **125**, matching the test
exactly — which is what validates the method). "Classes carrying floor primitives = 22"
and "Sacred selectors = 7" were both already correct and left alone.

### CB-3 · Sealing is one property with two representations that can disagree — **FIXED 2026-07-15**

_**Fixed 2026-07-15** — S-1 option **A**, user-ruled ("A now, B as a spike"). The `@variant`
gate now takes the **union** of the two sources: `sealed_by_attr || sealed_by_table`
(`compiler/attributes.rs`), with `VM::sealed_classes` threaded in via a new
`ExpandCtx::sealed_classes` field mirroring the existing `class_parents` borrow. Verified
before/after: the false diagnostic is gone and the **true** one takes its place —_
`attr.sealed_violation: `Foo` extends `@sealed` class `Option`, but was not declared in the same compilation unit`_._

_**CB-3's own prescription ("make the `@variant` gate consult `VM::sealed_classes`") would
have inverted the bug.** A user's own `@sealed class Shape` is **not** in that table while
its body is expanded — `class_decl.rs` inserts it only after the body compiles and the
global is defined — so a table-only gate rejects every user `@variant`. The attribute list is
the only evidence for the same-unit case; the table is the only evidence for the bootstrap
case. Neither source is complete; the union is required. Third CB entry whose stated fix
did not survive contact._

_**S-2 dissolved: the fixture it asks for cannot be written.** CB-3 called the missing
cross-unit `extends` of a **user** `@sealed` class a coverage gap. It is not — the scenario
is **unreachable**, on two independent grounds, both verified in the tree:_

1. _**Ordering.** `extends` resolves its superclass at **compile** time; `import` binds the
   module at **runtime**. Proof: give the imported lib a `System.print` side effect and it
   **never runs** — the "Unknown superclass" error fires first. An imported class cannot be
   a superclass at all, sealed or not._
2. _**Naming.** `extends S.Shape` does not parse (`extends` takes a bare identifier, not a
   member access), and ADR-0045's whole-module binding leaks no globals._

_So **`attr.sealed_violation` is dead code for user classes** — module structure already
supplies the protection `@sealed` advertises, and the check is reachable only for classes in
every unit's globals at compile time, i.e. the bootstrap-sealed kernel. **`@sealed`'s only
live effect on a user class today is gating `@variant`.** That is a real finding about the
decorator's value, not a test gap; recorded in `decorators/sealed.md` and
`drafts/sealed-classes.md` S-2._

_**Three fixtures added** (all verified to actually execute via a deliberate-corruption
mutation check — a silently-skipped fixture is indistinguishable from a passing one):
`compile-errors/annotation_variant_in_bootstrap_sealed_class.ph` (the CB-3 regression guard —
the exact case that raised the false diagnostic),
`decorators/decorators_sealed_same_unit_subclass_allowed.ph` (the positive half),
`compile-errors/decorators_sealed_cross_unit_needs_isolation.ph` (pins the unreachability,
and **must change** the day cross-module class references land). Also stale in this entry:
"`@sealed`/`@variant` are absent from `decorators-stdlib.md` and `attribute-classes.md`" —
`decorators/sealed.md` now specs both, and `attribute-classes.md` does not exist
([#34](#numbered-backlog-merged-from-phase-next))._

_**B (unification) is filed as [#35](#numbered-backlog-merged-from-phase-next)**, per the
ruling._

---

_Original entry follows._

_Verified 2026-07-15 by reading the tree. **This entry originally claimed Phalcom had "two
independent sealing mechanisms that do not know about each other." That was wrong** — an
adversarial check refuted it and the entry is rewritten to the defect that is actually
there. Recording the correction rather than quietly deleting it: the wrong version is the
kind of plausible-sounding finding this section exists to filter out._

**What is actually true: `extends` enforcement IS unified.** Both paths write and read one
table, `VM::sealed_classes: HashMap<Symbol, ObjRef>` (`phalcom-core/src/vm/mod.rs:194`,
symbol → owning module):

- the `@sealed` decorator writes it from the attribute (`compiler/lib/class_decl.rs:751-754`);
- bootstrap writes the *same* table directly for `Option`/`Some`/`None`
  (`vm/bootstrap.rs:215,220,261`), and says so in its own comment (`:209`): registered
  "directly in `self.sealed_classes` here (rather than via the `@sealed` decorator)
  because `None` has no `.ph` class reopen to carry the annotation";
- one check reads it (`class_decl.rs:364-371`), raising `attr.sealed_violation` when a
  subclass's module ≠ the sealed class's module.

So sealing is **sealed-to-the-compilation-unit**, uniformly, for kernel and user classes
alike. My "two mechanisms" framing was simply false.

**The real defect, one level down: the `@variant` gate reads a different source of truth
than the `extends` check.** `expand_class_attributes` computes
`let has_sealed = class_attrs.iter().any(|a| a.name == "sealed")`
(`compiler/attributes.rs:1540`) — from the **attribute list**, not from
`VM::sealed_classes`. `expand_variants` (`attributes.rs:1265`) then rejects `@variant`
without `@sealed`.

**Consequence.** `Option` is sealed-against-`extends` but does **not carry the `@sealed`
attribute** — bootstrap deliberately bypasses the decorator. So a `@variant` declared
inside an `Option` reopen would be rejected with
*"`@variant` requires its enclosing class `Option` to also carry `@sealed`"* — **a false
diagnostic about a class that is, in fact, sealed.** One property, two representations,
and they can disagree. Narrow and currently untested, but it is a real seam and the
cheapest moment to close it is before anything else reads either representation.

**The larger prize, and a genuine surprise: exhaustiveness is already enforced — by
dispatch, not by a checker.** `expand_variants` synthesizes one sibling class per variant
(each implicitly `@data`, `_`-prefixed mutable fields, superclass = the enclosing class),
each overriding a positional `__matchArm`, and generates `match(k1:, k2:, …)` on the
parent. Because selector identity is label-encoded
([ADR-0012](../adr/accepted/0012-selector-signature-encoding-and-dispatch.md)) and there are no
default arguments ([ADR-0043](../adr/accepted/0043-no-default-arguments-keep-selector-identity-pristine.md)),
**a missing arm is a different selector** — so an inexhaustive match cannot dispatch. Two
committed decisions that were made for unrelated reasons combine to give totality for
free. Green fixture: `tests/lang/errors/annotation_variant_visitor_exhaustive.ph`.

This sharpens what a future `match` construct would actually buy: **diagnosis** (naming
the missing arm) and **compile-time rather than dispatch-time** failure — not soundness.
`match` remains **OPEN** (open-Q7 residue;
[ADR-0046](../adr/accepted/0046-destructuring-bindings.md) shipped only irrefutable
destructuring). See [`drafts/sealed-classes.md`](../spec/v0.2/drafts/sealed-classes.md) §S-1.

**Spec-side divergences found in the same pass** (each its own small chore):
`@sealed`/`@variant` are **absent from** `decorators-stdlib.md` and `attribute-classes.md`
— their only spec is `experimental/annotations-data.md`, which *does* match the code;
U-ANNOT-LAYOUT's plan §3.4 specifies a finalize-phase end-of-unit post-pass while as-built
is an immediate subclass-site check (the code argues the equivalence; the plan was never
updated); and **no fixture tests cross-unit `extends` of a *user* `@sealed` class** — the
decorator's headline enforcement is exercised only through bootstrap-sealed core classes.

**Fix.** Make the `@variant` gate consult `VM::sealed_classes` (or record `@sealed` for the
bootstrap-sealed classes) so the two representations cannot diverge; add the missing
user-class `extends` fixture; spec `@sealed`/`@variant` where a reader would look for them.
Owning unit: unassigned.

### CB-4 · `experimental/default-arguments.md` specifies the mechanism its own banner forbids — **FIXED 2026-07-15**

_Verified 2026-07-15._
_**Fixed 2026-07-15** (docs-only). The doc is **retired/deleted**, not reconciled — user-ruled,
and the right call: `drafts/default-arguments.md` already carried everything it had, correctly
(§2 hazard, §5(a) ruled mechanism, §7 preclusions), so reconciling would have maintained two
answers to one question. Inbound links repointed (`deferred-work.md:50`/`:163`,
`experimental/README.md:9`); `deferred-work.md`'s standing chore is discharged; the draft's
DA-1 is closed and its §8 rewritten as the epitaph._

_**One half of this entry was wrong, and the error is instructive.** CB-4 claimed ADR-0043
"rejects arity-family expansion as *combinatorial*", in tension with Q12's ratification of
that mechanism where it is linear. **ADR-0043 never says that** — the word appears nowhere in
it (`grep -c combinatorial` → 0). The claim belonged to the retired `experimental/` doc, which
CB-4 read as speaking for the ADR. So the "general vs trailing-only" contradiction this entry
was filed to fix **did not exist**; retiring the doc removed its only source. **The real
defect, found by reading the ADR:** its Decision told a future ADR to choose "aliasing vs
**call-site fold**" — a door Q12 **permanently forbids** — and never mentioned trailing-only.
A reader following ADR-0043 alone would design against a forbidden mechanism. ADR-0043 now
carries a prose-only §Amendment recording that; the decision (no default arguments) is
untouched. Second Confirmed-Backlog entry to be partly refuted on contact (see CB-3) —
verify against the tree, never against the entry._

`docs/spec/v0.2/experimental/default-arguments.md` is 40 lines and self-contradictory:

- its **`## Decision`** section specifies **caller-side desugar with statically-known callees**;
- its own **2026-07-12 supersede banner** declares caller-side / static-callee resolution
  **"permanently forbidden"**, and names definition-time trailing-only overload desugar as
  the ratified-if-ever mechanism instead.

**A reader who skips the banner gets the forbidden answer** — and the banner is the part
readers skip. `docs/spec/v0.2/deferred-work.md:163` already logs this reconciliation as an
outstanding chore; it was never done.

**Related finding, worth an ADR amendment independent of the fix.**
[ADR-0043](../adr/accepted/0043-no-default-arguments-keep-selector-identity-pristine.md)
and this doc both reject arity-family expansion as *"combinatorial"* — while open-Q12
ratifies **that same mechanism** restricted to *trailing* params, where it is **linear**,
not combinatorial. The two are consistent on inspection (different scopes) but ADR-0043's
prose reads as a blanket rejection of the mechanism the ruling actually adopts, and it
never mentions the trailing-only refinement. See
[`drafts/default-arguments.md`](../spec/v0.2/drafts/default-arguments.md) §8 (DA-1/DA-2),
which records the contradiction rather than fixing it — editing a Proposed doc is outside
a draft's authority.

**Fix.** Reconcile the `## Decision` section to the banner's ruling, and amend ADR-0043's
prose to distinguish general (combinatorial, rejected) from trailing-only (linear,
ratified by Q12). Owning unit: unassigned.

### CB-5 · `Fiber`'s 11 primitives are installed but outside the frozen floor — **FIXED 2026-07-15**

_Found and verified 2026-07-15 while fixing CB-2, by reconciling the install site against
the test's audit set._
_**Fixed 2026-07-15** — option **(a)**, user-ruled: `Fiber` is admitted to the floor.
`("Fiber", c.fiber_class)` added to `core_class_rows` (28 → 29 rows), `NEW_FIBER = 11`
added with the 11 binding entries, **both** count assertions bumped 125 → **136** (the
second, on `live.len()`, is easy to miss — the first failure named it). Census gains
**§2.17** enumerating all 11 with sides and semantics read from `primitive/fiber.rs`, a
chain row, and §1.4 rewritten from "the defect" to "how the hole survived". Docs updated:
§1.1 (136/118/23, installed = audited), §7's coverage caveat, §8, `core/README.md`'s pin.
Test green — the set-difference check passing means the 11 enumerated selectors match the
installed set exactly._

_**Not a floor amendment, and no ADR opened.** No primitive was added: these bindings
shipped under ADR-0030. What changed is that they are now audited. The native boundary did
not move; the census's account of it did. §7's "open an ADR amending 0019" governs
**adding/removing** a primitive, which this is not._

_**The durable lesson, recorded in §1.4 and §7.** `core_class_rows` is the audit's real
boundary, and **nothing audits it** — a kernel class missing from that list is unfrozen in
fact, whatever ADR-0019 says. The census and the test agreed with each other (125 = 125,
green) because they were coupled, not because they were right; neither was ever compared
against the install site. A future kernel class added without its row reopens this hole
silently and identically. **Add the `core_class_rows` row in the same change.**_

**The defect.** `VM::new()` installs **136** native `(class, selector)` bindings. R-INV-0.1
(`floor_census_matches_installed_bindings`) audits **125** of them. The other **11** are
`Fiber`'s — and the gap is not a rounding error, it is a whole kernel class:

| | |
|---|---|
| Bound at | `universe/primitives.rs` L362-374 (`fiber_cls` block) |
| Class created at | `universe/core_classes.rs:152` — `make_core_class(heap, "Fiber", object_class, metaclass_class)` |
| Selectors | `Fiber.new(_)`, `#call`, `#call(_)`, `#try`, `#try(_)`, `Fiber.yield`, `Fiber.yield(_)`, `Fiber.current`, `Fiber.abort(_)`, `#isDone`, `#error` |
| In `core_class_rows` (the test's audit set, 28 rows)? | **No** |
| Mentioned in `floor-census.md`? | **Zero hits** before this pass |

**Consequence: the ADR-0019 freeze does not bind `Fiber`.** Add a primitive to `Fiber`, or
drop one, and the floor changes with **no red test and no doc edit** — the exact silent
drift R-INV-0.1 exists to prevent, for a class that ADR-0030 shipped as core concurrency.
`Fiber` is not exotic: it is a real kernel class carrying 11 real primitives, 8 distinct
native fns (`primitive/fiber.rs`).

**Why this went unseen.** The census and the test agree with each other perfectly (125 =
125, green), so every consistency check between *those two* passes. Nothing compares either
against the actual install site. CB-2's fix made §1.1 agree with the test — which is
correct but would have quietly cemented "the floor is 125" as settled truth. Documented as
`floor-census.md` §1.4 rather than silently reconciled.

**Two candidate fixes, needs a ruling.**

- **(a) Admit `Fiber` to the floor.** Add `("Fiber", c.fiber_class)` to `core_class_rows`,
  add a `NEW_FIBER: usize = 11` constant, write §2.17 enumerating the 11, write the
  amendment banner. Makes the freeze mean what it says. Cost: `Fiber`'s surface is then
  frozen under ADR-0019 — every future fiber primitive needs an ADR amendment. Given
  ADR-0030's scheduler work is live, that may be a real tax, and `Fiber` may not be settled
  enough to freeze.
- **(b) Declare `Fiber` deliberately out of scope** and say so in both the census and the
  test, so the omission is a recorded decision rather than an oversight. Cheaper, honest,
  but leaves a native surface nothing guards.

Leaning (a) — an unfrozen native class is precisely what ADR-0019 was written to prevent,
and "the floor is closed" is false while an 11-binding hole exists. But **(b) is defensible
if fiber primitives are still churning**; that is a question about ADR-0030's roadmap, not
about the census. Owning unit: unassigned. **Do not quote "the floor is 125" as the whole
native boundary until this is ruled** — it is the audited floor, not the installed one.

### CB-6 · `to_display_string` bypasses every container, not just `List` — FIXED 2026-07-16

_Found and verified 2026-07-15 while fixing CB-1, by testing an override nested one level
down. Exposed, not caused, by CB-1's fix. **Correction:** the entry as originally filed
framed this as "`List` is the odd one out among the collections" — that framing was wrong.
The repro that grounded the actual fix showed `System.print(m)` (a bare `Map`) *also*
disagreeing with `"\(m)"` on the same value; `List` was never uniquely broken, it was just
the first case found._

**The defect, correctly scoped.** Two independent sites, both bypassing dispatch:
- **Site A — `Value::to_display_string`** (`value/render.rs`, what `System.print` calls)
  hardcoded `Str`/`List`/`Map`/`Set`/`Tuple`/`Range` as "already handled natively" and
  skipped the `toString` send for all of them, falling back to the non-sending
  `Value::to_string`. That was true before those types grew `.ph` `toString` overrides;
  once they did, the special case silently disagreed with `"\(…)"` interpolation (which
  always sends) on the exact same value — e.g. `System.print(m)` rendered
  `{k: <Secret instance>}` while `"\(m)"` correctly rendered `{k: <redacted>}`.
- **Site B — `list_to_string`** (`primitive/list.rs`) rendered elements via the
  non-sending `Value::to_string`, so an override nested inside a `List` was bypassed
  regardless of Site A.

```phalcom
class Secret { toString => "<redacted>" }
let s = Secret.new()
System.print([s])   // was [<Secret instance>], now [<redacted>]
System.print(m)      // was {k: <Secret instance>}, now {k: <redacted>}  (Map, not just List!)
```

**The fix.** `to_display_string` now sends `toString` unconditionally — the
`handled_natively` special case is gone. `list_to_string` now renders each element via
`to_display_string` (a real send) instead of `Value::to_string`. `Value::to_string` itself
is untouched (still the non-sending renderer used by `to_debug`/diagnostics). Depth closes
by ordinary recursion through dispatch: a container's native `toString` renders its
elements by sending, so a nested container's own native `toString` sends again.
`list_to_string` stays a native primitive; floor stays 136 — no primitive added or removed.

Pinned by `tests/lang/strings/string_interp_sends_tostring.ph` (its former "wrong on
purpose" line now asserts the correct `[<redacted>]`), plus two new fixtures:
`tests/lang/strings/tostring_dispatch_depth.ph` (nesting `List`/`Map` inside each other)
and `tests/lang/strings/print_interp_agree_containers.ph` (the Site A regression guard —
`System.print(x)` and `"\(x)"` must agree for the same container value).

## Open entries

| file:line | Category | Rationale |
|---|---|---|
| `docs/spec/v0.2/drafts/decorators-dispatch-observability.md` D-1 (decision recorded 2026-07-13) | feature scope — deferred to v0.3 | A separate Dispatch-tier `@forwardMissing(to:)` decorator (forward *every* missed selector on a field via `doesNotUnderstand`) was considered and rejected for v0.2 in favor of Compile-tier `@delegate`'s explicit-selector-list-only surface. Revisit only if whole-interface delegation proves common enough in practice to earn a second decorator; until then the hand-written `Proxy`/DNU library ([proxy.md](../spec/v0.2/drafts/proxy.md)) covers the open case. |
| `docs/spec/v0.2/drafts/decorators-dispatch-observability.md` D-2 (decision recorded 2026-07-13) | feature scope — deferred to v0.3 | `@traced`'s `sink:` argument was specified as a pluggable duck-typed `Tracer` protocol (`enter`/`exit`/`threw`) with a `Tracer.stdout` builtin default, rather than three raw configuration blocks (`onEnter:`/`onExit:`/`onThrow:`). The raw-blocks form remains available as a fallback design if the `Tracer` protocol proves over-engineered once implemented. |
| `docs/spec/v0.2/drafts/decorators-dispatch-observability.md` D-3 (decision recorded 2026-07-13) | feature scope — deferred to v0.3 | `@featureFlag` was specified against a ratified ambient `Flags` core module (global registry, `Flags.enabled(name)`), not an injected/per-scope `FeatureFlags` service. Revisit once dependency injection (`@inject`, [decorators-stdlib.md](../spec/v0.2/drafts/decorators-stdlib.md)) is specified — the injected-service form is the natural upgrade path and does not require an `@featureFlag` grammar change, only a resolution-strategy change inside the decorator. |
| `phalcom-core/src/primitive/boolean.rs:34` `bool_class_new` (found porting Wren `test/core/bool/no_constructor.wren`) | correctness — panic, not a controlled error | `Bool.new()` (zero-arg) indexes `args[0]` unconditionally and panics (`index out of bounds: the len is 0 but the index is 0`) instead of raising a catchable `RuntimeError` — every send should fail loud-but-controlled, never a raw Rust panic. Also carries two pre-existing debug `println!`s (already flagged, U1/DEFERRED) that fire before the panic. Could not be ported as a `runtime-errors`/negative golden because the harness's `assert_no_panic` forbids exit-101/`panicked at` output; skipped in the Wren-porting sweep pending a real fix (bounds-check `args`, return `RuntimeError::Arity` on empty). |
| `phalcom-core/src/primitive/class.rs` `class_class_new`/metaclass `new()` (found porting Wren `test/core/class/no_constructor.wren`) | design divergence, not a bug | `Class.new()` succeeds silently (no error, exit 0) rather than rejecting instantiation the way `Bool.new()`/`Object.new()`/`System.new()` all reject theirs (`RuntimeError::NotAllowed`) — `Class` is the one metaclass-tower root left instantiable. Unclear if intentional (a raw `Class` instance may be meaningless) or an oversight; flagged rather than silently ported as a positive/negative case either way. |
| ~~`phalcom-lsp/src/semantic_tokens.rs` (U-LSP Stage 5)~~ | ~~feature scope~~ | **RESOLVED 2026-07-14:** `ClassDef`/`MethodDef`/`GetterDef`/`SetterDef`/`ConstructDef` grew an additive `name_range: SourceRange` field (`phalcom-ast/src/ast.rs`, populated in `parser.rs`) keying the declaration name's own span independent of the whole-declaration `range`. `semantic_tokens.rs` now runs an AST-assisted `apply_decl_name_overrides` pass on top of the flat lexer pass, upgrading declaration-name tokens to `class`/`method` (never downgrading references). No heuristic text re-scan needed — the rejected approach this entry originally flagged.
| `phalcom-lsp/src/semantic_tokens.rs` (U-LSP Stage 5) | feature scope — deliberately deferred | Comments (`//`, `///`, `//!`) are invisible to the flat lexer token stream (`Lexer::skip_trivia` discards them before `Iterator<Item = Spanned<Token,..>>`), so no `comment` semantic type is emitted. Not a visual regression in practice: VS Code layers semantic tokens over the TextMate grammar only for ranges the server actually classifies, so the grammar's own comment-coloring rule still applies underneath wherever no semantic token is emitted. A second raw scan of `doc.text` (shareable with Stage 4's Phaldoc harvest) would be needed to close this gap under the LSP path, if a `comment`/Phaldoc-distinct semantic type is ever wanted. |
| `tools/vsphalcom` client migration (U-LSP, all 5 stages) — **RESOLVED 2026-07-13** | correctness/UX | Per plan.md "Client migration": deleted `diagnostics.ts`, `completions.ts`, `context.ts`, `hover.ts` (all superseded server-side); flipped `phalcom.lsp.enabled` default to `true` (previously `false`, which — combined with the plan's stage-by-stage TS-provider deletion — would have left default-off users with no diagnostics/completion/hover at all, a real regression the plan's step ordering didn't call out). `extension.ts` collapsed to the ~60-line `LanguageClient` launcher the plan's end state describes (registers `phalcom.runFile` plus a single `lsp.enabled` branch). TextMate grammar (`syntaxes/phalcom.tmLanguage.json`) intentionally left registered as-is — no `package.json` "demotion" mechanism exists or is needed; VS Code natively layers `semanticTokens/full` over the grammar for any range the server classifies, and falls back to grammar coloring elsewhere (comments) or entirely (if a user sets `lsp.enabled: false`). DEC-VSP-C closed. |
| `phalcom-lsp/src/semantic_tokens.rs` (U-LSP Stage 5) | judgment call, confirmed from prior handoff | Structural punctuation (parens/braces/brackets/comma/dot/colon/arrows) and `Newline`/`Eof` are left uncolored rather than mapped to `operator` — avoids visual noise the plan didn't ask for; only binary/unary/compound-assignment operators (`+`, `==`, `+=`, `??`, …) get `operator`. No token modifiers declared for v1 (empty legend `token_modifiers`). `NameSymbol`/`SelectorSymbol` map to a custom server-declared `"selector"` token type rather than reusing `method`. |
| `phalcom-core/src/vm.rs` `Bytecode::MakeFamily` handler (U16-Open) | performance (deferred per ADR-0012) | An Open family call re-interns its target selector on every invocation (`family_does_not_understand` -> `encode_selector`/`get_or_intern`) rather than using a monomorphic inline cache keyed by `(call_site, class_id)`, which selectors.md §3's "Performance" section promises as the eventual fast path. ADR-0012 already defers general IC population; this unit keeps the *shape* IC-ready (the selector is still built from a small, fixed label list per call) but does not populate a cache. |
| `phalcom-core/src/heap.rs` `Object::Family` vs `Object::BoundMethod` (U16-Open) | design / future unification | `Family` (this unit) and `Method#bind(_)`'s `BoundMethodObject` (ADR-0028/U-CORE-3) are two independent routes to a bound callable value with no shared representation or protocol beyond both answering `Function`'s call surface. functions.md §3 already flags this as an open unification question; U16-Open does not resolve it, only keeps `Family` from precluding a future merge (no `Family`-specific opcode beyond `MakeFamily`/`FinalizeClass`, everything else rides the ordinary send path). |
| ~~`phalcom-core/src/value.rs` `Value::value_eq`~~ | ~~correctness~~ | **RESOLVED by U-LEX-HASH:** `(Value::Symbol(a), Value::Symbol(b)) => a == b` added to the match. |
| ~~`phalcom-ast/src/lexer.rs` (no `#IDENT` symbol-literal token)~~ | ~~feature / reserved syntax (U-LEX)~~ | **RESOLVED by U-LEX-HASH:** `Token::NameSymbol`/`Token::SelectorSymbol` land (selectors.md §2); `tests/lang/collections/literal_map_symbol_keys.ph` graduated out of `pending/`. |
| `phalcom-ast/src/parser.rs` `parse_comma_exprs` (~L1700) | feature / grammar hole (U-COLL) | A leading-`*` spread element (`[*xs, y]`, `(*a, b)`; spec §8) is reserved but rejected with a "not yet supported" diagnostic. Wiring it to a spread-send is additive once spread-at-call-site (`f(*args)`) is finalized (U9 follow-on). |
| `phalcom-ast/src/lexer.rs` (no `#{` / `..` tokens) | feature / reserved syntax (U-COLL non-goal) | The set literal `#{1,2,3}` (Open-Q6) and the range literals `1..5` / `1...5` (ADR-0032 §3.3, reserved-inactive) are not lexed. Distinct future lexer tokens, foreclosed by nothing in U-COLL; `Set(…)` sends remain the shipping answer for sets. |
| `docs/spec/v0.2/object-model.md` §5:210-211; `docs/spec/.../implementation-status.md` | docs-drift | Note claims "every metaclass's superclass wired to `Class`, breaking it" — stale pre-U2; native tower now satisfies ADR-0002 rule 4 (tested) and U-INH extends the same rule to user classes. Re-point both. |
| `phalcom-core/src/bytecode.rs` (`SuperSend`) | perf / IC follow-on | `SuperSend` is uncached (DEC-INH-F). Wire the inline-cache seam **with U15/U16** so a `superclass=` (U15) / override-epoch bump (ADR-0018) invalidates a cached `SuperSend` the same way it invalidates `Invoke`. |
| `docs/forge/units/README.md`, phase INDEX | docs roster | Add the `U-INH` roster row (landed). Not edited in-unit — shared-file concurrent-session hazard. |
| `phalcom-core/src/compiler/lib.rs:~1081` (`has_new_construct` guard) + `expr.rs:103` (arity guard) | correctness — **DISSOLVED BY RULING 2026-07-15 (DEC-CTOR-H), not fixed** | **This row's premise is now rejected.** It treated a wrong-arity `Sub.new()` falling through to the bare allocator as a *bug*. [ADR-0063](../adr/accepted/0063-constructors-are-ordinary-class-side-methods.md) §7 rules the opposite: **`new()` is an ordinary inherited method**, so `C.new()` on a class whose only constructor is `new(n)` returns an object with every field `None` — **specified behavior**, exactly as Smalltalk (defining `new: x` never removes the inherited `new`). The compile-time guard at `expr.rs:103` is therefore **deleted by U-CTOR-4**: it is *wrong*, not merely incomplete, because it rejects a legal send. It was also incompletable by construction — it needs `receiver_class_sym`, which exists only for a bare-identifier receiver, so the same call answered two ways (`Factory.new()` → compile error; `var C = Factory; C.new()` → `<Factory>` with `n = None`). Under the ruling both return an empty object and the asymmetry has nothing left to be asymmetric about. **What survives:** the one genuinely *unsound* case is blocked separately by ADR-0063 §6.1 — `new_` refuses on any class whose instances are not `Object::Instance` (`native_repr`), so a type-confused `Number` is still impossible. **Cost accepted knowingly:** the wrong-arity typo (`Factory.new()` when you meant `new(5)`) becomes silent. A future *general* lint — "receiver is statically a known class, selector matches nothing on it" — could return the diagnostic for **every** static send without reintroducing the asymmetry. Row kept, not deleted: the reopen-method-loss entry below cross-references it, and the historical detail (inheritance-aware `class_parents` chain-walk shipped 2026-07-13, [[ctor-inherit-guard-fix]]) explains code U-CTOR-4 is about to remove. |
| `phalcom-core/tests/lang/iteration/pending/` (not yet created) | test / cross-unit (U-FIBER) | U-ITER step 5 was cut: the PENDING generator fixtures `for_generator_suspends.ph` (C-ITER-8 — `Fiber.new { for (x in [1,2,3]) { Fiber.yield(x) } }` suspends and yields `1,2,3`) and `each_generator_raises.ph` (`.each { Fiber.yield }` → `CannotYieldAcrossNativeFrame`) graduate with the **U-FIBER** landing. The `for` disasm golden (C-ITER-4) already proves the compile-time half (no `block_call` in the `for` chunk); these pin the runtime half. |
| ~~`phalcom-core/core/core.ph` `class List` (`each`/`map`/`filter`/`reduce`/`includes`)~~ | ~~std-lib follow-on (U-STD)~~ | **RESOLVED by U-ITERABLE:** the combinators now live on the kernel `Iterable` root (`core.ph:309`), driven entirely by `iterate(_)`/`iteratorValue(_)` (DEC-ITER-A). `List` inherits `each`/`map`/`filter`/`reduce` from `Iterable` and only overrides `iteratorValue`. **Caveat:** landing this changed the cursor protocol to bare-index Route B (ADR-0048) and broke 5 golden fixtures that still assert the old `Some`-wrapped cursor output — see `UNITS-TRACKER.md` §5 U-ITERABLE entry; fix before treating this as fully closed. |
| `phalcom-core/src/compiler/lib.rs` `compile_for` (loop-variable slot) | semantics / correctness | The loop variable is one reused local rebound each iteration via `SetLocal`, so a closure captured in the body over it observes the loop's **final** value, not the per-step value (spec §3.3 wants per-iteration freshness). Matches the existing inlined-`while` capture behavior; not exercised by C-ITER-1..7. Fix needs a fresh cell per iteration (a `CloseUpvalue`-per-step in the loop body). |
| `phalcom-core/src/compiler/inliner.rs` `compile_while_true` | feature parity (out of write-set) | `break`/`continue` bind only inside a `for` body: a `while` lowers via the inliner's `compile_while_true`, which pushes no `LoopContext`, so `break`/`continue` inside a bare `while` currently raise the out-of-loop compile error. Spec §3.2 wants `while`+`break`/`continue` too; realizing it needs `inliner.rs` (outside U-ITER's write-set) to push/pop a loop context around its jump loop. |
| `phalcom-core/src/compiler/lib.rs:~1043` (`patch_forward_jump_to`) vs `inliner.rs:167` (`emit_jump`) | dedup / DX | U-ITER re-implements the jump/patch/loop helpers (`emit_forward_jump`/`patch_forward_jump_to`/`emit_backward_loop`) because the inliner's equivalents are module-private and `inliner.rs` was outside the write-set. Once both are co-editable, hoist a shared jump-emission helper set onto `Compiler` and drop the duplicates. |
| ~~`phalcom-lsp/src/completion.rs` `to_completion_item` (Stage 3, U-LSP)~~ | ~~correctness — over-offer~~ | **RESOLVED 2026-07-14:** new `ReceiverKind` (`Instance`/`ClassObject`) returned alongside the resolved class name; `completions()` filters `StaticMethod`/`Construct` members out on `Instance` and down to only those on `ClassObject`. Also fixed a related bug found in the process: `index.rs::member_kind` wasn't honoring `is_static`/`Construct` for user classes either (mis-tagged `Method`), now corrected. |
| ~~`phalcom-lsp/src/completion.rs` `collect_class_members` (Stage 3, U-LSP)~~ | ~~completeness — implicit Object~~ | **RESOLVED 2026-07-14:** a user class with no explicit `extends` (or one whose chain dead-ends at a user class) now defaults its effective parent to `Object` and walks its builtin member list from `core_table.rs`, same mechanism as an explicit `extends`. |
| ~~`phalcom-lsp/src/completion.rs` `ConstructResolver::resolve` (Stage 3, U-LSP)~~ | ~~precision — dataflow scope~~ | **RESOLVED 2026-07-14 (partial):** resolver now walks an enclosing lexical scope chain (innermost-to-outermost shadowing, only bindings lexically before the cursor) instead of whole-document last-binding-wins, and resolves `self.` to the enclosing class. Still NOT done, left deferred: method/block **parameters** aren't tracked (no call-site type inference) and **cross-file** bindings aren't followed (resolver only sees the open `Document`) — a future type-inference `ReceiverResolver` can still drop in behind the same trait. |
| ~~`phalcom-core/src/compiler/lib.rs` `compile_break`/`compile_continue` (`func_depth` guard, ~L1246/L1272)~~ | ~~semantics / correctness (reviewer-found, adjudicated)~~ | **RESOLVED by U-REOPEN-FIX (conservative option, adjudicated by user 2026-07-12):** `Compiler::emit_deopt_block_control_trap` (compiler/lib.rs) now emits `Error.new(message).raise()` with a descriptive message ("...materialized block...") instead of a bare `Error.new().raise()`, so `break`/`continue` reached through a **materialized** block fails loudly with a clear diagnostic rather than silently no-oping. `tests/lang/iteration/pending/{break,continue}_across_materialized_block.ph` graduated to `tests/lang/iteration/negative/` (NEGATIVE lane, `iteration_negative` test). Full non-local break/continue (threading the target across `FunctionState` frames) remains a larger follow-on, not attempted here. |
| ~~`phalcom-core/src/primitive/fiber.rs:~109` (`fiber_abort`)~~ | ~~correctness / spec~~ | **RESOLVED by U-FIBER-FIX:** `fiber_abort` now mirrors `fiber_yield`'s root-fiber guard (`resumer.is_none()` → `NotAllowed("cannot abort the root fiber")`). Golden: `tests/lang/concurrency/negative/fiber_abort_root_raises.ph`. |
| ~~`phalcom-core/tests/lang/concurrency/` (C-FIB-5, no golden)~~ | ~~test coverage~~ | **RESOLVED by U-FIBER-FIX:** golden `tests/lang/concurrency/negative/fiber_cross_fiber_non_local_return_dead_frame.ph` + `tests/invariants.rs::cross_fiber_non_local_return_raises_dead_frame_error`. |
| ~~`phalcom-core/src/primitive/fiber.rs:~144` (`fiber_resume` gate)~~ | ~~diagnostics / DX~~ | **RESOLVED by U-FIBER-FIX:** `fiber_resume`'s native-reentry gate now raises via a distinct `cannot_resume_across_native_frame` builder ("cannot resume a fiber across a native call frame…") instead of reusing the yield-specific message; the restriction itself is unchanged (still sound, still wider than spec §6's table by design). Golden: `tests/lang/concurrency/negative/fiber_resume_gate_call_native_frame.ph`. |
| ~~`phalcom-core/src/vm.rs` `run_until` failure loop~~ | ~~spec conformance~~ | **RESOLVED by U-FIBER-FIX**, then **completed in `94487af`:** U-FIBER-FIX's fix cleared only `.frames` on every fiber the cascade marks `Failed`; `94487af` found (via debug instrumentation on a 3-fiber-chain repro — `frames.len()=0` but `stack.len()=3` after clearing) that `.stack`/`.open_upvalues` were still left populated on an intermediate `Call`-mode resumer, contradicting the fix's own "pure retention — clear them here" comment. All three fields now cleared together (spec §5.1). Harmless pre-GC (nothing reads freed state; everything leaks regardless with no collector yet, see [U-GC](units/U-GC/plan.md)) but was a real inconsistency the collector would have inherited. |
| ~~`phalcom-core/src/primitive/fiber.rs:~228` (`fiber_yield`) vs `vm.rs` `switch_to_fiber_and_deliver`~~ | ~~dedup / DX~~ | **RESOLVED by U-FIBER-FIX:** `fiber_yield` now calls `VM::switch_to_fiber_and_deliver` (made `pub(crate)`) instead of hand-inlining its body. |
| ~~`phalcom-core/src/primitive/fiber.rs` `fiber_resume` (~L194-213, pre-`94487af`)~~ | ~~correctness / robustness~~ | **RESOLVED (`94487af`):** a not-yet-started callee's entry-arity check ran *after* `store_live_into` had already stolen the resumer's live stacks, leaving an early-return window where the resumer's state sat orphaned in its own `FiberObject` without being restored. Traced end-to-end (pre-fix vs post-fix binaries, several call/try/cascade shapes) — never externally observable in this codebase (the resumer is unconditionally marked `Failed` by the same cascade regardless, discarding the state before anything reads it), but still the wrong invariant (validate before mutating shared VM/heap state). Moved the check ahead of the steal. Same commit also fixed a genuinely user-visible bug caught along the way: the arity error always named the signature `"call"` even when raised from `try()`. Two goldens lock both: `tests/lang/concurrency/fiber_first_resume_arity_mismatch_does_not_corrupt_resumer.ph`, `tests/lang/concurrency/negative/fiber_try_first_resume_arity_mismatch_names_try.ph`. |
| ~~`phalcom-core/src/compiler/lib.rs` `Statement::Class` reopen branch (~L672-900) vs `vm.rs` `Bytecode::Class` (~L1201)~~ | ~~correctness (found while landing U13)~~ | **RESOLVED by U-REOPEN-FIX:** root cause was `vm.rs`'s `Bytecode::Class` handler unconditionally calling `create_class` (a fresh `ClassId`) — the compiler's compile-time reuse guard (`Statement::Class`, keyed on `self.vm.classes`) never fires for a same-unit reopen because the whole unit compiles to one closure before any `Bytecode::Class` executes, so `self.vm.classes` is still empty for both blocks at compile time. Fix: `Bytecode::Class` now checks `self.classes.get(&name_sym)` at *runtime* (populated once block 1's `Class` opcode has actually executed) and reuses the existing `ClassId` — pushing it for the member-install loop to append into — instead of allocating a new one. `class A { greet => "hi" } class A { farewell => "bye" }` now resolves both methods on one instance (`tests/lang/classes/class_reopen_appends_methods.ph`, PASS lane). Two reopen shapes remain explicitly out of scope and are now rejected at **compile time** with a clear diagnostic instead of being silently mishandled: adding fields (the reused `ClassId` is never relayouted — `tests/lang/classes/negative/class_reopen_add_field_rejected.ph`) and changing the superclass (U13 sealed inheritance — `tests/lang/classes/negative/class_reopen_superclass_conflict_rejected.ph`), both via `classes_negative`. |
| `phalcom-core/src/compiler/lib.rs` `is_non_error_literal` (U-ERR `throw` lowering) | feature scope | The compile-time "`throw` of a non-`Error` literal is a compile error" check (error-handling.md §1) only recognizes `Expr::Number`/`Expr::String`/`Expr::Boolean` — the AST has **no** dedicated list/map-literal node (`[1,2,3]`/`{k:v}` desugar to `Expr::MethodCall` chains in the parser, U-COLL), so a `throw [1,2,3]`/`throw {a:1}` cannot be cheaply proven non-`Error` at this AST layer without fragile pattern-matching on the desugar shape; it defers to the runtime `doesNotUnderstand` miss on `raise()` instead (same outcome, one hop later — never silently wrong). Fix (low priority): recognize the specific `List.new()...`/`{k:v}`-desugar `MethodCall` shapes, or give list/map literals a dedicated `Expr` node first. |
| `phalcom-core/core/core.ph` `class Ok`/`class Err` construction | surface sugar (DEC-ERR-B, resolved (B)) | `Ok(v)`/`Err(v)` bare call-form construction sugar does not exist — `Ok`/`Err` are plain `.ph` classes constructed via `Ok.new(v)`/`Err.new(v)` (the `Name(args)` postfix-call form always compiles to `Name.call(args)`, not `Name.new(args)`; there is no general "class with `construct`" sugar in the parser, confirmed by `Some(x)`'s own bare-call form being unlanded/PENDING at U-ERR dispatch time, `tests/lang/absence/pending/absence_option_some.ph`). Follow-on: generalize the `Name(args)` postfix-call desugar to `Name.new(args)` when `Name` resolves to a class with a matching `construct` (would also graduate `Some(x)`'s own pending fixture) — a `phalcom-ast/src/parser.rs` + `compiler/lib.rs` change, out of this unit's write-set discipline (keep small). |

| `phalcom-core/src/primitive/class.rs` / metaclass super lookup (U-ERR test-wave, correctness) | correctness — **RESOLVED 2026-07-13 (U-ERR-FIX)** | **FIXED**: root cause was in `compiler/lib.rs`'s `compile_super_send`, not the VM dispatch — it anchored every `SuperSend`'s `defining` name to the *instance* class regardless of static/instance context, so a class-side `super` walked the instance-side superclass chain looking for a method only installed on the metaclass. Fix re-anchors `defining` to the metaclass's own name (`"<Name>.class"`, `VM::create_class`'s ADR-0002 parallel-rule naming) when `self.is_static_context`, so `Bytecode::SuperSend` resolves the metaclass chain instead. **Write-set note**: this required editing `compiler/lib.rs` (outside the unit's declared write-set of `value.rs`/`primitive/class.rs`) — flagged per the unit's STOP-and-report clause; the fix is a 10-line, single-function, additive change with no ripple, and is the only place the defining-class name is computed for `SuperSend`, so no alternative in-write-set location exists. Graduated `tests/lang/inheritance/inheritance_static_override_super.ph` (3-level static override + `super` chain). |
| `phalcom-ast/src/parser.rs:1279` `parse_property_name` (U-ERR test-wave, grammar gap) | feature scope — **RESOLVED 2026-07-13 (U-ERR-FIX)** | **FIXED**: `parse_property_name` now admits the same operator-token set `parse_method_name` accepts at definition (`+ - * / % == != < <= > >= and or is`), so `super.<operator>(...)` parses and dispatches through the existing `SuperSend` path with no new binding. Graduated `tests/lang/inheritance/inheritance_super_operator.ph`. |
| `benchmarks/wren-suite/method_call.ph:13` (`_state = !_state`) (found landing U-NEG) | correctness — stale prefix `!` site, out of write-set | U-NEG retired prefix `!` as an expression operator; this benchmark port still uses `!_state` and will now fail to parse. Out of U-NEG's declared write-set (`parser.rs`/`token.rs`/`core.ph`/`tests/`/`is-tests.md` — `benchmarks/` isn't listed and isn't CI-wired, per the `benchmarks/math/*.ph` "not wired into CI, run manually" convention), so left as-is rather than edited under a different unit's banner. Fix: swap to `_state = not _state`. |
| `phalcom-ast/src/parser.rs` unary-prefix parse (U-ERR test-wave, dead token) | correctness / cleanup — **RESOLVED 2026-07-13 (U-ERR-FIX)** | **WIRED** (not removed): `syntax/grammar.md`'s `unary := ( "-" \| "!" \| "not" ) unary` and `syntax/expressions.md`'s precedence table both explicitly list `not` alongside `!` at the same unary-prefix precedence, so the default-to-remove rule didn't apply. `parse_unary` now treats `Token::Not` as a synonym for `Token::Bang` (same `UnaryOp::Not`). Graduated `tests/lang/lexical/lexical_not_keyword.ph`. |
| `phalcom-core/src/value.rs` `Value::to_string` vs `Object#toString` (U-ERR test-wave, correctness) | correctness — **RESOLVED 2026-07-13 (U-ERR-FIX)** | **FIXED**: added `Value::to_display_string` (value.rs), which keeps `Value::to_string`'s native formatting for `Str`/`List`/`Map`/`Set`/`Tuple`/`Range`/`None`/`Some` and sends `toString` for every other heap object; `system_class_print` (primitive/system.rs) now renders through it. `System.print(p)`/`p.toString` agree for instances, classes, and metaclasses. Graduated `tests/lang/values/value_object_print_matches_tostring.ph`; rebaselined the 5 goldens/fixtures whose pinned output encoded the old debug-form divergence (`golden.rs` `example_core_new`/`example_person`; `messages_unary_send`, `messages_class_reflection`, `collections/brace_disambiguation_blocks`, `collections/literal_tuple`). |

| `phalcom-core/src/interpret.rs::VM::import_module` (U15) | feature scope, explicitly deferred | **No compiled-bytecode `import`.** `import` resolves to and compiles a `.ph` **source** file only — the `resolve → obtain-a-compiled-chunk → instantiate-Module` seam is left abstract enough for a future loader, but no bytecode verifier exists and loading unverified bytecode is a security hole (ADR-0045 Part 1 item 7). Owning unit: unassigned, gated on a verifier unit landing first. |
| `phalcom-ast/src/ast.rs::Statement::Import`, `phalcom-ast/src/parser.rs::parse_import` (U15) | feature scope, explicitly deferred | **No selective import (`import a, b from "path"`) and no `export`/`_`-prefix privacy enforcement.** Every top-level name is a member in Draft 0.1 (DEC-U15). `from`/`export` are reserved but unlexed; the whole-module `ImportStatement` AST node can grow an optional selective-name list additively per the U15 plan §7 without another AST/opcode redesign. Owning unit: a follow-on module unit, ADR-0027 §2/§3's original grammar. |
| `phalcom-core/src/interpret.rs::resolve_import_path` (U15) | security, explicitly deferred | **No path-traversal / sandboxing policy.** A relative `import "../../..."` can walk outside the program's own directory tree; nothing confines resolution to a root. Not a concern for a single-author script today (no untrusted source is ever `import`ed) — flagged for a future security ADR before Phalcom runs untrusted `.ph` input (ADR-0045 Alternatives; `modules.md` §9). |

| `phalcom-core/src/vm.rs:396` `VM::register_source` (U15 reviewer-found, non-blocking) | diagnostics / DX | `SOURCE_MAP` is keyed by **logical name** (file stem) alone, not canonical path — two imported files sharing a basename (`./a/utils.ph` + `./b/utils.ph`) overwrite each other's diagnostic-source entry. Module **values/identity are unaffected** (registry is keyed correctly by canonical path); the only risk is a *runtime*-error diagnostic later showing the wrong source snippet for a same-basename collision (compile-time parse errors use the locally-held source string directly — correct source confirmed live). Also pre-existing: lines ~396-403 do the same `SOURCE_MAP...insert(...)` **twice** (dead duplicate, not introduced by U15). Fix: key `SOURCE_MAP` by canonical path; drop the duplicate insert. |
| `phalcom-core/src/compiler/lib.rs` `compile_pattern_bind_top_of_stack`/`compile_pattern_bind_from_slot` (U14) | perf / DX | Every destructuring `let`/`var`'s scratch locals (`$destructure`, and each compound sub-element's own claim) are never scoped away — unlike `compile_for`'s `$for_coll`/`$for_cursor`, they stay resolvable (and occupy a stack slot) for the rest of the enclosing block, because the pattern's *leaf* bindings must survive at the same depth and there is no cheap way to tear down only the internal scratches without also un-resolving the leaves. Harmless (correct, just a few extra reserved slots per destructuring statement) — a slot-reuse pass could reclaim them once un-day-one profiling shows it matters. |
| `phalcom-core/core/core.ph` `class List` (no slice/tail selector) (U14) | feature / floor-minimality | `let [first, *rest] = list`'s rest tail is built via a hand-rolled `while` copy loop in `compiler/lib.rs::compile_list_rest_and_bind` rather than a single `List#sliceFrom(_)`/`tail(_)` selector, because adding one would be a new floor primitive (ADR-0019) or a `core.ph` edit for a single call site — both outside U14's write-set. If a slice/tail selector is added for other reasons later, re-point the rest-tail lowering onto it. |
| `docs/spec/v0.2/deferred-work.md:41,51` (U14, out of this unit's write-set) | docs-drift | Both lines still say list/`*rest` destructuring is deferred to a future pattern-matching unit — stale as of ADR-0046 (U14 ships it now, irrefutable). `open-questions.md` Q7 and its summary-table row are already corrected in this unit; `deferred-work.md` itself was intentionally left untouched (outside U14's declared write-set) — a follow-on doc pass should reconcile these two lines with ADR-0046. |
| `phalcom-ast/src/lexer.rs` `Lexer::scan_symbol` (U-LEX-HASH) | feature scope, deliberately deferred | **`#[]` (the bracket-subscript operator selector) is not lexed.** The other two `selectors.md §2` operator-symbol examples (`#+`, `#==`) map onto an existing, always-arity-1 method-definition convention (`parse_method_name`'s operator set); `[]` has no such counterpart — there is no `[](...)`  method-definition surface syntax anywhere in the parser yet (no way to fix its arity/canonical-form convention against a real declaration), so guessing one (e.g. `SubscriptGet(1)`) would be unverifiable and easily wrong once subscript methods actually land. Fix: lex `#[]`/`#[]=` once subscript method-definition syntax exists to canonicalize against. |
| `phalcom-ast/src/lexer.rs` `Lexer::scan_symbol` (U-LEX-HASH) | feature scope, deliberately deferred | **`#+(_)` (explicit-paren operator selector form) is not lexed** — only the bare `#+` form is. selectors.md §2's canonical-form table shows `+(_)` as the selector's *conceptual* spelling, but the Lexing section's regex only covers identifier-headed names, with operators explicitly called out as "a separate branch" (implying bare-token-only); the surface grammar is genuinely ambiguous on whether `#+(_)` should also lex. Kept minimal rather than guessing. Blocks: `tests/lang/functions/pending/functions_method_for_invoke_on.ph` (`3.methodFor(#+(_))`) stays PENDING — unrelated to `functions_method_bind.ph`, which graduated this unit using the bare-identifier selector form (`#greet(_)`). |
| `phalcom-ast/src/parser.rs` `at_expr_start` (found while landing U-LEX-HASH, out of write-set discretion — kept minimal) | correctness (pre-existing) | `at_expr_start` (guards `return <expr>?`'s optional value) is missing `Token::StringInterp` and `Token::LBracket` from its match, predating this unit — `return "\(x)"` and `return [1,2]` both fall back to a bare `return` instead of parsing the literal as the return value. This unit only added its own `Token::NameSymbol`/`Token::SelectorSymbol` arms to the same match; the two pre-existing gaps are out of scope here (touching them isn't required for `#` literals to work as return values, which the added arms already cover) but should be swept up together in a follow-on. |
| `phalcom-core/src/vm.rs` `Bytecode::MakeFamily` (U16-Pinned reviewer-found, non-blocking) | robustness / DX | The Open-vs-Pinned family discriminator is **stringly-typed**: `open = !selector_str.contains('(')`. Sound TODAY given two grammar invariants — (a) Open's constant is a bare identifier from `parse_property_name` (identifiers structurally cannot contain `(`), (b) Pinned's constant is always `encode_selector(_, _, SignatureKind::Method(_))` which unconditionally appends `(`. Reviewer confirmed no current input misclassifies. But it silently breaks if either invariant changes (a future identifier grammar admitting `(`, or a paren-less `SignatureKind` variant reused for Pinned lowering). Hardening: carry the kind explicitly — a tag bit on the `MakeFamily` operand/constant, or a distinct opcode — instead of re-deriving it from the selector string. |
| `phalcom-core/src/primitive/family.rs:~53` (U16-Pinned reviewer-found, cosmetic) | docs-drift | Router doc prose quotes selectors.md §3 as "the selector is built from `family.name` + the call's label suffix" — reads as the *conceptual* family name but post-rename the struct field is `FamilyObject::selector`. Ambiguous (conceptual name vs field ref inside a spec quote); left as-is to avoid diverging from the spec quote. If touched, reconcile the prose with the renamed field or re-point to the spec's own wording. Non-blocking, `cargo doc` clean (plain text, not an intra-doc link). |
| `phalcom-core/src/class.rs:17` (`MethodsMap`) + `phalcom-core/src/vm.rs:1573` (`Invoke` lookup) | performance — **deferred to v0.3** (Wren-analysis 2026-07-13) | Every send resolves through an `IndexMap<Symbol,ObjRef>` **hash** lookup walked per superclass level (`lookup_method_in_hierarchy`, class.rs:65). Wren's O(1) dispatch replaces the hash with a dense selector-indexed array (`class->methods.data[symbol]`), but Phalcom needs a **separate selector-only interner first** — `Symbol(u32)` (interner.rs:10) is a global *mixed* space (vars/fields/selectors), so a raw-`Symbol`-indexed per-class row is massively sparse. Two designs: **(A) flatten** own+inherited into each class array (single index, but re-flatten storm on class reopen / U-CORE-3 conditional re-parent — fights Phalcom's dynamic object model); **(B) per-class own-method arrays + chain-walk** (removes the per-level hash, keeps the walk, no flatten invalidation — **preferred**). A **monomorphic inline cache** keyed `(class_id, SelectorId)` with guard-and-refill is the better-fitting alternative for Phalcom's dynamism (self-heals on class mutation; spec already reserves the `IC ->` slot at vm.rs:1578) and stacks on top of (B). Defer the whole cluster; do not adopt (A). |
| `phalcom-core/src/vm.rs` run loop (`Bytecode` dispatch) + `phalcom-core/src/bytecode.rs` | performance — **deferred to v0.3** (Wren-analysis 2026-07-13) | Operand-free superinstructions for the most common local/field reads — `LOAD_LOCAL_0..15` and `LOAD_FIELD_THIS` (Wren `wren_vm.c:925-950` uses 0..8; **use 0..15 for Phalcom**, user 2026-07-13). Folds the operand fetch into the opcode for hot `GetLocal`/receiver-field reads. Additive opcodes; needs a `bytecode.rs` opcode-budget check (u8 = 256 slots). Micro-win, no object-model coupling. |
| `phalcom-core/src/vm.rs:620` `call_method` (`Primitive` arm) | performance — **deferred to v0.3** (Wren-analysis 2026-07-13) | Wren primitives write `args[0]` in place and return `bool`, so a hit drops `stackTop` by `numArgs-1` with **no `CallFrame` push** (`wren_primitive.h:38-44`, `wren_vm.c:1048`). Verify Phalcom's `Primitive` arm at `call_method` (vm.rs:620): if it pushes a frame for native primitives, adopt the in-place stack-window write (`&mut [Value]`) to skip the frame push on the hot primitive path. |
| `phalcom-core/src/primitive/map.rs` `map_raw_has`/`map_raw_remove` (found porting `wren/test/core/map/*.wren`, out of that unit's write-set — golden-corpus-only) | correctness — consistency gap | `map_raw_put`'s mutable-collection-key guard (`is_mutable_collection_key`/`mutable_key_error`, module doc's "re-entrant key-hash crux") only fires on `rawPut` — `Map#includes(_)`/`Map#remove(_)` on a `List` (or other mutable-collection) key silently `locate()` via identity `hash` instead of raising `Map key must be immutable`, i.e. `{}.includes(List.new())` returns `false` and `{}.remove(List.new())` is a silent no-op rather than either erring like `at(_, put:)` does. Wren's `containsKey`/`remove` both reject a non-value-type key uniformly (`test/core/map/contains_key_not_value.wren`, `remove_key_not_value.wren`); Phalcom's guard is one-sided. Fix: route `map_raw_has`/`map_raw_remove` through the same `is_mutable_collection_key` check as `map_raw_put` (or, if the asymmetry is intentional — a miss/no-op is arguably harmless since no bucket entry is ever written — document it explicitly in map.rs's module doc instead of leaving it implicit). |

| `docs/spec/v0.2/core/floor-census.md:653` (§7 audit-hook count) | docs-drift | Pre-existing, unrelated to this unit: the prose said "count = 113" while `tests/invariants.rs`'s own `BASELINE + NEW... ` sum was already 115 before U-ANNOT-CONTRACTS's `+2` (U-SCHED's `+2` landed without updating this line). Bumped straight to the correct **117** (115 + this unit's own `+2`) rather than first restoring the intermediate 115, since the live test is the source of truth and the intermediate value has no reader. |
| ~~`phalcom-lsp/src/selectors.rs`, `phalcom-lsp/src/index.rs`, `phalcom-lsp/src/completion.rs` — `ClassMember::Field` cross-unit landing-order hazard~~ **RESOLVED at merge time** | correctness | Stage 4's implementer worked in a stale worktree where `ClassMember::Field` didn't yet exist on `phalcom-ast` and removed the (then-dangling) `Field` match arms to get a green build. By the time its diff was merged into `main`, a concurrent session had landed U-ANNOT-LAYOUT's `ClassMember::Field`/`FieldDef` on `phalcom-ast`, so the removal became a regression (non-exhaustive match, red build). Fixed during merge: re-added `field_selector` (`selectors.rs`, bare-name shape mirroring `getter_selector`), `ClassMember::Field(_) => MemberKind::Getter` (`index.rs`'s `member_kind`), the index-collection arm (`index.rs`'s `Collector::walk_class`, pushing `f.range` and walking `f.default`), and the completion scan arm (`completion.rs`, scanning `f.default`). Green again; no further action needed. |
| ~~`phalcom-lsp/src/hover.rs` `parse_tags`/`parse_doc_block` (U-LSP Stage 4)~~ **RESOLVED** | fidelity vs. doc-comments-phaldoc.md §3/§5 | Tag-continuation-line folding now compares each candidate continuation line's own indentation (`indent_width`, measured on the already marker-stripped body line) against the `@tag` line's indentation, per §3 — a same-or-lesser-indented line ends the payload without folding. `hover.rs::parse_tags` + tests `phaldoc_indented_continuation_line_folds_into_the_tag`/`phaldoc_non_indented_following_line_does_not_fold_into_the_tag`. |
| ~~`phalcom-lsp/src/hover.rs` `harvest_doc_for_selector` (U-LSP Stage 4)~~ **RESOLVED** | scope — top-level bindings not covered | `harvest_doc_for_selector`'s adjacency resolution now also tries `top_level_binding_name_at_line` (a top-level `Statement::Let`'s non-destructuring `Pattern::Name`) when `member_selector_at_line` finds no `ClassMember`, so a `///` block above a top-level `let`/`var` attaches by the bound name. `backend.rs`'s `hover_at` additionally falls back to `hover_for_top_level_binding` (via the new `index::top_level_binding_at_offset`) when the cursor is on neither a keyword nor a selector, so hovering a *usage* of the bound name (not just its declaration) surfaces the doc too. Tests: `hover.rs`'s `phaldoc_attaches_to_a_top_level_let_binding`/`_var_binding`, `index.rs`'s `top_level_binding_at_offset_*`, and the `stage4_hover.rs` integration test `hover_over_a_top_level_binding_usage_surfaces_its_doc`. |
| ~~`phalcom-lsp/src/backend.rs` `hover_at` selector branch~~ **RESOLVED** | UX — no `Hover.range` for selector hovers | `index::selector_at_offset` now returns `(String, SourceRange)` instead of just `String`, so `Backend::selector_at_position` converts that span through the document's `LineIndex` and `hover_at`'s selector branch sets `Hover.range` from it, matching the keyword branch. `goto_definition`/`references` (the other two `selector_at_position` callers) simply discard the span. Test: `stage4_hover.rs`'s `selector_hover_sets_range_to_the_resolved_selector_span`. |

| `phalcom-ast/src/parser.rs:1047-1065` `parse_attribute_arg_list` (M-ATTR-ROOT) | feature scope, forced deviation | **Attribute-arg lists are positional-only — no labeled args.** `parse_attribute_arg_list` calls `parse_expr()` per argument (`Attribute::args: Vec<Expr>`, not `Vec<Argument>`), so `attribute-classes.md`'s spec examples (`@On(Method, tier: Install)`, `@Author(name: "Ada")`) cannot parse as written. M-ATTR-ROOT's own `On` class and goldens use the positional form instead (`On.new(target)` / `On.new(target, tier)`; tier detected at compile time in `compiler::attributes::validate_attribute_class` by matching a bare `Var` argument's name against the five tier names, not a labeled `tier:` argument). Fix: extend `parse_attribute_arg_list` to accept `name: expr` labeled entries (mirroring ordinary call-site argument grammar) once a unit owns the parser change; then `On`'s constructors and any attribute-class consumer can move to the labeled form the spec actually describes, and `inherited:` (dropped entirely for now) can be added back. |

## Numbered backlog (merged from phase-next)

_Merged verbatim 2026-07-15 from the former `docs/forge/phase-next/DEFERRED.md` (deleted in
the same pass). **Numbers are frozen** — ~18 docs under `units/` cite them (`DEFERRED #30`, `#19`,
`#9`, …). Never renumber; never reuse a number. Append new numbered entries at #34+._

_**Partial triage, 2026-07-15.** Entries marked **RESOLVED** below were confirmed against
[STATE.md](STATE.md)'s landing record or `docs/adr/STATUS.md` and carry their evidence.
Every other entry is **unverified** — it was filed 2026-07-11/12 and has not been re-checked
against the tree since. Unverified ≠ live. Do not act on one without re-grounding it first._

| # | Idea | Source | Spec/ADR | Rank |
|---|------|--------|----------|------|
| 1 | `SyntaxErrorKind::InvalidInteger`/`InvalidFloat` lower to a zero-width `0..0` range, losing the offending literal's span in diagnostics. Carry the real span through `LexicalError` instead. | `phalcom-ast/src/parser.rs` (`lex_error_to_syntax`) | ADR-0016 | low |
| 2 | The hand-written parser accepts a few malformed assignment targets (e.g. `a+b = c`, `(a+b) = c`) that LALRPOP rejected at parse time; they are still caught by the compiler as invalid assignment targets, but could be rejected earlier with a precise diagnostic. | `phalcom-ast/src/parser.rs` (`parse_assignment`) | ADR-0016 | low |
| 3 | Pre-existing `clippy::extra_unused_lifetimes` warning: `format_num_arguments<'a>` declares an unused lifetime. Drop the `<'a>`. | `phalcom-core/src/error.rs:30` | — | low |
| 4 | ~~**F4 (`object_name` / instance `toString`, ADR-0015) needs a home unit.**~~ **RESOLVED — U-CORE-4 (`2061795`)** landed per-type `toString` + the unified native print path (ADR-0036 Accepted, code-confirmed `primitive/number.rs:88`). | U2 architect | ADR-0015 | done |
| 5 | ~~**Kernel `List` + collections.**~~ **RESOLVED** — `List` half landed as U-LIST (ADR-0019/0020 both Accepted); `Map`/`Set`/`Tuple`/`Range` landed as U-COLLTYPES (see #27). DEC-A closed. | U8/U9/U-STD | ADR-0020 | done |
| 6 | ~~Collection-literal lowering `(a,b)`/`[…]`/`{a:1}`.~~ **RESOLVED — U-COLL** (`1274504` list / `5bc31e8` tuple / `dc9eab0` map), ADR-0029 + ADR-0032 Accepted. Folds #28. | U-LEX | ADR-0029/0032 | done |
| 7 | ~~Reflection surface: `Method.bind(_)`/`invokeOn(_,_)`/`methodFor(_)`, `Function`/`Block`/`Method` reflection.~~ **RESOLVED — U-CORE-3 (`10ebd06`)**, ADR-0028 Accepted + code-confirmed (`primitive/method.rs`, `primitive/block.rs`). | U4/U-STD | functions.md §3 | done |
| 8 | Per-class dNU handler cache (keyed on `ClassId`, gated by open-Q4); spread call sites `f(*args)`. **Split**: the dNU-cache half duplicates #22; the spread half duplicates #21. Both still open. | U8 | open-Q4 | med |
| 9 | **Block variadics `{ *xs => }` — confirmed still out of scope (U9, 2026-07-12).** No `{ *xs => }` grammar exists; `parse_param_list` (shared by method/constructor lists but *not* block-literal params, which use a separate ad hoc scanner in `Parser::parse_primary`) never reaches block-literal parsing, so this doesn't even parse today — no silent-misbehavior risk, nothing to explicitly reject. Block variadics would need zero extra VM plumbing once the grammar exists (the same call-prologue collapse handles any closure); the grammar itself is unbuilt. `callWith(_:List)` remains unimplemented; no-op. | U4/U9 | functions.md §2 | low |
| 10 | ~~`for (x in xs)` runtime; derived control selectors in `core.ph`.~~ **RESOLVED — U-ITER** (for/break/continue + cursor protocol, ADR-0035 amended by ADR-0048, both Accepted). Per-call-site polymorphic IC remains open → see the `MethodsMap`/`Invoke` entry in [Open entries](#open-entries). | U5 | control-flow.md | done |
| 11 | ~~Concurrency runtime: `Fiber`/`Future`/`Error` classes.~~ **RESOLVED** — `Error` root U-CORE-6 (`85c4e1d`, ADR-0037); `Fiber` U-FIBER (ADR-0030 Accepted); `Future` Slice A U-FUTURE-A (`f0d128a`). Slice B (`async`/`await`) landed as U-SCHED (`34246a8`). | U-STD | concurrency.md | done |
| 12 | Lexer polish: nested block comments; lone-`?` ternary; carry real span through `LexicalError` (dup of #1). Block comments shipped flat (non-nesting) per U-LEX; nesting + lone-`?` still open. Folds #32. | U-LEX | ADR-0016 | low |
| 13 | **Reassignment of a *captured* immutable binding is not rejected.** An outer binding reached through an upvalue from inside a block: the compiler only enforces immutability for a current-function local and a module global. An inner-block `count = count + 1` over an outer immutable `count` compiles to `SetUpvalue` with **no diagnostic**. Extend the assignment path to walk enclosing function-states. **NB:** filed against ADR-0014's `let`(immutable)/`var`(mutable) spelling; [ADR-0064](../adr/accepted/0064-let-const-bindings-and-field-mutability.md) supersedes that spelling (`let`=mutable, `const`=immutable) but keeps every rule, so the hole is unchanged — it is now a hole in `const`. Re-verify against U-BINDINGS when it lands. | `phalcom-core/src/compiler/lib.rs` (`Expr::Assignment`, upvalue branch) | ADR-0014 → ADR-0064 | med |
| 14 | The `if(opt)` truthiness compile check (`CompilerError::OptionTruthiness`) is literal-only: it catches `None` and `Some.new(...)` as a condition, but not an Option-typed *variable* (that stays a runtime type error via the branch opcode's `Bool` requirement). No span attached. **This is the knowingly-accepted gap in [ADR-0021](../adr/accepted/0021-no-truthiness-enforcement.md), not a bug** — closing it needs flow analysis Phalcom does not have. Keep as a record. | `phalcom-core/src/compiler/lib.rs` (`is_option_literal`) | ADR-0007/0021 | low |
| 15 | Fixed slot layout + private-non-inherited fields (ADR-0011) forecloses (a) adding a field to a *live* class / `become:`-style reshape (offsets frozen at class-definition time) and (b) shared *protected* inherited fields (a subclass must go through accessors). Both deliberate per ADR-0011 — good for a future inline cache (stable offsets) — but flag if either is ever wanted; a cross-cutting reshape, not a local change. **Informational, not actionable.** | U7-plan §3 (rubric preclusion) | ADR-0011 | low |
| 16 | ~~The `Counter.new()` → `construct` selector redirect is a compile-time, same-compilation-unit, literal-receiver heuristic; an indirect receiver (`let C = Counter; C.new()`) silently reaches the bare allocator.~~ **DISSOLVED BY RULING, not fixed** — [ADR-0063](../adr/accepted/0063-constructors-are-ordinary-class-side-methods.md) §7 rules that `new()` is an ordinary inherited method, so the "fall through to the bare allocator" this entry treats as a bug is **specified behavior**. See the `has_new_construct` row in [Open entries](#open-entries) for the full ruling and what survives it (ADR-0063 §6.1's `native_repr` gate). | `phalcom-core/src/compiler/lib.rs` (`Expr::MethodCall`) | ADR-0011 → ADR-0063 | done |
| 17 | **`Statement::Class` unconditionally emits `DefineGlobal` at the end of every class body, reopen or not.** Harmless for every core class whose global already points at that class object, but `None`'s global is deliberately bound to the shared singleton *instance*, not the class (`VM::install_core`) — so a `class None { ... }` reopen silently clobbers that binding back to the class the moment `core.ph` runs, breaking every `x == None` downstream. U-LIST worked around it by dropping the empty `class None {}` skeleton from `core.ph`. **Whoever next needs real members on `None` must fix this compiler special case first** — e.g. skip `DefineGlobal` when reopening a class whose current global binding is not that same class object. ~~**Unverified since 2026-07-11**~~ **REPRODUCED LIVE 2026-07-19** — that caveat is discharged; the line number above still predates U-REOPEN-FIX (`e85f31a`) and should not be trusted. **OWNED BY [`U-CLASSCLOSE`](units/U-CLASSCLOSE/plan.md) §3.5** ([PDR-0001](../decisions/0001-classes-are-closed.md)): the guard lands there because that unit already edits this lowering path. Pulled out of U-BINDINGS scope (see its §12C). **This entry does not close** — under 0065 the defect *dissolves as a user-reachable bug* (ruling 3 reserves kernel names, so nobody outside core can write `class None`) but *survives as a bootstrap task*: `vm/bootstrap.rs:262-265` inserts the `None` class row expressly so `core.ph` **can** complete that stub, and ruling 4 sanctions exactly that — the guard is the prerequisite. Giving `None` a body, and #35's sealing unification downstream of it, are ruled out of scope: [`docs/deferred/class-sealing-followups.md`](../deferred/class-sealing-followups.md) item 3. **Mis-test warning:** `x == None` reads `true` either way (both sides read the clobbered binding); use a genuinely produced `None` — `Some.new(5).filter { x => false } == None` is `true` before, `false` after. `isNone` answers correctly throughout; only the binding moves. | `phalcom-core/src/compiler/lib.rs` (`Statement::Class`, ~L862 — stale) | — | high |
| 18 | `List::rawSet` (indexed write) is wired as a primitive but has no `.ph` wrapper — no `at(_,put:)` selector. **Still open, and the obvious resolution is the wrong one.** ADR-0055 (`xs[i] = v` desugars to `at(_, put:)`) would have entailed the selector — but **ADR-0055 is Retired**, reversed by **[ADR-0060](../adr/accepted/0060-index-operator-as-real-selector.md) (Accepted, shipped as U-INDEX)**: `[]` is its own dedicated, user-overridable selector and **does not lower to `at`** (`core.ph:812,879`). So indexed write is served by `[]`, and whether a *separate* `at(_,put:)` should also exist is now an open design question, not a bookkeeping gap. `Tuple` deliberately has neither (`core.ph:1015`). | U-LIST-plan §3/§4 | ADR-0020; ADR-0055 **retired** → ADR-0060 | low |
| 19 | **`List.toString` is a native primitive (`primitive/list.rs::list_to_string`), not `.ph`-defined over `each(_)`** — because when filed, no kernel value type had a general user-callable `toString`, so a `.ph` `List.toString` would render every non-`String` element wrong. **Precondition now met** — U-CORE-4 (`2061795`) landed real content `toString` — so the `.ph` move is unblocked but was never done. Same root cause as CB-1/#30. | U-LIST return contract | ADR-0019/0020 | med |
| 20 | ~~`map`/`reduce`/`filter`/`inject` and other collection combinators not defined on `List`.~~ **RESOLVED** — U-STD landed the pure-`.ph` combinator layer; U-ITERABLE then rehomed `each`/`map`/`filter`/`reduce` onto the kernel `Iterable` root (`core.ph:309`), driven by `iterate(_)`/`iteratorValue(_)` per ADR-0048. | U-LIST-plan §3/§8 | ADR-0020/0048 | done |
| 21 | **`Bytecode::SendDynamic` opcode not built.** U8 delivered `VM::send_dynamic` (the Rust helper) behind `perform`/dNU, but not the opcode — no call-site spread syntax exists to emit it. U9 explicitly scoped both the opcode and `f(*args)` out. Spread-call syntax remains a future unit's job. Folds the spread half of #8. See also the `parse_comma_exprs` leading-`*` row in [Open entries](#open-entries). | `phalcom-core/src/bytecode.rs`; `vm.rs::send_dynamic` | messages-and-selectors.md §5; ADR-0012 | med |
| 22 | **Per-class dNU handler cache** (method-lookup.md §2, optional; folds prior #8). Deliberately not built — correctness-first, the miss path is slow-by-design. If ever added: key on stable `ClassId`, keep **separate** from the call-site IC, invalidate on hierarchy mutation. **NB:** ADR-0026/0041 have since *sealed* superclass reparenting, so the invalidation story is narrower than when this was filed — only method redefinition (ADR-0018's override epoch) applies. The `dispatch_dnu_preserves_dispatch` golden guards a future IC against regression. | `phalcom-core/src/value.rs::lookup_method`; `vm.rs` Invoke arm | method-lookup.md §2 | low |
| 23 | **`perform(_,_)` selector/arity match is not pre-validated.** `Object.perform(selector, argsList)` trusts the selector symbol's encoded arity to agree with `argsList.size`; a mismatch surfaces only through ordinary lookup (likely a miss → dNU) rather than an eager, targeted `ArgumentError`. **NB:** filed as "once `#` selector literals land (U-LEX) a clearer diagnostic would improve DX" — **U-LEX-HASH landed** (`fac45ae`), so that precondition is met. | `phalcom-core/src/primitive/object.rs::object_perform_with` | messages-and-selectors.md §5 | low |
| 24 | **Duplicate variadic selector per class silently overwins in the method map.** Two variadic methods with the same bare name in one class body collide on the identical `<name>(*)` selector symbol; the second silently replaces the first — same as any duplicate-selector redefinition today, not new. A clean "duplicate method" diagnostic (for this or the general case) would improve DX. | `phalcom-core/src/class.rs` (`ClassObject.methods`) | messages-and-selectors.md §4 | low |
| 25 | ~~`blocks/pending/blocks_argument_to_method.ph` blocked on `List.reduce(_)`.~~ **RESOLVED (U-STD, 2026-07-12)** — `reduce` landed; fixture rewritten and promoted out of `pending/`. | `phalcom-core/tests/lang/blocks/blocks_argument_to_method.ph` | blocks.md | done |
| 26 | ~~Pre-existing `cargo doc` warning: `nil.rs` `some_new` links to private `wrap_some`.~~ **RESOLVED (U-ERR pass)** — repointed off the intra-doc link, `cargo doc` clean. Dup of #33. | `phalcom-core/src/primitive/nil.rs:64` | docs guidelines | done |
| 27 | ~~Collection classes `Map`/`Set`/`Tuple`/`Range` deferred; `Map`/`Set` block on `Object#hash`.~~ **RESOLVED** — `Object#hash` landed U-CORE-1 (`03764e3`, ADR-0023); the four classes landed as U-COLLTYPES (`be8426e` Map+Set / `2d140f0` Tuple / `f934cf1` Range) as native arena arms under ADR-0032/0039. | `phalcom-core/core/core.ph`; `phalcom-core/src/universe.rs` | ADR-0020/0032/0039 | done |
| 28 | ~~List-literal syntax `[a, b, c]` (and map/set/tuple literals) not built.~~ **RESOLVED — U-COLL** (ADR-0029/0032). `[a,b,c]` and `{k: v}` are active; `#{…}` (Set) and `..`/`...` (Range) remain **reserved-but-inactive by decision**, not deferred — see the `#{`/`..` lexer row in [Open entries](#open-entries). Folds #6/#20. | `phalcom-ast/src/{lexer,parser}.rs` | ADR-0029/0032 | done |
| 29 | ~~Scope-taxonomy divergence between the forge scheduling docs and `docs/spec/core/` on U-STD's ownership.~~ **RESOLVED (2026-07-12)** — U-STD scope settled via Option (B); the roster reconciliation landed as PHASE2-INDEX §7 pointing at `docs/spec/v0.2/core/README.md` as index of record rather than forking it. | `docs/forge/PHASE2-INDEX.md` §7 | — | done |
| 30 | **String-interpolation desugar targets `String.new(_)`, not content `toString`.** → **SUPERSEDED by [CB-1](#cb-1--string-interpolation-bypasses-tostring-overrides)**, which verified it against the tree on 2026-07-15, established that this entry's stated blocker (U-CORE-4) has landed, and added the security dimension. **Entry kept for its number only** — ~4 docs cite `DEFERRED #30`. Read CB-1, not this row. | `phalcom-ast/src/parser.rs` (`desugar_string_interp`) | ADR-0022 | **see CB-1** |
| 31 | **Interpolation `\(…)` scanning is balanced-paren only — it does not understand a string literal nested inside the expression.** `lexer.rs::scan_string` counts `(`/`)` depth to find the end of a `\(expr)` body, so a `)` inside a nested string literal (`"\(f(")"))"`) mis-terminates the expression. Accepted for v1; a full fix would re-enter string-scanning recursively inside the interpolation body. | `phalcom-ast/src/lexer.rs` (`scan_string`) | ADR-0022 | low |
| 32 | **Nested block comments + lone-`?` ternary still deferred (already #12).** U-LEX shipped flat (non-nesting) `/* … */`; nesting and the reserved lone-`?` remain. Not a separate item — noted only so U-LEX's tail is traceable. | `phalcom-ast/src/lexer.rs` (`skip_trivia`) | DEFERRED #12; ADR-0016 | low |
| 33 | ~~Pre-existing rustdoc warnings in `primitive/nil.rs` (`some_new` links to private `wrap_some`).~~ **RESOLVED (U-ERR pass)** — dup of #26. | `phalcom-core/src/primitive/nil.rs:64` | doc-clean | done |
| 35 | **Spike: unify the sealing representation (S-1 option B).** User-ruled 2026-07-15 as the follow-up to [CB-3](#cb-3--sealing-is-one-property-with-two-representations-that-can-disagree--fixed-2026-07-15)'s option-A fix. Goal: make the **attribute list the single source** and `VM::sealed_classes` derived from it, so the union-read in `attributes.rs` can collapse back to one source. Two unknowns to answer **before** any code: **(1)** `Option` (`core.ph:474`) and `Some` (`core.ph:544`) already have `.ph` reopens and could carry `@sealed` today — **`None` has no reopen**, which is the blocker `vm/bootstrap.rs:209` actually names. Can `None` get a reopen, or a native attach, without disturbing ADR-0044's `Option` bootstrap (`Nil`→`None` surfacing runs *during* bootstrap, before `.ph` decorators)? **(2)** Ownership: bootstrap writes `sealed_classes[Option] = bootstrap module`; a `@sealed` on core.ph's reopen writes core.ph's module. Same key, different value, last-writer-wins — and the `extends` check compares `sealed_in_module != self.module`, so this changes who may subclass. Answer both, then decide. **Not** B-wide (`@sealed class Option { @variant Some(v); @variant None }`) — that needs its own ADR: `@variant` generates `@data` classes with mutable fields, but `None` must stay a **zero-allocation singleton** (ADR-0044), and `Option#match(some:none:)` is already hand-rolled native (`universe/primitives.rs:191-198`) precisely because it is the eliminator `@variant` would generate. Note the perf motive routes elsewhere: `None` already allocates nothing; `Some`'s allocation is removed by **niche encoding**, deferred by ADR-0044 DEC-U17, orthogonal to sealing. | `compiler/attributes.rs` (`ExpandCtx::sealed_classes`) · `vm/bootstrap.rs:209-261` · `core.ph:474,544` | ADR-0044 · ADR-0007 | med |
| 34 | ~~**Write the five missing floor-census amendment banners.**~~ **DONE 2026-07-15** — all five written (U-SCHED +2/ADR-0030, U-ANNOT-CONTRACTS +2/ADR-0052, M-ATTR-ROOT +3/no ADR, U-GC +1/ADR-0050, U-STRING +4/ADR-0049), each naming its selectors, native fns, side, and fn-count delta; the chain now runs 73 → … → **125** and its fn chain 57 → … → **110**, both closing on the test's constants. The stale `Baseline:` line (frozen at post-U15/112) and the landing-history list were re-derived, and `core/README.md`'s "single source of truth for the baseline pin" section — which had sat at post-U-ERR/111 through all five — now distinguishes the *pin* (its job) from the *count* (the test's). **Found while writing:** M-ATTR-ROOT's three bindings cite `attribute-classes.md` as their spec, in both the banner and the test's `NEW_ATTR_ROOT` comment — **that file does not exist**, so those three have no spec outside the census and the code. Recorded in §1.3's M-ATTR-ROOT banner; not fixed. | `docs/spec/v0.2/core/floor-census.md` §1.1 | ADR-0019 | done |

_Closed pre-merge:_ #(ex-LALRPOP) — done in U1: dead `CompilerError::ParseError` variant +
`From<lalrpop_util::ParseError>` impl deleted (slice 3), `lalrpop-util` dropped from
`phalcom-core/Cargo.toml` + `Cargo.lock`.

## Homed entries

Every other deferral has been homed in its owning unit's plan — each carries an
**Adopted debt** note in its write-set section:

| Debt | Owning unit |
|---|---|
| `primitive/number.rs:~34` — type-error message hardcodes `"value"` | [U12](units/U12/plan.md) §3 |

**Landed this pass (U-ERR):** `primitive/nil.rs:~64` broken rustdoc link →
private `wrap_some` (repointed off the intra-doc link, `cargo doc` clean) and
`core/README.md`'s stale floor baseline (was 88/pre-U-COLLTYPES, now
re-baselined to 111/post-U-ERR) — both fixed in the same change as this
unit's own `+2` census bump.

Add a new entry here **only** when a debt has no plausible owning unit; otherwise
fold it into the relevant `units/<U>/plan.md` write-set as an **Adopted debt** note.
