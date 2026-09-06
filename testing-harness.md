# Phalcom In-Process Language Corpus Testing Harness

## Technical Specification

**Status:** Implemented  
**Subsystem:** `phalcom-core` testing, compiler/runtime integration  
**Reference implementation:** `6ac8b6d7` and preceding corpus-harness commits  
**Primary test target:** `language-corpus`

---

## 1. Purpose

This document specifies the architecture and behavior of Phalcom's in-process source-language corpus testing harness.

The harness provides production-path acceptance testing for `.ph` programs without launching a separate `phalcom` process for every fixture. Each fixture is compiled through the normal program compiler, executed in an isolated fresh VM, and evaluated using structured compilation/runtime outcomes together with fixture-local captured output.

The harness exists to provide:

- production-faithful language acceptance testing;
- isolation between fixtures without requiring OS-process isolation;
- reuse of immutable process-scoped compiler and canonical Universe products;
- deterministic program-output capture;
- structured distinction between compilation, bootstrap, I/O, and runtime failures;
- explicit negative-test failure-phase expectations;
- safe containment of VM/bootstrap failures within the Rust test process;
- a foundation for efficient bounded corpus parallelism.

The harness is not a replacement for CLI end-to-end testing. CLI behavior remains the responsibility of a separate subprocess-oriented integration target.

---

## 2. Background

### 2.1 Previous subprocess architecture

The original language corpus executed each fixture through the `phalcom` executable.

Conceptually:

```text
Rust corpus test
    ↓
spawn `phalcom`
    ↓
CLI initialization
    ↓
load fixture
    ↓
compile program
    ↓
construct VM
    ↓
execute program
    ↓
inspect exit status
    ↓
inspect stdout/stderr
```

This provided convenient process isolation and exercised the complete command-line path, but it became unsuitable as the primary broad language acceptance harness.

Each `.ph` fixture required an independent OS process and therefore an independent process-local compiler/runtime environment.

---

### 2.2 Process amplification

The subprocess model repeatedly paid costs that should have been reusable within one test run.

For every fixture it performed some combination of:

- OS process creation;
- executable and CLI initialization;
- process-local compiler initialization;
- canonical Universe source discovery;
- canonical linking;
- canonical semantic analysis;
- canonical compiler-product construction;
- VM bootstrap;
- fixture compilation;
- fixture execution.

This became particularly costly as semantic analysis became more sophisticated.

Process-scoped caches and immutable compiler products, including the canonical Universe compiler product, could not provide corpus-wide reuse because every fixture ran in a different process.

The resulting topology was effectively:

```text
fixture A → process A → canonical compiler world A
fixture B → process B → canonical compiler world B
fixture C → process C → canonical compiler world C
...
```

The desired topology is instead:

```text
one corpus process
    ↓
one reusable immutable compiler world
    ↓
many independently compiled fixtures
    ↓
one fresh mutable VM per executing fixture
```

---

### 2.3 Test scheduling amplification

The broad language corpus was also previously part of a larger integration-test topology.

That combined two sources of contention:

1. each corpus lane could execute concurrently under the Rust test harness;
2. each fixture within those lanes launched another `phalcom` process.

Consequently, expensive semantic/compiler initialization could be multiplied across many simultaneously active child processes.

The language corpus is now isolated into its own Rust integration-test target so broad fixture execution can be scheduled and measured independently from the focused `core` integration suite.

---

### 2.4 The subprocess boundary hid implementation defects

Performance was not the only problem.

A subprocess also creates a coarse error boundary:

```text
child succeeded
child failed
child printed stdout
child printed stderr
child exited with status N
```

That boundary concealed distinctions inside the compiler and runtime.

Failures originating from:

- fixture configuration;
- parsing;
- formal semantic analysis;
- program projection;
- canonical Universe bootstrap;
- VM-dependent bytecode compilation;
- runtime execution;
- runtime I/O;
- Rust panics;

could all eventually appear to the test harness as some combination of non-zero exit status and output text.

Moving the corpus in-process exposed these distinctions directly and forced the runtime and test harness to represent them explicitly.

---

## 3. Migration Issues Exposed by the Redesign

The move to in-process execution exposed several defects or weaknesses that had previously been hidden or tolerated by the subprocess architecture.

### 3.1 Runtime output depended on process-global stdout

Runtime primitives such as `System.print` wrote directly to Rust stdout.

This made program output:

- global to the test process;
- difficult to isolate between fixtures;
- unsafe for future concurrent corpus execution;
- awkward to capture without global redirection.

The redesign introduced VM-owned output sinks.

---

### 3.2 Failure phases were conflated

The old harness primarily reasoned about successful versus unsuccessful child processes.

It did not preserve a reliable distinction between:

- program-level semantic compilation;
- VM-dependent bytecode compilation;
- VM bootstrap;
- runtime-language failure;
- host I/O failure.

The new harness introduces a structured corpus outcome model.

---

### 3.3 Negative tests did not reliably protect phase semantics

Some negative tests effectively asserted:

> the fixture fails and the output contains this text.

That permits a regression such as:

```text
expected:
program compiles
→ runtime operation fails

regression:
semantic analyzer rejects program before execution
```

