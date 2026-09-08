# Task 4 — `@private`, `@protected`, and Uniform Runtime Access Control

> **Repository:** `aureat/phalcom-lang`
> **Depends on:** Tasks 1–3
> **Must finish before:** Tasks 5–6
> **Primary objective:** Implement real class-private and class-family-private method visibility, enforced uniformly across ordinary dispatch, reflection, method references, bound methods, `super`, and inline-cache paths.

---

## 1. Semantic contract

The language now has explicit visibility instead of underscore-naming conventions.

```phalcom
class A {
  @private
  secret() {
    ...
  }

  @protected
  helper(_ value) {
    ...
  }
}
```

Definitions:

```text
@private
    callable only from code lexically belonging to the defining source class

@protected
    callable from code lexically belonging to the defining source class
    or any subclass in that class family
```

Visibility applies to:

- methods;
- getters;
- setters;
- subscript getters;
- subscript setters;
- class-side variants of those members.

Visibility does not depend on explicit versus implicit `self`.

Fields do not use `@private`/`@protected`; direct-field semantics already control access.

Implementation selectors `_$name` use `Internal` visibility and are handled in Task 5.

---

## 2. Core design requirement: lexical caller, not receiver class

Access checks must use the class where the calling code was lexically defined.

This is required for nested blocks:

```phalcom
class A {
  @private
  secret() => 1

  test() {
    return { => secret() }.call()
  }
}
```

The block is still lexically code belonging to `A`; it must be allowed.

Likewise, a subclass method calling an inherited `@private` member must fail even though `self` at runtime is an instance of the subclass.

Do not authorize based only on `receiver.class`.

---

## 3. Files to edit

AST/attributes:

```text
phalcom-ast/src/ast.rs
phalcom-core/src/compiler/attributes.rs
phalcom-core/src/compiler/lib/class_decl.rs
phalcom-core/src/compiler/lib/mod.rs
phalcom-core/src/compiler/lib/state.rs
```

Runtime method/frame metadata:

```text
phalcom-core/src/method/object.rs
phalcom-core/src/frame.rs
phalcom-core/src/callable.rs                 # if closure metadata lives here
phalcom-core/src/heap/...                     # closure/method structs if needed
```

Dispatch/reflection:

```text
phalcom-core/src/vm/send.rs
phalcom-core/src/vm/dispatch.rs
phalcom-core/src/primitive/object.rs
phalcom-core/src/primitive/method.rs
phalcom-core/src/primitive/class.rs
phalcom-core/src/heap/class.rs
phalcom-core/src/value.rs
```

Inline cache:

```text
phalcom-core/src/chunk.rs
phalcom-core/src/vm/dispatch.rs
```

Search all method lookup/invocation paths:

```bash
rg -n "lookup_method|call_method|send_dynamic|methodFor|respondsTo|perform|invokeOn|BoundMethod|MakeFamily|SuperSend|Bytecode::Invoke" phalcom-core/src
```

Every path must be considered. Do not fix only ordinary `Invoke`.

---

## 4. Attribute legality and lowering

### 4.1 Builtin attributes

Task 1 should have introduced builtin names. Implement full legality now.

`@private` and `@protected`:

Allowed on:

```text
Method
Getter
Setter
Index getter
Index setter
```

Also allowed with `@class`.

Reject on:

```text
Field
Class declaration
Variant arm
Standalone invariant
```

unless another existing decorator system deliberately permits broader targets. The critical rule is that fields do not use these visibility decorators.

### 4.2 Mutual exclusion

Reject:

```phalcom
@private
@protected
foo() {}
```

with a dedicated compile diagnostic:

```text
member.visibility_conflict
```

### 4.3 Internal conflict

If a `_$selector` is authored in privileged core source, reject `@private`/`@protected` on it. Internal selector visibility is a separate stronger domain.

### 4.4 Constructors

Do not invent constructor visibility semantics in this task unless constructor lowering already makes the behavior completely unambiguous. Recommended migration scope: reject `@private`/`@protected` on `@constructor` with a targeted diagnostic and leave constructor visibility for a dedicated decision.

This avoids accidentally making only the hidden initializer private while leaving the class-side factory public, or vice versa.

---

## 5. Runtime method metadata

Task 1 should have added:

```rust
pub enum MemberVisibility {
    Public,
    Private,
    Protected,
    Internal,
}
```

and fields on `MethodObject`:

```rust
pub visibility: MemberVisibility,
pub access_owner: Option<ClassId>,
```

Now populate them correctly.

### 5.1 `holder` versus `access_owner`

Do not conflate these.

For an instance method on source class `A`:

```text
holder       = A
access_owner = A
```

