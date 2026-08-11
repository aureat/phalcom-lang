# LSP fixtures

Markers are `/*@name*/`. `tests/support/fixture.rs` removes them before sending source to Phalcom and records their UTF-16 cursor positions.

Prefer complete source such as:

```phalcom
person./*@completion*/greet()
```

Only `incomplete/` is intentionally malformed.

`workspace/` paths are semantically significant; do not flatten that directory.
