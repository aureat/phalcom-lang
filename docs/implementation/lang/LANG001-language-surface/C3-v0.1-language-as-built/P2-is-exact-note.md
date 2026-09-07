# U-IS — `is` / `is!` / `is not` type-test operators

Status: **PLANNED** (dispatch-ready). Sibling of **U-NEG**; fire **after U-NEG** (both touch
parser.rs + core.ph → serialize, single-writer). Contends `phalcom-ast` (parser) +
`phalcom-core/core/core.ph`.

## Role
Add surface operators for the two membership questions, desugaring to two overridable ("magic")
methods:
- **kind-of** `x is T` → `x.is(T)` (subclass-inclusive chain walk).
- **exact** `x is! T` → `x.isExactly(T)` (live direct-class identity).
- **negation** via the `not` keyword: `x is not T` → `(x.is(T)).not`; `x is! not T` →
  `(x.isExactly(T)).not`.

Only **two magic methods** (`is(_)`, `isExactly(_)`); negation is a compile-time `.not` wrap, not
a selector. `isA(_)` (landed U-CORE-1) becomes an alias over `is(_)` for back-compat.

## Spec anchor — [is-tests.md](../../../spec/current/is-tests.md) (Status: Proposed; Fork A ratified in-note)
AUTHORITATIVE for the surface table, the magic-method bodies, the `is not` compound-operator
disambiguation, the proxy P-2 policy, and the precludes list. Grounds on:
[object-model.md §8](../../../spec/current/object-model.md) (`isA`, metaclass tower),
[selectors.md §2](../../../spec/current/selectors.md) (the `#`-adjacency precedent reused for `is!`).

## Current state (verified)
- `Token::Is` exists (token.rs:73) and lexes (`"is" => Token::Is`, lexer.rs:280). **No `is!`
  token** — strict is formed by parser adjacency (below).
- `Token::Not` lexes (283) — the `is not` particle consumes it directly (works even pre-U-NEG,
  where `not` is not yet a prefix op).
- Binary-precedence table (parser.rs:2205, `binary_op`): `or`=1, `and`=2, equality=3. `Token::Is`
  is **absent** — `is` is not a plain binary op (it has affixes, is non-chaining, RHS-is-class), so
  parse it as a **dedicated comparison-tier step**, not a `binary_op` entry.
- `Object#isA` current body (core.ph:9–16) walks `self.class` up the superchain to `None`. This
  body **moves to `is(_)`** verbatim (plus the RHS-is-Behavior guard); `isA` becomes `=> self.is(cls)`.
- Internal `isA` callers (core.ph 389/493/556/605 — List/Map/Set/Tuple `==` guards) keep working
  unchanged via the alias.

## Parsing (desugar into EXISTING AST — no new node)
At the comparison tier (parser.rs ~1242 region, same precedence as `==`/`!=`, non-associative,
non-chaining), after parsing the left `shift`-level expr, if the next token is `Token::Is`:
1. **Strict suffix** — peek for a **contiguous** `Token::Bang` (span end of `is` == span start of
   `!`, the [selectors.md §2](../../../spec/current/selectors.md) adjacency test). Contiguous → strict
   (`isExactly`); consume the `Bang`. A space before `!` is **not** strict (and, post-U-NEG, a
   lone `!` in RHS position is a parse error — good).
