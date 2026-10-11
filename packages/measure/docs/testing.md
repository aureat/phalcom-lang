# Testing strategy

`measure` is both a library and a Telos conformance experiment. Verification is
therefore split into four layers.

1. **Runtime tests** exercise Rational, Dimension, unit composition, semantic
   quantity algebra, affine points, const Prefix behavior, and standard units.
2. **Compile-pass probes** stress associated outputs, exact unit evidence,
   affine associated differences, and const materialization.
3. **Compile-fail probes** assert semantic separation: incompatible kinds,
   affine-unit multiplication, wrong-unit observation, and unsupported semantic
   products must fail before runtime.
4. **Cross-project fixtures** define third-party kinds/conformances in one
   Project and consume them from another, exercising stable conformance metadata
   and runtime generic reification.

`scripts/verify.sh` runs the executable projects and checks that fail fixtures do
not compile when a `phalcom` binary is available.
