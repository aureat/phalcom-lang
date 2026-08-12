# Phalcom LSP Implementation Spec 1
## Source-Precise Semantic Targets, Lexical Scopes, and Binding Identity

**Repository baseline:** `e2ec9e5fb6dc362786c9dd9593470feb47c91d94`  
**Depends on:** `PHALCOM_LSP_ANALYSIS_DIAGNOSIS_AND_PLAN.md`  
**Primary crates:** `phalcom-ast`, `phalcom-lsp`  
**Goal:** make every hover/navigation/binding query refer to the exact semantic token under the cursor, with correct lexical identity.

---

# 1. Scope

This specification implements the source-semantic foundation required by all later inference work.

It MUST:

1. preserve exact ranges for source names/selectors/operators that currently only have whole-expression ranges;
2. introduce lexical `ScopeId` and `BindingId`;
3. resolve declaration/read/write occurrences to identities;
4. build one exact semantic occurrence index;
5. change hover target selection from enclosing selector ranges to exact semantic occurrences;
6. remove keyword/literal hover behavior;
7. provide `visible_bindings_at(offset)` for completion;
8. preserve existing workspace selector navigation while migrating it to exact ranges.

This spec MUST NOT implement the full expression inference rewrite. That is Spec 2.

---

# 2. Targeted baseline reads

Do not read whole files. Start with these slices/functions.

| Area | Target |
|---|---|
| AST member/parameter/binding definitions | `phalcom-ast/src/ast.rs:260-560` |
| Pattern/For/Expr | `phalcom-ast/src/ast.rs:560-840` |
| call/property/method-ref nodes | `phalcom-ast/src/ast.rs:840-1040` |
| closure parameters | `phalcom-ast/src/ast.rs:1080-1180` |
| interpolation AST construction example | `phalcom-ast/src/parser.rs:2100-2225` |
| selector target/index code | `phalcom-lsp/src/index.rs:430-780` |
| selector collector | `phalcom-lsp/src/index.rs:780-1040` |
| semantic class/member surfaces | `phalcom-lsp/src/semantic/surface.rs:1-300` |
| current semantic tokens | `phalcom-lsp/src/semantic_tokens.rs:350-520` |
| current completion scope helper | `phalcom-lsp/src/completion.rs:1-420` |
| current hover route | `phalcom-lsp/src/backend.rs:520-940` |
| current selector-position route | `phalcom-lsp/src/backend.rs:220-520` |
| keyword hover helpers | `phalcom-lsp/src/hover.rs:1-260` |

For parser construction sites, use targeted search:

```bash
rg -n "MethodCallExpr \{|UnqualifiedCallExpr \{|GetPropertyExpr \{|SetPropertyExpr \{|BinaryExpr \{|UnaryExpr \{|ForStatement \{|ClosureParameters" phalcom-ast phalcom-core phalcom-lsp
```

Do not open all matching files at once. Open only the local function surrounding each constructor.

---

# 3. AST range additions

## 3.1 Call selectors

Amend:

```rust
pub struct MethodCallExpr {
    pub object: Expr,
    pub method: String,
    pub method_range: SourceRange,
    pub args: Vec<PackItem>,
    pub range: SourceRange,
}

pub struct UnqualifiedCallExpr {
    pub name: String,
    pub name_range: SourceRange,
    pub args: Vec<PackItem>,
    pub range: SourceRange,
}

pub struct GetPropertyExpr {
    pub object: Expr,
    pub property: String,
    pub property_range: SourceRange,
    pub range: SourceRange,
}

pub struct SetPropertyExpr {
    pub object: Expr,
    pub property: String,
    pub property_range: SourceRange,
    pub value: Expr,
    pub range: SourceRange,
}
```

The new ranges MUST cover exactly the source spelling of the selector/property name, excluding:

- receiver;
- dot;
- parentheses;
- arguments;
- assignment operator.

When parser-synthesized AST has no faithful source token (e.g. interpolation-generated `toString`), use a clearly synthetic range policy rather than pretending the whole interpolated string is the selector token. Recommended:

```rust
pub enum SourceSpan {
    Written(SourceRange),
    Synthetic(SourceRange),
}
```

If introducing that wrapper is too invasive for this slice, keep `SourceRange` but ensure synthetic selector occurrences are **not inserted into the user-facing semantic occurrence index**.

## 3.2 Operators

Amend:

```rust
pub struct BinaryExpr {
    pub op: BinaryOp,
    pub op_range: SourceRange,
    pub left: Expr,
    pub right: Expr,
    pub range: SourceRange,
}

pub struct UnaryExpr {
    pub op: UnaryOp,
    pub op_range: SourceRange,
    pub expr: Expr,
    pub range: SourceRange,
}
```

