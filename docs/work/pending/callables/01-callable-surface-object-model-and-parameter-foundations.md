# Task Set 2 — Callable Surface, Object Model, Parser, and Parameter Foundations

**Project:** Phalcom
**Repository:** `aureat/phalcom-lang`
**Status:** Implementation-ready coding task-set
**Execution:** Coding task 1 of 3; must complete before Task Set 3
**Parallel work:** Task Set 1 documentation may run concurrently
**Primary areas:** source migration, parser/AST, kernel class hierarchy, heap surface mapping, parameter metadata foundations
**Must leave repository buildable and testable at completion**

---

## 1. Mission

Prepare the entire source/runtime model for the callable redesign without yet replacing the primitive invocation ABI.

This task-set must:

1. retire expression-bodied member `=>` syntax and migrate executable/test sources;
2. finish canonical Closure literal/trailing-Closure parsing;
3. add positional Closure rest syntax;
4. implement the ratified bare-brace trailing Closure sugar;
5. establish the new public callable hierarchy;
6. seal the VM-backed callable classes;
7. make `BoundMethod` a real public Function subclass;
8. make `Family` a Function subclass;
9. move `Method` out from under Function;
10. introduce shared argument/parameter-shape data structures needed by Task Set 3;
11. keep temporary invocation shims only where required to preserve a green intermediate state.

This task-set must **not** yet perform the full `PrimitiveFn` / `ArgumentView` / `CallOutcome` migration. That belongs to Task Set 3.

---

## 2. Ratified target hierarchy

Public/kernel hierarchy:

```text
Object
├── Method                         sealed/final
└── Function                       abstract, sealed
    ├── Closure                    sealed/final
    ├── BoundMethod                sealed/final
    └── Family                     sealed/final
```

User-defined classes may define `call` and be callable without inheriting `Function`.

No user subclass may derive from:

```text
Function
Closure
BoundMethod
Family
Method
```

Use the repository's existing sealed-class mechanism/invariants; do not invent a second sealing system.

---

## 3. Phase A — migrate and retire `=>` first

### 3.1 Canonical replacement

Named member:

```phalcom
size => _size
```

becomes:

```phalcom
size { _size }
```

Method:

```phalcom
double(_ value) => value * 2
```

becomes:

```phalcom
double(_ value) { value * 2 }
```

Legacy anonymous-arrow closures, if still present:

```phalcom
value => value * 2
```

become:

```phalcom
|value| value * 2
```

or the braced equivalent.

### 3.2 Migration strategy

Use regex/search as a **candidate ledger**, especially for:

- `.ph` files;
- Rust test strings containing Phalcom source;
- snapshots;
- fixture text.

Do not run an unscoped repository-wide replacement because Rust match expressions use `=>`.

Recommended search categories:

```text
*.ph
phalcom-ast tests/snapshots
phalcom-core tests
embedded string literals in *.rs
examples/benchmarks
```

For simple one-line member bodies, scripted conversion is acceptable.

For multiline expression bodies, convert with parser-aware or manually reviewed edits.

### 3.3 Parser removal

After source migration:

- remove `=>` as a named member-body form;
- remove old anonymous-arrow closure forms;
- leave `FatArrow` token only if some independently ratified grammar still needs it;
- otherwise remove it from lexer/token/LSP/snapshots too;
- if temporarily retained, emit a targeted migration diagnostic rather than accepting old syntax.

### 3.4 Gate

Run parser + compiler + bootstrap tests before proceeding to the hierarchy/parser redesign. Do not mix a broken syntax migration with later runtime failures.

---

## 4. Phase B — Closure literal grammar

Canonical forms:

```phalcom
|| expression
|| { ... }

|x| expression
|x| { ... }

|x, y| expression
|x, y| { ... }

|*rest| expression
|*rest| { ... }

|head, *tail| expression
|head, *tail| { ... }
```

### 4.1 Rest restrictions

Closure parameters support only:

```text
fixed positional parameters
optional one terminal positional rest parameter
```

Reject:

```phalcom
|x, *tail, y| { ... }
|*a, *b| { ... }
|**labels| { ... }
|***arguments| { ... }
```

Do not add labeled Closure parameters.

Do not interpret `args...` as rest.

### 4.2 Lexer

Keep `||` as two existing `Pipe` tokens unless the current lexer already has a compelling independent reason to tokenize it differently.

Preserve bitwise OR:

```phalcom
a | b
```

### 4.3 AST shape

Do not encode `*` inside the parameter name.

Introduce a structured Closure-parameter representation. A minimal acceptable shape is:

```rust
struct ClosureParameters {
    fixed: Vec<String>,
    positional_rest: Option<String>,
}
```

and use it from the existing closure/block AST node.

A broader shared parser-facing parameter representation is acceptable only if it does not accidentally enable labeled Closure parameters.

The compiler must receive normalized parameter meaning directly; it must never rediscover rest semantics by parsing strings.

---

## 5. Phase C — optimized D35-B bare-brace trailing Closure sugar

Ratified choice: a bare brace body following an eligible method-send expression is a zero-argument trailing Closure.

Examples:

```phalcom
resource.withLock {
    work()
}
```

means:

```phalcom
resource.withLock || {
    work()
}
```

and:

```phalcom
resource.withLock do: {
    work()
}
```

means:

```phalcom
resource.withLock do: || {
    work()
}
```

### 5.1 Optimization / parser design

Do **not** implement this through speculative parsing or rollback.

Reuse/introduce parser-local postfix eligibility state, conceptually:

```rust
enum TrailingTarget {
    None,
    MemberSend,
}
```

When all of these are true:

```text
current postfix target == MemberSend
next token == LBrace
```

consume the brace body immediately and attach a synthesized zero-parameter Closure argument.

No lookahead beyond `{` is necessary.

No method lookup is necessary.

No semantic knowledge is necessary.

No alternate parse is attempted.

This is O(1) dispatch in the postfix parser before ordinary brace-primary parsing could occur.

### 5.2 Ambiguity rule

In this postfix-send context:

```phalcom
obj.method { key: value }
```

means a trailing zero-argument Closure whose body contains the brace statements/expressions allowed by the code-block grammar.

If the programmer intends to pass a Map/Set value, parentheses disambiguate:

```phalcom
obj.method({ key: value })
```

General braces retain Map/Set meaning elsewhere.

Bare braces are **not** restored as general Closure primary expressions.

### 5.3 Eligibility safety

Do not infer eligibility solely from an AST node that may have been synthesized by desugaring.

Preserve/clear `TrailingTarget` explicitly.

Clear eligibility after constructs such as:

- optional-send desugaring;
- index expressions;
- method references;
- arbitrary synthetic wrapper calls;
- generic callable application where trailing-method syntax is not explicitly supported.

The parser must reject rather than silently attach a brace Closure to a synthetic outer call.

### 5.4 Explicit `|...|` trailing closures

Retain explicit trailing Closure syntax for parameterized closures:

```phalcom
users.any where: |user| {
    user.active
}
```

The bitwise-OR disambiguation rule from the existing pending Closure parser plan remains useful: unparenthesized trailing `|...|` Closure syntax should be structurally identifiable before consuming the first pipe.

---

## 6. Phase D — kernel class hierarchy

Update core-class creation from the old model:

```text
Function
├── Block
└── Method

Family < Object
BoundMethod surfaces as Block
```

to:

```text
Function < Object
Closure < Function
BoundMethod < Function
Family < Function
Method < Object
```

Required bootstrap fields:

```text
function_class
closure_class
bound_method_class
family_class
method_class
```

Remove public/kernel `block_class`.

Update metaclass-superclass parallelism through the existing class construction rules.

Mark Function abstract.

Mark the ratified VM-backed classes sealed/final with the repository's existing sealing machinery.

---

## 7. Phase E — heap/value surface mapping

At the end of this task-set, surface classification must report:

```text
Closure runtime value       → Closure class
BoundMethodObject           → BoundMethod class
FamilyObject                → Family class
MethodObject                → Method class
```

### 7.1 Transitional internal representation rule

To keep this task-set independently green, it is acceptable to retain the internal `Object::Block` / home-frame wrapper temporarily if current non-local-return machinery still requires it.

If retained:

- it must surface as `Closure`, never `Block`;
- comments must mark it transitional and scheduled for deletion in Task Set 4;
- no new code may depend conceptually on `Block` as a public class;
- do not duplicate Closure state into a second permanent representation.

Task Set 4 will remove the wrapper after local-return semantics are fully active.

### 7.2 BoundMethod

Keep the existing minimal payload concept:

```rust
BoundMethodObject {
    method,
    receiver,
}
```

Do not synthesize a Closure or clone a Method.

Do not create per-signature synthetic `call` Methods.

---

## 8. Phase F — sealing and method-table invariants

Enforce that user code/reflection cannot subclass the sealed core callable classes.

Also establish a kernel invariant reserving the Function `call` base-name family for the common gateway that Task Set 3 will install.

Do not rely only on parser restrictions; reflective method-table mutation must not be able to install an overriding Function-descendant `call` family.

