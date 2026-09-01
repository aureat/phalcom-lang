# Phalcom Callable Semantics, LSP, and IDE Infrastructure
## Architecture Review, Code Review, Correctness Audit, and Verification Report

**Repository:** `aureat/phalcom-lang`  
**Scope:** callable semantic analysis, canonical declaration knowledge, call typing, source attachment, semantic presentation, editor semantic queries, LSP integration, IDE behavior, incremental correctness, and verification infrastructure.

---

# 1. Executive Summary

The callable architecture is moving in the correct direction and has improved substantially.

The strongest architectural decision is now clear and mostly enforced:

> `phalcom-semantic` owns semantic truth; editor and LSP layers consume protocol-neutral semantic products rather than independently reconstructing language semantics.

That is the correct foundation for Phalcom, especially given the language's planned support for callable families, typed dispatch, multidispatch, refinement, ADTs/GADTs, exact variant types, generics, and advisory inference.

Important migrations that are already complete or substantially complete include:

- canonical callable declaration knowledge;
- semantic parameter identity through `CallableParameterId`;
- parameter source attachment through `CallableParameterId -> SourceSiteId`;
- compiler-owned callable and field presentation;
- `EditorSemanticQuery::type_hints`;
- explicit-annotation metadata represented semantically rather than rediscovered entirely in LSP;
- migration away from obsolete direct signature fields such as `parameter.ty` and `signature.return_type` in LSP consumers;
- explicit separation of formal and advisory type knowledge;
- increasing use of editor-semantic query APIs instead of direct AST-driven LSP semantics.

The remaining problems are increasingly **semantic-fidelity problems**, rather than merely missing APIs. The infrastructure is becoming cleaner, but some editor-facing paths can still diverge from the compiler's actual language model.

The most important findings are:

1. **`Self` / self-relative type presentation can lose semantic meaning by degrading to `Unknown`.**
2. **Constructor recovery in editor semantics is still partly coupled to source spelling such as `new`, instead of relying exclusively on canonical constructor/callable identity.**
3. **Privileged/internal member visibility is incompletely represented in editor member-resolution paths.**
4. **Source-less/native callable presentation can fall back to a generic `Method` classification when a more precise callable semantic kind should be preserved.**
5. **Inherited member enumeration risks incorrect shadowing/override behavior unless it is a projection of effective dispatch/member resolution rather than a separate hierarchy merge.**
6. **Return-type publication policy should be centralized rather than locally recomposed by presentation consumers.**
7. **The editor query layer still needs a stronger invariant that every result is a projection of compiler semantics, never a parallel approximation.**
8. **Incremental invalidation needs systematic proof around callable signatures, parameter source attachment, presentation, and editor products.**
9. **CI/toolchain configuration has recently obscured real semantic failures and must remain part of correctness verification.**
10. **Transitional compatibility fields and migration scaffolding should be removed once consumers are migrated, otherwise dual authority can re-emerge.**

Overall, the core direction is good. The next phase should be consolidation, not another broad redesign.

---

# 2. Architectural Standard Used for the Review

The target architecture used to judge the implementation is:

```text
canonical identity
    ↓
canonical declaration knowledge
    ↓
formal proof / checking
    ↓
advisory knowledge
    ↓
source attachment
    ↓
EditorSemanticQuery
    ↓
protocol-neutral presentation
    ↓
LSP protocol rendering
```

`phalcom-semantic` should be the sole semantic authority for declaration identity, callable identity, parameter identity, declared types, inferred body-result types, generic specialization, overload/family resolution, member lookup, accessibility, receiver resolution, effective dispatch, formal proof, advisory knowledge, source attachment, and editor-facing semantic facts.

The LSP must not maintain a competing semantic model.

A second architectural invariant is the separation:

```text
formal knowledge != advisory knowledge
```

Formal knowledge participates in language acceptance and proof. Advisory knowledge can improve hover, inlay hints, completion, navigation, and diagnostics, but must never silently become a formal premise.

A third critical distinction is:

```text
declared return type
!=
inferred body/result type
```

A declared return annotation is a contract/premise. The inferred body result is evidence produced by analysis. Editor publication may choose how to present them, but that publication rule must be canonical and centralized.

---

# 3. Strong Design Decisions

## 3.1 `phalcom-semantic` as the sole semantic authority

