# Architecture Decision Records

This directory holds **Architecture Decision Records (ADRs)** — short documents
that capture a single significant design decision, the context that forced it,
and its consequences. They are the durable "why" behind the code: the parts you
can't recover by reading `class.rs` or querying the graphify graph.

## Format

We use [Michael Nygard's format](https://cognitect.com/blog/2011/11/15/documenting-architecture-decisions.html):
**Context → Decision → Consequences**, plus a **Status**. One decision per file.

Statuses: `Proposed`, `Accepted`, `Deprecated`, `Deferred`, `Superseded by ADR-NNNN`.
Qualified variants of `Accepted` (e.g. `Accepted (all tiers)`) appear when an ADR
ratifies a decision in stages — see the ADR's own Status line for the scope.

## Conventions

- Files are named `NNNN-kebab-case-title.md`, numbered sequentially, and live
  under one of three status folders: `accepted/`, `proposed/`, `retired/`
  (`retired/` also covers `Deprecated`/`Deferred`/`Superseded`, mirroring
  `STATUS.md`'s bucket rule). A file moves folders the moment its Status line
  changes — same edit pass, per `STATUS.md`'s two-way sync rule.
- ADRs are immutable once `Accepted`. To change a decision, write a **new** ADR
  that supersedes the old one, and update the old one's status to point at it
  (which also moves it to `retired/`).
- Keep them short (a page or two). Link to `docs/object-model.md` for full detail.

## Next ADR number

**Last used: 0060.** The next new ADR is **0061**. Numbers are no longer
discoverable by `ls`-ing one flat directory — files are split across
`accepted/`/`proposed/`/`retired/` — so this line is the source of truth.
**Whenever an agent writes an ADR's Status, it must also update this line**
(bump it when adding a new ADR; the line itself doesn't change for a plain
status flip on an existing one). If this line and the highest number actually
on disk (`ls accepted proposed retired | grep -oE '^[0-9]{4}' | sort -n | tail -1`)
ever disagree, trust disk and fix this line.

## Index

| ADR | Title | Status |
| --- | ----- | ------ |
| [0001](accepted/0001-record-architecture-decisions.md) | Record architecture decisions | Accepted |
| [0002](accepted/0002-metaclass-tower-parallel-rule.md) | Metaclass tower follows the parallel rule | Accepted |
| [0003](accepted/0003-introduce-behavior-kernel-class.md) | Introduce `Behavior` as a shared kernel class | Accepted |
| [0004](accepted/0004-boolean-as-abstract-bool-with-true-false.md) | Represent booleans as abstract `Bool` + `True`/`False` | Accepted |
| [0005](retired/0005-number-as-flat-f64.md) | Keep a single flat `Number` type backed by `f64` | Superseded in part by [0024](accepted/0024-numeric-surface-split-int-float-and-division.md) |
| [0006](accepted/0006-function-as-abstract-callable-root.md) | `Function` as the abstract root of the callable tower | Accepted |
| [0007](accepted/0007-option-as-abstract-with-some-none.md) | Represent absence as abstract `Option` + `Some`/`None` | Accepted |
| [0008](accepted/0008-layered-exceptions-and-result.md) | Layered exceptions + `Result`, with terminating semantics | Accepted |
| [0009](accepted/0009-handle-arena-heap.md) | Object graph lives in a handle/arena heap | Accepted |
| [0010](accepted/0010-tagged-value-enum.md) | `Value` is a tagged `enum` with a private `Nil` sentinel | Accepted |
| [0011](accepted/0011-static-instance-slot-layout.md) | Instances use a static per-class slot layout | Accepted |
| [0012](accepted/0012-selector-signature-encoding-and-dispatch.md) | Label-encoded selectors and inline-cache-ready dispatch | Accepted |
| [0013](accepted/0013-closure-upvalues-and-frame-token-return.md) | Open/closed upvalues and frame-token non-local return | Accepted |
| [0014](accepted/0014-let-and-var-bindings.md) | Variable bindings are `let` (immutable) and `var` (mutable) | Superseded by ADR-0064 |
| [0015](accepted/0015-object-default-tostring.md) | `Object` default `toString` is `"<ClassName>"` | Accepted |
| [0016](accepted/0016-hand-written-lexer-and-recursive-descent-parser.md) | Hand-written lexer and recursive-descent parser (replacing LALRPOP) | Accepted |
| [0017](accepted/0017-class-side-stored-static-fields.md) | Class-side stored static fields live on the metaclass instance | Accepted |
| [0018](accepted/0018-sacred-selector-inliner-and-override-guard.md) | Sacred-selector inliner with override-epoch deopt guard | Accepted |
| [0019](accepted/0019-freeze-vm-blessed-primitive-floor.md) | Freeze the VM-blessed primitive floor | Accepted |
| [0020](accepted/0020-kernel-list-native-array-protocol.md) | Kernel `List` is a native-array-backed protocol on the critical path | Accepted |
| [0021](accepted/0021-no-truthiness-enforcement.md) | No-truthiness enforcement: typed branch floor + literal-only compile check | Accepted |
| [0022](accepted/0022-string-interpolation-backslash-paren-sigil.md) | String interpolation uses the `\(expr)` sigil | Accepted |
| [0023](accepted/0023-amend-floor-admit-hash-and-kernel-reflection.md) | Amend the frozen floor to admit `hash` + kernel reflection | Accepted |
| [0024](accepted/0024-numeric-surface-split-int-float-and-division.md) | Split `Number` into exact `Int` (bignum) + `Float`; `/` true division, `~/` integer division | Accepted |
| [0025](accepted/0025-external-internal-parameter-names.md) | Separate external labels from internal parameter names | Accepted |
| [0026](accepted/0026-class-hierarchy-mutability.md) | Methods are open; superclass reparenting is sealed | Accepted |
| [0027](retired/0027-modules-as-files-with-public-by-default-imports.md) | A module is a file; public-by-default exports; qualified/selective/aliased imports | Accepted |
| [0029](accepted/0029-list-literal-syntax.md) | List literals `[a, b, c]` desugar to `List` construction sends (no new floor) | Accepted |
| [0030](accepted/0030-fibers-and-futures-cooperative-concurrency.md) | Fibers and Futures: cooperative concurrency on a restricted re-entrant loop | Accepted |
| [0031](accepted/0031-error-handling-surface-syntax.md) | Error-handling surface syntax: `throw`/`try`/`catch`/`on`/`ensure` | Accepted |
| [0032](accepted/0032-collections-representation-and-literals.md) | Collections: native representation, shared protocol, and literal surface | Accepted |
| [0033](retired/0033-amend-fiber-execution-trampolined-block-callsite.md) | Amend fiber execution (ADR-0030 §4): trampoline the bytecode block call-site | Deferred |
| [0035](accepted/0035-iteration-protocol-cursor.md) | Iteration protocol: a Wren-style two-selector cursor | Accepted |
| [0036](accepted/0036-amend-floor-admit-number-tostring.md) | Amend the frozen floor — admit `Number#toString` | Accepted |
| [0037](accepted/0037-amend-floor-admit-error-root.md) | Amend the frozen floor — admit `Error#message`/`Error#raise` | Accepted |
| [0038](accepted/0038-amend-floor-admit-block-on-ensure.md) | Amend the frozen floor — admit `Block#on`/`Block#ensure` (error handling) | Accepted |
| [0039](accepted/0039-amend-floor-admit-collection-container-primitives.md) | Amend the frozen floor — admit collection-container primitives (`Map`/`Set`/`Tuple`/`Range`) | Accepted |
| [0040](accepted/0040-supersend-opcode.md) | Add the `SuperSend` dispatch opcode for `super.sel(…)` | Accepted |
| [0041](accepted/0041-hierarchy-stability-policy.md) | Hierarchy-stability policy: sealed reparenting + single inheritance | Accepted |
| [0042](retired/0042-flat-number-defer-integer-float-split.md) | Flat `Number` now; defer the `Integer`/`Float` split | Superseded by [0024](accepted/0024-numeric-surface-split-int-float-and-division.md) |
| [0043](accepted/0043-no-default-arguments-keep-selector-identity-pristine.md) | No default arguments; keep selector identity pristine | Accepted |
| [0044](accepted/0044-option-bootstrap-formalization-and-defer-niche-encoding.md) | `Option` bootstrap formalization; defer niche-encoding | Accepted |
| [0045](accepted/0045-module-import-relative-path-whole-module-binding.md) | `import` resolves by relative file path and binds a whole `Module`; amend the frozen floor +1 (`Module#doesNotUnderstand`) | Accepted |
| [0046](accepted/0046-destructuring-bindings.md) | Destructuring `let`/`var` bindings — irrefutable tuple + list, `at(_)` protocol | Accepted |
| [0047](accepted/0047-amend-floor-admit-family-call-router.md) | `::` method references (Open form, callable-only); amend the frozen floor +1 (`Family#doesNotUnderstand`) | Accepted |
| [0048](accepted/0048-amend-iteration-bare-cursor-sentinel-and-iterable-root.md) | Amend iteration: bare-cursor end-sentinel + kernel `Iterable` root | Accepted |
| [0049](accepted/0049-amend-floor-admit-string-byte-and-raw-write-primitives.md) | Amend the floor: admit String byte/slice accessors + raw stdout write | Accepted |
| [0050](accepted/0050-non-moving-mark-sweep-collector.md) | Reclamation is a non-moving precise mark-sweep collector | Accepted |
| [0051](accepted/0051-performance-strategy-measure-first-tiered-optimization.md) | Performance strategy: measure-first, tiered, behavior-invariant | Accepted |
| [0052](accepted/0052-invariant-reentrancy-scope-and-layout-confined-decorator-state.md) | Invariant re-entrancy is receiver-scoped; per-receiver decorator state is Layout-confined | Accepted |
| [0053](accepted/0053-runtime-decorator-interception-reuses-override-epoch-guard.md) | Runtime-tier decorator interception reuses the sacred-selector override-epoch guard | Accepted |
| [0054](accepted/0054-two-speed-ratification-annotation-decorator-tiers.md) | Two-speed ratification: annotation Compile/Layout tier + Install/Dispatch/Runtime tier, both gates satisfied | Accepted (all tiers) |
| [0055](retired/0055-index-syntax-sugar-over-at-selectors.md) | Subscript indexing syntax sugar over `at`/`at:put:` selectors | Retired — superseded by [0060](accepted/0060-index-operator-as-real-selector.md) |
| [0056](proposed/0056-phalcom-lsp-architecture.md) | Phalcom language intelligence is an in-process `phalcom-lsp` server | Proposed |
| [0057](accepted/0057-decorator-granularity-vs-proxy-granularity-split.md) | Decorator vs proxy granularity: method-declaration (`@name`) vs whole-object (`Proxy`) interception; both kept | Accepted |
| [0058](accepted/0058-reactive-tracking-context-needs-a-native-module.md) | Reactive tracking-context and effect scheduler need a native module, not class-side `.ph` state | Accepted |
| [0059](proposed/0059-amend-reactive-tracking-context-native-frame-coupling.md) | Amend ADR-0058 + ADR-0033: the reactive tracking context is bound to the native-frame switch guard | Proposed |
| [0060](accepted/0060-index-operator-as-real-selector.md) | `[]` is a real, directly-sent selector (`[_]`/`[_,put]`/`[]`/`[put]`) — no `at` lowering | Accepted |

> ADR-0029 (list literals) is now **Accepted** — its sub-decisions (desugar-to-sends,
> no subscript sugar, trailing comma) were ratified with the collections umbrella
> [ADR-0032](accepted/0032-collections-representation-and-literals.md). Design in
> [`docs/spec/current/syntax/list-literals.md`](../spec/current/syntax/list-literals.md).
> (0028 is claimed by a concurrent unit's `Method`-reflection floor amendment.)

> ADR-0032 (collections) was ratified by the user on 2026-07-12: native heap-arm
> representation for `Map`/`Set`/`Tuple`/`Range`, the binding collection protocol,
> and the literal surface — `[…]` / `{k:v}` / `(a,b)` ship; `#{…}` (set) and
> `..`/`...` (range) are reserved-inactive with committed meaning. It flips ADR-0029
> and the four `core/` collection specs to Accepted.

> ADR-0030 (Fibers & Futures) was ratified by the user on 2026-07-12, resolving the
> concurrency execution-model open decision (open-question 15 → **Option A**, the
> restricted re-entrant loop). It promotes the
> [`concurrency-adr.md`](../design/experimental/v0.2/concurrency-adr.md) draft and the
> [`forward-compat.md §7`](../spec/current/core/forward-compat.md) code-grounded audit.

> ADR-0031 (error surface syntax) was ratified by the user on 2026-07-12: the
> `throw`/`try`/`catch`/`on`/`ensure` spelling, 1:1 sugar over the ADR-0008 block
> protocol (`ensure` mirrors `.ensure{}`; `on T e` mirrors `.on(T){}`). It settles
> the surface ADR-0008 left illustrative; the error *model* is unchanged.

> ADR-0035 (iteration protocol) was ratified by the user on 2026-07-12, promoting
> `experimental/iteration-protocol.md` to normative
> [`iteration.md`](../spec/current/iteration.md): a Wren-style `iterate`/`iteratorValue`
> cursor, the `for` desugar, `break`/`continue`, and the inliner/`Fiber` interactions.
> (`Result`/`Ok`/`Err` was promoted to normative [`result.md`](../spec/current/result.md)
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

> ADR-0059 records a coupling that exists in the code but on no document: ADR-0058's
> single `VM`-owned `Reactive.current` is sound only because `native_reentry_depth != 0`
> inside `Reactive.trackedBy` blocks *every* fiber switch — including resume, which
> `primitive/fiber.rs:96` guards deliberately wider than ADR-0030 §4's letter requires.
> It pins `trackedBy` to the re-entrant `block_call` path, adds `Reactive.current` to
> ADR-0033 §Decision 4's sequencing constraint, and reframes "a `Computed` cannot
> await" as a designed guarantee (no signals implementation has async computeds).
> Proposed — not yet ratified by the user.

> ADRs 0024–0027 were ratified by the user on 2026-07-12, resolving open-questions
> Q2 (numeric split + division), Q3 (parameter names), Q4 (hierarchy mutability), and
> Q8 (modules). See [`docs/spec/current/open-questions.md`](../spec/current/open-questions.md).

> ADRs 0019–0020 were ratified by the user on 2026-07-11, clearing the U-LIST-plan
> §0 gate: they derive from the experimental draft
> `docs/spec/current/experimental/bootstrapping-and-self-hosting.md` (D1, D2/DEC-A). 0019
> consolidates an existing boundary; 0020 resolves DEC-A.

> ADR-0052 amends two unratified annotation/decorator drafts found during spec
> review: `experimental/annotations-contract-semantics.md`'s `@invariant`
> re-entrancy guard (fiber-global counter → receiver-scoped, unwind-safe
> identity set) and `drafts/decorators-stdlib.md`'s `@computed` (Install-tier
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
