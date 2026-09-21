```markdown
# Phalcom Remaining Corpus Failures — Requirements Analysis

## 1. Executive Summary

The remaining corpus failures do **not** represent twenty-seven independent missing features. They collapse into a much smaller number of architectural defects, plus a few genuine source-surface omissions.

The most important finding is that the initial category **“missing source collection APIs” is too broad**. Some selectors really are missing from the intended concrete collection surface, but many of the failing selectors already exist in the canonical `Iterator` / `Iterable` trait defaults and are explicitly specified to participate in ordinary instance dispatch.

Similarly, the `Resource.ensureOpen` failures are more precisely explained by a **hidden-layout authority mismatch** than by an ordinary receiver-type problem: `Resource`'s runtime class is stamped with one hidden slot, but source stub-completion constructs compiler layout metadata from source-visible fields and can publish a zero-field `Resource` layout. Subclasses then allocate their own first field at slot 0 and overwrite the hidden handle installed by `Resource.attach`.

The current failures reduce approximately to these root clusters:

| Root architectural defect | Principal affected corpora |
|---|---|
| Trait-evidenced ordinary dispatch/default-body transport incomplete | `absence`, `blocks`, `family`, `list`, `membership`, `rest_dispatch`, `collections`, `iterator_negative`, `sequence_negative`, probably part of `sequence` |
| Missing eager concrete collection wrappers | `decorators`, `iterator` |
| Nested generic application inference frame closes too early | `bytes`, `bytes_negative`, `collections_literals_negative`, `concurrency_negative`, `path`, `strings`, `system` |
| Empty literal loses nominal container evidence | `iteration`, `iteration_disasm` |
| Reserved implementation namespace rejected lexically before semantic diagnostic | `compile_errors` |
| Universe source name resolution / missing explicit dependency | `concurrency` |
| Internal native method ownership lost along an invocation/installation transport path | `errors`, `runtime_errors` |
| Trait-default nested requirement environment / iterator execution transport defect | `sequence` |
| Hidden `Resource` slot absent from compiler layout authority | `streams`, `streams_negative` |
| Unverified timeout/hang lanes | `lexical`, `time_negative`, `values` |

The architectural objective should therefore **not** be “make each corpus green independently.” The objective should be to restore the canonical ownership boundaries so that the corpora become green as consequences of a small number of semantic/runtime corrections.

---

# 2. Governing Architectural Principles

The fixes should preserve several existing Phalcom invariants.

## 2.1 Trait defaults remain trait-owned

`docs/specs/objects/traits.md` explicitly establishes:

```text
trait default
    trait-owned fallback implementation
```

and:

```text
available through ordinary instance dispatch
    ≠
inherent declaration owned by target
```

A selected trait default must therefore remain trait-owned. It must **not** be copied into every conforming class merely to make runtime lookup succeed.

This is especially important for the iteration migration. The intended architecture is:

```text
concrete collection
    |
    | explicit conformance
    v
Iterable
    |
    | selected default
    v
each / fold / contains / all / ...
```

and:

```text
concrete iterator
    |
    | explicit conformance
    v
Iterator
    |
    | selected default
    v
skip / take / map / filter / ...
```

If ordinary syntax such as:

```phalcom
xs.each(f)
xs.iterator.take(4)
bytes.contains(10)
```

has statically established conformance evidence, the compiler is responsible for executing the semantic selection. The VM should not be expected to rediscover trait semantics through ordinary class dictionaries.

## 2.2 Generic inference belongs to one application region

An expression such as:

```phalcom
Map.new().at(key, put: value)
```

cannot be correctly analyzed as:

```text
solve Map.new()
then
analyze .at(...)
```

because the second invocation is precisely where `K` and `V` become known.

The whole application region must permit information to flow bidirectionally:

```text
Map.new()
   ↓ result Map<K,V>
.at(key, put: value)
   ↑       ↑
   K       V
```

Generic constructors must therefore not be forced to fully solve before consuming context from their surrounding expression.

## 2.3 Runtime infrastructure layout is not source-visible layout

`Resource` intentionally owns a hidden physical slot.

That slot:

- is not a source field;
- must not appear in source reflection as an ordinary field;
- nevertheless occupies physical slot 0;
- must be inherited by every `Resource` subclass;
- must participate in every compiler/runtime field-count calculation.

A hidden slot cannot be represented only by mutating `ClassObject.field_count` if another layout authority can independently recompute the class as having zero fields.

## 2.4 Lexical invalidity and namespace reservation are different failures

A spelling may be intentionally **recognizable but forbidden**.

For example:

```phalcom
_$new()
```

belongs to a namespace the implementation reserves. It should therefore reach the namespace-reservation diagnostic rather than being destroyed by generic lexical rejection.

The lexer must preserve enough structure for the owning diagnostic layer to explain the error correctly.

---

# 3. Cluster A — Collection and Iterator Surface

## 3.1 Current source architecture

The canonical iteration surface is already present in:

```text
phalcom-core/core/universe/src/collections/iterator_laws.ph
```

`Iterator` defines defaults including:

```phalcom
iterator { self }

map(_ f)
filter(_ pred)
filterMap(_ f)
flatMap(_ f)
flatten
chain(_ other)
skip(_ n)
skipWhile(_ pred)
take(_ n)
takeWhile(_ pred)
mapWhile(_ f)
inspect(_ f)
zip(_ other)
```

`Iterable` defines defaults including:

```phalcom
iter
indexed
each(_ f)
each(indexed f)
contains(_ x)
includes(_ x)
count
count(where f)
find(where f)
index(where f)
any(where f)
all(where f)
none(where f)
nth(_ n)
...
```

Concrete conformances also exist.

For example, `list.ph` ends with:

```phalcom
impl<T> CanonicalIterator for ListIterator<T> {
  type Item = T
}

