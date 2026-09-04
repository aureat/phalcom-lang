# Phalcom ADT/GADT + Associated Lookup — Implementation Plan 01

## Surface Syntax, Parser, AST, and Selector/Family Grammar

> **For agentic workers:** REQUIRED SUB-SKILL: use `superpowers:subagent-driven-development` (recommended) or `superpowers:executing-plans` to execute this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Land the ratified `enum`/`@variant` front-end and the new `::` associated lookup/family grammar without implementing semantic resolution or routing the syntax through the legacy method-family runtime.

**Architecture:** Extend the existing hand-written `phalcom-ast` lexer/parser and shared structural selector model. Add dedicated enum/variant and associated lookup/invocation AST nodes, preserve ordinary dot sends unchanged, and update downstream exhaustive AST consumers only enough to compile safely. New associated forms intentionally receive explicit compiler staging errors until Parts 2–4 provide semantic resolution and lowering.

**Tech stack:** Rust workspace, `phalcom-ast`, `phalcom-common`, `phalcom-semantic`, `phalcom-core`, `phalcom-lsp`, existing parser snapshot/unit tests.

**Spec:** `docs/impl/adt-gadt-associated-lookup/part-1/01-surface-syntax-parser-ast-selector-family-grammar-technical-spec.md`

**Baseline:** `aureat/phalcom-lang` `main` at `1892bcff51f75dd2f3df2a0661b03371250d4090`.

**Proposed repository path:** `docs/impl/adt-gadt-associated-lookup/part-1/01-surface-syntax-parser-ast-selector-family-grammar-implementation-plan.md`

---

# 1. Global constraints

Every task in this plan inherits these constraints:

1. `.` remains exclusively message sending. Do not add class-object exceptions.
2. `::` becomes associated lookup/invocation syntax. Do not lower it through ordinary dNU/message dispatch.
3. Variants use canonical names such as `Option::Some(_)` and `Option::None`; never dot-form names.
4. Selector labels never contain `:` in identity strings.
5. Whole family syntax is `::*`; do not preserve associated `...` as an alias.
6. `owner::name` means exact getter-shaped member `#name`, not whole family.
7. `owner::name(args)` is a dedicated associated-family invocation node.
8. Exact named member narrowing uses a second `::`: `owner::name::shape`.
9. Operator/subscript exact references use their native selector grammar without redundant second narrowing.
10. Singleton variants are getter-shaped and are not rewritten into zero-argument constructors.
11. New variants are not ordinary methods and must not be published as `ClassMember::Method` or routed through class-side method dispatch.
12. `phalcom-semantic` remains the sole static semantic authority. Part 1 does not add semantic resolution to `phalcom-lsp`.
13. Existing `phalcom-common::selector::{Selector, SelectorKind, SelectorSlot, SelectorPattern}` remains the canonical structural selector model unless a concrete failing test proves an incompatibility.
14. Do not remove `Bytecode::MakeFamily` or redesign runtime family representation in Part 1; make it unreachable from the new parser syntax instead.
15. Do not implement `match` in this part.
16. Do not mutate built-in `Option`/`Some`/`None` integration in this part.
17. No GitHub mutation/commit is part of this planning deliverable. An executing agent may use the repository's normal branch/commit workflow separately.

---

# 2. File map

## Files that definitely change

```text
phalcom-ast/src/token.rs
phalcom-ast/src/lexer.rs
phalcom-ast/src/ast.rs
phalcom-ast/src/parser.rs
phalcom-ast/src/selector.rs
phalcom-ast/src/error.rs
phalcom-ast/tests/lexer.rs
phalcom-ast/tests/parser.rs
phalcom-ast/tests/family_selector_syntax.rs
```

## Files likely to change because they exhaustively consume the AST

```text
phalcom-core/src/compiler/lib/mod.rs
phalcom-core/src/compiler/lib/expr.rs
phalcom-core/src/compiler/lib/error.rs
phalcom-semantic/src/checker/declaration.rs
phalcom-semantic/src/advisory/analyzer.rs
phalcom-semantic/src/source_index/builder.rs
phalcom-semantic/src/source_index/occurrence.rs
phalcom-semantic/src/db/fingerprint.rs
phalcom-lsp/src/selectors.rs
phalcom-lsp/src/hover.rs
phalcom-lsp/src/semantic_tokens.rs
```

The executing agent must let Rust exhaustiveness errors identify additional consumers rather than guessing and preemptively restructuring unrelated modules.

## New focused test file

```text
phalcom-ast/tests/enum_syntax.rs
```

## Documentation updated in this part

```text
docs/spec/selectors.md
```

The full documentation migration is still Part 6; this edit is limited to preventing the active selector spec from contradicting the parser after Part 1.

---

# 3. Task ordering

The work should land in this dependency order:

```text
Token/lexer
  ↓
Enum AST
  ↓
Enum parser
  ↓
Associated AST
  ↓
Associated parser grammar
  ↓
Selector normalization helpers
  ↓
Diagnostics/migration errors
  ↓
Downstream AST consumers
  ↓
Tests + active selector docs
  ↓
Workspace verification
```

Do not start by changing `phalcom-core` runtime family code. The front-end contract must be explicit first.

---

# 4. Task 1 — Add the `enum` keyword at the lexical boundary

**Files**

- Modify: `phalcom-ast/src/token.rs`
- Modify: `phalcom-ast/src/lexer.rs`
- Test: `phalcom-ast/tests/lexer.rs`

**Existing symbols**

- `Token`
- `Lexer::scan_identifier`
- `Lexer::scan_token`

**Produces**

```rust
Token::Enum
```

- [ ] **Step 1: Add a failing lexer test**

Add tests equivalent to:

```rust
#[test]
fn enum_is_a_keyword_but_enum_prefixed_names_are_identifiers() {
    assert_eq!(tokens("enum Shape {}"), vec![
        Token::Enum,
        Token::Identifier("Shape".into()),
        Token::LBrace,
        Token::RBrace,
    ]);

    assert_eq!(tokens("enumerate"), vec![
        Token::Identifier("enumerate".into()),
    ]);
}
```

