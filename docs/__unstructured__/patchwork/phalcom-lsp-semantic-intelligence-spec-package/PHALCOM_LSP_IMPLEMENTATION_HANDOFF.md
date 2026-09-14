# Phalcom LSP Semantic Intelligence — Implementation Agent Handoff

**Repository baseline inspected:** `e2ec9e5fb6dc362786c9dd9593470feb47c91d94`  
**Documents in this package:**
1. `PHALCOM_LSP_ANALYSIS_DIAGNOSIS_AND_PLAN.md`
2. `PHALCOM_LSP_SPEC_01_SOURCE_TARGETS_AND_SCOPES.md`
3. `PHALCOM_LSP_SPEC_02_UNIFIED_INFERENCE_AND_DISPATCH.md`
4. `PHALCOM_LSP_SPEC_03_FLOW_SUMMARIES_FIELDS_PARAMETERS.md`
5. `PHALCOM_LSP_SPEC_04_LSP_INTEGRATION_TESTS_TYPING_BRIDGE.md`

---

# 1. Mission

Implement the four specs in order while preserving the working outer architecture of the current LSP.

The project is **not** asking for a new LSP rewrite.

Preserve:

- `SemanticDb`;
- module-qualified identities;
- module graph;
- coherent semantic generations;
- affected-module invalidation;
- bounded `ValueShape`;
- live core source;
- `phalcom-native-surface`;
- current completion recovery for incomplete `receiver.` syntax.

Replace the fragmented semantic walkers beneath those facilities.

---

# 2. Mandatory context discipline

This task can easily consume an agent's context before code is written.

Use **targeted reads and targeted writes only**.

Do not:

- dump/read all of `infer.rs` in one tool call unless a concrete edit genuinely spans the whole file;
- dump all of `backend.rs`;
- dump the whole AST/parser;
- reread the analysis document after every subtask;
- recursively inspect unrelated compiler modules;
- grep the entire repository and open every result.

Instead:

1. read the relevant spec section;
2. open only the named baseline line slice/function;
3. search exact constructors/callers by symbol;
4. open 30–120 lines around each real call site;
5. edit;
6. compile/test that slice;
7. proceed.

The line numbers are pinned to the inspected baseline. If main moved, find the named function/type and read only its local region.

---

# 3. First action: fix the protocol lie

Before semantic refactoring, make the independent LSP fix:

`phalcom-lsp/src/backend.rs`

Change inlay hint capability:

```text
resolveProvider: true
```

to false/absent.

There is no `inlayHint/resolve` implementation, and current hints contain `data: None`.

Add the transport assertion in `stage6_inlay_hints.rs`.

Run:

```bash
cargo test -p phalcom-lsp --test stage6_inlay_hints
```

This should eliminate the reported JSON-RPC `-32601 Method not found` VS Code error.

Do not implement a resolver unless later functionality genuinely needs lazy hint resolution.

---

# 4. Implementation order

Follow this order.

## Milestone A — exact source spans

Work in `phalcom-ast`.

Add target ranges from Spec 1.

After each AST structure change, use targeted search for constructors:

```bash
rg -n "MethodCallExpr \{" phalcom-ast phalcom-core phalcom-lsp
```

Open only the relevant surrounding functions.

Run parser/compiler tests frequently.

Do not proceed with hover heuristics to avoid adding spans; exact spans are foundational.

## Milestone B — scope/binding/occurrence graph

Add:

```text
semantic/scope.rs
semantic/occurrence.rs
```

Build per-file lexical identities.

Switch binding hover and visible completion names.

Prove shadowing/same-name-method isolation before touching full inference.

## Milestone C — dispatch

Add:

```text
semantic/dispatch.rs
```

Make side-aware inherited resolution authoritative.

Test class/instance collision and inherited method first.

Then `super`.

Do not refactor all inference at once before resolver tests pass.

## Milestone D — unified expression analyzer

Implement every `Expr` arm explicitly.

Migrate direct expression queries first.

Then migrate summary/parameter/dependency consumers.

## Milestone E — flow engine

Replace sequential local/summary/field/parameter/dependency walkers with one structured traversal.

Retain solver bottom-vs-Unknown distinction.

## Milestone F — LSP adapters and cleanup

Switch hover/completion/inlay/navigation.

Delete old paths only after replacement tests are green.

---

# 5. High-priority confirmed bugs

These are diagnosed, not hypotheses.

### Whole method hover

Cause:
`WorkspaceIndex::Collector` stores whole declaration/call expression ranges, and `selector_at_offset` selects containing ranges.

Fix:
exact source target spans + occurrence index.

### Keyword hover

Cause:
`Backend::hover_at` explicitly prioritizes `hover::keyword_at_offset`.

Fix:
remove branch; invert existing test.

### Getter inference

Cause:
`infer_expr` `GetProperty` only handles qualified class-name form.

Fix:
dispatch getter selector.

### `super.toString`

