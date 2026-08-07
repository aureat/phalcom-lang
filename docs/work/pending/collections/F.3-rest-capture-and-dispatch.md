# Spec F.3 — Rest Capture and Rest-Pattern Dispatch

Status: implementation specification. Requires F.1/F.2 and A's Tuple/Unit finalization.

## 1. Mission

Replace the repository's old positional-only U9 variadic model with the ratified lane-symmetric capture model:

```text
*rest      positional residual lane
**rest     labeled residual lane
***rest    complete residual pack
```

Capture into Tuple products, not List.

Exact selector lookup remains first.

Rest dispatch remains purely selector/pack-shape based; no type-based dispatch is introduced.

Expected public primitive-floor delta: **0**.

## 2. Baseline to retire

Current baseline U9 has:

```text
ParameterDef.is_rest: bool
SignatureKind::Variadic(fixedPositionalArity)
selector spelling name(*)
Signature.variadic: bool
call_method: trailing positional values -> List
```

and an exact-miss probe that derives `name(*)` only for all-positional calls.

F.3 supersedes that architecture.

Do not preserve List rest bindings as compatibility behavior.

## 3. Rest modes

Use one runtime/compiler enum, conceptually:

```rust
pub enum RestMode {
    None,
    Positional,
    Labeled,
    Split,
    Complete,
}
```

`Split` means the declaration has both:

```text
*positionalRest
**labeledRest
```

The parser-level parameter nodes may still mark the individual binders as `Positional`/`Labeled`; method signature metadata can normalize the pair to `Split`.

## 4. Rest layout metadata

A rest-capable method needs structured metadata independent of the encoded selector string.

Conceptually:

```rust
pub struct RestLayout {
    fixed_positionals: u8,
    fixed_labels: Box<[Symbol]>,
    mode: RestMode,

    positional_rest_param_index: Option<u16>,
    labeled_rest_param_index: Option<u16>,
    complete_rest_param_index: Option<u16>,
}
```

The exact local-slot indices can instead be derived from declaration order if simpler.

Attach to `Signature` or `MethodObject`.

Do not make runtime binding re-parse parameter source text.

## 5. Fixed-lane matching rule

Calls are already canonical packs:

```text
positionals
ordered labels
```

A declaration consumes **prefixes** of those lanes.

For:

```phalcom
method(a, b, *rest, timeout:, mode:, **extra)
```

the fixed requirements are:

```text
at least 2 positional values
labeled lane begins exactly:
    #timeout, #mode
```

`*rest` captures remaining positionals.

`**extra` captures remaining labels after the fixed labeled prefix.

Do not search/reorder labels by name.

This follows directly from Phalcom's decision that ordered labeled argument sequence is selector identity.

Therefore a call whose labels are:

```text
#debug, #timeout, #mode
```

does not match the above rest pattern merely because `#timeout` and `#mode` exist later.

## 6. Per-mode acceptance

Let:

```text
P = actual positional count
L = actual ordered label list
F = fixed positional count
K = fixed ordered label prefix
```

### 6.1 Positional rest only

Declaration has `*rest` but no `**rest`.

Accept iff:

```text
P >= F
L == K
```

Capture:

```text
positionals[F..P]
```

### 6.2 Labeled rest only

Declaration has `**rest` but no `*rest`.

Accept iff:

```text
P == F
L startsWith K
```

Capture:

```text
labels[K.len..]
```

### 6.3 Split rest

Declaration has both `*rest` and `**rest`.

Accept iff:

```text
P >= F
L startsWith K
```

Capture the two residual lanes separately.

### 6.4 Complete rest

Declaration has terminal `***rest`.

Accept iff:

```text
P >= F
L startsWith K
```

Capture both residual lanes into one Tuple.

## 7. Zero captures normalize to Unit

Examples:

```phalcom
method(*args) {
    // method()
    // args == ()
}
```

```phalcom
method(**labels) {
    // no extra labels
    // labels == ()
}
```

