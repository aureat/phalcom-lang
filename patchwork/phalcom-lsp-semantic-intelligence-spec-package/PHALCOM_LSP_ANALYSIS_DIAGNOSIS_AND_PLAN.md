# Phalcom LSP — Semantic Intelligence, Targeted Hover, and Inference
## Detailed Analysis, Diagnosis, and Implementation Plan

**Repository:** `aureat/phalcom-lang`  
**Inspected branch:** `main`  
**Inspected baseline:** `e2ec9e5fb6dc362786c9dd9593470feb47c91d94`  
**Date:** 2026-08-12  
**Primary scope:** `phalcom-lsp`, `phalcom-ast`, `phalcom-native-surface`, relevant `phalcom-core` lowering/dispatch code  
**Status:** Analysis and architectural plan. This document is intentionally written before the focused implementation specifications.

> Line references in this document are pinned to the baseline above. If the branch moves, use the named function/type as the authoritative anchor and the line range as the baseline locator.

---

# 1. Executive conclusion

The current LSP is not failing because VS Code lacks the necessary capabilities, and it is not failing because one or two AST variants were forgotten.

The central problem is that Phalcom's language intelligence has **several partially overlapping semantic paths which do not share one complete source model**:

1. declaration and selector indexing;
2. hover target lookup;
3. local binding inference;
4. expression inference;
5. callable return summaries;
6. parameter facts;
7. field facts;
8. callable dependency extraction;
9. completion receiver inference;
10. semantic-token classification.

Each path understands a different subset of the AST, a different notion of scope, and in some cases a different notion of dispatch. The result is predictable:

- direct declaration hover may know a method result while an expression using that method is `Unknown`;
- local facts can know a binding while callable summary inference loses it;
- method calls receive interprocedural treatment while getters, operators, subscripts, and many nested expressions do not;
- side-aware completion can work while the solver uses side-blind member resolution;
- whole method expressions are indexed as selector occurrences, so hovering anywhere in the declaration can select the entire method;
- variables are stored by bare spelling rather than lexical binding identity;
- several AST nodes do not retain the token-level ranges required for exact hover targeting.

The correct fix is therefore **a semantic consolidation**, not a long series of special-case patches.

The existing `SemanticDb`, module graph, generation/stamp mechanism, bounded `ValueShape`, fixed-point solver concept, live core-source model, and native-surface crate are good foundations and should be preserved. The refactor belongs *under* those facilities.

The target architecture should have four shared foundations:

1. **Source-precise semantic identities and occurrences**
   - exact ranges for declaration names, selector tokens, parameters, local bindings, operators, etc.;
   - lexical `ScopeId` and `BindingId`;
   - a single occurrence/target index used by hover/navigation/semantic refinement.

2. **One canonical dispatch resolver**
   - selector + receiver class + dispatch side + lookup mode;
   - inheritance-aware;
   - `super`-aware;
   - shared by expression inference, callable summaries, dependency extraction, parameter inference, completion, and hover.

3. **One exhaustive expression analyzer**
   - recursively handles every current `Expr` variant;
   - treats ordinary operators/getters/subscripts as the sends they actually are;
   - returns value knowledge while recording resolved calls/effects/dependencies.

4. **One scoped statement-flow engine**
   - sequential bindings and reassignment;
   - lexical scopes and shadowing;
   - method/closure/loop bodies;
   - non-local returns from blocks to their home callable;
   - field writes;
   - call-site parameter evidence;
   - callable return summaries.

This is also the correct preparation for Phalcom's future type system. `ValueShape` must remain advisory runtime-value knowledge, not be renamed into or confused with the future language `Type`. What should be shared with the future type checker is the **source graph, scope/name resolution, dispatch resolution, control-flow graph, expression identities, and fact propagation machinery**. Declared type information can later become a stronger evidence source over those same identities.

---

# 2. Required end state

The completed LSP should behave as if there is one coherent semantic understanding of the source program.

## 2.1 Hover contract

Hover should resolve the **smallest meaningful semantic source target under the cursor**.

Examples:

```phalcom
class Account {
  @constructor
  new(owner) {
    let local = owner
    local.toString
  }
}
```

Desired targets are independent:

- `Account` → class hover, range only `Account`;
- `new` → constructor/method hover, range only selector/name token;
- `owner` in parameter declaration → parameter binding hover, range only `owner`;
- `local` declaration → local binding hover, range only `local`;
- `owner` use → same parameter binding, range only that occurrence;
- `local` use → same local binding, range only that occurrence;
- `toString` → resolved getter selector hover, range only `toString`.

Hovering `{`, `}`, `let`, `@constructor`, numeric/string/bool/symbol literals, commas, parentheses, or unrelated whitespace should not return a hover merely because those bytes are inside an enclosing method/expression.

The same rules must hold:

- at module top level;
- inside methods/getters/setters/subscripts/constructors;
- inside nested closure bodies;
- inside `for` bodies;
- inside parsed/desugared `while`/`if` constructs;
- inside nested expressions and arguments.

## 2.2 Inference contract

Where the source and known program surface provide stable evidence, inference should propagate it through the full expression tree.

Examples that should work:

```phalcom
let a = Account.new("A")      // Account
let r = a.rate                // return shape of getter `rate`
let s = a.toString            // String, if getter summary says String
let t = "Account(\(a))"       // String
let u = 1 + 2                 // return knowledge for Int/Number +(_)
let v = list[0]               // subscript return knowledge
let w = factory.make().child  // chained receiver inference
```

Within methods:

```phalcom
describeSelector() {
  let sel = #deposit
  return sel
}
```

must summarize as `Symbol`.

Constructor factories must summarize as an instance of the owning class regardless of the initializer body's final expression.

`super` sends must start lookup at the superclass and preserve the current dispatch side.

## 2.3 Consistency contract

Hover, completion, inlay hints, and navigation should not each invent their own semantic answer.

If completion knows an expression is an instance of `Savings`, then hover/inlay inference should consume the same fact. If hover resolves `rate` to a particular inherited getter, dependency extraction and parameter/return inference should resolve that same callable.

---

# 3. Current architecture: what should be preserved

The latest LSP already contains several strong pieces.

## 3.1 `SemanticDb` and coherent generations

`phalcom-lsp/src/semantic/mod.rs` maintains:

- module-qualified class/callable identities;
- file snapshots;
- a module graph;
- callable summaries;
- field and parameter facts;
- reverse callable dependents;
- coherent semantic generations.

See baseline:

- `phalcom-lsp/src/semantic/mod.rs:100-520`
- `phalcom-lsp/src/semantic/mod.rs:500-920`

The update path computes affected modules and republishes a coherent generation rather than mutating editor-visible facts one map at a time.

