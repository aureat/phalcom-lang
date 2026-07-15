# Decorators (`@`) — the tier model and the built surface

- Status: **Partially implemented** — the Compile-tier expanders and the
  `Attribute` retention layer are built and green; the Install/Dispatch/Runtime
  tiers are **specified but not built** (see "What is built, by tier" below).
- Date: 2026-07-12 (five-tier model ratified 2026-07-14 under
  [ADR-0054](../../../adr/accepted/0054-two-speed-ratification-annotation-decorator-tiers.md));
  split into per-decorator as-built files 2026-07-15
- Evidence: `phalcom-core/src/compiler/attributes.rs` — `AttributeRegistry::new`
  (L660) registers exactly ten expander rows (L662-671);
  `expand_class_attributes` (L1548) is the pass. Fixtures under `phalcom-core/tests/lang/decorators/`,
  `tests/lang/classes/`, `tests/lang/errors/`, `tests/lang/compile-errors/`.
- Depends on: [annotations-core.md](../experimental/annotations-core.md) (the `@` mechanism, registry, phase pipeline)
- Related:
  [annotation-paradigm-bridges.md](../experimental/annotation-paradigm-bridges.md) (the method-table / layout tier line) ·
  [annotations-construct.md](../experimental/annotations-construct.md) (layout derives) ·
  [annotations-contracts.md](../experimental/annotations-contracts.md) (weave tier) ·
  [typing.md](../experimental/typing.md) (erasure invariant E, §5.2/§9) ·
  [method-lookup.md](../method-lookup.md) (`doesNotUnderstand`, `perform`) ·
  [object-model.md](../object-model.md) (metaclass tower, `Behavior`)

## The per-decorator files

Every decorator built into the compiler has its own **as-built** file. Those files
are authoritative for what the implementation does; this file is authoritative for
the tier model they sit in.

A file marked **Not built** below is the exception: it *specifies* work rather than
recording it, and says so in its own Status line. Read it as a plan, never as a
description of HEAD.

| File | Decorators | Tier | Status |
|---|---|---|---|
| [requires.md](requires.md) | `@requires` | Compile / weave | **Implemented** (U-ANNOT-CONTRACTS) |
| [ensures.md](ensures.md) | `@ensures` | Compile / weave | **Implemented** (U-ANNOT-CONTRACTS) |
| [invariant.md](invariant.md) | `@invariant` | Compile / weave | **Implemented** (U-ANNOT-CONTRACTS, + [ADR-0052](../../../adr/accepted/0052-invariant-reentrancy-scope-and-layout-confined-decorator-state.md)) |
| [construct.md](construct.md) | `@construct` | Compile / generate | **Implemented** (U-ANNOT-LAYOUT) |
| [accessors.md](accessors.md) | `@get`, `@set` | Compile / generate | **Implemented** (U-ANNOT-LAYOUT) |
| [data.md](data.md) | `@data` | Compile / generate | **Implemented** (U-ANNOT-LAYOUT) |
| [sealed.md](sealed.md) | `@sealed`, `@variant` | Compile / generate | **Implemented** (U-ANNOT-LAYOUT) |
| [on.md](on.md) | `@On` + the `Attribute` reflection layer | class-side declaration + retention | **Implemented** (M-ATTR-ROOT) |
| [native.md](native.md) | `@native` | Compile / generate (subtractive) | **Not built** — specified 2026-07-16 |
| [ignore.md](ignore.md) | `@ignore` | Compile / generate (subtractive) | **Not built** — specified 2026-07-16 |

Ten registered names, eight as-built files: `@get`/`@set` share
[accessors.md](accessors.md) (they are a pair), and `@variant` **requires**
`@sealed` so both live in [sealed.md](sealed.md).

