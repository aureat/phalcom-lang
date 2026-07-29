# Wren benchmark suite, ported

Direct ports of `wren/test/benchmark/*.wren` (from the sibling
`/Users/altunhasanli/dev/repos/wren` checkout) to Phalcom, for cross-language
throughput comparison. Every file's printed output matches its Wren original
exactly (cross-checked below) — these are correctness-verified, not just
"it ran."

Not ported: `api_call.wren`/`api_foreign_method.wren` (exercise the embedder C
API, not expressible as a standalone script), `fannkuch.*` (no `.wren` source
in the benchmark dir — only lua/py/rb), `delta_blue.wren` (blocked — needs
`List#removeAt`/stack-pop and index-write `list[i]=`, neither of which exist
on Phalcom's `List` today; see Porting notes).

## Running

```sh
cargo build -r -p phalcom-core --bin phalcom
time ./target/release/phalcom benchmarks/wren-suite/<file>.ph
```

## Results (release build, this machine, best-of-3 — not statistically rigorous)

Re-measured 2026-07-14 at `debadfa` (after U-PRIM-ABI cut 001, U-GC Win A cut
002, U-TRACE cut 003, U-GC's collector, and U-HOTPATH). The `(was …)` column
is the original measurement this table carried, taken before those cuts; every
row improved, `for` and `map_string` by an order of magnitude. Each Phalcom
run's stdout is compared byte-for-byte against its Wren original's (minus
Wren's `elapsed:` line) — a row is only reported if the output matches, since
a benchmark that computes the wrong answer is not a measurement.

| Benchmark | Wren | Phalcom | Slowdown | (was) | Output matches |
|---|---|---|---|---|---|
| `fib.wren` (fib(28) ×5) | 0.18s | 0.87s | 4.8x | 8.6x | ✓ 317811 ×5 |
| `for.wren` (1M list build+sum) | 0.05s | 0.73s | 13.6x | ~144x | ✓ 499999500000 |
| `fibers.wren` (100k chained) | 0.03s | 0.13s | 4.0x | ~12x | ✓ 4999950000 |
| `method_call.wren` (2M dispatch) | 0.09s | 0.53s | 5.7x | 8.2x | ✓ true / false |
| `string_equals.wren` (10M compares) | 0.11s | 1.10s | 10.0x | 17x | ✓ 3000000 |
| `binary_trees.wren` | 0.18s | 0.85s | 4.9x | 7.3x | ✓ all check lines |
| `binary_trees_gc.wren` | 0.56s* | 0.84s | 1.5x* | 2x* | ✓ all check lines |
| `map_numeric.wren` (2M map ops) | 0.86s | 3.85s | 4.5x | 5.9x | ✓ 2000001000000 |
| `map_string.wren` (~193k-key map) | 0.10s | 0.71s | 6.9x | 46x | ✓ 12799920000 |

\* `binary_trees_gc.wren`'s Wren time includes explicit `System.gc()` calls
(no Phalcom equivalent exists — dropped, see the file's header comment), so
this row is not apples-to-apples; Phalcom's number here is really closer to
plain `binary_trees.ph` (1.25s vs 1.32s, consistent).

`for.wren` was the outlier at 144x and is now 13.6x — the gap was allocation
and object size (cuts 001/002), not the list protocol, which is why no
`List`-specific unit was ever needed. `map_string`'s 46x → 6.9x collapsed the
same way. The suite now sits in a **4–14x** band with `for` and
`string_equals` at the top; the remaining spread, not any single row, is what
Tier 3 (U-IC) should be sized against.

**Re-measure with** `benchmarks/vm/compare-wren.py` (best-of-N, output-verified
against Wren, per-row slowdown), or run a single row by name.

## Porting notes (surface gaps hit, translated around)

- **No range-literal parser production.** `..`/`...` are lexed
  (`DotDot`/`DotDotDot`) but never consumed by an expression production —
  `for (i in 0...n)` is a hard parse error. Every range-based `for` below is
  a `while`-counter instead. (Same finding as skynet/mandelbrot.)
- **No postfix `[]` index operator**, read or write — only `[a, b, c]` list
  *literal* syntax exists. `list[i]` → `list.at(i)`; `map[k] = v` →
  `map.at(k, put: v)`; `map[k]` → `map.at(k)`.
- **`{}` is the empty-block literal, not empty-map** (spec §6 — Phalcom's
  own doc comment says so explicitly). `var map = {}` → `var map = Map.new()`.
- **Multi-line list literal parser bug**, found porting `map_string.ph`:
  `[a, b\n]` (no trailing comma before the newline-then-`]`) is a hard parse
  error; `[a, b,\n]` (trailing comma) parses fine. Every multi-line list in
  `map_string.ph` carries an added trailing comma to route around it. Looks
  like `parse_comma_exprs` skips newlines after a comma but not before the
  closing-bracket check — worth a real fix, not just a workaround.
- **`this` → `self`.** Phalcom's self-reference keyword.
- **`is` → `extends`** for inheritance; bare `super(args)` constructor call
  → `super.new(args)` (Phalcom dispatches constructors by selector, so the
  super initializer needs naming).
- **`%(expr)` → `\(expr)`** string interpolation (ADR-0022).
- **`System.clock` unimplemented** (pending fixture) — timing done via shell
  `time`, not in-language; every `elapsed:` print line is dropped.
- **No `System.gc()`** — dropped from `binary_trees_gc.ph` (see caveat above).
- Everything else — `_field` instance vars, `@constructor
new(...)`, getters
  (`value => _state` / `value { _state }`), setters (`name=(value) { }`),
  `super.propName` (no-paren super property read), closures, `Fiber.new{}`/
  `.call()`, `while`, string `+` concat, `List.new().add(...)`, `Map`'s
  `.at`/`.at(_,put:)`/`.remove`/`.includes` — ported with **zero** semantic
  change from the Wren source.