**Decision:** retain this architecture.

The refactor should make the inputs to `SemanticDb` more correct, not discard its invalidation/generation structure.

## 3.2 Module-qualified identities

`ClassId`/`CallableId` are already module-qualified and include dispatch side for callables. This is the correct direction for both LSP semantics and future typing.

**Decision:** retain module-qualified identities and make *all* semantic resolution use them consistently.

## 3.3 Bounded advisory value knowledge

`phalcom-lsp/src/semantic/facts.rs` defines `ValueShape`, `InferredValue`, confidence, provenance, and bounded union behavior.

`ValueShape` is explicitly documented as:

> advisory runtime value shape; deliberately not a language type.

This distinction is important.

**Decision:** retain `ValueShape` as runtime/editor knowledge. Do not turn it into the formal Phalcom type system.

## 3.4 Live core source and canonical native surface

`phalcom-lsp/src/semantic/core_source.rs:1-80` builds semantic core state from:

- live/bundled `core.ph`;
- `phalcom-native-surface`.

`phalcom-native-surface` is intentionally VM-, AST-, and LSP-independent.

**Decision:** retain this split, but enrich native semantic return contracts.

---

# 4. Root structural defect: three expression inferencers that understand different languages

The most important current code is in:

- `phalcom-lsp/src/semantic/infer.rs:20-245`

There are three layers:

```text
infer_expr
    syntax/local subset

infer_expr_with_returns
    MethodCall + UnqualifiedCall special cases
    else -> infer_expr

infer_expr_with_fields
    Field + MethodCall special cases
    else -> infer_expr_with_returns
```

This structure means semantic knowledge is **not recursively available to arbitrary child expressions**.

For example, `infer_expr_with_returns(Binary(...))` immediately falls back to `infer_expr(Binary(...))`. `infer_expr` has no `Binary` arm, so the expression is `Unknown`. Even if the binary operands contain perfectly resolvable method calls, that knowledge is unreachable.

The same failure appears for:

- `Unary`;
- `Binary`;
- `Index`;
- `SetIndex`;
- `SetProperty`;
- `Block`;
- `MethodRef`;
- many nested collection positions;
- getter dispatch;
- control-flow values.

## Required architectural change

Replace the wrapper family with one recursive evaluator, conceptually:

```rust
fn analyze_expr(
    expr: &Expr,
    ctx: &AnalysisContext<'_>,
    flow: &FlowState,
    sink: &mut AnalysisSink,
) -> ExprAnalysis
```

Where:

```rust
struct ExprAnalysis {
    value: InferredValue,
    // optionally later: normal-flow state / throws / exits
}

struct AnalysisContext<'a> {
    module: &'a ModuleId,
    lexical_class: Option<&'a ClassId>,
    lexical_callable: Option<&'a CallableId>,
    dispatch_side: Option<DispatchSide>,
    scopes: &'a ScopeGraph,
    resolver: &'a DispatchResolver<'a>,
    facts: &'a FactProvider<'a>,
}
```

Every recursive child call must go back through `analyze_expr`, never through a weaker syntax-only function.

This one change is the foundation on which most other fixes depend.

---

# 5. Getter/property dispatch is treated differently from real runtime semantics

## Current LSP behavior

In `infer_expr`, `Expr::GetProperty` only accepts a receiver shaped as `Expr::Var` and tries to interpret:

```phalcom
Module.Class
```

as a qualified class lookup.

Baseline:

- `phalcom-lsp/src/semantic/infer.rs:20-245`, `Expr::GetProperty` arm.

That means ordinary property/getter sends such as:

```phalcom
account.rate
x.toString
Savings.rate
super.toString
```

do not use callable return inference.

## Runtime/compiler behavior

The compiler treats `GetProperty` as a zero-argument selector send. It even has a dedicated `super.prop` path:

- `phalcom-core/src/compiler/lib/expr.rs:360-760`

Ordinary property:

```text
compile receiver
Invoke(0, property selector)
```

Super property:

```text
compile_super_send(property selector, [])
```

Therefore the LSP currently models the same syntax with different semantics from the compiler.

## Required change

`GetProperty` must use the canonical dispatch resolver.

Pseudo-semantics:

```text
receiver = analyze receiver

if receiver is super-target:
    resolve getter starting above lexical class
else:
    resolve selector = property
    using receiver class + side

return target callable's known return shape
record dependency edge
```

Qualified module/class lookup should remain supported, but it must be a **name-resolution case**, not the sole meaning of `GetProperty`.

---

# 6. `super` must be modeled as a dispatch target, not a normal value

Current `infer_expr` returns `Unknown` for `Expr::SuperVar`.

That is superficially understandable but insufficient, because `super` participates in real expressions only as a send receiver.

Compiler behavior confirms the correct model:

- `phalcom-core/src/compiler/lib/expr.rs:360-760` intercepts `super.method(...)` and `super.property`;
- `phalcom-core/src/compiler/lib/expr.rs:1160-1320` rejects bare `super` with `CompilerError::BareSuper`.

Therefore:

```text
super ≠ Instance(superclass)
```

in the language model.

It is better represented internally as:

```rust
enum ReceiverTarget {
    Value(InferredValue),
    Super {
        lexical_class: ClassId,
        side: DispatchSide,
    },
}
```

A `super` send preserves the actual receiver (`self`) but changes the lookup start point. This distinction will matter to future typing as well, especially for `Self`, inherited method signatures, and specialization.

---

# 7. String interpolation failure is a consequence of missing operator/getter inference

Parser implementation:

- `phalcom-ast/src/parser.rs:2100-2225`

`desugar_string_interp` lowers:

```phalcom
"a \(x) b"
```

approximately to:

```text
("a " + x.toString) + " b"
```

using:

- `Expr::GetProperty(... "toString")`;
- `Expr::Binary(BinaryOp::Add, ...)`.

The LSP loses both pieces:

1. getters are not dispatched semantically;
2. `Binary` has no inference arm.

This is why an interpolated string can become `Unknown` even though the final runtime value is necessarily a string for the lowered valid expression.

## Correct fix

Do not add a narrow `"if this came from interpolation => String"` patch as the main mechanism.

Instead:

1. make `GetProperty` resolve `toString`;
2. model `BinaryOp::Add` as the same selector send as the compiler;
3. give canonical native/source return knowledge for `String +(_)`.

Then string interpolation infers `String` naturally from the real semantics.

A source-origin marker for interpolation could still be useful later for diagnostics/formatting, but it should not be needed for basic value inference.

---

# 8. Operators are sends and should use the same dispatch machinery

Compiler implementation:

- `phalcom-core/src/compiler/lib/expr.rs:1160-1320`

Non-lazy binary operators compile as ordinary sends.

