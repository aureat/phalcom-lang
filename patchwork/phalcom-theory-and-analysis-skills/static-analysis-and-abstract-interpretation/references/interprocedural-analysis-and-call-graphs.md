# Interprocedural Analysis and Call Graphs

## Why summaries

Re-analyzing callee bodies at every call site is expensive, recursion-prone and hard to invalidate. Summaries compactly describe behavior relevant to callers.

Example summary components:

```text
parameter abstract facts
return fact
may throw/yield/dynamic-send
read/write sets
captured block invocation positions
call dependencies
proof/type contracts
revision/provenance
```

## Call graph

Nodes are callable semantic IDs. Edges represent possible calls.

Dynamic dispatch means a call site may have multiple targets or unknown target. Call-graph construction and value analysis can be mutually dependent.

## Context sensitivity

Context-insensitive: one summary per callable.

Call-site sensitive: summary per recent call site(s).

Object-sensitive: context includes receiver allocation/class abstraction.

Generic/type-context sensitive: summary parameterized by type arguments.

More context improves precision and costs memory/time. Start coarse, add only where evidence demands.

## Recursive SCC solve

Compute SCCs over known call graph and iterate summaries within SCC until fixed point/widening. Unknown/dynamic edges require conservative effects.

## Higher-order blocks

A method receiving a block may invoke it zero/one/many times, synchronously/later. Summary can record `invokes_parameter(i)` plus cardinality/timing/effect details as analysis evolves.

The current Phalcom semantic engine already records invoked block parameter positions; preserve and generalize rather than duplicate.

## Virtual dispatch

Receiver abstract value narrows target set. If receiver can be classes A|B, resolve selector on both and join return/effects. Missing selector on one alternative matters differently for advisory completion versus correctness checking.

## Native summaries

Core/Rust primitives need explicit semantic summaries. If absent, use conservative unknown effects rather than source-code assumptions.

## Incrementality

Maintain reverse dependency edges from callee summary to callers. A changed summary should invalidate dependents, while unchanged internals need not necessarily rebuild the world.