`@native` and `@ignore` are **not registered** — they raise `attr.unknown` on HEAD.
Their two files are specifications, and they are the first **subtractive**
decorators: every built decorator adds members or wraps bodies, while these remove
one. That exceeds what the `AttributeExpander` trait can express (`expand` takes
`&mut ClassMember` and cannot remove itself from `ClassDef::members`), so their
effect must live in `expand_class_attributes` — the same registered-no-op-plus-
driver-special-case shape `@invariant` already uses. Building them makes it twelve
names and ten files.

## What is built, by tier — read this before the model below

The five-tier model is the **ratified design**. The implementation has landed the
Compile tier and the retention/reflection layer, and nothing else:

| Tier | Built? | What exists on HEAD |
|---|---|---|
| **Compile / derive** | ✅ **built** | `@requires`/`@ensures`/`@invariant` (weave) and `@get`/`@set`/`@construct`/`@data`/`@sealed`/`@variant` (generate), all as AST→AST expanders in `attributes.rs`. |
| **Layout / slot** | ⚠️ **no distinct tier** | Nothing reserves a slot. `@construct` is classified Layout by the spec but is implemented as an ordinary generate-phase derive. No `finalizeLayout` hook, no `reserveSlot`/`slotAt`/`setSlotAt`. |
| **Install / metaobject** | ❌ **not built** | The `wrap(_)` selector is *reserved and validated* (`RESERVED_HOOKS`, attributes.rs L1405-1406) but **never dispatched**. No `Method.fromBlock`, no `Method.invokeOn`, no `Behavior.defineMethod`. |
| **Dispatch / DNU** | ❌ **not built** | `resolveMissing(_)` reserved and validated; never dispatched. |
| **Runtime / per-send** | ❌ **not built** | `aroundSend(_)` reserved and validated; never dispatched. No `Invocation` object, no `has_runtime_interceptor` guard bit ([ADR-0053](../../../adr/accepted/0053-runtime-decorator-interception-reuses-override-epoch-guard.md) is ratified but unimplemented). |

**The load-bearing consequence.** A user `Attribute` subclass declaring
`@On(Method, Install)` and implementing `wrap(m)` **compiles, and its instance is
constructed and retained** — but `wrap` is never called and the decorated method is
never wrapped. A tier declaration is today a *validated claim*, not an executed
hook. [on.md](on.md) §"Not built" documents exactly which enforcement does run.

Re-verified against the tree 2026-07-15: `Method.fromBlock`, `defineMethod`,
`reserveSlot`, `slotAt`, `setSlotAt`, `Invocation`, `has_runtime_interceptor`,
`Signal`, `Computed`, `Effect`, `Reactive`, `Monitor` return **zero** hits across
`phalcom-core/src`, `phalcom-core/core`, and `phalcom-ast/src`. `wrap`,
`resolveMissing`, `aroundSend` and `finalizeLayout` appear **only** in
`RESERVED_HOOKS` (attributes.rs L1405-1406) — i.e. as names to validate, never as
selectors to send.

The unbuilt library that would sit on the three runtime tiers now lives in
[drafts/decorators-behavioral.md](../drafts/decorators-behavioral.md),
[drafts/decorators-dispatch-observability.md](../drafts/decorators-dispatch-observability.md),
and [drafts/decorators-stdlib.md](../drafts/decorators-stdlib.md).

## Context

[annotations-core.md](../experimental/annotations-core.md) commits `@` to a single
model — a **compile-time, order-independent AST→AST derive pass** with "no new
bytecode, no `Value` arm, no VM change" and "nothing DNU-hookable." It leaves the
runtime-decorator and metaobject models explicitly foreclosed:

> Committing to derive-macro semantics forecloses ever making `@` a Python-style
> runtime decorator hook without a second mechanism.

The paradigm-bridges note already cracks that commitment open by drawing **two**
tiers (method-table macro vs. layout derive) rather than one, and observing that
reactive `@observable` state wants "the same `Map<Symbol,…>` shape a
`doesNotUnderstand`-delegation prototype object uses."

