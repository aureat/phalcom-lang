# Runtime tier — full design (v0.3 experimental)

- Status: **Experimental (v0.3 track), mandated by PDR-0018 §3.** Normative
  for that track once reviewed; binds nothing in v0.2. Cost model of record:
  ADR-0053. Decorator catalog: [interception.md](interception.md)
  (`@Traced`, `@FeatureFlag`).
- Design goal restated: an undecorated class pays **one bit read** where a
  comparison already happens; a decorated send pays one allocation + one
  indirect call per interceptor; sacred selectors stay sound via ADR-0018's
  existing deopt.

## 1. `Invocation` — the reified send

An ordinary `.ph` class over a small native spine (no new `Value` arm):

```phalcom
class Invocation {
  receiver          // the object the send targeted
  selector          // the Symbol, canonical comma form
  args              // a List (kernel List, ADR-0020)
  proceed()         // run the next interceptor, or the real method at the end
}
```

- `proceed()` is a real method, not a block handed loose — the chain position
  lives in the Invocation (`_index` into the pre-composed chain), so calling
  `proceed` twice **re-enters the rest of the chain twice**. That is defined
  behavior (retry-style interceptors need it), documented, and bounded by the
  ordinary call-depth floor (PDR-0007) — no special re-entry cap.
- Not calling `proceed` is also defined: the interceptor's return value
  becomes the send's result (`@FeatureFlag`'s skip mode is exactly this).
- The Invocation is **frame-local by convention, escapable by construction**
  — an interceptor may store it (it is an ordinary object), but a stored
  Invocation whose `proceed` is called after the send completed re-runs the
  method *fresh*; it is a closure over (receiver, selector, args, chain),
  not a continuation. No frame is captured. This must be stated loudly in
  the class doc because Python/CLOS users will expect `call-next-method`
  dynamic-extent restrictions that do not exist here.
- Allocation: one `Invocation` + one args `List` per intercepted send. This
  is the priced cost from ADR-0053; the args list reuses the argument values
  already on the stack (copy, not re-eval). No pooling in the first cut —
  measure first (ADR-0051); the per-fiber free-list is the recorded
  follow-up if the number demands it.

## 2. The guard bit and the send path

- `ClassObject.has_runtime_interceptor: bool`, set **once**, at
  class-definition time, iff any attached attribute (class-level or on any
  member) declares `tier: Runtime`. Valid as a one-time bit because
  retention is frozen (A-5) and classes are closed (PDR-0001) — both
  preconditions named at the build site; if either relaxes, this becomes an
  epoch counter *first* (ADR-0053's trigger).
- Fast path: the dispatch handler reads the bit alongside the class-identity
  comparison it already performs. Clear ⇒ today's path, byte-identical
  observables. Set ⇒ build the Invocation and enter the chain.
- **Granularity:** the bit is per-class; the chain is per-*member*. A
  class-level Runtime attribute intercepts every send to the class's
  instances; a member-level one intercepts that selector only. The per-class
  bit over-approximates (any Runtime attribute anywhere sets it); the
  member-level filter happens at chain lookup (a `Symbol → chain` map on the
  class object, empty entries meaning "class-wide chain only"). A decorated
  class's *undecorated* members therefore pay bit + one failed map probe —
  acceptable for v0.3-experimental; the recorded optimization is folding a
  per-member bit into the method-dict entry when ICs land.

## 3. Chain composition

- Collected at class-definition time, after attribute instantiation, in
  **source order, innermost-last** (the ratified stacking rule): class-level
  Runtime attributes outermost (in their own source order), then
  member-level ones. Pre-composed once into a flat `Vec` cached on the class
  (decorators/README.md's "Future optimizations" — build the fused chain at
  definition, never walk attribute lists per send); the `n = 1` case stores
  the interceptor directly, no vector.
- Each interceptor is the attribute instance's `aroundSend(_)` method,
  looked up once at composition time and stored as a bound method — a
  send-time lookup would make interceptor dispatch itself interceptable,
  which is the recursion trap §4 exists to close.

## 4. Re-entrancy and the per-fiber bypass

An interceptor that sends to *any* object with the same interceptor class
recurses through interception (a tracing sink logging via a traced logger is
the canonical death spiral). Design:

- Per-fiber `interceptor_depth: u32`, part of fiber-switch state (saved and
  restored exactly like ADR-0052's `checking` set, and for the same reason —
  a suspended fiber's bookkeeping must not leak into the next fiber).
- Sends made while `interceptor_depth > 0` **bypass Runtime interception
  entirely** (the bit read short-circuits). Entered on chain entry,
  decremented via the same unwind-safe `ensure` discipline the invariant
  guard uses — a throwing interceptor must not permanently disable
  interception on its fiber.
- Consequence, deliberate: interceptors observe *application* sends, never
  each other's. This is coarser than a per-attribute-instance flag (an
  interceptor cannot trace a *different* interceptor) and is chosen anyway:
  the alternative reintroduces the spiral one level up, and "interception is
  invisible to interceptors" is a rule a user can hold in their head.
  CLOS's around-method experience is the precedent: unconstrained
  meta-recursion is the classic MOP footgun.

## 5. Sacred selectors and deopt

No new mechanism — the ADR-0053 ruling, restated operationally: installing a
Runtime attribute on a class in the `Bool`/`Block` sacred families must route
through `note_method_installed` so the family's pristine flag flips and
ADR-0018's inlined fast paths deopt to real sends, which then see the guard
bit. Fixture: `@Traced` on a `Bool`-family class; the inlined `ifTrue` site
produces identical results pre/post decoration, with the trace appearing
post. If the flag flip is skipped, inlined sites silently bypass
interception — this is the one soundness cliff in the tier, and the fixture
is its fence.

## 6. Erasure and observability obligations

- The per-member erasure golden (D-5 granularity) lands **before** the first
  Runtime decorator ships (PDR-0018 §4): strip every `runtime: false`
  attribute ⇒ bytecode-identical members; strip a Runtime attribute ⇒
  identical bytecode *and* an identical execution trace of a probe program
  (the interception is the only delta).
- `respondsTo`/reflection are unaffected — interception wraps dispatch, it
  does not populate dictionaries. `Method#attributes` shows the Runtime
  attribute; that is the introspection story.

## 7. Test plan (first cut)

Positive: chain order (two interceptors, observed nesting), `proceed`-twice
re-execution, skip-without-proceed result, class-wide vs member-level
filtering, bypass depth (sink sends untraced), fiber-switch state isolation
(two fibers, one traced, interleaved yields). Negative: throwing interceptor
restores depth (`ensure` lane); sacred-family deopt fixture (§5). Perf lane:
undecorated-class benchmark before/after the bit lands — the "one bit" claim
gets a number or it is not a claim (ADR-0051).

## What this precludes

- Interceptors observing other interceptors (§4's rule — revisit only with a
  superseding design that solves the spiral).
- Any second interception substrate. Proxies (ADR-0057) wrap objects at the
  library level; this tier is the only *dispatch-level* hook, and Dispatch
  ([dispatch-tier.md](dispatch-tier.md)) fires on miss only.