If the final call-family enforcement cannot be activated before Task Set 3 installs the new gateway, add the invariant hook now and enable it during Task Set 3.

---

## 9. Phase G — shared parameter foundations

Introduce shared normalized types used by Task Set 3.

Recommended separation:

```rust
struct ArgumentShape {
    positional_count: ...,
    labels: ...,
}

struct ParameterShape {
    fixed_positionals: ...,
    fixed_labels: ...,
    rest: Option<RestKind>,
}

struct ParameterLayout {
    // bytecode-frame slot placement only
}

struct BindingPlan {
    // ephemeral result of matching ArgumentShape to ParameterShape
}
```

### 9.1 Rest kinds

Preserve F.3 lane semantics:

```text
Positional
Labeled
Split
Complete
```

Closure shapes may use only:

```text
None
Positional
```

### 9.2 Refactor existing F.3 metadata

Current Method `Signature` remains the owner of Method dispatch/acceptance shape.

Do not create a second independently mutable Method rest shape on `MethodObject`.

Move parameter-slot indices out of pure acceptance metadata when practical:

```text
ParameterShape
    describes acceptance

ParameterLayout
    describes local-slot layout
```

Task Set 3 will finish the binder migration.

### 9.3 Callable metadata

Prepare compiled Closure callables to carry structured Closure `ParameterShape` rather than relying solely on scalar `arity`.

A temporary compatibility `arity` field may remain until Task Set 3, but all new Closure-rest logic must use the structured shape.

---

## 10. Constructor-generation preparation

Preserve the conceptual model:

```text
source @constructor declaration
    → compiler-generated class-side factory
    → ordinary hidden/generated instance initializer Method
```

The generated factory:

1. allocates/reuses the receiver instance according to existing inheritance rules;
2. calls the initializer;
3. discards the initializer result;
4. returns the allocated instance.

The initializer is an ordinary Method at runtime.

Do not add a special constructor-return opcode.

Full final-expression/Unit behavior and the compile-time prohibition on `return value` in constructor initializer source are completed in Task Set 4.

---

## 11. Temporary invocation compatibility

This task-set may keep the current invocation machinery temporarily, including old native call shims, solely to maintain a green intermediate tree.

However:

- public classes must already be `Closure`, `BoundMethod`, `Family`, `Method`;
- do not expose old Block naming;
- do not add new dependencies on recursive `run_until`;
- do not expand the old finite call-overload architecture;
- mark any retained compatibility paths for deletion/replacement by Task Set 3.

---

## 12. Required tests

### Syntax migration

- no accepted named-member `=>`;
- no accepted anonymous-arrow Closure syntax;
- core/bootstrap `.ph` files compile after migration;
- embedded test strings migrated.

### Closure parser

Accept:

```text
||
|x|
|x, y|
|*rest|
|head, *tail|
```

Reject invalid rest layouts.

### Bitwise OR

```phalcom
a | b | c
```

must remain a bitwise expression.

### Bare trailing braces

Accept:

```phalcom
resource.withLock { ... }
resource.withLock do: { ... }
```

Ensure:

```phalcom
obj.method({ key: value })
```

still passes a Map.

Ensure optional-send/synthetic AST forms do not capture a trailing brace accidentally.

### Object model

Assert:

```text
Closure.superclass == Function
BoundMethod.superclass == Function
Family.superclass == Function
Method.superclass == Object
Function is abstract
all five VM-backed callable classes are sealed as specified
```

Assert BoundMethod and Family surface with their own classes.

---

## 13. Explicit non-goals

Do not yet:

- replace `PrimitiveFn`;
- add `CallOutcome`;
- implement final flat Function activation;
- remove `ReturnNonLocal`;
- remove home-frame token machinery;
- migrate all None/no-result sites to Unit;
- implement value-carrying `break`;
- move reflection APIs;
- remove `Function#arity`/`name` if doing so would break the temporary call layer.

Those are Task Sets 3 and 4.

---

## 14. Completion gate

Task Set 2 is complete when:

- all executable/test source is free of retired `=>` member bodies;
- canonical Closure syntax and positional Closure rest parse;
- D35-B trailing brace sugar works without speculative parsing;
- `Block` is no longer a public class;
- new callable hierarchy is live and invariant-tested;
- BoundMethod and Family are real Function subclasses;
- Method is outside Function;
- sealed-class rules are enforced;
- structured parameter-shape foundations exist;
- the repository builds and the existing relevant test suites pass;
- any retained internal Block/home-frame wrapper is explicitly transitional.
