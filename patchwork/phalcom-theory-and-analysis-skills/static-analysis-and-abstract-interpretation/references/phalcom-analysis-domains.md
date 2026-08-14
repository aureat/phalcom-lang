# Phalcom Analysis Domain Map

This is a conceptual map; inspect current repository source before implementation.

## Existing advisory semantic domains

The current LSP semantic engine includes:

- module/class/callable/field/binding identities;
- `ValueShape` over instance/class/module/tuple/record/list/set/map/range/callable/family/union/unknown;
- confidence/provenance-bearing inferred values;
- local flow facts;
- field evidence;
- call-site parameter facts;
- callable summaries/effects;
- module/call dependency invalidation;
- immutable published snapshots.

These are strong foundations for tooling.

## Future correctness domains

Do not overload `ValueShape` to carry all of these:

### Type domain

```text
nominal instance/class-object types
protocols
unions/intersections
applied generics
callables
Self
Dynamic/Any/Nothing/etc.
```

### Proof/path domain

```text
boolean propositions
presence/tag facts
numeric relations
contract assumptions
invariants
```

### Effect domain

```text
throws/yields/blocks
reads/writes
IO/process
reflection/native
```

### Heap/alias domain

Only if needed for proving/optimization beyond receiver-local fields.

## Bridges

Examples:

- exact runtime class shape can imply nominal instance type evidence;
- declared type can constrain possible runtime shape;
- Option type + tag test creates proof/path refinement;
- proof eliminates union alternative;
- callable type and runtime dispatch surface combine to verify a send;
- native type/effect contract seeds both checker and analysis.

Bridges should be explicit functions/relations with provenance.

## Consumers

```text
LSP completion/hover -> may accept advisory shape
checker -> requires type relation/trusted facts
static prover -> requires sound proof/effect model
lint -> choose minimum analysis tier per rule
optimizer -> requires sound effect/identity facts
runtime typed mode -> uses reified contract metadata
```

## Key Phalcom hazards

- metaclass/class-side lookup;
- implicit-self resolution;
- non-local block returns;
- dynamic selector families/packs;
- reflective method mutation;
- Option rather than surface nil;
- fibers and blocking native calls;
- core/native methods lacking source bodies;
- future module/package identity.
