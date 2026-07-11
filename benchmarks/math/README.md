# Math benchmark corpus

A set of **self-verifying** Phalcom programs that stress the language with real
mathematics. They are deliberately **not** wired into `cargo test` — they are a
staging corpus, meant to be promoted into the golden suite as implementation
phases land. Each program checks **mathematical identities** (e.g. `sin²+cos²=1`,
`φ²=φ+1`, `gcd·lcm=a·b`) rather than hardcoded decimals, so a passing run is
genuine evidence of correctness, not just "it printed something."

Every program prints a column of `true` values (one per assertion). Any `false`
is a failure and names the broken identity in the adjacent comment.

## Running

The workspace tree must build first. Run one file directly:

```sh
cargo run -p phalcom-core --bin phalcom -- benchmarks/math/<file>.ph
```

Or run the whole corpus with the bundled harness, which builds the CLI once and
asserts that every output line is `true`:

```sh
benchmarks/math/run.sh            # all files; Tier-2 files report PENDING, not FAIL
benchmarks/math/run.sh --strict   # also require Tier-2 files to pass
benchmarks/math/run.sh vectors.ph # only the named file(s)
```

A file PASSES iff it exits 0 and prints nothing but `true`. Tier-2 files (needing
`var`/collections) are expected to fail today, so they surface as `PENDING` and
don't fail the run until `--strict`.

> As of this writing the working tree is mid-U6 and does not compile; the last
> green commit exercising the Tier-1 feature set is `83c908a` (U5). These files
> are **unexecuted drafts** — expect to fix a selector name or two on first run,
> especially in the Tier-2 files, whose stdlib surface is still deferred.

## The programs

| File | Tier | Requires | What it tests |
|---|---|---|---|
| `numeric_core.ph` | 1 | U5 | Newton `sqrt`, Euclid `gcd`/`lcm`, factorial (rec vs iter), fast exponentiation, trial-division primality |
| `transcendental.ph` | 1 | U5 | Taylor `exp`/`sin`/`cos`/`atan`; Machin's π; Pythagorean & exponential-addition identities |
| `integration.ph` | 1 | U5 | Composite Simpson's rule generic over a **block-valued** integrand; π via `∫4/(1+x²)`; exactness on cubics; linearity |
| `number_theory.ph` | 1 | U5 | Fast modular exponentiation, Fermat's little theorem, perfect/abundant numbers, Collatz step counts, π(30) |
| `continued_fractions.ph` | 1 | U5 | Fixed-point iteration of continued fractions for φ and √2, checked against their defining identities |
| `rationals.ph` | 1.5 | U5 (object model) | Exact `Rational` class: operator methods `+ - * / ==`, gcd reduction, telescoping sums — exact `==` is legitimate here |
| `monte_carlo.ph` | 1 | U5 | In-language MINSTD PRNG; π by darts & by 3D ball, `e` by expected-crossing, LLN (mean→½, var→1/12), MC integration of a block-valued `f` |
| `random_walk.ph` | 1 | U5 | Random walks off the same PRNG: unbiasedness `E[S_n]→0`, 1D/2D diffusion `E[S_n²]→n`, fair-coin rate |
| `stats.ph` | 2 | U6 `var` + collections + stdlib | mean/variance/stddev/median over a list; two variance formulas cross-checked; shift-invariance |
| `vectors.ph` | 2 | U6 `var` + collections + stdlib | dot/norm/cosine; `dot(a,a)=‖a‖²`, Cauchy–Schwarz, orthogonality + Pythagoras |

### Tiers

- **Tier 1** — pure recursion / closures / control flow / arithmetic. Every
  construct was observed working at U5 (`83c908a`). These should run today on a
  buildable tree and are the first candidates for promotion into the golden set.
- **Tier 1.5** — adds the object model (instance fields, operator methods,
  static factories). Also U5-era; grouped separately only because exact rational
  arithmetic makes `==` assertions meaningful.
- **Tier 2** — needs `var` (U6, [ADR-0014](../../docs/adr/0014-let-and-var-bindings.md)),
  list/map literals ([lexical-structure.md §4/§6](../../docs/spec/lexical-structure.md)),
  the iteration protocol ([iteration-protocol.md](../../docs/spec/experimental/iteration-protocol.md)),
  and a standard-library collection surface that is **still deferred**
  ([typing-stdlib-surface.md](../../docs/spec/experimental/typing-stdlib-surface.md)).
  Selector names (`reduce`, `sorted`, `map`, list `+`) are best-effort guesses at
  the eventual surface and should be reconciled when U-STD is specified.

## Promotion checklist (per file, when its tier lands)

1. Run it; confirm the output column is all `true`.
2. Move it under `phalcom-core/tests/lang/math/` with the standard header
   (`// area:`, `// spec:`, `// status: PASS`).
3. Capture a golden snapshot of stdout so regressions are caught.
4. Delete the corresponding row above (or mark it **LANDED**).
