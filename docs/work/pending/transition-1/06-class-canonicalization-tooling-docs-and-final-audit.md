# Task 6 — `@class` Canonicalization, Tooling, Documentation, Generated Metadata, and Final Audit

> **Repository:** `aureat/phalcom-lang`  
> **Depends on:** Tasks 1–5  
> **Primary objective:** Finish the migration by removing transitional class-side syntax from canonical source, updating LSP/tooling/generated metadata/specification, deleting obsolete underscore heuristics, and running a comprehensive repository audit.

---

## 1. Final state this task must enforce

At the end of this task the repository should describe and demonstrate one coherent language.

Canonical source:

```phalcom
@class
factory(_ value) {
  ...
}

@class
_cache = None

@private
helper(_ value) {
  ...
}

@protected
familyHelper(_ value) {
  ...
}

method(_ positional, label local, sameNameLabel) {
  ...
}

property=(put value) {
  ...
}

[_ index, default fallback] {
  ...
}

[_ index, default fallback]=(put value) {
  ...
}
```

Namespace meanings:

```text
name       ordinary lexical name / selector
_name      source field
__name     implementation field
_$name     implementation selector
```

No canonical `.ph` code uses `static` for class-side members. No canonical `.ph` method uses leading underscore to signal privacy. No canonical primitive uses trailing underscore merely as pseudo-private convention.

---

## 2. Files to edit

Class-side parser/token cleanup:

```text
phalcom-ast/src/token.rs
phalcom-ast/src/lexer.rs
phalcom-ast/src/parser.rs
phalcom-ast/src/ast.rs
phalcom-core/src/compiler/attributes.rs
phalcom-core/src/compiler/lib/class_decl.rs
```

Core/source fixtures:

```text
phalcom-core/core/core.ph
examples/**/*.ph
phalcom-core/tests/**/*.ph
```

LSP:

```text
phalcom-lsp/src/index.rs
phalcom-lsp/src/completion.rs
phalcom-lsp/src/semantic_tokens.rs
```

Generated tooling:

```text
tools/vsphalcom/src/generated/core-table.json
tools/vsphalcom/**
```

Specification/decision records, at minimum inspect:

```text
docs/spec/current/syntax/lexical.md
docs/spec/current/object-model.md
docs/spec/current/functions.md
docs/spec/current/core/core-classes.md
docs/spec/design/decorators/canonical/placement.md
docs/pdr/0028-class-and-constructor-decorator-canon.md
docs/adr/accepted/0060-index-operator-as-real-selector.md
docs/adr/proposed/0061-underscore-prefix-reservation-fields-internals-reserved.md
docs/spec/numerical/float-protocol.md
docs/pdr/0027-float-protocol-and-explicit-narrowing.md
docs/adr/STATUS.md
```

Do not update historical/retired documents to pretend they always used the new design. Mark them superseded/amended where appropriate.

---

## 3. Migrate `static` to `@class`

PDR-0028 already establishes `@class` as canonical for class-side fields, methods, getters, and setters. Apply that decision consistently.

Before:

```phalcom
static count => _count

static from(_ record) {
  ...
}

static _cache = None
```

After:

```phalcom
@class
count => _count

@class
from(_ record) {
  ...
}

@class
_cache = None
```

Search:

```bash
rg -n --glob '*.ph' '\bstatic\b' .
rg -n --glob '*.md' '\bstatic\b' docs
```

For markdown, distinguish historical discussion from canonical examples. Historical records may retain quoted legacy syntax with explicit "legacy" labeling.

---

## 4. Parser compatibility policy for `static`

During this task choose and implement the repository's intended compatibility boundary.

Recommended final behavior:

1. Canonical source and tests contain no `static` member declarations.
2. Parser may still recognize `static` only to emit a targeted migration diagnostic for one release/migration window.
3. It must not lower `static` silently as a permanent alias.
4. Once the migration window is intentionally closed, remove `Token::Static` and lexer keyword recognition entirely.

If the project is still pre-release and compatibility is unnecessary, remove it immediately after canonical source is migrated.

Target diagnostic if retained:

```text
`static` member syntax is retired; use `@class`
```

Also locate transitional `class foo()` member syntax if it still exists and remove/diagnose it according to PDR-0028.

---