Cause:
bare `SuperVar` = Unknown and getter inference does not dispatch.

Fix:
dedicated super dispatch target.

### String interpolation

Cause:
parser lowers through `toString` getter + Binary `+`; both incomplete in inference.

Fix:
getter + operator dispatch; String native/source return knowledge.

### Local method-return summary

Cause:
summary evaluator does not run preceding local statement flow.

Fix:
flow engine.

### Constructor result

Cause:
source constructor body is summarized like ordinary method.

Fix:
constructor factory contract = instance.

### Field from constructor param

Cause:
field fact RHS inference uses empty environment.

Fix:
field writes collected during member flow with parameter state.

### Class/instance selector collision

Cause:
side-blind `members` map/resolver remains in solver paths.

Fix:
canonical side-aware resolver.

### Inherited callable return

Cause:
some paths fabricate `CallableId` with receiver class instead of resolved declaration owner.

Fix:
resolver returns actual `MemberSurface.callable`.

### VS Code "Method not found"

Cause:
server advertises `inlayHint.resolveProvider = true` but implements no resolver.

Fix:
advertise false.

---

# 6. Decisions that must not be reopened casually

The analysis already decided:

1. `ValueShape` is advisory runtime knowledge, not the future formal `Type`.
2. Preserve `SemanticDb`; do not replace it with a new architecture.
3. Add `BindingId`/`ScopeId`; do not continue spelling-keyed locals.
4. Exact ranges belong in AST; do not solve hover with broad regex rescans.
5. Use one side-aware dispatch resolver.
6. Use one recursive expression analyzer.
7. Use one structured statement-flow analyzer/event stream.
8. Native return contracts must be conservative.
9. `super` is a lookup mode/receiver target, not a normal superclass instance.
10. Keyword/literal syntax coloring remains; only hover is suppressed.

If implementation reveals a real contradiction with compiler/runtime semantics, document the evidence and adjust. Do not change these decisions merely for short-term code convenience.

---

# 7. Useful targeted search commands

Use exact-symbol searches:

```bash
rg -n "fn infer_expr|infer_expr_with_returns|infer_expr_with_fields" phalcom-lsp/src/semantic
rg -n "resolve_member_surface" phalcom-lsp/src
rg -n "collect_call_sites_expr|collect_dependency_expr|body_summary_value" phalcom-lsp/src/semantic
rg -n "selector_at_offset|selector_at_position" phalcom-lsp/src
rg -n "keyword_at_offset|keyword_blurb" phalcom-lsp/src
rg -n "resolve_provider" phalcom-lsp
rg -n "MethodCallExpr \{|GetPropertyExpr \{|SetPropertyExpr \{" phalcom-ast phalcom-core phalcom-lsp
rg -n "ForStatement \{|ClosureParameters" phalcom-ast phalcom-core phalcom-lsp
```

Do not use broad `cat`/whole-tree dumps.

---

# 8. Commit/checkpoint discipline

Prefer one coherent commit/checkpoint per milestone:

1. inlay capability fix;
2. AST span additions;
3. scope + occurrence graph;
4. dispatch resolver;
5. unified expression analyzer;
6. flow/summaries/events;
7. LSP adapters;
8. cleanup/tests.

At each checkpoint:

```bash
cargo fmt --all -- --check
cargo test -p <affected-crate>
```

Before final:

```bash
cargo test --workspace
cd tools/vsphalcom && npm run test:lsp:e2e
```

---

# 9. Testing discipline

Write regression tests before or alongside each bug fix.

Do not rely only on unit tests inside semantic modules.

For user-visible behavior, add transport-level tests through tower-lsp.

For VS Code-specific behavior, add extension E2E coverage.

Every hover regression test must assert `Hover::range`, not just hover text.

Every inference regression should compare at least one shared consumer pair where possible:

```text
inlay + completion
hover + callable summary
binding fact + receiver completion
```

---

# 10. Final cleanup warning

Do not delete old walkers at the beginning.

Parallel old/new code may exist briefly during migration.

Deletion is the final step after:

- behavior tests;
- solver convergence tests;
- invalidation tests;
- E2E tests.

The goal is not a heroic one-shot rewrite. The goal is to end with one authoritative semantic path without losing working LSP features during the transition.

---

# 11. Expected final architecture

The end state should be recognizable as:

```text
phalcom-ast exact source spans
        |
        v
scope / BindingId / semantic occurrences
        |
        v
canonical name + dispatch resolution
        |
        v
unified expression analyzer
        |
        v
structured statement flow
        |
        +--> binding facts
        +--> callable summaries
        +--> parameter facts
        +--> field facts
        +--> dependency edges
        |
        v
SemanticDb generation/invalidation
        |
        +--> hover
        +--> completion
        +--> inlay hints
        +--> navigation
        +--> semantic tokens
        +--> future type checker infrastructure
```

If the implementation starts growing another independent AST walker for one editor feature, stop and route that requirement through the shared analysis/event model instead.
