# Objects, Classes, Metaclasses, and Message Dispatch

This is the central dynamic-semantic chapter for Phalcom. The object model and selector model define ordinary computation; typing and tooling must describe this behavior rather than inventing another dispatch system.

## 1. Core object relations

```text
classOf(value) -> ClassId
superclass(ClassId) -> Option<ClassId>
```

Every surface value has a class. Every class object is itself a value with a metaclass:

```text
classOf(classObject(C)) = metaclassOf(C)
```

Instance-side and class-side dispatch therefore use the same lookup concept on different receiver classes.

## 2. Selector identity

A selector is not merely base text `foo`. It encodes the message call shape according to Phalcom selector rules: arity, positional slots, labels, accessor/index kind, and other ratified structure.

Critical rule:

```text
type annotations ∉ selector identity
```

Two methods cannot become distinct ordinary dispatch targets merely because parameter type metadata differs unless Phalcom explicitly introduces another dispatch mechanism.

## 3. Dispatch inputs

A complete send resolution depends on at least:

```text
receiver value
canonical selector
lookup start class
dispatch side
lexical/access context
current lexical class for super
current method-table/hierarchy state when reflection can mutate it
```

Caching may add versions/epochs, but those are implementation details.

## 4. Ordinary instance send

For:

```phalcom
obj.move(dx, to: destination)
```

semantic stages:

```text
vr = eval(obj)
vs = eval arguments in lexical order
s  = canonical selector
C0 = classOf(vr)
m  = lookup(C0, s, accessContext)
if m exists: invoke(m, receiver=vr, args=vs)
else: execute message-miss semantics
```

Lookup walks `C0`, then superclass chain.

## 5. Lookup rule

Conceptually:

```text
lookup(C, s, access):
    if methodTable(C) contains s:
        m = methodTable(C)[s]
        if permitted(access, m): return Found(m)
        else return AccessDenied(m)
    if superclass(C) = P:
        return lookup(P, s, access)
    return Miss
```

Whether access denial stops lookup or behaves differently is a language decision. Encode ratified policy; do not let cache code choose accidentally.

## 6. Invocation

Invoking `m` binds:

```text
self = original receiver
parameters = associated argument values
lexical class/access authority = defining context required by semantics
return target = current invocation frame
```

The holder of `m` need not equal `self.class` because the method may be inherited.

## 7. `super`

`super` is a lexical dispatch modifier.

If method is defined in class `C` and executing on receiver `r`:

```text
receiver         remains r
lookupStart      = superclass(C)
selector         unchanged
argument order   unchanged
```

Thus `self.class` may be `Subclass`, selected implementation may come from `Superclass`, and inside it `self` remains the subclass instance.

Never model `super` as loading a superclass object.

## 8. Class-side dispatch

Sending to class object `C` uses:

```text
receiver = classObject(C)
lookupStart = classOf(receiver) = metaclassOf(C)
```

The metaclass hierarchy parallels class inheritance according to Phalcom object-model rules. This keeps constructors and class methods inside ordinary object semantics.

## 9. Constructors

Constructor syntax may have special declaration/lowering rules, but semantics must state:

- receiver/class object;
- allocation timing;
- field-default/initializer timing;
- meaning of `self`;
- behavior if initialization throws;
- whether constructor result is forced to allocated instance;
- reflective identity of constructor method.

Do not infer these from current VM sequence.

## 10. Getter, setter, and subscript dispatch

These may have different selector encodings while sharing ordinary send semantics. For setter-like source:

```phalcom
obj.name = value
```

if it is a send, specify:

```text
evaluate obj
evaluate value
construct setter selector
lookup/invoke
```

Direct field syntax is a separate semantic operation.

## 11. Message miss / `doesNotUnderstand`

A miss should reify enough information for fallback:

```text
Message(receiver?, selector, argument pack, source metadata?)
```

Settle:

- whether receiver is stored;
- whether arguments are copied/retained;
- selector canonicalization;
- inheritance/override of dNU;
- access-denied behavior;
- redispatch through `perform`;
- diagnostic/source metadata.

## 12. Dynamic `perform`

```text
perform(receiver, selectorValue, pack)
```

may delay selector construction/validation until runtime, but once formed it should use ordinary lookup/access rules unless `perform` is explicitly privileged.

## 13. Method reflection

