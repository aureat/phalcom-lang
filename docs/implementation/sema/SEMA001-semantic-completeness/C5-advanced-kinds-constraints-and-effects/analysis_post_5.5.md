# Post–Step 5.5: what Phalcom should do next

The next phase should not simply be “continue the old checklist from Task 6 onward.”

Step 5.5 repaired the semantic database substrate: dependency-visible declaration metadata, pre-resolution reuse, demand-driven prerequisites, semantic-only body fingerprints, and a generic reuse law. Its purpose is explicitly to make the query engine trustworthy enough for later architectural work.  The parent architecture then expects Step 6 to turn declaration/hierarchy/dispatch/signature state into projections of DB-owned products, followed by compiler-owned module lifecycle, LSP integration, cold-vs-incremental parity, and stress testing.

After inspecting the current repository, however, I think there is a missing architectural stage between those two plans:

> **Phalcom now needs a semantic-completeness phase: make the formal analyzer closed over the language it claims to analyze.**

The incremental engine is becoming sophisticated faster than the semantic checker is becoming comprehensive. At the moment, it is possible to have a perfectly correct incremental dependency DAG that incrementally caches an `Unknown` produced because some AST construct was never analyzed.

That is the central problem to attack after 5.5.

One repository-grounding caveat first: the GitHub-visible `main` I can currently inspect still reflects the `06f6bcd...` generation and does not expose all of the Step 5.5 implementation described in your handoff—for example, the visible `SemanticDependency` still lacks `DeclarationShell`. So the first action for the next implementation agent must be to re-ground against the exact pushed HEAD/ref you have locally. I would treat your handoff as authoritative about work completed, but not let an agent assume its exact implementation shape without inspecting that commit.

---

## 1. The actual problem is now larger than incrementality

The architecture has made major progress. There is now a proper distinction between compiler-owned formal semantics and LSP advisory evidence; constructor `Self` specialization exists; callable analysis carries semantic dependencies; formal presentation identities exist; module query products are being pulled into snapshots; and query reuse is becoming dependency-correct.

But the semantic analyzer still has holes at a more basic level.

The most important example is `checker/expression.rs`: the expression checker still has a wildcard fallback that turns any expression not explicitly handled into:

```rust
UnknownReason::UncheckedExpression
```

rather than making the Rust compiler force us to account for every AST variant.

The statement checker has the equivalent problem:

```rust
_ => {}
```

after explicitly handling only `Let`, `Return`, expression statements, `Throw`, `Class`, and `For`. That means parsed statement forms such as type aliases and control-flow statements can simply pass through the formal analyzer without an explicit semantic decision.

That is dangerous for Phalcom in particular because of the checker philosophy you have now established:

> If the checker can prove something, its result is authoritative. If it cannot prove something, developer evidence may be accepted subject to consistency.

That philosophy is only sound if “cannot prove” means **genuinely insufficient semantic evidence**, not “the checker happened not to implement this AST variant.”

Those two states must never be conflated.

---

# 2. A concrete root-cause analysis of the current `CellNum` / Composition 1 problem

The current Composition 1 problem is a very useful miniature of the larger architecture problem.

The causal chain is approximately:

```text
fixture declares:
    of(raw: Int)

which means:
    selector = of(raw:)

call site uses:
    CellNum.of(42)

which means:
    selector = of(_)

therefore:
    dispatch cannot find the intended callable

therefore:
    expression formal type becomes Unknown

Unknown compared against declared Int:
    relation = Blocked / unknown
    NOT Refuted

therefore:
    checker correctly avoids inventing a mismatch

then:
    binding's effective downstream fact becomes declared Int

meanwhile LSP:
    sees a formal result
    therefore suppresses advisory ≈ CellNum
```

The relation layer deliberately maps unknown type knowledge to a blocked relation rather than pretending it has proof of incompatibility.  The policy layer correspondingly emits a mismatch only for a genuine `Refuted` result.

That part is correct.

The statement checker then only flips `is_assignable` to false when the relation is actually `Refuted`. Otherwise an annotated binding adopts the declared annotation as its effective fact.

Finally, current hover rendering says, effectively:

```text
if formal exists:
    show formal
else if observed exists:
    show observed
```

so a formal `Unknown` suppresses useful advisory evidence.

This explains why the tactical handoff fixes are sensible:

* change `of(raw: Int)` to `of(_ raw: Int)`;
* change `new(raw: Int)` to `new(_ raw: Int)`;
* allow hover to show advisory evidence alongside formal `Unknown`.

But I would make one important distinction:

> **The hover change is a recovery/UI fix. The selector correction is the actual semantic fix for that fixture.**

After the selector is corrected, the compiler should be able to prove:

```text
CellNum.of(42) : CellNum
```

and then prove:

```text
CellNum </: Int
```

so:

```text
const x: Int = CellNum.of(42)
```

must become a formal contradiction.

If, after correcting the selector, the compiler still says `Unknown`, that is not an LSP presentation problem. It is a checker integration bug.

---

# 3. There is a second problem hidden underneath the mismatch

Even once the checker successfully proves the mismatch, the binding model is currently too lossy.

`BindingState` contains:

```rust
declared: Option<TypeId>,
current: TypeKnowledge,
```

but no independent representation of:

* what the initializer was proven to be;
* whether the declaration and initializer agreed;
* the contradiction itself;
* the actual/expected types associated with that contradiction.

And `Statement::Let` currently does this after a mismatch:

```text
prove initializer = CellNum
prove declared = Int
emit mismatch
downstream effective binding type = Int
discard initializer denotation
```

The downstream use of `Int` is defensible as a cascade-suppression policy. We generally do not want one bad annotation to make every later use explode.

What is not ideal is losing the checker proof that the initializer was `CellNum`.