The move away from a separate LSP semantic engine is exactly right.

The desired invariant should be:

> If the IDE knows a semantic fact, the compiler semantic layer must be able to explain where that fact came from.

This matters particularly for Phalcom because duplicating rules for callable families, refinement, typed dispatch, GADTs, specialization, and associated families inside an LSP would become unmaintainable.

## 3.2 Canonical callable and field signature tables

Moving declaration knowledge into canonical semantic tables establishes a sound ownership boundary:

```text
syntax/declaration collection
        ↓
canonical semantic signature
        ↓
all later consumers
```

Consumers should not need to recover formal types from syntax nodes.

This also improves incremental dependency tracking because semantic queries can depend on stable semantic products instead of arbitrary syntax traversals.

## 3.3 `CallableParameterId` as semantic identity

This is one of the best decisions in the current callable infrastructure.

Parameter identity must not be defined by source range, name, or parameter index. Source position is an attachment, not identity.

The mapping:

```text
CallableParameterId -> SourceSiteId
```

is therefore the correct architecture and should become the only supported semantic-to-source mapping for parameters.

## 3.4 Compiler-owned callable presentation

Moving callable presentation into `phalcom-semantic` is correct.

The LSP should consume a semantic product conceptually like:

```text
CallablePresentation {
    callable
    selector
    kind
    owner
    parameters
    return knowledge
    documentation
}
```

rather than receive checker internals and decide how parameter types, return types, labels, formal/advisory precedence, or semantic kinds should be represented.

## 3.5 Compiler-owned type hints

The existence of `EditorSemanticQuery::type_hints` is an important architectural improvement.

An LSP-side explicit-annotation scanner is dangerous because it duplicates semantic work, can miss syntax forms, leaks AST ownership into protocol code, and can diverge from compiler understanding.

The semantic layer should determine whether a hint exists and why; the LSP should only render it.

## 3.6 Explicit annotation metadata in source indexing

Recording whether a declaration has an explicit annotation during indexing/semantic attachment is superior to rediscovering it later.

For example:

```phalcom
const x = ...
const y: Foo = ...
```

contain different editor-relevant semantic information. Hint suppression should be derived from semantic/source metadata, not a fresh LSP AST traversal.

## 3.7 Formal/advisory separation

This is foundational and must be preserved.

If the editor observes that a value has been used as both `Circle` and `Rectangle`, that may justify advisory completion across both alternatives. It must not silently mean the formal type has become `Circle | Rectangle` unless the type system actually proves that.

---

# 4. Confirmed and High-Confidence Problems

## 4.1 `Self` presentation can collapse to `Unknown`

**Severity: High**

A self-relative formal type is meaningful semantic knowledge. Mapping it to `Unknown` in presentation is lossy.

For example:

```phalcom
class Builder {
    clone() -> Self
}
```

A hover showing `clone() -> Unknown` is incorrect. The compiler knows more than that.

`Self` should either remain an explicit presentation term or resolve contextually to the concrete owning/specialized type.

Recommended direction:

```text
FormalPresentation::SelfType(...)
```

or equivalent context-resolved representation.

The general rule should be: never map a meaningful formal type to `Unknown` merely because the presentation layer does not yet support it.

## 4.2 Constructor recovery is partly coupled to source spelling

**Severity: High**

Editor semantic code should not decide constructor semantics by noticing that source text looks like `new`.

The compiler should already know:

- whether a target is a constructor;
- which constructor identity it has;
- its owner;
- its family/member identity;
- its specialization.

Source syntax may locate the expression, but it should not define the semantic meaning.

This becomes especially important for aliases, imported constructors, enum variant constructors, alternate constructor syntax, generated code, desugaring, and callable constructor families.

Constructor-ness should flow from canonical semantic identity, such as a `CallableSemanticKind::Constructor`-like representation.

## 4.3 Internal/privileged visibility is incompletely represented in editor resolution

**Severity: High**

Visibility is part of semantic lookup, not just display filtering.

If editor member resolution does not carry the same access context as the checker, the compiler can accept a member that completion hides, or the IDE can expose a member the compiler rejects.

The editor should ask the same semantic accessibility machinery used by the checker, with a real context containing information such as module/package, enclosing type, current callable, receiver/self relation, and privileged access state.

