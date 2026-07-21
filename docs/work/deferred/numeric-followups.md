# Numeric follow-ups

**Status:** deferred by PDR-0027. These are not permission to weaken its ratified contracts.

| Item | Why deferred | Re-entry condition |
|---|---|---|
| Constant-pool `LargeInt` GC roots | Must be verified against the compiled-constant lifetime; guessing risks dangling heap references. | Before compile-time LargeInt constants ship: prove tracing or add a root and a GC-stress golden. |
| Strict collection/index boundaries | Existing `expect_index` accepts integral Float; changing it crosses collection APIs and fixture corpus. | Dedicated boundary-tightening unit with machine-checkable marker removal. |
| Numeric budget defaults | Guard and `#numericLimit` are required now; safe limits require workload measurement. | Benchmark ordinary arithmetic, adversarial powers/shifts, and GC pressure. |
| Serialization/interchange | `toString` is display text, not a wire or constant-pool format. | Define versioning, NaN/Infinity policy, and LargeInt encoding together. |
| Extended numeric API | `sqrt`, `abs` variants, `min`, `max`, factorial, and domains/promotion need one coherent inventory. `**` is already ratified. | API-design record with result and error rules. |
| Public total order / Float bits | Key equivalence needs no public total order; exposing one commits NaN payload and signed-zero policy. | A sorting/interchange use case needs it. |
| Numeric literal diagnostic catalogue | Grammar and primary malformed span are specified; per-case message wording and subspan catalogue need implementation evidence. | Lexer/parser implementation has stable diagnostic IDs. |