I would therefore evolve the binding semantic model toward something like:

```text
BindingState
    declared_constraint: Int
    observed/initializer: CellNum
    effective/current: Int
    consistency: Refuted
    diagnostic_cause: ...
```

Then the formal presentation can say:

```text
x
Declared type: Int
Initializer type: CellNum
Status: Invalid — CellNum is not assignable to Int
```

without asking the advisory LSP analyzer to rediscover `CellNum`.

That is much closer to the principle you described: the checker remains the authority whenever it actually has proof.

---

# 4. The most serious checker gap: semantic coverage is not exhaustive

This should become a hard gate.

## 4.1 Expressions must stop silently falling back to `Unknown`

An AST variant that the parser supports should fall into exactly one of these categories:

| Category                    | Required semantic behavior                                   |
| --------------------------- | ------------------------------------------------------------ |
| Fully statically supported  | Produce formal type/evidence/denotation                      |
| Desugared syntax            | Analyze its canonical semantic lowering                      |
| Explicit dynamic boundary   | Produce `Dynamic` with a precise `DynamicReason`             |
| Compile-time-only construct | Produce its declaration/query product                        |
| Temporarily unsupported     | Produce an explicit diagnostic or explicit unsupported state |
| Internal-only AST form      | Verify it cannot originate from ordinary source              |

There should be no seventh category:

```text
“we forgot to implement it, so wildcard => Unknown”
```

The current expression fallback explicitly permits that.

The first new checker task should therefore be an AST coverage census covering every variant of:

* `Statement`;
* `Expr`;
* `Pattern`;
* `ClassMember`;
* `TypeAnnotationExpr`;
* call `PackItem`;
* selector/dispatch form.

Then remove wildcard semantic matches.

Rust should help enforce the architecture: adding a new AST variant should cause semantic compilation failures until the analyzer makes an explicit decision about it.

---

## 4.2 Patterns are a confirmed semantic hole

The AST already supports substantially richer patterns than simple names, including tuple, list, variant, record, and map patterns.

But the expression-side pattern binder currently only does this:

```rust
if let Pattern::Name { ... } = pattern {
    ...
}
```

The statement checker likewise binds only `Pattern::Name` for ordinary bindings and `for` lanes.

So syntax such as conceptually:

```phalcom
const (x, y) = (1, "hello")
```

may be perfectly parsable and compiler-lowerable while the formal semantic layer does not produce two independent bindings with:

```text
x : Int
y : String
```

This matters far beyond inlay hints. It affects:

* type inference;
* assignment consistency;
* shadowing;
* flow state;
* references;
* rename/navigation;
* exact binding identity;
* incremental invalidation;
* LSP presentation.

Pattern semantics need a recursive formal binder, not special cases scattered through `Let`, `for`, `if let`, and block parameters.

---

## 4.3 Type aliases appear particularly important

The runtime compiler explicitly recognizes `Statement::TypeAlias` as a compile-time declaration and emits no runtime opcode for it.

But the current semantic statement checker does not explicitly process it.

That is a major gap because aliases are not merely syntax decoration. They participate in:

```text
type resolution
kind checking
generic application
subtyping/assignability
diagnostic presentation
dependency invalidation
cross-module exports
hover/go-to-definition
```

More importantly for Step 5.5, aliases need dependency-visible semantic identity.

If:

```phalcom
type Id = Int
```

changes to:

```phalcom
type Id = String
```

then every unchanged consumer whose type annotation resolved through `Id` must become non-reusable.

I would not create a completely separate incremental architecture for aliases. Instead, I would generalize the idea introduced by `DeclarationShell`:

```text
DeclarationShell
    Nominal(...)
    Alias(...)
    ...
```

The key insight is:

> `DeclarationSurface` is specifically member-bearing nominal structure.
> `DeclarationShell` should eventually be the dependency-visible identity of every type-side declaration.

That gives alias consumers the same dependency semantics as class/generic consumers.

---

# 5. Generic inference is not finished enough for the language you are designing

There are several deeper gaps in the generic call solver.

The repository already contains a sophisticated `InferenceSession` with solver-local variables, compound terms, lower/upper bounds, subtype constraints, and generic signatures.

But some of the plumbing into it is still provisional.

## 5.1 Generic parameter kinds are currently flattened

`GenericSignature` carries real parameter identities and constraints.

Yet `instantiate_generic_signature()` currently creates every fresh inference variable with:

```rust
KindId::TYPE
```

rather than the actual kind of the corresponding type parameter.

That is fine for:

```text
<T: Type>
```

but not for Phalcom's planned:

```text
<F: Type -> Type>
```

or other higher-kinded generics.

This should be corrected before HKT-heavy standard libraries are built on top of the checker.

---

## 5.2 Generic constraints exist in the model but are not being fully fed into call inference

The semantic model has:

```rust
GenericSignature {
    parameters,
    constraints,
}
```

and supports subtype/equivalence constraints.

The current generic call resolution instantiates parameters and adds argument/result constraints, but the inspected path does not add the generic signature's own constraint set into the inference session before solving.

That means this sort of method can carry a formal constraint in metadata without the invocation solver necessarily proving it at the call site.

That is precisely the kind of “the model exists but the checking path does not consume it” problem your new tests should be designed to find.

---

## 5.3 Unknown parameter terms must not become `Unit`

There is also provisional logic in generic call inference where a parameter that does not expose a canonical `TypeId` is represented as:

```rust
InferenceTerm::Canonical(ctx.store.unit())
```

That is semantically dangerous.

`Unknown` does not mean `Unit`.

`Self` does not mean `Unit`.

“Not yet materialized” does not mean `Unit`.

Those states must remain distinguishable all the way through inference.

