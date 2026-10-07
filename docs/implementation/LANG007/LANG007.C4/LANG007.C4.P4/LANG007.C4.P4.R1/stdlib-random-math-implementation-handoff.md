# `universe.math` / `universe.random` — in-progress implementation handoff

**Status:** IN_PROGRESS / PARTIAL / FOCUSED_TESTED. This is a continuation record, **not** a claim that the math and random libraries are complete. The active checkout is `/Users/altunhasanli/dev/phalcom/phalcom` on `stdlib-random-math`; latest delivered commit before this continuation was `06e37110`.

## Worktree, authority, and instructions

- Continue on branch `stdlib-random-math` and push the completed work there. The duplicate local math branch has already been removed. Preserve unrelated work and stage only this plan's files.
- Read the user's requirements document at `docs/implementation/LANG007/LANG007.C4/LANG007.C4.P4/LANG007.C4.P4.R1/stdlib-random-math.md`. It was absent from this worktree at creation, then committed and incorporated from `stdlib-random-math` during delivery. Do not overwrite it. The requirements are ~2,655 lines and cover all M1/R1/R2/R3 phases. Also follow `AGENTS.md`, `docs/specs/README.md`, the owning specs, and the relevant implementation-record convention. The existing `LANG007.C4` checkpoint is occupied by an unrelated structural/static P4 plan: do not silently claim this work as that numbered plan or overwrite its active-plan state.
- Previously approved decisions: use Rust `f64` methods, keep math/random/secure-random/distributions as separate phases on this branch, add direct `getrandom` for secure randomness, expose math operations as exact Family captures, and fix imported `const` generally at the semantic-to-runtime boundary. The accepted topology keeps `FloatMathKernel` in hidden `math.float_kernel` and `IntegerMath` in hidden `math.integer` children.
- The approved T9 semantic owner is the module value boundary: resolve each linked public export to its canonical `SymbolId` and checked `ModuleBindingFact`, preserving type and denotation. Existing `synthesize_family_value_call -> apply_resolved_callable` remains the only call-checking owner. Member bodies receive this environment through `FormalQueryInputs` and `BodyAnalysisContext`, with exact import/export dependencies on `CallableBody`.
- The worktree started clean. An initial `cargo test -p phalcom-core --test language-corpus --no-run` compiled. A subsequent build temporarily failed for lack of disk space; the user fixed the disk. Do not treat that as a code failure.

## Math-phase changes committed in `3ab77b6e`

1. `phalcom-core/core/universe/src/math/package.ph`: six typed `const` values, exact Family captures of Float and integer methods, and public exports. `FloatMathKernel` and source-authored `FloatAngleHelpers` live in hidden `float_kernel.ph`; `IntegerMath` lives in hidden `integer.ph`. Neither child is exposed publicly.
2. `phalcom-core/core/universe/src/math/integer.ph`: `IntegerMath` class with source-authored Euclidean `gcd`, reduced-product `lcm`, and convergent exact Newton `isqrt`. Corrected the checker issue with `while true` and declared `Int` return by using a terminating `while next < x` followed by `return x`. `isqrt` rejects negatives with `ArgumentError`. Has **not** had full boundary/property coverage yet.
3. `phalcom-core/src/primitive/math.rs`: 21 unary and two binary `f64` kernels; direct `#[primitive]` declarations with public visibility. **Do not replace with a `macro_rules!` macro around the attribute**: `phalcom-native-surface-gen` scans `syn::Item::Fn` and does not expand macro invocations. The initial macro form produced 361 instead of 382 native surface rows; expanded function declarations fixed the generated manifest. The Rust proc macro also requires a literal selector, not `concat!(...)`. Registration is through `#[primitive]`, not a hand-written `Universe::install_primitives` addition.
4. `phalcom-native-meta/src/universe.rs`: source-owned `UniverseKey::FloatMathKernel`, canonical path `['math']`, name/from-name mapping. `phalcom-core/src/universe/core_classes.rs` handles this key as a source-owned class with no primordial slot. `phalcom-core/src/primitive/mod.rs` registers the module. `phalcom-native-surface/src/generated.rs` was regenerated with `RUSTFLAGS='' cargo run -p phalcom-native-surface-gen -- --root .` (382 declarations; previously 354).
5. `phalcom-modules/src/builtin.rs`: Universe math package registers both hidden children and embeds their source. Source provider requires explicit node and `include_str!` routing: merely adding a file to `src/math/` does **not** register it.
6. `phalcom-core/tests/fixtures/language/math/math_foundation.ph` and `.expected`; `phalcom-core/tests/core/language/corpus.rs` adds a focused `math` lane. Before the static constant correction, math `sqrt`/`hypot`/integer calls executed with expected results, but importing `pi`, `tau`, or module-qualified `math.e` returned `None`.
7. The general exported-static-constant identity path is implemented and focused-tested, including selected and qualified reads, re-exports, reflection, and heap-backed values surviving GC.

