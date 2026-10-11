# 12. Phase 2 recommendation and evidence

Status: architecture recommendation, 2026-10-07. The inspected revision was
`de9b16576e34ee9bbfa9676cabad12a8fffa68e3`, with ongoing working-tree work.
“Telos” retains the naming assumption in the Phase 1 README; repository paths
and tests belong to Phalcom.

## 12.1 Executive recommendation

Build optional `numerics` consumer modules, `numerics.statistics` and
`numerics.probability`, above the Phase 1 substrate. Keep distribution
evaluation in `numerics.probability`; place bounded
Bayesian updating and Monte Carlo helpers in its public `bayes` and `monte_carlo`
modules. Strengthen the existing `universe.random` package independently.

The shared contracts should be small:

- immutable, validated distribution descriptors;
- an explicit stateful RNG capability;
- independent sampling, mass, density, ordered-tail, and quantile capabilities;
- existing `Iterable`/`Iterator` for observations and sample streams;
- stable concrete moment summaries and model-specific sufficient statistics;
- ordinary binary64 results with named probability/log-domain operations.

Use `Bool` for Bernoulli outcomes, `Int` for counts, and `T` for categorical
labels. Preserve an associated sample type in generic sampling. A distribution
descriptor contains parameters, never the user's RNG. Priors and posteriors are
roles of ordinary distributions. Likelihood is an operation or a callable over
parameters; it is not a bounded probability value.

The recommended V1 includes Float descriptive statistics, mergeable second
moments, probability normalization, nine distribution families, explicit seeded
sampling, Beta–Bernoulli/Binomial and Gamma–Poisson updates, and fixed-budget
Monte Carlo expectation. This is a target boundary, with substantial foundation
dependencies listed in [13](13-phase2-foundation-amendments.md). The current tree
does not implement this package set.

### Reading guide against the Phase 2 deliverable

| Requested design area | Report owner |
|---|---|
| Recommendation, findings, evidence/confidence | 12 |
| Phase 1 requirements/amendments, exact combinatorics | 13 |
| Principles, ontology, type relationships, capability alternatives/extensions | 14 |
| Collection/streaming/weighted statistics, result/NaN/missing policy | 15 |
| Numerical stability, probability/log representation, normalization, errors | 16 |
| Discrete/continuous families, parameters, support, validation, tails/quantiles | 17 |
| Sampling, RNG state, determinism, allocation/stream ownership | 18 |
| Likelihood, Bayesian foundation, conjugacy, sufficient statistics, intervals | 19 |
| Monte Carlo/simulation, classifier stress test, performance | 20 |
| Detailed API and complete workflows | 21 |
| Package DAG, prior art, gaps, rejected choices, information utilities, V1/future/open boundary | 22 |

## 12.2 Concrete requirements before abstractions

| Workflow | Requirement derived from it | Owning design |
|---|---|---|
| Collection mean/variance/quantile | Explicit Float conversion, sample/population names, empty-input rules, materialization only for order statistics | 15 |
| Large observation stream | One-pass stable state, snapshots, merge laws, deterministic merge order | 15–16 |
| Large binomial upper tail | Direct survival/log-survival, exact count inputs, incomplete-beta backend | 13, 17 |
| Normal density and tiny tail | Separate density/probability, direct complementary and log-tail kernels | 16–17 |
| Seeded sampling | Explicit mutable RNG, uniform-bit law, named algorithm and seed contract | 18 |
| Coin posterior | Validated counts, concrete Beta return type, no runtime conjugacy guessing | 19 |
| Monte Carlo expectation | Sampling plus a Float callback and running moments; no sample list | 20 |
| Weighted categorical draw | Stable normalization, copied input tables, sample type preserved | 16–18 |
| Naive Bayes class scores | Log-mass/log-density, log normalization, clear all-impossible failure | 20 |

## 12.3 Bounded live evidence ledger

The investigation inspected the Phase 1 baseline and 28 targeted substrate files
(the numbered rows below), with three narrow follow-ups for Float ownership,
lazy/eager iteration, and constructor access. Searches were limited to the
owning source/test directories. No compiler or runtime validation was executed
for this prose-only task; cited tests are inspected test precedents.

All paths below are relative to the repository root. Short directory names are
expanded here: `U` = `phalcom-core/core/universe/src`, `C` =
`phalcom-core/tests/core/language`, `S` = `docs/specs`.

