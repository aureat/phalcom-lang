# Phalcom Families, Selector Patterns, and Captured Implementations — Core Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the current Open/Pinned `Family` model with the ratified exact/open selector-spec semantics; introduce a shared structural `Selector`/`SelectorPattern` foundation, immutable `MethodFamily` snapshots and `BoundMethodFamily`, preserve ordinary dynamic dispatch and rest fallback, make `Behavior >> selectorSpec` ordinary polymorphic reflection, and support arbitrary Method transplantation without unsafe field-slot aliasing.

**Architecture:** Exact selector identity stays interned and fast in the VM. Structural selector semantics move into `phalcom-common`, where `phalcom-ast`, `phalcom-core`, and `phalcom-lsp` can share one definition without coupling the LSP to the VM. `Family` is always receiver-bound and never stores a `Method`; `MethodFamily` stores a captured selector-to-Method routing snapshot plus the captured ordered rest-candidate chain; `BoundMethodFamily` adds a receiver and never performs receiver-side lookup. Normal sends continue through the existing VM dispatch workhorses. Transplanted bytecode methods use a guarded foreign-receiver execution mode only when nominal holder compatibility is absent, keeping the ordinary dispatch path free of new hierarchy checks.

**Tech Stack:** Rust 2024 workspace; hand-written Phalcom lexer/parser; `phalcom-common`; `phalcom-ast`; `phalcom-core`; handle/arena heap; bytecode VM; `IndexMap`; existing shape-aware `ArgumentView`/`CallOutcome` forwarding ABI; language acceptance corpus under `phalcom-core/tests/lang`.

**Pinned repository checkpoint:** `b5477b74dfa6f79a4b4487896a1d63699d98685e` (`main` when this plan was authored). All code links below are pinned to that commit. If implementation begins from a later commit, resolve conflicts by symbol/function identity first, not by line number.

---

## 1. Current checkpoint and constraints

### 1.1 Repository observations this plan is based on

