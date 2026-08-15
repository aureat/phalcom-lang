# Facts, Provenance, Uncertainty, and Trust

## 1. A fact is more than a value

A semantic fact should answer at least:

```text
what is claimed?
about which semantic entity/program point?
under what domain/assumptions?
how strong is the claim?
why is it believed?
in which generation is it valid?
what dependencies can invalidate it?
```

Flattening these dimensions into one enum such as `Unknown | Class(String)` is convenient initially and expensive later.

## 2. Separate knowledge domains

Phalcom must preserve explicit bridges among:

```text
RuntimeShapeFact     approximation of runtime value categories
TypeFact             judgment in the Phalcom language type system
EffectFact           possible/required effects
ProofFact            proposition established under proof assumptions
OptimizationFact     property strong enough for a guarded/unguarded transform
RecoveryFact         source/analysis availability information
```

A bridge may derive one from another only when the relationship is specified. Example: a declared and checked final class type might imply a receiver class set under closed-world assumptions; ordinary runtime shape observation does not define subtyping.

## 3. Current fact representation

**CURRENT:** `facts.rs` defines `ValueShape`, `Confidence`, `FactOrigin`, and `InferredValue`. Shapes include unknown, instances, class objects, modules, tuples, records, collections, callables, method families, and bounded unions. `InferredValue` carries known boolean information, confidence, and bounded provenance. Confidence distinguishes exact, flow, interprocedural, and heuristic evidence. This is already substantially better than naked shape inference.

The skill should treat this as a current advisory domain. Future correctness domains may reuse IDs/provenance infrastructure but need their own formal relations.

## 4. Do not overload `Unknown`

At minimum distinguish conceptual states such as:

```text
Known(v)                  exact/sufficient abstract knowledge
Conservative(a)           sound over-approximation
Dynamic                    unknown by language choice/boundary
NotYetInferred             scheduling state, not semantic result
Blocked(dep)               dependency unavailable
Ambiguous(candidates)      several semantic alternatives
Inconsistent(evidence)     contradictory constraints/program error
BudgetExceeded(partial)    analysis stopped intentionally
Unsupported(feature)       implementation gap
Unreachable                bottom/no concrete execution
```

A particular consumer can project several of these to a compact display, but the engine should not lose distinctions needed for correctness or diagnostics.

## 5. Lattice order versus confidence order

Do not confuse abstract precision with evidence confidence.

For a may-shape domain:

```text
Instance(String) ⊑ Union(String, Number) ⊑ Unknown
```

means the right side represents at least as many possible runtime values. A confidence relation such as `Exact > Flow > Interprocedural > Heuristic` is a different axis. A highly confident `Unknown` and a heuristic `String` are incomparable in purpose: one may be sound but imprecise; the other useful but unsafe for rejection/optimization.

Represent them independently.

## 6. Provenance as an explanation graph

A provenance model can be a DAG:

```rust
enum EvidenceKind {
    Syntax,
    BindingWrite,
    BranchRefinement,
    CallReturn,
    CallArgument,
    FieldWrite,
    DeclaredType,
    NativeContract,
    Widening,
}

struct EvidenceNode {
    kind: EvidenceKind,
    origin: SourceOrSemanticOrigin,
    parents: SmallVec<[EvidenceId; 2]>,
}
```

Facts point to one or more evidence nodes. To bound memory, hash-cons identical evidence, retain only diagnostic-relevant frontier nodes, or use compact “reason trees” with caps. But define the truncation policy: dropping provenance must not change semantic facts; it changes explanation quality.

## 7. Worked diagnostic chain

Suppose future typing says parameter `name: String` and a call supplies a numeric expression:

```text
expected String
  because parameter `name` is declared String at Greeter.ph:4
found Number
  because argument is `x` at main.ph:12
  and x receives foo() at main.ph:8
  whose reachable return at util.ph:21 is Number
```

The semantic engine should provide binding/callable/source identities and flow provenance. The type checker owns the judgment `Number` not assignable to `String`; it should not need to reverse-engineer the causal flow.

## 8. Provenance and joins

Joining facts needs both semantic and evidence policies:

```text
join((a, pa), (b, pb)) = (a ⊔ b, merge_provenance(pa, pb))
```

The provenance merge may be bounded while the semantic join remains sound. If widening occurs, add explicit evidence:

```text
Widened {
  previous_alternatives: count,
  reason: UnionLimit,
}
```

so a diagnostic/debug tool can explain lost precision.

## 9. Contradiction versus uncertainty

If future type constraints require both `T <: String` and `Number <: T` in a nominal hierarchy where this is impossible, the result is inconsistent, not unknown. Likewise two unique declarations for a name may create ambiguity/error rather than “could be either” if Phalcom forbids duplicates.

Uncertainty means insufficient information; inconsistency means available information violates a rule. Checkers need this distinction to avoid silently accepting broken programs.

## 10. Trust and optimization

Every fact used for semantics-changing optimization needs a trust classification. Example:

```text
Advisory        editor-only
SoundOpenWorld  valid under current dynamic/open-world semantics
SoundClosedWorld valid only under an explicit closed-world snapshot
Proved          established by trusted proof chain
GuardedSpeculation safe if runtime guard/deopt validates assumption
```

An optimizer may use a heuristic receiver shape to choose an inline-cache order, because misprediction affects only performance. It may not remove fallback dispatch based on that heuristic.

## 11. Native/core contracts

Source analysis cannot inspect every Rust primitive semantically. Native/core operations need contracts describing relevant behavior: return domain, parameter expectations if any, effects, callback invocation, throwing/yielding, and reflection/mutation impact.

A missing native contract should be an opaque/dynamic boundary, not a guessed pure function. Contracts are part of the trusted semantic interface and need conformance tests against runtime behavior.

## 12. Serialization and snapshots

If facts are cached/persisted, include enough schema/version/dependency identity to reject stale data. Never deserialize a fact and treat it as valid solely because its source range still exists. Its semantic target, source/module revisions, dependency fingerprints, analysis version, and relevant language/configuration profile may matter.

## 13. Tests

- join precision and confidence independently;
- provenance retained through assignment -> call -> return;
- provenance truncation does not change semantic fact;
- widening is marked distinctly from language dynamic boundary;
- unreachable does not join as ordinary unknown state;
- blocked import becomes resolved when dependency appears;
- contradiction produces diagnostic state rather than silent unknown;
- optimizer rejects advisory fact for unguarded transform;
- native contract absence produces conservative effects;
- incremental invalidation removes provenance from deleted contributions.

## 14. Review questions

1. What domain is this fact in?
2. Is it sound, heuristic, proved, or speculative?
3. What does its top/unknown/bottom mean?
4. Is uncertainty cause preserved?
5. What evidence explains the fact?
6. Can evidence be retracted incrementally?
7. Is confidence being confused with abstract precision?
8. Could a consumer accidentally escalate this fact into a checker error or optimization proof?
