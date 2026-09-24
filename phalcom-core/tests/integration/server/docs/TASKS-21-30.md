# Tasks 21–30 implementation status

| Task | Status | Primary files |
|---|---|---|
| 21. Publish `OrderPaid` | Implemented | `../application/checkout.ph`, `../application/command.ph`, `../testing/golden_executor.ph` |
| 22. Construct typed response | Implemented | `../application/checkout.ph` |
| 23. Connect middleware to handler | Implemented | `../application/pipeline.ph` |
| 24. Canonical deterministic fixtures | Implemented | `../testing/golden_fixtures.ph`, `../testing/fixtures.ph` |
| 25. Typed spies and runtime trace | Implemented | `../testing/trace.ph`, repository/event fixtures, `../application/checkout.ph`, `../application/pipeline.ph` |
| 26. Golden runner | Implemented | `../golden/run.ph`, `../golden/report.ph` |
| 27. Execute through `Fiber` | Implemented | `../golden/main.ph` |
| 28. Tuple-pack coverage | Implemented | `../golden/tuple_pack.ph`, `../negative/tuple_pack_mismatch.ph` |
| 29. Sequence-pack coverage | Implemented | `../golden/sequence_pack.ph`, `../negative/sequence_pack_mismatch.ph` |
| 30. Explicit type witnesses | Implemented | `../golden/type_witnesses.ph`, `../golden/assertions.ph`, `../negative/wrong_final_type_witness.ph` |

## Full positive path

```text
RawRequest
  -> requestContext
  -> attachRequestId (**R)
  -> decodeCheckoutRoute (**R)
  -> authenticate (**R; `...` for inspection)
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
  -> Order<Paid>
  -> Command::PublishPaid
  -> ()
  -> Response<Ok, CheckoutView>
  -> TestReport
  -> Fiber<TestReport>
```

## Runtime oracle

The canonical trace is typed as `List<TraceEvent>` and must equal:

```text
RequestIdentified(req-0001)
RouteDecoded(ord-123)
Authenticated(usr-001)
BodyDecoded
DraftLoaded(ord-123)
Submitted(ord-123)
PaymentAuthorized(ord-123)
Paid(ord-123)
Saved(ord-123)
Published(ord-123)
Responded
```

The repository call spy must record exactly `FindDraft(ord-123)` followed by `SavePaid(ord-123)`, and the event publisher must record exactly one `OrderPaid` value.

## Pack coverage

The package now deliberately contains all three named rest domains:

```text
*T      sequence rest
**R     structural record row
***S    heterogeneous tuple shape
```

They are represented by separate probes and separate expected-failure fixtures. No compile-time compile-time type-test branching branching is used; runtime type refinement remains expressed with `is` where needed.
