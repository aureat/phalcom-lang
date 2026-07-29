# `@On` and the `Attribute` reflection layer

- Status: **Implemented** — the `Attribute` root, `@On`, tier/hook validation, and
  retention+reflection are built. **The hooks are validated but never dispatched**;
  see "Not built" below.
- Unit: M-ATTR-ROOT (ratified 2026-07-14 per
  [ADR-0054](../../../adr/accepted/0054-two-speed-ratification-annotation-decorator-tiers.md)
  §2(b); A-1–A-5 resolved inline 2026-07-13, A-6 deferred to v0.3 non-blocking)
- Evidence: `phalcom-core/src/compiler/attributes.rs` — `OnExpander` (L607-632),
  registered at L650; `validate_attribute_class` (L1433-1490); `RESERVED_HOOKS`
  (L1405-1406); `TIER_NAMES` (L1413); `resolves_to_attribute_class` (L1386).
  Retention: `phalcom-core/src/primitive/attribute.rs`
  (`Object#__attach`/`__attributes`/`__freezeAttributes`);
  `phalcom-core/src/compiler/lib/class_decl.rs` `emit_attribute_attach` (L818),
  `emit_member_attribute_attaches` (L788), `is_attribute_class` (L64).
  Reflection: `phalcom-core/core/core.ph` L1605-1620
  (`Behavior#attributes`/`attributesOfType(_)`, `Method#attributes`/`attributesOfType(_)`).
  Fixtures: `phalcom-core/tests/lang/decorators/decorators_attribute_retention.ph`
  (PASS), `tests/lang/runtime-errors/runtime_attribute_store_frozen.ph`.
- Date: 2026-07-12 (A-1–A-5 resolved 2026-07-13)
- Depends on:
  [README.md](README.md) (the five-tier model; this note reifies its descriptor) ·
  [annotations-core.md](../experimental/annotations-core.md) (the `@` mechanism, registry, phase pipeline) ·
  [legality-grammar.md](../../../work/pending/ctor/notes/legality-grammar.md) (`Target`, the legality table)
- Related:
  [object-model.md](../object-model.md) (metaclass tower, `Behavior`, the `§8` metaobject surface) ·
  [method-lookup.md](../method-lookup.md) (`doesNotUnderstand`, `perform`) ·
  [typing.md](../experimental/typing.md) (erasure invariant `E`, §5.2/§9)

## Not built — read this first

The mechanism below is specified at ratification depth. What actually runs on HEAD:

**Built.** The `Attribute` root; `@On` as a registered `Class`-target attribute;
`validate_attribute_class`'s three errors (`attr.compile_tier_reserved`,
`attr.missing_hook`, `attr.undeclared_hook`); silent retention of any name that
resolves to an `Attribute` subclass (`resolves_to_attribute_class`, a `class_parents`
chain-walk); `Name.new(args)` + `__attach(_)` codegen at class-definition time for
both class-level and member-level attributes; the frozen store (A-5) enforced at
runtime as `attr.frozen`; and `Behavior#attributes`/`Method#attributes`/
`attributesOfType(_)` reflection. Fixture `decorators_attribute_retention.ph` passes.

**Not built — four divergences from the surface specified below:**

1. **`tier:` is not a labeled argument.** The parser's attribute-arg-list grammar has
   no label form (see `docs/forge/DEFERRED.md`), so `@On(Method, tier: Install)` —
   the surface this whole note specifies — **does not parse**. As built, tier is a
   **bare positional** argument matched by name against `TIER_NAMES` (L1413, L1435-1440):
   `@On(Method, Install)`. Every `@On(..., tier: X)` example below is aspirational.
2. **No hook is ever dispatched.** `wrap(_)`, `resolveMissing(_)`, `aroundSend(_)`,
   `expand(_)` and `finalizeLayout(_)` appear **only** in `RESERVED_HOOKS` — as names
   to validate, never as selectors to send. A class declaring `@On(Method, Install)`
   with a `wrap(m)` compiles and its instance is retained, but `wrap` is never called
   and nothing is wrapped. The tier declaration is a *validated claim*, not behavior.
   Consequently `Method.fromBlock`, `Method.invokeOn`, `Behavior.defineMethod`,
   `reserveSlot`/`slotAt`/`setSlotAt` and the `Invocation` object do not exist.