No source files in the original main checkout were modified by this branch. Re-check status and diff in this worktree before resuming implementation.

## Investigations and confirmed contracts

- `docs/specs/universe.md`: one canonical source-owned Universe module graph. `phalcom-modules/src/builtin.rs::UNIVERSE_NODES` and `UniverseSourceProvider::source_text` register embedded modules. `phalcom-core/src/native/source.rs::source_universe_identity` maps canonical `UniverseKey` paths, and `native/install.rs::install_source_owned_primitives` installs ordinary source-owned primitives **after** source module initialization. Match source anchors, generated native surface, and Rust descriptors. `vm/bootstrap.rs::run_universe_modules` compiles/executes the canonical bootstrap dependency order.
- `docs/specs/callables/families.md` and `docs/specs/source-declarations.md`: bound exact `&Class.method(_)` Families do **live** method lookup using the eventual **caller's** access authority. `vm/send.rs::activate_family_with_kind` passes `view.caller_authority()` to dispatch. Thus public `math.sqrt` cannot re-export an **internal** kernel method as a public callable. The failed experiment produced `internal.selector_access` at a user invocation. Approved layout: public methods on the unexported class in `math/package.ph`, selected exact captures; no wrapper closures or change to general Family authorization.
- `docs/specs/static-evaluation.md` §12 and `bindings-and-fields.md` §4: `const` has no executable initializer/global runtime binding; successful checked constants have `MaterializationDescriptor`. `phalcom-core/src/compiler/lib/mod.rs` explicitly emits **no runtime slot** for `BindingKind::Const`, and `compiler/lib/expr.rs` materializes same-module `const` reads from `lowering.static_constants`. The existing linker/runtime treated **all exported values** as slots; `modules/materialize.rs` created an unwritten export/import slot, which surfaced as `None`. This is the cause of the math-constant failure, independent of `f64` math.
- `modules/semantic_lowering.rs::project_static_constants` derives checked descriptors from `SemanticSnapshot::static_constants()`/`static_values()`. `CanonicalUniverseProgram` has compiled canonical module lowerings; `vm/bootstrap.rs::prepare_universe_static_constants` in the current patch attaches these lowerings to canonical runtime modules before bootstrap source initializers. For ordinary programs `materialize_program` already stores `compiled_mod.lowering` on newly allocated modules.
- Static constant materialization uses `RuntimeStaticConstantRef` with canonical owner identity. Owner modules retain and trace materialized values. Primitive descriptors are supported; unsupported/deferred structural values fail closed. Do not add a second constant value authority.
- `phalcom-core/core/universe/src/scalar/number.ph`: `Int` supports `bitLength`, shifts, bitwise AND/OR/XOR, exact BigInt backing, and floor `~/`. `Float.new(Int)` is explicit. `collections/bytes.ph` supports `Bytes.new(n)`, `.size`, `.at`/`.set`, `.fill`. `collections/list.ph` is the canonical random-access/mutation surface; `option` uses `Some`/`None`. `collections/iterator_laws.ph` is a model for trait associated types/defaults. `time` source and `primitive/time.rs` are examples of source-owned host-native declarations.

## Historical exported-constant failure — resolved

Command (from worktree):

```sh
RUSTFLAGS='' cargo test -p phalcom-core --test core exported_static_constant_materializes_for_selected_and_module_reads -- --nocapture
```

This failure occurred during the initial exported-constant repair. The runtime getter correction and canonical re-export model are complete. On 2026-10-07, the selected/module read regression and re-export/GC identity regression both passed; the math corpus passed as well.

The requirements' final example uses `math.tau * radius`; selective and module-qualified exports and `Export.value` reflection are verified. Both `RuntimeLinkedRead::StaticConstant` and `RuntimeExportRef::StaticConstant` retain the canonical `RuntimeStaticConstantRef`. The owning module stores and traces the materialized value; aliases retain source owner/name identity.

## Math M1 completion

The bounded M1 math scope is complete and focused-tested. The new
`audit_probes::math` suite passes 11/11 and covers the six-constant bit-pattern
oracle against both literal and standard-library binary64 bits, every public
Float operation against its matching same-platform Rust `f64` operation,
IEEE/domain/signed-zero/subnormal behavior, all 24 pure native descriptors and
direct no-allocation calls, independent BigInt gcd/lcm laws, arbitrary-size
isqrt boundaries and invariants through 10^1000, negative `ArgumentError`, and
numeric-limit enforcement. The accepted accuracy boundary remains: `sqrt` may
inherit stronger guarantees; unspecified-precision transcendental results have
no cross-platform last-bit promise.

