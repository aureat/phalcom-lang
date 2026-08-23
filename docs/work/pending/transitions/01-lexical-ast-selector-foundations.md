# Task 1 — Lexical Namespaces, AST Foundations, and Selector Kinds

> **Repository:** `aureat/phalcom-lang`
> **Baseline inspected:** `main` at `c406977666aa5d9d05d3dbf9a78e7c55b39d0b98`
> **Dependency:** none
> **Must finish before:** Tasks 2–6
> **Primary objective:** Introduce the structural language/runtime representations required by the migration without yet performing the repository-wide declaration-syntax flag day.

---

## 1. Purpose and scope

This task lays the structural foundation for the language-surface convergence. Do not migrate `core.ph` broadly in this task. Do not enable implicit `self` yet. Do not implement `@private` or `@protected` semantics yet. Do not rename all internal primitives yet. The goal is to make the lexer, AST, selector model, and method metadata capable of representing the final design so later tasks can change behavior without inventing ad-hoc compatibility structures.

The final namespace model that every later task relies on is:

```text
name       ordinary lexical identifier / ordinary selector
_name      source field
__name     implementation field
_$name     implementation selector
_          positional-slot marker in selector-bearing declarations
```

These namespaces are semantic. They must not be reconstructed by checking string prefixes in random parser/compiler code after lexing.

The final selector/accessor model that later tasks rely on is:

```text
foo
foo()
foo(_,label)
foo=(put)

[_]
[_,default]
[_]=(put)
[_,default]=(put)
```

The current repository instead has a single identifier token that includes underscore-prefixed names, a single `SignatureKind::Subscript`, and ordinary setter encoding `name=(_)`. This task changes those foundations first.

---

## 2. Files to modify

Edit these files:

```text
phalcom-ast/src/token.rs
phalcom-ast/src/lexer.rs
phalcom-ast/src/ast.rs
phalcom-ast/src/parser.rs
phalcom-core/src/method/mod.rs
phalcom-core/src/method/object.rs
phalcom-core/src/compiler/lib/class_decl.rs
phalcom-core/src/compiler/attributes.rs
```

Likely tests to update or extend:

```text
phalcom-ast/tests/lexer.rs
phalcom-ast/src/parser.rs                  # parser unit tests if colocated
phalcom-core/src/method/mod.rs             # selector encode/decode unit tests
phalcom-core/src/compiler/...              # existing compiler unit tests
```

Before editing, locate exact nearby tests:

```bash
rg -n "decode_inverts_encode|Subscript|SelectorSymbol|Identifier\\(" phalcom-ast phalcom-core
rg -n "#\\[test\\]" phalcom-ast/src/parser.rs phalcom-ast/tests phalcom-core/src/method
```

Do not create a parallel selector encoder or a second parser implementation. Extend the existing canonical paths.

---

## 3. Lexical namespace representation

### 3.1 Replace prefix ambiguity with token identity

In `phalcom-ast/src/token.rs`, add explicit token variants for the namespaces.

Target shape:

```rust
pub enum Token {
    // ...

    Identifier(String),

    /// The standalone `_` positional-slot marker.
    Underscore,

    /// `_name`: source/object field namespace.
    FieldIdentifier(String),

    /// `__name`: implementation-field namespace.
    ImplementationFieldIdentifier(String),

    /// `_$name`: implementation-selector namespace.
    ImplementationSelectorIdentifier(String),

    // ...
}
```

Keep the complete spelling in the string payload, e.g. `"_items"` rather than `"items"`. This minimizes churn in field-layout code, diagnostics, interning, and existing AST assumptions.

### 3.2 Lexer rules

In `phalcom-ast/src/lexer.rs`, make underscore scanning deterministic.

Required classification:

```text
_            -> Token::Underscore
_name        -> Token::FieldIdentifier("_name")
__name       -> Token::ImplementationFieldIdentifier("__name")
_$name       -> Token::ImplementationSelectorIdentifier("_$name")
name         -> Token::Identifier("name")
```

