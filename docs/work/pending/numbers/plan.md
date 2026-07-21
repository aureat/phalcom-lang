# U-NUMBERS — full numeric tower implementation plan

> **For agentic workers:** implement in dependency order. Every completed unit must pass its
> local gate before the next starts. Existing files `01`–`06` are concise handoff cards; this is
> the normative implementation plan.

**Goal:** land the ratified `Number` → `Int` / `Float` tower, exact integers, numeric syntax,
protocols, keys, and numeric diagnostics without weakening their contracts.

**Architecture:** replace the flat immediate `Value::Number(f64)` with immediate small `Int` and
`Float` arms plus heap-owned `LargeInt`. Keep arithmetic as ordinary sends, but re-home the
primitive floor on concrete classes. Preserve typed literal information through the AST and
compiler; propagate emitted instruction ranges into the existing error/traceback path.

**Tech stack:** Rust workspace (`phalcom-ast`, `phalcom-core`), `num-bigint`, hand-written lexer /
Pratt parser, arena heap and GC, `.ph` core protocol, golden language tests.

---

## Role and authority

This plan implements, but does not reinterpret:

- [Numeric tower](../../../spec/current/numbers/numeric-tower.md), including its representation,
  promotion, floor-census, and GC-root gates.
- [Float protocol](../../../spec/current/numbers/float-protocol.md) and
  [numeric text and errors](../../../spec/current/numbers/text-and-errors.md).
- [PDR-0025](../../../pdr/0025-numeric-tower-residue-rulings.md),
  [PDR-0026](../../../pdr/0026-numeric-literals.md), and
  [PDR-0027](../../../pdr/0027-float-protocol-and-explicit-narrowing.md).

The behavior is already ratified. This plan creates no new public numeric rule. When implementation
evidence contradicts an anchor, stop and amend the design record; do not silently choose a new
semantic branch.

## Scope boundary

| In scope | Explicitly not in this landing |
|---|---|
| `Int` / `Float` / heap `LargeInt`, class split, primitive-floor amendment | Strict Float rejection at collection/index boundaries |
| Exact literals, radix/exponent grammar, `~/`, right-associative `**` | Serialization or interchange format |
| Arithmetic, canonical equality/hash, Map/Set numeric-key coherence | Extended math library or public Float total order |
| Constructors, canonical rendering, resource guards | Measured tuning of numeric-budget defaults |
| Numeric runtime errors and source-backed traceback carets | Literal diagnostic wording catalogue beyond stable code/span |

Deferred work remains in [numeric follow-ups](../../deferred/numeric-followups.md). In particular,
`expect_index` must accept new `Int` values but continues accepting integral `Float` values until
its dedicated boundary-tightening unit.

## Preconditions — verify on HEAD

Do not trust historical line numbers or census totals. Before U-NUMBERS-01, record the result of:

| Claim | Verification |
|---|---|
| Flat numeric arm and its exhaustive consumers still exist | `rg -n 'Value::Number|Number\\(' phalcom-core/src phalcom-core/tests` |
| `Number` is the only numeric core class and owns its current bindings | inspect `core_classes.rs`, `universe/primitives.rs`, and `core/core.ph` |
| Lexer still loses integer/Float distinction | inspect `Token::Number` and `Lexer::scan_number` |
| Parser/operator and selector sites are synchronized | inspect `token.rs`, `lexer.rs`, `parser.rs`, `ast.rs`, and compiler operator emission |
| Compiler constants root heap objects across compilation and execution | trace constant-pool ownership before accepting a `LargeInt` literal |
| Existing traceback pipeline can carry instruction ranges | trace `Bytecode` ranges through `RuntimeError::Raise` and `diagnostics/caret.rs` |
| Live primitive floor and core-class rows | run `floor_census_matches_installed_bindings` and inspect its constants; test is source of record |

Current working tree may contain unrelated concurrent changes. Do not stage or edit outside the
numbers write-set below. Re-run the affected precondition whenever a dependency changes.

### External dependency — U-CTOR-4 allocation substrate

