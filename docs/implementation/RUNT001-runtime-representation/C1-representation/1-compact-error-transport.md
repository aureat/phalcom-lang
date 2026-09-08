# Implementation Specification 1 — Compact Error Transport, Allocation-Free Family Pattern Matching, Inliner Ownership, and Clippy Hygiene

**Status:** Ratified
**Repository baseline:** `aureat/phalcom-lang@f0e51699060d31722c68b282a2d2e9a5b3260dfe`
**Primary crates:** `phalcom-core`, `phalcom-lsp`, `phalcom-modules`
**Depends on:** none
**Must complete before:** 16-byte `Value` migration

## 1. Purpose

This implementation pass cleans up the host-side representations and hot paths that should be fixed independently of `Value`.

It has five architectural goals:

1. Make rich runtime failures cold and indirect instead of allowing one diagnostic-heavy `RuntimeError` variant to inflate every `PhResult<T>`.
2. Replace the current Family-pattern invocation path that clones `SelectorPattern`, reconstructs a structural `Selector`, and allocates temporary selector metadata on successful sends with a VM-native Symbol-backed matcher.
3. Preserve the existing boxed AST node through sacred-call recognition rather than unpacking a 136-byte `MethodCallExpr` into a large `Result`.
4. Remove avoidable cloning of `PhError` and module-failure trees at diagnostic boundaries.
5. Eliminate the remaining LSP/modules Clippy warnings through semantic refactors rather than lint suppression.

This specification does **not** change Phalcom language semantics.

---

## 2. Hard architectural decisions

The following are not implementation options. They are the ratified design.

### 2.1 Rich runtime errors are boxed at the rich variant, not at `PhError`

Do:

```rust
RuntimeError::SelectorPatternMismatch(
    Box<SelectorPatternMismatchContext>
)
```

Do **not** do:

```rust
PhError::Runtime(Box<RuntimeError>)
```

and do not change:

```rust
type PhResult<T> = Result<T, PhError>;
```

to:

```rust
Result<T, Box<PhError>>
```

The cold exceptional payload should pay the indirection, not every runtime error.

### 2.2 `Raise` remains inline for this pass

After boxing `SelectorPatternMismatch`, re-measure `RuntimeError` and `PhError`.

Do not box `Raise` merely to chase an arbitrary smaller number unless the measured type remains at or above Clippy's large-error threshold or another variant becomes an obvious pathological outlier.

The immediate acceptance target is:

```text
size_of::<RuntimeError>() < 128
size_of::<PhError>()      < 128
```

with a preferred budget of `<= 120` bytes on the supported 64-bit development target.

Spec 2 should reduce them further because `Value` itself becomes 16 bytes.

### 2.3 Family pattern matching uses interned Symbols on the successful VM path

`phalcom_common::selector::{Selector, SelectorPattern}` remain the rich semantic representation for:

- parser/compiler normalization;
- LSP;
- diagnostics;
- reflection;
- tests of language-level selector semantics.

The VM gets a separate compiled pattern representation whose named base and labeled slots are interned `Symbol`s.

Do not mutate `phalcom-common` into a VM-specific representation.

### 2.4 Rich selector reconstruction occurs only when it is actually needed

A successful Family-pattern call must not construct a `phalcom_common::selector::Selector` merely to test the shape.

A mismatch may construct/clone rich selector data because it is already on an exceptional path.

### 2.5 Sacred-call recognition is classification, not failure

Replace:

```rust
Result<SacredCall, MethodCallExpr>
```

with a domain-specific classification enum.

### 2.6 Diagnostic reporting borrows errors

Rendering an error must not require cloning it simply because the same error must subsequently be propagated or stored.

### 2.7 Sticky module initialization failures use shared ownership

Module DAG failure propagation is a genuine multiple-owner case. Use `Arc` there instead of recursively deep-cloning failure trees.

Do not make all errors `Arc`-owned globally.

---

# 3. Context-budget contract for the implementing agent

This section is mandatory process guidance.

The implementation agent must **not** begin with a repository-wide source read.

## 3.1 Initial read budget

Read only these windows before writing code:

| File | Initial lines/symbols |
|---|---|
| `phalcom-core/src/error.rs` | lines 1–35, 135–225 |
| `phalcom-core/src/heap/object.rs` | lines 125–205 |
| `phalcom-core/src/compiler/lib/expr.rs` | lines 775–795 |
| `phalcom-core/src/vm/send.rs` | lines 900–965 |
| `phalcom-common/src/selector.rs` | `SelectorPattern::matches`, approximately lines 250–315 |
| `phalcom-core/src/compiler/inliner.rs` | lines 120–205 |
| `phalcom-core/src/compiler/lib/expr.rs` | lines 500–535 |
| `phalcom-core/src/interpret.rs` | lines 115–150 |
| `phalcom-core/src/vm/dispatch.rs` | lines 495–530 |
| `phalcom-core/src/modules/registry.rs` | lines 1–55 |
| `phalcom-core/src/modules/initialize.rs` | lines 1–105 |
| `phalcom-lsp/src/analysis_service.rs` | lines 540–825 and 850–1050 |
| the exact LSP lint windows listed in §10 | only those windows |
| `phalcom-modules/tests/repair_regressions.rs` | only the nine warned lines |

Do not read complete 1,000+ line source files.

## 3.2 Compiler-as-index rule

Once an API is changed, use:

```bash
cargo check -p phalcom-core --all-targets --message-format short
```

or the corresponding target crate.

When a compiler error identifies an additional construction/match site:

1. open only approximately ±25 lines around that error;
2. fix it;
3. rerun the targeted check.

Do not respond to one compiler error by reading the entire containing file.

## 3.3 Search-output discipline

If a search is required, prefer:

```bash
rg -l 'PATTERN' path/... > /tmp/phalcom-sites.txt
```

over dumping hundreds of matching lines into the agent context.

Open files from that list only when compilation or this specification requires them.

## 3.4 Compaction checkpoints

Compact/summarize agent context after:

1. error representation + tests;
2. Family matcher + tests;
3. inliner/error ownership;
4. LSP cleanup.

The compacted state needs retain only:

- ratified invariants;
- changed public/internal signatures;
- failing test names;
- remaining tasks.

Discard exploratory reasoning and source excerpts already translated into code.

---

# 4. Task A — Box the complete selector-pattern mismatch context

## Current state

`phalcom-core/src/error.rs:149` currently defines:

```rust
SelectorPatternMismatch {
    pattern: SelectorPattern,
    selector: Selector,
    family: Value,
    receiver: Value,
},
```

This one variant is the principal reason `RuntimeError`, `PhError`, and hundreds of `Result<_, _>` instantiations are large.

## Required replacement

In `phalcom-core/src/error.rs`, immediately before `RuntimeError`, add:

```rust
#[derive(Debug, Clone)]
pub struct SelectorPatternMismatchContext {
    /// Captured structural selector predicate.
    pub pattern: SelectorPattern,

    /// Concrete selector derived from the attempted invocation.
    pub selector: Selector,

    /// Invoked Family object.
    pub family: Value,

    /// Receiver retained by the Family.
    pub receiver: Value,
}

impl std::fmt::Display for SelectorPatternMismatchContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "selector pattern mismatch: attempted selector `{}` does not match family pattern `{}`",
            self.selector,
            self.pattern,
        )
    }
}
```

Replace the existing enum struct variant with:

```rust
#[error("{0}")]
SelectorPatternMismatch(Box<SelectorPatternMismatchContext>),
```

Add a constructor so callers do not manually repeat the allocation policy:

```rust
impl RuntimeError {
    pub(crate) fn selector_pattern_mismatch(
        pattern: SelectorPattern,
        selector: Selector,
        family: Value,
        receiver: Value,
    ) -> Self {
        Self::SelectorPatternMismatch(Box::new(
            SelectorPatternMismatchContext {
                pattern,
                selector,
                family,
                receiver,
            },
        ))
    }
}
```

### Why the whole context is boxed

Do not instead box only:

```rust
pattern: Box<SelectorPattern>
```

The whole diagnostic is cold. Boxing the entire context:

- costs the same single exceptional-path allocation;
- makes the enum variant pointer-sized;
- prevents future diagnostic fields from silently regrowing `RuntimeError`;
- groups the diagnostic state semantically;
- keeps all ordinary errors allocation-free.

---

# 5. Task B — Add representation-size regression tests

Add a `#[cfg(test)]` module near the bottom of `phalcom-core/src/error.rs`.

The first test must fail on the current pre-change representation and pass after Task A:

```rust
#[test]
#[cfg(target_pointer_width = "64")]
fn runtime_error_transport_stays_below_large_result_threshold() {
    let runtime = std::mem::size_of::<RuntimeError>();
    let ph = std::mem::size_of::<PhError>();

    assert!(
        runtime <= 120,
        "RuntimeError grew to {runtime} bytes; rich cold variants must be boxed"
    );
    assert!(
        ph <= 120,
        "PhError grew to {ph} bytes; inspect the largest inline variant"
    );
}
```

Use a budget below Clippy's 128-byte threshold so a small future addition cannot immediately reintroduce hundreds of warnings.

Do not assert an exact byte count in this spec. Exact `repr(Rust)` enum sizes are not the long-term contract.

---

# 6. Task C — Introduce the VM-native runtime selector pattern

## New file

Create:

```text
phalcom-core/src/heap/selector_pattern.rs
```

Move the runtime `SelectorPatternObject` definition out of `heap/object.rs` into this file.

## Required types

Use:

```rust
use phalcom_common::selector::{
    SelectorBase,
    SelectorKind,
    SelectorKindPattern,
    SelectorPattern,
    SelectorSlot,
};

use crate::interner::{Interner, Symbol};
```

Define:

```rust
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum RuntimeSelectorBase {
    Named(Symbol),
    Subscript,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum RuntimeSelectorKindPattern {
    AnyNamed,
    Exact(SelectorKind),
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum RuntimeSelectorSlot {
    Positional,
    Label(Symbol),
}

#[derive(Debug, Clone)]
pub(crate) struct RuntimeSelectorPattern {
    base: RuntimeSelectorBase,
    kind: RuntimeSelectorKindPattern,
    prefix: Box<[RuntimeSelectorSlot]>,
    suffix: Box<[RuntimeSelectorSlot]>,
    has_gap: bool,
}
```

`RuntimeSelectorPattern` is an execution representation. It must contain no owned `String`.

## Compilation

Implement:

```rust
impl RuntimeSelectorPattern {
    pub(crate) fn compile(
        pattern: &SelectorPattern,
        interner: &mut Interner,
    ) -> Self
```

Mapping rules:

```text
SelectorBase::Named(String)  -> RuntimeSelectorBase::Named(Symbol)
SelectorBase::Subscript      -> RuntimeSelectorBase::Subscript

SelectorKindPattern::AnyNamed
    -> RuntimeSelectorKindPattern::AnyNamed

SelectorKindPattern::Exact(k)
    -> RuntimeSelectorKindPattern::Exact(k)

SelectorSlot::Positional
    -> RuntimeSelectorSlot::Positional

SelectorSlot::Label(String)
    -> RuntimeSelectorSlot::Label(Symbol)
```

The compiler pays this cost once when the first-class pattern object is materialized.

## `SelectorPatternObject`

Define:

```rust
#[derive(Debug, Clone)]
pub struct SelectorPatternObject {
    /// Rich semantic representation retained for reflection and diagnostics.
    pub pattern: SelectorPattern,

    /// VM-oriented immutable matcher compiled once.
    pub(crate) runtime: RuntimeSelectorPattern,
}

impl SelectorPatternObject {
    pub(crate) fn compile(
        pattern: SelectorPattern,
        interner: &mut Interner,
    ) -> Self {
        let runtime = RuntimeSelectorPattern::compile(&pattern, interner);
        Self { pattern, runtime }
    }
}
```

There must be no lazy first-call compilation and no per-call cache lookup.

The compiler already has mutable access to both the VM interner and heap at pattern materialization time.

---

# 7. Task D — Compile the runtime pattern when the selector literal is created

