# NANOID-ALGORITHM: Cryptographic Uniformity, Rejection Sampling, and Implementation Invariants

**Program:** Telos Cryptographic Foundations (`UNIV002.C3` / `NANOID.P1` & `NANOID.P2`)  
**Package:** `packages/nanoid/`  
**Reference Algorithms:** AI Nano ID (JavaScript reference), RFC 4086  

---

## 1. Overview and Core Objectives

The Nano ID algorithm generates compact, URL-friendly, cryptographically secure identifiers. To guarantee security and unbiased symbol distribution, the implementation must adhere to strict mathematical and algorithmic rules regarding entropy acquisition, index mapping, and string construction.

---

## 2. Modulo Bias and Bit-Mask Rejection Sampling

### 2.1 The Modulo-Bias Problem

Given a random byte $b \in [0, 255]$ and an alphabet of cardinality $N$:
Mapping $b \mapsto b \pmod N$ is biased whenever $256$ is not a multiple of $N$. For example, with $N = 62$ (alphanumeric):
$$256 = 4 \times 62 + 8$$
The first 8 alphabet symbols have probability $\frac{5}{256} \approx 0.01953$, while the remaining 54 symbols have probability $\frac{4}{256} \approx 0.01562$ (a ~25% bias). For security-sensitive tokens, this leakage significantly weakens collision resistance and entropy guarantees.

### 2.2 Bit-Mask Rejection Sampling

To achieve perfectly uniform selection without bias, we calculate the smallest all-ones bit-mask covering all alphabet indices:
$$\text{bits} = \lceil \log_2 N \rceil$$
$$\text{mask} = 2^{\text{bits}} - 1$$
For any candidate byte $b \leftarrow \text{entropy}$:
$$\text{index} = b \ \& \ \text{mask}$$
- If $\text{index} < N$, accept $\text{index}$.
- If $\text{index} \ge N$, reject the candidate and sample the next byte.

Because every candidate in $[0, \text{mask}]$ is produced by truncation of uniformly random source bits, each integer in $[0, \text{mask}]$ has identical probability $2^{-\text{bits}}$. Restricting to the subset $[0, N-1]$ via rejection yields a conditional probability distribution:
$$P(\text{index} = k \mid \text{accepted}) = \frac{1}{N} \quad \text{for all } 0 \le k < N$$
This is provably unbiased.

### 2.3 Power-of-Two Fast Path

When $N$ is a power of two ($N = 2^{\text{bits}}$, e.g. $N = 64$ for the default URL-safe alphabet):
$$\text{mask} = N - 1 = 63$$
In this case, $\text{index} = b \ \& \ \text{mask} < N$ is unconditionally true for every byte. Rejection probability is exactly $0\%$:
$$P(\text{rejection}) = 0$$
Every random byte produces exactly one output symbol with zero waste and zero rejection loops.

---

## 3. Batch Entropy Acquisition

1. **System Call Amortization:** Requesting individual random bytes from the OS kernel introduces prohibitive syscall overhead. The generator requests entropy in batches.
2. **Batch Sizing:**
   - For power-of-two alphabets: $\text{batchSize} = \min(\text{remaining}, 4096)$.
   - For general alphabets:
     $$\text{expectedBytes} = \text{remaining} \times \frac{\text{mask} + 1}{N}$$
     The generator applies a heuristic factor of $1.6$ to cover unlucky rejection streaks:
     $$\text{batchSize} = \min\left(\left\lfloor \frac{\text{remaining} \times (\text{mask} + 1) \times 8}{5N} \right\rfloor + 1, 4096\right)$$
3. **Bounded Chunk Limit:** Entropy batch allocation is capped at $4096$ bytes ($4 \text{ KB}$) to prevent unbounded memory spikes on large ID requests.

---

## 4. Immutable Byte-Buffer Assembly

1. **Zero Concatenation:** String generation does not perform repeated immutable string concatenations (`s = s + c`), avoiding quadratic copying overhead.
2. **Single Pass UTF-8 Decode:** Output symbols are written directly into a preallocated `Bytes.new(length)` buffer at byte offsets. Because the alphabet is validated to contain only printable ASCII (codes $33..126$), every byte is valid 1-byte UTF-8. The buffer is decoded once via `output.utf8`.

---

## 5. Phantom-Typed Identifiers (`Id<Tag>`)

To prevent accidental cross-domain identifier substitution (e.g., passing an `OrderId` where a `UserId` is expected), Nano ID provides optional nominal phantom tagging:
```telos
data Id<Tag> private(_ value: String)
```
- `Id<UserTag>` and `Id<OrderTag>` are distinct applied nominal types.
- The tag parameter is a phantom type carrying zero runtime memory payload.
- Generation via `NanoId.typed<Tag>()` produces a strongly-typed identifier while preserving underlying string storage efficiency.
