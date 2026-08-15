# Task 3 — Implicit `self`, Field Semantics, and Unqualified Name Resolution

> **Repository:** `aureat/phalcom-lang`  
> **Depends on:** Tasks 1–2  
> **Must finish before:** Tasks 4–6  
> **Primary objective:** Add implicit receiver capability for ordinary selectors, source fields, implementation fields, and implementation selectors while preserving lexical shadowing and global/module resolution.

---

## 1. Final semantic rules

Implement these rules exactly.

### 1.1 Namespace-directed forms

These forms never participate in ordinary-name lookup:

```text
_field       direct source field on self
__field      direct implementation field on self
_$foo        implementation getter send to self
_$foo(...)   implementation method send to self
```

Examples:

```phalcom
_items
_items.map(fn)
_callback()
__storage.read()
_$size
_$at(index)
```

They correspond conceptually to:

```phalcom
self._items
self._items.map(fn)
self._callback()
self.__storage.read()
self._$size
self._$at(index)
```

The parser/compiler must not decide whether `_items` is a field by examining the token after it. `_items.map(...)`, `_items.call(...)`, `_items(...)`, and `_items[index]` all begin with the same field-primary expression, after which normal postfix parsing continues.

### 1.2 Ordinary-name resolution

Inside a member or any nested block lexically originating inside that member:

```text
1. current lexical local / parameter
2. enclosing upvalue
3. statically known module/global/import/core binding
4. implicit send to self
```

Outside member lexical context, preserve existing top-level/global behavior.

This order is critical. A local named `map` must shadow a method `map`; a known global named `List` must remain the global `List`, not become `self.List`.

### 1.3 Explicit `self`

Explicit `self` remains legal and is the disambiguation mechanism:

```phalcom
self.foo
self.foo(value)
self.foo = value
self._field
self.__field
self._$primitive(value)
```

Implicit `self` is convenience, not a replacement.

---

## 2. Files to edit

Primary:

```text
phalcom-ast/src/ast.rs
phalcom-ast/src/parser.rs
phalcom-core/src/compiler/lib/expr.rs
phalcom-core/src/compiler/lib/mod.rs
phalcom-core/src/compiler/lib/scope.rs
phalcom-core/src/compiler/lib/state.rs
phalcom-core/src/compiler/lib/class_decl.rs
```

Potential VM/compiler helpers:

```text
phalcom-core/src/vm/dispatch.rs
phalcom-core/src/module.rs
```

Use existing module/global resolution helpers where possible. Do not create a second global-name model only for implicit `self`.

Tests likely live in compiler/parser modules and `.ph` integration fixtures. Discover exact locations:

```bash
rg -n "UndefinedVariable|resolve_local|resolve_upvalue|GetGlobal|SetGlobal|Expr::Var|Expr::Call" phalcom-core/src/compiler
rg -n "implicit self|self\\." phalcom-core/tests phalcom-ast/tests docs/forge
```

---

## 3. Add an explicit unqualified-call AST form

The current parser/compiler behavior can interpret `foo(x)` as loading `foo` and then calling the resulting callable. That is insufficient once `foo(x)` may mean either a local callable or an implicit send to `self`.

Do not resolve this ambiguity in the parser by inspecting currently declared methods. Phalcom is dynamic and methods can be replaced/added reflectively.

Add an AST form that preserves the syntactic fact that there was no explicit receiver.

Recommended:

```rust
pub struct UnqualifiedCallExpr {
    pub name: String,
    pub args: Vec<Argument>,
    pub range: SourceRange,
}

pub enum Expr {
    // ...
    UnqualifiedCall(Box<UnqualifiedCallExpr>),
    // ...
}
```

Parser behavior:

```phalcom
foo(x)
```

becomes `Expr::UnqualifiedCall`.

Explicit receiver:

```phalcom
obj.foo(x)
```

remains `Expr::MethodCall`.

A parenthesized/local expression call such as:

```phalcom
(blockExpr)(x)
```

continues through the existing generic callable path if supported.

---

## 4. Track member lexical context through nested blocks

Implicit `self` must work in:

```phalcom
class C {
  helper(_ x) { ... }

  run(_ xs) {
    return xs.map { x =>
      helper(x)
    }
  }
}
```

The nested block does not have `self` in slot 0 in the same way a method does; current compiler architecture already handles block access to method `self` through capture/upvalue behavior.

Add explicit compiler state recording whether the current lexical body has a member receiver available.

Recommended conceptual state:

```rust
struct AccessContext {
    lexical_class: Option<ClassId or ClassKey>,
    has_self: bool,
    internal_privilege: bool, // Task 5 will use this
}
```

Do not necessarily introduce this exact struct if the compiler already has enough state fields, but nested blocks must inherit the parent member's receiver/access context.

A top-level block/function that does not originate in a class member must not invent an implicit `self`.

---

## 5. Implement ordinary bare getter fallback