The same principle applies to the current general `TypeTerm::SelfType` handling inside the inference term conversion path, which currently uses a placeholder-like treatment.

Before advanced generics are considered reliable, all placeholder substitutions of this class should disappear.

---

# 6. Call checking needs a separate call-shape correctness layer

The current `resolve_call()` does useful bidirectional propagation and parameter checking.

But call resolution should be decomposed conceptually into:

```text
1. resolve selector
2. validate argument-pack shape
3. derive parameter ↔ argument mapping
4. instantiate generics
5. collect type constraints
6. solve
7. specialize result
8. enforce consistency
9. record evidence + dependency
```

The “argument-pack shape” step needs explicit coverage for:

* too many positional arguments;
* missing required positional argument;
* unknown labeled argument;
* duplicate label;
* missing required labeled argument;
* positional/labeled selector mismatch;
* rest arguments;
* expanded packs;
* dynamic labels if supported;
* constructor selectors;
* getter/setter/index selectors.

This would have caught the Composition 1 fixture immediately.

It should be impossible for a selector-shape mistake to degrade into some distant mysterious `Unknown` without an owning call-resolution status.

---

# 7. Iteration typing currently contains a heuristic that should be removed

The `for` protocol implementation first tries formal dispatch through `iteratorValue(_)` / `iteratorValue`, which is good.

But it then contains a fallback for any applied generic nominal type that returns:

```text
arguments[0]
```

as the element type if the origin has generic parameters.

That is too broad.

For:

```text
Map<K, V>
```

what is “the first type argument” supposed to mean for iteration?

For a custom:

```text
Pipeline<Input, Output, Error>
```

it is even less meaningful.

The checker should infer iterable element type because the type proves participation in the iteration protocol, not because it happens to be generic.

The protocol signature should be the evidence.

---

# 8. There is also a value-namespace hole to investigate

The runtime compiler has explicit module/global binding semantics. It tracks module-level `let`/`const`, classes, known globals, imported bindings, and linked global bindings.

The formal expression checker, by contrast, primarily resolves a bare identifier as:

```text
local binding
otherwise type name
otherwise unknown
```

The post-5.5 investigation therefore needs a dedicated audit of:

```text
module-level const/let
imported values
exported values
class names
type names
value/type namespace collisions
method-local lookup
```

If a class method does:

```phalcom
const DEFAULT_LIMIT = 50

class Worker {
  run() -> Int {
    DEFAULT_LIMIT
  }
}
```

the formal checker must have a canonical value-semantic product to consume.

Eventually I would expect something analogous to:

```text
GlobalBinding / ModuleValueSurface
```

rather than letting body checking query ad-hoc source state.

That gives incrementality a proper edge too:

```text
CallableBody(Worker.run)
    -> GlobalBinding(DEFAULT_LIMIT)
```

and allows:

```text
DEFAULT_LIMIT = 50
```

changing to a string to invalidate exactly its consumers.

---

# 9. Step 6 should still happen first — but it needs one addition

I would finish the existing Step 6 before doing the large semantic-coverage expansion.

The architectural specification is correct that:

```text
DeclarationTypeTable
MapTypeHierarchy
SurfaceDispatchResolver
CallableSignatureTable
```

should become publication-time projections of Ready DB products rather than parallel semantic authorities.

The visible session is already moving that way: it describes compatibility dispatch/signature tables as materializations from DB-owned formal products.

But I would add one more projection to Step 6:

> **`SemanticPresentationIndex` must become a first-class snapshot projection.**

The compiler already has the right model. `SemanticPresentationIndex` contains callable-qualified expression identities, callable-qualified binding identities, an exact binding site index, and most-specific expression lookup.

Yet the published `SemanticSnapshot` currently retains sources, surfaces, dispatch, signatures, declarations, hierarchy, diagnostics, callable analyses, and module products—but not the presentation index.

That leads directly to the next LSP problem.

---

# 10. The LSP already has the formal identity machinery but does not use it

Current LSP formal binding lookup still iterates callable analyses and searches by:

```text
binding name
+
source range
```

with this particularly permissive condition:

```rust
state.range.contains(offset)
|| state.range.end == offset
|| state.range.start <= offset
```

That final clause effectively says:

> any binding that started earlier may qualify.

It is exactly the sort of heuristic that produces wrong answers for:

* sibling bindings;
* shadowed locals;
* parameters shadowed by locals;
* destructuring;
* multiple sites with the same spelling.

And it is unnecessary, because the compiler already has `FormalSiteId::Binding { callable, binding }`.

Likewise, LSP callable presentation currently maps:

```text
SelfType -> Unknown
```

even though constructor `Self` specialization exists in the compiler.

So the post-5.5 rule should be:

> The LSP may format semantic results. It must stop rediscovering semantic identity.

---

# 11. Recommended execution order

I would revise the implementation sequence to this:

```text
5.5  Incrementality hardening
 |
 v
6.0  Finish DB-owned snapshot projections
 |
 v
6.5  Semantic coverage closure
 |
 v
6.6  Call + generic inference correctness
 |
 v
6.7  Evidence/contradiction model
 |
 v
6.8  Core/native semantic conformance
 |
 v
7.0  Persistent compiler-owned module/source lifecycle
 |
 v
7.5  Cross-module semantic closure
 |
 v
8.x  LSP formal-authority integration
 |
 v
16–21 Incremental equivalence / navigation / boundaries / stress / performance
```

That is the sequence I would give the next agent.

---

# 12. Gate 0 — verify the actual Step 5.5 implementation

Before touching anything else, re-ground against the actual pushed commit.

The agent should verify that the completed implementation—not merely the spec—has all of these properties:

* `DeclarationShell` is a typed DB product;
* declaration metadata reads record `DeclarationShell`;
* declaration surface reuse checks occur before semantic annotation resolution;
* surface recomputation records exact semantic dependencies;
* callable body requests its signature prerequisite instead of relying on prewarming;
* callable-body product fingerprint excludes source/presentation-only movement;
* `SemanticDb::is_reusable()` has no query-kind exception;
* all Step 5.5 regressions pass.

Those are the exact invariants Step 5.5 was written to establish.

If any are incomplete, finish those before adding semantic capabilities.

---

# 13. Gate 1 — finish Composition 1, but strengthen what it proves

Do the tactical fixes from the handoff now.

But modify the Composition 1 test so it does not merely test “something appears in hover.”

For the fixture:

```phalcom
class CellNum {
  @constructor
  new(_ raw: Int) { ... }

  @class
  of(_ raw: Int) -> CellNum { ... }
}

const wrong: Int = CellNum.of(42)
```

the compiler-layer assertions should prove all of these before LSP is tested:

| Semantic fact             | Expected                        |
| ------------------------- | ------------------------------- |
| `CellNum.of(42)` dispatch | exact canonical callable        |
| call result               | `CellNum`                       |
| call status               | `Ready`                         |
| call evidence             | `Proven` / declaration-derived  |
| annotation                | `Int`                           |
| assignability             | `Refuted`                       |
| diagnostic                | `BindingInitializerMismatch`    |
| actual type               | `CellNum`                       |
| expected type             | `Int`                           |
| callable dependency       | `CellNum.of(_)` signature       |
| declaration dependencies  | appropriate shell/surface edges |

Then test the LSP projection of that same result.

This distinction is crucial:

```text
compiler test proves language truth
LSP test proves language truth is projected correctly
```

Do not make an LSP test the first place where a type-checker property is validated.

---

# 14. Gate 2 — complete original Step 6

Finish the DB-as-authority transition.

The end-state should be:

```text
Ready DB products
    |
    +--> declarations projection
    +--> hierarchy projection
    +--> dispatch projection
    +--> signature projection
    +--> callable analyses projection
    +--> diagnostics projection
    +--> presentation index projection
    |
    v
SemanticSnapshot
```

The session should not independently “re-decide” any of these semantic facts during publication.

Step 6's acceptance test should include a body-only edit:

```text
method body changes
```

and prove that:

```text
declaration shell       reusable
declaration surface     reusable
hierarchy edge          reusable
callable signature      reusable
unrelated bodies        reusable
edited body             recomputed
presentation for unaffected sites structurally reusable/identical
```

This is the correct point at which the existing incremental architecture becomes a reliable substrate for expanding semantics.

---

# 15. Gate 3 / Step 6.5 — Semantic Coverage Closure

This should be a new explicit implementation phase.

## 15.1 Build an executable semantic coverage matrix

Do not start by coding individual missing constructs.

First enumerate every AST construct and classify it.

A useful matrix is:

| Language construct | Parsed | Formal synthesis | Expected-type checking | Formal status | Diagnostic negative case | Dependency tracking | Presentation | Incremental test |
| ------------------ | -----: | ---------------: | ---------------------: | ------------: | -----------------------: | ------------------: | -----------: | ---------------: |
| integer literal    |      ✓ |                ✓ |                      ✓ |             ✓ |                      n/a |                 n/a |            ? |                ? |
| variable           |      ✓ |                ✓ |                      ✓ |             ✓ |                        ✓ |                   ? |            ? |                ? |
| assignment         |      ✓ |                ✓ |                      ✓ |             ✓ |                        ✓ |                   ? |            ? |                ? |
| method send        |      ✓ |                ✓ |                      ✓ |             ✓ |                        ✓ |                   ✓ |            ? |                ? |
| class send         |      ✓ |                ✓ |                      ✓ |             ✓ |                        ✓ |                   ✓ |            ? |                ? |
| `super` send       |      ✓ |                ? |                      ? |             ? |                        ? |                   ? |            ? |                ? |
| method reference   |      ✓ |                ? |                      ? |             ? |                        ? |                   ? |            ? |                ? |
| tuple pattern      |      ✓ |                ✗ |                      ✗ |             ✗ |                        ✗ |                   ✗ |            ✗ |                ✗ |
| record pattern     |      ✓ |                ✗ |                      ✗ |             ✗ |                        ✗ |                   ✗ |            ✗ |                ✗ |
| type alias         |      ✓ |        ✗/partial |                      ✗ |             ✗ |                        ✗ |                   ✗ |      partial |                ✗ |
| type lambda        |      ✓ |          partial |                partial |             ? |                        ? |                   ? |            ? |                ✗ |
| explicit `Self`    |      ✓ |          partial |                partial |             ? |                        ? |                   ? |            ? |                ? |
| etc.               |        |                  |                        |               |                          |                     |              |                  |

That table should become a living test inventory rather than a design document that drifts.

---

## 15.2 Remove semantic wildcard fallbacks

After the census:

```rust
match expr {
    ...
    _ => Unknown(UncheckedExpression)
}
```

must disappear from supported-source semantic entry points.

Likewise:

```rust
match statement {
    ...
    _ => {}
}
```

must disappear.

For constructs that genuinely do nothing in the type checker, write the arm explicitly:

```rust
Statement::Export(export) => {
    // interface query owns semantic exposure; no body value effect
}
```

That is still much better than a wildcard because a future `Statement::Match` will then fail to compile until someone makes a semantic decision.

---

# 16. Gate 3A — recursive pattern semantics

Implement one central recursive pattern-checking/binding abstraction.

Conceptually:

```text
check_pattern(pattern, value_fact, expected_pattern_type)
```

It should handle:

```text
Name
Tuple
List
Variant
Record
Map
```

and be reused by:

```text
let/const
for lanes
if let
while let
block parameters
future match
```

