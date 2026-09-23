# IMPLEMENTATION INCIDENT

## 1. Identity

- Program: `LANG007`
- Checkpoint: `LANG007.C3`
- Plan: `LANG007.C3.P2`
- Task/Gate: T5/T7 static-term aliases and all-annotation availability; G3/G4
- Repository: `/Users/altunhasanli/dev/phalcom/phalcom`
- Branch: `main`
- Starting revision: `1c9985b6d449d8680667a91651ea64b2808c44ca`
- Current revision: same HEAD, with an uncommitted P2 working diff

## 2. Escalation trigger

Triggers 7, 8, 11, 15, and 16: annotation formation needs the declaration-owned
static-case authority before the current publication phase; moving it changes
the semantic scheduling contract and exact incremental dependencies. A local
store scan or eager special case would create a parallel case-selection path.

## 3. Required invariant

- `Shape::Scalar` is a canonical static term of `StaticDomain(Shape)`; it is
  legal in a static alias and as a heterogeneous generic argument.
- `AssociatedSurface` remains the sole owner/base namespace authority.
- Nominal and associated static domains converge before case selection.
- Alias/header/case dependents must recompute exactly; incremental agrees with
  cold analysis. Store-local constructor IDs are not stable declaration identity.
- Static terms have no runtime invocation target.

These are `INV-P2-04`, `INV-P2-07`, `INV-P2-14`, `INV-P2-17`,
`INV-P2-19`, `INV-P2-20`, and the T7 all-annotation amendment.

## 4. Expected architecture

T5 requires `type DefaultShape: Shape = Shape::Scalar`. T7 requires source
projections and static cases to be available in declaration annotations through
canonical selection and dependency ordering. A checked declaration product owns
case membership; lookup must not infer membership from TypeStore interning
history or source spelling.

## 5. Observed behavior

The new `Tensor<Int, Shape::Scalar>` alias regression fails on its first cold
analysis with `AnnotationUnresolved`: "contextual associated type projection
subject must have conformance evidence". Its header-edit and cold/incremental
assertions are not reached. The existing 12-case `incremental::type_members`
lane passes after the unrelated canonical Universe parser repair.

## 6. Minimal reproduction

```phalcom
type data Shape { Scalar }
data Tensor<T: Type, S: Shape>()
type V: Type = Tensor<Int, Shape::Scalar>
```

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic \
  incremental::type_members::alias_application_tracks_nominal_header_domain_edits_and_matches_cold_analysis -- --nocapture
```

The test must form a valid `V` on the initial source. It currently fails with
`AnnotationUnresolved` on `Shape::Scalar`.

## 7. Relevant code path

```text
parser::parse_type_form
  -> ExactEnumCase for zero-payload Shape::Scalar
session alias lowering (before nominal algebraic product construction)
  -> annotation::resolve_type_form(ExactEnumCase)
  -> resolve_associated_type_projection (trait evidence required)
session nominal algebraic product construction (later)
  -> build_enum_semantics / register_static_declaration
  -> TypeStore static constructors
expression::resolve_static_domain_case (separate expression consumer)
  -> static-domain case dependency and StaticCaseId