to remain green if both failure paths happen to contain compatible text.

This is especially significant in Phalcom because static typing is gradual. Whether behavior is rejected statically or deferred to runtime is part of the language contract.

The new harness therefore supports explicit expected failure phases.

---

### 3.4 Fixture flags were not sufficiently strict

Fixture-level flags could previously be interpreted too loosely.

Unknown flags could be ignored, and compile-mode conflicts could fail to reproduce CLI semantics.

The new harness parses the supported fixture configuration explicitly and fails closed.

---

### 3.5 Semantic diagnostic matching relied on implementation details

Semantic error expectations previously risked depending on internal Rust enum formatting rather than stable language-facing diagnostic codes.

For example, a Rust representation such as:

```text
ArgumentMismatch
```

is an implementation detail.

A stable diagnostic identifier such as:

```text
type.call.argument_mismatch
```

is suitable for corpus expectations.

The new harness uses the latter.

---

### 3.6 Diagnostic matching could inspect only a prefix

A diagnostic expected by a fixture could be present in the full semantic result but fall outside the subset displayed by the test harness.

The new implementation separates:

- complete diagnostic matching used by the oracle;
- bounded diagnostic presentation used in assertion failures.

The matching operation considers the full diagnostic set.

---

### 3.7 Bootstrap assumptions could escape as Rust panics

Full VM construction historically contained several `expect`, indexing, and other infallible assumptions.

These were acceptable only while VM bootstrap was treated as an internal operation that must always succeed.

An in-process acceptance harness needs a different boundary: bootstrap failure must become structured test data.

The VM therefore now provides a fallible full-bootstrap constructor.

---

### 3.8 `Bool.new()` contained a host-language panic path

The Boolean constructor accessed:

```rust
args[0]
```

while a zero-argument primitive wrapper could invoke it with an empty argument list.

This caused invalid Phalcom code to produce a Rust indexing panic.

The corrected implementation uses safe argument access and reports a catchable Phalcom runtime error for attempted direct construction of abstract `Bool`.

---

### 3.9 Debug output contaminated language output

`Bool.new(_)` also contained historical debug output.

Under process-global stdout, such output could be overlooked or attributed to executable behavior.

Once program output became explicitly VM-owned and fixture-captured, the distinction became unavoidable.

Debug output was removed rather than incorporated into the new output abstraction.

Host debugging output is not Phalcom program output.

---

## 4. Design Goals

The harness is designed around the following goals.

### 4.1 Production-path fidelity

Fixtures must exercise the real program compilation and VM execution paths.

The language corpus must not depend on a test-specific parser, linker, semantic analyzer, bytecode compiler, or runtime.

---

### 4.2 Fixture isolation

Every executing fixture must receive independent mutable runtime state.

No fixture may depend on mutations performed by another fixture.

---

### 4.3 Reuse of immutable process-scoped compiler products

Compiler products and caches explicitly designed to be immutable and process-shared should remain reusable across fixtures.

The harness must not force process isolation merely to obtain runtime isolation.

---

### 4.4 Structured failures

Failures should preserve their architectural source rather than being reduced to generic non-zero exit statuses.

---

### 4.5 Deterministic output

Program output must be attributable to one VM and one fixture.

Fixture assertions must not depend on global stdout interception.

---

### 4.6 Panic containment

Invalid Phalcom programs and representable VM/bootstrap failures should produce structured errors rather than uncontrolled Rust panics.

---

### 4.7 Test-phase fidelity

Negative tests should be able to specify whether rejection is expected during parsing, semantic analysis, compilation, or runtime execution.

---

### 4.8 Future concurrency

The architecture should permit bounded concurrent fixture execution without sharing mutable VM state or output channels.

Actual concurrency policy remains subject to measurement.

---

## 5. Non-Goals

The harness does not require or attempt to implement:

- one persistent VM reused across corpus fixtures;
- cloning a live VM;
- VM heap snapshots;
- serialization of initialized runtime state;
- a daemonized Phalcom compiler process;
- a batch CLI protocol;
- IPC between the Rust test harness and a long-running `phalcom` executable;
- process-global stdout capture;
- artificial reproduction of CLI exit codes inside the in-process corpus;
- replacement of dedicated CLI end-to-end tests;
- a test-only compiler pipeline;
- elimination of all per-fixture VM bootstrap work.

The principal optimization is removal of unnecessary process and process-local compiler duplication while retaining isolated runtime state.

---

## 6. Test Target Topology

`phalcom-core` explicitly defines separate integration-test targets for different responsibilities:

```text
core
language-corpus
cli-smoke
```

### 6.1 `core`

The `core` target contains focused compiler, VM, runtime, object-model, module, semantic integration, and related subsystem tests.

It is not the owner of broad source-language corpus execution.

---

### 6.2 `language-corpus`

The `language-corpus` target is the production-faithful source-language acceptance corpus.

Its integration root:

- imports the common corpus support implementation;
- imports the language corpus test definitions;
- contains corpus-specific output-isolation coverage.

The broad corpus therefore runs as its own Rust integration binary.

---

### 6.3 `cli-smoke`

