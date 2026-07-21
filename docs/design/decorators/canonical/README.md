# Decorators — canonical specification tree

- Status: **Living specification.** This tree is the single organizing spec for
  Phalcom's `@` decorator system: taxonomy, verification against the committed
  design, per-family specifications, and implementation plans. It sits *outside*
  the work-unit and decision-record systems on purpose — it proposes and
  organizes; it does not ratify. Anything marked **Proposed** here needs a PDR
  before an implementer may build on it.
- Date: 2026-07-20
- Authority relationships:
  - [`docs/spec/current/decorators/`](../v0.2/decorators/README.md) — **as-built**
    records. Those files are authoritative for what HEAD does; this tree cites
    them and never contradicts them silently.
  - [`docs/spec/current/experimental/annotations-*.md`](../v0.2/experimental/annotations-core.md)
    — the mechanism drafts (registry, grammar, phases). Ratified for the
    Compile/Layout tier by ADR-0054 §1.
  - [`docs/spec/current/drafts/decorators-*.md`](../v0.2/drafts/decorators-stdlib.md)
    — the library drafts. Design-ratified (ADR-0054 broad ruling, 2026-07-14);
    nothing in them is built.
  - ADRs 0052/0053/0054/0057/0063 and PDR-0001 — the decision floor this tree
    verifies against. `docs/adr/` is frozen; new rulings go to `docs/pdr/`
    as PDRs.
- Verification method: every claim below about HEAD was grounded 2026-07-20
  against `phalcom-core/src/compiler/attributes.rs` (registry, expanders,
  `expand_class_attributes`), `phalcom-ast/src/parser.rs`
  (`parse_attribute`/`attach_attrs`), `phalcom-core/src/primitive/attribute.rs`
  (retention floor), and `core.ph` (`class Attribute`, `class On`). File:line
  anchors in the family files cite greppable symbols; re-grep before trusting a
  bare line number.

## Why this tree exists

The decorator corpus grew in four strata, quickly: mechanism drafts
(`annotations-*`), as-built records (`v0.2/decorators/`), library drafts
(`drafts/decorators-*`), and rulings scattered across five ADRs. The strata
disagree in places — superseded surfaces (`@install`-marker era), tier
reclassifications (`@computed` Install→Layout), a decorator the forward design
deletes (`@construct`), and name collisions nobody was positioned to see because
no document held the whole inventory. This tree is that document: one taxonomy,
one status ledger, one collision registry, and per-family specs that verify each
decorator against the design philosophy before extending it.

## The philosophy test (applied throughout)

A decorator proposal must survive five checks drawn from Phalcom's committed
positions. Family files apply them explicitly; failures are recorded as
rejections, not silently dropped:

1. **Sugar over sends.** Almost every Phalcom surface feature lowers to
   message-sends on the existing object model. A decorator that needs a new VM
   mechanism must say so and route through the floor-admission rule
   (ADR-0019: *inexpressibility, never speed*).
2. **Selector identity is sacred** (ADR-0012, ADR-0043). Generated members are
   ordinary selectors; no decorator may introduce arity magic, default-arg
   folding, or install-time signature aliasing.
3. **One sigil, five tiers, total phase order** (ADR-0054). Every decorator
   names exactly one of Compile / Layout / Install / Dispatch / Runtime. A
   decorator whose timing cannot be placed on that axis is rejected — there is
   no sixth tier.
4. **Honest cost.** `runtime: false` decorators must strip to bytecode-identical
   members; Runtime-tier interception pays only ADR-0053's one-bit guard.
   Per-receiver state lives in Layout slots, never receiver-keyed side tables
   (ADR-0052).
5. **A decorator may not silently change a member's observable contract.**
   Wrapping that preserves the signature's meaning (memoize, trace, retry) is
   admissible; a decorator that changes what a call *returns* (e.g. an `@async`
   that turns `T` into `Future`) hides a type change behind a sigil and is
   rejected on least-surprise grounds. See [concurrency.md](concurrency.md).

## Naming convention (normative for new work, Proposed as a PDR candidate)

The tree on HEAD already follows a rule nobody wrote down. Written down:

| Spelling | Meaning | Resolution path |
|---|---|---|
| lowercase `@name` | **compiler-owned builtin** — a registry row in `AttributeRegistry` (Compile/Layout tiers, plus the subtractive pair) | registry lookup; unknown ⇒ `attr.unknown` |
| Capitalized `@Name` | **an `Attribute` subclass** — user or stdlib, Install/Dispatch/Runtime or passive metadata | `resolves_to_attribute_class` chain-walk |

