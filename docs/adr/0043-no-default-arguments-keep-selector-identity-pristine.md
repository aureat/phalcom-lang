# 43. No default arguments; keep selector identity pristine

- Status: Accepted
- Date: 2026-07-12
- Related: [ADR-0012](0012-selector-signature-encoding-and-dispatch.md)
  (selector-signature encoding — arity is *part of the dispatch key*; default
  arguments would make one method answer several arities), `docs/spec/v0.2/object-model.md`
  (message-send dispatch; signature = selector + arity/kind),
  `docs/forge/units/U18/plan.md`, `docs/forge/STATE.md` (DEC-U18 resolution record)

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
  dispatch table (aliasing vs call-site fold) without regressing the
  single-probe lookup — the design work this deferral avoids doing speculatively.
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
  superseding ADR designs it against those requirements.
