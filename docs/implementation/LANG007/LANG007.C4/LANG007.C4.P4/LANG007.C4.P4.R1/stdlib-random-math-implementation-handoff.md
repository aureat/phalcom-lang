# `universe.math` / `universe.random` — in-progress implementation handoff

**Status:** IN_PROGRESS / PARTIAL / UNVERIFIED. This is a continuation record, **not** a completion walkthrough or a claim that either library is complete. Prepared from worktree base `615f5887` and the math implementation committed as `3ab77b6e`. The last recorded focused test is **failing** for a known cause described below; it was not rerun during delivery.

## Worktree, authority, and instructions

- Work **only** in `/Users/altunhasanli/dev/phalcom/phalcom/.worktrees/stdlib-random-math` on local branch `feat/stdlib-random-math`. The earlier local-only delivery constraint is superseded by the user's later explicit instruction to commit and push the work to remote branch `stdlib-random-math`. Do not use subagents: the user asked for inline implementation.
- Read the user's requirements document at `docs/implementation/LANG007/LANG007.C4/LANG007.C4.P4/LANG007.C4.P4.R1/stdlib-random-math.md`. It was absent from this worktree at creation, then committed and incorporated from `stdlib-random-math` during delivery. Do not overwrite it. The requirements are ~2,655 lines and cover all M1/R1/R2/R3 phases. Also follow `AGENTS.md`, `docs/specs/README.md`, the owning specs, and the relevant implementation-record convention. The existing `LANG007.C4` checkpoint is occupied by an unrelated structural/static P4 plan: do not silently claim this work as that numbered plan or overwrite its active-plan state.
- The user explicitly said: **pause and ask approval for important nontrivial decisions**. Already approved decisions: (1) use Rust `f64` methods for binary64 math; (2) four reviewable phases in this one worktree: math, deterministic random, secure random, distributions; (3) add direct `getrandom` dependency for secure randomness; (4) move `FloatMathKernel` into `math/package.ph` and use public methods captured into exact public Families; (5) fix imported `const` generally at the semantic-to-runtime boundary, rather than downgrading math constants to `let`.
- The worktree started clean. An initial `cargo test -p phalcom-core --test language-corpus --no-run` compiled. A subsequent build temporarily failed for lack of disk space; the user fixed the disk. Do not treat that as a code failure.

## Math-phase changes committed in `3ab77b6e`

1. `phalcom-core/core/universe/src/math/package.ph`: source-owned `native class FloatMathKernel` declared **in the package**, 23 public class-side native binary64 methods, six typed `const` values, exact Family captures of those methods, and captures of integer methods. Exports constants/functions without exporting `FloatMathKernel` itself. `expose .integer` permits the package's own import; reconsider public exposure only if the general module system provides a legitimate private sibling-import path (do not invent one).
2. `phalcom-core/core/universe/src/math/integer.ph`: `IntegerMath` class with source-authored Euclidean `gcd`, reduced-product `lcm`, and convergent exact Newton `isqrt`. Corrected the checker issue with `while true` and declared `Int` return by using a terminating `while next < x` followed by `return x`. `isqrt` rejects negatives with `ArgumentError`. Has **not** had full boundary/property coverage yet.
3. `phalcom-core/src/primitive/math.rs`: 21 unary and two binary `f64` kernels; direct `#[primitive]` declarations with public visibility. **Do not replace with a `macro_rules!` macro around the attribute**: `phalcom-native-surface-gen` scans `syn::Item::Fn` and does not expand macro invocations. The initial macro form produced 361 instead of 382 native surface rows; expanded function declarations fixed the generated manifest. The Rust proc macro also requires a literal selector, not `concat!(...)`. Registration is through `#[primitive]`, not a hand-written `Universe::install_primitives` addition.
4. `phalcom-native-meta/src/universe.rs`: source-owned `UniverseKey::FloatMathKernel`, canonical path `['math']`, name/from-name mapping. `phalcom-core/src/universe/core_classes.rs` handles this key as a source-owned class with no primordial slot. `phalcom-core/src/primitive/mod.rs` registers the module. `phalcom-native-surface/src/generated.rs` was regenerated with `RUSTFLAGS='' cargo run -p phalcom-native-surface-gen -- --root .` (382 declarations; previously 354).
5. `phalcom-modules/src/builtin.rs`: Universe math package now has `integer` child and embeds `integer.ph`. Source provider requires explicit node and `include_str!` routing: merely adding a file to `src/math/` does **not** register it. The initially considered `kernel.ph` was removed after discovering Family access semantics.
6. `phalcom-core/tests/fixtures/language/math/math_foundation.ph` and `.expected`; `phalcom-core/tests/core/language/corpus.rs` adds a focused `math` lane. Before the static constant correction, math `sqrt`/`hypot`/integer calls executed with expected results, but importing `pi`, `tau`, or module-qualified `math.e` returned `None`.
7. An in-progress **general exported static constant** fix in `phalcom-core/src/modules/static_constant.rs`, `modules/mod.rs`, `modules/linkage.rs`, `modules/materialize.rs`, `vm/bootstrap.rs`, `vm/dispatch.rs`, `vm/send.rs`, `heap/module.rs`, `heap/trace.rs`, `modules/context.rs`, `modules/reflection_cache.rs`, `primitive/reflection.rs`. A regression was added to `tests/core/modules/parity.rs`, with exhaustiveness updates in `tests/core/modules/linking.rs`. This is unfinished; evaluate the design critically before building more on it (see below).