Adapt to the existing `phalcom-ast/tests/lexer.rs` helper names rather than introducing a second test harness.

- [ ] **Step 2: Run the focused lexer test and confirm failure**

```bash
cargo test -p phalcom-ast --test lexer enum_is_a_keyword_but_enum_prefixed_names_are_identifiers
```

Expected failure before implementation: `enum` is tokenized as `Identifier("enum")` or `Token::Enum` does not compile.

- [ ] **Step 3: Extend `Token`**

In `phalcom-ast/src/token.rs`, add `Enum` next to declaration keywords such as `Class`/`TypeKw`. Update `Display`/token-text/exhaustive token helpers in the same file if compilation identifies them.

- [ ] **Step 4: Extend `Lexer::scan_identifier`**

Add exactly:

```rust
"enum" => Token::Enum,
```

beside the existing `"class" => Token::Class` branch.

Do not create context-sensitive lexing for `@variant`; attributes remain parsed through the existing `@` + identifier mechanism.

- [ ] **Step 5: Re-run lexer tests**

```bash
cargo test -p phalcom-ast --test lexer
```

**Acceptance criteria**

- `enum` is never an ordinary identifier token.
- identifier prefixes such as `enumerate` are unaffected.
- no selector punctuation token changes are required for `::*`; `ColonColon` and `Asterisk` already exist.

---

# 5. Task 2 — Introduce dedicated enum/variant AST nodes

**Files**

- Modify: `phalcom-ast/src/ast.rs`
- Test: create `phalcom-ast/tests/enum_syntax.rs` initially as compile-shape scaffolding

**Existing symbols**

- `Statement`
- `ClassDef`
- `ClassMember`
- legacy `VariantDef`
- `MethodDef`
- `GetterDef`
- `SetterDef`
- `IndexMethodDef`
- `MemberBody`
- `ParameterDef`
- `TypeAnnotation`
- `Attribute`

**Produces**

```rust
Statement::Enum(EnumDef)
EnumDef
EnumMember
EnumBehaviorMember
VariantDecl
VariantPayloadSyntax
VariantBody
```

- [ ] **Step 1: Add the new `Statement` variant**

Add:

```rust
Enum(EnumDef),
```

beside `Statement::Class`.

Update `Statement` range helpers if one exists; let compiler exhaustiveness identify every downstream consumer later rather than adding wildcard arms.

- [ ] **Step 2: Add `EnumDef`**

