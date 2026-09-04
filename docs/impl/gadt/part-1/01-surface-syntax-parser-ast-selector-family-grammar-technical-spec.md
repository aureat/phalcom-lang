# Phalcom ADT/GADT + Associated Lookup — Technical Specification 01

## Surface Syntax, Parser, AST, and Selector/Family Grammar

**Status:** Proposed implementation specification — Part 1 of 6  
**Baseline repository:** `aureat/phalcom-lang`  
**Baseline branch:** `main`  
**Baseline commit:** `1892bcff51f75dd2f3df2a0661b03371250d4090`  
**Baseline commit subject:** `docs(semantic): record correctness plans and authority audit`  
**Specification series:** ADT/GADT + associated lookup implementation, Part 1/6  
**Proposed repository path:** `docs/impl/adt-gadt-associated-lookup/part-1/01-surface-syntax-parser-ast-selector-family-grammar-technical-spec.md`

---

# 1. Scope

This specification establishes the **front-end language contract** required by the ratified Phalcom enum/ADT/GADT and associated-lookup model. It defines the syntax, parser behavior, AST boundaries, selector-shape normalization rules, source ranges, syntax diagnostics, and migration rules needed before declaration identity, typing, runtime representation, or `match` elimination can be implemented soundly.

Part 1 deliberately stops at the boundary between syntax and semantics. It must make the parsed program say the right thing without pretending that the existing method-family runtime already implements the new meaning of `::`.

The core front-end goals are:

1. introduce a real `enum` declaration;
2. make `@variant` an explicit enum variant declaration rather than an attribute-driven sealed-class macro surface;
3. preserve the ratified distinction between `.` message sending and `::` associated lookup;
4. represent exact associated lookup, whole-family lookup, exact member narrowing, and direct associated-family invocation as semantically distinct AST forms;
5. retain the existing structural selector identity system where it already matches the ratified design;
6. eliminate the current parser ambiguity in which `receiver::name(...)` is interpreted as a selector reference rather than an associated call;
7. keep source information precise enough for later semantic indexing and LSP work without embedding LSP semantics in the parser;
8. prevent the new syntax from accidentally lowering through the legacy `MakeFamily`/ordinary-dispatch path.

This part does **not** implement variant identity, type checking, GADT proof obligations, runtime case classes, optimized values, exhaustiveness, or final IDE behavior. Those belong to later parts and consume the AST contract defined here.

---

# 2. Source-of-truth note and archaeology limitation

The handoff requires reading an older enum/ADT design document and a newer associated-lookup/family design document. In this conversation, those attachments were not discoverable through the available uploaded-file index. No content from unavailable attachments is inferred or fabricated.

Therefore this specification applies the required precedence as follows:

1. the ratified decisions in the handoff are normative;
2. current repository `main` is authoritative for implementation reality;
3. attachment-only details that cannot be verified are surfaced as explicit assumptions/open questions rather than silently invented.

If the unavailable design documents are later supplied and contradict a non-ratified assumption in this Part 1 specification, revise that assumption before implementation. Ratified handoff decisions do not reopen.

---

# 3. Repository baseline and current implementation findings

## 3.1 Front-end layout

At baseline commit `1892bcff...`, the relevant front-end is concentrated in:

- `phalcom-ast/src/token.rs`
- `phalcom-ast/src/lexer.rs`
- `phalcom-ast/src/parser.rs`
- `phalcom-ast/src/ast.rs`
- `phalcom-ast/src/selector.rs`
- `phalcom-ast/src/error.rs`
- `phalcom-common/src/selector.rs`
- `phalcom-ast/tests/family_selector_syntax.rs`
- `phalcom-ast/tests/lexer.rs`
- `phalcom-ast/tests/parser.rs`

The parser is hand-written recursive descent plus Pratt/postfix parsing. `Parser::parse_call` owns postfix `.` and `::` expression grammar. This is a good architectural location for the new associated syntax; no parser generator or parallel grammar should be introduced.

## 3.2 `enum` does not currently exist

`phalcom-ast/src/token.rs` has a `Class` token and the lexer maps `"class"` to it, but there is no `Enum` token. `enum` therefore lexes as an ordinary identifier.

`phalcom-ast/src/ast.rs::Statement` contains `Class`, `TypeAlias`, `Let`, `Return`, expression statements, loops, throws, and exports, but no `Enum` declaration node.

`Parser::parse_top_item` and parser synchronization similarly have no enum branch.

**Consequence:** a real `enum` declaration must be introduced at token, parser, AST, recovery, and downstream exhaustive-match boundaries.

## 3.3 Current `@variant` is a legacy sealed-class macro surface

`phalcom-ast/src/ast.rs` currently defines:

```rust
pub enum ClassMember {
    Method(MethodDef),
    Getter(GetterDef),
    Setter(SetterDef),
    Field(FieldDef),
    Variant(VariantDef),
    Index(IndexMethodDef),
}
```

The current `VariantDef` is documented as a `@variant Name(label1:, label2:, ...)` arm inside an `@sealed` class. Its payload is only `labels: Vec<String>`; it has no typed constructor parameters, no GADT result annotation, no case body, and no enum owner identity. Its compiler contract is to be expanded into sibling ordinary classes.

`Parser::parse_class_body` recognizes a pending `@variant` attribute and diverts to the legacy `parse_variant_decl`. That parser requires colon-bearing field-label syntax such as:

```phalcom
@variant Circle(radius:)
```

This is not the new enum variant grammar.

`phalcom-core/src/compiler/lib/class_decl.rs` states that `compiler::attributes::expand_variants` strips these nodes before ordinary class compilation. `ClassMember::Variant` is treated as unreachable after expansion.

`phalcom-semantic/src/checker/declaration.rs::register_class_surface` explicitly ignores `ClassMember::Variant(_)`.

**Consequence:** the legacy node is not a partially complete implementation of the ratified enum model. It is a different architecture. The new parser must create a dedicated `EnumDef`/`VariantDecl` AST. The old sealed-class expansion can remain temporarily as dead compatibility machinery, but new enum syntax must never flow through it.

## 3.4 Structural selector identity is already largely correct

`phalcom-common/src/selector.rs` is reusable and should remain the shared selector identity layer.

Current types include:

```rust
pub enum SelectorKind {
    Getter,
    Setter,
    Method,
    SubscriptGet,
    SubscriptSet,
}

pub enum SelectorSlot {
    Positional,
    Label(String),
}

pub struct Selector {
    pub base: SelectorBase,
    pub kind: SelectorKind,
    pub slots: Box<[SelectorSlot]>,
}
```