3. **`inherited:` (A-2) is unimplemented.** `validate_attribute_class` reads only the
   tier name from `@On`'s args; `attributesOfType(cls)` in `core.ph` (L1612) is a flat
   `self.__attributes.filter { a => a.isA(cls) }` with **no superclass chain-walk**.
   `inherited: true` has no effect.
4. **Validation only reaches *direct* subclasses.** `is_attribute_class` is
   `superclass.name == "Attribute"` (class_decl.rs L64) — a *direct* `is
   Attribute` only. A transitive subclass (`class B is A`, where `A is
   Attribute`) is retained correctly (retention walks the full chain via
   `resolves_to_attribute_class`) but is **never validated** — it can declare a
   reserved tier or implement a bare hook with no diagnostic.

## Context

[README.md](README.md) models `@` as a five-tier spectrum and gives the
descriptor as a **Rust-side registry row**:

```rust
struct Decorator {
    name:    String,
    tier:    Tier,            // Compile | Layout | Install | Dispatch | Runtime
    runtime: bool,
    apply:   TierHook,        // a phase-appropriate callback
}
```

That row is the metaobject the `@` sigil resolves to. In a Smalltalk-style
language with first-class classes and message-send dispatch, a "descriptor that
names a behavior and closes over state" is not a struct — it is an **object**.
This note reifies the descriptor: an attribute *is* a class extending a core
`Attribute` root; `@Name(args)` at a use site *is* the constructor send
`Name.new(args)`; the tier hook *is* a method the attribute instance implements.

This is the .NET model (`class FooAttribute : Attribute` + `[Foo("x")]`) rendered
in Phalcom's object model, and it subsumes two spec features at once:

- the **five active tiers** of [README.md](README.md) (the hook methods), and
- the **passive-metadata** model of Java/C# annotations (an attribute class with
  *no* hook method is inert, retained, and reflectable) — the case
  [README.md](README.md) "What this precludes" steers away from as a
  *blanket* rule, admitted here as a *declared, honest* case.

The design does not widen what user code may do: it lands exactly on the
user/compiler tier line [README.md](README.md) already draws (users own
Install/Dispatch/Runtime; the compiler owns Compile/Layout).

## Decision

An **attribute** is a class that is the core `Attribute` root. Using it as
`@Name` or `@Name(args)` at a legal target:

1. **instantiates** it — `@Name(args)` desugars to `Name.new(args)` (bare `@Name`
   to `Name.new`), a normal constructor send;
2. **dispatches its tier hook**, if it implements one, against the artifact the
   current phase hands it (the class member list, the reified `Method`, the DNU
   slot, the around-send chain); and
3. **retains the instance** on the decorated artifact's annotation list, where
   reflection can read it back.

The Rust `Decorator` row maps field-for-field onto the class:

| `struct Decorator` field | Attribute-class equivalent |
|---|---|
| `name` | the class name |
| `tier` | **declared explicitly**, `@On(Target, tier: Install)` (A-1, resolved — no inference from hook shape) |
| `runtime` | **inferred** from the tier (Runtime/Dispatch/Install-with-behavior ⇒ `true`) |
| `apply: TierHook` | an **overridable method** on the instance (`expand(_)` / `wrap(_)` / `resolveMissing(_)` / `aroundSend(_)`) |

### A-1 — tier is declared, not inferred (resolved 2026-07-13)

Earlier drafts of this note inferred tier from `respondsTo(_)` over the hook
selectors, with an escape-hatch `@tier(#install)` for classes implementing more
than one hook. Both the pure-inference and the `@tier(#install)` designs are
**superseded**: tier is now **always** declared explicitly, folded into the same
class-side decorator that already carries legality (`@AttributeUsage`, itself
renamed `@On` — see below), and a class may declare **at most one tier** (v0.2;
multi-tier-per-class is deferred to v0.3, see "What this precludes").

