# Task 5 — Internal Namespaces, Primitive Migration, Float Cleanup, and `core.ph` Convergence

> **Repository:** `aureat/phalcom-lang`
> **Depends on:** Tasks 1–4
> **Must finish before:** Task 6
> **Primary objective:** Move genuinely internal runtime operations into the `_$selector` namespace, reserve `__field` for implementation storage, make internal access enforceable, fix Float's public selector kinds, and migrate `core.ph` from transitional primitive/private naming to the final language.

---

## 1. Final distinction to preserve

Do not equate "native" with "internal".

A primitive implemented in Rust may be a normal public language method/getter:

```text
Float#rounded
Object#class
Number#+(_)
```

Such methods should occupy their public selector directly.

Use `_$name` only when the operation is genuinely an implementation hook not intended as ordinary user API.

Use `__name` only for implementation-owned storage.

This distinction is the core of this task.

---

## 2. Files to edit

Primitive installation:

```text
phalcom-core/src/universe/primitives.rs
phalcom-core/src/primitive/mod.rs
phalcom-core/src/primitive/*.rs
```

Compiler privilege:

```text
phalcom-core/src/compiler/lib/mod.rs
phalcom-core/src/compiler/lib/expr.rs
phalcom-core/src/compiler/lib/class_decl.rs
```

Runtime authorization:

```text
phalcom-core/src/method/object.rs
phalcom-core/src/vm/send.rs
phalcom-core/src/vm/dispatch.rs
```

Core source:

```text
phalcom-core/core/core.ph
```

Float:

```text
phalcom-core/src/primitive/float.rs
phalcom-core/src/universe/primitives.rs
docs/spec/library/numbers/float-protocol.md
docs/pdr/0027-float-protocol-and-explicit-narrowing.md
```

Tests/fixtures across workspace.

---

## 3. Add a dedicated internal primitive registration API

Do not manually remember to set visibility every time.

If current public primitive registration is:

```rust
primitive!(vm, cls, "name", SignatureKind::Getter, native_fn);
```

add an explicit helper/macro such as:

```rust
primitive_internal!(
    vm,
    cls,
    "_$name",
    SignatureKind::Getter,
    native_fn
);
```

The helper must:

1. require selector spelling beginning `_$`;
2. build the MethodObject;
3. set `visibility = MemberVisibility::Internal`;
4. set appropriate holder/access owner;
5. install it through the same method table and cache invalidation machinery as ordinary methods.

Do not create a second hidden method table unless the existing architecture absolutely requires it. Internal is a visibility/access property, not a second dispatch mechanism.

Add a debug assertion or construction-time assertion that `primitive_internal!` selectors use the required prefix.

Public `primitive!` should not automatically reject `_$` until all migration sites are converted if that makes staging difficult, but after migration it should preferably assert that internal selector spellings go through the internal helper.

---

## 4. Define privileged compilation/execution context

Ordinary user source must not be able to author or call `_$selector` or `__field`.

The core/runtime module is privileged.

Reuse/factor existing core-module identity logic currently present in class compilation. Add a compiler helper conceptually like:

```rust
fn compiling_privileged_core(&self) -> bool
```

When compiling:

```phalcom
_$rawAt(index)
self._$rawAt(index)
other._$rawAt(index)
__storage
self.__storage
```

reject in an ordinary module with:

```text
internal.namespace_reserved
```

In core module source, permit them.

Runtime reflection must still enforce `Internal` visibility even if a user dynamically constructs the selector symbol. Lexical rejection alone is insufficient.

Nested blocks compiled inside privileged core members inherit internal privilege.

---

## 5. Inventory internal primitives

Before renaming, create an inventory.

Run:

```bash
rg -n 'primitive!.*"[^"]+_"' phalcom-core/src/universe/primitives.rs
rg -n 'primitive!.*"__[^"]+"' phalcom-core/src/universe/primitives.rs
rg -n 'primitive_static!.*"[^"]+_"' phalcom-core/src/universe/primitives.rs
rg -n '\.[A-Za-z][A-Za-z0-9]*_' phalcom-core/core/core.ph
rg -n '\.__[A-Za-z]' phalcom-core/core/core.ph
```

For each hit classify it as:

```text
A. public primitive implemented natively
B. genuinely internal representation/runtime operation
C. accidental transitional wrapper/duplicate
```

Only category B migrates to `_$`.

Do not blindly rename based on trailing underscore.

---

## 6. Expected internal primitive migrations

Based on the currently inspected repository, review at least these families:

```text
Class:
new_                 -> _$new

List:
length_
at_
set_
push_

Bytes:
size_
at_
set_
fill_
... raw operations

Map:
size_
get_
put_
has_
remove_
keyAt_
valueAt_

Tuple:
size_
at_
positionalSize_
labelAt_
... raw tuple access

String:
byteCount_
byteAt_
slice_
... raw UTF-8 helpers

Object/runtime attribute/invariant hooks:
__invariantEnter
__invariantExit
__attributes
__attach
__freezeAttributes
```