The CLI target retains subprocess execution where the process boundary is itself what needs to be tested.

This includes representative checks for:

- actual executable invocation;
- command-line parsing;
- operating-system exit codes;
- stdout/stderr routing;
- CLI diagnostic presentation;
- user-facing compile modes and options.

The broad language corpus does not use subprocesses merely to test language behavior.

---

## 7. Harness Architecture

The in-process corpus uses the following topology:

```text
language-corpus process
│
├── shared immutable/process-scoped compiler state
│
├── fixture A
│   ├── configuration A
│   ├── ProgramCompiler
│   ├── fresh VM A
│   ├── BufferedOutput A
│   ├── VM-dependent compilation
│   └── execution
│
├── fixture B
│   ├── configuration B
│   ├── ProgramCompiler
│   ├── fresh VM B
│   ├── BufferedOutput B
│   ├── VM-dependent compilation
│   └── execution
│
└── fixture N
    ├── configuration N
    ├── ProgramCompiler
    ├── fresh VM N
    ├── BufferedOutput N
    ├── VM-dependent compilation
    └── execution
```

The key architectural separation is:

```text
immutable compiler world
        shared

mutable runtime world
        isolated
```

---

## 8. Shared and Isolated State

### 8.1 State that may be shared

The corpus may reuse process-scoped state that the production architecture defines as immutable or synchronization-safe.

This includes the canonical Universe compiler product and associated immutable compiler results.

The harness should benefit from ordinary production caching rather than construct a separate corpus-specific cache.

---

### 8.2 State that must remain fixture-local

Each fixture receives a fresh VM because mutable runtime state may include:

- heap objects;
- module globals;
- module initialization state;
- runtime classes;
- class reopening and monkey-patching;
- method tables;
- closures;
- dynamic runtime registries;
- scheduler and fiber state;
- runtime resources;
- object identity;
- VM-local caches;
- output state.

Sharing one mutable VM across fixtures would make the test result dependent on fixture order.

That is not an acceptable optimization.

---

## 9. Fixture Execution Pipeline

One corpus fixture proceeds through the following stages.

### 9.1 Fixture discovery

The harness locates `.ph` files under the language fixture tree.

Fixture paths are collected deterministically and sorted before execution.

Associated `.expected` files provide output or negative-test expectations.

---

### 9.2 Fixture configuration

A source fixture may contain a directive of the form:

```text
// flags: <options>
```

The current supported options are:

```text
--release
--unchecked
--strip-contract-metadata
```

The harness converts these into structured per-fixture options.

Default configuration uses debug compilation with contract metadata retained.

---

### 9.3 Configuration validation

Fixture configuration is intentionally strict.

The following produce configuration failure:

- unsupported flags;
- duplicate flags;
- mutually exclusive compile modes.

In particular:

```text
--release --unchecked
```

is rejected rather than interpreted according to ordering.

Likewise:

```text
--unknown-option
```

does not silently fall back to default execution.

Fixture metadata errors are harness configuration errors, not Phalcom compiler errors.

---

### 9.4 Program-level compilation

The source fixture is compiled through the production program compiler:

```rust
ProgramCompiler::compile_entry_selection(
    EntrySelection::Module(path)
)
```

This stage owns program-level operations such as:

- source loading;
- module discovery;
- parsing;
- linking;
- formal semantic analysis;
- program projection.

A failure here becomes a `ProgramCompileFailure`.

The harness does not require creation of a full VM before this stage.

---

### 9.5 Output sink allocation

Once program-level compilation succeeds, the harness creates a fixture-local `BufferedOutput`.

A readable handle is retained by the test harness while ownership of the actual sink is transferred to the VM.

Conceptually:

```text
BufferedOutput
├── VM-owned writer
└── test-owned read handle
```

This permits output inspection after successful execution, failed execution, or failed VM bootstrap.

---

### 9.6 VM bootstrap

Each executing fixture constructs a fresh full VM through the fallible API:

```rust
VM::try_new_with_output(Box::new(sink))
```

If bootstrap fails, the fixture returns a structured bootstrap outcome.

The harness does not convert bootstrap failure into a host panic.

---

### 9.7 Compile option application

After VM construction, the per-fixture VM/compiler settings are applied.

These include:

- compile mode;
- contract-metadata stripping.

This ordering is intentional.

`ProgramCompiler` performs program-level compilation first, while VM-dependent bytecode materialization and code generation still occur through the VM execution path.

---

### 9.8 VM-dependent compilation and execution

The compiled program is executed using:

```rust
vm.run_compiled(&program)
```

This may still perform VM-dependent compilation or materialization before actual runtime execution.

Consequently, an error arising from `run_compiled` is not automatically a runtime-language error.

The harness reclassifies errors according to their real architectural source.

---

### 9.9 Outcome collection

After execution or failure, the harness retains:

```text
CorpusRun
├── captured stdout bytes
└── structured CorpusOutcome
```

Program output and failure state are independent observations.

A fixture may therefore legitimately contain both:

```text
stdout emitted before failure
```

and:

```text
RuntimeFailure(...)
```

---

## 10. Corpus Outcome Model

The corpus uses a structured result taxonomy rather than operating-system exit status.