```phalcom
method(***remaining) {
    // no remaining values/labels
    // remaining == ()
}
```

Use A's canonical Unit.

Do not allocate empty List/Record/Tuple placeholders.

## 8. Non-empty positional capture

`*rest` binds a positional-only Tuple.

Example:

```phalcom
method(a, *rest) { ... }

method(1, 2, 3)
```

bindings:

```text
a    = 1
rest = (2, 3)
```

One residual value binds:

```phalcom
(2,)
```

not the bare value.

## 9. Non-empty labeled capture

`**rest` binds a labeled-only Tuple preserving residual label order.

Example:

```phalcom
method(timeout:, **rest) { ... }

method(timeout: 1, debug: true, trace: false)
```

bindings:

```text
timeout = 1
rest    = (debug: true, trace: false)
```

Do not convert labeled rest to Map or Record.

The capture is a pack-shaped Tuple because order and call-lane semantics must be preserved.

## 10. Complete capture

For:

```phalcom
method(a, fixed:, ***rest) { ... }
```

called with conceptual pack:

```text
positionals = [1, 2, 3]
labels      = [(#fixed, 4), (#x, 5), (#y, 6)]
```

bindings are:

```text
a     = 1
fixed = 4
rest  = (2, 3, x: 5, y: 6)
```

The fixed lane entries are removed before constructing residual capture.

## 11. Split and complete are mutually exclusive

Reject declarations containing:

```text
*rest + ***rest
**rest + ***rest
*rest + **rest + ***rest
```

`***rest` is terminal.

Split mode is exactly:

```text
*rest ... fixed labels ... **rest
```

## 12. Canonical rest selector pattern

F.1 reserves raw slot markers because literal Symbol labels with the same text are escaped.

Encode rest method selectors structurally enough for reflection/debugging.

Recommended spellings:

```text
sum(*)                         // *rest, F=0, K=[]
format(_,*)                    // fixed positional + *rest
log(_,*,format,**)             // fixed pos, *rest, fixed #format, **rest
options(_,debug,**)            // fixed positional exact, fixed #debug, **rest
forward(_,timeout,***)         // fixed lanes + complete rest
```

The precise formatter may differ, but it MUST include fixed positional slots and fixed labeled-prefix slots. Do not retain U9's lossy rule where `format(fmt,*args)` renders only `format(*)`.

Literal label `#*` is encoded through F.1 escaping and therefore cannot collide.

## 13. Temporary one-rest-pattern-per-family rule

During the core-language build, enforce:

> A class may define at most one rest-capable method for a given base selector name.

It may still define any number of exact selectors in the same family.

Example legal:

```text
foo()
foo(_)
foo(_,x)
foo(_,*,x,**)   // one rest fallback
```

Example rejected in one class:

```text
foo(*)
foo(_,*)        // second rest-capable pattern in same base family
```

This is a deliberate implementation restriction, not a claim that multiple rest overloads are mathematically impossible.

Reason: once multiple wildcard patterns overlap, Phalcom needs a separately ratified specificity/ambiguity rule. Do not silently invent one inside collections.

A future selector-dispatch spec may lift this restriction.

## 14. Inheritance behavior

Exact lookup remains ordinary:

```text
receiver class
-> superclass
-> ...
```

across the concrete selector.

Only after exact lookup fails everywhere, perform rest fallback lookup.

For rest fallback:

1. inspect receiver class's rest-family entry for the base name;
2. if present and its `RestLayout` accepts the concrete pack, dispatch it;
3. if present but does not accept, continue to superclass;
4. repeat;
5. if none accepts, dNU.

Thus a subclass can define one rest fallback for a family without deleting a superclass fallback that handles a different pack shape.

Exact inherited methods still beat subclass rest fallback because the exact pass occurs first.

This preserves U9's exact-before-variadic principle.

## 15. Rest-family index

Do not scan every method in a class on each miss.

Add a small secondary index to `ClassObject`, conceptually:

```rust
rest_methods: HashMap<Symbol /* base name */, ObjRef /* method */>
```

or an ordered equivalent consistent with class storage.

Install/update it when a rest-capable method is attached.

Because classes are closed after definition, the normal compiled path is stable.

If reflective method definition remains possible, its existing method-install API must update this index and enforce the one-rest-pattern-per-family rule too.

Do not let reflective mutation bypass the invariant.

## 16. Lookup remains selector-based

Rest matching uses only:

```text
base selector family
positional count
ordered label Symbol sequence
rest pattern metadata
```

Never inspect argument value types.

Never inspect annotations to choose the method.

This remains pure selector/pack-shape dispatch.

## 17. Dynamic and static calls share the same fallback

Factor rest fallback under the shared dispatch seam used by:

```text
Bytecode::Invoke
Bytecode::InvokePack
SuperSend where applicable
reflective send if it models an ordinary message
```

Do not implement one rest algorithm for expansion calls and leave static calls on old U9 behavior.

A static call to a rest method and a dynamically expanded call with the same concrete pack must resolve identically.

## 18. Super rest lookup

For:

```phalcom
super.foo(*args)
```

exact lookup starts above the defining class.

If exact misses, rest-family fallback must also start above the defining class, not at the receiver's dynamic class.

Reuse the same start-class rule as `SuperSend`.

## 19. Binding stack rewrite

Before entering a rest method's closure frame, the VM has a flat call window:

```text
receiver
actual positionals...
actual labeled values...
```

and the concrete call shape.

Rebuild that window into declaration-local parameter order.

For split example:

```phalcom
method(a, *rest, timeout:, **labels)
```

rebuild to:

```text
receiver
a
restTupleOrUnit
timeout
labelsTupleOrUnit
```

Then push the ordinary closure frame.

Do not make the compiled method body understand "raw call packs".

## 20. Shared rest-pack constructor

Implement a VM helper that builds capture products directly through A's Tuple finalizer semantics.

Conceptually:

```rust
finish_capture(
    positional_values: &[Value],
    labels: &[Symbol],
    labeled_values: &[Value],
) -> Value
```

Returns Unit if both lanes empty.

This helper must not stage through List or Map.

## 21. Retire U9 call-prologue List packing

Delete/replace:

```text
if signature.variadic {
    split_off(...)
    heap.alloc_list(rest)
}
```

The rest-aware binding helper owns all modes.

Update U9 tests that assert `rest.isA(List)`.

New invariant:

```text
rest is Unit or Tuple
```

depending on arity.

## 22. Signature metadata migration

Retire or deprecate:

```text
SignatureKind::Variadic(u8)
Signature.variadic: bool
```

Preferred:

```text
Signature.kind = Method(...) / rest-aware Method kind
Signature.rest = Option<RestLayout>
```

`positional_arity` may continue to report fixed positional arity for compatibility.

Do not use a single Bool to encode three lane modes.

If keeping `SignatureKind::Variadic` temporarily eases migration, it must no longer be the source of truth and must be removed before F completion.

## 23. Method installation duplicate checks

Current exact selector duplicate checking remains unchanged.

Additionally reject a second rest-family declaration in the same class/base family, even if its encoded rest selector differs.

Diagnostic should name:

```text
class
base family
first rest selector
second rest selector
```

and point at the second declaration.

## 24. Blocks/closures

The old U9 unit explicitly left block-literal variadics deferred because block parameters use a separate parser/runtime path.

F.3 SHOULD close the positional block-rest gap if actual HEAD can do so without inventing labeled block-parameter syntax:

```phalcom
{ *args => ... }
```

or the project's current block-parameter delimiter equivalent.

Required if implemented:

- capture is Tuple/Unit, not List;
- `Callable` stores a small optional positional-rest layout;
- both the flat bytecode `Block#call` entry path and the native/re-entrant reflective block-call path use one shared argument repacker;
- `Block#arity` continues to report the fixed/minimum arity.

