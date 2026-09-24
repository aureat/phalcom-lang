# Expected-failure compiler fixtures — tasks 1–20

These files are intentionally invalid and should be wired as independent
expected-failure fixtures in the Phalcom compiler test harness. They must not be
included in the positive executable package compilation unit.

The current corpus covers:

- typestate misuse,
- static response/status specialization mismatch,
- parameterized static-value identity mismatch,
- higher-kind mismatch,
- associated-type contract mismatch,
- missing structural handler fields,
- nested row-field mismatch,
- dropped row information,
- incompatible reuse of one inferred row,
- recursive row occurs-check behavior,
- incorrect result-indexed command branches,
- and the `Count(3)` payment-gateway specialization required by checkout.

`expected.md` records the intended principal diagnostic for each fixture.
The exact repository convention for negative compiler fixtures remains an
integration concern rather than something invented by this standalone package.

## Tasks 21–30 additions

- `tuple_pack_mismatch.ph` — heterogeneous `***S` shape mismatch.
- `sequence_pack_mismatch.ph` — `*T` rest mismatch.
- `wrong_fiber_result.ph` — Fiber result specialization mismatch.
- `wrong_final_type_witness.ph` — explicit witness catches wrong static response status.

The existing `broken_command_publish.ph` remains the principal negative test for Task 21: `.PublishPaid(...)` establishes command result `()` and must not return another type.
