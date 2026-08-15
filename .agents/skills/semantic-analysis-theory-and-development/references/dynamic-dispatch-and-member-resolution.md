# Dynamic Dispatch and Static Member Resolution

## 1. Static analysis approximates dispatch; it does not redefine it

For a message send, the analyzer's job is to conservatively approximate the runtime target set and expose useful candidate/member information. The runtime language semantics remain authoritative.

A useful decomposition is:

```text
1. evaluate receiver
2. evaluate arguments in normative order
3. construct canonical selector from syntax/labels/arity
4. determine dispatch side and lookup-start context
5. perform class/metaclass inheritance lookup + visibility/access rules
6. invoke selected method/fallback behavior
```

Static analysis may know only an abstract receiver set, so it computes a set of possible step-5 targets. It must not add types to selector identity merely to make resolution easier.

## 2. Current Phalcom anchor

**CURRENT:** current semantic `CallableId` contains owner `ClassId`, canonical selector string, and `DispatchSide::{Instance, Class}`. Semantic queries include receiver member lookup, completion members, ancestry tests, callable summaries, and inferred receiver expression facts. Current typing architecture analysis explicitly recommends that future type annotations remain non-dispatching and not alter selector identity.

Treat selector canonicalization as shared language infrastructure. If compiler/runtime and semantic engine each build selectors differently, every higher-level analysis is suspect.

## 3. Candidate-set semantics

Let abstract receiver fact `R#` denote a set of possible runtime receivers. Static dispatch produces:

```text
Targets(R#, selector, context) =
    ⋃ { RuntimeLookup(class(v), selector, context) | v ∈ γ(R#) }
```

In practice, the analyzer works over class candidates rather than concrete values. If `R# = Instance(A) | Instance(B)`, resolve `selector` independently from A and B using inheritance and access semantics, then deduplicate targets.

If `R#` is unbounded/dynamic, return a dynamic candidate state rather than an empty set. Empty means “no target can exist under the modeled assumptions”; dynamic means “analysis cannot enumerate targets.”

## 4. Instance side, class side, and metaclasses

Phalcom is class-based with class-side behavior. Do not treat `Foo` and an instance of `Foo` as the same receiver domain.

```text
Instance(Foo)   -> instance-side lookup
ClassObject(Foo)-> class-side/metaclass lookup according to Phalcom model
```

If metaclass inheritance differs from ordinary class inheritance, static lookup must model that exactly. A convenience “static methods HashMap” that skips metaclass semantics can make completion appear correct while reflection/super/runtime dispatch disagrees.

## 5. `super`

`super` changes where lookup begins while preserving the actual receiver. Conceptually:

```text
RuntimeLookup(receiver_class,
              selector,
              start_after = lexical_owner,
              receiver = self)
```

Static HIR should therefore carry `LookupContext::Super`, not rewrite the receiver to a superclass value. This matters for overridden methods, reflection, `self` identity, field access, and type/refinement reasoning.

## 6. Visibility/access context

Member existence and member accessibility are different. A completion surface may list inherited members but filter by current lexical class/module/access context. A checker diagnostic for inaccessible member should not report “member does not exist.”

Represent:

```text
Resolution = FoundAccessible(target)
           | FoundInaccessible(target, rule)
           | Ambiguous(candidates)
           | Missing
           | Dynamic/Unbounded
```

This preserves better diagnostics and refactoring behavior.

## 7. Method families, partial selectors, and packs

Phalcom-specific selector semantics can include open method families or call-site label completion. **CURRENT:** `ValueShape::Family { receiver, base }` exists to retain an open method family until call-site selector information is known.

Keep the distinction between:

```text
family identity / base name
canonical complete selector
call-site dynamic selector due to packs/spreads/reflection
```

If a dynamic pack can alter labels/arity, static resolution may need a bounded family candidate set or dynamic boundary. Never choose one exact selector merely from the visible prefix unless semantics guarantee it.

## 8. Reflection and mutation

Dynamic reflection can break static dispatch assumptions. Relevant operations may:

- add/replace methods;
- mutate superclass/metaclass relationships if allowed;
- invoke a selector reflectively (`perform`-like behavior);
- route misses through `doesNotUnderstand`-like fallback;
- expose native methods/classes not represented in source.

A static fact “send S resolves uniquely to M” needs an assumption set. For LSP navigation, source-visible unique resolution may be useful even in an open world. For devirtualizing optimizer, it is insufficient unless dispatch tables are stable, version-guarded, or closed-world/proved.

## 9. Inline caches versus semantic caches

Runtime inline caches optimize concrete dispatch and are invalidated/versioned according to runtime class/method mutation. A semantic cache answers source-analysis queries and is invalidated by source/module/semantic dependencies. They may use similar keys but are not the same cache.

Do not reuse runtime cache validity as static-analysis truth without an explicit bridge. Conversely, static candidate sets can inform optimization strategy but cannot replace runtime guards under open-world mutation.

## 10. Types and dispatch

Future optional typing must not silently introduce type-directed overload selection unless a normative design explicitly chooses that language change.

These are different:

```text
runtime selector dispatch:
    receiver runtime class + selector + lookup context -> method

static type checking:
    receiver expression type + selector surface -> verify call is allowed
```

The checker can use types to prove that all possible runtime receivers support a selector, but the type annotation itself should not become part of the selector key. This preserves dynamic semantics and reflection compatibility.

## 11. Call effects

A send can invoke arbitrary user code unless the target set and summary prove otherwise. Therefore unresolved/dynamic sends should conservatively include effects such as:

```text
may throw
may mutate reachable state
dynamic dispatch state may change
may invoke passed blocks/callables
may yield/suspend if concurrency semantics permit
```

Exact effect policy belongs to the effect/prover architecture, but semantic analysis must not equate “could not resolve” with “pure/no-op.”

## 12. Tests

- instance versus class-side same selector;
- inherited override and unique target;
- `super` begins lookup above lexical owner but keeps receiver;
- visibility: present-but-inaccessible versus missing;
- two receiver candidates resolving to different overrides;
- open method family completed by labels;
- dynamic pack prevents exact selector resolution;
- reflective method mutation invalidates optimizer-strength assumption;
- source completion and runtime conformance fixtures use same selector encoding;
- future annotations do not change `CallableId`/selector identity.

## 13. Review questions

1. What exact runtime dispatch rule is being approximated?
2. Where is selector canonicalization defined?
3. Is dispatch side explicit?
4. Does `super` preserve receiver identity?
5. Does candidate “none” mean impossible or merely unanalyzable?
6. Are access/visibility failures distinct from absence?
7. Can reflection/native code mutate the dispatch relation?
8. What validity assumption makes a cached unique target safe?
9. Are types checking the send or selecting a different runtime method?
