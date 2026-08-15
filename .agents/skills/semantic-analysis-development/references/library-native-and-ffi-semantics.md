# Core, Standard Library, Native, and FFI Semantic Contracts

## Why library semantics belong here

As Phalcom's standard library grows, semantic tooling will need trustworthy signatures/effects
for operations whose implementation may be Phalcom, Rust, or mixed.

Do not solve this with scattered name checks such as:

```rust
if class == "String" && method == "size" { ... }
```

Prefer source-backed declarations and centralized trusted native metadata.

## Core source

Bundled `core.ph` is valuable semantic input because:

- selectors/classes are visible exactly as users see them;
- source ranges/docs can drive LSP;
- ordinary declaration surfaces need no hard-coded mirror;
- future type annotations can live with declarations.

Native implementations may accelerate behavior but must preserve the visible contract.

## Native return shape/type

A native method may require metadata when body is unavailable to source analysis.

Potential metadata:

```text
return runtime shape
declared/resolved type
may throw
may yield
may block
mutation scope
callback invocation/escape
allocation/resource behavior if analysis uses it
```

Unknown metadata -> conservative unknown, not optimistic purity.

## Strings

Semantic/type assumptions must respect Unicode API semantics:

- byte length versus scalar/code-point length versus grapheme count;
- indexing model;
- slice validity;
- normalization not automatic unless specified.

A proof that `index < byteLength` permits code-point indexing is invalid if APIs use different
units.

## Bytes/buffers

Track mutability/ownership semantics once specified:

- immutable bytes vs mutable buffer;
- slice/view vs copy;
- encoding conversion may fail;
- FFI borrowing/retention effects.

## Path/filesystem

Do not treat paths as normalized strings semantically.

Potential effects/errors:

- filesystem access;
- symlink-sensitive resolution;
- race/TOCTOU;
- platform-specific path form;
- blocking/yielding behavior.

Static analyzer should not "prove" filesystem facts from lexical path normalization.

## I/O

Reads/writes can be partial and can fail. Effect summaries may include `does_io`, `may_block`,
`may_yield`, `may_throw`/Result-return contract depending on API design.

Do not assume one write consumes all bytes unless API guarantees it.

## Process/OS

Process launch semantics include:

- argv/environment;
- exit status;
- stdio inheritance/pipes;
- signals/cancellation;
- blocking waits;
- platform errors.

These affect effects and static resource reasoning more than value shape.

## Option/Result

These should have precise semantic/type metadata because they are central to bug detection and
flow refinement.

Useful analyzer contracts:

- constructors produce known variant/type;
- pattern matching refines payload;
- map/andThen-like higher-order methods invoke callback under defined conditions;
- unwrap-like operations may be partial/diagnostic according to API.

Do not hardcode every method if the future type/protocol metadata can express the behavior.

## Fibers/concurrency library

Scheduler/fiber primitives should publish effect metadata:

- yields;
- joins/waits;
- cancellation points;
- callback execution context;
- fiber-local state.

This enables lints/proofs such as "blocking native operation on scheduler thread" if Phalcom
chooses to provide them.

## FFI packages

Mixed Rust/Phalcom package semantics need a trusted boundary.

Metadata should eventually cover:

- Phalcom-visible signature/type descriptors;
- ownership/retention of Phalcom objects;
- callback behavior;
- error translation;
- panic containment;
- may block/yield;
- thread/fiber affinity;
- unsafe/native trust level.

Rust `Send`/`Sync` does not automatically become a Phalcom user-visible concept unless the
language exposes it. Translate intentionally.

## Validation

Native metadata should be validated against generated bindings where possible. Drift between
Rust implementation and semantic signature can make checker/prover unsound.

Use conformance tests that call the runtime and compare declared semantic contracts for exact
properties that are testable.