Current outcomes are:

```rust
Success
ConfigurationFailure
ProgramCompileFailure
BytecodeParseFailure
BytecodeCompileFailure
BootstrapFailure
IoFailure
RuntimeFailure
```

---

### 10.1 `Success`

The fixture completed all required compilation, bootstrap, and execution phases without error.

Positive fixtures require this outcome.

---

### 10.2 `ConfigurationFailure`

The fixture's harness metadata is invalid.

Examples include:

- unsupported fixture flag;
- duplicate compile-mode flag;
- conflicting `--release` and `--unchecked`;
- duplicate metadata-stripping option.

This category distinguishes malformed tests from malformed Phalcom programs.

---

### 10.3 `ProgramCompileFailure`

The program failed in the production `ProgramCompiler` pipeline.

This encompasses program-level loading, parsing, linking, semantic analysis, projection, and other failures owned by that pipeline.

The underlying `ProgramCompileError` retains additional phase information.

---

### 10.4 `BytecodeParseFailure`

A parser failure surfaced from the VM-dependent `run_compiled` path.

The category exists because the current architecture can still surface parse errors after the initial program-level compilation stage for VM-dependent compilation inputs.

It is treated as a parse-phase failure by the negative-test phase oracle.

---

### 10.5 `BytecodeCompileFailure`

VM-dependent bytecode or compiler work failed.

This is distinct from a runtime-language failure even though it may originate while executing `run_compiled`.

---

### 10.6 `BootstrapFailure`

Construction or initialization of the full VM failed.

The underlying `VmBootstrapError` retains the bootstrap category.

Bootstrap failure is not considered a fixture runtime error.

---

### 10.7 `IoFailure`

A host I/O operation used by the VM failed.

For example, a `RuntimeOutput` sink may return an `io::Error`.

This is represented structurally rather than as a panic or silently discarded write.

---

### 10.8 `RuntimeFailure`

The program reached the language runtime and produced a Phalcom runtime error.

This category is reserved for execution-time language behavior rather than compiler or bootstrap failures.

---

## 11. Expected Failure Phases

Negative fixtures may declare an expected architectural failure phase through `ExpectedFailurePhase`.

The supported phases are:

```rust
Any
Parse
Semantic
Compile
Runtime
```

The phase expectation is an assertion over `CorpusOutcome`.

---

### 11.1 `Parse`

The fixture must fail through either:

- `ProgramCompileFailure(Parse(...))`; or
- `BytecodeParseFailure(...)`.

This protects parser-negative fixtures from accidentally moving into another failure phase.

---

### 11.2 `Semantic`

The fixture must fail through:

```text
ProgramCompileFailure(Semantic(...))
```

A runtime error does not satisfy a semantic expectation.

---

### 11.3 `Compile`

The fixture must fail during compilation but not through parser or formal semantic failure.

The current implementation accepts:

- non-parse/non-semantic `ProgramCompileFailure`;
- `BytecodeCompileFailure`.

This reflects the current split between `ProgramCompiler` and VM-dependent bytecode compilation.

---

### 11.4 `Runtime`

The fixture must produce:

```text
RuntimeFailure
```

Bootstrap failure, compilation failure, and I/O failure do not satisfy this expectation.

This is important for tests that specifically establish that an operation remains dynamically accepted and fails only during execution.

---

### 11.5 `Any`

`Any` accepts any structured non-success outcome.

It exists for legacy corpus directories that historically combine multiple kinds of negative tests.

It should be treated as a compatibility mechanism, not the preferred organization for new phase-specific negative test suites.

---

## 12. Legacy Mixed-Phase Fixture Groups

Some older corpus directories contain heterogeneous tests.

A directory named `runtime-errors`, for example, may historically contain a mixture of:

- parser rejection;
- semantic argument validation;
- bytecode/compiler rejection;
- genuine runtime errors.

Likewise, older `compile-errors` directories may combine multiple compiler phases.

Such groups may temporarily use:

```text
ExpectedFailurePhase::Any
```

until they are split or individually categorized.

New negative fixture groups should use an explicit expected phase whenever that phase is part of the intended behavior.

---

## 13. Why Failure Phase Matters

A matching diagnostic string is not sufficient evidence that a negative test still means the same thing.

For example:

```text
Before:
program passes semantic analysis
→ execution fails dynamically

After regression:
semantic analyzer rejects the program
```

Both paths might contain similar terminology, but the language behavior has changed.

This distinction is especially important in Phalcom because typing is gradual and the semantic analyzer intentionally leaves some behavior to runtime.

The failure-phase oracle therefore protects not only error reporting but also the static/dynamic boundary of the language implementation.

---

## 14. VM Output Architecture

Runtime program output is owned by the VM through the `RuntimeOutput` abstraction.

Conceptually:

```text
VM
└── RuntimeOutput
    ├── StdoutOutput
    └── BufferedOutput
```

The VM does not require direct ownership of process-global stdout.

---

### 14.1 `StdoutOutput`

`StdoutOutput` preserves ordinary executable behavior.

When the normal CLI constructs a VM, program output continues to reach the user's stdout.

The output abstraction therefore does not change expected command-line semantics.

---

