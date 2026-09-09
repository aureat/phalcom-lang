# Phalcom Type System & Semantic Analysis Correctness Audit

Perform a deep, adversarial audit of Phalcom’s type system and semantic-analysis implementation.

The objective is not merely to review code quality or verify that existing tests pass. Treat the implementation as potentially incorrect until its semantic invariants have been independently reconstructed and validated.

Your primary goals are to:

1. identify correctness bugs;
2. identify type-system unsoundness;
3. identify incomplete or inconsistent semantics;
4. find cases where accepted programs should be rejected;
5. find cases where valid programs are incorrectly rejected;
6. find inference paths that produce incorrect, unstable, or order-dependent results;
7. identify mismatches between the intended language model and the actual implementation;
8. identify architectural decisions that make future soundness difficult to preserve;
9. identify pathological complexity, recursive expansion, inference blowups, or semantic-analysis instability;
10. identify missing tests capable of exposing these problems.

The audit must be implementation-driven. Do not assume that existing specifications, plans, comments, tests, or diagnostics are correct. Use them as evidence of intent, then verify the implementation independently.

---

# 1. Establish the Actual Type-System Model

Before judging individual implementation details, reconstruct the type system that Phalcom actually implements.

Determine and document:

- the universe of semantic types;
- the distinction between declared types, inferred types, runtime types, applied types, and internal inference representations;
- nominal versus structural typing boundaries;
- gradual-typing semantics;
- the meaning and behavior of:
  - `Dynamic`;
  - `Unknown`;
  - inferred but unresolved types;
  - error/recovery types;
  - bottom / impossible types, if any;
  - `Unit`;
  - literal values versus their nominal types;
  - `Self`;
- generic type representation;
- applied generic types;
- generic parameter identity and scoping;
- generic substitution;
- variance, whether explicit or implicit;
- subtyping / assignability;
- equality versus compatibility versus coercion;
- union-like or sum-type behavior;
- record and row types;
- enum / variant constructor typing;
- callable types;
- method-family polymorphism;
- constructor polymorphism;
- class-side versus instance-side typing;
- reified types and runtime-visible type information;
- exhaustiveness and pattern-space semantics.

Produce a concise map of the central semantic relations used by the implementation, for example:

```text
Type equality
    ↓
Normalization / canonicalization
    ↓
Subtype / assignability
    ↓
Generic constraint solving
    ↓
Inference
    ↓
Call resolution
    ↓
Flow refinement
    ↓
Exhaustiveness
```

Do not force this exact architecture if the repository implements something different. Reconstruct what actually exists.

Explicitly identify cases where multiple subsystems appear to implement overlapping but subtly different notions of type compatibility.

---

# 2. Audit Core Type Identity and Representation

Inspect how semantic types are represented and identified.

Audit:

- `TypeId` identity;
- interning;
- canonicalization;
- structural equivalence;
- nominal identity;
- applied generic identity;
- recursive types;
- aliases, if present;
- instantiated generic constructors;
- record-row identity;
- function / callable identity;
- class versus metaclass identity;
- variant constructor identity;
- `Self` representation;
- singleton or literal types, if present.

Look specifically for bugs caused by confusing:

```text
same TypeId
```

with:

```text
semantically equivalent type
```

and the reverse.

Investigate whether equivalent applications such as:

```phalcom
List<Int>
List<Int>
```

are guaranteed to canonicalize to the same semantic identity where appropriate.

Then investigate whether semantically different applications can accidentally collide.

Pay special attention to caches, memoization tables, cycle guards, substitution maps, and inference state indexed by `TypeId`.

Look for problems involving recursively instantiated generic types where each recursive expansion creates a different `TypeId`, thereby defeating cycle detection.

---

# 3. Audit Generic Semantics

Generics require especially aggressive scrutiny.

Audit:

- declaration of generic parameters;
- lexical ownership of parameters;
- shadowing;
- parameter identity;
- nested generic scopes;
- generic methods;
- generic getters;
- generic setters;
- generic constructors;
- generic index getters/setters;
- generic class methods;
- enum-level generics;
- variant-local generics;
- GADT-like constructor-local type variables;
- method-family generics;
- inference of omitted generic arguments;
- explicit generic applications;
- partial generic applications, if supported;
- generic substitution;
- substitution composition;
- substitution underneath recursive types;
- generic constraint propagation;
- constraint merging;
- occurs checks;
- escaping type variables;
- skolem-like behavior where applicable;
- polymorphic values;
- monomorphization assumptions, if any;
- higher-rank boundaries;
- rank-1 method-family polymorphism.

Test whether generic variables originating in different declarations can accidentally unify because they share:

- names;
- indices;
- local IDs;
- reused arenas;
- declaration order.

Test nested examples such as:

```phalcom
class Outer<T> {
  map<T>(_ value: T) { ... }
}
```

and verify that the two `T`s are distinct.

Investigate whether substitution is capture-safe.

---

# 4. Audit Applied Generic Types and Runtime Reification

Phalcom permits some type information to survive into runtime semantics.

Audit the exact boundary between:

```text
semantic type
runtime class
runtime applied type
runtime object representation
```

Investigate examples such as:

```phalcom
List<Int>.new()
List<String>.new()
List.new()
```

Determine:

- what class each resulting object has;
- what applied type information survives;
- whether mutation is checked statically, dynamically, both, or neither;
- whether runtime reification agrees with compile-time typing;
- whether two applied types accidentally share mutable type metadata;
- whether specialized runtime representations preserve semantic identity;
- whether erased execution paths can violate statically proven generic invariants.

Look for unsound transitions between typed and untyped code.

Especially audit interactions with `Dynamic`.

A value passing through `Dynamic` must not corrupt semantic state or cause the analyzer to subsequently make invalid static proofs.

---

# 5. Audit Assignability and Subtyping

Locate every important implementation of:

- assignability;
- compatibility;
- subtype checks;
- equality;
- unification;
- coercion;
- constraint satisfaction.

Determine whether these relations are consistent.

Construct a relation matrix for representative types.

Include at least:

```text
Int
String
Unit
Dynamic
Unknown
Self
T
List<T>
List<Int>
List<Dynamic>
record types
functions
enums
enum variants
recursive generic types
```

Check algebraic properties where applicable:

- reflexivity;
- transitivity;
- symmetry where equality is intended;
- antisymmetry where relevant;
- substitution stability.

Search for non-transitive behavior such as:

```text
A assignable to B
B assignable to C
A not assignable to C
```

unless explicitly intended.

Search for directionality bugs where:

```rust
is_assignable(expected, actual)
```

is accidentally invoked as:

```rust
is_assignable(actual, expected)
```

These frequently cause subtle false positives.

---

# 6. Audit `Dynamic` and `Unknown`

Do not treat `Dynamic` and `Unknown` as interchangeable.

Reconstruct their intended semantics and then inspect every important path where either appears.

Determine:

- whether `Dynamic` is an explicit escape hatch;
- whether `Unknown` means inference failure, insufficient information, budget exhaustion, incomplete source, or something else;
- whether either can flow into arbitrary types;
- whether either suppresses diagnostics;
- whether either accidentally proves exhaustiveness;
- whether either satisfies arbitrary generic constraints;
- whether either participates in overload or method resolution;
- whether either causes unsound narrowing.

Look for error paths where the analyzer silently replaces a failed proof with `Unknown` and then later treats `Unknown` as success.

Distinguish deliberately gradual behavior from unsoundness.

---

# 7. Audit `Self`

Audit all semantics involving `Self`.

Examples include:

```phalcom
class Builder {
  configure(...) -> Self { self }
}
```

and subclassing.

Determine whether `Self` means:

- declaring class;
- current receiver's static type;
- most-derived subtype;
- an implicit bounded generic;
- something else.

Test:

```text
instance methods
class methods
constructors
inheritance
overrides
method families
generic classes
generic subclasses
applied types
```

Look for places where concrete `Future<T>` or another declaring type is incorrectly compared directly with `Self`.

Check whether return-type checking respects the intended covariance of receiver identity.

---

# 8. Audit Unit and Literal Typing

Investigate semantic distinctions between language syntax and semantic types.

For example, ensure the implementation does not incorrectly distinguish:

```phalcom
()
```

from:

```text
Unit
```

when they represent the same semantic value/type relationship.

Audit literal typing for:

- integers;
- floats;
- strings;
- booleans;
- unit;
- symbols;
- collection literals;
- record literals;
- enum constructors.

Check whether literal expressions produce canonical semantic types or accidental AST-specific pseudo-types.

---

# 9. Audit Enum, Variant, and GADT Semantics

This area should receive a dedicated investigation.

Audit:

- enum-level generic parameters;
- applied enums;
- constructor result types;
- variant payload types;
- variant-local generic parameters;
- GADT-style constructors;
- constructor type inference;
- constructor-local substitutions;
- pattern matching against refined constructors;
- propagation of constructor refinements into branch scopes;
- exhaustiveness after refinement.

For a type like:

```phalcom
enum Expression<F, A> {
  Literal(A)
  Map<B>(Expression<F, B>, F)
}
```

verify that recursive constructor-local generics cannot:

- escape their scope;
- unify with unrelated variables;
- cause infinite expansion;
- destroy exhaustiveness termination;
- produce invalid recursive substitutions.

Investigate whether variant constructors are typed as proper functions returning the enclosing applied enum.

Look for errors where:

```phalcom
Option<Int>::Some(42)
```

is treated as though `Some` itself were an `Int`, or where the constructor result type is incorrectly inferred from its payload.

---

# 10. Audit Pattern Matching, Refinement, and Exhaustiveness

Audit both correctness and termination.

Inspect:

- pattern typing;
- enum refinement;
- nested patterns;
- recursive patterns;
- generic constructors;
- GADTs;
- structural records;
- guards;
- wildcard handling;
- unreachable branches;
- impossible patterns;
- duplicate cases;
- exhaustiveness.

Investigate the internal representation of pattern spaces.

Check cycle detection carefully.

A cycle guard based only on exact `TypeId` is suspicious if generic recursion can repeatedly instantiate semantically related but distinct types.

Construct adversarial recursive types that repeatedly change generic arguments.

Examples:

```text
Expression<A>
→ Expression<List<A>>
→ Expression<List<List<A>>>
→ ...
```

or equivalent structures supported by the language.

Audit for:

- exponential expansion;
- nontermination;
- stack overflow;
- massive memory growth;
- arbitrary analysis-budget exhaustion;
- incorrect fallback to `Unknown`;
- false exhaustiveness;
- false non-exhaustiveness.

---

# 11. Audit Record Rows and Structural Typing

Audit:

- closed records;
- open records;
- row variables;
- row extension;
- row subtraction, if present;
- field presence constraints;
- field type constraints;
- duplicate labels;
- nested rows;
- generic row parameters;
- row polymorphism;
- structural assignability.

Check whether:

```phalcom
{ x: Int, y: String }
```

is assignable to:

```phalcom
{ x: Int, ...R }
```

under the intended semantics.

Test contradictory constraints such as requiring the same field to have incompatible types.

Look for:

- row-variable capture;
- duplicate-field unsoundness;
- lost constraints;
- order-dependent field matching;
- incorrect canonicalization due to hash-map order.

---

# 12. Audit Callable and Method Typing

Audit how callable types are represented and compared.

Include:

- positional parameters;
- labeled parameters;
- keyword parameters;
- nullary methods;
- getters;
- setters;
- index getters;
- index setters;
- constructors;
- class methods;
- instance methods;
- blocks / closures;
- bound methods;
- unbound methods;
- method families.

