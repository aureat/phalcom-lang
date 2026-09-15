# Deferred & Future Work

Part of the [Phalcom Language Specification](README.md). Status: Draft 0.1.

This is the **single map of everything Phalcom has deliberately postponed** — design
decisions that were decided-to-defer, decisions still genuinely open, implementation
units not yet built, and the exploratory design corpus. It exists because deferral
state was previously smeared across five registers that did not cross-link; this file
is the index over them.

> **It does not duplicate the detail** in those registers — each item names its source
> of record. When an item lands, update *its source* and strike it here.

> **Baseline:** HEAD `0b21e60` (2026-07-12). Landed and therefore **not** listed below:
> U1–U11, U-LIST, U-FE, U-LEX, U-STD, U-CORE-1 (kernel reflection), U-CORE-2 (bulk), and
> TDR-0022–0027 (numeric split, parameter names, hierarchy mutability, modules). The
> `Int`/`Float` split itself is **decided** (TDR-0022) — only its *substrate
> implementation* is future work (see §3).

## The five source registers this indexes

| Register | Owns | Path |
|---|---|---|
| **This file** | the map + deferred *design decisions* + genuinely-open decisions | `docs/spec/deferred-work.md` |
| Open questions | the 14 resolved language questions (decision record) | [`open-questions.md`](open-questions.md) |
| Forge backlog | ranked implementation nits / DX / bug items surfaced by forge | [`forge/DEFERRED.md`](../../forge/DEFERRED.md) |
| Forward-compat | "must not preclude" hazards for the 4 unbuilt subsystems | [`core/forward-compat.md`](core/forward-compat.md) |
| Pending fixtures | ignored test fixtures → blocker/owner unit (executable view) | [`core/pending-retirement.md`](core/pending-retirement.md) |
| Catalog delta | per-class reserved-but-unbuilt class names (`Map`/`Range`/`Fiber`/…) | [`core/catalog-delta.md`](core/catalog-delta.md) |

---

## 1. Deferred design decisions (decided-to-defer; non-foreclosed)

Each was consciously ruled "not now" during the open-questions sweep or a forge unit,
with the door left open. Source is the decision record.

> **Confirmation (2026-07-12).** This session reviewed the underspecified items
> flagged for a keep-or-pull-in decision and confirmed **all stay deferred**, each
> with its recorded owner: **default arguments** → the (if-ever) definition-time
> overload-desugar; **list/`*rest` destructuring** → the pattern-matching unit;
> **`Family` reflective mirror** → the unified reflection unit; **circular-import
> policy** → the module unit; **`System` surface** → the System unit (§3);
> **`Int`/`Float` substrate + `number_hash` bignum-collision fix** → the
> numeric-substrate unit (§3/§4). None was pulled into current scope.

