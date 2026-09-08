# Phalcom ADT / GADT / Match Testing Scenario Catalog Amendment
## Part 05.1 Code-Review Regression, Correctness, Diagnostic, Performance, and Tooling Coverage

**Purpose:** Amend `phalcom-adt-comprehensive-testing-scenario-catalog.md` with targeted scenarios derived from the Part 05.1 code review. These scenarios exist specifically to prevent the reviewed defects from surviving, recurring, or being hidden by broader conformance tests.

**Companion documents:**
- `phalcom-adt-comprehensive-testing-implementation-plan.md`
- `phalcom-adt-comprehensive-testing-scenario-catalog.md`

**Review areas covered:**
- `phalcom-semantic/src/checker/pattern.rs`
- `phalcom-semantic/src/checker/pattern_space.rs`
- `phalcom-semantic/src/checker/gadt_proof.rs`
- `phalcom-semantic/src/checker/exhaustiveness.rs`
- `phalcom-semantic/src/checker/expression.rs`
- `phalcom-ast/src/selector.rs`
- `phalcom-lsp/src/inlay_hints.rs`

**Status vocabulary:**
- `RED-REVIEW`: expected to expose the reviewed bug until the implementation is fixed.
- `ADD-REGRESSION`: add even if the implementation has already been fixed; this becomes the permanent regression test.
- `STRENGTHEN`: an existing catalog scenario covers the general law but needs additional assertions or a more surgical fixture.
- `GATED`: depends on a language surface that is not currently admitted.

---

# 1. Amendment Policy

The original catalog remains authoritative for broad language-law coverage. This amendment adds narrow tests around reviewed failure mechanisms.

The execution agent must follow these rules:

1. Do not replace an existing broad scenario with a narrow regression scenario. Keep both when they prove different things.
2. A regression test should fail for the reviewed implementation defect, not merely for some downstream symptom.
3. Prefer the smallest fixture that isolates the defect.
4. For semantic bugs, assert the semantic product that was wrong before asserting diagnostics or source acceptance.
5. For diagnostic bugs, assert structured witnesses and machine-readable diagnostic fields before rendered text.
6. For performance regressions, test asymptotic behavior or a sufficiently wide stress fixture without introducing flaky wall-clock thresholds.
7. For LSP safety bugs, test malformed/boundary offsets directly as unit behavior, not through an unrelated integration scenario.
8. Every review-derived scenario must be referenced from `adts/COVERAGE.md` or the appropriate LSP/core coverage document with a note such as `Part 05.1 review C1`.

---

# 2. Review-to-Catalog Impact Matrix

| Review finding | Existing catalog coverage | Required amendment |
|---|---|---|
| C1 — Or-pattern bindings not name-checked | `MATCH-BIND-05`, `MATCH-BIND-06`, `MATCH-BIND-08`, `MATCH-DIAG-10` | Strengthen and add alternative-local binding/product tests |
| C2 — `PatternSpace::is_empty` / normalize ordering hazard | `MATCH-SPACE-01..18` | Add raw/non-normalized variant-space emptiness regressions |
| C3 — Free type parameter incorrectly refuted in GADT solver | `MATCH-GADT-01`, `04`, `05`, `11` | Add direct solver classification tests and generic reachability product assertions |
| C4 — Record/Map silently resolve as Wildcard | General pattern coverage does not catch this | Add unsupported-pattern fail-closed scenarios |
| M1 — UTF-8 unsafe slice in inlay hints | Not in ADT catalog | Add LSP UTF-8 boundary regressions |
| M2 — O(n²) pattern-space union dedup | `MATCH-SPACE-02..04` correctness only | Add wide-union structural/performance regression |
| M3 — callable selector hardcoded to Method | `MATCH-RES-10..13`, associated family scenarios | Add selector-kind projection/unit tests; gate non-method runtime form if syntax unavailable |
| M4 — debug witness rendering | `MATCH-DIAG-14` | Strengthen with user-surface rendering assertions |
| M5 — multi-field witness truncation | `MATCH-EXH-12`, `13`, diagnostic witness coverage | Add Cartesian multi-field witness scenarios |
| M6 — pattern binding cleanup ambiguity | `MATCH-FLOW-09`, `MATCH-GADT-09/10`, `PAT-CTX-*` | Add post-match symbol/flow absence and scope-authority regression tests |

---

# 3. C1 Amendment — Or-Pattern Binding Set and Type Joining

## REVIEW-C1-01 — Different names are rejected before parent bindings are published
**State:** `STRENGTHEN`
**Extends:** `MATCH-BIND-06`, `MATCH-DIAG-10`.