Use this compile-oriented shape:

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
```

Do **not** add a `superclass` field. Runtime enum-root inheritance is a semantic/runtime concern, not a source `is` clause in v1.

- [ ] **Step 3: Add a narrow behavior-member wrapper**

```rust
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
```

Add helper methods mirroring the useful parts of `ClassMember`:

```rust
impl EnumBehaviorMember {
    pub fn range(&self) -> SourceRange { ... }
    pub fn name_range(&self) -> SourceRange { ... }
    pub fn attributes(&self) -> &[Attribute] { ... }
}
```

Do not include `Field` or legacy `Variant` in this enum.

- [ ] **Step 4: Add `VariantDecl` and payload/body syntax**

```rust
#[derive(Debug, Clone)]
pub struct VariantDecl {
    pub name: String,
    pub name_range: SourceRange,
    pub variant_marker_range: SourceRange,
    pub payload: Option<VariantPayloadSyntax>,
    pub result_annotation: Option<TypeAnnotation>,
    pub body: Option<VariantBody>,
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

`payload: None` is the syntactic invariant for a singleton variant. Never encode singleton-ness as `Some(parameters = [])`.

- [ ] **Step 5: Keep legacy `VariantDef` clearly quarantined**

Do not delete `ClassMember::Variant(VariantDef)` yet if doing so creates compiler-attribute churn. Change its documentation to say it is legacy sealed-class expansion state and **not** the enum variant representation.

The new parser added in Task 3 must never create it.

**Acceptance criteria**

- New enum source has a dedicated AST home.
- `VariantDecl` can represent singleton, payload, GADT result annotation, and case body without synthetic class nodes.
- Existing method/getter/setter/index AST types are reused for behavior/contracts.
- No runtime or semantic ID types are added to `phalcom-ast`.

---

# 6. Task 3 — Parse top-level `enum` declarations and typed `@variant` declarations

**Files**

- Modify: `phalcom-ast/src/parser.rs`
- Modify: `phalcom-ast/src/error.rs`
- Test: `phalcom-ast/tests/enum_syntax.rs`
- Test: `phalcom-ast/tests/parser.rs`

**Existing symbols**

- `Parser::parse_top_item`
- `Parser::parse_class`
- `Parser::parse_class_body`
- legacy `Parser::parse_variant_decl`
- `Parser::parse_class_member`
- generic parameter parser used by `parse_class`
- type annotation parser
- ordinary parameter-list parser
- `Parser::synchronize`

**Interfaces produced**

```rust
fn parse_enum(&mut self) -> ParserResult<Statement>
fn parse_enum_body(&mut self) -> ParserResult<Vec<EnumMember>>
fn parse_enum_variant(&mut self, attributes: Vec<Attribute>) -> ParserResult<VariantDecl>
fn parse_enum_behavior_member(&mut self, attributes: Vec<Attribute>) -> ParserResult<EnumBehaviorMember>
```

Exact signatures may include current parser bookkeeping arguments; keep the names/responsibilities stable.

- [ ] **Step 1: Add a failing minimal enum parse test**

```rust
#[test]
fn parses_generic_enum_with_payload_and_singleton_variants() {
    let program = parse_source(r#"
        enum Option<T> {
            @variant Some(_ value: T)
            @variant None
        }
    "#, 0).expect("enum parses");

    let Statement::Enum(option) = &program.statements[0] else {
        panic!("expected enum declaration");
    };
    assert_eq!(option.name, "Option");
    assert_eq!(option.generic_parameters.len(), 1);

    let EnumMember::Variant(some) = &option.members[0] else { panic!() };
    assert_eq!(some.name, "Some");
    assert_eq!(some.payload.as_ref().unwrap().parameters.len(), 1);

    let EnumMember::Variant(none) = &option.members[1] else { panic!() };
    assert_eq!(none.name, "None");
    assert!(none.payload.is_none());
}
```

- [ ] **Step 2: Add `Token::Enum` handling in `parse_top_item` and synchronization**

Mirror the top-level `Class` route:

```rust
Token::Enum => self.parse_enum(),
```

Add `Token::Enum` to the statement/declaration starters used by `Parser::synchronize`.

- [ ] **Step 3: Implement `parse_enum` by reusing class generic/where helpers**

Pseudo-structure:

```rust
fn parse_enum(&mut self) -> ParserResult<Statement> {
    let start = self.cur_start();
    self.expect(Token::Enum, &["enum"])?;
    let (name, name_range) = self.expect_identifier()?;
    let generic_parameters = self.parse_optional_generic_parameters(GenericBinderContext::NominalDeclaration)?;
    let where_clause = self.parse_optional_where_clause()?;
    self.expect(Token::LBrace, &["{"])?;
    let members = self.parse_enum_body()?;
    self.expect(Token::RBrace, &["}"])?;

    Ok(Statement::Enum(EnumDef {
        name,
        name_range,
        generic_parameters,
        where_clause,
        members,
        attributes: Vec::new(),
        range: (start..self.prev_end).into(),
    }))
}
```

Use the actual existing helper names exposed by `parse_class`; do not duplicate generic parsing.

- [ ] **Step 4: Implement enum-body attribute collection**

Reuse the same attribute collection machinery used by `parse_class_body`, but route declarations based on whether the pending attribute list contains `BuiltinAttr::Variant`.

When `@variant` is present:

1. extract the marker and its range;
2. reject duplicate `@variant` markers;
3. preserve other attributes in declaration order;
4. call `parse_enum_variant`.

When it is absent, call `parse_enum_behavior_member`.

- [ ] **Step 5: Implement variant head parsing with existing parameter grammar**

For:

```phalcom
@variant Some(_ value: T)
```

reuse the ordinary declaration parameter parser. Do not use legacy `parse_variant_decl`, because that parser consumes colon-bearing bare field labels.

Record the parentheses range in `VariantPayloadSyntax`.

After parsing parameters, reject any `ParameterDef` whose `rest_mode != RestMode::None` with the structured v1 diagnostic.

- [ ] **Step 6: Preserve singleton shape without empty payload lists**

If no `(` follows the variant name, set:

```rust
payload: None
```

If `()` follows immediately, emit `SingletonVariantHasEmptyParameterList` rather than creating `Some(VariantPayloadSyntax { parameters: vec![] })`.

- [ ] **Step 7: Parse optional GADT result annotation**

Under the explicit Part 1 assumption, if `->` follows the variant head:

```rust
let result_annotation = if self.matches(Token::Arrow) {
    Some(self.parse_type_annotation()?)
} else {
    None
};
```

Use the parser's actual arrow token/helper names.

- [ ] **Step 8: Parse optional variant behavior body**

If `{` follows, parse only `EnumBehaviorMember` entries. Reject nested `@variant` inside the case body with a targeted syntax error.

- [ ] **Step 9: Reuse `MemberBody::Declaration` for root contracts**

Verify a fixture like:

```phalcom
enum Shape {
    area -> Float
    @variant Circle(_ radius: Float) {
        area -> Float { 0.0 }
    }
}
```

produces a declaration-only root behavior and a bodyful case behavior.

- [ ] **Step 10: Reject `@variant` in ordinary class bodies**

Replace the current `parse_class_body` route to legacy `parse_variant_decl` with a structured migration error.

Do not silently create the new enum node from a class body.

**Verification**

```bash
cargo test -p phalcom-ast --test enum_syntax
cargo test -p phalcom-ast --test parser
```

**Acceptance criteria**

- No new enum variant goes through `VariantDef`.
- Selector-relevant payload information is stored as `ParameterDef`.
- GADT result syntax is syntactically preserved but not type-checked.
- signature-only enum behavior survives as declaration-only AST.

---

# 7. Task 4 — Replace canonical `MethodRefExpr` parsing with associated lookup/invocation AST

**Files**

- Modify: `phalcom-ast/src/ast.rs`
- Modify: `phalcom-ast/src/parser.rs`
- Test: `phalcom-ast/tests/family_selector_syntax.rs`

**Existing symbols**

- `Expr::MethodRef`
- `MethodRefExpr`
- `MethodRefKind`
- `SelectorSpecSyntax`
- `Parser::parse_call`
- `Parser::parse_selector_spec_after_colon_colon`
- `legacy_method_ref_kind`

**Produces**

```rust
Expr::AssociatedLookup
Expr::AssociatedInvoke
AssociatedLookupExpr
AssociatedInvokeExpr
AssociatedMemberSyntax
AssociatedNamedMemberSyntax
AssociatedNamedMode
AssociatedResidualSelectorSyntax
```

- [ ] **Step 1: Add AST nodes before changing parser behavior**

Add the structures from Technical Specification 01. Include explicit source ranges for both `::` separators and `*` where relevant.

Add range dispatch in `Expr::range()` or equivalent exhaustive match.

- [ ] **Step 2: Write AST-shape tests for the three named base modes**

Add tests proving:

```phalcom
receiver::name
receiver::name::*
receiver::name(1)
```

produce three different AST modes:

```text
AssociatedLookup(Getter)
AssociatedLookup(Family)
AssociatedInvoke
```

This test must fail before parser migration.

- [ ] **Step 3: Introduce an associated-suffix parser entry point**

Replace the `::` branch in `Parser::parse_call` with a call conceptually like:

```rust
expr = self.parse_associated_suffix(expr, start)?;
```

The new helper owns `::` grammar and must not call `parse_selector_spec_after_colon_colon` as its top-level interpretation.

- [ ] **Step 4: Remove `legacy_method_ref_kind` from the parser path**

No new parser output may construct `MethodRefKind::Open`/`Pinned`.

- [ ] **Step 5: Keep `SelectorSpecSyntax` for `#` literals**

Do not delete the selector-symbol/pattern AST just because `MethodRefExpr` no longer uses it.

- [ ] **Step 6: Remove `Expr::MethodRef` / `MethodRefExpr` / `MethodRefKind` after consumers are migrated in Task 8**

If the branch cannot compile until downstream edits are made, keep it temporarily during the branch but remove it before Part 1 is declared complete.

**Acceptance criteria**

- A parser-produced `::` expression is never a `MethodRefExpr`.
- Direct associated invocation is represented explicitly.
- First-class `#selector` and `#selector...` syntax remains intact.

---

# 8. Task 5 — Implement the complete ratified named `::` grammar

**Files**

- Modify: `phalcom-ast/src/parser.rs`
- Rewrite: `phalcom-ast/tests/family_selector_syntax.rs`

**Existing symbols**

- `Parser::parse_call`
- selector-slot parsing helpers used by `parse_selector_spec_body`
- `parse_arg_list` / pack-item parsing used by ordinary calls
- `SelectorSlotSyntax`

**Interfaces produced**

A category-neutral parser for:

```text
owner::name
owner::name::
owner::name::*
owner::name::()
owner::name::(_)
owner::name::(_, label)
owner::name::=(put)
owner::name(args)
```

- [ ] **Step 1: Rewrite the existing exact-form test matrix**

Replace the old test that treats `receiver::name()` and `receiver::name(_)` as exact references with the new truth table.

Representative assertions:

```rust
#[test]
fn bare_associated_name_is_exact_getter() { ... }

#[test]
fn explicit_trailing_separator_is_same_getter_spelling() { ... }

#[test]
fn first_parentheses_after_name_are_direct_invocation() { ... }

#[test]
fn second_separator_parentheses_are_exact_method_shape() { ... }

#[test]
fn star_after_second_separator_reifies_whole_family() { ... }
```

- [ ] **Step 2: Parse `owner::name` as getter mode**

After the first separator and base identifier, if no direct call or second separator follows, construct:

```rust
AssociatedNamedMode::Getter {
    explicit_separator_range: None,
}
```

- [ ] **Step 3: Parse `owner::name::` as explicit getter alias**

If a second `::` is followed by a postfix boundary rather than `*`, `(`, or setter syntax, construct the same getter mode with:

```rust
explicit_separator_range: Some(second_separator_range)
```

Do not encode a different selector kind.

- [ ] **Step 4: Parse `owner::name::*` as family mode**

Consume exactly one `*` after the second separator and store both separator/star ranges.

Do not feed the result into `SelectorPatternSyntax`.

- [ ] **Step 5: Parse residual exact method shapes after the second separator**

Examples:

```phalcom
owner::name::()
owner::name::(_)
owner::name::(_, reason)
```

Reuse selector-slot syntax, not expression arguments. The slots normalize to `SelectorSlot::Positional` and `SelectorSlot::Label`.

- [ ] **Step 6: Parse exact setter**

Support:

```phalcom
owner::name::=(put)
```

as `AssociatedResidualSelectorSyntax::Setter`.

Reject variants of the source that attempt to rename the fixed `put` role unless current setter selector grammar explicitly permits it.

- [ ] **Step 7: Parse first-parenthesis named syntax as `AssociatedInvokeExpr`**

For:

```phalcom
owner::name()
owner::name(1)
owner::name(1, reason: "failed")
```

use the existing call argument/pack parser so computed labels/spreads have exactly the same syntax as ordinary calls.

Do not parse `_` as a selector slot in this position.

- [ ] **Step 8: Add targeted migration detection for `owner::name(_)`**

When the direct-call parser encounters `_` as the first token where an expression is required, produce `AssociatedExactShapeRequiresSecondSeparator` with guidance to `owner::name::(_)`.

Do not globally change underscore expression rules.

- [ ] **Step 9: Reject associated ellipsis forms**

Explicitly reject:

```phalcom
owner::name...
owner::name(...)
owner::name(_, ..., label)
```

with `AssociatedLegacyFamilyEllipsis` or the exact-shape migration error as appropriate.

- [ ] **Step 10: Preserve ordinary callable postfix after completed associated expressions**

Test:

```phalcom
Response::Error::*("failed")
(Response::Error::(_))("failed")
```

The first should parse as a completed family lookup followed by ordinary callable application. The second should parse as a grouped exact reference followed by ordinary callable application.

There is no parser special case that fuses an incompletely narrowed exact member with invocation.

**Verification**

```bash
cargo test -p phalcom-ast --test family_selector_syntax
```

---

# 9. Task 6 — Integrate exact operator and subscript associated references

**Files**

- Modify: `phalcom-ast/src/parser.rs`
- Modify: `phalcom-ast/src/ast.rs` if a dedicated source syntax wrapper is needed
- Test: `phalcom-ast/tests/family_selector_syntax.rs`

**Existing reusable architecture**

- operator token/parser tables already used by binary/unary expressions and current selector-reference parsing;
- `SelectorKind::Method` for operators;
- `SelectorKind::SubscriptGet` / `SubscriptSet`;
- `IndexMethodDef` parameter parsing;
- structural `SelectorBase::Subscript`.

- [ ] **Step 1: Add failing operator reference tests**

```rust
#[test]
fn associated_operator_refs_use_native_exact_selector_syntax() {
    let unary = parse_one("Response::+");
    let binary = parse_one("Response::+(_)");
    // assert exact associated member syntax, not AssociatedInvoke
}
```

Normalize according to the repository's existing operator-selector arity convention.

- [ ] **Step 2: Ensure a second `::` is not required for operators**

Reject or avoid generating grammar for:

```phalcom
Response::+::(_)
```

unless it is independently meaningful under another ratified rule. Part 1 must not require it.

- [ ] **Step 3: Add failing subscript reference tests**

```phalcom
Response::[x]
Response::[x, y]
Response::[x]=(put)
```

Assert structural normalization to subscript getter/setter selector kinds with one/two positional slots.

- [ ] **Step 4: Reuse subscript declaration-shape parsing rather than call-expression parsing**

Inside associated exact-reference context, bracket contents describe selector shape, not runtime index values. Normalize local placeholder names away from identity.

- [ ] **Step 5: Preserve ordinary runtime subscript syntax outside `::`**

Regression-test that:

```phalcom
array[x]
array[x] = value
```

still produce ordinary `Index`/`SetIndex` expressions.

**Acceptance criteria**

- Native operator/subscript selector syntax works under associated lookup.
- No redundant narrowing layer is introduced.
- Existing runtime operator/index syntax is unchanged.

---

# 10. Task 7 — Add structural selector derivation for new variant declarations

**Files**

- Modify: `phalcom-ast/src/selector.rs`
- Test: `phalcom-ast/tests/enum_syntax.rs`
- Test: `phalcom-common` selector tests only if a common-selector bug is discovered

**Existing symbols**

- `selector_from_method`
- `selector_from_getter`
- `selector_from_setter`
- `selector_from_index`
- `selector_from_member`
- `Selector`
- `SelectorSlot`
- `ParameterDef`

**Produces**

```rust
pub fn selector_from_variant(variant: &VariantDecl) -> Selector
```

and, if useful for de-duplication:

```rust
fn selector_slot_from_parameter(param: &ParameterDef) -> SelectorSlot
```

- [ ] **Step 1: Extract/reuse parameter-to-slot conversion**

Both method and variant declarations map parameters using the same rule:

```rust
match (&param.rest_mode, &param.label) {
    (RestMode::None, Some(label)) => SelectorSlot::Label(label.clone()),
    _ => SelectorSlot::Positional,
}
```

Because variant rest is rejected by the parser, variant callers only see `RestMode::None`.

- [ ] **Step 2: Implement singleton derivation**

```rust
None => Selector::getter(&variant.name)
```

For `@variant None`, assert:

```text
kind == Getter
encode() == "None"
```

- [ ] **Step 3: Implement payload derivation**

For `@variant Error(_ error: E, reason: String)`, assert:

```text
kind == Method
encode() == "Error(_,reason)"
```

- [ ] **Step 4: Prove same-base distinct exact variants remain distinct**

Parse:

```phalcom
enum Example {
    @variant None
    @variant None(_ value: Int)
}
```

and assert selectors:

```text
None
None(_)
```

The family identity itself is Part 2 and should not be created in this helper.

- [ ] **Step 5: Stop treating the new variant as a class member selector**

Do not extend `selector_from_member` to accept `VariantDecl`; it only accepts `ClassMember` today. If legacy `ClassMember::Variant` remains, document its getter mapping as legacy and ensure no new enum test reaches it.

**Acceptance criteria**

- Variant selector shapes are derived from the same common selector identity as methods.
- Owner qualification is absent from `Selector`.
- Labels encode without colons.

---

# 11. Task 8 — Add stable syntax diagnostics and migration guidance

**Files**

- Modify: `phalcom-ast/src/error.rs`
- Modify: `phalcom-ast/src/parser.rs`
- Test: `phalcom-ast/tests/parser.rs`
- Test: `phalcom-ast/tests/family_selector_syntax.rs`
- Test: `phalcom-ast/tests/enum_syntax.rs`

**Existing symbols**

- `SyntaxError`
- `SyntaxErrorKind`
- `SyntaxErrorKind::code`
- `UnrecognizedToken`
- `UnrecognizedEof`
- `Message`

- [ ] **Step 1: Add structured variants**

Compile-oriented proposed shape:

```rust
#[derive(Debug, Clone, Error, Eq, PartialEq)]
pub enum SyntaxErrorKind {
    // existing variants ...

    #[error("`@variant` may only declare a variant inside an `enum` body")]
    VariantOutsideEnum,

    #[error("associated whole-family lookup uses `::*`, not `...`")]
    AssociatedLegacyFamilyEllipsis,

    #[error("exact associated member narrowing requires a second `::`")]
    AssociatedExactShapeRequiresSecondSeparator,

    #[error("singleton variants omit parentheses")]
    SingletonVariantHasEmptyParameterList,

    #[error("rest parameters are not supported in variant payloads")]
    VariantRestParameterUnsupported,
}
```

- [ ] **Step 2: Assign stable codes**

Extend `SyntaxErrorKind::code()`:

```rust
Self::VariantOutsideEnum => "syntax.enum.variant_outside_enum",
Self::AssociatedLegacyFamilyEllipsis => "syntax.associated.legacy_family_ellipsis",
Self::AssociatedExactShapeRequiresSecondSeparator => "syntax.associated.exact_requires_second_separator",
Self::SingletonVariantHasEmptyParameterList => "syntax.enum.singleton_parentheses",
Self::VariantRestParameterUnsupported => "syntax.enum.variant_rest_unsupported",
```

- [ ] **Step 3: Assert both category and range**

Each parser test must assert the specific `SyntaxErrorKind` and that its range points at the smallest useful source:

- `@variant` marker for outside-enum;
- ellipsis token for old family syntax;
- underscore/selector-shape region for missing second separator;
- empty `()` for singleton-parentheses;
- rest marker for rest-payload rejection.

- [ ] **Step 4: Keep generic parse failures generic**

Do not convert every malformed enum/associated input into a bespoke error. Preserve existing parser recovery and `UnrecognizedToken`/`UnrecognizedEof` behavior for ordinary grammar mistakes.

**Acceptance criteria**

- Migration errors have stable machine-readable codes.
- No diagnostic uses dot-form variant names or colon-bearing selector labels.

---

# 12. Task 9 — Migrate `phalcom-core` away from old parser-produced `MethodRefExpr` semantics

**Files**

- Modify: `phalcom-core/src/compiler/lib/expr.rs`
- Modify: `phalcom-core/src/compiler/lib/mod.rs`
- Modify: `phalcom-core/src/compiler/lib/error.rs`
- Do **not** modify runtime semantics in `phalcom-core/src/vm/dispatch.rs` in this task
- Do **not** repurpose `phalcom-core/src/bytecode.rs::Bytecode::MakeFamily`

**Existing symbols**

- `Compiler::compile_expr_want`
- `Compiler::immediate_exact_method_ref_selector`
- `Bytecode::MakeFamily`
- `CompilerError`
- statement compilation in `phalcom-core/src/compiler/lib/mod.rs`

**Produces staging errors**

```rust
CompilerError::EnumNotLoweredYet(SourceRange)
CompilerError::AssociatedLookupNotLoweredYet(SourceRange)
CompilerError::AssociatedInvokeNotLoweredYet(SourceRange)
```

- [ ] **Step 1: Add explicit compiler staging errors**

In `phalcom-core/src/compiler/lib/error.rs` add:

```rust
#[error("enum lowering is not implemented until the ADT/GADT semantic/runtime parts land")]
EnumNotLoweredYet(SourceRange),

#[error("associated lookup lowering is not implemented until associated semantic resolution lands")]
AssociatedLookupNotLoweredYet(SourceRange),

#[error("associated family invocation lowering is not implemented until associated semantic resolution lands")]
AssociatedInvokeNotLoweredYet(SourceRange),
```

These errors are temporary architecture guards, not user-facing final enum diagnostics.

- [ ] **Step 2: Make `Statement::Enum` fail explicitly in bytecode compilation**

In the existing statement compilation match in `phalcom-core/src/compiler/lib/mod.rs`, add:

```rust
Statement::Enum(enum_def) => {
    return Err(CompilerError::EnumNotLoweredYet(enum_def.range));
}
```

Do not translate it to `Statement::Class`.

- [ ] **Step 3: Add explicit associated-expression arms**

In `Compiler::compile_expr_want`:

```rust
Expr::AssociatedLookup(expr) => {
    return Err(CompilerError::AssociatedLookupNotLoweredYet(expr.range));
}
Expr::AssociatedInvoke(expr) => {
    return Err(CompilerError::AssociatedInvokeNotLoweredYet(expr.range));
}
```

Do not emit `Invoke`, `InvokePack`, or `MakeFamily`.

- [ ] **Step 4: Remove the parser-era MethodRef optimization when `Expr::MethodRef` is removed**

Delete:

```rust
Compiler::immediate_exact_method_ref_selector
```

and any call site that only exists to optimize immediately-called old `MethodRefExpr` values.

If `MakeFamily` still has non-source internal tests, leave the opcode/runtime implementation in place; Part 4 owns its redesign/removal.

- [ ] **Step 5: Add staging tests**

Add/modify compiler tests to assert:

```phalcom
enum E { @variant X }
```

returns `EnumNotLoweredYet`, and:

```phalcom
E::X
E::X(1)
```

return the associated staging errors rather than dNU/family runtime errors.

**Verification**

```bash
cargo test -p phalcom-core
```

**Acceptance criteria**

- New syntax cannot execute with old semantics.
- `Bytecode::MakeFamily` is not repurposed in Part 1.
- No runtime family allocation is added.

---

# 13. Task 10 — Update `phalcom-semantic` exhaustive AST consumers without adding enum semantics yet

**Files**

Inspect and modify as compilation requires, beginning with:

```text
phalcom-semantic/src/checker/declaration.rs
phalcom-semantic/src/advisory/analyzer.rs
phalcom-semantic/src/source_index/builder.rs
phalcom-semantic/src/source_index/occurrence.rs
phalcom-semantic/src/db/fingerprint.rs
```

**Existing facts**

- `checker/declaration.rs::register_class_surface` ignores legacy `ClassMember::Variant`.
- source-index/advisory code currently matches `Expr::MethodRef`.
- `phalcom-semantic` is the single static semantic authority.

**Goal of this task**

Restore compilation and source traversal for new AST nodes **without** implementing Part 2/3 semantic publication/resolution early.

- [ ] **Step 1: Compile `phalcom-semantic` to enumerate exhaustive-match breakage**

```bash
cargo check -p phalcom-semantic
```

Capture the exact compiler errors and edit only reported AST consumers plus clearly coupled traversal helpers.

- [ ] **Step 2: Add `Statement::Enum` traversal placeholders**

Where syntax-tree walkers need to recurse for ranges/fingerprints, visit:

- enum generic/where syntax as current traversal conventions require;
- enum behavior member bodies;
- variant result annotations;
- variant behavior bodies.

Where semantic declaration registration occurs, **do not** publish variants or enum types yet. Add an explicit no-op/staging branch with a comment referencing Part 2 rather than mapping to class registration.

- [ ] **Step 3: Replace `Expr::MethodRef` traversal with associated nodes**

For syntax/source traversal:

```rust
Expr::AssociatedLookup(expr) => visit_expr(&expr.receiver),
Expr::AssociatedInvoke(expr) => {
    visit_expr(&expr.receiver);
    visit_pack_items(&expr.args);
}
```

Do not treat the base name as a resolved method selector yet.

- [ ] **Step 4: Keep source occurrence identity unresolved where necessary**

If `source_index` requires a semantic target ID that Part 2/3 cannot supply yet, record the occurrence as syntactic/unresolved using the existing unresolved-occurrence mechanism. Do not invent a fake `CallableId` or method identity.

- [ ] **Step 5: Remove MethodRef-specific branches after AST removal**

Once all semantic consumers compile against associated nodes, remove references to `Expr::MethodRef`, `MethodRefExpr`, and `MethodRefKind`.

**Verification**

```bash
cargo test -p phalcom-semantic
```

**Acceptance criteria**

- semantic crate compiles with new AST;
- no variant is published as a method;
- no associated target is resolved by an ad hoc parser/LSP heuristic;
- Part 2/3 still own semantic implementation.

---

# 14. Task 11 — Update LSP structural consumers without creating LSP semantic authority

**Files**

Begin with baseline matches in:

```text
phalcom-lsp/src/selectors.rs
phalcom-lsp/src/hover.rs
phalcom-lsp/src/semantic_tokens.rs
```

**Existing symbols**

- `phalcom-lsp/src/selectors.rs::selector_spec_from_ast`
- `class_member_selector`
- structural selector reexports from `phalcom_ast::selector`
- hover/semantic-token exhaustive matches over `ClassMember`/`Expr`

- [ ] **Step 1: Run an LSP compile to discover exact AST breakage**

```bash
cargo check -p phalcom-lsp
```

- [ ] **Step 2: Remove MethodRef-only imports/branches**

Replace any direct `MethodRefExpr` traversal with the new associated expression shapes.

- [ ] **Step 3: Do not implement associated lookup in `phalcom-lsp/src/selectors.rs`**

The selector utility may normalize syntax to a structural `Selector`, but it must not decide whether a base name denotes a method family, variant family, nested type, or another associated declaration. That decision belongs to `phalcom-semantic` in Part 3.

- [ ] **Step 4: Keep enum semantic tokens conservative**

If semantic token code must recognize the `enum` keyword/name to avoid a crash, it may classify the syntactic declaration token. Do not invent variant target resolution or completion behavior; full IDE semantics are the future LSP workstream.

- [ ] **Step 5: Run LSP tests**

```bash
cargo test -p phalcom-lsp
```

**Acceptance criteria**

- LSP compiles against the new AST.
- No duplicate semantic resolver is introduced.
- Existing class/method selector behavior remains unchanged.

---

# 15. Task 12 — Rewrite selector/family syntax tests around the ratified grammar

**Files**

- Rewrite: `phalcom-ast/tests/family_selector_syntax.rs`
- Extend: `phalcom-ast/tests/enum_syntax.rs`
- Extend: `phalcom-ast/tests/parser.rs`

**Existing test debt**

`family_selector_syntax.rs` currently asserts the superseded model:

```phalcom
receiver::name()
receiver::name(_)
receiver::name...
receiver::name(_, ..., foo)
```

as exact/family selector references.

Those assertions must be replaced, not kept as aliases.

- [ ] **Step 1: Add one table-driven positive syntax matrix**

Cover at least:

```text
receiver::name
receiver::name::
receiver::name::*
receiver::name::()
receiver::name::(_)
receiver::name::(_, reason)
receiver::name::=(put)
receiver::name()
receiver::name(42)
receiver::name(42, reason: "x")
receiver::+
receiver::+(_)
receiver::[x]
receiver::[x, y]
receiver::[x]=(put)
```

Assert AST category, selector kind, and slot shape where applicable.

- [ ] **Step 2: Add a migration-negative matrix**

Cover:

```text
receiver::#name
receiver::#name(_)
receiver::name...
receiver::name(...)
receiver::name(_)
receiver::name(_, reason)
receiver::+::(_)
```

Each should either hit the intended targeted diagnostic or a deliberate generic syntax error. Document the expected code in the test.

- [ ] **Step 3: Prove `#` selector patterns remain independent**

Retain/refine the existing test that these still parse:

```phalcom
#name
#name()
#name(_)
#name...
#name(_, ..., foo)
#name=(put)
```

This prevents the associated-family migration from accidentally deleting reflection syntax.

- [ ] **Step 4: Add dot non-regression tests**

Assert:

A legacy dot-access variant-like spelling and an ordinary getter send such as `Math.pi`

still parse into ordinary dot send/property AST, not associated nodes.

- [ ] **Step 5: Add canonical variant-selector tests**

For:

```phalcom
enum Response<E> {
    @variant Error(_ error: E)
    @variant Error(_ error: E, reason: String)
}
```

assert:

```text
Error(_)
Error(_,reason)
```

No assertion may contain `reason:` or a dot-form variant name.

**Verification**

```bash
cargo test -p phalcom-ast --test family_selector_syntax
cargo test -p phalcom-ast --test enum_syntax
cargo test -p phalcom-ast --test parser
```

---

# 16. Task 13 — Update the active selector specification to stop documenting superseded `::` syntax

**Files**

- Modify: `docs/spec/selectors.md`

**Existing conflict**

The active selector documentation describes the current method-reference/family grammar and therefore becomes misleading as soon as Tasks 4–6 land.

- [ ] **Step 1: Keep structural selector identity sections**

Preserve documentation that remains true:

- getter vs zero-argument method distinction;
- comma-form slots;
- colon-free external label identity;
- subscript selector kinds;
- first-class `#` selector values/patterns.

- [ ] **Step 2: Replace the `::` surface section**

Document exactly:

```phalcom
owner::name
owner::name::
owner::name::*
owner::name(args)
owner::name::(_)
owner::name::(_, reason)
owner::name::=(put)
owner::+
owner::+(_)
owner::[x]
owner::[x]=(put)
```

State that semantic family categories/resolution are implemented in later ADT/GADT associated-lookup parts.

- [ ] **Step 3: Mark associated ellipsis as removed**

Explain that `...` remains selector-pattern syntax only under `#`, not associated whole-family syntax.

- [ ] **Step 4: Add the dot/colon-colon conceptual split**

```text
.  = message send
:: = associated member/family lookup or invocation
```

Do not document a fallback from one to the other.

**Acceptance criteria**

- Active selector docs no longer teach syntax the parser rejects.
- Full enum/runtime documentation remains deferred to Part 6.

---

# 17. Task 14 — Run focused and workspace verification

This task is mandatory before claiming Part 1 implemented.

- [ ] **Step 1: Format**

```bash
cargo fmt --all
cargo fmt --all -- --check
```

- [ ] **Step 2: Run front-end tests**

```bash
cargo test -p phalcom-common
cargo test -p phalcom-ast
```

- [ ] **Step 3: Run affected downstream crates**

```bash
cargo test -p phalcom-semantic
cargo test -p phalcom-core
cargo test -p phalcom-lsp
```

- [ ] **Step 4: Run the workspace**

```bash
cargo test --workspace
```

If the baseline contains unrelated failures, record each exact failure and prove that the focused Part 1 tests pass. Do not relabel unrelated failures as Part 1 failures, and do not hide them.

- [ ] **Step 5: Run clippy under the repository's normal CI expectations**

```bash
cargo clippy --workspace --all-targets -- -D warnings
```

If the baseline is not clippy-clean, record pre-existing diagnostics separately and ensure no new warnings are introduced in touched files.

- [ ] **Step 6: Search for stale surface assumptions**

Run repository searches such as:

```bash
rg '::[^\n]*\.\.\.' phalcom-ast docs/spec
rg 'MethodRefExpr|MethodRefKind' phalcom-ast phalcom-core phalcom-semantic phalcom-lsp
rg 'ClassMember::Variant' phalcom-ast phalcom-core phalcom-semantic phalcom-lsp
rg '@variant .*:' phalcom-ast docs/spec
```

Interpret results carefully:

- `#name...` selector-pattern documentation/tests are allowed;
- legacy `ClassMember::Variant` may remain quarantined until Part 6;
- no parser-produced associated syntax may remain on `MethodRefExpr`;
- colon occurrences in **type annotations/call labels** are allowed; the search is for stale variant selector identity assumptions, not punctuation prohibition globally.

- [ ] **Step 7: Inspect generated parser ASTs for representative fixtures**

Use existing test helpers rather than adding a debug-only binary. Verify these exact examples one final time:

```phalcom
enum Option<T> {
    @variant Some(_ value: T)
    @variant None
}

Option::None
Option::Some(42)
Option::Some::(_)
Option::Some::*

Response::Error::(_, reason)
System::print::(_)
System::print::(_, to)
```

**Completion gate**

Do not claim Part 1 complete unless:

1. front-end tests encode every ratified syntax distinction;
2. new syntax cannot accidentally execute via old method-family semantics;
3. no duplicate semantic authority was added;
4. source ranges are present for later source indexing;
5. the workspace build/test status is documented truthfully.

---

# 18. Concrete expected AST examples

These examples are implementation contracts for reviewers.

## 18.1 Singleton variant

Source:

```phalcom
enum Option<T> {
    @variant None
}
```

Expected essential shape:

```text
Statement::Enum(EnumDef {
  name: "Option",
  members: [
    EnumMember::Variant(VariantDecl {
      name: "None",
      payload: None,
      result_annotation: None,
      body: None,
      ...
    })
  ],
  ...
})
```

Structural selector:

```text
Selector(kind = Getter, base = "None", slots = [])
```

## 18.2 Payload variant

Source:

```phalcom
enum Option<T> {
    @variant Some(_ value: T)
}
```

Expected shape:

```text
VariantDecl {
  name: "Some",
  payload: Some([
    ParameterDef {
      name: "value",
      label: None,
      annotation: T,
      ...
    }
  ]),
  ...
}
```

Structural selector:

```text
Some(_)
```

## 18.3 Labeled payload

Source:

```phalcom
@variant Error(_ error: E, reason: String)
```

Structural selector:

```text
Error(_,reason)
```

Never:

```text
colon-bearing selector-label identity
```

## 18.4 Bare associated getter

Source:

```phalcom
Option::None
```

Expected essential AST:

```text
Expr::AssociatedLookup {
  receiver: Var("Option"),
  member: Named {
    base: "None",
    mode: Getter { explicit_separator_range: None }
  }
}
```

No family object is implied by this AST.

## 18.5 Direct associated call

Source:

```phalcom
Option::Some(42)
```

Expected essential AST:

```text
Expr::AssociatedInvoke {
  receiver: Var("Option"),
  base: "Some",
  args: [42]
}
```

It must not be encoded as `call(AssociatedLookup(Getter "Some"), 42)`.

## 18.6 Exact constructor reference

Source:

```phalcom
Option::Some::(_)
```

Expected essential AST:

```text
Expr::AssociatedLookup {
  receiver: Var("Option"),
  member: Named {
    base: "Some",
    mode: Exact {
      residual: Method([Positional])
    }
  }
}
```

## 18.7 Whole family

Source:

```phalcom
Option::Some::*
```

Expected essential AST:

```text
Expr::AssociatedLookup {
  receiver: Var("Option"),
  member: Named {
    base: "Some",
    mode: Family
  }
}
```

This is not a `SelectorPattern`.

---

# 19. Review checklist against Technical Specification 01

Before handing the implementation to Part 2, perform this self-review:

- [ ] `enum` token and parser route exist.
- [ ] new enum AST is independent from `ClassMember::Variant`.
- [ ] variant payload uses `ParameterDef`, not legacy colon-bearing labels.
- [ ] singleton uses `payload: None`.
- [ ] GADT result annotation is preserved under the explicit assumption.
- [ ] case body and root contract syntax are represented.
- [ ] `.` parser behavior is unchanged.
- [ ] new `::` parser produces associated nodes, not MethodRef.
- [ ] `::name` is getter-shaped, not family-shaped.
- [ ] `::name(args)` is a direct associated invocation node.
- [ ] `::name::*` is family reification.
- [ ] `::name::shape` is exact named narrowing.
- [ ] getter and zero-argument method remain distinct.
- [ ] operators/subscripts avoid redundant narrowing.
- [ ] associated ellipsis is rejected.
- [ ] `#` selector patterns remain supported.
- [ ] colon-free selector identity is covered by tests.
- [ ] source ranges retain both separators where written.
- [ ] `MakeFamily` is not used for new syntax.
- [ ] semantic and LSP consumers compile without resolving associated names themselves.
- [ ] no `match` implementation leaked into Part 1.
- [ ] no built-in Option migration leaked into Part 1.

---

# 20. Part 1 done-state and handoff to Part 2

When this plan is complete, Part 2 should be able to start from a stable syntax contract rather than reinterpreting parser artifacts.

Part 2 receives:

```text
EnumDef
VariantDecl
exact structural selector per variant
AssociatedLookupExpr
AssociatedInvokeExpr
precise source ranges
explicit getter/family/exact/invoke syntax categories
```

Part 2 then introduces the semantic declaration model:

```text
Enum identity
VariantFamilyId
VariantId(owner, exact selector)
exact case type identity
case/runtime-class linkage metadata
base-name family reservation
shared enum contracts
visibility metadata
constructor taxonomy/publication
```

No Part 2 work should require restoring `MethodRefExpr`, dot-based variants, colon-bearing selector labels, or the legacy sealed-class `@variant` expansion surface.