```phalcom
@On(Method, tier: Install)
class Memoize is Attribute {
  var _cache
  @constructor
  new() { _cache = Map.new() }
  wrap(m) { ... }
}
```

- `Install`/`Dispatch`/`Runtime` (and the builtin-only `Compile`/`Layout`) are
  **first-class singleton objects**, not symbols — same pattern Phalcom already
  uses for `Bool`'s `True`/`False` — so `tier: Install` reads as an ordinary
  object reference, not a stringly-typed tag.
- **Correctness floor, both directions:** a declared tier with no matching hook
  implemented is `attr.missing_hook`; an implemented hook selector (`wrap`,
  `resolveMissing`, `aroundSend`, `expand`, `finalizeLayout`) with **no**
  declared tier is `attr.undeclared_hook` — these selector names are reserved on
  `Attribute` subclasses specifically so a same-named method written for an
  unrelated reason can't silently be drafted into a tier.
- A class implementing hooks for two different tiers is rejected at
  definition — split into two `Attribute` subclasses and stack them at the use
  site (`@Foo @Bar`), composing under the existing source-order,
  innermost-last rule. See "What this precludes."

### The `Attribute` root and the hook protocol

`Attribute` is a builtin core class. Its subclasses opt into a tier by
**declaring it explicitly** (`@On(Target, tier: ...)`, A-1) and implementing the
corresponding hook selector. A class with no tier declaration and no hook
selector is inert metadata:

| Hook selector | Tier | Signature (informal) | Fires |
|---|---|---|---|
| `expand(_)` | Compile | `expand(aClassDef) => members` | compiler pass (builtin-only, see below) |
| `finalizeLayout(_)` | Layout | reserve/read slots | compiler finalize (builtin-only) |
| `wrap(_)` | Install | `wrap(aMethod) => aMethod` | class-definition time, once |
| `resolveMissing(_)` | Dispatch | `resolveMissing(aSelector) => aMethodOrNone` | lookup miss |
| `aroundSend(_)` | Runtime | `aroundSend(anInvocation) => result` | every send |
| *(none)* | — | — | never; instance is pure retained metadata |

```phalcom
// class-side legality + tier — replaces the Rust `Target` table row
@On(Method, tier: Install)
class Memoize is Attribute {
  var _cache
  @constructor
  new() { _cache = Map.new() }

  // tier: Install declared above; must implement wrap(_) or attr.missing_hook
  wrap(m) {
    return Method.fromBlock { args =>
      return _cache.at(args).orElse {
        Some.new(m.invokeOn(self, args))
      }.unwrap
    }
  }
}
```

At a use site the attribute is instantiated at the decorated artifact's
definition time, its hook is dispatched, and the instance is retained:

```phalcom
class Fib {
  @Memoize
  fib(n) { return n < 2 ? n : self.fib(n - 1) + self.fib(n - 2) }
}

Fib.methodNamed(#fib).attributes    // => [aMemoize]   (C# GetCustomAttributes)
```

### Passive metadata: the honest Java/C# case

An attribute class that implements **no** hook selector changes no behavior. It is
instantiated, retained, and reflectable — nothing more. This is the Java/C#
annotation, expressed with zero new syntax:

```phalcom
@On(Class)
class Author is Attribute {
  var _name
  @constructor
  new(name:) { _name = name }   // no tier declared, no hook => inert metadata only
  name => _name
}

@Author(name: "Ada")
class Engine {}

Engine.attributesOfType(Author).first.name    // => "Ada"
```

Because it declares no hook, its `runtime` flag is `false` in the behavioral
sense: stripping it leaves the class's method bodies bytecode-identical (only the
retained instance disappears). It satisfies the erasure golden of
[README.md](README.md) §"two rules" the same way any other
`runtime: false` decorator does.

### `@On` — legality and tier as one class-side declaration (renamed, A-1)

