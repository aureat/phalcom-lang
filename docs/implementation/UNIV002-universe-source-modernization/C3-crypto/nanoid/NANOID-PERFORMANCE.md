# NANOID-PERFORMANCE: Execution Profiles, Memory Bounds, and Acceleration Strategy

**Program:** Telos Cryptographic Foundations (`UNIV002.C3` / `NANOID.P3`)  
**Target:** `packages/nanoid/`  
**Execution Tiers Evaluated:**
- **Tier 1:** Pure Telos algorithm over `SecureRandom.system.bytes` (Current baseline)
- **Tier 2:** Native bulk ASCII sampling in Rust host (`SecureRandom#rawFillUniformAscii`)

---

## 1. Benchmarked Workloads and Scenarios

Four benchmark profiles are established under `packages/nanoid/benchmarks/`:

1. **`default.ph`:** Default 21-character URL-safe generator across sustained iterations.
2. **`custom-alphabet.ph`:** Compares power-of-two (16-char hex) with rejection-sampling (62-char alphanumeric) alphabets to isolate the rejection overhead.
3. **`batch.ph`:** Evaluates scaling across sizes (12, 64, 128, 1024, 4096 bytes) to verify buffer chunking and system call amortization.
4. **`throughput.ph`:** Measures sustained generation throughput with reused generator instances.

---

## 2. Theoretical and Measured Resource Analysis

### 2.1 Entropy System Calls

- **Default 21-char ID (Alphabet size 64):**
  - Requires exactly 1 host entropy acquisition of 21 bytes.
  - Zero bytes rejected.
  - Exactly 1 system call per ID.
- **Custom 21-char ID (Alphabet size 62):**
  - Smallest bit-mask: 63 (6 bits).
  - Acceptance probability: $62 / 64 = 96.875\%$.
  - Expected bytes needed: $21 \times \frac{64}{62} \approx 21.68$ bytes.
  - With batch multiplier $1.6$, initial request is $28$ bytes.
  - Probability that 28 bytes yield $\ge 21$ valid symbols is $> 99.99\%$, ensuring that $> 99.9\%$ of IDs require only a single OS entropy request.

### 2.2 Memory Allocations per Generated ID

1. **Entropy Buffer:** 1 `Bytes` object allocated per generation (21 bytes for default).
2. **Output Buffer:** 1 `Bytes` object allocated per generation (21 bytes for default).
3. **Decoded String:** 1 `String` object allocated upon final `output.utf8`.
4. **Total Allocations:** Exactly 3 objects per ID, bounded and linear in length.
5. **Zero Intermediate Strings:** Does not allocate or concatenate intermediate substrings.

---

## 3. Tier 1 vs. Tier 2 Evaluation (Native Acceleration)

### 3.1 Tier 1 Advantages
- Written entirely in high-level Telos without expanding the native primitive boundary or modifying `phalcom-native-meta`.
- Immune to GC rooting edge cases that arise with custom native buffer loops.
- Trivially portable across any Telos runtime or future backend (e.g. LLVM/native compilation).

### 3.2 Tier 2 Trade-offs
- A native `SecureRandom#rawFillUniformAscii(output, alphabet)` would execute the index selection loop in native Rust machine instructions.
- Eliminates VM bytecode dispatch for `output.set(written, alphabetBytes.at(index))`.
- However, VM dispatch in the Phalcom VM is optimized with direct method inlining and unboxed immediates; the dominant latency in identifier generation remains host entropy acquisition (`getrandom`).
- Introducing Tier 2 would require adding new native methods to `SecureRandom` in `phalcom-native-meta` and `phalcom-core`, expanding production native authority before the Universe migration cutover.

### 3.3 Final Recommendation
**Retain Tier 1 as the primary implementation.** The pure Telos algorithm satisfies all performance and safety requirements with clean architecture and zero native footprint expansion.
