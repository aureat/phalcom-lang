# Dispatch, Members, Call Sites, Reflection, and Callable Summaries

Dispatch is where Phalcom's dynamic object model, selector identity, inheritance, reflection, inference, typing and optimization meet. A semantic model of dispatch must describe the language that the VM executes; it must not quietly replace dynamic lookup with whatever static information happens to be available.

## 1. Dispatch is a semantic pipeline

Model an ordinary send conceptually as:

```text
1. evaluate receiver
2. evaluate/build call arguments in language-defined order
3. construct canonical selector from the syntactic call shape
4. determine lookup mode/dispatch side
5. choose lookup start
6. search method/member surface according to runtime inheritance rules
7. enforce access/visibility rules where applicable
8. invoke selected callable with original receiver
9. propagate normal or abrupt completion
10. use fallback/dynamic behavior if the language defines one
```

Every step can affect semantics. A static analyzer may approximate one or more steps, but it should preserve the same conceptual order and uncertainty.

## 2. Selector identity is load-bearing

Phalcom ordinary dispatch is selector-oriented. Canonical selector formation must agree across parser/compiler/runtime/reflection/semantic analysis.

Do not identify a callable by:

- base method name only;
- guessed arity only;
- pretty-printed source;
- parameter type annotations;
- source ranges;
- declaration order.

**CURRENT:** `CallableId` contains owner class, canonical selector and `DispatchSide`. Current tests explicitly distinguish the same selector implemented by different classes and require independent summaries.

A future dedicated `CanonicalSelector` newtype may improve Rust API safety; the semantic requirement is the canonical selector contract, not the exact Rust representation.

## 3. Static typing does not silently select ordinary methods

Keep this invariant unless a ratified feature explicitly changes it:

```text
ordinary selector identity = dynamic call syntax + Phalcom selector rules
```

not:

```text
ordinary selector identity = syntax + inferred/declarative parameter types
```

If future `@typecase`, multimethods or typed dispatch are introduced, model them as an explicit dispatch layer or callable object with specified ambiguity/specificity rules. Do not allow the checker to create a hidden second method table.

## 4. Receiver categories

Static dispatch approximation should distinguish receiver kinds that have different lookup semantics:

```text
Instance(ClassId)
ClassObject(ClassId)
Module(ModuleId)
Callable(CallableId)
Family { receiver, base }
Union(...)
Unknown / dynamic
SuperContext { receiver, lookup_start }
```

The exact enum can differ. The important rule is not to collapse class objects, instances, modules and `super` merely because all can participate in member-like syntax.

## 5. Instance side and class side

Instance-side and class-side members are distinct semantic spaces. A class-side constructor/factory can return an instance without becoming an instance method.

Lookup needs at least:

```text
receiver semantic category
requested canonical selector
DispatchSide
current access context
class hierarchy/surface
```

Field storage side and method dispatch side also need explicit representation where language semantics distinguish them.

## 6. Inheritance lookup returns the actual declaration target

Suppose `Dog` inherits `Animal`, and `speak()` is declared only on `Animal`. A send to a known `Dog` instance resolves to the `Animal` declaration while preserving `Dog` as receiver knowledge.

A resolved dispatch fact should conceptually carry:

```text
requested receiver knowledge
lookup start
canonical selector
selected CallableId
actual declaring owner
side
access result
resolution strength/exactness
```

Returning only `Dog.speak()` as a fabricated owner loses navigation, reflection and invalidation identity.

## 7. `super` is a lookup context, not a superclass instance

`super` preserves the current receiver while changing where lookup begins. Conceptually:

```text
ordinary send:
  receiver = self
  lookup_start = dynamic class of receiver / ordinary current rules

super send:
  receiver = self
  lookup_start = superclass of current declaring class
```

Therefore `super` should not be modeled as `Instance(Superclass)`. That shortcut breaks overrides, `self` identity inside the callee, visibility context, diagnostics and optimizer assumptions.

A useful operational form is:

```text
DispatchContext {
  receiver_fact,
  lookup_start,
  side,
  lexical_declaring_class,
}
```

## 8. Implicit-self sends coordinate lexical resolution and dispatch

Where Phalcom syntax permits a bare name to become an implicit-self send, lexical resolution must run first according to normative rules:

```text
if a valid lexical binding resolves:
    use binding
else if syntax/context permits implicit self:
    attempt self dispatch
else:
    unresolved
```

Do not turn every unresolved identifier into an implicit send as an LSP convenience. That manufactures semantics and can corrupt rename/reference results.

## 9. Member surfaces versus runtime tables

A source `ClassSurface` is statically known declaration information. The VM may have runtime method tables, metaclass state, native methods and reflection-driven mutation.

Keep the distinction:

```text
source/member surface          source-known declarations and metadata
runtime dispatch table         live executable lookup state
semantic dispatch approximation what analysis can justify from source/runtime contracts
```

A completion query can use source surfaces even when runtime mutation means static resolution is not closed-world exact. An optimizer needs a stronger guard/version if it assumes the runtime table cannot change.

