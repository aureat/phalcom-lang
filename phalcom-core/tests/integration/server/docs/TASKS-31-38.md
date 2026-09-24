# Tasks 31–38 implementation status

| Task | Status | Primary files |
|---|---|---|
| 31. Reflection coverage | Implemented as golden source target | `../golden/reflection.ph`, `../golden/run.ph` |
| 32. Reflection + runtime type identity | Implemented | `../golden/runtime_identity.ph`, `../golden/run.ph` |
| 33. Deep `is` refinement | Implemented | `../golden/runtime_refinement.ph`, `../domain/event.ph`, negative refinement fixture |
| 34. Complete negative mutation corpus | Implemented | `negative/*.ph`, `../negative/manifest.tsv`, `../negative/expected.md` |
| 35. Clean-build certification | Harness implemented; repository execution pending | `../certification/run-certification.sh`, `driver-contract.md` |
| 36. Persistence/import certification | Harness implemented; repository execution pending | certification driver contract/scenario manifest |
| 37. Incremental-build certification | Harness implemented; repository execution pending | certification driver contract/scenario manifest |
| 38. Stable golden output | Implemented | `../golden/main.ph`, `../certification/expected-output.txt` |

## Reflection contract

The golden source intentionally exercises reified type values via `value.type`.
It requires specialization identity to distinguish `Order<Paid>`,
`Response<Ok, CheckoutView>`, and `RetryPolicy<Count(3)>`. Structural reflected
types expose ordered `fields`, allowing the inferred checkout context to be
observed rather than existing only transiently inside the solver.

## Runtime refinement contract

All type-test flow refinement is expressed with `is`. Coverage includes positive
branch refinement, a terminating negative guard whose fallthrough is refined,
and conjunction with a property available only after refinement. A negative
fixture verifies that refinement does not escape its valid region.

## Build certification contract

The standalone package cannot truthfully certify the repository's cache,
persistence, or incremental compiler behavior without the actual compiler driver.
Instead it includes a driver-neutral executable harness. Once integrated, tasks
35–37 become hard pass/fail gates rather than documentation-only procedures.