## 5. Remove obsolete field/method prefix heuristics

Now that lexical namespaces are structural, delete old compiler/parser checks that infer a field because a name string begins `_`.

Search:

```bash
rg -n "starts_with\\(\"_\"\\)|starts_with\\('_'\\)|strip_prefix\\(\"_\"\\)|strip_prefix\\('_'\\)" phalcom-ast phalcom-core
```

Review each hit.

Delete logic such as:

```rust
if getter.name.starts_with('_') {
    // infer field
}
```

Field layout collection must use:

- explicit `FieldDef`;
- actual field-expression/assignment AST where legacy inferred layout remains supported;
- field-kind metadata.

No method/getter selector spelling can imply field storage.

Likewise, no method visibility logic should inspect `_` prefixes.

---

## 6. Finalize selector display everywhere

Every user-visible selector formatter, debugger, disassembler, reflection display, LSP index, generated table, and test snapshot must use the same canonical forms.

Required:

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

Search for hand-built selector strings:

```bash
rg -n 'format!\([^)]*"[^"]*\(' phalcom-core phalcom-lsp tools
rg -n '"\\[.*put.*\\]"|"=\\(_\\)"' phalcom-core phalcom-lsp tools docs
```

Prefer calling the existing canonical `encode_selector` instead of reproducing formatting rules.

---

## 7. LSP semantic tokens

Update `phalcom-lsp/src/semantic_tokens.rs` so token classification matches final namespace semantics.

At minimum:

```text
name       ordinary variable/member context
_field     field
__field    internal/implementation field
_$name     internal/implementation method/member
```

If the semantic-token protocol has no dedicated "internal" modifier, use the closest stable token type plus a modifier already supported by the project. Do not invent an LSP extension solely for this migration.

External labels in declarations should highlight as parameter/label syntax according to existing conventions:

```phalcom
move(_ point, to destination, duration seconds)
```

The local bindings are:

```text
point
destination
seconds
```

The external selector labels are:

```text
_
to
duration
```

Ensure the indexer does not mistake `to`/`duration` for local parameter definitions.

---

## 8. LSP index and completion

### 8.1 Index

Update `phalcom-lsp/src/index.rs` to store/display canonical selector identity using the new parameter labels and subscript getter/setter split.

A source declaration:

```phalcom
[_ index, default fallback]=(put value)
```

must index as:

```text
[_,default]=(put)
```

not:

```text
[_,default,put]
```

### 8.2 Completion

Inside a member body, completion should account for implicit `self`.

When completing an unresolved ordinary identifier, offer:

- lexical locals/parameters;
- visible globals/imports;
- accessible self members.

Visibility filtering:

```text
public        always according to receiver/type knowledge
private       only in defining lexical class
protected     in defining class/family
internal      only in privileged core/runtime source
```

Do not offer `_$...` implementation selectors to normal user files.

Fields should be offered using `_field`; implementation fields only in privileged source.

Completion behavior does not need whole-program perfect type inference. Respect available class/member context and avoid obvious illegal suggestions.

---

## 9. Generated core metadata

The repository contains generated core selector metadata. Regenerate it using the project's generator rather than hand editing if a generator exists.

Locate generation commands:

```bash
rg -n "core-table.json|generated/core-table|generate.*core|vsphalcom" tools docs Cargo.toml
```

Expected changes include:

- Float `Method(0)` entries becoming getters;
- old trailing-underscore internal selectors becoming `_$...`;
- method-like `__...` runtime hooks becoming `_$...`;
- setters rendered `=(put)`;
- subscript setters rendered `[...] = (put)`;
- parameter selector labels reflecting Task 2 declarations.

After regeneration:

```bash
git diff -- tools/vsphalcom/src/generated/core-table.json
```

Review the diff semantically. Generated output should not unexpectedly remove unrelated APIs.

---

## 10. Specification update: one normative convergence record

Create or update one central design/decision record that states the final model in one place. It should supersede proposed ADR-0061 and amend ADR-0060/PDR-0028 as necessary.

The record must state:

### Namespaces

```text
name       ordinary namespace
_name      source field
__name     implementation field
_$name     implementation selector
```

### Visibility

```text
@private    defining source class only
@protected  defining source class and subclasses
```

### Implicit self

