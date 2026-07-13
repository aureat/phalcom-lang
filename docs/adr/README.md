# Architecture Decision Records

This directory holds **Architecture Decision Records (ADRs)** — short documents
that capture a single significant design decision, the context that forced it,
and its consequences. They are the durable "why" behind the code: the parts you
can't recover by reading `class.rs` or querying the graphify graph.

## Format

We use [Michael Nygard's format](https://cognitect.com/blog/2011/11/15/documenting-architecture-decisions.html):
**Context → Decision → Consequences**, plus a **Status**. One decision per file.

Statuses: `Proposed`, `Accepted`, `Deprecated`, `Superseded by ADR-NNNN`.

## Conventions

- Files are named `NNNN-kebab-case-title.md`, numbered sequentially.
- ADRs are immutable once `Accepted`. To change a decision, write a **new** ADR
  that supersedes the old one, and update the old one's status to point at it.
- Keep them short (a page or two). Link to `docs/object-model.md` for full detail.

## Index

| ADR | Title | Status |
| --- | ----- | ------ |
| [0001](0001-record-architecture-decisions.md) | Record architecture decisions | Accepted |
| [0002](0002-metaclass-tower-parallel-rule.md) | Metaclass tower follows the parallel rule | Accepted |
| [0003](0003-introduce-behavior-kernel-class.md) | Introduce `Behavior` as a shared kernel class | Accepted |
| [0004](0004-boolean-as-abstract-bool-with-true-false.md) | Represent booleans as abstract `Bool` + `True`/`False` | Accepted |
| [0005](0005-number-as-flat-f64.md) | Keep a single flat `Number` type backed by `f64` | Superseded in part by [0024](0024-numeric-surface-split-int-float-and-division.md) |
| [0006](0006-function-as-abstract-callable-root.md) | `Function` as the abstract root of the callable tower | Accepted |
| [0007](0007-option-as-abstract-with-some-none.md) | Represent absence as abstract `Option` + `Some`/`None` | Accepted |
| [0008](0008-layered-exceptions-and-result.md) | Layered exceptions + `Result`, with terminating semantics | Accepted |
| [0009](0009-handle-arena-heap.md) | Object graph lives in a handle/arena heap | Accepted |
| [0010](0010-tagged-value-enum.md) | `Value` is a tagged `enum` with a private `Nil` sentinel | Accepted |
| [0011](0011-static-instance-slot-layout.md) | Instances use a static per-class slot layout | Accepted |
| [0012](0012-selector-signature-encoding-and-dispatch.md) | Label-encoded selectors and inline-cache-ready dispatch | Accepted |
| [0013](0013-closure-upvalues-and-frame-token-return.md) | Open/closed upvalues and frame-token non-local return | Accepted |
| [0014](0014-let-and-var-bindings.md) | Variable bindings are `let` (immutable) and `var` (mutable) | Accepted |
| [0015](0015-object-default-tostring.md) | `Object` default `toString` is `"<ClassName>"` | Accepted |
| [0016](0016-hand-written-lexer-and-recursive-descent-parser.md) | Hand-written lexer and recursive-descent parser (replacing LALRPOP) | Accepted |
| [0017](0017-class-side-stored-static-fields.md) | Class-side stored static fields live on the metaclass instance | Accepted |
| [0018](0018-sacred-selector-inliner-and-override-guard.md) | Sacred-selector inliner with override-epoch deopt guard | Accepted |
| [0019](0019-freeze-vm-blessed-primitive-floor.md) | Freeze the VM-blessed primitive floor | Accepted |
| [0020](0020-kernel-list-native-array-protocol.md) | Kernel `List` is a native-array-backed protocol on the critical path | Accepted |
| [0021](0021-no-truthiness-enforcement.md) | No-truthiness enforcement: typed branch floor + literal-only compile check | Accepted |
| [0022](0022-string-interpolation-backslash-paren-sigil.md) | String interpolation uses the `\(expr)` sigil | Accepted |
| [0023](0023-amend-floor-admit-hash-and-kernel-reflection.md) | Amend the frozen floor to admit `hash` + kernel reflection | Accepted |
| [0024](0024-numeric-surface-split-int-float-and-division.md) | Split `Number` into exact `Int` (bignum) + `Float`; `/` true division, `~/` integer division | Accepted |
| [0025](0025-external-internal-parameter-names.md) | Separate external labels from internal parameter names | Accepted |
| [0026](0026-class-hierarchy-mutability.md) | Methods are open; superclass reparenting is sealed | Accepted |
| [0027](0027-modules-as-files-with-public-by-default-imports.md) | A module is a file; public-by-default exports; qualified/selective/aliased imports | Accepted |
| [0029](0029-list-literal-syntax.md) | List literals `[a, b, c]` desugar to `List` construction sends (no new floor) | Accepted |
| [0030](0030-fibers-and-futures-cooperative-concurrency.md) | Fibers and Futures: cooperative concurrency on a restricted re-entrant loop | Accepted |
| [0031](0031-error-handling-surface-syntax.md) | Error-handling surface syntax: `throw`/`try`/`catch`/`on`/`ensure` | Accepted |
| [0032](0032-collections-representation-and-literals.md) | Collections: native representation, shared protocol, and literal surface | Accepted |
| [0033](0033-amend-fiber-execution-trampolined-block-callsite.md) | Amend fiber execution (ADR-0030 §4): trampoline the bytecode block call-site | Deferred |
| [0035](0035-iteration-protocol-cursor.md) | Iteration protocol: a Wren-style two-selector cursor | Accepted |
| [0052](0052-invariant-reentrancy-scope-and-layout-confined-decorator-state.md) | Invariant re-entrancy is receiver-scoped; per-receiver decorator state is Layout-confined | Proposed |
| [0053](0053-runtime-decorator-interception-reuses-override-epoch-guard.md) | Runtime-tier decorator interception reuses the sacred-selector override-epoch guard | Proposed |
| [0054](0054-two-speed-ratification-annotation-decorator-tiers.md) | Two-speed ratification: annotation Compile/Layout tier now, Install/Dispatch/Runtime gated on ADR-0053 | Accepted (Compile/Layout only) |
| [0057](0057-decorator-granularity-vs-proxy-granularity-split.md) | Decorator vs proxy granularity: method-declaration (`@name`) vs whole-object (`Proxy`) interception; both kept | Accepted |

> ADR-0029 (list literals) is now **Accepted** — its sub-decisions (desugar-to-sends,
> no subscript sugar, trailing comma) were ratified with the collections umbrella
> [ADR-0032](0032-collections-representation-and-literals.md). Design in
> [`docs/spec/v0.2/core/list-literal-syntax.md`](../spec/v0.2/core/list-literal-syntax.md).
> (0028 is claimed by a concurrent unit's `Method`-reflection floor amendment.)

> ADR-0032 (collections) was ratified by the user on 2026-07-12: native heap-arm
> representation for `Map`/`Set`/`Tuple`/`Range`, the binding collection protocol,
> and the literal surface — `[…]` / `{k:v}` / `(a,b)` ship; `#{…}` (set) and
> `..`/`...` (range) are reserved-inactive with committed meaning. It flips ADR-0029
> and the four `core/` collection specs to Accepted.

> ADR-0030 (Fibers & Futures) was ratified by the user on 2026-07-12, resolving the
> concurrency execution-model open decision (open-question 15 → **Option A**, the
> restricted re-entrant loop). It promotes the
> [`concurrency-adr.md`](../spec/v0.2/experimental/concurrency-adr.md) draft and the
> [`forward-compat.md §7`](../spec/v0.2/core/forward-compat.md) code-grounded audit.

> ADR-0031 (error surface syntax) was ratified by the user on 2026-07-12: the
> `throw`/`try`/`catch`/`on`/`ensure` spelling, 1:1 sugar over the ADR-0008 block
> protocol (`ensure` mirrors `.ensure{}`; `on T e` mirrors `.on(T){}`). It settles
> the surface ADR-0008 left illustrative; the error *model* is unchanged.

> ADR-0035 (iteration protocol) was ratified by the user on 2026-07-12, promoting
> `experimental/iteration-protocol.md` to normative
> [`iteration.md`](../spec/v0.2/iteration.md): a Wren-style `iterate`/`iteratorValue`
> cursor, the `for` desugar, `break`/`continue`, and the inliner/`Fiber` interactions.
> (`Result`/`Ok`/`Err` was promoted to normative [`result.md`](../spec/v0.2/result.md)
> in the same session under the existing ADR-0008.)

> ADR-0033 (`CallBlock` trampoline) is **Deferred past v0.2**: the callback-generator
> ergonomic it targeted — a `Fiber.yield` inside iteration — is already delivered for
> v0.2 by ADR-0035's `for`/cursor loop (`for (x in coll) { Fiber.yield(x) }` lowers to
> an inlined `while` that suspends freely under ADR-0030 §4). ADR-0033 remains the
> general lift (`.each { yield }` and other native-callback generators) for when a case
> `for` cannot express becomes a real need, to land with the ADR-0030 §5 fiber-switch
> signal. (An earlier draft ADR-0034 that instead inlined `each` was dropped: it
> contradicted ADR-0035 §4/§6, which deliberately keep iteration selectors non-inlined
> and `.each { yield }` raising `CannotYieldAcrossNativeFrame`.)

> ADRs 0024–0027 were ratified by the user on 2026-07-12, resolving open-questions
> Q2 (numeric split + division), Q3 (parameter names), Q4 (hierarchy mutability), and
> Q8 (modules). See [`docs/spec/v0.2/open-questions.md`](../spec/v0.2/open-questions.md).

> ADRs 0019–0020 were ratified by the user on 2026-07-11, clearing the U-LIST-plan
> §0 gate: they derive from the experimental draft
> `docs/spec/v0.2/experimental/bootstrapping-and-self-hosting.md` (D1, D2/DEC-A). 0019
> consolidates an existing boundary; 0020 resolves DEC-A.

> ADR-0052 amends two unratified annotation/decorator drafts found during spec
> review: `experimental/annotations-contract-semantics.md`'s `@invariant`
> re-entrancy guard (fiber-global counter → receiver-scoped, unwind-safe
> identity set) and `next/decorators-stdlib.md`'s `@computed` (Install-tier
> receiver-keyed cache, which leaked every receiver forever → reclassified to
> Layout tier per `attribute-classes.md`'s own `@lazy` pattern). Not yet
> ratified by the user.

> ADR-0053 and ADR-0054 continue the same review: 0053 gives Runtime-tier
> decorator interception (`aroundSend`) an explicit cost model by reusing
> ADR-0018's override-epoch guard (Proposed, not yet ratified — it only
> matters once Install/Dispatch/Runtime are); 0054 resolves the
> annotations-core.md vs. decorators.md fork by ratifying the Compile/Layout
> tier and gating Install/Dispatch/Runtime on 0053 plus attribute-classes.md's
> remaining open questions (A-1–A-6, not yet resolved). **0054's Compile/Layout
> ratification was accepted by the user on 2026-07-13** — `annotations-core.md`,
> `annotations-legality-grammar.md`, `annotations-contracts.md`,
> `annotations-contract-semantics.md` (as amended by 0052), `annotations-construct.md`,
> `annotations-construct-inheritance.md` (as amended by the super-signature-inference
> fix), and `annotation-paradigm-bridges.md`'s tier line are now the normative
> design for `@` on the Compile/Layout tier. Install/Dispatch/Runtime remain
> unratified.

> ADRs 0003–0008 were ratified in the object-model / language-design sessions, and
> ADRs 0009–0015 in the Phase-2 VM-architecture session (2026-07-11); all are now
> **Accepted** — the design baseline for implementation. Per the immutability
> convention, changing any of them requires a new superseding ADR.
