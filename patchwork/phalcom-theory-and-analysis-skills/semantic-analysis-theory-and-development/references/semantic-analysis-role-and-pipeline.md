# Semantic Analysis Role and Pipeline

## Front end layering

A robust compiler/tooling front end separates concerns:

```text
text
-> tokens
-> recovered concrete/abstract syntax
-> declaration/scope surface
-> resolved semantic entities
-> normalized semantic representation
-> analyses
```

Each layer should remove ambiguity that downstream consumers should not repeatedly solve.

## Source syntax versus semantic syntax

Different spellings can denote the same semantic operation:

- syntactic sugar versus canonical message send;
- unary/binary/keyword forms normalized to selector identity;
- implicit receiver versus explicit `self`;
- collection sugar lowered to constructors/builders;
- attributes/decorators that transform declaration product.

Preserve source mapping while normalizing semantic meaning.

## Semantic passes

A typical dependency order:

1. declaration collection/surfaces;
2. lexical scope/binding identities;
3. import/module resolution;
4. reference/occurrence resolution;
5. inheritance/metaclass/member surface resolution;
6. body lowering/control flow;
7. local value/type/effect facts;
8. dispatch/call edges;
9. interprocedural fixed points;
10. checker/prover/lint queries.

Cycles can require staged shells/fixed points, especially modules, classes/protocols and call summaries.

## Semantic ownership

Each concept should have one canonical owner module/component. Consumers may cache rendered results but should not recalculate semantic truth.

## Static semantic errors

Examples:

- duplicate binding/declaration;
- unresolved import/name;
- illegal field access namespace;
- invalid `break`/`return` context;
- invalid super usage;
- inaccessible member;
- malformed type application;
- unprovable contract in strict mode.

Some belong to parser, some semantic analyzer, some checker. Keep error phase aligned with available information.

## IDE mode

Semantic analysis must operate on recovered/incomplete trees and may publish partial facts. It should avoid blocking every query on whole-project completeness.

## Compiler mode

Compiler can require stronger phase completion before code generation. Reuse semantic outputs where stable instead of reimplementing resolution.
