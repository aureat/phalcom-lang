# Task 2 — Selector Parameter Grammar, Setter Grammar, and Repository Declaration Migration

> **Repository:** `aureat/phalcom-lang`
> **Depends on:** Task 1
> **Must finish before:** Tasks 3–6
> **Primary objective:** Perform the deliberate source-breaking declaration grammar switch atomically and migrate every selector-bearing declaration to the new external-label/local-name form.

---

## 1. Why this task is an atomic flag day

Do not attempt to support both the old and new meaning of a bare parameter declaration.

Old Phalcom:

```phalcom
foo(value)
```

means one **positional** parameter.

New Phalcom:

```phalcom
foo(value)
```

means one **labeled** parameter whose external label and local binding are both `value`.

These syntaxes are textually identical and semantically different. A compatibility parser cannot infer intent. Therefore the parser semantic switch and the repository-wide declaration migration must happen in one task/branch boundary.

The new declaration model is:

```text
_ local        positional slot
label local    labeled slot with a different local name
label          labeled slot with the same external and local name
*rest          variadic positional tail
```

Examples:

```phalcom
at(_ index)

move(_ point, to destination, duration seconds)

from(record)

format(_ template, *values)
```

Canonical selector identities:

```text
at(_)
move(_,to,duration)
from(record)
format(*)
```

Call-site label syntax does **not** change:

```phalcom
move(point, to: destination, duration: seconds)
```

The colon is removed only from selector-bearing **declarations** so `:` remains available for future type annotations.

---

## 2. Files to edit

Primary parser/AST/compiler files:

```text
phalcom-ast/src/parser.rs
phalcom-ast/src/ast.rs
phalcom-core/src/compiler/lib/class_decl.rs
phalcom-core/src/compiler/lib/mod.rs
phalcom-core/src/compiler/lib/expr.rs
phalcom-core/src/method/mod.rs
```

Repository source migration targets:

```text
phalcom-core/core/core.ph
examples/**/*.ph
phalcom-core/tests/**/*.ph
phalcom-ast/tests/**/*.ph
tests/**/*.ph                         # if present
benchmarks/**/*.ph                    # if present
docs/**/*.md                          # executable/checked Phalcom examples
```

Generated/derived fixtures may also contain syntax. Discover them:

```bash
rg -n --glob '*.ph' '\b[A-Za-z_][A-Za-z0-9_]*\([^)]*\)' .
rg -n --glob '*.ph' '[A-Za-z_][A-Za-z0-9_]*:' .
rg -n --glob '*.md' '```phalcom' docs
```

Do not apply a blind regex replacement to all parenthesized text. The migration must distinguish declarations from calls, block parameters, symbols, and examples.

---

## 3. Split selector-parameter parsing from lexical-parameter parsing

The current parser reuses generic parameter parsing for several contexts. Stop doing that for selector-bearing members.

Introduce conceptual functions like:

```rust
fn parse_selector_params(
    &mut self,
    end: Token,
) -> ParserResult<Vec<ParameterDef>>

fn parse_block_params(
    &mut self,
    // existing block grammar
) -> ParserResult<Vec<String>>
```

Exact naming may follow local style, but the semantic split is mandatory.

Selector parameter parsing is used by:

- ordinary method declarations;
- constructors;
- operator method declarations if represented through the same path;
- subscript getter declarations;
- the bracket portion of subscript setter declarations.

Block/function-local parameter grammar should remain lexical-only unless it is separately specified. Do not introduce external labels into blocks as a side effect.

---

## 4. New selector-parameter grammar

Implement:

```ebnf
selector-parameter-list
  = [ selector-parameter { "," selector-parameter } ] ;

selector-parameter
  = "_" identifier
  | identifier identifier
  | identifier
  | "*" identifier ;
```

### 4.1 Positional

```phalcom
foo(_ value)
```

AST:

```rust
ParameterDef {
    name: "value",
    label: None,
    is_rest: false,
    // ...
}
```

### 4.2 Labeled, renamed local

```phalcom
move(to destination)
```

AST:

```rust
ParameterDef {
    name: "destination",
    label: Some("to"),
    is_rest: false,
    // ...
}
```

### 4.3 Labeled shorthand

```phalcom
from(record)
```

AST:

```rust
ParameterDef {
    name: "record",
    label: Some("record"),
    is_rest: false,
    // ...
}
```

### 4.4 Rest parameter

Preserve:

```phalcom
format(_ template, *values)
```

Rest is positional and final. It must not carry an external label.

### 4.5 Ordering rule

Preserve the existing selector invariant: positional parameters must precede labeled parameters.

Reject:

```phalcom
foo(label, _ value)
```

with the existing positional-after-label diagnostic if one exists, otherwise add a targeted parser error.

---

## 5. Reserve colon for types in declaration context

Old declaration label syntax:

```phalcom
foo(label:)
foo(label: local)
```

must no longer parse as a label declaration.

While type annotations are not implemented, produce a targeted migration error if the parser sees `:` where old label syntax would have been valid.

Preferred diagnostic:

```text
parameter declaration labels no longer use `:`;
write `label local`, or `label` when the external and local names are identical
```

Do not silently reinterpret:

```phalcom
foo(value: Thing)
```

as old label syntax. The point of this migration is to free this grammatical shape for typing.

Call-site argument labels remain:

```phalcom
foo(label: value)
```

Do not change `parse_arg_list` label behavior.

---

## 6. Ordinary setter declaration grammar

The final setter declaration is:

```phalcom
name=(put value) {
  ...
}
```

or expression-bodied equivalent if getters/setters support it.

The word `put` is fixed language syntax. It is not a user-selected external label.

Parser requirements:

1. Parse ordinary member name.
2. Parse `=`.
3. Require `(`.
4. Require identifier/token spelling `put`.
5. Parse one local identifier.
6. Require `)`.
7. Parse body.

Reject:

```phalcom
name=(value)
name=(set value)
name=(put:)
name=(put value, other)
```

unless an already-ratified grammar explicitly allows a shorthand. Do not invent one here.

AST should use the Task 1 `SetterDef.param` node.

The selector encoder already emits:

```text
name=(put)
```

from Task 1.

Surface assignment remains:

```phalcom
obj.name = value
```

No call-site syntax change is required.

---

## 7. Subscript getter grammar

Final forms:

```phalcom
[_ index] {
  ...
}