> **Superseded justification, per [ADR-0054](../../../adr/accepted/0054-two-speed-ratification-annotation-decorator-tiers.md).**
> Earlier drafts justified reopening that foreclosure via
> [typing.md §5.2](../experimental/typing.md)'s erasure invariant `E` — arguing
> that under a dynamically-typed Phalcom `E` no longer holds for typed members
> anyway, which removes the sole argument against runtime hooks. That is **no
> longer the live justification**: it leaned on a third, unrelated, equally
> unratified draft. The actual justification is
> [ADR-0053](../../../adr/accepted/0053-runtime-decorator-interception-reuses-override-epoch-guard.md),
> which gives the Runtime tier's interception cost an explicit, implementable
> guard. The erasure argument is kept as historical context for why this note was
> originally written, not as a standing argument.

This note unifies every decorator kind — compile-time, layout, dispatch, install,
runtime — under **one grammar and one registry**, distinguished by a declared
**tier** that says *when the decoration takes effect*.

## The tier model (design)

`@` is a **single sigil over a spectrum**, not one mechanism. Every decorator name
resolves through the [annotations-core](../experimental/annotations-core.md)
registry to a descriptor that declares a **tier**. The tiers form one axis — *when
the decoration fires* — from pure static codegen to per-send interception:

| Tier | Fires | What it touches | Runtime cost | `runtime` | Examples |
|------|-------|-----------------|--------------|-----------|----------|
| **Compile / derive** | compiler pass, once | AST → AST; emits static members | none | `false` | `@data`, `@variant`, `@get`/`@set`, `@requires`/`@ensures` |
| **Layout / slot** | compiler *finalize*, once | grows/reserves the instance slot vector (ADR-0011) | none per-call; changes layout | `false` | `@construct`, `@observable` storage |
| **Install / metaobject** | class-definition time, once | wraps/installs a real `Method` object | one-time; wrapped `Method` stays inline-cacheable | `true` | `@memoize`, `@timed`, `@synchronized`, type checks, schema `defineMethod` |
| **Dispatch / DNU** | on lookup **miss**, lazily | generates/installs `doesNotUnderstand` | slow-path only | `true` | `@delegate`, `@method_missing` |
| **Runtime / per-send** | **every** invocation | an around-send wrapper consulted per call | per-call | `true` | `aroundSend`, `@traced`, `@featureFlag` |

Read top-to-bottom, `@` is a single dial from "compile-time derive" to "consulted
on every message." A runtime-checked type annotation (`amount: Int`) slots in at
the **install** tier.

### The descriptor

The core registry's `AttributeExpander` row generalizes to a **decorator
descriptor** that names its tier and whether it survives to runtime:

```rust
struct Decorator {
    name:    String,
    tier:    Tier,            // Compile | Layout | Install | Dispatch | Runtime
    runtime: bool,            // false ⇒ the optimizer/erasure-test may ignore it
    apply:   TierHook,        // a phase-appropriate callback (see below)
}

enum Tier { Compile, Layout, Install, Dispatch, Runtime }
```

> **As built, this struct does not exist.** The real registry row is the
> `AttributeExpander` trait (attributes.rs L90-98): `legal_targets()` plus
> `expand(&mut ExpandCtx, &mut ClassMember, &[Expr])`. There is no `tier` field,
> no `runtime` flag, and no `TierHook` enum — every registered expander *is*
> Compile-tier by construction. "Tier" exists in the implementation only as five
> bare names `validate_attribute_class` matches inside an `@On(...)` argument list
> (`TIER_NAMES`, attributes.rs L1413). See [on.md](on.md).

- **Compile / Layout** hooks are the existing `AttributeExpander`:
  `expand(&ClassDef, Target) -> Vec<ClassMember>` — pure AST, run in the compiler
  pass. `Layout` additionally reserves/reads declared slots (gated on
  [annotations-construct.md](../experimental/annotations-construct.md)).
- **Install** hooks receive the reified `Method` at class-definition time and
  return a replacement: `wrap(Method) -> Method`. This is the first-class,
  value-level decorator — it may close over runtime state (a cache, a lock, a
  rate limiter). Requires the metaobject surface (`Behavior.defineMethod`,
  `Method.invokeOn`, `Method.bind`) sketched in
  [object-model.md §8](../object-model.md).
