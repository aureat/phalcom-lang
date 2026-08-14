# Integrating the Future Phalcom Type Checker

## Preserve existing semantic infrastructure

The checker should start after binding/surface construction, not beside it.

Reuse:

```text
ModuleId / module graph
ScopeGraph / BindingId
ClassId / CallableId / FieldId
ModuleSurface / ClassSurface / MemberSurface
OccurrenceIndex / source ranges
DispatchResolver
shared control-flow program points
callable dependency graph
snapshot/invalidation infrastructure
```

## Add a distinct type store

Conceptual structure:

```rust
struct TypeStore {
    types: Vec<TypeData>,
    // interning/canonicalization indices as needed
}

#[derive(Clone, Copy, Eq, Hash)]
struct TypeId(u32);
```

Exact design depends on typing specs. Use IDs rather than recursively cloned type trees once
canonicalization/substitution becomes common.

## Type syntax resolution

Pipeline:

```text
AST type annotation
 -> name resolution in type/declaration namespace
 -> TypeExpr/descriptor identity
 -> canonical TypeId
 -> retain source annotation metadata for reflection
```

Do not discard the distinction between absent annotation and explicit dynamic/special type.

## Solver metavariables

Use separate inference variables:

```text
InferVarId != TypeId
```

A metavariable accumulates constraints and is solved/substituted. Never intern it as a permanent
user-visible type.

## Bidirectional checking

Design expression checker around:

```text
synthesize(expr, context) -> TypeId + obligations
check(expr, expected TypeId, context) -> obligations/result
```

Expected type can guide:

- empty literals;
- blocks/closures;
- generic call arguments;
- union choices;
- return checking.

## Relations

Implement separate APIs for relations selected by spec:

```text
type_equal(A, B)
is_subtype(A, B)
is_assignable(A, B)
is_consistent(A, B)       # gradual boundary if applicable
conforms(A, Protocol)
join_type(A, B)
meet_type(A, B)
```

Do not use runtime class inheritance as the entire subtype relation once protocols, generics,
unions/intersections/special types exist.

## Substitution

Substitution must be first-class and tested:

```text
substitute(TypeId, Substitution { TypeParamId -> TypeId }) -> TypeId
```

Needs capture-safe handling for nested generic binders and `Self`/method-owned type parameters
according to spec.

Applied member views should reference origin declaration + substitution rather than cloning and
mutating source declarations if semantic canonicalization is ratified.

## Generic local inference

Typing proposal indicates local per-send inference rather than HM-style global inference.
Implementation can:

1. instantiate callable type parameters with fresh infer vars;
2. use expected result type to add constraints (check mode);
3. map actual arguments to parameter types;
4. generate subtype/variance constraints;
5. solve bounds;
6. substitute into result;
7. diagnose under/over-constrained/ambiguous cases per normative spec.

Do not let advisory `ParameterFacts` silently decide generic types unless spec explicitly uses
such evidence.

## Flow typing

Build on shared CFG/structured flow.

Maintain typed environment:

```text
BindingId -> TypeId/refined TypeId
```

On assignment:

- check assigned type against declaration/constraint;
- update flow-refined state as permitted;
- invalidate smart cast if mutable binding changes.

On branch predicate:

- produce true/false refinements;
- join with type lattice at merge.

## Shape/type bridge

Keep explicit:

```rust
fn shape_to_synthesized_type(shape: &ValueShape, ...) -> Option<TypeId>
fn type_to_runtime_capabilities(ty: TypeId, ...) -> CapabilitySet
```

These bridges may be partial.

Do not require every type to map to one runtime class.

## Checker diagnostics

Constraint failures should retain reason edges:

```text
argument source range
parameter declaration/type source
substitution/bound that failed
actual synthesized type provenance
```

Avoid dumping raw solver variables to users.

## Typed runner

Static checker and runtime contract instrumentation share type metadata but are separate phases.

The checker can mark obligations:

```text
proved -> no runtime check needed in typed mode if policy allows
unknown -> runtime check plan if mode promises validation
refuted -> static diagnostic
```

Runtime contract lowering must not affect ordinary selector identity.

## Incrementality

Type facts introduce new dependency edges:

- annotation references;
- generic origin/substitution dependencies;
- protocol requirement dependencies;
- inferred callable signature dependencies;
- flow/checker body dependencies.

Use existing module/callable invalidation framework or extend it; do not build a checker-only
global cache with no edit invalidation.
