# Cryptography — bind, do not build

- Status: **Draft** (exploration only — not proposed, not ratified, no owning unit)
- Date: 2026-07-15
- Depends on:
  [ADR-0019](../../../adr/accepted/0019-freeze-vm-blessed-primitive-floor.md) (the floor admission rule) ·
  [ADR-0024](../../../adr/accepted/0024-numeric-surface-split-int-float-and-division.md) (Accepted, **❌ not built** — the `Int` promotion channel, §4.2) ·
  [ADR-0050](../../../adr/accepted/0050-non-moving-mark-sweep-collector.md) (mark-sweep ⇒ no `Drop`, §7) ·
  [ADR-0008](../../../adr/accepted/0008-layered-exceptions-and-result.md) §4 (unified unwind — what makes `ensure`-scoping sound) ·
  [ADR-0018](../../../adr/accepted/0018-sacred-selector-inliner-and-override-guard.md) (the only speculation in the VM, §4.1)
- Related:
  [drafts/ffi.md](ffi.md) (**the delivery vehicle** — §2's census math is not repeated here) ·
  [drafts/bytes.md](bytes.md) (**the substrate** — §7 zeroization, §8 `equalsConstantTime_`, §9.1 the `Int` channel) ·
  [drafts/sealed-classes.md](sealed-classes.md) (**§5 owns the algorithm-confusion / JWT `alg` case and its CVE table**; §4 the reopening limit) ·
  [ADR-0043](../../../adr/accepted/0043-no-default-arguments-keep-selector-identity-pristine.md) · [ADR-0025](../../../adr/accepted/0025-external-internal-parameter-names.md) · [ADR-0021](../../../adr/accepted/0021-no-truthiness-enforcement.md) · [ADR-0015](../../../adr/accepted/0015-object-default-tostring.md) · [ADR-0022](../../../adr/accepted/0022-string-interpolation-backslash-paren-sigil.md) (the §6.3 defect)
- Floor counts cite **the tree / the census**, never an ADR (overlay §Known documentation defects #4).

> **How to use this document.** An exploration doc. It grows. Nothing here is committed
> or citable as a decision. New findings go under the section they refine; new
> uncertainties become a new `C-n` row in §9. Tree claims carry `file:line`; external
> claims carry a URL; committed positions carry an ADR §. Where this doc is unsure it
> says so. **It deliberately does not repeat [ffi.md](ffi.md) §2 (census math),
> [bytes.md](bytes.md) §7/§8/§9.1 (zeroization, constant-time compare, the `Int`
> channel), or [sealed-classes.md](sealed-classes.md) §5 (algorithm confusion + CVEs)** —
> it links them and adds the crypto-specific layer above.

## 1. Do not roll your own — the only load-bearing sentence in this document

**Phalcom must not implement cryptographic algorithms. Not in `.ph`, and — the part that
gets missed — not in Rust either.** Writing AES in Rust yourself is the same mistake as
writing it in `.ph`, just with better performance on the CVE. The correct move is to
**bind an audited crate** through [ffi.md](ffi.md)'s boundary and expose a task-shaped
surface (§6) over it.

Everything else in this document is subordinate to that sentence. §4's "Phalcom is better
positioned than JS" is *not* an invitation to write a hash function; it is an argument
about what a binding can honestly claim.

**Honest sourcing note, because this doc's own rule demands it.** "Don't roll your own
crypto" has **no authoritative primary source** — it is a folk distillation, not a
standards-body pronouncement. The citable substrate is:

- Schneier, *Memo to the Amateur Cipher Designer* (Crypto-Gram, 1998-10-15) —
  "Anyone, from the most clueless amateur to the best cryptographer, can create an
  algorithm that he himself can't break."
  <https://www.schneier.com/crypto-gram/archives/1998/1015.html>. The operative half is
  the follow-on: what is hard is an algorithm *no one else* can break, demonstrable only
  by years of analysis. Named "Schneier's Law" by Cory Doctorow in 2004
  (<https://www.schneier.com/blog/archives/2011/04/schneiers_law.html>).
- Blessing, Specter & Weitzner, *You Really Shouldn't Roll Your Own Crypto: An Empirical
  Study of Vulnerabilities in Cryptographic Libraries*, arXiv:2107.04940
  (<https://arxiv.org/abs/2107.04940>). **This one cuts sideways and is worth internalizing:**
  only **27.2%** of vulnerabilities in crypto libraries are cryptographic issues, while
  **37.2%** are memory-safety issues. The strongest modern argument for the maxim is
  *complexity and memory safety*, not cryptanalysis.

That last finding is directly relevant to Phalcom: it means "we write it in safe Rust"
neutralizes the *largest* vulnerability class in the corpus, and people will use that to
argue for rolling their own. **It does not follow.** The 27.2% that remains is the part
requiring cryptographic expertise and years of third-party analysis, and it is the part
that silently produces a key-recovery attack rather than a crash. Safe Rust buys the
memory-safety bucket and nothing else.

## 2. How other ecosystems actually do it

The dominant answer across every mainstream ecosystem is: **punt to native.** The survey
matters because the obvious counterexample (JS has crypto libraries in JS!) turns out to
be a story about constraint, not preference.

### 2.1 The mainstream path is C, wearing a JS costume

| Surface | What it actually is | Citation |
|---|---|---|
| **WebCrypto** (`crypto.subtle`) in Chrome | **BoringSSL**, in the renderer, in software | "Crypto is done directly in the renderer process, in software, using BoringSSL." — [Chromium `components/webcrypto/README.md`](https://chromium.googlesource.com/chromium/src/+/main/components/webcrypto/README.md) |
| **WebCrypto** in Firefox | **NSS** (C) | `dom/crypto/WebCryptoTask.cpp` includes `cryptohi.h`/`pk11pub.h` — [searchfox](https://searchfox.org/firefox-main/source/dom/crypto/WebCryptoTask.cpp) |
| **Node `crypto`** | **OpenSSL** | "a set of wrappers for OpenSSL's hash, HMAC, cipher, decipher, sign, and verify functions" — [nodejs.org/api/crypto.html](https://nodejs.org/api/crypto.html) |

**Neither is JavaScript.** Effectively all production JS crypto is a thin binding over C
written by people who count cycles. That is the precedent Phalcom should copy, and it is
§1 restated as an observation about the world rather than a rule.

### 2.2 `crypto.timingSafeEqual` — the precedent, corrected

The thesis that commissioned this doc held that Node ships `timingSafeEqual` as a native
builtin *"precisely because it cannot be written in JS"*, and that Node's docs say so.
**Verified against the docs: they do not.** The entire documented rationale at
<https://nodejs.org/api/crypto.html#cryptotimingsafeequala-b> is:

> "This function compares the underlying bytes … using a constant-time algorithm." /
> "Use of `crypto.timingSafeEqual` does not guarantee that the *surrounding* code is
> timing-safe."

No claim about JS's inability. The function is nonetheless native and delegates to
OpenSSL's `CRYPTO_memcmp`
([`src/crypto/crypto_timing.cc`](https://github.com/nodejs/node/blob/main/src/crypto/crypto_timing.cc)).

**But the real evidence is better than the claimed evidence.** A source comment in that
same file reads:

> "Moving the type checking into JS leads to test failures, most likely due to V8 inlining
> certain parts of the wrapper. Therefore, keep them in C++."
> — referencing [nodejs/node#34073](https://github.com/nodejs/node/issues/34073)

That is a documented, in-production instance of **the JIT defeating hand-written JS
timing discipline** — and it defeated it in the *type-check wrapper*, not even the
comparison. It is a stronger citation for §4.1's argument than the one the thesis reached
for, and it should be cited *as what it is*: evidence about V8 inlining, not a Node
statement about JS's expressiveness.

Two costs Node pays, worth copying deliberately or not at all (see [bytes.md](bytes.md)
§8 and its B-4): it **throws on length mismatch**, leaking length through the exception
path; and it lives in `crypto`, so users who reach for `===` get a vulnerable comparison
with no diagnostic.

### 2.3 Pure-JS crypto exists — and why

It exists because of **gaps in the platform**, not because anyone thinks JS is a good
place to do this:

- **No secp256k1 in WebCrypto.** Verified directly: the full W3C Web Cryptography Level 2
  spec (<https://www.w3.org/TR/webcrypto-2/>, FPWD 22 Apr 2025) contains **zero**
  occurrences of `secp256k1`. §23.4 lists only `P-256`/`P-384`/`P-521` (i.e.
  secp256**r**1 — note the near-miss trap). So every Bitcoin/Ethereum JS wallet reaches
  for pure-JS ECC. *Caveat: "can never" would overstate — §23.4 carries an "Other
  specifications may define additional values" extension clause; nobody has specced it.
  And the causal chain "no secp256k1 ⇒ wallets use pure-JS" is well-supported inference,
  not a cited fact.*
- **Ed25519 was missing for years — but this is no longer true and must not be repeated
  as present-tense.** It is now in the W3C draft (§25/§26) and shipped in all three
  engines: Firefox 129 (Aug 2024), Safari 17.0, **Chrome 137 (May 2025)** — the last,
  after "a three-year collaboration"
  ([Igalia](https://blogs.igalia.com/jfernandez/2025/08/25/ed25519-support-lands-in-chrome-what-it-means-for-developers-and-the-web/)).
  The incubation repo [WICG/webcrypto-secure-curves](https://github.com/WICG/webcrypto-secure-curves)
  was **archived 2026-01-28**, folded into w3c/webcrypto. "Late" is fair; "missing" is
  stale.
- **Some environments have no WebCrypto at all.**

### 2.4 What `@noble` actually says — do not put words in their mouth

`@noble/curves` + `@noble/hashes` (Paul Miller) are the modern pure-JS standard. Their
posture is **more nuanced than "honest about its limits"** and more affirmative than the
thesis assumed. Both READMEs carry an identical §"Constant-timeness" block
(<https://github.com/paulmillr/noble-curves#constant-timeness>):

> "We're targetting algorithmic constant time. *JIT-compiler* and *Garbage Collector* make
> "constant time" extremely hard to achieve timing attack resistance in a scripting
> language. Which means *any other JS library can't have constant-timeness*. … **If your
> goal is absolute security, don't use any JS lib — including bindings to native ones. Use
> low-level libraries & languages.**"

On BigInt specifically — **this closes [bytes.md](bytes.md) §9.1's open uncertainty**
("*Unsure of the exact wording of noble's published rationale*"). The wording is:

> "All arithmetics is done with JS bigints over finite fields… **Field operations are not
> constant-time**: see security. The fact is mostly irrelevant, but the important method to
> keep in mind is `pow`, which may leak exponent bits, when used naïvely."

But noble also documents *measurable positive* properties, and a fair reading must include
them: signed fixed-window tables with **data-oblivious table scans**; **scalar blinding**
(`s + r·n` with a random 128-bit `r`); and a **dudect-style Welch t-test harness**
(`benchmark/ct.ts`, max |t| ≤ 2.8 at 1000 samples). They also self-disclose a hole:
non-base-point secret-scalar multiplication on cofactored Edwards curves is unblinded.

**The accurate summary: "algorithmic constant time + blinding + statistical evidence, with
JIT/GC as an acknowledged unmodellable residue, and a blunt recommendation not to use JS
if you need absolute security."** Not "we make no guarantees."

Others, with sharply varying posture: **tweetnacl-js** (strongest disclaimer — "impossible
to guarantee that they are physically constant time given JavaScript runtimes, JIT
compilers"; Cure53-audited 2017); **sjcl** (**"[DEPRECATED] … Please do not use it in new
projects"**, <https://github.com/bitwiseshiftleft/sjcl>); **elliptic** (**no timing warning
at all** — the weakest posture, and a widely-depended-on wallet dependency);
**node-forge** (no timing discussion; notes native code "may have stronger security
properties").

### 2.5 What pure-JS does to approximate constant time — and why it does not hold

The techniques are real and are C-level techniques, best documented by
[BearSSL](https://www.bearssl.org/constanttime.html): branchless conditional copy
performing "identical memory accesses regardless of the control word"; constant-time table
lookup by reading *all* entries; 32-bit all-ones/all-zeros control words instead of
booleans "so the compiler can't emit a jump"; **bitsliced** AES instead of T-tables;
Montgomery ladder / fixed-window scalar multiplication; XOR-accumulate comparison.

The T-table prohibition has a canonical source: Bernstein, *Cache-timing attacks on AES*
(2005-04-14, <https://cr.yp.to/antiforgery/cachetiming-20050414.pdf>) — "complete AES key
recovery from known-plaintext timings of a network server on another computer," and
notably "This attack should be blamed on the AES design, not on the particular AES library."

**Why it does not hold in JS:**

1. **The JIT.** V8 does type feedback, speculates, and deopts; it can reintroduce a branch
   you deliberately removed. There is no `black_box`, no optimization-barrier pragma, no
   control over the machine code. §2.2's Node source comment is a production instance.
2. **Number representation leaks.** V8 tags a small integer (`Smi`) inline —
   "A `0` indicates a 31-bit *Small Integer* (`SMI`). The actual integer value is stored
   directly" — while "Larger numbers or those with decimal parts are stored indirectly as
   immutable **`HeapNumber`** objects on the heap" ([v8.dev/blog/mutable-heap-number](https://v8.dev/blog/mutable-heap-number),
   [pointer-compression](https://v8.dev/blog/pointer-compression)). So a secret's
   *magnitude* can drive representation → allocation → timing. **Nuance the thesis got
   slightly wrong:** this is a representation-*transition* behavior, not an invariant —
   V8 also has double field unboxing and mutable heap numbers, and Smi width varies
   (31-bit under pointer compression, 32-bit payload uncompressed). The channel is real;
   "doubles are always heap-allocated" is overstated.
3. **BigInt is variable-time by construction** — variable width, magnitude-dependent work
   — and BigInt is exactly what you reach for to do modexp. Noble documents this (§2.4).

**Why the web is not on fire — threat model, not soundness.** Post-Spectre the browser took
the timer away: `performance.now()` is coarsened ("To offer protection against timing
attacks and fingerprinting, `performance.now()` is coarsened based on whether or not the
document is cross-origin isolated" — MDN documents **5 µs** isolated / **100 µs**
non-isolated tiers, <https://developer.mozilla.org/en-US/docs/Web/API/Performance/now>;
these are MDN's documented tiers, not per-browser measured values), and
`SharedArrayBuffer` is gated behind COOP/COEP ("Shared memory and high-resolution timers
were effectively disabled at the start of 2018 in light of Spectre" — MDN).

**But this is mitigation, not elimination.** Brumley & Boneh, *Remote Timing Attacks Are
Practical*, 12th USENIX Security Symposium, 2003
(<https://www.usenix.org/conference/12th-usenix-security-symposium/remote-timing-attacks-are-practical>)
extracted an RSA private key from an OpenSSL-backed web server **over a network**,
demolishing the assumption that timing attacks are a smartcard problem. Brumley & Tuveri,
*Remote Timing Attacks are Still Practical* (ESORICS 2011,
<https://eprint.iacr.org/2011/232.pdf>) confirmed it held. **Honest framing: pure-JS crypto
is fine until the attacker can time you precisely; the industry's answer is to not be in
that position.**

### 2.6 WASM — more predictable, guarantees nothing

WASM gives more predictable codegen without V8's value-speculation, which is why libraries
ship wasm builds: **libsodium.js** ("The sodium crypto library compiled to WebAssembly and
pure JavaScript using Emscripten", <https://github.com/jedisct1/libsodium.js>) and
**argon2-browser** (<https://github.com/antelle/argon2-browser>). *Adversarial note:
neither README warns that the wasm port has weaker side-channel properties than native.*

**The wasm spec makes no constant-time guarantee, and says so in its own words:**

> "**WebAssembly doesn't provide any guarantees against resistance to side-channel
> attacks**"
> — [WebAssembly/constant-time `Overview.md`](https://github.com/WebAssembly/constant-time/blob/main/proposals/constant-time/Overview.md)

**And the proposal that would have fixed it is dead.** The WebAssembly CG lists "Constant
Time" under [inactive-proposals.md](https://github.com/WebAssembly/proposals/blob/main/inactive-proposals.md),
where inactive means proposals "presented to the community group but were subsequently
abandoned, withdrawn, or rejected." (Academic origin: [CT-Wasm](https://github.com/PLSysSec/ct-wasm);
it proposed secret types `s32`/`s64` and classify/declassify.) So: *"WASM makes crypto
constant-time"* is false, and *"there's a WASM constant-time proposal"* needs the
qualifier **"abandoned."** And wasm still runs on a CPU with data-dependent instruction
timing (§3.2).

## 3. Rust is not a free pass

### 3.1 The libraries

**RustCrypto** — `sha2`, `aes-gcm`, `chacha20poly1305`. **The audit story is narrower than
folklore says:** `aes-gcm` and `chacha20poly1305` each received "one security audit by NCC
Group, with no significant findings … funded by MobileCoin"
([README](https://github.com/RustCrypto/AEADs/blob/master/aes-gcm/README.md);
[report, 2020-02-26](https://research.nccgroup.com/2020/02/26/public-report-rustcrypto-aes-gcm-and-chacha20poly1305-implementation-review/)).
**`sha2` carries no audit notice.** Do not claim it is audited.

**`ring`** — BoringSSL-derived ("Most of the C and assembly language code in *ring* comes
from BoringSSL", [README](https://github.com/briansmith/ring/blob/main/README.md)), with
`third_party/fiat/` generated by Fiat Cryptography. **The "with proofs" claim is
overstated and must be corrected wherever it is repeated:** fiat-crypto's own README
admits **"none of the other backends have any proofs about them"** — only the Bedrock2
backend has proofs relating emitted AST to internal AST semantics — and further concedes
"there is no verification that the particular integer size casts that we emit are
sufficient" for the downstream C compiler
(<https://github.com/mit-plv/fiat-crypto>). Accurate statement: *the algorithm-level field
arithmetic is proven; the C/Rust printer and the compiler below it are not, and ring
hand-edits the generated output* (files marked `NOTE: edited after generation`).

**dalek** — `curve25519-dalek` documents "constant-time logic (no secret-dependent
branches, no secret-dependent memory accesses)" with variable-time code explicitly marked
(<https://docs.rs/curve25519-dalek/>), and a fiat backend option. It also documents that it
"does not attempt to zero stack data." `ed25519-dalek` claims constant-time signing and
key zeroization on scope exit, but **no third-party audit**, and it carried a real
key-recovery flaw: RUSTSEC-2022-0093, "Double Public Key Signing Function Oracle Attack"
(<https://rustsec.org/advisories/RUSTSEC-2022-0093.html>), fixed by API redesign in 2.0.

### 3.2 Rust is NOT automatically constant-time — the strongest citation in this document

LLVM will optimize branchless code back into branches. This is why the **`subtle`** crate
exists (`ConstantTimeEq`, `Choice`, `read_volatile` optimization barriers) and why
RustCrypto depends on it. `subtle`'s **own docs** admit the limit:

> "It represents a best-effort attempt to protect against **some** software side-channels."
> "Because side-channel resistance is not a property of software alone, but of software
> together with hardware, any such effort is fundamentally limited. **USE AT YOUR OWN
> RISK**" — <https://docs.rs/subtle/latest/subtle/>

Its constant-time claim is *conditional on the compiler cooperating*: operations hold only
provided the bitwise ops are "not recognized as a conditional assignment and optimized back
into a branch." (Also: `subtle`'s debug-mode assertions "involve secret-dependent
branches" — "This crate is intended to be used in release mode.")

And the Rust project disclaims **the entire language**:

> "This limitation is not specific to `black_box`; **there is no mechanism in the entire
> Rust language that can provide the guarantees required for constant-time cryptography.
> (There is also no such mechanism in LLVM, so the same is true for every other LLVM-based
> compiler.)**" — <https://doc.rust-lang.org/std/hint/fn.black_box.html>

That parenthetical is decisive and should be quoted at anyone who proposes writing
constant-time code in Phalcom's Rust. Trail of Bits demonstrated `subtle`'s barrier
*failing on WebAssembly* ([Part 1: The life of an optimization
barrier](https://blog.trailofbits.com/2022/01/26/part-1-the-life-of-an-optimization-barrier/)),
and is now upstreaming `__builtin_ct_select` intrinsics into LLVM
([2025-12-02](https://blog.trailofbits.com/2025/12/02/introducing-constant-time-support-for-llvm-to-protect-cryptographic-code/),
[llvm PR #166702](https://github.com/llvm/llvm-project/pull/166702)) — with the Rust
compiler team "exploring how to expose these intrinsics." **Watch this: it is the first
mechanism that would let any of this be a guarantee rather than a hope.**

### 3.3 Below the compiler: the CPU

- **ARM `FEAT_DIT`** (Armv8.4-A, `PSTATE.DIT`) and **Intel `DOITM`** both exist. Both
  **default OFF for userspace**. oss-security, 2023-01-25: "on recent Intel and Arm CPUs,
  by default the execution time of instructions may depend on the data values operated on.
  This even includes instructions like additions, XORs, and AES instructions"
  (<https://www.openwall.com/lists/oss-security/2023/01/25/3>). Linux v6.2 enabled DIT
  kernel-side only; "userspace code will still get data-dependent timing by default"
  (<https://lwn.net/Articles/921511/>). Intel's own caveat: DOIT "is not expected to
  significantly improve resistance to side channel attacks unless the software was
  carefully written to avoid such attacks."
  *Sourcing caveat: developer.arm.com is JS-rendered and intel.com's DOIT pages 403'd the
  fetcher; the DIT wording above is quoted via [golang/go#49702](https://github.com/golang/go/issues/49702).
  Verify against the ARM ARM by hand before publishing.*
- **Variable-time multipliers — reframe the thesis's claim.** RustCrypto's AEAD READMEs
  name "certain 32-bit PowerPC CPUs and some non-ARM microcontrollers" — pointing *away*
  from ARM. The ARM case that does exist is Cortex-**M** (`umull`/`smull` on M3), not
  Cortex-A, and is weakly sourced. Do not assert "some ARM cores have data-dependent
  multiplier timing" without a TRM cite (**C-6**).
- **And the ground moves.** **Hertzbleed** (<https://www.hertzbleed.com/>, USENIX Sec
  1) — DVFS turns power channels into remote timing: "even when implemented correctly as
  constant time, cryptographic code can still leak via remote timing analysis."
  **GoFetch** (<https://gofetch.fail/>, USENIX Sec 2024) — Apple M-series data
  memory-dependent prefetcher: "even if a victim correctly separates data from addresses by
  following the constant-time paradigm, the DMP will generate secret-dependent memory
  access on the victim's behalf." Both are refutations of *constant-time source ⇒
  constant-time execution*, one at the frequency layer, one at the prefetcher.

**Net for §4: "constant time" is not a property Phalcom can deliver. It is a property a
carefully-written, expert-audited, hardware-aware library approximates, and that Phalcom
can at most avoid destroying.** That framing is the honest ceiling on every guarantee below.

## 4. What Phalcom can actually guarantee

### 4.1 Better than JS in one specific way: no JIT

**The single biggest thing defeating JS constant-time does not apply to Phalcom.** A
bytecode interpreter with no speculative machine-code generation is far more predictable
than V8. Phalcom emits no machine code, performs no type feedback, and cannot silently
reintroduce a branch a binding deliberately removed. §2.2's Node failure mode — V8 inlining
a wrapper until the timing test went red — has no Phalcom analogue.

**The nuance, stated precisely, because ADR-0018 makes the "no speculation" claim false as
written.** The sacred-selector inliner **is** speculative and **does** have a deopt path.
Verified against the tree (`phalcom-core/src/compiler/inliner.rs:1-95`):

- What it speculates on is the **syntactic shape of the call site** and the
  **pristineness of a sacred selector family** — i.e. whether `ifTrue(_)`/`and(_)`/
  `whileTrue(_)` have been *overridden*. `recognize` is "a purely syntactic,
  zero-runtime-cost check" over the AST (`inliner.rs:10-11`); the runtime guard is the
  `GuardBool`/`GuardBlock` opcode (`inliner.rs:12-14`), and the per-family
  `bool_sacred_pristine`/`block_sacred_pristine` flag forces deopt on redefinition
  (ADR-0018).
- **It does not speculate on secret values.** The speculation axis is *selector
  overriding* and *receiver type*, not the *magnitude or content* of a datum. A secret
  byte does not steer the guard.
- The inliner emits both paths and they are "built to be **observationally identical** in
  every case a Phalcom program can detect" (`inliner.rs:24-27`).

**So the precise claim is: Phalcom's one speculative mechanism speculates on program
shape, not on data, and therefore does not leak the secret through the deopt edge.** That
is a narrower and defensible statement. *Residual caveat (C-2): a `GuardBool` on a
condition derived from a secret still branches — but that branch exists in the source
already; the inliner does not create it. And "the interpreter is predictable" is a
statement about Phalcom, not about the CPU underneath it (§3.3) — Hertzbleed and GoFetch
apply to a bytecode interpreter exactly as they do to C.*

### 4.2 Worse than JS in one specific way: `Int` auto-promotion is a side channel

**This is the novel finding this document exists to record.** ADR-0024 (Accepted; **❌ not
built** — the tree is still flat, `class Number {}`) makes `Int` an exact **auto-promoting
bignum**: a tagged immediate on the small path, a heap `LargeInt` on overflow, with
`checked_*` promotion between them (ADR-0024 §2).

**Auto-promotion is value-dependent heap allocation.** Whether an operation allocates
depends on the *magnitude* of the operand. That is structurally identical to V8's
SMI→HeapNumber boxing (§2.5) — the very mechanism that makes JS number handling leak — and
allocation is observable. Under ADR-0050's mark-sweep it is doubly observable: an
allocation can trigger a **collection**, which is a large, obvious timing event whose
occurrence a secret's magnitude helps determine.

**The overlay already records ADR-0024 as in tension with ADR-0051 Tier 2's "kill per-send
alloc" goal ("accepted knowingly"). Nobody has recorded that it is also a side-channel
surface.** That is the gap. It is not an argument against ADR-0024 — exactness is the
right call and the tension was accepted with open eyes — it is an argument about **what
must never be routed through `Int`**.

**Consequence: crypto must use fixed-width [`Bytes`](bytes.md)/limbs and never route
secrets through `Int`.** Noble reached the identical conclusion about BigInt for the
identical reason, and §2.4 now supplies their exact wording, **closing
[bytes.md](bytes.md) §9.1's self-flagged uncertainty about that attribution.**

Because ADR-0024 is unbuilt, this is cheap to honor now and expensive later (**C-1**).

### 4.3 The honest guarantee list

What a Phalcom crypto binding could truthfully claim:

- ✅ **No JIT-reintroduced branches** (§4.1) — a real, differentiating property.
- ✅ **The algorithm runs in an audited native crate**, not in Phalcom (§1).
- ✅ **Misuse-resistance by construction** (§6) — the genuinely strong suit.
- ⚠️ **Constant-time comparison**, delegated to `subtle`, with `subtle`'s own caveats
  inherited verbatim (§3.2). Not a guarantee; a best effort with a citation.
- ❌ **Constant-time end to end.** Cannot be claimed. §3.2's `black_box` quote forecloses it
  at the language level; §3.3 forecloses it at the hardware level.
- ❌ **Secret erasure.** See §7.

## 5. Floor vs FFI — this belongs behind the door, not in the floor

[ffi.md](ffi.md) §2 works the census math and **its table is not repeated here.** The
summary: a minimum useful suite (hash init/update/final × several algorithms, AEAD
seal/open, sign/verify, keygen, KDF, CSPRNG) is roughly **32–48 bindings** against a floor
of **~113–117** (`floor-census.md` §1.1 says 113 bindings / 98 fns; §7's live
`VM::new()` audit asserts **117** — the spread is itself a recorded finding, ffi.md F-9).
A **~28–42% floor expansion for one capability domain** would be the largest in the
project's history, for a capability the object model does not presuppose.

The crypto-specific half of the argument, which ffi.md states and this doc endorses:

- **`equalsConstantTime_` passes ADR-0019's rule cleanly.** The security property *is* a
  statement about representation and execution timing, which `.ph` cannot express — a `.ph`
  loop over `at_` short-circuits. It presupposes control below the `.ph` boundary. See
  [bytes.md](bytes.md) §8; it is useful long before any crypto suite exists (HMAC/token
  comparison).
- **SHA-256 FAILS the rule as written, and this must be said out loud.** It is bit
  twiddling over bytes, perfectly expressible in `.ph` — `String#byteAt_(_)` (ADR-0049)
  already exposes the bytes. It would be derivable and unusably slow, and ADR-0019 says
  **speed is never sufficient**, naming the counter-move: "fund an inline cache or JIT
  *above* the floor." A crypto floor amendment is **a speed argument wearing a
  derivability costume** (ffi.md §2's phrasing).
- **The argument that *could* admit it is a different argument, and it is an amendment,
  not an application.** One could say: the *security property* is inexpressible even
  though the *function* is expressible — a `.ph` SHA-256 is not merely slow, it is
  **wrong**, because it leaks. That is a real and interesting claim. But ADR-0019's rule
  is written about **capabilities**, not about **properties of implementations of
  capabilities**. Admitting it would **rewrite the admission rule** to read "cannot be
  expressed *with its required non-functional properties*" — which is a much larger door
  than it looks, since "must be fast enough to be usable" is a non-functional property
  too, and that is precisely what ADR-0019 exists to refuse. **This deserves its own ADR
  and should not be smuggled in as an application of the existing rule** (**C-3**).
- **And the census math kills it anyway.** Even granting the amendment, 30–50 primitives
  do not become one. **Category error → FFI.**

The honest floor-shaped subset of "crypto" is roughly **one binding** (`randomBytes_` —
entropy is underivable in the strong sense: no `.ph` reaches it, because it is not
computation) plus constant-time compare. **Build the door, not 40 doorways.**

## 6. Misuse-resistant API surface — what Phalcom's committed design gives for free

This is the part where Phalcom's existing decisions pay an unplanned dividend.

### 6.1 Task-shaped, not primitive-shaped — follow libsodium/NaCl, not OpenSSL

Expose **tasks** — `seal`/`open`, `sign`/`verify`, `hash`, `kdf` — never primitives (raw
RSA, AES-CBC, a mode selector). **No algorithm agility in the API surface.** libsodium's
scope is to "provide all of the core operations needed to build higher-level cryptographic
tools" with "design choices emphasiz[ing] security and ease of use"
(<https://doc.libsodium.org/>) — task-shaped, not an algorithm buffet.

*Sourcing caveat: NaCl's own features page (nacl.cr.yp.to) would not load through the
fetcher; its "no data flow from secrets to load addresses / branch conditions" list is
widely attested but was **not** read directly here. Verify by hand before quoting it.*

**What agility costs**, concretely — the JWT `alg` field, the canonical case.
**[sealed-classes.md](sealed-classes.md) §5 owns this bug class and carries the verified
CVE table (CVE-2016-10555, CVE-2017-11424, CVE-2018-1000531, CVE-2022-29217,
CVE-2022-23540); it is not duplicated here.** In brief
([Auth0, *Critical vulnerabilities in JSON Web Token libraries*](https://auth0.com/blog/critical-vulnerabilities-in-json-web-token-libraries/)):
`alg: none` — "some libraries treated tokens signed with the `none` algorithm as a valid
token with a verified signature"; and RSA→HMAC confusion — "If a server is expecting a
token signed with RSA, but actually receives a token signed with HMAC, it will think the
public key is actually an HMAC secret key," turning the *public* key into the forgery key.
Root cause: **the attacker-supplied message selects the algorithm.**

*Honest note: the thesis asked to "cite what OpenSSL's agility cost the world." The JWT
case is solid and citable. **A specific, citable indictment of OpenSSL's API complexity
causing misuse was not found** — the closest defensible evidence is Blessing/Specter/
Weitzner (§1), which correlates complexity with vulnerability frequency across crypto
libraries generally, not OpenSSL specifically. Do not assert the OpenSSL half without a
source (**C-7**).*

### 6.2 Four committed decisions that are misuse-resistance features here

| Decision | Why it helps crypto |
|---|---|
| **No default arguments** (ADR-0043) | No `encrypt(data, mode = CBC)` footgun. Every arity explicit, 1:1 with selector identity. **The cost ADR-0043 apologized for — repetitive manual arity-overloads — is a safety feature in this domain.** With ADR-0025 labels it reads `seal(plaintext, with: key, nonce: n)`: the nonce cannot be silently defaulted, because there is no defaulting mechanism to abuse. |
| **Default `toString` is `<ClassName>`** (ADR-0015) | A `SecretKey` renders `<SecretKey>`, not its bytes. **Safe by default, free, no opt-in.** Contrast every language where `print(key)` dumps the buffer. |
| **No truthiness + `GuardBool`** (ADR-0021) | `verify` cannot return something falsy you forget to check; a non-`Bool` condition is rejected at runtime by the floor, with **no coercion**. Apple's "goto fail" was exactly a skipped verify path. Both enforcement layers apply (runtime `GuardBool` + compile-time rejection of syntactically-literal Option conditions). |
| **Sealed algorithm sets** ([sealed-classes.md](sealed-classes.md)) | A sealed algorithm set blocks algorithm-confusion (the JWT `alg: none` class) structurally — **and this is not a wish: the `@sealed` mechanism already shipped**, consumed by commit `8d401f4` to seal `Option`/`Some`/`None` for ADR-0044's bootstrap reasons ([sealed-classes.md](sealed-classes.md) §1.1–1.2). **[sealed-classes.md](sealed-classes.md) §5 owns the algorithm-confusion case, including the verified CVE table; it is not repeated here**, and that doc explicitly defers the crypto surface itself to this one (its §5 preamble). ⚠️ **The limit, from its §4: sealing does not touch method reopening** — a sealed class's *methods* remain open (ADR-0026/0041: methods open, epoch-guarded; only the superclass is sealed). So sealing closes the *subclassing* half of §8's first hazard and leaves the *reopening* half open (**C-4**). |

### 6.3 A verified defect: a redacting `toString` is bypassed by string interpolation

§6.2's ADR-0015 dividend has a hole. **Verified against the tree, file:line by file:line:**

1. ADR-0022 desugars `\(x)` to **`String.new(x)`**, not `x.toString`
   (`phalcom-ast/src/parser.rs:1649-1653`: "wrapped as `String.new(expr)` — the working
   [content-stringify primitive]"). ADR-0022 §Decision states this explicitly.
2. `string_class_new` (`phalcom-core/src/primitive/string.rs:55`) calls
   **`arg.to_string(vm)`** — *not* `to_display_string`.
3. Only **`Value::to_display_string`** (`phalcom-core/src/value/render.rs:80`) sends the
   `toString` message (`render.rs:87-89`: `vm.get_or_intern("toString")` →
   `vm.send_dynamic`). It sends it for exactly the objects `to_string` has no bespoke
   native renderer for — which **includes a plain instance**.
4. **`Value::to_string`** (`render.rs:19`) falls through to **`to_debug`**
   (`render.rs:98`) for a plain instance (`render.rs:50`: `_ => self.to_debug(vm)`).

**Therefore a redacting `toString` override is bypassed by string interpolation.**
`"key: \(secretKey)"` does not call the override — it renders via `to_debug`.

**Today this is not a byte leak.** `Instance::to_debug` (`phalcom-core/src/heap/instance.rs:31`)
is `format!("<{} instance>", heap.class(self.class).name)` — the class name only, **no
field contents**. So `"key: \(secretKey)"` yields `key: <SecretKey instance>`. The
information disclosed is nil; the *override bypass* is real.

**Why it matters anyway:** any future enrichment of `to_debug` — dumping slots, a common
and entirely reasonable debug convenience — **silently becomes a leak at every
interpolation site in the language**, with no test that would go red and no reviewer
prompt. The mechanism that is supposed to prevent it (an overridden `toString`) is not on
the path.

**Provenance:** this is the **un-fixed half of U-ERR-FIX's BUG-PRINT-TOSTRING.** That fix
introduced `to_display_string` and routed **`System.print`** through it —
`render.rs:70-76` documents the case it closed ("a bare `Point` instance printed
`<Point instance>` via `System.print` but `<Point>` via `.toString`… so `System.print` and
an explicit `.toString` send always agree (U-ERR-FIX PRINT-TOSTRING)"). **The
interpolation path was not routed.** `String.new(_)` still calls `to_string`.

**The fix is one call site** — `string_class_new` calling `to_display_string` instead of
`to_string` — but it is not free: it makes `String.new(_)` fallible (a user `toString`
override can throw; the signature already returns `PhResult`) and re-entrant into the
interpreter from a primitive, which is exactly the shape ADR-0030 restricts (a native
re-entrant frame raises `CannotYieldAcrossNativeFrame`). ADR-0022 anticipated revisiting
the target "when U-CORE-4 lands a real content `toString`." **Recorded here as a finding,
not a proposal** (**C-5**).

## 7. Zeroization — the casualty

**[bytes.md](bytes.md) §7 owns this and is not repeated here.** Its conclusion stands
unchanged for crypto: ADR-0050's mark-sweep means **no deterministic destruction, no
`Drop`**; `zeroize` depends entirely on `Drop`; secrets linger in the arena until
collected, possibly forever. The precedents are a graveyard — Java's `char[]`-over-`String`
advice, .NET's `SecureString` deprecated as unfixable. The offer is an explicit
`key.zeroize` as a **documented obligation** with `ensure`-scoped lifetimes, and the
scoping **is** sound because ADR-0008 §4 unified `return`/`throw`/fiber `abort` into one
unwind. Enforcement is **written contract + golden test**, matching ADR-0052's precedent —
no flow analysis exists. **State plainly: this is not fully fixable.**

Three crypto-specific additions to that section:

1. **The precedents verify, with sharper wording than bytes.md quotes.** Microsoft's own
   guidance: "We recommend that you don't use the `SecureString` class for new development"
   (<https://learn.microsoft.com/en-us/dotnet/api/system.security.securestring>); DE0001
   is blunter — "it just makes the window getting the plain text shorter; it doesn't fully
   prevent it as .NET still has to convert the string to a plain text representation"
   (<https://github.com/dotnet/platform-compat/blob/master/docs/DE0001.md>). Java:
   `JPasswordField.getPassword()` — "it is recommended that the returned character array be
   cleared after use by setting each character to zero"; `getText()` is deprecated "For
   security reasons."
2. **`zeroize`'s own documented limits transfer, and they are the *ceiling* on what
   Phalcom could offer even with `Drop`.** It cannot "guarantee copies of the data were not
   previously made by buffer reallocation"; "stack spilling and other optimizations may
   leave temporary copies"; and it "makes no guarantees that zeroized values cannot be
   leaked through" microarchitectural covert channels
   (<https://docs.rs/zeroize/latest/zeroize/>). Note the asymmetry vs `subtle`: zeroize
   claims a *real* volatile-backed guarantee against the optimizer; its gaps are copies and
   microarchitecture. **Even Rust-with-`Drop` does not fully solve this. Phalcom is not
   failing to reach a bar someone else cleared.**
3. **`curve25519-dalek` "does not attempt to zero stack data"** (§3.1). A bound crate's
   own zeroization is partial, so Phalcom's obligation contract inherits a partial
   guarantee even if `.ph` honors it perfectly.

## 8. Interaction hazards

Named in the project's standing vocabulary:

- **dynamic power ⊗ untrusted input.** Phalcom has `doesNotUnderstand(_)`, `perform`, and
  open methods (ADR-0026/0041: *methods open*, reopen/redefine at runtime). A crypto API is
  the highest-value target for exactly this. **Sealing answers half of it and only half**:
  [sealed-classes.md](sealed-classes.md) verifies `@sealed` shipped (§1) — so an algorithm
  set cannot be *extended* with a rogue variant, which is the JWT bug class (its §5) — but
  its §4 verifies that **method reopening is deliberately untouched**. Nothing structurally
  stops a library from reopening `SecretKey` and redefining `toString`, or from
  `perform`-ing a selector derived from attacker-influenced data. The subclassing half is
  closed; the reopening half is the hazard with no committed answer (C-4).
- **speculative optimization ⊗ observable semantics.** ADR-0018's inliner is the project's
  canonical resolved case (guard failure "observably identical to the slow path"). §4.1
  argues it does not extend to a *timing* observable because the speculation axis is
  selector-pristineness, not data. **That argument should be checked by someone else
  before it is relied on** (C-2) — the existing ADR-0018 guarantee is about *program-visible
  behavior*, and timing is not program-visible in the sense the ADR means. The two notions
  of "observable" are not the same notion, and this doc is asserting a bridge between them.
- **cleanup ordering ⊗ unwinding.** [bytes.md](bytes.md) §9's finding transfers: `ensure`
  is sound against *all* unwind paths (ADR-0008 §4), but `ensure` blocks nest, and a
  `zeroize` in an outer `ensure` runs *after* an inner scope may have already passed the
  value onward. Scoping is a contract about the **whole lifetime**, not one frame.
- **value-dependent allocation ⊗ constant time.** §4.2 — the `Int` channel. The one hazard
  this document contributes.
- **zeroization ⊗ moving GC.** [bytes.md](bytes.md) §7.2 — admitting a zeroization
  contract converts ADR-0050's "reversibly open" moving-GC door into a security-relevant
  one. Crypto is the reason that door matters.

## 9. Open questions

Numbered for citation. Add rows; do not renumber.

| # | Question | Why it is open | Would resolve via |
|---|---|---|---|
| **C-1** | Should ADR-0024 record that `Int` auto-promotion is a side-channel surface, not only an allocation-cost tension? | §4.2. The overlay records the ADR-0051 tension as "accepted knowingly"; the security consequence is unrecorded. ADR-0024 is **unbuilt**, so a note costs nothing now. | An ADR-0024 amendment or an overlay row. Cheapest before U-NUMERIC. |
| **C-2** | Is §4.1's claim — ADR-0018 speculates on shape, not data, therefore no secret leaks via deopt — actually airtight? | Asserted from `inliner.rs:1-95`, not proven. ADR-0018's "observationally identical" guarantee is about *program-visible behavior*; **timing is not program-visible in that sense**. §8 flags the bridge. | Adversarial review by someone who did not write this. Possibly a dudect-style harness (noble's precedent, §2.4). |
| **C-3** | Is "the *security property* is inexpressible even though the *function* is expressible" a legitimate extension of ADR-0019's admission rule? | §5. It is a **real amendment**, not an application — and it generalizes dangerously ("must be fast enough" is a non-functional property too, which ADR-0019 exists to refuse). | Its own ADR. Must not be smuggled in as an application of the existing rule. |
| **C-4** | Sealing blocks *subclassing* a crypto algorithm set, but not *reopening* its methods. Is that enough for the misuse-resistance claim in §6.2? | §6.2, §8. [sealed-classes.md](sealed-classes.md) §1 verifies `@sealed` shipped (`8d401f4`); its §4 verifies method reopening is deliberately untouched (ADR-0026/0041). So a sealed `Cipher` set cannot be *extended* with a rogue algorithm, but `Cipher >> verify` can still be *redefined* at runtime. For the JWT bug class §5-of-that-doc addresses, sealing is sufficient; for a hostile-library threat model it is not. | A ruling on whether crypto classes need a *stronger* guarantee than the rest of the language (i.e. non-reopenable methods) — which would be a new mechanism, not an application of sealing. |
| **C-5** | Should `String.new(_)` route through `to_display_string`, closing the interpolation override bypass? | §6.3. Verified defect, currently harmless (`to_debug` leaks no fields), latent if `to_debug` is ever enriched. The fix makes `String.new(_)` fallible + re-entrant from a primitive (ADR-0030's restricted shape). ADR-0022 anticipated revisiting at U-CORE-4. | A ruling + a unit. It is the un-fixed half of U-ERR-FIX's BUG-PRINT-TOSTRING. |
| **C-6** | Is the "data-dependent multiplier" claim true for any CPU Phalcom targets? | §3.3. RustCrypto names 32-bit PowerPC / non-ARM MCUs, pointing *away* from ARM; the ARM case is Cortex-**M**, weakly sourced. The thesis's framing was wrong. | A TRM cite, or drop the claim. DIT/DOIT defaulting off is the better-sourced fact and carries the argument alone. |
| **C-7** | What did OpenSSL's algorithm agility actually cost, citably? | §6.1. The JWT `alg` case is solid. A specific citable indictment of *OpenSSL's* API complexity → misuse **was not found**. | A real source, or restrict the claim to JWT + Blessing/Specter/Weitzner's complexity correlation. |
| **C-8** | Which crate(s) would Phalcom bind? | §1 says bind, not build; it does not say what. RustCrypto (`sha2` **unaudited**, AEADs NCC-audited), `ring` (BoringSSL-derived, fiat-generated but **not** fully proven), dalek (`ed25519-dalek` **unaudited**, RUSTSEC-2022-0093 history). None is unambiguously the answer. | An evaluation, gated on FFI existing (ffi.md F-2/F-5). |
| **C-9** | Does binding a crypto crate force finalizers? | ffi.md F-3 generalizes: native key handles are the classic reason a language grows finalizers, and ADR-0050 §Context banks "No finalizers exist" as a reason the collector is hazard-free. §7's `ensure`-obligation is the alternative. | The same ruling as ffi.md F-3. Crypto is its sharpest instance. |
| **C-10** | Do LLVM's incoming `__builtin_ct_select` intrinsics change any of this? | §3.2. It is the first mechanism that would let a binding claim constant-time as a *guarantee*. Rust exposure is "being explored." | Watch [llvm#166702](https://github.com/llvm/llvm-project/pull/166702) and the Rust `core::intrinsics` discussion. Revisit §4.3's ❌ row if it lands. |

## 10. What this document precludes

**Nothing.** It is a draft with no owning unit. It is recorded so that:

1. The next person who proposes a crypto floor amendment finds §5 and [ffi.md](ffi.md) §2
   one link away, and has to argue against the census math and ADR-0019's rule rather than
   around them.
2. **§4.2's `Int` finding is on the record before ADR-0024 is built** — which is the one
   thing here with a closing window.
3. §6.3's interpolation defect is written down with `file:line` before someone enriches
   `to_debug` and turns it into a live leak.
4. Anyone tempted to write a hash function in Rust "because Rust is safe" first reads §1's
   27.2%/37.2% split and §3.2's *"there is no mechanism in the entire Rust language."*

## 11. References

**Ecosystem / JS**
- Chromium webcrypto README (BoringSSL) — <https://chromium.googlesource.com/chromium/src/+/main/components/webcrypto/README.md>
- Firefox `WebCryptoTask.cpp` (NSS) — <https://searchfox.org/firefox-main/source/dom/crypto/WebCryptoTask.cpp>
- Node `crypto` docs / `timingSafeEqual` — <https://nodejs.org/api/crypto.html#cryptotimingsafeequala-b> · source: <https://github.com/nodejs/node/blob/main/src/crypto/crypto_timing.cc> · V8-inlining issue: <https://github.com/nodejs/node/issues/34073>
- W3C Web Cryptography API Level 2 (FPWD 2025-04-22) — <https://www.w3.org/TR/webcrypto-2/>
- Ed25519 in Chrome 137 (Igalia) — <https://blogs.igalia.com/jfernandez/2025/08/25/ed25519-support-lands-in-chrome-what-it-means-for-developers-and-the-web/> · archived WICG repo: <https://github.com/WICG/webcrypto-secure-curves>
- noble-curves §Constant-timeness — <https://github.com/paulmillr/noble-curves#constant-timeness> · noble-hashes — <https://github.com/paulmillr/noble-hashes#constant-timeness>
- tweetnacl-js — <https://github.com/dchest/tweetnacl-js> · sjcl (**deprecated**) — <https://github.com/bitwiseshiftleft/sjcl> · elliptic — <https://github.com/indutny/elliptic> · node-forge — <https://github.com/digitalbazaar/forge>
- V8 Smi/HeapNumber — <https://v8.dev/blog/mutable-heap-number> · <https://v8.dev/blog/pointer-compression>
- MDN `performance.now()` coarsening — <https://developer.mozilla.org/en-US/docs/Web/API/Performance/now> · MDN `SharedArrayBuffer` / COOP+COEP — <https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/SharedArrayBuffer>

**WASM**
- WebAssembly constant-time proposal Overview (no CT guarantee) — <https://github.com/WebAssembly/constant-time/blob/main/proposals/constant-time/Overview.md> · **inactive** status: <https://github.com/WebAssembly/proposals/blob/main/inactive-proposals.md> · CT-Wasm: <https://github.com/PLSysSec/ct-wasm>
- libsodium.js — <https://github.com/jedisct1/libsodium.js> · argon2-browser — <https://github.com/antelle/argon2-browser>

**Rust**
- `subtle` (USE AT YOUR OWN RISK) — <https://docs.rs/subtle/latest/subtle/>
- `std::hint::black_box` ("no mechanism in the entire Rust language") — <https://doc.rust-lang.org/std/hint/fn.black_box.html>
- RustCrypto AEADs + NCC audit — <https://github.com/RustCrypto/AEADs/blob/master/aes-gcm/README.md> · <https://research.nccgroup.com/2020/02/26/public-report-rustcrypto-aes-gcm-and-chacha20poly1305-implementation-review/>
- `ring` — <https://github.com/briansmith/ring/blob/main/README.md> · fiat-crypto ("none of the other backends have any proofs") — <https://github.com/mit-plv/fiat-crypto>
- curve25519-dalek — <https://docs.rs/curve25519-dalek/> · RUSTSEC-2022-0093 — <https://rustsec.org/advisories/RUSTSEC-2022-0093.html>
- `zeroize` — <https://docs.rs/zeroize/latest/zeroize/>
- Trail of Bits, optimization barriers — <https://blog.trailofbits.com/2022/01/26/part-1-the-life-of-an-optimization-barrier/> · LLVM constant-time support — <https://blog.trailofbits.com/2025/12/02/introducing-constant-time-support-for-llvm-to-protect-cryptographic-code/> · <https://github.com/llvm/llvm-project/pull/166702>

**Attacks & hardware**
- Brumley & Boneh, *Remote Timing Attacks Are Practical* (USENIX Sec 2003) — <https://www.usenix.org/conference/12th-usenix-security-symposium/remote-timing-attacks-are-practical> · *Still Practical* (2011) — <https://eprint.iacr.org/2011/232.pdf>
- Bernstein, *Cache-timing attacks on AES* (2005) — <https://cr.yp.to/antiforgery/cachetiming-20050414.pdf>
- BearSSL constant-time programming — <https://www.bearssl.org/constanttime.html>
- Hertzbleed — <https://www.hertzbleed.com/> · GoFetch — <https://gofetch.fail/>
- DIT/DOIT default-off — <https://www.openwall.com/lists/oss-security/2023/01/25/3> · <https://lwn.net/Articles/921511/>

**Design posture**
- Schneier, *Memo to the Amateur Cipher Designer* — <https://www.schneier.com/crypto-gram/archives/1998/1015.html> · Schneier's Law — <https://www.schneier.com/blog/archives/2011/04/schneiers_law.html>
- Blessing, Specter & Weitzner, *You Really Shouldn't Roll Your Own Crypto* — <https://arxiv.org/abs/2107.04940>
- libsodium docs — <https://doc.libsodium.org/>
- Auth0, *Critical vulnerabilities in JSON Web Token libraries* — <https://auth0.com/blog/critical-vulnerabilities-in-json-web-token-libraries/>
- .NET `SecureString` — <https://learn.microsoft.com/en-us/dotnet/api/system.security.securestring> · DE0001 — <https://github.com/dotnet/platform-compat/blob/master/docs/DE0001.md>