`@On` is both (registered *and* a `core.ph` class) — it is the bridge row and
the deliberate exception. The library drafts predate this rule and spell
Install/Runtime decorators lowercase (`@memoize`, `@traced`); those spellings
flip to Capitalized (`@Memoize`, `@Traced`) when they land as `Attribute`
subclasses, **except** the Layout-tier builtins (`@lazy`, `@synchronized`,
`@observable`, `@computed`), which stay lowercase because they are
compiler-owned by ADR-0052's confinement rule.

Consequence that needs a ruling before the behavioral family lands
(**COLL-3** below): ADR-0057 deliberately kept `@retry`/`Retry`,
`@traced`/`Trace`, `@lazy`/`Lazy` as decorator/proxy pairs distinguished by
sigil-vs-class. Under this convention `@retry` becomes `@Retry` — and an
`Attribute` subclass named `Retry` now collides with the `Retry` *proxy* class
in the same global namespace. Options and a recommendation are in
[mechanism.md §Naming resolution](mechanism.md).

## Master status ledger

Legend: ✅ built · 🟦 ratified design, unbuilt · 📝 draft (design-ratified, far)
· 💡 proposed in this tree · ❌ rejected in this tree · 🧪 v0.3 experimental
design (PDR-0018 §3 mandate — full design exists, binds only the v0.3 track).

> **PDR-0018 (2026-07-20)** carries the four decorator ADRs forward under the
> PDR regime and mandates the runtime tiers, `@effect`, and the framework
> families as the v0.3 experimental track. Design set:
> [runtime-tier.md](runtime-tier.md) · [dispatch-tier.md](dispatch-tier.md) ·
> [effect.md](effect.md) · [frameworks-design.md](frameworks-design.md).
> The build-when-forced posture recorded in this tree's first cut is
> withdrawn where those files say so; the `@async`/perf-hint/doc-decorator
> rejections stand.

### Compile tier (builtin, `runtime: false`)

| Decorator | Target | Status | Spec | Notes |
|---|---|---|---|---|
| `@requires` | method/getter/setter | ✅ | [contracts.md](contracts.md) | woven Debug **and** Release |
| `@ensures` | method/getter/setter | ✅ | [contracts.md](contracts.md) | Debug only; **`result` binding defect** |
| `@invariant` | class | ✅ | [contracts.md](contracts.md) | Debug only; **ADR-0052 guard unimplemented** |
| `@construct` | class | ✅ | [derives.md](derives.md) | **scheduled for deletion** — unifies into `@constructor` (ADR-0063) |
| `@get` / `@set` | field | ✅ | [derives.md](derives.md) | `(priv)` arg parsed, unenforced; `@set`-on-`const` hole |
| `@data` | class | ✅ | [derives.md](derives.md) | `==`/`hash` pair rule; shallow `with` |
| `@sealed` / `@variant` | class / variant | ✅ | [derives.md](derives.md) | cross-unit enforcement unreachable today |
| `@native` | method/getter/setter | ✅ | [subtractive.md](subtractive.md) | provisional drop; N-2 (LSP anchor) open |
| `@ignore` | method/getter/setter | ✅ | [subtractive.md](subtractive.md) | the sanctioned drop |
| `@constructor` | class *or* method | 🟦 ADR-0063 | [placement.md](placement.md) | replaces `@construct`; U-CTOR |
| `@class` | method/getter/setter/field | 🟦 ADR-0063 | [placement.md](placement.md) | placement modifier; class-side install |
| `@delegate(to:, selectors:)` | field | 📝 | [interception.md](interception.md) | reclassified Dispatch→Compile (D-1) |
| `@override` | method/getter/setter | 💡 | [proposals.md](proposals.md) | class-def-time override check |

### Layout tier (builtin, per-receiver state)

| Decorator | Target | Status | Spec | Notes |
|---|---|---|---|---|
| `@lazy` | getter | 📝 | [behavioral.md](behavioral.md) | reserved slot; throw ⇒ retry |
| `@synchronized` | method | 📝 | [behavioral.md](behavioral.md) | **cooperative monitor**, not OS mutex |
| `@computed` | getter | 📝 | [reactive.md](reactive.md) | Install→Layout per ADR-0052; no dedicated spec yet |
| `@observable` | field | 📝 | [reactive.md](reactive.md) | gated on U-REACTIVE-NATIVE (ADR-0058) + R-1..R-5 |

### Install tier (`Attribute` subclasses)

