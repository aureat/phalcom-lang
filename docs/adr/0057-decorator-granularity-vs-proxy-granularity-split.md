# 57. Decorator granularity vs proxy granularity — the method-declaration / whole-object interception split

- Status: Accepted
- Date: 2026-07-13
- Related: `docs/spec/v0.2/next/decorators.md` (the five-tier decorator axis;
  already states "a decorator is a method-granularity proxy"),
  `docs/spec/v0.2/next/proxy.md` (the `Proxy`/`Lazy`/`Trace`/`Retry`/`Capability`
  object-granularity library), `docs/spec/v0.2/next/attribute-classes.md`
  (the `Attribute`/`@On` decorator descriptor),
  `docs/spec/v0.2/next/decorators-behavioral.md` +
  `docs/spec/v0.2/next/decorators-dispatch-observability.md` (the dedicated decorator
  specs whose name-overlaps with the proxy library this ADR resolves),
  [ADR-0052](0052-invariant-reentrancy-scope-and-layout-confined-decorator-state.md)
  (Layout-confined per-receiver decorator state),
  [ADR-0053](0053-runtime-decorator-interception-reuses-override-epoch-guard.md)
  (Runtime interceptor guard)

## Context

Phalcom builds several interception features on **one substrate** —
`perform` + `doesNotUnderstand` + method wrapping ([method-lookup.md],
[object-model.md]). Two libraries fall out of it and, written independently, ended
up with **three colliding names**:

- `proxy.md` specifies object-granularity wrappers `Lazy`, `Trace`, `Retry`
  (virtual proxy / observability / resilience), applied by wrapping an object.
- The decorator library (`attribute-classes.md`, `decorators-stdlib.md`, and the new
  dedicated specs) specifies method-granularity `@lazy`, `@traced`, `@retry`,
  applied by annotating a declaration.

Left unreconciled, a reader cannot tell whether `@retry` and `Retry` are the same
feature spelled two ways, competing proposals for the same job, or two different
things that happen to share a root word. `proxy.md` itself gestures at the answer
("a proxy is a decorator at object granularity") but never states it as a governing
rule, so the dedicated decorator specs had no citable authority for keeping both and
were at risk of each re-deciding the overlap differently. This is a
naming/architecture question that spans the whole `decorators-*` / `proxy.md` /
`reactivity.md` family, so it is fixed once here rather than per-doc.

## Decision

**Decorators and proxies are the same interception substrate at two orthogonal
granularities, and both are kept.** The split is defined on two axes that always
agree:

| Axis | **Decorator** (`@name`) | **Proxy** (a class you instantiate) |
|---|---|---|
| **Granularity** | one method / getter / field / class member | the whole protocol of one object |
| **Applied by** | the **author** of the class, at the declaration site | a **third party**, by wrapping an object it does not own |
| **When** | class-definition time (Compile/Layout/Install) or per-send (Dispatch/Runtime), per the tier | at each send that crosses the wrapper boundary (DNU / around-send) |
| **Surface** | `@Name` on a member; an `Attribute` subclass | `Name.on(target:)`; an ordinary object holding `_target` |
| **Identity** | the real object (no identity leak) | leaks `==`/`class`/`hash` unless the around-send tier is used (proxy.md P-1) |

The two are **complementary, never competing**: a decorator cannot instrument an
object the author does not own (it must be on the declaration); a proxy cannot be
part of a class's own declaration (it wraps from outside). Neither subsumes the
other, so name-overlap is resolved by *keeping both* with a stated division of labor,
not by deleting or renaming either.

### Applying the rule to the three overlaps

- **`@retry` (decorator) vs `Retry` (proxy).** `@retry` is the **primary,
  recommended** surface, because retry-safety is a **per-method** property (only some
  of an object's methods are idempotent). The author opts in method by method. The
  `Retry` proxy is retained only for the **black-box case** — retrying a third-party
  object you cannot annotate — carrying an explicit "unsafe unless *every* method is
  idempotent" caveat, since it retries the whole protocol indiscriminately.
- **`@traced` (decorator) vs `Trace` (proxy).** Two granularities of observability,
  both kept. `@traced` instruments *your own* declaration (Runtime tier, guarded by
  [ADR-0053](0053-runtime-decorator-interception-reuses-override-epoch-guard.md));
  `Trace` observes a *black-box* object from outside (DNU proxy, with the identity
  leak of proxy.md P-1). Argued *for* the split: "instrument my code" and "observe
  code I don't own" are different jobs; neither tool can do the other's.
- **`@lazy` (decorator) vs `Lazy` (proxy).** Genuinely **different mechanisms**, not
  merely two granularities: `@lazy` caches a **method result** in a reserved receiver
  slot (Layout, per-receiver, [ADR-0052](0052-invariant-reentrancy-scope-and-layout-confined-decorator-state.md));
  `Lazy` defers building a **whole object** until its first message (virtual proxy).
  Both keep their names — a sigil (`@lazy`) and a class (`Lazy`) live in different
  namespaces — disambiguated by the "defers a value vs defers an object" distinction.

### Consequences for the existing docs

- `proxy.md`'s `Lazy`/`Trace`/`Retry` subsections are annotated to point at the
  dedicated decorator specs for the owned-code case and to state the granularity rule;
  the `Retry` proxy additionally gains the whole-object-idempotence caveat. The proxy
  library is **not** deleted — its object-granularity, third-party, membrane, and
  prototype use cases (`Capability`, `Prototype`) have no decorator equivalent and
  remain the reason the library exists.
- The dedicated decorator specs cite this ADR as the authority for keeping both and for
  the primary-vs-fallback framing of each overlap.

## Consequences

- **Positive.** The three name-overlaps have one governing rule instead of three
  independent per-doc rulings; a reader can always place a feature by asking "am I
  annotating my own declaration, or wrapping someone else's object?"
- **Positive.** No capability is lost: every proxy use case survives, every decorator
  use case survives, and the recommended surface for each overlap is stated (per-method
  `@retry` over whole-object `Retry`; owned `@traced` over black-box `Trace`).
- **Positive.** The rule is a natural extension of what `decorators.md` and `proxy.md`
  already imply ("a decorator is a method-granularity proxy"), so it ratifies existing
  intent rather than inventing new architecture.
- **Negative / accepted.** Two spellings for adjacent concepts (`@retry` / `Retry`) is
  a minor vocabulary cost — mitigated by the sigil-vs-class surface distinction and the
  cross-references, and cheaper than collapsing them into one mechanism that would have
  to serve both granularities badly.

## What this precludes

- **Collapsing decorators and proxies into one mechanism.** They stay two surfaces on
  one substrate; a single "interceptor" abstraction spanning both granularities is not
  pursued, because the apply-site (author declaration vs third-party wrap) and the
  identity story (real object vs leaked identity) genuinely differ.
- **A decorator that wraps a foreign object, or a proxy that is part of a class
  declaration.** Each stays on its side of the granularity line; the cross-cases are
  the other tool's job.