Unary operations are also sends:

```text
-x     -> negated()
not x  -> not()
~x     -> ~()
```

`and` and `or` are the lazy exceptions; the RHS is wrapped as a block and sent through the relevant Bool protocol/inliner semantics.

Current LSP inference does not model `Binary` or `Unary`.

## Required change

Create one VM-free mapping shared by semantic analysis:

```rust
fn binary_selector(op: BinaryOp) -> Selector
fn unary_selector(op: UnaryOp) -> Selector
```

Prefer placing canonical selector spelling/mapping in a dependency that both compiler and semantic tooling can consume (`phalcom-common` is a candidate) rather than allowing compiler and LSP copies to drift indefinitely.

Expression analysis then resolves operators through the same `DispatchResolver` as explicit sends.

This has immediate value for:

- arithmetic;
- comparisons;
- Boolean operations;
- strings;
- user-defined operator methods;
- future typed operator constraints.

---

# 9. Side-aware member data exists, but the solver repeatedly bypasses it

`phalcom-lsp/src/semantic/surface.rs:1-300` currently stores both:

```rust
members: BTreeMap<String, MemberSurface>
members_by_side: BTreeMap<(String, DispatchSide), MemberSurface>
```

`members_by_side` is correct.

The side-blind `members` map is lossy: when an instance-side and class-side member share the same selector, the first one inserted wins.

`phalcom-lsp/src/semantic/mod.rs:500-920` contains:

```text
resolve_member_surface
    uses side-blind `members`

resolve_member_surface_for_side
    uses `(selector, side)`
```

Many solver callbacks use the side-blind form.

## A second inheritance bug

`infer_expr_with_returns` frequently creates a `CallableId` directly from the receiver class:

```text
owner = receiver class
selector = ...
side = ...
```

This is not inheritance resolution.

If:

```phalcom
class Base {
  foo() { ... }
}

class Child is Base {}
```

then a call on `Child` should resolve to the `CallableId` owned by `Base`, not a synthetic `Child/foo()` callable that has no summary entry.

## Required change: one canonical resolver

Introduce:

```rust
struct DispatchRequest<'a> {
    receiver_class: &'a ClassId,
    side: DispatchSide,
    selector: &'a str,
    lookup: LookupMode,
}

enum LookupMode {
    Ordinary,
    SuperFrom { lexical_class: ClassId },
}

fn resolve_dispatch(req: DispatchRequest<'_>) -> Option<MemberSurface>
```

This must be the only route used by:

- expression inference;
- getter/setter/subscript/operator inference;
- callable summary inference;
- parameter fact collection;
- dependency extraction;
- constructor recognition;
- hover member resolution;
- completion;
- navigation.

Once migrated, the side-blind resolver/map should be removed or restricted to explicitly non-dispatch uses.

---

# 10. Unqualified calls currently disagree with compiler lexical precedence

Compiler behavior for:

```phalcom
foo(...)
```

is not simply "send `foo` to self."

The compiler resolves a bare name in lexical order and only uses implicit-self dispatch when the name is not a local/upvalue/global callable target.

Current LSP `infer_expr_with_returns` treats `UnqualifiedCall` as a current-class instance method.

This can misanalyze:

```phalcom
let f = someCallable
f(x)
```

or a closure/upvalue/global callable whose spelling overlaps a member selector.

## Required change

The scope/name-resolution layer must answer:

```rust
enum BareNameResolution {
    Binding(BindingId),
    Global(GlobalId),
    Class(ClassId),
    ImplicitSelf,
    Unresolved,
}
```

The expression analyzer then follows the compiler's precedence.

This is one reason `BindingId` and lexical scopes are not optional cleanup work; they are necessary for correct call semantics.

---

# 11. Local facts are keyed by spelling instead of lexical identity

Current `LocalFacts` is conceptually:

```rust
BTreeMap<String, Vec<BindingFact>>
```

and `binding_at(name, offset)` picks the most recent source-range fact before the offset.

That cannot faithfully represent lexical scoping.

It can confuse:

- same-named locals in different methods;
- nested shadowing;
- closure captures;
- loop bindings;
- parameters and class/global names;
- reassignment after shadowing.

The fact that member-body locals from multiple methods are recorded into the same file-level structure makes the problem especially important.

## Required change

Introduce a source scope graph.

Recommended model:

```rust
#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct ScopeId(u32);

#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct BindingId(u32);

enum BindingKind {
    TopLevelLet,
    TopLevelConst,
    LocalLet,
    LocalConst,
    MethodParam,
    SetterParam,
    IndexParam,
    ClosureParam,
    ForBinding,
    Destructure,
}

struct Binding {
    id: BindingId,
    scope: ScopeId,
    name: String,
    declaration_range: SourceRange,
    kind: BindingKind,
}
```

Use:

```text
BindingId -> flow facts
SourceRange occurrence -> BindingId
ScopeId -> parent + bindings
```

instead of:

```text
name -> all writes in file
```

This structure will directly support:

- exact hover;
- definition/reference;
- completion visible names;
- inlay hints;
- future type inference/checking.

---

# 12. Parameters currently disappear when there is no call-site evidence

In `SemanticDb::infer_expression` (`phalcom-lsp/src/semantic/mod.rs:500-920`), method parameters are inserted into the environment only when `parameter_facts` already contains a value for them.

That is semantically wrong even when the value shape is unknown.

A parameter is still a lexical binding and must shadow:

- class names;
- globals;
- outer names.

The correct environment should contain:

```text
param BindingId -> Unknown
```

if there is no stronger evidence.

Call-site inference can later refine it.

This is automatically fixed by building scopes/bindings first and treating "unknown value" as different from "unresolved name."

---

# 13. Callable summaries do not execute statement flow

Relevant current code:

- `phalcom-lsp/src/semantic/infer.rs:1030-1270`

`summaries_for_surface` seeds an environment with parameters.

But `body_summary_value` then:

- scans only direct top-level `return` statements;
- evaluates them against the original parameter environment;
- does not update the environment for earlier `let` declarations or assignments;
- does not recursively collect returns from nested bodies;
- otherwise evaluates only the last statement.

Hence:

```phalcom
describeSelector() {
  let sel = #deposit
  return sel
}
```

can infer `sel` as `Symbol` in one local-fact pass but still summarize the method return as `Unknown`.

This is not a missing `Symbol` case. It is a missing statement semantics engine.

## Required change

One statement analyzer should walk sequentially:

```rust
fn analyze_statements(
    statements: &[Statement],
    ctx: &AnalysisContext<'_>,
    in_state: FlowState,
    sink: &mut AnalysisSink,
) -> FlowResult
```

Conceptually:

```rust
struct FlowResult {
    normal: Option<FlowState>,
    returns: Vec<InferredValue>,
    throws: bool,
    breaks: bool,
    continues: bool,
}
```

The initial implementation does not require a heavyweight compiler IR or SSA.

A structured AST flow engine is enough if it:

- updates local facts in source order;
- creates nested scopes;
- joins branch/loop facts conservatively;
- records callable return evidence wherever it occurs;
- records resolved call edges;
- records field writes;
- records call-site argument evidence.

The critical rule is: **all consumers derive from the same traversal**.

---

# 14. Non-local return from closures is a required flow rule

Phalcom's current block specification states that `return` inside a block is a non-local return to the method frame in which the block was created.

Therefore the semantic analyzer must not treat:

```phalcom
method() {
  xs.each || {
    return "found"
  }
  0
}
```

as if `"found"` were merely the block's local return type.

The block has two different concepts:

1. its own normal expression/body value;
2. a `return` statement that exits the home callable.

The analysis context should therefore retain:

```rust
home_callable: Option<CallableId>
```

when entering nested blocks.

This is a direct future-typing requirement too: return type checking must apply the same home-callable rule.

---

# 15. Field analysis is currently both too narrow and too broad

Relevant current code:

- `phalcom-lsp/src/semantic/infer.rs:220-430` (`field_facts_for_surface` region)

Problems:

1. RHS expressions are inferred with an empty environment.
   - `_owner = owner` cannot use a constructor parameter's inferred/declared knowledge.

2. It walks member bodies but only direct top-level assignment statements.
   - nested block/control-flow writes are missed.

3. Despite its "constructor-assigned" documentation, it iterates all members.
   - arbitrary later mutable writes can be mixed with initialization evidence.

4. Static vs instance field evidence is not represented strongly enough in the fact key.

5. Inherited field ownership/read lookup needs a defined policy.

## Required model

Field facts should distinguish at least:

```rust
FieldId {
    owner: ClassId,
    name: String,
    side: DispatchSide,
}
```

and evidence categories:

```text
initializer evidence
constructor initialization evidence
general write evidence
```

For editor completion/inference, a conservative joined shape is useful, but the analyzer should not lose the distinction between "guaranteed initialization" and "observed mutable writes." Future type checking will need that distinction.

Field writes should be collected as part of the same scoped flow pass, using the current environment.

---

# 16. Constructor return inference must follow the constructor contract

The compiler's constructor lowering is explicit:

- `phalcom-core/src/compiler/attributes.rs:2300-2445`
- `lower_constructors`

A source `@constructor` becomes a class-side factory which:

1. allocates `instance`;
2. invokes compiler-owned initializer;
3. returns `instance`.

The LSP surface correctly recognizes source constructors as class-side factory-like members:

- `phalcom-lsp/src/semantic/surface.rs:1-300`

But generic callable summary inference currently analyzes the source constructor body as an ordinary method body. If the body ends in:

```phalcom
_balance = 0
```

the assignment expression shape can become `Int`, which is not the constructor result.

## Required rule

For every source member marked as constructor:

```text
callable summary return = Instance(owner class)
```

This is exact semantic knowledge.

The source body is still analyzed for:

- parameter facts;
- field initialization;
- call dependencies;
- diagnostics/effects.

But it must not determine the factory result.

---

# 17. Dependency extraction is another partial AST interpreter

Relevant code:

- `phalcom-lsp/src/semantic/infer.rs:1270-1540`

`collect_dependency_expr` has its own match over expressions.

It records explicit `MethodCall` dependencies, but:

- `GetProperty` only recurses into the receiver and does not record the getter call itself;
- several expression shapes are absent or partial;
- side-aware inherited resolution is not consistently used.

The same drift exists in call-site parameter collection (`collect_call_sites_expr`).

## Required change

Do not maintain separate semantic AST interpreters.

When `analyze_expr` resolves a call, it should emit a semantic event:

```rust
sink.record_call(ResolvedCall {
    target: CallableId,
    site: SourceRange,
    argument_values: ...,
});
```

From this single event:

- callable dependency edges are produced;
- parameter evidence is produced;
- hover/navigation can cache resolved occurrence targets if useful;
- future effect analysis can be attached.

Getter/operator/subscript calls then participate automatically because they pass through the same dispatch function.

---

# 18. Hover's whole-method highlighting has a precise cause

This issue is fully diagnosed.

Relevant code:

- `phalcom-lsp/src/index.rs:430-780`
- `phalcom-lsp/src/index.rs:780-1040`
- `phalcom-lsp/src/backend.rs:220-520`

`Collector` records method definitions using:

```text
m.range
g.range
s.range
f.range
ix.range
```

which are whole declarations.

It records call/property references using the whole expression range:

```text
MethodCall.range
UnqualifiedCall.range
GetProperty.range
SetProperty.range
MethodRef.range
```

Then `selector_at_offset` finds the smallest selector-bearing range containing the cursor.

A method declaration's whole range contains its body.

Therefore, if the cursor is in a method body but not on a narrower selector reference, the method declaration is still a candidate, and hover can select the whole method.

This is exactly the reported behavior.

## Correct fix

Selector occurrences must be indexed by the exact source span of the selector/name token, not by enclosing declaration/expression range.

The declaration AST already has exact `name_range` for several member kinds. Reference expressions do not.

The right solution is AST source-range enrichment, not a large text-scanning workaround.

---

# 19. The AST currently lacks several exact spans required by the desired UX

Relevant AST regions:

- `phalcom-ast/src/ast.rs:260-560`
- `phalcom-ast/src/ast.rs:560-840`
- `phalcom-ast/src/ast.rs:840-1040`
- `phalcom-ast/src/ast.rs:1080-1180`

Already source-precise:

- `ClassDef::name_range`;
- method/getter/setter/index declaration `name_range`;
- `Pattern::Name.range`;
- whole parameter range.

Not source-precise enough:

- `MethodCallExpr.method: String` — no `method_range`;
- `UnqualifiedCallExpr.name: String` — no `name_range`;
- `GetPropertyExpr.property: String` — no `property_range`;
- `SetPropertyExpr.property: String` — no `property_range`;
- method-ref selector/name — no selector range;
- `BinaryExpr` — no operator token range;
- `UnaryExpr` — no operator token range;
- `ForStatement.binding: String` — no binding range;
- `ClosureParameters.fixed: Vec<String>` — no parameter ranges;
- `FieldDef.name` — no field-name range;
- `VariantDef.name` — no variant-name range;
- `ParameterDef` — whole param range but no separate local-name / external-label ranges;
- subscript call/write — whole expression range only.

## Recommended AST additions

At minimum:

