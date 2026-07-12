# U6 — Absence → `Option` + `let`/`var` bindings (as-built)

- **Status:** ✅ Landed — `318e752` / `51f56e4` / `aa8bb8b` (build order per `U6-plan.md`; the two code commits close the two Invariant-4 sentinel-leak paths, the third is docs/ADR-0021).
- **Realizes:** [ADR-0007](../../../../adr/0007-option-as-abstract-with-some-none.md) (absence → Option), [ADR-0014](../../../../adr/0014-let-and-var-bindings.md) (`let`/`var`), [ADR-0021](../../../../adr/0021-no-truthiness-enforcement.md) (no-truthiness enforcement); spec [values-and-absence §2](../../values-and-absence.md) (nil is private), [§3](../../values-and-absence.md) (Absence is Option), [§3.4](../../values-and-absence.md) (`??`/`?.`), [§3.5](../../values-and-absence.md) (no truthiness). Consumes [ADR-0010](../../../../adr/0010-tagged-value-enum.md) (private `Value::Nil`).
- **Reviewer gate:** **ON** (load-bearing — can corrupt the value model / leak the private sentinel). BLOCKed once on inlined≠non-inlined body result; fixed in `51f56e4`; re-verified → **PASSED**.

## Mission
Make absence a first-class `Option` — no surface `nil`, no truthiness. Add `let`/`var`
binding forms (`var x` with no initializer reads `None`); bootstrap the
`Option`/`Some`/`None` kernel classes over U1's **private** `Value::Nil` sentinel, which
must never leak to user code (Invariant 4); desugar `??`/`?.` to Option sends.

## Surface / behavior
- **No surface `nil`.** The `nil` literal/keyword is gone; user code expresses absence
  only through `Option`. `Some` (one field `_value`) and `None` are concrete subclasses of
  abstract `Option`. `None` is a single shared, identity-comparable, zero-allocation
  singleton bound as a global; `Some(v)` is constructed via the explicit static send
  **`Some.new(x)`** — deliberately **no bare `Some(x)` call-construction syntax**.
- **The one eliminator** that leaves Option-world is **`match(some:none:)`** (keyword-labelled
  selector spelling, not a comma form). Combinators (`map`/`flatMap`/`filter`/`orElse`/…)
  were deferred to U-STD, built as `.ph` over `match`.
- **`let`/`var` (ADR-0014):** `let` is immutable, `var` mutable. `var x` with no initializer
  reads as `None`. Compile errors: `let x` with no initializer, reassigning a `let`, and any
  surface `nil`.
- **`??` / `?.`** desugar in the parser (short-circuiting): `a ?? b ≡ a.orElse { b }`,
  `opt?.foo ≡ opt.map { x => x.foo }`; a chained `?.` stays in `Option` and the first `None`
  short-circuits the rest.
- **No truthiness (ADR-0021):** `Option` never implements the boolean-branch protocol, so a
  non-`Bool` condition is a hard **runtime** type error (via U5's `GuardBool`, ADR-0018);
  additionally the compiler rejects **syntactically-literal** Option conditions (`if (None)`,
  `if (Some.new(…))`) at compile time.

```phalcom
var x                          // no initializer → reads None
let y = Some.new(42)
y.match(some: { v => v }, none: { 0 })   // → 42
let z = x ?? 7                 // x is None → 7
```

## Implementation
- **`value.rs`** — the private `Value::Nil` sentinel exists **only** as an allocator/storage
  default; no public constructor, surfaced to `None` at every read boundary.
- **`vm.rs`** — `none_value()` helper returns the shared `None` singleton. `Bytecode::Nil`
  now pushes the `None` singleton (was the raw sentinel). Bare `return`, fall-off-end,
  uninitialized `var`/module-slot reads all surface `None`.
- **`compiler/lib.rs`** — `let`/`var` mutability tracking (locals + a compile-time
  immutable-global set, since globals span statements); `var x`→`None`; reject `let x`
  no-init and assign-to-`let`; `??`/`?.` desugar; `is_option_literal` / `branch_condition_of`
  reject literal Option conditions. `51f56e4` fixed value-less block/method bodies: they
  emitted a bare `Return` that surfaced slot 0 (the receiver / `self`); the compiler now
  tracks whether the last statement leaves a value and pushes `Bytecode::Nil` before the
  fallback `Return` when it does not — restoring inlined ≡ non-inlined (both yield `None`).
- **`primitive/{nil,boolean,block,class,system}.rs`** — surface-reachable primitives
  (`bool_if_true`, `bool_if_false`, `block_while_true`, `system_class_print`,
  `Class::superclass`) return the `None` singleton, not the raw sentinel (`318e752`). `Some`
  construction asserts its argument is never the sentinel.
- **`core.ph`** — `Option`/`Some`/`None` skeletons + the `None` global (combinator bodies →
  U-STD).
- **`phalcom-ast`** (`token.rs`/`lexer.rs`/`ast.rs`/`parser.rs`) — `var`, `??`, `?.` tokens
  and lexing; `LetBinding` mutability; `Expr::Nil` and the `nil` keyword removed.

## Invariants & tests
- **Invariant 4 (sentinel non-leak):** value-level `invariants.rs` test asserts no program
  can print or compare the raw sentinel, and `Value::Nil` never appears inside a `Some`
  (`Some.new(None)` is legal — only the *raw sentinel* is barred). `318e752`/`51f56e4` add
  the leak-repro coverage.
- **`absence` goldens:** `absence_iftrue_empty_body_is_none`, `absence_iftrue_false_branch_is_none`,
  `absence_print_result_is_none`, `absence_root_superclass_is_none` (`318e752`);
  `absence_empty_block_call_is_none`, `absence_empty_method_body_is_none`,
  `absence_match_empty_none_branch_is_none` (`51f56e4`) — all four value-less-position repros
  print `<None instance>`.
- **Green gate:** `verify.sh` exit 0; `cargo doc --workspace --no-deps` clean.

## Deviations & deferrals
- **Deliberate deviations (pre-authorized defaults):** `Some.new(x)`, not `Some(x)` — no
  call-construction syntax; `match(some:none:)` selector spelling; bare `return` ≡ `return None`.
- **ADR-0021 refines spec §3.5:** "compile error" is realized as "compile error where
  statically detectable + hard runtime type error otherwise" — Phalcom is dynamically typed
  with no flow analysis, so the general `if(opt)` case cannot be a pure compile error.
- **Deferred:** captured-`let` reassignment through an upvalue compiles to `SetUpvalue` with
  no diagnostic (the compile check is syntactic, current-fn + module only) →
  [deferred-work §4](../../deferred-work.md); the `if(opt)` literal-only diagnostic carries no
  source span; `Some` niche-encoding into `Value` deferred pending GC/benchmarks
  ([deferred-work §1](../../deferred-work.md)).

## Sources
- Forge: `U6-plan.md` (folded into this spec; see git history), [STATE.md](../../../../forge/archive/phase2/STATE.md) "U6 — LANDED".
- Commits `318e752`, `51f56e4`, `aa8bb8b`.
- Code: `phalcom-core/src/{value,vm,compiler/lib,module,bytecode}.rs`,
  `phalcom-core/src/primitive/{nil,boolean,block,class,system}.rs`, `core/core.ph`,
  `phalcom-ast/src/{token,lexer,ast,parser}.rs`.