A simplistic `include_private: bool` model is not sufficient if language visibility rules are richer.

## 4.4 Source-less/native callables can be classified too generically

**Severity: Medium**

Semantic callable kind should not be inferred from whether normal source metadata exists.

A callable may be a free function, method, static/type method, constructor, variant constructor, intrinsic, native callable, closure, callable family, or bound callable family.

Missing source information should only affect source-oriented metadata such as range/documentation attachment. It should not determine semantic callable kind.

## 4.5 Inherited member enumeration may mishandle override/shadowing precedence

**Severity: Potentially High**

A naive editor implementation that merges own members with base members and deduplicates names is not equivalent to effective semantic dispatch.

Phalcom members may represent overloads, family members, typed-dispatch candidates, overrides, hidden members, or specializations.

Failure modes include:

- showing overridden members that are not actually reachable;
- suppressing overloads that should coexist;
- choosing the wrong declaration for navigation;
- duplicate completion entries;
- showing a base signature when dispatch chooses a derived member.

Completion/member listing should therefore be a projection of a compiler-owned `effective_member_set(...)`-style operation, not an independent inheritance traversal.

## 4.6 Return publication logic should not be duplicated

**Severity: Medium**

A local rule such as:

```text
if inferred exists:
    show inferred
else:
    show declared
```

is too easy to reproduce in multiple places and will become incorrect for richer cases.

Future callable results may involve declared contracts plus narrower inferred results, generic specialization, `Never`, `Self`, refined/dependent result types, native declarations without bodies, and callable family composition.

One compiler-owned operation should define publishable return knowledge, for example conceptually:

```text
CallableSemanticSignature::published_return_knowledge()
```

Presentation should format the result, not decide its semantics.

---

# 5. Source Attachment and Identity

## 5.1 Exact local binding attachment is required

A previously identified source-attachment defect involved checker binding state using an entire declaration/statement range while source indexing used the exact bound-name range.

For example:

```text
checker range:  const foo = expression
source range:   foo
```

The wrong fix is containment matching, because containment is ambiguous.

The correct fix is to preserve the exact `Pattern::Name` range when constructing the binding seed/state. Then `BindingId -> SourceSiteId` attachment can remain exact and deterministic.

This directly affects hover, rename, references, inlay hints, type-at-position, and incremental updates.

## 5.2 Transitional `parameter_name_ranges` should disappear

If `CallableSourceInfo` still contains both positional name ranges and canonical `parameter_sites`, the former becomes architectural temptation.

Code such as:

```text
parameter_name_ranges[parameter.index]
```

recreates index-based identity.

The correct path is:

```text
parameter.id
    ↓
parameter_sites
    ↓
SourceSiteId
    ↓
range
```

Once all consumers are migrated, compatibility fields should be removed.

---

# 6. Callable Signature Semantics

## 6.1 Avoid ambiguous flattened fields such as `ty`

Earlier LSP breakage after the canonical model changed was useful evidence: consumers expected fields such as `parameter.ty` and `signature.return_type`.

A callable parameter can have several different notions of type:

- declared/formal type;
- normalized type;
- specialized type;
- advisory/inferred type;
- runtime representation type;
- presentation type.

A generic `ty` field obscures which one is being consumed.

Prefer semantically explicit APIs and types.

## 6.2 Declared return is not inferred return

For:

```phalcom
fn f() -> Animal {
    Dog()
}
```

the system may simultaneously know:

```text
declared return = Animal
body result     = Dog
```

These are not contradictory. The former is the contract; the latter is body evidence.

The internal model must preserve both even if a particular editor presentation chooses one view.

## 6.3 Native/intrinsic callables need full canonical signatures

Native callables should not become semantically second-class merely because they have no Phalcom body.

They still need canonical identity, parameter IDs, labels, arity/rest information, generic parameters, formal types, result type, visibility, callable kind, and documentation metadata where available.

Otherwise every consumer gains special cases.

---

# 7. Generic and Specialized Callable Analysis

The correct conceptual pipeline is:

```text
lookup callable/family
    ↓
obtain formal generic signature
    ↓
collect argument constraints
    ↓
solve substitutions
    ↓
specialize parameter/result types
    ↓
check argument compatibility
    ↓
produce specialized call result
```

Important invariants:

1. the generic declaration remains immutable canonical knowledge;
2. each call produces specialization/proof knowledge;
3. specialization does not overwrite the declaration signature;
4. advisory observations do not mutate formal generic constraints;
5. signature help may show a specialized view while retaining canonical declaration identity.

A likely future problem is cache-key design. A declaration presentation can be keyed by `CallableId`, but a specialized call-site presentation may also depend on substitution, receiver specialization, refinements, and dispatch selection.

`identity<Int>` and `identity<String>` cannot safely share a cache entry if the cached presentation contains instantiated types.

---

# 8. Callable Families and Overloads

Phalcom's callable-family model is richer than the ordinary LSP assumption:

```text
name -> overload list
```

Resolution may depend on:

```text
selector/family
+ receiver
+ labels
+ argument types
+ generic constraints
+ dispatch rules
-> viable callable set
```

Therefore family formation, candidate enumeration, viability, typed-dispatch filtering, ranking, ambiguity, specialization, and selected-callable identity must remain compiler-owned.

The LSP should not independently filter overloads based on argument count or labels beyond rendering protocol data already produced by semantic resolution.

---

# 9. Signature Help

Signature help is architecturally strongest when it consumes `CallablePresentation` or a specialized semantic equivalent.

The LSP should be responsible for:

- `SignatureInformation` construction;
- active signature index;
- active parameter index;
- protocol documentation encoding;
- source-position conversion.

It should not recompose formal/advisory precedence, interpret raw type terms, or decide which signatures are semantically viable.

---

# 10. Inlay Hints

The move to semantic-owned type hints is correct and should be kept strict.

The semantic layer should decide:

- whether a hint exists;
- which declaration it belongs to;
- whether an explicit annotation suppresses it;
- formal versus advisory source;
- stability/eligibility policy;
- obviousness suppression if that is language policy;
- exact source site.

The LSP should decide only protocol-level position, hint kind, formatting, and tooltip representation.

A suitable conceptual product is:

```text
EditorTypeHint {
    site
    range
    kind
    formal
    advisory
    target
}
```

---

# 11. Hover

Hover is a strong test of whether the architecture is truly canonical.

The desired eventual output can support full signatures such as:

```phalcom
method(
    _ param1: Type1,
    _ param2: Type2,
    *positionals: Type3,
    externalLabel1 label1: Type4,
    externalLabel2 label2: Type5,
    **labeled: Type6
) -> ReturnType

+5 overloads
```

Potential failure modes to test:

- advisory type displayed as if formally declared;
- generic substitutions lost;
- `Self` lost;
- exact variant type widened too early;
- callable family rendered as one arbitrary member;
- wrong overload count;
- declaration signature shown where call-site specialization is expected.

---

# 12. Completion

Completion is the IDE feature most likely to accidentally recreate a semantic engine.

For:

```phalcom
receiver.<cursor>
```

the semantic receiver result may be a formal exact type, union, refined type, exact variant, generic bound, advisory alternative set, or unknown.

The LSP must not inspect syntax and independently guess the receiver's type.

A particularly important case is advisory union-like completion:

```text
formal receiver knowledge = unknown
advisory observations     = Circle | Rectangle
        ↓
editor-only receiver alternatives
        ↓
completion across alternatives
```

This must not become a formal type union unless the checker proves it.

---

# 13. Definition, References, and Rename

Callable navigation should flow through canonical semantic target identity:

```text
source location
    ↓
semantic target
    ↓
CallableId / family/member identity
    ↓
definition source site
```

It should not rely on spelling, selector alone, range similarity, or reconstructed call shape.

For overloaded/family references, the project should define an explicit policy: selected callable, family declaration, or multiple definitions. It should not emerge accidentally from implementation details.

---

# 14. Visibility and Access Context

Visibility should be represented by a semantic context, conceptually:

```text
AccessContext {
    module
    package
    enclosing_type
    enclosing_callable
    receiver/self relation
    privileged capability
}
```

Then checker and editor queries can answer the same accessibility question.

This is superior to editor-specific booleans such as `include_private` that cannot encode the language's real rules.

---

# 15. Effective Dispatch Must Be Shared

Canonical declaration knowledge answers:

```text
what declarations exist?
```

Dispatch answers:

```text
which declaration(s) apply here?
```

Those responsibilities should stay separate.