Use the existing ordinary identifier continuation rules after the prefix. Require an alphabetic identifier start after `_`, `__`, or `_$` unless the current language already permits a broader start and there is a deliberate reason to preserve it.

Explicitly reject or reserve malformed/undefined forms. At minimum add tests for:

```text
___foo
__$foo
_$
__
```

Choose one stable lexer error path for malformed forms. Do not silently classify them as ordinary identifiers.

### 3.3 `static` remains transitional for now

Do **not** remove `Token::Static` in Task 1. Task 6 performs the final `static` → `@class` source migration and compatibility cleanup. The important change here is namespace tokenization, not class-side syntax.

---

## 4. AST changes

### 4.1 Preserve semantic namespace in expressions

Inspect the current `Expr` enum in `phalcom-ast/src/ast.rs`. It already contains `Expr::Field`. Keep that for `_name`.

Add a distinct expression form for implementation fields unless the existing `Expr::Field` can safely carry a field-kind enum without large churn.

Recommended explicit representation:

```rust
pub enum FieldKind {
    Source,
    Implementation,
}

pub enum Expr {
    // ...
    Field {
        value: String,
        kind: FieldKind,
        range: SourceRange,
    },
    // ...
}
```

This is preferable to adding two almost-identical expression variants because the compiler's slot read/write logic is largely shared. If converting the existing variant to this shape creates excessive churn, two variants are acceptable, but do not later infer implementation status from `value.starts_with("__")`.

### 4.2 Prepare subscript accessor identity

Current `IndexMethodDef` has only:

```rust
pub struct IndexMethodDef {
    pub params: Vec<ParameterDef>,
    pub body: Vec<Statement>,
    // ...
}
```

Change it to distinguish read and write accessors structurally.

Recommended:

```rust
#[derive(Debug, Clone)]
pub enum IndexAccessor {
    Get,
    Set {
        put: ParameterDef,
    },
}

pub struct IndexMethodDef {
    /// Indexing arguments only. The assignment value is not included here.
    pub params: Vec<ParameterDef>,
    pub accessor: IndexAccessor,
    pub body: Vec<Statement>,
    pub attributes: Vec<Attribute>,
    pub range: SourceRange,
    pub name_range: SourceRange,
}
```

Do **not** represent a setter's assigned value by appending a fake `put:` parameter into `params`. `params` should describe the bracket contents only. The assignment operand is a different semantic role and becomes the `put` payload.

### 4.3 Prepare ordinary setter parameter metadata

Current `SetterDef` stores a bare `String` parameter. Replace it with a proper parameter object or a setter-specific structure with name/range.

Preferred minimal change:

```rust
pub struct SetterDef {
    pub name: String,
    pub param: ParameterDef,
    pub body: Vec<Statement>,
    pub is_static: bool,
    pub attributes: Vec<Attribute>,
    pub range: SourceRange,
    pub name_range: SourceRange,
}
```

The selector label is fixed `put`, so the AST parameter's local binding should be `value`, `newValue`, etc. Its `label` should not pretend that the local declaration itself supplied a user-selectable selector label.

### 4.4 Add future visibility representation now

Add a compiler/runtime-neutral visibility enum in an appropriate shared location. If `phalcom-ast` should remain syntax-only, put the final runtime enum in `phalcom-core` and store visibility annotations in member attributes until expansion. If the compiler already lowers builtin attributes before member compilation, do not duplicate visibility in the parse AST unnecessarily.

