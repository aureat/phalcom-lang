---
id: LANG004.C1.P1
category: LANG
program: LANG004
checkpoint: LANG004.C1
kind: implementation
status: IN_PROGRESS
completion: PARTIAL
verification: BASELINE_BLOCKED
depends_on: []
follows: null
supersedes: null
deferred_reason: null
---

# LANG004.C1.P1 — `&` callable references and `::` associated lookup

## Phalcom `&` Callable References and `::` Associated Lookup — Patch-Grade Implementation Specification

**Repository:** `aureat/phalcom-lang`  
**Prepared against remote branch:** `main`  
**Baseline HEAD:** `288da3f5da322dba60d0be9dfa4b52f5f1505d2f` (`chore(repo): fix formatting drift`, 2026-09-09)  
**Repository visibility:** GitHub remote state was inspected. Local uncommitted working-tree state was not observable. Before implementation, compare the local checkout to this revision and apply the repository-drift procedure below if primary symbols have changed.

## 0. Purpose and scope

This specification implements the ratified surface and semantic split between:

```text
.     ordinary object member lookup / message send
::    associated lookup
&     callable or callable-family capture
```

It covers:

1. associated lookup and associated invocation;
2. singular associated callable references;
3. associated family references;
4. exact and pattern-selected associated family references;
5. singular bound method references;
6. bound method-family references;
7. exact and pattern-selected bound family references;
8. parser/AST, semantic analysis, lowering, compiler, runtime handoff, source indexing, advisory analysis, diagnostics, fixtures, tests, examples, and current language documentation required by that migration.

This specification deliberately does **not** redesign callable typing, generic inference, selector identity, match/destructuring patterns, class-side dispatch, module/package path lookup, import syntax, or runtime module descriptors. Module/package associated lookup and logical paths are covered by the second implementation specification.

### 0.1 Ratified syntax

Associated invocation remains:

```phalcom
Option::Some(42)
```

Associated family capture becomes:

```phalcom
&Option::Some
```

Singular associated reference becomes:

```phalcom
&Option::Some(_)
```

Bound method-family capture becomes:

```phalcom
&object.method
```

Singular bound method reference becomes:

```phalcom
&object.method(_, _, debug)
```

Existing selector-pattern forms remain valid under the new `&` introducer:

```phalcom
&object.method(...)
&object.method...
&object.method(_, _, ...)
&object.method(..., _, param)
```

The same selector-pattern machinery applies when the reference target is associated:

```phalcom
&Owner::member(...)
&Owner::member...
&Owner::member(_, _, ...)
&Owner::member(..., _, param)
```

A selected reference can itself be invoked explicitly:

```phalcom
(&Option::Some(_))(42)
```

The direct form remains preferred and semantically direct:

```phalcom
Option::Some(42)
```

### 0.2 Selector-signature rule

A callable reference is selected by the callable's selector signature.

Correct:

```phalcom
&object.method(_, _, debug)
&Option::Some(_, _, param)
```

The labels `debug` and `param` are selector lanes. They are not value patterns.

Destructuring remains separate and unchanged:

```phalcom
match status {
    Connected(host, agent, debug: _) => agent.dumpInfo()
    (_, _, param1: _, param2: _) => ...
    { label1: _, label2: _ } => ...
    Person(firstName, lastName, age: _) => ...
}
```

Do not reuse destructuring syntax in callable-reference signatures.

### 0.3 Class-side methods remain message sends

A class object remains an ordinary message receiver:

```phalcom
Fiber.new {
    Fiber.yield(42)
}
```

Class-side method capture uses the same bound-reference syntax as any other receiver:

```phalcom
&Fiber.new
&Fiber.new(_)
```

`::` must not become a synonym for class-side dispatch.

### 0.4 Explicitly retired surface

The following current forms are obsolete after this program:

```phalcom
object::method
object::method::(_)
object::method::*
Option::Some::(_)
Option::Some::*
```

The migration must remove the implementation paths that give ordinary receiver-bound behavior to `::`.

### 0.5 Important non-goals

Do not introduce a replacement syntax for any currently supported operator/subscript reference form unless the syntax is already independently ratified in the repository by the time implementation begins. The new ratified surface in this specification is for named callable families. If production source uses an old bound operator/subscript `::` reference, classify that occurrence during migration instead of inventing new punctuation.

Do not invent a new exact-getter capture spelling. Under the ratified surface, bare `&receiver.name` means family capture. Existing old `receiver::name` sites that specifically depended on exact-getter capture must be identified and escalated rather than silently translated.

---

# 1. Repository architecture and evidence

## 1.1 Parser and AST ownership

Primary files:

- `phalcom-ast/src/ast.rs`
  - `Expr::AssociatedLookup`
  - `Expr::AssociatedInvoke`
  - `AssociatedLookupExpr`
  - `AssociatedMemberSyntax`
  - `AssociatedNamedMemberSyntax`
  - `AssociatedNamedMode`
  - `AssociatedResidualSelectorSyntax`
  - `SelectorSpecSyntax`
  - `MethodCallExpr`
  - `GetPropertyExpr`
- `phalcom-ast/src/parser.rs`
  - postfix handling for `Token::ColonColon`
  - ordinary `Token::Dot` member/method parsing
  - `parse_associated_suffix`
  - selector-spec parsing helpers
- `phalcom-ast/src/error.rs`
  - `AssociatedLegacyFamilyEllipsis`
  - `AssociatedExactShapeRequiresSecondSeparator`
- `phalcom-ast/tests/family_selector_syntax.rs`
- `phalcom-ast/tests/parser.rs`

Current AST ownership is too broad: `AssociatedLookupExpr` currently represents both declaration-associated lookup and ordinary receiver-bound method-family/reference behavior. `AssociatedNamedMode` also encodes the now-retired second-`::` and `::*` syntax.

The lexer already has `Token::Ampersand` because `&` exists as an infix operator. No new token is required; the parser needs a prefix-reference production that is unambiguous with infix bitwise use.

## 1.2 Selector identity and pattern ownership

Canonical selector identity is owned by `phalcom-common` selector types and existing parser normalization. Existing structural selector-pattern support already models:

- exact selector slots;
- named labels as identity;
- ellipsis/gap patterns;
- fixed prefix/suffix lanes.

This program must reuse that machinery. It must not implement a second pattern matcher for `&`.

## 1.3 Semantic ownership

Primary files:

- `phalcom-semantic/src/checker/associated.rs`
  - `AssociatedResolution`
  - `AssociatedResolutionKind`
  - `BehavioralFamilySpec`
  - `resolve_associated_owner`
  - `resolve_effective_associated_family`
  - `resolve_bound_behavioral_family`
- `phalcom-semantic/src/checker/expression.rs`
  - `analyze_expression_inner`
  - `synthesize_associated_lookup`
  - `synthesize_associated_invoke`