For a class-side method on `A`, runtime lookup may place the method on a metaclass/class-side holder, but access semantics still mean "private/protected to source class A":

```text
holder       = A's class-side holder/metaclass
access_owner = A
```

This distinction is mandatory.

### 5.2 Generated members

Compiler-generated methods created from attributes must receive intentional visibility.

Default generated public API remains `Public`.

Compiler-internal helpers that are not source-visible may later become `Internal`, but do not broadly relabel generated code without reason.

---

## 6. Carry lexical access context in compiled closures/frames

The runtime must know the lexical source class of the currently executing member.

Recommended metadata on compiled callable/closure:

```rust
pub struct AccessContext {
    pub lexical_class: Option<ClassId>,
    pub internal_privilege: bool,
}
```

A method compiled from source class `A` has:

```text
lexical_class = Some(A)
```

A nested block compiled inside that method inherits the same access context.

A module-level/top-level closure has:

```text
lexical_class = None
```

Task 5 will use `internal_privilege`.

If putting the metadata on `ClosureObject` is simpler than `CallFrame`, store it there and copy/reference it when a frame is pushed. The runtime access check must be able to retrieve it cheaply for the current caller.

Do not derive lexical owner by inspecting the call stack's receiver.

---

## 7. Implement one authorization function

Create one shared runtime helper. Suggested conceptual API:

```rust
fn authorize_method_access(
    &self,
    method: ObjRef,
    caller: AccessContext,
) -> Result<(), RuntimeError>
```

or:

```rust
fn method_is_accessible(
    &self,
    method: ObjRef,
    caller: AccessContext,
) -> bool
```

with a single error-building wrapper.

Rules:

```rust
match method.visibility {
    MemberVisibility::Public => true,

    MemberVisibility::Private => {
        caller.lexical_class == method.access_owner
    }

    MemberVisibility::Protected => {
        caller.lexical_class == method.access_owner
            || caller.lexical_class
                .is_some_and(|caller_class| {
                    self.is_subclass_of(caller_class, method.access_owner.unwrap())
                })
    }

    MemberVisibility::Internal => {
        caller.internal_privilege
    }
}
```

Use the repository's canonical class inheritance relation. Do not implement subclass checks by comparing names.

---

## 8. Unauthorized access is not `doesNotUnderstand`

A selector can exist and still be inaccessible.

For:

```phalcom
a.secret()
```

where `secret()` exists but is private, do not route to `doesNotUnderstand`.

Return a dedicated runtime access error, for example:

```text
member.private_access
member.protected_access
internal.selector_access
```

This preserves the distinction:

```text
missing selector        -> doesNotUnderstand
existing inaccessible   -> access violation
```

Proxy/missing-method semantics must not hide access-control failures.

---

## 9. Ordinary `Invoke` path

Find `Bytecode::Invoke` handling in the VM dispatch loop.

Current conceptual flow:

```text
resolve selector
lookup method
possibly use inline cache
call method
```

Change to:

```text
resolve selector
lookup/cached method
authorize against current lexical caller
call method
```

Authorization happens on **every invocation**, including an inline-cache hit.

Do not authorize only when filling the cache. A cache can be shared/reused in contexts where access assumptions may differ, and a later refactor could otherwise create a bypass.

---

## 10. `super` sends

A `super` send changes lookup start, not lexical access owner.

Example:

```phalcom
class Base {
  @protected
  helper() => 1
}

class Child is Base {
  run() => super.helper()
}
```

must succeed.

Private:

```phalcom
class Base {
  @private
  helper() => 1
}

class Child is Base {
  run() => super.helper()
}
```

must fail.

The caller lexical class is `Child` in both cases.

Do not grant private access merely because `super` starts lookup in `Base`.

---

## 11. Reflective dispatch

### 11.1 `perform`

`obj.perform(selector, ...)` must enforce the same visibility as direct source send.

Do not let a user bypass:

```phalcom
obj.secret()
```

by writing:

```phalcom
obj.perform(#secret())
```

Route reflective invocation through the same authorization function.

### 11.2 `send_dynamic`

Current runtime has a shared dynamic-send helper. Integrate authorization into that path before `call_method`.

### 11.3 `methodFor`

A caller must not retrieve an unrestricted private/protected/internal `MethodObject` and invoke it later.

Recommended behavior: if method exists but is not accessible from current context, return the existing absence value used for a failed `methodFor`, or raise an access error if the current API is error-based. Pick the behavior most consistent with current `methodFor` contract and document it.

Do **not** return an invokable object that bypasses later authorization.

### 11.4 `respondsTo`

Recommended semantics:

```text
respondsTo(selector) == selector exists and is callable from current access context
```