### 14.2 `BufferedOutput`

`BufferedOutput` stores emitted bytes in an independently accessible buffer.

The corpus creates one instance for every fixture.

Conceptually:

```text
fixture A → VM A → buffer A
fixture B → VM B → buffer B
fixture C → VM C → buffer C
```

No corpus assertion depends on redirecting process-global stdout.

---

### 14.3 Captured output sources

The VM-owned channel is used for Phalcom-visible output including:

- `System.print`;
- raw system writes;
- display conversion invoked by printing;
- user-defined `toString` behavior reached through display;
- output emitted before a subsequent runtime error.

Host debugging output must not be inserted into this channel merely because the channel is easy to access.

It represents program-visible output.

---

### 14.4 Output isolation

Each fixture's output is independent of every other fixture.

This allows:

- deterministic golden testing;
- safe future concurrency;
- output assertions after errors;
- embedded VM execution without stdout interception.

---

### 14.5 Output I/O failures

The output abstraction is fallible.

An output-sink error propagates through the VM as structured I/O failure.

Conceptually:

```text
RuntimeOutput::write
        ↓
io::Error
        ↓
PhError::Io
        ↓
CorpusOutcome::IoFailure
```

The error must not be silently dropped.

It must not become a Rust panic merely because output is unavailable.

---

## 15. Positive Fixture Oracle

A normal positive fixture must:

1. have valid harness configuration;
2. compile successfully through `ProgramCompiler`;
3. construct a full VM successfully;
4. complete VM-dependent compilation successfully;
5. execute successfully;
6. produce the expected output.

A positive fixture failing in any structured phase is reported with that phase's diagnostic summary.

---

## 16. Golden Output Comparison

Positive fixture output is compared as bytes after minimal trailing-line-ending normalization.

The current normalization removes at most one final line ending from both actual and expected output.

Specifically:

```text
"...\n"   → "..."
"...\r\n" → "..."
```

No general whitespace normalization is performed.

Therefore:

```text
"hello "
```

and:

```text
"hello"
```

remain different.

Multiple additional trailing line breaks also remain observable except for the single final line ending normalized by the comparison helper.

The normalization exists to avoid making a single final fixture newline significant while preserving the rest of the output stream exactly.

---

## 17. Negative Fixture Oracle

A negative fixture must not succeed.

The oracle evaluates three distinct dimensions.

### 17.1 Failure existence

The observed `CorpusOutcome` must not be `Success`.

---

### 17.2 Failure phase

When an explicit `ExpectedFailurePhase` is configured, the observed structured outcome must satisfy that phase.

---

### 17.3 Diagnostic or output expectation

The associated expected sidecar provides the expected diagnostic or output substring.

A negative case succeeds when the expected text occurs in either:

- the structured error representation; or
- captured output emitted by the program.

Allowing captured output is necessary for tests whose observable behavior includes output immediately before a deliberate failure.

---

## 18. Semantic Diagnostic Matching

Semantic analysis can produce multiple diagnostics.

The harness distinguishes diagnostic matching from diagnostic presentation.

### 18.1 Oracle behavior

The negative-test oracle examines the complete semantic diagnostic set.

If an expected diagnostic is the ninth, twentieth, or later diagnostic, it remains eligible to satisfy the fixture.

---

### 18.2 Failure presentation

When constructing a human-readable assertion message, the harness may display only a bounded prefix of the diagnostics.

The current presentation limit is eight diagnostics.

If additional diagnostics exist, the report indicates that more were omitted.

The presentation limit has no effect on oracle matching.

---

### 18.3 Stable diagnostic codes

Semantic diagnostic strings are constructed using stable diagnostic-code display values.

For example:

```text
[type.annotation.unresolved] ...
```

rather than an implementation-specific Rust enum representation.

Corpus expectations should therefore remain stable across internal Rust refactors that preserve the diagnostic API.

---

## 19. Disassembly and Compiler-Structure Assertions

Not every corpus test is purely behavioral.

Some fixtures verify compiler properties through disassembly or generated-code inspection.

Examples include asserting:

- direct lowering of loop constructs;
- absence of unnecessary closure materialization;
- absence of indirect call forms where optimized direct lowering is expected.

These tests share source fixture infrastructure with the language corpus but use structural compiler assertions rather than only stdout/runtime outcomes.

They should remain distinct from behavioral golden tests in the oracle implementation.

---

## 20. Pending Fixtures

Pending fixtures represent known language/runtime behavior that is intentionally not yet accepted.

Pending is distinct from failure.

A pending fixture should remain explicitly discoverable and gated rather than disappearing from the corpus or being treated as an ordinary unexpected failure.

This preserves visibility into planned language support without turning incomplete functionality into a silently ignored test.

---

## 21. Fallible VM Bootstrap

The in-process corpus uses:

```rust
VM::try_new_with_output(...)
```

which returns:

```rust
Result<VM, VmBootstrapError>
```

This is the robust test/embedding boundary for full VM construction.

---

### 21.1 Production convenience constructors

The VM may also expose convenience constructors such as:

```rust
VM::new()
VM::new_with_output(...)
```

These preserve the traditional assumption that a correctly built Phalcom runtime must bootstrap successfully.