- `phalcom-semantic/src/types/denotation.rs`
  - `SemanticDenotation`
  - `AssociatedValueDenotation`
  - `CapturedBehavioralMember`
- `phalcom-semantic/src/types/family.rs`
- `phalcom-semantic/src/advisory/analyzer.rs`
- `phalcom-semantic/src/source_index/builder.rs`
- `phalcom-semantic/src/source_index/occurrence.rs`

Current semantic leakage to remove:

- `synthesize_associated_lookup` currently falls back from non-associated receivers to ordinary receiver-bound behavior;
- `AssociatedResolutionKind` currently contains `BoundBehavioralFamily` and `BoundBehavioralInvoke`;
- `AssociatedValueDenotation` currently contains `BehavioralFamily`.

After this migration, associated semantic products must describe associated lookup only. Ordinary method-reference capture must have its own semantic expression path/product, even if it reuses `BehavioralFamilySpec`, family types, and dispatch helpers.

## 1.4 Lowering/compiler ownership

Primary files:

- `phalcom-core/src/modules/semantic_lowering.rs`
  - `LoweringSiteKind`
  - `AssociatedLoweringSpec`
  - `ModuleLoweringSemantics`
  - `build_module_lowering_semantics`
  - `project_associated_resolution`
- `phalcom-core/src/compiler/lib/associated.rs`
  - `compile_associated_lookup`
  - `compile_associated_invoke`
  - family-application compilation
- ordinary expression compiler modules that currently compile `MethodCallExpr`/`GetPropertyExpr`
- `phalcom-core/src/bytecode.rs`
- `phalcom-core/src/vm/dispatch.rs`
- `phalcom-core/src/heap/*family*`
- `phalcom-core/src/heap/associated.rs`
- `phalcom-core/src/vm/associated.rs`

The runtime already has a useful split:

- ordinary receiver-bound behavioral `Family` values use `Bytecode::MakeFamily`;
- associated-family values have a distinct associated-family runtime representation;
- exact variant constructor references can lower through the existing constructor-thunk machinery.

The migration should preserve these runtime abstractions. The main change is which syntax/semantic product reaches them.

## 1.5 Source/editor ownership

AST visitors that must learn the new reference expression include:

- `phalcom-semantic/src/advisory/analyzer.rs`
- `phalcom-semantic/src/source_index/builder.rs`
- `phalcom-semantic/src/source_index/occurrence.rs`

LSP must continue consuming compiler-owned semantic/source products. Do not add an LSP-only interpretation of `&` or `::`.

---

# 2. Source-of-truth declarations

| Concern | Source of truth | Derived consumers | Forbidden competing authority |
|---|---|---|---|
| Selector identity | canonical selector representation in `phalcom-common` | AST, semantic dispatch, runtime sends, reference selection | punctuation/string parsing in semantic/runtime code |
| Selector-pattern matching | existing selector-pattern representation and matcher | bound family capture, associated family filtering | a new `&`-specific matcher |
| Associated member namespace | semantic associated surface for the declaration owner | checker, lowering, compiler | ordinary class method table |
| Ordinary bound method behavior | ordinary dispatch lookup + behavioral `Family` machinery | reference checker, lowering, VM | `AssociatedResolutionKind` |
| Associated invocation | `Expr::AssociatedInvoke` + associated semantic resolution | lowering/compiler | ordinary `.` method dispatch |
| Class-side method dispatch | normal message send to class object | checker/compiler/VM | `::` associated lookup |
| Runtime family behavior | existing `Family` object / `MakeFamily` path | captured bound references | new reference-only runtime object |
| Runtime associated family behavior | existing associated-family representation | associated family references | behavioral `Family` object |
| Source navigation | compiler-owned source index | LSP | LSP AST reinterpretation |

---

# 3. Tempting wrong fixes

Do not:

1. keep `::` runtime-value fallback and merely add `&` as an alias;
2. parse `&Option::Some(_)` as `Unary(&, Call(...))`;
3. turn `&` into a generic unary operator over an already-evaluated call expression;
4. parse `debug` in `&object.method(_, _, debug)` as a binding or ignored pattern;
5. reuse `debug: _` inside reference signatures;
6. route `Fiber.new` through associated lookup because the receiver is a class object;
7. duplicate selector-pattern matching in parser, semantic checker, or VM;
8. make a captured exact bound method eagerly freeze a method implementation if current `Family` semantics intentionally retain live dispatch;
9. collapse behavioral and associated family runtime representations;
10. keep `AssociatedResolutionKind::BoundBehavioralFamily` merely because lowering currently consumes it;
11. make source indexing infer reference semantics from punctuation after semantic analysis has already resolved them;
12. weaken current selector-kind distinctions to simplify parsing;
13. silently translate old exact-getter reference syntax to a whole-family reference;
14. invent new operator/subscript reference syntax inside this patch.

---

# 4. Implementation program / checkpoint map

| Checkpoint | Tasks | Semantic boundary | Required evidence | Deferred evidence |
|---|---:|---|---|---|
| C0 | 1–4 | Parser/AST expresses `&` references independently from associated lookup, while preserving existing selector patterns | AST/parser focused tests; negative old-syntax tests; `cargo check -p phalcom-ast` | semantic/core suites |
| C1 | 5–8 | Semantic analyzer cleanly separates associated lookup from bound callable capture | semantic reference/associated regressions including hostile class-side and runtime-receiver cases | compiler/VM execution |
| C2 | 9–12 | Lowering/compiler routes new semantic products into existing behavioral vs associated runtime machinery | lowering tests + core execution tests for exact/family/pattern capture | broad workspace |
| C3 | 13–15 | Compiler-owned advisory/source-index/editor products understand the new AST and canonical targets | semantic source-index tests + targeted LSP navigation/token tests | fixture-wide migration |
| C4 | 16–19 | Repository source, fixtures, tests, examples, and current docs contain only the new ratified surface | migration searches; affected crate suites; representative examples | final workspace gates |
| Final | — | No obsolete production authority remains and broad delivery gates pass | fmt/check/test/clippy + negative searches | none |

---

# 5. Checkpoint C0 — AST and parser own the new reference grammar

Tasks:
- Task 1 — Introduce a dedicated callable-reference AST expression.
- Task 2 — Parse prefix `&` reference targets and reuse selector-spec/pattern grammar.
- Task 3 — Simplify `::` associated syntax by removing second-`::`/`::*` reference modes.
- Task 4 — Replace parser diagnostics/tests for the retired surface.

Why this is a checkpoint:

The parser and AST must establish an unambiguous semantic distinction before the checker can safely migrate. Partial conversion would leave downstream code guessing whether one `AssociatedLookupExpr` represents lookup, invocation, or receiver-bound capture.

Entry conditions:

- baseline parser builds;
- existing selector signature/pattern parser tests are understood;
- `Token::Ampersand` remains available as the existing lexical token.