`Selector::encode()` already produces the ratified comma-form identity:

```text
name
name()
name(_,reason)
name=(put)
[_]
[_,_]=(put)
```

External labels are represented as `SelectorSlot::Label("reason")`; **no colon is part of selector identity**. Getter `#name` and zero-argument method `#name()` are already distinct through `SelectorKind`.

**Consequence:** do not replace or owner-qualify `Selector` in Part 1. Owner qualification belongs to semantic identities such as `VariantId` in Part 2. Preserve this selector core and adapt the surface grammar around it.

## 3.5 Current `::` is a method-reference/family operator

`phalcom-ast/src/ast.rs` currently exposes:

```rust
Expr::MethodRef(Box<MethodRefExpr>)
```

with:

```rust
pub struct MethodRefExpr {
    pub receiver: Expr,
    pub spec: SelectorSpecSyntax,
    pub kind: MethodRefKind,
    pub selector_range: Option<SourceRange>,
    pub range: SourceRange,
}
```

The comments and compatibility enum `MethodRefKind` describe `::` as a bound method-family reference. `Parser::parse_call` consumes `::`, calls `parse_selector_spec_after_colon_colon`, and constructs this node.

The current test `phalcom-ast/tests/family_selector_syntax.rs` proves the old grammar:

```phalcom
receiver::name       // current exact getter selector spec
receiver::name()     // current exact zero-argument selector spec
receiver::name(_)    // current exact unary selector spec
receiver::name...    // current family/pattern form
```

That is materially incompatible with the ratified grammar, where:

```phalcom
receiver::name          // exact getter-shaped associated lookup
receiver::name()        // direct associated-family invocation, zero arguments
receiver::name::()      // exact zero-argument member reference
receiver::name::(_)     // exact unary member reference
receiver::name::*       // whole family
```

**Consequence:** `MethodRefExpr` cannot remain the canonical AST representation of `::`.

## 3.6 Existing `SelectorSpecSyntax` still has a valid job

The AST also contains `SelectorSpecSyntax`, `ExactSelectorSyntax`, `SelectorPatternSyntax`, and normalization to common `Selector`/`SelectorPattern` values. These are used for first-class `#` selector and selector-pattern values.

Current tests intentionally preserve forms such as:

```phalcom
#name
#name()
#name(_)
#name...
#name(_, ..., foo)
```

The ratified family syntax does not require deleting the reflection-oriented selector-pattern language.

**Consequence:** keep `SelectorSpecSyntax` for `#` selector values/pattern values. Do not reuse its `...` pattern semantics to represent associated whole-family lookup. Associated family reification is the distinct syntax `::*`.

## 3.7 Dot is already structurally a send

`Parser::parse_call` currently parses:

```phalcom
receiver.name
receiver.name(args)
```

into property/message-send AST forms. `phalcom-core/src/compiler/lib/expr.rs` lowers property reads and method calls through ordinary `Bytecode::Invoke` / dynamic-pack dispatch. The VM path performs method lookup, inherited behavior lookup, rest-family routing, and `doesNotUnderstand` fallback.

That is exactly the behavioral machinery that `.` should continue to use.

**Consequence:** Part 1 does not generalize `.`. It explicitly prevents class-object dot syntax from being repurposed for associated declarations.

## 3.8 Existing runtime family machinery must not define new `::` semantics

`phalcom-core/src/bytecode.rs` defines:

```rust
Bytecode::MakeFamily {
    spec: u16,
    kind: FamilySpecKind,
}
```

Its documentation says it builds a bound `::` **method-reference Family** and deliberately defers future invocation to the existing dispatcher.

`phalcom-core/src/compiler/lib/expr.rs` lowers `Expr::MethodRef` to `MakeFamily`. It also has `immediate_exact_method_ref_selector`, an optimization that can bypass Family materialization for a narrow immediately-called exact-method case.

The VM's method path (`phalcom-core/src/vm/dispatch.rs`) performs exact lookup, rest-family lookup, and dNU fallback.

The ratified associated lookup model expressly forbids defining `::` this way:

- associated lookup does not fall back to ordinary message dispatch;
- it does not call dNU;
- `owner::family(args)` is a direct associated-family invocation path, not “make a family object, then call it”; and
- variants must not be represented as methods merely to reuse method dispatch.

**Consequence:** new associated AST nodes must not lower through `Bytecode::MakeFamily` in Part 1. Runtime generalization/replacement belongs to Parts 3–4.

## 3.9 Current `match` foundation is absent, but pattern infrastructure exists

There is no dedicated `MatchExpr`, `Expr::Match`, or `Statement::Match` at this baseline. However, `phalcom-ast::ast::Pattern` already supports destructuring and refutable use by constructs such as `if let` / `while let`.

**Consequence:** Part 1 does not invent a general `match` grammar. Part 5 will introduce the minimum sound match foundation while reusing existing pattern infrastructure. The new enum AST must be compatible with that later work.

## 3.10 Documentation organization

Current detailed implementation work is stored under `docs/impl/...`; `docs/implementation/` currently contains the roadmap rather than detailed active specifications.

A dedicated path is therefore appropriate:

```text
docs/impl/adt-gadt-associated-lookup/
  part-1/
  part-2/
  ...
  part-6/
```

This keeps the cross-cutting workstream separate from `docs/impl/semantic/semantic-correctness/...` while remaining in the repository's active implementation-spec area.

---

# 4. Stale design reconciliation

| Current/stale assumption | Ratified replacement | Part 1 implementation consequence |
|---|---|---|
| `@variant` arm in `@sealed class` | real `enum` + explicit `@variant` | add dedicated enum/variant AST; stop parsing new variants as `ClassMember::Variant` |
| variant payload is `label:` field list | variant payload is typed selector-shaped parameters | reuse `ParameterDef`; selector labels are colon-free identities |
| variant expands to sibling ordinary class | exact case semantics come later | parser records declaration, does not synthesize classes |
| legacy dot-access variant construction | `Option::Some(...)` | dot remains send-only; no dot variant AST special case |
| `receiver::name` is an open/bound method family | exact getter-shaped associated member | replace `MethodRefExpr` as canonical `::` AST |
| `receiver::name()` is exact zero-arg method reference | direct associated-family invocation | exact zero-arg reference becomes `receiver::name::()` |
| `receiver::name(_)` is exact ref | direct call grammar owns first parentheses | exact ref becomes `receiver::name::(_)` |
| `receiver::name...` is family/pattern | whole family is `receiver::name::*` | reject associated ellipsis; preserve `#name...` reflection patterns |
| variant is mapped to getter selector in `selector_from_member` | variant is a distinct semantic declaration/callable | add variant selector-shape helper without making it a class member method |
| associated lookup can eventually route through method Family + dNU | associated lookup has its own resolution path | Part 1 AST must preserve this separation |
| selector labels may appear colon-bearing in old prose | label identity is bare name | keep `SelectorSlot::Label(String)` and colon-free encoding |

