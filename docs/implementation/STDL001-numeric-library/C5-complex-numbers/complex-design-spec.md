# Complex numbers: approved design

The user approved D1 (generic capability-based scope) and D2 (principal-branch
binary64 math) and D3 (source kernels, precise cuts, log1p). Accepted behavior
will be recorded in the owning `docs/specs/` document; this companion preserves
design rationale and the approval boundary.

## Type and ownership: D1 approved

`universe.numbers` exports `Complex<T>`, an ordinary nominal, transparent,
shallowly immutable data family with `real` and `imag` components. The primary
constructor and a convenience `new` factory perform explicit rectangular
construction. No imaginary literal syntax or numeric promotion lattice is added.

Arithmetic asks for small nominal capabilities on T. Int and Float conform
through existing scalar behavior. A third-party component can implement these
same capabilities. Component types remain visible in result types.

Int components support exact addition, subtraction, multiplication,
conjugation, and squared norm under the existing integer resource policy.
Float components additionally support division, magnitude, phase, polar
construction, powers, and elementary functions. Complex has no numerical order.

`universe.math.complex` exports captured method Families, following the existing
Float math facade. The data owner contains inherent behavior; the math backend
contains numerical algorithms. Dependency direction must avoid a cycle between
the type owner and its elementary-function consumer.

## Elementary semantics: D2 approved

The function surface includes `sqrt`, `exp`, natural `log`, `log1p`, powers,
`sin`, `cos`, `tan`, `asin`, `acos`, `atan`, `sinh`, `cosh`, `tanh`, `asinh`,
`acosh`, and `atanh`. The proposal lists `log1p`; it was accidentally omitted
from the earlier conversation's enumerated summary.

Integer powers use exponentiation by squaring. Complex powers use
`exp(w * log(z))` on the selected principal branch, with endpoint behavior
specified before the corresponding implementation slice.

D7 specifies separate integer `**`/Float `powInt` and complex `pow` operations.
Zero exponent is one even for zero base. Negative Int-component powers raise
ArgumentError; negative Float integer powers use reciprocal division. Complex
zero base with positive real exponent yields zero, negative real exponent yields
+infinity+0i, and nonzero imaginary or NaN exponent yields NaN components.

Special values use component IEEE results. NaN payload/sign bits are outside
the source contract. Indeterminate arithmetic produces NaN components rather
than a new exception family. Transcendental final bits may vary with the
supported binary64 backend. Algorithms must avoid unnecessary intermediate
overflow, underflow, and cancellation.

`math.sqrt(-1.0)` remains Float NaN. Complex functions accept explicit complex
inputs and return Complex<Float>, including when the imaginary result is zero.

## Backend and exact cuts: D3 approved

Use source-authored kernels above existing Float math. Adapt the scaled finite
algorithms and special-value conventions of CPython's `cmath` implementation,
retaining the applicable license and recording the adaptation. Phalcom's facade
returns IEEE components at poles/domain edges; it does not adopt Python's
ValueError/OverflowError wrappers or Python's implicit argument conversions.

The source reference is CPython v3.13.9 `Modules/cmathmodule.c`, commit
`8183fa5e3f78ca6ab862de7fb8b14f3d929421e0`, retrieved on 2026-10-08.
Retain the corresponding copyright/license notices for adapted code.

| Function | Principal branch cut |
| --- | --- |
| sqrt, log | Negative real axis |
| log1p | Real axis at and left of -1 |
| asin, acos, atanh | Real axis outside [-1,1] |
| atan, asinh | Imaginary axis outside [-i,i] |
| acosh | Real axis at and left of 1 |
| exp, sin, cos, sinh, cosh | No branch cut |
| tan, tanh | Meromorphic poles, no logarithmic branch cut |
| complex pow | Inherits the base logarithm's cut |

Signed zero selects the approached side: imaginary zero signs on real-axis
cuts, real zero signs on imaginary-axis cuts. Phase uses atan2(imag,real),
including both signed boundary values -pi and +pi. The earlier description
`(-pi,pi]` was inconsistent with its explicitly approved -pi example.

Required exact examples:

```text
sqrt(-1 + +0i) = +0 + 1i
sqrt(-1 + -0i) = +0 - 1i
log(-1 + +0i) = +0 + pi*i
log(-1 + -0i) = +0 - pi*i
```

Scaled square root, logarithm, division, and hyperbolic/exponential products
must handle representable results whose naive intermediates overflow.
Stable `log1p` must retain small real corrections and signed imaginary zeros.

## Verification design

Test value semantics and formal availability at their owning layers. Runtime
tests cover exact Int arithmetic, Float data identity, component equality/hash,
construction, facade imports, captured Families, and GC survival. Numerical
fixtures use independent high-precision reference values rather than production
formulas as an oracle. Branch sides and IEEE cases have explicit component
classification/sign assertions. Include subnormals, large finite inputs,
near-unit logarithms, near-zero log1p, inverse branch points, and poles.

Checks run serially with the pinned toolchain and cleared local Rust flags.
Broader workspace/release verification requires an explicit gate.

## References

- `docs/math-proposals/01-executive-recommendation.md`
- `docs/math-proposals/06-complex-numbers.md`
- `docs/math-proposals/07-generic-abstractions-and-math-api.md`
- `docs/math-proposals/08-package-architecture.md`
- `docs/specs/objects/data-objects.md`
- `docs/specs/objects/impl.md`
- `docs/specs/objects/traits.md`
- `docs/specs/universe.md`
- https://docs.python.org/3.13/library/cmath.html
- https://github.com/python/cpython/blob/3.13/Modules/cmathmodule.c
