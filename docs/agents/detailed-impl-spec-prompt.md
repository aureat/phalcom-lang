You are producing a **detailed implementation specification for Phalcom**, intended to be handed directly to another engineering agent that must implement the change correctly without rediscovering the design.

Your task is to translate the requested language/runtime/tooling change into a **repository-grounded, decision-complete implementation plan**.

## Core requirements

1. **Inspect the actual Phalcom codebase first.**
  - Identify the existing architecture, relevant abstractions, execution paths, data structures, naming conventions, tests, and extension points.
  - Cite concrete files, types, functions, methods, compiler phases, runtime components, and existing patterns.
  - Distinguish clearly between:
    - what exists today,
    - what must change,
    - what must be added,
    - what should deliberately remain unchanged.
  - Never invent an abstraction that already exists under another name.

2. **Specify implementation, not merely design intent.**
   The document must tell an agent how to build the feature. Resolve implementation questions wherever the available design permits instead of leaving vague recommendations such as “consider,” “possibly,” or “could.”

3. **Preserve Phalcom's semantic model.**
   Analyze the feature in the context of Phalcom's existing object model, message sends, selectors, methods, attributes, parser/compiler architecture, runtime, native primitives, reflection, errors, tooling, and future-facing language design where relevant.

   Prefer extending existing abstractions coherently over introducing isolated special cases.

4. **Trace the feature end-to-end.**
   Where applicable, cover the complete path:

   ```text
   source syntax
       ↓
   lexer / parser
       ↓
   AST / semantic representation
       ↓
   validation / resolution
       ↓
   lowering / compilation
       ↓
   bytecode / IR / runtime
       ↓
   object-model or native behavior
       ↓
   reflection / diagnostics / LSP / documentation
   ```

   Omit stages that genuinely do not apply, but explicitly explain where the feature enters and exits the implementation pipeline.

## Required specification structure

Use dense, highly structured Markdown with descriptive headings, tables where they improve comparison, and substantial explanatory prose.

Include at least:

### 1. Objective and semantic contract
Define precisely:
- what is being introduced or changed;
- user-visible semantics;
- invariants that must hold;
- deliberately unsupported behavior;
- representative Phalcom syntax and usage examples.

### 2. Current implementation
Explain how the relevant subsystem works today.

Reference concrete repository locations and describe their responsibilities rather than merely listing filenames.

Identify architectural constraints, assumptions, technical debt, or existing behavior that affects this implementation.

### 3. Design decisions
Record all important decisions explicitly.

For each non-obvious choice, state:
- chosen behavior;
- rationale;
- rejected alternatives when useful;
- compatibility consequences;
- whether the choice constrains future features.

Do not silently leave semantic decisions to the implementer.

### 4. Proposed architecture
Describe the resulting architecture and responsibility boundaries.

Name new or modified:
- types/classes/structs/enums;
- AST nodes;
- compiler or resolver phases;
- methods/functions;
- runtime objects;
- primitive interfaces;
- metadata;
- error types;
- caches or state;
- public APIs.

Give concrete signatures or pseudocode where useful.

### 5. Detailed change-set
Organize implementation work by subsystem and preferably by repository file.

For each affected area specify:
- what changes;
- why;
- relevant existing code;
- new control/data flow;
- interactions with other components.

Do not merely say “update parser” or “add runtime support.” Describe the actual mechanism.

### 6. Algorithms and lowering rules
For syntax or semantic features, show exact transformations.

For example:

```phalcom
<surface syntax>
```

lowers conceptually to:

```phalcom
<primitive Phalcom semantics>
```

and then explain how that primitive operation is represented internally.

Include precedence, evaluation order, dispatch behavior, side effects, error propagation, laziness/eagerness, identity, ownership, caching, or concurrency rules whenever relevant.

### 7. Examples
Provide rich examples covering:
- normal use;
- boundary cases;
- interactions with existing features;
- invalid programs;
- diagnostics;
- reflection or metaprogramming where applicable.

Examples should expose semantics, not merely demonstrate syntax.

### 8. Errors and diagnostics
Specify:
- compile-time versus runtime failures;
- error types/messages;
- source spans;
- recovery behavior;
- diagnostic wording or structure where important;
- how malformed and unsupported cases differ.

Diagnostics should follow existing Phalcom conventions.

### 9. Compatibility and migration
Analyze:
- existing programs;
- parser ambiguities;
- runtime compatibility;
- serialized/cached artifacts if relevant;
- native API/ABI implications;
- reflection;
- tooling;
- future modules/typing/concurrency/etc. where materially affected.

Call out changes that would be difficult to reverse after release.

### 10. Testing strategy
Specify concrete tests, not just categories.

Include as applicable:
- lexer/parser tests;
- AST tests;
- semantic/resolver tests;
- compiler/lowering tests;
- runtime tests;
- negative/error tests;
- regression tests;
- reflection tests;
- integration/end-to-end `.ph` tests;
- native boundary tests;
- LSP/tooling tests.

For important behavior, provide example test cases and expected results.

Every semantic rule introduced by the specification should have an obvious corresponding test.

### 11. Implementation sequence
Give a dependency-aware order of work so another agent can implement the feature incrementally without leaving the tree in an incoherent state.

Identify intermediate milestones where the project should compile and tests should pass.

### 12. Acceptance criteria
End with a precise checklist defining when implementation is complete.

The criteria must be observable and testable.

## Quality bar

The implementation specification must be sufficiently precise that an engineer unfamiliar with the preceding design conversation can implement it without guessing at fundamental semantics.

In particular:

- Do not substitute broad architectural discussion for concrete implementation instructions.
- Do not duplicate machinery when an existing Phalcom abstraction can be extended.
- Do not hide difficult cases behind “implementation-defined.”
- Do not overfit the design to the easiest patch if doing so damages the language model.
- Separate **semantic guarantees** from **implementation strategy**.
- Preserve room for known future Phalcom features without prematurely implementing them.
- Explicitly flag uncertain repository facts and investigate them rather than guessing.
- Point out inconsistencies between the requested design and current implementation.
- If the proposed feature exposes an existing architectural defect, say so and specify whether it should be repaired as part of this change-set.
- Prefer small, composable primitives and ordinary Phalcom message/object semantics over compiler/runtime magic unless special treatment is materially justified.
- Treat tests, diagnostics, reflection, and developer tooling as part of the feature rather than follow-up work.

The final document should read less like a proposal and more like a **technical construction blueprint plus semantic specification**.

The feature/change to specify is:

> **[Infer from the user prompt/request what the feature/change is, and clearly state it in the specification.]**

Any already-ratified semantic decisions are authoritative and must be preserved:

> **[List any semantic decisions that have already been made and must be preserved in the implementation.]**
