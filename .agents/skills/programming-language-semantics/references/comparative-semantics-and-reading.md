# Comparative Semantics and Reading

Use other languages as semantic case studies, not templates. Ask: **what problem forced this language to make a semantic distinction explicit, and do Phalcom's assumptions match?**

## 1. Smalltalk and Self

Study for message send, metaclasses, blocks, method dictionaries, late binding, dNU-style fallback, and inline-cache reasoning.

Phalcom differs in selector syntax, closed source classes, Option/no-surface-nil direction, attributes, modules, and future typing.

## 2. ECMAScript specification

Useful as specification-engineering precedent. It explicitly models environment records as specification mechanisms, completion records for abrupt control, execution contexts, algorithmic evaluation order, module records, and job scheduling.

Lesson is not to copy JavaScript behavior. Lesson is to invent explicit semantic records for behavior that would otherwise hide inside implementation control flow. The current ECMAScript specification continues to use dedicated Completion Records and Environment Records for this purpose. 

## 3. Python / CPython

Study dynamic class/metaclass behavior, modules/import caching, exceptions, reflection, descriptor-like indirection, and native-extension trust boundaries.

Python attribute lookup is not Phalcom selector dispatch; compare problems, not syntax.

## 4. Ruby

Study blocks/control distinctions, `method_missing`, dynamic metaprogramming, and reflective method mutation. Phalcom's closed source classes mean Ruby's open-class assumptions cannot be imported wholesale.

## 5. Rust

Study semantic separation between surface language and compiler IR, unsafe/native trust boundaries, explicit drop/panic effects, and analysis-friendly intermediate representations.

Do not leak Rust ownership/lifetime semantics into Phalcom user semantics merely because FFI implementation uses Rust.

## 6. OCaml / Haskell

Study formal cores, ADTs, pattern semantics, modules, typed calculi, and effect/control extensions. Their functional assumptions differ sharply from mutable reflective Phalcom objects.

## 7. Erlang / BEAM

Study scheduling, process/failure isolation, mailboxes, and supervision. Contrast isolated processes with Phalcom fibers potentially sharing one mutable object heap.

## 8. Go

Study goroutine/channel and memory-model specification style. Do not infer Phalcom fibers are parallel simply because APIs look concurrent.

## 9. CompCert

Study compiler-correctness precedent: source/intermediate/target languages are assigned formal semantics and compilation passes are connected by semantic-preservation simulations. Current CompCert documentation describes compiler correctness in terms of source/target semantic preservation and composes simulations across passes.

Phalcom can adopt pass invariants and semantic refinement discipline without immediately pursuing a full mechanized verified compiler.

## 10. PFPL / PLFA / Software Foundations-style developments

Study defining statics and dynamics together, progress/preservation, substitution/induction, evaluation contexts, abstract machines, and distinction between declarative derivations and algorithms. PLFA's formal development is a useful concrete example of progress and preservation being proven from an explicit reduction relation.

## 11. Reading questions

For every precedent answer:

1. What are observable behaviors?
2. What is core evaluation relation?
3. How is abrupt control represented?
4. What is name/environment/store model?
5. How is dynamic dispatch defined?
6. What reflection invalidates static assumptions?
7. What concurrency/memory model exists?
8. Which assumptions differ in Phalcom?

## 12. Primary-source reading anchors

Prefer primary specifications/formal developments when making semantic decisions:

- language specifications for ECMAScript, Rust, Python where normative behavior matters;
- original/formal Smalltalk/Self descriptions for object-model questions;
- CompCert's documented semantic preservation development for compiler simulation;
- PFPL and mechanized PLFA developments for metatheoretic technique.

## 13. Cargo-cult warning

A beautiful mechanism is wrong if it solves a problem Phalcom does not have or violates ratified invariants. Import the reasoning pattern, then re-derive Phalcom rule.
