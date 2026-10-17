# NATIVE SECURITY CONTRACT: Cryptographic Randomness and Host Primitives

**Program:** Telos Cryptographic Foundations (`UNIV002.C3` / `CRYPTO.P2`)  
**Status:** VERIFIED  
**Host Substrate:** `phalcom-core/src/primitive/random.rs`  
**CSPRNG Dependency:** `getrandom = "0.3.3"`  
**Language Facade:** `packages/universe/src/random/secure.ph` → `packages/universe/src/crypto/random/package.ph`  

---

## 1. Scope and Objective

This contract specifies and audits the security, concurrency, memory, and failure invariants of the host cryptographic entropy substrate supporting `universe.crypto` and downstream packages (including `nanoid`).

---

## 2. Entropy Source and Host Integration

1. **OS Entropy Provider:** Entropy is acquired exclusively via Rust `getrandom::fill(&mut [u8])`.
   - On Linux/Android: `getrandom(2)` syscall (urandom fallback if unseeded).
   - On macOS/iOS: `getentropy(2)` / `SecRandomCopyBytes`.
   - On Windows: `BCryptGenRandom`.
   - On WebAssembly: Web Cryptography API (`crypto.getRandomValues`).
2. **Zero Deterministic Fallback:** Under no circumstance does the native primitive fall back to user-space PRNGs (such as `Xoshiro256PlusPlus`, linear congruential generators, timestamp seeds, or process IDs).
3. **Blocking Behavior:** On systems where the CSPRNG has not yet initialized, `getrandom` will block until sufficient host entropy has been gathered, guaranteeing cryptographically sound random bytes.

---

## 3. Failure Semantics and Atomicity

1. **Failure Propagation:** Host entropy failure returns `Err(getrandom::Error)` to Rust.
2. **Telos Representation:** The native method `SecureRandom#rawFill(_ bytes: Bytes) -> Option<String>` returns:
   - `Option::None` on complete success.
   - `Option::Some(error_message)` on failure.
3. **Exception Elevation:** In `packages/universe/src/random/secure.ph`, non-None return raises `EntropyError.new(message).raise()`.
4. **Buffer State on Failure:** If entropy acquisition fails, destination buffer contents are considered poisoned and unspecified. No successful return value is ever produced from a failed operation.
5. **No Partial Return:** High-level APIs such as `SecureRandom#bytes(count: Int)` only allocate and return a `Bytes` instance after `fill` succeeds unconditionally.

---

## 4. Memory Lifecycle, GC Safety, and Reentrancy

1. **Heap Allocation Safety:**
   - In `phalcom-core/src/primitive/random.rs`:
     ```rust
     let id = expect_bytes(vm, &args[0])?;
     let outcome = fill_system_random(vm.heap.bytes_mut(id).as_mut_slice());
     match outcome {
         Ok(()) => Ok(Value::none()),
         Err(error) => Ok(vm.alloc_string_value(error.to_string()).wrap_some()?),
     }
     ```
   - The mutable borrow `vm.heap.bytes_mut(id)` is strictly scoped and released before any subsequent heap allocation (`vm.alloc_string_value`), preventing RefCell/arena borrow conflicts and ensuring GC root safety.
2. **Buffer Limits and Validation:**
   - Public APIs validate `count >= 0`. Negative counts immediately raise `ArgumentError`.
   - Buffer size is constrained by the host address space and VM buffer bounds (up to `2^31 - 1` bytes).
3. **No Unsynchronized Shared Buffers:** Each call to `SecureRandom#bytes(n)` produces an independent heap-allocated `Bytes` object with unique generational identity. No global mutable byte pool is shared across fibers.

---

## 5. Effect Tracking and Semantic Metadata

The primitive metadata is registered via `#[phalcom_native_macros::primitive]`:

```rust
#[phalcom_native_macros::primitive(
    SecureRandom,
    "rawFill(_)",
    params = [Bytes],
    returns = "Option<String>",
    types = "(Bytes) -> Option<String>",
    visibility = internal,
    effects = [mutation, nondeterminism, blocking]
)]
```

- **`mutation`:** The destination `Bytes` object is modified in place.
- **`nondeterminism`:** Outputs are cryptographically unpredictable; constant folding and pure memoization are prohibited.
- **`blocking`:** Host calls may block waiting for OS entropy pool readiness.
- **Compiler Safety:** The semantic checker and optimizer must never eliminate or reorder calls across observable state boundaries or execute `rawFill` during static evaluation.

---

## 6. Concurrency and Multi-Fiber Semantics

1. **Host Thread-Safety:** `getrandom` is fully thread-safe and reentrant across OS threads.
2. **Fiber Isolation:** Telos fibers executing `SecureRandom.system.bytes(n)` receive unshared memory. Because generators are stateless and buffers are fiber-local, race conditions on random output buffers cannot occur.

---

## 7. Performance Characteristics and Acceleration Evaluation

1. **System Call Overhead:** Direct OS entropy calls incur context switch and kernel transition overhead (~100–500 ns per call depending on platform).
2. **Batching Requirement:** Requesting entropy in batches (e.g. 21 bytes for a 21-char Nano ID, or chunked buffers up to 4096 bytes) amortizes syscall costs.
3. **Acceleration Evaluation (Tier 1 vs. Tier 2):**
   - **Tier 1 (Current):** Pure Telos sampling using `SecureRandom.system.bytes(...)` and bit-mask rejection sampling in byte buffers. Clean, portable, easily audited.
   - **Tier 2 (Future):** A generic native ASCII sampling primitive (`SecureRandom#rawFillUniformAscii`) would eliminate VM byte loop overhead for high-throughput workloads (>100,000 IDs/sec). Tier 2 remains deferred until profiling demonstrates measurable bottlenecks in Tier 1.