`op_range` MUST be only the written operator token (`+`, `==`, `and`, `not`, etc.).

For parser-generated binary expressions such as interpolation lowering, mark/handle as synthetic so they do not create fake hover targets over the string.

## 3.3 Loop binding

Amend:

```rust
pub struct ForStatement {
    pub binding: String,
    pub binding_range: SourceRange,
    pub iter: Expr,
    pub body: Vec<Statement>,
    pub range: SourceRange,
}
```

Do not continue using the whole `for` statement range as the loop variable's source location.

## 3.4 Closure parameters

Replace string-only closure params with source-bearing params.

Recommended:

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClosureParameter {
    pub name: String,
    pub range: SourceRange,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ClosureParameters {
    pub fixed: Vec<ClosureParameter>,
    pub positional_rest: Option<ClosureParameter>,
}
```

Update compiler consumers to use `.name`.

Do not re-derive closure parameter ranges in the LSP by scanning text.

## 3.5 Method/member parameters

Amend `ParameterDef`:

```rust
pub struct ParameterDef {
    pub name: String,
    pub name_range: SourceRange,
    pub label: Option<String>,
    pub label_range: Option<SourceRange>,
    pub rest_mode: RestMode,
    pub range: SourceRange,
}
```

Rules:

- `name_range` is the local binding token.
- `label_range` is the external label token when source contains a distinct label.
- If one token serves as both external and local name, the ranges may be equal.
- positional `_ local` forms: `_` is not the local binding; `name_range` is `local`.
- rest marker is not part of `name_range`.

Update `ParamSurface` later to retain both where useful.

## 3.6 Field/variant declarations

Add:

```rust
FieldDef.name_range
VariantDef.name_range
```

Do not use `FieldDef.range` or `VariantDef.range` as a semantic name occurrence.

## 3.7 Method references

Add an exact selector range at the `MethodRefExpr` level:

```rust
pub struct MethodRefExpr {
    pub receiver: Expr,
    pub kind: MethodRefKind,
    pub selector_range: SourceRange,
    pub range: SourceRange,
}
```

For:

```phalcom
obj::name
obj::#name(_,to)
```

the range covers `name` or the selector-symbol portion as appropriate, excluding the receiver and `::`.

A symbol literal remains a literal for hover policy. A pinned method reference, however, is a semantic member reference. The occurrence index may classify this as `MemberReference` because the syntactic construct is a method reference rather than a free symbol literal.

## 3.8 Subscripts

`IndexMethodDef` already has `name_range`.

For call sites add:

```rust
IndexExpr.selector_range
SetIndexExpr.selector_range
```

Use the bracketed selector portion only.

For:

```phalcom
xs[i]
```

target range should be `[i]` or, if the UI should emphasize selector syntax rather than argument expression, at minimum the bracket token span. Prefer the full bracket selector span because bracket methods have no independent method-name token.

For:

```phalcom
xs[i] = v
```

the selector range should include the bracket selector and assignment form only if it remains a single contiguous source range. If that becomes visually noisy, target the bracket selector and render the setter identity in hover text.

Lock the convention in tests.

---

# 4. Parser updates

Every new range MUST be captured at parse time from lexer token boundaries.

Do not compute it later by searching source strings.

After changing a parser constructor:

1. update only that local parser function;
2. run its parser/unit/snapshot tests;
3. then move to the next constructor.

Important parser-generated nodes:

- string interpolation creates synthetic `GetProperty(toString)` and `Binary(Add)`;
- coalescing/control-flow sugar creates synthetic calls/blocks.

Synthetic nodes MUST preserve analysis semantics without becoming misleading written-source occurrences.

Recommended helper:

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SpanOrigin {
    Written,
    Synthetic,
}
```

If adding origin to every AST node is too broad, store written target ranges as `Option<SourceRange>`:

```rust
pub method_range: Option<SourceRange>
```

where `None` means compiler/parser-generated. This is acceptable and may be simpler than a global `SourceSpan` wrapper.

**Preferred decision for this implementation:** use `Option<SourceRange>` for newly added *target* ranges where nodes can be synthetic. Existing declaration name ranges remain required because declarations are written.

---

# 5. Add lexical scope identities

Create:

```text
phalcom-lsp/src/semantic/scope.rs
```

Recommended public-internal model:

```rust
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ScopeId(u32);

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct BindingId(u32);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SemanticBindingKind {
    TopLevelLet,
    TopLevelConst,
    LocalLet,
    LocalConst,
    MethodParameter,
    SetterParameter,
    IndexParameter,
    ClosureParameter,
    ForBinding,
    Destructure,
}

#[derive(Clone, Debug)]
pub struct BindingInfo {
    pub id: BindingId,
    pub scope: ScopeId,
    pub name: String,
    pub kind: SemanticBindingKind,
    pub declaration_range: SourceRange,
    pub mutable: bool,
}

#[derive(Clone, Debug)]
pub struct ScopeInfo {
    pub id: ScopeId,
    pub parent: Option<ScopeId>,
    pub range: SourceRange,
    pub bindings: BTreeMap<String, BindingId>,
}
```

`BindingId` only needs to be stable within one file semantic snapshot. Do not invent persistent IDs across reparses yet.

---

# 6. Scope construction rules

Build scopes from the recovered AST.

## 6.1 Module scope

Contains:

- top-level `let` / `const`;
- imports;
- class declarations as names, although classes may remain separate `ClassId` symbols rather than `BindingId` if preferred.

## 6.2 Member scope

Each method/getter/setter/index/constructor body gets an independent lexical scope.

Parameters are declared before body statements.

No facts may leak between same-named locals in different members.

## 6.3 Block scope

Every `Expr::Block` gets a child scope.

Closure parameters bind in that scope.

Body declarations shadow parents.

## 6.4 `for` scope

The loop binding is visible in the body only.

Recommended:

```text
parent flow scope
  -> loop body ScopeId
       binding
       statements
```

The iterable expression is analyzed in the parent scope, before the loop binding exists.

## 6.5 Destructuring

Every `Pattern::Name` creates its own `BindingId`.

For:

```phalcom
let (a, [b, *rest]) = value
```

create independent IDs for `a`, `b`, `rest`.

## 6.6 Shadowing

Nearest lexical scope wins.

Same-scope redeclaration is already a compiler error; the LSP scope builder should still recover deterministically from malformed source:

- keep the first valid binding as the resolution target;
- optionally retain an error/recovery binding for editor targeting;
- do not panic.

---

# 7. Resolve variable occurrences

During the scope/occurrence walk, resolve each `Expr::Var` as:

1. nearest lexical binding;
2. module/global/import/class name;
3. implicit-self message candidate;
4. unresolved.

Do not decide value shape here.

This layer answers *what name is this?*, not *what value does it have?*

Represent:

```rust
pub enum NameResolution {
    Binding(BindingId),
    Class(ClassId),
    Module(ModuleId),
    ImplicitSelf,
    Global(String),
    Unresolved,
}
```

The exact global model may be refined later; the critical requirement is that a parameter/local with `Unknown` value still resolves as a binding and shadows outer/class names.

---

# 8. Add semantic occurrence index

Create:

```text
phalcom-lsp/src/semantic/occurrence.rs
```

Recommended model:

```rust
#[derive(Clone, Debug)]
pub struct SemanticOccurrence {
    pub range: SourceRange,
    pub kind: SemanticOccurrenceKind,
    pub role: OccurrenceRole,
    pub target: SemanticTarget,
}

pub enum OccurrenceRole {
    Declaration,
    Read,
    Write,
    Call,
    Reference,
}

pub enum SemanticTarget {
    Binding(BindingId),
    Class(ClassId),
    Callable(CallableId),
    Field(FieldId),       // can be introduced in Spec 3 if not ready
    Operator(OperatorTarget),
}

pub enum SemanticOccurrenceKind {
    Binding,
    Parameter,
    Class,
    Member,
    Field,
    Operator,
}
```

Store occurrences sorted by `(start, end)`.

Query:

```rust
pub fn occurrence_at(offset: usize) -> Option<&SemanticOccurrence>
```

Selection rule:

1. range must contain `offset`;
2. choose shortest range;
3. for exact ties use semantic precedence:
   binding/parameter > member > class > operator.

Never add occurrences for:

- keywords;
- numeric/string/bool literals;
- free symbol literals;
- punctuation;
- whitespace.

---

# 9. Replace selector hover targeting

Current source:

- `index::selector_at_offset`
- `Backend::selector_at_position`
- `Backend::hover_at`

The existing selector index still has value for workspace references, but it MUST NOT be the primary hover-target detector.

New path:

```text
position
-> UTF-16 offset
-> semantic.occurrence_at(uri, offset)
-> render occurrence
```

For a method declaration, only `method.name_range` becomes a `Callable` occurrence.

For a method call, only `method_range` becomes a `Callable` occurrence after semantic dispatch resolves it. Before Spec 2 lands, it may carry a selector-only unresolved target; convert to `CallableId` once dispatch is available.

---

# 10. Remove keyword/literal hover path

In `Backend::hover_at`, delete the early:

```text
hover::keyword_at_offset
hover::keyword_blurb
```

branch.

Do not replace it with literal hovers.

If `hover.rs` keyword documentation becomes unused after tests are migrated, remove:

- keyword table;
- keyword token scanning;
- related rendering functions.

Do not delete Phaldoc rendering/harvesting helpers that remain used.

---

# 11. Binding hover rendering

Introduce a binding renderer in `hover.rs`.

Minimum output:

```text
let local: Savings
```

or:

```text
parameter owner: String
```

or, if unknown:

```text
parameter owner: ?
```

Do not suppress the hover merely because value knowledge is unknown.

If Phaldoc exists for a top-level binding, append it.

Do not require Phaldoc to produce a semantic binding hover.

Set `Hover::range` to the exact occurrence range.

---

# 12. Class hover

Keep class hover behavior, but source-target it through `SemanticOccurrence`.

Avoid the current generic text fallback (`qualified_identifier_at_offset`) for normal hover once class reference occurrences are indexed.

Text recovery may remain only for malformed/incomplete editor state if it is proven necessary.

---

# 13. Update `WorkspaceIndex` occurrence ranges

In `phalcom-lsp/src/index.rs`, change collector entries:

### Declarations

Use:

- class/member exact name/selector ranges;
- not whole declaration ranges.

### References

Use newly retained target ranges:

- `MethodCall.method_range`;
- `UnqualifiedCall.name_range`;
- `GetProperty.property_range`;
- `SetProperty.property_range`;
- `MethodRef.selector_range`;
- index selector range.

Do not add synthetic selector ranges.

This improves:

- go to definition;
- find references;
- hover compatibility;
- exact locations in UI.

Keep the canonical comma-form selector key unchanged.

---

# 14. Completion visible names

Replace `completion.rs::visible_names_at` with:

```rust
SemanticDb::visible_bindings_at(uri, offset)
```

Then merge:

- lexical bindings;
- imported/module names;
- classes;
- implicit-self members as current completion does.

The important behavior is nested scope visibility.

Test:

```phalcom
class C {
  m(p) {
    let x = 1
    || {
      let y = 2
      /* completion here */
    }
  }
}
```

Must include `p`, `x`, `y`, but not locals from another method.

---

# 15. Semantic-token refinement

This is secondary but cheap after occurrences exist.

Do not replace the lexer token pass.

Instead, during AST/semantic override:

- class occurrences -> `CLASS`;
- callable declaration/reference -> `METHOD`;
- parameter occurrences -> add standard `PARAMETER` token type;
- binding occurrences -> `VARIABLE`;
- fields/properties -> add `PROPERTY`.

Keep:

- keyword tokens;
- string/number tokens;
- symbol syntax coloring;
- operator tokens.

Again: hover suppression is independent from coloring.

---

# 16. `SemanticDb` snapshot changes

Extend `FileSemanticSnapshot` with:

```rust
pub scopes: ScopeGraph,
pub occurrences: OccurrenceIndex,
```

Build them before flow/value inference.

Recommended update order:

```text
parse
surface
scope graph
syntactic occurrences/name resolution
flow/inference facts
resolved callable occurrences
publish generation
```

Spec 2 may enrich unresolved member occurrences with exact `CallableId`.

---

# 17. Tests

## 17.1 AST/parser tests

Add assertions for exact ranges for:

- method call;
- getter access;
- setter access;
- unqualified call;
- binary operator;
- unary operator;
- for binding;
- closure parameter;
- parameter external/local names;
- field declaration;
- method reference;
- index read/write.

Use source slicing:

```rust
assert_eq!(&source[range.start..range.end], "toString");
```

This is more robust than hardcoded byte offsets.

## 17.2 Hover negative tests

Replace the existing Stage 4 keyword success expectation.

Assert `result == null` on:

- `let`;
- `class`;
- `return`;
- `true`;
- `123`;
- `"text"`;
- `#deposit`.

## 17.3 Exact hover range tests

For each:

- method declaration;
- getter declaration;
- method call;
- getter call;
- class name;
- local declaration;
- local read;
- local write;
- parameter declaration;
- parameter read;
- closure parameter;
- for binding.

Assert returned LSP range slices exactly to the semantic token and never the enclosing declaration.

## 17.4 Scope tests

Must cover:

- same local name in two methods;
- nested shadowing;
- closure capture;
- for binding visibility;
- destructuring;
- parameter shadows class name;
- reassignment targets correct binding.

---

# 18. Acceptance gate

Spec 1 is complete only when:

1. hovering whitespace/body text cannot select an enclosing method;
2. all binding declarations/references use lexical identity;
3. exact target spans come from AST/parser, not text search;
4. keywords/literals return no hover;
5. nested scope completion sees correct bindings;
6. existing selector navigation still works with exact ranges;
7. parser/compiler tests compile after AST field additions.

Do not start deleting the old selector/index helpers until replacement tests are green.
