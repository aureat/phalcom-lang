# Phalcom Integrated Runtime Architecture Audit

## 1. Objective

Perform a deep, implementation-driven audit of Phalcom's runtime execution architecture.

This is an integrated audit of:

1. **Runtime object/value model and dispatch**
2. **VM / interpreter execution machinery**
3. **Compiler lowering and bytecode generation insofar as they directly define runtime execution**

The audit is **not** a general review of the compiler frontend, parser, lexer, syntax, AST design, or unrelated semantic-analysis architecture.

Compiler investigation is in scope only where compiler behavior directly establishes or depends upon runtime contracts, including:

- lowering source/IR constructs into executable bytecode;
- bytecode instruction selection;
- operand encoding;
- stack effects;
- frame construction;
- call lowering;
- dispatch lowering;
- control-flow lowering;
- closure/capture lowering;
- exception/unwind lowering;
- object/value construction;
- constants;
- jumps and branches;
- return behavior;
- bytecode metadata;
- source/span metadata where it affects runtime behavior;
- compiler assumptions consumed by the VM;
- optimization decisions that alter runtime execution.

The central system under review is therefore:

```text
language operation
    ↓
compiler lowering
    ↓
bytecode / chunk representation
    ↓
instruction decoding
    ↓
VM execution
    ↓
Value / object representation
    ↓
method dispatch / calls / frames
    ↓
runtime result, mutation, return, or unwind
```

Treat this as one architecture.

---

# 2. Primary Audit Goal

Determine whether Phalcom's runtime foundation is:

- correct;
- internally coherent;
- robust under edge cases;
- architecturally sound;
- performant on hot execution paths;
- extensible toward the planned language;
- free of hidden representation assumptions;
- adequately tested;
- resistant to pathological runtime behavior;
- suitable for future optimization.

Do not merely identify questionable code.

For important findings, determine:

1. what invariant the implementation appears to rely upon;
2. whether that invariant is actually established;
3. where it is established;
4. where it is consumed;
5. whether all producers and consumers agree;
6. how the invariant can fail;
7. what observable failure follows;
8. whether tests cover the failure;
9. what the strongest practical remediation is.

---

# 3. Audit Philosophy

The repository is the source of truth.

Do not infer architecture primarily from comments, names, documentation, or intended design.

Trace actual implementations.

For significant runtime operations, reconstruct the full execution path across crate/module boundaries.

Prefer findings supported by concrete code paths over speculative concerns.

Do not recommend rewrites merely because another architecture is fashionable or theoretically cleaner.

Evaluate every recommendation against:

- Phalcom's current language model;
- existing implementation constraints;
- runtime performance;
- implementation complexity;
- future optimization potential;
- future typing/reification requirements;
- migration cost.

Preserve good architecture when it is already good.

---

# 4. Required Execution-Path Reconstruction

Before forming major architectural conclusions, reconstruct representative execution paths end-to-end.

At minimum investigate operations equivalent to:

```phalcom
42
```

```phalcom
object.foo
```

```phalcom
object.foo()
```

```phalcom
object.foo(x)
```

```phalcom
object.foo(label: x)
```

```phalcom
Class.foo(x)
```

```phalcom
Foo.new(...)
```

```phalcom
a + b
```

```phalcom
if condition { ... }
```

```phalcom
while condition { ... }
```

```phalcom
closure.call(x)
```

```phalcom
return value
```

and any relevant existing forms of:

- constructor invocation;
- getter invocation;
- setter invocation;
- index access;
- index assignment;
- enum/variant creation;
- matching where runtime bytecode participates;
- native method invocation;
- dynamic dispatch;
- superclass dispatch;
- inheritance;
- exceptions/errors/unwinding;
- closure capture;
- block invocation.

For each important path determine:

```text
source construct
→ lowered representation
→ emitted instructions
→ bytecode operands
→ stack state
→ instruction decoding
→ runtime lookup
→ dispatch
→ frame creation
→ argument binding
→ execution
→ result handling
```

Identify duplicated or unnecessary work along this path.

---

# 5. Evaluation Areas

## 5.1 Runtime Representation Invariants

Audit all important assumptions around runtime representation.

Evaluate:

- invariants of `Value`;
- object identity;
- class identity;
- runtime type identity;
- singleton identity;
- enum/variant identity;
- native object identity;
- reference versus immediate values;
- equality versus identity;
- mutability assumptions;
- null/sentinel/special-value representation;
- invalid/uninitialized states;
- whether impossible states are actually impossible;
- whether internal invariants can be violated through unusual execution paths.

