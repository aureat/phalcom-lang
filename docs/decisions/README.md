# Telos Decision Records

This is Phalcom's single decision registry. A Telos Decision Record (TDR) records why a language, runtime, tooling, or architecture rule exists. It is historical decision evidence, not normative specification; `docs/specs/` remains the specification authority.

## Registry rules

- Accepted records receive the only canonical numeric sequence: `TDR-0001`, `TDR-0002`, and so on.
- Accepted records are ordered by ratification date. Same-day records use their legacy sequence as the stable tie-breaker.
- Proposed and retired records are intentionally unnumbered. Their legacy ADR/PDR identifier is retained as provenance and for migration review.
- Status, supersession, amendment, and related-decision edges remain in each record's metadata and body. The legacy source identifier is listed below for lookup.
- `docs/design/` is outside this registry. Proposals and research there cannot override the specification or an accepted TDR.

The consolidation contains 81 accepted TDRs, 9 proposed records, and 9 retired records. Accepted numbering excludes every proposed and retired record, including legacy files that were stored in the wrong status directory.

A `*` date was inferred because the source lacked an explicit ratification date. A `†` date uses the explicit ratification date because it conflicts with the source's older `Date` field.

## Accepted registry

| TDR | Ratified | Title | Legacy record | Status |
| --- | --- | --- | --- | --- |
| [TDR-0001](./accepted/0001-record-architecture-decisions.md) | 2026-07-11 | Record architecture decisions | ADR-0001 | Accepted |
| [TDR-0002](./accepted/0002-metaclass-tower-parallel-rule.md) | 2026-07-11 | Metaclass tower follows the parallel rule | ADR-0002 | Accepted |
| [TDR-0003](./accepted/0003-introduce-behavior-kernel-class.md) | 2026-07-11 | Introduce `Behavior` as a shared kernel class | ADR-0003 | Accepted |
| [TDR-0004](./accepted/0004-boolean-as-abstract-bool-with-true-false.md) | 2026-07-11 | Represent booleans as abstract `Bool` + `True`/`False` | ADR-0004 | Accepted |
| [TDR-0005](./accepted/0005-function-as-abstract-callable-root.md) | 2026-07-11 | `Function` as the abstract root of the callable tower | ADR-0006 | Accepted |
| [TDR-0006](./accepted/0006-option-as-abstract-with-some-none.md) | 2026-07-11 | Represent absence as abstract `Option` + `Some`/`None` | ADR-0007 | Accepted |
| [TDR-0007](./accepted/0007-layered-exceptions-and-result.md) | 2026-07-11 | Layered exceptions + `Result`, with terminating semantics | ADR-0008 | Accepted |
| [TDR-0008](./accepted/0008-handle-arena-heap.md) | 2026-07-11 | Object graph lives in a handle/arena heap | ADR-0009 | Accepted |
| [TDR-0009](./accepted/0009-tagged-value-enum.md) | 2026-07-11 | `Value` is a 16-byte explicit tagged representation with a private `Nil` sentinel | ADR-0010 | Accepted |
| [TDR-0010](./accepted/0010-static-instance-slot-layout.md) | 2026-07-11 | Instances use a static per-class slot layout | ADR-0011 | Accepted |
| [TDR-0011](./accepted/0011-selector-signature-encoding-and-dispatch.md) | 2026-07-11 | Label-encoded selectors and inline-cache-ready dispatch | ADR-0012 | Accepted |
| [TDR-0012](./accepted/0012-closure-upvalues-and-frame-token-return.md) | 2026-07-11 | Open/closed upvalues and frame-token non-local return | ADR-0013 | Accepted |
| [TDR-0013](./accepted/0013-object-default-tostring.md) | 2026-07-11 | `Object` default `toString` is `"<ClassName>"` | ADR-0015 | Accepted |
| [TDR-0014](./accepted/0014-hand-written-lexer-and-recursive-descent-parser.md) | 2026-07-11 | Hand-written lexer and recursive-descent parser (replacing LALRPOP) | ADR-0016 | Accepted |
| [TDR-0015](./accepted/0015-class-side-stored-static-fields.md) | 2026-07-11 | Class-side stored static fields live on the metaclass instance | ADR-0017 | Accepted |
| [TDR-0016](./accepted/0016-sacred-selector-inliner-and-override-guard.md) | 2026-07-11 | Sacred-selector inliner with override-epoch deopt guard | ADR-0018 | Accepted |
| [TDR-0017](./accepted/0017-freeze-vm-blessed-primitive-floor.md) | 2026-07-11 | Freeze the VM-blessed primitive floor | ADR-0019 | Accepted |
| [TDR-0018](./accepted/0018-kernel-list-native-array-protocol.md) | 2026-07-11 | Kernel `List` is a native-array-backed Phalcom protocol on the critical path | ADR-0020 | Accepted |
| [TDR-0019](./accepted/0019-no-truthiness-enforcement.md) | 2026-07-11 | No-truthiness enforcement: typed branch floor + literal-only compile check | ADR-0021 | Accepted |
| [TDR-0020](./accepted/0020-string-interpolation-backslash-paren-sigil.md) | 2026-07-12 | String interpolation uses the `\(expr)` sigil | ADR-0022 | Accepted |
| [TDR-0021](./accepted/0021-amend-floor-admit-hash-and-kernel-reflection.md) | 2026-07-12 | Amend the frozen floor — admit `hash`, kernel reflection, `Number#toString`, and `Error#message`/`raise` | ADR-0023 | Accepted |
| [TDR-0022](./accepted/0022-numeric-surface-split-int-float-and-division.md) | 2026-07-12 | Split `Number` into exact `Int` (auto-promoting bignum) and `Float`; `/` is true division, `~/` is integer division | ADR-0024 | Accepted |
| [TDR-0023](./accepted/0023-external-internal-parameter-names.md) | 2026-07-12 | Separate external labels from internal parameter names | ADR-0025 | Accepted |
| [TDR-0024](./accepted/0024-amend-floor-admit-method-reflection.md) | 2026-07-12† | Amend the frozen floor — admit the `Method` reflection surface | ADR-0028 | Accepted |
| [TDR-0025](./accepted/0025-list-literal-syntax.md) | 2026-07-12 | List literals `[a, b, c]` desugar to `List` construction sends | ADR-0029 | Accepted |
| [TDR-0026](./accepted/0026-fibers-and-futures-cooperative-concurrency.md) | 2026-07-12 | Fibers and Futures: cooperative concurrency on a restricted re-entrant loop | ADR-0030 | Accepted |
| [TDR-0027](./accepted/0027-error-handling-surface-syntax.md) | 2026-07-12 | Error-handling surface syntax: `throw` / `try` / `catch` / `on` / `ensure` | ADR-0031 | Accepted |
| [TDR-0028](./accepted/0028-collections-representation-and-literals.md) | 2026-07-12 | Collections: native representation, shared protocol, and literal surface | ADR-0032 | Accepted |
| [TDR-0029](./accepted/0029-iteration-protocol-cursor.md) | 2026-07-12 | Iteration protocol: a Wren-style two-selector cursor | ADR-0035 | Accepted |
| [TDR-0030](./accepted/0030-amend-floor-admit-number-tostring.md) | 2026-07-12† | Amend the frozen floor — admit `Number#toString` | ADR-0036 | Accepted |
| [TDR-0031](./accepted/0031-amend-floor-admit-error-root.md) | 2026-07-12† | Amend the frozen floor — admit `Error#message`/`Error#raise` | ADR-0037 | Accepted |
| [TDR-0032](./accepted/0032-amend-floor-admit-block-on-ensure.md) | 2026-07-12 | Amend the frozen floor — admit `Block#on`/`Block#ensure` (error handling) | ADR-0038 | Accepted |
| [TDR-0033](./accepted/0033-amend-floor-admit-collection-container-primitives.md) | 2026-07-12 | Amend the frozen floor — admit collection-container primitives (`Map`/`Set`/`Tuple`/`Range`) | ADR-0039 | Accepted |
| [TDR-0034](./accepted/0034-supersend-opcode.md) | 2026-07-12† | Add the `SuperSend` dispatch opcode for `super.sel(…)` | ADR-0040 | Accepted |
| [TDR-0035](./accepted/0035-hierarchy-stability-policy.md) | 2026-07-12 | Hierarchy-stability policy: sealed reparenting + single inheritance | ADR-0041 | Accepted |
| [TDR-0036](./accepted/0036-no-default-arguments-keep-selector-identity-pristine.md) | 2026-07-12 | No default arguments; keep selector identity pristine | ADR-0043 | Accepted |
| [TDR-0037](./accepted/0037-option-bootstrap-formalization-and-defer-niche-encoding.md) | 2026-07-12 | `Option` bootstrap formalization; defer niche-encoding | ADR-0044 | Accepted |
| [TDR-0038](./accepted/0038-module-import-relative-path-whole-module-binding.md) | 2026-07-12 | `import` resolves by relative file path and binds a whole `Module`; amend the frozen floor +1 (`Module#doesNotUnderstand`) | ADR-0045 | Accepted |
| [TDR-0039](./accepted/0039-destructuring-bindings.md) | 2026-07-13 | Destructuring `let`/`var` bindings — irrefutable tuple + list, `at(_)` protocol | ADR-0046 | Accepted |
| [TDR-0040](./accepted/0040-amend-floor-admit-family-call-router.md) | 2026-07-13 | `::` method references (Open form, callable-only); amend the frozen floor +1 (`Family#doesNotUnderstand`) | ADR-0047 | Accepted |
| [TDR-0041](./accepted/0041-amend-iteration-bare-cursor-sentinel-and-iterable-root.md) | 2026-07-13 | Amend iteration: bare-cursor end-sentinel + kernel `Iterable` root | ADR-0048 | Accepted |
| [TDR-0042](./accepted/0042-invariant-reentrancy-scope-and-layout-confined-decorator-state.md) | 2026-07-13 | Invariant re-entrancy is receiver-scoped; per-receiver decorator state is Layout-confined | ADR-0052 | Accepted |
| [TDR-0043](./accepted/0043-runtime-decorator-interception-reuses-override-epoch-guard.md) | 2026-07-13 | Runtime-tier decorator interception reuses the sacred-selector override-epoch guard | ADR-0053 | Accepted |
| [TDR-0044](./accepted/0044-two-speed-ratification-annotation-decorator-tiers.md) | 2026-07-13 | Two-speed ratification: Compile/Layout tier now, Install/Dispatch/Runtime gated on ADR-0053 | ADR-0054 | Accepted |
| [TDR-0045](./accepted/0045-decorator-granularity-vs-proxy-granularity-split.md) | 2026-07-13 | Decorator granularity vs proxy granularity — the method-declaration / whole-object interception split | ADR-0057 | Accepted |
| [TDR-0046](./accepted/0046-reactive-tracking-context-needs-a-native-module.md) | 2026-07-13 | Reactive tracking-context and effect scheduler need a native module, not class-side `.ph` state | ADR-0058 | Accepted |
| [TDR-0047](./accepted/0047-non-moving-mark-sweep-collector.md) | 2026-07-14† | Reclamation is a non-moving precise mark-sweep collector | ADR-0050 | Accepted |
| [TDR-0048](./accepted/0048-performance-strategy-measure-first-tiered-optimization.md) | 2026-07-14† | Performance strategy: measure-first, tiered, behavior-invariant | ADR-0051 | Accepted |
| [TDR-0049](./accepted/0049-index-operator-as-real-selector.md) | 2026-07-14 | `[]` Is a Real, Overridable Selector — No `at` Lowering | ADR-0060 | Accepted |
| [TDR-0050](./accepted/0050-amend-floor-admit-string-byte-and-raw-write-primitives.md) | 2026-07-15† | Amend the floor: admit String byte/slice accessors + raw stdout write | ADR-0049 | Accepted |
| [TDR-0051](./accepted/0051-constructors-are-ordinary-class-side-methods.md) | 2026-07-15 | Constructors are ordinary class-side methods: `@constructor`/`@class` decorators, `new_` allocator | ADR-0063 | Accepted |
| [TDR-0052](./accepted/0052-let-const-bindings-and-field-mutability.md) | 2026-07-15 | `let`/`const` bindings; unkeyworded mutable fields | ADR-0064 | Accepted |
| [TDR-0053](./accepted/0053-classes-are-closed.md) | 2026-07-19 | Classes are closed: remove class reopening | PDR-0001 | Accepted |
| [TDR-0054](./accepted/0054-class-declarations-join-the-binding-namespace.md) | 2026-07-19 | Class declarations join the binding namespace; the duplicate diagnostic carries both spans | PDR-0002 | Accepted |
| [TDR-0055](./accepted/0055-no-user-visible-threads-fibers-and-isolates.md) | 2026-07-20 | No user-visible shared-memory threads: fibers now, isolates if ever | PDR-0003 | Accepted |
| [TDR-0056](./accepted/0056-io-is-future-shaped-reactor-owned.md) | 2026-07-20 | IO is `Future`-shaped and reactor-owned; the reactor is built before the IO surface | PDR-0004 | Accepted |
| [TDR-0057](./accepted/0057-resources-are-disposable-handles-not-finalized.md) | 2026-07-20 | Native resources are closeable handles with a generation-tagged table; no finalizers | PDR-0005 | Accepted |
| [TDR-0058](./accepted/0058-repl-completeness-is-a-parser-signal.md) | 2026-07-20 | REPL completeness is a parser signal; the lexer reports unterminated modes | PDR-0006 | Accepted |
| [TDR-0059](./accepted/0059-bounded-call-depth-and-native-reentrancy.md) | 2026-07-20 | Bounded call depth and native re-entrancy: two counters, one error | PDR-0007 | Accepted |
| [TDR-0060](./accepted/0060-cell-boundary-diagnostics-and-state-hygiene.md) | 2026-07-20 | Every failed cell reports, and reporting happens before unwinding | PDR-0008 | Accepted |
| [TDR-0061](./accepted/0061-defer-lsp-backed-repl-surface.md) | 2026-07-20 | The LSP-backed REPL surface waits for ADR-0056 to be ratified | PDR-0009 | Accepted |
| [TDR-0062](./accepted/0062-errors-carry-structure-and-cheap-origin.md) | 2026-07-20 | Errors carry structure and cheap origin: one cause chain, a `kind` symbol, incremental capture | PDR-0010 | Accepted |
| [TDR-0063](./accepted/0063-admit-bytes-native-octet-buffer.md) | 2026-07-20 | Admit `Bytes`: a native octet buffer arm, ten floor primitives, and the container bulk-op posture | PDR-0011 | Accepted |
| [TDR-0064](./accepted/0064-numeric-tower-implementation-and-floor-amendment.md) | 2026-07-20 | The numeric tower lands: `Int`/`Float` implementation rulings and the floor amendment (137 → 153) | PDR-0012 | Accepted |
| [TDR-0065](./accepted/0065-path-is-bytes-backed-filesystem-surface.md) | 2026-07-20 | `Path` is bytes-backed, not a `String`; the filesystem surface | PDR-0013 | Accepted |
| [TDR-0066](./accepted/0066-diagnostics-renderer-is-in-house.md) | 2026-07-20 | The diagnostics renderer is in-house; miette leaves the workspace | PDR-0014 | Accepted |
| [TDR-0067](./accepted/0067-decorator-carry-forward-and-v03-runtime-mandate.md) | 2026-07-20 | Decorator governance: ADR carry-forward verified; Install/Dispatch/Runtime tiers, `@effect`, and the framework families mandated as the v0.3 experimental track | PDR-0018 | Accepted |
| [TDR-0068](./accepted/0068-bitwise-operations-on-int.md) | 2026-07-21 | Bitwise operations on `Int`: infinite two's complement, operator selectors, no wrapping | PDR-0020 | Accepted |
| [TDR-0069](./accepted/0069-numeric-tower-residue-rulings.md) | 2026-07-21 | Numeric tower residue: `~/` is total over the tower and returns `Int`; construction never narrows; the `Bool` arm dies | PDR-0025 | Accepted |
| [TDR-0070](./accepted/0070-numeric-literals.md) | 2026-07-21 | Numeric literal grammar: radix prefixes, separators, decimal exponents, no `n` suffix | PDR-0026 | Accepted |
| [TDR-0071](./accepted/0071-float-protocol-and-explicit-narrowing.md) | 2026-07-21 | Numeric semantics completion | PDR-0027 | Accepted |
| [TDR-0072](./accepted/0072-class-and-constructor-decorator-canon.md) | 2026-07-21 | `@class` placement and `@constructor` method canon | PDR-0028 | Accepted |
| [TDR-0073](./accepted/0073-string-literals-and-interpolation-completion.md) | 2026-07-22 | Complete string interpolation; defer multiline literals | PDR-0029 | Accepted |
| [TDR-0074](./accepted/0074-replace-extends-keyword-with-is.md) | 2026-07-22 | Replace `extends` keyword with `is` for Class Inheritance | PDR-0030 | Accepted |
| [TDR-0075](./accepted/0075-range-slicing-floor-amendment.md) | 2026-08-08 | Range slicing uses normalized bounds with two native collection seams | PDR-0031 | Accepted |
| [TDR-0076](./accepted/0076-transition-1-language-surface-convergence.md) | 2026-08-08 | Converge lexical namespaces, selectors, visibility, and class placement | PDR-0032 | Accepted |
| [TDR-0077](./accepted/0077-immediate-bounded-option.md) | 2026-08-11 | Make Option an immediate bounded sum value | PDR-0033 | Accepted |
| [TDR-0078](./accepted/0078-multiline-string-text-blocks.md) | 2026-08-15 | Add indentation-safe multiline string text blocks | PDR-0034 | Accepted |
| [TDR-0079](./accepted/0079-poller-backend-is-mio.md) | 2026-09-12† | The poller backend is `mio`, wrapped once at the reactor seam; syscalls try first and register second | PDR-0016 | Accepted |
| [TDR-0080](./accepted/0080-data-is-a-nominal-transparent-immutable-value-product.md) | 2026-09-12 | `data` is a nominal transparent immutable value product | PDR-0035 | Accepted |
| [TDR-0081](./accepted/0081-tuple-and-record-are-transparent-structural-value-products.md) | 2026-09-13 | Tuple and Record Are Transparent Structural Value Products | PDR-0036 | Accepted |

