# Task 2 — Compiler/Runtime Invariants and Executable Source Migration

**Project:** Phalcom  
**Repository:** `aureat/phalcom-lang`  
**Baseline inspected:** `main` at `32580cc6599ccebd31447d69e2557bfcacdcd95f`  
**Status:** Implementation-ready  
**Primary areas:** compiler/runtime audit plus migration of executable `.ph` source  
**Depends on:** Task 1 parser support for `|...|` closures and trailing-closure syntax  
**Must preserve:** existing `Block` / `ClosureObject` runtime machinery, bytecodes, captures, selector encoding, non-local returns, sacred inlining, and callable protocol

---

## 1. Objective

Migrate all executable Phalcom source to the new closure syntax while proving that closure semantics remain exactly those of the existing `Expr::Block` pipeline.

This task must **not** redesign closure execution.

The expected runtime/compiler semantic code changes are zero or minimal. The principal work is:

1. audit existing compiler/runtime assumptions;
2. preserve the invariant that new syntax still produces `Expr::Block`;
3. migrate `core.ph` and all executable fixtures away from legacy anonymous-block syntax;
4. run runtime, compiler, bootstrap, sacred-inliner, and synthetic-block regressions;
5. enable Task 1 to remove the old parser syntax safely.

---

## 2. Runtime architecture that must remain unchanged

The existing AST node is:

```rust
Expr::Block(Box<BlockExpr>)
```

with:

```rust
pub struct BlockExpr {
    pub params: Vec<String>,
    pub body: Vec<Statement>,
    pub expr_body: bool,
    pub range: SourceRange,
}
```

The compiler already lowers this node through `compile_block`, creates a `ClosureObject`, records arity/upvalues, and emits:

```rust
Bytecode::Closure
```

Nested blocks already inherit lexical capture behavior through the compiler's `FunctionState` stack.

The runtime `Block`/`Function` machinery already provides:

```text
arity
name
call(...)
```

with:

- closure invocation;
- argument-count checks;
- home-frame tracking;
- non-local-return handling;
- escaped/dead-home-frame behavior.

Do not add:

```rust
Expr::Closure(...)
Expr::Lambda(...)
Object::Lambda(...)
Bytecode::MakeLambda
```

Do not rename the internal `ClosureObject`.

Do not rename the public/runtime `Block` class.

Architectural invariant:

```text
source
  |x, y| { ... }
        │
        ▼
phalcom-ast
  Expr::Block(BlockExpr { ... })
        │
        ▼
compiler
  compile_block(...)
        │
        ▼
existing ClosureObject
  + existing upvalue descriptors
        │
        ▼
Bytecode::Closure
        │
        ▼
existing Block object
        │
        ▼
existing Function#call protocol
```

This invariant is the central compatibility boundary.

---

## 3. Files to audit but not redesign

Audit these areas for assumptions, stale comments, tests, and source snippets:

```text
phalcom-core/src/compiler/lib/expr.rs
phalcom-core/src/compiler/lib/mod.rs
phalcom-core/src/compiler/inliner.rs
phalcom-core/src/primitive/block.rs
phalcom-core/src/bytecode.rs
phalcom-core/src/vm/*
phalcom-core/src/heap/*
```

Expected semantic implementation changes: **none**.

If any semantic compiler/runtime modification appears necessary merely because the surface syntax changed, stop and re-check the parser implementation first. The parser should still produce the same `Expr::Block` nodes.

---

## 4. `Expr::Block` compilation invariant

Keep the existing compiler branch conceptually unchanged:

```rust
Expr::Block(block_expr) => {
    ...
    self.compile_block(...)
    ...
    self.emit(Bytecode::Closure(...))
}
```

Do not introduce:

- closure-specific new bytecodes;
- special expression-body lowering;
- implicit-return bytecodes;
- a second callable representation.

Expression-body closures such as:

```phalcom
|x| x + 1
```

must already arrive as a `BlockExpr` whose body contains one expression statement and `expr_body = true`.

The compiler's existing final-expression/block-value behavior remains authoritative.

---

## 5. Capture semantics

Do not change capture analysis.

Existing local/upvalue resolution must continue to make:

```phalcom
let offset = 10
const f = |x| {
  x + offset
}
```

behave exactly like the previous equivalent block representation.

Required regression categories:

- immutable captured locals;
- mutable captured locals;
- nested captures;
- closures returned from closures;
- explicit `self` capture.

No capture modifiers or capture-by-value syntax are part of this task.

---

## 6. `return` and home-frame semantics

A source closure is still a runtime `Block`.

Therefore:

```phalcom
method() {
  values.each |value| {
    if (value.bad) {
      return value
    }
  }

  None
}
```

must continue to perform a **non-local return from the lexical home method**, not merely return from the closure activation.