Look specifically for representation invariants that are:

- documented but unenforced;
- enforced only in debug builds;
- enforced by convention;
- duplicated across modules;
- dependent upon instruction sequencing;
- vulnerable to malformed bytecode;
- vulnerable to future runtime changes.

---

# 5.2 `Value` Layout and Tagging

Audit the complete `Value` representation.

Evaluate:

- memory size;
- alignment;
- tagging strategy;
- discriminants;
- spare bits;
- immediate values;
- pointer representation;
- pointer provenance assumptions;
- integer range assumptions;
- floating-point representation if relevant;
- NaN boxing if relevant;
- sentinel values;
- representation of booleans;
- representation of `None`/nil-like values;
- heap objects;
- enum/variant values;
- native values;
- string representation;
- ownership semantics;
- cloning behavior;
- conversions between representations.

Search for:

- undefined or implementation-sensitive behavior;
- accidental truncation;
- sign-extension bugs;
- alignment assumptions;
- platform-width assumptions;
- uninitialized bits;
- invalid pointer states;
- overly expensive cloning;
- redundant heap allocation;
- unnecessary boxing.

Assess whether the representation supports future metadata optimizations safely.

---

# 5.3 Object and Instance Representation

Audit:

- instance layout;
- field storage;
- class reference storage;
- method-related metadata;
- object headers;
- native object wrappers;
- inheritance-related state;
- instance-variable lookup;
- field offsets;
- mutable versus immutable fields;
- storage allocation;
- resizing if applicable.

Determine the runtime cost of accessing an ordinary instance variable.

Look for:

- repeated class metadata lookup;
- hash-based field lookup on hot paths;
- duplicated class information;
- avoidable pointer chasing;
- poor locality;
- oversized object headers;
- per-instance data that should be class-level;
- class-level data repeatedly reconstructed per instance.

---

# 5.4 Class Identity and Runtime Type Identity

Determine exactly what constitutes runtime class/type identity.

Audit:

- class objects;
- metaclass or class-side behavior;
- class IDs;
- runtime IDs;
- native classes;
- inheritance;
- enum classes;
- variant identities;
- dynamically created or loaded classes;
- module interactions;
- equality of class identity;
- identity stability across compilation/loading boundaries.

Investigate whether today's identity model can cleanly support future:

```text
List
List<Int>
Option<String>
Some<Int>
```

or equivalent applied/reified runtime type concepts.

Do not assume that all generic types must become runtime-distinct. Instead identify which existing assumptions would break if some applied types become runtime-reified.

---

# 5.5 Method Lookup and Dispatch

Reconstruct dispatch completely.

Audit:

- selector representation;
- selector identity;
- method-table representation;
- lookup key construction;
- inheritance traversal;
- class-side versus instance-side dispatch;
- native dispatch;
- getter dispatch;
- setter dispatch;
- operators;
- constructors;
- closure invocation;
- index access;
- super dispatch;
- missing-method behavior if present.

Determine exactly how many runtime operations an ordinary method send requires.

Identify:

- hashes computed;
- maps queried;
- pointer dereferences;
- allocations;
- temporary values;
- selector conversions;
- class lookups;
- inheritance traversals;
- cache accesses.

Search aggressively for:

- repeated selector hashing;
- string-based runtime lookup;
- unnecessary allocation of dispatch keys;
- repeated reconstruction of data known at compile time;
- duplicated lookup work;
- dispatch paths that unexpectedly differ;
- correctness differences between native and language-defined methods.

---

# 5.6 Dispatch Caching and Inline Caches

Audit any existing caching mechanisms.

Evaluate:

- monomorphic inline caches;
- polymorphic inline caches;
- megamorphic behavior;
- per-call-site caching;
- per-class caching;
- selector caches;
- inheritance invalidation;
- method replacement invalidation if applicable;
- class mutation;
- module reload implications;
- cache key quality;
- cache memory overhead;
- synchronization requirements.

If inline caches do not currently exist, determine whether the architecture naturally supports them.

Identify what stable identifiers would be required.

Assess whether a `DispatchKey`, class ID, method ID, call-site ID, or related representation could materially reduce dispatch cost.

Do not assume such a cache is desirable without tracing the current path.

---