impl<T> CanonicalIterable for List<T> {
  type Item = T
  type Iterator = ListIterator<T>

  iterator -> ListIterator<T> {
    ListIterator.new(self)
  }
}
```

Likewise, `Tuple`, `Bytes`, and `Range` already publish canonical `Iterator` / `Iterable` conformances.

This changes the interpretation of several failures.

---

# 4. `each`, `fold`, `contains`, `skip`, and `take` Are Not Primarily Missing Methods

The following failures:

```text
List does not understand each(_)
List does not understand fold(initial,using)
tuple does not understand each(_)
Bytes does not understand contains(_)
RangeIterator does not understand take(_)
ListIterator does not understand skip(_)
```

must first be treated as a **trait dispatch failure**, not as evidence that copies of those algorithms should be added to concrete classes.

The architecture explicitly says trait behavior selected by an applicable conformance participates in ordinary syntax.

Therefore:

```phalcom
list.each(f)
```

must conceptually lower as:

```text
semantic receiver type = List<T>
        ↓
prove List<T> |= Iterable
        ↓
select Iterable.each
        ↓
publish TraitInvocationSpec
        ↓
InvokeTraitSelected
```

It must not become:

```text
Invoke ordinary selector "each(_)"
        ↓
runtime class dictionary
        ↓
DNU
```

### Requirement COLL-DISPATCH-01

For a concrete receiver with statically established applicable trait evidence, ordinary member syntax must retain the selected `TraitDispatchSite` through semantic projection and compiler lowering.

### Requirement COLL-DISPATCH-02

A selected trait default must execute through the existing trait invocation machinery:

```text
TraitDispatchSite::Evidenced
ModuleLoweringSemantics::trait_invocations
TraitInvocationSpec
InvokeTraitSelected
RuntimeConformanceEnvironment
```

No runtime string-based trait rediscovery is permitted.

### Requirement COLL-DISPATCH-03

The compiler must not fall back to an ordinary `Invoke` merely because the selected callable is trait-owned rather than target-owned.

### Requirement COLL-DISPATCH-04

The correction must cover at minimum:

```text
List<T> |= Iterable
Tuple |= Iterable
Bytes |= Iterable
Range |= Iterable

ListIterator<T> |= Iterator
TupleIterator |= Iterator
BytesIterator |= Iterator
RangeIterator |= Iterator
```

and custom user-defined conformances.

### Forbidden solution

Do not repair:

```text
List.each
Tuple.each
Bytes.contains
ListIterator.skip
RangeIterator.take
```

by duplicating the canonical algorithms into every class.

That would:

1. violate trait-default ownership;
2. create separate algorithm implementations;
3. make custom `Iterable` / `Iterator` implementations second-class;
4. hide the actual trait dispatch defect;
5. cause semantic and reflection surfaces to disagree.

---

# 5. `List.map` and `List.filter` Are Different

`List.map` and `List.filter` are a genuine exception.

The accepted stateful-iteration migration intentionally distinguishes:

```text
collection.map/filter
    eager collection operation

iterator.map/filter
    lazy stateful iterator adapter
```

Therefore direct collection operations such as:

```phalcom
[1, 2, 3].map(...)
[1, 2, 3].filter(...)
```

need an inherent collection-facing surface if existing language semantics require eager results.

The appropriate architecture is a **thin inherent wrapper**, not another implementation of traversal.

Conceptually:

```phalcom
impl<T> List<T> {
  map<U>(_ f) -> List<U> {
    self.iterator.map(f).toList
  }

  filter(_ pred) -> List<T> {
    self.iterator.filter(pred).toList
  }
}
```

Exact spelling and generic syntax should follow the live language surface, but ownership should remain:

```text
iteration algorithm      Iterator defaults/adapters
collection eagerness     List wrapper
materialization          canonical toList/default
```

### Requirement COLL-SURFACE-01

`List.map` and `List.filter` must preserve their accepted **eager collection semantics**.

### Requirement COLL-SURFACE-02

They should delegate to the canonical iterator machinery rather than reproduce iteration state machines.

### Requirement COLL-SURFACE-03

No hidden runtime superclass should be reintroduced simply to inherit these methods.

---

# 6. `Bytes.contains`

`Bytes.contains(_)` is already supplied by `Iterable.contains`.

That means the reported:

```text
Bytes does not understand contains(_)
```

should initially be treated as another trait-default dispatch failure.

A dedicated `Bytes.contains` is justified only if the language specification intentionally requires specialized membership semantics or measured optimized implementation later.

Correctness does not require one.

Therefore the first acceptance test should prove:

```phalcom
const b = Bytes.fromList([1, 2, 3])

