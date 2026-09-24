# Tasks 1–10 implementation status

| Task | Status | Primary files |
|---|---|---|
| 1. Package shell / executable boundary | Implemented | `../README.md`, `../golden/main.ph` |
| 2. Primitive domain model | Implemented | `../domain/identity.ph`, `money.ph`, `user.ph`, `payment.ph`, `event.ph`, `order.ph` |
| 3. Static order states | Implemented | `../domain/order.ph` |
| 4. Executable transitions | Implemented | `../domain/order.ph`, `../golden/smoke.ph` |
| 5. HTTP static domains | Implemented | `../http/http.ph` |
| 6. Parameterized retry static values | Implemented | `../domain/payment.ph`, `../golden/smoke.ph` |
| 7. Higher-kinded effect contract | Implemented | `../effect/effect.ph` |
| 8. `TestFx<T>` | Implemented | `../effect/test_fx.ph` |
| 9. Associated-type interfaces | Implemented | `application/*.ph` |
| 10. Deterministic fixtures | Implemented | `testing/*.ph`, `../golden/smoke.ph` |

## Added by the tasks 11–20 checkpoint

The package now also includes row-polymorphic middleware (`**R` / `...`), the result-indexed command algebra and interpreter, and checkout application composition through persistence of `Order<Paid>`.

Tasks 21–30 now add event publication, Fiber execution, tuple/sequence pack probes, the complete runner, and final type witnesses. Reflection and build/persistence certification remain in Tasks 31–38.

No compile-time type-test branching is present.