The [legality-grammar.md](../../../work/pending/ctor/notes/legality-grammar.md)
`Target` table is driven, for attribute classes, from a single class-side
`@On(target…, tier: ..., inherited: ...)` declaration rather than a Rust `match` —
this **replaces** the earlier two-decorator design (`@AttributeUsage(...)` +
`@tier(...)`) with one:

```
enum Target { Class, Method, Getter, Setter, Field, Module }   // + Module, per next/module tier
```

- `target…` — positional, as before (`Method`, `Class`, ...).
- `tier:` — optional labeled arg (A-1); a first-class `Tier` singleton object
  (`Install`/`Dispatch`/`Runtime`; `Compile`/`Layout` are builtin-only, see the
  compile-time boundary below). Omitted ⇒ the class is passive metadata and must
  implement no hook selector.
- `inherited:` — optional labeled arg, default `false` (A-2); when `true`,
  `attributesOfType(_)` walks the superclass chain (single inheritance, no
  diamond to reason about) instead of reading strictly per-artifact.

`@On` is itself a builtin attribute (the recursion bottoms out at the
`Attribute` root, whose usage is fixed in Rust). An attribute applied to an
illegal target raises the same expansion-time, table-citing error the grammar note
specifies — the table row is now read off the attribute class's own `@On`
declaration.

## The compile-time boundary (the one hard limit)

Which tier an attribute class can occupy is gated by **when its instance can
exist**:

- **Install / Dispatch / Runtime** fire *at or after* class-definition time. The
  object world is already up, so `Name.new` runs and its hook dispatches against a
  reified `Method` or send. **User attribute classes live here.** ✅
- **Compile / Layout** fire *inside the compiler pass*, before the world is built.
  Instantiating a user attribute there (`Name.new().expand(…)`) requires **staged
  compile-time evaluation** — running user VM code during compilation, the
  CLOS-MOP end-state [annotations-core.md](../experimental/annotations-core.md)
  and [README.md](README.md) explicitly defer.

This boundary is not new policy: it *coincides* with the user/compiler tier line
[README.md](README.md) already fixed ("user-defined decorators are
Install/Runtime only; the static tiers stay compiler-owned"). So the two static
tiers remain **Rust-owned builtins** with `expand(_)`/`finalizeLayout(_)` hooks the
compiler calls directly (no instantiation of a user class); the `Attribute`-class
mechanism is the **user-facing** surface at Install and beyond. Compile/Layout
builtins *may* still be authored as `Attribute` subclasses for uniform reflection,
but their hook is a native method, not user Phalcom evaluated at compile time.

**A-3, resolved 2026-07-13 — enforcement is immediate, not deferred.** Because
tier is now an explicit declaration (A-1), the compiler doesn't need to inspect a
user class's body at all to enforce this boundary: any non-builtin `@On(...,
tier: Compile)` or `@On(..., tier: Layout)` is rejected the moment that
declaration is compiled — `attr.compile_tier_reserved`, citing that these two
tiers are compiler-native only. This is a cheap, precise check at the attribute
class's own definition site, not a runtime/instantiation-time discovery.

A user attribute's **constructor may run arbitrary code** — no restriction to
literal/const args (unlike Java/C# annotation-value rules). Install/Dispatch/
Runtime already fire at ordinary class-definition-time, as normal program
execution (the object world is already up); there is no "observe half-built
program state" hazard to guard against beyond the same forward-reference rules
that already govern any other class instantiation in the file. This is also
already assumed by this note's own worked examples (`Memoize`'s
`_cache = Map.new()`, `Synchronized`'s `_lock = Lock.new()`) — restricting
constructor args would break them for no corresponding safety gain.

## Bootstrap in `core.ph`

The `Attribute` root and `@On` follow the same reopen-a-native-class
pattern `Error`/`Option` already use (core.ph): a native `_attributes` slot and
the `attach`/`attributes` machinery live in Rust; the `.ph` skeletons make the
class names surface-visible and carry the derivable accessors.

```phalcom
// Root. Every attribute extends this. Usage fixed in Rust at the root.
class Attribute {}