## Proposed registry

Proposed records do not reserve or consume TDR numbers. They become numbered only if ratified and moved into `accepted/`.

| Record | Proposed | Title | Legacy record | Status |
| --- | --- | --- | --- | --- |
| [TDR — Phalcom language intelligence is an in-process `phalcom-lsp` server](./proposed/phalcom-lsp-architecture.md) | 2026-07-13 | Phalcom language intelligence is an in-process `phalcom-lsp` server | ADR-0056 | Proposed |
| [TDR — Amend ADR-0058 + ADR-0033: the reactive tracking context is bound to the native-frame switch guard](./proposed/amend-reactive-tracking-context-native-frame-coupling.md) | 2026-07-14 | Amend ADR-0058 + ADR-0033: the reactive tracking context is bound to the native-frame switch guard | ADR-0059 | Proposed |
| [TDR — The network surface: TCP is poller-backed and `Future`-shaped, DNS rides the pool, endpoints are address-plus-port](./proposed/network-surface-tcp-dns-endpoints.md) | 2026-07-20 | The network surface: TCP is poller-backed and `Future`-shaped, DNS rides the pool, endpoints are address-plus-port | PDR-0015 | Proposed |
| [TDR — `Future#cancel` is renunciation: settle `#cancelled` now, suppress unstarted work best-effort, interrupt nothing](./proposed/future-cancel-is-renunciation.md) | 2026-07-20 | `Future#cancel` is renunciation: settle `#cancelled` now, suppress unstarted work best-effort, interrupt nothing | PDR-0017 | Proposed |
| [TDR — Process and environment: argument-vector spawning, a read-only environment, and `System` stays the one effect namespace](./proposed/process-and-environment-surface.md) | 2026-07-20 | Process and environment: argument-vector spawning, a read-only environment, and `System` stays the one effect namespace | PDR-0019 | Proposed |
| [TDR — Decorator naming: lowercase names are compiler builtins, Capitalized names are `Attribute` classes](./proposed/decorator-naming-convention.md) | 2026-07-20 | Decorator naming: lowercase names are compiler builtins, Capitalized names are `Attribute` classes | PDR-0021 | Proposed |
| [TDR — Attribute-class resolution is suffix-first: `@Name` resolves `NameAttribute`, then `Name`](./proposed/attribute-suffix-resolution.md) | 2026-07-20 | Attribute-class resolution is suffix-first: `@Name` resolves `NameAttribute`, then `Name` | PDR-0022 | Proposed |
| [TDR — Contract inheritance: lexical replacement now (documented, Liskov-unsound), runtime combination later behind the metadata gate](./proposed/contract-inheritance-replacement-now-combination-later.md) | 2026-07-20 | Contract inheritance: lexical replacement now (documented, Liskov-unsound), runtime combination later behind the metadata gate | PDR-0023 | Proposed |
| [TDR — The metaobject gate: `Method.fromBlock`, `Method#invokeOn`, `Behavior#defineMethod` (floor amendment; amends ADR-0019)](./proposed/metaobject-gate-floor-amendment.md) | 2026-07-20 | The metaobject gate: `Method.fromBlock`, `Method#invokeOn`, `Behavior#defineMethod` (floor amendment; amends ADR-0019) | PDR-0024 | Proposed |