Do not put dispatch-derived type authority into declaration tables, and do not let the LSP interpret a declaration list as if it were an effective dispatch result.

This becomes especially important for typed multiple dispatch.

---

# 16. `EditorSemanticQuery` as the Correct Boundary

`EditorSemanticQuery` is the right consolidation point for protocol-neutral editor semantics.

Appropriate query categories include:

- target at position;
- definition;
- references;
- access/member resolution;
- receiver resolution;
- completion candidates;
- callable presentation;
- field presentation;
- type hints;
- hover semantic information.

The key design rule is:

> An editor query must not silently implement an alternate typing rule.

If an editor feature needs semantic knowledge that the core semantic API does not expose, the normal answer should be to add/reuse a semantic operation, not approximate it inside `editor.rs`.

---

# 17. Presentation Layer Requirements

The presentation layer should preserve semantic distinctions while hiding internal representation details.

It should not collapse, unless explicitly required by the selected UI view:

```text
Self        -> Unknown
exact case  -> parent enum
formal      -> advisory
declared    -> inferred
family      -> arbitrary member
```

Phalcom may eventually benefit from distinct presentation modes for declaration, specialized call-site, hover, diagnostic, and reflection views, backed by shared lower-level type presentation.

---

# 18. Incremental Correctness

Canonical tables only solve half the problem. Query invalidation must also be complete.

Callable edits that should invalidate dependent semantic/editor products include:

- parameter annotation changes;
- return annotation changes;
- external label changes;
- rest/labeled-rest shape changes;
- generic parameter changes;
- visibility changes;
- callable-kind changes;
- owner changes;
- body-derived return changes;
- source-site movement where attachments depend on it.

Important dependency chains to test include:

```text
CallableSignature
    ↓
CallablePresentation
    ↓
hover / signature help
```

and:

```text
source declaration
    ↓
CallableParameterId -> SourceSiteId
    ↓
type hint position
```

Mutation-style incremental tests should prove these products refresh without requiring a full rebuild.

---

# 19. CI and Verification Infrastructure

Recent repository history showed that CI configuration can obscure actual semantic failures.

Stable jobs were affected by repository-local nightly/toolchain behavior and host-oriented flags. Each CI job should explicitly select its intended toolchain and environment.

Typical stable commands should use explicit stable selection and neutralized repository-specific flags where necessary, while Miri intentionally uses its pinned nightly.

CI should clearly separate:

1. formatting/toolchain failures;
2. compilation failures;
3. semantic tests;
4. LSP integration tests;
5. Clippy policy;
6. Miri/runtime-safety checks;
7. VS Code extension E2E.

A broken CI bootstrap should not mask a real callable/LSP regression.

---

# 20. Dead Code and Migration Residue

Warnings around unused semantic types/functions are worth architectural inspection rather than automatic suppression.

During a migration, dead semantic abstractions often mean either:

1. a planned new abstraction was never connected to the real path; or
2. an old path is still active and the new abstraction is ornamental.

Both are important signals.

Temporary migration scripts/workflows are acceptable during staged development, but the final state must be production Rust implementing the architecture and CI merely verifying it.

---

# 21. Future Risk: Specialized Presentation Cache Keys

A canonical declaration presentation can be keyed by `CallableId`.

A specialized call-site presentation may depend on:

```text
CallableId
Substitution
Receiver specialization
Call-site refinements
Dispatch selection
```

These should not share a declaration-only cache key if the presentation includes instantiated types.

This should be designed explicitly before generic/higher-kinded callable functionality grows further.

---

# 22. Future Risk: Exact Variant and GADT Result Types

With ADTs/GADTs becoming first-class, callable presentation and editor receiver resolution must preserve exact-case information.

For example, a declaration may return `Option<Int>` while body knowledge proves `Option::Some<Int>` at a particular point.

Premature widening will degrade hover, branch-local completion, pattern-refined calls, exhaustiveness diagnostics, and typed dispatch.

---

# 23. Future Risk: Family Identity vs Selected Member Identity

Phalcom increasingly treats callable families as first-class values/concepts.

IDE queries therefore need to distinguish:

```text
family identity
selected callable identity
bound family identity
```

This matters for hover, completion, references, rename, reflection, passing families as values, and typed dispatch.

A single concrete `CallableId` may not be sufficient for every editor semantic target.

