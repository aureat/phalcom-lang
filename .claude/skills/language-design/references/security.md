# Security & Robustness

> Generic design-space layer of the `language-design` skill — matrices + hazards, no textbook prose. Phalcom's committed choice on any axis here: see [../phalcom/overlay.md](../phalcom/overlay.md).
> **Load when:** designing/critiquing memory safety, untrusted-code execution, sandboxing, resource limits, FFI/unsafe boundaries, or runtime robustness.

## Contents
- Axis 1 — Memory safety model
- Axis 2 — Bad input: panic vs UB vs recoverable
- Axis 3 — Untrusted code execution & sandboxing
- Axis 4 — eval / dynamic loading / deserialization
- Axis 5 — Integer & numeric safety
- Axis 6 — Resource exhaustion / DoS
- Axis 7 — FFI / `unsafe` trust boundary
- Axis 8 — Untrusted bytecode & input validation

## Axis 1 — Memory safety model
| Option | Langs | Consequence |
|---|---|---|
| GC / managed | JS, Java, Python, Go, Smalltalk | UAF/double-free impossible; OOB checked; pauses, no lifetime control |
| Ownership + borrow | Rust | UAF/data-race compile-rejected; `unsafe` is the only hole |
| Bounds-checked + escape hatch | Java, C#, Rust `unsafe` | Safe by default; each escape reintroduces the full bug class |
| Manual malloc/free | C, C++ | UAF, double-free, OOB all latent; every bug exploitable |
| Region / arena | Zig alloc, Cyclone | Bulk-free by scope; UAF if a ref outlives its region |

**Impl.** GC (tracing/refcount) makes reachability decide liveness; ownership makes lifetimes static; the escape hatch localizes danger to audited spans.
**Hazard — safety guarantee ⊗ one unsound `unsafe`.** A single `unsafe` block that violates aliasing/lifetime invariants voids the *whole* language's memory-safety claim — the compiler trusts it and stops checking. → overlay

## Axis 2 — Bad input: panic vs UB vs recoverable
| Option | Langs | Consequence |
|---|---|---|
| Recoverable error value | Rust `Result`, Go | Every malformed input has a defined, handleable outcome |
| Panic / trap (safe abort) | Rust `panic!`, Swift trap | Defined: unwinds/aborts, no corruption; still a DoS if reachable |
| Exception | Java, Python, Ruby | Defined; unwinds to handler; invisible control flow |
| Undefined behavior | C, C++ signed overflow, OOB | Attacker-controlled; the exploit-primitive source |

**Impl.** A VM must map *every* malformed program/input to a defined error or diagnostic — never to UB, and (for user input) never to a raw `panic!` (see errors.md Axis 5).
**Hazard — `panic!` on user input ⊗ robustness.** In a Rust VM every `panic!`/`unwrap`/`unreachable!` reachable from source or bytecode is a robustness bug: it is a defined abort but still a trivial remote DoS. Convert to a diagnostic, not an abort. → overlay
**Hazard — UB as "can't happen".** Treating a malformed-input state as unreachable (C-style) hands the attacker the primitive; the case must be a defined error, not an assumption.

## Axis 3 — Untrusted code execution & sandboxing
| Option | Langs | Consequence |
|---|---|---|
| Capability-based (no ambient authority) | E, Newspeak, Wasm(WASI) | Code can only touch what it's handed; least-privilege by construction |
| VM/Wasm sandbox | Wasm, JVM SecurityManager(dead), Lua | Linear memory / verified bytecode fences the guest; escapes = full compromise |
| OS/process isolation | seccomp, containers, V8 isolates+broker | Kernel enforces; coarse, heavyweight, survives VM bugs |
| Same-process `eval` | JS, Python, Ruby | Porous — guest shares heap & ambient authority with host |

**Syntax.** Capability tokens are plain object references passed in (`fs.open` given, not imported); ambient `import os` / global `File` is the anti-pattern.
**Impl.** Capability safety needs no global mutable namespace reaching authority: authority flows only through references. Wasm fences via linear memory + typed imports.
**Hazard — ambient authority ⊗ untrusted guest.** If guest code reaches host capability through globals/reflection (not just explicitly-passed refs), the sandbox is porous — same-process eval cannot be secured by blacklisting. → overlay

## Axis 4 — eval / dynamic loading / deserialization
| Option | Langs | Consequence |
|---|---|---|
| First-class `eval` of source | JS, Python, Ruby, Lisp | Max metaprogramming; direct code-injection surface |
| Object deserialization that runs code | Python `pickle`, Ruby `Marshal`/YAML, Java | Decoding attacker bytes → RCE (gadget chains) |
| Prototype/class mutation at runtime | JS `__proto__`, Ruby monkeypatch | Prototype pollution; global behavior hijack from data |
| Data-only deserialization | JSON, `serde` typed | No code path; safe if schema-validated |
| No eval; static loading only | Rust, Go, compiled | Injection surface closed; loses runtime codegen |

**Syntax.** `eval(userStr)` · `pickle.loads(bytes)` · `JSON.parse` merging into `obj.__proto__` · `Marshal.load`. Data-only: `serde_json::from_str::<T>`.
**Impl.** Power = "turn data into executing code/behavior"; safety = keep a hard wall between the data plane and the code plane.
**Hazard — metaprogramming/eval ⊗ untrusted input.** Any facility that materializes data into code or live behavior (eval, code-generating deserialization, runtime class mutation) becomes an injection primitive the instant its input is attacker-influenced. Dynamic power ⊗ untrusted bytes = RCE. → overlay