U-NUMBERS starts only after U-CTOR-4 is green. It supplies the single allocation path
`Class >> new -> new_` and its `native_repr` rule. U1 adds `Number`'s abstract-class metadata to
that path; the check order is **abstract class before native representation**. This prevents a
second allocator design and makes every inherited/reflection allocation route agree.

The legacy static `Number.new` remains temporarily while U1–U3 establish representation and
arithmetic. U4 deletes that override and installs the strict `Int.new` / `Float.new` primitives.
Thus `Number.new()` becomes `#abstractClass` through the shared allocator, while the immediate
numeric classes receive their explicit constructors. Do not claim the final constructor contract
before U4's re-homing gate is green.

## Non-negotiable invariants

1. `Value::Int` is canonical for every integer that fits `i64`; `Object::LargeInt` never stores a
   representable small value. One `normalize(BigInt) -> Value` is the only LargeInt constructor.
2. `Number` is a normal abstract class: allocation fails on every path, while lookup/reflection
   remain normal. `Int` and `Float` are concrete subclasses.
3. Exact-int work never round-trips through `f64`. Any Float operand selects the binary64 path,
   except `~/`, which returns an exact `Int` or fails.
4. Equal numeric values hash the same. Public `==` leaves NaN unordered; internal Map/Set key
   equivalence gives NaN lookup coherent behavior without changing public equality.
5. Numeric resource exhaustion, including allocating powers/shifts, becomes `#numericLimit`,
   never host OOM text or a panic.
6. Runtime numeric failure with an available source range produces the existing structured error
   plus innermost primary caret. Source-less native/REPL frames must not invent locations.

## Dependency order

`U-CTOR-4 → U1 → U2 → U3 → U4 → U5 → U6`. No parallel implementation lanes: each later unit
relies on value arms, class handles, constructor ownership, or instruction ranges introduced
earlier.

### U-NUMBERS-01 — value model, heap, and class allocation

**Owns:** `Value::Int(i64)`, `Value::Float(f64)`, `Object::LargeInt(BigInt)`, normalization,
GC tracing, `Number < Object`, `Int < Number`, `Float < Number`, and allocator abstractness.

**Required changes:**

- Pin `num-bigint` in workspace dependencies; add only conversion support actually used.
- Replace, never alias, `Value::Number`; repair every exhaustive semantic match deliberately.
- Add explicit no-child tracing for `LargeInt`; ensure object/class/type/render/equality hooks map
  it to surface `Int`.
- Add `int_class` / `float_class` handles, bootstrap rows, core declarations, class-row invariants,
  and independent `toString` override-epoch flags for Int and Float.
- Enforce `Number` abstractness in shared allocation metadata, before allocation. A selector-level
  override is insufficient.

**Exit gate:** shared allocation records and enforces `Number` abstractness; `i64::MIN.negated`
can become a traced LargeInt; nonnumeric behavior stays unchanged. Final `Number.new()` and
concrete numeric-constructor behavior is U4's gate because the legacy static override has not yet
been removed.

### U-NUMBERS-02 — typed literals and power syntax

**Owns:** literal token payloads, PDR-0026 grammar/errors, `~/`, `**`, AST/bytecode ranges.

**Required changes:**

- Carry integer digits plus radix (for oversized values) separately from binary64 Float literals;
  compiler, not `phalcom-ast`, constructs `BigInt`.
- Make malformed literal recognition atomic: one `numeric.literal` diagnostic and primary span over
  the entire lexeme.
- Add `~/` at multiplicative precedence, including operator-selector and `super` spelling. Keep
  bare `~` invalid and do not introduce `~/=`.
- Lex `**` before `*`; make it a selector/operator, never `**=`. Parse
  `power := postfix [ "**" unary ]` so it is right-associative and unary has Python binding.
- Preserve the operator range through bytecode emission; it is U5's caret anchor.

**Exit gate:** radix, separators, decimal exponents, huge integers and negative-syntax fixtures
parse with their specified ASTs and spans; `2 ** 3 ** 2`, `-2 ** 2`, `2 ** -2`, and
`(-2) ** 2` have specified parses. Runtime power behavior and its caret assertion belong to U3
and U5 respectively.

