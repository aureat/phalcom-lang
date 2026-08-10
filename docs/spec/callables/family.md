# Family

[`Family`](README.md) is a sealed/final [`Function`](function.md) produced by a
bound `::` method-family reference. It carries the bound receiver and reference
context, so it needs only explicit call arguments to proceed.

The two reference forms are specified conceptually as:

```phalcom
obj::name
obj::#selector(...)
```

The first is an open Family. The second is a pinned Family.

## Open and pinned references

An open Family derives its target selector at call time from the family base
name and the actual call shape. Its selector choice therefore participates in
the existing method-family lookup rules.

A pinned Family retains its pinned selector as authoritative according to the
existing method-reference semantics. Call arguments do not replace that
selector identity.

After routing, target dispatch is an ordinary Phalcom message send. Family
callability is part of the common Function activation gateway; it must not be
specified through intentional `doesNotUnderstand` misses.

## Arguments

Phalcom uses one rest/spread notation everywhere:

```text
*      positional rest/spread
**     labeled rest/spread
***    complete rest/spread
```

Examples:

```phalcom
family(*values)
family(**labels)
family(***arguments)
```

The spelling `args...` is never rest/spread syntax in Phalcom. `...` is not a
spread operator.

Family calls enter through `call(***arguments)`. The Family preserves the
complete argument shape while routing; the selected target then applies its
own parameter acceptance. Family creation does not require a successful
intentional dNU probe.

## Related callable types

- [`Callable model`](README.md) — hierarchy and common execution rules
- [`Function`](function.md) — gateway used by Family calls
- [`Method`](method.md) — target behavior selected after routing