```text
MethodCallExpr.method_range
UnqualifiedCallExpr.name_range
GetPropertyExpr.property_range
SetPropertyExpr.property_range
MethodRefExpr.selector_range
BinaryExpr.op_range
UnaryExpr.op_range
ForStatement.binding_range
FieldDef.name_range
VariantDef.name_range
ParameterDef.name_range
ParameterDef.label_range: Option<SourceRange>
```

Closure parameters should become structured:

```rust
struct ClosureParameter {
    name: String,
    range: SourceRange,
}
```

For subscript sends, define a stable exact hover target convention. A practical first version is:

- declaration: existing `IndexMethodDef.name_range` (the bracket selector);
- call/reference: bracket portion `[ ... ]`, not receiver and not RHS.

These spans are not merely LSP cosmetics. They are useful for compiler diagnostics and future typing metadata, so adding them to the AST is justified.

---

# 20. Hover currently contains a product behavior that must be deliberately removed

Current hover path:

- `phalcom-lsp/src/backend.rs:520-940`
- `phalcom-lsp/src/hover.rs:1-260`

The first branch of `hover_at` is:

```text
keyword_at_offset
-> keyword_blurb
-> return hover
```

The user requirement is the opposite:

- no hover for keywords;
- no hover for obvious literals;
- no hover for symbol literals merely because they are literals.

Therefore this branch should be removed from the runtime hover path.

The keyword-doc code can be deleted if no other feature uses it, or retained temporarily only if a separate documentation command has a concrete consumer. It must not participate in `textDocument/hover`.

Existing Stage 4 coverage explicitly expects keyword hover, so tests must be changed rather than merely supplemented.

---

# 21. Binding hover is top-level-only and even there unnecessarily requires documentation

Current `hover_for_top_level_binding`:

- `phalcom-lsp/src/backend.rs:520-940`
- supported by `index::top_level_binding_at_offset` in `phalcom-lsp/src/index.rs:430-780`

Problems:

1. only top-level bindings;
2. name-based, not lexical identity;
3. the fallback requires a Phaldoc block before it returns a hover;
4. it sets `range: None`;
5. method parameters and locals have no equivalent path.

The semantic target model should replace this entire special case.

A binding hover should be able to render from:

```text
BindingId
declaration kind (let/const/param/for/closure param)
current inferred value
optional declared type later
optional documentation
```

Documentation is additive, not a prerequisite.

---

# 22. Semantic tokens and hover are separate concerns

Current semantic-token implementation:

- `phalcom-lsp/src/semantic_tokens.rs:1-350`
- `phalcom-lsp/src/semantic_tokens.rs:350-520`

It is primarily lexer-driven.

It already handles interpolated strings relatively well:

- literal pieces are `string`;
- interpolation expressions are recursively lexed/classified.

Its AST-assisted refinement upgrades only:

- class declaration names;
- method/getter/setter/index declaration names.

Fields, parameters, local declarations/references, and call selectors largely remain generic `variable`.

## Important product distinction

The request to "remove hover highlighting for literals/keywords" should **not** be interpreted as "stop syntax highlighting literals/keywords."

Semantic tokens should continue to color syntax.

Hover targeting should return no hover for those tokens unless some future feature gives them non-obvious semantic content.

## Recommended later refinement

Once the semantic occurrence index exists, semantic tokens can optionally use it to refine identifier roles:

- variable;
- parameter;
- property;
- method;
- class.

But hover correctness does not need to wait for that classification upgrade.

---

# 23. Completion already uses semantic facts, but lexical completion is not truly lexical

`phalcom-lsp/src/completion.rs:1-420` has good member-completion infrastructure:

- incomplete receiver recovery;
- union receiver completion;
- visibility filtering;
- inheritance-aware completion through `SemanticDb::completion_members`.

However `visible_names_at` scans the top-level program statements and does not represent nested lexical scopes correctly.

`Backend::semantic_receiver` (`phalcom-lsp/src/backend.rs:220-520`) also has several ad hoc cases before reparsing the receiver slice.

## Required change

Use the new scope graph for:

```text
visible bindings at offset
binding represented by receiver occurrence
parameter/local/closure/for shadowing
```

Keep the syntax-light incomplete-dot recovery. It solves a different problem and is useful.

---

# 24. Expression-environment collection is another incomplete recursive walker

`SemanticDb::infer_expression` currently calls a helper that collects only a narrow subset of variables from the expression before running inference.

Relevant region:

- `phalcom-lsp/src/semantic/mod.rs:500-920`
- `collect_expression_environment`

It handles mainly:

- `Var`;
- `MethodCall`;
- `GetProperty`.

It misses many expression forms.

With `BindingId`/scope-aware expression analysis, this pre-pass should disappear. Variable resolution should happen at the variable occurrence itself.

---

# 25. Native return knowledge is not yet useful enough for broad inference

`phalcom-native-surface/src/lib.rs:1-260` defines:

```rust
enum NativeReturnKnowledge {
    Unknown,
    Declared,
}
```

The current `native!` macro assigns `Unknown` to every native member.

The canonical native surface includes many selectors whose result behavior is stable enough to help the editor:

- `String +(_)`;
- Number comparisons;
- Bool `not()`;
- `String.hash`;
- class `new` allocators;
- size/count getters;
- selected collection operations;
- reflection getters.

See also:

- `phalcom-native-surface/src/lib.rs:240-430`.

## Required change

Enrich this VM-free crate with a conservative semantic return descriptor.

Example:

```rust
enum NativeReturnShape {
    Unknown,
    Instance(&'static str),
    Receiver,
    ClassObject(&'static str),
    Argument(usize),
}
```

Only encode contracts that are guaranteed by the primitive/runtime semantics.

Do not guess.

This remains runtime-value knowledge, not formal type metadata.

When formal typing metadata arrives, declared return types can provide stronger/more general evidence through a separate bridge.

---

# 26. Inlay hint “Method not found” notification is a protocol-advertisement bug

This issue is independently confirmed.

Current server initialization:

- `phalcom-lsp/src/backend.rs:900-1015`

advertises:

```rust
InlayHintOptions {
    resolve_provider: Some(true),
    ...
}
```

But the server implements `textDocument/inlayHint` and has no `inlayHint/resolve` implementation.

Repository search finds no `inlay_hint_resolve`.

VS Code is therefore allowed to send:

```text
inlayHint/resolve
```

and tower-lsp can correctly reply with JSON-RPC:

```text
-32601 Method not found
```

This exactly matches the reported notification.

Furthermore, current hints are already returned fully populated with `data: None`, so there is no reason to advertise lazy resolve.

## Immediate fix

Change capability advertisement to:

```rust
resolve_provider: Some(false)
```

or omit it.