Current construction is in:

```text
phalcom-core/src/compiler/lib/expr.rs:775–790
```

The existing code contains:

```rust
.heap
.alloc(crate::heap::Object::SelectorPattern(Box::new(
    crate::heap::SelectorPatternObject { pattern }
)));
```

Replace it with a two-stage operation so the mutable interner borrow ends before the heap allocation:

```rust
let pattern_object =
    crate::heap::SelectorPatternObject::compile(
        pattern,
        &mut self.vm.interner,
    );

let pattern = self
    .vm
    .heap
    .alloc(crate::heap::Object::SelectorPattern(
        Box::new(pattern_object),
    ));
```

Then preserve the existing:

```rust
let idx = self.add_constant(Value::Obj(pattern));
self.emit(Bytecode::Constant(idx), range);
```

until Spec 2 changes `Value` constructors.

## Construction-site rule

After this change, run:

```bash
cargo check -p phalcom-core --lib --message-format short
```

If another direct `SelectorPatternObject { pattern }` construction exists, open only that compiler-reported window and convert it to `SelectorPatternObject::compile`.

Do not repository-scan first.

---

# 8. Task E — Implement allocation-free runtime matching

`phalcom-common/src/selector.rs`'s `SelectorPattern::matches` is the semantic oracle.

The runtime matcher must be equivalent to it.

Add:

```rust
impl RuntimeSelectorPattern {
    pub(crate) fn matches_call(
        &self,
        kind: SelectorKind,
        positional_count: usize,
        labels: &[Symbol],
    ) -> bool
```

The structural call slots are logically:

```text
[Positional × positional_count, Label(labels[0]), Label(labels[1]), ...]
```

but **do not allocate this array**.

Implement an indexed helper:

```rust
fn call_slot_at(
    index: usize,
    positional_count: usize,
    labels: &[Symbol],
) -> Option<RuntimeSelectorSlot>
```

that returns:

```text
index < positional_count
    => Positional

otherwise
    => labels[index - positional_count].map(Label)
```

Then compare prefix/suffix in-place.

### Length rule

Preserve `SelectorPattern::matches` exactly:

```rust
let actual_len = positional_count + labels.len();
let minimum = self.prefix.len() + self.suffix.len();

if self.has_gap {
    if actual_len < minimum {
        return false;
    }
} else if actual_len != minimum {
    return false;
}
```

Then compare:

```text
prefix against slots [0 .. prefix.len]
suffix against slots [actual_len - suffix.len .. actual_len]
```

### Kind rule

Preserve:

```text
AnyNamed:
    Getter | Setter | Method only

Exact(k):
    actual kind must equal k
```

### Getter/setter structural arity

This distinction is critical.

A setter has one **runtime value argument**, but the structural selector:

```text
foo=(put)
```

has no `SelectorSlot` corresponding to the assigned value.

Therefore the matcher is called with:

```text
Getter -> positional_count = 0, labels = []
Setter -> positional_count = 0, labels = []
Method -> actual method positionals + actual labels
```

Do not accidentally pass the setter's assigned value as a structural positional slot.

---

# 9. Task F — Rewrite Family-pattern activation

Target:

```text
phalcom-core/src/vm/send.rs:approximately 915–958
```

The existing path currently:

1. clones `pattern.pattern`;
2. uses rich `String` base data;
3. allocates `Vec<Option<String>>` for a method shape;
4. interns an encoded selector;
5. decodes that selector string back into a rich `Selector`;
6. invokes `pattern.matches(&structural)`;
7. throws the temporary structures away on success.

Replace the decision ordering.

## Required order

### Step 1 — Borrow the pattern object and match the call shape

Derive:

```rust
let selector_kind = match invocation {
    FamilyInvocationKind::Getter => SelectorKind::Getter,
    FamilyInvocationKind::Setter => SelectorKind::Setter,
    FamilyInvocationKind::Method => SelectorKind::Method,
};

let structural_positionals = match invocation {
    FamilyInvocationKind::Getter
    | FamilyInvocationKind::Setter => 0,

    FamilyInvocationKind::Method => view.positional_count(),
};
```