---

# 5. Goals

Part 1 is complete when all of the following are true:

1. the lexer recognizes `enum` as a keyword;
2. the parser produces a dedicated enum AST for typed singleton/payload variants;
3. the parser preserves optional GADT result syntax and variant bodies;
4. signature-only enum-root behavior remains representable for explicit shared contracts;
5. dot parsing remains ordinary message sending with no associated-lookup exception;
6. `::` has an explicit associated syntax AST, not a method-reference AST;
7. bare associated name, whole family, exact narrowing, setter, operator, subscript, and direct associated call forms are syntactically distinguishable;
8. getter and zero-argument method shapes remain distinct;
9. `::*` is the only canonical associated whole-family syntax;
10. associated ellipsis forms are rejected with migration guidance;
11. selector identities remain structural and colon-free;
12. parser/source ranges retain enough spelling information to distinguish `owner::name` from explicit alias `owner::name::` without producing different semantic selector identities later;
13. compiler/semantic/LSP consumers compile against the new AST without inventing duplicate semantics;
14. new associated expressions do not silently lower via `MakeFamily` or ordinary send fallback;
15. focused parser/AST tests encode the new language contract.

---

# 6. Non-goals

Part 1 does not implement:

- `VariantFamilyId` or `VariantId` semantic allocation;
- family-reservation collision checking;
- exact enum-case types in `TypeData`;
- GADT equality solving;
- constructor typing or generic constructor schemes;
- associated member resolution;
- method/variant overload selection;
- inherited behavioral family reservation;
- runtime enum roots or runtime case classes;
- variant discriminants or payload layout;
- optimized singleton/payload representation;
- runtime `VariantConstructor` or `VariantConstructorFamily` values;
- `VariantInfo` reflection;
- `match` syntax or exhaustiveness;
- final hover, definition, completion, semantic-token, or overload UI;
- removal of all legacy runtime family machinery;
- migration of built-in `Option`, `Some`, or `None`.

Those are deliberately assigned to Parts 2–6.

---

# 7. Normative surface model

## 7.1 Dot is exclusively message sending

The parser must continue to interpret:

```phalcom
object.name
object.print("hello")
System.print("hello")
Math.pi
```

as message sends/property sends. It must not inspect whether the receiver is syntactically a class name and switch to associated declaration lookup.

No variant-specific dot grammar is introduced. Therefore:

A legacy dot-access spelling has only ordinary message-send meaning. It does not name or construct a variant.

## 7.2 Double colon is exclusively associated lookup/invocation syntax

`::` introduces associated syntax. The parser must create associated AST nodes independent of ordinary `MethodCallExpr`/`GetPropertyExpr` semantics.

The front-end syntactic categories are:

1. exact getter-shaped lookup;
2. explicit exact getter alias;
3. whole-family reification;
4. exact named member narrowing;
5. direct associated-family invocation;
6. exact operator member reference;
7. exact subscript member reference.

No parser fallback from associated syntax to dot/message-send syntax is permitted.

---

# 8. Enum declaration grammar

## 8.1 Declaration head

Normative v1 shape:

```phalcom
enum Option<T> {
    ...
}
```

Generic binders and `where` constraints reuse the existing generic declaration syntax used by `ClassDef` and `TypeAliasDef`.

Recommended grammar:

```text
enum-decl
  := 'enum' Identifier generic-parameters? where-clause? '{' enum-member* '}'
```

No source-level superclass clause is accepted in v1. The runtime/semantic superclass relationships of enum roots and exact cases are implicit language semantics, not an `is` clause on the enum declaration.

## 8.2 Explicit variant declaration

Variants require `@variant`:

```phalcom
enum Option<T> {
    @variant Some(_ value: T)
    @variant None
}
```

A bare declaration-looking identifier in an enum body must not be guessed to be a variant. Enum-root behavior already uses member-like syntax, so explicit `@variant` is the unambiguous category marker.

## 8.3 Payload syntax

Payload-bearing variants reuse the ordinary parameter grammar:

```phalcom
@variant Some(_ value: T)

@variant EmailAddress(
    _ username: String,
    host: "gmail" | "outlook"
)
```

This gives the parser existing support for:

- positional slots;
- external labels;
- local binding names;
- type annotations;
- precise per-parameter source ranges.

The selector shape ignores local binding names and type annotations. For example:

```phalcom
@variant Error(_ error: E, reason: String)
```

has exact selector shape:

```text
Error(_,reason)
```

with no colon punctuation in the selector label identity.

## 8.4 Singleton variants

A singleton variant omits payload parentheses:

```phalcom
@variant None
```

Its exact selector is getter-shaped:

```text
#None
```

Part 1 recommends rejecting:

```phalcom
@variant None()
```

with a targeted syntax diagnostic, because that spelling would create an artificial zero-argument constructor shape `#None()` contrary to the ratified singleton model.

This is an explicit assumption pending unavailable attachment verification, but it follows directly from the ratified requirement that singleton variants are getter-shaped and not zero-argument constructors.

## 8.5 Overloaded variant families

The grammar permits multiple variants sharing a base name so long as complete selector shapes differ:

```phalcom
enum Example {
    @variant None
    @variant None(_ value: Int)
}
```

and:

```phalcom
enum Response {
    @variant Error(_ message: String)
    @variant Error(_ code: Int, reason: String)
}
```

Part 1 only preserves the shapes. Duplicate exact selector detection and incompatible family-category collision checks belong to Part 2.

## 8.6 GADT result specialization

Assumed syntax, because the attachment that would verify it is unavailable:

```phalcom
enum Expr<T> {
    @variant Int(_ value: Int) -> Expr<Int>
    @variant Bool(_ value: Bool) -> Expr<Bool>
}
```

Grammar:

```text
variant-decl
  := attributes-with-variant-marker
     Identifier
     variant-payload?
     ('->' type-annotation)?
     variant-body?
```

The AST must preserve the result annotation exactly as a `TypeAnnotation`. It must not try to solve or validate the equality in Part 1.

## 8.7 Variant body

Payload variants and singleton variants may have case-specific behavior:

```phalcom
enum Shape {
    @variant Circle(_ radius: Float) {
        area -> Float {
            3.14159 * radius * radius
        }
    }
}
```

Variant bodies contain behavior members, not nested variants.