Working set:

Primary:
- `phalcom-ast/src/ast.rs`
- `phalcom-ast/src/parser.rs`
- `phalcom-ast/src/error.rs`
- `phalcom-ast/tests/family_selector_syntax.rs`
- `phalcom-ast/tests/parser.rs`

Secondary — inspect only if evidence requires it:
- selector syntax helpers in `phalcom-common`;
- lexer precedence/token tests involving `&`.

Out of scope:
- semantic types;
- VM behavior;
- module/import paths;
- match pattern grammar.

Semantic contract established by this checkpoint:

- `&target` is syntax-level callable/family capture, not a normal unary arithmetic expression.
- `receiver::name(args)` remains associated invocation.
- bare associated lookup remains represented distinctly from reference capture.
- `&receiver.name` means a bound named family capture.
- `&receiver.name(signature-or-pattern)` means bound selector selection/pattern capture.
- `&owner::name` means associated family capture.
- `&owner::name(signature-or-pattern)` means selected/pattern-associated reference capture.
- selector patterns retain their existing semantics.
- labeled selector lanes are encoded as labels, e.g. `method(_, _, debug)`.
- match/destructuring syntax is unchanged.

Semantic risks:

- precedence ambiguity between prefix `&` and infix `&`;
- accidentally parsing reference selector parentheses as call arguments;
- loss of ellipsis prefix/suffix structure;
- classifying a whole family as an exact getter;
- allowing old second-`::` paths to coexist silently.

Hostile cases:

- `a & b` must remain the existing infix expression;
- `&object.method(_, _, debug)` must not produce argument-expression nodes for `_` or `debug`;
- `&object.method(..., _, param)` must preserve both gap and suffix;
- `Option::Some(42)` must remain an invocation, not a reference;
- `&Option::Some(_)` must not invoke;
- `(&Option::Some(_))(42)` must parse as invocation of a reference value;
- `Connected(host, agent, debug: _)` in a match must remain destructuring syntax;
- old `Option::Some::(_)` and `Option::Some::*` must fail with intentional migration diagnostics.

Required evidence:

1. `cargo test -p phalcom-ast --test family_selector_syntax`
   - proves exact/family/pattern reference grammar and retired forms.
2. focused parser tests for `&` precedence and parenthesized reference invocation.
3. `cargo check -p phalcom-ast`
   - proves exhaustive AST caller updates inside the crate compile.
4. negative search in production parser/AST:
   - `rg 'AssociatedNamedMode::(Exact|Family)|second_separator_range|star_range' phalcom-ast`
   - expected: zero production hits after replacement, except intentionally retained historical docs outside production code.

Do not run yet:
- semantic/core workspace suites; they cannot pass until C1/C2 migrate downstream AST consumers.

Escalate immediately if:
- the parser cannot distinguish a reference selector specification without changing canonical selector syntax;
- `Token::Ampersand` prefix parsing would alter infix precedence;
- operator/subscript references appear in non-test production source and require a new user-facing syntax decision.

Checkpoint completion:
- [ ] all four tasks implemented
- [ ] focused parser evidence passes
- [ ] old reference modes removed from production AST/parser
- [ ] selector-pattern AST fidelity preserved
- [ ] no match/destructuring grammar changes
- [ ] implementation state updated
- [ ] no active incident remains

## Task 1 — Introduce a dedicated callable-reference AST expression

Purpose:

Give downstream layers an explicit semantic carrier for capture, independent of associated lookup and invocation.

Risk:
- Semantic: HIGH
- Implementation fanout: multi-file / cross-crate

Owned files and symbols:
- `phalcom-ast/src/ast.rs` — `Expr`, `AssociatedLookupExpr`, `AssociatedNamedMode`, selector syntax structures.

Inspect before editing:
- `Expr::AssociatedLookup`
- `Expr::AssociatedInvoke`
- `MethodCallExpr`
- `GetPropertyExpr`
- `SelectorSpecSyntax`
- range/accessor implementations over `Expr`.

Do not inspect unless evidence forces expansion:
- runtime heap;
- module resolver;
- type solver.

Dependencies:
- existing selector-spec AST structures.

Source of truth:
- syntax must retain receiver, lookup mode (`.` vs `::`), base name, selector specification/pattern, and full source range without reconstructing punctuation later.

Implementation boundary:

STRUCTURAL.

Changes:

1. Add a dedicated reference expression variant, recommended shape:

```rust
pub enum CallableReferenceTarget {
    BoundNamed {
        receiver: Box<Expr>,
        name: String,
        name_range: SourceRange,
        selector: Option<SelectorSpecSyntax>,
    },
    AssociatedNamed {
        receiver: Box<Expr>,
        separator_range: SourceRange,
        name: String,
        name_range: SourceRange,
        selector: Option<SelectorSpecSyntax>,
    },
}

pub struct CallableReferenceExpr {
    pub ampersand_range: SourceRange,
    pub target: CallableReferenceTarget,
    pub range: SourceRange,
}
```

The exact names may follow repository conventions. The critical requirement is that `selector: None` means whole-family capture and `Some(...)` means selected/pattern capture.

2. Do not reuse `AssociatedNamedMode::Getter` to mean family capture.
3. Remove second-separator/star fields once all consumers are migrated.
4. Keep `AssociatedLookupExpr` only for actual associated lookup semantics that are not capture.
5. Keep `AssociatedInvokeExpr` for direct associated calls.

Must not:
- store reference selector syntax as ordinary `PackItem` arguments;
- encode bound vs associated target only as punctuation ranges;
- make the semantic checker infer target kind from receiver type.

Current implementation:
- `AssociatedLookupExpr` conflates associated and behavioral reference behavior.

Target implementation:
- capture has an explicit AST identity; associated lookup remains associated-only.

Edit operations:
1. OPEN `phalcom-ast/src/ast.rs`.
2. FIND `Expr` associated variants.
3. ADD the callable-reference expression/target structures.
4. UPDATE `Expr::range()` and all AST helpers.
5. SIMPLIFY associated named mode after Task 3.
6. SEARCH `Expr::AssociatedLookup` consumers and record them for C1–C3 rather than patching semantics prematurely.

Testing classification:
- no standalone semantic test; parser evidence belongs to C0.

Checkpoint state update:
- record the final AST type names and whether `selector: None` is the whole-family representation.

## Task 2 — Parse prefix `&` callable references

Purpose:

Parse the ratified reference surface while preserving infix `&`.

Risk:
- Semantic: HIGH
- Implementation fanout: local parser with downstream AST fanout

Owned files and symbols:
- `phalcom-ast/src/parser.rs` — prefix expression parser, postfix loop, selector-spec parser.
- `phalcom-ast/src/token.rs` only if expected-token rendering requires it.

