# PLAN-DECORATORS — the `Attribute`/decorator mechanism (Install/Dispatch/Runtime/Layout) + the 8 named decorators

_Architect plan. Dependency-ordered, write-set-annotated units for one implementer each. Grounded in
**[ADR-0054](../adr/0054-two-speed-ratification-annotation-decorator-tiers.md)** (Accepted — ratifies
all five tiers), **[ADR-0052](../adr/0052-invariant-reentrancy-scope-and-layout-confined-decorator-state.md)**
(receiver-scoped guard — LANDED; Layout-confined per-receiver state),
**[ADR-0053](../adr/0053-runtime-decorator-interception-reuses-override-epoch-guard.md)** (Runtime
interceptor guard bit), **[ADR-0057](../adr/0057-decorator-granularity-vs-proxy-granularity-split.md)**
(decorator vs `Proxy` split), and the normative specs
[attribute-classes.md](../spec/current/decorators/on.md),
[decorators.md](../spec/current/decorators/README.md),
[decorators-behavioral.md](../spec/current/drafts/decorators-behavioral.md),
[decorators-dispatch-observability.md](../spec/current/drafts/decorators-dispatch-observability.md),
[decorators-observable.md](../spec/current/drafts/decorators-observable.md)._

---

## 0. Grounding — what already exists on HEAD (verified 2026-07-13, do not re-plan)

| Piece | State | Evidence |
|---|---|---|
| `@` compile-time mechanism: `Attribute` AST node, parser attr-collection loop, `compiler/attributes.rs` (`AttributeExpander`/`AttributeRegistry`/`Target`/`ExpandCtx`/`CompileMode`), `@requires`/`@ensures`/`@invariant` | **LANDED** (U-ANNOT-CONTRACTS) | `phalcom-core/src/compiler/attributes.rs` (`expand_class_attributes`, `RequiresExpander`/`EnsuresExpander`/`InvariantExpander`) |
| ADR-0052 receiver-scoped invariant guard: `VM::checking`, `FiberObject::checking`, `__invariant*` primitives, fiber-switch swap | **LANDED** — skip | `heap/fiber.rs` L117/137/158, `vm/mod.rs` L166–175 |
| `Method#invokeOn(_,_)`, `Method#bind(_)`, `Method#selector`/`holder`, `BoundMethod`, `VM::invoke_method_object` | **LANDED** (U-CORE-3) | `primitive/method.rs` L41/60, `vm/send.rs` L221 |
| `Fiber` (bare: new/call/try/yield/current/abort) — the substrate `Monitor`/`Backoff.waitBefore` need | **LANDED** (U-FIBER) | `primitive/fiber.rs`, `heap/fiber.rs` |
| `Map`, `System`, `doesNotUnderstand` miss path | **LANDED** | `core.ph` L455/L760/L247 |

**Absent — new surface this plan builds:**
- `Attribute` root class, `@On` builtin attribute, the `Install`/`Dispatch`/`Runtime`/`Compile`/`Layout`
  tier singletons, the `_attributes` retention store + reflection API, tier-declaration diagnostics
  (`attr.missing_hook`/`attr.undeclared_hook`/`attr.compile_tier_reserved`). (`grep class Attribute|On` → none.)
- `Method.fromBlock` (`method_class_new` currently **rejects** direct creation — `primitive/method.rs` L21–23)
  and `Behavior.defineMethod` (`grep define_method` → none) — the object-model §8 install surface.
- Layout reserved-slot primitives `reserveSlot`/`slotAt`/`setSlotAt` + the `finalizeLayout(_)` builtin hook.
- Runtime interceptor: `has_runtime_interceptor: bool`, the `aroundSend(_)` chain, the `Invocation` object
  (`grep has_runtime_interceptor|around_send` → none).
- Dispatch `resolveMissing(_)` hook.
- Reactivity substrate `Signal`/`Computed`/`Effect`/`Reactive` (`grep class Signal` → none; reactivity.md is
  **Proposed/exploratory, not ratified**).
- Support classes: `Monitor`, `Backoff`, `Tracer`+`Tracer.stdout`, `Flags`, `OffBehavior`, `FeatureDisabled`,
  `Pair`, `Clock`, `Map#evictOldest` (`grep` → none).
- `FieldDef` / `ClassMember::Field` / `Target::Field` — **NOT landed** (U-ANNOT-LAYOUT unlanded); gates any
  `Field`-targeted decorator (`@delegate`, `@observable`).

**Standing hazards:** `core.ph` and `universe/primitives.rs` are single-writer chokepoints (per
`phalcom-concurrent-session-hazards` memory + `docs/forge/STATE.md`). `vm/send.rs` is the dispatch spine.
Every decorator that is "just a `.ph` class" still contends on `core.ph` — this is the dominant
serialization constraint in the wave schedule (§4).

---

## 1. Mechanism units (the substrate — built first)

### M-ATTR-ROOT — `Attribute` root, `@On`, tier singletons, retention store, reflection

- **Goal.** The reified-descriptor layer: a builtin `Attribute` root class every attribute extends; the `@On`
  builtin attribute carrying legality + tier (A-1); the `Install`/`Dispatch`/`Runtime`/`Compile`/`Layout`
  tier singleton objects; a native `_attributes: Vec<Value>` retention slot on `ClassObject`/`MethodObject`/
  `ModuleObject`; the reflection API (`Behavior.attributes`, `Method.attributes`/`attributesOfType(_)`,
  module mirror); tier-declaration validation with `attr.missing_hook`/`attr.undeclared_hook`/
  `attr.compile_tier_reserved`; and the compiler lowering `@Name(args) ⇒ Name.new(args); artifact.attach(_a)`
  for the passive-metadata case. Satisfies attribute-classes.md §"Decision"/§"`@On`"/§"Bootstrap"/§"What it
  needs" (A-1–A-5), ADR-0054 §2(b).
