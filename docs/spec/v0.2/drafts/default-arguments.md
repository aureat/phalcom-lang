# Default arguments — an exploration of a rejected feature

- Status: **Draft — exploration of a REJECTED feature.** Default arguments are
  rejected for v0.2 by
  [ADR-0043](../../../adr/accepted/0043-no-default-arguments-keep-selector-identity-pristine.md)
  (Accepted). This document does **not** propose re-adding them. It exists to
  record what the feature would buy, what it would cost, and what a future
  superseding ADR would have to prove. **Do not cite it as a plan.**
- Date: 2026-07-15
- Governed by:
  [ADR-0043](../../../adr/accepted/0043-no-default-arguments-keep-selector-identity-pristine.md) (the ruling) ·
  [ADR-0012](../../../adr/accepted/0012-selector-signature-encoding-and-dispatch.md) (selector identity = the dispatch key) ·
  [ADR-0025](../../../adr/accepted/0025-external-internal-parameter-names.md) (labels vs internal names) ·
  [open-questions.md](../open-questions.md) Q12 (the *mechanism* ruling — narrower and more specific than ADR-0043)
- Supersedes: `experimental/default-arguments.md`, **retired 2026-07-15** (DEFERRED CB-4).
  That doc was 40 lines, `Status: Proposed`, and advocated in its body the exact
  mechanism its own banner declared permanently forbidden. This doc absorbs it; see
  [§8](#8-supersedes-experimentaldefault-argumentsmd-retired). Read §5 for the mechanism
  question it tried to answer.

## 1. The status of default arguments in Phalcom

**They do not exist. They are not planned. They were ruled out, deliberately, on
2026-07-12.**

| Fact | Source |
|---|---|
| ADR-0043 "No default arguments" is **Accepted**, ✅ shipped | [`docs/adr/STATUS.md:73`](../../../adr/STATUS.md) |
| A method's arity is **fixed**, and signature identity is **1:1** with it | ADR-0043 §Decision |
| `open-Q12` is **RESOLVED — by declining the feature** | [open-questions.md:214](../open-questions.md), item 12 |
| The idiom for "optional" params is **manual arity-overload** — `foo`, `foo(_)`, `foo(_,_)` as separate methods | ADR-0043 §Decision |
| DEC-U18 → **A**, landed as an affirm-ADR unit (`f16b58a`), **no runtime change** | [`docs/forge/STATE.md:281-282`](../../../forge/STATE.md) |

The unit that asked the question (U18) shipped documentation and nothing else.
No opcode, no method-dict entry, no parser rule ever existed for this feature.
The tree at HEAD has no notion of a defaulted parameter: `Signature` carries
`{ selector, kind, positional_arity, variadic }` and nothing more
([`phalcom-core/src/method/mod.rs:49-61`](../../../../phalcom-core/src/method/mod.rs)).

`open-Q12` additionally fixed the **mechanism** for any hypothetical future add
(see [§5](#5-designs-a-superseding-adr-could-take)). That is a constraint on a
feature that does not exist — not a roadmap entry.

## 2. Why rejected — the identity-dispatch ⊗ optional arity hazard

This is the textbook instance of a two-feature collision, and it is worth stating
precisely rather than by slogan.

**Premise 1 — identity.** A Phalcom selector is an interned string encoding
`name + labels + kind`, built by the single source-of-truth encoder
`encode_selector(name, labels, kind)`
([`phalcom-core/src/method/mod.rs:102-118`](../../../../phalcom-core/src/method/mod.rs), ADR-0012).
Arity is spelled *into* the name: `foo()`, `foo(_)`, `foo(_,_)` are three
distinct strings, three distinct `Symbol`s, three distinct methods. This is not
an accident of encoding — it is what makes arity-overloading work at all, and
it is exercised as a golden fixture
([`phalcom-core/tests/lang/dispatch/dispatch_arity_zero_one_two.ph`](../../../../phalcom-core/tests/lang/dispatch/dispatch_arity_zero_one_two.ph)).

**Premise 2 — lookup.** The method dictionary is keyed by that symbol, and
lookup is **one hashmap probe** per class in the superclass chain
([`lookup_method_in_hierarchy`, `phalcom-core/src/heap/class.rs:74-85`](../../../../phalcom-core/src/heap/class.rs)),
fronted by a monomorphic inline cache keyed on `(call_site, class_id)`
([`phalcom-core/src/vm/dispatch.rs:415-425`](../../../../phalcom-core/src/vm/dispatch.rs)).
The call site knows the selector as a *compile-time constant* in the chunk. It
does not know the receiver's class.

**The collision.** Anything that varies **effective arity** — a default
argument, an optional trailing argument, implicit-`self` elision — makes the
call site emit a *different selector symbol* than the one the method installed
under. `foo(1)` interns `foo(_)`. A method declared `foo(a, b = 0)` installs
`foo(_,_)`. The probe for `foo(_)` misses. It does not "find the method and
notice an argument is absent" — there is no shared entry to find. It falls
through to `doesNotUnderstand(_)`
([`phalcom-core/src/vm/dispatch.rs:456`](../../../../phalcom-core/src/vm/dispatch.rs)).

Both resolutions are bad, in different ways:

- **Arity-family expansion at install time** — register `foo(_)` *and*
  `foo(_,_)`, synthesizing a forwarder for the short form. In the general case
  (defaults at arbitrary positions) this is **combinatorial**: `k` defaulted
  params → up to `2^k` selectors. Restricted to trailing-only it collapses to
  linear, `k+1` (this refinement is Q12's, not ADR-0043's — see
  [§8](#8-reconciliation-with-experimentaldefault-argumentsmd)). Either way it
  **silently populates the method dictionary with entries the author never
  wrote**, which is directly observable: `Behavior::methods` returns the raw
  keys of `ClassObject.methods`
  ([`phalcom-core/src/primitive/class.rs:68-86`](../../../../phalcom-core/src/primitive/class.rs)),
  and `Object::respondsTo(_)`
  ([`phalcom-core/src/primitive/object.rs:169`](../../../../phalcom-core/src/primitive/object.rs))
  would answer `true` for a selector with no source.
- **Static callee knowledge at the call site** — unavailable. Phalcom is
  dynamically dispatched; the compiler emits `Invoke(selector_const)` without
  any claim about the receiver's class (ADR-0012 §Decision).

**The real lesson is timing, not defaults.** Selector identity is *load-bearing*
across the compiler, the interner, the IC, `perform`, `SEND_DYNAMIC`, the dNU
reification path, and the `Family` router (ADR-0047) — each reads or builds a
selector under the same 1:1 assumption. Adding defaults **before** identity
becomes load-bearing costs a design tax; adding them **after** costs a migration
across every consumer of the invariant. Phalcom ruled the day the question was
asked (DEC-U18) and got the ordering right.

## 3. The convention that exists instead

The convention is **manual arity-overload, made self-documenting by ADR-0025
labels**. It is not a workaround bolted on after the rejection; it is the thing
the rejection preserves.

A family reads as named shapes rather than one shape with holes:

```phalcom
class Text {
  slice(from:)      { return self.slice(from: from, to: self.count) }
  slice(from:, to:) { /* the real implementation */ }
}
```

Those intern as `slice(from)` and `slice(from,to)`
(`comma_form_slots`, [`method/mod.rs:121-124`](../../../../phalcom-core/src/method/mod.rs)) —
two selectors, two methods, one probe each. Compare the defaulted spelling
`slice(from, to = end)`: the call site `slice(from: 3)` tells the reader
nothing about what `to` becomes, and the reader must open the declaration. The
labeled family says it at the call site.

ADR-0025 sharpens this: the **external label** is selector identity, the
**internal binding** is a frame slot, and they may differ — so the word that
reads well at the call site and the word that reads well in the body can both
win:

```phalcom
class Point {
  move(to target:) { _position = target }   // label `to`, binding `target`
}
p.move(to: origin)
```

(Verified spelling — ADR-0025 §Decision, and the labeled-selector fixtures
[`messages/messages_labeled_selector_identity.ph`](../../../../phalcom-core/tests/lang/messages/messages_labeled_selector_identity.ph)
and
[`classes/class_labeled_arg_method_definition.ph`](../../../../phalcom-core/tests/lang/classes/class_labeled_arg_method_definition.ph).)

**The convention is live in the core library.** `String#trim` is exactly a
defaultable parameter written as a family
([`phalcom-core/core/core.ph:260-270`](../../../../phalcom-core/core/core.ph)):

```phalcom
  trim()      { return self.trim(" \t\n\r") }
  trimStart() { return self.trimStart(" \t\n\r") }
  trimEnd()   { return self.trimEnd(" \t\n\r") }

  trim(chars) => self.trimStart(chars).trimEnd(chars)
```

and `List#join` is the getter-plus-arity-1 form of the same pattern
([`core.ph:743-745`](../../../../phalcom-core/core/core.ph)):

```phalcom
  join => self.join("")
  join(sep) { /* … */ }
```

**Swift landed in exactly this spot** — labels make a call site read as prose,
and the labeled family is one of Swift's most-liked traits. Be precise about what
that proves: **Swift also has default arguments**, so it is not a pure precedent
for doing without them. What it demonstrates is that *labels carry the
readability* — not that defaults are unnecessary. Swift affords both (see
[§6](#6-precedent-with-consequence)); Phalcom takes the half its dispatch model
permits.

## 4. The benefits — steelmanned

ADR-0043 concedes its own cost in one line ("Callers repeat arguments that a
default would elide, and library authors write multiple arity overloads"). That
concession is thinner than the real case. The real case:

**4.1 It kills mechanical source duplication.** `k` trailing optional params
require `k+1` hand-written forwarders today. The trim family above is **9 lines
of pure forwarding** to express "one optional charset param" three times, with
zero semantic content. The tax recurs at core-library scale: the largest
selector families in `core.ph` are `at` (9), `iteratorValue` (9), `iterate` (5),
`map`/`includes`/`each` (4 each).

**4.2 Single source of truth for the default *value*.** This is the strongest
argument and it has an in-tree witness. The literal `" \t\n\r"` appears **three
times** — [`core.ph:261`, `:264`, `:267`](../../../../phalcom-core/core/core.ph).
An overload family duplicates the default value once per forwarder, and
duplicated constants drift. A defaulted param states it once. Phalcom is
*already paying* this cost, today, in its own standard library.

**4.3 The default expression can see the callee's scope.** A default may depend
on `self` and on earlier parameters — `slice(from, to = self.count)` — evaluated
per call in the callee's frame. Swift and Python allow this; C++ restricts it
(no `this`, and only preceding params in some positions). Today's forwarder
achieves this manually, which is precisely why the forwarder exists rather than
a constant.

**4.4 API evolution.** Adding a trailing defaulted param is **source-compatible
for every existing caller**. Under the overload convention, adding an optional
param means writing a new overload *and* keeping the old one as a forwarder
forever — the family grows monotonically, and the author cannot delete the
narrow arity without a breaking change. For a library ecosystem this is a
large, recurring benefit, and it is the benefit that would matter most if
Phalcom ever grows a package registry.

**4.5 Discoverability.** One signature in documentation and in an LSP hover,
instead of `k+1` near-identical entries that the reader must diff to find which
one they want. `Behavior::methods` on a defaulted `trim` would show one
selector, not three.

None of this is disputed by ADR-0043. The ADR's position is that the *dispatch*
cost outweighs it at v0.2 scope — not that the benefits are illusory.

## 5. Designs a superseding ADR could take

ADR-0043 names the bar: **specify how default-argument methods register against
the signature-keyed dispatch table without regressing the single-probe lookup.**
Each candidate below is judged against that bar, and against what it forecloses.

### (a) Install-time arity-family expansion — *the ruled mechanism*

Definition-time desugar: `f(a, b = 0)` installs `f(_,_)` (the real body) **and**
`f(_)` (a synthesized forwarder calling `f(a, 0)`). Pure codegen over the
arity-overloading that already works.

- **vs the bar: PASSES.** Every installed selector is a real method dict entry;
  lookup is unchanged, one probe, IC-cacheable. No call-site machinery.
- **Combinatorics:** `2^k` in the general case; **`k+1` if defaults are
  restricted to trailing params** — which is exactly the restriction Q12
  ratified.
- **What it costs:** the method dict gains selectors with no source line.
  `Behavior::methods` ([`primitive/class.rs:84`](../../../../phalcom-core/src/primitive/class.rs))
  and `respondsTo(_)` report them; a debugger stepping into `f(1)` lands in
  synthesized code.
- **Worth recording:** the ADR's bar is *single-probe*, and (a) meets it — so
  the stated bar does not discriminate against the design the ADR calls
  "combinatorial". The combinatorics and the reflection pollution are a
  **second, unstated bar**. A superseding ADR should name it (say: *expansion is
  linear in the number of defaults, and synthesized selectors are
  distinguishable from authored ones through reflection*) rather than leave it
  implicit.

### (b) Call-site arity-fold

Probe `f(_)`; on miss, re-probe `f(_,_)` and fill defaults.

- **vs the bar: FAILS as stated** — it adds a miss to every defaulted call, on
  the hottest path in the VM.
- **But be honest: Phalcom already does exactly this for variadics.** The miss
  arm of `Invoke` derives `name(*)` and does a second `lookup_method` walk
  ([`dispatch.rs:437-456`](../../../../phalcom-core/src/vm/dispatch.rs),
  [messages-and-selectors.md §4](../messages-and-selectors.md)), memoised through
  `variadic_selector_cache` and absorbed by the IC on refill. "Single probe" is
  therefore already a *steady-state* claim, not a literal one, and a superseding
  ADR could argue defaults deserve the same treatment. The counter: variadics
  have **no** install-time expansion available (unbounded arity cannot be
  enumerated) whereas defaults do — so a fold buys nothing (a) does not.
- **What it precludes:** nothing structural; it spends hot-path budget.

### (c) Compile-time call-site rewrite

The compiler injects the default expression when it can prove the callee.

- **vs the bar: PASSES on cost, FAILS on availability** — and it is
  **permanently forbidden** by the Q12 ruling, which names call-site resolution
  the "expensive-to-retrofit approach" ([open-questions.md item 12](../open-questions.md)).
- The narrow case is worth naming anyway: for a statically-known receiver — a
  literal class name, a module-local top-level function, a `self`/`super` send —
  the compiler *does* know the callee. But it buys a language where
  `p.slice(from: 3)` works and `someList.first.slice(from: 3)` raises, with the
  difference invisible at the call site. Worse than no defaults, which is
  presumably why Q12 forbade it outright rather than deferring it.
- **What it precludes:** method reopening (ADR-0026) — a call-site-baked default
  does not change when the method is replaced. C++'s ODR hazard
  ([§6](#6-precedent-with-consequence)), shipped.

### (d) A trailing options `Map` — *available today, zero language change*

```phalcom
configure(options: { host: "localhost", port: 8080 })
```

- **vs the bar: PASSES trivially** — it is not a language feature. One selector,
  one probe. [messages-and-selectors.md §4](../messages-and-selectors.md)
  already directs open-ended keyed config here ("**No `**kwargs`** … take a
  `Map`"), which ADR-0012's label-identity invariant implies.
- **Honest ergonomics:** a real answer for *open-ended config*, a poor one for
  *one optional param*. It costs a Map allocation per call, moves every key from
  a compile-time selector to a runtime string lookup, defeats the IC, defers
  typos from a dNU at the call site to a `None` read somewhere downstream, and
  relocates the defaults into the callee body as
  `options.at("port").unwrapOr(8080)` — the forwarder problem wearing a
  different hat. It addresses an adjacent need; it does not subsume defaults.

### (e) `doesNotUnderstand`-based fallback

Let the miss happen; a dNU handler fills defaults and re-sends.

- **vs the bar: FAILS.** Every defaulted call is a miss, a `Message`
  reification ([`decode_selector`, `method/mod.rs:156`](../../../../phalcom-core/src/method/mod.rs)),
  and a re-send — the slowest path in the VM, chosen for the most common calls.
- **It also breaks `respondsTo(_)`**, which is an *exact* probe against the
  method dict, not against dNU — pinned by
  [`dispatch/dispatch_respondsto_across_arity_family.ph`](../../../../phalcom-core/tests/lang/dispatch/dispatch_respondsto_across_arity_family.ph).
  So `o.respondsTo(Symbol.new("trim()"))` says `false` while `o.trim()` works.
  That is the Ruby wart ([§6](#6-precedent-with-consequence)) reproduced by
  construction.
- **What it precludes:** using dNU for its actual purposes (proxies, delegation)
  without colliding with defaults.

**Least-bad if ever revisited: (a), trailing-only** — which is what Q12 already
ruled. It is the only candidate that meets the stated bar *and* buys something
(d) does not, and its cost (reflection sees synthesized selectors) is
addressable by marking synthesized entries rather than by redesigning dispatch.

## 6. Precedent with consequence

Precedent alone proves nothing; each entry names what the choice **cost**.

| Language | What they did | What it cost them |
|---|---|---|
| **Smalltalk** | Keyword selectors bake arity into the name. No defaults, ever — same reason as Phalcom. | `at:put:`-style overload families everywhere; `copyFrom:to:` and friends are written out in full at every call. The forwarder tax Phalcom pays in `trim` is the Smalltalk steady state. |
| **Objective-C** | Same selector model (`SEL` encodes the keyword shape), same absence. | Same: families of `-initWith…:` methods, plus a culture of designated-initializer discipline to keep the family from drifting. |
| **Python** | Defaults, because dispatch keys on **name only** — arity never enters identity. Evaluated **once, at `def` time**. | The mutable-default bug (`def f(x=[])`), a permanent teachable-moment-grade wart that every Python programmer must be taught explicitly. **If Phalcom ever adds defaults, evaluate-per-call is mandatory**; there is no upside to the Python semantics and (a)'s forwarder gives per-call evaluation for free. |
| **C++** | Defaults are a **call-site** rewrite. | Changing a default in a header does not affect already-compiled callers → ODR violations and ABI surprises across translation units. This is design (c)'s hazard, shipped. |
| **Ruby** | Defaults + `method_missing` + `respond_to?`. | The three interact badly, and `Method#arity` **returns negative numbers** to encode optionality (`-n-1` means "n required, more optional") — a reflection API that lies by convention rather than by type. |
| **JavaScript** | Defaults + `arguments.length` + `fn.length`. | `fn.length` **stops counting at the first default**: `(a, b = 1, c) => {}` reports `1`. Reflection under-reports arity, and libraries that dispatch on `fn.length` (currying, DI containers) break silently. |
| **Swift** | Defaults **and** labels; the compiler synthesizes thunks per default combination. | It requires static dispatch knowledge — the compiler knows the callee's declaration. Under `dynamic`/`@objc` dispatch Swift **loses defaults** for exactly Phalcom's reason. This is why the Swift solution does not port: Phalcom's every send is the case Swift excludes. |

The pattern is legible: **every language with defaults dispatches on something
that is not arity** (name only, or a statically-resolved declaration). Every
language whose *identity* includes arity declines them. Phalcom is in the second
group by construction, not by preference.

## 7. What re-adding would preclude

- **The 1:1 selector↔method reading of `Behavior::methods`.** Under (a), the
  method dict is no longer a faithful list of what the author wrote. Any tool
  built on it (documentation generator, LSP outline, coverage) must learn the
  difference.
- **`respondsTo(_)` as a proxy for "the author wrote this".** It would answer
  `true` for synthesized arities. Correct for callers; misleading for tooling.
- **The `Family` router's candidate list** (ADR-0047, [selectors.md §3.1](../selectors.md)) —
  `obj::slice` would enumerate synthesized members alongside authored ones, and
  the "missing label" diagnostic ([selectors.md:200](../selectors.md)) would
  cite selectors with no source line.
- **Deleting a narrow arity.** Once `f(_)` is synthesized, removing the default
  is a breaking change to callers who never named the param — the same trap the
  overload convention makes visible.
- **Method reopening under (c) only.** A call-site-baked default ignores a later
  method replacement, contradicting ADR-0026's open-methods position. This is
  one more reason the Q12 forbiddance is correct.

Design (a) precludes none of the *dispatch* invariants — that is its whole
appeal. Its preclusions are all in the reflection surface, which is exactly where
a superseding ADR would have to do its work.

## 8. Supersedes `experimental/default-arguments.md` (retired)

**Resolved 2026-07-15 (DEFERRED CB-4). That document is deleted; this section is its
epitaph.** An earlier revision of this doc said "this one does not replace it" and left the
reconciliation owed (DA-1). The reconciliation is now done — by retirement rather than by
edit, since the two docs answered the same question and only one of them was right.

**What was wrong with it.** 40 lines, `Status: Proposed`, and self-contradictory:

- its **Decision** section specified "*No runtime defaults. Defaults are caller-side
  desugar, statically-known callees only*" — i.e. design **(c)**;
- its own **superseding banner** (added 2026-07-12) declared caller-side / static-callee
  resolution "**permanently forbidden**" and named design **(a)**, trailing-only, as the
  ratified-if-ever mechanism.

The body advocated precisely the design the banner forbade. The banner was right — it is
the later, ratified position ([open-questions.md item 12](../open-questions.md)) — but the
banner is the part readers skip, so the doc's most likely reading was its wrong one. It was
retired rather than reconciled because this doc already carried everything it had, correctly:
§2 for the hazard, §5(a) for the ruled mechanism, §7 for what re-adding precludes.

**What it uniquely held: nothing.** It stated a problem and a superseded decision. This doc
adds the status record and its verification, the in-tree evidence (the `trim`/`join`
forwarders, the 3× `" \t\n\r"` duplication, `Behavior::methods`, the variadic two-probe
precedent), the steelmanned benefit case, the five-design evaluation against the ADR's
stated bar, precedent-with-consequence, and the preclusion list.

**The ADR-0043 prose tension — resolved 2026-07-15 (CB-4), but not as filed.** An earlier
revision of this section, and DEFERRED CB-4 after it, claimed **ADR-0043 rejects
arity-family expansion as "combinatorial"**, putting it in tension with Q12's ratification
of that mechanism where it is linear. **That claim is false.** The word appears nowhere in
ADR-0043; it was the *retired* `experimental/default-arguments.md` that rejected
arity-family expansion as combinatorial ("*The alternative (arity-family expansion) is
rejected as combinatorial*"), and both this section and CB-4 read that doc as speaking for
the ADR. Retiring it removed the only source of the contradiction. Recording the correction
rather than quietly dropping it — a claim about an ADR that no one checked against the ADR
is the failure mode this tier exists to catch.

**The real gap, now closed.** ADR-0043's Decision told a future ADR to choose between
"aliasing vs **call-site fold**" — but Q12 **permanently forbids** call-site resolution and
fixes the mechanism to definition-time trailing-only expansion. The ADR left open a door the
ruling had nailed shut and never mentioned trailing-only at all, so a reader following the
ADR alone would design against a forbidden mechanism. ADR-0043 now carries an
[§Amendment](../../../adr/accepted/0043-no-default-arguments-keep-selector-identity-pristine.md)
saying so. The ADR itself **stands**: no default arguments.

## 9. Open questions

Exploratory. **None of these is scheduled, and answering them does not create a
proposal.**

| # | Question | Bearing |
|---|---|---|
| ~~DA-1~~ | ~~`experimental/default-arguments.md` still advocates design (c) in its body while its banner forbids (c). Who edits it, and does it become a stub pointing at Q12?~~ | **CLOSED 2026-07-15 (CB-4).** Neither — the doc is **retired**, not stubbed. It held nothing this doc lacks (§8). `deferred-work.md`'s chore is discharged. |
| DA-2 | ADR-0043's bar is "no single-probe regression", which design (a) **meets**. Should the ADR be amended to name its real second bar (linear expansion + reflection distinguishability)? | **Still open.** 2026-07-15 (CB-4) amended ADR-0043 to record that Q12 *fixed* the mechanism (call-site fold forbidden, trailing-only definition-time expansion) — but the **bar** itself is unchanged: 0043 still states only the single-probe bar, which the ruled mechanism already meets, so a future ADR can clear the stated bar while doing what 0043 meant to prevent. The ADR's §Amendment now says this explicitly instead of leaving it implied. |
| DA-3 | The variadic miss-arm ([dispatch.rs:437-456](../../../../phalcom-core/src/vm/dispatch.rs)) already spends a second probe. Is "single probe" a literal invariant or a steady-state (post-IC) one? The two readings judge design (b) differently. | Affects how ADR-0012's invariant is cited going forward, independent of defaults. |
| DA-4 | Under design (a), should synthesized selectors be distinguishable through reflection (a flag on `MethodObject`, or omission from `Behavior::methods`)? | Determines whether (a)'s cost is a real preclusion or a solved detail. Also bears on `Family` candidate lists (ADR-0047). |
| DA-5 | The 3× `" \t\n\r"` duplication ([core.ph:261,264,267](../../../../phalcom-core/core/core.ph)) is the benefit case in miniature. Is a shared `String.defaultTrimChars` constant the cheap answer, making DA-2..DA-4 moot for the core library? | If the convention's real cost is *duplicated constants* rather than *duplicated methods*, it is fixable without touching the language. |
| DA-6 | Q12 fixes the mechanism as trailing-only. Does "trailing" mean trailing **positional**, or may a labeled param be defaulted? Labels are unordered at the call site (`move(to:, duration:)`), so "trailing" is ill-defined for them. | The ruling did not say. Any future ADR must answer this before it can claim linearity. |