```text
_field / __field / _$selector:
    namespace-directed, unconditional self semantics

ordinary name:
    local -> upvalue -> known global -> implicit self
```

### Declaration parameters

```text
_ local
label local
label
*rest
```

### Setters

```text
name=(put)
```

### Subscripts

```text
[_,default]
[_,default]=(put)
```

### Native versus internal

A native implementation may occupy a public selector. `_$` means internal, not merely native.

---

## 11. Update lexical specification

`docs/spec/current/syntax/lexical.md` must document the prefix forms and reserved/malformed cases.

Explicitly state that `_` alone is the positional declaration marker, while `_name` is a field token.

Document whether:

```text
___name
__$name
_$
__
```

are invalid or reserved.

Do not leave the lexer behavior as an implementation detail.

---

## 12. Update object model / selector specification

Document direct fields:

```phalcom
_field
self._field
```

and reject foreign receiver field access:

```phalcom
other._field
```

Document implementation fields as privileged.

Document that private/protected are selector visibility, not naming conventions.

Document explicit/implicit self equivalence.

Document reflection rules:

- physical method enumeration may expose selector existence if intended;
- invocation still enforces visibility;
- `perform` does not bypass access;
- method objects/references do not bypass access.

---

## 13. Update function/method declaration documentation

All canonical examples must use:

```phalcom
foo(_ x)
foo(label)
foo(label local)
foo(_ x, label y)
```

Remove declaration examples using:

```phalcom
foo(x)           # if intended positional
foo(label:)
foo(label: local)
```

Call examples retain:

```phalcom
foo(label: value)
```

Make this declaration/call difference explicit because it will otherwise be a common migration mistake.

---

## 14. Update ADR-0060 subscript decision

ADR-0060 currently documents write identity using the old `put` slot inside bracket selector identity.

Amend/supersede the write portion to:

```text
getter: [_,default]
setter: [_,default]=(put)
```

Explain that bracket slots describe indexing arguments only; the assignment value occupies the fixed setter role `(put)`.

Update examples:

```phalcom
[_ index] { ... }

[_ index]=(put value) {
  ...
}

[_ index, default fallback]=(put value) {
  ...
}
```

Do not rewrite the historical motivation; add a clear amendment noting the later setter-kind split.

---

## 15. Supersede proposed ADR-0061

The old proposal explored underscore-prefix reservation. Replace its proposed model with the ratified final model or mark it superseded by the new convergence decision.

Important differences to record:

```text
_name      field
__name     implementation field
_$name     implementation selector
@private   class-private selector visibility
@protected class-family visibility
```

Do not leave contradictory proposed documents appearing active in `docs/adr/STATUS.md`.

---

## 16. Float docs

Ensure Float docs describe the eleven operations as getter/property selectors.

Canonical examples:

```phalcom
2.5.rounded
3.0.isInteger
value.toIntExact
```

Do not show:

```phalcom
2.5.rounded()
```

unless discussing explicitly distinct user-added method selectors.

If PDR text says a count of native bindings that differs from the final implemented protocol, correct the count or avoid stale hard-coded counts where the project's conventions prefer derived census.

Do not change numeric behavior in this documentation task.

---

## 17. Migration guide

Add a compact but complete migration table.

Required entries:

```text
OLD                               NEW
foo(x) positional                 foo(_ x)
foo(label:)                       foo(label)
foo(label: local)                 foo(label local)
foo=(value)                       foo=(put value)
[idx, put:]                       [_ idx]=(put value)
[idx, default:, put:]             [_ idx, default fallback]=(put value)
static foo(...)                   @class + foo(...)
static _field                     @class + _field
_helper(...) private convention   @private helper(...)
size_ internal primitive          _$size
__runtimeHook method              _$runtimeHook
```

State clearly that call-site `label:` syntax is unchanged.

---

## 18. Repository-wide audits

Run all of these after implementation.

### 18.1 Legacy `static`

```bash
rg -n --glob '*.ph' '\bstatic\b' .
```

Expected: zero canonical source hits.

### 18.2 Old declaration label syntax

```bash
rg -n --glob '*.ph' '[A-Za-z_][A-Za-z0-9_]*:\s*[A-Za-z_][A-Za-z0-9_]*'
```