Inspect before editing:
- prefix/unary parser entry point;
- infix precedence table containing `Token::Ampersand`;
- `parse_associated_suffix`;
- `parse_selector_spec` / pattern helper names in current HEAD;
- dot member parsing.

Source of truth:
- existing selector-spec parser.

Target grammar, conceptually:

```text
callable_reference :=
    "&" bound_reference_target
  | "&" associated_reference_target

bound_reference_target :=
    expression "." identifier [ selector_spec ]

associated_reference_target :=
    expression "::" identifier [ selector_spec ]

selector_spec :=
    existing exact selector signature
  | existing selector pattern
```

Parser mechanics may require a more constrained target parser to avoid greedily parsing a normal call before the reference marker owns its signature.

Required examples:

```phalcom
&object.method
&object.method(_)
&object.method(_, _, debug)
&object.method(...)
&object.method...
&object.method(_, _, ...)
&object.method(..., _, param)

&Option::Some
&Option::Some(_)
&Owner::member(...)
&Owner::member(..., _, param)
```

Must not:
- interpret the selector specification as an invocation;
- admit `debug: _` in the callable signature;
- alter match/destructuring parser rules;
- make `&` bind so weakly that `&object.method(_)(x)` changes meaning unexpectedly.

Edit operations:
1. FIND the prefix parser that handles unary operators.
2. BEFORE generic unary `&` handling, route prefix `&` into callable-reference parsing.
3. Parse the receiver expression up to the final `.`/`::` reference target without consuming selector-signature parentheses as a call.
4. REUSE exact/pattern selector parser.
5. ENSURE outer postfix invocation can apply after the reference, enabling `(&Option::Some(_))(42)`.
6. ADD range tests for ampersand, receiver, name, selector, and complete expression.

Testing classification:
- focused parser regression required at C0.

## Task 3 — Simplify `::` to associated lookup/invocation only

Purpose:

Remove the old reference-specific second separator and star family surface.

Risk:
- Semantic: HIGH
- Implementation fanout: cross-crate because AST enums change

Owned files and symbols:
- `phalcom-ast/src/ast.rs`
- `phalcom-ast/src/parser.rs`

Current implementation:
- `owner::name::*` selects a family;
- `owner::name::shape` selects a reference;
- `owner::name(args)` invokes;
- runtime receivers may later fall back to behavioral reference behavior.

Target implementation:
- `owner::name(args)` invokes associated callable;
- bare `owner::name` keeps its established non-reference associated value behavior where applicable;
- all capture is introduced with `&`.

Edit operations:
1. REMOVE `AssociatedNamedMode::Family`.
2. REMOVE exact reference modes whose only purpose was the second separator.
3. RETAIN only syntax modes necessary for bare associated value lookup and invocation.
4. DELETE parser branches for `::*` and second-`::` exact narrowing.
5. DELETE parser branch that rejects ellipsis merely because it appears in a reference; ellipsis is now valid after `&`.
6. Ensure `Option::Some(42)` still emits `AssociatedInvokeExpr`.

Testing classification:
- C0 parser evidence.

## Task 4 — Replace migration diagnostics and parser tests

Purpose:

Make syntax failures explain the new surface rather than the superseded one.

Risk:
- Semantic: MEDIUM
- Implementation fanout: local

Owned files and symbols:
- `phalcom-ast/src/error.rs`
- `phalcom-ast/tests/family_selector_syntax.rs`
- `phalcom-ast/tests/parser.rs`

Changes:
- retire diagnostics saying whole-family lookup uses `::*`;
- retire diagnostic saying exact reference needs second `::`;
- add targeted diagnostics where practical:
  - old `owner::name::*` → use `&owner::name`;
  - old `owner::name::(...)` → use `&owner::name(...)`;
  - old runtime `receiver::name` used as reference should no longer parse/resolve as behavioral capture.

Do not overfit diagnostics if parser recovery cannot reliably distinguish intent; syntax rejection with a stable generic message is preferable to a wrong suggestion.

Testing classification:
- focused parser tests.

Suggested commit grouping:
- `refactor(ast): add explicit callable-reference syntax`
- `refactor(ast): make double-colon associated-only`
- `test(ast): lock reference and selector-pattern grammar`

---

# 6. Checkpoint C1 — Semantic ownership split

Tasks:
- Task 5 — Add semantic synthesis for callable references.
- Task 6 — Move bound behavioral-family resolution out of associated resolution.
- Task 7 — Restrict associated resolution to declaration-associated semantics.
- Task 8 — Preserve family application and generic specialization behavior through the new denotations.

Why this is a checkpoint:

The AST migration is not semantically complete until the checker publishes separate products for bound references and associated references. Lowering must consume stable semantic facts rather than infer the split from syntax.

Entry conditions:
- C0 COMPLETE;
- new reference AST available;
- current `BehavioralFamilySpec` and associated-family resolution compile.

Working set:

Primary:
- `phalcom-semantic/src/checker/expression.rs`
- `phalcom-semantic/src/checker/associated.rs`
- `phalcom-semantic/src/types/denotation.rs`
- `phalcom-semantic/src/types/family.rs`
- semantic tests covering associated reification/family application/bound families.

Secondary:
- `phalcom-semantic/src/checker/call.rs`
- dispatch resolution helpers.

Out of scope:
- module/package descriptors;
- import resolution;
- callable type redesign.

Semantic contract:
- a non-associated runtime receiver can never acquire special semantics merely because source used `::`;
- bound `&receiver.name...` uses ordinary dispatch family semantics;
- associated `&Owner::name...` uses associated namespace semantics;
- class-side `&Fiber.new...` is bound behavioral capture on the class object;
- direct `Fiber.new(...)` remains ordinary message dispatch;
- associated direct calls remain associated calls.

Semantic risks:
- retaining the old runtime-value fallback;
- losing receiver-once capture semantics;
- freezing behavioral target implementations instead of existing live-family behavior;
- losing generic owner/application context for associated constructors;
- family applications no longer distinguishing behavioral vs associated representations.

Hostile cases:
- `&instance.method(_)` must not query associated declaration surfaces;
- `&Fiber.new(_)` must use class-side dispatch, not associated lookup;
- `&Option::Some(_)` must use associated variant identity, not a class-side method with the same base;
- a runtime value used with `value::name` must not silently become a bound family;
- family calls after method replacement must retain current live-dispatch behavior for behavioral families;
- selected associated variant constructor reference must preserve exact `VariantId`.

Required evidence:
1. focused semantic tests for exact associated constructor reference, whole associated family, bound exact reference, bound pattern family;
2. hostile test proving `Fiber.new` remains ordinary class-side dispatch;
3. hostile test proving runtime `x::method` no longer falls back to behavioral family;
4. `cargo test -p phalcom-semantic <focused module>`;
5. `cargo check -p phalcom-semantic`;
6. negative search:
   - `rg 'BoundBehavioralFamily|BoundBehavioralInvoke' phalcom-semantic/src/checker phalcom-semantic/src/types`
   - expected: removed from associated resolution/denotation ownership; any retained names must be in the new dedicated reference product and justified.