## 10. Call-site selector construction

Phalcom selector identity can depend on labels/call form. A call analyzer should preserve:

```text
source evaluation order
base selector name
known positional/label lanes
canonical labels in selector order
pack/spread/dynamic label contributions
exact source ranges of arguments
closure/block literal identity/effects
```

If a dynamic pack prevents constructing one exact selector, the result is a dynamic/conservative call, not a guessed selector. Static tools can still provide partial completion/signature candidates from a selector family without claiming exact dispatch.

## 11. Families and callable values

An open method family represents deferred selector completion/receiver dispatch. It differs from a captured concrete method/callable.

Conceptually:

```text
Family {
  receiver: ValueShape,
  base: selector base
}
```

Calling the family can incorporate call-time labels to produce a selector. Preserve this deferred nature in inference. Do not prematurely bind the family to an arbitrary method merely because one current candidate exists.

## 12. Dispatch over unions

For receiver shape:

```text
Union(Instance(A), Instance(B))
```

a static resolution query may produce:

```text
ExactOne(target)                     if both paths provably select same declaration
FiniteCandidates({targetA,targetB})  if bounded and distinguishable
Dynamic/Unknown                      if lookup cannot be bounded reliably
MissingOnSomePaths                   if selector not supported by every alternative
```

Completion policy can choose to show union/intersection member surfaces, but the semantic query should preserve enough information to distinguish "member exists on at least one possible receiver" from "member exists on all possible receivers." This is a classic may/must distinction.

## 13. Visibility and access context

Access legality can depend on the current class/module/lexical context, not only the receiver. Do not bake visibility filtering into a reusable class surface in a way that loses context.

Conceptually:

```text
resolve(receiver, selector, access_context) -> DispatchResult
```

Navigation may still know the target even when access is illegal; diagnostics may report the violation; completion may rank/filter it. Separate target identity from access policy where possible.

## 14. Call-site argument mapping

A call site needs a stable mapping from source arguments to callable parameter slots:

```text
CallSite {
  target/candidates,
  selector,
  arguments: [
    {source_order, label, range, value_fact, parameter_slot?}
  ],
  dynamic_pack_state,
}
```

This mapping supports:

- parameter-shape inference;
- future type constraints;
- signature help;
- diagnostics for missing/extra/duplicate labels;
- closure-effect propagation;
- refactoring call labels.

Do not rederive a different mapping in LSP, checker and compiler.

## 15. Evaluation order is part of call semantics

Even if selector construction is mostly syntactic, receiver/argument expressions can have side effects, throw, yield or mutate dispatch state. The analyzer/compiler must preserve normative evaluation order.

A static fact about a later argument cannot be moved before an earlier side-effectful argument without proving the transformation semantics. This matters for future optimization and proof VC generation.

## 16. Callable summary contract

A callable summary is a compact interprocedural abstraction, not runtime reflection metadata by default. Conceptually:

```text
CallableSummary {
  id: CallableId,
  parameter_facts,
  return_fact,
  effects,
  direct_dependencies,
  optional field/global reads/writes,
  generation/revision validity,
}
```

Possible future fields include declared/inferred types, thrown error domain, no-return facts, closure escape behavior, generic templates and postconditions. Add a field only when its semantics, merge rule, dependency ownership and invalidation are defined.

## 17. CURRENT summary and inference behavior

**CURRENT:** Phalcom semantic analysis computes callable summaries, propagates return knowledge across call chains/modules, tracks caller/callee dependencies and joins call-site parameter evidence. Current tests exercise:

- multi-hop return propagation;
- recursive callables with concrete evidence;
- separate summaries for same selector under different owners;
- parameter/return propagation across imported modules;
- caller edits removing stale parameter contributions.

This is already an interprocedural semantic substrate, not merely per-feature LSP guessing.

## 18. Return semantics

Return analysis must combine only reachable result-producing exits under Phalcom's dynamic semantics. Distinguish:

```text
explicit return values
tail/implicit result if language specifies one
constructor-specific return convention
non-local returns from blocks
throw/abrupt exits
non-returning paths
```

Do not join an unreachable normal path as `Unknown`; that loses precision. Future language typing also needs to distinguish inferred runtime shape, inferred language type and declared result type.

## 19. Recursive summaries and SCCs

A recursive call graph creates equations:

```text
S_A = F_A(S_B)
S_B = F_B(S_A)
```

Solve by worklist/SCC iteration over an abstract domain with termination policy. A recursion guard that simply returns `Unknown` on the first back-edge is safe only if its precision policy is intentional; it can be much less precise than a fixed point.

A checker may impose additional rules such as requiring explicit annotations for some recursive inference. That is a typing policy, not a reason to weaken advisory summary analysis everywhere.

## 20. Higher-order callables and blocks

Block construction and block execution are separate events. A callable summary can record that parameter `N` may be invoked:

```text
invokes_parameter(N)
```

Then a caller passing a known block can propagate block effects only when the callee may invoke it.