- **Depends on.** U-ANNOT-CONTRACTS registry — **corrected 2026-07-13: NOT landed as usable.**
  `phalcom-core/src/compiler/attributes.rs`'s `expand_class_attributes`/`AttributeRegistry`/`ExpandCtx`
  exist (commit `dc01b07`) but have **zero call sites** anywhere in the workspace — `compile_class`
  (`compiler/lib/class_decl.rs`) parses `@Name(...)` into the AST and then silently drops it at lowering.
  Confirmed by a red baseline: `cargo test -p phalcom-core --test lang` fails 6 groups
  (`compile_errors`, `concurrency`, `errors`, `indexing`, `indexing_negative`, `runtime_errors`) even
  before this unit starts; `compile_errors::annotation_unknown_error` and
  `runtime_errors::contracts_invariant_fiber_yield` are directly attribute-expansion-shaped failures
  (`@bogus` never raises `attr.unknown` because expansion never runs). The other four groups need
  separate triage — not assumed related, not yet confirmed unrelated.
- **Write-set.** `phalcom-core/core/core.ph` (`class Attribute`, `class On`, error/skeleton classes,
  `attributes`/`attributesOfType` `.ph`-derivable accessors); `phalcom-core/src/heap/class.rs` +
  `.../method/object.rs` + `.../heap/module` struct (the `_attributes` Vec + `attach`); a **new**
  `phalcom-core/src/primitive/attribute.rs` (native `attach`/`attributes`/frozen-store enforcement, tier
  singletons); `phalcom-core/src/universe/primitives.rs` + `.../vm/bootstrap.rs` (register `Attribute`/`On`/
  tier singletons; floor census bump); `phalcom-core/src/compiler/attributes.rs` (recognize `@On`, validate
  declared tier vs implemented hook selector, emit the three `attr.*` diagnostics, drive the
  `Name.new(args)`+`attach` desugar for passive attributes); **added to write-set 2026-07-13:**
  `phalcom-core/src/compiler/lib/class_decl.rs` (and `compiler/lib/mod.rs` if the attach-desugar statements
  thread through `Statement::Class` lowering rather than living inside `compile_class` itself) — invoke
  `expand_class_attributes` from the class-compilation path, and add the codegen hook to emit the
  post-class-definition `Name.new(args); artifact.attach(_a)` sequence (no such "run this after the class
  opcode" mechanism exists yet; this is new codegen, not a one-line call-site fix).
- **Design decision.** Tier singletons follow the `True`/`False` precedent (attribute-classes.md L91) — real
  singleton objects, not symbols. Retention store is a plain `Vec<Value>`, **unallocated (null) for
  un-annotated artifacts** (attribute-classes.md §Hazards "Retention cost") — do not use a `Map`. Store is
  **frozen after class-definition** (A-5) — mutation is an error; this is what keeps ADR-0053's
  `has_runtime_interceptor` a one-time never-invalidated flag (do not build an epoch counter). Reserved hook
  selectors (`expand`/`finalizeLayout`/`wrap`/`resolveMissing`/`aroundSend`) are recognized only on
  `Attribute` subclasses so an unrelated same-named method is never drafted into a tier (attribute-classes.md
  L99). `attr.compile_tier_reserved` fires at the *attribute class's own definition site* (A-3) — a cheap
  static check, no body inspection.
- **Risk.** Bootstrap ordering — `Attribute` and `@On` must exist before any attribute (including `@On`
  itself) is used; the recursion bottoms out in Rust at the root (attribute-classes.md §Hazards "Bootstrap").
  Getting the `_attributes` slot onto three different heap structs risks a borrow-model regression in the
  arena accessors (standing risk). The desugar must run at the enclosing class's definition time, once
  (attribute-classes.md §"What the compiler lowers").
- **Test strategy.** `verify_invariants()`: `Attribute`/`On`/tier singletons present post-bootstrap; every
  `Attribute` subclass with a declared tier has the matching hook, and vice-versa. Golden corpus: the
  `@Author(name:)` passive-metadata case (`Engine.attributesOfType(Author).first.name` ⇒ `"Ada"`);
  `attr.missing_hook`/`attr.undeclared_hook`/`attr.compile_tier_reserved` negative goldens (span-carrying,
  recover-not-panic); frozen-store mutation-is-error golden. Snapshot: the `Name.new(args)`+`attach` desugar.
- **Forward-looking.** Must not preclude A-6 (v0.3 per-instance/`aReceiver`-scoped Install + `_attributes`
  hanging off arbitrary objects). Keep `attach`/retention keyed on the artifact via a generic slot, not a
  class-only special case, so v0.3 can generalize the store to any object "for free" (attribute-classes.md
  §"Deferred to v0.3"). Do **not** admit post-definition mutation (would force the epoch counter A-5/ADR-0053
  deferred).

### M-METAOBJECT — `Method.fromBlock` + `Behavior.defineMethod` (object-model §8 install surface)

- **Goal.** The two metaobject primitives every Install/Dispatch hook needs to *re-install* a wrapped method:
  `Method.fromBlock { … }` (reify a block as a callable `Method`) and `Behavior.defineMethod(selector, method)`
  (install a `Method` into a class/metaclass dictionary). Satisfies attribute-classes.md §"The install surface
  is `Behavior`-side" + §"What it needs" (Reflection surface gate D-2), object-model §8.
- **Depends on.** Existing `Method`/`BoundMethod`/`invoke_method_object` infra (LANDED). Independent of
  M-ATTR-ROOT logically, but **shares `core.ph` + `universe/primitives.rs`** → sequence after M-ATTR-ROOT.
- **Write-set.** `phalcom-core/src/primitive/method.rs` (`method_from_block` — replace the current
  `method_class_new` reject with a block→`MethodObject` reifier); a **new** `phalcom-core/src/primitive/behavior.rs`
  (`behavior_define_method` — writes into `ClassObject.methods`, `Behavior`-only, never an instance);
  `phalcom-core/src/heap/class.rs` (helper to insert/replace a method by selector, honoring
  `constructor_aliases`/`has_new_construct` invariants); `phalcom-core/core/core.ph` (`Behavior#defineMethod`,
  `Method.class#fromBlock` surface); `phalcom-core/src/universe/primitives.rs` (registration; floor bump).
- **Design decision.** `defineMethod` lives on `Behavior` (superclass of `Class`+`Metaclass`) so
  `Point.defineMethod` is instance-side and `Point.class.defineMethod` is class-side — one API, "which
  `Behavior` receives it" chooses the side (attribute-classes.md L315–319). An ordinary instance does **not**
  respond (no per-instance dictionary in v0.2). `fromBlock`'s reified `Method` reuses `invoke_method_object`
  as its body-runner so `m.invokeOn(self, args)` in a `wrap` closure funnels through the exact existing path.
- **Risk.** Re-installing a method must not corrupt selector-signature invariants (arity/kind encoding,
  ADR-0012) or the inline-cache/sacred-selector assumptions — installing over a sacred selector must trip
  the same pristine-flag flip M-RUNTIME uses (cross-reference; do not silently bypass). Borrow-model: mutating
  a `ClassObject.methods` `IndexMap` while the arena holds other live borrows.
- **Test strategy.** Golden: `Method.fromBlock { … }` builds a callable whose `invokeOn` runs the block;
  `SomeClass.defineMethod(#foo, m)` then makes `instance.foo` dispatch to it (warm again after redefinition).
  `verify_invariants()`: a `defineMethod`-installed method has a well-formed signature keyed by its selector.
  Negative: `anInstance.defineMethod(...)` does not respond (dNU).
- **Forward-looking.** Keep `defineMethod` strictly `Behavior`-side (attribute-classes.md §"Deferred to v0.3":
  per-instance `anObject.defineMethod` is a v0.3 floor change touching allocation + IC shape — do not open it
  here). `fromBlock`'s `Method` shape must stay compatible with a future inline-cache warm site.

### M-INSTALL — Install-tier `wrap(_)` composition dispatch

- **Goal.** At class-definition time, for each **Install**-tier attribute on a method: instantiate it
  (`Name.new(args)`, arbitrary constructor code allowed, A-3), call `wrap(theMethod) ⇒ aMethod`, and
  re-install the result via `Behavior.defineMethod`; multiple attributes on one member compose
  **source-order, innermost-last** (A-4, attribute-classes.md §"Instantiation order"). Retention (`attach`)
  and the behavioral wrap are independent (attribute-classes.md §"What the compiler lowers"). Satisfies
  decorators.md Install tier, attribute-classes.md Install rows.
- **Depends on.** M-ATTR-ROOT (tier/retention), M-METAOBJECT (`fromBlock`/`defineMethod`).
- **Write-set.** `phalcom-core/src/compiler/attributes.rs` (the Install-tier driver: after a member's method
  object exists, emit the instantiate→wrap→defineMethod sequence in composition order); `phalcom-core/src/compiler/lib/class_decl.rs`
  (call the Install driver at the class-def site, **after** members are compiled into real `Method` objects —
  ordering is load-bearing, mirrors the U-ANNOT-CONTRACTS weave call-site discipline).
- **Design decision.** Install fires **once**, at class-def, as ordinary program execution (world is up) — no
  staged compile-time evaluation (attribute-classes.md §"compile-time boundary"). The wrapped `Method` stays
  a plain dictionary entry ⇒ **inline-cacheable** (decorators.md Install row: "wrapped `Method` stays
  inline-cacheable"). Composition is a fold: `m₁ = attrₙ.wrap(m₀); … ; m_out = attr₁.wrap(m_{n-1})` for
  source order `@attr₁ @attr₂ … @attrₙ` (innermost-last).
- **Risk.** Running user constructor + `wrap` code during class definition can throw — a failed decorator must
  produce a clean compile/def-time diagnostic, not a half-installed class. Erasure golden (decorators.md
  "two rules"): stripping a `runtime:false`-behaving Install attr must leave bytecode identical.
- **Test strategy.** Golden: a trivial `@Identity` Install attr returning the method unchanged is behaviorally
  and (for the erasure axis) bytecode-inert. Composition golden: `@A @B foo` installs `A.wrap(B.wrap(foo))`,
  verified by observable order. This unit ships **no named decorator** — its consumers are D-MEMOIZE/D-RETRY/
  D-SYNC(class-wide). Ship one synthetic Install attr as its own green gate.
- **Forward-looking.** Multi-tier-per-class is v0.3 (A-1) — the driver handles exactly one tier per attribute
  class; do not build a "which hook wins" resolver. Keep the fold order the single source of truth so the
  Runtime-chain (M-RUNTIME) can reuse the same composition rule.

### M-LAYOUT-SLOTS — Layout reserved-slot mechanism + `finalizeLayout(_)` builtin hook

- **Goal.** The per-receiver-state substrate: a builtin-only Layout hook `finalizeLayout(_)` that reserves
  extra slot(s) on a class's instance layout, plus the receiver slot primitives `reserveSlot(#name)`/
  `slotAt(#name)`/`setSlotAt(#name, v)`. This is the "per-receiver ⇒ Layout ⇒ builtin" row of
  attribute-classes.md's scope table (ADR-0052 confirms per-receiver state is Layout-confined). Satisfies
  decorators.md Layout tier; the substrate under `@lazy`/`@synchronized`(per-receiver)/`@observable`.
- **Depends on.** M-ATTR-ROOT (tier singletons + `Compile/Layout` reserved-for-builtin gate). **Needs
  `ClassLayout`/`field_layouts` from U-ANNOT-LAYOUT's field-layout work** — verify U-ANNOT-LAYOUT landed
  (see §3 external deps) or that `ClassLayout` slot-growth is available; if only inference-based layout exists,
  this unit extends `ClassLayout` to admit reserved (non-source-field) slots.
- **Write-set.** `phalcom-core/src/vm/mod.rs` (`ClassLayout` reserved-slot count; slot allocation for reserved
  names); `phalcom-core/src/heap/instance.rs`/`object.rs` (per-receiver slot vector read/write by reserved
  symbol); a **new** `phalcom-core/src/primitive/layout.rs` (native `reserveSlot`/`slotAt`/`setSlotAt`);
  `phalcom-core/src/compiler/attributes.rs` (the `finalizeLayout` builtin-hook call at the finalize phase);
  `phalcom-core/src/universe/primitives.rs` (registration; floor bump).
- **Design decision.** Reserved slots extend the ADR-0011 slot vector — **no new `Value` arm, no side table**
  (ADR-0052: side tables are exactly what Layout-confinement forbids). `slotAt`/`setSlotAt` key on an interned
  reserved `Symbol` resolved to a slot index at finalize, so the runtime access is an index, not a map lookup.
  `finalizeLayout` is a **native** hook (builtin Layout is compiler-reserved, A-3) — user `tier: Layout`
  hits `attr.compile_tier_reserved`.
- **Risk.** Growing the instance layout interacts with inheritance (subclass slot offsets must stay stable —
  reuse U-ANNOT-LAYOUT's superclass-count-read pattern) and with `@data`/`@construct` field-order-is-API
  (reserved slots must not shift declared-field slot indices — allocate reserved slots *after* source fields).
  Borrow-model fragility on the instance slot vector.
- **Test strategy.** The ADR-0052 snapshot assertion, generalized: a Layout builtin stores state in a reserved
  slot, **never** a receiver-keyed side table (pin via a no-side-table probe). Golden: `reserveSlot`+
  `slotAt`/`setSlotAt` round-trip per receiver; two instances have independent reserved slots; a subclass of a
  reserved-slot class keeps its own source-field slots at the right offsets (regression).
- **Forward-looking.** Weak-key/GC-reclaim of per-receiver caches is ADR-0052's revisit trigger (needs the
  non-moving mark-sweep collector, ADR-0050, to gain weak refs) — do **not** design the slot as weak now;
  reserved slots are strong, and that is documented behavior. Keep the reserved-slot allocation additive so
  v0.3 per-instance behavior (A-6) can layer on without a layout redesign.

### M-RUNTIME — `aroundSend(_)` interceptor + `has_runtime_interceptor` bit + `Invocation`

- **Goal.** The per-send interception path: a per-class `has_runtime_interceptor: bool` set once at class-def
  when a **Runtime**-tier attribute installs; the dispatch path consults the bit and, when set, routes the
  send through the attribute's `aroundSend(anInvocation)` chain (chained innermost-last, D-3); an `Invocation`
  object exposing `selector`/`name`/`args`/`proceed`; and the sacred-selector interlock — installing a Runtime
  decorator on a sacred family (`Bool`/`Block`) flips that family's existing `*_sacred_pristine` flag,
  deopting the inliner (ADR-0053, verbatim). Satisfies decorators.md Runtime tier, ADR-0053, attribute-classes.md
  `aroundSend(_)` row.
- **Depends on.** M-ATTR-ROOT, M-METAOBJECT.
- **Write-set.** `phalcom-core/src/heap/class.rs` (the `has_runtime_interceptor: bool` field);
  `phalcom-core/src/vm/send.rs` **(SPINE — reviewer ON)** (bit-check in the send funnel; the `aroundSend`
  chain invocation; `proceed` = the cached direct call); `phalcom-core/src/universe/primitives.rs` +
  `.../universe.rs` (sacred pristine-flag flip on Runtime install; `Invocation` primitives); a **new**
  `phalcom-core/src/primitive/invocation.rs`; `phalcom-core/core/core.ph` (`class Invocation`);
  `phalcom-core/src/compiler/attributes.rs`/`class_decl.rs` (set the bit + register the interceptor chain at
  class-def when a Runtime attr installs).
- **Design decision (governed by ADR-0053).** The bit is a **one-time, never-invalidated** flag — valid
  precisely because retention is frozen post-def (A-5) and tiers are class-def-time-only; **do not build an
  epoch counter** (ADR-0053 explicitly prices but does not build it). For sacred sends, reuse the existing
  `bool_sacred_pristine`/`block_sacred_pristine` mechanism verbatim — no new opcode/flag. For ordinary sends,
  the bit is checked in `vm/send.rs`'s dispatch; **when the inline cache (U-IC) lands, the guard reads the bit
  alongside the `ClassId` compare** (ADR-0053) — the check works today in plain dispatch without IC, and
  extends into the IC guard later without redesign. An undecorated class pays at most one bit-check.
- **Risk.** `vm/send.rs` is the hottest path in the language — the bit-check must be branch-predictable and
  off the fast path for undecorated classes (the whole point of ADR-0053). Interceptor recursion: a sink/hook
  that re-sends to the same decorated receiver recurses (decorators-dispatch-observability.md §Hazards) —
  document, do not machinery-guard. `proceed` must preserve non-local return / throw / fiber-yield semantics
  through the wrapper.
- **Test strategy.** Golden: an undecorated class is behaviorally + (fast-path) allocation-identical with the
  bit present-but-false. A `@traced`-style synthetic Runtime attr observes entry/exit/`proceed` order; two
  Runtime attrs chain innermost-last. Sacred interlock golden: a Runtime decorator on `Bool` flips
  `bool_sacred_pristine` and deopts (reuse ADR-0018 regression harness). `verify_invariants()`: the bit is
  set iff a Runtime attr is installed.
- **Forward-looking.** Keep the `Invocation` object general (`selector`/`name`/`args`/`proceed`) — do not
  generator-specialize; it is the seam `@featureFlag`'s bypass-probe optimization (decorators.md "Future
  optimizations") and any future Dispatch/aroundSend unification will reuse. The epoch-counter upgrade path
  (A-5/open-Q4, v0.3) must remain a drop-in replacement for the bool.

### M-DISPATCH — Dispatch-tier `resolveMissing(_)` hook (mechanism completeness; **no batch consumer**)

- **Goal.** On a lookup **miss**, consult Dispatch-tier attributes' `resolveMissing(aSelector) ⇒ aMethodOrNone`
  and install/invoke the result (reuses the `doesNotUnderstand` slow path). Satisfies decorators.md Dispatch
  tier, attribute-classes.md `resolveMissing(_)` row.
- **Depends on.** M-ATTR-ROOT, M-METAOBJECT, existing DNU path (`core.ph` L247).
- **Write-set.** `phalcom-core/src/vm/send.rs` (miss path), `phalcom-core/src/heap/class.rs`.
- **Design decision.** Slow-path only (decorators.md: "slow-path only") — never touches warm sends. D-4
  `attr.dispatch_collision` guards a Dispatch attr colliding with a hand-written `doesNotUnderstand`.
- **Risk.** Shares `vm/send.rs` with M-RUNTIME — **sequence after M-RUNTIME** (same spine file), do not
  parallelize.
- **Test strategy.** A synthetic `@forwardMissing`-style Dispatch attr resolves a missed selector.
- **Forward-looking / status.** **None of the 8 named decorators are Dispatch-tier** (`@delegate` is
  resolved to **Compile**, D-1). This unit is mechanism completeness only → **DEFERRED register** (not on the
  critical path; build after the consuming decorators land, or when a Dispatch consumer is specced). Recorded
  here so the tier is not silently dropped.

---

## 2. Decorator units (built on the mechanism)

### D-DELEGATE — `@delegate` (Compile / builtin)

- **Goal.** `@delegate(to:, selectors:)` on a field generates one static forwarding method per selector
  literal; `attr.delegate_shadow` (selector already defined on the class) and `attr.delegate_conflict` (two
  delegates name one selector) compile diagnostics. decorators-dispatch-observability.md §"`@delegate`" (D-1
  resolved: Compile-only, explicit-selector).
- **Depends on.** U-ANNOT-CONTRACTS registry (LANDED) **+ `FieldDef`/`Target::Field` (U-ANNOT-LAYOUT — NOT
  landed, external dep §3)**. Does **not** depend on M-ATTR-ROOT/M-INSTALL/M-RUNTIME — it is a pure
  compiler derive, same class as `@get`/`@set`/`@data`.
- **Write-set.** `phalcom-core/src/compiler/attributes.rs` (the `"delegate"` generate-phase expander, reusing
  the shared `attr.accessor_collision`/shadow helper); `phalcom-core/tests/lang/decorators/` goldens.
- **Design decision.** `selectors:` entries are **selector literals** (`#rpm`, `#start(_)`) so arity+kind are
  pinned (`#start` getter ≠ `#start(_)` unary, ADR-0012). Generated forwarders are ordinary `Method`s →
  monomorphic, inline-cacheable, reflectable in `Car.methods`. No DNU touch → no `dispatch_collision` hazard.
- **Risk.** Selector-list drift from the delegate's real protocol is accepted+visible (spec §Hazards) — do not
  add runtime tracking. The shadow check must read the *rest of the class member list* post-field-parse.
- **Test strategy.** Forwards each selector to the field; `#start` vs `#start(_)` independent; negative-lane
  `attr.delegate_shadow`/`attr.delegate_conflict` with spans; reflection golden (`Car.methods` includes
  forwarders); erasure golden (strip removes exactly the forwarders, `runtime:false`).
- **Forward-looking.** Do not fold open whole-protocol forwarding in (that is the `Proxy`/DNU library, D-1;
  a `@forwardMissing` Dispatch decorator is v0.3 DEFERRED). Keep forwarders plain methods so a future
  `@forwardMissing` can coexist.

### D-MEMOIZE — `@memoize` (Install / user `.ph`)

- **Goal.** `@memoize(max: None)` caches a method result keyed by `(receiver-identity, args)`; unbounded by
  default, opt-in LRU via `max:`. decorators-behavioral.md §"`@memoize`" (B-1 resolved: class-wide Install
  cache, `max:` LRU only).
- **Depends on.** M-INSTALL. Sub-deps: `Map` (LANDED), **`Pair`** (new — `Pair.of(self,args)` key),
  **`Map#evictOldest`** (new — for `max:` LRU).
- **Write-set.** `phalcom-core/core/core.ph` (`class Memoize is Attribute`, `class Pair`,
  `Map#evictOldest`); `phalcom-core/tests/lang/decorators/` goldens. **core.ph chokepoint — serialize.**
- **Design decision.** Key is `(ObjRef identity, args)`, never `args` alone (spec: sharing across instances is
  silently wrong). Cache lives in the attribute instance (per-class, shared) — the per-method state row, so
  **user-authorable Install**, no reserved slot. Default unbounded is the documented retention leak (ADR-0052),
  not a bug.
- **Risk.** Memoizing a suspending or mutating method is caller-contract misuse (no purity analysis, ADR-0021
  floor-not-proof) — state the contract, do not enforce.
- **Test strategy.** Same-args repeat returns cached (side-effect counter proves single run); two receivers
  cache independently; `max:2` LRU-evicts on 3rd key; thrown compute not cached; negative-lane stale-after-
  mutation golden (documents contract).
- **Forward-looking.** No per-receiver `@memoize` (that is a future Layout builtin gated on weak-key GC, B-1/
  ADR-0052) — keep the cache class-wide.

### D-RETRY — `@retry` + `Backoff` (Install / user `.ph`)

- **Goal.** `@retry(times:, on: Error, backoff: Backoff.none)` re-invokes on matching failure with configurable
  backoff; new `Backoff` core class (`.none`/`.fixed(ms)`/`.exponential(base:,max:)`, `waitBefore(attempt)`).
  decorators-behavioral.md §"`@retry`" (B-2 resolved: `Backoff` is a ratified core class).
- **Depends on.** M-INSTALL; `Block#on(_)` (U-ERR, LANDED); `Fiber.yield` for the cooperative wait (LANDED).
- **Write-set.** `phalcom-core/core/core.ph` (`class Retry is Attribute`, `class Backoff` + strategy
  subtypes); `phalcom-core/tests/lang/decorators/` goldens. **core.ph chokepoint — serialize.**
- **Design decision.** `waitBefore` is a **fiber-yielding** wait (suspends to scheduler, does not busy-block),
  consistent with concurrency.md. `on:` filter defaults to all `Error` (broad — narrow in real code). Retry
  counter is frame-local; config is per-method (attribute instance) ⇒ stateless Install row.
- **Risk.** Backoff wait from inside a native combinator (`each`) raises `CannotYieldAcrossNativeFrame` (the
  restricted-yield rule) — noted interaction, not a bug. Retry re-runs side effects once per attempt (caller
  contract).
- **Test strategy.** Succeeds on attempt k<times (runs k times); exhausts→rethrows last; `on: TimeoutError`
  propagates a non-Timeout on attempt 1; `exponential` waits grow (fake-clock seam — the reason `Backoff` is a
  class, B-2); side-effect-per-attempt golden.
- **Forward-looking.** `Backoff` is the fake-clock test seam a raw block can't offer — keep `waitBefore` a
  method so the clock is injectable. Do not merge with the `Retry` *proxy* (ADR-0057 — different granularity).

### D-LAZY — `@lazy` (Layout / builtin)

- **Goal.** `@lazy` on a getter computes once, caches per-receiver in a reserved slot; initializer-throws ⇒
  retry-next-access (slot stays empty). decorators-behavioral.md §"`@lazy`" (fork resolved: retry-next-access).
- **Depends on.** M-LAYOUT-SLOTS (`finalizeLayout`/reserved slot).
- **Write-set.** `phalcom-core/src/compiler/attributes.rs` (the `"lazy"` builtin Layout expander:
  `finalizeLayout` reserves `#__lazy`, `wrap` routes the getter through `slotAt`/`setSlotAt`);
  `phalcom-core/core/core.ph` (the `class Lazy` surface skeleton, builtin-owned);
  `phalcom-core/tests/lang/decorators/` goldens.
- **Design decision.** Builtin Layout — a user `@lazy` hits `attr.compile_tier_reserved` (A-3). Per-receiver
  cache in the reserved slot, never a side table (ADR-0052). On initializer throw the slot stays `None` and
  the exception propagates (retry-next-access chosen because caching-the-throw is user-recoverable-from,
  the reverse is not).
- **Risk.** A suspending initializer can double-force under a second fiber (last `setSlotAt` wins) — not
  guarded; compose `@synchronizedClassWide` or keep synchronous.
- **Test strategy.** Initializer runs on first access only across many reads; two instances force
  independently; throw→next-access re-runs then succeeds; reserved-slot / no-side-table snapshot (ADR-0052).
- **Forward-looking.** Getter-target only; a `@lazy` that must force under mutual exclusion uses
  `@synchronizedClassWide` (Method-target `@synchronized` does not apply to a getter). Keep the reserved slot
  strong (weak-key reclaim is the v0.3/ADR-0052 revisit).

### D-SYNC — `@synchronized` (Layout/builtin, per-receiver) + `Monitor` + `@synchronizedClassWide` (Install)

- **Goal.** Per-receiver cooperative reentrant monitor guarding suspension windows; new `Monitor` cooperative
  primitive; `@synchronizedClassWide` (Install, user) as the class-wide shared-monitor variant.
  decorators-behavioral.md §"`@synchronized`".
- **Depends on.** M-LAYOUT-SLOTS (per-receiver monitor slot) **+ M-INSTALL** (for the class-wide variant);
  `Fiber` yield/queue (LANDED); `Block#ensure(_)` (LANDED, ADR-0008) for release-on-unwind.
- **Write-set.** `phalcom-core/core/core.ph` (`class Monitor`, `class Synchronized`, `class
  SynchronizedClassWide`); `phalcom-core/src/compiler/attributes.rs` (the `"synchronized"` Layout expander);
  `phalcom-core/tests/lang/decorators/` goldens. **core.ph chokepoint — serialize.**
- **Design decision.** `Monitor` = fiber-queue + owner cell + depth counter (a **cooperative** primitive, not
  `Mutex`/`Lock` — there is no OS thread to exclude, concurrency.md §1). Reentrant on the **(receiver, owning
  fiber)** pair (depth++), never deadlocks (cooperative scheduler never preempts the owner). Release wired
  through `ensure` so a throw still releases. Per-receiver monitor state in a reserved slot ⇒ Layout/builtin
  (ADR-0052); class-wide shared monitor in the attribute instance ⇒ Install/user.
- **Risk.** On a suspension-free method `@synchronized` is a silent no-op (pure overhead) — correct but
  over-appliable (lint opportunity, deferred). Reentrancy must key on (receiver,fiber), not fiber alone.
- **Test strategy.** Suspension-free method identical with/without (no-op guard); two fibers on the **same**
  receiver serialize across a suspend (interleaved-print order); two fibers on **different** receivers run
  concurrently (independent monitors); reentrancy on `self` same-fiber no deadlock; release-on-throw lets a
  later fiber enter.
- **Forward-looking.** OS-thread `@synchronized` is foreclosed (single-threaded by ratified design); if OS
  threads ever land it is a superseding ADR — keep `Monitor` cooperative-only. Two decorators (per-receiver vs
  class-wide) because a class declares at most one tier (A-1) — do not try to unify.

### D-TRACED — `@traced` + `Tracer` protocol + `Tracer.stdout` (Runtime / user)

- **Goal.** `@traced(entry:, exit:, timing:, errors:, sink:)` logs sends via an `aroundSend` interceptor;
  new `Tracer` core protocol (`enter`/`exit`/`threw`) with a `Tracer.stdout` default over `System.print`.
  decorators-dispatch-observability.md §"`@traced`" (D-2 resolved: `Tracer` ratified core protocol).
- **Depends on.** M-RUNTIME (`aroundSend`/`Invocation`/interceptor bit); `System.print` (LANDED);
  **`Clock.now`** (new — for `timing:`); `Block#on(_)`/`andThen` (U-ERR).
- **Write-set.** `phalcom-core/core/core.ph` (`class Traced is Attribute`, `class Tracer`,
  `Tracer.stdout`, `class Clock` or `Clock.now` surface); `phalcom-core/src/primitive/*` (a `Clock.now`
  native if no monotonic clock exists — verify); `phalcom-core/tests/lang/decorators/` goldens.
  **core.ph chokepoint — serialize.**
- **Design decision.** Runtime (not Install) so whole-object tracing observes inherited + dynamically
  dispatched sends. **Never swallows** — observes and re-raises (stripping `@traced` must leave results
  identical, `runtime:true` but result-preserving). `sink:` accepts any `Tracer`-protocol object (the
  test-double / structured-logger seam).
- **Risk.** Per-send cost is a real branch on the decorated receiver (ADR-0053 bit). A custom sink that
  re-sends into the traced object graph recurses (caveat, not machinery).
- **Test strategy.** Default set logged (timing absent); `timing:true` adds elapsed; thrown method logs
  `threw` and re-raises (assert propagation, never swallowed); custom `sink:` receives structured calls
  (recording double); erasure golden (result identical with/without).
- **Forward-looking.** Keep `Tracer` a plain protocol (`enter`/`exit`/`threw`) — the raw-three-blocks
  alternative is v0.3 DEFERRED. Do not merge with the `Trace` proxy (ADR-0057, two granularities).

### D-FEATUREFLAG — `@featureFlag` + `Flags` + `OffBehavior` + `FeatureDisabled` (Runtime / user)

- **Goal.** `@featureFlag(name:, whenOff: OffBehavior.raise)` gates a send on a runtime-queried flag; new
  ambient `Flags` core module (`Flags.enabled(name)`), `OffBehavior` (`.raise`/`.fallback(#sel)`/`.skip(value)`),
  `FeatureDisabled` error. decorators-dispatch-observability.md §"`@featureFlag`" (D-3 resolved: `Flags`
  ratified ambient core module).
- **Depends on.** M-RUNTIME.
- **Write-set.** `phalcom-core/core/core.ph` (`class FeatureFlag is Attribute`, `module Flags`,
  `class OffBehavior` + variants, `class FeatureDisabled is Error`);
  `phalcom-core/tests/lang/decorators/` goldens. **core.ph chokepoint — serialize.**
- **Design decision.** Runtime because the flag flips at runtime and must be **queried per call** (an Install
  wrapper would bake state at boot — wrong). `Flags` is a module singleton (same shape as `System`), queried
  not injected. Default `OffBehavior.raise` (not silent `None` — the silent-wrong this family rejects);
  `fallback(#sel)` signature is checked at class-def against the receiver's protocol.
- **Risk.** `fallback(#sel)` mismatch caught at class-def (fail-early). The off-case can expose the
  bypass-probe optimization (decorators.md "Future optimizations") — keep the `Invocation` seam general.
- **Test strategy.** Flag on→runs; off default→`FeatureDisabled`; `fallback(#sel)` off→fallback with same
  args; `skip(v)` off→returns v; flag flipped between two calls→2nd observes new state (proves per-send, not
  baked); negative-lane `fallback` signature mismatch→compile error.
- **Forward-looking.** Injected/per-scope `FeatureFlags` service is v0.3 DEFERRED (natural upgrade once `@inject`
  is specced) — keep `Flags` a name-resolved ambient so a scoped service can shadow it later.

### R-REACTIVITY — `Signal`/`Computed`/`Effect`/`Reactive` push-pull substrate

- **Goal.** The reactive runtime `@observable` layers over: `Signal` (value + observer set), `Computed`
  (derived, cached, per-receiver), `Effect` (re-runs on dependency change), `Reactive.current`/`trackedBy`
  dependency-tracking context. reactivity.md (full runtime).
- **Depends on.** M-LAYOUT-SLOTS (per-receiver `Computed` cache is a reserved slot). **New:**
  `U-REACTIVE-NATIVE` (not yet a separate unit in this plan — see below).
- **Write-set.** `phalcom-core/core/core.ph` (`Signal`/`Computed`/`Effect`/`Reactive`), likely a
  `phalcom-core/src/primitive/reactive.rs`, `universe/primitives.rs`. **core.ph chokepoint.**
- **Status.** **BLOCKED-ON-DECISION #1 — RESOLVED 2026-07-13.** reactivity.md is now **Accepted**;
  R-1 (boolean stale-flag, three-color deferred), R-3 (shallow default), R-4 (sync flush only), R-5
  (manual dispose only) are all ruled and need no new VM surface beyond M-LAYOUT-SLOTS. **R-2 (tracking-context
  home) resolved to a design call that itself needs new native support**: no `.ph`-reachable class-side/module
  mutable state exists today (`concurrency.md:234`), so `Reactive.current`/`trackedBy`/`untracked`/
  `schedule`/`batch`/`flush` need a native module — [ADR-0058](../adr/0058-reactive-tracking-context-needs-a-native-module.md),
  same shape as `System.schedule`'s landed precedent for `Future`. **This unit is now BLOCKED only on a new
  prerequisite unit, tentatively `U-REACTIVE-NATIVE`** (not yet scoped/write-set-annotated in this plan — a
  follow-on planning pass, not a design question), not on any further user decision.
- **Forward-looking.** Ownership/disposal (R-5, resolved to manual dispose) is the leak seam if a `Reactive.root`
  owner tree is ever added later — do not build the substrate in a way that forecloses adding one, per R-5's own
  note in reactivity.md.

### D-OBSERVABLE — `@observable` (Layout + generate)  **(BLOCKED, transitively)**

- **Goal.** `@observable var _x` reboxes the field slot as a `Signal` (finalize/Layout) and generates a tracked
  getter + notifying setter (generate/Compile). decorators-observable.md (one unified `@observable`).
- **Depends on.** M-LAYOUT-SLOTS + **R-REACTIVITY** (BLOCKED) + `FieldDef`/`Target::Field` (U-ANNOT-LAYOUT,
  external dep §3).
- **Write-set.** `phalcom-core/src/compiler/attributes.rs` (the `"observable"` Layout+generate expander);
  `phalcom-core/core/core.ph` (`class Observable` skeleton); goldens.
- **Design decision.** Layout (builtin) + generate accessors — **not** Install (an Install `@observable` would
  put per-receiver `Signal` state in a class-level attribute instance ⇒ the exact ADR-0052 leak). `@data`'s
  `==` over an `@observable` field must compare **unboxed values** via the generated getter, never `Signal`
  identities (decorators-observable.md §Composition — a required interaction rule). Shallow by default.
- **Status.** **BLOCKED** on R-REACTIVITY (⇒ BLOCKED-ON-DECISION #1) and on U-ANNOT-LAYOUT `FieldDef`.
- **Test strategy (when unblocked).** Read-in-`Effect` registers dependency + write reruns once; no-op write
  does not rerun (equality bail); `@observable @data` compares unboxed; `@observable @construct` seeds the
  `Signal`; reserved-slot/no-side-table snapshot; erasure of the generate half.
- **Forward-looking.** Exactly one `@observable` (no second/third — persistence dirty-tracking and the stdlib
  reboxed-slot are the *same* decorator from a consumer / older surface). Inherits reactivity.md's open set,
  none of which block the *decorator surface* once the substrate exists.

---

## 3. External dependencies (already-planned units, not user-blockers)

- **U-ANNOT-LAYOUT** (`FieldDef` / `ClassMember::Field` / `Target::Field`) — **NOT landed** (verified: absent
  from `ast.rs`/`attributes.rs`). Gates **D-DELEGATE** and **D-OBSERVABLE** (both `Field`-targeted). Its own
  plan exists (`units/U-ANNOT-LAYOUT/annot-layout.md`); it depends only on U-ANNOT-CONTRACTS (landed).
  Land it before the two Field-targeted decorators; it does **not** block the mechanism units or the six
  Method/Class-targeted decorators.
- **Sub-class deps (small, in-unit, not blockers):** `Pair` + `Map#evictOldest` (D-MEMOIZE), `Clock.now`
  (D-TRACED) — new but trivial; build inside their consuming unit.

---

## 4. Wave schedule (dependency-satisfied ∧ write-sets pairwise disjoint)

Foundational units are **serialized alone on the critical path** (M-ATTR-ROOT then M-METAOBJECT share
`core.ph`+`primitives.rs`; both are spine). `core.ph` is a **single-writer chokepoint** — at most one
`core.ph`-touching unit per wave. `vm/send.rs` is shared by M-RUNTIME/M-DISPATCH (serialize those).

- **Wave 0 (critical path, alone):** **M-ATTR-ROOT**. (heap structs + core.ph + attributes.rs + primitives)
- **Wave 1 (critical path, alone):** **M-METAOBJECT**. (method.rs + behavior.rs + core.ph + primitives)
- **Wave 2 (fan-out — disjoint write-sets):**
  - **M-INSTALL** (`compiler/attributes.rs` + `compiler/lib/class_decl.rs`)
  - **M-LAYOUT-SLOTS** (`vm/mod.rs` `ClassLayout` + `heap/instance.rs` + `primitive/layout.rs`)  ⟵ needs
    U-ANNOT-LAYOUT `ClassLayout` availability; if unlanded, this unit extends `ClassLayout` itself — still
    disjoint from M-INSTALL.
  - **M-RUNTIME** (`vm/send.rs` + `heap/class.rs` + `primitive/invocation.rs`)
  - **D-DELEGATE** (`compiler/attributes.rs` **conflicts with M-INSTALL** on that file) → **do NOT co-schedule
    with M-INSTALL**; run D-DELEGATE in Wave 3 instead. (Alternatively sequence M-INSTALL's attributes.rs
    slice first.)
  > Note: M-INSTALL, M-LAYOUT-SLOTS, M-RUNTIME all touch `compiler/attributes.rs` for their driver hook. If
  > the attributes.rs slices cannot be kept disjoint (separate functions), serialize them; the VM/heap slices
  > are disjoint and can proceed in parallel. Keep each tier-driver in its own function in attributes.rs to
  > preserve disjointness.
- **Wave 3 (decorators — mechanism satisfied; `core.ph` writers SERIALIZED one-per-wave):**
  - Parallel-safe (disjoint, non-core.ph or single core.ph writer):
    - **D-LAZY** (`compiler/attributes.rs` "lazy" slice + small core.ph skeleton) — needs M-LAYOUT-SLOTS
    - **D-DELEGATE** (`compiler/attributes.rs` "delegate" slice) — needs U-ANNOT-LAYOUT
  - core.ph-heavy decorators — **one per sub-wave** (serialize on core.ph):
    - **D-MEMOIZE** → then **D-RETRY** → then **D-SYNC** (all Install/Layout-ready after Wave 2)
    - **D-TRACED** → then **D-FEATUREFLAG** (both need M-RUNTIME)
  > Practical schedule: interleave one attributes.rs-only decorator (D-LAZY/D-DELEGATE) with one core.ph
  > decorator per sub-wave to keep two workers busy without a core.ph collision.
- **Deferred / blocked (not scheduled):**
  - **M-DISPATCH** — DEFERRED register (no batch consumer; shares `vm/send.rs` with M-RUNTIME → after it).
  - **R-REACTIVITY** — BLOCKED-ON-DECISION #1 (reactivity.md unratified).
  - **D-OBSERVABLE** — BLOCKED behind R-REACTIVITY + U-ANNOT-LAYOUT.

Critical path: **M-ATTR-ROOT → M-METAOBJECT → {M-INSTALL | M-LAYOUT-SLOTS | M-RUNTIME} → decorators**.

---

## 5. BLOCKED-ON-DECISION (needs the user before the affected units start)

1. **reactivity.md ratification (Signal/Computed/Effect).** reactivity.md is Proposed/exploratory with an
   unresolved open set (R-1…R-5) that shapes the substrate. **R-REACTIVITY and D-OBSERVABLE cannot be scoped
   until reactivity.md is ratified** (its own ADR) or R-1…R-5 are ruled. Recommendation: ratify reactivity.md
   as a prerequisite ADR before scheduling `@observable`; the other 7 decorators + full mechanism proceed
   without it.
2. **Decorator-spec status confirm (soft).** ADR-0054 ratifies the *mechanism/tiers*; the four decorator specs
   (behavioral / dispatch-observability / observable) are headed "Proposed — not ratified" though their own
   design questions (B-1/B-2/D-1/D-2/D-3 + the `@observable` unification) are marked *resolved* inline. Not a
   hard blocker (implementable as-resolved), but confirm before dispatch that ADR-0054's ratification is
   intended to cover these named-decorator specs, or bump their status headers. Recommendation: treat as
   ratified per the task framing; flag the status-header mismatch for a doc-sync pass.
3. **M-DISPATCH build-or-defer (not user-blocking).** No named decorator is Dispatch-tier (`@delegate` resolved
   to Compile, D-1). Recommendation: **defer** M-DISPATCH to the deferred register; build when a Dispatch
   consumer is specced. Recorded so the tier is not silently dropped.