### U-NUMBERS-03 — exact Int/LargeInt operations and resource guards

**Owns:** centralized promotion/coercion, exact integer arithmetic, floor division/modulo,
bitwise operations, shifts, exact power, and allocation/work limits.

**Required changes:**

- Split `primitive/number.rs` into concrete Int/Float ownership, sharing one coercion helper only;
  retain an `i64` checked-arithmetic fast path before BigInt promotion.
- Implement `%` with floor-compatible sign on the exact path; retain `fmod` behavior for Float.
  `/` always takes binary64 semantics. `~/` rejects zero, non-finite operands, and non-finite
  quotients, then normalizes its exact floor result.
- Use exponentiation by squaring, estimate result bits before allocation, and enforce shift/
  exponent/heap work budgets deterministically.
- Prove compile-time LargeInt constants are rooted through compiler constant-pool lifetime. Until
  this proof and forced-GC test are green, do not enable those literal constants.

**Exit gate:** boundary promotion/demotion, divisor signs, zero errors (including `0 ** -1`),
negative exponents, forced-GC temporaries/constants, and hostile shifts/powers are covered. No
numeric path panics.

### U-NUMBERS-04 — Float protocol, constructors, text, and keyed collections

**Owns:** PDR-0027's Float bindings, explicit narrowing, text grammar/rendering, numeric equality
and hash, `hash -> Int`, and Map/Set key behavior.

**Required changes:**

- Install ten Float-native protocol bindings and two concrete `**` bindings; keep derivable Int
  identities/predicates in `core.ph`. Finite Float narrowing converts directly to BigInt then
  normalizes; NaN/infinities fail.
- Implement the exact cross-representation equality/hash canonicalization, including integral
  finite floats above `2^53`; update both Phalcom `hash` and Rust-side `Hash for Value`.
- Keep public NaN unordered; introduce only the internal key relation/canonical hash Map and Set
  require. `send_hash` accepts Int only and reports `#invalidHash` for all other values.
- Implement strict constructors and byte-offset diagnostics, canonical shortest-roundtrip Float
  text, special spellings, signed zero, and overflow-to-infinity.
- Reject `0 ** negative` explicitly before host power; all other Float power behavior follows
  binary64.

**Exit gate:** Float classification/narrowing boundaries, NaN/signed-zero/infinity/subnormal cases,
Int/Float equality and hash law, Map/Set NaN and cross-class keys, text accept/reject tables, and
sampled text round trips pass.

### U-NUMBERS-05 — structured numeric errors and tracebacks

**Owns:** one numeric-error construction path, source ranges for numeric operations/conversions,
and golden runtime/JSON diagnostics.

**Required changes:**

- Map every condition in `text-and-errors.md` to its fixed `Error.kind` and message template;
  reuse existing Error/raise machinery rather than adding native exception classes.
- Emit/carry ranges for binary operators, conversion calls and arguments, shifts, bit indices, and
  allocating powers. Do not derive a range from rendered text.
- Reuse the current traceback renderer with a primary label on operator or argument, optional
  secondary input label only for shift/index errors. Lexer/parser failures stay diagnostics.

**Exit gate:** human fixtures assert caret/source/operator, including `0 ** -1`; JSON fixtures
assert kind, message, frame location, and primary range; source-less native and REPL cases have
no fabricated caret.

### U-NUMBERS-06 — census, conformance, and landing verification

**Owns:** integration-only edits after U1–U5 are green.

**Required changes:**

- Recompute rather than copy primitive-floor arithmetic: record removal of Number bindings,
  concrete split additions, `NEW_NUMERIC_POWER`, and `NEW_FLOAT_PROTOCOL`; update core-class rows
  and floor-census record in the same commit.
- Audit selector/operator inventories and all compiler-minted numeric constants/counts.
- Synchronize only status/docs demonstrably changed by shipped implementation; do not turn deferred
  items into scope.

