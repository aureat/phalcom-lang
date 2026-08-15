# Phalcom Families and Selector Patterns — LSP & Semantic Analysis Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Extend `phalcom-lsp` so exact/open Families, exact Selectors, SelectorPatterns, Methods, MethodFamilies, BoundMethods, and BoundMethodFamilies are represented with the same semantics as the runtime; provide precise definition/reference/rename/completion/signature/hover behavior; preserve current immutable-snapshot and incremental-worker architecture; and make family-aware analysis cheap enough to remain off the editor latency critical path.

**Architecture:** `phalcom-lsp` remains VM-free and continues to depend only on `phalcom-ast`, `phalcom-common`, and `phalcom-native-surface`. Exact selector structure and SelectorPattern matching come from the shared `phalcom-common::selector` layer introduced by the core plan. Shallow source ingestion stores structural selector/rest metadata once. Deep semantic analysis has two routing backends: dynamic `Family` values resolve exact call selectors through the existing `DispatchResolver`; captured `MethodFamily`/`BoundMethodFamily` values carry an immutable effective selector-to-`CallableId` snapshot plus ordered rest candidates and never re-resolve on the bound receiver. Family-aware editor requests read current source products plus coherent immutable semantic generations; they do not scan the workspace or run deep convergence synchronously.

**Tech Stack:** Rust 2024; `tower-lsp`; `tokio`; immutable `Arc` semantic generations; `DashMap` document/source layer; `phalcom-ast`; `phalcom-common`; `phalcom-native-surface`; existing `ValueShape`/flow/interprocedural fact engine; current callable-granular invalidation and one-writer worker pipeline.

**Pinned repository checkpoint:** `b5477b74dfa6f79a4b4487896a1d63699d98685e`. This plan assumes the async/performance architecture present at that revision and the recently landed contribution-local/callable-granular semantic work. Code links are pinned to that commit.

**Dependency on companion plan:** Begin Task 1 only after the companion core plan has landed its shared selector model and AST/parser seam (Core Tasks 1–3). Runtime MethodFamily implementation may proceed in parallel after that point; the LSP must not depend on `phalcom-core`.

---

## 1. Current checkpoint and constraints

### 1.1 Repository observations