2. **`not` particle** — if the next token is `Token::Not`, consume it as the **negation particle**
   (greedy: `not` right after `is`/`is!` is ALWAYS the particle, never a prefix on the RHS —
   Python's `is not`). Record negate=true.
3. **RHS** — parse one `shift`-level expression (the class expr).
4. **Non-chaining** — if another `is` follows, emit a compile error (left result is `Bool`).
5. **Desugar** into existing nodes:
   - base send: `MethodCallExpr { recv: lhs, selector: is | isExactly, args: [rhs] }`
   - if negate: wrap in `Expr::Unary(UnaryExpr { op: UnaryOp::Not, expr: base })`.

This reuses the existing send lowering + `UnaryOp::Not` lowering (compiler lib.rs:2142–2152) →
**no compiler edit, no new AST variant, no new bytecode.** `is!` needs no new token — pure parser
adjacency. Verify `MethodCallExpr` field names against ast.rs before writing (U16 touched it).

> Lexer alternative (only if parser-adjacency proves awkward): emit a single `is!` token from the
> lexer like the `#move` selector rule. **Prefer parser adjacency** — keeps the token set minimal
> and mirrors how strict was scoped. Note the choice in the return shape.

## core.ph (over the floor)
```phalcom
class Object {
  // No RHS guard: a non-class cls never matches any c in the chain → false (I-4).
  is(cls) {
    var c = self.class
    while (c != None) { (c == cls).ifTrue { return true }; c = c.superclass }
    return false
  }
  isExactly(cls) => self.class == cls
  isA(cls) => self.is(cls)   // retained alias; U-CORE-1 fixtures keep working
}
```
- Move the existing `isA` body into `is` **verbatim** — no guard line.
- **I-4 ratified = `false`** (non-class RHS returns false via the natural chain walk). Do **not**
  add a `cls.is(…)` guard: it re-enters `is` through the alias and recurses forever, and it would
  target `Behavior`, which is **not bootstrapped** (ADR-0003 designs it; core.ph has only
  `Object`/`Class`/`Metaclass`, and it is absent from `phalcom-core/src/`). A raising variant is
  deferred until a non-recursive native class-predicate exists (would also break floor-0).
- **No `Behavior` reference anywhere in this unit.**

## Write-set (STOP-and-report if outside)
- `phalcom-ast/src/parser.rs` — the comparison-tier `is` step (adjacency + `not` particle +
  non-chaining + desugar).
- `phalcom-ast/src/ast.rs` — **read-only expected** (reuse `MethodCallExpr` + `UnaryExpr`); touch
  only if a field is genuinely missing.
- `phalcom-core/core/core.ph` — `is`/`isExactly` added, `isA` → alias.
- `phalcom-core/tests/` — goldens + graduate any pending is-test fixture.
- `docs/spec/current/is-tests.md` — mark surface IMPLEMENTED; `docs/forge/units/README.md` +
  DEFERRED/STATE as the index requires.
- **Floor: +0** (pure parser desugar + `.ph` methods over the floor; no native primitive). If a
  primitive turns out needed → STOP-and-report per ADR-0019.

## Tests / graduation
- **Positive goldens** (stdout byte-exact) covering the semantics table:
  `3 is Number` → true; `3 is! Number` → false; `3 is! Int` → true; `3 is not Number` → false;
  `3 is! not Number` → true; the `Dog extends Animal` block; **`Point is Class` → true /
  `Point is! Class` → false** (the metaclass-tower discriminator — keep this example).
- **Override goldens:** a `Shape#is(cls)` structural override → `s is Drawable` true and
  `s is not Drawable` false "for free" (proves the single-method negation coupling).
- **Alias regression:** an existing `isA` fixture (`3.is(Number)`) still passes.
- **Non-class RHS** (I-4 = false): `3 is "str"` → `false`; `3 is 4` → `false` (positive golden, not
  an error). Guards the no-guard/no-recursion decision.
- **Negative lane** (`check_negative`): `a is B is C` (chaining) → compile error; `x is` (missing
  RHS) → parse error.
- WORKTREE-VERIFY the batch before commit ([[phalcom-golden-test-lanes]]).

## Open questions carried (do not silently resolve)
- **I-2** super-fallthrough default, **I-3** identity-vs-name exact compare, **I-5** whether
  `is!`/`is! not` earn their keep, **I-6** `match`-guard narrowing. See is-tests.md. Ship the spec's
  shown defaults; note any the reviewer flips. (**I-4 closed = `false`**, above — not carried.)

## Reviewer
ON — parser surface + selector-identity + a security-relevant override/proxy policy. Independent
phalcom-reviewer; writer ≠ approver. Reviewer confirms: desugar reuses existing nodes (no stray
bytecode), `isA` alias intact, non-chaining + non-class-RHS diagnostics present, `is not` particle
is greedy (not a RHS prefix).

## Return shape (implementer)
commit SHA(s) · `is!` adjacency mechanism (parser vs lexer, chosen) · `not`-particle greediness ·
desugar target nodes (MethodCallExpr + UnaryExpr, no new variant) · core.ph `is`/`isExactly`/`isA`
disposition (no guard) · I-4=false non-class goldens · goldens (mark negative + the metaclass
discriminator) · alias regression · floor delta (exp 0) · verify + cargo doc tails · write-set confirm.

## Follow-on
- Resolve I-2/I-3/I-5/I-6 in a later ratification pass; none block this unit.
- `match`-guard integration (I-6) is a separate destructuring unit, NOT this one.
