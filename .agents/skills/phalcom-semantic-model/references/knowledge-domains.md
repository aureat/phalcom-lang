# Knowledge Domains: Shape, Type, Proof, Effect, Uncertainty, and Provenance

Phalcom's semantic engine must relate several kinds of knowledge without collapsing them. The most important architectural discipline in this skill is:

```text
dynamic language semantics
!= runtime value representation
!= semantic-analysis approximation
!= language type
!= proof fact
!= optimization fact
```

The domains can inform each other through explicit bridges. They do not become equal merely because two consumers can render them with similar words.

## 1. Separate the subject from the epistemic state

A semantic fact has at least two dimensions:

```text
Fact = DomainValue × KnowledgeState × Provenance
```

The *domain value* says what is being claimed: possible runtime shapes, a type, an effect set, a proposition, a resolved target. The *knowledge state* says how/why the analyzer has that claim: exact, inferred, widened, blocked, ambiguous, budget-limited, dynamic by language choice, and so on.

This is better than creating one giant enum in which every domain must encode every uncertainty case independently.

A conceptual representation is:

```rust
struct Fact<T> {
    value: Option<T>,
    state: KnowledgeState,
    evidence: EvidenceRef,
}
```

This is a model, not a mandate to replace current structures. Current `InferredValue` already has a domain value (`ValueShape`), a strength (`Confidence`) and compact provenance.

## 2. CURRENT runtime-value shape domain

**CURRENT:** `phalcom-lsp/src/semantic/facts.rs` explicitly defines `ValueShape` as advisory runtime value knowledge, not a language type. It includes:

```text
Unknown
Instance(ClassId)
ClassObject(ClassId)
Module(ModuleId)
Tuple([...])
Record([(label, shape), ...])
List(element)
Set(element)
Map(key, value)
Range(bound)
Callable(CallableId)
Family(receiver, base)
Union([...])
```

Unions are bounded; the current cap is eight incompatible alternatives. `Unknown` means the shape analysis has no useful runtime-shape information. It is intentionally conservative.

This domain is useful for completion, hover, dispatch approximation, local/interprocedural editor inference and future type-inference seeding. It is not a correctness contract.

## 3. Runtime representation is another domain

The VM's concrete `Value` representation answers a different question:

> How is this value represented and manipulated during execution?

A tagged immediate integer, heap pointer, object handle or native resource can all have runtime representation details that do not belong in semantic `ValueShape`. Conversely, `ValueShape::Union` is an analysis object that no running program necessarily contains.

Do not couple semantic equality to VM bit representation. An optimizer may build a bridge from semantic facts to representation assumptions, but that bridge needs guards/deoptimization/invalidation appropriate to runtime mutability.

## 4. Language type domain

A future language type answers normative questions:

```text
Γ ⊢ e : T
A <: B
A assignable-to B
C conforms-to P
T[U/α]
```

A type representation may need:

- nominal class instance types;
- generic parameters and applied types;
- protocols/structural requirements;
- `Self`/recursive forms;
- union/intersection types if ratified;
- special dynamic/unknown/top/bottom-like types if ratified;
- variance and bounds;
- aliases/normalization;
- existential/skolem forms where inference requires them.

Do not stretch `ValueShape` into this algebra. A runtime shape may be an input to type synthesis; it is not the type itself.

## 5. Type syntax, resolved type, and solver metavariable differ

Preserve this pipeline:

```text
source annotation text
  -> type-expression syntax
  -> name-resolved type expression
  -> canonical/semantic type representation
  -> constraints over inference variables
  -> solved/substituted type where possible
```

An absent annotation is not the same source fact as an explicit dynamic annotation. An inference metavariable such as `?T42` is not a user-visible type. A type alias spelling is not necessarily the same representation as its normalized target even if the checker treats them as semantically equivalent in some relation.

The full mathematics belongs to `type-theory`; this skill owns the boundary and bridge.

## 6. Proof facts are propositions

A proof domain answers propositions about a program state:

```text
x != None
n > 0
index < list.size
variant(v) = Some
field initialized
call terminates under precondition P
```

A Hoare-style judgment is:

```text
{P} C {Q}
```

where `P` and `Q` are propositions, not types by default. Proof facts can justify a type refinement if the typing rules define such a bridge:

```text
Γ(x) = T | None
P proves x != None
-------------------------
refined Γ'(x) = T
```

A solver timeout, unsupported heap feature, bounded loop unrolling or absence of a counterexample is not proof. Proof status needs at least:

```text
Proved
Refuted(counterexample/evidence)
Unknown(reason)
```

The full proof machinery belongs to the static-prover skill.

## 7. Effect facts are behavioral summaries

Effects answer questions such as:

```text
may throw
may return non-locally
may yield/suspend
may block an OS thread
may mutate field/global state
may perform dynamic send
may invoke callable parameter N
may cross native/FFI boundary
may perform I/O
```