The exact list must come from the actual primitive table on the implementation branch. Do not use this list as a substitute for inventory.

Rename method-like double-underscore hooks to `_$...` because `__name` is now implementation-field syntax.

Examples:

```text
__invariantEnter      -> _$invariantEnter
__invariantExit       -> _$invariantExit
__attributes          -> _$attributes
__attach              -> _$attach
__freezeAttributes    -> _$freezeAttributes
```

Update compiler-generated code/decorator expansion that invokes these hooks.

---

## 7. Do not mechanically rename source fields to `__`

A source-level field can be known to native code and still remain a normal `_field`.

For example, if `Error` declares/uses:

```phalcom
_message
_kind
_cause
```

and native primitives understand their layout, that does not automatically make them `__message`, `__kind`, etc.

Use `__field` only when the storage itself is owned by compiler/VM implementation machinery rather than the source-level object model.

If no current source-level implementation field needs to be exposed, it is acceptable for the new `__field` namespace to have few or zero immediate `core.ph` uses. The namespace is reserved for the correct semantic category, not created to force migrations.

---

## 8. Migrate `core.ph` primitive wrappers

Change genuinely internal calls.

Before:

```phalcom
class Class {
  new() => self.new_()
}
```

After Task 2 declaration grammar and Task 3 implicit self:

```phalcom
class Class {
  new() => _$new()
}
```

Before:

```phalcom
size => self.byteCount_
```

After:

```phalcom
size => _$byteCount
```

Before:

```phalcom
at(_ index) => self.at_(index)
```

After:

```phalcom
at(_ index) => _$at(index)
```

Cross-object internal calls in privileged core source remain explicit about the receiver:

Before:

```phalcom
_map.keyAt_(cursor)
```

After:

```phalcom
_map._$keyAt(cursor)
```

Do not incorrectly rewrite this to bare `_$keyAt(cursor)`, which would target `self`, not `_map`.

---

## 9. Migrate old underscore helper methods to visibility decorators

Search core and `.ph` sources:

```bash
rg -n --glob '*.ph' '^\s*_[A-Za-z][A-Za-z0-9_]*\s*(\(|=>|\{)' .
```

Any old method-like `_helper` declaration must become an ordinary selector plus explicit visibility.

Typical migration:

Before:

```phalcom
_findLabel(_ symbol) {
  ...
}

_access(_ key) {
  let idx = self._findLabel(key)
  ...
}

get(_ key) => self._access(key)
```

After:

```phalcom
@private
findLabel(_ symbol) {
  ...
}

@private
access(_ key) {
  let idx = findLabel(symbol)
  ...
}

get(_ key) => access(key)
```

Choose `@protected` only when subclasses are intentionally allowed to call the helper. Default implementation helpers should be `@private`.

Do not use `_$` for source-level private helpers. `_$` is runtime/core implementation namespace, not a replacement for `@private`.

---

## 10. Fix Float completely

### 10.1 Public API decision

These eleven Float-specific operations are public getters:

```text
abs
sign
floor
ceil
truncated
rounded
toIntExact
isInteger
isNaN
isFinite
isInfinite
```

They must be installed directly under those getter selectors.

### 10.2 Primitive registration

Change current zero-argument method registrations:

```rust
primitive!(vm, float_cls, "abs", SignatureKind::Method(0), float_abs);
primitive!(vm, float_cls, "sign", SignatureKind::Method(0), float_sign);
// ...
```

to:

```rust
primitive!(vm, float_cls, "abs", SignatureKind::Getter, float_abs);
primitive!(vm, float_cls, "sign", SignatureKind::Getter, float_sign);
primitive!(vm, float_cls, "floor", SignatureKind::Getter, float_floor);
primitive!(vm, float_cls, "ceil", SignatureKind::Getter, float_ceil);
primitive!(vm, float_cls, "truncated", SignatureKind::Getter, float_truncated);
primitive!(vm, float_cls, "rounded", SignatureKind::Getter, float_rounded);
primitive!(vm, float_cls, "toIntExact", SignatureKind::Getter, float_to_int_exact);
primitive!(vm, float_cls, "isInteger", SignatureKind::Getter, float_is_integer);
primitive!(vm, float_cls, "isNaN", SignatureKind::Getter, float_is_nan);
primitive!(vm, float_cls, "isFinite", SignatureKind::Getter, float_is_finite);
primitive!(vm, float_cls, "isInfinite", SignatureKind::Getter, float_is_infinite);
```

Do not create `_$abs`, `_$rounded`, etc. The native implementation *is* the public operation.

### 10.3 Remove Float wrappers from `core.ph`

Delete forwarding getters such as:

```phalcom
abs => self.abs()
rounded => self.rounded()
```

Delete recursive getters such as:

```phalcom
isInteger => self.isInteger
isNaN => self.isNaN
isFinite => self.isFinite
isInfinite => self.isInfinite
```

Target:

```phalcom
class Float is Number {}
```

unless unrelated derived Float behavior remains.