b.contains(2)   // true
b.contains(9)   // false
```

through canonical `Iterable` evidence.

---

# 7. Cluster B — Nested Generic Factory Inference

## 7.1 Reproduction

The simplest representative is:

```phalcom
const m = Map.new()
m.at(a.toTuple, put: "ok")
```

and especially:

```phalcom
Map.new().at(key, put: value)
```

The relevant declaration is:

```phalcom
native class Map<K, V> is Collection {
  @class
  native new() -> Map<K, V>

  at(_ k, put) {
    self.rawPut(k, put)
    return self
  }
}
```

`Map.new()` has no argument evidence for either `K` or `V`.

The next send does:

```text
k     constrains K
put   constrains V
```

Yet the current checker can create an independent inference context for the nested generic factory when its incoming `ExpectedType` is not already an inference context.

From `call.rs`:

```rust
let (context, root_frame, owns_context) = match expected {
    ExpectedType::Inference { context, .. } => {
        ...
        (*context, frame, false)
    }

    _ => {
        let (context, frame) = ctx.create_inference_context();
        (context, frame, true)
    }
};
```

and after application:

```rust
ctx.pop_inference_frame(context, root_frame);

if let Some(session) = ctx.inference_session(context) {
    session.borrow_mut().close_frame(root_frame);
}

if owns_context {
    ctx.finish_inference_context(context);
}
```

That behavior is locally reasonable for a standalone application but wrong for a nested generic receiver whose result participates in a surrounding call.

The child application is being required to solve before its consumer has supplied the evidence that makes it solvable.

## 7.2 Required inference model

The application:

```phalcom
Map.new().at(k, put: v)
```

must be representable as one inference region:

```text
fresh α, β

Map.new() -> Map<α, β>

receiver of Map.at must be Map<K, V>

unify:
    α == K
    β == V

argument k  => K
argument v  => V

solve region
```

The generic result of the nested call may remain symbolic while the surrounding member invocation contributes constraints.

### Requirement INF-01

A generic child expression used as the receiver or argument of another application must be able to participate in the parent inference context.

### Requirement INF-02

Inference frames may be nested for ownership/lifetime purposes, but a child frame must not be finalized as underconstrained while unresolved variables are legitimately constrained by its parent application.

### Requirement INF-03

Expected receiver type is a legitimate source of generic inference evidence.

For:

```phalcom
genericFactory().method(...)
```

the outer method's owner/receiver specialization must be able to contextualize the factory result.

### Requirement INF-04

The solution must generalize beyond `Map.new()`.

It must support equivalent patterns such as:

```phalcom
Set.new().add(x)

Box.new().consume(...)

Factory.new().member(arguments...)

genericProducer().genericConsumer(...)
```

where surrounding use determines otherwise-unconstrained result parameters.

### Requirement INF-05

Underconstrained inference must still be diagnosed when the complete inference region closes without enough evidence.

For example, if no context exists:

```phalcom
const m = Map.new()
```

and Phalcom requires concrete `K` and `V`, the checker may still report underconstraint.

The correction is **delayed/contextual solving**, not suppression of legitimate underconstraint.

## 7.3 Forbidden solutions

Do not:

```text
K = Dynamic
V = Dynamic
```

whenever inference fails.

Do not special-case `Map.new`.

Do not defer all generic checking to runtime.

Do not infer from unrelated later statements through unrestricted global flow.

Do not make `Dynamic` a universal error-recovery answer.

Those approaches weaken the type system rather than fix the inference-region boundary.

## 7.4 Expected corpus consequences

Correcting the shared inference boundary should unblock:

```text
bytes
bytes_negative
collections_literals_negative
concurrency_negative
path
strings
system
```

Importantly, several are negative corpora. They should then proceed far enough to produce their **intended runtime/semantic failures**, rather than becoming positive tests accidentally.

---

# 8. Cluster C — Empty List in Canonical `for`

The remaining iteration failure is specifically:

```phalcom
for x in [] {
    ...
}
```

The existing list-literal checker currently publishes bare `[]` with no element evidence as:

```text
Unknown(NoTypeEvidence)
```

Canonical `for` lowering, correctly, refuses to fabricate iteration behavior. It requires:

```text
Source |= Iterable
Source::Item
Source::Iterator
selected iterator requirement
selected next requirement
```

Once the entire expression has collapsed to `Unknown`, none of these facts can be proven.

The problem therefore sits **before lowering**.

## 8.1 The crucial modeling error

An empty list is not completely unknown.

Even when its element type is unknown, this much is known:

```text
container constructor = List
element type          = unknown / inference variable
```

Current representation effectively loses both:

```text
[]
    ->
Unknown(NoTypeEvidence)
```

A more accurate model is conceptually:

```text
[]
    ->
List<α>
```

where `α` is a fresh inference variable or another representation of unresolved element evidence.

Then:

```text
List<α> |= Iterable
List<α>::Item = α
List<α>::Iterator = ListIterator<α>
```

are all structurally provable without knowing `α`.

## 8.2 Requirements

### Requirement ITER-EMPTY-01

An empty collection literal must preserve its known nominal constructor even when element evidence is absent.

### Requirement ITER-EMPTY-02

`for` must consume normal canonical `Iterable` evidence for an empty list exactly as it does for a non-empty list.

No special “empty list loop” compiler lane should exist.

### Requirement ITER-EMPTY-03

The loop variable may carry an unresolved contextual element variable while the loop body is analyzed.

Body use may add constraints.

Example:

```phalcom
for x in [] {
    const y: Int = x
}
```

can constrain the element variable toward `Int`, even though the runtime list contains no values.

### Requirement ITER-EMPTY-04

An unresolved element variable that does not escape and is irrelevant to executable behavior must not prevent proving the `Iterable` structure itself.

The checker needs to distinguish:

```text
unknown nominal structure
```

from:

```text
known List structure with unresolved element argument
```

### Forbidden solutions

Do not compile:

```phalcom
for x in [] { ... }
```

directly into zero iterations as a constant-folding exception.

That would bypass:

- ordinary source evaluation rules;
- canonical iteration semantics;
- loop-variable typing;
- generic consistency;
- disassembly invariants.

The empty case should work because `[]` is still a `List<α>`, not because the compiler recognizes a special AST shape.

---

# 9. Cluster D — Reserved Constructor/Internal Namespace Diagnostic

Fixture:

```phalcom
class Foo {
  _$new() { 1 }
}
```

Expected:

```text
internal.namespace_reserved
```

Actual:

```text
Invalid token
```

The lexer currently describes identifiers approximately as:

```text
[A-Za-z_][A-Za-z0-9_]*
```

but its underscore-specific scanner intentionally rejects malformed/reserved sequences containing `$`.

Therefore:

```text
_$new
```

dies before parser/compiler namespace validation can see it.

This is a diagnostic-layer precedence defect.

## 9.1 Required architecture

Reserved implementation spellings should be **lexically recognizable but semantically unavailable**.

Conceptually:

```text
source
  ↓
