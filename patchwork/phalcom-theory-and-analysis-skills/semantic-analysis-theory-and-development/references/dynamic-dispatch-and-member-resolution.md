# Dynamic Dispatch and Member Resolution

## Static analyzer objective

The analyzer approximates which ordinary runtime dispatch target(s) a send may reach without changing dispatch semantics.

Inputs:

```text
receiver semantic/value/type fact
canonical selector
class/dispatch side
lexical access context
class/member surfaces
open-world revision assumptions
```

## Resolution levels

Distinguish:

- exact receiver class -> exact ordinary lookup target under current class graph;
- bounded receiver alternatives -> candidate set;
- protocol/type guarantee -> selector contract without exact implementation;
- dynamic/unknown receiver -> unresolved/dynamic result;
- super send -> lexical lookup start with current receiver;
- reflective method object -> implementation identity, not ordinary lookup.

## Access control

Private/protected/internal checks depend on lexical access class/context, not only receiver type. Completion can hide inaccessible members, but semantic resolution should retain enough metadata for diagnostics.

## Inheritance and metaclasses

Instance sends walk class superclass chain. Class-side sends use metaclass/parallel hierarchy according to current object model. Never approximate class-side methods by scanning ordinary class fields alone.

## Dynamic method mutation

If method surfaces can change reflectively, exact target fact is generation-dependent. Runtime inline caches and static semantic caches both need invalidation/version assumptions.

## Protocol conformance

Protocol requirement lookup describes capabilities, not ordinary target selection. A protocol-typed receiver can verify selector availability while runtime still chooses implementation by receiver class.

## Families

A receiver-side selector family can retain receiver fact and base selector until call labels determine concrete selector. Static packs/dynamic labels affect whether exact selector can be resolved.

## Diagnostics

Different failures:

```text
selector definitely absent
selector absent on one union alternative
selector exists but inaccessible
selector shape dynamic/unknown
receiver dynamic
ambiguous module/class identity
```

Do not render all as "method not found."