```

## 8. Scoped implementation diff

The P2 working diff already contains the Alias/Header/Member graph and retained
reverse invalidation. This session repaired `fiber.ph` parsing, updated stale
semantic test constructors, and added the failing alias/header regression.
No static-case publication or annotation-selection code has been changed.

## 9. Evidence

- `docs/specs/type-level-domains.md` classifies `Shape::Scalar` as a static
  term; the P2 T5 plan explicitly requires a static alias using it.
- `types/annotation.rs` forwards zero-payload `ExactEnumCase` to contextual
  trait-associated projection resolution.
- `session.rs` lowers aliases near line 3216 and builds nominal algebraic
  products near line 4134; `register_static_declaration` interns constructors
  during that later build.
- `checker/expression.rs` has its own static-case selection and exact case-set
  dependency path.
- Focused existing T7 tests: 12 selected, 12 passed. New regression: one
  selected, failed on its initial cold analysis. T4 static-domain inference:
  six selected, six passed.

## 10. Implementer's causal hypothesis

Static case selection cannot work in an alias because case products are not
available when alias formation runs. The zero-payload annotation fallback then
misclassifies the case as a trait-associated projection.

## 11. Alternative hypotheses

The parser could be wrong, but it produces `ExactEnumCase`, which is an existing
representation of nominal `::` cases. A pure resolver dispatch change would
still lack the checked case product during cold alias lowering.

## 12. Attempts already made

No speculative semantic correction was made. The failing test isolates the
cold-formation failure before incremental assertions. The parser and test
fixture corrections restored the test lane but did not alter this behavior.

## 13. Why local authority is insufficient

Making cases available before aliases requires changing declaration-product
ordering and likely the unified dependency graph. The alternatives differ in
identity, validation timing, cycle handling, and invalidation. The plan forbids
an ad hoc early case registry or a second selection authority.

## 14. Decision requested

Should T7 extend the unified declaration dependency graph with checked
static-case/domain products, scheduling their prerequisites before dependent
aliases and headers, and expose one canonical case-selection product to both
annotation and expression consumers? If so, specify how cycles involving
static-case payload annotations and aliases are classified, and whether
singleton case identity may be published before payload validation.

## 15. Ratified architecture (2026-09-23)

The user ratified the following resolution. This section supersedes the open
decision request above and authorizes implementation in LANG007.C3.P2.

- `StaticDomainCaseCatalogProduct` remains the early structural authority for
  every stable case identity, selector, shape, and source range. Catalogs are
  not signature-graph nodes.
- Add one checked `StaticCaseSignatureProduct` per constructor-shaped case.
  Singleton getters need no signature; every constructor, including `Unit()`,
  requires one before materialization. Invalid payload signatures withhold the
  contract but retain catalog identity.
- Add `StaticCaseSignature(StaticCaseId)` to the existing Alias/Header/Member
  graph. A cycle containing Member or StaticCaseSignature is one deterministic
  `TypeSignatureCycle`; preserve current Alias-only and Header/Alias-only
  behavior. Domain-header recursion remains legal because case signatures
  depend on headers, never vice versa.
- Keep `AssociatedSurface` as the only associated-domain namespace authority,
  the catalog as the only case-membership authority, the signature product as
  constructor-contract authority, and `TypeStore` as canonical interner only.
- Replace expanded named kind syntax with `KindSyntax::StaticDomain { form:
  Box<TypeAnnotation>, ... }`. Resolve the form through the existing type-form
  path, verify its canonical kind is a static domain, and preserve heterogeneous
  owner arguments for forms such as `Container<Int, R>::State`.
- Algebraic declaration aggregates and both annotation/expression consumers
  must consume catalog plus checked signatures; no consumer may rediscover
  membership by scanning `TypeStore` intern history.

Implementation order: static-domain type-form kinds; signature query/product;
graph and SCC integration; unified scheduling; catalog/signature-backed
builders; shared materialization; focused cold/incremental precision coverage.

## 16. Implementation follow-through (2026-09-23)

The ratified slice is implemented in the current working tree and has focused
evidence recorded in `LANG007.C3-CHECKPOINT.md`. In addition to the named gates
above, the continuation fixed the legacy TypeOnly aggregate path that treated a
recursive static-domain payload as an unsaturated proper runtime type, and
redirected the remaining runtime-data inherent-impl associated-domain builder
call through the catalog/signature products.

P2 remains `IN_PROGRESS / PARTIAL / FOCUSED_TESTED`. T4 heterogeneous row
inference, T8 cross-module metadata/import parity, remaining static type-member
convergence, and T9 stabilization are still open. This record closes the
architectural decision request; it does not declare P2 complete.
