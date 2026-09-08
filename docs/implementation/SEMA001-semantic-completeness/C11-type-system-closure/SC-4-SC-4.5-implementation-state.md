# SC-4 / SC-4.5 Implementation State

## Baseline
- Plan baseline: `2b6f28a943d9a76ca33f66763b6a1d391c623424`.
- Execution baseline: `a37664e17e5e9f31378b7d497e51ad349d5ba905` (`main`).
- Working-tree note: pre-existing modifications and untracked files are preserved; this state file is SC-4 work.

## Established invariants
- I-001: `phalcom-semantic` is the current static type authority; existing `InferenceSession` keeps solver variables outside `TypeStore`.
- I-002: SC-4 execution scope is C0-C12 / Tasks 1-52; C0-C12 are certified.
- I-003: SC-3 focused implementation is complete and consumable: canonical Record rows, separate row solving, publication, and incremental gates are green; unrelated repository-wide delivery failures remain deferred from SC-3.
- I-004: Ordinary value subtyping is restricted to canonical `KindId::TYPE` forms; unsaturated constructors and type lambdas cannot inherit `Never`/`Object` value relations.
- I-005: Bidirectional expected typing is consumed only at structural components and result-producing canonical application boundaries; receivers, operands, scrutinees, and assignment/unit paths remain synthesis-directed.
- I-006: Empty collection contextual selection publishes `Assumed` evidence; arbitrary applied type arguments do not become collection/map element expectations without canonical origin identity.
- I-007: Ordinary membership is RHS-owned `contains(_)` application; lifted type membership and negation publish intrinsic `Bool` through explicit semantic paths.
- I-008: Opaque call boundaries preserve immutable local facts, remove mutable alias-sensitive predicates, and drop concrete instance-field current knowledge without rewriting field contracts or initialization.
- I-009: Generic inference reconciles recorded lower/upper bounds after argument and equivalence substitutions; early variable binding cannot bypass declaration constraints.
- I-010: Family denotations retain structural members and behavioral invocation targets through local bindings; both structural and behavioral applications publish exact target selections.
- I-011: Formal expression, binding, return, and presentation products preserve Ready, Invalid, Blocked, Dynamic, Cancelled, BudgetExceeded, and InternalFailure as distinct states; terminal incremental outcomes do not replace last-known-good snapshots.
- I-012: Durable export accepts only canonical TypeStore forms and rejects solver-local inference variables; cold/incremental parity compares stable denotation, identity, status, and origin rather than arena-local TypeIds.

## Decisions
- D-001: Continue the SC-4.5 handoff in checkpoint slices; C0-C6 history remains immutable while C7-C12 close the whole-language typing boundary.
- D-002: Adapt plan mechanics to current repository APIs after source and test inspection.
- D-003: C7 treats SC-3 focused implementation evidence as the dependency gate; unrelated SC-3 repository release gates remain deferred and are not absorbed into SC-4.5.
- D-004: Keep canonical relation authority in `types/relation.rs`; add a proper-kind domain guard rather than teaching individual relation arms about constructor/lambda exclusions.
- D-005: Preserve explicit `UncheckedExpression` only for structural-shape failure, associated/family resolution failure, intentional unsupported comparison links, unreachable match branches, iteration-argument unavailability, and compiler-internal Ellipsis/ImplementationSelector boundaries; no supported ordinary expression variant falls through an unclassified wildcard.
- D-006: Centralize conservative call invalidation at resolved, union-resolved, and unresolved application completion boundaries; structured control inliners remain explicit flow analyses rather than opaque sends.
- D-007: Callable variant patterns use canonical method selectors only; same-named singleton getters remain outside callable-family coverage.
- D-008: List pattern elimination is enabled only for canonical universe `List<T>`; exact empty and non-empty-rest partitions are represented structurally, while unsupported lengths remain conservative.
- D-009: Nested GADT proofs are merged into the current match branch product only when canonical proof merging succeeds; no equality is written to global generic state.
- D-010: Bound constraints are validated after every inference reconciliation pass, including substitutions introduced by source arguments and equality constraints; the declaration-owned bound remains diagnostic authority.
- D-011: Captured structural and behavioral Families share one target-extraction funnel; structural type interning does not collapse declaration denotation or dispatch-side identity.
- D-012: C12 closes supported whole-language type consumption; staged/gated non-type-system coverage and explicitly blocked K09 remain classified in the ledger rather than being promoted by broad green suites.