Review every hit; valid call-site labels will remain.

### 18.3 Underscore methods

```bash
rg -n --glob '*.ph' '^\s*_[A-Za-z][A-Za-z0-9_]*\s*(\(|=>|\{)' .
```

Expected: zero method declarations.

### 18.4 Method-like double underscore

```bash
rg -n --glob '*.{ph,rs,json,md}' '__[A-Za-z][A-Za-z0-9]*\s*(\(|=>)'
```

Review; Phalcom selector-level method hits should be gone. Rust identifiers/comments can remain.

### 18.5 Trailing-underscore internal selectors

```bash
rg -n --glob '*.{ph,rs,json,md}' '\b[A-Za-z][A-Za-z0-9]*_\b' .
```

Review every selector-level hit. Rust snake_case/native function names are unrelated.

### 18.6 Old setter encodings

```bash
rg -n --glob '*.{rs,ph,md,json}' '=\\(_\\)'
```

Expected: no canonical ordinary setter selector strings.

### 18.7 Old subscript put-slot encoding

```bash
rg -n --glob '*.{rs,ph,md,json}' '\[[^]]*put[^]]*\]'
```

Expected: no canonical selector identity that places assignment `put` inside brackets.

### 18.8 Field-prefix heuristics

```bash
rg -n "starts_with\\(\"_\"\\)|starts_with\\('_'\\)" phalcom-ast/src phalcom-core/src
```

Every remaining hit needs a reason unrelated to deciding field-vs-method namespace or privacy.

---

## 19. Full test/verification matrix

Run:

```bash
cargo fmt --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

Then execute any repository-specific generated-file check.

If there is a CLI smoke test:

```bash
cargo run -p phalcom-core --bin phalcom -- <core/example invocation>
```

use the repository's actual command to verify VM bootstrap loads and executes `core.ph`.

Required behavioral smoke coverage:

```text
1. core.ph compiles and executes
2. Float getter protocol works
3. implicit self works
4. private/protected visibility works
5. reflection cannot bypass visibility
6. internal selectors reject user access
7. subscript getter/setter dispatch are distinct
8. @class fields and methods work
9. legacy syntax is absent/diagnosed according to policy
```

---

## 20. Final completion checklist

Do not mark the overall migration complete until every item is true.

### Language surface

- [ ] `_name` is always source field.
- [ ] `__name` is implementation field.
- [ ] `_$name` is implementation selector.
- [ ] `_` is positional declaration marker.
- [ ] `@private` is class-private.
- [ ] `@protected` is class-family-private.
- [ ] ordinary selector declarations use external-label/local-name grammar.
- [ ] setter canonical identity is `=(put)`.
- [ ] subscript setter canonical identity is `[...] = (put)`.

### Compiler/runtime

- [ ] implicit self respects lexical/global shadowing.
- [ ] nested blocks preserve lexical access context.
- [ ] reflection and cached dispatch enforce visibility.
- [ ] internal selector privilege cannot be forged from user source.
- [ ] no field/method classification relies on follower-token or string-prefix heuristic.

### Core

- [ ] `core.ph` uses canonical parameter declarations.
- [ ] `core.ph` uses `@class`.
- [ ] source-private helpers use decorators.
- [ ] raw primitives use `_$`.
- [ ] Float has no forwarding wrapper layer.
- [ ] `core.ph` compiles and runs cleanly.

### Tooling/docs

- [ ] LSP understands new namespaces and labels.
- [ ] generated core metadata is regenerated.
- [ ] active specs agree.
- [ ] proposed ADR-0061 is superseded.
- [ ] ADR-0060 is amended for setter identity.
- [ ] migration guide exists.
- [ ] repository-wide audits are clean.
- [ ] full workspace tests pass.

---

## 21. Recommended final commit sequence

Suggested reviewable commits:

```text
refactor(core): migrate static members to @class
refactor(parser): retire legacy static member syntax
refactor(compiler): remove underscore field-name heuristics
feat(lsp): index implicit-self and new selector namespaces
chore(vsphalcom): regenerate core selector metadata
docs(language): ratify member namespaces visibility and setter signatures
docs(migration): document language-surface convergence
test: add final canonical-source and migration audits
```

The final branch should leave no ambiguity about which Phalcom language design is current.