Distinguish:

```text
physical method exists
method is visible/enumerable
method is accessible from context
method can be invoked reflectively
```

Enumeration is not authority. Reifying a private method must not automatically grant permission to invoke it if policy says otherwise.

## 14. Closed source classes versus reflective mutation

Phalcom source classes are closed under current design, but reflective APIs may still mutate method tables. Distinguish:

- source-level reopening;
- reflective install/replace;
- whether existing instances observe changes immediately;
- cache invalidation/versioning;
- method identity after replacement.

## 15. Inline caches

An inline cache is semantically invisible only if it selects exactly what ordinary lookup would under same dispatch state/access context.

Cache key may require:

```text
receiver class
selector
method-table/hierarchy version
access-relevant context
```

A stale cache surviving reflective mutation is a semantic bug, not merely a performance bug.

## 16. Inlining and sacred selectors

Inlining a send is sound only under sufficient guards.

For:

```phalcom
x + y
```

syntax alone does not prove primitive integer addition because `+` is a selector. A fast path generally needs proven/checked receiver representation/class plus fallback/deoptimization or a semantic guarantee that override cannot apply.

Boolean-control inlining must preserve lazy block invocation and errors.

## 17. Families and method families

Preserve Phalcom's distinction:

```text
Family
  receiver-bound delayed dispatch
  concrete target selected later

MethodFamily
  captured concrete implementations
  implementation-side reflection
```

Do not collapse them into one callable model in semantic analysis.

## 18. Type-directed multimethods

A future `@typecase`/`@multimethod` mechanism must be explicitly layered above ordinary selector lookup. It must not silently mutate selector identity.

One possible layering:

```text
ordinary selector lookup selects dispatcher method
inside dispatcher semantics, runtime/type-pattern selection chooses branch
```

Other designs are possible, but the base dispatch relation stays explicit.

## 19. Runtime class, type, and shape

Keep separate:

```text
x.class == C              exact runtime class identity
x.isA(C)                  runtime inheritance/instance relation
T <: U                    static subtype relation
shape(x)=Instance(C)      analyzer fact
```

None implies all others without specified bridges.

## 20. Access control

Private/protected/internal access should be defined by lexical/source authority and inheritance, not simply caller stack position. This keeps direct send, cached send, `perform`, `methodFor`, bound method, and reflective invocation consistent.

## 21. Worked dispatch example

Assume:

```phalcom
class A {
  f() { return "A:" + self.g() }
  g() { return "A.g" }
}

class B is A {
  g() { return "B.g" }
  h() { return super.f() }
}
```

For `B.new().h()`:

1. receiver `b` has `classOf(b)=B`;
2. `h()` lookup begins at `B`, selecting `B>>h`;
3. `super.f()` keeps receiver `b`, but starts lookup at superclass of lexical class `B`, namely `A`;
4. selects `A>>f`, invoked with `self=b`;
5. inside `A>>f`, `self.g()` is ordinary send, lookup starts at runtime class `B`;
6. selects `B>>g`.

This is the canonical test for lexical `super` versus dynamic `self`.

## 22. Static-analysis correspondence

An analyzer should produce candidates consistent with dynamic relation:

```text
exact receiver class -> exact/inherited target candidate
union of classes      -> union of possible targets
unknown/open receiver -> conservative dynamic/unknown result
```

A checker may reject incompatible calls, but cannot claim a different runtime selector target unless typed dispatch semantics explicitly says so.

## 23. Conformance fixtures

Maintain tests for inherited send, override, lexical `super`, class-side metaclass inheritance, getters/setters/subscripts, direct/reflective visibility, dNU override, reflective replacement/cache invalidation, `perform` parity, and delayed Family dispatch.

## 24. Common failures

- keying method identity only by base name;
- class-side members in disconnected static namespace;
- changing receiver for `super`;
- reflection bypassing visibility;
- type annotations entering selector keys;
- cache lacking mutation invalidation;
- method enumeration treated as invocation permission.

## 25. Competency checks

1. Explain why `super.f()` can cause overridden `self.g()` to run.
2. What exact semantic input changes between ordinary send and `super` send?
3. Why must `perform` use same access policy as ordinary dispatch?
4. Which observations make stale inline caches semantically incorrect?
5. How would future type-directed dispatch coexist with selector identity?
