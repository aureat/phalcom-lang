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

Pattern Families retain an immutable structural predicate and the receiver:

    receiver: bound target
    pattern: SelectorPattern

The predicate is created at reference construction, but method lookup remains
live. At each call the incoming shape is normalized, matched against the
predicate, and dispatched through the current method table on the stored
receiver. Construction does not invoke the receiver, probe a selector,
allocate a Message, or call `doesNotUnderstand`.

Named AnyNamed patterns consider getter, setter, and method forms; exact getter
and setter forms retain their own zero- and one-value shapes. Subscript
patterns retain their index labels and assigned-value lane. Replacing or
adding a matching method after Family construction is visible to the next
call.

    let family = object::render(...)
    family(value)

Adding or replacing matching methods after the second line changes the target
selected by the next call. A call with no matching route reaches ordinary
`doesNotUnderstand` at the target call boundary.

## 4. Function and reflection surface

Family participates in the shared Function call gateway. MethodFamily is a
separate immutable reflection payload returned by `Behavior#>>` for a pattern
and exposes:

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
        -> exact lookup or live pattern selection at call
        -> exact Method activation

The structural predicate remains immutable. The stored receiver determines the
current method table, target layout, and dynamic self inside the selected body,
so method-table changes can change the selected route. Ordinary sends inside
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