**Release blockers:** missing LargeInt root proof; a panic/raw-host numeric error; a source-backed
numeric runtime error without caret; or equal Map/Set numeric keys with unequal hashes.

## Write-set

| Area | Primary files | Purpose |
|---|---|---|
| Dependencies/value/heap | root `Cargo.toml`; `phalcom-core/Cargo.toml`; `value/{mod.rs,render.rs}`; `heap/{object.rs,mod.rs,class.rs}`; `vm/gc.rs` | value arms, BigInt ownership, tracing, display/hash/class semantics |
| Core classes/primitive floor | `universe/{core_classes.rs,mod.rs,primitives.rs,invariants.rs}`; `vm/bootstrap.rs`; `core/core.ph`; `primitive/{mod.rs,int.rs,float.rs,number.rs}` | classes, allocator metadata, split registrations, protocol |
| Syntax/compiler | `phalcom-ast/src/{token.rs,lexer.rs,ast.rs,parser.rs}`; `phalcom-core/src/compiler/lib/{expr.rs,patterns.rs,scope.rs}` | typed literals, operators, selector spelling, spans, constants |
| Direct `Value` consumers | `chunk.rs`; `compiler/lib/{expr.rs,mod.rs,patterns.rs}`; `error.rs`; `primitive/{block.rs,boolean.rs,bytes.rs,list.rs,map.rs,mod.rs,number.rs,object.rs,resource.rs,set.rs,string.rs,tuple.rs}`; `universe/mod.rs` | repair every former `Value::Number` match, construction path, and conversion deliberately |
| Collections/errors | `primitive/{map.rs,set.rs,list.rs}`; `error.rs`; existing diagnostics/VM range path only where required | key contract, transitional indexing, error labels |
| Tests/docs | focused AST/core tests; `tests/lang.rs`; `tests/lang/{numbers,runtime-errors,compile-errors}`; `tests/{gc.rs,invariants.rs}`; floor/status docs proven stale by landing | behavior, GC, census, diagnostics |

Do not create a general numeric utility module, public total-order API, serialization format, or
collection-boundary migration as incidental refactors.

## Test and verification contract

Run only the unit's named lane while iterating. After U6, run these exact gates:

1. `cargo test -p phalcom-ast`
2. `cargo test -p phalcom-core floor_census_matches_installed_bindings`
3. `cargo test -p phalcom-core --test lang numbers` (add the `numbers` positive-fixture lane in
   `tests/lang.rs`); `cargo test -p phalcom-core --test lang compile_errors`; and
   `cargo test -p phalcom-core --test lang runtime_errors`
4. `cargo test -p phalcom-core --test gc` for forced-GC LargeInt constants and temporaries; name
   numeric property tests with the `numeric_` prefix and run
   `cargo test -p phalcom-core numeric_`
5. `cargo fmt --check`; `cargo test`; `cargo clippy --workspace`; `./scripts/verify.sh`; and
   `git diff --check`

For any new golden fixture, prove the harness sees it: intentionally corrupt its expected output,
confirm the focused lane fails, then restore it. Record executed commands and results in the
implementation handoff; do not claim an unrun green suite.

## What this must not preclude

- Future niche/NaN-box representation: preserve the `Value` API boundary; do not expose tags.
- Inline caches and late binding: all numeric operations remain ordinary sends; `**` and `~/` are
  selector-encoded, not magic opcodes.
- Future strict index boundaries: retain a marked, transitional integral-Float `expect_index` arm
  until its dedicated unit removes it.
- Future budget tuning: require guards now but do not commit defaults without measurements.
- Future serialization/total order: canonical display and internal key equivalence must not become
  a public wire format or NaN-bit commitment.

## Handoff rules

- Work one unit per commit. Never merge U(n+1) over a red U(n) gate.
- Use a clean throwaway worktree for final verification of each committed unit; concurrent sessions
  may modify unrelated paths in the main tree.
- After final code/docs edits, run `graphify update .` and inspect the scoped numeric result only.
- Keep `01`–`06` as small execution cards until their work is complete; this plan is their common
  requirement source, not a duplicate semantic specification.