Current `Expr::Var` resolution is approximately:

```text
local -> upvalue -> global
```

Change it inside member lexical context to:

```text
local -> upvalue -> known global -> implicit self getter
```

The phrase **known global** is important.

Do not treat every unresolved name as a global merely because `GetGlobal` can be emitted. If you do that, implicit self will never run.

Implement a helper such as:

```rust
fn resolves_known_global(&self, name: Symbol) -> bool
```

It should return true for names known from:

- current compilation-unit global bindings;
- imported bindings;
- already existing globals in the current module;
- core-module bindings visible through the language's existing fallback rules.

Reuse existing `global_bindings`, `import_bindings`, module object state, and core-module resolution. Do not create a speculative runtime lookup instruction that tries global then self.

Pseudo-code:

```rust
if let Some(slot) = self.resolve_local(name_sym) {
    emit GetLocal
} else if let Some(upvalue) = self.resolve_upvalue(name_sym) {
    emit GetUpvalue
} else if self.resolves_known_global(name_sym) {
    emit GetGlobal
} else if self.has_implicit_self() {
    emit self
    emit Invoke(getter selector)
} else {
    preserve existing undefined/global behavior
}
```

The implicit selector for bare `foo` is getter selector `foo`, not zero-argument method `foo()`.

---

## 6. Implement unqualified-call resolution

For:

```phalcom
foo(arg1, label: arg2)
```

compile according to this order:

### 6.1 Local/upvalue/global callable exists

If `foo` resolves to a lexical or known-global value, compile the value and invoke its callable protocol exactly as the current unqualified call path does.

A local callable must win:

```phalcom
let foo = { x => x + 1 }
foo(3)
```

must not dispatch `self.foo(_)`.

### 6.2 No binding exists, member context exists

Compile as:

```phalcom
self.foo(arg1, label: arg2)
```

using the exact same selector encoder and ordinary `Invoke` bytecode path as an explicit receiver.

Do not add a separate "implicit send" bytecode instruction.

### 6.3 No binding and no member context

Use the current top-level behavior. If current language treats this as undefined variable/callable, preserve that behavior.

---

## 7. Implement implicit setter fallback

For:

```phalcom
name = value
```

resolution must be:

```text
mutable current local -> SetLocal
immutable current local -> compile error
mutable upvalue -> SetUpvalue
immutable upvalue -> compile error
known global -> existing global assignment rules
otherwise in member context -> self.name = value
otherwise -> existing top-level/global behavior
```

The implicit setter selector is canonical:

```text
name=(put)
```

Do not allow a `const` local/upvalue/global to fall through into a setter. A lexical binding shadows the member even when the attempted operation is illegal.

Example:

```phalcom
class C {
  name=(put value) { ... }

  f() {
    const name = "local"
    name = "x"
  }
}
```

must report assignment-to-immutable-local, not invoke `self.name=(put)`.

---

## 8. Field primary parsing and postfix behavior

With Task 1 tokens, parser primary behavior must be direct:

```rust
Token::FieldIdentifier(name) => Expr::Field {
    value: name,
    kind: FieldKind::Source,
    range,
}

Token::ImplementationFieldIdentifier(name) => Expr::Field {
    value: name,
    kind: FieldKind::Implementation,
    range,
}
```

After returning that primary expression, existing postfix parsing handles:

```phalcom
_field.map(fn)
_field.call(x)
_field(x)
_field[index]
_field?.foo
```

Do not special-case `.map`, `.call`, `(`, `[`, or any other follower.

Add regression tests specifically because the old heuristic only recognized field declarations/references under limited lookahead conditions.

---

## 9. Explicit field receiver restrictions

The language model treats fields as direct storage rather than public selectors.

Allow:

```phalcom
self._field
self.__field
```

Reject:

```phalcom
other._field
other.__field
```

The parser can reject foreign-field syntax early because the token after `.` is structurally a field token. If preserving parser generality is preferable, build an AST node and emit a dedicated compile diagnostic. Either way, do not turn `other._field` into a method send.

Recommended diagnostic:

```text
field.foreign_receiver:
fields are direct receiver state; only `self._field` is valid
```

The internal implementation selector syntax is different:

```phalcom
other._$rawAt(index)
```

may be syntactically valid in privileged core/runtime source because `_$rawAt` is a selector, not storage.

---

## 10. Implementation field privilege hook

Task 5 will enforce core/runtime-only usage. Add or preserve enough AST/compiler distinction now so `__field` can be rejected based on compilation-unit privilege later.

Do not merge `__field` into ordinary source-field layout identity just because both use slots. If the layout structure stores a single slot map, maintain an accompanying set/classification indicating implementation-owned fields.

---

## 11. Interaction with class-side methods

Implicit receiver applies in `@class` methods/getters/setters too.

Example:

```phalcom
class Registry {
  @class
  _items = List.new()

  @class
  add(_ item) {
    _items.add(item)
  }
}
```

