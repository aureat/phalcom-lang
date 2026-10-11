# Verification status

## Implemented source

- canonical exact `Rational`;
- persistent immutable Dimension algebra;
- exact `UnitExpr` composition/cancellation;
- callable `LinearUnit<K>` and `Prefix`;
- exact and approximate unit observation capabilities;
- unit-eliminating `Quantity<K,N>`;
- explicit scalar scaling without selector overloading;
- semantic `ProductKind` / `QuotientKind` with runtime dimension-law checks;
- affine `Point` and `AffineUnit`;
- unit-retaining `Measure` / `PointMeasure`;
- complete SI decimal prefixes;
- seven SI base dimensions and representative base units;
- geometry, mechanics, electromagnetism, photometry, radiological, chemistry,
  angle, information, accepted non-SI units;
- runtime tests, compile pass/fail probes, examples, extension projects,
  benchmarks.

## Not verified in this environment

No local Telos compiler binary/repository checkout is available in the artifact
runtime, so syntax/type/runtime execution has not been asserted here.

The highest-risk current-language probes are:

1. associated-output construction in `Quantity.*` and `Quantity./`;
2. `where N == Rational` / `where N == Float` specialization on observation;
3. generic `ExactUnitFor<K>` / `FloatUnitFor<K>` evidence;
4. associated `PointKind::Difference` inside Point methods;
5. runtime generic reification across the custom-units dependency boundary;
6. `const` data materialization followed by ordinary `call` dispatch.

A failure in these valid language-model probes should be treated as a compiler
incident before weakening the library to Dynamic.

## Static consistency checks performed in the artifact environment

- all relative source imports resolve to an existing module/package path;
- no TODO/placeholder source modules remain;
- project/test manifests follow current repository package conventions;
- the final archive is rooted at `packages/measure/` for direct repository
  extraction.