**Fixture:**
```phalcom
enum Either {
    @variant Left(_ value: Int) -> Either
    @variant Right(_ value: String) -> Either
}

class Test {
    inspect(_ value: Either) {
        match value {
            Either::Left(x) | Either::Right(y) => 1
        }
    }
}
```

**Bug caught:** the resolver previously appended alternative bindings into one shared vector without checking that every alternative introduced the same names.

**Required semantic assertions:**
- exact diagnostic code is the or-binding mismatch code used by the repository;
- diagnostic primary range covers the or-pattern or the mismatching alternative according to the final diagnostic contract;
- no branch-visible parent binding named `x`;
- no branch-visible parent binding named `y`;
- neither alternative-local binding is accidentally published into the enclosing `MatchArmResolution.bindings`;
- `PatternResolution::Or` may retain diagnostic/debug alternative resolutions if the product design allows it, but the valid shared-binding product must be absent.

**Why the negative product assertions matter:** a diagnostic alone is insufficient. Part 05.2 must never lower a malformed or-pattern with a half-valid binding table.

**Failure triage:**
1. inspect binding vectors created for each alternative;
2. inspect the parent binding vector before arm body analysis;
3. check whether names are compared before joining types;
4. ensure error recovery does not retain either alternative's private `BindingId`.

---

## REVIEW-C1-02 — Same names across alternatives produce exactly one parent binding
**State:** `STRENGTHEN`
**Extends:** `MATCH-BIND-05`.

**Fixture:**
```phalcom
match value {
    Either::Left(x) | Either::Right(x) => x
}
```

**Required assertions:**
- one and only one arm-visible `BindingId` for `x`;
- `x` type is the canonical join `Int | String`;
- alternative-local temporary binding identities, if created internally, are not independently visible to the arm body;
- the arm expression resolves `x` to the joined parent binding;
- no duplicate-binding diagnostic.

**Regression mechanism:** catches implementations that merely compare names but then push both alternative bindings into the parent list.

---

## REVIEW-C1-03 — Three-way or-pattern requires equality of complete binding sets
**State:** `ADD-REGRESSION`

**Fixture:**
```phalcom
enum Choice {
    @variant A(_ left: Int, right: String) -> Choice
    @variant B(_ left: Int, right: String) -> Choice
    @variant C(_ left: Int) -> Choice
}

match value {
    Choice::A(x, right: y)
    | Choice::B(x, right: y)
    | Choice::C(x) => x
}
```

**Required assertions:**
- mismatch diagnostic identifies missing `y` in the third alternative;
- shared set is not silently reduced to `{x}`;
- no valid parent `x` or `y` bindings are published for the malformed arm.

**Why:** the correct rule is equality of binding sets, not intersection-of-names acceptance.

---

## REVIEW-C1-04 — Same binding set in different syntactic order is valid
**State:** `ADD-REGRESSION`

Use alternatives that bind `x` and `y` in different source traversal order.

**Assert:**
- no mismatch diagnostic;
- parent binding set contains exactly `{x, y}`;
- each parent binding type is joined by name, never by vector position.

**Bug class prevented:** an implementation that fixes C1 by zipping alternative binding vectors positionally instead of keying them by name.

---

## REVIEW-C1-05 — Or-pattern type joining is per name
**State:** `ADD-REGRESSION`

Construct variants so:
- alternative A binds `x: Int`, `y: String`;
- alternative B binds `x: Bool`, `y: Symbol`.

**Assert:**
- `x == Int | Bool`;
- `y == String | Symbol`;
- no cross-wiring such as `x == Int | Symbol`.

---

## REVIEW-C1-06 — Duplicate binding inside one alternative remains distinct from or-binding mismatch
**State:** `STRENGTHEN`
**Extends:** `MATCH-BIND-04`.

Fixture with one alternative binding the same name twice and another alternative valid.

**Assert:**
- duplicate-binding diagnostic is emitted for the malformed alternative;
- the system does not misclassify this only as an or-set mismatch;
- malformed alternative does not poison the valid alternative into a published parent binding.

---

# 4. C2 Amendment — Pattern-Space Emptiness and Normalization Ordering

The code review correctly identifies a subtle distinction: Cartesian-product emptiness uses `any(field.is_empty())`, but raw/non-normalized field spaces can make callers depend on normalization ordering. Tests must therefore distinguish the mathematical law from normalization preconditions.

## REVIEW-C2-01 — Variant with one truly empty normalized field is empty
**State:** `ADD-REGRESSION`

