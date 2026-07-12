# Decorators (`@`) — the five-tier model

- Status: **Proposed** (experimental; not ratified — exploratory)
- Date: 2026-07-12
- Depends on: [annotations-core.md](../experimental/annotations-core.md) (the `@` mechanism, registry, phase pipeline)
- Related:
  [annotation-paradigm-bridges.md](../experimental/annotation-paradigm-bridges.md) (the method-table / layout tier line) ·
  [annotations-construct.md](../experimental/annotations-construct.md) (layout derives) ·
  [annotations-contracts.md](../experimental/annotations-contracts.md) (weave tier) ·
  [typing.md](../experimental/typing.md) (erasure invariant E, §5.2/§9) ·
  [method-lookup.md](../method-lookup.md) (`doesNotUnderstand`, `perform`) ·
  [object-model.md](../object-model.md) (metaclass tower, `Behavior`)

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
`doesNotUnderstand`-delegation prototype object uses." Under a **dynamically-typed
Phalcom** (types checked at runtime, per the working assumption for this note),
the erasure invariant `E` ([typing.md §5.2](../experimental/typing.md)) no longer
holds for typed members anyway — a type annotation *is* a per-call runtime check.
That removes the sole argument against runtime hooks and lets `@` span its whole
natural range.

This note unifies every decorator kind — compile-time, layout, dispatch, install,
runtime — under **one grammar and one registry**, distinguished by a declared
**tier** that says *when the decoration takes effect*.

## Decision

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
  static tiers compiler-owned until that ADR is taken.
- **Ordering surprises across `install` vs `runtime`.** Two decorators in the same
  later phase on one member compose in **source order, innermost-last** (the
  Python stacking convention). This is a genuine ordering the user controls (unlike
  the cross-tier order, which is fixed) and must be documented at the call site.

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
| D-1 | Does the `runtime` flag live per-decorator only, or also per-*module* (a `--decorators=static` build that rejects Install/Runtime tiers wholesale)? |
| D-2 | Install-tier wrapping needs `Method.invokeOn(recv, *args)` and `Behavior.defineMethod(sel, block)` — ratify these as object-model surface, or keep them behind a reflection unit? |
| D-3 | Do Runtime around-send hooks compose as a chain (multiple `@traced`-like hooks) or is at most one interceptor per class allowed? |
| D-4 | Should Dispatch-tier `@delegate` and a user `doesNotUnderstand` coexist (delegate first, hand-written fallback second), or is declaring both an error? |
| D-5 | Exhaustiveness of the erasure golden test: is "strip `runtime: false` → identical bytecode" checked per-member or whole-program? |
