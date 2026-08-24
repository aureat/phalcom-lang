# Composition1 LSP Hover and Selector Fix Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make `composition1.rs` use the intended positional constructor/factory selectors and expose its observed `CellNum` shape in binding hover when formal analysis is `Unknown`.

**Architecture:** Fix selector syntax at the fixture boundary. Keep compiler-owned formal presentation separate from advisory LSP `ValueShape`; when formal knowledge is `Unknown`, render both states instead of hiding useful observed evidence. Preserve formal `Dynamic` precedence.

**Tech Stack:** Rust 2024, `phalcom-lsp`, `tower-lsp`, `serde_json`, `cargo test`.

**Spec:** `phalcom-lsp/tests/composition1.rs` — executable LSP composition contract; no separate normative specification is required for this scoped fix.

## Global Constraints

- Implementation scope: `phalcom-lsp/tests/composition1.rs` and `phalcom-lsp/src/hover.rs` only.
- Do not change backend dispatch, semantic ownership, `phalcom-semantic`, runtime behavior, or selector identity.
- Observed shape must never be relabeled as formal type.
- Formal `Dynamic` remains authoritative and must not gain an observed-type section.
- Preserve unrelated dirty/untracked files. Do not commit or clean without authorization.

---

### Task 1: Align fixture declarations with positional calls

**Files:**
- Modify: `phalcom-lsp/tests/composition1.rs:13-20`
- Test: `constructor_factory_inference_is_authoritative_across_lsp_features`

**Interfaces:** Existing call sites `CellNum.new(raw)` and `CellNum.of(42)` consume positional selectors. The fixture must produce `new(_)` and `of(_)`.

- [ ] **Step 1: Change both typed parameters to positional parameters**

Use:

```phalcom
@constructor
/*@constructor_decl*/new(_ raw: Int) {
  _raw = raw
}

@class
/*@factory_decl*/of(_ raw: Int) {
  CellNum./*@constructor_call*/new(raw)
}
```

Keep all markers, calls, and the intentionally conflicting `const x: Int` annotation unchanged. `_ raw: Int` is positional; `raw: Int` is labeled and produces `new(raw)`/`of(raw)`, which cannot resolve `new(raw)`/`of(42)` as positional calls.

- [ ] **Step 2: Compile the integration target**

Run:

```bash
cargo test -p phalcom-lsp --test integration constructor_factory_inference_is_authoritative_across_lsp_features --no-run
```

Expected: compile succeeds. Existing unrelated `ModuleQueryFacade` unused-import warning may remain.

---

### Task 2: Show advisory shape beside formal `Unknown`

**Files:**
- Modify: `phalcom-lsp/src/hover.rs:682-713`
- Test: `phalcom-lsp/src/hover.rs` tests near `render_binding_hover_keeps_formal_dynamic_state_authoritative`

**Interfaces:** `render_binding_hover_with_formal(binding, formal, value, phaldoc)` already receives both formal and advisory products. Extend rendering policy without changing its signature.

- [ ] **Step 1: Add the failing unit regression**

Add this test beside the existing formal-dynamic regression:

```rust
#[test]
fn render_binding_hover_shows_observed_shape_when_formal_unknown() {
    let uri = tower_lsp::lsp_types::Url::parse("file:///main.ph").unwrap();
    let document = crate::documents::Document::new("let value = 1\n".to_string());
    let db = crate::semantic::SemanticDb::new();
    db.update_file(&uri, crate::semantic::FileRevision(1), &document.parse.program);
    let binding = db
        .snapshot()
        .files
        .values()
        .next()
        .and_then(|file| file.source.scopes.bindings.values().next())
        .expect("binding");
    let observed = InferredValue::flow(
        ValueShape::Instance(crate::semantic::ClassId::new(crate::semantic::ModuleId::new("file:///factory.ph"), "CellNum")),
        (0..0).into(),
    );
    let rendered = render_binding_hover_with_formal(binding, Some(&FormalPresentation::Unknown), Some(&observed), None);
    assert!(rendered.contains("**Formal type:** `Unknown`"));
    assert!(rendered.contains("**Observed type:** `≈ CellNum`"));
}
```