[_ index, default fallback] {
  ...
}
```

Selector identity:

```text
[_]
[_,default]
```

The bracket parser must use the selector-parameter grammar, not call-argument grammar and not the old declaration label grammar.

Examples:

```phalcom
[_ index, from start, to end]
```

would encode:

```text
[_,from,to]
```

if such an API exists.

---

## 8. Subscript setter grammar

Final form:

```phalcom
[_ index, default fallback]=(put value) {
  ...
}
```

This is represented as:

```rust
IndexMethodDef {
    params: vec![
        ParameterDef { name: "index", label: None, ... },
        ParameterDef { name: "fallback", label: Some("default"), ... },
    ],
    accessor: IndexAccessor::Set {
        put: ParameterDef {
            name: "value",
            // fixed assignment role; do not treat "put" as a user label
            ...
        },
    },
    // ...
}
```

Selector identity:

```text
[_,default]=(put)
```

Surface call:

```phalcom
obj[index, default: fallback] = value
```

The compiler must use:

```rust
SignatureKind::SubscriptSet(index_arg_count)
```

and pass index labels separately. It must **not** append `Some("put")` to the index-label vector before encoding.

Runtime stack order remains:

```text
receiver
index arg 1
...
index arg N
assigned value
```

---

## 9. Update duplicate selector detection

In `phalcom-core/src/compiler/lib/class_decl.rs`, current duplicate-member scanning computes selectors from member parameter labels.

Update subscript handling:

```rust
match idx.accessor {
    IndexAccessor::Get => {
        encode_selector(
            "",
            &labels,
            SignatureKind::SubscriptGet(labels.len() as u8),
        )
    }
    IndexAccessor::Set { .. } => {
        encode_selector(
            "",
            &labels,
            SignatureKind::SubscriptSet(labels.len() as u8),
        )
    }
}
```

Getter and setter with the same bracket slots are valid independent members:

```text
[_]
[_]=(put)
```

They must **not** trigger duplicate-selector detection.

Two getters with identical slots must collide. Two setters with identical slots must collide.

---

## 10. Update member compilation

Locate where `ClassMember::Index`, `SetterDef`, and normal methods compile into `MethodObject`.

Requirements:

- ordinary setter selector uses `SignatureKind::Setter`;
- subscript getter uses `SubscriptGet`;
- subscript setter uses `SubscriptSet`;
- setter local binding receives the assigned value at the final runtime argument slot;
- the number of compiler locals matches the actual runtime argument layout;
- selector labels come from external labels, while parameter-local names come from `ParameterDef.name`.

This is the central reason external label and local binding must remain separate in the AST.

---

## 11. Repository declaration migration

Migrate all selector-bearing declarations.

### 11.1 Positional parameters

Before:

```phalcom
indexOf(value) {
  ...
}
```

After:

```phalcom
indexOf(_ value) {
  ...
}
```

Before:

```phalcom
slice(start, end)
```

After:

```phalcom
slice(_ start, _ end)
```

If selector identity historically had two positionals, the migrated declaration must still encode `slice(_,_)`.

### 11.2 Same-name labeled parameters

Before:

```phalcom
from(record:)
```

or:

```phalcom
from(record: record)
```

After:

```phalcom
from(record)
```

Only perform this conversion where the old `record:` was a declaration label.

### 11.3 Renamed labeled parameters

Before:

```phalcom
lookup(default: fallback)
```

After:

```phalcom
lookup(default fallback)
```

### 11.4 Mixed

Before:

```phalcom
foo(value, default: fallback)
```

After:

```phalcom
foo(_ value, default fallback)
```

### 11.5 Constructors

Before:

```phalcom
@constructor
new(value)
```

After:

```phalcom
@constructor
new(_ value)
```

If the constructor intentionally exposes a label, use the labeled form.

### 11.6 Operators

For operator methods, positional operands remain explicit positional declarations.

Before, if applicable:

```phalcom
+(other)
```

After:

```phalcom
+(_ other)
```

Do not change operator call syntax.

### 11.7 Variadics

Before:

```phalcom
format(fmt, *args)
```

After:

```phalcom
format(_ fmt, *args)
```

Preserve existing fixed/minimum arity semantics.

---

## 12. Do not migrate block parameters

Examples like:

```phalcom
{ value => ... }
```

are lexical block bindings, not selector signatures.

Do not rewrite them to:

```phalcom
{ _ value => ... }
```

unless a separate language decision says so.

Similarly, destructuring bindings, `for` variables, `let`, `const`, exception handler bindings, and lambda/function parameters that do not define selector identity must retain their own grammar.

---

## 13. Subscript call-site migration

Normal subscript reads already use:

```phalcom
obj[index]
obj[index, default: fallback]
```

Keep them.

Normal writes already use:

```phalcom
obj[index] = value
```

Keep them.

Only declaration syntax and canonical selector identity change.

Any tests or reflection assertions expecting old write selectors such as:

```text
[_,put]
[put]
```

must be updated to:

```text
[_]=(put)
[]=(put)
```

Likewise old source declaration shapes such as:

```phalcom
[index, put:]
```

must become:

```phalcom
[_ index]=(put value)
```

---

## 14. Selector symbols and method references

Audit source/tests for selector literals referencing old setter/subscript identity.

Search:

```bash
rg -n '#[^"\s]*put|\\[.*put.*\\]|=\\(_\\)' phalcom-core phalcom-ast docs examples tools
```

Update selector literals:

```text
# old conceptual forms
#name=(_)
#[_,put]

