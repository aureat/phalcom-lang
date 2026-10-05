# Static Evaluation

**Status:** Normative  
**Semantic ownership:** compile-time constants, executable static `type` definitions,
static values, control flow, proof predicates, recursion, residualization, budgets,
materialization, and cross-module static semantics.

## 1. Core law

Static evaluation is an alternate semantic evaluation strategy for a defined subset of
ordinary Telos constructs.

It is not bytecode execution, a compile-time VM, arbitrary runtime method dispatch, or
arbitrary native execution.

## 2. Static contexts

A constant binding evaluates its initializer statically:

```telos
const x = expression
```

Parameterized `type` declarations define reusable static computation:

```telos
type Optional<T> =
    T | None

type IsSubtype<A: Type, B: Type>: Bool =
    Subtype<A, B>

type Count<*P>: Int {
    let mut count = 0
    for _ in P {
        count += 1
    }
    count
}
```

Expression bodies use `=`. Block bodies do not.

The default result domain of a `type` declaration is `Type`; explicit static result
domains such as `Bool`, `Int`, `ProductShape`, or `Variants` are permitted when
defined by the type/static-domain system.

C4 does not define const functions or const methods.

## 3. Static values

Static evaluation may produce immutable semantic values including:

```text
Unit
Bool
Int
Float
String
Symbol
Type
static-domain terms
Pack
Tuple
Record
transparent data values
Sum values
```

Static values are semantic values, not VM objects. They never depend on runtime heap
identity, arena offsets, or process-local symbol IDs.

Reduced structural static values form an acyclic DAG. Recursive semantic entities are
represented by stable semantic references rather than recursively expanded values.

## 4. Allowed control and expressions

Subject to static eligibility, static evaluation supports ordinary Telos forms including:

- literals and static references;
- deterministic arithmetic, comparison, boolean operations, and semantic equality;
- construction and projection of static Tuple, Pack, Record, data, and Sum values;
- `if`;
- `match`;
- `for`;
- `while`;
- local `let`;
- local `let mut`;
- assignment to evaluator-local mutable slots;
- `return`, `break`, and `continue`;
- destructuring and structural reflection;
- parameterized and recursive `type` application;
- compiler relation/proof predicates.

Untaken branches are not evaluated.

### 4.1 `while`

`while` is an explicit semantic control-flow construct.

The static evaluator directly evaluates canonical `while` semantics. A selector,
method, protocol, or user declaration named `whileTrue` has no intrinsic or privileged
meaning in static evaluation.

Runtime lowering may independently compile semantic `while` into efficient control
flow after semantic analysis.

## 5. Forbidden effects

Static evaluation does not execute arbitrary runtime functions or methods and cannot
observe:

- I/O, filesystem, network, or process state;
- clock or randomness;
- runtime mutable global state;
- runtime object identity;
- fibers, scheduler, threads, or similar execution state;
- arbitrary native calls.

Unsupported runtime behavior in an untaken branch does not invalidate a computation
whose selected branch is otherwise statically valid.

## 6. Evaluation outcomes

The semantic evaluator distinguishes at least:

```text
Reduced(value)
Symbolic(residual)
Invalid(error)
Recursive(cycle)
BudgetExceeded(report)
Cancelled
```

A closed `const` must finish `Reduced`.

An open/generic static definition may remain `Symbolic`.

## 7. Proof predicates

`Equivalent`, `Subtype`, and `Conforms` have public static result domain `Bool`.

Internally:

```text
proof       -> true
refutation  -> false
unknown     -> symbolic residual
blocked     -> preserved blocked/residual state as appropriate
dynamic     -> symbolic only where the static context permits it
```

Failure to prove is never converted to `false`.

In an open generic definition, an unresolved predicate remains part of normalized
semantic residual computation. Tooling may report that the expression is unresolved
until specialization.

A context requiring a concrete closed result diagnoses failure to determine the Bool
rather than choosing a branch arbitrarily.

## 8. Patterns and structural control