reserved implementation spelling token
  ↓
AST/member declaration
  ↓
namespace policy
  ↓
internal.namespace_reserved
```

rather than:

```text
source
  ↓
generic lexical error
```

### Requirement DIAG-NS-01

The scanner must recognize the reserved `_$...` namespace as a structured spelling.

### Requirement DIAG-NS-02

Recognizing it must not make `_$...` a normal user identifier namespace.

A dedicated lexical representation is preferable to silently broadening ordinary identifier grammar.

For example, conceptually:

```rust
Token::ReservedInternalIdentifier(String)
```

would preserve enough information for later diagnostic ownership without legitimizing the namespace.

Exact token architecture may vary.

### Requirement DIAG-NS-03

The semantic/compiler owner of the implementation namespace must emit:

```text
internal.namespace_reserved
```

for user declarations.

### Requirement DIAG-NS-04

Canonical compiler-generated or privileged Universe use of the namespace must remain possible through explicit authority, not parser ambiguity.

### Forbidden solution

Do not special-case only:

```text
_$new
```

in the parser.

The namespace rule applies to the namespace, not one constructor spelling.

---

# 10. Cluster E — `Backoff` Resolves `Duration` Against `self`

The source currently contains:

```phalcom
class Backoff {
    ...

    waitBefore(_ attempt) {
        ...
        System.sleep(Duration.milliseconds(delay)).await
        ()
    }
}
```

The file `concurrency/fiber.ph` does not declare an explicit `Duration` import.

Compiler bare-name resolution currently falls through approximately as:

```rust
if local { ... }
else if upvalue { ... }
else if local global { ... }
else if linked import { ... }
else if privileged universe && canonical_universe_binding(name).is_some() {
    ...
}
else if known global { ... }
else if method has self {
    BareNameResolution::ImplicitSelf
}
```

Therefore, when `Duration` is not present as a linked/materialized global at that point:

```phalcom
Duration.milliseconds(...)
```

can become conceptually:

```phalcom
self.Duration.milliseconds(...)
```

and for this body:

```text
self = Backoff
```

hence:

```text
Backoff does not understand Duration
```

This is a deterministic consequence of the current fallback order.

## 10.1 Preferred architectural interpretation

`Duration` is a source-owned type from:

```text
universe.time.duration
```

and is not an ordinary public prelude binding.

The clean dependency is therefore explicit:

```phalcom
from universe.time.duration import Duration
```

in the concurrency source that consumes it.

This makes dependency identity independent of Universe materialization order.

### Requirement NAME-01

Cross-module source dependencies inside the Universe should use the same explicit module-linking model as ordinary source unless the referenced declaration is intentionally part of the prelude.

### Requirement NAME-02

A missing source dependency must not be silently converted into an implicit-self send when the compiler already has semantic evidence that the spelling denotes a type/global declaration.

Longer-term, the semantic/compiler name-resolution products should converge sufficiently that this class of disagreement is impossible.

### Requirement NAME-03

Universe bootstrap order must not determine whether the same source token means:

```text
global Duration
```

or:

```text
self.Duration
```

### Non-solution

Do not make every canonical Universe class a global implicit prelude name merely to fix `Backoff`.

That would weaken module boundaries and create new shadowing ambiguities.

---

# 11. Cluster F — Internal Selector Ownership

Failures:

```text
invariantEnter() owned by <unbound>
attachAttribute(_) owned by <unbound>
```

Expected ownership is known.

Both selectors are canonical internal `Object` primitives.

`object.ph` declares:

```phalcom
internal native invariantEnter() -> Dynamic
internal native invariantExit() -> Dynamic

internal native rawAttributes -> Dynamic
internal native attachAttribute(_ attribute: Dynamic) -> Dynamic
internal native freezeAttributes() -> Dynamic
```

The legacy primitive installer also explicitly installs them on `Object`.

`primitive_internal!` sets:

```rust
let mut method =
    MethodObject::new_primitive(..., $class);

method.visibility =
    MemberVisibility::Internal;

method.access_owner =
    Some($class);
```

The newer descriptor installer likewise does:

```rust
method.visibility = desc.surface.visibility;
method.access_owner = desc.internal_access_owner(owner);
```

Therefore `<unbound>` is **not created at either canonical primitive declaration seam**.

The ownership information is being lost later, or an alternative method representation is being selected.

This is an important narrowing of the defect.

## 11.1 Runtime authorization path

`authorize_method_access_as` eventually renders:

```rust
let owner = owner
    .map(|id| self.heap.class(id).name.as_str())
    .unwrap_or("<unbound>");
