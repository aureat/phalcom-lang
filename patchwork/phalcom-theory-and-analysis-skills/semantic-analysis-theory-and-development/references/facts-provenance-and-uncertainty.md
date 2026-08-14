# Facts, Provenance, and Uncertainty

## Fact schema

A semantic fact should conceptually include:

```text
subject semantic ID/program point
fact domain value
confidence/trust
provenance
revision/generation
```

Not every struct must store all fields inline; provenance can be interned/side-tabled.

## Separate domains

Keep:

```text
RuntimeShapeFact
TypeFact
EffectFact
ProofFact
ConstFact
AliasFact
```

rather than one `SemanticType` enum that mixes unrelated meanings.

## Provenance graph

A diagnostic explanation may need a DAG:

```text
return type fact
  <- call summary
      <- return statement
          <- binding assignment
              <- literal
```

Bound/compress provenance on hot paths while retaining enough roots to explain.

## Uncertainty reasons

`Unknown` should often carry reason:

```text
NoEvidence
DynamicDispatch
WidenedUnion
UnresolvedImport
UnsupportedConstruct
NativeBoundary
RecursiveCycle
RecoverySyntax
```

This lets consumer decide whether to omit hover detail, issue a warning, require annotation, or keep runtime check.

## Confidence versus soundness

Current LSP confidence categories (`Exact`, `Flow`, `Interprocedural`, `Heuristic`) describe evidence strength for advisory shape facts. A future checker/prover needs additional trust/soundness categories; do not reinterpret `Heuristic` as a type-checking proof level.

## Contradictions

When two required facts conflict, represent an error/contradiction rather than widen to unknown if doing so would hide a correctness violation.

For advisory analysis, joining incompatible runtime possibilities can be correct; for declared type contract, assignment of incompatible value is an error. Same evidence, different domain/relation.
