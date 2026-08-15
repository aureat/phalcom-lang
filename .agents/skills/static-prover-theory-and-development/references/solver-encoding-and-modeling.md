# Solver Encoding and Semantic Modeling

## Build a proposition IR first

Do not generate raw solver API calls directly from AST. Define a solver-independent logic IR:

```text
BoolExpr
IntExpr
Real/FloatExpr
ADT terms
Object/type tags
Heap select/store
Uninterpreted calls
Quantifiers only where necessary
```

Benefits:

- simplification;
- testing without solver;
- multiple solver backends;
- source provenance;
- deterministic serialization/hash.

## Semantic sort discipline

Map language domains to solver sorts explicitly.

Examples:

```text
Int -> mathematical Int sort
Bool -> Bool or ADT depending proof question
Option<T> -> ADT Option(T)
Object identity -> uninterpreted Object sort / finite symbolic IDs
ClassId -> datatype/uninterpreted sort with axioms
String -> solver String/Sequence only when semantics align
```

## Object/class modeling

A naive single `Object -> Int` encoding can confuse class tags, identity and fields. Separate:

```text
object identity
dynamic class function classOf(obj)
field heap functions
selector/protocol predicates if required
```

Closed-world finite class enumerations are only valid under explicit closed assumptions.

## Function calls

Prefer verified summaries/contracts over encoding arbitrary recursive function bodies as solver functions. Function axioms with universal quantifiers can make solving unpredictable.

## Heap snapshots

Use SSA-style heap versions:

```text
H0
H1 = store(H0, obj, field, value)
```

Frame conditions state locations unchanged.

## Source integers versus VM representation

Proof of language-level `Int` arithmetic uses mathematical semantics. Proof of bytecode overflow/boxing correctness may use bit-vectors plus bignum model. Do not mix levels.

## Floating point

Use FP theory for exact runtime equivalence. Real-number approximation can be a heuristic analysis only if never used to prove FP-sensitive claims.

## Canonicalization

Hash-cons/sort commutative operands, simplify constants and intern terms. Stable VC hashes improve caching and test snapshots.

## Solver isolation

Run solver with time/memory limits. Treat crashes/malformed responses as prover failure/Unknown, not compiler panic.

---

## Deep treatment: encoding correspondence and model validity

### Encoding contract

Treat solver encoding as a translation with an explicit correctness obligation:

```text
encode : ProofIR -> SolverFormula
```

The desired property is:

```text
for every valid Phalcom/proof state s represented by proof IR,
interpretation(s) satisfies the encoded semantic constraints;

and if the solver finds a countermodel reported to the user,
it must correspond to at least one valid represented program state.
```

For proving validity, over-constraining the model can be unsound because it may remove real executions and make a false property appear valid. Under-constraining can create spurious counterexamples. Sound proof therefore prioritizes not excluding real behaviors; diagnostic quality additionally needs enough constraints to reject impossible witnesses.

### Object model encoding

A flexible encoding separates sorts/functions:

```text
Obj       uninterpreted object identity sort
Class     class identity sort
Field     field identity sort (or statically indexed field functions)
Value     tagged sum or family of typed sorts

classOf : Obj -> Class
fieldVal : Heap × Obj × Field -> Value
```

Typed proof terms may avoid one giant `Value` sum by projecting to domain-specific sorts after type evidence. The translation must still model object identity distinctly from value equality if Phalcom distinguishes them.

### Class constraints

If an object term is known to be exact class `C`:

```text
classOf(o) = C
```

If known only to conform to protocol/type `P`, use the ratified subtype/conformance relation rather than inventing class enumeration. Closed-world enumeration:

```text
classOf(o) = C1 ∨ classOf(o) = C2
```

is valid only under an explicit sealed/world assumption.

### Heap encoding alternatives

Common approaches:

1. **Single array heap:** `H : (Obj,Field) -> Value`. Simple but heterogeneous value sort is heavy.
2. **Per-field arrays:** `balance_H : Obj -> Int`, `name_H : Obj -> String`. Efficient when field identity/layout is statically known.
3. **Region/object abstraction:** model only locations relevant to the obligation; unknown rest is abstract/havoced.

Choose based on Phalcom semantic IR and proof goals. The model must support aliasing: if `o1 = o2`, reads through both names agree.

### Frame encoding

For modifies set `W`, conceptually:

```text
∀loc. loc ∉ W => H1[loc] = H0[loc]
```

A universal quantifier may be expensive. Alternatives include finite relevant-location framing, per-field heap versions, or region summaries. The finite approach is sound only for properties that mention the tracked locations and when untracked writes cannot indirectly affect tracked abstract values.

### Collection abstraction

Do not expose Rust `Vec` or hash table layout. Model a mutable list abstractly:

```text
SeqContent(H, o) : Seq<Value>
Length(H,o) = len(SeqContent(H,o))
```

Library contracts define effects of `append`, indexing, iteration, etc. This keeps the proof model stable if runtime representation changes.

### Selector/method semantics

Avoid representing message send as a pure solver function unless the method contract justifies it. For a verified pure total method, it may be modeled as a function symbol with a defining theorem. For ordinary dynamic send, model the call through summary transitions at proof-IR level before backend encoding.

### Naming and model reconstruction

Backend symbol names should encode stable internal IDs, not user strings alone:

```text
v_42      -> SsaLocalId(42)
obj_17    -> SymbolicObjectId(17)
h_3       -> HeapVersionId(3)
```

A side table maps these to user names/spans. Never parse the user explanation back out of solver-generated names.

### Canonical serialization

To cache queries, define a proof-model version and canonical ordering:

```text
hash = H(
  proof_model_version,
  sorted declarations,
  canonical formula DAG,
  solver theory/configuration
)
```

Do not expect hash stability across intentional encoding changes unless the version remains semantically compatible.

### Backend validation

Before calling solver, validate:

- every symbol declared once with stable sort;
- every term well-sorted;
- no dangling arena IDs;
- no unsupported theory accidentally encoded;
- formula size/resource limits;
- all assumptions tagged for provenance.

After `sat`, validate model decoding. After `unsat`, no model is needed for ordinary proof but optional unsat cores/proof certificates may support debugging or higher-assurance modes.

### Example: Option

If `Option<Int>` is a sealed ADT:

```text
OptionInt = None | Some(Int)
```

Path after a successful presence test can assume:

```text
is-Some(x)
```

and project:

```text
value = Some.value(x)
```

Do not represent `None` as arbitrary integer sentinel unless that is a proven abstraction preserving all properties in scope.

### Failure modes

- Constraining object classes to source-known subclasses in an open world.
- Encoding mutable collection operations as pure functions of object identity alone.
- Treating pointer/identity equality as language value equality.
- Dropping NaN or exceptional values because tests did not cover them.
- Using one unconstrained `Int` to stand for “any Phalcom value.”
- Emitting raw solver text without a validated IR boundary.
