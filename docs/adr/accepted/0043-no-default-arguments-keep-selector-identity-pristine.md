# 43. No default arguments; keep selector identity pristine

- Status: Accepted
- Date: 2026-07-12
- Amended: 2026-07-15 — prose only, **the decision is unchanged**. Reconciles this ADR's
  forward-looking clause with [open-Q12](../../spec/current/open-questions.md)'s mechanism
  ruling, which is narrower and more specific than what this ADR left open. See
  [§Amendment](#amendment-2026-07-15--the-mechanism-is-no-longer-open). Prompted by
  DEFERRED CB-4.
- Related: [ADR-0012](0012-selector-signature-encoding-and-dispatch.md)
  (selector-signature encoding — arity is *part of the dispatch key*; default
  arguments would make one method answer several arities), `docs/spec/current/object-model.md`
  (message-send dispatch; signature = selector + arity/kind),
  `../../forge/units/U18/u18.md`, `docs/forge/STATE.md` (DEC-U18 resolution record)

## Context

[U18](../forge/units/U18/plan.md) asks whether a method may declare **default
argument values** — `foo(a, b: 10)` callable as both `foo(1)` and `foo(1, 2)` —
so that one definition answers multiple arities.

This was **BLOCKED-ON-DECISION** in the U18 work order (DEC-U18). It bears
directly on the dispatch model: Phalcom keys method lookup on a **signature
symbol that encodes arity** (ADR-0012), which is exactly what lets `foo` and
`foo(_)` coexist as distinct methods. Default arguments break that one-to-one
selector↔method correspondence: a single method body would have to be
registered under, or dispatched to from, several distinct arity signatures,
forcing either signature aliasing at install time or an arity-fold at the call
site — new machinery on the hottest path in the VM.

## Decision

**DEC-U18 = A — no default arguments in v0.2; a method's arity is fixed and its
signature identity is one-to-one.** Resolved by orchestrator autonomous
authority, 2026-07-12 (the architect-recommended conservative option;
reversible pre-release, per the standing delegated-decision protocol).

- No runtime change lands with this unit. Every method has exactly one arity;
  callers supply every argument. Overloading by arity (`foo`, `foo(_)`,
  `foo(_,_)` as separate methods) remains the idiom for "optional" parameters.
- The feature is **not precluded.** If added later, a superseding ADR must
  specify how default-argument methods register against the signature-keyed
  dispatch table without regressing the single-probe lookup — the design work
  this deferral avoids doing speculatively. **Amended 2026-07-15:** this bullet
  originally offered that choice as "aliasing vs call-site fold". open-Q12 has
  since **closed it** — call-site fold is *permanently forbidden*, and the
  mechanism is fixed to definition-time trailing-only expansion. A superseding
  ADR inherits that constraint; it does not get to re-open it. See §Amendment.
- U18 is therefore a **tiny affirm-ADR unit**: it records the ruling and its
  reversibility, and adds no code.

## Consequences

- **Positive.** Selector identity stays pristine — one signature, one method,
  one arity — preserving the ADR-0012 dispatch invariant and its single-hashmap
  probe with no new call-site or install-time machinery.
- **Positive.** No ambiguity between "arg omitted" and "arg passed as the
  default value," and no interaction to design between defaults and keyword/
  named-argument selectors.
- **Negative / accepted.** Callers repeat arguments that a default would elide,
  and library authors write multiple arity overloads instead of one defaulted
  method. Acceptable at v0.2 scope.
- **Revisit trigger.** Ergonomic pressure from real library surfaces plus a
  dispatch design that keeps defaults off the single-probe hot path — then a
  superseding ADR designs it against those requirements **and against Q12's
  fixed mechanism** (§Amendment).

## Amendment (2026-07-15) — the mechanism is no longer open

**Prose only. The decision stands: no default arguments.** This ADR was written as if the
*mechanism* for a future re-add were an open design question ("aliasing vs call-site fold").
[open-Q12](../../spec/current/open-questions.md) — a ruling, no ADR, narrower and more
specific than this one — has since fixed it:

- **Call-site / caller-side resolution is permanently forbidden.** It needs static callee
  knowledge, which dynamic dispatch does not have. This is the expensive-to-retrofit
  approach and the one Q12 was actually about.
- **If ever added, defaults desugar to real arity-family overloads at *definition* time** —
  each installed selector a real forwarding method. Pure codegen over the arity-overloading
  that already works; no dispatch change.
- **Restricted to trailing parameters**, which keeps the expansion **linear** (`n` defaults →
  `n+1` selectors) rather than combinatorial.

**On "combinatorial" — a correction worth recording.** DEFERRED CB-4 asserted that this ADR
"rejects arity-family expansion as *combinatorial*", putting it in tension with Q12, which
ratifies that same mechanism where it is linear. **This ADR never said that** — the word
appears nowhere in it. The claim came from `experimental/default-arguments.md` (retired
2026-07-15, superseded by [`drafts/default-arguments.md`](../../spec/current/drafts/default-arguments.md)),
which *did* reject arity-family expansion as combinatorial and which CB-4 read as speaking
for the ADR. There was no general-vs-trailing contradiction here to fix.

The real gap was the one closed above: this ADR left a door open (call-site fold) that Q12
had already nailed shut, and never mentioned the trailing-only refinement at all. A reader
following this ADR alone would design against a forbidden mechanism.

**Still open** (`drafts/default-arguments.md` DA-2, DA-6): this ADR states only the
"no single-probe regression" bar, which the ruled mechanism already meets — so a future ADR
could clear the stated bar while doing what this one meant to prevent. And Q12 says
"trailing" without saying whether a *labeled* parameter may be defaulted; labels are
unordered at the call site, so "trailing" is ill-defined for them. Neither is scheduled.