## Retired registry

Retired records do not reserve or consume TDR numbers. They remain for historical context and supersession tracing.

| Record | Retired | Title | Legacy record | Status |
| --- | --- | --- | --- | --- |
| [TDR — Keep a single flat `Number` type backed by `f64`](./retired/number-as-flat-f64.md) | 2026-07-11 | Keep a single flat `Number` type backed by `f64` | ADR-0005 | Retired |
| [TDR — Variable bindings are `let` (immutable) and `var` (mutable)](./retired/let-and-var-bindings.md) | 2026-07-11 | Variable bindings are `let` (immutable) and `var` (mutable) | ADR-0014 | Retired |
| [TDR — Methods are open; superclass reparenting is sealed](./retired/class-hierarchy-mutability.md) | 2026-07-12 | Methods are open; superclass reparenting is sealed | ADR-0026 | Retired |
| [TDR — A module is a file; exports are public by default; imports are qualified, selective, or aliased](./retired/modules-as-files-with-public-by-default-imports.md) | 2026-07-12 | A module is a file; exports are public by default; imports are qualified, selective, or aliased | ADR-0027 | Retired |
| [TDR — Amend the fiber execution model — trampoline the bytecode block call-site](./retired/amend-fiber-execution-trampolined-block-callsite.md) | 2026-07-12 | Amend the fiber execution model — trampoline the bytecode block call-site | ADR-0033 | Retired |
| [TDR — Flat `Number` now; defer the `Integer` / `Float` split](./retired/flat-number-defer-integer-float-split.md) | 2026-07-14 | Flat `Number` now; defer the `Integer` / `Float` split | ADR-0042 | Retired |
| [TDR — Subscript Indexing Syntax Sugar over `at` Selectors](./retired/index-syntax-sugar-over-at-selectors.md) | 2026-07-14 | Subscript Indexing Syntax Sugar over `at` Selectors | ADR-0055 | Retired |
| [TDR — Underscore prefixes are reserved: `_` fields, `_$` language internals, `__` reserved](./retired/underscore-prefix-reservation-fields-internals-reserved.md) | 2026-07-14 | Underscore prefixes are reserved: `_` fields, `_$` language internals, `__` reserved | ADR-0061 | Retired |
| [TDR — Amend floor — admit `String` raw byte accessors + `System.rawWrite(_)`](./retired/amend-floor-admit-string-raw-byte-accessors-supersedes-0049-naming.md) | 2026-07-15 | Amend floor — admit `String` raw byte accessors + `System.rawWrite(_)` | ADR-0062 | Retired |