| Decision | Ruling / future shape | Gate — why deferred | Source · owner |
|---|---|---|---|
| **Stateful mixins / multiple inheritance** | single class inheritance remains; **stateless method-only traits** are ratified by the C3 extension | state-bearing MI breaks the TDR-0010/0017 fixed slot offsets | [open-Q10](open-questions.md); [First-Class Traits](../extensions/traits.md) |
| **Default arguments** | none now; if ever, **desugar to trailing-only arity-family overloads at definition time**; call-site resolution **permanently forbidden** | incompatible with selector-identity dispatch; mechanism fixed so a later add is non-breaking | [open-Q12](open-questions.md); [TDR-0036](../../decisions/accepted/0036-no-default-arguments-keep-selector-identity-pristine.md); [drafts/default-arguments.md](drafts/default-arguments.md) |
| **List/`*rest` destructuring + pattern matching** | irrefutable **tuple** destructuring ships; refutable `let [first, *rest]`, `match`/`if let`, map patterns deferred | refutable bind needs a failure branch (a pattern-matching unit); reuses U9 `*rest` | [open-Q7](open-questions.md); ruling |
| **`Family` reflective mirror** | `Family` callable-only now; `.candidates`/`.arities`/`.respondsTo` mirror deferred | design it with the U8 `Message`/`perform` surface as one reflection API | [open-Q14](open-questions.md); [experimental/bound-callable-unification.md](experimental/bound-callable-unification.md) |
| **Set literal sigil** | `Set(...)` constructor ships; the `#{ }` literal is **reserved-inactive** with committed meaning ([TDR-0028](../../decisions/accepted/0028-collections-representation-and-literals.md)) | additive sugar; activate in a later U-LEX slice | [open-Q6](open-questions.md); [TDR-0028](../../decisions/accepted/0028-collections-representation-and-literals.md) |
| **`Some` niche-encoding** | `Some(x)` stays an ordinary heap instance; niche-encode into `Value` later | wait for a GC + benchmarks; slots behind the existing `surface_none` boundary | [open-Q13](open-questions.md); TDR-0006/0010 |
| **`reshape` / superclass reparenting** | reparenting sealed by policy; opt-in `reshape`-with-migration left open | would shift TDR-0010/0017 offsets; TDR-0008 keeps it implementable | [TDR — Methods are open; superclass reparenting is sealed](../../decisions/retired/class-hierarchy-mutability.md) |
| ~~**Collection-literal lowering** `(a,b)`/`[…]`/`{a:1}`~~ | **Ratified** ([TDR-0028](../../decisions/accepted/0028-collections-representation-and-literals.md)): list/map/tuple literals desugar to construction sends; `{}` stays a block, set/range reserved | implementation → U-LEX (§3) | [TDR-0028](../../decisions/accepted/0028-collections-representation-and-literals.md) |
| **Parameterized / first-class modules** | file-modules ship (TDR-0024); first-class module objects deferred | out of core scope → module unit; a module object can subsume file-modules | [open-Q8](open-questions.md); [core/forward-compat.md §3](core/forward-compat.md) |
| **Circular-import policy** | hard-error vs lazy-binding unspecified | implementation detail of the module unit, not a language decision | [TDR — A module is a file; exports are public by default; imports are qualified, selective, or aliased](../../decisions/retired/modules-as-files-with-public-by-default-imports.md) |

### Live re-opening concerns (recorded, *not* adopted)
Resolutions that stand but were flagged as worth a future revisit:
- **`var x` ⇒ `None` vs a VM-only `Uninit` trap** — with uninitialized `var` reading as
  `None`, every variable is effectively `T | None`. A `Uninit`-sentinel-that-traps-on-read
  alternative keeps `None` a *chosen* absence. Not adopted. ([open-Q1 tail](open-questions.md); ADR-0014)
- **`ifTrue`/`ifFalse` → `Option` chaining** — `cond.ifTrue{…}.ifFalse{…}` sends `ifFalse`
  to an `Option`, not a `Bool`; a paired `ifTrue(_)ifFalse(_)` primary was floated. Not
  adopted. ([open-Q1 tail](open-questions.md); TDR-0006/0018)

> **Reviewed 2026-07-12 — both stand.** This session's decision round confirmed
> keeping `var x ⇒ None` (no trapping `Uninit` sentinel) and the `Option`-returning
> `ifTrue`/`ifFalse` (no paired-primary revision). They remain recorded as live
> concerns for a future revisit, not adopted.

---

## 2. Genuinely-open decisions (not yet made)

**None.** The three that stood here — the concurrency execution model, the error
surface syntax, and the collections representation/literals — were all ratified on
2026-07-12:

| Was-open decision | Resolved by |
|---|---|
| Concurrency re-entrant-loop model | [TDR-0026](../../decisions/accepted/0026-fibers-and-futures-cooperative-concurrency.md) (Option A — restricted re-entrant loop) |
| Error surface syntax | [TDR-0027](../../decisions/accepted/0027-error-handling-surface-syntax.md) (`throw`/`try`/`catch`/`on`/`ensure`) |
| Collections representation + literals | [TDR-0028](../../decisions/accepted/0028-collections-representation-and-literals.md) (native arms; list/map/tuple literals; set/range reserved) |

Remaining work is **implementation** (§3) and the **decided-to-defer** design
decisions (§1), not open decisions.

---

## 3. Deferred implementation units

The successor track lives in [`core/README.md`](core/README.md) (index of record).
Reserved-but-unbuilt class names sit in `primitive/mod.rs::ClassName` (`Range`, `Map`,
`Fiber`, `Future`).