```

Thus `<unbound>` proves that the `MethodObject` passed into authorization actually has:

```rust
access_owner == None
```

at that point.

This is not merely a rendering issue.

## 11.2 Required investigation boundary

All paths that can transport or reconstruct an executable method must preserve:

```text
holder
access_owner
visibility
origin
implementation provenance
selector/signature
```

Candidates include:

```text
reflective perform
method-family capture
bound method construction
exact method invocation
method replacement during native descriptor reconciliation
legacy → descriptor installation replacement
detached/native callable reconstruction
```

The `runtime_attribute_store_frozen` fixture is particularly useful because it reaches the selector through:

```phalcom
Engine.perform(
    #"attachAttribute(_)",
    ***(Author.new("Bob"),)
)
```

Therefore reflective forwarding must preserve the selected method's lexical/access ownership rather than fabricating a semantically incomplete callable.

### Requirement ACCESS-01

Every executable `MethodObject` representing a declared member must have canonical ownership metadata before it can be published.

### Requirement ACCESS-02

Any operation that copies, binds, captures, wraps, replaces, or forwards a Method must preserve its access authority.

### Requirement ACCESS-03

Reflective `perform` changes invocation mechanics and caller authority; it must not alter the callee's declaration owner.

### Requirement ACCESS-04

Internal selectors must remain ordinary selector identities.

Do not infer internal ownership from their names at invocation time.

### Requirement ACCESS-05

There must be an invariant-level test over the actual installed methods:

```text
Object#invariantEnter()     access_owner == Object
Object#invariantExit()      access_owner == Object
Object#attachAttribute(_)   access_owner == Object
Object#rawAttributes        access_owner == Object
Object#freezeAttributes()   access_owner == Object
```

and another test after reflective capture/binding.

### Forbidden solutions

Do not change:

```rust
None => receiver.class(...)
```

inside access authorization.

That would turn missing metadata into ambient authority and could incorrectly authorize private/protected/internal members.

`<unbound>` is evidence of broken metadata provenance and should remain detectable.

---

# 12. Cluster G — `sequence_all_false_short_circuits`

The source implementation of `all(where:)` itself is straightforward:

```phalcom
all(where f) {
  let it = self.iterator
  let step = it.next

  while (step.isSome) {
    let val = step.match(
      some: |v| { v },
      none: || { None }
    )

    if (not f.call(val)) {
      return false
    }

    step = it.next
  }

  return true
}
```

The expected fixture behavior is:

```text
false
3
```

for `[1,2,3,4,5]` with predicate `x < 3`.

There is no obvious short-circuit error in this source algorithm.

The observed:

```text
None does not understand next
```

therefore points below the algorithm itself.

## 12.1 Likely architectural seam

An `Iterable` default body requires nested selected behavior:

```text
Iterable.all
    ↓
Iterable.iterator requirement
    ↓
selected List.iterator
    ↓
ListIterator
    ↓
Iterator.next
```

Trait default execution already uses an explicit runtime conformance environment.

A default cannot execute merely with a handle to its own callable. It needs the complete selection environment that tells calls inside the body which requirement/default/witness each trait operation denotes.

This is particularly relevant to the existing architecture where trait defaults are detached and do not become target-owned methods.

Thus a failure in which a nested operation receives `None` is consistent with incomplete/default-body environment transport.

### Requirement ITER-EXEC-01

Entering a trait default must materialize the complete requirement environment selected for that conformance.

### Requirement ITER-EXEC-02

Default → requirement and default → default calls must preserve the current conformance environment.

### Requirement ITER-EXEC-03

`Iterable.all` must receive the same selected `iterator` witness that a direct call:

```phalcom
xs.iterator
```

would receive.

### Requirement ITER-EXEC-04

The returned iterator object's `Iterator` evidence must remain available for its subsequent `next` execution where required by lowering.

### Requirement ITER-EXEC-05

Short-circuiting must perform no read beyond the first false result.

The fixture's `counter == 3` is a semantic requirement, not merely an output snapshot.

### Important relationship

This likely belongs to the same family as the previously identified LANG005 default-body transport problem:

```text
trait default selected
    +
nested requirement evidence omitted
    =
default body executes with incomplete environment
```

It should be investigated together rather than patched as a special `all(where:)` issue.

---

# 13. Cluster H — `Resource` / Streams Hidden Slot

This failure can now be diagnosed more precisely.

Bootstrap explicitly stamps:

```rust
let res_class = vm.universe.classes.resource_class;
vm.heap.class_mut(res_class).field_count = 1;
```

The design requires:

```text
slot 0 = hidden packed Resource handle
slot 1+ = source subclass fields
```

`bytes.ph` then defines:

```phalcom
native class Resource is Object {
  internal native attach(_ kind: String) -> Unit
  internal native rawClose() -> Dynamic
  internal native rawIsClosed -> Bool
  internal native ensureOpen -> Unit
}
```

and:

```phalcom
class BufferedWriter is Resource {
  @constructor
  new(_ inner) {
    self.attach("BufferedWriter")
    _inner = inner
    _buf = Bytes.new(8192)
    _len = 0
    _busy = false
  }
}
```

Correct physical layout must therefore be:

```text
BufferedWriter
slot 0  Resource handle
slot 1  _inner
slot 2  _buf
slot 3  _len
slot 4  _busy
```

## 13.1 Compiler stub-completion conflict

`class_decl.rs` handles bootstrapped class completion specially.

For an already-existing stub, superclass field count is derived from the **stub's superclass**:

```rust
let existing_class = self.vm.classes[&name_key];

