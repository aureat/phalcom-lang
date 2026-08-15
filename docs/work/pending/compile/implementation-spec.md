# U-COMPILE — Implementation spec (compile-time & startup, behavior-invariant)

> Companion to [`err-plan.md`](plan.md). **This file supersedes plan.md's file/line refs and resolves
> DEC-COMPILE-A differently** — plan.md's recommended "in-process memoization of the compiled core"
> is **unsound on HEAD**; see §1. Written against HEAD 2026-07-14 (`2b75429`).
>
> Behavior-invariant unit: same bytecode out, same diagnostics, same spans. Any golden diff is a bug.

## 0. Path corrections vs plan.md

| plan.md ref | HEAD |
|---|---|
| `vm.rs:279,309-313` core load | `phalcom-core/src/vm/bootstrap.rs:14` (`VM::new`), `:125` (`run_core_module()` call), `:155-160` (`run_core_module`), `:157`/`:165` (`include_str!`) |
| `chunk.rs:27` `add_constant` | `phalcom-core/src/chunk.rs:27` ✅ still correct |
| `compiler/lib.rs:2060-2063` literal `Str` alloc | `phalcom-core/src/compiler/lib/expr.rs:200` (`self.vm.alloc_string_value(value)`); also `patterns.rs:161`, `loops.rs:231` (compiler-generated messages) |
| `compiler/lib.rs:475-477,516-524` scope resolve | `phalcom-core/src/compiler/lib/scope.rs:131-134` (`resolve_local_in`), `:172-185` (`add_upvalue`) |
| `lexer.rs:216` `replace('_',"")` | `phalcom-ast/src/lexer.rs:216` ✅ still correct |
| `lexer.rs:288` identifier `String` | `phalcom-ast/src/lexer.rs:288` ✅ still correct |

## 1. DEC-COMPILE-A — RESOLVED: cache the **AST**, not the bytecode

**Why plan.md's recommendation cannot work.** A compiled `Chunk`'s constant pool is
`Vec<Value>` (`chunk.rs:9`), and a string literal constant is `Value::Obj(ObjRef)` allocated into
**that VM's heap** at compile time (`compiler/lib/expr.rs:200` → `vm.alloc_string_value`). An
`ObjRef` is an index into one specific `Heap`. Memoizing a compiled core chunk and reusing it in a
second `VM::new` would hand VM #2 handles pointing into VM #1's arena — a silent cross-heap
type-confusion, not a perf win. **Do not cache compiled bytecode across VMs.** (A serialized blob has
the same problem plus a bytecode-format dependency; it is a real future unit, not this one.)

**What IS cacheable:** the **lex + parse** result. The AST (`phalcom-ast`) holds no heap handles and
no VM state — it is a pure function of the source `&'static str` from `include_str!`. Cache it:

```rust
// phalcom-core/src/vm/bootstrap.rs
use std::sync::OnceLock;

/// `core.ph`'s parsed AST, lexed and parsed once per process.
///
/// Compilation is deliberately NOT cached: a compiled `Chunk`'s constants are
/// `ObjRef` handles into the compiling VM's heap, so a shared chunk would leak
/// handles across VMs. The AST holds no heap state and is safe to share.
static CORE_AST: OnceLock<Ast> = OnceLock::new();
```

`run_core_module` (`bootstrap.rs:155-160`) then compiles from `CORE_AST.get_or_init(...)` instead of
re-lexing `include_str!("../../core/core.ph")`.

**Invalidation is free and you must say so:** `core.ph` is `include_str!`-embedded, so its contents
are fixed at build time and any edit rebuilds the crate. There is **no stale-cache hazard and no
content hash is needed.** (plan.md's content-hash requirement applies only to the on-disk-blob design
that §1 rejects.) plan.md's "editing core.ph busts the cache" test is therefore **not applicable** —
`cargo test` after an edit already recompiles. Say this explicitly in the return shape.