Run:

```bash
cargo test -p phalcom-lsp --lib render_binding_hover_shows_observed_shape_when_formal_unknown
```

Expected before implementation: FAIL because the current formal branch suppresses observed rendering.

- [ ] **Step 2: Implement the narrow rendering policy**

Keep the existing formal section. Inside the `if let Some(formal)` branch, append the existing observed-shape rendering block only when `matches!(formal, FormalPresentation::Unknown)` and the value is non-`Unknown` with non-heuristic confidence:

```rust
if let Some(formal) = formal {
    sections.push(format!("**Formal type:** `{}`", formal.text()));
    if matches!(formal, FormalPresentation::Unknown)
        && let Some(value) = value.filter(|value| !matches!(value.shape, ValueShape::Unknown) && value.confidence != Confidence::Heuristic)
    {
        sections.push(format!(
            "**Observed type:** `≈ {}`\n\nConfidence: {}",
            crate::semantic::render_value_shape(&value.shape),
            crate::semantic::confidence_name(value.confidence)
        ));
    }
} else if let Some(value) = value.filter(|value| !matches!(value.shape, ValueShape::Unknown) && value.confidence != Confidence::Heuristic) {
    // Keep current advisory-only rendering unchanged.
}
```

Do not add observed rendering for `Known`, `Dynamic`, `Invalid`, `Blocked`, `Cancelled`, `BudgetExceeded`, `InternalFailure`, or `Partial` in this scoped change.

- [ ] **Step 3: Run renderer regressions**

Run:

```bash
cargo test -p phalcom-lsp --lib render_binding_hover
```

Expected: new `Unknown` test passes; `render_binding_hover_keeps_formal_dynamic_state_authoritative` still passes and still omits observed type.

---

### Task 3: Verify all composition assertions

**Files:**
- Test: `phalcom-lsp/tests/composition1.rs::constructor_factory_inference_is_authoritative_across_lsp_features`
- Check: `phalcom-lsp/src/hover.rs`, `phalcom-lsp/tests/composition1.rs`

**Interfaces:** Tasks 1–2 provide canonical `new(_)`/`of(_)` resolution and formal-plus-observed binding hover.

- [ ] **Step 1: Run focused composition test**

```bash
cargo test -p phalcom-lsp --test integration constructor_factory_inference_is_authoritative_across_lsp_features -- --nocapture
```

Expected: one pass. Factory hover and definition resolve to class-side `of(_)`; constructor definition resolves to `new(_)`; `x` hover includes observed `CellNum`; completion includes `value()`; `x`, `Int`, and virtual `phalcom/sourceText` navigation remain independent.

- [ ] **Step 2: Run full LSP integration target**

```bash
cargo test -p phalcom-lsp --test integration --no-fail-fast
```

Expected: existing integration coverage remains green; no unrelated semantic/workspace changes are needed.

- [ ] **Step 3: Run scoped format and diff checks**

```bash
rustfmt --edition 2024 --check phalcom-lsp/src/hover.rs phalcom-lsp/tests/composition1.rs
git diff --check -- phalcom-lsp/src/hover.rs phalcom-lsp/tests/composition1.rs
```

Expected: no formatting or whitespace errors. Leave unrelated worktree changes untouched.

## Self-review checklist

- Fixture syntax, not production dispatch, fixes the initial `of(_)` lookup failure.
- Formal `Unknown` remains visible; advisory `CellNum` is not promoted to formal type.
- Formal `Dynamic` precedence remains covered by the existing regression.
- No semantic database owner, callable solver, dispatch identity, or runtime behavior changes.
- Focused and full LSP gates are explicit; no unauthorized commit or cleanup is included.