Construct:
```text
Variant(V, fields=[Exact(Int), Empty])
```

**Assert:**
- `is_empty() == true`;
- `normalize() == Empty`.

**Purpose:** locks in the correct Cartesian-product law so a future “fix” does not incorrectly change `any` to `all`.

---

## REVIEW-C2-02 — Variant with no empty fields is non-empty
**State:** `ADD-REGRESSION`

Construct:
```text
Variant(V, fields=[Exact(Int), Exact(String)])
```

Assert non-empty before and after normalization.

---

## REVIEW-C2-03 — Variant containing raw `Union([])` normalizes safely to Empty
**State:** `RED-REVIEW` until fixed if current ordering fails.

Construct:
```text
Variant(V, fields=[Union([]), Exact(Int)])
```

**Assert:**
- normalization terminates;
- normalized result is `Empty`;
- no panic;
- `is_empty(normalize(space)) == true`.

**Important:** also call whichever public/internal path exercises the exact production ordering that triggered the review concern. A test only calling `normalize()` after manually normalizing children does not catch the bug.

---

## REVIEW-C2-04 — Nested raw empty union is detected through recursive normalization
**State:** `ADD-REGRESSION`

Construct:
```text
Variant(
    Outer,
    fields=[
        Variant(Inner, fields=[Union([])])
    ]
)
```

Assert whole space normalizes to `Empty`.

---

## REVIEW-C2-05 — Empty union beside non-empty union in a product remains empty
**State:** `ADD-REGRESSION`

```text
Variant(V, fields=[Union([A,B]), Union([])])
```

Assert `Empty`.

---

## REVIEW-C2-06 — Emptiness invariant property
**State:** `ADD-REGRESSION`

For a table of hand-built spaces, assert:

```text
normalize(S) == Empty  => normalize(S).is_empty()
normalize(S).is_empty() == normalize(normalize(S)).is_empty()
```

Where equality semantics permit, also assert idempotence:

```text
normalize(normalize(S)) == normalize(S)
```

This should include raw nested unions, variants, tuples, and unions of variants.

---

# 5. C3 Amendment — GADT Free-Type-Parameter Compatibility

## REVIEW-C3-01 — Free parameter vs concrete case is satisfiable
**State:** `RED-REVIEW`
**Extends:** `MATCH-GADT-01`.

Test the GADT proof classifier as directly as the test seam allows:

```text
scrutinee: Expr<T>
candidate: Expr<Int>
```

**Assert:** result is `Proven/Compatible` with branch equality `T = Int`, not `Refuted`.

If the public enum is `GadtProofResult::{Proven, Refuted, Blocked}`, assert the exact variant.

---

## REVIEW-C3-02 — Symmetric position of free parameter is satisfiable
**State:** `ADD-REGRESSION`

If the solver can encounter:
```text
case side: Parameter(T)
scrutinee side: Int
```
or equivalent reversed constraint orientation, assert the same satisfiable result.

**Purpose:** prevents a one-sided patch such as “only permit parameter on scrutinee RHS”.

---

## REVIEW-C3-03 — Two distinct free parameters are not refuted merely for being unequal IDs
**State:** `ADD-REGRESSION`

Constraint:
```text
T = U
```

Assert:
- not `Refuted`;
- branch proof records equality/unification according to the current proof model, or `Blocked` if parameter-parameter unification is intentionally deferred;
- never classified impossible solely because `TypeId(T) != TypeId(U)`.

---

## REVIEW-C3-04 — Concrete incompatible types are still refuted
**State:** `ADD-REGRESSION`

`Expr<Int>` candidate `Expr<Bool>`.

Assert:
- `Refuted`;
- candidate becomes `PatternUsefulness::Impossible` when used as an arm;
- it does not contribute to exhaustiveness.

This prevents an over-broad C3 fix that treats every inequality as satisfiable.

---

## REVIEW-C3-05 — Compatible nominal subtype relationship remains admissible if specification permits
**State:** `ADD-REGRESSION`

Create concrete types with a valid subtype relation in a GADT index position if that form is legal.

Assert solver follows the ratified compatibility semantics rather than reducing all concrete inequalities to refutation.

---

## REVIEW-C3-06 — Generic evaluator publishes both candidates and distinct proofs
**State:** `STRENGTHEN`
**Extends:** `MATCH-GADT-01`.

Assert:
- candidate list includes Int and Bool exact `VariantId`s;
- neither candidate is removed during reachable-space construction;
- Int arm proof: `T = Int`;
- Bool arm proof: `T = Bool`;
- branch-local binding `x` specializes accordingly;
- post-match environment contains no `T = Int` or `T = Bool`.