| Decorator | Status | Spec | Notes |
|---|---|---|---|
| `@Memoize` | 📝 | [behavioral.md](behavioral.md) | class-wide cache, `max:` LRU (B-1) |
| `@Retry(times:, on:, backoff:)` | 📝 | [behavioral.md](behavioral.md) | `Backoff` class already in `core.ph` |
| `@SynchronizedClassWide` | 📝 | [behavioral.md](behavioral.md) | rare; design smell noted |
| `@Timed`, `@Authorize`, `@Transactional`, `@RateLimit`, `@Validate`, `@Idempotent` | 📝 sketch-only | [behavioral.md §Residue](behavioral.md) | no dedicated spec anywhere — dispositions recorded |

### Dispatch / Runtime tiers

| Decorator | Status | Spec | Notes |
|---|---|---|---|
| `@ForwardMissing(to:)` | 🧪 | [dispatch-tier.md](dispatch-tier.md) | first Dispatch decorator; transient resolution, miss-path only |
| `@Traced` | 🧪 | [interception.md](interception.md) + [runtime-tier.md](runtime-tier.md) | `Tracer` protocol already in `core.ph` |
| `@FeatureFlag(name:, whenOff:)` | 🧪 | [interception.md](interception.md) + [runtime-tier.md](runtime-tier.md) | off-default raises, never silent `None` |
| `@Metered` | 📝 sketch-only | [interception.md](interception.md) | disposition recorded |
| `@effect` | 🧪 | [effect.md](effect.md) | explicit activation, scope-owned disposal |

### Passive metadata

| Decorator | Status | Spec | Notes |
|---|---|---|---|
| `@On(targets, tier, inherited)` | ✅ (positional args only) | [mechanism.md](mechanism.md) | labeled `tier:` surface does not parse yet |
| `@Deprecated(reason:)` | 💡 | [metadata-and-docs.md](metadata-and-docs.md) | passive now, warn-tier later |
| `@Author`, arbitrary user metadata | ✅ (mechanism) | [metadata-and-docs.md](metadata-and-docs.md) | retention + reflection built |
| `@suspends` | 💡 optional | [concurrency.md](concurrency.md) | advisory "may yield" marker |

### Rejected families

| Family | Verdict | Where |
|---|---|---|
| Performance hints (`@inline`, `@cold`, `@hot`, `@unroll`…) | ❌ | [compiler-directives.md](compiler-directives.md) |
| `@async` / `@await` method decorators | ❌ | [concurrency.md](concurrency.md) |
| Doc-comment decorators (`@doc("…")`) | ❌ — documentation is `///` trivia, never decoration | [metadata-and-docs.md](metadata-and-docs.md) |
| A second `@observable` (persistence dirty-tracking, stdlib sketch) | ❌ duplicate | [reactive.md](reactive.md) |

### Framework families (design-ratified, v0.3+ horizon)

| Family | Decorators | Spec |
|---|---|---|
| Persistence | `@Entity @Column @Id @Index @Unique @Default @BelongsTo @HasMany` + lifecycle | [frameworks.md](frameworks.md) · 🧪 [frameworks-design.md](frameworks-design.md) |
| Web | `@Resource @Get @Post @Put @Patch @Delete @Route @Body @Param @Query @Header @Subscribe @Cron` | [frameworks.md](frameworks.md) · 🧪 [frameworks-design.md](frameworks-design.md) |

## Collision registry

Every known name conflict in the decorator namespace, with disposition:

| # | Collision | Disposition |
|---|---|---|
| COLL-1 | `@construct` (built derive) vs `@constructor` (ADR-0063 marker) | **Ruled** by ADR-0063 DEC-CTOR-E: two names one character apart is "a trap"; unified into one target-polymorphic `@constructor`. `@construct` is deleted with `ConstructDef` when U-CTOR lands. [placement.md](placement.md) |
| COLL-2 | web `@get`/`@delete` (HTTP verbs on methods) vs builtin `@get` (field accessor) | **Dissolved by the naming convention**: web decorators are library `Attribute` subclasses ⇒ `@Get(path:)`, `@Delete`. Zero registry overlap. [frameworks.md](frameworks.md) |
| COLL-3 | `@Retry`/`@Traced`/`@Lazy` attribute classes vs `Retry`/`Trace`/`Lazy` **proxy** classes (ADR-0057 kept both names) | **Open — needs a PDR.** Recommended: C#-style suffix resolution (`@Retry` resolves `RetryAttribute` first, then `Retry`), keeping ADR-0057's surface intact. Full options in [mechanism.md](mechanism.md) |
| COLL-4 | web `@on(event:)` vs builtin `@On(targets…)` | Case-distinct but one character apart — same trap shape COLL-1 rejected. Web draft's `@on` renamed `@Subscribe`/`@Handles`. [frameworks.md](frameworks.md) |
| COLL-5 | Phaldoc `///` tags (`@param`, `@deprecated`…) vs code decorators | **Disjoint by construction** (doc-comments-phaldoc.md §6): tags live in comment trivia, never tokenized. Standing obligation: every new Phaldoc tag is checked against the decorator registry. [metadata-and-docs.md](metadata-and-docs.md) |