match self.vm.heap.class(existing_class).superclass {
    Some(sc_id) => (
        self.vm.heap.class(sc_id).field_count,
        ...
    ),
    None => (0, 0),
}
```

Then source layout is computed as:

```text
field_count =
    superclass field count
    +
    source-declared own fields
```

For source `Resource`:

```text
superclass Object field count = 0
source Resource own fields     = 0

compiler Resource layout       = 0
```

But runtime bootstrap says:

```text
runtime Resource field count   = 1
```

So two authorities disagree:

```text
heap ClassObject        Resource.field_count = 1
compiler field_layouts  Resource.field_count = 0
```

A subsequent source subclass consults `field_layouts` first when deriving inherited offsets.

Therefore:

```text
BufferedWriter._inner -> slot 0
```

is a plausible and architecturally consistent consequence.

Constructor execution then does:

```text
self.attach(...)
    slot 0 = packed handle

_inner = inner
    slot 0 = inner
```

The handle is immediately destroyed.

Later:

```phalcom
self.ensureOpen
```

reads slot 0 and sees an ordinary object rather than the packed handle.

That explains why stream operations fail at `Resource.ensureOpen` even though construction initially reached `attach`.

This is substantially more precise than “incorrect Resource receiver binding.”

## 13.2 Required architecture

There must be one layout authority for hidden runtime-prefix fields.

Conceptually:

```text
physical inherited prefix
    +
source-visible inherited fields
    +
source own fields
```

not:

```text
source-visible superclass layout only
```

### Requirement RES-LAYOUT-01

Stub completion must preserve bootstrapped physical field prefixes.

### Requirement RES-LAYOUT-02

`Resource` must publish a compiler-visible physical count of 1 even though it publishes zero source-visible fields.

### Requirement RES-LAYOUT-03

Field visibility and physical offset allocation must be separate concepts.

The following must be simultaneously true:

```text
Resource.field_count physical = 1
Resource reflected user fields = []
```

### Requirement RES-LAYOUT-04

Every `Resource` subclass's first source field must have offset >= 1.

### Requirement RES-LAYOUT-05

Compiler layout metadata and runtime `ClassObject` layout must not disagree after canonical Universe materialization.

### Requirement RES-LAYOUT-06

The solution should generalize to any future runtime-hidden inherited storage.

Do not encode a special:

```text
if class == Resource then +1
```

rule into subclass compilation.

A class/layout representation should be able to describe hidden physical prefixes structurally.

## 13.3 Required discriminating invariant

A focused test should establish:

```text
Resource runtime field_count       == 1
Resource compiler layout count     == 1
Resource source-visible fields     == 0

BufferedWriter._inner offset       == 1
BufferedWriter._buf offset         == 2
BufferedWriter._len offset         == 3
BufferedWriter._busy offset        == 4
BufferedWriter runtime field_count == 5
```

and construction must prove:

```text
slot 0 after attach
==
slot 0 after all ordinary field initialization
```

Only after this invariant holds is it useful to test stream protocol semantics.

### Forbidden solutions

Do not weaken `Resource.ensureOpen`.

Do not search other slots for something that “looks like” a handle.

Do not make the handle a normal `_handle` source field again.

Do not move it into an unrelated side table solely to avoid fixing layout metadata.

The hidden-slot model is coherent; the current compiler layout representation is failing to preserve it.

---

# 14. Consequence for `streams_negative`

`unflushed_close.ph` is currently preempted by the same corrupted Resource state.

Once the hidden slot is preserved, the fixture should reach its actual contract:

```text
BufferedWriter.close
    with pending bytes
        ->
UnflushedError
```

The stream negative fixture must not be altered to expect the present `Resource` failure.

---

# 15. Failure Dependency Graph

The observed corpus failures should be understood approximately as:

```text
                           ┌─────────────────────────────┐
                           │ trait ordinary dispatch     │
                           │ + default env transport     │
                           └──────────────┬──────────────┘
                                          │
                ┌─────────────────────────┼────────────────────────┐
                │                         │                        │
                v                         v                        v
         each/fold/contains          skip/take                all(where)
         List/Tuple/Bytes           iterators                 None.next
                │                         │                        │
                v                         v                        v
       absence / blocks /          collections /              sequence
       family / list /             iterator_negative /
       membership / rest           sequence_negative


┌──────────────────────────────┐
│ nested generic inference     │
└──────────────┬───────────────┘
               │
               ├── bytes
               ├── bytes_negative
               ├── collections_literals_negative
               ├── concurrency_negative
               ├── path
               ├── strings
               └── system


┌──────────────────────────────┐
│ hidden Resource layout split │
└──────────────┬───────────────┘
               │
               ├── streams
               └── streams_negative


┌──────────────────────────────┐
│ empty literal loses List     │
│ nominal type                 │
└──────────────┬───────────────┘
               │
               ├── iteration
               └── iteration_disasm