## Axis 5 — Integer & numeric safety
| Option | Langs | Consequence |
|---|---|---|
| Wrapping (2's complement) | C unsigned, Rust `wrapping_*`, Java | Deterministic but silent; length/index math overflows quietly |
| Trap / panic on overflow | Swift, Rust debug builds | Overflow becomes a defined abort, not a corrupt value |
| Checked (`Option`/`Result`) | Rust `checked_*` | Caller must handle; no silent wrap; verbose |
| Saturating | Rust `saturating_*`, DSP | Clamps to bound; no wrap; hides magnitude loss |
| Arbitrary precision (bignum) | Python, Ruby, Lisp | No overflow ever; allocation/DoS cost, boxing |
| UB on signed overflow | C, C++ | Optimizer-exploitable; the classic exploit primitive |

**Impl.** Also define NaN/±Inf semantics (float compare, NaN≠NaN, div-by-zero) and int↔bignum promotion rules; a VM computing sizes/indices must never wrap silently into an OOB.
**Hazard — silent wrap ⊗ allocation size.** An overflowing length/capacity computation that wraps (not traps) produces a tiny allocation for a huge logical size → later OOB write. Size arithmetic must be checked. → overlay
**Hazard — bignum promotion ⊗ DoS.** Auto-promotion to arbitrary precision turns `2**n` on attacker `n` into unbounded allocation/CPU (Axis 6).

## Axis 6 — Resource exhaustion / DoS
| Option | Langs | Consequence |
|---|---|---|
| Bounded call/recursion depth | JVM `StackOverflowError`, guard page | Deep/infinite recursion → defined error, not memory scribble |
| Native stack, unbounded | C recursion | Deep recursion smashes past the guard page → UB |
| Allocation / heap caps | Wasm mem limit, `ulimit`, gas | Runaway alloc bounded; needs a cap knob to exist |
| Instruction/time budget (gas) | EVM gas, watchdog | Infinite loops terminated; per-op accounting cost |
| Randomized/DoS-resistant hashing | Rust `SipHash`, Python `PYTHONHASHSEED` | Hash-flood O(n²) collisions defeated; slower hash |
| Backtracking regex | PCRE, JS, Python `re` | ReDoS: catastrophic backtracking on crafted input |

**Impl.** Robustness needs explicit depth + allocation + time limits and non-deterministic hash seeding; DFA/linear regex (RE2/`regex`) removes ReDoS by construction.
**Hazard — unbounded recursion/alloc ⊗ no limits.** Without depth/alloc/time caps a 10-line script DoSes the host: infinite recursion (stack), `[0]*huge` (heap), or a tight loop (CPU). The VM, not the guest, must own the ceiling. → overlay
**Hazard — hash table ⊗ adversarial keys.** Non-randomized string/hash keys let an attacker force all inserts into one bucket → O(n²) hash flooding. Seed the hasher per-process. → overlay
**Hazard — backtracking regex ⊗ user pattern/input.** `(a+)+$` on adversarial input is exponential; either ban backtracking engines or bound match steps.

## Axis 7 — FFI / `unsafe` trust boundary
| Option | Langs | Consequence |
|---|---|---|
| `unsafe {}` blocks | Rust | Danger localized & greppable; soundness manually owed |
| Native call bridge | JNI, Python C-API, Lua C API | Guest safety ends at the boundary; C bugs are host bugs |
| Safe wrapper over unsafe core | Rust `Vec`/`std` | Invariants enforced at the API edge; audited once, used freely |
| No FFI / pure | pure Wasm, sandboxed | No native hole; loses OS/native reach |

**Syntax.** Rust `unsafe { *ptr }` · `extern "C" fn` · `#[no_mangle]` · JNI `native` methods · Lua `luaL_check*` at the C boundary.
**Impl.** Every native/`unsafe` call is a hole in the guarantee: minimize surface, encapsulate behind a safe API that upholds the invariants callers rely on, audit each site.
**Hazard — `unsafe`/FFI ⊗ safety guarantee.** The boundary is where a memory-safe language meets unchecked code; one wrong pointer/lifetime/aliasing assumption there is UB that corrupts the whole managed heap — and the borrow checker will never flag it. Minimize and audit. → overlay
**Hazard — panic across FFI ⊗ abort.** A Rust `panic!` unwinding into `extern "C"` is UB; native boundaries must `catch_unwind` and convert to a code (see errors.md Axis 5). → overlay

## Axis 8 — Untrusted bytecode & input validation
| Option | Langs | Consequence |
|---|---|---|
| Verify before execute | JVM bytecode verifier, Wasm validation | Forged/ill-typed bytecode rejected pre-run; type safety upheld |
| Trust the loader | CPython `.pyc`, unverified VMs | Handcrafted bytecode → type confusion, memory corruption |
| Re-derive safety at runtime | interpreter re-checks each op | No trust in the stream; per-op checking cost |
| Sign/authenticate the artifact | code signing | Provenance, not well-formedness — a signed-but-buggy blob still runs |

**Impl.** A VM ingesting external bytecode must verify stack balance, operand types, jump targets in-range, and constant-pool indices *before* dispatch — or the interpreter trusts a forged type and reads it as the wrong repr.
**Hazard — missing bytecode verification ⊗ type confusion.** Unverified bytecode lets an attacker forge an op that treats an integer as a pointer (or overruns the operand stack) → the type system is bypassed → memory corruption in a "safe" VM. Verify or re-check every op. → overlay
**Hazard — parser/loader ⊗ malformed input.** The parser and bytecode loader are the robustness substrate: harden them and fuzz continuously (libFuzzer/AFL), run the corpus under miri/ASan, treat any crash/UB on malformed input as a security bug. → overlay