The runtime's existing `home_frame_token` machinery must remain in force.

Escaping closures that attempt non-local return after their home frame is dead must retain the current dead-frame error behavior.

Do not bypass or simplify this machinery.

---

## 7. Explicit `self` only

Do not implement transition-1 Task 3's implicit-`self` behavior here.

Explicit `self` in closures continues to use the existing lexical/upvalue model:

```phalcom
const f = || {
  self.value
}
```

Transition-1 Task 3 later requires member lexical context to propagate into nested blocks. Keeping `Expr::Block` stable is specifically what allows that future work to build on this migration.

Any executable future-spec examples that use the old syntax must be rewritten, but Task 3 semantics themselves remain out of scope.

Example migration:

```phalcom
xs.map { x =>
  helper(x)
}
```

becomes:

```phalcom
xs.map |x| {
  helper(x)
}
```

---

## 8. Sacred inliner invariant

The sacred inliner currently recognizes relevant calls when arguments are literal:

```rust
Expr::Block(_)
```

Do not alter that semantic criterion.

The new syntax must still hit exactly the same recognizer.

For example:

```phalcom
condition.ifTrue || {
  work()
}
```

must be eligible for the same fast path as the old literal-block call.

Likewise:

```phalcom
predicate
  .ifTrue || { a() }
  ifFalse: || { b() }
```

must produce the same argument labels expected by the existing:

```text
ifTrue(_:ifFalse:)
```

selector/inliner logic.

Changing the AST category would silently deopt these constructs; therefore no new closure AST is permitted.

---

## 9. Primary executable source migration

Primary source:

```text
phalcom-core/core/core.ph
```

Also inspect and migrate:

```text
phalcom-core/core/_future.ph
examples/**/*.ph
benchmarks/**/*.ph
phalcom-core/tests/**/*.ph
other executable .ph fixtures
```

The inspected `core.ph` contains extensive legacy block usage, including forms such as:

```phalcom
condition.ifTrue({ ... }, ifFalse: { ... })

condition.ifTrue { ... }

while (...).and({ ... })
```

Migration is mandatory. Once the old parser syntax is removed, bootstrap source must still parse and boot.

---

## 10. Mechanical migration rules

### 10.1 Unbraced expression closure

Legacy:

```phalcom
x => expr
```

New:

```phalcom
|x| expr
```

### 10.2 Parameterized braced closure

Legacy:

```phalcom
{ x => body }
```

New:

```phalcom
|x| {
  body
}
```

### 10.3 Multi-parameter braced closure

Legacy:

```phalcom
{ acc, x =>
  body
}
```

New:

```phalcom
|acc, x| {
  body
}
```

### 10.4 Zero-parameter block used as a value

Legacy:

```phalcom
{ body }
```

New:

```phalcom
|| {
  body
}
```

### 10.5 Legacy trailing block

Legacy:

```phalcom
items.each { item =>
  use(item)
}
```

New:

```phalcom
items.each |item| {
  use(item)
}
```

### 10.6 Parenthesized zero-parameter blocks

Legacy:

```phalcom
condition.ifTrue({ yes() }, ifFalse: { no() })
```

Preferred first-pass mechanical migration:

```phalcom
condition.ifTrue(
  || { yes() },
  ifFalse: || { no() }
)
```

This form is recommended for a first broad pass through `core.ph` because it changes only anonymous callable syntax while preserving the surrounding call structure.

A later readability cleanup may use trailing syntax:

```phalcom
condition
  .ifTrue || {
    yes()
  }
  ifFalse: || {
    no()
  }
```

---

## 11. Migration safety rules

### 11.1 Do not blindly replace braces

A repository-wide regex must **not** rewrite every `{...}`.

Braces also represent:

- class bodies;
- method bodies;
- getter/setter bodies;
- `if` bodies;
- `while` bodies;
- `for` bodies;
- Map literals;
- Record-related syntax;
- other declaration/control-flow bodies.

Use text searches only as a candidate ledger. Classify every occurrence by syntactic context before modifying it.

### 11.2 Do not blindly replace `=>`

`=>` remains valid for method/getter expression bodies:

```phalcom
size => _size

isEmpty => size == 0
```

Only anonymous callable use of `=>` is retired.

This distinction is critical in `core.ph`, which contains many expression-bodied members.

### 11.3 Do not implement B.3b while migrating

Removing bare brace closures unblocks future empty-Map/Set work, but executable migration must not introduce new Set or `{}` semantics.

### 11.4 Do not implement collection D

Collection D already expects the new closure syntax. This task only aligns executable source with the new syntax; it must not implement pending collection APIs.

---

## 12. Migration audit commands

Before and after migration, use repository-wide searches as a candidate ledger:

```sh
rg -n '=>' \
  --glob '*.ph' \
  --glob '*.rs' \
  --glob '*.md'

rg -n '\.ifTrue\s*\{' .
rg -n '\.ifFalse\s*\{' .
rg -n '\.each\s*\{' .
rg -n '\.map\s*\{' .
rg -n '\.filter\s*\{' .
rg -n '\.whileTrue\s*\{' .
```

Also search for terminology in comments and active implementation guidance:

```sh
rg -n 'trailing block|block literal|arrow block|unbraced arrow|braced block'
```

Do not treat every search result as a bug.

Legitimate `=>` member-body syntax and non-closure braces must remain untouched.

---

## 13. Required compiler/runtime regression tests

Even if semantic Rust code does not change, the new source syntax must prove the old semantics survive.

### 13.1 Capture

```phalcom
let x = 10
const f = |y| {
  x + y
}

f(2)
```

Expected result:

```text
12
```

### 13.2 Mutable captured local

Use an existing legal mutable-upvalue fixture and migrate only its source syntax. Verify that mutation and observation behavior are unchanged.

### 13.3 Nested captures

```phalcom
const outer = |x| {
  |y| {
    x + y
  }
}
```

Invoke the returned closure and verify that `x` remains captured.

### 13.4 Arity

```phalcom
const f = |x, y| x + y
```

Calling with one or three arguments must retain the existing `Function` arity error semantics.

### 13.5 Direct invocation

```phalcom
const f = |x| x + 1
f(2)
```

must continue lowering through the current callable/`call` protocol.

### 13.6 Non-local return

Migrate an existing block non-local-return fixture to `|...|` syntax and assert identical behavior.

### 13.7 Escaped non-local return

Migrate the existing `DeadFrameError`/home-frame fixture and retain its expected failure behavior.

### 13.8 Explicit `self` capture

Inside a method:

```phalcom
const f = || {
  self.value
}
```

must keep resolving `self` lexically.

Do not add an implicit-`self` test in this task.

---

## 14. Sacred inliner regression suite

These regressions are high-value because an accidental new AST category would silently disable fast paths.

Verify:

```phalcom
true.ifTrue || {
  1
}
```

and:

```phalcom
false.ifFalse || {
  1
}
```

and paired:

```phalcom
true
  .ifTrue || {
    1
  }
  ifFalse: || {
    2
  }
```

Also migrate and run current `and`, `or`, and `whileTrue` sacred/lazy cases according to current selector spelling.

All existing override/deopt tests must stay green.

The recognizer should continue to see:

```rust
matches!(argument.expr, Expr::Block(_))
```

with no semantic modifications.

---

## 15. Synthetic-block regressions

The parser/compiler internally synthesizes `Expr::Block` for constructs including:

- `if`;
- `while`;
- `??`;
- `?.`;
- lazy `and`;
- lazy `or`;
- other current sacred/lazy desugarings.

These are internal AST constructions, not source block syntax. Do not rewrite them into textual closure syntax inside Rust code.

Run all existing tests around these transformations.

Only update comments that describe observable source syntax.

For example, a comment such as:

```text
desugars to `a.orElse { b }`
```

should become conceptually:

```text
desugars to `a.orElse(|| { b })`
```

or the ratified trailing form.

The Rust AST remains `Expr::Block`.

---

## 16. Optional-send safety audit

Task 1 should use parser-local trailing-target state to prevent trailing closures from accidentally attaching to synthetic calls created by `?.`.

Audit this specifically with executable/parser integration tests.

This source:

```phalcom
obj?.foo |x| {
  ...
}
```

must **not** silently append the closure to the synthetic outer `Option#map` produced by optional-send desugaring.

Automatic trailing closures after `?.` are an explicit non-goal of this migration.

If the source is rejected, it must be rejected rather than miscompiled.

---

## 17. Selector equivalence

Trailing syntax must not create a runtime concept of a "trailing selector".

For:

```phalcom
items.any where: |item| {
  item.valid
}
```

the parser should provide ordinary labeled arguments, and the existing compiler selector encoder remains authoritative.

Parenthesized and trailing equivalents must produce identical selector semantics:

```phalcom
items.any(where: |item| {
  item.valid
})
```

and:

```phalcom
items.any where: |item| {
  item.valid
}
```

Likewise for:

```phalcom
result.match(
  ok: |v| { ... },
  err: |e| { ... }
)
```

versus:

```phalcom
result.match
  ok: |v| { ... },
  err: |e| { ... }
```

No compiler change should be needed to encode these selectors.

---

## 18. Core migration strategy

Use a staged migration rather than a broad formatting rewrite.

### Stage 1 — Inventory

Run the audit searches and classify:

- anonymous arrow closure;
- braced parameterized closure;
- zero-argument callable block;
- trailing callable block;
- legitimate method/getter `=>`;
- control/declaration brace;
- Map brace.