## Next implementation phases, after math passes

### Math M1

- Freeze all six constants' `Float` bits against independent `std::f64::consts` oracle and verify the canonical module export surfaces. Do not rely solely on formatted strings. Add tests that `sqrt(4)` is rejected without implicit exact-to-inexact conversion and `gcd` requires Int. Verify exact Families retain `Family` identity, a single selector, and proper native association. Avoid depending on a `Math` class.
- Exercise IEEE edges for all kernels (NaN, infinities, signed zero, subnormals, domain limits; `atan2` signed-zero quadrants) and representative finite accuracy with appropriate ULP or rigorously chosen reference data. `f64` backend was approved; do not promise cross-toolchain bit-for-bit equality for transcendental functions. Document radians and exactness boundaries.
- Add property/boundary tests for arbitrary-size `gcd`, `lcm` and `isqrt`, including negative `ArgumentError`, zero, very large perfect squares/adjacent inputs. Verify native resource limits remain respected. Source `IntegerMath` currently uses `let mut x/y/next`; check `isqrt` convergence and overflow/BigInt behavior.
- Both implementation children remain hidden from external import. `Family.receiver` may reveal the kernel receiver; this is accepted technical debt for this slice. The public math API uses exact Family captures.

### Deterministic random R1

- Register every new `universe.random` child in `phalcom-modules/src/builtin.rs`; update `random/package.ph` with actual import/expose/export grammar, using source `trait`/`impl` syntax from `collections/iterator_laws.ph`. Class fields/constructors from `time/clock.ph` or `collections/bytes.ph` show current syntax. Don't add native deterministic RNG primitives, heap variants, VM opcodes, or global/default generator.
- `Xoshiro256PlusPlus.seed(Int)` must check `0 <= seed < 2^64` before constructing state. Expand seed with published SplitMix64; mask **each** arithmetic/rotation/state word with `(1 << 64) - 1`; freeze state ordering and output via independently sourced golden vectors for seeds 0, 1, 42, max u64. Test interleaving of `nextU64`/`nextBool`/`nextFloat`/bounded draws. `nextFloat = Float.new(nextU64 >> 11) / 2^53` and `nextBool` consumes one word via high bit.
- `nextInt(below:)`: unbiased arbitrary-precision rejection sampling of `bitLength(upper-1)` bits; concatenate earlier generated words into higher-order chunks, mask excess high bits, retry if candidate >= upper. Handle upper=1 (zero bits, document consumption), powers of two, >64-bit bounds, invalid bounds, and explicit rejection-path mocks. Byte extraction: big-endian from each `nextU64`, one word per up to eight bytes, discard unused final bytes. `choose(List<T>)` gives `Option<T>`; `shuffle(List<T>)` in-place Fisher–Yates descending; freeze golden consumption.
- The requirements file specifically calls for named `Generator` trait with derived default methods, all source-authored where possible, and explicit documentation that this generator and its bytes are not secure. Explore ability to express `List<T>` method constraints; do not quietly weaken to `Dynamic` when generics exist.

### Secure random R2

- Add **direct** `getrandom` dependency (user approved) and a source-owned `SecureRandom` declaration (likely class with a validated `.system` constructor and source `.bytes(count)` built from `Bytes.new(count)` plus native `.fill`). Its only host-native boundary is OS CSPRNG filling `Bytes`; no fallback to xoshiro/time/hash keys. Do not conform to `Generator`.
- Investigate `getrandom` version/API already present in `Cargo.lock` (transitive `getrandom 0.3.3` was compiled). On host failure propagate the existing appropriate system error or request approval if a new error type is required. Validate counts/resource policy, zero-length fill, and GC/rooting while native code borrows the mutable `Bytes` buffer. Add an injectable failure seam/test only if it can be done without expanding the public API or duplicating entropy sources.
- Register its canonical `UniverseKey` and regenerate `phalcom-native-surface/src/generated.rs` after native function declarations; check source anchors and descriptor parity.

### Distributions R3

- Source `Distribution` trait with associated `Output` and constrained `sample` method; validated private `UniformInt` (`upper > lower`) and `Bernoulli` (finite, `0 <= p <= 1`, explicit NaN reject). `UniformInt` delegates to half-open `nextInt`; Bernoulli's exact endpoints must yield false/true. Decide and test whether boundary sampling consumes zero or one words as part of replay, and request approval if not settled by the requirements. Negative conformance test: `SecureRandom` must not satisfy `Generator`.
- Run focused corpus/API tests for this gate. Do not add continuous distributions, global RNG, implicit seeding, or security claims for xoshiro.