The runtime representation required by Task 4 is:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemberVisibility {
    Public,
    Private,
    Protected,
    Internal,
}
```

Add this to the method/runtime layer now so `MethodObject` can carry it. Initialize all existing methods as `Public`, except explicit runtime-only registrations that are already obviously internal may remain `Public` temporarily until Task 5 migrates them. Do not change behavior yet.

---

## 5. Selector signature-kind split

### 5.1 Replace ambiguous `Subscript`

In `phalcom-core/src/method/mod.rs`, replace:

```rust
SignatureKind::Subscript(u8)
```

with:

```rust
SignatureKind::SubscriptGet(u8),
SignatureKind::SubscriptSet(u8),
```

The payload means number of **index arguments**, not total runtime stack arguments.

Examples:

```text
[_]                  SubscriptGet(1)
[_,default]          SubscriptGet(2)
[_]=(put)            SubscriptSet(1)
[_,default]=(put)    SubscriptSet(2)
```

A `SubscriptSet(2)` invocation receives three runtime argument values: two index arguments plus the assigned value.

Update every exhaustive `match SignatureKind` in the workspace. Find them with:

```bash
rg -n "SignatureKind::Subscript|match .*SignatureKind|SignatureKind::Setter" phalcom-core phalcom-ast phalcom-lsp tools
```

### 5.2 Change ordinary setter encoding

Current encoding is:

```text
name=(_)
```

Final encoding must be:

```text
name=(put)
```

Update:

```rust
SignatureKind::Setter => format!("{name}=(put)")
```

The runtime arity remains one.

### 5.3 Implement subscript encoding

Target logic:

```rust
match kind {
    SignatureKind::SubscriptGet(_) => {
        format!("[{}]", comma_form_slots(labels))
    }
    SignatureKind::SubscriptSet(_) => {
        format!("[{}]=(put)", comma_form_slots(labels))
    }
    // ...
}
```

For subscript setters, the `labels` input contains index labels only. Do not append `put` before calling `encode_selector`.

### 5.4 Update decoder

`decode_selector` must be the inverse of the new encoding.

Required examples:

```text
"[_]"                  -> ("[]",  [None],                  SubscriptGet(1))
"[_,default]"          -> ("[]",  [None, Some("default")], SubscriptGet(2))
"[_]=(put)"            -> ("[]=", [None],                  SubscriptSet(1))
"[_,default]=(put)"    -> ("[]=", [None, Some("default")], SubscriptSet(2))
"name=(put)"           -> ("name", [Some("put") or agreed setter representation], Setter)
```

Be consistent about how `decode_selector` reports setter labels to reflective callers. Since a reified `Message` has one runtime argument for an ordinary setter, exposing `"put"` as its label is preferable to the old positional placeholder. For a subscript setter, Message reification should produce index labels followed by `"put"` so `message.labels.len() == message.args.len()` remains true.

If `decode_selector` is used elsewhere under the assumption that Setter yields `[None]`, update those callers and tests explicitly.

### 5.5 Round-trip tests

Add direct unit tests in `phalcom-core/src/method/mod.rs`.

Required cases:

```rust
#[test]
fn subscript_getter_round_trips() {
    // [_]
    // [_,default]
}

#[test]
fn subscript_setter_round_trips() {
    // [_]=(put)
    // [_,default]=(put)
}

#[test]
fn setter_uses_put_slot() {
    // name=(put)
}

#[test]
fn getter_and_setter_selectors_are_distinct() {
    assert_ne!("[_]", "[_]=(put)");
}
```

Also retain operator/setter disambiguation tests such as `==(_)` not being parsed as a setter.

---

## 6. MethodObject metadata foundation

In `phalcom-core/src/method/object.rs`, extend `MethodObject` with visibility/access fields required by Task 4.

Target shape:

```rust
pub struct MethodObject {
    pub kind: MethodKind,
    pub signature: Signature,
    pub holder: Option<ClassId>,

    pub visibility: MemberVisibility,

    /// Lexical source class controlling @private/@protected access.
    pub access_owner: Option<ClassId>,