---

## REVIEW-C3-07 — Generic exhaustiveness depends on retaining satisfiable specialized cases
**State:** `ADD-REGRESSION`

Same generic `Expr<T>` evaluator with both arms.

Assert exhaustiveness is `Proven`.

Then remove Bool arm:
- assert non-exhaustive, with Bool case remaining in residual space.

**Why:** catches a solver that incorrectly refutes Bool and thereby falsely proves the one-arm match exhaustive.

---

## REVIEW-C3-08 — Multi-parameter mixed open/concrete GADT
**State:** `ADD-REGRESSION`

Example shape:
```phalcom
enum PairExpr<A, B> {
    @variant Left(_ value: Int) -> PairExpr<Int, B>
    @variant Right(_ value: String) -> PairExpr<A, String>
}
```

Match `PairExpr<X,Y>`.

Assert each candidate can introduce only the equality it constrains while the other parameter remains open.

---

# 6. C4 Amendment — Unsupported Pattern Forms Must Fail Closed

The reviewed implementation's `_ => Wildcard` fallback is dangerous because unsupported syntax can silently cover the entire space and falsify exhaustiveness.

## REVIEW-C4-01 — Record pattern is never silently converted to Wildcard
**State:** `RED-REVIEW`.

Use the smallest parseable `Pattern::Record` in a match.

**Assert one of the following, according to the intended Part 05.1 contract:**
- a dedicated unsupported-pattern diagnostic; or
- a typed record-pattern resolution if record matching is now implemented.

**Forbidden outcomes:**
- `PatternResolution::Wildcard`;
- matched space equals entire expected space without an explicit wildcard;
- match becomes proven exhaustive solely because of this pattern.

---

## REVIEW-C4-02 — Map pattern is never silently converted to Wildcard
**State:** `RED-REVIEW`.

Mirror C4-01 for `Pattern::Map`.

---

## REVIEW-C4-03 — Unsupported record plus missing enum arm does not become exhaustive
**State:** `ADD-REGRESSION`

Construct a closed enum match where an unsupported record-shaped pattern appears before one valid variant arm.

Assert:
- analysis is blocked/rejected;
- no `ExhaustivenessResult::Proven`;
- the unsupported pattern cannot consume residual space as wildcard.

---

## REVIEW-C4-04 — Unsupported map pattern does not make a later arm redundant
**State:** `ADD-REGRESSION`

If the unsupported pattern were treated as wildcard, every later arm would be reported redundant.

Assert:
- later valid arm is not classified redundant *because of* unsupported map coverage;
- analysis instead carries unsupported/blocked state.

---

## REVIEW-C4-05 — No catch-all fallback remains in resolver
**State:** `ADD-REGRESSION / architecture gate`.

Where practical, add a unit-level exhaustive match over `Pattern` variants or an architecture test that forces new pattern enum variants to be consciously handled.

Preferred implementation:
- explicit arms for supported patterns;
- explicit diagnostic-producing arms for known unsupported patterns;
- no semantic `_ => Wildcard`.

Do not make source-text `rg` the only protection, but an architecture search can supplement behavior tests.

---

# 7. M1 Amendment — UTF-8-Safe LSP Inlay Hint Slicing

These tests belong to `phalcom-lsp`, not the semantic ADT subtree, but the review finding must be captured in the testing program.

## REVIEW-M1-01 — Valid UTF-8 boundary works
**State:** `ADD-REGRESSION`.

Call `obvious_initializer_text` (or the smallest unit exposing its slice logic) with source containing multibyte characters before the initializer and a range ending at a valid byte boundary.

Assert expected initializer text.

---

## REVIEW-M1-02 — Interior UTF-8 byte offset never panics
**State:** `RED-REVIEW` if current function directly slices.

Provide text such as:
```text
"const café = 1"
```
and a deliberately invalid `range.end` inside `é`.

Use `catch_unwind` only if necessary to prove old panic behavior; after fix prefer direct:
- result is `None`, or
- graceful non-hint outcome.

**Required invariant:** LSP request path cannot panic.

---

## REVIEW-M1-03 — End-of-source offset
Assert `range.end == text.len()` safely returns the expected empty/no-initializer result.

## REVIEW-M1-04 — Offset beyond source length
Assert graceful `None`, never panic.

## REVIEW-M1-05 — Non-ASCII initializer/body
Use emoji/CJK/combining characters around initializer text and assert extraction is byte-safe and semantically unchanged.

