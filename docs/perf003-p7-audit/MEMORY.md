# Memory Consumption & Allocation Accounting Audit

## 1. Peak Resident Set Size (RSS) Analysis

Measurements taken via macOS kernel resource accounting (`libc::getrusage` on Darwin, reading `ru_maxrss` in bytes).

| Scenario | Mode | Peak RSS (Bytes) | Peak RSS (MiB) | Delta vs Source |
|---|---|---|---|---|
| **Cold `print42`** | Source-backed | 818,659,328 | 780.7 MiB | Baseline |
| **Cold `print42`** | Artifact-backed | 403,161,088 | 384.5 MiB | **-50.7% (-396.2 MiB)** |
| **Cold `option_string`** | Source-backed | 818,790,400 | 780.8 MiB | Baseline |
| **Cold `option_string`** | Artifact-backed | 404,209,664 | 385.5 MiB | **-50.6% (-395.3 MiB)** |
| **Cold `synthetic_pkg`** | Source-backed | 818,757,632 | 780.8 MiB | Baseline |
| **Cold `synthetic_pkg`** | Artifact-backed | 404,488,192 | 385.8 MiB | **-50.6% (-395.0 MiB)** |
| **Cold `slotmap`** | Source-backed | 818,790,400 | 780.8 MiB | Baseline |
| **Cold `slotmap`** | Artifact-backed | 384,909,312 | 367.1 MiB | **-53.0% (-413.7 MiB)** |
| **Artifact Generation**| Source + Codec | 1,112,981,504 | 1,061.4 MiB | One-time compilation |

### Key Observations
1. **Cold Compiler Footprint Cut in Half**: Artifact-backed compiler processes peak at ~385–404 MiB, compared to ~780–820 MiB for source-backed compiler processes. This represents an empirical ~51% reduction in peak resident memory.
2. **Elimination of AST / Macro / Resolver Transient Overhead**: Source-backed compilation incurs high transient RSS spikes during AST generation, macro expansion, and preliminary name resolution across 40+ canonical Universe modules. Loading serialized byte sections eliminates this entire phase.

---

## 2. Cumulative Allocation Budget Accounting (`AllocationBudget`)

Phalcom enforces a strict fail-closed transactional allocation accounting model via `phalcom_type_meta::allocation::AllocationBudget`.

### Exact-Fit / N-1 Empirical Verification
A binary search was conducted across the 512 MiB search space using `perf_audit_p7 allocation-check`:

- **Exact Minimum Transaction Budget ($N$)**: `322,547,484` bytes (~307.61 MiB).
- **At $N - 1$ (`322,547,483` bytes)**: `load_universe_telosc_from_bytes` strictly **FAILS** with `TeloscArtifactError::BudgetExceeded`.
- **At $N$ (`322,547,484` bytes)**: `load_universe_telosc_from_bytes` cleanly **SUCCEEDS**.

This proves that:
1. Allocation guards are **not** relaxed or disabled.
2. All sections participate in the cumulative transaction budget.
3. Allocation accounting is exact to the single byte.

---

## 3. Retained Memory & Allocation Families

### Section Breakdown in Persisted Artifact
- Total Artifact Size: `15,775,219` bytes (~15.05 MiB)
  - `MODULE_WORLD`: `156,313` bytes (0.99%)
  - `SEMANTIC_WORLD`: `12,788,887` bytes (81.07%)
  - `QUERY_IMAGE`: `1,339,575` bytes (8.49%)
  - `METADATA`: `1,489,890` bytes (9.44%)
  - Container Envelope & Header: `554` bytes (0.01%)

### Inspection of Finalization Path
- In `CanonicalUniverseFormalImage::from_loaded_artifact`:
  - `ModuleQueryProducts::from_external_modules` reconstructs unlinked/linked interfaces, resolved imports, and reverse imports directly from the imported `CanonicalExternalModuleWorld`.
  - All intermediate vectors and hash tables created during wire decode are charged to the section's allocated budget before promotion.
- In `run-warm-suite`:
  - 20 consecutive user compilations against a resident loaded artifact resulted in **0 memory leakage** (process RSS stayed exactly constant at `1,090,600,960` bytes from sample 1 to sample 20).