## 8.8 Enum-root behavior and shared contracts

Existing `MemberBody::Declaration` is reused for signature-only declarations:

```phalcom
enum Shape {
    area -> Float

    @variant Circle(_ radius: Float) {
        area -> Float { ... }
    }
}
```

A bodyful enum-root method/getter remains ordinary shared/default behavior syntax:

```phalcom
enum Shape {
    describe -> String {
        ...
    }
}
```

Part 1 parses the distinction. Part 2 gives declaration-only root members their explicit closed-enum contract semantics.

## 8.9 Initial enum behavior-member set

Recommended v1 AST member set:

```text
Method
Getter
Setter
Index
```

Payload parameters, not arbitrary field declarations, define per-case stored state in the initial enum model. This avoids conflating class layout fields with algebraic constructor payload.

This is a non-ratified assumption pending attachment verification. The AST should be easy to extend later if enum-root or case fields are explicitly ratified.

## 8.10 Variant rest parameters

The initial grammar should reject rest parameters in variant payloads:

```phalcom
@variant X(*values: Int)       // reject in v1
@variant X(**named: Int)       // reject in v1
@variant X(***pack: Dynamic)   // reject in v1
```

Reason: exact `VariantId` is keyed by a complete selector shape, while rest-family constructor semantics and identity have not been ratified. Ordinary method rest families remain unaffected.

This is a non-ratified assumption and should be revisited only if the unavailable design material explicitly defines rest-bearing data constructors.

---

# 9. Target enum AST

The target front-end should introduce dedicated nodes rather than overloading `ClassDef`.

## 9.1 Proposed Rust structures

**Compile-oriented proposed code:**

```rust
#[derive(Debug, Clone)]
pub struct EnumDef {
    pub name: String,
    pub name_range: SourceRange,
    pub generic_parameters: Vec<GenericParameterSyntax>,
    pub where_clause: Option<WhereClauseSyntax>,
    pub members: Vec<EnumMember>,
    pub attributes: Vec<Attribute>,
    pub range: SourceRange,
}

#[derive(Debug, Clone)]
pub enum EnumMember {
    Variant(VariantDecl),
    Behavior(EnumBehaviorMember),
}

#[derive(Debug, Clone)]
pub enum EnumBehaviorMember {
    Method(MethodDef),
    Getter(GetterDef),
    Setter(SetterDef),
    Index(IndexMethodDef),
}

#[derive(Debug, Clone)]
pub struct VariantDecl {
    pub name: String,
    pub name_range: SourceRange,
    /// Span of the explicit `@variant` marker.
    pub variant_marker_range: SourceRange,
    /// `None` means getter-shaped singleton variant `#name`.
    pub payload: Option<VariantPayloadSyntax>,
    /// GADT result specialization, if written.
    pub result_annotation: Option<TypeAnnotation>,
    /// Case-specific behavior.
    pub body: Option<VariantBody>,
    /// Non-marker attributes preserved for later visibility/metadata semantics.
    pub attributes: Vec<Attribute>,
    pub range: SourceRange,
}

#[derive(Debug, Clone)]
pub struct VariantPayloadSyntax {
    pub parameters: Vec<ParameterDef>,
    pub range: SourceRange,
}

#[derive(Debug, Clone)]
pub struct VariantBody {
    pub members: Vec<EnumBehaviorMember>,
    pub range: SourceRange,
}
```

`Statement` gains:

```rust
Enum(EnumDef),
```

## 9.2 Why not reuse `ClassMember::Variant`

Reusing the legacy node would encode the wrong assumptions into every later part:

- variants would still appear to be class members;
- payloads would be label-only field lists;
- no exact constructor parameter syntax would exist;
- no GADT result annotation would fit naturally;
- no case-specific behavior body would exist;
- the legacy attribute expander would remain a tempting but incorrect lowering route.

A separate `VariantDecl` is therefore required even if `ClassMember::Variant(VariantDef)` remains temporarily for legacy backend compatibility.

## 9.3 Why keep `EnumBehaviorMember` separate from `ClassMember`

Using `ClassMember` directly would also admit `Field` and legacy `Variant` variants that are not part of the initial enum behavior surface. A narrow wrapper makes the grammar and downstream exhaustive matches encode the intended distinction.

The contained `MethodDef`, `GetterDef`, `SetterDef`, and `IndexMethodDef` remain shared, avoiding duplicate method syntax or contract-body representation.

---

# 10. Associated syntax grammar

## 10.1 Exact getter-shaped lookup

```phalcom
owner::name
```

means exact selector:

```text
#name
```

It does **not** mean whole family.

Examples:

```phalcom
Option::None
Math::pi
```

Semantic category resolution comes later. The parser only records exact getter-shaped associated lookup.

## 10.2 Explicit getter alias

```phalcom
owner::name::
```

is the fully explicit residual-selector spelling of the same exact getter selector.

The AST must preserve whether the trailing `::` was written for source tooling, but semantic normalization later must produce the same selector identity as `owner::name`.

## 10.3 Whole family

```phalcom
owner::name::*
```

reifies the whole reserved base-name family.

Examples:

```phalcom
System::print::*
Option::Some::*
Response::Error::*
```

The family syntax is not a selector pattern and must not normalize to `SelectorPattern`.

## 10.4 Direct associated-family invocation

```phalcom
owner::name(arguments)
```

is a distinct syntactic operation: associated-family invocation.

Examples:

```phalcom
Option::Some(42)
Response::Error("failed")
EmailRepresentation::EmailAddress("jsmith", host: "gmail")
```

The parser must not first build an `owner::name` lookup node and then attach an ordinary callable call. The AST should preserve that the programmer wrote the fused associated invocation path.

This is essential because later phases must implement:

```text
resolve reserved family
→ derive incoming argument shape
→ select applicable exact member
→ type/constraint/overload resolution
→ invoke selected target
```

without mandatory family object materialization.

## 10.5 Exact named member narrowing

A second `::` enters residual exact-selector syntax:

```phalcom
owner::name::()
owner::name::(_)
owner::name::(_, reason)
owner::name::=(put)
```

These expressions reify exact members and do not invoke them.

Examples:

```phalcom
const unaryError = Response::Error::(_)
const someConstructor = Option::Some::(_)
const zeroArg = Service::start::()
const setter = Configuration::name::=(put)
```

The canonical immediate call remains the short associated invocation:

```phalcom
Response::Error("failed")
```

If an exact reference is deliberately reified first, ordinary callable application may be used on the completed expression:

```phalcom
(Response::Error::(_))("failed")
```

There is no special syntax such as:

No special incomplete-narrowing invocation spelling is introduced.

## 10.6 Getter versus zero-argument method

The grammar must preserve:

```phalcom
Response::name       // exact #name
Response::name::     // exact #name, explicit alias
Response::name::()   // exact #name()
Response::name()     // associated-family invocation with zero-argument method call shape
```

No parser normalization may collapse these.

## 10.7 Whole-family calls

Because a family expression is first-class, ordinary callable postfix syntax may follow the completed family expression:

```phalcom
Response::Error::*("failed")
```

This is parsed as:

```text
AssociatedLookup(Family)
then ordinary callable invocation on that completed expression
```

It is intentionally different in AST from:

```phalcom
Response::Error("failed")
```

which is direct `AssociatedInvokeExpr`.

## 10.8 Operators

Operator exact references use native operator selector grammar and do not add a second narrowing layer:

```phalcom
Response::+
Response::+(_)
```

Do not require:

```phalcom
Response::+::(_)
```

The parser should continue to use the existing operator-selector arity rules when normalizing the exact selector.

## 10.9 Subscripts

Subscript exact references likewise use their native bracket grammar:

```phalcom
Response::[x]
Response::[x, y]
Response::[x]=(put)
```

No second `::` is inserted around bracket selector syntax.

The names written in positional placeholder positions establish selector arity; local placeholder spelling does not become part of selector identity. Normalization produces the existing `SubscriptGet` / `SubscriptSet` structural selector kind and slots.

## 10.10 Associated ellipsis is retired

These old forms are not canonical associated family syntax:

```phalcom
owner::name...
owner::name(...)
owner::name(_, ..., foo)
```

They must be rejected after `::` with targeted migration guidance to `::*` where the intent is whole-family reification.

The independent `#` selector-pattern language may continue to accept:

```phalcom
#name...
#name(_, ..., foo)
```

for reflection/pattern matching over selectors.

## 10.11 Hash-prefixed `::` selector specs remain invalid

The existing rejection of:

```phalcom
owner::#name
owner::#name(_)
```

should remain. `::` already establishes associated-member context; `#` is for first-class selector values.

---

# 11. Target associated AST

`Expr::MethodRef` is too method-specific and encodes the wrong call semantics. The canonical parser output should become distinct lookup and invocation nodes.

## 11.1 Proposed expression nodes

**Compile-oriented proposed code:**

```rust
pub enum Expr {
    // existing variants ...
    AssociatedLookup(Box<AssociatedLookupExpr>),
    AssociatedInvoke(Box<AssociatedInvokeExpr>),
    // `MethodRef` is removed once all baseline consumers migrate.
}

#[derive(Debug, Clone)]
pub struct AssociatedLookupExpr {
    pub receiver: Expr,
    pub first_separator_range: SourceRange,
    pub member: AssociatedMemberSyntax,
    pub range: SourceRange,
}

#[derive(Debug, Clone)]
pub enum AssociatedMemberSyntax {
    Named(AssociatedNamedMemberSyntax),
    Operator(ExactSelectorSyntax),
    Subscript(ExactSelectorSyntax),
}

#[derive(Debug, Clone)]
pub struct AssociatedNamedMemberSyntax {
    pub base: String,
    pub base_range: SourceRange,
    pub mode: AssociatedNamedMode,
    pub range: SourceRange,
}

#[derive(Debug, Clone)]
pub enum AssociatedNamedMode {
    /// `owner::name` or `owner::name::`.
    Getter {
        explicit_separator_range: Option<SourceRange>,
    },
    /// `owner::name::shape`.
    Exact {
        second_separator_range: SourceRange,
        residual: AssociatedResidualSelectorSyntax,
    },
    /// `owner::name::*`.
    Family {
        second_separator_range: SourceRange,
        star_range: SourceRange,
    },
}

#[derive(Debug, Clone)]
pub enum AssociatedResidualSelectorSyntax {
    Method {
        slots: Vec<SelectorSlotSyntax>,
        range: SourceRange,
    },
    Setter {
        put_range: SourceRange,
        range: SourceRange,
    },
}

#[derive(Debug, Clone)]
pub struct AssociatedInvokeExpr {
    pub receiver: Expr,
    pub first_separator_range: SourceRange,
    pub base: String,
    pub base_range: SourceRange,
    pub args: Vec<PackItem>,
    pub range: SourceRange,
}
```

Exact field names can be adjusted during implementation to fit existing AST conventions, but the semantic distinctions in this structure are normative.

## 11.2 Why direct invocation is a separate node

Representing:

```phalcom
Response::Error("failed")
```

as:

```text
Call(AssociatedLookup(Getter Error), ...)
```

would encode the wrong semantics: `Response::Error` means exact getter `#Error`, not whole family. The direct call is a family operation chosen by syntax, not a speculative attempt to call the result of getter lookup.

A dedicated `AssociatedInvokeExpr` eliminates that ambiguity before typing begins.

## 11.3 Why family mode is not `SelectorPatternSyntax`

A selector pattern such as `#print(_, ..., to)` describes a predicate over exact selectors. A whole family `System::print::*` identifies the reserved base-name family itself.

These are not interchangeable identities. Reusing `SelectorPatternSyntax` for `::*` would preserve the old conceptual error and make Part 2 family reservation harder to model.

## 11.4 Getter alias source fidelity

`owner::name` and `owner::name::` must normalize to the same semantic selector later, but source tools need to know which spelling occurred.

`AssociatedNamedMode::Getter { explicit_separator_range }` records this without allocating two semantic identities.

---

# 12. Parsing strategy

## 12.1 Pratt/postfix boundary

Keep `Parser::parse_call` as the owner of postfix syntax.

The relevant control flow becomes conceptually:

```text
primary expression
  loop:
    '.'   → ordinary send/property parse
    '::'  → parse associated suffix
    '('   → ordinary callable application to completed expression
    '['   → ordinary subscript application
    ...
```

## 12.2 Associated suffix decision table

After consuming the first `::`:

| Following source | Parse result |
|---|---|
| identifier + end/postfix boundary | exact getter `AssociatedLookup` |
| identifier + `(` values `)` | `AssociatedInvoke` |
| identifier + second `::` + end | explicit getter `AssociatedLookup` |
| identifier + second `::` + `*` | whole-family `AssociatedLookup` |
| identifier + second `::` + `(` selector slots `)` | exact member `AssociatedLookup` |
| identifier + second `::` + `=(put)` | exact setter `AssociatedLookup` |
| operator selector | exact operator `AssociatedLookup` |
| subscript selector | exact subscript `AssociatedLookup` |
| `#` | syntax error |
| legacy ellipsis pattern | targeted syntax error |