    // existing metadata...
}
```

Constructor defaults:

```rust
visibility: MemberVisibility::Public,
access_owner: holder,
```

Do not enforce visibility in Task 1. The purpose is to make all construction paths compile and give Task 4 a single field to consume.

Search every explicit `MethodObject { ... }` initializer:

```bash
rg -n "MethodObject\s*\{" phalcom-core
```

Update all of them.

---

## 7. Parser compatibility in this task

Do not switch the meaning of `foo(x)` yet. That flag day belongs to Task 2 because existing source uses bare identifiers for positional declarations.

However, make `parse_primary` and member-name parsing consume the new token variants correctly enough that current source can continue to parse during the foundation branch.

Temporary parser compatibility rules allowed in Task 1:

- `FieldIdentifier` produces `Expr::Field { kind: Source, ... }`.
- `ImplementationFieldIdentifier` produces `Expr::Field { kind: Implementation, ... }`.
- `ImplementationSelectorIdentifier` may be accepted only in explicit property/member contexts needed by tests; full privileged semantics come later.
- Existing positional declaration grammar may continue recognizing ordinary `Identifier` as positional until Task 2.
- Existing legacy underscore method declarations must not be expanded further. If the token split causes them to stop parsing, either migrate the tiny number of foundation tests immediately or add a short-lived targeted compatibility branch that is deleted in Task 2. Do not turn `FieldIdentifier` back into an unrestricted method-name token globally.

---

## 8. Field compiler preparation

In `phalcom-core/src/compiler/lib/class_decl.rs`, identify all places that infer member kind from string prefix.

Search:

```bash
rg -n "starts_with\('_'\)|starts_with\(\"_\"\)|strip_prefix\('_'\)|strip_prefix\(\"_\"\)" phalcom-core/src/compiler
```

Do not necessarily remove all field-layout inference in Task 1, because repository source still uses transitional forms. Add comments or helper functions so Task 5/Task 6 can remove them cleanly.

If an AST field-kind enum was added, update direct field read/write compilation so both `Source` and `Implementation` fields use the same slot machinery but remain distinguishable for privilege checks later.

---

## 9. Attribute registry preparation

In `phalcom-ast/src/ast.rs`, add builtin attribute names if the enum is the canonical parser/compiler registry:

```rust
BuiltinAttr::Private,
BuiltinAttr::Protected,
```

Update:

```rust
BuiltinAttr::name()
BuiltinAttr::parse()
```

In `phalcom-core/src/compiler/attributes.rs`, register legality stubs if necessary so parsing these attributes does not produce `attr.unknown`. Do not implement final access semantics yet. If the current attribute pipeline requires an expander, create a minimal expander that records/retains the attribute and lets Task 4 consume it. Do not silently discard it.

---

## 10. Tests and commands

Run targeted tests first:

```bash
cargo test -p phalcom-ast
cargo test -p phalcom-core method::
```

Then:

```bash
cargo test --workspace
cargo fmt --check
```

If the repository uses Clippy in CI:

```bash
cargo clippy --workspace --all-targets -- -D warnings
```

Use the repository's actual CI command if it differs.

---

## 11. Required acceptance criteria

Task 1 is complete only when all of the following are true:

- [ ] `_`, `_name`, `__name`, and `_$name` lex into structurally distinct token categories.
- [ ] No newly written parser code decides namespace by inspecting string prefixes.
- [ ] `SignatureKind` distinguishes subscript get from subscript set.
- [ ] `encode_selector` emits `name=(put)` for ordinary setters.
- [ ] `encode_selector` emits `[...]=(put)` for subscript setters.
- [ ] `decode_selector` round-trips every new setter/subscript form.
- [ ] `IndexMethodDef` can represent getter and setter declarations without putting the assigned value into the bracket argument vector.
- [ ] `SetterDef` stores a proper local parameter node/range rather than only a string.
- [ ] `MethodObject` carries visibility and lexical access-owner metadata with behavior-neutral defaults.
- [ ] `@private` and `@protected` are recognized names, but no access-control behavior is claimed yet.
- [ ] Existing workspace tests pass or any unavoidable transitional parser failures are explicitly migrated and documented.
- [ ] No broad `core.ph` migration has been mixed into this foundational task.

---

## 12. Commit guidance

Suggested commits:

```text
refactor(ast): split member identifier namespaces
refactor(selectors): distinguish subscript getter and setter kinds
refactor(method): add visibility metadata foundation
```

Do not combine Task 2's declaration flag day into these commits. A reviewer must be able to validate that Task 1 is primarily structural.
