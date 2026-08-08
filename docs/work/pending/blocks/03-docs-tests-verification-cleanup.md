# Task 3 — Documentation, Tests, Verification, and Cross-Project Cleanup

**Project:** Phalcom  
**Repository:** `aureat/phalcom-lang`  
**Baseline inspected:** `main` at `32580cc6599ccebd31447d69e2557bfcacdcd95f`  
**Status:** Implementation-ready  
**Depends on:** Task 1 parser syntax and Task 2 executable-source migration  
**Primary areas:** normative docs, guides, pending specs, LSP fixtures, fuzzing, full test matrix, repository hygiene  
**Must preserve:** historical records unless actively normative; existing AST/runtime architecture; current selector semantics; B.3b as separate future work

---

## 1. Objective

Finish the closure/trailing-closure migration at the repository level.

This task does not introduce new language semantics. Its purpose is to make every **current** specification, guide, fixture, test, future implementation instruction, and verification gate agree with the new source grammar:

```phalcom
|| { ... }

|x| expression

|x, y| {
  ...
}
```

and with trailing send syntax such as:

```phalcom
items.map |item| {
  transform(item)
}

items.any where: |item| {
  predicate(item)
}

result.match
  ok: |value| {
    ...
  },
  err: |error| {
    ...
  }
```

It also records the important consequence that removing bare brace closures resolves the parser-level blocker for collection phase B.3b, without implementing B.3b itself.

---

## 2. Normative language rules docs must reflect

Current documentation must consistently state the following.

### 2.1 Closure literal grammar

```text
ClosureLiteral
  := "|" ClosureParameters? "|" ClosureBody

ClosureParameters
  := Identifier ("," Identifier)*

ClosureBody
  := Expression
   | "{" BlockStatements "}"
```

Examples:

```phalcom
|| 1

|x| x + 1

|x, y| {
  x + y
}
```

### 2.2 Runtime identity

A closure literal evaluates to the existing runtime `Block`.

`Block` remains a concrete `Function`.

Do not describe a new `Closure`/`Lambda` runtime class.

### 2.3 Expression and braced bodies

Both are valid ordinary closure forms:

```phalcom
|x| x + 1

|x| {
  x + 1
}
```

### 2.4 Trailing-closure body restriction

Unparenthesized trailing closures must use a braced body:

```phalcom
items.map |x| {
  x + 1
}
```

This is not valid trailing syntax:

```phalcom
items.map |x| x + 1
```

Use either:

```phalcom
items.map(|x| x + 1)
```

or:

```phalcom
items.map |x| {
  x + 1
}
```

The braced requirement is what protects bitwise `|` parsing.

### 2.5 `=>` meaning changed

`=>` is no longer an anonymous-block header token.

It remains valid where the language still uses method/getter expression bodies, e.g.:

```phalcom
size => _size
```

Documentation must not state that block headers and method expression bodies share `=>`.

### 2.6 Bare brace callable literals are gone

This no longer means a closure:

```phalcom
{ doSomething() }
```

Use:

```phalcom
|| {
  doSomething()
}
```

Likewise:

```phalcom
{ x => x + 1 }
```

becomes:

```phalcom
|x| {
  x + 1
}
```

### 2.7 Existing semantics remain unchanged

Docs must preserve and explicitly state:

- closure parameters retain current binding semantics;
- final-expression value semantics are unchanged;
- lexical capture semantics are unchanged;
- explicit `self` capture remains unchanged;
- direct `blk(...)` invocation remains unchanged;
- arity semantics remain unchanged;
- non-local return semantics remain unchanged;
- dead-home-frame behavior remains unchanged.

Do not document implicit `self` yet.

---

## 3. Required current-spec files

At minimum inspect and migrate:

```text
docs/spec/current/blocks.md
docs/spec/current/functions.md
docs/spec/current/control-flow.md
docs/spec/current/iteration.md
docs/spec/current/error-handling.md
docs/spec/current/values-and-absence.md
docs/spec/current/selectors.md
docs/spec/current/concurrency.md
```

