# Compiler–Runtime–Semantic Conformance

## 1. Static analysis must model the executable language

The semantic engine is not authoritative about runtime behavior. For any fact that claims correspondence with execution, establish a conformance path from language specification to compiler lowering/bytecode to VM behavior to static approximation.

```text
normative semantics
      |
      +--> compiler lowering / bytecode
      +--> VM dispatch/control/storage
      +--> semantic analyzer approximation
```

If these diverge, LSP/checker/prover may confidently describe a language that does not execute.

## 2. High-risk correspondence points

For Phalcom, test especially:

- selector canonicalization and argument labels;
- receiver/argument evaluation order;
- instance/class/metaclass lookup;
- `super` lookup start and actual receiver;
- getters/setters/subscripts/method families;
- lexical bindings/captures/upvalues;
- block invocation and non-local return;
- field/class storage semantics;
- module import/initialization behavior;
- exceptions/throws and abrupt completion;
- reflective invocation and method mutation;
- native primitives and FFI contracts;
- fiber yield/suspension when flow/effects start modeling concurrency.

## 3. Current dynamic-first architecture

**CURRENT:** the draft typing architecture analysis documents that compilation goes directly from parsed AST to VM-coupled bytecode, current callable metadata has selector/arity information but no formal types, and current LSP `ValueShape` is advisory. Future static infrastructure must therefore be introduced as a model of current dynamic semantics, not assumed to be already enforced by runtime.

## 4. Differential fixtures

Create small programs where runtime result/trace is compared to semantic expectations. Example for evaluation order:

```phalcom
trace("receiver").m(trace("a"), b: trace("b"))
```

Runtime trace establishes actual order. HIR/semantic tests assert the same ordering dependencies. Similar fixtures can validate `super`, closure capture, and non-local returns.

## 5. Dispatch conformance

Centralize selector encoding if possible. At minimum have tests that feed the same source forms through compiler/runtime selector construction and semantic selector construction and assert equality.

A semantic target set may be broader than runtime's single target because of abstraction. Required relationship for a sound may-analysis:

```text
actual_runtime_target ∈ static_candidate_set
```

unless result is explicitly Dynamic/Unbounded. The reverse is not required.

## 6. Native contracts

Rust-native behavior is invisible to source analysis unless described. A native semantic contract should be checked against implementation with tests. If a primitive is marked pure/non-throwing/returns `String`, conformance tests should exercise it and fail when implementation changes incompatibly.

Do not infer purity from “implemented in Rust.” Native code can allocate, invoke user code, mutate classes, throw, yield, or access IO.

## 7. Reflection and open-world behavior

If runtime reflection can modify method tables, static “unique target” facts need either:

- advisory status;
- a closed-world/profile assumption;
- runtime class/method version guard and fallback;
- proof that mutation cannot occur in the region.

Conformance tests should mutate dispatch state and verify optimized/typed assumptions are invalidated or guarded.

## 8. Compatibility tests as language locks

A conformance fixture is often more durable than an implementation unit test. When behavior is normative, keep source program + expected output/error/trace so compiler refactors, semantic-engine changes, and optimizer changes all run against the same semantic lock.

## 9. Review questions

1. Which normative runtime rule justifies this static transfer/resolution?
2. Is the analyzer allowed to over-approximate here, and by how much?
3. Is selector/evaluation-order logic duplicated?
4. Does native behavior have an explicit semantic contract?
5. Could reflection invalidate this fact?
6. Is there a runtime fixture that would catch semantic drift?