# 5.7 VM Instruction Representation

Audit the bytecode instruction representation itself.

Evaluate:

- opcode representation;
- operand width;
- instruction width;
- packed versus variable-width instructions;
- alignment;
- decoding cost;
- instruction locality;
- constant-pool access;
- branch targets;
- relative versus absolute jumps;
- index widths;
- overflow handling;
- chunk-size limits;
- method-size limits;
- constant-count limits;
- register/stack index limits where relevant.

Look for boundary bugs at:

```text
0
1
max - 1
max
max + 1
```

for every encoded integer width.

Search for truncating casts and unchecked arithmetic during bytecode emission.

---

# 5.8 Compiler ↔ VM Bytecode Contract

This is a major focus.

For every important opcode, determine:

- what stack/state it expects before execution;
- what stack/state it produces;
- operand meaning;
- operand valid range;
- whether operands are checked;
- what runtime types it assumes;
- what compiler invariant guarantees those types;
- what malformed or inconsistent bytecode would do.

Find cases where:

> the compiler assumes the VM will handle something that it does not,

or:

> the VM assumes the compiler can never emit something that the compiler actually can.

Search for opcode behavior duplicated independently in emitter/compiler and VM code.

Identify places where stack effects or operand semantics are implicit rather than encoded/tested.

---

# 5.9 Stack Discipline

Audit stack behavior rigorously.

Evaluate:

- push/pop balance;
- temporary values;
- receiver placement;
- argument placement;
- return value placement;
- branch joins;
- loops;
- early returns;
- calls;
- nested calls;
- closures;
- exception paths;
- constructors;
- native calls.

Look specifically for:

- off-by-one errors;
- underflow;
- stale values remaining on stack;
- incorrect stack restoration;
- differing stack shape at control-flow joins;
- stack growth over loops;
- recursion limits;
- unchecked stack capacity.

For complex opcodes, derive their stack effect explicitly.

Example:

```text
CALL argc=2

before:
[..., receiver, arg0, arg1]

after:
[..., result]
```

Verify that compiler and VM agree.

---

# 5.10 Frames and Function Calls

Audit frame representation and call behavior.

Evaluate:

- frame allocation;
- frame reuse;
- frame layout;
- instruction pointer representation;
- base pointer;
- local-variable addressing;
- argument addressing;
- receiver addressing;
- return address;
- closure environment;
- stack ownership;
- frame teardown.

Search for:

- unnecessary frame heap allocation;
- frame cloning;
- redundant metadata;
- excessive indirection;
- large frame structures;
- incorrect frame restoration;
- recursion edge cases;
- native/language call inconsistencies.

Determine the runtime cost of one ordinary function/method call.

---

# 5.11 Argument Binding

Audit:

- positional arguments;
- labeled arguments;
- selector labels;
- arity;
- default behavior if present;
- variadics if present;
- constructors;
- native functions;
- closures.

Look for edge cases involving:

- zero arguments;
- maximum supported arguments;
- duplicate labels;
- argument ordering;
- receiver inclusion/exclusion;
- class-side calls;
- malformed call bytecode;
- compiler/runtime disagreement about arity.

---

# 5.12 Control Flow

Audit bytecode lowering and execution for:

- conditional branches;
- loops;
- short-circuit boolean operations;
- match-related branching where runtime-relevant;
- early return;
- break;
- continue;
- nested loops;
- nested branching.

Verify jump offset computation around:

- forward jumps;
- backward jumps;
- empty blocks;
- first instruction;
- last instruction;
- largest encodable offset.

Search for arithmetic overflow and sign errors.

---

# 5.13 Closures and Captures

Audit closure representation and runtime execution.

Evaluate:

- capture discovery insofar as it affects runtime lowering;
- capture storage;
- closed-over locals;
- nested closures;
- mutable captures;
- capture lifetime;
- stack-to-heap promotion if applicable;
- recursive closures;
- closure invocation;
- receiver capture;
- environment allocation.

Look for:

- use-after-frame behavior;
- accidental copying instead of reference semantics;
- excessive environment allocation;
- incorrect capture indexing;
- capture aliasing bugs;
- nested closure bugs.

---

# 5.14 Constructors and Object Initialization

Audit construction end-to-end.

Investigate:

- allocation;
- zero/uninitialized state;
- constructor dispatch;
- field initialization;
- failure during initialization;
- inheritance;
- native classes;
- constructor return handling;
- constructor methods returning unexpected values;
- partially initialized objects.