Phalcom's selector model makes this particularly important.

Verify that selector identity and type identity are not accidentally conflated.

Audit examples such as:

```text
User#login(password)
User::login::(password)

User.class#anonymous()
User.class::anonymous::()
```

Determine whether binding a receiver correctly transforms the callable type without changing unrelated generic variables.

Look for accidental loss of:

- receiver constraints;
- generic parameters;
- `Self`;
- labels;
- variance;
- return type.

---

# 13. Audit Method Resolution and Pattern-Based Selection

If method selection supports or is being prepared to support pattern-sensitive dispatch, inspect existing assumptions that method identity is determined solely by selector shape.

Search for architectural coupling between:

```text
selector lookup
call candidate discovery
type checking
pattern refinement
ambiguity detection
runtime dispatch
```

Identify places that would become incorrect if multiple methods share the same selector but differ by argument-pattern constraints.

Even if this feature is not yet implemented, flag assumptions that make future implementation unsafe.

---

# 14. Audit Inference

Treat inference as a constraint-solving system.

Trace representative programs from syntax through semantic results.

Audit:

- inference-variable creation;
- constraint generation;
- constraint ordering;
- solving;
- substitution;
- generalization;
- defaulting;
- unresolved constraints;
- conflicting constraints;
- diagnostics.

Look explicitly for order dependence.

Equivalent programs should not infer different types merely because expressions, fields, constraints, or declarations appear in a different order.

Test permutations.

Example:

```phalcom
foo(a: 1, b: "x")
foo(b: "x", a: 1)
```

where labels make the calls semantically equivalent.

Also test reordering of:

- generic constraints;
- record fields;
- enum branches;
- imported declarations.

Inspect union-find, worklists, or recursive solving algorithms for stale-state bugs.

---

# 15. Audit Constraint Solving

For each constraint kind, identify:

```text
producer
representation
solver
failure mode
diagnostic
```

Look for constraints that can silently disappear.

Investigate whether solving one constraint mutates types in a way that invalidates previously established conclusions.

Check:

- occurs checks;
- recursive unification;
- substitution cycles;
- constraint deduplication;
- contradiction detection;
- delayed constraints;
- unresolved variables;
- solver fixed points.

Create minimal examples that force cycles such as:

```text
T = List<T>
```

unless explicitly legal through recursive type machinery.

The solver must terminate and report the intended result.

---

# 16. Audit Flow-Sensitive Typing

If Phalcom performs refinement based on control flow, audit:

- branches;
- pattern matches;
- boolean predicates;
- early returns;
- loops;
- closures;
- captured variables;
- mutable variables;
- reassignment;
- exceptional control flow.

Check whether refinements survive beyond scopes where they are valid.

A classic unsoundness is:

```text
prove x has type A
capture x
mutate x elsewhere
continue relying on x : A
```

Look for equivalent Phalcom cases.

---

# 17. Audit Inheritance and Overrides

Audit:

- superclass substitution;
- inherited generics;
- overriding generic methods;
- return-type variance;
- argument variance;
- `Self`;
- class-side methods;
- constructors;
- inherited fields;
- method families.

Test multi-level examples:

```text
Base<T>
  ↓
Middle<U> : Base<List<U>>
  ↓
Concrete : Middle<Int>
```

Verify that inherited member types become:

```text
List<Int>
```

where appropriate.

Search for single-level substitution implementations that fail across multiple inheritance layers.

---

# 18. Audit Class-Side Typing and Metaclasses

Phalcom has Smalltalk-inspired class-side semantics.

Audit the relationship among:

```text
User
User.class
Metaclass
```

and their semantic types.

Verify:

- class methods resolve against the correct class-side type;
- generic applications preserve class-side type information;
- constructors are associated with the correct receiver;
- bound class methods have correct callable types;
- `Self` works correctly on class-side methods;
- subclass class-side dispatch respects inheritance.