They may convert bootstrap failure into a host panic where that is appropriate for the calling context.

The corpus does not use that contract.

---

### 21.2 Corpus bootstrap contract

For corpus execution, bootstrap failure is test data.

The fallible result path covers operations including:

- canonical Universe compiler-product acquisition;
- canonical module materialization;
- root module lookup;
- native binding installation;
- primordial class binding;
- canonical module identity resolution;
- alias synchronization;
- linked Universe module execution;
- semantic-root validation;
- Universe invariant validation.

Representable failures must propagate to the harness as `BootstrapFailure`.

---

### 21.3 Materialization hardening

The migration replaced several assumptions in canonical Universe materialization with structured error handling.

Examples include:

- invalid canonical module components;
- missing Universe root;
- missing child module;
- missing parent module;
- absent canonical binding owner;
- binding-slot overflow;
- missing registry entries.

This is part of making full VM construction usable as an in-process embedding boundary rather than only as an infallible executable startup routine.

---

## 22. Host Panic Versus Language Failure

Running the corpus in-process makes the distinction between a Phalcom error and a Rust panic much more important.

Under the old architecture:

```text
Rust panic
    ↓
child process terminates
    ↓
test sees failed executable
```

Under the new architecture:

```text
Rust panic
    ↓
same Rust integration-test process
```

A host panic therefore indicates a runtime/compiler implementation defect unless it is enforcing a genuinely impossible internal invariant.

Invalid Phalcom source or runtime operations should normally flow through the structured Phalcom error model.

---

## 23. `Bool.new()` Regression

The migration exposed a concrete panic-safety defect in the Boolean constructor.

The old implementation accessed:

```rust
args[0]
```

without verifying that an argument existed.

A zero-argument constructor wrapper could invoke that function with an empty slice.

The result was a Rust indexing panic rather than a Phalcom error.

The corrected implementation performs safe argument inspection and returns an abstract-class runtime error for direct construction:

```text
cannot instantiate abstract class Bool
```

The behavior is covered both through:

- direct primitive invocation;
- the zero-argument primitive wrapper;
- a language corpus regression fixture.

This incident demonstrates why an in-process corpus requires runtime input validation to stay within the language error boundary.

---

## 24. Debug Output Regression

The Boolean constructor also contained legacy debugging writes.

During output-sink migration it was important not merely to redirect those writes into `RuntimeOutput`.

Doing so would have transformed implementation debugging noise into formally captured language output.

The debug writes were therefore removed.

This distinction applies generally:

```text
Phalcom-visible output
    → VM RuntimeOutput

compiler/runtime diagnostics and developer tracing
    → dedicated diagnostic/tracing facilities
```

The two channels must not be conflated.

---

## 25. Fixture Isolation Model

A fresh VM is constructed for every fixture that reaches execution.

This is the mechanism used to isolate mutable runtime state.

---

### 25.1 Heap isolation

Objects allocated by one fixture must not be visible to another.

---

### 25.2 Global and module-state isolation

Module globals and module initializer state are fixture-local.

---

### 25.3 Class and method isolation

Class reopening, monkey-patching, method installation, and other runtime class mutations must not survive into another fixture.

---

### 25.4 Scheduler isolation

Fiber, scheduler, continuation, or other execution state must not survive between fixtures.

---

### 25.5 Resource isolation

Runtime resources owned by one VM remain associated with that VM.

---

### 25.6 Output isolation

Each fixture has its own output sink and buffer.

---

### 25.7 Order independence

The same fixture should produce the same result regardless of which other corpus fixtures execute before or after it.

A performance optimization that violates this property is not acceptable.

---

## 26. Why a Persistent VM Is Not Used

Reusing a single full VM across the entire corpus would reduce per-fixture bootstrap work but introduce shared mutable execution state.

Such sharing could leak:

- globals;
- loaded module mutations;
- runtime class modifications;
- dynamically installed methods;
- object identities;
- scheduler state;
- caches whose semantics depend on runtime mutation.

The harness therefore intentionally optimizes the immutable compiler side while retaining fresh mutable runtime state.

The design is:

```text
reuse immutable compiler products
do not reuse mutable execution worlds
```

---

## 27. CLI Integration Boundary

The in-process corpus and subprocess CLI testing have different responsibilities.

Neither replaces the other.

---

### 27.1 Language corpus responsibilities

The in-process corpus primarily verifies:

- source-language behavior;
- parser/semantic behavior;
- compiler behavior;
- VM-dependent compilation;
- runtime behavior;
- structured errors;
- output;
- fixture isolation;
- selected generated-code properties.

It should not depend on executable startup merely to exercise these systems.

---

### 27.2 CLI subprocess responsibilities

A small dedicated subprocess suite verifies behavior where the OS/executable boundary is itself relevant.

Examples include:

- actual `phalcom` executable invocation;
- Clap argument parsing;
- file/package entry selection;
- exit status mapping;
- stdout/stderr destinations;
- CLI rendering of diagnostics;
- user-facing compile flags;
- missing-file behavior.

This keeps broad semantic acceptance testing efficient without sacrificing end-to-end executable coverage.

---