Determine whether partially initialized instances can escape.

---

# 5.15 Enum and Variant Runtime Behavior

Audit runtime representation and bytecode interaction for enums and variants.

Evaluate:

- variant identity;
- tag representation;
- payload layout;
- field access;
- construction;
- matching;
- equality;
- runtime class/type identity.

Check whether generic enum evolution could invalidate current assumptions.

---

# 5.16 Native Runtime Boundary

Audit all native/runtime FFI-like boundaries.

Evaluate:

- argument validation;
- return representation;
- error propagation;
- ownership;
- lifetime;
- panics;
- conversion between native Rust values and Phalcom `Value`;
- stack discipline;
- native method lookup;
- native constructor behavior.

Look for cases where native code can violate invariants impossible in ordinary Phalcom bytecode.

---

# 5.17 Error, Exception, Panic and Unwind Behavior

Audit all abnormal execution paths.

Distinguish clearly between:

- language-level errors;
- language exceptions;
- VM runtime errors;
- compiler bugs;
- internal invariant violations;
- Rust panics.

Investigate:

- stack restoration;
- frame cleanup;
- resource cleanup;
- nested calls;
- native boundaries;
- constructor failure;
- closure invocation failure.

Look for runtime inputs that can produce Rust panics instead of controlled language/VM errors.

---

# 5.18 Allocation Behavior

Identify allocation on hot paths.

Audit allocations involving:

- values;
- objects;
- frames;
- selectors;
- dispatch keys;
- strings;
- arrays/vectors;
- closures;
- argument lists;
- method lookup;
- constants;
- temporary runtime metadata.

For each frequent allocation determine whether it is:

```text
necessary
amortized
avoidable
cacheable
stack-allocatable
representable inline
```

Do not optimize cold paths merely because allocation exists.

---

# 5.19 Pointer Chasing and Cache Locality

Trace memory accesses through important operations.

Evaluate:

- object → class;
- class → method map;
- map → method;
- method → chunk;
- frame → chunk;
- chunk → constants;
- value → heap object;
- enum → payload;
- closure → environment.

Identify deeply indirect paths.

Assess opportunities for:

- stable IDs;
- direct indexing;
- compact metadata;
- contiguous storage;
- interned selectors;
- cached dispatch targets;
- improved object headers.

---

# 5.20 Hashing and Map Lookups on Hot Paths

Locate runtime `HashMap`/equivalent use.

For each hot lookup determine:

- key;
- hash frequency;
- lookup frequency;
- expected table size;
- whether key is already interned;
- whether direct indexing could replace hashing;
- whether lookup happens once or repeatedly.

Pay particular attention to:

- method dispatch;
- class lookup;
- selector lookup;
- field lookup;
- constant lookup;
- native lookup.

---

# 5.21 Canonicalization and Stable IDs

Audit opportunities for replacing reconstructed structural values with canonical IDs.

Relevant candidates may include:

- selectors;
- classes;
- methods;
- chunks;
- call sites;
- strings;
- runtime types;
- applied types;
- native functions.

Evaluate whether canonicalization would improve:

- equality;
- hashing;
- cache keys;
- cycle detection;
- dispatch;
- memory;
- metadata storage.

Also identify where canonicalization could introduce global contention or lifetime complexity.

---

# 5.22 Ownership and Lifetime Architecture

Audit Rust ownership choices that materially affect runtime architecture.

Look for:

- unnecessary `Arc`;
- unnecessary cloning;
- long-lived references;
- cycles;
- excessive shared ownership;
- mutexes/locks on runtime paths;
- unnecessarily heap-owned immutable structures;
- lifetime workarounds obscuring ownership.

Do not flag `Arc` or cloning mechanically. Determine measured architectural consequence.

---

# 5.23 Metadata Duplication

Search for the same information represented in multiple places.

Examples:

- class identity;
- method identity;
- selector information;
- arity;
- source locations;
- stack metadata;
- frame metadata;
- type metadata;
- dispatch metadata.

Determine:

- whether copies can diverge;
- whether duplication is deliberate;
- whether one representation can become canonical;
- whether duplication increases object/frame size.

---

# 5.24 Branch Predictability and Interpreter Dispatch

Audit the interpreter loop.

Evaluate:

- opcode dispatch mechanism;
- match/switch structure;
- hot versus cold instructions;
- error checks;
- branch-heavy opcodes;
- instruction decoding;
- repeated bounds checks;
- fast-path versus slow-path separation.

Do not recommend computed goto, threaded interpretation, JIT, or similar techniques automatically.

First establish whether current dispatch overhead is significant relative to operation cost.

---

# 5.25 Correctness Across Runtime Object Categories

Explicitly verify important operations across:

- primitive/immediate values;
- normal instances;
- classes;
- class-side objects;
- inherited classes;
- native classes;
- enums;
- variants;
- closures;
- collections;
- strings;
- numeric objects;
- singleton-like runtime values.

Find paths that accidentally special-case one category incorrectly.

---

# 5.26 Boundary and Pathological Inputs

Search systematically for bugs around:

- empty chunks;
- empty functions;
- zero arguments;
- maximum arguments;
- maximum locals;
- maximum constants;
- maximum jump offsets;
- deeply nested calls;
- deep inheritance;
- recursive dispatch;
- recursive closures;
- very large methods;
- large constant pools;
- repeated loops;
- malformed internal bytecode;
- absent methods;
- wrong arity;
- invalid operands.

Where integer widths define limits, explicitly test boundary values.

---

# 5.27 Future Runtime Reification Compatibility

Phalcom may increasingly preserve selected type information into runtime.

Audit current assumptions for compatibility with:

- generic classes;
- applied types;
- runtime-reflective types;
- specialized collections;
- typed dispatch;
- generic constructors;
- generic enum variants.

Identify architecture that assumes:

```text
runtime class == complete runtime type
```

if that assumption may eventually become false.

Do not redesign the runtime type system in this audit.

Instead report incompatibilities and sensible extension points.

---

# 5.28 Compiler/Runtime Architectural Coupling

Determine whether boundaries are appropriately placed.

Identify:

- VM knowledge embedded unnecessarily in compiler modules;
- compiler knowledge embedded unnecessarily in VM modules;
- duplicated opcode semantics;
- runtime representation leaked widely;
- bytecode representation leaking into unrelated layers;
- fragile dependency direction;
- abstractions that obscure rather than clarify invariants.

Distinguish harmful coupling from deliberate performance-oriented coupling.

---

# 5.29 Latent Bugs and Unsound Assumptions

Actively search for code that is technically correct only under undocumented assumptions.

Examples:

- unchecked indexing;
- `unwrap()` on runtime-dependent state;
- impossible-state assumptions;
- unchecked casts;
- integer overflow;
- stale cache assumptions;
- identity assumptions;
- stack-shape assumptions;
- instruction sequencing assumptions;
- class hierarchy assumptions;
- bytecode validity assumptions.

Attempt to construct concrete failure scenarios.

---

# 5.30 Test Quality and Missing Tests

Evaluate tests not simply by count but by what invariants they establish.

Identify missing:

- unit tests;
- boundary tests;
- integration tests;
- compiler ↔ VM contract tests;
- runtime execution tests;
- malformed-bytecode tests;
- regression tests;
- property tests;
- fuzzing candidates.

For every serious issue, specify the regression test that should exist.

---

# 5.31 Performance Architecture

Identify optimization opportunities at three levels.

### Level 1 — Waste removal

Examples:

- unnecessary allocation;
- duplicate hashing;
- repeated lookup;
- cloning;
- redundant conversions.

### Level 2 — Representation improvement

Examples:

- stable IDs;
- compact frames;
- direct indexing;
- improved object headers;
- interned selectors.

### Level 3 — Runtime execution architecture

Examples:

- inline caches;
- specialized opcodes;
- instruction fusion;
- quickening;
- adaptive specialization.

Prefer Level 1 and Level 2 improvements where they capture most of the benefit with much lower complexity.

---

# 6. Issue Classification

Every finding must receive:

```text
Severity
Category
Confidence
Runtime impact
Fix priority
```

## Severity

### Critical

Can cause:

- memory unsafety;
- corruption;
- incorrect execution;
- severe semantic violation;
- uncontrolled runtime failure;
- architecture that fundamentally blocks core language requirements.

### High

Can cause:

- realistic incorrect behavior;
- major performance pathology;
- incorrect dispatch;
- stack/frame bugs;
- serious scalability problems;
- substantial future architectural blockage.

### Medium

Meaningful problem but locally contained.

Examples:

- measurable avoidable runtime cost;
- fragile invariant;
- unnecessary complexity;
- poorly structured runtime boundary.

### Low

Minor optimization, maintainability concern, test deficiency, or cleanup.

---

## Category

Use one or more:

```text
Correctness
Runtime Safety
Architecture
Performance
Memory
Dispatch
Bytecode
VM
Compiler-Runtime Contract
Maintainability
Testing
Future Compatibility
```

---

## Confidence

```text
Confirmed
High confidence
Probable
Needs targeted verification
```

Do not present speculation as confirmed fact.

---

# 7. Deliverables

The audit should produce two classes of artifacts.

---

# 7.1 Executive Audit Report

Produce one primary Markdown document:

```text
runtime-audit-executive-report.md
```

This is the high-value reasoning artifact.

It should contain the audit's strongest analysis, prioritization, architecture conclusions, and recommended direction.

Do not waste its token budget reproducing every implementation detail.

The report should include:

## Executive Summary

State:

- overall assessment;
- most serious correctness risks;
- most important architecture findings;
- largest performance opportunities;
- whether major runtime redesign is warranted;
- what should be preserved.

## Runtime Architecture Reconstruction

Provide a concise but accurate description of the actual architecture discovered.

## Critical and High-Priority Findings

Summarize every major issue.

For each include:

```text
ID
Title
Severity
Category
Affected components
Core finding
Why it matters
Root cause
Recommended direction
Detailed issue document
```

## Architectural Insights

Discuss cross-cutting discoveries that may not correspond to individual bugs.

Examples:

- identity architecture;
- dispatch architecture;
- object layout;
- compiler/VM contract quality;
- VM abstraction quality;
- optimization readiness.

## Runtime Hot-Path Analysis

Summarize actual execution cost for representative operations.

## Correctness Risk Analysis

Summarize major latent failure classes.

## Performance Analysis

Separate:

```text
current hot-path waste
representation opportunities
architectural optimization opportunities
premature optimizations to avoid
```

## Future Compatibility

Discuss implications for:

- applied types;
- type reification;
- specialized collections;
- typed dispatch;
- future VM optimization.

## Recommended Remediation Order

Give an ordered program of fixes.

Distinguish:

```text
fix immediately
fix before further runtime work
fix opportunistically
defer until benchmark evidence exists
```

## Positive Findings

Explicitly document good architecture worth preserving.

This prevents future agents from "fixing" sound design.

---

# 7.2 Detailed Issue Documents

For each significant issue, create a separate Markdown document.

Closely related findings may be grouped when they share the same root cause.

Naming convention:

```text
AUD-RUNTIME-001-short-description.md
AUD-RUNTIME-002-short-description.md
...
```

These documents should capture enough evidence that another agent can independently expand the analysis or implement the remediation without repeating the entire repository investigation.

Do not make every minor observation its own document.

Create issue documents for:

- Critical findings;
- High findings;
- important Medium architectural findings;
- coherent groups of related performance problems.

---

# 8. Required Issue Document Format

Each issue document must follow this structure.

```markdown
# AUD-RUNTIME-XXX — Title

## Classification

- Severity:
- Category:
- Confidence:
- Priority:
- Affected components:

## Executive Finding

A concise explanation of the issue and why it matters.

## Observed Architecture

Describe only the implementation context needed to understand the issue.

## Evidence

List concrete implementation evidence:

- files;
- types;
- functions;
- bytecodes;
- call paths;
- relevant tests.

Prefer precise references over large copied code blocks.

## Runtime / Compilation Path

Where relevant:

source
→ lowering
→ bytecode
→ VM
→ runtime state
→ failure/performance consequence

## Invariant

State the invariant that should hold.

## Current Behavior

State what actually happens.

## Failure Scenario

Construct one or more concrete examples demonstrating how the issue can manifest.

## Root Cause

Identify the underlying architectural or implementation cause.

Do not stop at the nearest buggy line if a deeper cause exists.

## Impact

Discuss relevant impact:

### Correctness

### Performance

### Memory

### Architecture

### Future extensibility

Only include applicable subsections.

## Recommended Direction

Give the best recommended solution at architectural level.

Explain why it is preferable to obvious alternatives.

## Alternative Solutions

Briefly cover meaningful alternatives and why they are weaker or more expensive.

## Implementation Outline

Provide enough structure for another implementation agent to expand into a detailed plan.

Do not spend excessive tokens producing line-by-line implementation instructions.

## Required Tests

Specify regression and invariant tests.

## Verification Criteria

Define what must be true after remediation.

## Related Findings

Link related audit IDs.

## Open Questions

Only include genuinely unresolved questions.
```