The distinction between **argument lists** and **selector-shape lists** is syntactic:

- first parentheses after the base belong to direct invocation and parse expressions/pack items;
- parentheses after a second `::` parse selector slots such as `_` and labels, never argument expressions.

This removes the current ambiguity.

## 12.3 Invalid old exact narrowing

When the parser sees:

```phalcom
owner::name(_)
```

`_` cannot be a normal argument expression. Instead of reporting a generic expression error, Part 1 should recognize this high-confidence legacy exact-reference attempt and emit guidance:

```text
exact associated member narrowing uses a second `::`; write `owner::name::(_)`
```

The same principle applies to other selector-only syntax encountered in direct-call position.

## 12.4 Ordinary callable postfix remains reusable

Once an associated lookup expression is complete, existing callable postfix parsing applies:

```phalcom
(Response::Error::(_))("failed")
Response::Error::*("failed")
```

No special “invoke exact reference” syntax is added.

---

# 13. Selector normalization rules

## 13.1 Keep `phalcom-common::selector` canonical

All exact member shapes eventually normalize to the existing `Selector` type.

Examples:

| Source | Structural selector |
|---|---|
| `owner::name` | `Selector { kind: Getter, base: "name", slots: [] }` |
| `owner::name::` | same getter selector |
| `owner::name::()` | `Method`, `name`, `[]` |
| `owner::name::(_)` | `Method`, `name`, `[Positional]` |
| `owner::name::(_, reason)` | `Method`, `name`, `[Positional, Label("reason")]` |
| `owner::name::=(put)` | `Setter`, `name`, `[]` |

## 13.2 Variant selector-shape helper

Part 1 should add a structural helper in `phalcom-ast/src/selector.rs` for new `VariantDecl` values. It does not allocate a `VariantId`; it only derives the exact selector shape later consumed by Part 2.

**Compile-oriented proposed code:**

```rust
pub fn selector_from_variant(variant: &VariantDecl) -> Selector {
    match &variant.payload {
        None => Selector::getter(&variant.name).expect("parsed variant name is valid"),
        Some(payload) => {
            let slots = payload.parameters.iter().map(selector_slot_from_parameter);
            Selector::method(&variant.name, slots.collect::<Vec<_>>())
                .expect("parsed variant selector is valid")
        }
    }
}
```

The helper must not add owner text or colon punctuation to the selector.

## 13.3 Owner qualification happens later

Part 2 may build semantic identities conceptually like:

```text
VariantId {
    owner,
    selector,
}
```

Part 1 therefore must not encode strings such as `"Option::Some(_)"` into `Selector.base`. The owner remains a separate semantic identity.

---

# 14. Source range contract

Every new declaration/expression must preserve enough source location information for diagnostics and future source indexing.

Minimum required ranges:

## Enum declarations

- entire enum declaration;
- enum name;
- each generic parameter through existing nodes;
- each `@variant` marker;
- variant name;
- payload range;
- every payload parameter through `ParameterDef`;
- GADT result annotation through `TypeAnnotation`;
- variant body;
- behavior-member ranges through existing nodes.

## Associated expressions

- receiver range through child `Expr`;
- first `::` range;
- associated base name range;
- second `::` range where present;
- `*` range for whole family;
- residual selector-shape range;
- invocation argument ranges through `PackItem`;
- full associated expression range.

This avoids later reparsing to determine whether a spelling was `::name` or `::name::` and gives source-index builders stable spans.

---

# 15. Syntax diagnostics

Part 1 should prefer stable `SyntaxErrorKind` variants for new high-value migration errors rather than routing everything through `SyntaxErrorKind::Message`.

Recommended additions:

```rust
LegacyVariantDeclaration,
AssociatedLegacyFamilyEllipsis,
AssociatedExactShapeRequiresSecondSeparator,
SingletonVariantHasEmptyParameterList,
VariantRestParameterUnsupported,
```

Representative diagnostics:

```text
syntax.enum.variant_outside_enum
`@variant` declares an enum variant and may only appear inside an `enum` body
```

```text
syntax.associated.legacy_family_ellipsis
associated whole-family lookup uses `::*`; write `owner::name::*`
```

```text
syntax.associated.exact_requires_second_separator
exact associated member narrowing uses a second `::`; write `owner::name::(_)`
```

```text
syntax.enum.singleton_parentheses
singleton variants omit parentheses; write `@variant None`
```

```text
syntax.enum.variant_rest_unsupported
variant payloads must have a finite exact selector shape; rest parameters are not supported in v1
```

Generic malformed grammar should continue to use existing `UnrecognizedToken`, `UnrecognizedEof`, and recovery infrastructure.

---

# 16. Error recovery

`Parser::synchronize` must recognize `enum` as a top-level statement introducer so an error in one declaration does not swallow later declarations.

Inside an enum body, recovery should synchronize on:

- `@` starting an attribute/variant marker;
- the start of a behavior member;
- `}` closing the enum/variant body;
- newline/semicolon boundaries consistent with current class parsing.

Recovery must not synthesize fake variants. A malformed `@variant` should be omitted or represented through the parser's existing invalid-node policy rather than converted into a behavior member.

---

# 17. Legacy architecture handling in Part 1

## 17.1 Legacy `ClassMember::Variant`

The safest staged migration is:

1. introduce new `EnumDef` / `VariantDecl` nodes;
2. stop `Parser::parse_class_body` from producing legacy `ClassMember::Variant` for source `@variant`;
3. reject `@variant` in ordinary class bodies with a targeted migration diagnostic;
4. leave the legacy AST node and compiler attribute-expansion code temporarily present if removing it would create unnecessary backend churn;
5. mark it as legacy/dead-source-compatible in comments;
6. remove the obsolete machinery in Part 6 after built-ins and runtime integration have migrated.

The important invariant is that **new enum source never enters the legacy expansion path**.

## 17.2 Legacy `MethodRefExpr`

Unlike legacy `ClassMember::Variant`, `MethodRefExpr` directly conflicts with the new meaning of a live surface operator. Part 1 should therefore migrate parser consumers to the new associated AST and remove `Expr::MethodRef` / `MethodRefKind` once exhaustive downstream compilation is restored.

`SelectorSpecSyntax` remains because `#` selector values still need it.

## 17.3 Legacy `Bytecode::MakeFamily`

Do not remove or repurpose `MakeFamily` in Part 1. It is a runtime design decision for Part 4.

However, after the parser stops producing `MethodRefExpr`, no new associated syntax may emit `MakeFamily` by accident.

The compiler should have explicit staging errors for new AST forms until Parts 3–4 add semantics/lowering, for example:

```rust
CompilerError::EnumNotLoweredYet(SourceRange)
CompilerError::AssociatedLookupNotLoweredYet(SourceRange)
CompilerError::AssociatedInvokeNotLoweredYet(SourceRange)
```

This is preferable to silently compiling the wrong semantics.

---

# 18. Downstream front-end compatibility

Part 1 necessarily touches consumers that exhaustively match AST variants, but it must not move semantic authority into them.

## 18.1 `phalcom-semantic`

Update exhaustive traversals in the semantic crate to understand that enum/associated nodes exist. Until Part 2/3:

- do not publish variants as methods;
- do not manufacture inferred enum types;
- do not resolve associated lookup through LSP-like heuristics;
- do not treat new associated syntax as ordinary message sends.

Source-index traversal may record raw source occurrences/ranges, but semantic target identity is deferred until the declaration model exists.

Likely baseline consumers include:

- `phalcom-semantic/src/checker/declaration.rs`
- `phalcom-semantic/src/advisory/analyzer.rs`
- `phalcom-semantic/src/source_index/builder.rs`
- `phalcom-semantic/src/source_index/occurrence.rs`
- database fingerprint/traversal code that exhaustively matches `Statement`, `ClassMember`, or `Expr`.

## 18.2 `phalcom-core`

Update AST exhaustive matches so the workspace compiles. The compiler must fail explicitly on enum/associated nodes until their semantic/lowering parts land. It must not route them into:

- class variant expansion;
- `Bytecode::Invoke` as if `::` were dot;
- `Bytecode::MakeFamily` as if associated lookup were a method family.

## 18.3 `phalcom-lsp`

Only structural/exhaustive AST compatibility is required in this six-part core series. Full enum/associated hover, go-to-definition, completion, semantic tokens, and source-index presentation are deferred to the future LSP workstream.

Any Part 1 LSP changes must consume AST/semantic data rather than create a second associated resolver.

---

# 19. Testing requirements

## 19.1 Lexer tests

Add coverage proving:

```phalcom
enum
```

lexes as the dedicated keyword while names such as `enumerate` remain identifiers.

## 19.2 Enum parser tests

At minimum:

```phalcom
enum Option<T> {
    @variant Some(_ value: T)
    @variant None
}
```

Verify:

- `Statement::Enum`;
- enum name/range;
- generic parameter `T`;
- `Some` payload has one positional parameter;
- `None` has `payload: None`;
- no `ClassMember::Variant` appears.

GADT fixture:

```phalcom
enum Expr<T> {
    @variant Int(_ value: Int) -> Expr<Int>
    @variant Bool(_ value: Bool) -> Expr<Bool>
}
```

Overloaded family fixture:

```phalcom
enum Example {
    @variant None
    @variant None(_ value: Int)
}
```

Contract/body fixture:

```phalcom
enum Shape {
    area -> Float

    @variant Circle(_ radius: Float) {
        area -> Float { 0.0 }
    }
}
```

## 19.3 Associated syntax tests

Rewrite `phalcom-ast/tests/family_selector_syntax.rs` around the new grammar.

Positive cases:

```phalcom
receiver::name
receiver::name::
receiver::name::*
receiver::name::()
receiver::name::(_)
receiver::name::(_, reason)
receiver::name::=(put)
receiver::name()
receiver::name(1)
receiver::name(1, reason: "x")
receiver::+
receiver::+(_)
receiver::[x]
receiver::[x, y]
receiver::[x]=(put)
receiver::name::*(1)
```

Negative/migration cases:

```phalcom
receiver::#name
receiver::name...
receiver::name(...)
receiver::name(_)
receiver::name::*(...) // if ellipsis appears as argument, ordinary expression rules apply; no selector-pattern interpretation
```

## 19.4 Selector normalization tests

Verify colon-free shapes:

```text
Some(_)
Error(_,reason)
EmailAddress(_,host)
```

and distinct getter/method shapes:

```text
None
None()
```

The variant grammar should never create `None()` for a singleton declaration.

## 19.5 Dot non-regression

Add tests proving:

A legacy dot-access variant-like spelling and an ordinary getter send such as `Math.pi`

still parse as ordinary dot/send forms, not associated lookup.

The test is about syntax category, not whether those sends succeed at runtime.

## 19.6 Staging tests

Until later parts land, compiler tests should prove that parsed enum/associated constructs fail with the explicit “not lowered yet” error rather than:

- panicking;
- producing `MakeFamily`;
- falling through to dNU;
- expanding enum variants as sealed-class variants.

---

# 20. Migration and compatibility

This is an intentionally breaking syntax migration.

## 20.1 Associated syntax

| Old/current source | New source |
|---|---|
| `receiver::name()` exact ref | `receiver::name::()` |
| `receiver::name(_)` exact ref | `receiver::name::(_)` |
| `receiver::name(_, label)` exact ref | `receiver::name::(_, label)` |
| `receiver::name...` family | `receiver::name::*` |
| selector pattern after `::` | keep pattern as `#...` reflection value, not associated family syntax |

## 20.2 Variant syntax

The legacy sealed-class arm:

```phalcom
@sealed
class Shape {
    @variant Circle(radius:)
}
```

is not silently translated to the new enum model. New source is explicit:

```phalcom
enum Shape {
    @variant Circle(_ radius: Float)
}
```

The parser should diagnose the old class-body form and point at `enum` migration rather than generating a sibling class behind the user's back.

## 20.3 Documentation naming

All new tests, comments, diagnostics, and specs use canonical variant naming:

```text
Option::None
Option::Some(_)
Response::Error(_, reason)
```

Never use dot-form variant names.

---

# 21. Cross-part dependencies

## Part 2 consumes Part 1

Part 2 will assign semantic identities to:

- `EnumDef`;
- `VariantDecl`;
- variant base family;
- exact selector shape;
- exact case type;
- shared enum contracts.

Part 1 must therefore preserve complete selector shape and declaration ranges.

## Part 3 consumes Part 1 associated AST

`AssociatedLookupExpr` and `AssociatedInvokeExpr` become the syntax-level inputs to exact associated lookup, family lookup, overload resolution, constructor typing, and first-class member typing.

The separate direct-invocation node is required so Part 3 does not infer syntax intent from a generic callable call.

## Part 4 consumes the semantic distinction

Part 4 may generalize or replace `MakeFamily`, add lightweight exact-member/family handles, and optimize direct calls. None of those runtime representation choices leak into Part 1 AST identity.

## Part 5 consumes enum declarations and existing patterns

Part 5 introduces/extends match patterns using `VariantId`. It must not need to redesign enum declaration syntax.