Then:

```rust
let matches = {
    let pattern = match self.heap.get(pattern_id) {
        Object::SelectorPattern(pattern) => pattern,
        _ => {
            return Err(
                RuntimeError::Internal(
                    "Family pattern handle is not a selector pattern".into()
                ).into()
            );
        }
    };

    pattern
        .runtime
        .matches_call(
            selector_kind,
            structural_positionals,
            &labels,
        )
};
```

Make sure the immutable heap borrow has ended before later mutable VM operations.

### Step 2 — Mismatch: construct rich diagnostic data only now

On failure:

1. clone the retained rich `SelectorPattern`;
2. materialize the actual rich `Selector`;
3. construct `RuntimeError::selector_pattern_mismatch(...)`.

It is acceptable for this branch to allocate.

The diagnostic selector must faithfully encode the actual invocation shape.

Do not call `Selector::decode` on every successful send.

### Step 3 — Success: construct/intern only the dispatch selector

The VM still ultimately needs the canonical selector `Symbol` because ordinary method dispatch is symbol keyed.

Preserve that behavior.

For this spec, it is acceptable for canonical selector text generation to allocate one `String`. It is **not** necessary to redesign the global selector interner.

However, remove avoidable semantic reconstruction:

- no `SelectorPattern::clone()`;
- no `Selector::decode()` success-path round trip;
- no rich `Selector` solely for pattern matching.

A later measured optimization may introduce direct structured selector interning if selector text generation itself becomes significant.

---

# 10. Task G — Runtime matcher tests

Add unit tests in:

```text
phalcom-core/src/heap/selector_pattern.rs
```

Test at least:

### Exact method prefix/suffix

Pattern equivalent to:

```text
move(_, ..., duration)
```

must:

```text
match move(_, duration)
match move(_, to, duration)
match move(_, to, easing, duration)
reject move(duration)
reject move(_, to)
```

Use interned Symbol labels in the runtime matcher tests.

### Positionals before labels

Verify:

```text
[_ positional × N] + [label symbols]
```

matches the same shapes as rich `SelectorPattern::matches`.

For representative patterns, construct both rich and runtime forms and assert:

```rust
assert_eq!(
    rich.matches(&selector),
    runtime.matches_call(kind, positionals, &labels),
);
```

### AnyNamed

Verify getter, setter, and method acceptance and subscript rejection.

### Exact getter/setter

Verify the setter's assigned value does not become a structural pattern slot.

### Long labels / escaped selector labels

Intern a label requiring selector escaping and verify Symbol matching is independent of its encoded selector text spelling.

---

# 11. Task H — Preserve the boxed AST through inliner classification

Current warning:

```text
phalcom-core/src/compiler/inliner.rs:138
```

Current API:

```rust
pub(crate) fn recognize(
    call: MethodCallExpr,
) -> Result<SacredCall, MethodCallExpr>
```

## Replace it with

```rust
pub(crate) enum Recognition {
    Sacred(SacredCall),
    Ordinary(Box<MethodCallExpr>),
}

pub(crate) fn recognize(
    call: Box<MethodCallExpr>,
) -> Recognition
```

### Sacred branch rule

Inspect through the box without moving it first:

```rust
match (call.method.as_str(), call.args.len()) {
    // guards...
}
```

Only after a sacred arm is known should it destructure:

```rust
let MethodCallExpr {
    object,
    args,
    ..
} = *call;
```

### Ordinary branch

Return:

```rust
Recognition::Ordinary(call)
```

This must return the original allocation.

Do not:

```rust
Box::new(*call)
```

and do not clone the node.

## Caller

Target:

```text
phalcom-core/src/compiler/lib/expr.rs:approximately 505–535
```

Replace:

```rust
Err(*method_call)
```

and:

```rust
inliner::recognize(*method_call)
```

with:

```rust
let recognized = if self.in_deopt_fallback() {
    inliner::Recognition::Ordinary(method_call)
} else {
    inliner::recognize(method_call)
};
```

Then match:

```rust
match recognized {
    Recognition::Sacred(sacred) => { ... }
    Recognition::Ordinary(method_call) => { ... }
}
```

The semantic behavior must be unchanged.

---

# 12. Task I — Borrow diagnostic errors instead of cloning them

## `vm/dispatch.rs`

Target:

```text
phalcom-core/src/vm/dispatch.rs:503
```

Add:

```rust
pub fn report_runtime_error(&mut self, err: &PhError) {
    let config = crate::diagnostics::active_render_config();

    crate::diagnostics::traceback::render_traceback(
        self,
        err,
        &config,
        self.trace_core,
        self.trace_format_json,
    );
}
```

Rewrite existing:

```rust
pub fn runtime_error(&mut self, err: PhError) -> PhResult<()> {
    self.report_runtime_error(&err);
    Err(err)
}
```

This retains the convenience API while allowing move-preserving callers.

Change:

```rust
pub fn compiler_error(
    &mut self,
    err: PhError,
    ...
)
```

to borrow:

```rust
pub fn compiler_error(
    &mut self,
    err: &PhError,
    ...
)
```

Its existing body already pattern-matches through `&err`, so this is semantically natural.

## `interpret.rs`

Target:

```text
phalcom-core/src/interpret.rs:133–143
```

Replace combinator-based clone reporting with explicit ownership:

```rust
let closure = match self.compile_closure(module, source) {
    Ok(closure) => closure,
    Err(err) => {
        let source_id = self
            .heap
            .module(module)
            .sources
            .len()
            .saturating_sub(1) as u32;

        self.compiler_error(&err, module, source_id);
        return Err(err);
    }
};

if let Err(err) = self.run_in_module(module, closure) {
    self.report_runtime_error(&err);
    return Err(err);
}

Ok(())
```

This is preferable to clever combinators because the ownership flow is explicit.

---

# 13. Task J — Share sticky module failure trees

Current:

```text
phalcom-core/src/modules/registry.rs:18–27
```

uses recursive `Box` ownership and derives `Clone`.

Replace the persistent representation with:

```rust
use std::sync::Arc;

pub type ModuleFailureRef = Arc<ModuleFailure>;

#[derive(Debug)]
pub enum ModuleFailure {
    Initializer {
        cause: Arc<PhError>,
    },

    Dependency {
        dependency: ModuleId,
        cause: ModuleFailureRef,
    },
}
```

Change:

```rust
pub failure: Option<ModuleFailure>
```

to:

```rust
pub failure: Option<ModuleFailureRef>
```

## `ModuleInitializationError`

In `error.rs`, change:

```rust
pub failure: crate::modules::ModuleFailure,
```

to:

```rust
pub failure: crate::modules::ModuleFailureRef,
```

Adjust rendering through ordinary deref coercion.

## `initialize.rs`

Replace deep clones with `Arc::clone`.

Example direct initializer failure:

```rust
if let Err(err) = self.run_in_module(obj, closure) {
    self.report_runtime_error(&err);

    let failure = Arc::new(ModuleFailure::Initializer {
        cause: Arc::new(err),
    });

    let rec = self.module_registry.get_mut(id).unwrap();
    rec.state = ModuleState::Failed;
    rec.failure = Some(Arc::clone(&failure));

    return Err(self.build_initialization_error(id, failure));
}
```

Change:

```rust
fn build_initialization_error(
    &self,
    id: &ModuleId,
    failure: &ModuleFailure,
) -> PhError
```

to take:

```rust
fn build_initialization_error(
    &self,
    id: &ModuleId,
    failure: ModuleFailureRef,
) -> PhError
```

Walk its dependency chain through borrowed dereferences, but store the same `Arc` in `ModuleInitializationError`.

### Important limitation

Do **not** remove `Clone` from all of `PhError`/`RuntimeError` during this task unless the resulting compiler work is trivially local.

The architectural improvement required here is eliminating known deep-clone paths, not undertaking an unrelated public Rust API cleanup.

---

# 14. Task K — LSP scan-context refactor

Current Clippy warnings:

```text
phalcom-lsp/src/analysis_service.rs:866
refresh_disk_sources(... 9 args ...)

phalcom-lsp/src/analysis_service.rs:934
process_scan_batch(... 10 args ...)
```

Create immediately above these helpers:

```rust
struct ScanEnv<'a> {
    db: &'a SemanticDb,
    workspace_index: Option<&'a WorkspaceIndex>,
    source_cache: Option<&'a SourceCache>,
    shared: &'a WorkerShared,
    event_tx: &'a mpsc::UnboundedSender<AnalysisEvent>,
}
```

Do not use:

```rust
Option<&Arc<WorkspaceIndex>>
```

The callee does not need Arc ownership.

At worker-loop call sites use:

```rust
workspace_index.as_deref()
```

Add:

```rust
struct DiskRefreshDelta<'a> {
    file_updates: &'a mut BTreeMap<
        Url,
        (FileRevision, Program)
    >,

    source_texts: &'a mut BTreeMap<
        Url,
        Arc<str>
    >,

    removals: &'a mut BTreeSet<Url>,
}
```

Then signatures become approximately:

```rust
fn refresh_disk_sources(
    env: &ScanEnv<'_>,
    refreshes: BTreeSet<Url>,
    delta: &mut DiskRefreshDelta<'_>,
)
```

and:

```rust
fn process_scan_batch(
    env: &ScanEnv<'_>,
    engine: &mut SemanticEngine,
    mode: AnalysisMode,
    files: Vec<DiscoveredFile>,
    source_catalog: &mut BTreeMap<
        Url,
        (FileRevision, Arc<str>, Program)
    >,
    selected_core_uri: Option<&Url>,
)
```

Do not create a mutable twenty-field “worker context.” Keep persistent services and operation-specific mutation separate.

---

# 15. Task L — Mechanical LSP Clippy corrections

Apply these without semantic redesign.

## `backend.rs`

At current warnings:

```text
710
712
734
736
1443
1450
1462
```

remove immediately dereferenced borrows.

Example:

```rust
hover_member_kind(&member)
```

becomes:

```rust
hover_member_kind(member)
```

where Clippy identifies `member` as already the expected reference.

At approximately `backend.rs:918` replace:

```rust
let Some((name, range)) =
    hover::identifier_at_offset(&text, offset)
else {
    return None;
};
```

with:

```rust
let (name, range) =
    hover::identifier_at_offset(&text, offset)?;
```

## `completion.rs:140`

Collapse the nested uppercase-receiver check into the condition suggested by Clippy.

## `completion.rs:193`

Remove the `offset` parameter from:

```rust
collect_expression_classes
```

and recursive calls.

Do not change visibility semantics in this task.

The surrounding outer scan at approximately lines 150–160 already applies:

```rust
binding.range.start < offset
```

for source-position-sensitive top-level binding discovery. The recursive helper is structural class inference and currently does not use the offset for any decision.

Add or retain a completion regression test demonstrating that a later top-level binding does not become visible before its declaration through the outer path.

## `semantic/flow.rs:389`

Do not merely create a tuple alias.

Introduce:

```rust
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct BlockEffectKey {
    start: usize,
    end: usize,
    bindings: Vec<(BindingId, ValueShape)>,
}
```

Add:

```rust
impl BlockEffectKey {
    fn new(
        block: &BlockExpr,
        state: &FlowState,
    ) -> Self {
        Self {
            start: block.range.start,
            end: block.range.end,
            bindings: state
                .bindings
                .iter()
                .map(|(binding, value)| {
                    (*binding, value.shape.clone())
                })
                .collect(),
        }
    }
}
```

Change:

```rust
block_effects:
    BTreeMap<
        (usize, usize, Vec<(BindingId, ValueShape)>),
        BlockEffects
    >
```

to:

```rust
block_effects:
    BTreeMap<BlockEffectKey, BlockEffects>
```

At `ensure_block_effect`, use:

```rust
let key = BlockEffectKey::new(block, state);
```

Do **not** introduce flow-state interning in this pass. The named key creates the abstraction seam; Spec 3 can later show whether allocating the binding vector is materially expensive.

## `semantic/invalidation.rs:91`

