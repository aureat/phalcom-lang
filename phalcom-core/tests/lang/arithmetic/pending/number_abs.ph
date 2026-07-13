// area: arithmetic/operators
// spec: values-and-absence.md
// status: PENDING
// ported from wren/test/core/number/abs.wren: `Number::abs()` is not on the
// floor yet (see phalcom-core/src/primitive/number.rs — only `+ - * / % < <=
// > >= negated hash toString` are implemented). Pins the intended surface.
System.print(0.abs())
System.print(12.abs())
System.print((-12).abs())