## Defect ledger (spec-vs-code, verified 2026-07-20)

Divergences between what a spec/ADR states and what HEAD does. Each family file
carries the fix plan; this is the index. None of these is newly discovered by
this tree — the as-built files record most — but they have never been listed in
one place with owners.

| # | Defect | Where recorded | Fix owner |
|---|---|---|---|
| DEF-1 | `@ensures` documents a `result` binding; woven code binds `__result`, not in predicate scope — the documented surface cannot work | ensures.md "Not built" | [contracts.md](contracts.md) plan §1 |
| DEF-2 | ADR-0052's invariant re-entrancy guard (per-fiber `Set<ObjRef>`) is ratified and cited in the expander's doc comment, but the weave emits a plain entry/exit check — no guard exists | invariant.md | [contracts.md](contracts.md) plan §2 |
| DEF-3 | `@On(…, tier: Install)` — the ratified labeled-arg surface — does not parse; tier is bare-positional | on.md divergence 1 | [mechanism.md](mechanism.md) plan §1 |
| DEF-4 | Transitive `Attribute` subclasses are retained but never validated (`is_attribute_class` checks direct parent only) | on.md divergence 4 | [mechanism.md](mechanism.md) plan §2 |
| DEF-5 | `inherited:` (A-2) accepted and ignored; `attributesOfType` never chain-walks | on.md divergence 3 | [mechanism.md](mechanism.md) plan §3 |
| DEF-6 | No expander validates argument arity — `@native(foo)`, `@sealed(x)` compile with args silently discarded | native.md | [mechanism.md](mechanism.md) plan §4 |
| DEF-7 | `@get(priv)` argument parsed, never enforced; derives a public getter regardless | accessors.md | [derives.md](derives.md) |
| DEF-8 | `@set` on a `const` field derives a setter that either mis-fires `field.const_write` on synthesized code or silently violates `const` | docs/deferred/field-decorator-followups.md §1 | [derives.md](derives.md) |
| DEF-9 | `@sealed` cross-unit enforcement (`attr.sealed_violation`) is unreachable for user classes; live effect is only gating `@variant` | sealed.md / DEFERRED CB-3 | [derives.md](derives.md) |
| DEF-10 | `@native` anchors ⊆ installed-native-bindings invariant test not built; zero anchors exist | native.md | [subtractive.md](subtractive.md) |
| DEF-11 | Contract metadata (`MethodObject::contracts`) plumbed via `strip_metadata` but never emitted — the reflectable contract view (Phaldoc §8) has no data source | requires.md | [contracts.md](contracts.md) plan §3 |

## Reading order

1. [mechanism.md](mechanism.md) — grammar, registry, `Attribute`/`@On`,
   retention; the substrate everything else assumes.
2. [placement.md](placement.md) — `@class` and `@constructor` (ADR-0063).
3. [contracts.md](contracts.md), [derives.md](derives.md),
   [subtractive.md](subtractive.md) — the built Compile tier, verified.
4. [behavioral.md](behavioral.md), [interception.md](interception.md),
   [reactive.md](reactive.md) — the unbuilt runtime tiers, in build order;
   then the v0.3 designs: [runtime-tier.md](runtime-tier.md),
   [dispatch-tier.md](dispatch-tier.md), [effect.md](effect.md),
   [frameworks-design.md](frameworks-design.md).
5. [concurrency.md](concurrency.md), [compiler-directives.md](compiler-directives.md)
   — evaluations that end mostly in rejections, recorded so they stay rejected.
6. [metadata-and-docs.md](metadata-and-docs.md) — passive metadata and the
   documentation boundary.
7. [frameworks.md](frameworks.md) — persistence/web, verified and renamed.
8. [proposals.md](proposals.md) — new decorators originating in this tree.

## What this tree precludes

- **A decorator spec that contradicts an as-built file without flagging it.**
  The as-built layer is ground truth for HEAD; this tree layers intent and
  plans over it.
- **Ratification by stealth.** Nothing here flips a status. Proposed rulings
  (COLL-3, the naming convention, every 💡 row) are PDR candidates, cited as
  open until ratified.
- **A sixth tier, a second sigil, or a second drop mechanism** — inherited
  from ADR-0054, the tier model's own preclusions, and ignore.md.