## Preliminary revision flags

These are triage flags from consolidation, not fresh semantic certification. Unflagged records were not independently re-ratified or code-audited in this pass.

- [TDR — Variable bindings are `let` (immutable) and `var` (mutable)](./retired/let-and-var-bindings.md) (`ADR-0014`): Source status says superseded while the legacy index listed it as accepted; verify the spelling-only supersession.
- [TDR — Methods are open; superclass reparenting is sealed](./retired/class-hierarchy-mutability.md) (`ADR-0026`): Stored under the legacy accepted directory but status says retired; it is now in retired/ and excluded from numbering.
- [TDR — A module is a file; exports are public by default; imports are qualified, selective, or aliased](./retired/modules-as-files-with-public-by-default-imports.md) (`ADR-0027`): Legacy file was stored in retired/ while its header said Accepted; it is retained as retired and excluded from numbering pending reconciliation.
- `TDR-0047` (`ADR-0050`): Legacy tracker reports only partial implementation verification, and its explicit ratification date differs from Date.
- `TDR-0048` (`ADR-0051`): Policy was ratified after earlier implementation cuts; its explicit ratification date differs from Date.
- `TDR-0042` (`ADR-0052`): Legacy tracker marked implementation state unverified; revalidate before treating the decision as current.
- `TDR-0043` (`ADR-0053`): Legacy tracker marked implementation state unverified; revalidate the runtime interception claim.
- `TDR-0044` (`ADR-0054`): Legacy tracker marked implementation state unverified; reconcile with the later decorator records.
- [TDR — Phalcom language intelligence is an in-process `phalcom-lsp` server](./proposed/phalcom-lsp-architecture.md) (`ADR-0056`): Proposed source, but the legacy tracker reports the language-server crate shipped; requires a ratification/status ruling.
- `TDR-0045` (`ADR-0057`): Legacy tracker marked implementation state unverified; revalidate against the decorator implementation.
- `TDR-0046` (`ADR-0058`): Legacy tracker marked implementation state unverified; revalidate against the reactive implementation.
- `TDR-0049` (`ADR-0060`): Source has no explicit Date field; chronology uses the 2026-07-14 supersession date recorded by retired ADR-0055.
- [TDR — Underscore prefixes are reserved: `_` fields, `_$` language internals, `__` reserved](./retired/underscore-prefix-reservation-fields-internals-reserved.md) (`ADR-0061`): Stored under the legacy proposed directory but status says retired by PDR-0032; it is now in retired/ and excluded from numbering.
- `TDR-0051` (`ADR-0063`): Status remains Accepted while the body says its surface is superseded by PDR-0028; reconcile the historical/current boundary.
- `TDR-0079` (`PDR-0016`): Header ratification date (2026-09-12) conflicts with Date (2026-07-20); chronology uses the ratification date.
- `TDR-0068` (`PDR-0020`): No explicit Date field; chronology uses the ratification date embedded in Status.
- `TDR-0069` (`PDR-0025`): No explicit Date field; chronology uses the ratification date embedded in Status.
- `TDR-0070` (`PDR-0026`): No explicit Date field; chronology uses the ratification date embedded in Status.
- `TDR-0071` (`PDR-0027`): No explicit Date field; chronology uses the ratification date embedded in Status.
- `TDR-0080` (`PDR-0035`): Newly ratified record; implementation and specification alignment were not revalidated during consolidation.
- `TDR-0081` (`PDR-0036`): Newly ratified record; implementation and specification alignment were not revalidated during consolidation.

## Migration notes

The former ADR and PDR registries and their separate status trackers were consumed by this README. Existing citations to a legacy ADR/PDR identifier remain meaningful as provenance; new decision citations should use the canonical TDR entry or the unnumbered proposed/retired slug. Design-proposal materials were intentionally not migrated.