---

# 24. Future Risk: Bound Callables

For:

```phalcom
const f = object.method
```

the semantic value is not simply the declaration `method`.

It is conceptually a bound callable containing receiver context and possibly specialization.

Hover and call typing must preserve that distinction, especially for receiver-dependent `Self` semantics.

---

# 25. Closures and Callable Symmetry

Phalcom's increasingly uniform model of functions, methods, constructors, variant constructors, closures, callable families, and bound callable families is a strength.

The editor/type infrastructure should exploit that symmetry rather than develop separate presentation and typing pipelines for each callable category.

A closure should ideally expose a callable signature representation that can reuse the same presentation machinery.

---

# 26. Labels and Rest Parameters Are Semantic Call Shape

External labels are not merely documentation if they participate in call shape.

Likewise, positional rest and labeled rest are semantic properties.

Canonical signatures should encode them directly, and call resolution should own their meaning. Signature help should merely render the semantic result.

The LSP should not infer call-shape compatibility from visible parameter counts.

---

# 27. `Unknown` vs `Dynamic`

Phalcom should preserve the distinction:

```text
Unknown != Dynamic
```

`Unknown` means analysis lacks sufficient information.

`Dynamic` means dynamic behavior is semantically permitted or explicitly represented.

Collapsing them in callable analysis or presentation makes diagnostics and editor behavior less truthful.

---

# 28. Diagnostics and Callable Proof Explanations

Callable resolution should connect to Phalcom's structured explanation architecture.

A failed call should eventually be able to explain candidate-by-candidate rejection:

```text
candidate A rejected: label mismatch
candidate B rejected: argument 2 requires Foo, got Bar
candidate C rejected: generic constraint not satisfied
```

The semantic layer should produce this reasoning. The LSP should render it rather than reconstruct it from syntax after failure.

---

# 29. Architectural Anti-Patterns to Forbid

The following should be treated as explicit architecture violations:

## 29.1 AST semantic reconstruction in LSP

Bad:

```text
walk AST in LSP to rediscover declarations/types/annotations
```

Acceptable:

```text
use syntax to locate a query position, then ask semantic layer
```

## 29.2 Source range as semantic identity

Bad:

```text
parameter = nth range
```

Correct:

```text
CallableParameterId -> SourceSiteId
```

## 29.3 LSP-owned declared/inferred precedence

Bad: protocol code decides which semantic type wins.

Correct: compiler returns published semantic knowledge.

## 29.4 Editor-owned inheritance merge

Bad: independently walk base classes and deduplicate names.

Correct: consume compiler-owned effective member resolution.

## 29.5 Advisory facts entering formal checking

Bad: observed call-site types become declaration truth.

Correct: advisory remains a separate evidence category.

## 29.6 Semantic kind inferred from syntax/source availability

Bad: source token or source presence determines callable kind.

Correct: callable kind is canonical semantic identity.

---

# 30. Recommended Remediation Order

## Priority 0 — Semantic fidelity

1. Preserve `Self` and other meaningful type terms through presentation.
2. Eliminate source-spelling constructor inference.
3. Unify visibility/accessibility through one semantic access context.
4. Audit effective member resolution so completion reflects real override/dispatch semantics.

## Priority 1 — Canonical publication

5. Centralize published return knowledge.
6. Make callable kind canonical for source and source-less callables.
7. Remove compatibility identity fields such as positional parameter-name ranges once migration is complete.

## Priority 2 — Editor architecture

8. Ensure hover, completion, signature help, hints, definition, and references all consume editor semantic products.
9. Prevent recursive AST semantic discovery from returning to the LSP layer.

## Priority 3 — Incremental correctness

10. Add mutation regressions for callable declarations and editor results.
11. Validate semantic query dependencies and stale-product invalidation.

## Priority 4 — Advanced callable semantics

12. Model family identity separately from selected concrete member identity.
13. Design specialization-aware presentation/cache keys.
14. Add bound-callable presentation and receiver-dependent specialization support.

---

# 31. Testing Plan

## 31.1 Semantic tests

Cover:

- canonical callable signature publication;
- declared versus inferred result;
- parameter identity;
- external labels;
- positional rest;
- labeled rest;
- native/intrinsic callables;
- visibility;
- generic specialization;
- `Self`;
- inheritance/override;
- callable-family candidate resolution.