Add a protocol transport test asserting that the initialize result does not advertise resolve support.

This should be the first small patch because it removes a noisy false server capability without depending on the larger semantic refactor.

---

# 27. Test coverage currently encodes some of the undesired behavior

Current test suite includes:

- `phalcom-lsp/tests/stage4_hover.rs`
- `phalcom-lsp/tests/stage5_semantic_tokens.rs`
- `phalcom-lsp/tests/stage6_inlay_hints.rs`
- `phalcom-lsp/tests/semantic_completion.rs`
- `phalcom-lsp/tests/semantic_consistency.rs`
- semantic fixtures under `phalcom-lsp/tests/fixtures/semantic/`

Important gaps:

1. Stage 4 explicitly tests keyword hover as a success case.
   - This must be inverted into a negative hover test.

2. Stage 6 tests `textDocument/inlayHint` but does not exercise/validate the advertised resolve capability.
   - Thus it cannot catch the VS Code failure.

3. Existing semantic consistency tests cover simple cases but not a full cross-feature semantic matrix.

4. The current semantic fixtures do not adequately force:
   - getter return propagation;
   - inherited getter/method return resolution;
   - class-vs-instance same-selector resolution;
   - `super` getter/method resolution;
   - string interpolation;
   - all operator families;
   - nested local flow;
   - closure captures;
   - non-local block return;
   - loop binding scope;
   - shadowing;
   - index get/set;
   - field initialization from constructor parameters;
   - constructor result guarantee;
   - method references;
   - side effects of edits/invalidation.

---

# 28. Target architecture

The following is the recommended completed architecture.

## 28.1 Layer A — parsed source + exact spans

`phalcom-ast` remains the syntax source of truth.

Add missing token-level spans needed by semantic consumers.

No LSP types belong in this layer.

## 28.2 Layer B — source semantic graph

Add under `phalcom-lsp/src/semantic/` initially:

```text
scope.rs
occurrence.rs
dispatch.rs
analysis.rs        (or split expr.rs / flow.rs)
```

Recommended responsibilities:

### `scope.rs`

Build:

- `ScopeId`;
- `BindingId`;
- binding declarations;
- parent/child scopes;
- occurrence-to-binding resolution;
- visible bindings at offset;
- closure capture relation if needed.

### `occurrence.rs`

Build exact semantic targets:

```rust
enum SemanticOccurrenceKind {
    Class,
    MemberDeclaration,
    MemberReference,
    BindingDeclaration,
    BindingReference,
    ParameterDeclaration,
    Field,
    Operator,
    // later TypeReference
}
```

Each occurrence contains:

```rust
range: SourceRange
target identity
role: read/write/call/declaration
```

### `dispatch.rs`

One side-aware, inheritance-aware resolver.

### `analysis.rs`

One recursive expression analyzer and one structured statement-flow analyzer.

## 28.3 Layer C — facts

Retain `ValueShape`/`InferredValue`, but key local flow by `BindingId`, not spelling.

Recommended distinction:

```rust
BindingFacts
FieldFacts
CallableSummary
ParameterFacts
```

All should be products of one analysis traversal / event sink.

## 28.4 Layer D — `SemanticDb`

Retain:

- file/module snapshots;
- graph;
- generation;
- invalidation;
- fixed-point solve.

Publish source graph + facts coherently.

## 28.5 Layer E — LSP adapters

`backend.rs`, `hover.rs`, `completion.rs`, `inlay_hints.rs`, `semantic_tokens.rs` should become consumers.

They should not re-implement AST semantics.

---

# 29. Proposed semantic query API

The LSP surface will be much simpler if `SemanticDb` exposes source-oriented queries.

Recommended APIs:

```rust
pub fn occurrence_at(
    &self,
    uri: &Url,
    offset: usize,
) -> Option<SemanticOccurrence>;

pub fn binding_info(
    &self,
    id: BindingId,
) -> Option<BindingInfo>;

pub fn value_at_occurrence(
    &self,
    occurrence: &SemanticOccurrence,
) -> Option<InferredValue>;

pub fn expression_value_at(
    &self,
    uri: &Url,
    offset: usize,
) -> Option<InferredValue>;

pub fn resolved_member_at(
    &self,
    uri: &Url,
    offset: usize,
) -> Option<ResolvedMember>;

pub fn visible_bindings_at(
    &self,
    uri: &Url,
    offset: usize,
) -> Vec<BindingInfo>;
```

Hover should primarily be:

```text
offset
-> occurrence_at
-> render occurrence
```

rather than:

```text
keyword scan
-> class text heuristic
-> selector index
-> top-level binding fallback
```

---

# 30. Hover target precedence

If multiple semantic ranges overlap, choose the smallest exact semantic occurrence.

Recommended precedence when equal-sized ranges collide:

1. local/parameter/field binding occurrence;
2. member selector/reference;
3. class reference/declaration;
4. operator;
5. enclosing expression only if a future feature explicitly supports expression hover.

Do **not** fall back to an enclosing method declaration when the cursor is in the body.

No hover targets for:

- keyword tokens;
- string/number/bool literals;
- symbol literals;
- punctuation;
- comments;
- structural delimiters.

A symbol *binding* or a selector *reference object* can still acquire a hover if the cursor is on a semantic reference that is not merely the literal itself, but `#foo` as an obvious literal should not produce the old generic hover.

---

# 31. Expression coverage matrix

The unified analyzer should explicitly cover every current `Expr` variant.

| AST variant | Required semantic behavior |
|---|---|
| `Int` | exact `Int` instance |
| `Float` | exact `Float` |
| `String` | exact `String` |
| `Boolean` | exact `Bool` |
| `Symbol` | exact `Symbol`; no literal hover |
| `Var` | resolve `BindingId`/global/class/implicit getter according to compiler semantics |
| `Field` | resolve field identity + current joined field evidence |
| `SelfVar` | instance/class-side receiver appropriate to current callable |
| `SuperVar` | only valid as dispatch target; never ordinary value |
| `Assignment` | analyze RHS; update target flow; expression value follows language assignment semantics |
| `Range` | `Range<joined-bound-shape>` advisory knowledge |
| `Unary` | canonical operator send |
| `Binary` | canonical operator send; `and`/`or` lazy flow |
| `UnqualifiedCall` | lexical callable resolution first, implicit self last |
| `MethodCall` | ordinary/super side-aware dispatch |
| `ImplementationSelector` | privileged implicit receiver dispatch where applicable |
| `GetProperty` | getter dispatch or qualified module/class path |
| `SetProperty` | setter dispatch; expression result per compiler/language rule |
| `Index` | bracket selector dispatch |
| `SetIndex` | bracket setter dispatch; RHS/result semantics |
| `Block` | closure value knowledge + nested lexical scope + home-callable return propagation |
| `MethodRef` | `Family`/callable knowledge where surface is resolvable |
| `TupleLiteral` | recursively analyze all entries/expansions |
| `RecordLiteral` | recursively analyze fields/expansions |
| `MapLiteral` | recursively analyze keys/values/expansions |
| `SetLiteral` | recursively analyze elements/expansions |
| `ListLiteral` | recursively analyze elements/expansions |