Future refinements may distinguish:

```text
invoked never / once / maybe-many
synchronous / deferred
escapes / stored
may return non-locally
may yield/suspend
captures mutable state
```

Do not assume a block executes merely because it is passed.

## 21. Reflection and dynamic mutation

Reflection/open classes can invalidate closed-world assumptions in several ways:

```text
dynamic selector construction
reflective method lookup/invocation
method installation/replacement/removal
class hierarchy mutation, if permitted
runtime-generated classes/methods
fallback such as does-not-understand behavior, if specified
native code mutating language-visible dispatch state
```

Static analysis should represent a conservative dynamic effect/candidate uncertainty when it cannot close the target set.

A call graph is not "complete" merely because every source send currently resolves.

## 22. Reflection × optimization

Suppose an optimizer sees all current sends to `C.foo()` target one method and wants to inline it. If reflection can replace `foo()` at runtime, source-known monomorphism is insufficient.

Safe optimization needs one of:

```text
class/method table frozen by language/build mode
runtime method-table version guard
inline-cache guard on receiver class + selector version
global dispatch epoch + deoptimization/fallback
proof that reflective mutation cannot reach this state
```

Semantic analysis can provide candidate targets. Optimization owns the stronger operational validity contract. Do not upgrade advisory resolution into an unguarded optimization fact.

## 23. Reflection × typing/proving

A type checker can describe allowed sends while runtime reflection remains dynamic, but proof claims about exact target bodies need stronger assumptions.

Separate:

```text
"receiver type supports selector s"          typing/member availability
"runtime lookup currently resolves to m"    dispatch fact
"every execution will invoke body m"        proof/closed-world assumption
```

These are different propositions. A static prover must account for reflection/mutation or restrict the proof mode with explicit assumptions.

## 24. Native/FFI callables

A native primitive needs semantic metadata that matches its actual runtime contract:

```text
selector/owner/side
argument mapping
return behavior
may-throw
may-yield/block
state mutation
host boundary
runtime type/contract metadata when ratified
```

Prefer generating/deriving this from one primitive declaration source so compiler bootstrap, docs, semantic engine and checker do not drift.

Never give native code a stronger pure/non-throwing summary merely to improve inference.

## 25. Dispatch-result domain

Avoid `Option<CallableId>` if the semantics need more states. Conceptually:

```rust
enum DispatchResult {
    Resolved(ResolvedDispatch),
    Candidates(Vec<ResolvedDispatch>),
    Missing,
    Ambiguous,
    Dynamic(DynamicReason),
    RecoveryBlocked,
}
```

This lets consumers behave correctly:

- definition can navigate exact result;
- signature help can show bounded candidates;
- completion can use a family/partial selector;
- checker can report missing selector under a sufficiently known receiver type;
- optimizer can require exact+guarded result.

The exact current Rust API may remain simpler; this is a semantic completeness test for future changes.

## 26. Testing obligations

Dispatch/callable changes should test:

- same selector under different classes;
- same member name on instance and class side;
- inherited method resolves to actual declaring owner;
- override shadows inherited declaration;
- `super` starts lookup at superclass but preserves current receiver;
- lexical binding beats implicit-self send where specified;
- dynamic labels/packs do not fabricate exact selectors;
- union receiver: member available on all vs only some alternatives;
- recursion/mutual recursion reaches stable summary;
- block passed but never invoked does not inherit execution effects;
- known invoked callable parameter propagates relevant block effects;
- imported callable summary propagates cross-module;
- runtime/native summary matches actual primitive behavior;
- reflective/dynamic send prevents unsound closed-target claims;
- type annotations do not change ordinary selector identity;
- incremental method-body/selector edits invalidate exactly the required call dependencies.

Metamorphic property:

```text
adding a semantically compatible type annotation to a parameter
must not change ordinary runtime dispatch target
```

unless an explicitly ratified typed-dispatch feature says otherwise.

## 27. Failure modes to reject

Reject base-name-only dispatch, treating `super` as a superclass instance, using type annotations as ordinary overload keys, fabricating exact selectors from dynamic packs, assuming a passed block executes, recursively analyzing callees without a summary/fixed point, declaring a call graph closed in the presence of unresolved reflection, storing source surfaces as if they were live runtime method tables, and feeding heuristic dispatch facts directly into unguarded optimization/proof claims.

## 28. Review questions

Before approving dispatch/callable semantics, answer:

- How is the canonical selector constructed?
- What is the receiver category and dispatch side?
- What is lookup start, especially for `super`?
- Which declaration actually owns the selected member?
- What access context is required?
- What happens when labels/packs are dynamic?
- Can the result be a candidate set rather than one target?
- Does reflection allow the target set to change at runtime?
- What summary inputs/outputs/effects cross a call?
- What is the recursion/fixed-point policy?
- Who owns each parameter/effect contribution so edits can retract it?
- Does typing describe member legality without altering ordinary dispatch?
- What guard would an optimizer/prover need before treating target identity as exact?