Do not run yet:
- core VM suite.

Escalate immediately if:
- family application requires `AssociatedValueDenotation::BehavioralFamily` specifically and cannot be generalized without changing runtime semantics;
- moving the product would require altering selector matching laws;
- class-side method resolution currently depends on the associated fallback.

Checkpoint completion:
- [ ] reference synthesis established
- [ ] associated checker has no ordinary behavioral fallback
- [ ] class-side method hostile case passes
- [ ] associated variant identity case passes
- [ ] family application remains behaviorally equivalent
- [ ] implementation state updated
- [ ] no active incident remains

## Task 5 — Add semantic synthesis for `CallableReferenceExpr`

Purpose:

Resolve the reference based on syntactic lookup mode, not receiver-type fallback.

Risk:
- Semantic: HIGH
- Implementation fanout: multi-file

Owned files and symbols:
- `phalcom-semantic/src/checker/expression.rs`
- new semantic reference product location chosen in this task.

Implementation boundary:

STRUCTURAL.

Target flow:

```text
Expr::CallableReference
    ├─ BoundNamed
    │   ├─ analyze receiver once
    │   ├─ normalize exact/pattern selector spec
    │   ├─ resolve behavioral family/candidates using existing dispatch helpers
    │   └─ publish bound-reference denotation/resolution
    └─ AssociatedNamed
        ├─ analyze associated owner
        ├─ resolve associated family
        ├─ optionally select/filter exact/pattern members
        └─ publish associated denotation/resolution
```

Whole family means no selector specification. Pattern reference means an existing normalized selector pattern, not argument values.

Must not:
- choose bound vs associated based on “try associated then fallback”;
- evaluate receiver more than once;
- turn failed associated-owner resolution into bound behavior.

## Task 6 — Move behavioral family resolution out of `AssociatedResolutionKind`

Purpose:

Make semantic ownership mirror language semantics.

Risk:
- Semantic: HIGH
- Implementation fanout: cross-crate downstream lowering impact

Owned files and symbols:
- `phalcom-semantic/src/checker/associated.rs`
- `phalcom-semantic/src/types/denotation.rs`

Changes:
1. retain reusable `BehavioralFamilySpec` if it remains the correct selector/pattern value;
2. move `resolve_bound_behavioral_family` to a reference-oriented module or clearly rename its responsibility;
3. introduce a dedicated semantic resolution record for bound callable references;
4. move `CapturedBehavioralMember` and behavioral denotation out of `AssociatedValueDenotation` if downstream consumers do not require that enum;
5. keep associated value denotation limited to actual associated values/families.

Testing classification:
- checkpoint C1.

## Task 7 — Make associated resolution fail closed

Purpose:

`::` must mean associated lookup only.

Risk:
- Semantic: HIGH
- Implementation fanout: local checker, broad behavior

Owned files and symbols:
- `synthesize_associated_lookup`
- `synthesize_associated_invoke`
- `resolve_associated_owner`

Current implementation:
- getter-only `::` on runtime values can warn and continue as ordinary behavioral lookup.

Target implementation:
- associated owner resolution succeeds only for supported associated-namespace owners;
- unsupported runtime receiver produces the normal associated-owner diagnostic/unknown result;
- no “useful warning but keep behavioral semantics” branch remains.

Must not:
- special-case class objects into class-side method dispatch; class-side behavior uses `.`.

## Task 8 — Preserve family application and specialization

Purpose:

Ensure captured values remain callable exactly as before once constructed.

Risk:
- Semantic: HIGH
- Implementation fanout: multi-file

Owned files and symbols:
- family application analysis in `phalcom-semantic/src/checker/call.rs` / associated helpers;
- `FamilyApplicationKind`;
- family operation shapes and denotations.

Changes:
- update detection of behavioral family values to the new reference denotation;
- keep `FamilyApplicationKind::{Associated, Behavioral}` or equivalent distinction;
- preserve associated generic specialization, constructor-local generics, and expected-type inference already implemented;
- do not make this checkpoint a callable-type redesign.

Suggested commit grouping:
- `refactor(semantic): separate bound references from associated lookup`
- `fix(semantic): make double-colon associated-only`
- `test(semantic): cover reference capture boundaries`

---

# 7. Checkpoint C2 — Lowering/compiler route new semantics into existing runtime machinery

Tasks:
- Task 9 — Add lowering-site/product ownership for callable references.
- Task 10 — Remove behavioral variants from associated lowering.
- Task 11 — Compile bound references using existing `MakeFamily`/bound-method machinery.
- Task 12 — Compile associated references using existing associated-family/constructor-thunk machinery and verify runtime behavior.

Why this is a checkpoint:

Parser/checker changes are only executable once lowering attaches the correct semantic product to each source site. The existing runtime split should be reused rather than replaced.

Entry conditions:
- C1 COMPLETE;
- semantic snapshot publishes dedicated reference resolution;
- existing core lowering tests understood.

Working set:

Primary:
- `phalcom-core/src/modules/semantic_lowering.rs`
- `phalcom-core/src/compiler/lib/associated.rs`
- expression compiler dispatch for the new AST node
- `phalcom-core/src/bytecode.rs`
- `phalcom-core/src/vm/dispatch.rs`

Secondary:
- `phalcom-core/src/heap/associated.rs`
- behavioral family heap/accessor files
- `phalcom-core/src/vm/associated.rs`

Out of scope:
- redesign of `Bytecode::MakeFamily`;
- new family object representation;
- import/module runtime.

Semantic contract:
- bound family capture still evaluates receiver once and stores it;
- bound family invocation retains existing live-dispatch semantics;
- associated family capture retains frozen associated candidate/identity semantics already used by associated-family runtime;
- exact variant constructor capture uses existing constructor thunk/direct representation;
- `AssociatedLoweringSpec` no longer owns ordinary bound behavioral behavior.

Semantic risks:
- source-range lowering-site collisions after introducing a new site kind;
- compiler accidentally evaluating receiver twice;
- exact reference compiled as direct invocation;
- behavioral capture compiled through associated family object;
- associated capture compiled through behavioral `Family`;
- stale fallback in standalone compiler path preserving old syntax.

Hostile cases:
- `const f = &makeReceiver().method(_); f(x)` calls `makeReceiver()` once;
- replacing a matching method after bound family construction affects subsequent call as existing runtime semantics require;
- `const Some = &Option::Some; Some(42)` uses associated family execution;
- `const ctor = &Option::Some(_); ctor(42)` constructs the exact variant;
- direct `Option::Some(42)` remains no-reference fast path;
- old `object::method` cannot reach `Bytecode::MakeFamily`.

