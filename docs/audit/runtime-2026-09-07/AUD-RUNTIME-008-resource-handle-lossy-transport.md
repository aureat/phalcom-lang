# AUD-RUNTIME-008 — Resource identity is transported through a lossy Float

## Classification

Severity: High at the resource identity boundary; normal-workload frequency unmeasured. **Confirmed by source and representability analysis; not executed.** HEAD `1e22b57ff8cd0c61b12aeca4128671c9fabdef6e`.

## Producer/consumer evidence

`src/resource.rs::ResourceHandle` contains two u32 fields. `pack` concatenates them into a u64 and converts that integer numerically with `packed as f64`; `unpack` converts the Float numerically back to u64. This is not bit-preserving serialization.

`primitive/resource.rs::resource_register` publishes the packed Float as the resource's handle value. `extract_handle` retrieves the first instance slot, reads a Float (or converts an Int to Float), and unpacks it. The table checks exact index and generation in resolve/close; `is_closed` treats an absent or mismatching handle as closed.

## Representability proof

For index 2097152 (2^21) and generation 1, concatenation yields 2^53 + 1. Binary64 cannot represent that integer; conversion rounds it to 2^53. Unpacking therefore produces generation 0 rather than 1 at the same index. This is a concrete non-round-tripping pair derived from the implementation, without allocating millions of resources or executing a probe.

Fresh table rows use generation 1 and indices increase when no free row exists. This pair can arise if the table grows sufficiently; no such workload was run. Generation reuse creates additional precision questions. Wrong-resource closure or an exploit is not established: the shown case fails the generation check and can report a newly registered resource as closed/stale.

## Why the existing endpoint check is insufficient

`test_handle_pack_unpack_boundary` checks (0,1) and (u32::MAX,u32::MAX). The latter's endpoint conversion back to u64 saturates, which can recover u64::MAX despite the intervening precision loss. Endpoint success therefore does not prove injectivity or identity preservation between endpoints. The test was inspected, not run.

## Direction and verification

Retain full handle bits through a dedicated representation or exact payload rather than numeric Float conversion. Preserve generation validation and distinguish this separate resource table from the heap's generational ObjRef arena, which is not implicated by this transport path.

Future checks: round trips around 2^53, several generations at the same high index, and public register/extract agreement. A mathematical or property-level check suffices for encoding; do not require an enormous live-resource workload merely to establish the loss.

No fixes, runtime probes or commits were performed.