---

# 8. M2 Amendment — Pattern-Space Union Deduplication Scalability

Avoid fragile timing assertions. Test operation counts, structural behavior, or a broad stress input with a generous timeout only if the test harness supports deterministic timeouts.

## REVIEW-M2-01 — Wide union deduplicates correctly
**State:** `ADD-REGRESSION`.

Construct at least hundreds of distinct exact-case spaces plus duplicates.

Assert:
- every unique member appears once;
- deterministic ordering/canonicalization;
- output size equals unique input count.

---

## REVIEW-M2-02 — Duplicate-heavy union
Construct thousands of entries drawn from a much smaller unique set.

Assert correct deduplication and no stack/allocator pathology.

---

## REVIEW-M2-03 — Wide enum initial pattern-space smoke
**State:** `ADD-REGRESSION`.

Generate a source enum with many variants (e.g. 256 or 512) and analyze a complete match or initial-space construction.

Assert:
- analysis completes;
- exact variant count is retained;
- no duplicate candidates.

Do not encode a tight millisecond budget in CI.

---

## REVIEW-M2-04 — Dedup complexity instrumentation
**State:** `ADD-REGRESSION` if a test-only normalization counter seam is acceptable.

Instrument key comparisons/hash insertions in test builds and assert growth is near-linear/log-linear rather than quadratic for doubled input sizes.

If adding instrumentation would pollute production code, omit this scenario and retain wide stress tests plus benchmark coverage.

---

## REVIEW-M2-05 — Criterion/manual benchmark
**State:** optional non-CI benchmark.

Add a benchmark for:
```text
normalize_union_32
normalize_union_128
normalize_union_512
normalize_union_2048
```

Record this as performance observability, not correctness gating.

---

# 9. M3 Amendment — Callable Variant Selector Kind Projection

## REVIEW-M3-01 — Method-kind callable pattern projects Method
**State:** `STRENGTHEN`
**Extends:** `MATCH-RES-10`.

Given an ordinary callable ADT variant, assert `selector_pattern_from_variant_pattern` produces the exact expected `SelectorKindPattern`.

This locks current intended behavior for standard variants.

---

## REVIEW-M3-02 — Getter/singleton remains distinct
**State:** `STRENGTHEN`
**Extends:** `MATCH-RES-02`, `MATCH-RES-14`.

Assert:
- singleton form projects getter-kind selector;
- callable family form does not accidentally include it;
- whole-family form may include it.

---

## REVIEW-M3-03 — Selector helper does not silently invent kind when syntax carries kind information
**State:** `ADD-REGRESSION`.

Unit-test AST selector projection with all currently representable variant pattern forms.

Assert kind comes from pattern/declaration semantics rather than a universal hardcoded value where the AST already distinguishes the form.

---

## REVIEW-M3-04 — Non-method callable variant kind
**State:** `GATED` unless Phalcom currently supports declaring a variant whose callable selector kind is Subscript/operator/other non-method kind.

When supported:
- declare such a variant;
- exact pattern resolves it;
- callable family pattern includes it if its selector shape matches;
- `SelectorPattern::matches` remains sole compatibility authority.

**Important:** do not invent syntax solely to make this test green.

---

# 10. M4 Amendment — User-Facing Coverage Witness Rendering

## REVIEW-M4-01 — Missing singleton renders source syntax
**State:** `STRENGTHEN`
**Extends:** `MATCH-DIAG-14`, `MATCH-EXH-02`.

For missing `Option::None`, assert rendered diagnostic contains human-readable surface spelling such as:
```text
Option::None
```
and does not expose:
```text
VariantId(...)
CoverageWitness { ... }
```

Use the repository's diagnostic presenter; do not assert semantic layer must itself own final ANSI rendering.

---

## REVIEW-M4-02 — Missing payload variant renders wildcard payload
**State:** `ADD-REGRESSION`.

Missing `Option::Some(_)` should render recognizable Phalcom pattern syntax.

Assert structured witness retains exact `VariantId`; presentation maps it to source-level owner/base/shape.

---

## REVIEW-M4-03 — Missing labeled payload renders external labels
**State:** `ADD-REGRESSION`.

For:
```phalcom
@variant Error(_ code: Int, reason message: String)
```

Expected rendered witness should use external selector label (`reason:`), not local storage name if they differ.

---

## REVIEW-M4-04 — Missing nullary vs singleton render differently
**State:** `ADD-REGRESSION`.

For enum containing both:
```text
Dog
Dog()
```