Do not limit the search to this list. Use repository-wide searches to find current normative claims and examples.

---

## 4. Rewrite `docs/spec/current/blocks.md`

The current block specification defines forms such as:

```phalcom
n => n * 2
{ acc, n => acc + n }
{ System.print("hi") }
```

and states that `=>` has the same "yields" role in block headers and method expression bodies.

That is obsolete after this migration.

Rewrite the forms section around:

```phalcom
|n| n * 2

|acc, n| {
  acc + n
}

|| {
  System.print("hi")
}
```

The rewritten specification must state:

- closure literals evaluate to `Block`;
- `Block` remains a concrete `Function`;
- parameters are existing immutable lexical bindings under current semantics;
- final-expression value behavior is unchanged;
- non-local return semantics are unchanged;
- captures are unchanged;
- direct `blk(...)` invocation remains unchanged;
- both expression and braced bodies are supported;
- unparenthesized trailing closures require braced bodies;
- parenthesized closure arguments may use expression bodies.

Remove any claim that `=>` is the block header syntax.

Mention `=>` only in the context where expression-bodied members/getters still use it.

---

## 5. Required guide files

Inspect and migrate at minimum:

```text
docs/guide/blocks.md
docs/guide/collections.md
docs/guide/control-flow.md
docs/guide/concurrency.md
docs/guide/errors.md
```

Guide examples should prefer the canonical new syntax and should teach the distinction between:

```phalcom
items.map(|x| x + 1)
```

and:

```phalcom
items.map |x| {
  x + 1
}
```

When showing multiple labeled callbacks, use the ratified trailing-label form where pedagogically appropriate:

```phalcom
result.match
  ok: |value| {
    ...
  },
  err: |error| {
    ...
  }
```

---

## 6. Active pending specifications

Inspect:

```text
docs/work/pending/collections/D*.md
docs/work/pending/transition-1/*.md
```

These documents describe future executable work and therefore must not instruct future coding agents to reintroduce retired block syntax.

### 6.1 Transition-1 Task 3

Update old examples such as:

```phalcom
xs.map { x =>
  helper(x)
}
```

to:

```phalcom
xs.map |x| {
  helper(x)
}
```

Do not implement implicit `self` in this task. Only migrate its examples and prose to the new closure spelling.

### 6.2 Collection D alignment

Pending collection D already treats closure syntax such as:

```phalcom
collection.map |value| { ... }

collection.filter |value| { ... }

collection.each |value| { ... }

collection.map(indexed: |index, value| { ... })

collection.find(where: predicate)
```

as canonical, while labeled trailing syntax was pending.

After this parser work, update the future-facing material to use forms such as:

```phalcom
collection.find where: |value| {
  ...
}
```

where that matches the ratified API design.

Do not implement collection D in this task.

---

## 7. B.3b brace-literal consequence

Update:

```text
docs/work/pending/collections/B.3-brace-literals-and-atomic-map-construction.md
```

The document currently states that B.3b is blocked because `{}`/Set literals conflict with existing brace block literals.

That parser blocker is eliminated by this closure migration.

Update its status text to the equivalent of:

```text
B.3b is no longer blocked by closure/block syntax. The closure migration
removed bare `{...}` callable literals. Empty-Map and Set brace classification
remain pending implementation as their own collection phase.
```

Do not implement:

```phalcom
{}         // empty Map
{a, b, c}  // Set
```

in this task.

The status change is documentation of an unblocked dependency, not completion of B.3b.

---

## 8. Historical documentation policy

Do not mass-edit:

```text
docs/forge/archive/**
historical as-built records
retired ADRs
old investigation logs
```

when they are accurately describing the syntax that existed at the time.

Historical records should remain historical.

For accepted/current ADRs that still make normative claims about old block spelling, prefer a clear supersession/update note rather than silently rewriting historical reasoning.

The rule is:

- current guidance: migrate;
- future implementation guidance: migrate;
- historical record: preserve unless it is incorrectly presented as current.

---

## 9. LSP impact and cleanup

Because the AST remains:

```rust
Expr::Block
```