| Unit / feature | What it is | Gate / status | Source |
|---|---|---|---|
| **U-CORE-3** callables reflection | `methodFor`/`invokeOn`/`bind`/`signature`/`holder` (+5 floor) | dispatch-ready; **next** on the track | [core/U-CORE-3](../../forge/units/U-CORE-3/ucore3.md) |
| **U-CORE-4** value `toString` | per-type `toString` (`Number`/`String`/`Symbol`/`Bool`/`Option`) | dispatch-ready; closes DEFERRED #19/#30, F4 | [core/U-CORE-4](../../forge/units/U-CORE-4/ucore4.md) |
| **U-CORE-5** collection contract | shared protocol contract + `.ph` `List#==` | dispatch-ready; deps U-CORE-1 `isA` (landed) | [core/U-CORE-5](../../forge/units/U-CORE-5/ucore5.md) |
| **U-CORE-6** errors | `Error` root + `MessageNotUnderstood` raise; reserve `Result`/`Ok`/`Err` | dispatch-ready; error surface **ratified** ([TDR-0027](../../decisions/accepted/0027-error-handling-surface-syntax.md)) | [core/U-CORE-6](../../forge/units/U-CORE-6/ucore6.md) |
| **`Int`/`Float` substrate** | build the TDR-0022 split: `Value::Int(i64)`/`Float(f64)`, heap `LargeInt` bignum, `checked_*` promotion, `~/` opcode, cross-repr `==`/`hash` | **decided** (TDR-0022); code unbuilt; see §4 hash flag | [TDR-0022](../../decisions/accepted/0022-numeric-surface-split-int-float-and-division.md) |
| **Collections classes** `Map`/`Set`/`Tuple`/`Range` | whole classes + storage + literals | `Object#hash` landed; representation + literals **ratified** ([TDR-0028](../../decisions/accepted/0028-collections-representation-and-literals.md)); code unbuilt (U-STD) | [core/catalog-delta.md](core/catalog-delta.md) |
| **Collection literal syntax** | `[a,b,c]` / `{k:v}` / `(a,b)` → constructor desugar (set `#{…}` / range `..` reserved) | **ratified** ([TDR-0028](../../decisions/accepted/0028-collections-representation-and-literals.md)); lexer/parser → U-LEX | forge/DEFERRED.md #28 |
| **Module / import unit** | `import` semantics per TDR-0024 (qualified/selective/aliased), namespace protocol | token exists, semantics unbuilt | [TDR — A module is a file; exports are public by default; imports are qualified, selective, or aliased](../../decisions/retired/modules-as-files-with-public-by-default-imports.md) |
| **System unit** | `System.args`/`clock`/`gc`/scheduler surface | pending `system_*` fixtures | [core/pending-retirement.md](core/pending-retirement.md) |
| **Concurrency** `Fiber`/`Future` | cooperative coroutines + async layer | surface + execution model **ratified** ([TDR-0026](../../decisions/accepted/0026-fibers-and-futures-cooperative-concurrency.md), Option A); code unbuilt | [concurrency.md](concurrency.md); experimental/ |
| **Typing layer** | optional/structural/erasable gradual types | experimental, uncommitted | [experimental/typing.md](experimental/typing.md) |
| **Annotations `@`** | `@attr` mechanism, contracts, `@construct`/`@get`/`@set` | experimental (10 drafts), `@` not lexed | experimental/annotations-*.md |
| **`Result`/`Ok`/`Err`** | `Option`-mirrored expected-failure channel + bridges | design **normative** ([result.md](result.md); TDR-0007); reserved by U-CORE-6, built by a later value-classes unit | [result.md](result.md); TDR-0007 |
| **U-STRING follow-ons** | `System.print(_)`/`writeObject_(_)` funnel unification (blocked on `Map`/`Set`/`Tuple`/`Range`/instance `toString` message wording), character (codepoint) indexing, derived regex-free search helpers | **U-STRING landed** ([TDR — Amend floor — admit `String` raw byte accessors + `System.rawWrite(_)`](../../decisions/retired/amend-floor-admit-string-raw-byte-accessors-supersedes-0049-naming.md)); these three follow-ons unbuilt | [core/floor-census.md §2.5/§2.11](core/floor-census.md) |

---

## 4. Forward-looking implementation flags

Concrete things that must be handled **when a specific unit lands** — surfaced here so
they aren't lost:

- **`number_hash` 53-bit masking vs bignum `Int`.** U-CORE-1's implemented `number_hash`
  masks the `f64` to 53 bits. Under TDR-0022's exact bignum `Int`, two integers differing
  above 2⁵³ would collide, and a bignum `Int` must still hash equal to the `Float` of the
  same value. **Revisit when the TDR-0022 substrate (§3) is implemented.** (core/forward-compat.md §4)
- **Interpolation desugar target.** `\(expr)` currently desugars to `String.new(_)`, not a
  content `toString` — blocked on U-CORE-4's value `toString`. (forge/DEFERRED.md #30)
- **`None`-reopen clobber.** `Statement::Class` unconditionally emits `DefineGlobal`, which
  would clobber the immediate `None` value if `None` is reopened — fix before real `None`
  members land. (forge/DEFERRED.md #17)
- **`SendDynamic` opcode + spread `f(*args)`.** The opcode and call-site spread syntax are
  not built; no spread syntax exists yet. (forge/DEFERRED.md #21)
- **Captured-`let` reassignment** via an upvalue compiles to `SetUpvalue` with no diagnostic
  (U6's check is syntactic, current-fn + module only). (forge/DEFERRED.md #13)
- **Runtime-tier interceptor-chain caching.** Once Install/Dispatch/Runtime decorators
  (`decorators/README.md`, `decorators/on.md`) are built: pre-compose a class's
  chained `aroundSend` interceptors into one fused closure at class-definition time
  (not a per-send list walk), cache it behind TDR-0044's `has_runtime_interceptor`
  guard bit, and specialize the common single-interceptor case. Safe with no
  invalidation logic because the retained-attribute store is frozen post-definition
  (attribute-classes.md A-5) — pure work-hoisting, not speculation. (`decorators/README.md`
  "Future optimizations")

The full ranked list of ~33 such items is [`forge/DEFERRED.md`](../../forge/DEFERRED.md).

---

## 5. The `experimental/` design corpus

[`experimental/`](experimental/README.md) is the **staging area for unratified design** —
proposals promote to `adr/` + `spec/` on ratification. Grouped by the subsystem each will
feed:

- **Concurrency** — [concurrency-adr.md](experimental/concurrency-adr.md) (promoted → [TDR-0026](../../decisions/accepted/0026-fibers-and-futures-cooperative-concurrency.md)), [scheduler-unit.md](experimental/scheduler-unit.md), [fiber-ensure-and-limits.md](experimental/fiber-ensure-and-limits.md)
- **Iteration** — [iteration-protocol.md](experimental/iteration-protocol.md) (promoted → [TDR-0029](../../decisions/accepted/0029-iteration-protocol-cursor.md) + normative [iteration.md](iteration.md))
- **Indexing / numeric** — [numeric-and-string-indexing.md](experimental/numeric-and-string-indexing.md) *(its integral-index + codepoint-string decisions are keep-worthy; its "f64 / 2⁵³ / bignum-deferred / split-open" claims are **superseded by TDR-0022**)*
- **Equality / hash** — [equality-and-hash.md](experimental/equality-and-hash.md) *(heavy overlap with the now-landed TDR-0021 + core/decisions.md Q1/Q5 — candidate to become normative; de-dupe first)*
- **Typing** — [typing.md](experimental/typing.md) + `typing-initialization`/`-subtyping`/`-inference`/`-stdlib-surface`
- **Annotations** — [annotations-core.md](experimental/annotations-core.md) + 7 satellites + [annotation-paradigm-bridges.md](experimental/annotation-paradigm-bridges.md) *(interacts with TDR-0023 param names + ADR-0026 `@sealed`)*
- **Self-hosting** — [bootstrapping-and-self-hosting.md](experimental/bootstrapping-and-self-hosting.md) *(its open-Q8 module dependency is now TDR-0024)*
- **Callables** — [bound-callable-unification.md](experimental/bound-callable-unification.md) (open-Q14; mirrors the landed `Method < Function`)
- **Default arguments** — [drafts/default-arguments.md](drafts/default-arguments.md) *(**chore discharged 2026-07-15, DEFERRED CB-4**: `experimental/default-arguments.md` was retired rather than reconciled — its body advocated the caller-side desugar its own banner permanently forbade, and the draft already carried its content correctly. TDR-0037 stands.)*
- **Doc comments** — [doc-comments-phaldoc.md](experimental/doc-comments-phaldoc.md) (inert `///`/`//!` convention)