```

This is why corpus-by-corpus patching would be particularly damaging: one architectural repair is intended to close several lanes simultaneously.

---

# 16. Corrected Classification of the Reported Failures

| Corpus | Original symptom | Refined diagnosis |
|---|---|---|
| `absence` | `List.each` missing | Trait-evidenced ordinary dispatch/default selection not reaching execution |
| `blocks` | `List.fold` missing | Same trait-default dispatch cluster |
| `bytes` | generic underconstraint | Nested generic receiver factory inference closes before consumer constraints |
| `bytes_negative` | same | Preempted intended runtime negative |
| `collections` | `RangeIterator.take` missing | Iterator default dispatch failure |
| `collections_literals_negative` | generic underconstraint | Same nested inference defect |
| `compile_errors` | `Invalid token` | Reserved implementation namespace dies in lexer too early |
| `concurrency` | `Backoff.Duration` | Missing explicit source dependency / order-sensitive bare-name fallback to implicit `self` |
| `concurrency_negative` | generic underconstraint | Same nested inference defect |
| `decorators` | `List.filter` missing | Genuine eager List wrapper absent |
| `errors` | `invariantEnter` owner `<unbound>` | Method authority metadata lost after canonical declaration/installation |
| `family` | tuple `each` missing | Trait-default dispatch failure |
| `iteration` | `for []` | Empty literal loses nominal `List` evidence |
| `iteration_disasm` | same | Same semantic-product absence |
| `iterator` | `List.map` missing | Genuine eager List wrapper absent |
| `iterator_negative` | `ListIterator.skip` missing | Iterator default dispatch preempts intended argument validation |
| `list` | `List.each` missing | Trait-default dispatch failure |
| `membership` | `Bytes.contains` missing | Trait-default dispatch failure |
| `path` | generic underconstraint | Nested inference defect |
| `rest_dispatch` | tuple `each` missing | Trait-default dispatch failure |
| `runtime_errors` | `attachAttribute` owner `<unbound>` | Method ownership transport/reconstruction defect |
| `sequence` | `None.next` in `all` | Likely incomplete trait-default requirement environment, not `all` algorithm |
| `sequence_negative` | `ListIterator.skip` missing | Iterator default dispatch preempts intended count validation |
| `streams` | `ensureOpen` failure | Hidden Resource slot overwritten due compiler/runtime layout authority split |
| `streams_negative` | same | Same layout defect preempts intended `UnflushedError` |
| `strings` | generic underconstraint | Nested inference defect |
| `system` | generic underconstraint | Nested inference defect |

---

# 17. Required Negative-Lane Behavior

Several negative corpora are currently failing “too early.”

This distinction must be preserved.

A negative fixture passes only when it reaches the intended rejection boundary.

Examples:

```text
bytes_negative
    must reach invalid Bytes-as-key behavior

collections_literals_negative
    must reach concurrent mutation behavior

concurrency_negative
    must reach native-frame resume restriction

iterator_negative
    must reach invalid skip-count rejection

sequence_negative
    must reach negative-count rejection

streams_negative
    must reach UnflushedError
```

Therefore acceptance must not merely be:

```text
program exited unsuccessfully
```

It must assert the intended diagnostic/error category.

---

# 18. Cross-Cutting Source-of-Truth Requirements

## 18.1 Type inference

Owner:

```text
phalcom-semantic generic inference session
```

Forbidden duplicate:

```text
compiler/runtime guessing unresolved generic parameters
```

## 18.2 Trait selection

Owner:

```text
semantic conformance resolution
```

Consumers:

```text
semantic lowering
compiler
VM selected-trait execution
```

Forbidden duplicate:

```text
runtime class scanning for traits
```

## 18.3 Collection algorithms

Owner:

```text
Iterator / Iterable canonical defaults
```

Concrete collection wrappers own only:

```text
collection-specific eagerness
collection-specific result shape
```

## 18.4 Resource physical layout

Owner must become a single coherent layout product containing both:

```text
runtime-hidden prefix
source-visible fields
```

Forbidden duplicate:

```text
ClassObject.field_count
vs
ClassLayout.field_count

with independently derived meanings
```

## 18.5 Internal access authority

Owner:

```text
MethodObject declaration metadata
```

Consumers:

```text
ordinary invocation
perform
bind
method references
native activation
```

Forbidden fallback:

```text
infer owner from receiver
infer owner from selector spelling
```

---

# 19. Architecture-Level Non-Solutions

The following approaches should be rejected even if they make selected corpus fixtures green.

### 19.1 Add every missing selector directly to every collection

Reject.

This destroys the trait-default model.

### 19.2 Default unresolved generics to `Dynamic`

Reject.

This turns missing inference propagation into permanent unsoundness/precision loss.

### 19.3 Special-case `Map.new()`

Reject.

The defect is generic nested application inference.

### 19.4 Special-case `for x in []`

Reject.

The defect is loss of partially known nominal type structure.

### 19.5 Allow all `_$...` identifiers normally

Reject.

The implementation namespace should remain reserved.

### 19.6 Make `Duration` universally prelude-visible

Reject unless separately specified.

The concurrency module should express its dependency.

### 19.7 Treat `<unbound>` access owners as receiver-owned

Reject.

That turns metadata corruption into incorrect authority.

### 19.8 Search Resource instance slots for the handle

Reject.

Slot 0 is deliberately canonical.

### 19.9 Re-expose `_handle`

Reject.

The hidden Resource state is an implementation invariant and should remain non-source-addressable.

---

# 20. Acceptance Requirements by Root Cause

## Trait dispatch/default transport

The following must execute through canonical trait selection:

```phalcom
[1, 2].each(...)
(1, 2).each(...)
Bytes.fromList([1, 2]).contains(2)