### Stage 2 — Parenthesized closure migration

For dense `core.ph` code, first prefer structural-preserving replacements:

```phalcom
foo({ ... })
```

to:

```phalcom
foo(|| { ... })
```

and:

```phalcom
foo({ x => ... })
```

to:

```phalcom
foo(|x| { ... })
```

This minimizes simultaneous parser/layout changes.

### Stage 3 — Trailing-closure readability cleanup

Where established by the design and clearly readable, rewrite selected method sends to:

```phalcom
items.each |item| {
  ...
}
```

or labeled forms.

Do not make cosmetic trailing-closure conversion a prerequisite for the mechanical migration.

### Stage 4 — Bootstrap gate

Ensure:

```text
phalcom-core/core/core.ph
```

parses and boots entirely under the new syntax before Task 1 removes the old parser forms.

### Stage 5 — Remove legacy parser syntax

Once all executable sources pass, Task 1 can delete old anonymous arrow/bare-brace/trailing-brace parsing.

---

## 19. Source locations to inspect

At minimum:

```text
phalcom-core/core/core.ph
phalcom-core/core/_future.ph
examples/**/*.ph
benchmarks/**/*.ph
phalcom-core/tests/**/*.ph
```

Also search for other executable `.ph` fixtures outside those directories.

Inspect Rust source and tests for embedded Phalcom snippets, especially under:

```text
phalcom-core/src/compiler/*
phalcom-core/src/primitive/*
phalcom-core/src/vm/*
phalcom-core/src/heap/*
```

Only migrate embedded source text and comments where appropriate; do not alter internal synthetic `Expr::Block` construction.

---

## 20. Expected compiler/runtime files to remain structurally stable

The following files should normally need only test/comment/source-snippet edits, if any:

```text
phalcom-core/src/compiler/lib/expr.rs
phalcom-core/src/compiler/lib/mod.rs
phalcom-core/src/compiler/inliner.rs
phalcom-core/src/primitive/block.rs
phalcom-core/src/bytecode.rs
phalcom-core/src/vm/*
phalcom-core/src/heap/*
```

Any proposal to add a new bytecode, closure object, callable protocol, or capture algorithm is outside scope and should be rejected.

---

## 21. Focused verification during migration

Run continuously:

```sh
cargo test -p phalcom-ast
cargo test -p phalcom-core
```

The parser suite catches syntax/AST regressions; the core suite catches bootstrap and semantic regressions.

When source migration reaches LSP fixtures, Task 3 owns the dedicated `phalcom-lsp` gate.

---

## 22. Acceptance criteria for Task 2

Task 2 is complete only when all of the following hold.

### Runtime equivalence

The new syntax preserves:

- capture semantics;
- mutable upvalues;
- nested captures;
- direct closure invocation;
- arity checks;
- non-local return;
- escaped/dead-home-frame behavior;
- explicit `self` capture.

### Compiler invariant

Every source closure still lowers through:

```text
Expr::Block
→ compile_block(...)
→ ClosureObject
→ Bytecode::Closure
→ Block
→ Function#call
```

### Sacred optimization equivalence

Literal closures supplied to sacred selectors remain `Expr::Block` and retain fast-path eligibility.

Existing sacred override/deopt tests remain green.

### Bootstrap

```text
phalcom-core/core/core.ph
```

parses and boots after migration.

### Executable repository hygiene

No executable `.ph` source depends on:

```phalcom
x => ...
{ x => ... }
{ ... } // when used as an anonymous callable
```

### Method/getter arrow syntax preserved

Legitimate forms such as:

```phalcom
size => _size
```

continue to work.

### No accidental semantic expansion

No new behavior is introduced for:

- `?.` trailing closures;
- implicit `self`;
- Set literals;
- empty Map literals;
- collection D;
- function types;
- closure annotations;
- callable bytecodes.

---

## 23. Explicit non-goals

Do not implement:

- new closure AST variants;
- runtime `Closure` or `Lambda` classes;
- `Block` renaming;
- new bytecodes;
- new allocation strategies;
- escape analysis;
- singleton optimization for zero-capture closures;
- implicit `self`;
- function/closure typing;
- closure parameter annotations;
- closure destructuring/rest/default parameters;
- async closures;
- capture modifiers;
- collection D;
- B.3b Set/empty-Map implementation;
- optional-send trailing-closure semantics.

---

## 24. Handoff to Task 3

Once executable code is migrated and core/runtime regressions pass, Task 3 must:

- migrate current docs and guides;
- update active pending implementation specs;
- mark the B.3b closure-syntax blocker as resolved;
- update fuzz seeds;
- migrate LSP snippets/tests;
- run the full repository verification matrix;
- perform a final repository-wide legacy-syntax audit.

Do not declare the migration complete solely because `core.ph` boots.
