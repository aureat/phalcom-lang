# U-NUMBERS-01 — value model and class allocation

## Outcome

Replace the flat numeric representation with `Value::Int(i64)`, `Value::Float(f64)`, and heap
`Object::LargeInt(BigInt)`. Install `Number < Object`, `Int < Number`, and `Float < Number`.
`Number` is allocator-abstract; its type relationship remains normal.

## Write set

- `phalcom-core/src/value/{mod.rs,render.rs}`: Value arms, display/type/class/equality hooks.
- `phalcom-core/src/heap/object.rs` and heap tracing: `LargeInt` storage and mark path.
- `phalcom-core/src/universe/{mod.rs,core_classes.rs,primitives.rs}` and VM bootstrap: class rows,
  core handles, allocator metadata/check.
- `phalcom-core/core/core.ph`: class declarations and derivable Int protocol only.
- all compiler-reported exhaustive `Value` matches; `phalcom-core/tests/invariants.rs` class rows.

## Steps

1. Add `Int`, `Float`, and `LargeInt`; define one `normalize(BigInt) -> Value` that returns small
   Int when representable and otherwise allocates `LargeInt`. No alternate normalization sites.
2. Teach every type/class/render/equality match about all three arms before changing semantics.
   An unhandled match is a correctness failure, not a wildcard opportunity.
3. Add `Int` and `Float` core class handles and bootstrap rows. Keep `Number` real and empty of
   numeric primitives. Confirm `1.class == Int`, `1.0.class == Float`, and both are `isA(Number)`.
4. Add an `abstract` bit (or equivalent non-overridable allocator metadata) to class layout.
   The common allocation entry point rejects abstract classes before instance construction. Mark
   `Number`; do not override a `new` selector as a substitute. Reflective dispatch must reach the
   same check.
5. Allocate and trace `LargeInt` as an owned heap object. Verify no raw `BigInt` reference crosses
   a collection, native re-entry, or GC boundary.

## Acceptance matrix

- `Number.new()`, `Number.new(1)`, and reflective `Number.send(#new, ...)` each raise
  `#abstractClass`; `Number.respondsTo(#new)` keeps ordinary lookup behavior.
- `Int.new()` / `Float.new()` remain constructible; `Int.is(Number)` and `Float.is(Number)`.
- `i64::MIN.negated` normalizes to `LargeInt`; a traced `LargeInt` survives forced collection.
- Existing nonnumeric values preserve type/class, hash, render, and equality behavior.

Do not implement literals, arithmetic, or constructor parsing here beyond minimal bootstrap.