[1, 2].iterator.skip(1)
(1..10).iterator.take(2)
```

A user-defined iterator implementing only:

```phalcom
type Item
next
```

must receive the same defaults.

No concrete-method duplication may be necessary.

## Eager collection wrappers

Direct:

```phalcom
list.map(...)
list.filter(...)
```

must produce eager `List` values while:

```phalcom
list.iterator.map(...)
list.iterator.filter(...)
```

remain lazy iterators.

## Generic inference

These must infer:

```phalcom
Map.new().at("a", put: 1)
Set.new().add(1)
```

while truly context-free underconstrained factories still receive a precise diagnostic if required by the language.

## Empty list iteration

This must compile through the exact same canonical `for` machinery:

```phalcom
for x in [] {
    System.print(x)
}
```

with zero body executions.

## Namespace diagnostic

This:

```phalcom
class Foo {
    _$new() { 1 }
}
```

must yield:

```text
internal.namespace_reserved
```

not a generic lexer error.

## Backoff

`Backoff.waitBefore` must resolve `Duration` through a deterministic linked declaration, independent of Universe initialization order.

## Internal ownership

Installed and forwarded internal Object methods must retain:

```text
holder       = Object
access_owner = Object
visibility   = Internal
```

where applicable.

The top-level reflective call to `attachAttribute(_)` must therefore fail with the expected owner:

```text
owned by `Object`
```

not:

```text
owned by `<unbound>`
```

## Sequence

`all(where:)` must produce:

```text
false
3
```

for the existing short-circuit fixture.

No fourth element may be observed.

## Resource

After constructing:

```phalcom
BufferedWriter.new(inner)
```

the packed handle in physical slot 0 must still exist after `_inner`, `_buf`, `_len`, and `_busy` initialization.

---

# 21. Timeout / Incomplete Lanes

The following must remain explicitly **unverified**:

```text
lexical
time_negative
values
```

`time_negative` may have historical passing evidence, but the current run did not reconfirm it.

`lexical` exceeding five minutes deserves particular attention.

That should not automatically be treated as a correctness failure, but neither should it be marked green.

It requires classification among:

```text
legitimate unusually expensive corpus
test harness deadlock
semantic fixed-point nontermination
runtime infinite loop
bootstrap repeated-work regression
pathological compilation complexity
```

The correct state is therefore:

```text
lexical        UNKNOWN / TIMEOUT
time_negative  HISTORICALLY GREEN, CURRENTLY UNVERIFIED
values         UNKNOWN / TIMEOUT
```

No baseline closure should silently convert these into passes.

---

# 22. Relative Architectural Risk

| Cluster | Semantic risk | Runtime risk | Fanout | Why |
|---|---:|---:|---:|---|
| Nested generic inference | Very high | Low | High | Changes inference-region ownership and generic result contextualization |
| Trait dispatch/default environment | Very high | High | High | Crosses semantic selection → lowering → VM execution |
| Resource hidden layout | High | Very high | Medium | Silent physical-slot aliasing corrupts infrastructure state |
| Empty collection typing | High | Low | Medium | Changes representation of unresolved collection literals |
| Internal method ownership | Medium-high | High | Medium | Access-control authority must survive every callable transport |
| Backoff/Duration | Low-medium | Low | Low | Mostly source dependency/name-resolution discipline |
| Reserved namespace diagnostic | Low | None | Low-medium | Lexer/parser/compiler diagnostic precedence |
| Eager List wrappers | Low | Low | Low | Surface restoration over canonical algorithms |

---

# 23. Strongest Overall Diagnosis

The current corpus is revealing a common pattern:

> **Phalcom increasingly has the right semantic objects, but several execution boundaries still collapse them into older, less expressive runtime representations.**

Examples:

```text
trait-evidenced method
    becomes ordinary class-dictionary send

partially-known List<α>
    becomes completely Unknown

nested generic result with live variables
    becomes prematurely closed inference result

hidden physical class layout
    becomes source-visible field layout only

internal Method authority
    becomes callable with owner = None

cross-module declaration identity
    becomes unresolved bare name → implicit self
```

These are not isolated library bugs. They are **information-loss bugs at subsystem boundaries**.

That should guide the fixes.

The desired architecture is:

```text
semantic fact established once
        ↓
stable structured representation
        ↓
compiler projection
        ↓
runtime transport
        ↓
execution

with no reconstruction and no loss
```

not:

```text
semantic fact
   ↓
string / ordinary selector / unknown / source-only layout
   ↓
try to rediscover meaning later
```

---

# 24. Final Requirements Position

The remaining corpus should be stabilized by restoring the following invariants:

1. **Trait behavior selected semantically remains selected all the way to execution.**
2. **Nested generic applications share inference evidence for as long as surrounding syntax can legitimately constrain them.**
3. **Partially known types retain known structure rather than collapsing wholesale to `Unknown`.**
4. **Runtime-hidden storage participates in the same physical layout authority used by subclass compilation.**
5. **Callable access authority survives every installation, binding, reflection, and forwarding operation.**
6. **Universe module dependencies are explicit or deterministically linked, never dependent on initialization order.**
7. **Reserved implementation syntax remains recognizable enough for its owning diagnostic to fire.**
8. **Concrete collection methods exist only where collection semantics actually differ from canonical iterator/iterable behavior.**

If these invariants are restored, most of the listed corpus failures should disappear in clusters rather than one fixture at a time.

The highest-value correction is therefore not adding methods until the test matrix turns green. It is restoring the semantic information that the current compiler/runtime boundary is dropping.
```

The most significant newly verified issue is the `Resource` layout split: the runtime hidden slot and compiler `ClassLayout` can disagree, which explains the stream failures much better than a nominal receiver mismatch. The collection cluster also needs to be split: `each`/`contains`/`skip`/`take` belong to trait-dispatch repair, while direct `List.map`/`filter` need eager inherent wrappers.