- `phalcom-lsp/Cargo.toml` deliberately excludes `phalcom-core` and documents the VM-free startup boundary. Preserve it.
- `phalcom-lsp/src/selectors.rs` currently duplicates the VM's comma-form selector construction and label escaping because the common crate did not yet own selector semantics. It contains `comma_form`, `comma_form_from_labels`, `call_selector`, getter/setter/index helpers, and a private `encode_label_component`. This duplication should collapse onto `phalcom-common::selector`.
- `ValueShape` currently has `Callable(CallableId)` and only one Family shape: `Family { receiver: Box<ValueShape>, base: String }`. See [`phalcom-lsp/src/semantic/facts.rs`](https://github.com/aureat/phalcom-lang/blob/b5477b74dfa6f79a4b4487896a1d63699d98685e/phalcom-lsp/src/semantic/facts.rs#L1-L120).
- The analyzer currently treats `MethodRefKind::Open` as `ValueShape::Family`, but treats `MethodRefKind::Pinned` as immediately resolved `ValueShape::Callable`, which conflicts with the ratified semantics: exact `obj::foo(_)` remains a live dynamic Family. See [`phalcom-lsp/src/semantic/analyzer.rs`](https://github.com/aureat/phalcom-lang/blob/b5477b74dfa6f79a4b4487896a1d63699d98685e/phalcom-lsp/src/semantic/analyzer.rs#L1-L380), especially `analyze_unqualified_call` and `analyze_method_ref`.
- `DispatchResolver` already centralizes VM-free side-aware inherited lookup and resolves to the declaration owner's `CallableId`. See [`phalcom-lsp/src/semantic/dispatch.rs`](https://github.com/aureat/phalcom-lang/blob/b5477b74dfa6f79a4b4487896a1d63699d98685e/phalcom-lsp/src/semantic/dispatch.rs#L1-L210). Extend this instead of creating a second resolver.
- `ClassSurface.members` is currently `BTreeMap<String, MemberSides>`, keyed by canonical selector String; `MemberSurface` stores `CallableId`, `MemberKind`, visibility, side, source ranges, and parameter surface, but not a structural Selector or rest layout. See [`phalcom-lsp/src/semantic/surface.rs`](https://github.com/aureat/phalcom-lang/blob/b5477b74dfa6f79a4b4487896a1d63699d98685e/phalcom-lsp/src/semantic/surface.rs#L1-L300).
- `ParamSurface` currently stores name/label/ranges but drops `ParameterDef.rest_mode`. MethodFamily semantic capture needs finite rest-candidate acceptance metadata, so this must be retained structurally.
- The current source/inference architecture already has module/class surfaces, callable summaries, scoped local facts, parameter contributions, immutable generations, and callable-level invalidation. Family support must plug into those products rather than add request-local mutable caches.
- Recent performance work at the pinned head has made semantic solving contribution-local, invalidation callable-seeded, published generations structurally shared, and editor refreshes selective. Family indexing must preserve those properties.

### 1.2 Semantic invariants for tooling

1. Exact `obj::foo(_)` is `Family(receiver, Exact(#foo(_)))`, not `Callable` and not captured Method.
2. Open `obj::foo(...)` is `Family(receiver, Pattern(#foo(...)))`.
3. Calling a Family dynamically resolves against the receiver shape in the current semantic generation.
4. `C >> #foo(_)` is Method extraction: it captures/returns one effective callable identity at the extraction point.
5. `C >> #foo(...)` is MethodFamily extraction: it captures a finite immutable effective exact-map + ordered rest candidates in the semantic value fact.
6. `MethodFamily#bind(receiver)` does not resolve methods on `receiver`; BoundMethodFamily calls use captured callable IDs.
7. A body-only edit to an existing member does not change family membership; its callable summary may change and should propagate through ordinary dependency edges.
8. Declaration signature/member addition/removal/visibility/dispatch-side/superclass changes can change family membership and must invalidate family-capture facts and family-aware queries.
9. Editor requests never wait for deep convergence and never scan all classes/methods to answer a family query.
10. The LSP's selector semantics must be exactly the shared common semantics, not another hand-written parser/encoder.

---

## 2. Target semantic model

### 2.1 Extend source member surfaces

Each `MemberSurface` should carry a structural exact Selector in addition to the compatibility String in `CallableId`:

```rust
pub struct MemberSurface {
    pub callable: CallableId,
    pub selector: phalcom_common::selector::Selector,
    pub rest: Option<RestSurface>,
    // existing fields...
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct RestSurface {
    pub fixed_positionals: usize,
    pub fixed_labels: Box<[String]>,
    pub mode: RestSurfaceMode,
}
```

`RestSurface::accepts(positionals, labels)` must mirror core `RestLayout::accepts`, but remain VM-free. Prefer to put a runtime-independent rest-acceptance descriptor in `phalcom-common` if both core and LSP can share it without importing AST/runtime concepts. Otherwise keep the LSP adapter small and table-tested against AST rest forms.

### 2.2 Captured family shape

Do not embed a mutable map or workspace pointer in `ValueShape`. Use immutable content-addressable-ish shape data:

```rust
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CapturedMethodFamilyShape {
    pub source_behavior: ClassId,
    pub pattern: SelectorPattern,
    pub exact: Box<[(Selector, CallableId)]>,
    pub rest: Box<[CallableId]>,
}
```

Order `exact` deterministically by selector canonical order (or effective capture order if the UI should preserve declaration order; choose one and keep it stable). `rest` remains subclass-to-superclass lookup order.

If clone profiling shows this shape is material in solver cost, wrap it in `Arc<CapturedMethodFamilyShape>` and ensure equality/hash/order compare content, not pointer identity. Do not introduce a global mutable family registry solely to reduce clones.

### 2.3 ValueShape variants

Extend `ValueShape` approximately as follows:

```rust
pub enum ValueShape {
    // existing...
    Callable(CallableId),
    Selector(Selector),
    SelectorPattern(SelectorPattern),
    Family {
        receiver: Box<ValueShape>,
        spec: SelectorSpec,
    },
    Method(CallableId),
    MethodFamily(Arc<CapturedMethodFamilyShape>),
    BoundMethod {
        receiver: Box<ValueShape>,
        method: CallableId,
    },
    BoundMethodFamily {
        receiver: Box<ValueShape>,
        family: Arc<CapturedMethodFamilyShape>,
    },
    Union(Vec<ValueShape>),
}
```

The exact names can vary, but **do not** collapse Method/MethodFamily into generic `Callable`; editor presentation and call routing need the distinction.

### 2.4 Two call-resolution backends

Dynamic:

```text
Family + call shape
  -> exact Selector
  -> pattern membership if open
  -> DispatchResolver(receiver, selector)
  -> callable summary
```

Captured:

```text
BoundMethodFamily + call shape
  -> exact Selector
  -> captured pattern membership
  -> captured exact map
  -> else captured ordered rest candidates + RestSurface.accepts
  -> captured CallableId summary
```

No `DispatchResolver` call occurs on the bound receiver in the second path.

---

## 3. Concurrency, snapshots, and performance constraints

This plan inherits the async LSP implementation plan's architecture as hard constraints:

- document/source truth may be newer than the latest semantic generation;
- one semantic worker owns deep mutable convergence state;
- published semantic generations are immutable and structurally shared;
- request handlers use compatible current-source + semantic snapshots and degrade gracefully when semantic facts are stale;
- stale generations are never published over newer work;
- semantic work is contribution/callable local where possible;
- completion/hover/signature/definition do not trigger workspace-wide recomputation;
- no filesystem reads are introduced into open-document request hot paths.

Family-specific consequence:

> Selector-family enumeration belongs in source/semantic products, not in request handlers.

---

## 4. Parallel execution and ownership

### Wave A — selector adapters and source surfaces

Tasks 1–2 may run immediately after Core Task 3. They touch `selectors.rs` and semantic `surface.rs` and should be merged before analyzer/captured routing changes.

### Wave B — semantic values and routing

Tasks 3–6 touch `facts.rs`, `analyzer.rs`, `dispatch.rs`, `flow.rs`, and engine integration. Keep `analyzer.rs` and `facts.rs` single-owner during this wave.

### Wave C — editor features and indexing

Tasks 7–9 may split by file ownership: occurrence/index/rename in one branch; hover/completion/signature/definition in another; semantic tokens/VS Code syntax in another.

### Wave D — invalidation/performance/broad validation

Tasks 10–12 run after semantic correctness. Do not optimize by removing semantic distinctions.

---

## 5. File ownership map

| Area | Files | Responsibility |
|---|---|---|
| Selector adapters | `phalcom-lsp/src/selectors.rs` | AST/member helper adapters over `phalcom-common::selector` |
| Source surfaces | `semantic/surface.rs`, `semantic/core_source.rs`, native surface adapters | structural Selector/rest metadata |
| Semantic values | `semantic/facts.rs`, `semantic/flow.rs` | Selector/Pattern/Family/MethodFamily shapes and joins |
| Dynamic/captured routing | `semantic/dispatch.rs`, `semantic/analyzer.rs` | exact lookup, effective family capture, call result inference |
| Solver/dependencies | `semantic/engine.rs`, `semantic/infer.rs`, `semantic/invalidation.rs`, `semantic/callable.rs` | family dependencies and incremental recomputation |
| Occurrences/index | `semantic/occurrence.rs`, `index.rs`, `selectors.rs` | exact/pattern references and component ranges |
| Queries | `semantic/query.rs`, `semantic/mod.rs`, backend/query consumers | bounded family-aware semantic reads |
| Editor UX | `completion.rs`, `hover.rs`, `backend.rs`, `analysis_service.rs`, `index.rs`, semantic query APIs | user-visible family intelligence |
| Semantic tokens | `phalcom-lsp/src/semantic_tokens.rs` | selector spec components |
| VS Code syntax | `tools/vsphalcom/syntaxes/phalcom.tmLanguage.json`, extension tests | lexical highlighting of new forms |
| Tests | `phalcom-lsp/tests/**`, semantic module unit tests, VS Code tests | semantic + async/incremental proof |

---

# Task 0 — Baseline the current LSP and semantic worker

- [x] Complete this task and its focused validation: **Baseline the current LSP and semantic worker**

**Files:** no source edits.

- [ ] Verify revision and run current focused/broad tests:

```bash
git rev-parse HEAD
cargo fmt --all -- --check
cargo clippy -p phalcom-lsp --all-targets -- -D warnings
CARGO_TARGET_DIR=target cargo test -p phalcom-lsp
```

- [ ] Run the existing async/performance tests introduced by the attached implementation work; record the names and current timings/counters used by the project.
- [ ] Inventory selector/family-specific references:

```bash
rg -n 'MethodRefKind|SymbolLiteralKind|ValueShape::Family|comma_form|call_selector|SelectorSymbol|NameSymbol' phalcom-lsp
```

- [ ] Record semantic worker counters if available: files reanalyzed, callables recomputed, parameter slots recomputed, generation publication count, stale generation drops.
- [ ] Do not change behavior in this task.

---

# Task 1 — Replace LSP selector encoding duplication with common structural semantics

- [x] Complete this task and its focused validation: **Replace LSP selector encoding duplication with common structural semantics**

**Files:**
- `phalcom-lsp/src/selectors.rs`
- selector helper tests
- `phalcom-lsp/Cargo.toml` only if common feature flags are needed (prefer none)

## Step 1.1 — Preserve helper API temporarily

Keep current public/internal helper names where many callers rely on them, but implement them through common Selector construction:

```rust
pub fn method_selector(m: &MethodDef) -> String {
    selector_from_method(m).encode()
}
```

Add structural helpers:

```rust
pub fn selector_from_member(member: &ClassMember) -> Selector;
pub fn selector_from_call(name: &str, args: &[PackItem]) -> Option<Selector>;
pub fn selector_spec_from_ast(spec: &SelectorSpecSyntax) -> Result<SelectorSpec, SelectorError>;
```

`selector_from_call` returns `None` when a computed/dynamic pack prevents exact static selector reconstruction; callers must then conservatively degrade.

## Step 1.2 — Delete duplicate label escaping

Remove the LSP-private `encode_label_component` after tests prove common output is identical.

## Step 1.3 — Tests

Port existing `selectors.rs` tests to assert both structural value and canonical text:

```rust
let selector = selector_from_member(member);
assert_eq!(selector.kind, SelectorKind::Method);
assert_eq!(selector.encode(), "move(_,to,duration)");
```

Run:

```bash
cargo test -p phalcom-lsp selectors
```

Commit:

```bash
git commit -am "refactor(lsp): share structural selector semantics"
```

---

# Task 2 — Store structural Selector and rest acceptance in ClassSurface

- [x] Complete this task and its focused validation: **Store structural Selector and rest acceptance in ClassSurface**

**Files:**
- `phalcom-lsp/src/semantic/surface.rs`
- `phalcom-lsp/src/semantic/core_source.rs`
- native member surface adapters
- surface tests

Current reference: [`semantic/surface.rs`](https://github.com/aureat/phalcom-lang/blob/b5477b74dfa6f79a4b4487896a1d63699d98685e/phalcom-lsp/src/semantic/surface.rs#L1-L300).

## Step 2.1 — Extend MemberSurface

Add:

```rust
pub selector: Selector,
pub rest: Option<RestSurface>,
```

Keep `CallableId.selector: String` as a stable compatibility/index key for now. Construct it using `selector.encode()` exactly once during shallow ingestion.

## Step 2.2 — Preserve rest information

`ParamSurface` currently discards `ParameterDef.rest_mode`. Either:

- add `rest_mode` to `ParamSurface`; and
- normalize a method-level `RestSurface` during `build_module_surface`;

or store only the normalized method-level descriptor if no editor UI needs individual rest mode later.

The normalized accept rule must mirror runtime:

```text
Positional: positionals >= fixed && labels == fixed_labels
Labeled:    positionals == fixed && labels starts_with fixed_labels
Split/Complete: positionals >= fixed && labels starts_with fixed_labels
```

## Step 2.3 — Structural member family helpers

Add bounded iterators on `ClassSurface`:

```rust
pub fn members_matching<'a>(&'a self, side: DispatchSide, pattern: &'a SelectorPattern)
    -> impl Iterator<Item = &'a MemberSurface>;
```

This scans one class's direct members, not inheritance. Inheritance precedence remains centralized in `DispatchResolver`.

## Step 2.4 — Native/core surfaces

Ensure native members receive the same structural Selector/rest metadata from `phalcom-native-surface`. Do not make source and native classes use different matching semantics.

Tests:

- source method/getter/setter/subscript structural selector;
- rest forms `*`, `**`, split, `***`;
- same selector on instance/class sides;
- native core member representation.

Run:

```bash
cargo test -p phalcom-lsp semantic::surface
```

Commit:

```bash
git commit -am "feat(lsp): retain selector and rest structure in class surfaces"
```

---

# Task 3 — Extend ValueShape without bloating joins or losing semantic category

- [x] Complete this task and its focused validation: **Extend ValueShape without bloating joins or losing semantic category**

**Files:**
- `phalcom-lsp/src/semantic/facts.rs`
- `phalcom-lsp/src/semantic/flow.rs`
- fact/flow tests

Current reference: [`semantic/facts.rs`](https://github.com/aureat/phalcom-lang/blob/b5477b74dfa6f79a4b4487896a1d63699d98685e/phalcom-lsp/src/semantic/facts.rs#L1-L120).

## Step 3.1 — Add exact semantic variants

Add `Selector`, `SelectorPattern`, exact/open `Family`, `Method`, `MethodFamily`, `BoundMethod`, `BoundMethodFamily` as described in §2.3.

Use `Arc<CapturedMethodFamilyShape>` if profiler/allocation counters show repeated cloning during joins. Because semantic generations are immutable, Arc sharing is safe and consistent with current structural sharing.

## Step 3.2 — Join rules

Implement only semantically valid joins:

- identical Selectors/Patterns remain exact;
- Families with equal `SelectorSpec` join receiver shapes;
- Method with same CallableId remains exact; different Methods become bounded Union;
- MethodFamily with equal captured content remains exact; different captured families become bounded Union, not merged routing maps;
- BoundMethod with same Method may join receiver shapes;
- BoundMethodFamily with equal captured family may join receiver shapes;
- oversized unions still widen to Unknown under `MAX_SHAPE_UNION`.

**Do not union two different MethodFamily maps into one map.** That would incorrectly make mutually exclusive captured routes simultaneously available.

## Step 3.3 — Tests

Add adversarial joins proving:

```text
Family(receiver A, exact foo) + Family(receiver B, exact foo)
    -> Family(A|B, exact foo)

MethodFamily(snapshot v1) + MethodFamily(snapshot v2)
    -> Union[v1, v2]
```

Run:

```bash
cargo test -p phalcom-lsp semantic::facts
cargo test -p phalcom-lsp semantic::flow
```

Commit:

```bash
git commit -am "feat(lsp): model dynamic and captured callable families"
```

---

# Task 4 — Make exact and open `::` analysis uniformly dynamic

- [x] Complete this task and its focused validation: **Make exact and open `::` analysis uniformly dynamic**

**Files:**
- `phalcom-lsp/src/semantic/analyzer.rs`
- analyzer tests
- `semantic/flow.rs` only if expression propagation needs support

Current semantic bug is visible in [`analyzer.rs`](https://github.com/aureat/phalcom-lang/blob/b5477b74dfa6f79a4b4487896a1d63699d98685e/phalcom-lsp/src/semantic/analyzer.rs#L1-L380): Pinned MethodRef resolves to `Callable` immediately.

## Step 4.1 — Analyze first-class selector specs

`Expr::SelectorSpec`:

- exact -> `ValueShape::Selector(exact)`;
- pattern -> `ValueShape::SelectorPattern(pattern)`.

This allows:

```phalcom
const p = #foo(...)
const family = C >> p
```

to retain pattern identity through normal binding facts.

## Step 4.2 — Rewrite `analyze_method_ref`

```rust
fn analyze_method_ref(...) -> InferredValue {
    let receiver = analyze_expr(&reference.receiver, context);
    let spec = reference.spec.normalize()?;
    exact(ValueShape::Family {
        receiver: Box::new(receiver.shape),
        spec,
    }, range)
}
```

Do not call `context.resolver` at reference creation.

## Step 4.3 — Route Family calls by spec

Refactor `analyze_unqualified_call` and generic callable-value handling into one helper:

```rust
fn analyze_callable_value_call(
    shape: &ValueShape,
    args: &[PackItem],
    range: SourceRange,
    context: &AnalysisContext<'_>,
) -> InferredValue;
```

Family exact:

1. derive static call Selector when pack is static;
2. validate it equals the stored exact Method selector (or for exact getter/setter only the correct gateway is accepted by editor modeling);
3. resolve stored selector dynamically on receiver targets.

Family pattern:

1. derive exact call selector;
2. test common `SelectorPattern::matches`;
3. if mismatch, produce Unknown/error fact as the diagnostics design requires, but **do not** resolve receiver;
4. if match, use `DispatchResolver` on receiver targets.

Dynamic/computed packs degrade conservatively; do not invent labels.

## Step 4.4 — Getter/setter Family protocol

Model known native Family `get()` and `set(_)` calls so selector kind is Getter/Setter, not Method `get()`/`set(_)` sent to the bound target. Prefer a semantic intrinsic keyed by receiver `ValueShape::Family`, not string exceptions scattered across completion and analyzer.

Tests:

- exact Family returns current target summary;
- exact Family is still shown as Family before call;
- open pattern chooses correct selector;
- mismatch never contributes call-site facts to target method;
- getter/nullary method distinction;
- labeled call shape.

Commit:

```bash
git commit -am "fix(lsp): keep exact method references dynamically dispatched"
```

---

# Task 5 — Extend DispatchResolver with effective MethodFamily capture

- [x] Complete this task and its focused validation: **Extend DispatchResolver with effective MethodFamily capture**

**Files:**
- `phalcom-lsp/src/semantic/dispatch.rs`
- `semantic/surface.rs`
- dispatch tests

Current exact resolver: [`semantic/dispatch.rs`](https://github.com/aureat/phalcom-lang/blob/b5477b74dfa6f79a4b4487896a1d63699d98685e/phalcom-lsp/src/semantic/dispatch.rs#L1-L210).

## Step 5.1 — Keep exact resolve unchanged semantically

Do not make exact `resolve` pattern-aware. Exact dispatch remains exact.

## Step 5.2 — Add effective capture operation

```rust
pub(crate) fn capture_method_family(
    &self,
    receiver: &DispatchReceiver,
    pattern: &SelectorPattern,
    access: &AccessContext,
) -> CapturedMethodFamilyShape;
```

Algorithm:

```text
seen_exact = ordered set
exact = []
rest = []
walk receiver lookup start subclass -> superclass:
    for direct member on requested side:
        if member.selector matches pattern and selector not seen:
            if reflectively accessible:
                capture (selector, callable)
            mark selector seen according to lookup shadowing rule
    if class has rest method for pattern base/kind:
        capture candidate in hierarchy order if accessible
```

Important shadowing rule: an inaccessible subclass method still shadows a superclass method for ordinary lookup. Therefore decide before implementation whether reflective pattern capture should omit the selector entirely or expose an access error marker; **do not** simply skip the inaccessible subclass and then capture the superclass method, because that would produce a Method ordinary dispatch could never reach for that selector. Recommended behavior: mark selector seen, omit inaccessible entry, and allow UI to report an inaccessible shadow when querying that exact selector.

## Step 5.3 — Rest candidates

The current `ClassSurface` needs enough information to identify rest methods by base and test acceptance. Preserve hierarchy order. Do not create concrete selector expansions.

## Step 5.4 — Tests

- three-level override flattening;
- instance/class side isolation;
- private subclass shadow does not leak superclass implementation;
- two rest methods in subclass/superclass where subclass rejects shape but superclass accepts;
- cyclic/invalid hierarchy protection remains bounded through existing visited set.

Run:

```bash
cargo test -p phalcom-lsp semantic::dispatch
```

Commit:

```bash
git commit -am "feat(lsp): capture effective MethodFamily routing snapshots"
```

---

# Task 6 — Analyze `Behavior >> selectorSpec`, Method binding, and BoundMethodFamily calls

- [x] Complete this task and its focused validation: **Analyze `Behavior >> selectorSpec`, Method binding, and BoundMethodFamily calls**

**Files:**
- `phalcom-lsp/src/semantic/analyzer.rs`
- `semantic/dispatch.rs`
- `semantic/facts.rs`
- `semantic/callable.rs` if intrinsic return metadata is centralized there
- tests

## Step 6.1 — Recognize extraction without new syntax

`C >> rhs` remains `Expr::Binary(BinaryOp::ShiftRight)`. Before generic numeric/operator result analysis, detect:

```text
left shape is ClassObject/Behavior-compatible
right shape is Selector or SelectorPattern
```

Then:

- Selector -> call `DispatchResolver::resolve` for exact reflection, yield `ValueShape::Method(callable)` when known;
- SelectorPattern -> call `capture_method_family`, yield `ValueShape::MethodFamily(Arc<...>)`.

If left/right are not this semantic combination, fall back to ordinary `>>(_)` dispatch. This preserves `Int >> Int` and user-defined polymorphic `>>`.

Prefer encapsulating this in `analyze_behavior_extraction` rather than embedding a long special case in the binary match.

## Step 6.2 — Method binding

Known `Method#bind(receiver)` yields:

```rust
ValueShape::BoundMethod {
    receiver,
    method: callable,
}
```

No nominal receiver-class filter is applied; transplantation is legal.

A BoundMethod call obtains return facts directly from the captured `CallableId`; dynamic sends inside that Method are already represented in the Method's callable summary/body analysis and do not change the capture identity.

## Step 6.3 — MethodFamily binding/calling

Known `MethodFamily#bind(receiver)` yields BoundMethodFamily with the captured snapshot.

BoundMethodFamily call:

1. derive exact Selector from call shape;
2. check captured pattern;
3. select captured exact CallableId if present;
4. else test captured rest candidates in order using their `RestSurface`;
5. obtain selected CallableId return summary;
6. contribute call-site parameter facts to **that captured callable**;
7. never call `DispatchResolver` on the bound receiver.

## Step 6.4 — Adversarial semantic test

Source:

```phalcom
class A { foo(_ x) { 1 } }
class B { foo(_ x) { "live" } }
const captured = A >> #foo(...)
const bound = captured.bind(B.new())
const result = bound(3)
```

Expected inferred result follows `A#foo(_)`, not `B#foo(_)`.

Also test dynamic counterpart:

```phalcom
const live = B.new()::foo(...)
const result = live(3)
```

Expected follows B.

Commit:

```bash
git commit -am "feat(lsp): analyze MethodFamily capture and bound routing"
```

---

# Task 7 — Make occurrence/index/reference/rename selector-spec aware

- [x] Complete this task and its focused validation: **Make occurrence/index/reference/rename selector-spec aware**

**Files:**
- `phalcom-lsp/src/semantic/occurrence.rs`
- `phalcom-lsp/src/index.rs`
- `phalcom-lsp/src/selectors.rs`
- reference/rename tests

## Step 7.1 — Add structured selector occurrence kinds

Do not index an entire pattern only as one opaque string span. Record at least:

```rust
SelectorOccurrence {
    base_range: SourceRange,
    base: String,
    kind: ExactOrPattern,
    whole_range: SourceRange,
    labels: Vec<(String, SourceRange)>,
}
```

Exact selector occurrence can resolve to one exact selector key. Pattern occurrence refers to a family and may correspond to multiple effective declarations depending on semantic receiver.

## Step 7.2 — Definition/reference behavior

- `C >> #foo(_)`: one current effective Method definition when semantic snapshot is compatible.
- `C >> #foo(...)`: multiple effective captured member definitions; return multiple LSP locations/links deterministically.
- `obj::foo(_)`: one currently resolved definition if receiver shape is known, but keep dynamic classification in hover/metadata.
- `obj::foo(...)`: current possible matching definitions only when receiver shape is bounded; otherwise do not workspace-scan.

## Step 7.3 — Rename

Base rename `foo` updates:

- declarations;
- direct sends;
- exact Selector literals;
- SelectorPatterns whose base is `foo`;
- `::foo...` family refs;

but only when the existing rename safety rules can establish semantic correspondence. Pattern labels such as `bar` in `#foo(_, ..., bar)` should be independently renameable as external selector labels if the current rename system supports parameter-label identity.

Never use regex/text replacement for selector patterns.

## Step 7.4 — Stale-snapshot behavior

When deep semantic snapshot is incompatible with current source revision:

- exact syntactic occurrence still works locally;
- multi-target pattern semantic navigation degrades rather than returning stale targets.

Tests must cover open-document edits between pattern creation and query.

Commit:

```bash
git commit -am "feat(lsp): index selector patterns and family references structurally"
```

---

# Task 8 — Add family-aware hover, completion, signature help, and definition UX

- [x] Complete this task and its focused validation: **Add family-aware hover, completion, signature help, and definition UX**

**Files:**
- `phalcom-lsp/src/completion.rs` — callable/member completion construction
- `phalcom-lsp/src/hover.rs` — hover presentation
- `phalcom-lsp/src/backend.rs` — LSP handler glue, definition/references/signature-help request routing where those handlers currently live
- `phalcom-lsp/src/index.rs` — definition/reference targets and selector-key index data
- `phalcom-lsp/src/analysis_service.rs` — coherent source/semantic snapshot query entry points
- `phalcom-lsp/src/semantic/query.rs` and `semantic/mod.rs` — bounded semantic query APIs

## Step 8.1 — Central semantic presentation helper

Add one formatter over semantic shapes rather than bespoke feature strings:

```rust
fn describe_callable_shape(shape: &ValueShape, snapshot: &SemanticSnapshot) -> CallableDescription;
```

It must distinguish:

```text
Family — exact selector, dynamic receiver lookup
Family — selector pattern, dynamic receiver lookup
Method — exact captured implementation
MethodFamily — captured implementation set
BoundMethod — exact captured implementation + receiver
BoundMethodFamily — captured routing set + receiver
```

## Step 8.2 — Hover

Examples:

```text
Family
receiver: Point
selector: #move(_,to)
dispatch: dynamic
```

```text
MethodFamily
source behavior: Point
pattern: #move(...)
captured methods: 3 exact, 1 rest candidate
```

Do not dump full bodies or huge candidate lists; cap preview and provide count.

## Step 8.3 — Completion/signature help

For a bounded Family receiver and pattern:

- enumerate matching effective selector shapes from semantic class surfaces;
- filter by pattern structurally;
- offer labels/signatures from currently possible declarations;
- for broad families, cap candidate count and sort deterministically.

For MethodFamily/BoundMethodFamily:

- use captured exact map/rest candidates only;
- do not include methods added to the bound receiver class.

Signature help for rest candidates should reflect finite declaration parameter shape, not synthesize infinite overloads.

## Step 8.4 — Query complexity budget

Editor query helpers may traverse:

- one already-known receiver class chain from the immutable snapshot; or
- one already-captured MethodFamily shape.

They may not scan every class in every module. Add a test hook/counter if existing query tests support this.

Commit:

```bash
git commit -am "feat(lsp): surface dynamic and captured family intelligence"
```

---

# Task 9 — Update semantic tokens and VS Code grammar without duplicating semantics

- [x] Complete this task and its focused validation: **Update semantic tokens and VS Code grammar without duplicating semantics**

**Files:**
- `phalcom-lsp/src/semantic_tokens.rs`
- `tools/vsphalcom/syntaxes/phalcom.tmLanguage.json`
- syntax/token tests

## Step 9.1 — Semantic tokens from range-rich AST

Classify selector spec components using AST ranges:

- base selector name/operator;
- exact/pattern label names;
- `...` punctuation;
- `::` operator;
- hash prefix punctuation according to current token legend policy.

Do not infer label spans by slicing selector strings.

## Step 9.2 — TextMate grammar

Highlight new forms:

```text
#foo
#foo()
#foo(...)
#foo(_, ..., bar)
#foo=...
obj::foo(...)
```

Do not assign semantic meaning in regex. TextMate only handles lexical coloring; LSP semantic tokens remain authoritative.

Run:

```bash
cargo test -p phalcom-lsp semantic_tokens
cd tools/vsphalcom
npm ci
npm run lint
npm run compile
npm test
```

Commit:

```bash
git commit -am "feat(tooling): highlight selector patterns and families"
```

---

# Task 10 — Integrate family dependencies into incremental invalidation

- [x] Complete this task and its focused validation: **Integrate family dependencies into incremental invalidation**

**Files:**
- `phalcom-lsp/src/semantic/invalidation.rs`
- `semantic/engine.rs`
- `semantic/infer.rs`
- `semantic/callable.rs`
- `semantic/surface.rs`
- incremental tests

## Step 10.1 — Distinguish body summary dependencies from family-surface dependencies

A Family/MethodFamily consumer has two possible dependencies:

1. **selected callable summary dependency** — body-only edits may change return/parameter facts;
2. **family membership dependency** — declaration/superclass/visibility/side/rest-layout changes may change which callable is selected/captured.

Represent these through existing dependency/invalidation structures rather than a second graph. If current edges are CallableId-only, add a compact class-surface dependency key such as:

```rust
SemanticDependency::ClassDispatchSurface(ClassId)
```

Only add it to callables that analyze Family/MethodFamily extraction/dispatch for that class.

## Step 10.2 — Source fingerprinting

Ensure shallow surface fingerprints change for:

- member add/remove;
- selector base/arity/labels/kind change;
- instance/class side change;
- rest mode/fixed labels change;
- visibility change if it affects reflection results;
- superclass change.

A pure body edit must leave the selector-family surface fingerprint unchanged.

## Step 10.3 — Captured MethodFamily dependency law

Once a MethodFamily shape is captured in one semantic generation:

- its member IDs come from that generation's class surface;
- later source revision recomputation builds a new fact/snapshot;
- within one immutable generation, call analysis never re-resolves captured membership.

This naturally mirrors runtime snapshot semantics without retaining historical semantic generations indefinitely.

## Step 10.4 — Tests with counters

Add incremental tests:

1. body edit to one `foo(_)` invalidates its summary consumers but does not rebuild unrelated selector-family indexes;
2. add `foo(label)` invalidates callables that captured `#foo(...)` for affected class descendants;
3. add unrelated `bar()` does not invalidate `#foo(...)` family consumers if class-surface dependency indexing is selector-family granular; if implementation chooses class-level granularity initially, document/measure that tradeoff and cap affected set;
4. edit unrelated module: no family consumer recompute;
5. rapid edits: stale family generation never publishes over newer source.

Commit:

```bash
git commit -am "perf(lsp): invalidate family semantics from dispatch-surface deltas"
```

---

# Task 11 — Preserve asynchronous request latency and optimize family queries

- [x] Complete this task and its focused validation: **Preserve asynchronous request latency and optimize family queries**

**Files:**
- semantic snapshot/query code
- feature handlers
- performance tests/bench harness

## Step 11.1 — No request-time deep family construction

Audit with:

```bash
rg -n 'capture_method_family|members_matching|SelectorPattern::matches' phalcom-lsp/src
```

Expected:

- deep capture is in semantic analysis/worker code;
- request handlers use snapshot products or bounded receiver-chain queries;
- no handler loops over all modules/classes.

## Step 11.2 — Precompute only useful indexes

Initial recommendation:

- retain exact `BTreeMap<String, MemberSides>` for O(log n) exact lookup;
- retain structural Selector on each member;
- scan only one class's direct member set when matching a pattern during semantic capture;
- do **not** add a global pattern cache before measurement.

If profiling shows large-class pattern capture significant, add a direct per-class index keyed by `(SelectorBase, SelectorKind)`:

```rust
BTreeMap<SelectorFamilyKey, Arc<[CallableId]>>
```

Build it once during `ClassSurface` construction. Pattern prefix/suffix matching then filters only the relevant family bucket.

Do not cache arbitrary SelectorPattern objects: the key space is user-controlled/unbounded.

## Step 11.3 — Memory budgets

Track/compare:

- `ClassSurface` retained bytes before/after structural Selector/rest metadata;
- semantic generation Arc reuse ratio;
- average/maximum CapturedMethodFamilyShape entry count;
- number of family shapes retained in local/interprocedural facts;
- clone counts if instrumentation exists.

Use Arc for captured family shapes only if it reduces real copying; do not introduce Arc everywhere speculatively.

## Step 11.4 — Latency tests

Add/extend tests proving:

- hover/completion remains responsive while semantic worker is behind;
- open document exact selector syntax works from current source snapshot even if semantic generation is older;
- family completion on a known class is bounded by that class hierarchy, not workspace size;
- cancellation/latest-wins behavior remains intact under rapid edits to family-heavy files.

Commit:

```bash
git commit -am "perf(lsp): bound selector-family semantic queries"
```

---

# Task 12 — Documentation, diagnostics, typing bridge, and final validation

- [x] Complete this task and its focused validation: **Documentation, diagnostics, typing bridge, and final validation**

**Files:**
- LSP docs/spec package if retained in repo
- selector/callable docs from core plan
- typing bridge docs where callable shapes are discussed
- tests

## Step 12.1 — Diagnostics

Provide distinct diagnostics/messages for:

- invalid selector pattern syntax (parser/source diagnostic);
- exact Family called with incompatible call kind/shape;
- open Family invocation shape outside pattern;
- captured MethodFamily call shape with no captured exact/rest route;
- exact Method extraction inaccessible/unresolved;
- pattern capture with inaccessible shadowing, if exposed to editor UX.

Do not report a selector-pattern mismatch as “method not found”; they are different semantic layers.

## Step 12.2 — Typing bridge

Document the future typing seam:

```text
selector identity / SelectorPattern
    -> callable candidate identity
    -> optional typed parameter/return constraints
```

Do not add type predicates to SelectorPattern. Later typed dispatch should consume the candidate set after selector routing.

## Step 12.3 — Focused tests

```bash
cargo fmt --all -- --check
cargo clippy -p phalcom-lsp --all-targets -- -D warnings
cargo test -p phalcom-lsp selectors
cargo test -p phalcom-lsp semantic::surface
cargo test -p phalcom-lsp semantic::dispatch
cargo test -p phalcom-lsp semantic::facts
cargo test -p phalcom-lsp semantic::analyzer
cargo test -p phalcom-lsp semantic_tokens
```

Use actual test filters present after implementation; if module paths differ, filter by concrete test name rather than skipping the gate.

## Step 12.4 — Broad Rust gates

```bash
CARGO_TARGET_DIR=target cargo test -p phalcom-ast
CARGO_TARGET_DIR=target cargo test -p phalcom-native-surface
CARGO_TARGET_DIR=target cargo test -p phalcom-lsp
CARGO_TARGET_DIR=target cargo test --workspace
```

## Step 12.5 — VS Code gates

```bash
cd tools/vsphalcom
npm ci
npm run lint
npm run compile
npm test
npm run test:lsp:e2e
```

## Step 12.6 — Retirement audit

```bash
rg -n 'MethodRefKind::Open|MethodRefKind::Pinned|ValueShape::Family\s*\{[^}]*base|NameSymbol|SelectorSymbol' phalcom-lsp phalcom-ast
```

Expected: no live old semantic model remains.

Commit:

```bash
git commit -am "test(lsp): validate selector-family semantic intelligence"
```

---

## 6. Acceptance-to-test matrix

| Requirement | Minimum proof |
|---|---|
| LSP uses shared selector semantics | selector adapter tests; no duplicate encoder |
| Class surfaces retain structural selector | surface unit tests |
| Rest acceptance retained | all rest modes surface tests |
| Exact `::` remains Family | analyzer test asserting `ValueShape::Family` |
| Open pattern Family | analyzer pattern shape test |
| Selector literal propagation through locals | local-flow test |
| `C >> exact` -> Method | binary/extraction analyzer test |
| `C >> pattern` -> captured MethodFamily | effective capture test |
| Capture respects inheritance overrides | dispatch three-level test |
| Inaccessible shadow does not expose superclass | access regression |
| Captured rest chain preserves fallback order | subclass-reject/superclass-accept test |
| BoundMethodFamily call never resolves receiver | adversarial A/B test |
| Dynamic Family call does resolve receiver | paired A/B dynamic test |
| Call-site parameter facts target captured callable | parameter contribution assertion |
| Family mismatch contributes no target facts | negative contribution assertion |
| Pattern definition can return multiple links | LSP integration test |
| Rename touches selector-spec base structurally | rename integration test |
| Label ranges are exact | occurrence/semantic-token test |
| Hover distinguishes dynamic/captured | hover integration test |
| Completion bounded to current family | completion result + query counter |
| Body edit does not change membership surface | incremental delta test |
| Member signature/add/remove invalidates captures | incremental recompute test |
| Unrelated module edit stays isolated | worker counter test |
| Stale semantic generation does not leak targets | rapid-edit integration test |
| No workspace scan on family hover/completion | instrumentation/query budget test |
| Existing Int shift-right semantics unchanged | workspace/core regression via companion plan |
| VS Code grammar highlights patterns | extension syntax test |

---

## 7. Performance and memory validation protocol

Do not accept “looks fast” reasoning. Record before/after metrics under the repository's existing LSP performance harness.

### 7.1 Workload shapes

Create fixtures with:

- 1 class, 20 selectors, 5 same-base overloads;
- 20-class inheritance chain, same-base overrides every 3 levels;
- 1 large class with 1,000 methods, only 20 matching one base;
- 100 modules with unrelated classes plus one open document family query;
- family-heavy file with 100 local bindings to exact/open Families;
- MethodFamily captures with 5, 50, and 500 exact members plus rest candidates.

### 7.2 Metrics

Record:

- shallow source build time;
- semantic solve time after body-only edit;
- semantic solve time after matching-selector addition;
- number of callables recomputed;
- number of class surfaces rebuilt;
- number of captured family shapes recomputed;
- request latency for hover/completion/signature help while worker is current and while worker is stale;
- peak retained semantic generation memory if harness exposes it.

### 7.3 Expected complexity

- exact dispatch: unchanged hierarchy lookup complexity;
- pattern capture without optional family index: O(methods in visited class chain) once in semantic analysis;
- pattern query from captured MethodFamily: O(1)/O(log n) exact map + O(rest depth) fallback;
- editor query must not be O(workspace classes);
- body-only edit should not rebuild family membership structures for unaffected classes.

### 7.4 Optimization decision gate

Only add the optional `(base,kind)` direct family index if large-class capture profiles show meaningful cost. If added, prove:

- it is immutable in `ClassSurface`;
- it is structurally shared between generations when class declaration surface is unchanged;
- it is rebuilt only on declaration-surface change;
- it contains direct members only; inheritance precedence remains in `DispatchResolver`.

---

## 8. Validation of dynamic-vs-captured semantics

Keep this paired regression in both unit and integration form because it is the easiest place for future refactors to become unsound.

```phalcom
class A {
  foo(_ x) { 1 }
}

class B {
  foo(_ x) { "B" }
}

const b = B.new()
const live = b::foo(...)
const captured = A >> #foo(...)
const bound = captured.bind(b)
```

Required semantic facts:

```text
live(0)  -> resolves B#foo(_) in current semantic generation
bound(0) -> resolves captured A#foo(_) without consulting B's surface
```

After editing B's `foo` body:

```text
live return fact may change with B summary
bound remains dependent on A summary
```

After editing A's selector from `foo(_)` to `foo(label)`:

```text
new semantic generation rebuilds the captured MethodFamily fact
old generation remains internally coherent until retired
```

This test simultaneously validates routing semantics, dependency edges, immutable generation coherence, and incremental recomputation.

---

## 9. Implementation anti-patterns to reject in review

1. **Resolving exact `::` at reference creation.** Exact Family is still live dynamic dispatch.
2. **Treating MethodFamily as `Union<CallableId>`.** A union loses selector-to-method routing and rest precedence.
3. **Using bound receiver type to resolve BoundMethodFamily.** That is semantically wrong even when it seems to improve autocomplete.
4. **Scanning workspace classes for pattern completion.** Use known receiver hierarchy or captured family only.
5. **Re-parsing canonical selector Strings in each feature.** Store structural Selector in surfaces.
6. **Adding a second selector encoder in LSP.** Common semantics are authoritative.
7. **Merging different MethodFamily snapshots during flow join.** Preserve them as alternatives.
8. **Ignoring inaccessible shadowing during family capture.** Skipping a private override and exposing the superclass is unsound.
9. **Treating rest declarations as infinite exact overloads.** Preserve finite ordered rest candidates.
10. **Blocking editor requests on semantic convergence.** Degrade when generation is stale.
11. **Putting mutable family caches into backend/request state.** Deep mutable state remains one-writer worker-owned.
12. **Adding type matching to SelectorPattern.** Typing composes after selector identity.

---

## 10. Completion criteria

The LSP work is complete only when:

- `phalcom-lsp/src/selectors.rs` no longer owns a divergent selector encoding algorithm;
- every source/native member surface has structural exact selector metadata and rest acceptance metadata where relevant;
- exact and open `::` are both represented as dynamic Family values;
- Selector and SelectorPattern values survive lexical/local/interprocedural propagation when facts are known;
- exact Method extraction and MethodFamily capture use the canonical `DispatchResolver`/surface snapshot;
- MethodFamily shape retains selector routing and ordered rest candidates;
- BoundMethodFamily calls never use the bound receiver as a dispatch target selector source;
- occurrences/references/rename use AST component ranges, not textual parsing;
- hover/completion/signature/definition explain and respect dynamic-vs-captured semantics;
- declaration-surface changes invalidate affected family consumers while body-only changes stay callable-local;
- family-aware editor requests remain bounded and non-blocking under stale semantic generations;
- Rust workspace and VS Code LSP/end-to-end gates pass;
- performance measurements show no material regression in ordinary non-family editor workloads.
