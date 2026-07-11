# Default arguments (proposed resolution of open-Q12)

- Status: Proposed · resolves open-Q12 / selectors §7.3
- Hazard: **identity-dispatch ⊗ optional arity** (canonical case)

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