- **Dispatch** hooks install/extend a `doesNotUnderstand` handler; they fire only
  on a lookup miss ([method-lookup.md §2](../method-lookup.md)).
- **Runtime** hooks register an around-send interceptor consulted on *every* send
  to the receiver — strictly more than DNU, which fires only on failure.

Builtins are the first rows. User-defined decorators register at **Install** or
**Runtime** only (the static tiers stay compiler-owned). Same `@name` /
`@name(args)` surface throughout; the compiler dispatches on `tier`.

### Composition is one fixed, total phase order

[annotations-core §D2](../experimental/annotations-core.md) already fixes a
three-phase pipeline (`generate → weave → finalize`). Two phases extend it into a
**total order across all five tiers**:

```
1. generate   (Compile)   — member-adding derives             → raw methods
2. weave      (Compile)   — body-wrapping derives; within weave:
                            invariant(outermost) → post → pre (Eiffel order)
3. finalize   (Layout)    — slot assignment, base-name index
4. install    (Install)   — wrap/replace the reified Method (once)
5. dispatch   (Dispatch)  — register the DNU handler
   runtime    (Runtime)   — register the around-send hook; consulted later, per send
```

Each later phase can only decorate what an earlier phase produced. `@construct`
(generate) must create `new` before an `@memoize` (install) can wrap it; a woven
`@invariant` (weave) is baked into the body before the method is installed.
Because the order is total, a member may wear several tiers with **no user-visible
ordering ambiguity**:

```phalcom
class Account {
  @observable var _balance          // Layout — reserves reactive slots

  @invariant(_balance >= 0)         // Weave (Compile) — post-check in body
  @synchronized                     // Install — wrap Method in a lock (once)
  @traced                           // Runtime — log around every send
  withdraw(amount: Int) -> Self {   // Install — runtime type check on `amount`
    _balance = _balance - amount
  }
}
```

Resolution for `withdraw`:

1. `@invariant` is woven into the body at compile time.
2. Layout reserves `_balance`'s observable slots at finalize.
3. The method is reified; `@synchronized` wraps that real `Method` once at
   definition time; `amount: Int` installs an entry check on the same wrapper.
4. `@traced` registers a per-send interceptor consulted at dispatch.

The tiers never fight because each acts in its own phase on the artifact the
previous phase handed it.

> **The example does not compile on HEAD.** Only `@invariant` is registered.
> `@observable`, `@synchronized` and `@traced` raise `attr.unknown` unless a user
> `Attribute` subclass of that exact name is in scope (fixture:
> `tests/lang/compile-errors/annotation_unknown_error.ph`), and a parameter type
> annotation installs nothing. **The real as-built order** inside
> `expand_class_attributes` (attributes.rs L1527) is: class-level attributes
> (`@invariant` collects predicates; `@construct`/`@data` derive) → attribute-class
> validation → `derive_accessors` (`@get`/`@set`, L1593) → `expand_variants`
> (`@variant`, L1599) → member-level attributes (`@requires`/`@ensures` weave,
> L1602) → the `@invariant` weave across every member (L1661). The invariant weave
> running **last**, over already-woven bodies, is what produces the Eiffel
> `invariant → post → pre` nesting.

## The two rules that keep it coherent

1. **Phase order is fixed and total** (the pipeline above). A later tier wraps only
   what an earlier tier produced; there is no cross-tier ordering choice for the
   user to get wrong, and within `weave` the Eiffel `invariant → post → pre` order
   from core §D2 is unchanged.
2. **Every decorator declares `runtime`.** This is the seam that replaces
   annotations-core's *blanket* ban on runtime hooks with a *per-decorator* fact.
   The erasure test and the optimizer read the flag:
   - Strip every `runtime: false` decorator → **identical bytecode** for the
     annotated member's body (the static tiers preserve `E` in spirit; `Layout`
     is the one static tier that changes *layout*, so it is stripped structurally,
     not behaviorally).
   - A `runtime: true` decorator **voids `E` for that member only** and forces an
     inline-cache **version guard** on method redefinition ([object-model.md
     §5](../object-model.md); the IC already guards on class shape).