# new canonical forms
#name=(put)
#[_]=(put)
```

Use whatever selector-symbol grammar the lexer currently supports. If bracket selector symbols are not currently surface syntax, update only internal string assertions and reflection fixtures.

---

## 15. Tests

### Parser tests

Add explicit tests for:

```phalcom
class C {
  a(_ x) {}
  b(label) {}
  c(label local) {}
  d(_ x, label y) {}
  e(_ x, *rest) {}
  value=(put newValue) {}
  [_ index] {}
  [_ index, default fallback] {}
  [_ index]=(put value) {}
  [_ index, default fallback]=(put value) {}
}
```

Verify exact AST labels/local names.

Add failures for:

```phalcom
class C {
  bad(label:) {}
  bad2(label: local) {}
  bad3(label, _ later) {}
  prop=(value) {}
  [_ index]=(set value) {}
}
```

### Selector tests

Assert exact identities:

```text
a(_)
b(label)
c(label)
d(_,label)
value=(put)
[_]
[_,default]
[_]=(put)
[_,default]=(put)
```

### Runtime dispatch tests

Define both:

```phalcom
[_ index]
[_ index]=(put value)
```

and verify read/write reach different bodies.

### Full repository test

Run:

```bash
cargo fmt
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

---

## 16. Migration audit commands

After migration:

```bash
# Find likely old declaration labels. Review every hit manually because calls still use label:
rg -n --glob '*.ph' '[A-Za-z_][A-Za-z0-9_]*:\s*[A-Za-z_][A-Za-z0-9_]*'

# Find method declarations likely missing explicit positional marker.
# This is heuristic: review results, do not automatically rewrite calls.
rg -n --glob '*.ph' '^\s*[A-Za-z_+*/%<>=!~-][A-Za-z0-9_+*/%<>=!~-]*\([^)]*\)\s*(\{|=>)'

# Find old subscript put-slot declarations/selector assertions.
rg -n --glob '*.{ph,rs,md,json}' '\[[^]]*put[^]]*\]'

# Find old setter canonical selector.
rg -n --glob '*.{rs,md,ph,json}' '=\\(_\\)'
```

The expected result is not necessarily zero for every broad heuristic, but every remaining hit must have an explained reason.

---

## 17. Acceptance criteria

- [ ] Bare selector parameter `name` now means same-name labeled parameter.
- [ ] Positional declaration parameters require `_ local`.
- [ ] Renamed labeled declaration parameters use `label local`.
- [ ] Declaration labels no longer use `:`.
- [ ] Call-site labels still use `label: value`.
- [ ] Blocks and other lexical parameter lists are not accidentally changed.
- [ ] Ordinary setters declare `=(put local)`.
- [ ] Subscript setters declare `[...] = (put local)` without embedding `put` in bracket parameters.
- [ ] Getter/setter selector identity round-trips correctly.
- [ ] Every canonical `.ph` source declaration in the repository has been migrated.
- [ ] No compatibility mode guesses whether `foo(x)` is old-positional or new-labeled.
- [ ] `cargo test --workspace` is green.

---

## 18. Commit guidance

Because this is a semantic flag day, keep the parser semantic switch and source migration in the same reviewable change set.

Suggested commit sequence within the branch:

```text
feat(parser): adopt external-label local-name declaration grammar
feat(parser): add canonical property and subscript setter declarations
refactor(core): migrate phalcom declarations to explicit positional slots
test(selectors): migrate setter and subscript selector identities
```

Do not merge only the parser change while leaving the repository source unmigrated.