## Part 6 removes legacy machinery

Part 6 completes built-in migration, reflection, old test cleanup, legacy `@variant` attribute-expander removal, obsolete family syntax cleanup, and final documentation migration.

---

# 22. Risks

## 22.1 Parser precedence regressions

`::` lives in the postfix loop next to dot, call, and subscript parsing. A careless rewrite can change precedence or evaluation grouping for completed expressions.

**Mitigation:** snapshot AST shape for nested combinations and keep ordinary call postfix logic unchanged after an associated expression is completed.

## 22.2 Accidental semantic compatibility with wrong runtime

The existing `MakeFamily` path is tempting because it already produces callable family objects.

**Mitigation:** new associated AST nodes have no lowering to `MakeFamily` in Part 1. Explicit staging errors are required.

## 22.3 Duplicate selector representations

Creating an enum-specific selector string format would fork `phalcom-common::selector`.

**Mitigation:** derive variant/member shapes through the existing `Selector` and `SelectorSlot` types; owner stays separate.

## 22.4 Legacy `ClassMember::Variant` leaking into new semantics

Keeping the old node temporarily could cause a later agent to reuse it.

**Mitigation:** comments and tests explicitly identify it as legacy; parser no longer produces it for source variants; new enum AST uses `VariantDecl` only.

## 22.5 Source-index churn

Removing `MethodRefExpr` changes source traversal before Part 3 can resolve associated targets.

**Mitigation:** preserve exact source ranges and update traversal to record unresolved associated occurrences without inventing target identity.

## 22.6 Attachment-dependent syntax assumptions

GADT result placement, visibility-axis syntax, and enum-body field policy could be clarified by unavailable design documents.

**Mitigation:** assumptions are isolated below and are not allowed to alter ratified `.`/`::`/family rules.

---

# 23. Open questions and explicit assumptions

These are non-blocking for Part 1 unless later attachment evidence contradicts them.

## Q1. Exact GADT result spelling

**Question:** Is `@variant Int(_ value: Int) -> Expr<Int>` the final syntax?

**Recommendation/assumption:** yes. Reuse existing `-> TypeAnnotation` parsing immediately after the variant head and before an optional body.

**Consequence:** Part 2 can interpret it as a declared constructor result/equality without another parser change.

## Q2. Empty payload parentheses

**Question:** Should `@variant None()` be accepted as an alias for singleton `@variant None`?

**Recommendation/assumption:** no. Reject it. Source syntax should preserve the semantic distinction between getter-shaped singleton `#None` and zero-argument method shape `#None()`.

## Q3. Rest-bearing variant constructors

**Question:** May variant constructors declare `*`, `**`, or `***` rest parameters?

**Recommendation/assumption:** not in v1. Reject them until exact `VariantId`, exhaustiveness, and constructor-family semantics for unbounded shapes are deliberately designed.

## Q4. Arbitrary fields in enum/case bodies

**Question:** May enum roots or exact cases declare class-style fields independently of variant payloads?

**Recommendation/assumption:** no in v1. Payload parameters define case data; enum/case bodies contain behavior. This keeps algebraic state explicit and avoids a second layout model before Part 4.

## Q5. Visibility-axis source syntax

**Question:** What exact syntax independently controls naming/matching, construction, and payload projection visibility?

**Recommendation:** do not invent it in Part 1. Preserve non-`@variant` attributes on `VariantDecl` plus precise ranges so Part 2 can model multi-axis metadata once syntax is ratified.

## Q6. Exact case type source syntax

**Question:** What user-visible type syntax denotes the exact case type for `Option::Some(_)`?

**Recommendation:** leave unresolved in Part 1. Do not resurrect dot-associated type syntax. Part 2 can implement exact case types semantically without requiring public syntax immediately.

## Q7. Initial `match` syntax

**Question:** What exact syntax will introduce enum elimination?

**Recommendation:** defer to Part 5. The current repository has no dedicated `match` AST, and Part 1 does not need one to land correct declarations/associated syntax.

---

# 24. Acceptance criteria

Part 1 is accepted when all of the following observable conditions hold against the implementation branch based on `1892bcff...`:

- [ ] `enum` is a lexer keyword and top-level declaration starter.
- [ ] `Statement::Enum(EnumDef)` exists.
- [ ] `@variant Some(_ value: T)` parses to a typed payload `VariantDecl`.
- [ ] `@variant None` parses as a singleton with no synthetic empty argument list.
- [ ] GADT result annotations are preserved in the AST under the stated assumption.
- [ ] enum-root declaration-only methods/getters remain representable as `MemberBody::Declaration` contracts.
- [ ] variant bodies preserve case-specific behavior members.
- [ ] ordinary class-body `@variant` no longer enters the new enum model and receives a migration diagnostic.
- [ ] `.` syntax remains exclusively ordinary message-send/property AST.
- [ ] `owner::name` parses as exact getter-shaped associated lookup.
- [ ] `owner::name::` parses as the same exact getter identity with explicit spelling retained.
- [ ] `owner::name::*` parses as whole-family reification.
- [ ] `owner::name(args)` parses as a dedicated direct associated invocation.
- [ ] `owner::name::shape` parses as exact member reification.
- [ ] getter `#name` and zero-argument method `#name()` remain distinct.
- [ ] operator and subscript exact references use their native selector grammar without redundant second narrowing.
- [ ] associated `...` family syntax is rejected; `#` selector patterns remain available.
- [ ] selector labels normalize without colon punctuation.
- [ ] new associated syntax cannot silently emit `Bytecode::MakeFamily` in Part 1.
- [ ] all changed AST consumers compile without introducing a second semantic resolver.
- [ ] focused `phalcom-ast` tests pass.
- [ ] `cargo fmt --check` passes.
- [ ] the workspace test suite is run and any unrelated pre-existing failures are recorded rather than hidden.

---

# 25. Architectural invariant carried forward

The most important output of Part 1 is not merely a parser feature. It is a clean boundary:

```text
`.` source
  → message-send AST
  → behavioral semantics

`::` source
  → associated lookup/invocation AST
  → declaration/family semantics (Parts 2–3)

`#selector` source
  → first-class selector / selector-pattern value
  → reflection/selector semantics
```

And for enums:

```text
enum declaration
  → EnumDef
  → VariantDecl + behavior declarations
  → semantic identities (Part 2)
  → associated constructor resolution (Part 3)
  → efficient runtime representation (Part 4)
  → elimination/match (Part 5)
  → core migration/reflection/cleanup (Part 6)
```

No later implementation part should need to reinterpret Part 1 syntax to recover the programmer's intent.
