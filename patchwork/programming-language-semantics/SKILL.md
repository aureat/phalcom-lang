---
name: programming-language-semantics
description: Use when defining, reviewing, formalizing, or implementing Phalcom evaluation order, scope, closures, message sends, classes/metaclasses, exceptions, modules, control flow, reflection, effects, fibers, runtime behavior, static/dynamic correspondence, or language changes whose exact observable meaning must be specified.
compatibility: Designed for agents working on Phalcom specifications, parser/compiler/VM semantics, checker semantics, runtime contracts, semantic analysis, static proving, and language evolution.
---

# Programming Language Semantics for Phalcom

This skill answers one question: **what does a Phalcom program mean?**

It is not a compiler cookbook and it is not a type-system specification. It supplies the semantic machinery needed to state language behavior precisely enough that the parser, compiler, VM, LSP, checker, prover, optimizer, reflection APIs, and tests can agree on one language.

**REQUIRED BACKGROUND:** Use `language-design` when choosing among language designs. Use `phalcom-semantic-model` for repository semantic identities and facts. Use `type-theory` for type relations. Use `static-analysis-and-abstract-interpretation` when computing safe approximations rather than defining execution.

## Core doctrine

Keep these layers distinct:

```text
surface syntax
    ↓ denotes
source-language semantics
    ↓ implemented/refined by
lowering + bytecode + VM

source-language semantics
    ↓ approximated by
semantic analysis / abstract interpretation

source-language semantics + typing rules
    ↓ justify
checker guarantees

source-language semantics + contracts
    ↓ justify
static proofs
```

A parser node is not a semantic rule. A bytecode instruction is not a semantic rule. An LSP `ValueShape` is not a semantic rule. A proposed type annotation is not a semantic rule unless it changes execution by an explicitly ratified mechanism.

## Semantic vocabulary

Choose the least powerful formalism that makes the question unambiguous:

```text
Static resolution:       Γ ⊢ x ↦ b
Type checking:           Γ ⊢ e : T
Big-step evaluation:     ρ; σ ⊢ e ⇓ o; σ'
Small-step evaluation:   ⟨e, ρ, σ, κ⟩ → ⟨e', ρ', σ', κ'⟩
Trace semantics:         C --α--> C'
Hoare judgment:          {P} c {Q}
Refinement/simulation:   S ≈ V
```

Notation is a tool, not a goal. If executable pseudocode states a rule more precisely than incomplete inference notation, use the pseudocode.

## Phalcom semantic invariants

1. **Message send is the semantic computational primitive.** Specialized bytecodes and inlining must refine send semantics, not replace it with a second language.
2. **Selector identity is independent of type metadata.** Type annotations do not silently become overload keys.
3. **A class is an object.** Class-side behavior follows the metaclass tower and ordinary lookup principles.
4. **`super` is not a different receiver.** It changes lookup start while preserving the current receiver.
5. **Binding identity is not spelling.** Shadowing creates distinct bindings even when text is identical.
6. **Mutable lexical capture is storage sharing.** Captured mutable variables require location/cell semantics or an equivalent model.
7. **Block creation is not block execution.** Latent effects do not occur merely because a block literal is evaluated.
8. **Abrupt control is explicit.** Return, non-local return, throw, break, continue, yield, cancellation, and similar transfers must not disappear into implicit host-language control.
9. **Module identity, namespace, dependency, loading, and initialization are distinct axes.** Do not model imports as textual inclusion unless that is actually the language semantics.
10. **Reflection is semantic.** Observable method identity, class identity, source metadata, access checks, and reflective mutation constrain optimization.
11. **Fibers require scheduler semantics even when cooperative.** Yield points, blocking native calls, cancellation, and fairness are observable.
12. **Static claims are downstream of dynamic meaning.** Checker/prover rules may strengthen guarantees, but cannot invent a parallel runtime object model.

## Required feature analysis

For any new or changed language feature, answer all of the following before considering semantics complete:

| Axis | Required question |
|---|---|
| Syntax | What surface forms denote the construct? |
| Values | What runtime values/descriptors can it produce? |
| Identity | Which identities are stable and observable? |
| Evaluation | What is evaluated, in what order, and how many times? |
| Binding | Which declaration does each name occurrence denote? |
| Store | What state may be allocated, read, or mutated? |
| Dispatch | What selector is used, what receiver is preserved, where does lookup begin? |
| Access | Which lexical/dynamic authority is required? |
| Control | What normal and abrupt outcomes can occur? |
| Effects | What observations may escape the expression? |
| Modules | How does the rule interact with module identity/loading? |
| Fibers | Can it yield, block, cancel, or cross a fiber boundary? |
| Reflection | What can reflection observe or mutate? |
| Typing | What static judgment corresponds to the dynamic behavior? |
| Analysis | What sound approximation can tooling compute? |
| Proving | Which proof obligations/trust assumptions arise? |
| Lowering | Which implementation transformations preserve the rule? |
| Tests | Which programs distinguish this rule from plausible alternatives? |

## Formalization workflow

1. **Define observations first.** Decide what users can distinguish: values, exceptions, output, mutation, identity, reflection, traces, scheduling, termination.
2. **Define semantic domains.** Values, environments, store, classes, methods, modules, frames, outcomes, events.
3. **Define static identities.** Bindings, selectors, classes/modules, lexical access context.
4. **Specify evaluation contexts/order.** Effects make unspecified order a language feature whether intended or not.
5. **Specify the core dynamic relation.** Start with literals, names, sequencing, assignment, blocks, sends, control.
6. **Specify derived forms.** Show how sugar lowers without changing observations.
7. **State meta-properties.** Determinism where applicable, safety claims for typed subsets, and simulation obligations for lowering.
8. **Add executable conformance fixtures.** Every disputed semantic choice needs a program that distinguishes alternatives.
9. **Connect downstream analyses.** State what the LSP/checker/prover may conclude and what uncertainty remains.

## Reference map

### Formal foundations

- [Semantic judgments and methods](references/semantic-judgments-and-methods.md)
- [Semantic domains and machine state](references/semantic-domains-and-state.md)
- [Operational semantics](references/operational-semantics.md)
- [Evaluation order and evaluation contexts](references/evaluation-order-and-contexts.md)
- [Abstract machines, continuations, and frames](references/abstract-machines-continuations-and-frames.md)
- [Traces, observations, and nondeterminism](references/traces-observations-and-nondeterminism.md)
- [Contracts and axiomatic semantics](references/contracts-and-axiomatic-semantics.md)
- [Metatheory and proof techniques](references/metatheory-and-proof-techniques.md)

### Phalcom runtime meaning

- [Environments, stores, and binding](references/environments-stores-and-binding.md)
- [Objects, classes, and message dispatch](references/objects-classes-and-message-dispatch.md)
- [Closures, control, and exceptions](references/closures-control-and-exceptions.md)
- [Modules, imports, and initialization](references/modules-imports-and-initialization.md)
- [Effects and observational equivalence](references/effects-and-observational-equivalence.md)
- [Reflection and metaprogramming](references/reflection-and-metaprogramming.md)
- [Fibers, concurrency, and scheduling](references/fibers-concurrency-and-scheduling.md)

### Correspondence and implementation

- [Static/dynamic correspondence](references/static-dynamic-correspondence.md)
- [Type safety, progress, and preservation](references/type-safety-progress-and-preservation.md)
- [Compiler correctness and semantic-preserving lowering](references/compiler-correctness-and-lowering.md)
- [Formalizing Phalcom](references/formalizing-phalcom.md)
- [Semantic specification patterns](references/semantic-specification-patterns.md)
- [Comparative semantics and reading](references/comparative-semantics-and-reading.md)
- [Review and validation scenarios](references/review-and-validation-scenarios.md)

## Stop conditions

Do not approve a semantic change when any of these is true:

- implementation behavior is being used as the only definition of the language;
- two subsystems independently implement name resolution or dispatch with incompatible rules;
- evaluation order is described as "obvious" rather than specified;
- control transfer is modeled with host-language `return`/panic and target identity is lost;
- compiler optimization assumes purity the semantic model has not established;
- reflection can distinguish an optimization the proposal claims is invisible;
- static analysis turns lack of knowledge into a runtime fact;
- a checker guarantee depends on native/reflective behavior outside its stated trust boundary;
- module or fiber semantics are postponed even though the feature crosses those boundaries.

## Completion standard

A semantic change is mature when an implementer can answer **what happens**, **why that is the language rule**, **which observations distinguish it**, **how compiler/VM execution refines it**, **what static tools may soundly infer**, and **which tests would catch semantic drift**.