Effect domains often use may-analysis joins, so `may_throw` combines with logical OR. A future no-yield/no-block region could consume these facts, but "no effect observed in current analysis" is not equivalent to "effect impossible" unless analysis completeness establishes it.

Effects matter across closures, fibers, FFI, contracts and optimization. They should be reusable semantic summaries rather than attributes computed independently by each consumer.

## 8. Optimization facts have a stronger operational obligation

An optimizer may use facts such as:

```text
receiver class is exactly C under guard G
method table version is V
call target is monomorphic for this inline cache entry
field representation is stable while class layout version is L
```

These are neither language types nor proof facts by default. Their validity is operational:

```text
assumption + validity condition + fallback behavior
```

A speculative fact is safe only if violating it cannot change language semantics because a guard/deoptimization/invalidation mechanism restores the general path.

## 9. Knowledge states that must not collapse to `Unknown`

At minimum distinguish conceptually:

| State | Meaning | Typical consumer consequence |
|---|---|---|
| Exact | guaranteed by syntax/semantic rule | may support strong tooling claim |
| Flow-derived | valid from local flow | program-point-sensitive |
| Interprocedural | derived through summaries | depends on call graph/summaries |
| Conservative approximation | sound over-approximation | safe but imprecise |
| Widened | precision deliberately discarded | expose reason in debug/diagnostics |
| Heuristic | useful guess, not guaranteed | ranking/UI only unless policy says otherwise |
| Not-yet-inferred | work not completed | retry/defer, not semantic error |
| Dependency-missing | import/declaration unavailable | diagnostic/recovery state |
| Ambiguous | several candidates/facts remain | do not pick one silently |
| Inconsistent | requirements contradict | candidate correctness error |
| Budget-exhausted | analysis stopped by operational limit | conservative/unknown with reason |
| Recovery-blocked | malformed source prevented reliable fact | local suppression, preserve other facts |
| Deliberately dynamic | language/program opted out of static precision | checker policy boundary |
| Unreachable | no concrete state reaches here | bottom, not ignorance |

Several of these can render as "unknown" in a compact hover. They must remain distinguishable inside the semantic model when downstream behavior differs.

## 10. Knowledge-state ordering is not one total confidence scale

Do not assume all states fit on a single line:

```text
Exact > Flow > Interprocedural > Heuristic > Unknown
```

That ranking is useful for some current editor evidence, but future semantic states are often incomparable. For example:

- an exact *runtime shape* is not automatically stronger than a declared protocol type for a type-checking question;
- `BudgetExhausted` and `DependencyMissing` are different reasons for absence of proof;
- `Dynamic` is a language policy state, not "very low confidence";
- `Unreachable` is more precise than every reachable value fact in a state-set ordering, not less confident.

Represent precision in the domain where it has formal meaning; represent cause/status separately.

## 11. CURRENT confidence and provenance

**CURRENT:** `InferredValue` carries:

```text
shape: ValueShape
known_boolean: optional exact/refined boolean information
confidence: Exact | Flow | Interprocedural | Heuristic
provenance: bounded FactOrigin list
```

Current fact origins include syntax, binding, callable, call-site and constraint evidence. Joins retain a bounded sample of origins. This is a compact editor-oriented representation.

Do not assume these four confidence variants are the final ontology for typing/proving. Preserve the architectural principle: facts carry strength and origin, but normative proof status gets its own semantics.

## 12. Provenance should support explanation

A future type/checker diagnostic should be able to produce a causal chain resembling:

```text
expected String
  because parameter `name` is declared String

found Number
  because expression `x` resolves to binding #17
  whose current flow fact comes from assignment at line 8
  whose RHS calls `foo()`
  whose return summary includes Number
  because reachable return at line 42 constructs Number
```

Do not flatten this into "Number" at the first join and later reconstruct a story using source text heuristics.

A scalable representation can use evidence IDs:

```text
Fact -> EvidenceRef
EvidenceNode -> {kind, source range, semantic parent facts}
```

Hot editor mode can cap or sample evidence. Checker/prover modes can retain richer graphs. Keep semantic value and explanation-storage policy separable.

## 13. Bridge contracts

Every cross-domain conversion should be documented as a function with preconditions and precision loss.

### Shape -> type

Potentially sound when the typing specification authorizes it:

```text
exact Instance(C) -> nominal instance type C
exact ClassObject(C) -> class/meta-object type for C
exact tuple shape -> product/tuple type under ratified tuple typing rules
```

Unsafe shortcut:

```text
"all observed calls passed Int" -> normative parameter type Int
```

Use-site observations are open-world evidence unless the checker explicitly defines a closed-world inference mode.

### Type -> shape

A type can constrain runtime possibilities without naming one runtime class:

```text
Protocol P      -> many implementing classes
T | U           -> several runtime families
Box<T>          -> runtime class Box plus erased/reified parameter policy
Dynamic         -> intentionally unconstrained static knowledge
```

Therefore conversion may lose type structure.

### Proof -> type refinement

A proof fact can refine only through a trusted typing rule. Mutation/aliasing may invalidate the predicate; the refinement must be scoped to the validity region.

### Type -> runtime contract

If typed-runner mode dynamically checks a contract, specify:

```text
which type forms are reified/checkable
when the check executes
what runtime error occurs
how generic/protocol types are represented
what native/FFI boundaries guarantee
```

Static type metadata does not automatically imply runtime enforcement.

### Semantic fact -> optimizer assumption

Require:

```text
fact
+ proof/guard strength
+ runtime mutation model
+ invalidation/version condition
+ fallback/deopt behavior
```

Advisory LSP shape alone is insufficient for unguarded optimization.

## 14. Product facts and cross-domain coherence

A semantic query may need several domains simultaneously:

```text
ExpressionFacts {
  resolution: ResolvedTarget,
  runtime_shape: ShapeFact,
  declared_type: Option<TypeFact>,
  inferred_type: Option<TypeFact>,
  effects: EffectFact,
  proof_context: ProofFacts,
}
```

Do not force them into one enum. Coherence means they refer to the same semantic identities/program point/generation and any bridge relations have been validated.

A checker may report inconsistency when:

```text
declared type = String
runtime-shape evidence = Number
```

but whether that is a hard error depends on the checker semantics and strength/completeness of the evidence. The semantic layer should preserve enough metadata to make that decision correctly.

## 15. Recovery facts are not language facts

Editor recovery introduces information such as:

```text
parser inserted/missed token
member declaration incomplete
import path half-written
call selector incomplete
```

These are *source recovery facts*. They explain why semantic analysis is blocked or partial. They are not dynamic language behavior and must not leak into batch semantics as if invalid complete programs had valid alternate meanings.

Local recovery should be quarantined: malformed code in one method should not automatically turn unrelated module/class facts into generic unknowns.

## 16. Dynamic boundaries are explicit policy boundaries

Future optional typing needs to distinguish:

```text
no annotation written
explicit dynamic escape hatch
analysis has not inferred a type yet
analysis cannot infer because dependency is missing
analysis found multiple solutions
constraints are inconsistent
runtime value class is unknown to LSP
```

These states lead to different checker and IDE behavior. "Dynamic" should mean a language-level policy decision if Phalcom ratifies such a construct, not a garbage bin for analysis failure.

## 17. Information loss should be monotone and explainable

When a bridge or widening loses precision, record why:

```text
Shape Union(A..H) + I
  -> Unknown
  reason = union cap exceeded

Proof predicates > budget
  -> drop selected predicates
  reason = path budget

Dynamic selector construction
  -> unresolved target set
  reason = selector not statically constructible
```

A consumer can choose not to display this detail, but semantic debugging and future diagnostics benefit from having the cause available.

## 18. Testing obligations

Test domain distinctions directly:

- `Unknown` versus unreachable;
- absent annotation versus explicit dynamic annotation once available;
- unresolved versus ambiguous target;
- widened versus never-inferred fact;
- heuristic versus exact evidence;
- shape union versus language type union;
- proof `Unknown` versus `Refuted`;
- mutation invalidating a proof refinement;
- protocol type mapping to multiple runtime classes;
- native/FFI effect uncertainty;
- recovery in one region not poisoning unrelated exact facts;
- incremental full-rebuild equivalence preserving statuses/provenance where contractually observable.

Useful metamorphic property:

```text
inserting a correctly inferred explicit type annotation
preserves checker success and dynamic behavior
```

when the typing specification says the annotation is semantically compatible and does not alter dispatch.

## 19. Failure modes to reject

Reject these equations unless a normative specification explicitly proves them:

```text
ValueShape == Type
RuntimeClass == TypeId
Unknown == Any
Unknown == Dynamic
Unknown == Bottom
NoAnnotation == DynamicAnnotation
Heuristic == Proof
SolverTimeout == Refuted
NoCounterexample == Proved
TypeMetadata == DispatchIdentity
SourceRecovery == DynamicSemantics
```

## 20. Review questions

Before adding or bridging a knowledge domain, answer:

- What question does this domain answer?
- What concrete/runtime/static meaning does each value have?
- Which epistemic states can accompany it?
- What is top/bottom/unknown for this domain, if any?
- Is uncertainty due to program semantics, open-world behavior, recovery or budget?
- What evidence/provenance must survive?
- Which bridges to shape/type/proof/effect/runtime exist and are they total or partial?
- Does the bridge preserve soundness or intentionally produce an advisory heuristic?
- Which consumer is allowed to turn the fact into an error?
- Can runtime reflection/mutation invalidate the fact?
- Does a future language type system remain free to choose a different algebra?