// Builtin attribute carrying legality + tier (recursion bottoms out here).
class On is Attribute {
  var _targets
  var _tier        // None (passive) | Install | Dispatch | Runtime | Compile | Layout (builtin-only)
  var _inherited    // default false (A-2)
  @constructor
  new(targets, tier: None, inherited: false) {
    _targets = targets
    _tier = tier
    _inherited = inherited
  }
  targets => _targets
  tier => _tier
  inherited => _inherited
}
```

The retained-instance store is a native `_attributes` slot on the class object,
`Method`, and `ModuleObject`; the read API is `.ph`-derivable over the floor
(`isA` already exists, core.ph):

```phalcom
class Method {
  attributes => _attributes                                  // native slot, Rust-backed
  attributesOfType(cls) => _attributes.filter { a => a.isA(cls) }   // pure .ph
}
```

**What the compiler lowers.** `@Deprecated(reason: "use v2")` on a member desugars,
once, at the enclosing class's definition time:

```phalcom
// synthesized when the class is defined:
let _a = Deprecated.new(reason: "use v2")    // @Name(args) => Name.new(args)
theMethod.attach(_a)                          // append to the method's _attributes
// Deprecated implements no hook selector => nothing else runs
```

An **active** attribute emits the same two lines *plus* its hook dispatch — e.g.
`@Synchronized` additionally runs `theMethod.replaceWith(_a.wrap(theMethod))`. The
retained instance and the behavioral wrap are independent: passive attributes stop
after `attach`; active ones also fire their hook.

## The install surface is `Behavior`-side, never per-instance

`Behavior.defineMethod` — the metaobject call an Install/Dispatch attribute uses to
re-install its wrapped method — is **`Behavior` protocol**, so it lives on class
objects and metaclasses only, never on ordinary instances. This is a direct
consequence of where methods live: a method sits in a **class's** dictionary
(`ClassObject.methods`, class.rs), dispatch always starts from `receiver.class` and
walks the superclass chain (`lookup_method_in_hierarchy`), and an instance row has
slots/fields but **no method dictionary** to define into. `Behavior` — the
superclass of `Class` and `Metaclass` ([object-model.md](../object-model.md), the
`Behavior` row: *"method dictionary, superclass, name, allocation, reflection"*) —
owns exactly this protocol, inherited only by the things that *have instances*.

Instance-side vs class-side is *which* `Behavior` receives the call, not a separate
API:

```phalcom
Point.defineMethod(...)         // Point is-a Behavior => instance-side method (all Points see it)
Point.class.defineMethod(...)   // the metaclass, also a Behavior => class-side (static) method
aPoint.defineMethod(...)        // aPoint is NOT a Behavior => does NOT respond
```

Consequences for attributes:

- An attribute's `wrap(_)`/`resolveMissing(_)` hook re-installs onto the **holder
  class's** dictionary via `Behavior.defineMethod`, never onto an instance. This is
  coherent with retention: `attach` and the hook both act on `Method` / class /
  `ModuleObject` — all `Behavior`-reachable artifacts — so an attribute never needs
  to reach a bare instance to do its job.
- `Method.invokeOn(recv, args)` is the one surface that *takes* an instance, but it
  is **`Method` protocol, not `Behavior`** — a reified method invoked *against* a
  receiver, not a definition on it. Attributes call it to run the original method;
  they never define through an instance.
- **Per-instance behavior is out of scope for v0.2 — but not foreclosed.** Giving a
  single `aPoint` its own method needs a *different* mechanism — a per-instance
  method dictionary (prototype model) or `doesNotUnderstand` delegation (the
  Dispatch tier, [README.md](README.md)). Phalcom's v0.2 floor gives
  instances no dictionary, so `Behavior.defineMethod` deliberately does not span
  instances; an attribute that wants per-receiver *state* (not behavior) takes the
  Layout-slot route instead (see `@lazy` below), which is builtin-owned.

### Deferred to v0.3: reflection directly on instances/objects

The `Behavior`-only restriction above is a **v0.2 scoping choice, not a permanent
one.** A future revision may admit reflective mutation directly on an ordinary
object — `anObject.defineMethod(sel, block)`, `anObject.respondsTo(sel)` resolving
against per-instance behavior, `anObject.become(...)`, per-object attribute
attachment — so that a single receiver can carry its own methods without touching
its class. This is the prototype/singleton-method end of the design space (Ruby
singleton methods, JS own-properties, Self/NewtonScript prototypes).

It is **deferred to v0.3** because it requires a floor change v0.2 has not taken:
an instance must gain an *optional per-object method dictionary* (or the object
model must admit implicit per-instance metaclasses, the Smalltalk route). Both
touch allocation, dispatch's start-of-lookup, and the inline-cache shape (a warm
site keyed on class shape must now also guard on the presence of an own-method
dict). Until that floor exists:

- v0.2 keeps `defineMethod` and the reflective install/attach surface strictly on
  `Behavior` (classes and metaclasses).
- User attribute classes therefore decorate class members only; there is no
  attribute that installs behavior onto one instance.
- When v0.3 lands per-instance behavior, attribute *retention* generalizes for free
  (the `_attributes` store can hang off any object), and an Install-tier hook could
  gain an `aReceiver`-scoped variant — but that is an explicit v0.3 ADR, not an
  implicit consequence of this note.

This defers alongside the other v0.3 items (e.g. the `?:` operator,
[iteration Route B ADR-0048](../../../adr/accepted/0048-amend-iteration-bare-cursor-sentinel-and-iterable-root.md));
it is recorded here so the v0.2 `Behavior`-only line is understood as a floor
limitation, not a design rejection.

## Worked examples

All three are Install-tier: a `wrap(m)` returning a fresh `Method` that closes over
state. The organizing constraint is **where that state lives** — it decides
user-definable vs builtin:

| State scope | Lives in | User-definable? |
|---|---|---|
| stateless / per-call | the call frame | ✅ pure user Install/Runtime |
| per-method / per-class | the **attribute instance** (created once at class-def, shared by all receivers) | ✅ user Install — semantics are class-wide |
| **per-receiver** | a reserved **slot on the receiver** | ❌ needs Layout → builtin-owned |

`Method.fromBlock`/`.invokeOn`/`.replaceWith`/`Lock`/`.slotAt` below are the
proposed [object-model.md §8](../object-model.md) metaobject surface, not yet
built; the surrounding *language* syntax is current.

**`@retry`** — stateless, pure user Install. Uses `Block#on(_)` (U-ERR, core.ph)
and non-local `return` out of the wrapper on first success:

```phalcom
@On(Method, tier: Install)
class Retry is Attribute {
  var _times
  @constructor
  new(times:) { _times = times }

  wrap(m) {
    return Method.fromBlock { args =>
      var attempt = 0
      while (attempt < _times) {
        { return m.invokeOn(self, args) }.on(Error) { e =>   // success => returns from wrapper
          attempt = attempt + 1
          (attempt >= _times).ifTrue { throw e }             // exhausted => rethrow
        }
      }
    }
  }
}
```

**`@synchronized`** — lock in the attribute instance ⇒ **class-wide** mutual
exclusion (every receiver serializes on the one lock). Per-*instance* locking would
put the lock on `self`, a reserved slot — row 3, so it becomes a builtin:

```phalcom
@On(Method, tier: Install)
class Synchronized is Attribute {
  var _lock
  @constructor
  new() { _lock = Lock.new() }

  wrap(m) {
    return Method.fromBlock { args =>
      return _lock.critical { m.invokeOn(self, args) }
    }
  }
}
```

**`@lazy`** — **builtin**: the compute-once cache must live per receiver, so it
needs a Layout-reserved slot (`finalizeLayout`, a builtin-only hook). A user `@lazy`
could only cache in the shared attribute instance — class-level, not per-object:

```phalcom
// BUILTIN — per-receiver storage is a Layout concern (cf. @observable, decorators.md)
// tier: Layout is compiler-reserved (A-3) — this class could not be authored in
// user .ph source; a user attempt would hit attr.compile_tier_reserved.
@On(Getter, tier: Layout)
class Lazy is Attribute {
  finalizeLayout(field) { field.reserveSlot(#_lazy) }   // Layout hook (builtin-only)

  wrap(getter) {
    return Method.fromBlock {
      self.slotAt(#_lazy).ifNone { self.setSlotAt(#_lazy, getter.invokeOn(self)) }
      return self.slotAt(#_lazy).unwrap
    }
  }
}
```