The product must preserve exact identifier token ranges, not whole declaration ranges.

Required tests should include:

```phalcom
const (x, y) = (1, "s")
```

with exact assertions:

```text
x : Int
y : String
x BindingId != y BindingId
x range = exact x token
y range = exact y token
```

Then add nested and negative cases:

```phalcom
const ((x, y), z) = ((1, "s"), true)
```

and incompatible arity/shape tests according to Phalcom's pattern semantics.

---

# 17. Gate 3B — declaration and type-form completeness

This deserves its own slice.

Cover:

* aliases;
* generic aliases;
* nominal references;
* qualified references;
* applied types;
* tuple types;
* callable types;
* union types;
* record rows;
* `Self`;
* type lambdas;
* higher-kinded bounds;
* generic constraints;
* declaration-level `where`;
* method-level `where`.

Type annotation lowering is already broad—it handles nominal references, application, tuple, records, callables, unions, `Self`, and type lambdas. But some of those paths need semantic-context auditing rather than mere parser coverage.

For example, explicit source `Self` depends on knowing the current declaration context, whereas linked production resolution must actually provide that context. Type lambdas also need genuine scoped type-parameter binding rather than merely constructing an arrow kind around a body that was resolved without those names.

The test rule should be:

> Every type syntax feature gets at least one kind test, one resolution test, one valid use-site test, and one invalid use-site test.

---

# 18. Gate 3C — module/global value semantics

Introduce a formal product for module-level value bindings.

I would separate value declarations from callable bodies in the same way the architecture separates declaration signatures from implementation bodies.

Conceptually:

```text
GlobalBindingSurface
    identity
    mutability
    declared constraint
    inferred/observed initializer type
    effective type
    export visibility
    source site
```

Then:

```text
CallableBody
    -> GlobalBindingSurface
```

when a callable reads a global.

For exported globals:

```text
LinkedInterface / export identity
    -> GlobalBindingSurface
```

as appropriate.

This gives Phalcom proper formal handling of what its bytecode compiler already treats as a real module namespace.

---

# 19. Gate 4 / Step 6.6 — call and inference hardening

Once syntax coverage is explicit, harden the reasoning engine.

## Call-shape tests

Create separate tests for:

```text
positional exact match
labeled exact match
mixed positional/labeled
wrong label
missing label
duplicate label
missing positional
extra positional
rest parameter
expanded argument
wrong selector side
class vs instance
getter
setter
index get
index set
constructor
super send
```

Every failing call needs a specific call-resolution result and diagnostic.

## Generic inference tests

At minimum:

```phalcom
@class
identity<T>(_ value: T) -> T {
  value
}
```

must prove:

```text
identity(1) : Int
identity("x") : String
```

Then test bidirectional inference:

```phalcom
const x: Int = Generic.make()
```

if the language permits expected-result inference.

Then constraints:

```text
<T where T <: Number>
```

positive and negative.

Then HKT:

```text
<F: Type -> Type>
```

with the solver actually respecting the parameter's kind rather than allocating every inference variable as `Type`.

---

# 20. Gate 5 / Step 6.7 — formal evidence and contradiction modeling

This is where I would fully ratify your “checker is authoritative” model in data structures.

I would evolve `BindingState` from:

```text
declared
current
```

to something semantically richer:

```text
declared_constraint
observed
effective
consistency
```

where consistency is something like:

```text
Proven
Refuted
Unknown
DynamicBoundary
Blocked
Cancelled
BudgetExceeded
InternalFailure
```

This parallels the already richer expression status model.

The key distinction is:

```text
observed = what the checker proved about the value
effective = what downstream analysis should assume after applying annotation/cascade policy
```

For the mismatch example:

```text
declared = Int
observed = CellNum
effective = Int
consistency = Refuted(CellNum !<: Int)
```

That is significantly better than either:

```text
binding = Int
```

or:

```text
binding = Unknown
```

Both throw away information.

---

# 21. Gate 6 / Step 6.8 — source/native/semantic conformance

The core/native implementation needs a stronger conformance test.

The existing `core_surface_conformance` test is useful, but the validator mainly verifies that the native surface catalog is internally well-formed and that its type forms can be resolved.

It does not comprehensively prove that:

```text
universe source declaration
native metadata
semantic surface
runtime behavior
```

agree.

A current example illustrates why this matters.

Universe source says:

```phalcom
@native /(_ other: Number) -> Number
```

whereas native primitive metadata declares:

```text
(Number) -> Float
```

This is not necessarily wrong: `Float` may deliberately be an implementation refinement of a public `Number` contract.

Likewise an abstract constructor may have a public nominal signature while its concrete native implementation is `Never` because it always raises.

But the repository needs an explicit rule for that.

I would make the conformance law:

```text
source surface = canonical public contract

native implementation:
    accepts at least the public parameter domain
    returns a subtype/refinement of public return
    does not weaken declared effects/raises/flow constraints
```

If instead native metadata is intended to be the public semantic authority, then source must exactly match it.

Pick one authority. Do not permit the checker and LSP to choose whichever table they happened to read.

---

# 22. Gate 7 — then continue the original compiler-owned module lifecycle work

Only after the single-module checker is semantically closed would I continue with the large module-lifecycle transfer.

The parent specification's direction remains correct:

```text
SemanticWorkspaceSession
    owns ProjectUniverse lifetime
    owns overlay provider
    owns semantic revision
    invokes phalcom-modules
    owns SemanticDb
```

while:

```text
phalcom-modules
    remains sole authority for module semantics
```

and:

```text
LSP
    sends source events
    consumes immutable snapshots
```



Parts of this are already present: module products exist in the semantic snapshot, and `module_queries()` is exposed as a pure facade.

Finish:

```text
set_workspace_roots
resolve_document_path
apply_document_change
apply_source_change
commit_revision
```

and make the old cold `update(SemanticWorkspaceInput)` path a compatibility wrapper over those primitives, not a second architecture.

---

# 23. Gate 7.5 — cross-module semantic completeness

Do not consider Step 7 done merely because imports link.

Now repeat the semantic-coverage tests across module boundaries.

For example:

```text
module A:
    type Id = Int

module B:
    import A
    const x: A.Id = 1
```

Then mutate:

```text
A.Id = String
```

and prove:

```text
DeclarationShell(A.Id) changes
B's exact consumer invalidates
B gets mismatch
unrelated module C reuses
incremental == cold
```

Do the same for:

* exported class signatures;
* generic constraints;
* superclass changes;
* exported globals;
* fields;
* native/core surfaces;
* constructors;
* aliases;
* package exposure.

This is where the new semantic model and the Step 5.5 dependency engine meet.

---

# 24. Gate 8 — make LSP a pure formal projection

At this stage, fix the LSP comprehensively rather than adding more heuristics.

## Formal site lookup

Publish the compiler's `SemanticPresentationIndex` in `SemanticSnapshot`.

Then LSP binding lookup becomes:

```text
URI
 -> canonical ModuleId
 -> exact source site
 -> FormalSiteId::Binding
 -> BindingId
 -> compiler BindingState
```

No:

```text
name scan
first matching binding
state.range.start <= offset
```

The compiler already has most of this index structure.

## Expression lookup

Use the compiler's most-specific-containing-expression query.

Do not iterate arbitrary map order.

## Callable presentation

A declaration signature containing `Self` can display:

```text
-> Self
```

At a specialized call site:

```phalcom
Derived.new()
```

the expression presentation should show:

```text
Derived
```

The existing semantic regression already proves `Base.new() -> Base`, `Derived.new() -> Derived`, and `Derived.ordinary() -> Base`, which is exactly the right distinction.

But the LSP currently still maps callable `SelfType` directly to `Unknown`.

Remove that disconnect.

---

# 25. Define one formal/advisory composition matrix

The LSP should not make this decision independently in hover, inlays, completion, and signature help.

Ratify one matrix:

| Formal state      | Advisory evidence                                                                                            |
| ----------------- | ------------------------------------------------------------------------------------------------------------ |
| `Known(T)`        | Formal wins. Do not replace with advisory.                                                                   |
| `Invalid`         | Show formal contradiction/evidence. Advisory may be separately labeled only if useful.                       |
| `Unknown`         | Show `Formal: Unknown`; may additionally show `Observed: ≈ T` with confidence.                               |
| `Dynamic`         | Show `Dynamic`; do not pretend advisory shape is formal type.                                                |
| `Blocked`         | Show blocked state; advisory must not replace it.                                                            |
| `Cancelled`       | Preserve/cite last published formal state as appropriate; never substitute advisory as current formal truth. |
| `BudgetExceeded`  | Same principle.                                                                                              |
| `InternalFailure` | Never mask with advisory inference.                                                                          |
| `Partial`         | Show only the site-specific formal facts that are actually ready.                                            |

This is compatible with the parent architecture's explicit requirement that advisory evidence must never replace formal `Unknown`, `Dynamic`, `Invalid`, `Blocked`, cancellation, budget, or failure states.

The tactical “Unknown + observed shape” hover change therefore fits the architecture, as long as the observed shape remains visibly advisory.

---

# 26. Current testing is too weak for this architecture

This is probably the biggest process change I would make.

The current basic semantic integration suite mostly checks:

```text
no errors
```

or a single diagnostic code.

And `compiler_capabilities.rs` currently contains one real literal synthesis assertion plus a test named `just_testing` that merely prints the result.

There are broader Phase 2 expression tests, which are useful, but many positive cases still succeed merely by asserting there are no diagnostics.

That is insufficient because the type system intentionally treats `Unknown` as non-refuting.

Therefore this test can falsely pass:

```text
expected Int
checker forgot to analyze expression
actual = Unknown
Unknown vs Int = Blocked
no mismatch emitted
test asserts "no errors"
PASS
```

This is exactly what we must eliminate.

---

# 27. New rule: positive semantic tests must prove knowledge, not absence of errors

A successful capability test should assert at least:

```text
TypeKnowledge
AnalysisStatus
EvidenceAuthority
semantic denotation if relevant
resolved callable if relevant
dependency edge if relevant
```

For example, arithmetic should not be:

```rust
assert!(!report.has_errors());
```

It should prove:

```text
expression `1 + 2`
    status = Ready
    type = Int
    authority = Proven
    call = Number.+(_)
    hierarchy edge = Int -> Number if traversed
    signature dependency = Number.+(_)
```

Likewise a negative test should prove:

```text
actual type
expected type
relation = Refuted
diagnostic code
diagnostic range
formal site status = Invalid
```

And an intentional unknown test should prove:

```text
UnknownReason == specific expected reason
```

not merely accept any unknown.

---

# 28. Four layers of testing for every important capability

I would standardize the testing surface into four levels.

### Layer A — semantic primitive/unit tests

Test:

```text
type relation
generic solver
substitution
Self specialization
kind checking
argument pack matching
pattern decomposition
```

No workspace orchestration.

### Layer B — compiler/session integration tests

Use actual:

```text
SemanticWorkspaceSession
SemanticDb
query products
real declaration surfaces
real dispatch
real callable analyses
```

Assert final semantic answers and dependency edges.

This should become the primary place to test language truth.

### Layer C — LSP composition tests

Take one published compiler snapshot and prove consistency across:

```text
hover
inlay
signature help
diagnostics
go to definition
references
completion
```

The test should verify that all features refer to the same canonical semantic identity.

