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

## Results (release build, this machine, single run — not statistically rigorous)

| Benchmark | Wren | Phalcom | Slowdown | Output matches |
|---|---|---|---|---|
| `fib.wren` (fib(28) ×5) | 0.17s | 1.47s | 8.6x | ✓ 317811 ×5 |
| `for.wren` (1M list build+sum) | 0.05s | 7.18s | ~144x | ✓ 499999500000 |
| `fibers.wren` (100k chained) | 0.02s | 0.24s | ~12x | ✓ 4999950000 |
| `method_call.wren` (2M dispatch) | 0.09s | 0.74s | 8.2x | ✓ true / false |
| `string_equals.wren` (10M compares) | 0.11s | 1.89s | 17x | ✓ 3000000 |
| `binary_trees.wren` | 0.18s | 1.32s | 7.3x | ✓ all check lines |
| `binary_trees_gc.wren` | 0.60s* | 1.25s | 2x* | ✓ all check lines |
| `map_numeric.wren` (2M map ops) | 0.92s | 5.41s | 5.9x | ✓ 2000001000000 |
| `map_string.wren` (~193k-key map) | 0.13s | 6.09s | 46x | ✓ 12799920000 |

\* `binary_trees_gc.wren`'s Wren time includes explicit `System.gc()` calls
(no Phalcom equivalent exists — dropped, see the file's header comment), so
this row is not apples-to-apples; Phalcom's number here is really closer to
plain `binary_trees.ph` (1.25s vs 1.32s, consistent).

`for.wren`'s 144x gap is the outlier — far past every other benchmark's
8-46x band — and points at `List#add`/`List#at` (`rawPush`/`rawAt` primitives)
or the `for`-loop cursor protocol as a specific hot spot worth profiling
before assuming dispatch alone explains it (see
[`benchmarks/concurrency/skynet.ph`](../concurrency/skynet.ph) for the
dispatch-cache analysis — that finding doesn't obviously explain a 144x list
benchmark when method-call-heavy benchmarks land at 8-17x).

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
- Everything else — `_field` instance vars, `construct new(...)`, getters
  (`value => _state` / `value { _state }`), setters (`name=(value) { }`),
  `super.propName` (no-paren super property read), closures, `Fiber.new{}`/
  `.call()`, `while`, string `+` concat, `List.new().add(...)`, `Map`'s
  `.at`/`.at(_,put:)`/`.remove`/`.includes` — ported with **zero** semantic
  change from the Wren source.