Assert diagnostic renders distinct missing patterns.

---

## REVIEW-M4-05 — No Debug-format leakage
**State:** `ADD-REGRESSION`.

Presentation test scans output for forbidden internal fragments:
```text
VariantId(
TypeId(
CoverageWitness::
CoverageWitness {
```

Keep this narrow so diagnostic implementation can change internal field names without giant golden churn.

---

# 11. M5 Amendment — Multi-Field Coverage Witness Correctness

## REVIEW-M5-01 — Two-field product witness contains both field witnesses
**State:** `RED-REVIEW` if current `.next()` truncation affects it.

Construct a residual:
```text
Pair(field0=A|B, field1=X|Y)
```

Generate witnesses.

**Assert:** every generated `CoverageWitness::Variant` has two field entries.

A witness with only one field is structurally invalid.

---

## REVIEW-M5-02 — Missing combination in two-field variant
**State:** `ADD-REGRESSION`.

Example enum:
```phalcom
enum Pair {
    @variant Both(_ left: Bool, right: Bool) -> Pair
}
```

Use supported Bool/singleton patterns to cover all but one combination.

Assert generated witness represents the complete missing pair, not just the first missing field.

---

## REVIEW-M5-03 — Nested multi-field witness
**State:** `ADD-REGRESSION`.

Variant has two fields and one field is itself a closed ADT.

Assert nested witness tree preserves:
- outer exact `VariantId`;
- exact number/order of fields;
- nested child witness.

---

## REVIEW-M5-04 — Labeled multi-field witness presentation
**State:** `ADD-REGRESSION`.

Structured witness field order maps back to labels correctly in user-facing output.

---

## REVIEW-M5-05 — Witness generation is deterministic
**State:** `ADD-REGRESSION`.

Repeated analysis of same residual gives same witness ordering/canonical representative.

This matters because a Cartesian witness generator can otherwise depend on unstable hash iteration.

---

## REVIEW-M5-06 — Witness representative vs complete residual
**State:** `ADD-REGRESSION`.

Clarify and test the contract:
- semantic residual space remains complete;
- witness list may contain representative missing examples rather than enumerate every possible missing value;
- each representative itself must be structurally valid and complete for all fields.

This prevents tests from accidentally demanding exponential witness enumeration while still catching `.next()` truncation.

---

# 12. M6 Amendment — Pattern Binding Scope and Post-Match Flow Cleanup

The review notes uncertainty over whether manual removal is redundant or compensating for scope-pop behavior. Tests should define the observable law without prematurely prescribing which internal mechanism wins.

## REVIEW-M6-01 — Pattern binding unavailable after match
**State:** `STRENGTHEN`
**Extends:** `MATCH-FLOW-09`.

```phalcom
match value {
    Some(x) => x
    None => 0
}

x
```

Assert post-match reference to `x` is unresolved/out of scope.

---

## REVIEW-M6-02 — Binding absent from joined FlowState
**State:** `ADD-REGRESSION`.

Inspect semantic flow product after the match.

Assert the pattern `BindingId` is absent from:
- joined binding map;
- branch facts;
- stable facts/refinements;
- any post-match visible symbol table used by subsequent expression synthesis.

---

## REVIEW-M6-03 — Branch-local facts are removed with binding
**State:** `ADD-REGRESSION`.

Inside arm, establish additional fact about `x`.

After branch:
- binding absent;
- fact keyed by binding absent.

This catches partial cleanup where symbol disappears but fact arena still contains a live fact.

---

## REVIEW-M6-04 — Outer same-name binding is restored, not deleted
**State:** `ADD-REGRESSION`.

```phalcom
const x = "outer"

match value {
    Some(x) => ...
    None => ...
}

x
```

Assert post-match `x` resolves to the outer binding with String type/identity.

**Bug class prevented:** manual `bindings.remove(pattern_binding.binding)` accidentally interacting with shadowing/scope restoration.

---

## REVIEW-M6-05 — Pattern binding does not leak from one arm into another
**State:** `ADD-REGRESSION`.

First arm binds `x`; second arm body refers to `x` without binding it.

Assert second-arm reference is invalid, even though arms are synthesized sequentially.

---

## REVIEW-M6-06 — Or-pattern joined binding remains only for arm body
**State:** `ADD-REGRESSION`.

Valid `A(x) | B(x)`:
- `x` visible in arm body;
- not visible after match.

This specifically combines C1 parent binding publication with M6 cleanup.

---

## REVIEW-M6-07 — Scope cleanup authority unit regression
**State:** `ADD-REGRESSION`.