Required evidence:
1. lowering tests in `phalcom-core/tests/core/language/compiler/lowering.rs` and/or `lowering_scenarios.rs`;
2. associated reification tests in `phalcom-core/tests/core/language/algebraic_data/associated_reification.rs`;
3. behavioral family runtime tests covering exact and pattern references;
4. receiver-evaluated-once regression;
5. `cargo test -p phalcom-core <focused test modules>`;
6. `cargo check -p phalcom-core`;
7. negative search for `MakeBehavioralFamily` / `InvokeBoundBehavioral` in `AssociatedLoweringSpec`.

Do not run yet:
- full workspace.

Escalate immediately if:
- `MakeFamily` semantics differ from the existing live-dispatch laws expected by current family tests;
- an associated constructor thunk cannot be produced without a second-`::` syntax artifact;
- compiler standalone fallback reconstructs semantics that are absent from semantic lowering.

Checkpoint completion:
- [ ] new reference lowering product attached
- [ ] associated lowering behavioral variants removed
- [ ] behavioral Family runtime reused
- [ ] associated-family runtime reused
- [ ] exact associated constructor reference executes
- [ ] receiver-once hostile test passes
- [ ] no active incident remains

## Task 9 — Introduce reference lowering sites/products

Purpose:

Give compiler code an immutable semantic decision for each `&` expression.

Risk:
- Semantic: HIGH
- Implementation fanout: multi-file

Owned files and symbols:
- `LoweringSiteKind`
- `ModuleLoweringSemantics`
- `build_module_lowering_semantics`

STRUCTURAL target shape:

```rust
pub enum CallableReferenceLoweringSpec {
    MakeBehavioralFamily { /* existing spec */ },
    MakeResolvedBoundMethod { /* if exact specialization remains valid */ },
    MakeAssociatedFamily { /* existing descriptor */ },
    MakeVariantConstructorThunk { /* existing data */ },
}
```

Exact enum boundaries should follow inspected downstream compiler needs. The key requirement is that behavioral reference cases no longer live inside `AssociatedLoweringSpec`.

Add a separate map keyed by a dedicated lowering site kind if that is the cleanest repository fit.

## Task 10 — Purge behavioral behavior from associated lowering

Purpose:

Make `AssociatedLoweringSpec` match associated semantics only.

Risk:
- Semantic: HIGH
- Implementation fanout: cross-crate

Remove/migrate:
- `MakeBehavioralFamily`
- `InvokeBoundBehavioral`
- any “ordinary receiver-bound” comments/branches in associated projection.

Preserve:
- singleton loads;
- variant construction;
- resolved associated invocation;
- associated exact reference representation if still associated-owned;
- associated family;
- associated dynamic candidate invocation.

## Task 11 — Compile bound `&receiver.method...`

Purpose:

Emit current behavioral family/reference bytecode from the new AST.

Risk:
- Semantic: HIGH
- Implementation fanout: compiler + lowering

Target flow:

```text
compile receiver once
→ consume CallableReferenceLoweringSpec
→ exact specialization or MakeFamily
→ resulting first-class callable value
```

Do not perform method lookup in the parser/compiler if semantic lowering already supplied the decision.

## Task 12 — Compile associated `&Owner::member...` and verify runtime reuse

Purpose:

Preserve established associated runtime behavior under the new syntax.

Risk:
- Semantic: HIGH
- Implementation fanout: compiler/runtime integration

Reuse:
- associated family descriptor construction;
- variant constructor thunk;
- exact associated member identity;
- family application router.

Only modify VM/heap code if tests prove it currently depends on obsolete source syntax/semantic enum names. A syntax migration alone is not justification for runtime representation churn.

Suggested commit grouping:
- `refactor(lowering): add callable-reference lowering product`
- `refactor(core): route bound refs outside associated compiler`
- `test(core): verify new reference execution semantics`

---

# 8. Checkpoint C3 — Advisory analysis, source index, and LSP parity

Tasks:
- Task 13 — Update semantic AST visitors/advisory products.
- Task 14 — Index reference targets and selector occurrences from the new AST.
- Task 15 — Verify LSP consumes the compiler-owned products without special syntax semantics.

Why this is a checkpoint:

A compiler-correct syntax migration is incomplete if editor navigation, tokens, references, or hover lose targets. These consumers must follow compiler-owned identities.

Entry conditions:
- C2 COMPLETE;
- new AST and semantic resolution are stable.

Working set:

Primary:
- `phalcom-semantic/src/advisory/analyzer.rs`
- `phalcom-semantic/src/source_index/builder.rs`
- `phalcom-semantic/src/source_index/occurrence.rs`
- `phalcom-semantic/src/source_index/*`
- relevant LSP feature tests

Secondary:
- `phalcom-lsp/src/backend.rs`
- semantic token/hover/definition adapters only if source product shape changed.

Out of scope:
- LSP-side resolver;
- import completion;
- module path syntax.

Semantic contract:
- receiver occurrences remain ordinary references;
- callable name/signature occurrences can attach the canonical callable/variant/family target already proven by semantic analysis;
- LSP reads snapshot/editor products and does not reinterpret `&`.

Semantic risks:
- new AST omitted from visitors;
- target ranges shift to whole expression instead of callable name;
- associated variant constructor reference loses `VariantId` definition target;
- bound method references navigate differently from ordinary method calls without evidence.

Hostile cases:
- go-to-definition on `Some` in `&Option::Some(_)` reaches the variant declaration;
- go-to-definition on method name in `&object.method(_, _, debug)` reaches the same callable identity as an ordinary compatible send when statically known;
- pattern family reference with multiple candidates must not fabricate one exact target;
- semantic tokens remain stable around `&`, `::`, labels, and `_`.

Required evidence:
1. focused semantic source-index tests;
2. one LSP definition/reference integration test for associated reference;
3. one LSP test for bound reference if the source product exposes an exact target;
4. `cargo test -p phalcom-lsp <targeted tests>`;
5. no new LSP resolver/helper that duplicates selector semantics.

Do not run yet:
- broad LSP/workspace suite until C4 source migration.

Escalate immediately if:
- LSP currently depends directly on old `AssociatedNamedMode`;
- source index lacks enough canonical semantic attachment to distinguish family vs singular references.

Checkpoint completion:
- [ ] all AST visitors exhaustive
- [ ] source targets correct
- [ ] ambiguous pattern families do not fabricate exact targets
- [ ] LSP remains adapter-only
- [ ] no active incident remains

## Task 13 — Migrate semantic visitors

Purpose:
Prevent dropped analysis for the new expression.

Risk:
- Semantic: MEDIUM
- Implementation fanout: multi-file

Edit operations:
- add `Expr::CallableReference` handling to advisory analyzer;
- visit receiver exactly once;
- retain normalized selector/pattern facts where advisory analysis currently exposes associated family information;
- remove obsolete `AssociatedNamedMode` branches.