The implementation spec must contain an exhaustive-match test so a future AST variant cannot silently fall into `_ => Unknown` without an explicit decision.

---

# 32. Statement-flow coverage

The flow analyzer should cover every `Statement`:

| Statement | Required behavior |
|---|---|
| `Let`/`Const` | analyze initializer, bind pattern, record exact declaration fact |
| assignment expression statement | update resolved binding/field fact |
| `Return` | record value into home callable return sink |
| ordinary expression | analyze for value/calls/effects |
| `For` | analyze iterable once, bind element shape in loop scope, analyze body, join loop-carried writes conservatively |
| `Break` | terminate current loop path |
| `Continue` | terminate current iteration path |
| `Throw` | analyze thrown expr, terminate normal path |
| `Class` | create class/member scopes; analyze member bodies independently |
| `Import` | graph/name binding only |

Parsed `while`/`if` sugar that appears as sends/blocks is covered by the expression/block analyzer. Native `ForStatement` remains a statement-flow case.

---

# 33. Future type-system relationship

This refactor should deliberately do work once.

## 33.1 What should be shared

The future type checker should reuse:

- `ModuleId`, `ClassId`, `CallableId`;
- `ScopeId`, `BindingId`;
- exact semantic occurrences;
- name resolution;
- dispatch resolution;
- inheritance traversal;
- statement/control-flow graph;
- expression identity/ranges;
- callable dependency graph;
- incremental invalidation/generations.

## 33.2 What should remain separate

Keep separate:

```text
ValueShape / InferredValue
```

from:

```text
formal Type / TypeExpression / subtype / equivalence / constraints
```

The current typing specification explicitly preserves:

- normal selector identity independent of types;
- ordinary dispatch independent of types;
- absent annotations as absent rather than fabricated `Dynamic`/`Any`/`Object`.

Therefore the future architecture should allow multiple evidence providers:

```rust
enum EvidenceSource {
    Syntax,
    RuntimeShapeInference,
    DeclaredType,
    ConstraintSolve,
}
```

without collapsing their semantics.

## 33.3 Extraction strategy

Do not create a new workspace analysis crate solely for aesthetic reasons right now.

Keep the new modules VM-free and LSP-protocol-free inside `phalcom-lsp/src/semantic`.

When the compiler/type checker becomes a second real consumer, extract the reusable layers to a shared crate with a mechanical move rather than a redesign.

This preserves momentum while avoiding editor-specific assumptions in the analysis core.

---

# 34. Implementation sequence

The following order minimizes rework.

## Phase 0 — protocol correctness fix

1. Set inlay `resolve_provider` to false/absent.
2. Add transport test.
3. Confirm VS Code no longer issues the noisy unsupported resolve request.

Independent and safe.

## Phase 1 — source precision

1. Add exact source spans to AST structures.
2. Update parser construction sites.
3. Update AST/parser snapshots/tests.
4. Change index declaration/reference recording to exact spans where possible.
5. Add exact hover-range tests before semantic redesign.

This removes the whole-method-hover failure at its source.

## Phase 2 — lexical semantic graph

1. Add `ScopeId`/`BindingId`.
2. Build scopes and declarations for module/member/block/loop.
3. Resolve variable occurrences to bindings.
4. Publish occurrence index.
5. Move hover local/parameter targeting to this index.
6. Replace completion `visible_names_at` with scope query.

## Phase 3 — canonical dispatch + unified expression analysis

1. Add `DispatchResolver`.
2. Move all call/getter/operator/subscript resolution through it.
3. Replace `infer_expr`, `infer_expr_with_returns`, `infer_expr_with_fields` with one analyzer.
4. Add native return contracts.
5. Add exact constructor return rule.
6. Implement `super` dispatch target.
7. Make interpolation succeed naturally.

## Phase 4 — unified statement flow

1. Sequential local flow.
2. scope-aware reassignment/shadowing;
3. nested blocks/loops;
4. non-local returns;
5. field evidence;
6. parameter argument evidence;
7. callable dependencies as analyzer events;
8. integrate results into fixed-point solver.

## Phase 5 — switch editor consumers

1. Hover entirely from semantic occurrences/facts.
2. Completion entirely from shared resolution/visible bindings.
3. Inlay hints query binding identities.
4. Navigation uses semantic occurrences and resolved members.
5. Semantic tokens optionally refine roles from occurrences.
6. Remove obsolete top-level-only/keyword/selector-range heuristics.

## Phase 6 — comprehensive regression matrix

Run unit + transport + fixture + extension E2E tests.

Only after these are green should old parallel walkers/helpers be removed.

---

# 35. Implementation decomposition for the follow-up specs

The detailed implementation work should be split into **four** focused specs, not ten small documents.

## Spec 1 — Source-precise semantic targets and scoped identities

Covers:

- AST range additions;
- parser changes;
- `ScopeId`/`BindingId`;
- semantic occurrence index;
- exact hover target rules;
- keyword/literal negative behavior;
- exact declaration/reference ranges;
- completion visible bindings.

## Spec 2 — Unified expression inference and canonical dispatch

Covers:

- `DispatchResolver`;
- inheritance and side;
- `super`;
- getters/setters;
- methods/unqualified calls;
- operators;
- subscripts;
- method refs;
- containers;
- string interpolation;
- native return contracts;
- constructor result contract.

## Spec 3 — Flow, callable summaries, fields, parameters, and invalidation

Covers:

- scoped statement flow;
- sequential locals/reassignment;
- nested body analysis;
- non-local block returns;
- callable return summaries;
- field evidence;
- parameter call-site inference;
- dependency/event extraction;
- fixed-point solver integration;
- invalidation.

## Spec 4 — LSP surfaces, protocol fix, test matrix, and typing bridge

Covers:

- inlay resolve `-32601` fix;
- hover renderer;
- completion/inlay/navigation adapters;
- semantic-token refinement;
- test fixtures and transport tests;
- VS Code E2E expectations;
- future formal-typing integration boundaries;
- cleanup/removal order.

This is the most compact decomposition that still gives an implementation agent enough local context to work without rereading the whole repository.

---

# 36. Key code-reference map for the implementation agent

Use these targeted reads first. Do not read entire large files.