`_items` resolves as the class-side field using the existing static-field access rules.

Ordinary unresolved method:

```phalcom
@class
run() {
  helper()
}
```

means:

```phalcom
self.helper()
```

where `self` is the class object/metaclass-side receiver as already modeled by the compiler.

Do not special-case class-side implicit receiver differently from explicit `self`.

---

## 12. Interaction with constructors

Constructors use the same implicit receiver rules.

Example:

```phalcom
@constructor
new(_ value) {
  _value = value
  validate()
}
```

`_value` is direct field assignment; `validate()` is implicit send to self unless shadowed.

Preserve existing constructor restrictions on `const` field writes and initializer compilation.

---

## 13. Do not add implicit subscript syntax

Do **not** make:

```phalcom
[index]
[index] = value
```

mean:

```phalcom
self[index]
self[index] = value
```

Bracket expressions have other syntactic roles and the ambiguity is unnecessary.

Explicit receiver remains required:

```phalcom
self[index]
self[index] = value
```

---

## 14. Compiler helper design

Centralize resolution to avoid different behavior for getter/call/assignment.

Suggested helpers:

```rust
enum BareNameResolution {
    Local(u16),
    Upvalue(u16),
    Global(Symbol),
    ImplicitSelf,
    Unresolved,
}

fn resolve_bare_name(&mut self, name: Symbol) -> BareNameResolution
```

For assignment, add mutability information or use existing mutable-resolution helpers.

Do not duplicate local/upvalue/global search logic in three unrelated match arms.

The exact helper type may differ, but resolution order must be one shared invariant.

---

## 15. Required tests

### 15.1 Field postfix regression

```phalcom
class C {
  _items
  _callback

  test() {
    _items.map { x => x }
    _callback.call()
    _callback()
  }
}
```

Verify these parse as field primary + postfix, never underscore methods.

### 15.2 Implicit getter

```phalcom
class C {
  value => 42
  test() => value
}
```

Equals explicit `self.value`.

### 15.3 Implicit method

```phalcom
class C {
  inc(_ x) => x + 1
  test() => inc(41)
}
```

Returns `42`.

### 15.4 Local shadows method

```phalcom
class C {
  run(_ x) => 100

  test() {
    let run = { x => x + 1 }
    return run(41)
  }
}
```

Must invoke local callable.

### 15.5 Upvalue shadows method

Nested block captures outer local with same name as a method; local/upvalue wins.

### 15.6 Known global shadows getter

Use `List`, `Map`, or a test-defined top-level binding and verify bare reference remains global.

### 15.7 Implicit setter

```phalcom
class C {
  _value
  value=(put v) { _value = v }

  test() {
    value = 42
    return _value
  }
}
```

### 15.8 Immutable local does not fall through

Verify `const value` reassignment remains a const error even if class has a setter `value=(put)`.

### 15.9 Nested block inherits self

```phalcom
class C {
  helper(_ x) => x + 1
  test() {
    return { => helper(41) }.call()
  }
}
```

### 15.10 Class-side implicit receiver

Add an `@class` fixture.

### 15.11 Foreign field rejection

```phalcom
other._field
other.__field
```

must fail with the intended diagnostic.

---

## 16. Commands

Targeted:

```bash
cargo test -p phalcom-ast
cargo test -p phalcom-core compiler
```

Full:

```bash
cargo fmt
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

Audit for remaining follower-token field heuristics:

```bash
rg -n "peek_next.*_|starts_with\\(\"_\"\\)|starts_with\\('_'\\)" phalcom-ast/src/parser.rs phalcom-core/src/compiler
```

Any remaining prefix check must have a reason unrelated to distinguishing field versus method namespace.

---

## 17. Acceptance criteria

- [ ] `_field` is always a self-field reference in expression position.
- [ ] `_field.map(...)`, `_field.call(...)`, `_field(...)`, and `_field[...]` all work through normal postfix parsing.
- [ ] `__field` is structurally an implementation field, not a selector.
- [ ] `_$foo` is structurally an implementation selector and supports implicit `self`.
- [ ] Ordinary bare getter names use local → upvalue → known global → implicit self resolution.
- [ ] Ordinary bare calls preserve local/global callable semantics and fall back to a self method send only when no binding exists.
- [ ] Bare assignments preserve mutability errors before implicit setter fallback.
- [ ] Nested blocks inherit the enclosing member's receiver context.
- [ ] Class-side members receive the same implicit-self behavior.
- [ ] Explicit `self` remains valid.
- [ ] Foreign direct field access is rejected.
- [ ] No implicit-self subscript syntax is introduced.
- [ ] Full workspace tests pass.

---

## 18. Commit guidance

Suggested commits:

```text
feat(compiler): add implicit self bare-name resolution
feat(parser): make field namespaces unconditional primaries
test(language): cover implicit self shadowing and field postfix access
```