The rule is invariant: the moment a decorator needs storage *on the object it
decorates*, it crosses from user Install into builtin Layout.

## What it needs (all additive; no new `Value` arm, no VM primitive)

| Piece | Detail |
|---|---|
| Core `Attribute` root | the class every attribute extends; usage fixed in Rust at the root. |
| `@On(target…, tier:, inherited:)` | builtin attribute carrying legality **and** tier (A-1) in one declaration; drives the `Target` table and the tier/inheritance floor. |
| Annotation store | a `Vec<Value>` (retained instances) on the class object, `Method`, and `ModuleObject` — small next to the existing method dictionaries / `name_to_slot`. **Frozen after class-definition (A-5)** — no post-definition attach/detach; attempting to mutate it is an error. |
| Reflection surface | `Behavior.attributes`, `Method.attributes` / `Method.attributesOfType(_)`, mirrored on modules — message-sends returning the stored instances. Gated on the [object-model.md §8](../object-model.md) metaobject surface (`Behavior.defineMethod`, `Method.invokeOn`, `Method.bind`), the same gate as [README.md](README.md) D-2. |
| Tier declaration | explicit only (A-1) — `tier:` on `@On`, no inference from hook shape. Declared tier without the matching hook ⇒ `attr.missing_hook`; a reserved hook selector present without a matching declared tier ⇒ `attr.undeclared_hook`. A class may declare at most one tier in v0.2 (multi-tier deferred to v0.3). |
| Dedup | two applications of the same attribute class on one member are legal and compose normally (A-4) — same source-order, innermost-last rule as any two distinct attributes; no enforced uniqueness. |

## Hazards

- **Hook-selector collisions — resolved (A-1).** An attribute implementing hook
  selectors for two different tiers (`wrap(_)` *and* `aroundSend(_)`) is a
  compile error in v0.2, not a "which tier wins" guess: `@On` declares **one**
  tier, and only that tier's hook may be implemented. A decorator needing both
  Install and Runtime behavior is two `Attribute` subclasses, stacked at the use
  site (`@Foo @Bar`), sharing state via a plain companion object if needed.
  Multi-tier-per-class is deferred to v0.3 (see "What this precludes").
- **Frozen retention — resolved (A-5).** The retained-attribute store is
  immutable once the class is defined; there is no reflective attach/detach
  after that point (no monkey-patching a decorator on later). Attempting to
  mutate it is an error. This keeps [ADR-0053](../../../adr/accepted/0053-runtime-decorator-interception-reuses-override-epoch-guard.md)'s
  `has_runtime_interceptor` bit valid as a one-time, never-invalidated flag —
  admitting mutation would force it into a full epoch counter, a real VM cost
  ADR-0053 explicitly priced but did not build. Deferred to v0.3 alongside
  runtime class-hierarchy mutation (open-Q4) — the same gate, not paid twice.
- **Retention cost.** Every retained attribute is a live heap object reachable
  from its artifact. A `@Author("…")` on ten thousand classes is ten thousand
  instances. Keep the store a plain `Vec<Value>` (empty and unallocated for
  un-annotated artifacts) so the common case is a null pointer, not a map.
- **`runtime` honesty, again.** An attribute that implements `wrap(_)` but returns
  the method *unchanged* while recording state is behaviorally inert yet claims the
  Install tier. The erasure golden ([README.md](README.md) §"two rules")
  is still the regression guard: stripping every `runtime: false` attribute must
  leave method bodies bytecode-identical. An attribute lies iff that golden breaks.
- **Instantiation order at one site.** Two attributes on one member instantiate
  and compose in **source order, innermost-last** — the same Python-stacking rule
  [README.md](README.md) fixes for `install`/`runtime`. `Name.new` side
  effects (a constructor that registers globally) therefore run in written order;
  document it at the call site.
