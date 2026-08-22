# Pyrefly semantic behavior: bindings, flow, exports, and module facts

## Purpose

This document reconstructs what Pyrefly computes before and during type solving. It focuses on semantic behavior: how syntax becomes binding keys, how flow state is represented, how exports cross modules, how first-use inference works, and how the solver consumes those products.

Pyrefly does not infer by repeatedly walking an AST and guessing a class. It first builds a binding graph with stable indexed keys and explicit flow relationships. The solver evaluates that graph.

## Evidence boundary

Pinned revision: 43467e64e36550f232a18e89f24fda79b1020b6b.

Primary files:

- [ARCHITECTURE.md](https://github.com/facebook/pyrefly/blob/43467e64e36550f232a18e89f24fda79b1020b6b/ARCHITECTURE.md) — define/use/anonymous/export keys and phi-style recursive bindings.
- [bindings.rs](https://github.com/facebook/pyrefly/blob/43467e64e36550f232a18e89f24fda79b1020b6b/pyrefly/lib/binding/bindings.rs) — binding table, indexed keys, scopes, deferred names, initialization state.
- [binding.rs](https://github.com/facebook/pyrefly/blob/43467e64e36550f232a18e89f24fda79b1020b6b/pyrefly/lib/binding/binding.rs) — binding key/value definitions and flow metadata.
- [stmt.rs](https://github.com/facebook/pyrefly/blob/43467e64e36550f232a18e89f24fda79b1020b6b/pyrefly/lib/binding/stmt.rs) — statement traversal and binding construction.
- [expr.rs](https://github.com/facebook/pyrefly/blob/43467e64e36550f232a18e89f24fda79b1020b6b/pyrefly/lib/binding/expr.rs) — expression binding construction.
- [narrow.rs](https://github.com/facebook/pyrefly/blob/43467e64e36550f232a18e89f24fda79b1020b6b/pyrefly/lib/binding/narrow.rs) — narrowing operation representation.
- [exports.rs](https://github.com/facebook/pyrefly/blob/43467e64e36550f232a18e89f24fda79b1020b6b/pyrefly/lib/export/exports.rs) — module export products.
- [type_order.rs](https://github.com/facebook/pyrefly/blob/43467e64e36550f232a18e89f24fda79b1020b6b/pyrefly/lib/solver/type_order.rs) — semantic lookup boundary consumed by subset solving.

Local Phalcom seams:

- phalcom-semantic/src/identity.rs
- phalcom-semantic/src/surface.rs
- phalcom-semantic/src/dispatch.rs
- phalcom-semantic/src/snapshot.rs
- phalcom-lsp/src/semantic/flow.rs
- phalcom-lsp/src/semantic/infer.rs
- phalcom-lsp/src/semantic/module_graph.rs

## Three semantic stages

Pyrefly's architecture describes:

~~~text
1. determine module exports
2. convert each module into bindings, scopes, and flow
3. solve bindings, including cross-module answers
~~~

The order is operationally important. Exports must exist before imports can refer to them. Bindings must exist before the solver can ask for a binding answer. Flow and scope facts must be attached to binding keys before recursive answer solving can construct phi-like equations.

## Binding table design

The binding table is an indexed semantic database for one module:

~~~text
Index<K>
    assigns a compact typed index to a key

IndexMap<K, V>
    stores key-indexed values in dense indexed storage
~~~

A key identifies what is being solved. A value stores the data needed to solve it. The index is used instead of carrying large key/value objects through every reference.

Conceptual examples:

~~~text
define int@0 = imported built-in int
define x@1   = literal 4 with declared/int relation
use x@2      = reference to x@1
anon @2      = statement result that has no named consumer
export x     = exported reference to x@2
~~~

The byte offset in Pyrefly's Python binding key disambiguates occurrences inside a source file. Phalcom should not make offsets the durable identity of a semantic entity. Use a stable BindingId or occurrence ID and retain ranges separately.

## Why define, use, anonymous, and export are separate

Each key category has a different consumer and invalidation behavior:

- define keys own declarations or assignments;
- use keys reference an existing definition/flow value;
- anonymous keys represent expressions/statements whose result is checked but not named;
- export keys provide cross-module references;
- class/function/field keys carry declaration-specific behavior;
- type-alias and metadata keys support specialized semantic queries.

A change to an expression result need not invalidate a declaration surface. An export consumer can depend on an export key rather than every internal expression.

Phalcom should model its own categories:

~~~rust
enum BindingKind {
    Definition,
    Use,
    Anonymous,
    Export,
    MemberSend,
    FieldRead,
    FieldWrite,
    Callable,
    Parameter,
    Return,
    FlowJoin,
}
~~~

The set must follow Phalcom semantics, not Pyrefly naming.

## Scope lookup result

Binding construction does not return only an index. It also returns flow initialization status and scope information. A name can be:

~~~text
Found {
    key/index,
    initialized: Yes | Conditionally | No | DeferredCheck,
    module-scope flag,
    outer-class-type-parameter flag
}
NotFound
~~~

Initialization is not just a boolean:

- Yes means definitely initialized under modeled flow;
- Conditionally means some incoming path initializes it;
- No means the analyzer sees no valid initialization;
- DeferredCheck means the answer depends on termination keys or later Never facts.

The binding stage records a deferred obligation instead of prematurely reporting a final error.

Phalcom equivalent:

~~~rust
struct BindingLookup {
    binding: Option<BindingId>,
    initialization: InitializationState,
    scope: ScopeId,
    source: SourceOrigin,
}
~~~

## Flow keys and phi values

Assignments do not always mutate one variable node. In flow-sensitive analysis, a new assignment creates a new value/version and later reads refer to a join of versions.

~~~text
x = 1
if condition:
    x = "s"
print(x)
~~~

becomes:

~~~text
x@1 = Literal[1]
x@2 = Literal["s"]
x@3 = phi(x@1, x@2)
use x@3
~~~

The phi key makes control-flow joins explicit. A visitor that stores only the last seen type depends on traversal order and loses branch alternatives.

Loops create recursive phi equations:

~~~text
x@entry = Int
x@loop = phi(x@entry, x@loop_body)
x@exit = x@loop
~~~

The binding builder constructs the equation; the solver computes the fixed point.

## Deferred bound names

Pyrefly defers some bound-name processing until after AST traversal because lookup may refer to a phi or forward reference whose final table entry is not populated yet.

The builder records:

~~~text
reserved bound-name key
lookup result key
usage context
promotion request
~~~

After traversal and phi population, it creates the final binding. This avoids committing an incorrect first-use or forward-reference decision while control flow is incomplete.

Phalcom needs a deferred-resolution list for:

- patterns whose target type depends on the complete pattern;
- closure captures whose flow version is created later;
- branch joins whose owner is created after both arms;
- module exports collected after declarations;
- class-side/instance-side information unavailable at first occurrence.

## First-use inference

Pyrefly supports a policy where partially typed values, such as an empty container, acquire an element type from first use and are then fixed.

Operationally:

~~~text
partial inference variable
  -> use creates a bound or pinning constraint
  -> later uses are checked against the pinned variable
  -> final boundary sanitizes/freezes the variable
~~~

This is not repeated union widening. It is a stateful inference policy with a defined pinning point.

Phalcom must choose explicitly among:

- annotation-driven inference;
- first-use inference for literals/collections;
- bidirectional expected-type inference;
- flow-only refinement;
- dynamic fallback.

The policy belongs in solver variable state and invalidation. A first-use pin can affect later expressions and callable summaries.

## Narrowing and branch facts

Pyrefly stores narrowing operations as semantic data associated with uses and flow. A branch may refine a value without changing its declared static type.

~~~text
declared x: Int | String
condition is_string(x)
then-flow x: String
else-flow x: Int
after join x: Int | String
~~~

A narrowing operation records:

- source expression/binding;
- predicate or test;
- positive/negative branch;
- relation used;
- resulting flow fact;
- provenance;
- dependency if the predicate relies on a callable or protocol fact.

Phalcom must keep this separate from TypeStore descriptors. Flow type is a program-point fact; declared TypeId is a contract.

## Expression binding behavior

An expression binding operation typically:

1. creates references to bindings required to evaluate the expression;
2. creates a key for the expression result or effect when a consumer needs it.

Examples:

~~~text
literal:
    exact/literal fact

name:
    use key pointing to current flow definition

send/call:
    receiver key, selector/member key, argument keys,
    callable/dispatch obligation

branch:
    predicate key and arm flow versions

collection:
    element/key/value obligations and partial-inference state

lambda/block:
    callable key, capture edges, parameter keys,
    control/effect metadata
~~~

The binding phase records enough structure for answer solving to run without re-walking the whole AST.

## Statement binding behavior

Statements create control edges and value ownership:

- assignment creates or updates a definition key;
- conditional creates arm scopes and a join;
- loop creates a back edge and a phi;
- return creates a result/effect edge;
- exception handling creates exceptional successors and narrowing;
- import creates module/export dependency keys;
- class/function definitions create declaration and callable surfaces;
- deletion or rebinding changes initialization and lookup state;
- yield records generator/yield keys used later by the solver.

The analysis must not model block construction as block execution. Captures, non-local returns, effects, and suspension require explicit semantic edges.

## Module exports

The Exports stage computes what a module provides before solving all body facts. It handles:

- named exports;
- implicit exports;
- re-exports;
- wildcard exports;
- transitive wildcard exports;
- submodule imports;
- export metadata;
- export origin;
- type aliases;
- class/member metadata needed by downstream relations.

The export product is narrower than the full Answers product. A dependent module that only needs to know whether a name exists should not pay to solve the whole source module.

Phalcom equivalent:

~~~rust
struct ModuleExports {
    names: IndexMap<Symbol, ExportTarget>,
    wildcard: WildcardExports,
    metadata: ExportMetadata,
    fingerprint: ExportFingerprint,
}
~~~

The fingerprint must distinguish existence, contract/type, metadata, and wildcard changes.

## Cross-module binding behavior

An import should produce a dependency edge to an export key, not clone the remote value into the importing module. The imported binding can ask for:

- existence;
- type/contract;
- metadata;
- re-export origin;
- class surface;
- type-alias target.

This enables asymmetric invalidation: documentation changes need not invalidate type-only consumers; type changes need not invalidate existence-only consumers when the name remains present.

## Class and callable behavior

Class and function definitions create several products:

~~~text
declaration identity
surface/signature
parameters
body bindings
return/exit facts
member/field facts
inheritance or dispatch facts
metadata
~~~

They should not collapse into one function-type node. A body-only edit can preserve the declaration surface while invalidating callable summaries. A superclass or member edit can invalidate consumers without changing the function body.

Phalcom must preserve this separation for:

- instance versus class-side declarations;
- selector identity;
- callable identity;
- fields and accessors;
- family dispatch;
- native implementations;
- closure captures;
- effects and non-local returns.

## Solver consumption

After binding construction, the solver evaluates keys without re-walking syntax:

~~~text
solve(key)
  -> inspect key kind/value
  -> solve referenced keys
  -> add/check constraints
  -> query exports/surfaces/dispatch through semantic boundary
  -> join/narrow/instantiate
  -> record answer and dependency set
~~~

The binding table is an executable semantic graph, not merely a diagnostic index.

## Phalcom mapping

Use current infrastructure as a bridge:

- DeclarationId, CallableId, FieldId, and ModuleId remain stable identity;
- source surfaces remain declaration-level products;
- ValueShape remains advisory runtime-shape evidence;
- formal TypeId facts remain separate;
- SemanticEngine owns construction and invalidation;
- SemanticSnapshot publishes immutable products;
- infer.rs callable summaries remain a bridge until formal answer queries exist;
- dispatch.rs answers lookup independently of subtype/equivalence;
- flow.rs owns branch/control facts, not canonical types.

## Non-transferable behavior

Do not copy:

- Python byte offsets as Phalcom binding identity;
- Python import execution assumptions;
- Python protocol and attribute semantics;
- Python first-use policies without a Phalcom typing decision;
- collapse of NotFound, uninitialized, dynamic, and unresolved states;
- flow shape as declared type;
- annotations changing selector or runtime dispatch identity.

## Implementation sequence

1. Define Phalcom binding key categories and owners.
2. Add a binding table with typed indexes and source-origin metadata.
3. Build explicit flow joins and recursive phi keys.
4. Add deferred name-resolution records for forward, branch, and capture cases.
5. Build module export products before body facts.
6. Add callable/member/capture/effect edges.
7. Make formal checker queries consume binding facts.
8. Record dependencies while solving each key.
9. Compare clean and incremental graphs.
10. Add LSP query adapters over the same semantic products.

## Verification gates

- control-flow join result is independent of AST traversal order;
- loop facts converge or report explicit budget status;
- definitions and uses resolve through semantic identity;
- body edits preserve unchanged surfaces;
- export-only consumers do not force full body solving;
- dynamic sends invalidate affected facts;
- closure construction is not treated as execution;
- cross-module imports retain source provenance;
- formal checker and LSP use the same binding IDs;
- clean full binding graph equals incremental binding graph for equal final source.

## Performance measurements

Record:

- binding keys created per module;
- bytes per key/value;
- table lookup time;
- percentage of AST nodes revisited during solving;
- number of phi keys;
- average/max flow versions per binding;
- deferred lookup count;
- export queries by dependency kind;
- cross-module answer cache hit rate;
- callable summary propagation count;
- flow join and narrowing counts.

## Conclusion

Pyrefly's semantic behavior is efficient because it performs expensive interpretation once during binding construction, stores compact references to the result, and lets the solver operate on an indexed equation graph. Phalcom should build the same kind of executable semantic graph, adapted to message sends, side-aware dispatch, closures, reflection, families, native contracts, and open-world behavior.