Static and runtime matching use one Telos pattern language and one semantic pattern
resolver. Static evaluation consumes the same resolved variant candidates, projections,
exhaustiveness/usefulness results, and branch proof environments used by ordinary
matching.

C4 adds only the structural pattern forms required to express Pack/product tails and
captures; it does not define a separate static pattern language.

Static structural `for` does not invoke the runtime iterator protocol. It iterates
canonical structural entries directly.

For heterogeneous structural subjects, the body is evaluated with per-iteration exact
refinement while remaining valid across all reachable iterations.

Anonymous open tails describe possible structure and are not finite iterations.

## 9. Recursion and residualization

Static recursion uses explicit evaluator frames, not unbounded host-language recursion.

The memo/cycle key is:

```text
stable static-definition identity
+ canonical static arguments
+ relevant specialization/proof environment
```

Exact repetition of an in-progress key is a cycle. Changing-argument recursion is
permitted until it reaches a base case or budget.

Unresolved generic computation is represented by normalized semantic residual IR.
Residuals are not raw source AST and not suspended evaluator stacks.

## 10. Budgets and cancellation

Static evaluation uses the compiler's normal query-budget and cancellation authorities.
Work such as expression evaluation, definition application, loop iteration, structural
construction, and residual growth is bounded and cooperatively cancellable.

Budget and cancellation terminal states are not silently converted into valid semantic
results.

## 11. Reflection recursion safety

Reflection describes immediate structure only.

If a product field contains a recursive type, reflection stores the contained `Type`
value/reference; it does not recursively expand that type's `ProductShape`.

Likewise, constructing a Pack/ProductShape/Variant value does not implicitly reflect
its own type.

Capability derivation and reflection originate from lower-level canonical declaration
facts; neither recursively proves the other.

## 12. Constant materialization

A reduced static value may be consumed at runtime through a semantic materialization
descriptor.

```text
source initializer
    -> semantic static evaluation
    -> StaticValue
    -> materialization descriptor
    -> runtime lowering/materialization
```

The initializer is never rerun at runtime.

Materialization may choose immediate constants, shared immutable objects, flat products,
or other physical representations without changing semantic value identity.

## 13. Cross-module semantics

Exported constants include stable value identity/content in dependency fingerprints.

Exported reusable static definitions expose checked semantic definition information and
stable normalized residual semantics only as needed for specialization. Metadata never
serializes evaluator frames, AST pointers, VM object references, or session-local IDs.

Cold and incremental evaluation must agree.


## 14. Computed Record keys and static formation cycles

A computed Record key is written as `[expression]`. Its expression is checked
with ordinary Telos name and binding semantics, then evaluated only when it is
statically eligible under this specification. Runtime bindings and arbitrary
runtime effects do not become static merely because they appear in a key.

A computed key required to form a closed Record type or a resolved structural
pattern must reduce to one concrete key in the Record key domain: `String`,
`Symbol`, `Int`, or `Bool`. The reduced semantic value determines the key's
identity. In particular, a Symbol key and a String key with the same printed
text are distinct. Duplicate detection occurs after reduction and uses
canonical Record-key identity. A symbolic result is not a concrete key and may
remain deferred only in a context whose language rules explicitly permit
symbolic structural residualization; it must not be treated as a closed key.

Static constants may be used by computed keys through their checked static
values. Forming a constant's declared type may itself require computed keys and
other constants; these dependencies are demand-driven and recursively checked.
There is no source-order rule requiring all constants to be evaluated before all
types, or all types before all constants.

A static formation cycle is invalid when forming a closed static constant's
declared type requires a computed Record key whose static evaluation directly
or transitively requires that same constant. Such a dependency cycle has no
fixed-point meaning. It must not be resolved by source order, initializer-first
speculation, a guessed value, a symbolic key placeholder, or a value retained
from an earlier successful analysis. The cycle produces a deterministic
invalid/recursive semantic result with a diagnostic; it must not hang or publish
a concrete Record type based on stale information.

Cold analysis and incremental analysis of the same source must agree on reduced
keys, resulting Record types, diagnostics, and downstream semantic results.