Replace:

```rust
} else if module.as_str() == CORE_MODULE_URI {
    SourceChangeKind::BodyOnly
} else {
    SourceChangeKind::BodyOnly
}
```

with:

```rust
} else {
    SourceChangeKind::BodyOnly
}
```

Retain the earlier `CoreSurface` special case for declaration-fingerprint changes.

## `workspace_scan.rs:44`

Replace the inherent permissive `from_str` with a real `FromStr`.

Add:

```rust
#[derive(Debug, Clone, Eq, PartialEq, thiserror::Error)]
#[error(
    "invalid analysis mode `{0}`; expected `local` or `workspace`"
)]
pub struct ParseAnalysisModeError(String);

impl std::str::FromStr for AnalysisMode {
    type Err = ParseAnalysisModeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim() {
            "local" => Ok(Self::Local),
            "workspace" => Ok(Self::Workspace),
            other => Err(
                ParseAnalysisModeError(other.to_owned())
            ),
        }
    }
}
```

When compilation identifies the configuration boundary that previously called:

```rust
AnalysisMode::from_str(...)
```

open only that call site's ±25 lines.

If the LSP protocol/configuration boundary cannot propagate a parse error to the client, explicitly:

1. log/report the invalid setting;
2. fall back to `AnalysisMode::Local`.

Do not silently interpret every typo as `Local`.

## Test-module placement

Move test modules to the physical end of:

```text
phalcom-lsp/src/semantic/engine.rs
phalcom-lsp/src/semantic/invalidation.rs
```

rather than moving production functions below them.

---

# 16. Task M — `phalcom-modules` unit struct construction

In:

```text
phalcom-modules/tests/repair_regressions.rs
```

replace all nine warned occurrences:

```rust
SyntheticProjectIdAllocator::default()
```

with:

```rust
SyntheticProjectIdAllocator
```

Current warned lines:

```text
40
50
61
70
78
86
133
143
408
```

Do not remove `Default` from the type merely because these tests do not need it.

---

# 17. Required verification sequence

Do not jump directly to the full workspace test after every tiny edit.

Use the following progression.

### Runtime/error/matcher work

```bash
cargo fmt --all -- --check

cargo test -p phalcom-core --lib
cargo test -p phalcom-core --test integration
cargo test -p phalcom-core --test invariants
```

Run focused Family selector tests if they have a separate test target or filter.

Then:

```bash
cargo clippy \
  -p phalcom-core \
  --all-targets \
  --all-features \
  -- \
  -D warnings
```

### LSP

```bash
cargo test -p phalcom-lsp

cargo clippy \
  -p phalcom-lsp \
  --all-targets \
  --all-features \
  -- \
  -D warnings
```

### Modules

```bash
cargo test -p phalcom-modules

cargo clippy \
  -p phalcom-modules \
  --all-targets \
  --all-features \
  -- \
  -D warnings
```

### Final workspace gate

```bash
cargo fmt --all -- --check

cargo test --workspace

cargo clippy \
  --workspace \
  --all-targets \
  --all-features \
  --locked \
  -- \
  -D warnings
```

---

# 18. Definition of done

This spec is complete only when all of the following are true:

- `RuntimeError::SelectorPatternMismatch` contains one boxed context.
- `RuntimeError` and `PhError` are below the explicit size budget.
- no global `Box<PhError>` policy has been introduced.
- selector patterns contain an immutable Symbol-backed VM representation compiled at object creation.
- a successful Family pattern call performs no `SelectorPattern::clone()` and no `Selector::decode()` solely for matching.
- runtime pattern matching has equivalence tests against rich `SelectorPattern::matches`.
- inliner ordinary fallback preserves the original `Box<MethodCallExpr>`.
- sacred recognition is represented as classification rather than `Result` failure.
- `interpret_source` does not clone `PhError` merely to report it.
- module initialization does not deep-clone failure trees to preserve sticky state.
- the listed LSP and modules warnings are fixed semantically.
- no new blanket `#[allow(clippy::...)]` or crate-wide lint suppression was added.
- full workspace Clippy passes with `-D warnings`.