## Evidence ledger
| Checkpoint | Command | Result | Proves |
|---|---|---|---|
| C0 | `cargo test -p phalcom-ast --test integration parser::parse_compact_type_lambda_as_generic_argument -- --nocapture` | PASS (1) | compact type-lambda syntax remains available |
| C0 | `RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic semantic::foundations::inference -- --nocapture` | PASS baseline (22); RED tests fail as expected (2) | existing inference behavior and genuine non-suffix constructor gaps |
| C0 | `RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic semantic::capabilities::higher_kinded_generics -- --nocapture` | PASS (1) | expected-result HKT selection already exists |
| C0 | `RUSTFLAGS='' cargo test -p phalcom-core --test core monads:: -- --nocapture` | PASS (35, 65.19s) | protected MON baseline |
| C0 | `RUSTFLAGS='' cargo test -p phalcom-core --test core either:: -- --nocapture` | PASS (27, 58.58s) | protected Either baseline |
| C1 | `RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic semantic::capabilities::higher_kinded_generics -- --nocapture` | PASS (2) | nested generic calls share caller context and contextual result resolves |
| C1 | `RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic semantic::capabilities::higher_order -- --nocapture` | PASS (5) | existing higher-order call behavior remains green |
| C1 | `RUSTFLAGS='' cargo test -p phalcom-semantic --lib checker::inference::tests::nested_frames_share_allocator_without_reusing_variables -- --nocapture` | PASS (1) | child frames share one allocator without variable reuse |
| C2 | `RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic semantic::foundations::inference -- --nocapture` | PASS (26) | structural constructor alignment, binary/higher-order kinds, non-suffix and middle-hole reconstruction |
| C2 | `RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic semantic::capabilities::higher_kinded_generics -- --nocapture` | PASS (2) | HKT source behavior and nested contextual result remain green |
| C2 | `RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic semantic::capabilities::type_lambdas -- --nocapture` | PASS (1) | canonical lambda candidate publication remains green |
| C2 | `RUSTFLAGS='' cargo test -p phalcom-core --test core monads:: -- --nocapture` | PASS (35, 55.50s) | protected MON constructor, rejection, runtime, and inheritance gates |
| C3 | `RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic semantic::capabilities::higher_kinded_generics -- --nocapture` | PASS (3) | expected-result HKT selection, nested symbolic result propagation, and conflict retention |
| C3 | `RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic semantic::capabilities::generics -- --nocapture` | PASS (15) | ordinary generic evidence/context policy remains green |
| C3 | `RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic semantic::foundations::generic_application -- --nocapture` | PASS (7) | expected-result and generic constraint behavior remains green |
| C4 | `cargo test -p phalcom-semantic --test semantic semantic::capabilities::higher_kinded_generics -- --nocapture` | PASS (10) | HKT declaration restrictions, F-bounds, transformed generic supertypes, and direct/multi-hop `Self` specialization |
| C4 | `cargo test -p phalcom-semantic --lib checker::inference::tests:: -- --nocapture` | PASS (10) | inference/canonical variance direction parity, callable polarity, frame ownership, and constructor solving |
| C4 | `cargo test -p phalcom-core --test core monads::inheritance:: -- --nocapture` | PASS (3) | protected MON generic hierarchy and higher-kinded inheritance specialization |
| C5 | `cargo test -p phalcom-ast --test integration getter -- --nocapture` | PASS (7) | generic class/enum getter syntax, `where` clauses, variance rejection, and ordinary getter parser behavior |
| C5 | `cargo test -p phalcom-semantic --test semantic semantic::capabilities::getters -- --nocapture` | PASS (9) | contextual/no-context generic getters, constraints, inherited/transformed receivers, `F<Self>`, enum getters, and callable ownership |
| C5 | `cargo test -p phalcom-semantic --test semantic zero_argument_iteration_getter -- --nocapture` | PASS (1) | ordinary zero-argument getter application remains intact |
| C6 | `cargo test -p phalcom-semantic --test semantic semantic::foundations::union_calls -- --nocapture` | PASS (8) | union HKT result selection, per-arm generic joins, single callback analysis, nested callback inference, and missing-arm policy |
| C6 | `cargo test -p phalcom-semantic --test semantic semantic::adts -- --nocapture` | PASS (235); 31 ignored | constructor, enum, variant, GADT, and exact-case executable parity |
| C6 | `cargo test -p phalcom-semantic --test semantic semantic::families -- --nocapture` | PASS (11) | retained family targets use canonical generic application and failure remains fail-closed |
| C6 | `cargo test -p phalcom-semantic --test semantic semantic::incremental -- --nocapture` | PASS (118); 4 ignored | generic dependency invalidation, cold/incremental equivalence, and stable product reuse |
| C6 | `cargo test -p phalcom-core --test core monads:: -- --nocapture` | PASS (35) | protected MON constructor, HKT, inheritance, rejection, and runtime gates |
| C6 | `cargo test -p phalcom-core --test core either:: -- --nocapture` | PASS (27) | protected Either nested, higher-order, rejection, and runtime gates |
| C6 | `cargo test -p phalcom-semantic --test semantic` | PASS (1026); 48 ignored | full semantic package integration gate |
| C7 | `RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic semantic::advanced -- --nocapture` | PASS (37) | SC-3 Record-row behavior plus canonical relation matrix, constructor/lambda domain, exact-case, and callable shape coverage |
| C7 | `RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic semantic::foundations::record_row -- --nocapture` | PASS (5) | row/type domain separation and canonical row materialization |
| C7 | `RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic semantic::integration::record_row_polymorphism -- --nocapture` | PASS (9) | open-row source formation, calls, expected results, diagnostics, and publication |
| C7 | `RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic semantic::incremental::record_rows -- --nocapture` | PASS (6) | row signature invalidation and cold/incremental equivalence |
| C8 | `RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic semantic::foundations::bidirectional_calls -- --nocapture` | PASS (5) | canonical expected-type context ownership and collection-origin guard |
| C8 | `RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic semantic::foundations::expression_composition -- --nocapture` | PASS (16) | nested structural expected propagation and causal composition |
| C8 | `RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic semantic::foundations::canonical_call_application -- --nocapture` | PASS (35) | expected result forwarding through calls, operators, getters, and index paths |
| C8 | `RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic semantic::foundations::expression_engine -- --nocapture` | PASS (10) | expression dispatch and existing intrinsic/control-flow typing |
| C8 | `RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic semantic::capabilities::structural -- --nocapture` | PASS (11) | tuple/product contextual components, collection shapes, rows, membership, and type membership |
| C8 | `RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic semantic::capabilities::getters -- --nocapture` | PASS (9) | generic getter result context remains canonical and callable-owned |
| C9 | `RUSTFLAGS='' cargo test -p phalcom-semantic --lib checker::flow::state::field_tests -- --nocapture` | PASS (3) | field contract/current separation and opaque-call invalidation lattice |
| C9 | `RUSTFLAGS='' cargo test -p phalcom-semantic --lib checker::loop_analysis::tests -- --nocapture` | PASS (3) | preheader/backedge fixed-point convergence, break exclusion, and no-progress detection |
| C9 | `RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic semantic::capabilities::bindings -- --nocapture` | PASS (8) | broad contract persistence across narrow initialization and later refuted writes |
| C9 | `RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic semantic::capabilities::flow_branches -- --nocapture` | PASS (28) | reachable branch joins, abrupt-arm exclusion, nested shadowing, and causal preservation |
| C9 | `RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic semantic::capabilities::flow_loops -- --nocapture` | PASS (14) | loop preheader/body/exit facts, break/continue ownership, field invalidation, and field contracts |
| C9 | `RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic semantic::capabilities::callable_publication -- --nocapture` | PASS (13) | closure construction isolation and nested capture identity |
| C9 | `RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic semantic::capabilities::deep_regressions -- --nocapture` | PASS (5) | flow provenance, exact unknown causes, recovery unions, and closure isolation |
| C9 | `RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic semantic::capabilities::control_regions -- --nocapture` | PASS (10) | executable-region normal/abrupt classification and nested flow scope |
| C9 | `RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic semantic::foundations::flow_graph -- --nocapture` | PASS (12) | predicate invalidation, branch intersection, loop graph structure, and conservative joins |
| C9 | `RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic semantic::foundations::field_flow -- --nocapture` | PASS (2) | field validity/current joins remain conservative |
| C10 | `RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic adts::matching -- --nocapture` | PASS (161); 18 ignored | canonical variant resolution, callable-family residuals, tuple/list partitions, GADT branch proofs, exact-case impossibility, witnesses, and match-flow integration |
| C11 | `RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic capabilities::constraints -- --nocapture` | PASS (3) | source class/method constraint ownership, equality/lower/upper bounds, diagnostics, and generic superclass substitution |
| C11 | `RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic capabilities::variance -- --nocapture` | PASS (4) | source covariance/contravariance/invariance, transformed superclass relation, callable polarity, and source binding relation |
| C11 | `RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic capabilities::aliases -- --nocapture` | PASS (2) | transparent non-generic/generic/nested/type-lambda aliases, alpha/beta normalization, inference, and cycle quarantine |
| C11 | `RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic capabilities::self_types -- --nocapture` | PASS (1) | constructor `Self` specialization and counterexample |
| C11 | `RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic integration::self_formation -- --nocapture` | PASS (2) | source `Self` owner and dispatch-side publication plus instance-only ambient generic scope |
| C11 | `RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic families::invocation -- --nocapture` | PASS (9) | source structural/behavioral Family capture, storage, exact targets, wrong-shape diagnostics, generic specialization, and distinct denotations |
| C11 | `RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic families::values -- --nocapture` | PASS (1) | variant Family exact authorized member shapes |
| C11 | `RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic integration::family_capabilities -- --nocapture` | PASS (1) | Family denotation survives local flow storage |
| C11 | `RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic integration::workspace -- --nocapture` | PASS (17) | source kind/publication, alias workspace behavior, generic signatures, and workspace identity |
| C11 | `RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic incremental::type_store_revisions -- --nocapture` | PASS (12) | alias, constraint, kind, store identity, retained denotation, and cold/incremental revision behavior |
| C12 | `RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic semantic::foundations -- --nocapture` | PASS (264) | canonical relation, inference, authority, publication, structural, and type-formation foundations |
| C12 | `RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic semantic::capabilities -- --nocapture` | PASS (218) | source constraints, variance, aliases, generic getters, Families, flow, and callable capabilities |
| C12 | `RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic semantic::advanced -- --nocapture` | PASS (37) | canonical relation matrix, Record rows, effects/control, and termination integration |
| C12 | `RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic semantic::adts -- --nocapture` | PASS (239); 27 ignored | ADT/GADT declaration, constructor, exact-case, Family, match, and residual semantics |
| C12 | `RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic semantic::families -- --nocapture` | PASS (14) | structural/behavioral Family capture, storage, invocation, targets, and wrong-shape failure |
| C12 | `RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic semantic::incremental -- --nocapture` | PASS (119); 4 ignored | cold/incremental generic getter, alias, constraint, kind, Record-row, and dependency parity |
| C12 | `RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic semantic::integration -- --nocapture` | PASS (116) | formal/public presentation states, metadata boundaries, workspace products, and source integration |
| C12 | `RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic` | PASS (1041); 44 ignored | full semantic integration binary after all closure changes |
| C12 | `RUSTFLAGS='' cargo test -p phalcom-core --test core monads:: -- --nocapture` | PASS (35) | protected MON higher-kinded, inheritance, rejection, and runtime gates |
| C12 | `RUSTFLAGS='' cargo test -p phalcom-core --test core either:: -- --nocapture` | PASS (27) | protected Either inference, higher-order, rejection, substitution, and runtime gates |

