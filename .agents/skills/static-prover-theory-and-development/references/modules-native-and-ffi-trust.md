# Modules, Native Code, and FFI Trust

## Module summaries

Proofs depend on imported declarations/contracts. Cache dependencies must include module/package identity and semantic revision.

## Top-level initialization

Module initialization can mutate globals, perform IO or throw. Proofs about global state after import need an initialization contract/summary, not only symbol resolution.

## Native primitives

Rust implementations are opaque to Phalcom source prover unless:

- modeled with a trusted contract;
- translated to a proof model;
- verified separately.

Do not infer purity from `#[native]` or absence of source body.

## FFI boundary

Rust functions can:

- panic;
- mutate aliased data;
- block;
- call back into Phalcom;
- retain handles;
- violate declared assumptions if unsafe code is wrong.

A safe FFI contract needs type, ownership/lifetime, mutation/effect, exception/panic and blocking semantics.

## Trust tiers

Possible policy:

```text
verified Phalcom source
trusted core/native contract
runtime-checked external contract
untrusted dynamic/FFI boundary
```

Proof engine should record which tier each assumption uses.

## Package security

Untrusted packages must not mark their own native metadata as system-trusted. Signature/build policy may be needed when native packages arrive.

## Versioning

Changing Rust primitive behavior without updating its semantic contract can invalidate proofs. Tie contract version to build/package revision and test native conformance against declared contracts.

---

## Deep treatment: trust manifests and module-world dependencies

### Module initialization state machine

A module may conceptually be:

```text
Uninitialized
Initializing
Initialized(summary/state)
Failed(error)
```

Cycles can expose partially initialized state depending on Phalcom's ratified module semantics. A proof about globals after import must use the actual initialization guarantee, not an idealized “imports are declarations only” model.

### Native semantic manifest

A trusted primitive summary should identify:

```text
callable semantic ID
argument/result type contract
requires/ensures
may-write regions
may-throw/panic mapping
may-yield/block/callback
ownership/retention behavior
semantic version/hash
trust authority
```

The Rust function body is not proof-visible unless separately modeled/verified. Conformance tests bind implementation behavior to the manifest.

### Panic and process effects

Rust panic may abort, unwind, or be caught depending on build/runtime integration. The FFI contract must map it to actual Phalcom behavior. Treating panic as an ordinary Phalcom exception without runtime support is unsound.

### Retained handles

Native code may retain references beyond the call. This matters for GC rooting and for proof alias/effect models: future native callbacks could mutate retained objects. A summary needs retention/escape effects when relevant.

### Trust authority

Third-party packages must not self-assert system trust. Consider controlled signatures/build metadata or a compiler-owned allowlist for trusted axioms. User contracts can still be runtime-checked or separately proved.

### Versioned dependency

Proof cache should depend on the semantic manifest hash, not merely native function name. If implementation changes while manifest does not, conformance testing must catch divergence; if manifest changes, dependent proofs invalidate.