Rule 2 is the whole trick: purity is no longer all-or-nothing for the sigil, it is
declared per name, so static and dynamic decorators coexist without either
poisoning the other's optimization story.

> **Rule 2 is not implemented.** No registry row carries a `runtime` flag, and
> there is no erasure golden test over a `runtime: false` subset. Every built
> decorator is `runtime: false` by construction, so the rule is currently vacuous
> rather than violated — but the regression guard it describes does not exist. The
> one stripping axis that *does* exist is `CompileMode`
> (`Debug`/`Release`/`Unchecked`, attributes.rs L26-36), which is unrelated: it
> strips **contract guards** specifically. See [requires.md](requires.md).

## Interaction with dynamic typing

Under runtime-checked types, a binding/parameter annotation is an **Install-tier
decorator** — a wrapper that checks `arg.isA(T)` (or protocol conformance) on
entry and raises `TypeError` on violation. Consequences:

- Types, contracts (`@requires`/`@ensures`), and value-level decorators
  (`@memoize`) become **the same wrapper substrate** — four spec features, one
  mechanism. See [annotation-paradigm-bridges.md](../experimental/annotation-paradigm-bridges.md).
- `E` ([typing.md §5.2](../experimental/typing.md)) is *already* surrendered for
  typed members, so admitting the runtime tiers costs no additional purity — the
  bill was paid at "dynamic typing" (see typing.md §9's non-goals).
- The static erasable typing design of typing.md is **not** deleted: it remains the
  `runtime: false` reading of the same annotations, selectable per module/flag. A
  program may run its type annotations as an erased linter (typing.md) *or* as
  Install-tier runtime checks (this note); the descriptor's `runtime` flag is the
  switch.

None of this is built: Phalcom performs no type-annotation checking at any tier.

## Hazards

- **Inline-cache invalidation.** Install/Dispatch/Runtime tiers make a method
  redefinable, so a warm call site needs a version guard keyed on the holder's
  method-dictionary generation. Install-tier wrapping is otherwise IC-friendly (it
  yields one `Method`); only Runtime per-send hooks add a genuine per-call branch.
- **Erasure creep, made explicit.** The danger annotations-core avoided by fiat now
  lives in the `runtime` flag. A builtin that lies about its tier (claims
  `runtime: false` but changes behavior) breaks the strip-annotations golden test —
  so the erasure test must run over *only* the `runtime: false` subset and is the
  regression guard for the flag's honesty.
- **User-defined decorators are Install/Runtime only.** Letting user code emit at
  the Compile/Layout tiers would reintroduce the "attributes are compile-time
  metaobjects (CLOS-MOP)" end state annotations-core explicitly defers. Keep the
  static tiers compiler-owned until that ADR is taken. **This one is enforced** —
  `attr.compile_tier_reserved`, see [on.md](on.md).
- **Ordering surprises across `install` vs `runtime`.** Two decorators in the same
  later phase on one member compose in **source order, innermost-last** (the
  Python stacking convention). This is a genuine ordering the user controls (unlike
  the cross-tier order, which is fixed) and must be documented at the call site.
- **Dispatch-tier collision with hand-written `doesNotUnderstand` (D-4, resolved).**
  A class declaring both a Dispatch-tier attribute (e.g. `@Delegate`) and its own
  `doesNotUnderstand(_)` is `attr.dispatch_collision` at compile time — not
  last-wins (the Ruby `method_missing`-redefinition footgun this house style
  rejects everywhere else it recurs). Not built: there is no Dispatch tier.

## Future optimizations (not built now)

Recorded so a future implementer doesn't have to rediscover them; none of this
changes the semantics above, all of it is enabled by **D-3's resolution being
"frozen after class-definition"** ([on.md A-5](on.md)) — a chain that can never
change post-freeze needs no invalidation logic, so every item below is pure
work-hoisting (per-send cost moved to per-definition cost), not speculation:

- **Pre-compose the chain once, at class-definition time.** Instead of storing
  `Vec<Interceptor>` and looping it per send, build one fused closure
  (`traced.wrap(rateLimited.wrap(realMethod))`) exactly once when the class is
  defined, and cache *that*, not the list.
- **Cache the composed chain behind [ADR-0053](../../../adr/accepted/0053-runtime-decorator-interception-reuses-override-epoch-guard.md)'s
  `has_runtime_interceptor` guard bit.** A monomorphic call site can hold a
  direct pointer to the pre-composed chain alongside the existing `ClassId`
  check — a warm decorated call site then costs one bit-check + one direct
  jump, not an N-length walk.
- **Specialize the common `n = 1` case.** Most decorated methods carry exactly
  one Runtime interceptor, not a chain — skip the generic composition
  machinery and store the interceptor directly when there's only one.
- **Optional interceptor-declared bypass check.** An interceptor that's
  frequently a no-op (e.g. `@FeatureFlag` off) can expose a cheap "would I
  actually do anything" probe the composed chain consults first, skipping the
  full `aroundSend` body — author opt-in, not required.

## What this precludes

- **A sixth, unphased tier.** Every decorator must name one of the five phases;
  there is no "run whenever" escape. A decorator whose timing cannot be placed on
  the compile→send axis is rejected, not special-cased.
- **Per-decorator opt-out of the phase order.** The pipeline is total by
  construction; a decorator cannot request to run *before* a tier that logically
  precedes it (e.g. a Runtime hook cannot observe a method that `@construct` has
  not yet generated). This keeps composition analyzable.
- **Silent erasure loss.** Because erasability is a declared flag, no decorator can
  quietly cost runtime while presenting as static — the opposite of the Java
  annotation-processor model where inert metadata and active processors are
  indistinguishable at the use site.

## Open questions

| # | Question |
|---|---|
| D-1 | **DEFERRED** (2026-07-13): not decided now. A per-module `--decorators=static` flag would be the same *shape* of decision as `CompileMode` (`Debug`/`Release`/`Unchecked`, U-ANNOT-CONTRACTS) — revisit once that axis ships rather than bolting on a second, unrelated build-mode dimension speculatively. **Note (2026-07-15): `CompileMode` has since shipped** (attributes.rs L26-36), so this deferral's own revisit trigger has fired. |
| D-2 | Install-tier wrapping needs `Method.invokeOn(recv, *args)` and `Behavior.defineMethod(sel, block)` — ratify these as object-model surface, or keep them behind a reflection unit? **Still open; neither surface exists on HEAD.** |
| ~~D-3~~ | **RESOLVED** (2026-07-13, design-session ruling): Runtime around-send hooks **chain** (multiple `@traced`-like hooks compose), reusing the same source-order-innermost-last rule as Install. See "Future optimizations" above for the composed-chain-caching consequence. |
| ~~D-4~~ | **RESOLVED** (2026-07-13): a class that both hand-writes `doesNotUnderstand(_)` and carries a Dispatch-tier attribute is a compile error (`attr.dispatch_collision`), not silent last-wins — matches the `attr.accessor_collision` house style used throughout this spec family (`@get`/`@set`/`@construct`/`@data`) rather than Ruby's `method_missing` last-definition-wins footgun. A delegate-then-fallback proxy is still expressible — hand-write one `doesNotUnderstand` that runs the delegation logic itself — just not auto-composed. |
| ~~D-5~~ | **RESOLVED** (2026-07-13): the erasure golden test ("strip `runtime: false` → identical bytecode") is checked **per-member**, not whole-program — matches the receiver-scoped granularity already used elsewhere in this design (`@invariant`'s guard, the contract test-strategy's stripping checks). Localizes failures to the specific member whose stripping broke, and is cheaper to compute than a whole-file double-compile diff. |
