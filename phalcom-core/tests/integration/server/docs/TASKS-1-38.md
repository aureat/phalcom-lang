# Phalcom typing golden — Tasks 1–38

The package now contains the complete proposed golden specimen and its
certification surface.

## Functional spine

```text
RawRequest
  -> open-row middleware
  -> enriched checkout context
  -> Command<Option<Order<Draft>>>
  -> Order<Draft>
  -> Order<Submitted>
  -> effectful payment authorization
  -> Order<Paid>
  -> Command<Order<Paid>>
  -> Command<()>
  -> Response<Ok, CheckoutView>
  -> reflection/runtime identity/refinement probes
  -> TestReport
  -> Fiber<TestReport>
```

## Typing domains covered

- user-defined static domains and parameterized static values,
- typestate specialization,
- higher-kinded `F: Type -> Type`,
- associated types,
- structural rows and open remainder,
- heterogeneous tuple packs `***S`,
- sequence rest packs `*T`,
- result-dependent `Command<R>` cases,
- contextual closure inference,
- structural handler compatibility,
- public type reflection,
- runtime `is` flow refinement,
- persistence/import and incremental certification scenarios.

No compile-time type-test branching operator is used by the golden source.
