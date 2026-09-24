# Phalcom Typing Golden — Tasks 1–38

This package is the complete source specimen and certification shell for the
next-generation Phalcom typing golden program.

## Implemented source surface

Tasks 1–30 build the executable checkout specimen: nominal domain values,
typestate, static HTTP/retry domains, higher-kinded effects, associated-type
capabilities, deterministic fixtures, row-polymorphic middleware, `Command<R>`
refinement, payment/persistence/publication, `Response<Ok, CheckoutView>`, typed
runtime tracing, `Fiber<TestReport>`, tuple/sequence packs, and inference
witnesses.

Tasks 31–34 add public reflection, runtime specialization identity, deep
flow-sensitive `is` refinement, and the complete expected-failure mutation
corpus.

Tasks 35–38 add the repository-facing certification harness for clean builds,
semantic persistence/import, incremental invalidation, and a stable executable
stdout oracle.

## Syntax conventions

- Closures: `|x| expr` or `|x| { ... }`.
- Trailing closures are preferred where applicable.
- Static variants in type position are unqualified: `Order<Draft>`,
  `Response<Ok, T>`, `RetryPolicy<Count(3)>`.
- Runtime type refinement uses `is`.
- Named rest captures are `*T`, `**R`, and `***S`; ignored remainder is `...`.

## Reflection target

The golden reflection probes use the proposed reified type surface through
`value.type`, including specialization identity and structural `fields`. These
are intentional language acceptance targets, not user-space reflection shims.

## Certification integration

The package does not guess a repository-specific compiler command line or module
manifest syntax. Set `PHALCOM_GOLDEN_DRIVER` to a repository adapter implementing
`certification/driver-contract.md`, then run:

```text
certification/run-certification.sh
```

Until the actual repository driver runs those scenarios, tasks 35–37 should be
read as **implemented certification gates, not claimed compiler pass results**.

## Stable runtime path

```text
RawRequest
  -> request ID
  -> route decode
  -> authentication
  -> CheckoutRequest decode
  -> LoadDraft
  -> Order<Draft>
  -> submit
  -> Order<Submitted>
  -> payment authorization
  -> pay
  -> Order<Paid>
  -> SavePaid
  -> PublishPaid
  -> Response<Ok, CheckoutView>
  -> reflection / runtime identity / `is` refinement probes
  -> TestReport
  -> Fiber<TestReport>
```

See `docs/TASKS-1-38.md`, `docs/TASKS-31-38.md`, `negative/manifest.tsv`, and
`certification/README.md` for the complete acceptance surface.
