# Spec A — Product Foundations

This implementation unit establishes the structural product substrate required by the rest of Phalcom collections: Symbol-capable labels, canonical Unit, two-lane Tuple, immutable Symbol-field Record, zero-product normalization, and direct product construction. It deliberately excludes typing/reflection and later collection behavior such as slicing, Map conversion, and expansion.

The semantic authority is the supplied `collections-next` specification set. The repository's older `U-COLL`/`U-COLLTYPES` plans describe legacy implementation choices and are useful only for locating existing code.

## Repository diagnosis

The current repository is not greenfield. It already has first-class `Value::Symbol`, Symbol literal AST support, a native positional-only `TupleObject { elements: Box<[Value]> }`, three Tuple primitives, and parser lowering of `(a, b)` into `Tuple.fromList(List.new()...)`. It does not have Unit or Record runtime values, and its lexer does not yet admit `#{...}` or quoted Symbols. Spec A is therefore a controlled migration rather than a new Tuple implementation.

The selected target architecture is:

```text
source syntax
  -> explicit Tuple/Record AST
  -> direct product-build bytecode
  -> shared product finalizer
       zero closed arity -> Value::Unit
       positive Tuple    -> TupleObject
       positive Record   -> RecordObject
```

Unit is an immediate `Value::Unit`. Positive Tuple and Record remain native heap values because they contain traceable `Value`s. Tuple uses one total-order value buffer plus an ordered labeled-Symbol suffix descriptor. Record uses parallel Symbol/value arrays in encounter order. Both are immutable by representation.

## Phase order

### A.1 — Product Syntax and AST Foundation

Adds Record/quoted/operator Symbol lexical forms, explicit Tuple/Record AST nodes, product-label syntax, empty/singleton/labeled Tuple parsing, and Record parsing. Existing positive positional Tuple programs remain executable through one temporary compiler bridge to `Tuple.fromList`.

Artifact: `A.1-product-syntax-and-ast.md`.

### A.2 — Unit, Tuple, and Record Runtime Representation

Adds immediate Unit and its class, migrates Tuple storage to two lanes, adds positive immutable Record storage, updates GC/exhaustive runtime plumbing, and introduces the shared finalizers that canonicalize zero products to Unit. The primitive floor is expected to remain unchanged in this phase.

Artifact: `A.2-product-runtime-representation.md`.

### A.3 — Product Construction, Structural Semantics, and Surface Integration

Adds direct build bytecodes, static/dynamic label validation, duplicate detection, minimal lane/field raw observations, Tuple lane projections, structural equality/hash, Record safe Symbol lookup, and removes the legacy `Tuple.fromList` construction dependency. Any required primitive-floor amendment is handled here as one coherent change.

Artifact: `A.3-product-construction-and-surface.md`.

## Cross-phase invariants

Every phase must preserve a green repository gate and must not create an observable pre-normalized empty product. After A.2 lands, all runtime product allocation must pass through a finalizer that returns Unit at zero arity. After A.3 lands, source product construction must use that same path directly rather than staging through List.

The following remain explicitly outside Spec A: generic collection indexing/slicing and negative-index policy; Range; Map/Set semantics; `Map.from(record:)`; `*`, `**`, and `***` expansion behavior; variadic argument capture; destructuring; complete Record field-access syntax; Record update/merge APIs; general Record iteration shape; common product protocol hierarchy; exact printing/serialization; typing/generics; and reflection.

## Repository verification gate

Each implementer should re-read HEAD before editing because Phalcom is evolving quickly. The mandatory final gate for every subphase is:

```sh
./scripts/verify.sh --full
```

The current script runs a workspace build in `--full` mode, the full workspace test suite (including AST snapshots and language goldens), and Clippy across all targets.
