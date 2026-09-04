# Code Review — Part 05.1: Match Surface, Pattern Semantics, Exhaustiveness & GADT Proofs

**Reviewed files**
- [`token.rs`](file:///Users/altunhasanli/dev/phalcom/phalcom/phalcom-ast/src/token.rs) — `Token::Match`, `Token::FatArrow`
- [`selector.rs`](file:///Users/altunhasanli/dev/phalcom/phalcom/phalcom-ast/src/selector.rs) — `selector_from_exact_variant_pattern`, `selector_pattern_from_variant_pattern`
- [`match_semantics.rs`](file:///Users/altunhasanli/dev/phalcom/phalcom/phalcom-semantic/src/match_semantics.rs) — public product types
- [`pattern_space.rs`](file:///Users/altunhasanli/dev/phalcom/phalcom/phalcom-semantic/src/checker/pattern_space.rs) — internal value-space algebra
- [`exhaustiveness.rs`](file:///Users/altunhasanli/dev/phalcom/phalcom/phalcom-semantic/src/checker/exhaustiveness.rs) — arm usefulness loop
- [`pattern.rs`](file:///Users/altunhasanli/dev/phalcom/phalcom/phalcom-semantic/src/checker/pattern.rs) — pattern resolver
- [`gadt_proof.rs`](file:///Users/altunhasanli/dev/phalcom/phalcom/phalcom-semantic/src/checker/gadt_proof.rs) — GADT branch solver
- [`expression.rs` (L2615–2761)](file:///Users/altunhasanli/dev/phalcom/phalcom/phalcom-semantic/src/checker/expression.rs#L2615-L2761) — `synthesize_match_expr` orchestrator
- [`match_syntax.rs`](file:///Users/altunhasanli/dev/phalcom/phalcom/phalcom-ast/tests/match_syntax.rs) — AST parser tests

---

## Overall Assessment

**Verdict: Strong foundation, production-ready for the semantic pipeline. Several correctness gaps and one spec divergence require attention before Part 05.2 lowering consumes the products.**

The architecture honours every hard constraint from the spec: no second variant resolver in lowering, correct `PatternSpace` separate from `FlowState`, `SelectorPattern::matches` reused as the sole selector compatibility authority, and stable `MatchResolutionIndex` published onto `CallableAnalysis`. The algebraic core (`normalize`, `intersect`, `subtract`) is well-structured and the GADT refutation path is correct in the nominal case.

---

## Findings

### 🔴 Critical

---

#### C1 — Or-pattern bindings not validated for name consistency
**File:** [`pattern.rs` L66–82](file:///Users/altunhasanli/dev/phalcom/phalcom/phalcom-semantic/src/checker/pattern.rs#L66-L82)

The spec mandates that every alternative in an or-pattern introduces the same binding names, with types joined. The implementation loops over alternatives and accumulates `alt_space` but does **not** track or cross-check binding names across alternatives. Alternatives that bind different names silently produce different `PatternBindingResolution` lists with no diagnostic. This is a semantic correctness gap that Part 05.2 will rely on.

```rust
// Current: each alternative pushes independently into the shared bindings vec
for alt in alternatives {
    let (alt_res, s) = resolve_pattern(ctx, alt, expected_ty, expected_space, bindings);
    ...
}
```

**Fix:** Collect bindings per-alternative into separate `Vec`s, compute the intersection of names, emit a diagnostic if any alternative is missing a name that another provides, then join the types for shared names and push only the joined bindings to the parent `bindings` vec.

---

#### C2 — `is_empty` for `Variant` is wrong: any-field vs all-fields semantics
**File:** [`pattern_space.rs` L49](file:///Users/altunhasanli/dev/phalcom/phalcom/phalcom-semantic/src/checker/pattern_space.rs#L49)

```rust
Self::Variant(v) => v.fields.iter().any(Self::is_empty),
```

A `VariantSpace` with payload fields is empty only when **all** fields are empty (the space degenerates to the Cartesian product of field spaces, which is empty if any component is empty). The `any` here is correctly capturing that "if any field space is empty, the whole variant space is empty" — but semantically this means *any empty field makes the variant empty*. This is actually correct for Cartesian product, **however**, `normalize` then short-circuits by calling `is_empty` before normalizing: a `Variant` with a field that happens to be a `Union([])` will incorrectly report itself as empty before `normalize` flattens that union. The order of operations creates a correctness hazard.

**Fix:** Either normalize first then test, or change `is_empty` for `Variant` to recursively normalize field spaces inline before testing, or ensure callers always normalize before calling `is_empty`.

---

#### C3 — GADT refutation compares concrete types but misses the open-type-parameter case
**File:** [`gadt_proof.rs` L55–61](file:///Users/altunhasanli/dev/phalcom/phalcom/phalcom-semantic/src/checker/gadt_proof.rs#L55-L61)

```rust
if case_ty != scrutinee_arg_ty {
    let sub_left = is_subtype(store, hier, case_ty, scrutinee_arg_ty);
    let sub_right = is_subtype(store, hier, scrutinee_arg_ty, case_ty);
    if !sub_left && !sub_right {
        return GadtProofResult::Refuted;
    }
}
```

The spec example: *"a generic `Expr<T>` can match `Expr::Int` because that match establishes `T = Int`"*. When `scrutinee_arg_ty` is a free type parameter (`TypeData::Parameter`), neither subtype direction typically holds, so the code returns `Refuted` — incorrectly silencing valid generic matches. The correct check: if either type is a free type parameter, the constraint is satisfiable; only refute when both types are concrete and structurally incompatible.

---

#### C4 — `resolve_pattern` falls through to `Wildcard` for unhandled pattern forms
**File:** [`pattern.rs` L125](file:///Users/altunhasanli/dev/phalcom/phalcom/phalcom-semantic/src/checker/pattern.rs#L125)

```rust
_ => (PatternResolution::Wildcard, expected_space.clone()),
```

`Pattern::Record` and `Pattern::Map` are silently treated as wildcards instead of producing a diagnostic or a typed resolution. This masks user mistakes and will cause Part 05.2 to receive incorrect product metadata (the record is gone from the semantic product). At minimum this should emit an `Unsupported` diagnostic; better: wire through partial resolution and mark the arm as having an opaque coverage contribution.

---

### 🟡 Moderate

---

#### M1 — `obvious_initializer_text` in `inlay_hints.rs` slices bytes, not chars
**File:** [`inlay_hints.rs` L53–63](file:///Users/altunhasanli/dev/phalcom/phalcom/phalcom-lsp/src/inlay_hints.rs#L53-L63)

> _Note: this file is not part of 05.1 proper but was touched in context._

`text[range.end..]` indexes UTF-8 source by byte offset. If `range.end` falls inside a multi-byte character the slice will panic at runtime. Source offsets are byte offsets, so this is only safe when `range.end` is always a valid char boundary — which is not guaranteed by the type system. Use `text.get(range.end..)` to return `None` gracefully.

---

#### M2 — `normalize` deduplication uses `Vec::contains`, O(n²) for wide union types
**File:** [`pattern_space.rs` L74–77](file:///Users/altunhasanli/dev/phalcom/phalcom/phalcom-semantic/src/checker/pattern_space.rs#L74-L77)

```rust
if !flat.contains(&other) {
    flat.push(other);
}
```

For large enum families this becomes O(n²) per normalization call. During initial space construction (enumerating every variant), normalization is called repeatedly. Switch to an index-based deduplication (`IndexSet` or a `BTreeSet` with a cheaper key) or move deduplication to a single pass after flattening.

---

#### M3 — `selector_pattern_from_variant_pattern` hardcodes `SelectorKind::Method`
**File:** [`selector.rs` L183](file:///Users/altunhasanli/dev/phalcom/phalcom/phalcom-ast/src/selector.rs#L183)

```rust
SelectorKindPattern::Exact(SelectorKind::Method),
```

A callable-pattern like `Dog(...)` is always mapped to `SelectorKind::Method`. Variants declared as subscript or other non-method kinds won't match. Per the spec, the kind should be derived from the variant's existing declared kind (getter for singleton, method for callable). For most current ADT variants this is correct, but it's a latent incompatibility if non-method callable variants are introduced.

---

#### M4 — Witness message uses `{:?}` formatting for user-facing diagnostics
**File:** [`exhaustiveness.rs` L146](file:///Users/altunhasanli/dev/phalcom/phalcom/phalcom-semantic/src/checker/exhaustiveness.rs#L146)

```rust
format!("non-exhaustive match expression: missing cases {:?}", witnesses),
```

`CoverageWitness` debug output is internal `VariantId`-keyed and unreadable to users. The witness renderer should produce Phalcom surface syntax like `Animal::Dog(_)`. This blocks the user from acting on exhaustiveness errors in any meaningful way.

---

#### M5 — `generate_coverage_witnesses` picks only the first field witness per variant
**File:** [`exhaustiveness.rs` L169](file:///Users/altunhasanli/dev/phalcom/phalcom/phalcom-semantic/src/checker/exhaustiveness.rs#L169)

```rust
let w = generate_coverage_witnesses(f).into_iter().next().unwrap_or(CoverageWitness::Wildcard);
```

`.next()` silently drops all witnesses beyond the first for each field position. For a multi-field variant, the generated witness for the whole case is incorrect: it shows at most one example per field rather than the full coverage gap. This affects diagnostic quality.

---

#### M6 — `synthesize_match_expr` leaks pattern bindings into post-match flow
**File:** [`expression.rs` L2646–2649](file:///Users/altunhasanli/dev/phalcom/phalcom/phalcom-semantic/src/checker/expression.rs#L2646-L2649)

```rust
for pattern_binding in &arm_bindings {
    branch_flow.bindings.remove(&pattern_binding.binding);
    branch_flow.facts.invalidate_binding(pattern_binding.binding);
}
```

Bindings introduced in pattern arms are cleaned from the *branch* flow before it is contributed to the join. However, the `push_scope`/`pop_scope` pair already encloses the arm — bindings declared inside should not appear in `branch_flow` after `pop_scope`. The manual removal is either a redundancy (harmless) or a signal that `pop_scope` doesn't reliably clear bindings from the flow snapshot. This should be audited against `pop_scope` semantics to decide which mechanism is authoritative.

---

### 🟢 Positive Observations

- **`SelectorPattern::matches` delegation** is clean: [`matches_selector_constraint`](file:///Users/altunhasanli/dev/phalcom/phalcom/phalcom-semantic/src/checker/pattern.rs#L504-L521) delegates entirely to the canonical selector authority with no second implementation. Spec-compliant.
- **`BranchProofEnvironment` intersection** in `pattern_common_proof` correctly retains only equalities established by *every* alternative — the right semantics for or-patterns.
- **`PatternSpace::subtract` multi-field Cartesian difference** is implemented with the correct `∪` expansion; the algorithm is mathematically sound for product types.
- **Token longest-match ordering** is correct: `=>>` before `=>` before `=` in the operator scanner.
- **Public product types** (`MatchResolution`, `MatchArmResolution`, `ResolvedVariantCandidate`) carry `exact_case: TypeId` — Part 05.2 can lower without re-resolving names, satisfying the hard spec boundary.
- **Test coverage at the AST layer** is good: `match_syntax.rs` covers qualified variants, whole-family `*`, callable gaps, labeled arguments, nested patterns, and or-patterns in a single parse round-trip.

---

## Summary Table

| # | Severity | File | Issue |
|---|----------|------|-------|
| C1 | 🔴 Critical | `pattern.rs:66` | Or-pattern bindings not name-checked across alternatives |
| C2 | 🔴 Critical | `pattern_space.rs:49` | `is_empty` / `normalize` ordering hazard for Variant |
| C3 | 🔴 Critical | `gadt_proof.rs:55` | Free type parameter case incorrectly refuted |
| C4 | 🔴 Critical | `pattern.rs:125` | `Record`/`Map` silently treated as wildcards |
| M1 | 🟡 Moderate | `inlay_hints.rs:54` | Byte-offset UTF-8 slice panics on multi-byte chars |
| M2 | 🟡 Moderate | `pattern_space.rs:74` | O(n²) union deduplication |
| M3 | 🟡 Moderate | `selector.rs:183` | Hardcoded `Method` kind for callable-pattern selector |
| M4 | 🟡 Moderate | `exhaustiveness.rs:146` | Debug-printed witnesses in user-facing diagnostics |
| M5 | 🟡 Moderate | `exhaustiveness.rs:169` | Multi-field witness truncated to first match only |
| M6 | 🟡 Moderate | `expression.rs:2646` | Redundant/conflicting binding removal in post-arm flow |

---

## Recommended Action Before Part 05.2

Fix **C1**, **C3**, **C4** before Part 05.2 starts consuming semantic products — they affect what goes into `MatchResolutionIndex` and will silently propagate incorrect or missing data to the lowering layer. Fix **M4** before any user-facing acceptance test runs. **C2**, **M2**, **M5** are correctness/quality improvements that can be batched in a follow-up.