---

# 9. Token-Efficient Investigation Strategy

Use this model primarily for:

- repository understanding;
- tracing;
- diagnosis;
- architectural reasoning;
- prioritization;
- root-cause analysis;
- identifying the best remediation direction.

Do **not** spend large amounts of output on mechanical elaboration that a weaker follow-up agent can perform.

The executive report should contain the most valuable reasoning.

Issue documents should preserve:

- evidence;
- root cause;
- key insight;
- recommended direction;
- implementation constraints;
- required tests.

They should not become exhaustive implementation plans.

A later agent can take:

```text
AUD-RUNTIME-00X
```

and expand it into:

- a full technical investigation;
- technical specification;
- implementation plan;
- test plan;
- benchmark plan.

---

# 10. Group Findings by Root Cause

Avoid producing fragmented reports such as:

```text
Issue 12: extra hash in function A
Issue 13: extra hash in function B
Issue 14: extra hash in function C
```

when the stronger finding is:

```text
AUD-RUNTIME-012
Runtime selector representation causes repeated hashing across dispatch paths
```

Group symptoms when they share an architectural cause.

Conversely, do not group unrelated defects merely because they occur in the same file.

The preferred unit of reporting is:

> one underlying problem and its consequences.

---

# 11. Prioritization Standard

Prioritize by:

```text
correctness risk
× frequency
× architectural reach
× future cost
```

A bug in a rarely used helper may matter less than a dispatch representation flaw affecting every method invocation.

A performance issue that costs one allocation on every method send is more important than an expensive cold-path diagnostic.

An architectural problem that will make future applied-type reification prohibitively difficult may deserve higher priority than several local cleanups.

---

# 12. Required Cross-Cutting Questions

By the end of the audit, explicitly answer these questions.

1. What exactly does an ordinary method invocation cost today?

2. What exactly does a getter invocation cost?

3. What exactly does a native method invocation cost?

4. What runtime information is reconstructed repeatedly that could instead have stable identity?

5. Which maps/hashes are on actual hot paths?

6. What allocations occur during ordinary execution?

7. What information is duplicated between `Value`, object, class, frame, method, and chunk representations?

8. Are compiler and VM stack effects formally consistent?

9. Can unusual control flow leave the VM stack or frames inconsistent?

10. Which opcode operands have boundary-risk or truncation-risk?

11. Which runtime inputs can cause Rust panics?

12. Which runtime invariants are enforced only by compiler behavior?

13. Could malformed internal bytecode violate memory/runtime invariants?

14. Is dispatch architecture ready for inline caching?

15. Is the identity model strong enough for future runtime reification?

16. Are class identity and runtime type identity incorrectly conflated anywhere?

17. Which runtime structures dominate pointer chasing?

18. Which hot data structures have poor locality?

19. What are the top five changes with the highest performance payoff per implementation complexity?

20. What are the top five correctness risks?

21. What parts of the architecture are already strong and should explicitly remain unchanged?

---

# 13. Expected Final Assessment

Conclude with a clear judgment of the runtime architecture.

Choose and justify something approximately equivalent to:

```text
A — fundamentally sound; mostly localized fixes
B — sound foundation with several important structural improvements needed
C — functional but important runtime architecture should be corrected before major expansion
D — foundational redesign required
```

Do not force the assessment into a negative category merely to produce findings.

Base it on implementation evidence.

---

# 14. Audit Boundary

Do not turn this into a general audit of:

- parsing;
- lexing;
- syntax design;
- AST quality;
- general semantic typing;
- LSP;
- module architecture;
- formatter;
- unrelated compiler frontend logic.

Investigate those systems only when necessary to establish a concrete runtime contract or runtime defect.

The subject of this audit is the machinery that transforms executable program behavior into runtime execution.

That includes compiler lowering only to the extent that lowering defines the bytecode and runtime contract.

---

# 15. Final Principle

The most valuable output is not the number of issues found.

The goal is to establish a trustworthy model of Phalcom's actual runtime architecture and identify the relatively small number of underlying decisions that most strongly determine:

- correctness;
- execution cost;
- scalability;
- optimization potential;
- future runtime capabilities.

Find root causes, not merely symptoms.