At checker/context unit level, construct:
1. outer flow;
2. `push_scope`;
3. introduce pattern binding/fact;
4. snapshot branch;
5. `pop_scope`.

Assert documented context semantics.

The implementation team must decide one authority:
- `pop_scope` removes lexical binding visibility and flow facts; or
- match synthesis explicitly strips pattern bindings from branch-flow snapshots before join.

Tests should encode the chosen invariant and remove redundant compensating behavior if possible.

**Do not assert implementation duplication.** Assert observable state before/after each API boundary.

---

# 13. Cross-Cutting Regression Scenarios

## REVIEW-X-01 — Malformed pattern can never create false exhaustiveness
**State:** `ADD-REGRESSION`.

Table-drive:
- or-binding mismatch;
- unsupported record;
- unsupported map;
- selector no-candidate;
- blocked GADT analysis.

For each:
- match is not `Proven` merely through malformed arm coverage;
- residual is not silently emptied;
- lowering product is absent/rejected.

---

## REVIEW-X-02 — Invalid semantic match never reaches Part 05.2 executable lowering
**State:** `ADD-REGRESSION`.

For representative C1/C4 invalid matches:
- semantic diagnostics exist;
- no accepted/proven `MatchResolution` suitable for lowering, or lowering explicitly rejects it;
- backend does not synthesize wildcard behavior as recovery.

---

## REVIEW-X-03 — Generic GADT theorem survives semantic-to-lowering boundary
**State:** `ADD-REGRESSION`.

After C3 fix:
- semantic product contains both candidates and branch proofs;
- executable projection contains both exact candidate IDs;
- executable projection does **not** contain the proof equalities;
- runtime executes both cases.

This catches a “fix” where semantics becomes correct but projection accidentally sees only one candidate.

---

## REVIEW-X-04 — Witness semantic structure and presentation are tested separately
**State:** `ADD-REGRESSION`.

One non-exhaustive match should have:
1. semantic test for structured `CoverageWitness`;
2. presenter/diagnostic test for surface syntax.

Do not let prettier output hide an invalid witness tree, or vice versa.

---

# 14. AST-Level Regression Additions

The review praised AST match coverage, so preserve it and add only tests that protect selector-kind/fallback boundaries.

## REVIEW-AST-01 — Every currently supported match pattern AST variant round-trips
**State:** `STRENGTHEN`.

Enumerate all supported `Pattern` variants, not only variant patterns.

Assert parser produces the intended discriminant so semantic fallback bugs cannot be blamed on parsing.

## REVIEW-AST-02 — Record/map pattern parse discriminants remain Record/Map
**State:** `ADD-REGRESSION`.

If parseable, assert they do not become wildcard at AST layer.

## REVIEW-AST-03 — Callable family selector constraints
Assert exact AST distinction among:
```text
Dog(...)
Dog(x, ...)
Dog(..., named: y)
Dog(x, ..., named: y)
Dog*
```

---

# 15. Coverage-Ledger Amendments

Add these review-derived rows or notes to `phalcom-semantic/tests/semantic/adts/COVERAGE.md`.

Recommended IDs:

```text
REVIEW-C1-01 .. REVIEW-C1-06
REVIEW-C2-01 .. REVIEW-C2-06
REVIEW-C3-01 .. REVIEW-C3-08
REVIEW-C4-01 .. REVIEW-C4-05
REVIEW-M2-01 .. REVIEW-M2-05
REVIEW-M3-01 .. REVIEW-M3-04
REVIEW-M4-01 .. REVIEW-M4-05
REVIEW-M5-01 .. REVIEW-M5-06
REVIEW-M6-01 .. REVIEW-M6-07
REVIEW-X-01  .. REVIEW-X-04
REVIEW-AST-01 .. REVIEW-AST-03
```

LSP-owned M1 scenarios should live in the LSP test ledger/module instead:

```text
REVIEW-M1-01 .. REVIEW-M1-05
```

For rows that extend an existing catalog law, record both IDs:

```text
MATCH-GADT-01 / REVIEW-C3-06
MATCH-BIND-06 / REVIEW-C1-01
MATCH-DIAG-14 / REVIEW-M4-01
```

Do not mark the broad law `READY` if its review regression remains red.

---

# 16. Suggested Test Placement

## `phalcom-semantic`

