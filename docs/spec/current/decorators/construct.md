# `@construct` — derive constructor methods from class fields

- Status: **Canonical design; implementation currently uses legacy internals**
- Governing decision: [PDR-0028](../../../pdr/0028-class-and-constructor-decorator-canon.md)
- Related: [`@constructor`](constructor.md) · [Classes](../classes.md) · [pending notes](../../../work/pending/ctor/notes/construct-derive.md)

## What it does

`@construct` is legal on class declarations only. It derives an ordinary
`@constructor` method from the class's declared fields, in declaration order.

```phalcom
@construct
class Point {
  _x
  _y
}
```

The derived constructor uses each field's name with one leading underscore
removed as its parameter name and label (`_x` becomes `x`). Defaulted fields keep
the field declaration's default timing and ordering.

The derived member has `@constructor` semantics. `construct` is not generated
source syntax; it is a retired declaration keyword retained only for migration
recognition.

## Legal targets

| Target | Result |
|---|---|
| class declaration | derive constructor from fields |
| method | error: use `@constructor` |
| field | error: use `@class` for class-side storage, or no placement decorator |

An explicitly written constructor with the same selector collides with the
derived constructor. Different selectors coexist.

Inheritance and super-constructor chaining remain pending implementation detail;
see the notes under `docs/work/pending/ctor/notes/`.