the existing LSP walkers should not require structural semantic changes.

Audit:

```text
phalcom-lsp/src/index.rs
phalcom-lsp/src/semantic_tokens.rs
phalcom-lsp/src/completion.rs
```

Migrate:

- embedded source snippets;
- parser fixtures;
- completion fixtures;
- semantic token tests;
- other tests containing legacy closure syntax.

### 9.1 Semantic token scope

`|` is already lexically classified as an operator.

For this initial closure migration, it is acceptable for closure delimiters to retain that lexical semantic-token classification.

Do **not** build a parser-aware contextual punctuation-coloring subsystem solely for this feature.

Closure-specific coloring can be a later presentation improvement.

---

## 10. Fuzz dictionary

Update:

```text
fuzz/phalcom.dict
```

with closure-focused seeds:

```text
"|| {}"
"|x| x"
"|x| { x }"
"|x,y| { x }"
".map |x| { x }"
"where: |x| { x }"
```

Keep existing `|` seeds for operator/bitwise usage.

The parser now has meaningful context sensitivity around the same token, so fuzz coverage is important.

---

## 11. Parser/lexer verification matrix

Task 1 should already contain focused parser tests. This task must ensure they are retained and integrated into the full repository verification.

### 11.1 Lexer proof

Verify:

```phalcom
|| {}
```

lexes as:

```text
Pipe
Pipe
LBrace
RBrace
```

and:

```phalcom
|x, y|
```

uses ordinary existing tokens.

Do not introduce `DoublePipe`.

Do not reinterpret `||` as logical OR.

### 11.2 AST shape tests

Retain tests for:

```phalcom
const f = |x, y| x + y
```

with:

```text
Expr::Block
params = ["x", "y"]
expr_body = true
```

and:

```phalcom
const f = |x, y| {
  x + y
}
```

with:

```text
expr_body = false
```

and zero-parameter:

```phalcom
const f = || {
  1
}
```

### 11.3 Labeled trailing closures

Retain:

```phalcom
items.any where: |item| {
  item.valid
}
```

with:

```text
method = "any"
label = Some("where")
expr = Expr::Block
```

### 11.4 Multiple labels

Retain:

```phalcom
result.match
  ok: |v| { v },
  err: |e| { e }
```

with ordered labels:

```text
["ok", "err"]
```

### 11.5 Positional + labeled

Retain:

```phalcom
predicate
  .ifTrue || { 1 }
  ifFalse: || { 2 }
```

with:

```text
labels = [None, Some("ifFalse")]
```

---

## 12. Mandatory parser ambiguity regressions

These tests are non-negotiable because `|` is both closure delimiter and bitwise operator.

### 12.1 Bitwise stays bitwise

```phalcom
a | b
```

must remain:

```text
Expr::Binary(BitOr)
```

Likewise:

```phalcom
a | b | c
```

must remain a left-associative bitwise chain.

Critically:

```phalcom
obj.flags | mask | other
```

must not become a trailing closure call.

### 12.2 Structurally complete trailing closure wins

```phalcom
obj.map |value| {
  value
}
```

must become a `map(_)` send with one `Expr::Block` argument.

### 12.3 Ordinary closure primary remains usable

A representative syntax regression:

```phalcom
something | (|x| x)
```

must remain syntactically valid according to ordinary expression precedence, irrespective of runtime type validity.

---

## 13. Newline/chaining regression matrix

These layouts must parse as a single expression.

### 13.1 Repeated member chain with positional closures

```phalcom
numbers
  .map |value| {
    value * 2
  }
  .filter |value| {
    value > 10
  }
```

### 13.2 Repeated labeled trailing closures across chained sends

```phalcom
users
  .removeAll where: |user| {
    user.expired
  }
  .any where: |user| {
    user.expired
  }
```

### 13.3 Multiple labeled arguments

```phalcom
result.match
  ok: |v| {
    v
  },
  err: |e| {
    e
  }
```

### 13.4 Negative newline boundary

```phalcom
foo
|x| {
  x
}
```