## Commands and workflow

Run from the worktree, serially for Cargo:

```sh
git status --short
git diff --check
RUSTFLAGS='' cargo check -p phalcom-core
RUSTFLAGS='' cargo test -p phalcom-core --test core exported_static_constant_materializes_for_selected_and_module_reads -- --nocapture
RUSTFLAGS='' cargo test -p phalcom-core --test language-corpus math -- --nocapture
RUSTFLAGS='' cargo run -p phalcom-native-surface-gen -- --root . --check
```

After adding native primitives: `RUSTFLAGS='' cargo run -p phalcom-native-surface-gen -- --root .` regenerates the VM-free surface. Note the generated file is large; review its scoped diff before delivery. Prefer focused tests during BUILD; run broader directly affected suites only at a coherent gate. `cargo fmt --all -- --check` is useful at completion; formatting the whole worktree could create unrelated edits, so inspect the diff afterward. Do **not** run workspace release suites by reflex or claim verification unless the command actually finishes successfully. Keep worktree local, respect unrelated main-checkout edits, and ask the user before any new nontrivial decision.

## Explicit delivery — 2026-10-07

The original math implementation and its initial docs were committed and
pushed in prior deliveries. Those delivery notes below are historical; current
status and evidence are recorded in the continuation update.

## Corrective math plan T0 baseline — 2026-10-07

The supplied corrective math plan was executed from `stdlib-random-math` at
`9742d085580a98fee8822e1c66026d179ddd772e`, matching its prepared revision.
The only pre-existing working-tree entries were the supplied, untracked audit
and implementation-plan documents under `LANG007.C4.P4/math/`.

- `git diff --check`: PASS.
- `RUSTFLAGS='' cargo check -p phalcom-core`: PASS.
- `RUSTFLAGS='' cargo test -p phalcom-core --test core exported_static_constant_materializes_for_selected_and_module_reads -- --nocapture`: FAIL, 1 selected. `constants.pi` materialized as `3.141592653589793`, then module getter dispatch attempted `call` and raised `does not understand 'call'` at `main.ph:4:17`.
- `RUSTFLAGS='' cargo test -p phalcom-core --test language-corpus math -- --nocapture`: FAIL, 1 selected. `math.e` materialized as `2.718281828459045`, then module getter dispatch attempted `call` at `<root>.ph:9:14`.

The planned failure was reproduced at this baseline and was subsequently
resolved by the exported-static-constant work described above.

## Corrective math execution update — 2026-10-07

The runtime and facade slices advanced after the T0 baseline:

- RuntimeStaticConstantRef now projects canonical static-constant identity into
  VM materialization; owner modules retain and trace the materialized values.
- Selected imports, module getters, re-exports, reflection, and linked reads
  resolve that canonical reference. Focused getter and re-export/GC tests pass.
- FloatMathKernel and source-authored angle helpers now live in the hidden
  universe.math.float_kernel child; IntegerMath remains hidden in
  universe.math.integer. External imports of either child are rejected.
- The public facade adds log(_,base), degrees, and radians; the focused math
  corpus, including finite, special-value, and large-integer fixtures, passes.

The T9 type-boundary issue was resolved at the canonical module-value
boundary. The following additional acceptance evidence now passes:

- Selective and qualified Family calls in member bodies reject incorrect
  arguments. The same regression includes accepted top-level shapes and a
  facade re-export.
- Member-body products record `ModuleStaticBindings` dependencies. When the
  provider's exported Family changes from `Int` to `String`, the consumer
  diagnostic updates and matches a cold analysis.
- The compiler-facing exact-Family/numeric-rejection test and hidden-child
  import rejection test pass.
- Generated native surface regeneration made no file changes; generator
  `--check` passes with 383 records.

T7 through T11 are implemented and focused-tested. The static-constant
selected/module regression, the re-export/GC identity regression, hidden-child
rejection, and the `language-corpus math` test all pass. The four T12 crate
checks for core, modules, native-meta, and native-surface pass. T12's broad
core run had 160 passes and three unrelated library failures; workspace format
checking reports existing drift outside this continuation's Rust files. The
continuation walkthrough/handoff record exact commands and residuals. M1 math
is complete. No random implementation has started; random, secure random, and
distributions remain successor work.

The final wider module cluster ran 60 tests: 54 passed and six non-math fixture
or Telogue generic-inference failures were classified D relative to this
test-only continuation. Native source/descriptor tests passed 11/11. The math
walkthrough owns the exact final ledger and deferred verification.
