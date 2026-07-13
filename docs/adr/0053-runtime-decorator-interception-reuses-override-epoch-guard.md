# 53. Runtime-tier decorator interception reuses the sacred-selector override-epoch guard

- Status: Proposed
- Date: 2026-07-13
- Related: [ADR-0018](0018-sacred-selector-inliner-and-override-guard.md)
  (sacred-selector inliner + override-epoch deopt guard — the mechanism this
  generalizes), [ADR-0012](0012-selector-signature-encoding-and-dispatch.md)
  (selector dispatch, inline-cache-ready design, IC population deferred),
  `docs/spec/v0.2/next/decorators.md` (five-tier model — Runtime tier, the
  "Inline-cache invalidation" hazard this resolves), `docs/spec/v0.2/next/attribute-classes.md`
  (`aroundSend(_)` hook)

## Context

`decorators.md`'s Runtime tier lets a decorator's `aroundSend(_)` hook
intercept every message send to its receiver, not just the decorated member.
Review left two costs unspecified:

1. Whether a class with **no** Runtime interceptor pays any tax on ordinary
   sends.
2. Whether interception reaches **sacred-selector** sends (`ifTrue`,
   `whileTrue`, `+`, …) that [ADR-0018](0018-sacred-selector-inliner-and-override-guard.md)'s
   inliner splices directly into bytecode, bypassing ordinary dispatch
   entirely.

Left unaddressed, (2) is unsound in exactly the shape ADR-0018 already solved
once: the inliner assumes a sacred selector still resolves to the kernel
method. Attaching a Runtime decorator to a class changes that assumption the
same way a hand-written override does; an unguarded inline would silently
skip the interceptor, producing observably different behavior on the fast
path versus the slow path — the same "speculative inlining ⊗ late binding"
hazard ADR-0018 exists to close, now reopened by a new source of "late
binding."

## Decision

Runtime-tier decorator installation is treated as the **same class of event**
as a sacred-method redefinition for ADR-0018's guard, and is given a direct
analog for ordinary sends.

### Sacred selectors — reuse the existing pristine flags verbatim

If a Runtime decorator is ever installed on `Bool`/`Block` (or any class in a
sacred-selector family), installation flips that family's existing
`bool_sacred_pristine`/`block_sacred_pristine` flag (`universe.rs`), exactly as
a hand-written override of `ifTrue`/`whileTrue` already does. No new opcode,
no new flag: an installed `aroundSend` hook is indistinguishable, from the
inliner's point of view, from "this family's sacred method might not behave
as pristine." The existing deopt path (guarded fast path → real send on guard
failure) already routes through ordinary dispatch, which is where a Runtime
interceptor would be consulted. Decorating `Bool`/`Block` directly is expected
to be vanishingly rare, matching ADR-0018's own "sacred-selector overrides are
exceptional" assumption — no new cost model needed, the existing one already
covers it.

### Ordinary sends — a general per-class interceptor bit, IC-guard-checked

For non-sacred sends, generalize ADR-0018's per-class-*family* flag into a
per-**class** bit: `has_runtime_interceptor: bool`, set once at
class-definition time when a Runtime-tier decorator installs. Decorators
install at class-definition time only, per `decorators.md`'s fixed phase
order — before any instance exists to be sent to, so the bit is set before any
call site could have cached a decision. An inline cache's guard
([ADR-0012](0012-selector-signature-encoding-and-dispatch.md), IC population
deferred but designed to be addable without a redesign) reads this bit
alongside its existing `ClassId` compare: a monomorphic hit on a class with
`has_runtime_interceptor == false` costs nothing beyond the existing IC check;
a hit on a decorated class routes through the interceptor chain instead of
the cached direct call.

Because installation happens once, before instances exist, this needs no
cache-invalidation epoch bump for already-warm sites under the current spec
(Install/Dispatch/Runtime tiers are class-definition-time-only, not
toggleable later). If a future revision admits post-definition attribute
mutation (`attribute-classes.md` open question A-5) or runtime hierarchy
mutation (open-Q4), `has_runtime_interceptor` becomes a proper epoch counter
at that point, following the same "epoch bump on mutation" discipline
ADR-0018 already established for the sacred-selector case — noted as a
revisit trigger below, not built now.

## Consequences

- Undecorated classes — the overwhelming majority of all code — pay exactly
  the existing IC check cost, unchanged: one bit read alongside a comparison
  already being made.
- A class carrying a Runtime decorator on `Bool`/`Block` deopts its
  sacred-selector fast path via the exact mechanism that already exists for
  redefinition — no new soundness surface, no new testing burden beyond
  extending ADR-0018's existing `control_flow_inline_override_honored`-style
  coverage to interceptor installation.
- `decorators.md`'s "Inline-cache invalidation" hazard is resolved:
  Install-tier wrapping was already IC-friendly (one `Method` swap);
  Runtime-tier now has an explicit, cheap guard instead of an open question.
- **Negative / accepted.** Every class gains one bit of metadata
  (`has_runtime_interceptor`), whether or not it is ever decorated —
  negligible next to the existing per-class method-dictionary/slot-vector
  overhead.
- **Revisit trigger.** If attribute retention becomes mutable post-definition
  (A-5) or class hierarchies become mutable at runtime (open-Q4),
  `has_runtime_interceptor` must become a real epoch counter with
  invalidation on mutation, superseding this ADR's "set once, never
  invalidated" simplification.

## What this precludes

Nothing new. This is strictly an implementation-cost commitment for a
capability `decorators.md` already specified; it forecloses only an
*unguarded* Runtime interceptor — one that either taxes every send
unconditionally or silently fails to intercept sacred selectors — which was
never a sound design to begin with.
