# `measure`

`measure` is a standalone Telos project exploring exact units, normalized
quantities, structural dimension algebra, affine points, and nominal semantic
quantity kinds using ordinary Telos source code.

## Examples

```telos
from measure.standard.si.base import h, km
from measure.standard.si.mechanics import kmph

let distance = kmph(90) * h(2)
System.print(distance.in(km)) // 180
```

```telos
from measure.standard.si.temperature import degC, deltaC

let target = degC(21)
let lower = target.retreated(by: deltaC(1))
```

## Design laws

- units compose structurally through ordinary message sends;
- quantities eliminate source units after normalization;
- semantic kinds remain nominal even when dimensions are equal;
- dimensions never infer a unique semantic kind;
- exact unit algebra is Rational-backed and never routes through Float;
- affine units are excluded from multiplicative unit algebra;
- no global unit registry or conversion graph exists;
- third-party packages extend the model through ordinary traits/conformances.

## Current verification status

The package source is substantially complete, including the standard catalog,
tests, examples, extension projects, and benchmarks. It has **not** been run
through the Telos compiler in this artifact environment. `VERIFICATION.md`
therefore distinguishes implemented source from compiler/runtime-verified
behavior.