### 10.4 Tests

Positive property-style tests:

```phalcom
(-3.5).abs
3.5.floor
3.5.ceil
3.5.truncated
3.5.rounded
3.0.toIntExact
3.0.isInteger
NaN.isNaN
```

Use actual existing constructors/constants for NaN/infinity in the repository.

Negative selector-shape tests should verify that the native floor does not provide accidental:

```text
abs()
rounded()
isFinite()
```

Do not require these sends to fail if user code deliberately defines such methods separately; the requirement is that the primitive table does not install them.

### 10.5 Numeric semantics are out of scope

Do not change rounding algorithm, NaN semantics, narrowing errors, signed zero, or power/remainder behavior in this task. If tests reveal an existing semantic disagreement between PDR and current spec, record it separately rather than mixing it into selector cleanup.

---

## 11. Update compiler-generated runtime hook names

Search attribute/invariant code:

```bash
rg -n "__invariant|__attributes|__attach|__freeze" phalcom-core docs
```

Update emitted selector names to `_$...`.

Because generated/compiler code may be privileged without originating from literal `core.ph`, make sure its compiled closures carry `internal_privilege = true` where needed.

Do not rely on the source spelling alone to grant privilege.

---

## 12. Reflection and dynamic internal-access tests

Ordinary user code must fail even if it constructs the internal selector dynamically.

Examples conceptually:

```phalcom
obj.perform(#"_$rawAt(_)", ...)
```

or whatever quoted/full selector-symbol API exists.

Test through the actual reflective selector-construction mechanism supported by the repository.

Expected: internal access error, not successful invocation and not `doesNotUnderstand` if the selector exists.

Privileged core code should succeed through ordinary dispatch.

---

## 13. Remove trailing-underscore pseudo-privacy from generated metadata

Generated core tables, method lists, docs, tests, and selector snapshots may contain old names.

Search:

```bash
rg -n --glob '*.{rs,json,md,ph}' '\b[A-Za-z][A-Za-z0-9]*_\b' .
rg -n --glob '*.{rs,json,md,ph}' '__[A-Za-z][A-Za-z0-9]*' .
```

Review every hit. Some Rust function names such as `float_to_int_exact` are implementation identifiers and do not need migration. The audit concerns **Phalcom selector spellings** and `.ph` member names, not Rust naming conventions.

---

## 14. Core source style convergence

After semantic migrations pass tests, use implicit self idiomatically.

Preferred:

```phalcom
size => _$size
```

rather than:

```phalcom
size => self._$size
```

Preferred private helper use:

```phalcom
@private
normalize(_ value) {
  ...
}

publicMethod(_ value) => normalize(value)
```

Keep explicit receiver where the receiver is not self or where clarity matters:

```phalcom
_map._$keyAt(cursor)
other.publicMethod(value)
```

Do not mechanically delete every `self.`. The purpose is to demonstrate the final model, not minimize characters.

---

## 15. Tests and commands

Targeted:

```bash
cargo test -p phalcom-core float
cargo test -p phalcom-core primitive
cargo test -p phalcom-core universe
cargo test -p phalcom-core compiler
```

Bootstrap/core tests:

```bash
cargo test -p phalcom-core core
```

Use the actual test filter names available on the branch.

Full:

```bash
cargo fmt
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

Audit:

```bash
rg -n --glob '*.ph' '^\s*_[A-Za-z][A-Za-z0-9_]*\s*(\(|=>|\{)' .
rg -n 'primitive!.*"__|primitive!.*"[A-Za-z][A-Za-z0-9]*_"' phalcom-core/src/universe/primitives.rs
rg -n --glob '*.ph' '\.[A-Za-z][A-Za-z0-9]*_' .
```

Every remaining selector-level hit must be intentional and documented.

---

## 16. Acceptance criteria

- [ ] `_$name` is the only implementation-selector namespace.
- [ ] `__name` is not used for methods/getters/setters.
- [ ] Internal primitive registration sets `Internal` visibility automatically.
- [ ] Ordinary user modules cannot author internal selectors/implementation fields.
- [ ] Dynamic/reflection-based user calls cannot bypass internal visibility.
- [ ] Core/runtime privileged code can use internal selectors.
- [ ] Source-level private helpers use `@private`, not `_helper` or `_$helper`.
- [ ] Cross-object core internal sends use explicit receiver + `_$selector`.
- [ ] No primitive is renamed internal merely because it is implemented in Rust.
- [ ] All eleven Float protocol operations are public native getters.
- [ ] Float forwarding/recursive wrappers are removed from `core.ph`.
- [ ] `core.ph` compiles and executes after the migration.
- [ ] Full workspace tests pass.

---

## 17. Commit guidance

Suggested commits:

```text
feat(runtime): add internal primitive selector registration
refactor(core): migrate raw primitive selectors to _$ namespace
refactor(core): replace underscore helpers with explicit visibility
fix(float): install float protocol directly as native getters
refactor(core): remove float forwarding wrappers
```
