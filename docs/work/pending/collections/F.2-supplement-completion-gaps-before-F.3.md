# F.2 Supplement — Closure Record Before F.3

Status: **closure record — F.2 completion gate satisfied, with separately deferred GC-stress infrastructure**

This document supersedes its earlier role as a list of missing F.2 work. It does not amend the ratified semantics in **F.2 — Outgoing Pack Assembly and Dynamic Send (amended)**. It records closure evidence for the former gaps and the one infrastructure item that remains deliberately unbuilt.

Primary focused evidence:

- `phalcom-core/tests/outgoing_packs.rs`
- `phalcom-core/tests/outgoing_packs_completion.rs`
- `phalcom-core/tests/f2_pack_gc.rs`

## Closure status

| Former gap | Status | Closure evidence |
|---|---|---|
| Tuple/Unit positional `*` fast lane | **IMPLEMENTED + VERIFIED** | `PackTryExpandTuplePositionals`; positional-lane projection and Unit behavior are covered by outgoing-pack tests and dynamic disassembly. |
| Generic iterable `*` | **IMPLEMENTED + VERIFIED** | Compiler cursor-bytecode lowering is covered for Range, user Iterable, and bounded pipeline sources; disassembly proves bytecode loop lowering. |
| E.3 boundedness integration | **IMPLEMENTED + VERIFIED** | Pack-site unbounded Range rejection and bounded `take(3)` acceptance are covered by `positional_spread_reuses_e3_at_the_actual_pack_site`. |
| Structured F.2 runtime errors | **IMPLEMENTED + VERIFIED** | Completion tests cover computed-label type, duplicate labels, invalid `**` Map keys, invalid `***`, non-Iterable `*`, and dynamic arity failures. |
| Dynamic subscript writes | **IMPLEMENTED + VERIFIED** | Pack lowering, lexical receiver/index/RHS timing, pre-RHS `put` reservation, selector formation, result preservation, and setter arity are covered. |
| Dynamic Tuple construction | **IMPLEMENTED + VERIFIED** | Dynamic Tuple lane preservation/empty normalization and construction above 255 elements are covered. |
| Lexical evaluation and collision timing | **VERIFIED** | Receiver-first, single evaluation, full generic-spread exhaustion before later items, computed-label validation, and duplicate short-circuiting are covered. |
| 255/256 dynamic send arity | **VERIFIED** | 255 succeeds; 256 fails before lookup/dNU. |
| Subscript-set implicit `put` arity | **VERIFIED** | 254 indices plus RHS succeeds; 255 indices plus RHS fails before lookup/dNU. |
| Compiler-internal authority | **VERIFIED** | Dynamic compiler-generated authority remains available; source cannot forge the reserved internal selector path. |
| Fiber suspend/resume during spread | **VERIFIED** | Generic cursor expansion remains bytecode-driven and focused Fiber regression coverage passes. |
| Static send/Tuple fast-path disassembly | **VERIFIED** | Static calls retain `Invoke` and static Tuple retains `BuildTuple`, with no pack machinery; dynamic source emits the pack lane. |
| Focused PackBuilder GC tracing | **VERIFIED** | `f2_pack_gc.rs` forces collection through `VM::force_gc()` and proves PackBuilder-held values remain traced. |
| `PHALCOM_GC_STRESS` every-safepoint lane | **DEFERRED INFRASTRUCTURE** | F.2 has focused forced-GC proof. General every-safepoint stress harness remains separately specified and unbuilt; this record does not claim it exists. |

## F.3 start gate

F.3 may begin when:

- [x] positional Tuple/Unit fast lane verified
- [x] generic Iterable `*` verified
- [x] E.3 integration verified
- [x] lexical timing/collision semantics verified
- [x] structured expansion errors verified
- [x] dynamic subscript set verified
- [x] dynamic Tuple verified
- [x] 255/256 send arity verified
- [x] setter arity includes implicit `put`
- [x] >255 Tuple construction verified
- [x] Fiber suspension verified
- [x] compiler-internal authority verified and non-forgeable
- [x] static bytecode fast path verified
- [x] PackBuilder GC tracing verified
- [deferred] every-safepoint `PHALCOM_GC_STRESS` infrastructure

The deferred general stress harness is not a reason to reopen F.2 implementation work. It remains infrastructure work with its own specification and should strengthen future collection coverage without changing this focused F.2 closure evidence.

## Scope retained for F.3

F.3 starts from the completed outgoing-pack substrate: shared compiler pack assembly, canonical positional and labeled lanes, dynamic dispatch or Tuple finalization, lexical timing, structured failures, and rooted builder state across allocation and fiber activity. F.3 does not reopen F.2 semantics merely to build rest capture and rest-pattern dispatch.