Look for assumptions that collapse:

```text
the class object
the instance type represented by the class
the metaclass of that class object
```

into one semantic identity.

---

# 19. Audit Error Recovery

Semantic analyzers frequently become unsound because recovery state leaks into successful analysis.

Investigate:

- poison/error types;
- `Unknown`;
- placeholder substitutions;
- missing declarations;
- unresolved names;
- duplicate definitions;
- invalid generic applications.

After an error occurs, verify that later analysis:

- does not panic;
- does not produce spurious cascades unnecessarily;
- does not treat invalid state as proven;
- does not poison unrelated declarations;
- remains deterministic.

Construct malformed programs that continue into otherwise valid code.

---

# 20. Audit Diagnostics as Evidence of Internal Correctness

Diagnostics often expose deeper type-model bugs.

For suspicious errors, inspect not just wording but the underlying semantic values being compared.

Examples:

```text
required Unit
proven ()
```

or:

```text
required Self
proven Future<T>
```

are likely evidence that semantic normalization is missing or that distinct abstraction layers are being compared directly.

For every strange diagnostic, trace backward until the actual semantic mismatch is understood.

Do not "fix" these by special-casing message formatting unless the type relation itself is correct.

---

# 21. Audit Termination and Complexity

Semantic correctness includes predictable termination.

Identify algorithms with potentially unbounded expansion:

- recursive generic substitution;
- exhaustiveness decomposition;
- structural subtype comparison;
- row unification;
- recursive type normalization;
- generic constraint propagation;
- overload candidate exploration;
- method-family resolution.

For each such algorithm, identify:

```text
termination measure
cycle key
memoization key
complexity
budget behavior
failure behavior
```

A cycle guard is only correct if its key represents semantic recursion rather than merely incidental allocation identity.

Construct adversarial programs designed to create:

- deep recursive generic nesting;
- broad enum products;
- nested records;
- mutually recursive aliases/types;
- repeated structurally equivalent types with distinct IDs;
- competing generic constraints.

Look for exponential behavior even when termination technically occurs.

---

# 22. Audit Determinism

Semantic output should not depend on:

- hash-map iteration order;
- pointer addresses;
- arena allocation order;
- thread scheduling;
- file discovery order;
- declaration traversal order;
- import order where semantics should be order-independent.

Run suitable tests repeatedly where practical.

Look for unstable:

- inferred types;
- diagnostics;
- selected candidates;
- constraint-solving outcomes;
- exhaustiveness conclusions.

---

# 23. Cross-Subsystem Invariant Audit

Do not inspect semantic modules in isolation.

Trace types through:

```text
parser/AST representation
        ↓
semantic declaration
        ↓
type construction
        ↓
inference
        ↓
constraint solving
        ↓
call resolution
        ↓
pattern refinement
        ↓
lowering
        ↓
runtime-visible type metadata
```

The parser itself is not the target of this audit, but representation assumptions introduced there may matter if they reach semantic analysis.

Likewise, compiler lowering is relevant only where it depends on semantic conclusions or preserves/violates runtime type guarantees.

Identify every semantic fact that later runtime correctness assumes.

Examples:

```text
"this call is type-safe"
"this cast cannot fail"
"this constructor produces Option<Int>"
"this match is exhaustive"
"this field definitely exists"
"this generic argument is Int"
```

Verify that those facts are actually proven.

---

# 24. Adversarial Testing Strategy

Do not limit testing to conventional positive cases.

For every feature, generate tests in these classes:

### A. Positive canonical case

Expected valid usage.

### B. Minimal invalid case

The smallest program that must fail.

### C. Boundary case

Examples involving empty, singleton, recursive, or deepest valid structures.

### D. Cross-feature interaction

For example:

```text
GADT + recursive generic + exhaustiveness
record row + generic method
Self + inheritance + applied generic
Dynamic + generic mutation
variant-local generic + pattern refinement
```

### E. Semantic-equivalence permutation

Rewrite the program without changing meaning and verify the result is identical.

### F. Recovery case

Introduce an earlier semantic error and verify later unrelated analysis remains sane.

### G. Stress case

Exercise recursion, depth, breadth, or repeated substitutions.

### H. Regression case

Turn every discovered bug into a minimal permanent test.

---

# 25. Differential and Metamorphic Testing

Where no external reference implementation exists, use metamorphic testing.

Programs related by semantics-preserving transformations should yield equivalent type-analysis results.

Examples:

```text
reorder independent declarations;
rename generic variables;
rename local variables;
reorder labeled arguments;
reorder record fields;
introduce an identity helper;
replace explicit generic argument with inferable equivalent;
factor an expression into a temporary binding;
inline a temporary binding.
```

If these transformations change inferred types or semantic acceptance without a principled reason, investigate.

---

# 26. Soundness Hunt

Explicitly search for programs that:

1. are accepted by semantic analysis;
2. permit a runtime operation inconsistent with the proven static type.

Examples include:

```text
reading a String as Int;
inserting String into proven List<Int>;
calling a method absent from the proven receiver type;
constructing an invalid generic variant;
escaping a constructor-local generic variable;
incorrectly assuming an exhaustive match;
returning an incompatible type through Self;
using a structurally missing record field;
invalidly narrowing Dynamic/Unknown.
```

These are the highest-priority findings.

Attempt to construct actual executable proof-of-unsoundness programs where possible.

A runtime crash is not required for something to be unsound. Silent value corruption, invalid dispatch, or reaching a state that the type system claimed impossible also qualifies.

---

# 27. Completeness Hunt

Separately search for valid programs rejected by the analyzer.

Classify these as false negatives rather than unsoundness.

Examples:

```text
valid generic inference rejected;
equivalent Unit representation rejected;
valid Self return rejected;
correct constructor application rejected;
legal row-polymorphic call rejected.
```

Determine whether the cause is:

- missing normalization;
- incomplete solver;
- incorrect relation direction;
- missing substitution;
- lost generic scope;
- overly conservative approximation.

---

# 28. Architecture Audit

Evaluate whether the implementation has a coherent semantic kernel.

Look for duplicated logic across:

```text
assignability
unification
constraint solving
pattern matching
method resolution
exhaustiveness
diagnostics
```

If the language's type relation is independently reimplemented in several modules, identify divergence risk.

Prefer architecture where central invariants have a single authoritative implementation or clearly defined layered relations.

Identify technical debt likely to produce future soundness bugs.

Do not recommend abstraction merely for cleanliness. Recommend architectural changes only where they materially improve:

- correctness;
- auditability;
- termination;
- consistency;
- performance;
- extensibility.

---

# 29. Performance Audit

Assess semantic-analysis efficiency alongside correctness.

Look for:

- repeated type reconstruction;
- repeated generic substitution;
- unnecessary cloning;
- large hash-map churn;
- noncanonical temporary types;
- repeated structural comparisons;
- missed memoization;
- quadratic candidate resolution;
- recursive traversals lacking caches;
- per-call reconstruction of applied types.

Distinguish legitimate performance optimizations from shortcuts that weaken semantic correctness.

Correctness must win over premature optimization.

However, flag designs where canonicalization or memoization would improve both performance and correctness.

---

# 30. Existing Test Audit

Inspect the current test suite and determine what it actually proves.

Do not infer correctness merely from coverage count.

Identify:

- features with only happy-path coverage;
- diagnostics tested without semantic-state verification;
- test helpers that bypass real compiler paths;
- duplicated tests that add little assurance;
- important invariants with no direct tests;
- integration gaps between semantic subsystems.

Where Rust tests inspect internal semantic structures, verify that they assert meaningful invariants such as:

```text
exact inferred TypeId/type structure;
generic substitution contents;
constructor result type;
resolved declaration identity;
constraint state;
refined branch type;
row contents;
selected method-family member.
```

Do not rely solely on "compiles / does not compile".

---

# 31. Required Finding Format

For every substantive finding, report:

## [Severity] Finding title

**Category:** Soundness / Correctness / Completeness / Termination / Performance / Architecture / Diagnostics / Test Gap

**Location:** exact modules, files, functions, and relevant lines.

**Observed behavior:** what the implementation currently does.

**Expected invariant:** what must be true.

**Root cause:** explain the implementation-level mechanism producing the problem.

**Minimal reproducer:** preferably a Phalcom program.

**Impact:** explain what semantic guarantees are violated.

**Fix direction:** describe the correct architectural or algorithmic repair.

**Tests required:** exact regression tests that should be added.

Use severity:

- `Critical` — demonstrable type-system unsoundness, memory/runtime corruption potential, or fundamental invalid semantic proof.
- `High` — serious semantic incorrectness, nontermination, broadly reachable invalid inference, or major architectural soundness risk.
- `Medium` — important false rejection, edge-case inconsistency, unstable inference, diagnostic evidence of deeper modeling errors.
- `Low` — localized robustness, maintainability, or missing-test issue with limited semantic impact.

Do not inflate severity.

---

# 32. Required Final Report Structure

Produce the final audit in this order:

# Executive Summary

Give a concise assessment of the current semantic system:

- overall correctness confidence;
- soundness confidence;
- most dangerous subsystem;
- strongest part of the implementation;
- highest-priority fixes.

# Semantic Model Reconstructed

Describe what type system the code actually implements.

# Core Invariants

List the invariants that must hold across the implementation.

# Critical Findings

# High-Severity Findings

# Medium-Severity Findings

# Low-Severity Findings

# Soundness Proof-of-Concepts

Show any programs accepted by the analyzer that violate static guarantees.

# False-Rejection Cases

Show valid programs incorrectly rejected.

# Termination and Complexity Risks

# Architectural Assessment

Evaluate whether the semantic architecture is suitable for the intended language.

# Test-Suite Assessment

Explain what is currently missing.

# Proposed Regression Tests

Provide concrete test cases and where they should live.

# Prioritized Remediation Plan

Order fixes based on dependency and semantic risk, not convenience.

Use roughly:

```text
P0 — restore soundness
P1 — repair core semantic invariants
P2 — fix inference/completeness
P3 — harden termination and pathological cases
P4 — consolidate architecture
P5 — improve performance and diagnostics
```

# Confidence and Remaining Unknowns

Explicitly state areas that could not be fully proven from the available implementation.

---

# 33. Audit Discipline

During the audit:

- inspect implementation before trusting documentation;
- trace suspicious behavior through the full semantic path;
- distinguish symptoms from root causes;
- verify proposed fixes against neighboring semantics;
- do not patch around diagnostics when the underlying relation is wrong;
- do not assume `TypeId` identity means semantic identity;
- do not assume passing tests establish soundness;
- do not treat `Dynamic` as justification for arbitrary unsoundness;
- do not solve nontermination merely by lowering recursion budgets;
- do not introduce ad hoc special cases unless demanded by the language semantics;
- prefer small counterexamples that prove a general defect.

When you believe something is correct, explain why it is correct and what invariant guarantees it.

When you believe something is unsound, demonstrate the concrete semantic path that makes it unsound.

The standard for this audit is:

> Could a language implementer rely on the semantic analyzer's conclusions as proof obligations for compiler lowering and runtime execution?

Anything preventing that deserves investigation.

The final result should leave the repository with a clear understanding of:

1. what Phalcom's type system actually guarantees today;
2. which guarantees are currently violated;
3. which cases remain unproven;
4. exactly which fixes and regression tests are needed to make the semantic layer trustworthy.