If `foo` is a complete independent expression and no member-send/trailing context exists, the next-line closure must not attach to `foo`.

---

## 14. Parenthesized-equivalence matrix

For each important trailing shape, retain a comparison with its parenthesized equivalent.

### Positional

```phalcom
items.map |x| {
  x
}
```

vs:

```phalcom
items.map(|x| {
  x
})
```

### Labeled

```phalcom
items.any where: |x| {
  x
}
```

vs:

```phalcom
items.any(where: |x| {
  x
})
```

### Multiple labeled

```phalcom
result.match
  ok: |v| { v },
  err: |e| { e }
```

vs:

```phalcom
result.match(
  ok: |v| { v },
  err: |e| { e }
)
```

Require semantic equality for:

- base method;
- argument count;
- argument order;
- labels;
- `Expr::Block` parameter/body shape.

Range equality is not required.

---

## 15. Compiler/runtime verification matrix

Retain and/or migrate tests proving:

### Capture

```phalcom
let x = 10
const f = |y| {
  x + y
}

f(2)
```

Expected: `12`.

### Mutable upvalue

Use the existing legal mutable capture fixture.

### Nested capture

```phalcom
const outer = |x| {
  |y| {
    x + y
  }
}
```

Then invoke the returned closure.

### Arity

```phalcom
const f = |x, y| x + y
```

Wrong argument counts must retain existing failure semantics.

### Direct invocation

```phalcom
const f = |x| x + 1
f(2)
```

must continue using the existing callable protocol.

### Non-local return

Migrate and retain current behavior.

### Escaped non-local return

Migrate the existing dead-home-frame fixture and retain its expected error.

### Explicit `self`

```phalcom
const f = || {
  self.value
}
```

must continue resolving lexically.

Implicit `self` is not part of this migration.

---

## 16. Sacred inliner verification

Verify the new syntax still reaches sacred recognition.

At minimum:

```phalcom
true.ifTrue || {
  1
}
```

```phalcom
false.ifFalse || {
  1
}
```

```phalcom
true
  .ifTrue || {
    1
  }
  ifFalse: || {
    2
  }
```

Also retain current source-level `and`, `or`, and `whileTrue` cases.

Existing override/deopt tests must remain green.

The recognizer must still see:

```rust
Expr::Block(_)
```

for literal closure arguments.

No semantic recognizer rewrite is expected.

---

## 17. Synthetic block regressions

Internal compiler/parser code synthesizes `Expr::Block` for:

- `if`;
- `while`;
- `??`;
- `?.`;
- lazy `and`;
- lazy `or`;
- other current sacred/lazy desugarings.

Run existing regressions around all of them.

Do not convert internal AST construction into new source-level abstractions.

Only migrate source-facing comments.

Example comment migration:

```text
desugars to `a.orElse { b }`
```

should become something such as:

```text
desugars conceptually to `a.orElse(|| { b })`
```

or the corresponding trailing form.

---

## 18. Repository-wide migration audit

Before declaring completion, search the entire repository.

Candidate commands:

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

Search terminology:

```sh
rg -n 'trailing block|block literal|arrow block|unbraced arrow|braced block'
```

Every match must be classified.

Do not blindly edit:

- legitimate method/getter `=>`;
- control-flow braces;
- declaration braces;
- Map literals;
- historical docs.

The desired final state is: **no executable `.ph` file relies on retired anonymous closure syntax.**

---

## 19. Full landing sequence

This task corresponds to the final repository-wide phases of the larger migration.

### Phase E — Documentation and development-spec migration

Update:

- current language specification;
- guides;
- active pending implementation specs;
- transition-1 Task 3 examples;
- collection D examples where applicable;
- B.3b blocker status;
- LSP fixtures;
- source-facing comments;
- fuzz dictionary.

Do not rewrite archival history indiscriminately.

### Phase F — Verification

Run focused gates:

```sh
cargo test -p phalcom-ast
cargo test -p phalcom-core
cargo test -p phalcom-lsp
```

Then run the full repository verification:

```sh
./scripts/verify.sh --full
```

