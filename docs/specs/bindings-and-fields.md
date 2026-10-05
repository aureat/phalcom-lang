# Bindings and Fields

**Status:** Normative  
**Semantic ownership:** runtime binding phase and mutability, initialization state,
field declaration and mutability, immutable-field construction lifecycle.

## 1. Binding forms

Telos has three binding forms:

```telos
let x = expression
let mut y = expression
let mut pending: T
const z = staticExpression
```

| Form | Phase | Mutable | Initializer |
| --- | --- | ---: | --- |
| `let` | runtime | no | required |
| `let mut` | runtime | yes | optional |
| `const` | compile time | no | required |

Staticness is declared by `const`; it is never inferred from optimizer knowledge.

## 2. Immutable runtime bindings

A plain `let` binds one runtime value and cannot be reassigned.

```telos
let name = computeName()
name = otherName       // error
```

An immutable `let` without an initializer is invalid.

## 3. Mutable runtime bindings and initialization

`let mut` creates mutable runtime storage.

```telos
let mut count = 0
count += 1
```

The initializer may be omitted:

```telos
let mut result: Result
```

Such a binding begins in the **uninitialized storage state**.

Uninitialized is not a Telos value. In particular:

```telos
let mut result: Result
```

does not mean:

```telos
let mut result: Result | None = None
```

Reading an uninitialized binding traps at runtime.

### 3.1 Local definite-initialization analysis

Local flow state tracks:

```text
Uninitialized
MaybeInitialized
DefinitelyInitialized
```

Assignments transition the binding to `DefinitelyInitialized`. Branch and loop joins
merge initialization state using ordinary control-flow reachability.

At a read:

- `DefinitelyInitialized` — the read is proven safe and requires no runtime
  initialization check;
- `Uninitialized` — the compiler diagnoses a guaranteed invalid read;
- `MaybeInitialized` — the read remains valid to compile but carries a runtime
  initialization check.

When initialization cannot be proven through a closure, module, or other effect
boundary, runtime checking remains the semantic backstop.

The language therefore does not require whole-program definite assignment.

## 4. Compile-time constants

`const` is specified by [Static Evaluation](static-evaluation.md).

A `const` initializer must be statically evaluable, and a closed constant must fully
reduce before runtime lowering. Runtime code consumes/materializes the frozen result;
it does not execute the initializer.

## 5. Fields

Fields must be declared explicitly. Assignment never introduces a field.

Fields are immutable by default:

```telos
class User {
    _name: String
    mut _visits: Int
}
```

`mut` is the only ordinary mutable-field marker.

### 5.1 Immutable-field initialization

An immutable field may receive its value during the object's constructor lifecycle.
This is initialization, not mutation.

Semantic analysis must enforce:

- every required immutable field is definitely initialized before the object becomes
  fully usable;
- an initialization write is valid only in the defined constructor lifecycle;
- a field initialized by a declaration default is already initialized;
- an immutable field cannot be reassigned after initialization;
- branch-sensitive constructor paths must establish definite initialization.

No first-write-anywhere or runtime write-once rule is defined.

### 5.2 Mutable fields

A `mut` field may be assigned after construction subject to its declared type and
ordinary access rules.

## 6. Captures and globals

The runtime uninitialized state is preserved when mutable storage is captured or stored
at module scope. Reads that semantic analysis cannot prove initialized remain checked.

Local proof does not imply a requirement for interprocedural or whole-module definite
initialization.

## 7. Invariants

- **BIND-PHASE-SEPARATION** — runtime immutability and compile-time staticness are
  distinct properties.
- **BIND-UNINITIALIZED-NOT-NONE** — uninitialized storage is not `None` or any other
  Telos value.
- **BIND-RUNTIME-BACKSTOP** — every unproven read of possibly uninitialized mutable
  storage remains runtime-safe by trapping rather than surfacing an invented value.
- **FIELD-EXPLICIT** — assignment cannot create object layout.
- **FIELD-CONSTRUCTION-ONLY** — immutable field writes are construction-lifecycle
  initialization only.