### Layer D — persistent incremental LSP tests

One actual server/session:

```text
open
change
change
change
close/reopen
```

without restart.

Compare every revision to a fresh cold analysis.

---

# 29. A concrete test backlog I would implement

Here is the first serious semantic-capability suite I would build after the infrastructure gates.

|  # | Capability                  | Core proof                                       |
| -: | --------------------------- | ------------------------------------------------ |
|  1 | Constructor `Self`          | inherited constructor specializes to receiver    |
|  2 | Annotation contradiction    | `CellNum` cannot satisfy `Int`                   |
|  3 | Ordinary nominal return     | inherited explicit `-> Base` remains `Base`      |
|  4 | Tuple destructuring         | `(Int, String)` produces separate exact bindings |
|  5 | Nested destructuring        | recursive pattern facts preserved                |
|  6 | Record destructuring        | each field gets correct formal type              |
|  7 | `for` destructuring         | loop element fact decomposes through pattern     |
|  8 | Module global               | callable resolves typed module `const`           |
|  9 | Type alias                  | alias transparently resolves in annotation       |
| 10 | Generic alias               | `Alias<T>` specializes correctly                 |
| 11 | Alias invalidation          | alias target edit invalidates consumer           |
| 12 | Explicit `Self`             | source-written `Self` resolves in method context |
| 13 | Type lambda                 | parameter is actually scoped and kind-correct    |
| 14 | HKT parameter               | `<F: Type -> Type>` preserves arrow kind         |
| 15 | Generic identity            | argument inference produces exact return         |
| 16 | Generic constraint          | violated bound is rejected                       |
| 17 | Expected-result inference   | context constrains generic result                |
| 18 | Wrong call label            | exact selector/argument-shape diagnostic         |
| 19 | Missing argument            | explicit arity diagnostic                        |
| 20 | Extra argument              | explicit arity diagnostic                        |
| 21 | Rest/expand                 | pack shape and element types verified            |
| 22 | Getter/setter               | proper read/write types                          |
| 23 | Index get/set               | key/value/result semantics verified              |
| 24 | Custom iterable             | element comes from protocol evidence             |
| 25 | Generic iterable            | no arbitrary `arguments[0]` fallback             |
| 26 | Primitive arithmetic        | `Int + Int -> Int`                               |
| 27 | Mixed numeric operation     | semantics match ratified numeric contract        |
| 28 | Division                    | public/native/runtime contract agrees            |
| 29 | Callee body-only edit       | caller reuses                                    |
| 30 | Callee signature edit       | unchanged caller recomputes                      |
| 31 | Field type edit             | unchanged reader recomputes                      |
| 32 | Superclass edit             | inherited call consumer recomputes               |
| 33 | Cross-module alias edit     | exact remote consumer recomputes                 |
| 34 | Range-only edit             | semantic dependent reuses                        |
| 35 | LSP mismatch lifecycle      | error appears/clears without restart             |
| 36 | Shadowed binding            | hover/inlay identifies correct `BindingId`       |
| 37 | Constructor LSP composition | hover/inlay/signature help all agree             |
| 38 | Formal Unknown composition  | advisory appears only as observed evidence       |
| 39 | Dynamic boundary            | advisory never masquerades as formal known       |
| 40 | Incremental/cold parity     | every edit class produces same semantics         |

I would expand from these rather than continue accumulating isolated bug reproductions.

---

# 30. Add a marker-driven semantic test harness

The current tests will become cumbersome once you need to inspect exact expressions and bindings.

I would add a fixture API that allows tests to write something conceptually like:

```text
/*@call*/ CellNum.of(42)
```

and then query the exact compiler semantic site by marker.

The test API should be able to say:

```text
expression("@call")
binding("@x")
callable("Client.test")
diagnostics_at("@call")
dependencies_of("Client.test")
```

with assertions such as:

```text
assert_known("CellNum")
assert_invalid()
assert_declared("Int")
assert_observed("CellNum")
assert_calls("CellNum.of(_)")
assert_depends_on_signature(...)
```

That is vastly safer than tests doing:

```rust
analysis.bindings.values().find(|b| b.name == "x")
```

The existing constructor regression still does name-based lookup inside the test.  That's adequate for one isolated fixture, but it will not scale to shadowing/destructuring/composition testing.

---

# 31. Make dependency assertions part of semantic tests

You specifically said you want not merely the result, but the path/evidence used to get there.

I agree.

Not a complete internal proof trace, but enough observable evidence to catch bad reasoning.

For:

```phalcom
const x = Derived.new()
```

a semantic test should be able to assert something like:

```text
result:
    Derived

evidence:
    call resolved to Base.new(...)
    constructor return term = Self
    receiver declaration = Derived
    Self specialized to Derived

dependencies:
    CallableSignature(Base.new)
    DeclarationShell(Base)
    DeclarationShell(Derived)
    HierarchyEdge(Derived)
```

Likewise:

```phalcom
const x: Int = CellNum.of(42)
```

should retain:

```text
initializer proof:
    CellNum

declared constraint:
    Int

relation:
    Refuted

diagnostic:
    BindingInitializerMismatch
```

That gives you precisely the intermediate visibility you described without exposing every internal solver operation.

---

# 32. What I would not do next

I would specifically avoid these directions right now:

1. **Do not immediately add lots of new LSP intelligence.** If the checker is incomplete, the LSP will accumulate more compensating inference.

2. **Do not fix every `Unknown` by teaching hover to use advisory types.** Some unknowns are legitimate; others are checker holes. The formal layer must tell us which.

3. **Do not jump straight from Step 6 to performance tuning.** A fast cache of incomplete semantics is not useful.

4. **Do not make “test passes” mean “no diagnostic.”** This is actively unsafe under Phalcom's epistemic relation model.