Because this feature changes parser interpretation of an existing operator token, also run:

```sh
./scripts/verify.sh --fuzz
```

before merge when the fuzz toolchain is available.

---

## 20. Final acceptance criteria

The entire closure/trailing-closure migration is complete only when every item below is true.

### Syntax

All of these parse:

```phalcom
const a = || 1
const b = |x| x + 1
const c = |x, y| x + y

const d = || {
  1
}

const e = |x, y| {
  x + y
}
```

### Ordinary closure arguments

```phalcom
items.map(|x| x + 1)
items.map(indexed: |i, x| { ... })
```

parse.

### Positional trailing closure

```phalcom
items.map |x| {
  x + 1
}
```

parses as `map(_)`.

### Labeled trailing closure

```phalcom
items.any where: |x| {
  predicate(x)
}
```

parses as `any(where:)`.

### Multiple trailing closures

```phalcom
result.match
  ok: |v| {
    ...
  },
  err: |e| {
    ...
  }
```

parses as one `match(ok:err:)` send according to the existing selector encoder.

### Mixed positional/labeled

```phalcom
predicate
  .ifTrue || {
    ...
  }
  ifFalse: || {
    ...
  }
```

produces the same sacred selector shape as the parenthesized equivalent.

### Chaining

```phalcom
numbers
  .map |x| {
    x
  }
  .filter |x| {
    x.valid
  }
```

is one expression.

### Bitwise safety

```phalcom
a | b
a | b | c
obj.flags | mask | other
```

retain binary bitwise semantics.

### Old anonymous syntax rejected

```phalcom
x => x + 1
{ x => x + 1 }
```

no longer produce closures.

### B.3a Map syntax retained

Currently landed association Map literals still parse exactly as before.

### Runtime equivalence

Capture, arity, direct calls, non-local returns, escaped closures, and explicit `self` behavior remain unchanged.

### Sacred optimization equivalence

Closure literals passed to sacred selectors remain `Expr::Block` and retain fast-path eligibility.

### Bootstrap

```text
phalcom-core/core/core.ph
```

parses and boots after migration.

### Repository hygiene

No executable `.ph` file relies on retired closure syntax.

### B.3b status

The pending collection spec clearly states that closure/block syntax no longer blocks B.3b, while empty-Map and Set classification remain unimplemented.

### LSP

LSP tests/snippets use current syntax, and no structural AST walker changes were introduced merely for closures.

### Fuzzing

Closure seeds are present while bitwise `|` seeds remain.

### Full verification

```sh
./scripts/verify.sh --full
```

passes.

When available:

```sh
./scripts/verify.sh --fuzz
```

also passes.

---

## 21. Explicit non-goals

Do not include:

- function/closure type annotations;
- `->` function-type implementation;
- parameter type annotations;
- closure destructuring;
- rest/default/named closure parameters;
- async closures;
- capture modifiers;
- capture-by-value syntax;
- new closure allocation strategies;
- escape analysis;
- zero-capture singleton optimization;
- runtime `Closure` class rename;
- `Block` class rename;
- new callable bytecodes;
- implicit `self`;
- collection D implementation;
- Set literal completion;
- empty-Map `{}` decision/implementation;
- generalized whitespace-sensitive `|` lexing;
- automatic trailing closures after `?.`;
- arbitrary non-closure unparenthesized arguments;
- parser-aware closure-delimiter semantic coloring.

---

## 22. Final architectural invariant

The repository should finish with exactly this model:

```text
source
  |x, y| { ... }
        │
        ▼
phalcom-ast
  Expr::Block(BlockExpr {
      params: ["x", "y"],
      body: ...,
      ...
  })
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

and:

```phalcom
users.any where: |user| {
  user.expired
}
```

must remain nothing more exotic than:

```text
Expr::MethodCall {
    object: users,
    method: "any",
    args: [
        Argument {
            label: Some("where"),
            expr: Expr::Block(...),
        }
    ]
}
```

Everything beneath these two existing AST abstractions stays unchanged.

That invariant is the final merge criterion for the migration.
