# CONC002 current C3 supersession amendment

Apply this amendment to the current `C3-concurrency-library-polish-and-composition-primitives` checkpoint and its P1/P2 plans when migrating the top-level graph.

## Required lifecycle change

The existing C3 documents are **superseded as executable implementation plans**, because their present scope mixes work now assigned to several distinct dependency levels:

- reactor/timers/external readiness → new C3;
- completion/composition/time library utilities → new C4;
- TaskScope/cancellation/structured task policy → C5;
- channels/select → C6.

Use:

```yaml
status: SUPERSEDED
completion: PARTIAL
verification: UNVERIFIED
```

where the current records require lifecycle frontmatter. Do not delete their substantive design content until new plans extract the still-valid requirements.

## Banner to add

> **Superseded execution ownership.** This document is retained as source material only. The CONC002 program graph was restructured so reactor/external readiness is C3, standard-library composition/time is C4, cancellation/structured concurrency is C5, and channels/select is C6. Do not execute this plan directly. Reuse its still-valid requirements only through newly issued plans under the owning checkpoint.

## Link correction

Any dependency text that currently points at the deleted original `CONC002.C2.P1-native-suspension-and-reactor-groundwork.md` must instead refer to canonical `CONC002.C2.P1-R1-native-suspension-and-vm-control-continuations.md` for VM continuation prerequisites.