## Negative/deletion gates
| Checkpoint | Search/assertion | Result | Proves |
|---|---|---|---|
| C0 | `InferenceSession::new()` and `ExpectedType::Inference` ownership inspection | per-call session and context-free expectations confirmed | C1 architectural gaps are real |
| C0 | getter rejection search | production rejection remains in class and enum parser paths | C5 work is still required |
| C1 | `InferenceSession::new()` production search | only context creation and row-inference standalone session remain; generic call path no longer allocates per-call session | C1 allocator ownership is centralized |
| C2 | canonical constructor view and lambda kind checks | no string-based identity, suffix-only helper removed, candidate lambda validated against formal `KindId` | C2 constructor abstraction uses canonical TypeStore/TypeLambdaArena authority |
| C3 | `ExpectedType::Inference` context equality guard and durable publication inspection | foreign symbolic terms are not inserted; symbolic result exists only in `TypedExpression`/call boundary, not `ExpressionAnalysis` | context cannot overwrite or leak solver-local evidence |
| C4 | HKT source tests and inference parity unit tests | declaration restrictions remain admissibility-only; variance uses declaration metadata; generic supertype projection uses `TypeHierarchy::supertype_template`; no getter/full-workspace gate run | C4 semantic boundary remains scoped |
| C5 | getter rejection search and `git diff --check` on C5 files | getter parser rejection removed; generated getter literals updated; no whitespace errors | getter syntax/publication changes stay within ordinary selector/call boundaries |
| C6 | `InferenceSession::new()` search | only context manager, row-inference standalone support, and inference unit tests remain; no nested generic call allocation | solver ownership remains query-local and centralized |
| C6 | `ExpectedType::Inference` search | all production matches preserve or inspect owning context; foreign symbolic contexts remain rejected | inference expectations cannot cross query ownership boundaries |
| C6 | getter rejection / `Object` fallback searches | getter rejection has zero hits; only unrelated pattern-space structural fallback remains | no SC-4 generic getter rejection or constructor/variant Object fallback remains |
| C6 | `apply_resolved_callable(...ExpectedType::None)` search | remaining hits are binary/unary/index/setter/iteration synthesis paths; getter path passes caller expectation | generic getter contextual application is not hardcoded to synthesis mode |
| C6 | `graphify update .` | PASS; graph rebuilt with 53,206 nodes and 80,097 edges | code graph reflects final SC-4 source relationships |
| C7 | `rg -n 'RecordAccess' phalcom-semantic/src` | PASS; zero hits | SC-3 Record relation has no obsolete production authority |
| C7 | relation kind-domain audit | PASS; non-`Type` constructor/lambda relations are refuted except identity | constructors are not runtime value subtypes |
| C8 | `rg -n 'ExpectedType::None' phalcom-semantic/src/checker` | PASS; every site is an explicit receiver/operand/scrutinee/statement/control-flow or assignment/unit boundary, with result expectations forwarded where supported | no supported result-producing path silently discards caller context |
| C8 | `rg -n 'UnknownReason::UncheckedExpression' phalcom-semantic/src/checker` | PASS; remaining sites match D-005 classifications; dispatcher has explicit Ellipsis and ImplementationSelector arms | supported expression dispatch has no unclassified `UncheckedExpression` fallthrough |
| C9 | `invalidate_opaque_calls` call-site audit | PASS; resolved, union-resolved, and unresolved application completions invoke one flow invalidation boundary; structured branches/loops retain explicit control semantics | opaque calls cannot preserve alias-sensitive field/predicate refinements accidentally |
| C9 | flow contract/current and loop fixed-point audit | PASS; contracts remain in `BindingState`/`FieldState.contract`, joins exclude unreachable states, loop key strips versions, and exhaustion weakens unstable facts | C9 flow state cannot launder current evidence into persistent contracts or claim non-progress |
| C10 | selector/proof/list-domain audit | PASS; callable patterns require `SelectorKind::Method`, nested child proofs are branch-product inputs, list algebra recognizes canonical universe `List<T>` only, and unsupported exact-length residuals remain conservative | ADT/GADT elimination does not collapse constructor identity, leak branch equalities, or widen arbitrary object domains |
| C11 | generic bound reconciliation audit | PASS; post-reconciliation validation checks both recorded lower and upper bounds against every solved substitution, preserving declaration-owned diagnostic provenance | source argument/equality substitutions cannot bypass generic `where` constraints |
| C11 | Family denotation/target audit | PASS; `captured_family_target` handles structural and behavioral denotations, and family application publication accepts both forms without a duplicate solver | local Family binding preserves denotation, member shape, and dispatch target |
| C11 | `graphify update .` | PASS; graph rebuilt with 53,252 nodes, 80,255 edges, and 4,034 communities | code graph reflects C11 source and semantic closure changes |
| C12 | epistemic/publication state matrix | PASS; presentation tests distinguish Ready, Invalid, Blocked, Dynamic, Cancelled, BudgetExceeded, and InternalFailure; relation and incremental tests retain terminal outcomes | unavailable/failure states are not collapsed into Unknown, Object, Dynamic, or Ready |
| C12 | solver-local public-boundary audit | PASS; export keeps `InferenceVariable` as an explicit non-exportable error, while no inference context/frame/fact/term or row variable occurs in durable metadata products | solver-local state cannot leak into exported canonical type products |
| C12 | generic getter cold/incremental parity | PASS; getter return/`where` edit recomputes declaration surface and matches cold formatted type/status | query-path invariant generic getter semantics and dependency ownership |
| C12 | obsolete-authority deletion audit | PASS; `LocalConstraintSolver`, `TypeData::Infer`, getter prohibition, `RecordAccess`, and forbidden call/associated `Object` fallback searches are empty; remaining `InferenceSession::new` hits are documented root/context, row standalone, or unit-test paths; no hidden cutoff | one canonical solver/type relation authority remains |
| C12 | `graphify update .` | PASS; graph rebuilt with 53,253 nodes, 80,257 edges, and 4,032 communities | code graph reflects final C12 source relationships |

## Deferred gates
- Workspace format/check/test/clippy -> SC-4 completion delivery gate as scoped by the handoff.
- `cargo fmt --all -- --check` remains non-clean because repository-wide pre-existing formatting drift is outside SC-4 ownership; no broad formatting rewrite was applied.
- SC-3 repository release gates (unrelated core/LSP/REPL failures, broad format drift, and existing Clippy diagnostics) remain deferred; SC-3 focused dependency evidence is complete.

## Active incident
None. C0 constructor RED cases are green after C2; no incident recorded.

## Next resume action
SC-4.5 — COMPLETE at C12. Remaining action: run/record final workspace delivery gates only; preserve explicit non-type-system staged/gated ledger rows and blocked K09.
