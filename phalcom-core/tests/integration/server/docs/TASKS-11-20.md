# Tasks 11–20 implementation status

| Task | Status | Primary files |
|---|---|---|
| 11. Structural request middleware | Implemented | `../application/middleware.ph` |
| 12. Composed middleware pipeline | Implemented | `../application/pipeline.ph`, `../golden/tasks_11_20.ph` |
| 13. Typed command algebra | Implemented | `../application/command.ph` |
| 14. Command interpreter / GADT refinement | Implemented | `../testing/golden_executor.ph` |
| 15. `CheckoutServices` capability aggregate | Implemented | `../application/services.ph` |
| 16. `checkoutHandler` | Implemented | `../application/checkout.ph` |
| 17. Ownership validation + submission | Implemented | `../application/checkout.ph` |
| 18. Payment authorization | Implemented | `../application/checkout.ph` |
| 19. `Submitted -> Paid` transition | Implemented | `../application/checkout.ph` |
| 20. Persist paid order | Implemented | `../application/checkout.ph` |

## Checkpoint behavior

The tasks 11–20 executable path is:

```text
RawRequest
  -> structural requestContext
  -> attachRequestId (**R)
  -> decodeCheckoutRoute (**R)
  -> authenticate (**R; `...` used by token inspection)
  -> decodeJson<CheckoutRequest> (**R)
  -> checkoutHandler
  -> Command::LoadDraft
  -> Order<Draft>
  -> submit
  -> Order<Submitted>
  -> PaymentGateway<TestFx, Count(3)>
  -> pay
  -> Order<Paid>
  -> Command::SavePaid
  -> Response<Ok, CheckoutView>
```

The current package extends this checkpoint with Task 21, so the shared `checkout.ph` now inserts `PublishPaid` between persistence and response construction. The historical Tasks 11–20 runner remains as a compatibility replay against the newer application path.

No compile-time compile-time type-test branching branching is used anywhere. Runtime `is` remains used only in the deterministic task-10 fixture implementations where concrete specialization/event refinement is useful.