Thus external code sees private/protected/internal methods as unavailable.

If the current method is intentionally defined as physical-table introspection rather than callable-protocol introspection, preserve that only with explicit spec justification. The safer language-level behavior is access-context aware.

### 11.5 `Behavior.methods`

This may continue enumerating physical own selectors. Visibility is access control, not secrecy. Do not filter it unless the specification says so.

---

## 12. Method objects, bound methods, and `invokeOn`

Inspect:

```text
phalcom-core/src/primitive/method.rs
```

A user must not obtain a `Method` object through a legal path and then call `invokeOn` to bypass visibility.

Either:

1. authorize when the Method is retrieved/bound; and/or
2. authorize again at invocation.

Preferred: authorize at invocation as the final security boundary, even if retrieval is also filtered.

A MethodObject's `access_owner` metadata must survive binding.

---

## 13. Method references / families

Open or pinned method references must respect access.

Cases:

```phalcom
obj::secret
obj::#secret()
```

If reference creation resolves a concrete method immediately, reject inaccessible method reference creation.

If an open family defers resolution until invocation, enforce access when the family resolves/calls a concrete target.

Do not permit capturing a private method externally and invoking it later.

---

## 14. Inline-cache and override tests

The repository has inline cache logic and tests around method replacement/override. Add access-control coverage around warm caches.

Required scenario:

1. Define public method.
2. Call enough to populate cache.
3. Replace or otherwise resolve to a private/protected method through allowed reflective mutation if supported.
4. Ensure external cached call does not bypass visibility.

Or, if visibility cannot mutate after method creation, create calls from two lexical caller classes sharing the same receiver/selector cache shape and verify authorization differs appropriately.

The exact test should target the real cache architecture, not a synthetic helper.

---

## 15. Required visibility tests

### Private, same class

```phalcom
class A {
  @private
  secret() => 7

  own() => secret()
}
```

`A.new().own()` succeeds.

### Private, explicit self

```phalcom
own() => self.secret()
```

also succeeds.

### Private, external

```phalcom
A.new().secret()
```

fails with private access error.

### Private, subclass

```phalcom
class B is A {
  attempt() => secret()
}
```

fails.

### Private, nested block

```phalcom
class A {
  @private
  secret() => 7

  own() => { => secret() }.call()
}
```

succeeds.

### Protected, subclass

Subclass call succeeds.

### Protected, unrelated class

Fails.

### Protected, external

Fails.

### Protected through `super`

Succeeds from subclass.

### Private through `super`

Fails from subclass.

### Class-side private/protected

Add equivalent tests using `@class`.

### Reflection

For each relevant API:

```text
perform
methodFor
respondsTo
method invoke/bind
method reference
```

prove that access cannot be bypassed.

---

## 16. Diagnostics

Add stable error categories. Suggested names:

```text
member.private_access
member.protected_access
member.visibility_conflict
member.invalid_visibility_target
```

Error payload should include:

- attempted selector;
- access owner class;
- caller lexical class if available;
- source range of call where available.

Do not leak raw internal pointers/IDs.

---

## 17. Commands

Search dispatch paths:

```bash
rg -n "lookup_method|call_method|send_dynamic|respondsTo|methodFor|invokeOn|BoundMethod|MakeFamily|SuperSend|Invoke" phalcom-core/src
```

Targeted tests:

```bash
cargo test -p phalcom-core visibility
cargo test -p phalcom-core method
cargo test -p phalcom-core chunk
cargo test -p phalcom-core universe
```

Full:

```bash
cargo fmt
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

---

## 18. Acceptance criteria

- [ ] `@private` means defining-source-class private.
- [ ] `@protected` means defining-source-class-family private.
- [ ] Nested blocks inherit lexical class access.
- [ ] Subclasses cannot call inherited private methods.
- [ ] Subclasses can call inherited protected methods.
- [ ] Class-side visibility uses source class identity, not raw metaclass holder identity.
- [ ] Explicit and implicit `self` have identical visibility behavior.
- [ ] Unauthorized access does not route through `doesNotUnderstand`.
- [ ] `perform` cannot bypass access control.
- [ ] `methodFor`/Method invocation cannot bypass access control.
- [ ] Method references cannot bypass access control.
- [ ] Inline-cache hits cannot bypass access control.
- [ ] `super` follows the same visibility model.
- [ ] Existing public dispatch behavior remains unchanged.
- [ ] Full workspace tests pass.

---

## 19. Commit guidance

Suggested commits:

```text
feat(method): add private and protected visibility metadata
feat(vm): enforce lexical method access during dispatch
feat(reflection): enforce visibility for dynamic method access
test(vm): cover private protected access across cached and reflective sends
```
