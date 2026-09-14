# 12. Label-encoded selectors and inline-cache-ready dispatch

- Status: Accepted
- Date: 2026-07-11
- Related: `docs/spec/current/messages-and-selectors.md` §2–3; `docs/spec/current/method-lookup.md` §1; `docs/spec/current/object-model.md` §7; forge findings F1, F7, F8; [ADR-0009](0009-handle-arena-heap.md)

> **Amended 2026-07-11:** canonical selector spelling is comma form
> `move(_,to,duration)` (see [`docs/spec/current/selectors.md`](../../spec/current/selectors.md)); the
> original colon spelling in this ADR is superseded. The decision and reasoning
> below are otherwise unchanged — only the selector's string encoding changes.

## Context

A Phalcom selector is an interned symbol encoding **name + argument labels**,
Smalltalk style ([Messages & Selectors §2](../../spec/current/messages-and-selectors.md)):
`move(to,duration)` and `move(_,_)` are **distinct methods**, and `name=(_)`,
`+(_)`, `add(_,_)` are all their own selectors. Because labels are baked into the
interned symbol, lookup stays a single hashmap probe ([Method Lookup §1](../../spec/current/method-lookup.md)).

The current tree dispatches on **arity only** (`SignatureKind::Method(u8)`), which
cannot distinguish label sets and forces malformed encodings. The forge audit
pinned three defects in exactly this code:

- **F1** — the `Invoke` handler discards `call_method`'s `Result`, so a primitive
  error is silently swallowed (`Point.new("abc")` → no output, exit 0) with a
  latent stack desync and no `?`.
- **F7** — `object`'s static `new()` is registered as `Method(1)` for a 0-arg
  selector: arity metadata mismatch.
- **F8** — the `Greater` opcode interns a malformed selector `">( _)"` (stray
  space) — a divergent encoder producing a selector the compiler never emits.

## Decision

Replace arity-only dispatch with **label-encoded selector symbols** and structure
dispatch to host an inline cache:

- A selector is the interned label-encoded string (`add(_,_)`, `move(to,duration)`,
  `name=(_)`, `+(_)`, variadic `sum(*)`) → a `Symbol`. A **single**
  `encode_selector(name, labels, kind)` helper is the sole source of truth, shared
  by the compiler and every runtime selector builder (`perform`, `SEND_DYNAMIC`,
  `doesNotUnderstand` forwarding) so they cannot diverge (F8 was exactly a divergent
  encoder).
- `Signature { selector, kind, positional_arity, variadic }`. The method dictionary
  is keyed by the selector symbol; lookup is **one hashmap probe**.
- `Invoke` keeps its selector-constant operand — the dot-send → `Invoke(selector_const)`
  shape is retained; only what the selector symbol encodes changes.
- Dispatch is built **inline-cache-ready**: each call site owns a monomorphic cache
  slot (receiver class → resolved method), keyed by the stable class handle
  ([ADR-0009](0009-handle-arena-heap.md)). The IC *population* may be deferred, but
  the dispatch shape must not preclude it.

This supersedes the arity-only `SignatureKind::Method(u8)` model and folds the
fixes for **F1** (the missing `?` on the rewritten `Invoke` handler), **F7**
(correct 0-arg metadata), and **F8** (correct operator selector, no stray space)
into the rewrite rather than scheduling them as separate patches.

## Consequences

- `foo`, `foo()`, `foo(_)`, `move(to,duration)`, and `move(_,_)` are all
  distinct, resolvable methods — the Smalltalk selector semantics the spec requires.
- Errored/failed sends propagate correctly once `Invoke` threads the `Result` (F1),
  turning a silent exit-0 into a surfaced error.
- One `encode_selector` helper keeps compile-time and run-time selector construction
  byte-identical, closing the F8 divergence permanently.
- The monomorphic IC slot is the hook for later polymorphic/megamorphic caching and
  for the variadic-table fallback ([Messages & Selectors §4](../../spec/current/messages-and-selectors.md)).
- **Inline-cache population is deferred.** Dispatch is built IC-*ready* here; the
  cache itself is a speed item in the deferred register, not part of the accepted
  decision. The shape is fixed now so adding the cache later is not a redesign.
- `Signature` reserves a separate internal-binding field and the variadic flag now,
  so external≠internal parameter names ([open question Q3](../../spec/current/open-questions.md))
  and variadics can be added later **without** changing selector identity.

## Alternatives considered

- **Arity-only dispatch** (the current `SignatureKind::Method(u8)`). Cannot tell
  `move(to,duration)` from `move(_,_)`, forced the F7/F8 metadata mismatches, and
  contradicts the spec's selector identity. Rejected.
- **Populate the inline cache now.** Faster warm sends, but it front-loads
  polymorphic/megamorphic bookkeeping onto dispatch that is not yet correct.
  Deferred; the IC-ready shape captures the design intent at zero present cost.