- `phalcom-common` currently exports only `range`; it is explicitly described as compiler-stage-agnostic shared infrastructure. See [`phalcom-common/src/lib.rs`](https://github.com/aureat/phalcom-lang/blob/b5477b74dfa6f79a4b4487896a1d63699d98685e/phalcom-common/src/lib.rs#L1-L27).
- The AST currently represents `::` as `MethodRefExpr { kind: MethodRefKind }`, where `MethodRefKind` is `Open { name } | Pinned { name, labels }`, and `#name` is represented as a `SymbolLiteralKind::Name` meaning a base-name family rather than an exact getter. See [`phalcom-ast/src/ast.rs`](https://github.com/aureat/phalcom-lang/blob/b5477b74dfa6f79a4b4487896a1d63699d98685e/phalcom-ast/src/ast.rs#L650-L1050), especially `MethodRefExpr`, `MethodRefKind`, `SymbolExpr`, and `SymbolLiteralKind`.
- The lexer atomically scans `#name` / `#name(...)` in `scan_symbol`, `scan_symbol_name`, and `scan_selector_labels`; `scan_selector_labels` currently accepts only `_` and identifier labels. See [`phalcom-ast/src/lexer.rs`](https://github.com/aureat/phalcom-lang/blob/b5477b74dfa6f79a4b4487896a1d63699d98685e/phalcom-ast/src/lexer.rs#L650-L1050).
- The parser's `parse_call` currently interprets `Token::SelectorSymbol` after `::` as Pinned, rejects a `Token::NameSymbol` after `::`, and otherwise parses an identifier as Open. See [`phalcom-ast/src/parser.rs`](https://github.com/aureat/phalcom-lang/blob/b5477b74dfa6f79a4b4487896a1d63699d98685e/phalcom-ast/src/parser.rs#L2450-L2780). `parse_primary` lowers the current symbol tokens to `Expr::Symbol`; see [`parser.rs`](https://github.com/aureat/phalcom-lang/blob/b5477b74dfa6f79a4b4487896a1d63699d98685e/phalcom-ast/src/parser.rs#L3000-L3340).
- The compiler's `Expr::MethodRef` arm interns either a bare name or encoded selector and emits one `Bytecode::MakeFamily`; the VM later guesses Open/Pinned from whether the string contains `(`. See [`phalcom-core/src/compiler/lib/expr.rs`](https://github.com/aureat/phalcom-lang/blob/b5477b74dfa6f79a4b4487896a1d63699d98685e/phalcom-core/src/compiler/lib/expr.rs#L500-L720).
- `Bytecode::MakeFamily(u16)` is currently documented to perform a reference-time empty-family check. See [`phalcom-core/src/bytecode.rs`](https://github.com/aureat/phalcom-lang/blob/b5477b74dfa6f79a4b4487896a1d63699d98685e/phalcom-core/src/bytecode.rs#L320-L520).
- The VM `MakeFamily` arm performs the string heuristic, method/base-name lookup, access check, dNU override check, and then allocates `FamilyObject { recv, selector, open }`. See [`phalcom-core/src/vm/dispatch.rs`](https://github.com/aureat/phalcom-lang/blob/b5477b74dfa6f79a4b4487896a1d63699d98685e/phalcom-core/src/vm/dispatch.rs#L1250-L1540).
- The heap currently has `Object::Family(FamilyObject)` and `Object::BoundMethod(BoundMethodObject)`; `FamilyObject` is exactly `{ recv: Value, selector: Symbol, open: bool }`. See [`phalcom-core/src/heap/object.rs`](https://github.com/aureat/phalcom-lang/blob/b5477b74dfa6f79a4b4487896a1d63699d98685e/phalcom-core/src/heap/object.rs#L1-L280).
- The existing shape-aware callable path is the correct implementation substrate. `activate_function` routes `BoundMethod` and `Family`; `activate_bound_method` activates a captured Method; `activate_family` derives/uses a selector and calls `dispatch_shape_at_as`. See [`phalcom-core/src/vm/send.rs`](https://github.com/aureat/phalcom-lang/blob/b5477b74dfa6f79a4b4487896a1d63699d98685e/phalcom-core/src/vm/send.rs#L300-L700).
- The VM already separates exact lookup/rest fallback/dNU in `dispatch_shape_at_as`, and rest method installation is separately represented by `RestLayout`/`RestMode`. Preserve this resolver order.
- `Behavior` already exists natively. `behavior_name` and `behavior_methods` live in [`phalcom-core/src/primitive/class.rs`](https://github.com/aureat/phalcom-lang/blob/b5477b74dfa6f79a4b4487896a1d63699d98685e/phalcom-core/src/primitive/class.rs#L1-L180), and they are installed on `behavior_class` in [`phalcom-core/src/universe/primitives.rs`](https://github.com/aureat/phalcom-lang/blob/b5477b74dfa6f79a4b4487896a1d63699d98685e/phalcom-core/src/universe/primitives.rs#L1-L180). `Int` separately owns `>>(_)`, so `Behavior#>>(_)` can coexist through ordinary dispatch.
- `Method#invokeOn` and `Method#bind` currently reject receivers using `VM::method_receiver_compatible`; see [`phalcom-core/src/primitive/method.rs`](https://github.com/aureat/phalcom-lang/blob/b5477b74dfa6f79a4b4487896a1d63699d98685e/phalcom-core/src/primitive/method.rs) and the beginning of [`vm/send.rs`](https://github.com/aureat/phalcom-lang/blob/b5477b74dfa6f79a4b4487896a1d63699d98685e/phalcom-core/src/vm/send.rs#L1-L120).
- Bytecode field access is physical-slot based. The compiler emits `GetField(slot)` / `SetField(slot)` in [`compiler/lib/expr.rs`](https://github.com/aureat/phalcom-lang/blob/b5477b74dfa6f79a4b4487896a1d63699d98685e/phalcom-core/src/compiler/lib/expr.rs#L930-L1170). Arbitrary Method transplantation is therefore unsound if the holder check is simply removed.
- `>>` already parses as `BinaryOp::ShiftRight` and compiles as the ordinary selector `>>(_)`; no new extraction AST operator is required. The mapping is in [`compiler/lib/expr.rs`](https://github.com/aureat/phalcom-lang/blob/b5477b74dfa6f79a4b4487896a1d63699d98685e/phalcom-core/src/compiler/lib/expr.rs#L1080-L1360).

### 1.2 Ratified semantic invariants to preserve

1. `receiver::selectorSpec` always produces `Family`.
2. Exact `Family` stores receiver + exact selector, never a Method.
3. Open `Family` stores receiver + selector pattern, never a Method or candidate snapshot.
4. Family construction does **zero target-method resolution**. Missing methods are a future-send concern.
5. Family invocation derives/uses one exact selector, validates selector-pattern membership, then uses ordinary receiver dispatch including inheritance, access control, rest fallback, and dNU.
6. `Behavior >> exactSelector` returns one exact effective `Method`.
7. `Behavior >> selectorPattern` returns immutable `MethodFamily` snapshot state.
8. `MethodFamily`/`BoundMethodFamily` never consult the bound receiver's class to select a Method after capture.
9. A captured family must snapshot **rest resolution** as well as exact selector bindings. Because the runtime can have one rest method per base per class, capture the ordered subclass-to-superclass rest candidate chain and test `RestLayout::accepts` at call time; do not flatten rest acceptance into infinitely many exact selectors.
10. Method binding/invocation may target arbitrary receivers. Ordinary `self.foo()` sends inside the captured Method remain dynamic on the new receiver; lexical `super` remains lexical to the captured Method.
11. Arbitrary receiver support must not allow physical field slot aliasing. Foreign bytecode activation is guarded; normal dynamic dispatch remains on the existing fast path.
12. SelectorPattern matching is structural selector matching only. It does not perform argument type matching and is not a replacement for rest acceptance.
13. Exact selectors stay canonical/interned dispatch keys. Patterns are predicates, not dispatch keys.
14. Optimizations may erase abstraction overhead, but may not convert dynamic `Family` semantics into captured `Method` semantics or vice versa.

---

## 2. Target semantic data model

### 2.1 Shared exact selector representation

Create `phalcom-common/src/selector.rs` with an owned, runtime-independent model:

```rust
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum SelectorKind {
    Getter,
    Setter,
    Method,
    SubscriptGet,
    SubscriptSet,
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum SelectorBase {
    Named(String),
    Subscript,
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum SelectorSlot {
    Positional,
    Label(String),
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Selector {
    pub base: SelectorBase,
    pub kind: SelectorKind,
    pub slots: Box<[SelectorSlot]>,
}
```

Required invariants enforced by constructors, not callers:

- Getter: named base, zero slots.
- Setter: named base, exactly the semantic `put` role; encode canonically as `name=(put)`.
- Method: named base, any legal slot sequence.
- SubscriptGet/SubscriptSet: subscript base only.
- Positional slots precede labeled slots for exact selectors.
- Slot count must fit the VM's `u8` send arity where a runtime dispatch selector is required.

Expose canonical operations in the same module:

```rust
impl Selector {
    pub fn encode(&self) -> String;
    pub fn decode(text: &str) -> Selector;
    pub fn method(name: impl Into<String>, slots: impl Into<Box<[SelectorSlot]>>) -> Result<Self, SelectorError>;
    pub fn getter(name: impl Into<String>) -> Result<Self, SelectorError>;
    pub fn setter(name: impl Into<String>) -> Result<Self, SelectorError>;
}

pub fn encode_label_component(text: &str) -> String;
pub fn decode_label_component(text: &str) -> String;
```

`decode` must remain total for runtime-originated arbitrary selector Symbols. If strict structural parsing is needed by the compiler/parser, expose a separate `try_decode_exact` rather than making the dNU/reflection path panic.

### 2.2 SelectorPattern representation

Use a single-gap structural pattern. It directly represents the ratified forms without a combinatorial predicate type:

```rust
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum SelectorKindPattern {
    AnyNamed,
    Exact(SelectorKind),
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SelectorPattern {
    pub base: SelectorBase,
    pub kind: SelectorKindPattern,
    pub prefix: Box<[SelectorSlot]>,
    pub suffix: Box<[SelectorSlot]>,
    pub has_gap: bool,
}

impl SelectorPattern {
    pub fn matches(&self, selector: &Selector) -> bool;
}
```

Normalization examples:

| Source | Normalized form |
|---|---|
| `#name...` | `Named("name")`, `AnyNamed`, gap=true, no prefix/suffix |
| `#name(...)` | `Named("name")`, `Exact(Method)`, gap=true |
| `#name(_, ...)` | Method; prefix `[Positional]`; gap=true |
| `#name(..., foo)` | Method; suffix `[Label("foo")]`; gap=true |
| `#name(_, ..., foo)` | Method; prefix positional; suffix label `foo`; gap=true |
| `#name=...` | `Exact(Setter)` family |

Exact forms are `Selector`, not `SelectorPattern`:

| Source | Exact selector |
|---|---|
| `#name` | getter `name` |
| `#name()` | method `name()` |
| `#name(_)` | method `name(_)` |
| `#name(foo)` | method `name(foo)` |
| `#name=(put)` | setter `name=(put)` |

Reject more than one `...` in one selector pattern. Reject a positional pattern slot after a concrete label where it would imply an impossible exact selector. Pattern matching is prefix/suffix structural matching after base/kind filtering; it must not allocate.

### 2.3 Runtime representations

Do not store common `String`-heavy patterns directly on the hot VM path. Compile first-class patterns once to interned runtime symbols:

```rust
#[derive(Clone, Debug)]
pub struct SelectorPatternObject {
    pub base: Symbol,
    pub kind: RuntimeSelectorKindPattern,
    pub prefix: Box<[RuntimeSelectorSlot]>,
    pub suffix: Box<[RuntimeSelectorSlot]>,
    pub has_gap: bool,
}

#[derive(Clone, Copy, Debug)]
pub enum FamilySpec {
    Exact(Symbol),
    Pattern(ObjRef), // Object::SelectorPattern
}

#[derive(Clone, Copy, Debug)]
pub struct FamilyObject {
    pub receiver: Value,
    pub spec: FamilySpec,
}

pub struct MethodFamilyObject {
    pub source_behavior: ClassId,
    pub pattern: ObjRef,
    pub exact_methods: IndexMap<Symbol, ObjRef>,
    pub rest_candidates: Box<[ObjRef]>,
}

#[derive(Clone, Copy, Debug)]
pub struct BoundMethodFamilyObject {
    pub family: ObjRef,
    pub receiver: Value,
}
```

`rest_candidates` is ordered from most-derived to least-derived behavior. It preserves current rest fallback precedence after capture. It may contain candidates that do not accept every shape; `RestLayout::accepts` is tested at BoundMethodFamily call time.

---

## 3. Parallel execution and review protocol

### Wave A — shared syntax/semantics seam; serialize these edits

Tasks 1–3 edit `phalcom-common` and `phalcom-ast`. Keep these on one branch or one sequential integration lane. The LSP companion plan may begin after Task 3 is merged.

### Wave B — runtime object and Family changes

Tasks 4–6 can proceed after Task 3. `bytecode.rs`, `compiler/lib/expr.rs`, `heap/object.rs`, and `vm/dispatch.rs` are shared-core hotspots; do not have multiple agents edit the same file simultaneously.

### Wave C — captured implementation and transplantation

Tasks 7–10 may split by ownership: one worker can own MethodFamily/Behavior reflection while another owns transplantation/frame-layout guarding, but merge `vm/send.rs` sequentially.

### Wave D — optimization, docs, broad validation

Tasks 11–13 follow semantic correctness. Do not introduce specialization before the semantic corpus passes.

### Review rule

For each task:

1. Run the focused red test before implementation.
2. Implement only the behavior needed for that task.
3. Run focused tests.
4. Run directly affected crate tests.
5. Self-review the diff for semantic boundary violations.
6. Commit with one task-scoped commit.
7. Before merging a wave, have a second reviewer check dynamic-vs-captured semantics and a third pass check performance/regression risk for VM hot paths.

---

## 4. File ownership map

| Area | Files | Primary responsibility |
|---|---|---|
| Shared selector semantics | `phalcom-common/src/lib.rs`, new `phalcom-common/src/selector.rs`, new tests | exact selector/pattern structure and matching |
| Lexer/tokens | `phalcom-ast/src/token.rs`, `lexer.rs`, lexer snapshots | component-token selector syntax |
| AST/parser | `phalcom-ast/src/ast.rs`, `parser.rs`, `tests/parser.rs` | range-rich SelectorSpec syntax and `::` ownership |
| Runtime exact selector adapter | `phalcom-core/src/method/mod.rs` | `Signature`/rest metadata + thin common selector adapter |
| Compiler/bytecode | `phalcom-core/src/compiler/lib/expr.rs`, `bytecode.rs`, `chunk.rs` if disassembly/fusion matches variants | explicit Family spec lowering |
| Heap/GC/value | `phalcom-core/src/heap/object.rs`, `heap/mod.rs`, `heap/accessors.rs`, `heap/trace.rs`, `value/mod.rs` | new object variants and tracing |
| Dynamic/captured call routing | `phalcom-core/src/vm/send.rs`, `vm/dispatch.rs` | Family and BoundMethodFamily activation |
| Reflection | `phalcom-core/src/primitive/class.rs`, new `primitive/family.rs`, new `primitive/method_family.rs`, `primitive/method.rs`, `primitive/mod.rs`, `universe/primitives.rs` | `Behavior#>>`, object protocols |
| Transplant safety | `phalcom-core/src/frame.rs`, `vm/send.rs`, `vm/dispatch.rs`, `primitive/method.rs`, field bytecode handling | guarded foreign receiver execution |
| Core classes/invariants | `universe/core_classes.rs`, `tests/invariants.rs`, native surface generator/table if required | class identity and floor registration |
| Language tests | `phalcom-core/tests/lang.rs`, `tests/lang/functions/**`, `tests/lang/reflection/**`, negative fixtures | end-to-end semantics |
| Performance | `phalcom-core/benches/vm_bench.rs`, `benchmarks/**`, `docs/forge/perf-log/**` as project convention requires | allocation/send regression proof |
| Documentation migration | `docs/spec/current/selectors.md`, `docs/spec/callables/{family,method,bound-method,reflection,dispatch}.md`, ADR/status references | normative semantic update |

---

# Task 0 — Establish a reproducible baseline

- [ ] Complete this task and its focused validation: **Establish a reproducible baseline**

**Files:** no source edits.

- [ ] Confirm exact starting revision:

```bash
git rev-parse HEAD
# Expected for the plan baseline:
# b5477b74dfa6f79a4b4487896a1d63699d98685e
```

- [ ] Run formatter/lints/tests before changes and record any pre-existing failures separately:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
CARGO_TARGET_DIR=target cargo test -p phalcom-common
CARGO_TARGET_DIR=target cargo test -p phalcom-ast
CARGO_TARGET_DIR=target cargo test -p phalcom-core
```

- [ ] Capture baseline VM benchmark rows used later for Family/callable overhead. Use the repository's existing perf harness; do not compare debug builds.

- [ ] Inventory all obsolete Open/Pinned dependencies before editing:

```bash
rg -n 'MethodRefKind|NameSymbol|SelectorSymbol|MakeFamily|\.open\b|responds_to_base_name|base_names|finalize_class_base_names|method_receiver_compatible' \
  phalcom-common phalcom-ast phalcom-core phalcom-lsp docs
```

- [ ] Commit no source change. Save the baseline output in the implementation PR description or project perf log.

---

# Task 1 — Introduce the authoritative shared Selector / SelectorPattern model

- [ ] Complete this task and its focused validation: **Introduce the authoritative shared Selector / SelectorPattern model**

**Files:**
- Create `phalcom-common/src/selector.rs`
- Modify `phalcom-common/src/lib.rs`
- Create `phalcom-common/tests/selector.rs` or colocated `#[cfg(test)]` tests in `selector.rs`
- Modify `phalcom-core/src/method/mod.rs` only after common tests pass

## Step 1.1 — Write failing structural tests

- [ ] Add round-trip exact-selector tests for getter, nullary method, positional/labeled method, setter, subscript get/set, escaped labels, Unicode labels.
- [ ] Add pattern membership tables for every ratified pattern.
- [ ] Add negative-construction tests: multiple ellipses, impossible slot ordering, setter with method-style slot pattern, illegal subscript/name combination.
- [ ] Add a total-decode test proving malformed runtime strings never panic.

Example table test:

```rust
#[test]
fn method_gap_pattern_matches_prefix_and_suffix_without_allocation() {
    let p = SelectorPattern::named_method(
        "foo",
        vec![SelectorSlot::Positional],
        vec![SelectorSlot::Label("bar".into())],
        true,
    ).unwrap();

    assert!(p.matches(&Selector::method("foo", [
        SelectorSlot::Positional,
        SelectorSlot::Positional,
        SelectorSlot::Label("bar".into()),
    ]).unwrap()));
    assert!(!p.matches(&Selector::getter("foo").unwrap()));
}
```

Run:

```bash
cargo test -p phalcom-common selector
# Expected: RED before implementation, GREEN after.
```

## Step 1.2 — Implement common selector semantics

- [ ] Add the structures from §2.1/§2.2.
- [ ] Move the canonical comma-form and label escaping algorithm from `phalcom-core/src/method/mod.rs` into common.
- [ ] Keep a total decoding API for runtime reflection/dNU and a strict construction API for source/compiler validation.
- [ ] Implement `SelectorPattern::matches` as index-based prefix/suffix comparison. No `Vec` allocation inside `matches`.
- [ ] Derive `Eq`, `Hash`, `Ord`, `PartialOrd` because the LSP's `ValueShape` and deterministic maps need these traits.

## Step 1.3 — Turn `phalcom-core::method` into a runtime adapter

- [ ] Keep `SignatureKind`, `Signature`, `RestLayout`, and `RestMode` in core.
- [ ] Replace local encoding/escaping implementation with calls/reexports to `phalcom_common::selector`.
- [ ] Preserve existing public helper names temporarily if that avoids a repo-wide flag day, but add tests proving the wrapper and common implementation are identical.

Validation:

```bash
cargo test -p phalcom-common
cargo test -p phalcom-core method::tests
```

Commit:

```bash
git add phalcom-common phalcom-core/src/method/mod.rs
git commit -m "refactor(selectors): centralize structural selector semantics"
```

---

# Task 2 — Replace atomic hash-symbol token semantics with range-preserving selector-spec tokens

- [ ] Complete this task and its focused validation: **Replace atomic hash-symbol token semantics with range-preserving selector-spec tokens**

**Files:**
- `phalcom-ast/src/token.rs`
- `phalcom-ast/src/lexer.rs`
- `phalcom-ast/tests/lexer.rs`
- affected `phalcom-ast/tests/snapshots/*symbol*`

**Reasoning:** Keeping the entire selector literal in one `Token::SelectorSymbol` prevents first-class source ranges for labels and `...`. The LSP needs component spans for rename, semantic tokens, and diagnostics. Do not deepen the atomic token payload; move selector structure to the parser.

## Step 2.1 — Change tokenization boundary

- [ ] Add a plain `Token::Hash` for selector/symbol literal prefix.
- [ ] Retain `Token::RecordLBrace` for `#{` and `Token::QuotedSymbol` for the existing `#"..."` form.
- [ ] Retire `Token::NameSymbol` and `Token::SelectorSymbol`.
- [ ] In lexer dispatch, emit `Hash` for `#` that is not `#{` and not the quoted-symbol opener.
- [ ] Let the existing ordinary lexer emit `Identifier`, `Underscore`, `LParen`, `RParen`, `DotDotDot`, `Equal`, and operator tokens after `Hash`.
- [ ] Preserve maximal munch: `#foo...` must lex `Hash`, `Identifier("foo")`, `DotDotDot`; `#foo (...)` is distinct because parser adjacency rules see the byte ranges.

## Step 2.2 — Lexer tests

Add explicit snapshots for:

```text
#name
#name()
#name(_)
#name(foo)
#name...
#name(...)
#name(_, ..., foo)
#name=...
#+
#==
#{ key: value }
#"quoted"
```

- [ ] Verify record and quoted-symbol forms are unchanged.
- [ ] Verify whitespace adjacency behavior is explicit, not accidental.

Run:

```bash
cargo test -p phalcom-ast lexer
```

Commit:

```bash
git add phalcom-ast/src/token.rs phalcom-ast/src/lexer.rs phalcom-ast/tests
git commit -m "refactor(ast): tokenize selector specs by components"
```

---

# Task 3 — Add range-rich SelectorSpec AST and make `::` own its selector syntax

- [ ] Complete this task and its focused validation: **Add range-rich SelectorSpec AST and make `::` own its selector syntax**

**Files:**
- `phalcom-ast/src/ast.rs`
- `phalcom-ast/src/parser.rs`
- `phalcom-ast/src/error.rs` if dedicated diagnostics are added
- `phalcom-ast/tests/parser.rs`
- parser snapshots

## Step 3.1 — Replace Open/Pinned AST

- [ ] Replace `MethodRefKind::{Open,Pinned}` with one selector-spec payload:

```rust
pub struct MethodRefExpr {
    pub receiver: Expr,
    pub spec: SelectorSpecSyntax,
    pub selector_range: SourceRange,
    pub range: SourceRange,
}
```

- [ ] Replace `SymbolExpr` / `SymbolLiteralKind::{Name,Selector}` with `SelectorSpecExpr` plus a source-oriented `SelectorSpecSyntax` that records component spans.
- [ ] Keep syntax AST independent of runtime `Symbol`; normalize to `phalcom_common::selector::{Selector,SelectorPattern}` through a method such as `SelectorSpecSyntax::normalize()`.
- [ ] Preserve exact ranges for base name/operator, every explicit slot/label, ellipsis, accessor `=`, and whole spec.

## Step 3.2 — Implement parser entry points

Add dedicated helpers near `parse_call`/`parse_primary`:

```rust
fn parse_hash_selector_spec(&mut self, hash_start: usize) -> ParserResult<Expr>;
fn parse_selector_spec_after_colon_colon(&mut self) -> ParserResult<(SelectorSpecSyntax, SourceRange)>;
fn parse_selector_spec_body(&mut self, mode: SelectorSpecMode) -> ParserResult<SelectorSpecSyntax>;
```

- [ ] `parse_primary` on `Token::Hash` parses a first-class exact Selector or SelectorPattern expression.
- [ ] The `::` branch in `parse_call` invokes `parse_selector_spec_after_colon_colon`; it does not create an Open Family and then allow ordinary postfix `()` to steal the selector's parens.
- [ ] Therefore `obj::name()` parses as one `MethodRefExpr` whose spec is exact method `name()`, **not** `(obj::name)()`.
- [ ] `obj::name` parses exact getter.
- [ ] `obj::name...`, `obj::name(...)`, `obj::name(_, ..., foo)`, `obj::name=...` parse open patterns.
- [ ] Keep `C >> #spec` as ordinary `Expr::Binary(ShiftRight)`.
- [ ] Permit first-class `const p = #foo(...); C >> p` with no special RHS AST.

## Step 3.3 — Parser diagnostic matrix

Add negative tests for:

- multiple `...` gaps;
- unterminated selector spec;
- positional slot after a fixed label where exact selector ordering is impossible;
- setter pattern with illegal method slots;
- invalid `::` spec;
- `obj::name()` ownership regression;
- newline/ASI boundaries around `::` and selector components.

Run:

```bash
cargo test -p phalcom-ast parser
cargo test -p phalcom-ast
```

Commit:

```bash
git add phalcom-ast
git commit -m "feat(ast): parse exact and patterned selector specs"
```

**Cross-plan gate:** The LSP implementation plan may now begin. Do not start LSP selector modeling before this AST/common seam is stable.

---

# Task 4 — Lower exact selectors, selector patterns, and explicit Family spec kind

- [ ] Complete this task and its focused validation: **Lower exact selectors, selector patterns, and explicit Family spec kind**

**Files:**
- `phalcom-core/src/compiler/lib/expr.rs`
- `phalcom-core/src/bytecode.rs`
- `phalcom-core/src/heap/object.rs`
- `phalcom-core/src/heap/mod.rs`
- compiler tests where bytecode is asserted

## Step 4.1 — Keep one opcode index, add an explicit discriminator

Do **not** add two bytecodes unless measurement proves it useful. Replace:

```rust
MakeFamily(u16)
```

with:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FamilySpecKind {
    Exact,
    Pattern,
}

MakeFamily {
    spec: u16,
    kind: FamilySpecKind,
}
```

Reason: this removes the punctuation heuristic while preserving `BYTECODE_NAMES`, `Bytecode::VARIANTS`, histogram indexes, and downstream profiling stability.

## Step 4.2 — Add first-class runtime SelectorPattern constant

- [ ] Add `Object::SelectorPattern(Box<SelectorPatternObject>)` or a compact inline variant after measuring object size; boxing is preferred initially so the arena slot size does not grow.
- [ ] Add compiler helper:

```rust
fn compile_selector_spec_constant(&mut self, spec: SelectorSpecSyntax) -> Result<(u16, FamilySpecKind), CompilerError>
```

Exact:
- normalize to common `Selector`;
- canonical encode;
- intern to `Value::Symbol`.

Pattern:
- normalize to common `SelectorPattern`;
- intern base/label symbols once into `SelectorPatternObject`;
- allocate immutable object;
- put `Value::Obj(pattern_id)` in the constant pool.

## Step 4.3 — Rewrite `Expr::MethodRef`

Replace the current `MethodRefKind` switch in [`compiler/lib/expr.rs`](https://github.com/aureat/phalcom-lang/blob/b5477b74dfa6f79a4b4487896a1d63699d98685e/phalcom-core/src/compiler/lib/expr.rs#L500-L720) with:

```rust
let method_ref = *method_ref;
let (spec_idx, kind) = self.compile_selector_spec_constant(method_ref.spec)?;
self.compile_expr(method_ref.receiver)?;
self.emit(Bytecode::MakeFamily { spec: spec_idx, kind }, method_ref.range);
```

- [ ] First-class exact Selector expression still evaluates to `Value::Symbol`.
- [ ] First-class pattern expression evaluates to `Object::SelectorPattern`.
- [ ] Replace `canonical_symbol` call sites that currently interpret `SymbolLiteralKind::Name` as raw base-name symbol. Product labels that intentionally use ordinary Symbols must remain ordinary symbols; do not accidentally reinterpret map/record labels as selector specs.

Run focused compiler/unit tests, then:

```bash
cargo test -p phalcom-core compiler
cargo test -p phalcom-core --test lang functions
```

Commit:

```bash
git add phalcom-core/src/compiler phalcom-core/src/bytecode.rs phalcom-core/src/heap
git commit -m "feat(core): lower explicit exact and patterned Family specs"
```

---

# Task 5 — Make Family construction pure capture of receiver + selector spec

- [ ] Complete this task and its focused validation: **Make Family construction pure capture of receiver + selector spec**

**Files:**
- `phalcom-core/src/heap/object.rs`
- `phalcom-core/src/vm/dispatch.rs`
- `phalcom-core/src/heap/class.rs`
- tests for bytecode/runtime construction

## Step 5.1 — Generalize FamilyObject

Replace `{ recv, selector, open }` with `{ receiver, spec: FamilySpec }`.

## Step 5.2 — Rewrite VM `MakeFamily`

In the current arm at [`vm/dispatch.rs`](https://github.com/aureat/phalcom-lang/blob/b5477b74dfa6f79a4b4487896a1d63699d98685e/phalcom-core/src/vm/dispatch.rs#L1250-L1540):

- [ ] Delete `sym_str.contains('(')` discrimination.
- [ ] Delete `responds_to_base_name` check.
- [ ] Delete exact-selector lookup/access check at reference time.
- [ ] Delete dNU override exception logic.
- [ ] Validate only the constant's representation against `FamilySpecKind`.
- [ ] Pop receiver and allocate immutable `Family`.

Expected core invariant:

```text
Family construction cannot fail because the receiver currently lacks a matching Method.
```

## Step 5.3 — Retire obsolete base-name machinery only after proving usages

Run:

```bash
rg -n 'base_names|responds_to_base_name|finalize_class_base_names|FinalizeClass' phalcom-core docs
```

- [ ] If `base_names` is only supporting the old Family empty check, remove it and `FinalizeClass` end-to-end, including bytecode/histogram/docs/tests.
- [ ] If another live feature still relies on it, keep it but rename/document it as a reflection/index cache and make `install_method_binding` update it transactionally. Do not leave a cache whose semantics silently go stale after reflective method replacement.

## Step 5.4 — End-to-end regression

Create language fixtures proving:

```phalcom
const f = object::futureMethod(_)
// construction succeeds before implementation exists
```

Where a surface mutation API is unavailable, add a Rust VM test that constructs a Family before installing a Method, installs the Method with the same internal installation path used by reflection, then invokes the Family and observes the new Method.

Commit:

```bash
git commit -am "fix(family): make references pure future-send values"
```

---

# Task 6 — Implement exact/open Family invocation over the existing dispatcher

- [ ] Complete this task and its focused validation: **Implement exact/open Family invocation over the existing dispatcher**

**Files:**
- `phalcom-core/src/vm/send.rs`
- new `phalcom-core/src/primitive/family.rs`
- `phalcom-core/src/primitive/mod.rs`
- `phalcom-core/src/universe/primitives.rs`
- runtime errors
- language tests

## Step 6.1 — Centralize invocation-to-selector construction

Add a helper that builds a runtime exact selector from an invocation kind and `ArgumentView` without temporary Strings where possible:

```rust
pub(crate) enum FamilyInvocationKind {
    Method,
    Getter,
    Setter,
}

fn selector_for_family_invocation(
    &mut self,
    base: Symbol,
    kind: FamilyInvocationKind,
    view: &ArgumentView,
) -> PhResult<Symbol>;
```

For Method, use positional count + ordered labels. Getter requires zero arguments. Setter requires exactly one value and canonical `=(put)` selector identity.

## Step 6.2 — Runtime pattern match must be allocation-free

Implement `SelectorPatternObject::matches_call(...)` against:

- base Symbol;
- invocation kind;
- positional count;
- label Symbol slice.

Do not resolve label Symbols to Strings on every call.

## Step 6.3 — Rewrite `activate_family`

Current code is at [`vm/send.rs`](https://github.com/aureat/phalcom-lang/blob/b5477b74dfa6f79a4b4487896a1d63699d98685e/phalcom-core/src/vm/send.rs#L520-L610).

Exact branch:

1. read the exact selector descriptor;
2. validate invocation shape/kind exactly;
3. replace Function receiver stack slot with bound receiver;
4. call `dispatch_shape_at_as` using the fixed exact selector.

Pattern branch:

1. derive one exact selector from call/get/set shape;
2. check `pattern.matches_call`;
3. on mismatch return a dedicated `RuntimeError::SelectorPatternMismatch` (or project-consistent named error), not dNU;
4. on match, replace receiver stack slot and call `dispatch_shape_at_as`.

The dispatcher must remain responsible for exact lookup → rest fallback → dNU.

## Step 6.4 — Add Family reflection/protocol primitives

Create thin primitives:

```phalcom
Family#receiver
Family#selector      // exact -> Selector/Symbol; pattern -> absence or explicit protocol decision
Family#pattern       // pattern -> SelectorPattern; exact -> absence
Family#isExact
Family#get()
Family#set(_)
```

`get`/`set` must enter the same `activate_family` router with explicit invocation kind; they must not duplicate lookup logic.

## Step 6.5 — Tests

Add positive/negative language fixtures for:

- exact getter `obj::name`;
- exact nullary method `obj::name()` distinct from getter;
- exact labeled selector;
- broad `obj::name...` selecting getter/method/setter through the correct gateway;
- method-only `obj::name(...)` rejecting `get`/`set`;
- constrained pattern membership;
- ordinary miss after successful pattern membership reaching dNU;
- mismatch failing before dNU;
- later Method replacement observed by an already-created Family;
- rest fallback preserved.

Focused gate:

```bash
cargo test -p phalcom-core --test lang functions
cargo test -p phalcom-core --test lang reflection
```

Commit:

```bash
git commit -am "feat(family): route exact and patterned future sends"
```

---

# Task 7 — Add exact effective Method extraction and immutable MethodFamily capture

- [ ] Complete this task and its focused validation: **Add exact effective Method extraction and immutable MethodFamily capture**

**Files:**
- `phalcom-core/src/primitive/class.rs`
- new `phalcom-core/src/primitive/method_family.rs`
- `phalcom-core/src/heap/object.rs`
- `phalcom-core/src/heap/accessors.rs`
- `phalcom-core/src/heap/trace.rs`
- `phalcom-core/src/universe/primitives.rs`
- `phalcom-core/src/vm/send.rs`
- tests

## Step 7.1 — Add shared live effective-family enumeration

Implement VM helper, not primitive-local logic:

```rust
pub(crate) fn capture_method_family(
    &mut self,
    behavior: ClassId,
    pattern: ObjRef,
    caller_authority: (Option<ClassId>, bool),
) -> PhResult<MethodFamilyObject>;
```

Exact bindings:

- walk `behavior`, then superclasses;
- visit each class's live `methods` dictionary in deterministic declaration order;
- decode/obtain selector structure;
- pattern-match structurally;
- first binding for each exact selector wins;
- skip inaccessible members for pattern enumeration, using the same visibility relation as reflection;
- never invoke dNU.

Rest snapshot:

- for the pattern's named Method family base, walk subclass → superclass;
- capture every distinct live rest candidate in lookup order;
- preserve `RestLayout` on each Method object;
- do not pre-expand accepted shapes.

Do not use stale `base_names` for correctness.

## Step 7.2 — `Behavior#>>(_)` is an ordinary primitive

Add to `primitive/class.rs`:

```rust
pub fn behavior_extract(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value>
```

Semantics:

- exact canonical Selector Symbol -> effective exact reflection result (`Method`) or the project-consistent absence/error behavior selected for `>>`;
- SelectorPattern object -> allocate `Object::MethodFamily` snapshot;
- any other RHS -> type error.

Install on `behavior_class` in `universe/primitives.rs`:

```rust
primitive!(vm, behavior_cls, ">>", SignatureKind::Method(1), behavior_extract);
```

Do not alter `Int#>>(_)`.

## Step 7.3 — Access semantics

- Exact extraction of an inaccessible selected Method must report access failure.
- Pattern capture omits methods that are not reflectively visible to the extracting caller; document this explicitly.
- Invocation of a captured Method/MethodFamily must **again** authorize the current caller. Capturing a Method is not an access-control capability escalation.

## Step 7.4 — Snapshot tests

Use live method replacement if exposed; otherwise use Rust VM-level installation tests:

1. capture MethodFamily;
2. replace `#foo(_)` binding in source Behavior;
3. bind/call captured family -> old Method;
4. create/call `obj::foo(_)` Family -> new Method;
5. extract a new MethodFamily -> new Method.

Also test inherited override flattening and rest-candidate order.

Commit:

```bash
git commit -am "feat(reflection): capture immutable MethodFamily snapshots"
```

---

# Task 8 — Add BoundMethodFamily and captured dispatch routing

- [ ] Complete this task and its focused validation: **Add BoundMethodFamily and captured dispatch routing**

**Files:**
- `phalcom-core/src/heap/object.rs`
- `phalcom-core/src/heap/trace.rs`
- `phalcom-core/src/value/mod.rs`
- `phalcom-core/src/primitive/method_family.rs`
- `phalcom-core/src/vm/send.rs`
- `phalcom-core/src/universe/core_classes.rs` if new surface classes require registration
- `phalcom-core/src/universe/primitives.rs`
- `phalcom-core/tests/invariants.rs`
- language tests

## Step 8.1 — Heap and Function hierarchy

Add:

```rust
Object::MethodFamily(Box<MethodFamilyObject>)
Object::BoundMethodFamily(BoundMethodFamilyObject)
```

Add tracing:

- MethodFamily traces pattern ObjRef and every captured Method ObjRef.
- BoundMethodFamily traces MethodFamily ObjRef plus any heap receiver Value.

Add class mapping/type names/debug display and invariants for new runtime classes.

## Step 8.2 — Bind protocol

`MethodFamily#bind(receiver)` returns `BoundMethodFamily` without looking at `receiver.class`.

`MethodFamily` itself is reflective/non-Function. `BoundMethodFamily` is Function-like and enters `activate_function`.

## Step 8.3 — Captured call resolver

Add `activate_bound_method_family` to `vm/send.rs`:

```text
ArgumentView
  -> derive exact Method selector
  -> verify captured SelectorPattern matches
  -> exact_methods.get(selector)
  -> else scan captured rest_candidates in order and test RestLayout::accepts
  -> selected Method direct activation on bound receiver
  -> no receiver.lookup_method, no dNU
```

If no captured route accepts the shape, return a MethodFamily call mismatch error. **Never** call `dispatch_shape_at_as`, because that would reintroduce live receiver lookup.

## Step 8.4 — Prove no redispatch

Add a regression where:

- captured family contains `Original#foo(_)`;
- bound receiver's actual class defines a different `foo(_)`;
- `bound(x)` executes captured Original Method;
- an ordinary `receiver::foo(_)(x)` executes receiver's live method.

Commit:

```bash
git commit -am "feat(function): add captured BoundMethodFamily routing"
```

---

# Task 9 — Permit arbitrary Method receivers with guarded representation access

- [ ] Complete this task and its focused validation: **Permit arbitrary Method receivers with guarded representation access**

**Files:**
- `phalcom-core/src/primitive/method.rs`
- `phalcom-core/src/vm/send.rs`
- `phalcom-core/src/frame.rs`
- `phalcom-core/src/vm/dispatch.rs`
- `phalcom-core/src/error.rs`
- tests for field access and transplantation

**Non-negotiable soundness rule:** Do not implement transplantation by simply deleting `method_receiver_compatible`. Physical `GetField(slot)` / `SetField(slot)` can otherwise read/write unrelated slots on a foreign object.

## Step 9.1 — Separate nominal compatibility from permission to invoke

Rename or reinterpret the existing helper as a fast-path predicate:

```rust
pub(crate) fn method_receiver_nominally_compatible(&self, method: ObjRef, receiver: Value) -> bool;
```

It no longer rejects exact invocation. It answers whether the method can use ordinary unguarded layout assumptions.

## Step 9.2 — Add foreign-receiver frame mode

Extend `CallFrame` with a compact optional guard:

```rust
pub(crate) struct ForeignReceiverGuard {
    pub layout_owner: ClassId,
}

pub(crate) foreign_receiver_guard: Option<ForeignReceiverGuard>,
```

Rules:

- ordinary selector dispatch: `None`;
- exact Method/BoundMethod/BoundMethodFamily activation on nominally compatible receiver: `None`;
- exact bytecode Method activation on foreign receiver: `Some(holder/layout_owner)`;
- primitive Method activation: no bytecode layout guard; primitive code remains responsible for semantic/type preconditions.

Add a dedicated exact activation entry point rather than contaminating ordinary send calls with an `allow_foreign` boolean:

```rust
pub(crate) fn activate_captured_method_as(
    &mut self,
    receiver: Value,
    method: ObjRef,
    view: ArgumentView,
    source_range: SourceRange,
) -> PhResult<CallOutcome>;
```

Reuse this from Method#invokeOn, BoundMethod, and BoundMethodFamily.

## Step 9.3 — Guard `GetField`/`SetField` only for foreign frames

At the VM field opcodes:

```rust
if let Some(guard) = self.frames.last().and_then(|f| f.foreign_receiver_guard) {
    self.guard_foreign_layout_access(receiver, guard.layout_owner, slot, access_kind)?;
}
```

The guard must verify semantic layout compatibility, not merely `slot < slots.len()`.

For instance receiver:
- accept if receiver's class is the layout owner or subclass whose inherited layout preserves the owner's slots.

For class/static receiver:
- validate the represented class/static layout against the captured method's lexical class; do not confuse the class object's metaclass with the represented class's static-slot layout.

On failure raise a dedicated error such as `RuntimeError::IncompatibleMethodLayout { selector, required, found }` before reading/writing the slot.

Ordinary calls therefore pay no hierarchy walk; the only extra hot-op cost is a predictable `Option::is_some()` branch in field opcodes.

## Step 9.4 — Update Method and BoundMethod gateways

In `primitive/method.rs`:

- remove holder/subclass rejection from `method_bind`;
- remove it from `method_invoke_on_shape`;
- preserve access authorization and argument-shape validation;
- route exact activation through `activate_captured_method_as`.

In `activate_bound_method`, remove the current compatibility error and use the captured activation helper.

## Step 9.5 — Transplant tests

Add cases:

1. Method uses only `self.title()`/`self.details()`; bind to unrelated receiver that supplies them -> succeeds and dispatches dynamically.
2. Method calls `super.foo()`; transplanted receiver still runs lexical super chain of captured Method.
3. Method directly reads `_field`; unrelated receiver with equal number of slots but different layout -> deterministic layout error, never aliased value.
4. Compatible subclass receiver -> direct field access succeeds.
5. Foreign primitive Method -> primitive's own checks determine behavior, no artificial holder block.
6. BoundMethodFamily-selected Method obeys the same rules.

Commit:

```bash
git commit -am "feat(method): guard representation access for transplanted receivers"
```

---

# Task 10 — Complete reflection protocols, GC, value identity, and invariants

- [ ] Complete this task and its focused validation: **Complete reflection protocols, GC, value identity, and invariants**

**Files:**
- `phalcom-core/src/primitive/family.rs`
- `phalcom-core/src/primitive/method_family.rs`
- `phalcom-core/src/primitive/method.rs`
- `phalcom-core/src/heap/{object,trace,accessors}.rs`
- `phalcom-core/src/value/mod.rs`
- `phalcom-core/src/universe/core_classes.rs`
- `phalcom-core/src/universe/primitives.rs`
- `phalcom-core/tests/invariants.rs`

- [ ] Define construction policy: SelectorPattern, MethodFamily, BoundMethodFamily cannot be directly allocated from generic `new` unless specifically ratified.
- [ ] Define `toString`/debug representation without recursively enumerating arbitrary Methods.
- [ ] Define `MethodFamily#selectors`, `#size`, `#methodFor(_)`, `#bind(_)` as reflection operations. Returning a fresh List for `selectors` is fine; the underlying routing snapshot remains immutable.
- [ ] Define exact `Method#selector` using canonical selector Symbol as today.
- [ ] Ensure new object variants receive the correct class in `Value::class`/object class mapping.
- [ ] Add GC reachability tests: a captured Method remains live solely through MethodFamily; bound receiver remains live solely through BoundMethodFamily.
- [ ] Update invariant/floor census tests for core classes and primitive counts.

Run:

```bash
cargo test -p phalcom-core --test invariants
cargo test -p phalcom-core --test lang functions
cargo test -p phalcom-core --test lang reflection
```

Commit:

```bash
git commit -am "feat(core): finish Family and MethodFamily object protocols"
```

---

# Task 11 — Add semantics-preserving compiler specializations

- [ ] Complete this task and its focused validation: **Add semantics-preserving compiler specializations**

**Files:**
- `phalcom-core/src/compiler/lib/expr.rs`
- optional new `phalcom-core/src/compiler/family.rs`
- optimizer/fusion code only if required
- compiler tests
- benchmarks

Do this **after** Tasks 1–10 pass without specialization.

## Optimization 11.1 — Elide immediately-called exact Family allocation

Recognize only provably non-escaping syntax such as an immediately invoked MethodRef:

```phalcom
obj::foo(_)(x)
```

or a compiler-proven immutable local whose sole use is the call, if existing IR analysis can prove it cheaply.

Lower to the same ordinary `Invoke`/pack path as `obj.foo(x)`. Do not cache/capture the currently resolved Method.

## Optimization 11.2 — Open Family with statically known call shape

For:

```phalcom
System::print...(value)
```

compile-time pattern membership can derive exact `#print(_)`; lower directly to ordinary dynamic send on `System`.

Guardrail test: replace `System#print(_)` after Family creation/compile and prove the optimized call observes the new implementation exactly like unoptimized Family semantics.

## Optimization 11.3 — Captured MethodFamily known-shape call

If compiler can prove the captured MethodFamily object and call shape are constant, it may select the captured Method directly, but must use exact captured activation. It may **not** rewrite to `receiver.foo(...)`.

## Optimization 11.4 — Allocation/cost budgets

Measure at minimum:

- Family construction exact vs pattern;
- Family call exact vs ordinary direct send;
- Family pattern call with statically known selector vs dynamic materialized Family;
- BoundMethodFamily call vs BoundMethod;
- MethodFamily capture cost over shallow/deep inheritance and 10/100/1000 methods;
- field-read hot benchmark before/after foreign-receiver guard branch.

Acceptance goals:

- no String parsing/allocation in steady-state Family pattern calls;
- no new heap allocation per Function call after Family object exists;
- ordinary direct send benchmark statistically unchanged within repository noise threshold;
- field-read hot path does not acquire a hierarchy walk;
- MethodFamily capture is linear in visited live method entries, not quadratic in inheritance × selector count.

Record measurements following existing `docs/forge/perf-log` conventions.

Commit only optimizations that are neutral/positive under the repository benchmark protocol.

---

# Task 12 — Migrate specifications and explicitly retire old semantics

- [ ] Complete this task and its focused validation: **Migrate specifications and explicitly retire old semantics**

**Files:**
- `docs/spec/current/selectors.md`
- `docs/spec/callables/family.md`
- `docs/spec/callables/method.md`
- `docs/spec/callables/bound-method.md`
- `docs/spec/callables/reflection.md`
- `docs/spec/callables/dispatch.md`
- `docs/spec/current/syntax/{lexical,expressions}.md`
- old U16/ADR references as project policy permits

- [ ] State that `obj::name` is exact getter, not old open-family base name.
- [ ] State `obj::name()` is exact nullary method and is parsed as part of the `::` selector spec.
- [ ] Document selector-pattern grammar and matching laws.
- [ ] State Family construction never probes receiver behavior.
- [ ] Document MethodFamily exact map + captured rest chain snapshot.
- [ ] Document dynamic/captured mutation law with paired examples.
- [ ] Document arbitrary Method receiver semantics and representation guard.
- [ ] Mark old `MethodRefKind::Open/Pinned`, `::#selector` pinned-only syntax, string heuristic, and empty-family construction rule as retired/superseded.
- [ ] Keep `>>` documented as ordinary operator dispatch; Behavior and Int simply implement different `>>(_)` methods.

Commit:

```bash
git commit -am "docs(callables): specify selector patterns and captured families"
```

---

# Task 13 — Final validation and acceptance matrix

- [x] Complete this task and its focused validation: **Final validation and acceptance matrix**

## 13.1 Focused semantic commands

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p phalcom-common
cargo test -p phalcom-ast
cargo test -p phalcom-core --test lang functions
cargo test -p phalcom-core --test lang reflection
cargo test -p phalcom-core --test invariants
```

## 13.2 Broad gates

```bash
CARGO_TARGET_DIR=target cargo test -p phalcom-ast
CARGO_TARGET_DIR=target cargo test -p phalcom-native-surface
CARGO_TARGET_DIR=target cargo test -p phalcom-core
CARGO_TARGET_DIR=target cargo test --workspace
```

## 13.3 Search-based retirement checks

```bash
rg -n 'MethodRefKind::Open|MethodRefKind::Pinned|open:\s*bool|contains\('\''\('\''\).*Family|empty family|::#' \
  phalcom-ast phalcom-core docs/spec
```

Expected: no live implementation path relies on the old Open/Pinned distinction or punctuation inference. Documentation may mention retired syntax only in migration/history sections.

## 13.4 Acceptance-to-test matrix

| Requirement | Minimum automated proof |
|---|---|
| One shared exact selector encoder | common round-trip tests + existing core selector tests |
| Pattern structural semantics | common exhaustive table tests |
| `#name` exact getter | parser + compiler + language fixture |
| `#name()` distinct method | parser ownership + runtime fixture |
| `obj::name` exact Family | AST + language fixture |
| Family construction without current Method | VM/runtime fixture |
| Exact Family remains live | method replacement regression |
| Pattern Family derives exact selector | labeled/positional fixtures |
| Pattern mismatch != dNU | negative fixture with dNU spy |
| Ordinary Family miss reaches dNU | proxy fixture |
| Rest Family call mirrors ordinary dispatch | exact miss + rest accept fixture |
| `Behavior >> exact` returns Method | reflection fixture |
| `Behavior >> pattern` returns MethodFamily | reflection fixture |
| Effective override flattening | three-level inheritance fixture |
| Captured exact map is immutable | replacement snapshot fixture |
| Captured rest chain is immutable | rest replacement/inheritance fixture |
| BoundMethodFamily never redispatches | conflicting receiver method fixture |
| Arbitrary Method receiver works | behavior-only transplant fixture |
| Lexical super survives transplant | transplant/super fixture |
| Foreign field slot cannot alias | equal-slot-count adversarial fixture |
| Normal field access fast path unchanged | benchmark + unit assertion on frame guard |
| Access control not bypassed by capture | private/protected/internal fixtures |
| GC retains captured methods/receiver | heap tracing test |
| `Int >>` remains shift-right | arithmetic regression |
| Optimized Family remains dynamic | optimized/unoptimized equivalence test |
| Optimized captured call remains captured | redispatch adversarial test |

## 13.5 Manual validation scenarios

Run a small script/REPL scenario that prints identity-observable results for:

```phalcom
const live = obj::foo(...)
const snap = C >> #foo(...)
const bound = snap.bind(obj)
```

Then replace a Method through the supported reflective API and confirm:

```text
live(...)  -> replacement implementation
bound(...) -> captured implementation
```

Also test an unrelated receiver transplant whose methods satisfy the captured Method's dynamic sends, and separately a captured Method that touches a direct field and must fail the representation guard.

## 13.6 Verification record — 2026-08-14

Separate read-only verification ran against feature handoff `9a18f068`.

Passing:

- `cargo fmt --all -- --check`.
- `cargo test -p phalcom-common`: 37 tests/doctests.
- `cargo test -p phalcom-ast`: 134 tests.
- Family-focused `phalcom-core` tests: functions (2), reflection (1), invariants (45).
- `cargo test -p phalcom-native-surface`: 1 test.
- `cargo test -p phalcom-core`: all 68 unit tests, 173 integration tests, and 45 invariant tests pass; the language harness passes 52/55 non-ignored cases.
- `cargo test --workspace --exclude phalcom-core`: all AST/common/LSP unit tests pass; LSP integration passes 41/43 non-ignored cases.

Baseline/unrelated:

- Three legacy `phalcom-core` language fixtures fail identically at parent `cfe3db30`: old reference-time empty-Family expectations, old DNU-backed `::typo` call shape, and obsolete hash-adjacency negative syntax.
- Two existing `phalcom-lsp` integration fixtures fail on current-syntax semantic-token expectations.
- Strict workspace clippy remains red on pre-existing LSP lint debt and `phalcom-core/src/compiler/inliner.rs:138`; family-introduced clippy findings were fixed and no longer remain.
- The retirement search still reports compatibility/history references and stale legacy fixture text; obsolete examples are non-blocking and require separate migration cleanup.

Unverified:

- Manual REPL scenarios in §13.5.
- Release-mode performance benchmarks and comparison against a recorded Task 0 baseline.
- Full green workspace acceptance until the baseline fixtures/lints above are migrated.

---

## 14. Implementation notes for efficiency and maintainability

1. **Do not build a second dispatcher.** Dynamic Families enter `dispatch_shape_at_as`; captured families select a stored Method then enter the existing exact activation path.
2. **Do not parse canonical selector strings on every Family call.** Exact Family holds Symbol; pattern Family holds interned runtime pattern; call shape supplies counts/labels structurally.
3. **Do not precompute an unbounded open-family candidate list.** Family is a predicate + future receiver send.
4. **Do not use a stale flattened base-name cache for MethodFamily correctness.** Initial capture may scan live dictionaries. Add an incremental direct-family index only if profiling shows extraction cost matters, and update it transactionally in method installation/replacement.
5. **Preserve deterministic MethodFamily iteration.** Use `IndexMap` and inheritance/declaration order even if dispatch lookup itself is keyed.
6. **Captured rest is finite.** Snapshot the finite ordered Method chain; evaluate each stored `RestLayout::accepts` against the call shape.
7. **Foreign-receiver safety belongs at representation use, not binding.** The frame guard preserves behavioral transplantation while making direct slot access explicit and safe.
8. **Keep hot-path branches cheap.** Ordinary sends must not gain selector-pattern checks or hierarchy validation. `GetField`/`SetField` should see at most one `Option` branch for foreign frames.
9. **Keep selector structure below AST/core/LSP.** If future typing adds type predicates, it should compose after selector identity rather than mutate `SelectorPattern` into a type matcher.
10. **Make every optimizer prove semantic category.** Dynamic Family → ordinary send; captured MethodFamily → exact captured Method. Never cross those arrows.

---

## 15. Completion criteria

The core feature is complete only when all of the following are true simultaneously:

- no live `open: bool` Family representation remains;
- no runtime punctuation heuristic determines selector semantics;
- exact and pattern selector semantics have one shared implementation;
- Family construction has no behavior lookup;
- Family call semantics use ordinary lookup/rest/dNU;
- MethodFamily is an immutable exact+rest snapshot;
- BoundMethodFamily never redispatches on receiver;
- Method and MethodFamily can be bound to arbitrary receivers;
- direct representation access on foreign receivers cannot alias unrelated slots;
- ordinary send/field performance is not materially regressed;
- parser, runtime, invariants, language corpus, workspace tests, and performance validation all pass;
- the old U16 Open/Pinned semantics are explicitly retired in normative documentation.
