# Phalcom Current Contracts and Future Proof Integration

## Purpose and status discipline

This reference exists to prevent a specific architectural error: treating Phalcom's current contract implementation as though it were already a static verification system, or replacing its current runtime semantics while adding proof.

The repository must always be re-inspected before implementation. The observations below describe the repository state examined when this skill was deepened. Classify newer evidence according to repository status markers rather than this document.

## CURRENT: runtime contract weaving

Current repository documentation records `@requires`, `@ensures`, and `@invariant` as implemented compiler/weave features. Their predicates are compiled into ordinary runtime behavior rather than translated to a theorem prover.

The current architectural shape is approximately:

```text
source contract attribute
   -> compiler attribute expansion / AST weave
   -> ordinary Phalcom predicate/send code
   -> runtime check
   -> contract-specific Error on failure
```

This means a contract can have three distinct future consumers without three competing meanings:

```text
normative contract semantics
   ├─ runtime weave/checking
   ├─ static checker/prover obligations
   └─ reflection/docs/LSP metadata
```

The semantic predicate and its binding rules should be shared even if each consumer lowers it differently.

## CURRENT: `@requires`

Repository path to re-check:

```text
docs/spec/current/decorators/requires.md
phalcom-core/src/compiler/attributes.rs
```

Current documentation states that `@requires(pred)` is legal on methods, getters, and setters and injects an entry check. The runtime shape is a predicate send equivalent to:

```phalcom
pred.ifFalse { PreconditionError.new(message).raise() }
```

The current compiler also runs a conservative syntactic purity validator. This is explicitly a floor, not a semantic proof of purity. Static proving must not treat “passed current purity validation” as sufficient evidence that evaluating the predicate is mathematically pure, deterministic, non-throwing, non-yielding, or heap-independent.

Current compile modes distinguish whether guards are woven/stripped. A future prover must account for execution mode before using proof to remove or rely on runtime guards.

### Future proof mapping

For method verification:

```text
@requires P
```

becomes an entry assumption:

```text
Assume(P)
```

For a call to the method, it becomes an obligation:

```text
Assert(P[actual/formal])
```

Those two roles must never be reversed.

## CURRENT: `@ensures` and `old(...)`

Repository path to re-check:

```text
docs/spec/current/decorators/ensures.md
phalcom-core/src/compiler/attributes.rs
```

Current documentation describes a runtime rewrite that:

1. hoists `old(...)` subexpressions before the body;
2. rewrites explicit normal returns to bind the result and execute checks;
3. handles fall-through/tail return values similarly;
4. does not run the postcondition on a throwing exit.

This provides useful evidence about intended normal-exit scope, but a static prover should model `old` semantically as entry-state evaluation rather than blindly reuse AST weave mechanics. In a proof model:

```text
old(field read) -> select(H_entry, receiver, field)
```

whereas current runtime weaving may materialize a concrete value in a fresh local.

The current repository documents a divergence around the user-facing `result` binding versus an internal `__result` weave. A future proof frontend must follow whatever contract surface is ratified/fixed at implementation time; it must not encode an obsolete divergence as permanent language semantics.

### Future proof mapping

For each normal method exit with result `r` and final heap `H_out`:

```text
Assert(Q[result := r, old := H_entry, heap := H_out])
```

For verified/trusted callers after normal return:

```text
Assume(Q[actuals, result, H_before, H_after])
```

Throw/non-local/suspension outcomes require separate semantics.

## CURRENT: `@invariant`

Repository path to re-check:

```text
docs/spec/current/decorators/invariant.md
docs/adr/accepted/0052-invariant-reentrancy-scope-and-layout-confined-decorator-state.md
```

Current documentation records class-level invariants woven around non-static member boundaries and constructor exit. It also records a gap between a ratified receiver-scoped re-entrancy guard design and the current emitted weave in the inspected repository state.

This gap is especially important to static proof. A class invariant is meaningful only at ratified stable/visible boundaries. A prover must not assume it at arbitrary points inside a method, especially while the method temporarily mutates the receiver into an invariant-breaking intermediate state.

Callbacks and reentrancy matter. If user code can be called while the invariant is broken, the invariant may need restoration before the call-out or the model must prove the receiver is not observable/reentrant at that point.

## CURRENT DESIGN: contracts are not yet the static prover

The repository's canonical contract design explicitly characterizes the current family as runtime checks with a syntactic purity floor rather than an SMT/static-verification system. That does not conflict with a future static prover; it defines a migration constraint:

```text
Do not reinterpret existing runtime contracts silently.
Add a proof consumer that assigns proof roles to the same normative contracts.
```

If future static verification requires stronger restrictions on contract predicates, decide whether those restrictions apply:

- to all contract uses, changing language semantics;
- only to “statically provable” contracts, leaving runtime-only contracts valid;
- to a new proof-safe subset/effect guarantee.

Do not retroactively label current syntactic purity validation as a proof-safe effect system.

## Contract expression evaluation

The most important design decision before static proof is whether contract predicates are:

1. ordinary Phalcom expressions evaluated at runtime;
2. restricted pure expressions with statically enforced effect rules;
3. a separate logic language;
4. ordinary expressions plus proof models/contracts for invoked methods.

A separate logic language gives clean proof semantics but duplicates language concepts. Ordinary expressions maximize coherence but require proof-safe summaries for method sends and other effectful operations. A hybrid should be explicit about where semantic bridges occur.

Recommended architectural direction for investigation, not a ratified rule:

```text
ordinary Phalcom predicate syntax
+ semantic effect/purity facts
+ verified/trusted logical models for supported operations
+ Unknown/runtime fallback when proof safety is unavailable
```

This preserves one surface semantics while refusing unsound proof translations.

## Contract inheritance is a blocker for polymorphic modular proof

The inspected canonical contract design identifies contract inheritance/override behavior as unresolved and notes that current independent weaving does not enforce Liskov-style behavioral compatibility.

A future static prover cannot safely verify a dynamically dispatched caller against a base contract until a normative rule exists. Candidate semantic shapes include:

```text
base Pre  => override Pre

override Post => base Post

override MayWrite ⊆ base allowed effects

override exceptional/control effects remain within base guarantee
```

or an explicit protocol-contract mechanism independent of inherited implementation contracts.

The language-design decision belongs upstream of prover implementation. Do not hide the gap by selecting one target body.

## Metadata and semantic identity

Static proof should key contracts by semantic IDs, not by re-parsed attribute strings. Reflection/docs may retain source-printed predicates, but the prover needs resolved semantic expression structure plus source provenance.

A future normalized contract record might contain:

```text
ContractId
ContractKind
Owner CallableId/ClassId
Resolved predicate semantic representation
Bindings (`result`, `old`, formals, self)
SourceRange
Runtime weave policy
Proof eligibility/effect facts
Trust/verification state
Revision/content hash
```

This should be produced from shared semantic truth and consumed by runtime weaving/checker/LSP/prover as appropriate.

## Migration strategy

A safe staged integration is:

```text
Stage A
  preserve existing runtime behavior
  expose normalized semantic contract representation

Stage B
  generate static obligations for proof-safe local predicates
  keep runtime checks when Unknown

Stage C
  add verified/trusted callee summaries and polymorphic contract rule
  prove callers modularly

Stage D
  add heap/frame/object invariant reasoning

Stage E
  permit runtime-check elimination only with proof + stable assumptions
```

At each stage, differential tests should compare runtime contract behavior with static counterexamples for supported fragments.

## Repository review checklist

Before modifying contract/prover integration, inspect:

- current contract spec files and status labels;
- accepted ADRs concerning contract semantics/reentrancy;
- `phalcom-core/src/compiler/attributes.rs` or current replacement;
- contract fixtures and compile/runtime modes;
- semantic ID/snapshot architecture;
- current type/effect/call-summary machinery;
- reflection metadata implementation status;
- dynamic method/class mutation semantics;
- module/package revision model.

Then classify every statement in the implementation plan as CURRENT, RATIFIED/NORMATIVE, PROPOSED, FUTURE, or RECOMMENDATION.

## Pressure tests

1. Current runtime purity validator accepts a getter send. May static proof treat it as pure? **No, not without semantic evidence/model.**
2. `@ensures` passes at runtime on tests. Is the postcondition statically proved? **No.**
3. Override adds a stronger precondition. Can a caller proved against base contract remain valid? **Not until contract inheritance/substitutability semantics guarantee it.**
4. Release mode strips a postcondition guard. May code assume it anyway? **Only if the static proof/runtime mode guarantees it; a stripped runtime check itself provides no evidence.**
5. Contract metadata is reflected as text. Is re-parsing it suitable proof identity? **No; use resolved semantic IDs/representation.**