No source files in the original main checkout were modified by this branch. Re-check status and diff in this worktree before resuming implementation.

## Investigations and confirmed contracts

- `docs/specs/universe.md`: one canonical source-owned Universe module graph. `phalcom-modules/src/builtin.rs::UNIVERSE_NODES` and `UniverseSourceProvider::source_text` register embedded modules. `phalcom-core/src/native/source.rs::source_universe_identity` maps canonical `UniverseKey` paths, and `native/install.rs::install_source_owned_primitives` installs ordinary source-owned primitives **after** source module initialization. Match source anchors, generated native surface, and Rust descriptors. `vm/bootstrap.rs::run_universe_modules` compiles/executes the canonical bootstrap dependency order.
- `docs/specs/callables/families.md` and `docs/specs/source-declarations.md`: bound exact `&Class.method(_)` Families do **live** method lookup using the eventual **caller's** access authority. `vm/send.rs::activate_family_with_kind` passes `view.caller_authority()` to dispatch. Thus public `math.sqrt` cannot re-export an **internal** kernel method as a public callable. The failed experiment produced `internal.selector_access` at a user invocation. Approved layout: public methods on the unexported class in `math/package.ph`, selected exact captures; no wrapper closures or change to general Family authorization.
- `docs/specs/static-evaluation.md` §12 and `bindings-and-fields.md` §4: `const` has no executable initializer/global runtime binding; successful checked constants have `MaterializationDescriptor`. `phalcom-core/src/compiler/lib/mod.rs` explicitly emits **no runtime slot** for `BindingKind::Const`, and `compiler/lib/expr.rs` materializes same-module `const` reads from `lowering.static_constants`. The existing linker/runtime treated **all exported values** as slots; `modules/materialize.rs` created an unwritten export/import slot, which surfaced as `None`. This is the cause of the math-constant failure, independent of `f64` math.
- `modules/semantic_lowering.rs::project_static_constants` derives checked descriptors from `SemanticSnapshot::static_constants()`/`static_values()`. `CanonicalUniverseProgram` has compiled canonical module lowerings; `vm/bootstrap.rs::prepare_universe_static_constants` in the current patch attaches these lowerings to canonical runtime modules before bootstrap source initializers. For ordinary programs `materialize_program` already stores `compiled_mod.lowering` on newly allocated modules.
- Current `modules/materialize.rs` attempts to use `VM::static_constant_spec` to create `RuntimeLinkedRead::StaticConstant { module, value }` and `RuntimeExportRef::StaticConstant(value)` instead of runtime slots. `modules/static_constant.rs::materialize_static_constant` handles primitive descriptors (Unit, Bool, small/large Int, Float, String, Symbol) and errors for unsupported/deferred structural values. `heap/trace.rs` traces value edges. Check initialization ordering and alias/re-export identities; do not invent a second constant value authority.
- `phalcom-core/core/universe/src/scalar/number.ph`: `Int` supports `bitLength`, shifts, bitwise AND/OR/XOR, exact BigInt backing, and floor `~/`. `Float.new(Int)` is explicit. `collections/bytes.ph` supports `Bytes.new(n)`, `.size`, `.at`/`.set`, `.fill`. `collections/list.ph` is the canonical random-access/mutation surface; `option` uses `Some`/`None`. `collections/iterator_laws.ph` is a model for trait associated types/defaults. `time` source and `primitive/time.rs` are examples of source-owned host-native declarations.

## Latest observed failure — exact continuation point

Command (from worktree):

```sh
RUSTFLAGS='' cargo test -p phalcom-core --test core exported_static_constant_materializes_for_selected_and_module_reads -- --nocapture
```

Result: 1 selected, **FAILED**, after `cargo check -p phalcom-core` succeeded and exhaustiveness fixes allowed the test target to compile. The test builds a package with `constants.ph` exporting `const pi`, and `main.ph` using both `from .constants import pi` and `import .constants as constants; constants.pi`. Runtime error on `constants.pi`: `3.141592653589793 does not understand 'call'` at `main.ph:4:17`. This means the new static export is now providing a real Float, but **module getter dispatch** proceeds to call it as if it were a callable. The first selected import may have worked; confirm independently rather than assuming. The current `vm/send.rs::try_module_export_send_with_environment` branch for `RuntimeExportRef::StaticConstant` sets `target_val = value` but lacks the getter early return used for `Binding`/`Module`; `try_module_export_send` (other path around line 1700) already has an early return for static constants. Fix the environment-aware getter branch by returning the value directly for `SignatureKind::Getter`, as the existing binding branch does. Run the exact discriminator above again. This is a mechanical cause with a clear observed output, not permission to redesign module dispatch.