5. **Do not add another semantic authority for aliases, globals, primitives, or module values.** Extend the query/product vocabulary.

6. **Do not preserve wildcard checker matches for forward compatibility.** In a compiler semantic layer, exhaustiveness is a feature.

7. **Do not make LSP binding identity depend on source spelling.** The compiler already has `BindingId`.

8. **Do not make native metadata and universe source two competing public signatures.** Ratify their compatibility relationship.

---

# 33. Revised mapping of the old remaining tasks

I would reinterpret the old plan like this:

| Existing work     | Revised meaning                                                           |
| ----------------- | ------------------------------------------------------------------------- |
| Composition 1 fix | Immediate tactical gate; strengthen compiler assertions                   |
| Task 6            | Finish DB-owned projections + add presentation-index projection           |
| **NEW 6.5**       | Semantic AST coverage closure                                             |
| **NEW 6.6**       | Call/generic inference hardening                                          |
| **NEW 6.7**       | Formal evidence/contradiction model                                       |
| **NEW 6.8**       | Core/native/source semantic conformance                                   |
| Task 7            | Persistent compiler-owned module lifecycle                                |
| Tasks 8–15        | LSP consumes canonical compiler products; no parallel meaning             |
| Task 16           | persistent no-restart semantic equivalence                                |
| Task 17           | body-vs-signature invalidation isolation                                  |
| Task 18           | cross-module transitive invalidation expanded to aliases/globals/generics |
| Task 19           | canonical occurrences/navigation                                          |
| Task 20           | project/exposure boundaries                                               |
| Task 21           | lifecycle stress, performance, full cold parity                           |

I would not delete Tasks 16–21. They are still very good acceptance gates. They are simply too late to discover fundamental checker-coverage holes.

---

# 34. Recommended commit sequence

I would keep this highly reviewable:

```text
test(semantic): verify step-5.5 hardening invariants

fix(lsp): correct composition selector fixture and unknown hover composition

refactor(semantic): finish db-owned snapshot projections

feat(semantic): publish formal presentation index in semantic snapshots

test(semantic): add exhaustive semantic capability matrix

refactor(semantic): make statement and expression checking exhaustive

feat(semantic): implement recursive pattern typing and binding

feat(semantic): integrate aliases and remaining type forms

feat(semantic): add formal module value binding semantics

fix(semantic): validate callable shape before type inference

fix(semantic): enforce generic constraints and parameter kinds

refactor(semantic): preserve observed and declared binding evidence

test(core): enforce source-native-semantic contract compatibility

feat(semantic): complete persistent module lifecycle

refactor(lsp): consume compiler formal-site identity exclusively

test(lsp): add semantic composition matrix

test(semantic): enforce incremental-cold equivalence

test(lsp): enforce no-restart incremental equivalence

perf(semantic): add lifecycle and invalidation stress gates
```

That sequence gives every major architectural change its own regression boundary.

---

# 35. The exit condition before calling the semantic analyzer “complete”

I would establish a very hard definition of done.

For every language construct accepted by the parser, the formal semantic system must be able to answer:

```text
What semantic thing is this?
What type/kind information do we know?
How do we know it?
What formal state is that knowledge in?
What declarations/signatures/hierarchy/module facts were consumed?
If developer evidence exists, is it consistent with the proven facts?
What invalidates this answer?
Where is this exact semantic site in source?
```

The answer may legitimately be:

```text
Unknown
```

But then it must also answer:

```text
Why is it unknown?
```

For example:

```text
Unknown(UnannotatedDynamicBoundary)
Unknown(UnresolvedImport)
Unknown(UnderconstrainedTypeVariable)
Unknown(UnsupportedReflection)
```

It should not be:

```text
Unknown(UncheckedExpression)
```

because someone forgot an AST match arm.

That is the qualitative line I would make the next implementation phase cross.

---

# 36. My recommended immediate sequence for the next agent

If I were handing this to the next implementation agent today, I would give them this order:

1. **Re-ground the exact pushed HEAD and verify Step 5.5 acceptance gates.** The visible remote I inspected is behind the state described in your handoff, so this cannot be skipped.

2. **Finish Composition 1.** Correct positional selectors and hover composition, but add compiler-level assertions proving `CellNum.of(42) -> CellNum` and `CellNum !<: Int`. Do not accept a passing LSP test with compiler `Unknown`.

3. **Finish Step 6.** Make all compatibility maps pure projections from current Ready DB products. Add `SemanticPresentationIndex` to the snapshot.

4. **Stop and perform the exhaustive AST semantic-coverage audit.** Produce a matrix before changing the checker.

5. **Implement Step 6.5 semantic closure.** Remove wildcard fallbacks, implement patterns, aliases, missing expression/statement/type constructs, and formal module value resolution.

6. **Implement Step 6.6 inference hardening.** Call-pack validation, generic constraints, kind-aware inference variables, real evidence identities, no `Unit` placeholders.

7. **Implement Step 6.7 evidence preservation.** Separate declared constraint, observed checker fact, effective downstream type, and consistency status.

8. **Audit core/native public-vs-implementation contracts.** Establish a mechanical compatibility test.

9. **Resume Step 7 persistent module lifecycle.** Then repeat all semantic tests cross-module.

10. **Replace LSP heuristics with compiler identities.** After that, build the large Composition 2/3/... suite and finally execute Tasks 16–21.

The important conceptual change is this:

> **Step 5.5 made incremental semantic computation trustworthy. The next job is to make the semantic computation itself comprehensive. Step 6 makes its products authoritative; Step 6.5 makes the analyzer complete; only then should Step 7+ make that completeness workspace-wide and the LSP expose it.**

That ordering will catch far more bugs than continuing to chase each observed hover/inference failure independently.