```text
tests/semantic/adts/matching/
    bindings.rs
        REVIEW-C1-*
        REVIEW-M6-04..06 where source-driven

    pattern_space.rs
        REVIEW-C2-*
        REVIEW-M2-* correctness/stress

    gadt_refinement.rs
        REVIEW-C3-*

    patterns.rs
        REVIEW-C4-01..05

    exhaustiveness.rs
        REVIEW-M5-*
        semantic side of REVIEW-M4-*

    diagnostics.rs
        presenter-facing semantic diagnostic structure
        REVIEW-M4-*
        REVIEW-X-04 semantic half

    flow.rs
        REVIEW-M6-01..03
        REVIEW-M6-07 where checker seam fits

    conformance.rs
        REVIEW-X-01
        REVIEW-X-03 semantic half
```

## `phalcom-ast`

```text
tests/match_syntax.rs
    REVIEW-AST-01..03

src/selector.rs unit tests or dedicated tests/selector_patterns.rs
    REVIEW-M3-01..03
```

## `phalcom-lsp`

Place under the existing inlay-hint unit/integration module:

```text
REVIEW-M1-01..05
```

## `phalcom-core` after two-binary consolidation

```text
tests/language/compiler/lowering.rs
    REVIEW-X-02
    REVIEW-X-03 lowering half

tests/language/algebraic_data/matching.rs
    REVIEW-X-03 runtime half
```

---

# 17. Execution Priority

Before Part 05.2 lowering is allowed to consume Part 05.1 products, implement and pass:

```text
REVIEW-C1-01
REVIEW-C1-02
REVIEW-C1-03
REVIEW-C3-01
REVIEW-C3-04
REVIEW-C3-06
REVIEW-C3-07
REVIEW-C4-01
REVIEW-C4-02
REVIEW-C4-03
REVIEW-C4-04
REVIEW-X-01
REVIEW-X-02
```

Then complete correctness/diagnostic regressions:

```text
REVIEW-C2-*
REVIEW-M4-*
REVIEW-M5-*
REVIEW-M6-*
```

Then performance/latent compatibility/tooling:

```text
REVIEW-M1-*
REVIEW-M2-*
REVIEW-M3-*
REVIEW-AST-*
```

---

# 18. Review-Specific Debugging Playbook

## Or-pattern failure
Print per-alternative:
```text
binding name
BindingId
TypeKnowledge
source range
```
Then print parent joined binding map.

Expected:
```text
validate names -> join by name -> publish parent bindings
```

Never:
```text
append alt A bindings -> append alt B bindings -> diagnose later
```

## Pattern-space emptiness failure
Print both raw and normalized forms:

```text
raw.is_empty()
normalize(raw)
normalize(raw).is_empty()
```

If they disagree in a way production code relies on, audit call ordering rather than changing Cartesian-product semantics.

## Generic GADT failure
Print each equality:
```text
case type
scrutinee argument type
TypeData of both
is_subtype(left,right)
is_subtype(right,left)
solver classification
```

If either side is an open parameter, inequality of `TypeId`s is not enough to refute.

## Unsupported-pattern failure
Inspect:
```text
AST Pattern discriminant
PatternResolution
matched PatternSpace
diagnostics
ExhaustivenessResult
```

Any path:
```text
Record/Map -> Wildcard -> expected_space
```
is a regression.

## Witness failure
Inspect structured witness before renderer.

For variants:
```text
VariantId
number of witness fields
child witness per field
```

Only after structure is correct inspect surface rendering.

## Binding-leak failure
Print flow/scope states at:
```text
before push_scope
after pattern bind
after arm body
after pop_scope
before branch join
after match join
```

Track by `BindingId`, not source name, because shadowing is part of the test.

---

# 19. Amendment Completion Criteria

This amendment is complete when:

- malformed or-patterns cannot publish inconsistent binding products;
- valid or-patterns publish one joined binding per shared name;
- raw/non-normalized pattern-space states cannot trigger an incorrect emptiness result in production paths;
- free type parameters are never refuted merely because neither subtype direction holds;
- concrete incompatible GADT indices remain refuted;
- unsupported record/map patterns fail closed and cannot act as wildcards;
- UTF-8 source slicing in LSP inlay hints cannot panic;
- wide pattern-space unions have regression/stress coverage against pathological dedup behavior;
- callable variant selector kind behavior is explicitly tested;
- non-exhaustive diagnostics render surface-syntax witnesses without internal debug IDs;
- multi-field witness representatives contain every field position;
- pattern bindings and their facts do not leak across arms or after `match`;
- outer shadowed bindings are restored correctly;
- invalid semantic products cannot reach executable lowering as if proven;
- generic GADT candidate/proof correctness survives projection to Part 05.2 while proof data itself remains erased from runtime IR.
