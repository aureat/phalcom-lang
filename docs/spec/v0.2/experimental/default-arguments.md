# Default arguments (proposed resolution of open-Q12)

- Status: Proposed · resolves open-Q12 / selectors §7.3
- Hazard: **identity-dispatch ⊗ optional arity** (canonical case)

> **Partially superseded (2026-07-12).** open-Q12 is now **RULED** (see
> [open-questions.md](../open-questions.md)): **no default arguments now.** If ever
> added, the ratified mechanism is a **definition-time desugar to TRAILING-ONLY
> arity-family overloads** — *linear* (n defaults → n+1 selectors), not the
> combinatorial general-position expansion this doc rejects, so the two are
> consistent. **Caller-side / static-callee resolution is permanently forbidden.**
> Read this doc's "reserved mechanism" as superseded by that ruling where they differ.
> Index: [deferred-work.md](../deferred-work.md).

## Problem

Method identity is `name + labels + kind` (ADR-0012). A default that lets a call
omit an argument produces a call of *different arity* → a *different selector* →
lookup misses the full-arity method. Arity-family expansion is combinatorial;
static-callee knowledge is unavailable under dynamic dispatch. Precedent: Python
has defaults **because** it dispatches on name only; Smalltalk/Wren avoid defaults
entirely to keep arity in the selector.

## Decision

**No runtime defaults. Defaults are caller-side desugar, statically-known callees only.**

- A def `f(a, b = 0)` declares exactly one selector, `f(_,_)`. `b`'s default is
  compile metadata, not a second entry point.
- A call `f(a)` is rewritten to `f(a, 0)` **only** when the callee is statically
  resolvable (top-level function, `self`/`super` send, module-local). One
  selector, no phantom.
- On a dynamically-dispatched receiver, `f(a)` is a plain `ArgumentError` — the
  default does not travel through the send.

## Precludes

Defaults on dynamically-dispatched sends. Accepted explicitly. The alternative
(arity-family expansion) is rejected as combinatorial. Revisit only if a static
callee-type layer ever lands (not planned).
