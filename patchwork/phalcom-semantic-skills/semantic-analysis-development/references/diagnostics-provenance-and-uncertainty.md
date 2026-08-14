# Diagnostics, Provenance, and Uncertainty

Semantic analysis is not complete when it can produce an answer. It should retain
enough information to explain the answer, distinguish certainty levels, and avoid
turning recovery or approximation into a false language error.

## 1. The diagnostic question

For every non-trivial fact, be able to answer:

> Why does the analyzer believe this?

Example:

```text
expected String
found Number
```

is much more useful when the semantic engine can reconstruct:

```text
parameter `name` requires String
  because `Person.rename(name: String)` declares it
argument has type Number
  because `count()` returns Number
```

Do not make UI code rediscover this chain from raw AST after inference.

## 2. Fact provenance versus diagnostic rendering

Keep them separate.

Semantic layer owns compact evidence:

```text
FactOrigin
ConstraintOrigin
DispatchEvidence
TypeConstraintOrigin
ProofObligationOrigin
```

Diagnostic layer owns wording, labels, notes, fix suggestions, and presentation.

This prevents semantic code from depending on LSP-specific or CLI-specific diagnostic
structures.

## 3. Current provenance baseline

Current `InferredValue` already retains:

- confidence;
- a bounded set of origins;
- syntax/binding/callable/call-site/constraint provenance.

Future domains should follow the same principle, but a different domain may need a
different evidence graph rather than reusing `FactOrigin` mechanically.

## 4. Evidence graph

For richer typing/proving, a flat list may become insufficient.

Conceptual structure:

```text
FactId
  conclusion
  rule
  parents: [FactId]
  source_sites: [SourceRange]
```

Example:

```text
x : String
  from branch refinement
    because condition proved x is Some<String>
      from declaration x : Option<String>
```

Keep the graph bounded or lazily materialized. IDE latency is more important than
retaining every derivation step eagerly.

## 5. Confidence is not soundness

Current confidence categories describe evidence strength for advisory runtime shapes.
They do not define checker soundness.

Do not write rules like:

```text
Confidence::Exact => statically proven type fact
```

An exact runtime shape fact and a language type theorem are different domains.

For future checker/prover results use explicit statuses such as:

```text
Proven
Disproven
Unknown
NotApplicable
```

or whatever the normative design chooses.

## 6. Unknown reasons

A single `Unknown` payload may be enough for a small abstract domain, but diagnostics
and debugging often need to know *why* precision was lost.

Possible analysis-only reasons:

```text
UnresolvedName
DynamicDispatch
UnionWidened
RecursionWidened
MissingAnnotation
MalformedSyntax
UnresolvedImport
NativeContractMissing
BudgetExceeded
UnsupportedConstruct
```

Do not expose this taxonomy as language semantics unless specified. It can be debug or
provenance metadata.

## 7. Recovery versus semantic failure

Distinguish:

- parser recovery;
- unresolved semantic identity;
- insufficient information;
- a proven invalid program;
- an internal analyzer bug.

Only the fourth is automatically a user-facing semantic error.

Examples:

```text
incomplete `foo(` while typing
```

should generally yield partial/unknown semantic facts, not a checker proof that the
program violates a type contract.

## 8. Diagnostics should attach to semantic identities

Prefer diagnostics that know:

- the exact declaration/use identity;
- the exact source site;
- related declaration site(s);
- the semantic rule violated.

Avoid searching by spelling at rendering time. Two identical names in different scopes
must not produce cross-linked diagnostics.

## 9. Primary and secondary spans

A good semantic diagnostic usually has:

- one primary span: where the user should look first;
- one or more secondary spans: evidence/context;
- concise explanation;
- optional fix/help.

For type mismatch:

```text
primary: argument expression
secondary: parameter annotation
secondary: source of inferred return, if useful
```

For duplicate/redefinition:

```text
primary: second declaration
secondary: original declaration
```

## 10. Constraint provenance

A future type solver should retain where constraints came from.

Example:

```text
T :> Int    from argument 1
T :> String from argument 2
T <: Number from declared bound
```

If solving fails, the diagnostic should name the competing constraints instead of
printing an opaque "cannot infer T".

Data structures may use compact IDs into a constraint arena rather than storing full
source structures on each variable.

## 11. Dispatch provenance

When a send resolves, retain enough to explain:

- receiver knowledge;
- selector;
- dispatch side;
- chosen declaration owner;
- superclass traversal if relevant;
- ambiguity/dynamic reason if not exact.

This supports hover, signature help, definition, and future checker diagnostics from a
shared result.

Do not re-run a different dispatch algorithm in each consumer just to obtain display
metadata.

## 12. Flow provenance

Flow-sensitive facts should identify the controlling reason when practical:

```text
x narrowed here because condition at range R was true
```

When a refinement is invalidated by assignment or unknown effect, debugging metadata
should make that loss visible in tests/logging.

## 13. Contract proof diagnostics

For `@requires`, `@ensures`, and invariants, distinguish:

- proved satisfied;
- proved violated;
- cannot prove;
- unreachable obligation;
- solver unsupported/timeout.

"Cannot prove" should normally be a weaker diagnostic category or informational result
than "proved violated", depending on checker mode.

Never map SMT `unknown` to `false`.

## 14. Dynamic boundaries

When optional typing meets dynamic code, diagnostics should identify the boundary:

```text
value became dynamically typed here
```

rather than inventing a later type mismatch with no explanation.

If runtime validation is inserted in a typed-runner mode, diagnostic provenance should
connect:

- source annotation;
- boundary/cast/check site;
- runtime value class/type evidence;
- originating call if available.

## 15. Native/core contracts

When a native primitive lacks a sufficiently precise semantic contract, report or log
that precision stopped at the native boundary. Do not infer a stronger return/effect
fact merely because today's Rust implementation happens to behave that way.

The source-visible/native contract is the authority.

## 16. Bounded provenance

Provenance can explode in loops and unions.

Use strategies such as:

- first/most relevant N source origins;
- canonical deduplication;
- summary nodes;
- lazy parent expansion;
- truncation markers;
- per-domain budgets.

Truncating explanation detail must not change the semantic conclusion.

## 17. Deterministic diagnostics

Stable tests and editor UX require deterministic ordering.

Order by semantic/source criteria, not hash iteration order:

- module path/identity;
- source offset;
- declaration order;
- selector canonical order;
- stable diagnostic code.

Solver traversal order should not cause diagnostic text to change randomly.

## 18. Diagnostic codes

Semantic errors should have stable machine-readable codes where the project diagnostic
system supports them.

Codes should identify the rule, not the English wording.

Good conceptual examples:

```text
type.argument_mismatch
type.inference_ambiguous
name.unresolved
class.already_defined
contract.precondition_unsatisfied
```

Do not encode source position or transient implementation details into codes.

## 19. Fixes and code actions

A semantic quick fix must be justified by exact semantic identity and transformation
preconditions.

Examples:

- add an explicit type argument when inference is genuinely under-constrained;
- qualify an ambiguous module/class reference;
- rename a shadowing binding only after references are identity-resolved.

Do not offer edits based on heuristic shape inference if the edit could change program
behavior.

## 20. Debugging semantic uncertainty

When an expected fact is missing, inspect in this order:

1. source occurrence/target;
2. scope/name resolution;
3. receiver/value fact;
4. dispatch resolution;
5. flow reachability;
6. call summary;
7. module dependency;
8. invalidation/revision;
9. widening/budget;
10. provenance truncation.

This is more effective than adding logs to hover/completion code.

## 21. Tests

For each semantic diagnostic family test:

- exact primary range;
- related ranges;
- stable code;
- deterministic ordering;
- malformed-source behavior;
- unknown versus proven error distinction;
- shadowed same-name identities;
- cross-module evidence;
- recursive/widened cases;
- native/dynamic boundaries;
- incremental edit removing the cause;
- incremental edit changing only related evidence.

## 22. Review questions

1. Can the fact explain its origin without re-parsing source?
2. Is confidence being confused with proof?
3. Is `Unknown` reason useful for debugging?
4. Is parser recovery accidentally surfaced as semantic invalidity?
5. Are source spans tied to the right semantic identities?
6. Is the diagnostic deterministic?
7. Are constraints/dispatch decisions explainable?
8. Does failure to prove stay distinct from disproval?
9. Is provenance bounded?
10. Can CLI/LSP/checker render the same semantic evidence differently without
    duplicating analysis?