## 28. CLI Compile-Before-VM Ordering

The executable path was also changed so program-level compilation occurs before full VM construction.

Conceptually:

```text
old:
construct full VM
    ↓
compile selected program
    ↓
execute

new:
compile selected program
    ↓
construct full VM
    ↓
execute
```

This means source-loading, linking, semantic, and program-projection failures do not unnecessarily pay full runtime bootstrap.

This does not mean all compilation occurs before VM construction.

VM-dependent bytecode compilation still occurs later through `run_compiled`.

That distinction is reflected in the corpus failure taxonomy.

---

## 29. Performance Model

The in-process harness solves a test-topology multiplication problem.

It does not by itself guarantee that semantic analysis is intrinsically inexpensive.

These must remain separate performance concerns.

---

### 29.1 Costs removed or reduced

The new harness removes per-fixture:

- OS process startup;
- CLI startup;
- repeated process-local initialization;
- repeated inability to reuse process-scoped canonical compiler products.

---

### 29.2 Costs intentionally retained

Every executing fixture still receives:

- its own full mutable VM;
- fixture-local Universe runtime materialization as required;
- fixture-local module initialization;
- fixture-local runtime execution.

These costs provide isolation.

---

### 29.3 Canonical compiler reuse

The canonical Universe compiler product is process-shared according to the production architecture.

Repeated fixtures in one `language-corpus` process can therefore benefit from reuse that was impossible when every fixture launched a new process.

---

### 29.4 Semantic-analysis performance

If recursive coverage analysis, GADT inhabitation, or another semantic subsystem remains expensive after the harness topology is corrected, that is an independent optimization problem.

The test harness should not weaken or bypass semantic analysis to compensate for intrinsic semantic-analysis cost.

---

## 30. Concurrency Model

The current architecture is designed to support safe bounded concurrency.

The important prerequisites are now present:

- fresh VM per fixture;
- fixture-local output;
- no reliance on global stdout;
- reusable process-scoped immutable compiler products.

Actual worker count remains a performance-policy decision.

---

### 30.1 Concurrency must be measured

Increasing Rust test parallelism does not automatically improve corpus wall-clock time.

Each fixture can still perform substantial:

- semantic analysis;
- VM bootstrap;
- allocation;
- bytecode compilation.

Unbounded concurrency can therefore cause CPU and memory contention.

The appropriate worker count should be determined empirically.

---

### 30.2 Bounded execution

Future corpus concurrency should be explicitly bounded where measurement shows that Rust's default scheduling causes oversubscription.

Correctness must remain independent of the selected worker count.

---

## 31. Regression Coverage Required by the Architecture

The harness should retain focused regression tests for the architectural mechanisms introduced during the migration.

Important areas include:

### Output

- `System.print` writes to the configured VM sink;
- raw system writes use the same sink;
- separate VMs do not share output;
- custom display paths reach the same sink;
- an output sink failure becomes `PhError::Io`;
- Boolean coercion does not emit debug output.

### Bootstrap

- fallible VM construction returns structured invariant/bootstrap failures;
- canonical compiler products remain process-reused where intended;
- multiple full VMs retain independent mutable state.

### Fixture configuration

- unsupported flags fail;
- duplicate flags fail;
- conflicting compile modes fail;
- valid compile modes reach VM-dependent compilation.

### Failure phases

- parser fixtures fail as parse failures;
- semantic fixtures fail as semantic failures;
- phase-specific runtime fixtures reach runtime;
- mixed legacy lanes remain explicitly marked `Any`.

### Panic containment

- zero-argument `Bool.new()` returns a language error rather than panicking;
- bootstrap/materialization failures reachable through the fallible API return structured errors.

---

## 32. Verification Procedure

Changes to the corpus harness should be verified at multiple levels.

### 32.1 Build verification

At minimum:

```bash
cargo check -p phalcom-core --tests
```

---

### 32.2 Focused harness regressions

Run focused tests covering:

- fixture configuration;
- output capture;
- output isolation;
- output I/O failure;
- fallible bootstrap;
- Boolean constructor panic regression;
- negative phase mapping.

---

### 32.3 Language corpus

Run:

```bash
cargo test -p phalcom-core --test language-corpus
```

when the canonical runtime baseline is green.

---

### 32.4 Focused core tests

Run:

```bash
cargo test -p phalcom-core --test core
```

to ensure the target split has not removed focused compiler/runtime integration coverage.

---

### 32.5 CLI integration

Run:

```bash
cargo test -p phalcom-core --test cli-smoke
```

for subprocess-oriented executable behavior.

---

### 32.6 Full package verification

Once independent semantic/bootstrap blockers are resolved:

```bash
cargo test -p phalcom-core
```

should verify the complete package-level combination.

---

## 33. Current Implementation Status

At the time this specification was established, the in-process harness architecture and its hardening changes were implemented.

Implemented areas include:

- dedicated `language-corpus` integration target;
- separate `cli-smoke` target;
- production-path `ProgramCompiler` execution;
- fresh VM per executing fixture;
- VM-owned `RuntimeOutput`;
- fixture-local `BufferedOutput`;
- strict fixture configuration;
- structured `CorpusOutcome`;
- expected failure phases;
- complete semantic diagnostic matching;
- stable diagnostic-code rendering;
- fallible VM bootstrap;
- bootstrap/materialization error hardening;
- output I/O error propagation;
- `Bool.new()` panic repair;
- removal of Boolean debug output;
- regression coverage for output and bootstrap behavior.

---

## 34. Current External Blocker

At the time of implementation, complete runtime-corpus execution remained blocked by an independent canonical Universe semantic-baseline mismatch encountered during full VM construction.

The resulting flow is:

```text
fixture compilation succeeds
    ↓
full VM bootstrap begins
    ↓
canonical Universe semantic validation fails
    ↓
CorpusOutcome::BootstrapFailure
```

This is not considered a defect in the corpus architecture.

On the contrary, the fact that the blocker appears as a structured bootstrap outcome demonstrates that the new error boundary is functioning correctly.

The canonical Universe semantic mismatch must be repaired through the semantic/bootstrap architecture rather than bypassed by the corpus runner.

---

## 35. Deferred Work

### 35.1 Legacy mixed-phase cleanup

Historical negative directories using `ExpectedFailurePhase::Any` should gradually be reorganized where stable phase expectations can be identified.

This is test-organization debt rather than a prerequisite for the harness architecture.

---

### 35.2 Bounded concurrency

The harness is structurally ready for safe concurrency, but worker count should be selected from measurement rather than assumed.

---

### 35.3 Semantic coverage performance

Coverage/inhabitation optimizations remain independent work.

Potential work in that area includes:

- cheap terminal inhabitation paths;
- avoiding recursive fixed-point machinery for nonrecursive domains;
- safe analysis-local canonical inhabitation memoization;
- deterministic structural performance metrics.

These changes should be undertaken only after measuring residual semantic cost under the corrected in-process corpus topology.

---

## 36. Implementation Surfaces

The current implementation is primarily distributed across the following areas.

### Test target configuration

```text
phalcom-core/Cargo.toml
```

Defines:

- `core`;
- `language-corpus`;
- `cli-smoke`.

---

### Language corpus integration root

```text
phalcom-core/tests/language_corpus/mod.rs
```

Owns the production-faithful corpus integration target and composes:

- shared corpus support;
- language corpus test definitions;
- output-specific integration coverage.

---

### Shared corpus support

```text
phalcom-core/tests/support/mod.rs
```

Contains the core harness machinery including:

- fixture option parsing;
- `CorpusOutcome`;
- `ExpectedFailurePhase`;
- `run_corpus_case`;
- diagnostic matching;
- positive golden comparison;
- negative assertions;
- fixture collection.

---

### Corpus definitions

```text
phalcom-core/tests/core/language/corpus.rs
```

Defines the feature-labelled source-language corpus lanes reused by the dedicated corpus integration target.

---

### VM output abstraction

```text
phalcom-core/src/vm/output.rs
```

Defines VM-owned runtime output implementations.

---

### VM bootstrap

```text
phalcom-core/src/vm/bootstrap.rs
```

Defines full VM construction and the fallible bootstrap path.

---

### Canonical Universe materialization

```text
phalcom-core/src/modules/builtin_materialize.rs
```

Contains structured materialization and native-binding error propagation used during full VM bootstrap.

---

### Runtime output primitives

```text
phalcom-core/src/primitive/system.rs
```

Routes Phalcom-visible system output through the VM-owned output sink.

---

### Boolean primitive regression

```text
phalcom-core/src/primitive/boolean.rs
```

Contains safe Boolean constructor behavior and regression coverage for zero-argument construction and debug-output removal.

---

### CLI smoke tests

```text
phalcom-core/tests/cli_smoke/mod.rs
```

Retains subprocess coverage for executable-boundary behavior.

---

## 37. Architectural Summary

The previous corpus architecture used operating-system processes as both execution mechanism and isolation mechanism:

```text
N fixtures
    ↓
N `phalcom` processes
    ↓
N independent compiler/runtime worlds
```

The corrected architecture separates immutable compiler reuse from mutable runtime isolation:

```text
one language-corpus process
        │
        ├── one reusable immutable canonical/compiler world
        │
        ├── fixture A → fresh VM A → output A
        ├── fixture B → fresh VM B → output B
        ├── fixture C → fresh VM C → output C
        └── ...
```

This yields a more precise testing architecture:

```text
.ph fixture
    ↓
strict fixture configuration
    ↓
production ProgramCompiler
    ↓
structured compiled program
    ↓
fixture-local output sink
    ↓
fallible fresh VM bootstrap
    ↓
VM-dependent compilation
    ↓
runtime execution
    ↓
captured output + structured CorpusOutcome
    ↓
positive / negative / structural oracle
```

The design removes unnecessary process multiplication without weakening runtime isolation.

It also establishes clearer production boundaries for:

- VM output;
- bootstrap failure;
- host I/O;
- compiler failure provenance;
- runtime error containment;
- gradual static/runtime failure-phase behavior.

For the language corpus, the resulting model is:

> **share immutable compiler products, isolate mutable runtime state, execute through production paths, and preserve failures as structured architectural outcomes.**