| Purpose | Baseline target |
|---|---|
| syntax/basic/return/field expression inference split | `phalcom-lsp/src/semantic/infer.rs:20-245` |
| field facts + solver entry | `phalcom-lsp/src/semantic/infer.rs:220-430` |
| call-site collection | `phalcom-lsp/src/semantic/infer.rs:430-900` |
| local fact collection | `phalcom-lsp/src/semantic/infer.rs:880-1080` |
| callable summary setup | `phalcom-lsp/src/semantic/infer.rs:1030-1270` |
| summary expression/body flow | `phalcom-lsp/src/semantic/infer.rs:1030-1270` |
| dependency extraction | `phalcom-lsp/src/semantic/infer.rs:1270-1540` |
| semantic state/query/rebuild | `phalcom-lsp/src/semantic/mod.rs:100-520`, `500-920` |
| source class/member surface | `phalcom-lsp/src/semantic/surface.rs:1-300` |
| runtime-shape fact model | `phalcom-lsp/src/semantic/facts.rs` (`ValueShape`, `LocalFacts`, `FieldFacts`, `ParameterFacts`) |
| current selector target/index behavior | `phalcom-lsp/src/index.rs:430-780`, `780-1040` |
| current completion target/resolution UI | `phalcom-lsp/src/completion.rs:1-420` |
| current hover path | `phalcom-lsp/src/backend.rs:520-940` |
| semantic receiver + selector position | `phalcom-lsp/src/backend.rs:220-520` |
| LSP capability advertisement | `phalcom-lsp/src/backend.rs:900-1015` |
| current semantic tokens | `phalcom-lsp/src/semantic_tokens.rs:1-350`, `350-520` |
| current inlay hints | `phalcom-lsp/src/inlay_hints.rs` |
| hover keyword machinery | `phalcom-lsp/src/hover.rs:1-260` |
| AST member/param/binding shapes | `phalcom-ast/src/ast.rs:260-560` |
| Pattern/For/Expr variants | `phalcom-ast/src/ast.rs:560-840` |
| call/property/method-ref ranges | `phalcom-ast/src/ast.rs:840-1040` |
| closure parameter shape | `phalcom-ast/src/ast.rs:1080-1180` |
| interpolation lowering | `phalcom-ast/src/parser.rs:2100-2225` |
| runtime send semantics / property / super | `phalcom-core/src/compiler/lib/expr.rs:360-760` |
| runtime operator/block/bare-super semantics | `phalcom-core/src/compiler/lib/expr.rs:1160-1320` |
| constructor lowering contract | `phalcom-core/src/compiler/attributes.rs:2300-2445` |
| canonical native surface | `phalcom-native-surface/src/lib.rs:1-260`, `240-430` |
| live core assembly | `phalcom-lsp/src/semantic/core_source.rs:1-80` |
| typing design boundary | `docs/spec/typing/STATUS.md` |
| non-local block return semantics | `docs/spec/current/blocks.md` |

---

# 37. Risks and implementation traps

## 37.1 Do not patch only `hover_at`

Changing only `Hover::range` cannot fix target identity. If the index says the whole method is the selector occurrence, the backend still selects the wrong semantic thing.

Fix source ranges and occurrence indexing.

## 37.2 Do not infer `super` as simply `Instance(superclass)`

That loses lookup-start semantics and can be wrong for class-side dispatch.

Use a dedicated super dispatch target.

## 37.3 Do not keep both side-aware and side-blind dispatch in active semantic paths

As long as both exist, regressions will recur.

Consolidate.

## 37.4 Do not key locals by string spelling

That prevents correct shadowing and makes future typing painful.

Introduce binding identity now.

## 37.5 Do not make string interpolation a one-off special case and stop

It is a useful regression test, but the real missing feature is complete getter/operator propagation.

## 37.6 Do not turn every native primitive into guessed return metadata

Only add stable semantic contracts.

`Unknown` is preferable to an attractive incorrect hover/completion result.

## 37.7 Do not conflate semantic hover with semantic tokens

A keyword can remain syntax-colored while returning no hover.

## 37.8 Do not create a second formal type system in the LSP

Keep runtime shape facts separate. Reuse infrastructure, not semantics.

## 37.9 Do not rewrite the incremental database

Its generation/invalidation design is useful. Replace the incomplete per-feature walkers beneath it.

---

# 38. Acceptance criteria for the entire effort

The implementation is complete when all of the following hold.

### Hover

- Hovering a method body does not select the enclosing method declaration.
- Method declaration hover range is only its selector/name.
- Call-site hover range is only the selector/property/subscript/operator target.
- Local/const declarations and all references/reassignments resolve to the same lexical binding.
- Method/setter/index/closure/for parameters resolve individually.
- Classes resolve individually.
- No hover is returned for keywords or obvious literals/symbol literals.
- Nested bodies behave the same as top-level code.

### Inference

- Getter/property return values propagate.
- Inherited methods/getters resolve to the actual declaring callable.
- Class-side and instance-side same-selector members never collide.
- `super.method` and `super.property` resolve from the superclass with correct side.
- Interpolated strings infer `String`.
- Unary/binary operators use dispatch return knowledge.
- Subscript reads/writes participate in inference/dependencies.
- Sequential locals affect method return summaries.
- Nested returns are included; block `return` is non-local to the home callable.
- Constructor factories always return owning-class instances.
- Constructor field writes can use parameter facts.
- Shadowing never leaks facts across scopes/methods.

### Cross-feature consistency

- Completion, hover, inlay hints, summaries, and dependency resolution consume the same underlying facts.
- Editing an upstream getter/method invalidates dependent summaries.
- No unsupported `inlayHint/resolve` is advertised.
- Existing module-aware/live-core behavior remains intact.

---

# 39. Final architectural recommendation

The current LSP is close to having the right *outer* architecture but not yet the right *semantic core*.

Do not continue expanding the current pattern of:

```text
one walker for locals
one walker for returns
one walker for dependencies
one walker for parameter facts
one selector collector for hover
```

That approach will keep producing feature-specific blind spots.

The next implementation should make Phalcom source analysis look more like:

```text
AST with exact spans
        |
        v
Scope + Binding + Occurrence graph
        |
        v
Canonical name/dispatch resolution
        |
        v
Unified expression + statement-flow analysis
        |
        +--> runtime ValueShape facts
        +--> callable summaries
        +--> parameter evidence
        +--> field evidence
        +--> dependency edges
        |
        v
SemanticDb coherent generation
        |
        +--> hover
        +--> completion
        +--> inlay hints
        +--> navigation
        +--> semantic-token refinement
        +--> future type checker
```

That is the level at which the LSP stops being a collection of editor features and becomes a real language semantic service.

The four follow-up implementation specifications should implement exactly this architecture in focused slices.