## 31.2 Editor semantic query tests

Test protocol-neutral APIs directly, including:

```text
callable_presentation
field_presentation
type_hints
resolve_receiver_at
member/candidate queries
definition_at
references_of
```

These should prove IDE semantics without involving JSON-RPC timing.

## 31.3 LSP adapter tests

Restrict these mostly to protocol translation:

```text
semantic range -> LSP range
presentation -> SignatureInformation
semantic type hint -> InlayHint
semantic target -> Location
```

Do not re-test the type system inside the protocol layer.

## 31.4 End-to-end LSP tests

Use deterministic publication synchronization rather than ad-hoc polling.

Important regressions include:

- local binding edit;
- parameter annotation edit;
- return annotation edit;
- inherited override;
- internal/private access;
- advisory multi-alternative receiver;
- generic call specialization;
- native callable signature;
- constructor navigation/completion;
- `Self` return hover.

---

# 32. Suggested Architecture Assertions

Some invariants are important enough to test mechanically:

```text
phalcom-lsp must not import checker internals
```

```text
phalcom-lsp must not traverse AST to derive semantic type annotations
```

```text
no LSP consumer directly selects raw declared/inferred return fields
```

```text
no editor consumer identifies parameters by positional source-range indexing
```

```text
advisory facts cannot be passed to formal checker APIs as proof
```

```text
member completion routes through semantic effective-member resolution
```

These can be enforced with module visibility, targeted architecture tests, or grep-style CI assertions.

---

# 33. Verification Checklist

A credible callable/LSP completeness claim should require at least:

```bash
cargo +stable fmt --all -- --check
git diff --check

RUSTFLAGS="" cargo +stable check -p phalcom-semantic
RUSTFLAGS="" cargo +stable test -p phalcom-semantic

RUSTFLAGS="" cargo +stable check -p phalcom-lsp
RUSTFLAGS="" cargo +stable test -p phalcom-lsp --test integration

RUSTFLAGS="" cargo +stable check --workspace --all-targets
RUSTFLAGS="" cargo +stable test --workspace

RUSTFLAGS="" cargo +stable clippy --workspace --all-targets -- -D warnings
```

Then separately:

- pinned-nightly Miri;
- VS Code extension E2E;
- architecture assertions;
- incremental-edit regressions.

---

# 34. Overall Assessment

## Semantic architecture

**Good and improving rapidly.**

The project has chosen the correct semantic ownership boundary. Canonical declaration knowledge and editor semantic queries are the right foundation.

## Callable model

**Promising, but not fully normalized yet.**

The largest remaining concerns are self-relative types, effective dispatch, callable-kind identity, family/member distinction, and specialization-aware presentation.

## LSP architecture

**Substantially healthier than the legacy model.**

The remaining goal is to make it genuinely thin: protocol conversion and rendering, not language interpretation.

## IDE support

**Architecturally viable, with semantic-fidelity gaps.**

Hover, completion, signature help, hints, and navigation can become very strong if every one of them consumes canonical semantic query/presentation products.

## Incremental correctness

**Needs systematic proof.**

The architecture supports correct invalidation, but mutation regressions should prove it across callable edits.

## Future extensibility

**Strong if current boundaries are enforced.**

The design can support callable families, bound families, typed multidispatch, GADT refinement, exact variants, higher-order callable typing, and specialized editor views without another architectural rewrite.

---

# 35. Final Recommendation

Do not perform another broad callable/LSP redesign.

The core architecture is now good enough. The correct next step is a strict consolidation phase:

1. eliminate remaining semantic-fidelity gaps;
2. make effective member, visibility, constructor, and callable-kind semantics compiler-owned everywhere;
3. preserve `Self` and other rich type terms through presentation;
4. centralize publication rules;
5. remove transitional identity and AST-derived mechanisms;
6. prove incremental correctness;
7. add architecture tests that prevent semantic logic from migrating back into LSP.

The most important invariant for the next phase is:

> The editor must never know less precisely what a semantic fact means merely because it is being presented to an IDE, and it must never invent a semantic fact that the compiler itself did not produce.

If that invariant is enforced, the current callable infrastructure is a strong base for the more advanced callable-family, typed-dispatch, ADT/GADT, refinement, and higher-kinded features planned for Phalcom.
