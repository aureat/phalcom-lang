# Type Checker Integration

## Reuse, do not replace

The checker should reuse:

```text
ModuleId/ClassId/CallableId/BindingId
scope graph and resolved occurrences
class/member surfaces
selector/dispatch logic
body lowering/CFG
call graph/summaries
source provenance
```

It adds canonical type descriptors, inference variables, constraints, subtyping/conformance and contract diagnostics.

## Two fact directions

Semantic runtime facts can seed/check types:

```text
literal 1 -> runtime shape Int + type Int
resolved constructor -> instance type
```

Declared types can constrain analysis:

```text
param: Protocol P -> guaranteed selector surface
```

But bridge through explicit functions, not shared enum variants.

## Type representation

Introduce separate canonical `TypeId` arena/interner. It may reference `ClassId` for nominal instance types and `ProtocolId` for protocols.

Do not reuse `ValueShape::Union` as language union type.

## Program points

Flow typing needs the same program-point/CFG identities as value analysis. A binding has declared/base type plus refined flow type/facts at a point.

## Checker modes

Potential modes:

```text
synthesize type
check against expected
collect generic constraints
verify override/conformance
validate declared contract
```

These are semantic queries over resolved/lowered program.

## Dispatch

Type checker verifies ordinary send contract; it does not choose ordinary runtime target by type annotation. Protocol/union receiver may produce a callable contract even with multiple runtime targets.

## Diagnostics

Constraint origins map to semantic/source IDs. Reuse occurrence/definition metadata for related spans.

## Incrementality

Type facts depend on declarations, imports, callable bodies, type metadata and callee contracts. Extend semantic dependency graph rather than invent a second project scanner.