- **Bootstrap of `Attribute` itself.** The root class and `@On` must
  exist before any attribute (including `@On`) is used. `Attribute` and
  its usage are defined in Rust / early `core.ph`, before the user surface opens —
  the same bootstrap discipline the metaclass tower already follows.

## What this precludes

- **User attribute classes at Compile/Layout.** Foreclosed until staged
  compile-time evaluation is ratified (see the boundary section). A user attribute
  declaring `tier: Compile`/`tier: Layout` is `attr.compile_tier_reserved` at
  its own definition site (A-3), not silently promoted to or from a builtin.
- **A tier with no hook and no metadata role.** An attribute either implements a
  hook selector (active, one of five tiers) or is pure retained metadata. There is
  no third thing — no "runs at some unphased time" escape, preserving the
  [README.md](README.md) "no sixth tier" guarantee.
- **Reflection without the metaobject surface.** `attributesOfType(_)` and friends
  do not ship before [object-model.md §8](../object-model.md) is ratified; until
  then attribute *hooks* can fire (compiler-driven at definition time) but the
  *retained-instance read API* stays behind the same gate as D-2.
- **Multi-tier on one `Attribute` subclass.** A class declares exactly one tier
  in v0.2 (A-1); implementing hooks for two is rejected, not composed. Deferred
  to v0.3 alongside A-6.
- **Mutating the retained store after class-definition.** Frozen once the class
  is defined (A-5); no reflective attach/detach later. Deferred to v0.3,
  grouped with runtime class-hierarchy mutation (open-Q4) — the same floor
  change would be needed for either.

## Open questions

| # | Question |
|---|---|
| ~~A-1~~ | **RESOLVED** (2026-07-13): explicit declaration, always — `@On(Target, tier: Install)`, folded into the same decorator as legality (renamed from `@AttributeUsage`). No inference from `respondsTo(_)`; a class declares at most one tier (multi-tier deferred to v0.3). Reserved hook-selector names + `attr.missing_hook`/`attr.undeclared_hook` close the silent-misclassification and vocabulary-growth-breakage risks pure inference had. |
| ~~A-2~~ | **RESOLVED** (2026-07-13): per-attribute-class `inherited:` labeled arg on `@On`, default `false` (matches C#/Java's opt-in-false convention). Chain-walk is single-inheritance-simple, no diamond case. |
| ~~A-3~~ | **RESOLVED** (2026-07-13): **arbitrary code allowed** — no literal/const restriction. Install/Dispatch/Runtime already fire at ordinary class-definition-time (the object world is up); restricting would break this note's own worked examples for no matching hazard. Compile/Layout tiers stay compiler-owned, now enforced as an immediate `attr.compile_tier_reserved` compile error at the attribute class's own definition site (cheap, thanks to A-1's explicit tier). |
| ~~A-4~~ | **RESOLVED** (2026-07-13): **allowed, compose normally** — two `@Memoize` on one member is legal (double-wraps, wasteful not wrong), reusing the existing source-order-innermost-last composition rule rather than adding a dedup/`@Repeatable` mechanism. Stricter enforcement is addable later as a lint without breaking existing code. |
| ~~A-5~~ | **RESOLVED** (2026-07-13): **frozen** after class-definition; mutating the retained store is an error. Keeps [ADR-0053](../../../adr/accepted/0053-runtime-decorator-interception-reuses-override-epoch-guard.md)'s one-time `has_runtime_interceptor` bit valid without a redesign. Deferred to v0.3, grouped with class-hierarchy mutation (open-Q4). |
| A-6 | **(v0.3, still open)** When per-instance behavior lands (`anObject.defineMethod`, per-object dict / implicit metaclass), does an Install hook gain an `aReceiver`-scoped variant, and does `_attributes` retention hang off arbitrary objects — or does the `Behavior`-only surface stay even then? Requires its own v0.3 floor ADR. Grouped with A-5's mutability deferral and open-Q4 — likely one v0.3 design session, not three. |