| # | Area / file or symbol | Observed behavior or existing intended architecture | Design consequence |
|---|---|---|---|
| E01 | `S/README.md`, authority rule and §5 | Current specs own semantics; no free-function declarations | APIs are methods, associated factories, and exported callable values |
| E02 | `U/math/package.ph`, lines 14–50 | Captures exact Families; Float elementary functions and Int gcd/lcm/isqrt | Reuse facade/capture style; distinguish `ln` from base-`log` |
| E03 | `U/math/float_kernel.ph`, `FloatMathKernel` | Typed Float-only sqrt/ln/ln1p/exp/expm1/hypot | Statistical conversion is explicit; stability primitives already partly exist |
| E04 | `U/math/integer.ph`, `IntegerMath` | Source algorithms use exact `Int`, `~/`, bitLength; negative isqrt raises ArgumentError | Combinatorics can be source-authored, resource-policy-aware |
| E05 | `U/random/package.ph` | One documentation annotation, no generator or sampler declarations | RNG is a real stdlib prerequisite, not an available service |
| E06 | `U/package.ph`, lines 19, 52, 85 | Exposes/imports/exports random namespace | Namespace publication does not imply RNG implementation |
| E07 | `U/collections/iterator_laws.ph`, `Iterator` | Associated Item; stateful next returns Option; lazy map/filter/take/zip | Sampling streams implement this canonical protocol |
| E08 | `U/collections/list.ph`, `List<T>`/`ListIterator<T>` | Mutable generic list, append, fresh iterator, typed conformances | Ordinary sample collections are List<T>; copy retained parameter tables |
| E09 | `U/collections/collection.ph` | Collection is only a nominal root | Require Iterable, not Collection, for statistics |
| E10 | `U/collections/package.ph` | Exposes protocols and concrete collections | Consumers import canonical collection contracts |
| E11 | `U/errors/result.ph`, `Result<T,E>` | Ok/Error, match, map, andThen, unwrap getter | Recoverable construction uses existing Result vocabulary |
| E12 | `U/errors/error.ph`, `Error` | Catchable error base, constructors, raise() | Invalid input uses ordinary terminating errors |
| E13 | `U/option/option.ph`, `Option<T>` | Some/None, typed eliminators and transformations | Undefined summary access can be optional without a missing-data type |
| E14 | `packages/hypothesis/src/choices/provider.ph`, `SeededRandom` | Mutable Int state; 48271 modulo 2147483647; bounded draws by remainder | State/seeding precedent; this small generator is not a statistical RNG contract |
| E15 | Same package, `strategies/strategy.ph`, `Strategy<T>` | Generic method `draw<P>` with conformance bound | Sampling can quantify over the supplied RNG without trait objects |
| E16 | Same package, `strategies/primitives.ph` | Data descriptors and explicit Strategy<Int>/Strategy<Bool> impls | Concrete descriptor + capability is a live package pattern |
| E17 | `C/generic_conformance.rs`, `bounded_generic_function_dispatches_exact_witness_and_default` | Bounded methods carry selected trait witnesses | Third-party participation uses canonical conformance evidence |
| E18 | `C/associated_traits.rs`, associated-type default/explicit-witness tests | Item projections normalize for generic targets; trait-only methods remain detached | Sample/result association needs no type-directed selector redesign |
| E19 | `C/data_e2e.rs`, generic identity and primary-constructor tests | Data construction differs from ordinary `.new`; applied data retain exact identity | Validated descriptor factories must control raw construction |
| E20 | `phalcom-core/tests/fixtures/language/iterator/iterator_terminals.ph` | count/fold/toSet over a pipeline | Use general reduction already provided; avoid a new Statistic protocol |
| E21 | `phalcom-core/tests/integration/server/domain/money.ph` | Module-restricted data constructor and explicit factories | Wrapper validation has a current source precedent; individual arithmetic examples are not numeric authority |
| E22 | Same server fixture, `application/services.ph` | Heterogeneous typed dependencies and associated error equalities appear in a complex fixture | Evidence for advanced intended usage, not blanket certification of all binder syntax |
| E23 | `packages/similarity/src/traits.ph` | Untracked design source with private UnitScore and independent capabilities | Local precedent for bounded scores; no claim this package has passed tests |
| E24 | `S/objects/data-objects.md`, primary-constructor visibility | Normative private/module/package/internal raw-constructor restrictions; shallow immutability | Distribution factories can validate; nested lists still need defensive ownership |
| E25 | `S/objects/traits.md`, conformance ownership and §Scope limits | Explicit nominal conformance, associated types, trait-or-target ownership; no supertraits or trait objects | Independent capabilities, user-owned target impls; callable adapters for foreign targets |
| E26 | `S/generic-constraints.md`, §§2–3 | `|=` conformance, `<:` subtyping, `==` type equivalence; bounds belong in where | Keep sample/observation relations explicit; avoid `T: Trait` sketches |
| E27 | `S/type-level-domains.md`, §§3–5 | Types, static domains, and kinded packs are distinct | Runtime shape/rate/trial count remain values; no distribution parameter type explosion |
| E28 | `S/structural-products.md`, §§1–2 | Tuple/Record/Pack structure and compiler-derived structural evidence | Tuples serve paired observations; packs do not automatically supply numeric iteration |

Follow-ups: `U/scalar/number.ph` confirms Float.new, classification and narrowing;
`iterator_iter_is_lazy_and_idempotent.ph` and
`iterator_direct_map_still_eager.ph` distinguish `.iter.map` from direct `.map`;
`phalcom-semantic/tests/semantic/integration/data_constructor_visibility.rs`,
`module_constructor_access_requires_the_exact_defining_module`, inspects
same-module acceptance and imported raw-constructor rejection.

## 12.4 Contradictions and confidence

The language-design skill overlay still describes dynamic-only typing, an
unbuilt Int/Float split, and absent refutable matching. Live data/trait tests,
`U/scalar/number.ph`, and Option/Result match bodies contradict those status
claims. Use current owning specifications and the live evidence above. Preserve
the Phase 1 baseline's separation of intended semantics and observed source.

The server fixture contains `T: Trait`-style bounds, while E26 specifies
conformance in `where`. Its Money fixture also annotates an Int return around
true division. Neither is copied as current numeric or generic API authority.
The Phase 2 sketches use `where T |= Trait`, `let mut` for reassignable locals,
and the current `.new` versus primary-data-constructor distinction.

High confidence: current Float facade, collection protocols, nominal capability
architecture, error/Option/Result mechanisms. Medium confidence: exact inference
and cross-module capture ergonomics for the new generic signatures; they have
precedents but are not newly exercised here. Absent implementation evidence:
public simulation RNG, statistical reducers, distributions, special-function
backend, probability wrappers, and Bayesian or Monte Carlo modules.
