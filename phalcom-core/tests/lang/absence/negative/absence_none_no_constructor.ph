// area: absence
// spec: values-and-absence.md; ADR-0007
// status: NEGATIVE
// Ported from Wren `test/core/null/no_constructor.wren`: `None` is the
// shared blessed singleton (ADR-0007), not an instantiable class — sending
// it `new()` is a plain does-not-understand, not a special-cased
// diagnostic (Phalcom's `None`'s class has no `new()` method at all, unlike
// Wren's `Null metaclass does not implement 'new()'` message).

None.new()