**Shape work required:** `compile_closure` (`bootstrap.rs:158`) currently takes `source: &str`. Split
the existing pipeline into `parse(source) -> Ast` and `compile_ast(module, &Ast) -> closure`; the
compiler must accept `&Ast` (borrowed), not an owned one. If the compiler consumes/mutates the AST,
**STOP and report** — cloning the AST per `VM::new` gives back most of the win and needs a decision.

**Honest expectation:** this wins only when a process constructs more than one `VM` (the test suite
does — it is a real test-time win). A single-`VM` CLI run still pays one lex+parse. If U-BENCH shows
first-startup compile cost dominating for the CLI, that is the serialized-blob unit, deferred.

## 2. Change 2 — `add_constant` dedup

`chunk.rs:27-30` pushes unconditionally. A method that mentions `"foo"` or the selector `#at(_)`
three times gets three pool entries, and each string literal additionally allocates a fresh heap
`Str` (`expr.rs:200`).

**Do:** put the dedup in the **compiler**, not in `Chunk::add_constant`.

Why: dedup must key on **content**, and `Value::Obj(ObjRef)` compares by handle
(`Value`'s `PartialEq` derive, `value/mod.rs:36`). `Chunk` cannot see the heap, so it cannot compare
two `Str` handles by content. The compiler can (it holds `self.vm`).

Add to `FunctionState` (`phalcom-core/src/compiler/lib/state.rs:32`), i.e. per chunk being emitted:

```rust
/// Content-keyed constant-pool dedup: a repeated literal/selector reuses its
/// existing pool index instead of appending a duplicate entry.
constant_index: HashMap<ConstKey, u16>,
```

with

```rust
#[derive(PartialEq, Eq, Hash)]
enum ConstKey {
    Number(u64),      // f64::to_bits — matches `Value`'s own Hash (value/mod.rs:309)
    Bool(bool),
    Symbol(Symbol),
    Str(String),      // by CONTENT, never by ObjRef
    /// Never deduped: closure templates, and anything else the compiler must
    /// keep as a distinct pool slot.
    Unique,
}
```

Route the compiler's `add_constant` (`scope.rs:20`) through it. Rules:

- **`Str` deduping is the point** — dedup *before* calling `vm.alloc_string_value`, so a repeated
  literal allocates one heap `Str`, not N. Restructure `expr.rs:200` to check the map first.
- **`Value::Nil` must never be deduped or interned** — it is the private sentinel; if it appears in a
  pool at all, leave it on the `Unique` path.
- **Closure templates (`Bytecode::Closure`'s constant) are `Unique`.** Two textually identical block
  literals are different templates; collapsing them is a correctness bug.
- Dedup is **per-`FunctionState`** (per chunk). Do not share the map across functions — pool indices
  are chunk-local.
- **Spans must not move.** `add_constant` does not touch `chunk.spans` (`chunk.rs:27`), so this is
  safe by construction — but re-read `chunk.rs:22-30` and confirm before you commit. A dedup that
  corrupts diagnostics fails the errors-as-UX bar.

## 3. Change 4 (cheap part) — `scan_number` alloc

`phalcom-ast/src/lexer.rs:216`: `let cleaned = slice.replace('_', "");` allocates a `String` for
**every** number literal, even `1`. Do:

```rust
let parsed = if slice.contains('_') {
    slice.replace('_', "").parse::<f64>()?
} else {
    slice.parse::<f64>()?
};
Ok(Token::Number(parsed))
```

Keep the `?` error type identical (`LexicalError::InvalidFloat` via `From<ParseFloatError>`) — the
rustdoc at `lexer.rs:203-207` documents that behavior; do not change it.

**DEC-COMPILE-C stays DEFERRED**: do **not** touch `lexer.rs:288` (`Token::Identifier(slice.to_string())`).
Borrowing `&'input str` threads a lifetime through `Token`/`ast.rs`/`parser.rs` — a large
`phalcom-ast` refactor and its own unit. If you find yourself editing `token.rs` or `ast.rs`,
**STOP and report**.

## 4. Change 3 — hashmap scope resolution: MEASURE FIRST, probably skip

`scope.rs:133`: `(0..func.num_locals).rev().find(|&i| func.locals[i].name == name)`.

Two reasons this is likely not worth doing:
1. `num_locals` is per-function and small (a handful); a linear scan over ≤16 `u32` compares beats a
   hash.
2. **`rev()` is load-bearing** — it implements shadowing (the *innermost/latest* declaration wins).
   A `HashMap<Symbol, usize>` must therefore be a **stack per name** (`HashMap<Symbol, Vec<usize>>`),
   pushed in `add_local` (`scope.rs:114`) and **popped in `end_scope`** (`scope.rs:91`) in exact
   lockstep with the `locals` truncation. Get the pop wrong and a shadowed variable resolves to the
   dead slot — a silent wrong-variable bug that goldens may not catch.

**Do this:** run `scripts/vm_bench.rs` / time `cargo build`-side compilation on the largest `.ph` in
`examples/` + `core.ph`. If local resolution is not visible in the profile, **skip Change 3 and
report it skipped with the measurement.** Only implement it if the number justifies the shadowing
risk.

## 5. Write-set (STOP-and-report if outside)

- `phalcom-core/src/vm/bootstrap.rs` — `CORE_AST` `OnceLock`, `run_core_module` (§1).
- `phalcom-core/src/compiler/lib/state.rs` — the `constant_index` field (§2).
- `phalcom-core/src/compiler/lib/scope.rs` — `add_constant` routing (§2); Change 3 only if §4 says so.
- `phalcom-core/src/compiler/lib/expr.rs` — `:200` dedup-before-alloc (§2).
- `phalcom-core/src/compiler/mod.rs` / `lib/mod.rs` — the `parse` / `compile_ast` split (§1).
- `phalcom-ast/src/lexer.rs` — `:216` only (§3).
- **Floor: +0.** No surface change, no ADR.

## 6. Build order — commit each green step

1. **§3** `scan_number` — one line, prove golden-clean. Commit.
2. **§2** constant dedup — golden-clean **plus** the pool-count assertion (§7). Commit.
3. **§1** AST cache — golden-clean, `cargo test` wall-clock recorded before/after. Commit.
4. **§4** measure; implement or write down the skip. Commit (or don't).

## 7. Tests

- **Primary gate: zero golden diff** across the whole corpus, every step.
- **§2 needs a direct assertion**, not a vibe. Add to `phalcom-core/tests/` (near the existing
  compiler/invariant tests): compile a source with a literal repeated N times (`"x"` ×5, `1` ×5) and
  assert `chunk.constants.len()` equals the distinct count. Follow whatever access the existing
  `tests/invariants.rs` uses to reach a compiled chunk; if none exists, a `bin/phalcom` disasm-based
  check (`bin/phalcom/disasm.rs`) is acceptable — say which you used.
- **§2 string-identity check:** two occurrences of `"abc"` must yield the **same** `ObjRef` (one heap
  `Str`). Assert it — that is the allocation win, and `"abc" == "abc"` already compares by content
  (`value/mod.rs:250`), so identity-sharing changes nothing observable.
- **§1 needs no cache-bust test** — see §1 (`include_str!` is build-time). Record the reasoning.
- `cargo build && cargo test && cargo clippy --workspace` green; `cargo doc` clean
  ([[rust-doc-mandatory]]); WORKTREE-VERIFY each SHA ([[clean-checkout-verify-each-commit]]).

## 8. Return shape

commit SHAs · **DEC-COMPILE-A resolved as AST-cache-not-bytecode-cache** + the cross-heap `ObjRef`
reason + `cargo test` wall-clock delta · constant dedup landed + the distinct-count assertion + the
same-`ObjRef`-for-equal-literals assertion + confirmation closure templates stay `Unique` and spans
are untouched · `scan_number` cut · Change 3 implemented **or skipped with the measurement** ·
confirmation DEC-COMPILE-C (lexer `&str` borrow) was **not** started · zero golden diff · any
`unsafe` (expect none) · floor delta (0) · verify + `cargo doc` tails · write-set confirm.