Testing classification:
- no standalone test if C3 source/LSP tests exercise visitor completeness; use `cargo check -p phalcom-semantic` for exhaustive matches.

## Task 14 — Migrate source indexing

Purpose:
Preserve canonical source occurrence identities.

Risk:
- Semantic: HIGH
- Implementation fanout: multi-file

Owned symbols:
- `SourceIndexBuilder` expression visitor;
- `OccurrenceBuilder::expr`;
- `OccurrenceKind`, `OccurrenceRole`;
- canonical semantic target attachment maps.

Required behavior:
- `&` itself need not become a semantic declaration target;
- receiver is indexed as read/reference according to current rules;
- selected callable name is indexed as a `Reference`;
- whole-family reference may target the family declaration identity where such identity exists;
- ambiguous pattern families stay non-exact unless semantic analysis provides a legitimate family target.

## Task 15 — LSP parity

Purpose:
Verify no editor-specific semantic authority appears.

Risk:
- Semantic: MEDIUM
- Implementation fanout: LSP adapter

Inspect:
- `phalcom-lsp/src/backend.rs`
- definition/reference handlers;
- token classification that matches AST variants.

Must not:
- parse selector signatures from source text in LSP;
- resolve associated vs bound reference based on punctuation strings.

---

# 9. Checkpoint C4 — Repository-wide source migration

Tasks:
- Task 16 — Migrate production `.ph` sources and core Universe sources.
- Task 17 — Migrate fixtures and semantic/compiler/runtime test sources.
- Task 18 — Migrate examples and current user-facing language documentation.
- Task 19 — Delete obsolete syntax expectations and enforce negative gates.

Why this is a checkpoint:

The new implementation must become the only active source language. Keeping old fixtures silently alive through compatibility behavior would conceal incomplete removal.

Entry conditions:
- C3 COMPLETE;
- all affected crates compile with new AST.

Working set:

Primary:
- `phalcom-core/core/**/*.ph`
- `phalcom-core/tests/**/*.ph` and embedded Phalcom source strings
- `phalcom-semantic/tests/**`
- `phalcom-ast/tests/**`
- `phalcom-lsp/tests/**` and embedded sources
- `examples/**/*.ph`
- current spec/guide documents describing selectors, families, associated lookup, ADTs.

Secondary:
- old implementation-history documents only when they are presented as current instructions or are used by automated checks.

Out of scope:
- rewriting immutable historical review records merely to erase history;
- module/import path syntax, handled by Spec 2.

Semantic contract:
- every active source site uses the new syntax;
- pattern selector examples use the existing pattern forms;
- selector signatures use labels without destructuring syntax;
- class-side methods use `.`.

Semantic risks:
- mechanical replacement changes direct associated invocation into reference;
- `receiver::name` old exact-getter reference translated incorrectly to `&receiver.name`;
- historical `::*` in Rust glob imports produces false-positive searches;
- documentation retains contradictory current semantics.

Hostile cases:
- source tree contains `::Some::(` after migration;
- current docs still say `::*` is whole family;
- a test still expects runtime-value `::` to produce behavioral Family;
- class-side examples are changed to `Fiber::new`.

Required evidence:
1. targeted search inventory before edits, classifying every language-syntax occurrence rather than Rust `::*`;
2. affected crate suites:
   - `cargo test -p phalcom-ast`
   - `cargo test -p phalcom-semantic`
   - `cargo test -p phalcom-core`
   - targeted `phalcom-lsp` tests
3. representative CLI/example executions already used by the repository for Family/ADT examples;
4. negative searches described below.

Do not run yet:
- full workspace until C4 is green.

Escalate immediately if:
- an active production source relies on old exact getter, operator, or subscript reference behavior with no ratified new spelling;
- a fixture demonstrates behavior not covered by the new semantic contract.

Checkpoint completion:
- [ ] production sources migrated
- [ ] fixtures/tests migrated
- [ ] examples migrated
- [ ] current docs migrated
- [ ] obsolete parser compatibility removed
- [ ] negative gates clean
- [ ] no active incident remains

## Task 16 — Migrate production Phalcom source

Purpose:
Use new capture syntax in executable library sources.

Risk:
- Semantic: MEDIUM
- Implementation fanout: repository-wide mechanical with semantic review

Procedure:
1. search for language-level `::` references;
2. classify each as:
   - associated invocation → keep;
   - associated value lookup → keep;
   - associated exact reference → rewrite with `&`;
   - associated family reference → rewrite with `&`;
   - bound method/family reference → rewrite from `::` to `&receiver.member...`;
   - unresolved old getter/operator/subscript case → escalate.
3. never mass-replace all `::`.

## Task 17 — Update fixtures and tests

Purpose:
Make test evidence express the new language rather than compatibility syntax.

Risk:
- Semantic: HIGH
- Implementation fanout: broad tests

Required fixture matrix:

### Associated

```phalcom
Option::Some(42)
&Option::Some
&Option::Some(_)
&Owner::member(...)
&Owner::member(_, _, ...)
&Owner::member(..., _, param)
```

### Bound

```phalcom
&object.method
&object.method(_)
&object.method(_, _, debug)
&object.method(...)
&object.method...
&object.method(_, _, ...)
&object.method(..., _, param)
```

### Composition

```phalcom
const Some = &Option::Some
const ctor = &Option::Some(_)
const bound = &object.method(_)

Some(42)
ctor(42)
bound(value)
(&Option::Some(_))(42)
```

### Class-side boundary

```phalcom
Fiber.new { ... }
&Fiber.new
&Fiber.new(_)
```

### Pattern/destructuring non-regression

```phalcom
match status {
    Connected(host, agent, debug: _) => agent.dumpInfo()
}
```

Tests must explicitly reject the old second-`::` and `::*` reference surface.

## Task 18 — Update examples and current language docs

Purpose:
Prevent implementer/user confusion after the cutover.

Risk:
- Semantic: LOW
- Implementation fanout: documentation

Primary current documents found during investigation:
- `docs/spec/current/selectors.md`
- `docs/spec/callables/family.md`
- `docs/spec/associated-lookup-surface.md`
- `docs/spec/adts.md`
- relevant current guides/examples.

Required documentation invariant:
- `::` is described only as associated lookup;
- `&` is the capture introducer;
- class-side methods remain normal sends;
- selector signatures use `method(_, _, debug)`;
- pattern examples retain ellipsis syntax.

Do not update old dated implementation plans merely to make searches zero unless they are treated by repository tooling as current documentation. Negative searches should target production/current-doc directories separately from archival implementation history.

## Task 19 — Delete obsolete authority and add negative gates

Purpose:
Prove migration completeness.

Risk:
- Semantic: HIGH
- Implementation fanout: repository-wide verification

Production-code negative gates:

```bash
rg 'AssociatedLegacyFamilyEllipsis|AssociatedExactShapeRequiresSecondSeparator' \
  phalcom-ast/src phalcom-semantic/src phalcom-core/src

rg 'AssociatedNamedMode::Family|second_separator_range|star_range' \
  phalcom-ast/src phalcom-semantic/src phalcom-core/src

rg 'BoundBehavioralFamily|BoundBehavioralInvoke' \
  phalcom-semantic/src/checker phalcom-core/src/modules
```

Expected result:
- zero obsolete ownership hits, except any deliberately renamed dedicated reference product that is documented in the checkpoint state.

Source-syntax searches must be narrow enough not to match Rust glob imports:

```bash
rg '::[A-Za-z_][A-Za-z0-9_]*::\*' \
  phalcom-core/core examples phalcom-ast/tests phalcom-semantic/tests phalcom-core/tests phalcom-lsp/tests

rg '::[A-Za-z_][A-Za-z0-9_]*::\(' \
  phalcom-core/core examples phalcom-ast/tests phalcom-semantic/tests phalcom-core/tests phalcom-lsp/tests
```

Expected:
- zero active old-language reference spellings.

Suggested commit grouping:
- `refactor(core): migrate callable reference syntax`
- `test: migrate associated and family fixtures`
- `docs: document ampersand references and associated-only double-colon`

---

# 10. Failure and diagnosis procedure

If checkpoint evidence fails unexpectedly, stop expansion.

Record:

1. exact command;
2. exact failing test/check;
3. important error output;
4. direct path from source fixture → parser → semantic product → lowering/runtime/LSP product;
5. one nearby passing comparator.

Classify the failure as:

- **PRODUCT** — implementation semantics are wrong;
- **FIXTURE** — test source still expresses obsolete or unintended behavior;
- **DEPENDENCY/PUBLICATION** — correct semantic product exists but downstream consumer is stale/missing;
- **BACKEND/HARNESS** — compiler/runtime/test harness failure outside intended semantic change;
- **BASELINE** — predates current checkpoint;
- **PLAN DRIFT** — repository no longer matches this specification.

Before repair, state the narrow allowed subsystem.

Rejected broad repairs:
- do not restore runtime-value `::` fallback;
- do not weaken a failing selector identity assertion;
- do not special-case LSP;
- do not invent new callable-reference punctuation;
- do not reintroduce second-`::` compatibility to make fixtures pass.

A checkpoint with failed required evidence is **INCIDENT**, not complete.

---

# 11. Repository drift procedure

Before each checkpoint:

1. verify primary files still exist;
2. verify named symbols retain the responsibilities described here;
3. inspect changes from earlier checkpoints;
4. search for new consumers of changed AST/semantic enums;
5. adapt local mechanics where needed.

Do not redo full investigation unless:
- a primary symbol disappeared;
- ownership moved materially;
- evidence contradicts the semantic design.

Mechanics may adapt. The semantic split may not silently change.

---

# 12. Implementation state file

Maintain a concise state document during execution.

Recommended content:

```md
## Established invariants
- I-01: `::` has no ordinary behavioral fallback.
- I-02: `&receiver.name` captures a bound family.
- I-03: selector patterns reuse canonical selector-pattern machinery.
- I-04: class-side methods remain ordinary `.` sends.

## Decisions
- D-01: <final AST type names>
- D-02: <final reference lowering product>

## Evidence ledger
| Checkpoint | Command | Result | Proves |
|---|---|---|---|

## Deferred gates
- <command> → <checkpoint/final>

## Active incident
None.

## Next resume action
Begin <checkpoint/task>.
```

After each checkpoint record:
- status COMPLETE/INCIDENT;
- semantic contract established;
- changed files/symbols;
- non-obvious repository findings;
- decisions/rejected shortcuts;
- exact evidence;
- negative-search evidence;
- deferred gates.

---

# 13. Final delivery gates

## 13.1 Checkpoint evidence summary

The implementer must fill:

| Checkpoint | Semantic contract | Evidence | Status |
|---|---|---|---|
| C0 | AST/parser owns new capture grammar | parser tests/check | |
| C1 | semantic associated/bound split | semantic focused tests | |
| C2 | lowering/runtime reuse | core lowering/runtime tests | |
| C3 | source/LSP parity | source-index/LSP tests | |
| C4 | repository migration complete | crate suites + negative searches | |

No checkpoint may be marked COMPLETE without its required evidence.

## 13.2 Final broad gates

Run smallest-first, then:

```bash
cargo +stable fmt --all -- --check
cargo +stable check --workspace --all-targets
cargo +stable test --workspace --all-targets
cargo +stable clippy --workspace --all-targets -- -D warnings
```

Purpose:
- formatting gate;
- exhaustive cross-crate compile/caller migration;
- broad behavioral compatibility;
- lint/delivery readiness.

These broad commands do not replace focused semantic evidence.

## 13.3 Final negative/deletion audit

Confirm:
- no production parser branch recognizes `::*` family capture;
- no production parser branch recognizes second-`::` exact reference capture;
- no semantic associated checker falls back to ordinary bound behavior;
- no associated lowering enum owns ordinary behavioral family construction;
- no current language docs teach the old reference syntax;
- no active fixture requires old syntax.

## 13.4 Deferred-evidence audit

No deferred command may remain without:
- successful execution;
- explicit justified removal from scope; or
- recorded release blocker.

## 13.5 Known scope exclusions

Not included:
- callable structural typing;
- changes to generic rank/generalization rules;
- new exact-getter capture syntax;
- new operator/subscript reference syntax;
- module/package `::` lookup;
- import/export path migration;
- changes to match/destructuring syntax;
- redesign of behavioral Family or associated-family runtime representation.

## 13.6 Release-complete criteria

Implementation is complete only when:

- [ ] C0–C4 are COMPLETE;
- [ ] all focused semantic evidence passes;
- [ ] hostile cases pass;
- [ ] obsolete parser/semantic/lowering authorities are removed;
- [ ] production/core/test/example source is migrated;
- [ ] current docs use only the new surface;
- [ ] final format/check/test/clippy gates pass;
- [ ] no unresolved incident exists;
- [ ] no deferred evidence remains unaccounted for.

---

# 14. Supervisor checkpoint report template

At each checkpoint, report only reviewable facts:

```text
Checkpoint C<N> COMPLETE

Established:
    <dominant semantic invariant>

Changed:
    <path> — <symbol/responsibility>
    ...

Evidence:
    <command> — PASS

Hostile cases:
    <case> — PASS

Negative gates:
    <search> — expected result observed

Deferred:
    <broad command> → <later checkpoint>

Unexpected findings:
    none | <fact>

Next:
    C<N+1> — <name>
```

This specification is an implementation guide, not authorization to change adjacent language semantics. If the repository forces a semantic choice not ratified above, stop and escalate with repository evidence.
