# Remaining implementation/verification gates

## G0 — compile the project
Run the real Telos CLI against `packages/measure` and repair syntax only where
current repository grammar requires it.

## G1 — Rational runtime
Run exact arithmetic and arbitrary-precision tests.

## G2 — Dimension runtime
Verify cancellation, powers, order-independent equivalence, persistence, and
absence of mutable backing state.

## G3 — generic unit observation
Verify `Quantity<Length,Rational>.in(LinearUnit<Length>)` and reject wrong-kind
units statically.

## G4 — associated-output semantic algebra
Verify `mps(100) * h(4)` reifies exactly as `Quantity<Length,Rational>` and no
`Dynamic` enters runtime metadata.

## G5 — affine associated difference
Verify point subtraction and named translation methods.

## G6 — cross-project evidence
Compile `tests/projects/custom-units-consumer` and inspect exact generic type
identity across the dependency boundary.

## G7 — const materialization
Run `tests/compile/pass/const-data-runtime-call.ph`. If a transparent const data
value loses its ordinary runtime behavior after materialization, record a
compiler/runtime incident.
