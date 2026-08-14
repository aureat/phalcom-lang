# Family

[Callables](README.md) · [Dispatch and lowering](dispatch.md) · [Arguments and rest](arguments.md) · [Runtime and activation](runtime.md) · [Reflection](reflection.md) · [Function](function.md) · [Method](method.md)

Family is a sealed Function created by a bound method-reference expression.
It stores one receiver and one selector specification. It is not a Method:
construction never resolves or authorizes a target implementation.

## 1. Selector specifications

    object::name
    object::name()
    object::#name(_,to)
    object::name(...)
    object::#name(...,to)

The bare form object::name is an exact getter selector. The parenthesized
object::name() is an exact nullary method selector. A hash-prefixed selector
specifies its exact kind and slots. An ellipsis creates a structural pattern
with fixed prefix/suffix slots.

The old Open/Pinned MethodRefKind split, pinned-only hash syntax, and
reference-time empty-family rejection are retired. There are no unbound
Type::name forms. The receiver expression is evaluated once and retained.

## 2. Exact Family calls

Exact Families retain selector identity and derive no replacement selector
from the call's labels. The incoming argument shape must satisfy that exact
selector. The runtime then looks up that selector on the stored receiver and
activates the selected Method using ordinary access and rest rules.

    let family = object::#render(_)
    family(value)

Replacing render(_) after Family construction is visible to the next call.
The Family did not capture a Method; it captured selector identity.

## 3. Pattern Family calls

Pattern Families capture an immutable MethodFamily snapshot at construction:

    exact_methods: effective exact selector -> Method
    rest_candidates: captured compatible rest Methods in hierarchy order

Capture walks the receiver behavior hierarchy once. It applies the initiating
caller's access authority and omits inaccessible matching routes. It does not
invoke the receiver, probe a selector, allocate a Message, or call
doesNotUnderstand. A later method-table mutation does not change the snapshot.

At call time the incoming shape is matched against the captured routes. Named
AnyNamed patterns consider getter, setter, and method forms; exact getter and
setter forms retain their own zero- and one-value shapes. Subscript patterns
retain their index labels and assigned-value lane. The chosen captured Method
is activated exactly on the stored receiver.

    let family = object::render(...)
    family(value)

Adding or replacing matching methods after the second line does not change the
captured route set. A call with no captured route reaches ordinary
doesNotUnderstand at the target call boundary.

## 4. Function and reflection surface

Family participates in the shared Function call gateway. MethodFamily is the
immutable reflection payload for a pattern Family and exposes:

    family.selectors
    family.size
    family.methodFor(selector)
    family.bind(receiver)

selectors returns a fresh List of canonical captured selectors. size counts
exact and rest routes. methodFor returns only a captured route and applies the
current caller's access authority. bind stores the snapshot plus a receiver;
it does not inspect receiver behavior or re-capture routes.

## 5. Mutation and dispatch law

    Family construction
        -> immutable receiver + selector specification
        -> exact lookup or captured-route selection at call
        -> exact Method activation

The stored receiver affects only target layout and dynamic self inside the
selected body. It never changes a pattern's route set. Ordinary sends inside
the body remain dynamically dispatched on that receiver. Lexical super keeps
the selected Method's defining holder.

## 6. Implementation boundary

The compiler emits MakeFamily for general references. A narrow
semantics-preserving specialization may lower an immediately-called exact
MethodRef whose static call shape is identical to an ordinary direct send.
Escaping references, structural patterns, mismatched shapes, and dynamic packs
retain Family construction and the shared runtime gateway.

See vm/send.rs, heap/object.rs, primitive/method_family.rs, and
compiler/lib/expr.rs for the implementation paths.

## 7. Related chapters

- Function — shared call gateway
- Dispatch and lowering — selector identity and target sends
- Method — exact behavior and arbitrary receiver guards
- Reflection — MethodFamily and exact invocation
- Runtime and activation — direct routing implementation