Do **not** invent `**`/`***` block parameter syntax if no such labeled block domain has been ratified.

If block rest remains deferred, record it explicitly as a callable-syntax follow-up; method rest capture in this spec is still mandatory.

## 25. Native rest methods

Do not silently change primitive ABI expectations.

Unless a kernel primitive explicitly needs rest binding in F, reject/forbid a native method declaration carrying a rest layout.

All F rest fixtures should use bytecode methods.

A future primitive-ABI unit can define whether a native rest method receives:

```text
raw actual args
or already-packed captures
```

## 26. Arity/reflection compatibility

Until the separate reflection/type spec lands:

```text
Method#arity
```

should continue to mean the fixed/minimum positional arity for a rest method, matching existing U9 behavior.

Do not add `ArgumentPackType`, `restMode`, or interpreted pack-type reflection in F.

The encoded selector string may now show the rest pattern more accurately.

## 27. dNU

If no exact or rest fallback accepts the call, dNU receives the original **concrete** selector and original flat argument values.

Do not forward the rest method's wildcard selector.

This preserves "what was sent" rather than "what fallback was attempted".

## 28. Evaluation is complete before binding

Rest binding runs only after outgoing pack assembly has completed.

Therefore capture construction has no user callbacks and cannot alter argument evaluation order.

It only rearranges already-evaluated `Value`s into fixed slots and Tuple captures.

## 29. Error behavior

A rest family entry that exists but does not accept a call is a lookup miss, not an arity exception by itself.

Continue superclass rest lookup, then dNU.

This mirrors exact selector dispatch: shape mismatch means selector-pattern mismatch.

Only direct reflective invocation of a specific Method object may need an explicit arity/shape error; if that path is updated in F, use existing `RuntimeError::Arity`/ArgumentError conventions.

## 30. Tests

### 30.1 Capture values

- zero positional rest -> Unit;
- one positional rest -> singleton Tuple;
- many positional rest -> Tuple;
- zero labeled rest -> Unit;
- labeled rest preserves order;
- zero complete rest -> Unit;
- complete rest preserves both lanes.

### 30.2 Matching

- positional rest minimum F;
- positional rest rejects unexpected labels;
- labeled rest requires exact positional count;
- labeled rest matches fixed label prefix + extras;
- split captures both;
- complete captures both;
- wrong fixed label order does not match.

### 30.3 Dispatch precedence

- exact same-class selector beats rest fallback;
- inherited exact selector beats subclass rest fallback;
- subclass accepting rest fallback beats superclass rest fallback;
- subclass non-accepting rest fallback allows superclass fallback;
- no fallback -> ordinary dNU;
- dynamic expanded call resolves same method as equivalent static concrete call.

### 30.4 Declaration diagnostics

- two rest methods same base/class rejected;
- split + complete rejected;
- parameter after `**rest` rejected;
- parameter after `***rest` rejected.

### 30.5 U9 migration

- old rest binding no longer List;
- current `sum(*numbers)` behavior still computes same result after changing capture container;
- fixed+rest works;
- exact fixed overload still wins.

### 30.6 Reflection

- rest method `arity` reports fixed/minimum positional arity;
- selector text includes fixed slots and unambiguous rest marker;
- literal label `#*` method is distinct.

## 31. Completion checklist

F.3 is complete when:

- lane-aware rest metadata replaces `is_rest`/Bool semantics;
- `*`, `**`, `***` bind residual lanes correctly;
- all captures are Tuple/Unit;
- fixed labeled parameters match ordered prefix, never name-reordered;
- exact lookup precedes rest lookup;
- one rest pattern per base/class is enforced across normal and reflective installation;
- inheritance rest fallback is deterministic;
- static and dynamic sends share the same rest lookup;
- super rest lookup starts at the correct class;
- old U9 List packing is removed;
- dNU receives the original concrete message;
- no type-based dispatch was introduced;
- public primitive floor remains unchanged.