Be aware the requirements' final example uses `math.tau * radius`; both selective and module-qualified exports must work, as must ordinary `Export.value` reflection. `RuntimeLinkedRead::StaticConstant` keeps its owner module in the variant for GC/dependency reflection. `RuntimeExportRef::StaticConstant` currently stores only `Value`; if this loses canonical export owner identity for re-exports/reflection, solve at the owning materialization boundary after a focused reproducer; request user approval before materially changing ownership/identity.

## Remaining static-const verification and potential problems

1. After the getter correction, rerun the focused parity regression and `RUSTFLAGS='' cargo test -p phalcom-core --test language-corpus math -- --nocapture`. Confirm nonzero test selection. Expect additional issues: `const` exports may not be in `canonical.program().modules` for user compilation; cross-module alias/re-export chains may resolve to the source `SymbolId` or an intermediate, so test a re-export separately. The patch currently materializes **one value per import/export** and additionally at bootstrap; assess identity and rooting for String/BigInt constants (sharing is not guaranteed by spec, but semantic value identity/content must be preserved). No `Float` bit-pattern tests have been added yet.
2. The `const` code path should not silently fall back to creating an empty slot when a known constant lacks a successful materialization descriptor. Distinguish genuine runtime bindings from static constants using canonical semantic products, and fail closed for unsupported static descriptors. A silent `None` is worse than an explicit error. Check ordinary and canonical Universe program compilation, native `VM::new` bootstrap, GC after import, reflection `Export.value`, and source/private import access. Consider whether `RuntimeExportRef::StaticConstant` needs owner identity as a field; do not silently decide identity semantics.
3. The new regression in `tests/core/modules/parity.rs` currently asserts **no** `pi` slot in `constants` after runtime execution, consistent with the `const` rule. After fixing the getter, make sure this assertion and both value assertions pass. Add a separate user-owned package test and Universe math fixture if one alone fails to distinguish source/linking from bootstrap.
4. `vm/bootstrap.rs` has an existing test near lines 1161–1179 that exhaustively matches expected `LinkedReadSpec` against runtime reads and assumes every binding is `RuntimeLinkedRead::Binding`. If any canonical bootstrap dependency imports a `const`, update the test to assert the appropriate static shape and provenance; do not weaken it with a wildcard. Avoid broad suite runs until the coherent math gate.
5. The work already touches GC-sensitive `heap/trace.rs`. Before calling any const fix complete, add a focused test for a heap-backed exported static String/BigInt surviving `vm.force_gc()` and remaining readable; consider whether `RuntimeExportRef::StaticConstant(Value)` is rooted through module exports. Current tracing includes that edge.

## Next implementation phases, after math passes

### Math M1

- Freeze all six constants' `Float` bits against independent `std::f64::consts` oracle and verify the canonical module export surfaces. Do not rely solely on formatted strings. Add tests that `sqrt(4)` is rejected without implicit exact-to-inexact conversion and `gcd` requires Int. Verify exact Families retain `Family` identity, a single selector, and proper native association. Avoid depending on a `Math` class.
- Exercise IEEE edges for all kernels (NaN, infinities, signed zero, subnormals, domain limits; `atan2` signed-zero quadrants) and representative finite accuracy with appropriate ULP or rigorously chosen reference data. `f64` backend was approved; do not promise cross-toolchain bit-for-bit equality for transcendental functions. Document radians and exactness boundaries.
- Add property/boundary tests for arbitrary-size `gcd`, `lcm` and `isqrt`, including negative `ArgumentError`, zero, very large perfect squares/adjacent inputs. Verify native resource limits remain respected. Source `IntegerMath` currently uses `let mut x/y/next`; check `isqrt` convergence and overflow/BigInt behavior.
- The `integer` child is `expose`d to allow source import. If public `IntegerMath` is undesirable, consider a checked direct declaration in `math/package.ph` or a supported internal-only import path; request approval for an observable API change. The current primary public API is the captured `gcd/lcm/isqrt`, which are source-authored members.

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

The user explicitly requested that all math-worktree changes be staged, tracked,
committed, and pushed to remote branch `stdlib-random-math`. The complete
27-file math-phase change set, including this handoff, was committed as
`3ab77b6e` (`feat: add initial universe math implementation`) and pushed after
the documentation commits `3369adf1` and `eaf95877`. The local worktree branch
remains `feat/stdlib-random-math`; its HEAD was rebased onto the requested
remote branch before push.

No tests or builds were run during delivery. The last recorded focused
discriminator remains failing: module-qualified `constants.pi` resolves to a
Float and then module getter dispatch tries to call it, reporting
`3.141592653589793 does not understand 'call'` at `main.ph:4:17`. Fix and rerun
the named focused parity and `language-corpus math` gates before treating the
math slice as focused-tested. Keep this slice `IN_PROGRESS / PARTIAL /
UNVERIFIED`; deterministic random, secure random, and distributions have not
been implemented.
