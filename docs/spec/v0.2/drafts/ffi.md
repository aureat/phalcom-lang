# FFI & native modules — the door that is not the floor

- Status: **Draft** (exploration only — not proposed, not ratified, no owning unit)
- Date: 2026-07-15
- Depends on:
  [ADR-0019](../../../adr/accepted/0019-freeze-vm-blessed-primitive-floor.md) (the floor admission rule) ·
  [ADR-0009](../../../adr/accepted/0009-handle-arena-heap.md) (handle/arena heap) ·
  [ADR-0050](../../../adr/accepted/0050-non-moving-mark-sweep-collector.md) (non-moving mark-sweep) ·
  [ADR-0045](../../../adr/accepted/0045-module-import-relative-path-whole-module-binding.md) (imports resolve to source only) ·
  [floor-census.md](../core/floor-census.md) (authoritative floor counts)
- Related:
  [ADR-0049](../../../adr/accepted/0049-amend-floor-admit-string-byte-and-raw-write-primitives.md) (`System.write_`, trailing-`_` convention) ·
  [ADR-0058](../../../adr/accepted/0058-reactive-tracking-context-needs-a-native-module.md) (native `Reactive` module) ·
  [ADR-0020](../../../adr/accepted/0020-kernel-list-native-array-protocol.md) (native-backed kernel collection) ·
  [bootstrapping-and-self-hosting.md](../experimental/bootstrapping-and-self-hosting.md) §D6 (bytecode verifier)

> **How to use this document.** An exploration doc, not a spec. It grows. Nothing
> here is committed or citable as a decision. Sections are written to be
> **appended to** — new findings under the section they refine, new
> uncertainties as a new `F-n` row in §8. When an `F-n` resolves it leaves this
> doc and becomes an ADR. Tree claims carry `file:line`; committed positions
> carry an ADR §. Where this doc is unsure it says so.

## 1. Thesis

Phalcom will eventually need capabilities the object model does not contain:
crypto, sockets, filesystem, a clock, native math. There are two doors, and the
obvious one is wrong.

**The wrong door is the ADR-0019 floor.** The floor is not "where native code
lives" — it is where things the object model *presupposes* live. Its six entries
(ADR-0019 §Decision 1–6) — alloc/object-graph, dispatch, absence + Bool roots,
`Number` arithmetic, `Block#call`, `System` I/O — are each there because they
recurse into themselves: you cannot write field access without field access, or
addition without addition. SHA-256 does not presuppose itself. It is ordinary
computation over bytes that is merely *inconvenient* in `.ph` and *absent* from
the platform — two different failures, and ADR-0019's rule recognises only the
first: admission requires proof the capability "cannot be expressed in `.ph` at
all", and **speed is explicitly never sufficient** (ADR-0019 §Consequences;
overlay §"Floor admission rule").

**The right door is FFI** — one boundary carrying arbitrary native capability,
built once, instead of N floor amendments each of which is a category error and
each of which permanently shrinks the dogfoodable surface.

Two things make this unusually cheap for Phalcom, one makes it delicate:

- **ADR-0045's `Module#doesNotUnderstand(_)` is already a native-module seam** —
  on the floor, routing arbitrary member sends into native Rust state
  (floor-census §2.12). A native module may expose a surface through it with
  **+0 new floor bindings** (§5.1 — the key structural finding, unverified: F-7).
- **ADR-0050's non-moving collector removes the classic FFI hazard** — though not
  for the reason the commissioning thesis gave. §3 corrects it.
- **ADR-0009 commits "no `unsafe` for the object graph"**; FFI is unsafe by
  definition. §4 resolves the tension — the tree already settles it.

## 2. Why not the floor — the census math

The floor is small, deliberately, and **growing under a ratchet**. The
authoritative enumeration is [floor-census.md](../core/floor-census.md) — the
overlay's standing instructions are *"don't cite a fixed enumeration, cite the
census"* (§"Primitive vs library boundary") and *"never quote a floor number from
an ADR"* (§Known documentation defects item 4).

**Current floor size — and a caveat.** The census does not agree with itself:

| Source | Bindings | Distinct fns |
|---|---|---|
| floor-census.md §1.1 summary table | **113** | **98** |
| floor-census.md §1.1 "Baseline" note (post-U15) | 112 | 97 |
| floor-census.md §7.2 (live `VM::new()` audit, R-INV-0.1) | **117** | — |

§7's 117 is the only number a test asserts
(`floor_census_matches_installed_bindings`, `phalcom-core/tests/invariants.rs`)
and it accounts for U-ANNOT-CONTRACTS's `__invariantEnter`/`__invariantExit`
(+2, ADR-0052), which §1.1's table does not. **Treat the floor as ~113–117, and
treat the spread itself as a finding** (F-9). Nothing here turns on the exact
value; the argument is about magnitude.

**The math.** A *minimum* crypto suite — not a competitive one — costs roughly
this many bindings. **Estimate, not a tree measurement**; offered as an
order-of-magnitude argument, to be replaced with a real count if anyone scopes
the unit (F-1):

| Capability | Rough bindings | Note |
|---|---|---|
| Hashing (SHA-256/512, `new`/`update_`/`digest_`) | 6–9 | streaming, so ≥3 per algorithm |
| HMAC | 3–4 | keyed variant of the above |
| AEAD (AES-GCM / ChaCha20-Poly1305: key, nonce, seal, open) | 8–12 | two algorithms is table stakes |
| CSPRNG (`randomBytes_`) | 1–2 | must be OS-backed |
| Constant-time compare | 1 | cannot be `.ph` — `.ph` `==` is not constant-time |
| Hex / base64 encode+decode | 4 | arguably derivable in `.ph` over `byteAt_` |
| KDF (PBKDF2 / HKDF / Argon2) | 3–6 | |
| Ed25519 / X25519 (keygen, sign, verify, agree) | 6–10 | |
| **Total** | **~32–48** | |

Against a floor of ~113–117, a single capability domain proposes a **~28–42%
floor expansion**. Sockets, filesystem, clock, and process each carry a
comparable bill. ADR-0019 froze the floor precisely to stop this: its Context
names the *native-vs-library boundary-creep hazard* and observes that "no single
commit is ever the one to blame."

And every one of these amendments would have to lie. ADR-0019 demands proof the
capability is *underivable*. SHA-256 **is** derivable in `.ph` — integer
arithmetic over bytes, and `String#byteAt_(_)` (ADR-0049) already exposes the
bytes. It would be derivable and unusably slow, which is exactly what ADR-0019
pre-rejects: the named counter-move to hot-path cost is *"fund an inline cache or
JIT above the floor"* (§Consequences). **A crypto floor amendment is a speed
argument wearing a derivability costume.**

The genuinely underivable part is smaller and differently shaped: **entropy, the
clock, sockets, the filesystem** are underivable in the strong sense — no `.ph`
reaches them, because they are not computation at all. They are *platform
access*. That is the category FFI serves. Note what follows: the honest
floor-shaped subset of "crypto" is ~1 binding (`randomBytes_`) plus
constant-time compare; everything else is a library. **Build the door, not 40
doorways.**

## 3. `FFI/unsafe boundary ⊗ handle heap + GC`

The classic hazard: native code holds a pointer into the VM heap; the collector
moves the object; the pointer dangles. The escapes are copy (safe, slow) or pin
(needs collector cooperation; pinning windows stall collection — JNI, §7).

**Phalcom's exposure is unusually low — but the commissioning thesis got the
reason wrong, and the correction matters.**

ADR-0050 §Decision 1 is **non-moving**: "Objects keep their `SlotMap` slot for
life." That kills *relocation*. But **non-moving is not address-stable**.
ADR-0050 §Decision 9's own measurement note records that the `SlotMap` backing
array "grows at 40 B/slot" with "a memmove of the whole array on every realloc"
— so a `&Object` borrowed out of the arena **is** invalidated by arena growth.
Handle stability ≠ address stability.

What rescues the zero-copy claim is representation, verified in the tree:

- `Object::Str(StringObject)` is stored **inline** in the arena
  (`phalcom-core/src/heap/object.rs:47` — not boxed, unlike `Class`/`Fiber`/
  `Map` at `:33`/`:70`/`:79`).
- `StringObject { value: String, hash: u32 }`
  (`phalcom-core/src/heap/string.rs:11-16`), and `as_str()` returns `&self.value`
  (`:44`).
- A Rust `String`'s **payload buffer is a separate malloc allocation**. The arena
  memmove relocates the 24-byte `String` header, not the bytes it points at. Same
  for `ListObject { elements: Vec<Value> }` (`phalcom-core/src/heap/list.rs:22`).

So: **the bytes behind a `String`/`Vec` are address-stable across both arena
growth and collection**, because neither the arena nor the collector touches
them. `StringObject` is immutable by contract (`heap/string.rs:1-7`), so there
is no mutate-under-native-reader hazard either. A `&[u8]` crosses the boundary
with no copy and no pinning **against relocation**.

**What is not free: liveness.** A sweep can free the `StringObject`, dropping the
`String` and the buffer under the native reader. ADR-0050 §Decision 7 specifies
the mechanism — `vm.push_temp_root(h)`/`pop_temp_root()`, the Wren
`wrenPushRoot` model — and §Consequences names the obligation: "the *only* way
this collector drops a live object is a native primitive holding a fresh handle
across a re-entrant send without a temp-root."

**And it is not built.** `vm/gc.rs:119-121`: "the `temp_roots` escape hatch that
makes the native side safe [is] U-GC step 4; until then this is driven only by
tests." `:128-134` documents `push_root_for_test` as test scaffolding explicitly
"Not a temp-root". No `push_temp_root` definition exists in `phalcom-core/src`.
**FFI's liveness mechanism is specified, named, and absent** (F-2).

Two mitigations, both already true:

- **Safepoint latching** (ADR-0050 §Decision 6): collection runs only at
  interpreter back-edges, "never in the middle of a native primitive holding raw
  `ObjRef`s in Rust locals." A native call that does not re-enter the interpreter
  cannot be collected under, by construction — and most FFI calls are that shape.
- **No finalizers** (ADR-0050 §Context) ⇒ no finalizer-ordering or resurrection
  hazard — *unless FFI adds one*. Native resources (sockets, files) want
  release-on-collect, the standard reason a language grows finalizers. Phalcom
  should probably refuse, and use `ensure`-scoped ownership (F-3).

**A consequence ADR-0050 does not claim.** Its Consequences list handle stability
for inline caches, `==` identity, and fiber `Value`s — not "a native boundary can
borrow payload bytes without pinning." That benefit is real but **narrower than
"non-moving ⇒ zero-copy"**: it holds for out-of-line payloads
(`String`/`Vec`/`Box`ed variants), not for `&Object`. If FFI is taken up,
ADR-0050 deserves an amendment in those terms (F-4).

## 4. Does FFI violate ADR-0009's "no `unsafe`"?

**The tension, stated fairly.** The overlay's security row reads: *"Memory safety
| Rust ownership + handle heap; **no `unsafe` for the object graph** |
ADR-0009"*. FFI is `extern "C"` and unsafe by definition. If ADR-0009's
commitment is crate-wide, FFI is foreclosed and this document is moot.

**It is not crate-wide, and the tree settles it.** ADR-0009's Decision says "No
`Rc`, no `RefCell`, no `MaybeWeak`"; its Consequences claim no Rc-cycle leak and
no `RefCell` double-borrow surface. **The word `unsafe` does not appear in
ADR-0009 at all** — the phrasing is the *overlay's* compression, and the overlay
itself scopes it ("for the object graph"; "Cyclic apex done via handle patches,
not raw pointers/`unsafe`").

Decisive: **the tree already carries `unsafe` outside the object graph, and no
ADR treats it as a violation.**

- `phalcom-core/src/interner.rs:105` — `unsafe { &*(interned as *const str) }`,
  the classic arena-interner lifetime extension. Same crate as the heap. Not the
  object graph.
- It is the **only** `unsafe` across `phalcom-core/src`, `phalcom-ast/src`,
  `phalcom-common/src`, `phalcom-repl/src`, `phalcom-lsp/src` (grep: 4 matches
  for `unsafe`, 3 of them prose in doc comments).
- **No `#![forbid(unsafe_code)]`/`deny` anywhere** in the workspace (grep
  `unsafe_code`: zero hits). The commitment is documentary, not enforced.
- Note a related doc defect: `heap/mod.rs:15` claims `slotmap` gives the
  ADR-0009 shape "with **zero `unsafe`** in this crate" — true of the arena,
  false of the crate, which contains `interner.rs:105`.

**Proposed precise statement** (draft; not ratified — F-5):

> ADR-0009's memory-safety commitment is scoped to the **object graph**: the
> arena, handles, class/metaclass wiring, and field access contain no `unsafe`
> and no interior-mutability panic surface. It is not a whole-crate ban, as
> `interner.rs:105` already demonstrates. An FFI boundary is therefore
> *compatible* with ADR-0009 **iff** its `unsafe` is confined to an
> isolated, auditable region that (a) lives outside the object graph, (b) never
> hands a raw pointer *into* the arena to native code, and (c) converts at the
> boundary — native code sees `&[u8]`/`&str`/scalars, never `ObjRef`, never
> `Value`, never `*mut Object`.

Condition (c) is the load-bearing one, and it is the Lua lesson (§7): **the
object graph never leaves the VM**. Under it, ADR-0009's claim survives
literally rather than by reinterpretation.

**Honest counter-argument, recorded rather than dismissed.** ADR-0009's
Consequences say the failure modes are "removed by construction, not patched."
An FFI region re-introduces a class of memory-safety failure that is currently
absent from the workspace *in aggregate*, even if it is absent from the object
graph *specifically*. Whether the overlay's Memory-safety row should therefore
be amended to name FFI as a carved-out exception — rather than silently relying
on "object graph" doing that work — is a real question, not a formality (F-5).
The `#![forbid(unsafe_code)]` gap (F-6) is what would make the carve-out
mechanical instead of documentary: forbid at the crate root, `#[allow]` on
exactly one module.

## 5. `primitive/library boundary ⊗ bootstrap order` — and what already exists

### 5.1 Native modules are established, not new

ADR-0019 §Context makes the floor **the kernel-load-DAG boundary**: blessed
classes are built by `create_core_classes`/`install_primitives` before any
`core.ph` loads, and "an unwritten boundary means an unwritten DAG edge, which
is exactly the shape that produces a hard boot failure with no user frame to
blame."

This is the hazard FFI must respect, and it is also the reason FFI should be a
**module**, not a class in the tower. A module loaded *after* bootstrap has no
DAG edge into the kernel at all. The precedent chain:

| Precedent | Status | Evidence |
|---|---|---|
| `System` — native I/O module | **built** | `primitive/system.rs`: `system_class_print:22`, `system_raw_write:107`, plus `system_schedule:56`, `system_next_scheduled:70`, `system_gc:92` |
| `System.write_(_)` — native, `.ph`-wrapped | **built** | ADR-0049; floor-census §2.11 |
| `Module#doesNotUnderstand(_)` — native member-access router | **built** | ADR-0045; floor-census §2.12; `primitive/module.rs` |
| `Reactive` — native module over VM/Universe state | **ratified, NOT built** | ADR-0058 (Accepted). Grep for `Reactive` in `phalcom-core/src`: **zero hits** |

**Correction to the commissioning thesis.** It cited ADR-0058's `Reactive` as
precedent that "a native module is an established pattern." ADR-0058 is
Accepted, but `Reactive` **does not exist in the tree** — STATUS.md marks 0058
`?` (unverified) and grep confirms zero hits. The built precedent is `System`
alone. `Reactive` is precedent for the *decision* that ambient native state
needs a native module, not for the *mechanism*. Cite it as the former.

**The gift.** `Module#doesNotUnderstand(_)` was admitted (ADR-0045, +1 binding)
so `math.pi` and `math.distance(1, 2)` reach a module's own `globals` table via
the miss path. That is a general native-member-dispatch seam. An FFI module can
route `crypto.sha256(bytes)` through the *same* mechanism with **zero new floor
bindings**. Phalcom's floor already contains the FFI dispatch primitive; it was
paid for by a different unit. Whether this actually holds for a *native-backed*
module rather than a `.ph`-source-backed one needs verification before anyone
relies on it (F-7).

### 5.2 Naming — reconciling with U-NATIVE-MARKER

`docs/forge/units/U-NATIVE-MARKER/plan.md:24-30` adopts the Wren convention: "a
trailing `_` on a selector marks a native/private primitive", and it is
explicitly *"a naming convention, not a semantic change"* with **"New governing
ADR: none"**.

This reconciles cleanly, and note it is *not* a floor marker — it marks
**native/private, wrapped by `.ph` above**. That is precisely an FFI binding's
shape. So:

```phalcom
// native, FFI-bound, private — trailing _
crypto.sha256_(bytes)          // raw: takes/returns byte buffers, no Option
// .ph surface above it, public — no underscore
crypto.sha256(aString)         // wraps, validates, returns Option/Result
```

The convention is load-bearing here in a way it is not elsewhere: the trailing
`_` becomes the **visual marker of the unsafe boundary**. Every `_` selector in
an FFI module is a place where the auditable region is entered. That is a
stronger claim than the convention currently makes, and would need a ruling
(F-8).

## 6. The three tiers

### (a) In-tree native module, compiled into the VM

The `System` model. Rust code in `phalcom-core/src/primitive/`, bound at
bootstrap by `install_primitives`, surfaced as a global.

| | |
|---|---|
| **Buys** | No loader. No ABI. No verifier. No dynamic-code hazard. The floor census (R-INV-0.1) can audit the whole native surface from a live `VM::new()`. `unsafe` (if any) is in-tree, reviewed, and covered by the repo's miri/fuzz lanes (overlay §Robustness substrate). |
| **Costs** | **Every module ships in the core binary** — a crypto+sockets+math VM is one fat binary whether or not a program imports any of it. Third parties cannot extend Phalcom without a fork + recompile. Adding a capability is a VM release. |
| **Precludes** | An ecosystem. Nothing else. |
| **Status** | This is what Phalcom does today, for exactly one module. |

### (b) Out-of-tree dynamic library, loaded at runtime (`dlopen`)

Real extensibility: `import "crypto"` finds `libphalcom_crypto.so`, dlopens it,
calls a registration entry point.

| | |
|---|---|
| **Buys** | A package ecosystem. Modules version independently of the VM. Users pay only for what they import. |
| **Costs** | A loader, a **stable ABI** (§7's central lesson), a symbol-resolution story, and a **security story**. |
| **Precludes / blocked by** | This is the **`dynamic power ⊗ untrusted input`** hazard in its pure form. ADR-0045 §Decision 7 resolves `import` to **source only** and explicitly defers a compiled-bytecode loader because "Phalcom has no bytecode verifier, and loading unverified bytecode is a security hole." A `.so` is *strictly worse than unverified bytecode* — it is unverifiable in principle. No verifier can check machine code; loading a dylib is granting arbitrary native execution, full stop. It also breaks the overlay's §Untrusted-input posture ("bytecode is compiler-produced, not externally loaded — so no verifier is needed yet"). |
| **Verdict** | **Not now.** It is not blocked on engineering; it is blocked on a capability/permission model Phalcom has never discussed. Note the asymmetry honestly: a verifier (D6, `bootstrapping-and-self-hosting.md` §226) would unblock *bytecode* imports. Nothing unblocks dylibs except a decision to trust them. |

### (c) Build-time static registration — the "native extension, compiled in" model

Out-of-tree *source*, in-tree *binary*. A native module is a Rust crate
implementing a registration trait; the VM is built with a manifest of enabled
extensions (a Cargo feature, a build script, or a generated registry). Python C
extensions' authoring model, minus the dynamic loading.

| | |
|---|---|
| **Buys** | Extension authorship outside the core repo — third parties write modules without a fork. The binary contains exactly the modules that were enabled, so unused capability costs nothing. **No loader, no ABI, no verifier, no dynamic-code hazard**: it is a compile-time link, so the ABI is Rust's and the compiler enforces it. `unsafe` in an extension is still statically present and auditable. |
| **Costs** | Extending Phalcom means **rebuilding Phalcom**. No binary distribution of modules; no `import` of something not compiled in. A cross-compilation and feature-matrix burden. Registration seam must be designed and kept stable-ish. |
| **Precludes** | Binary-only/proprietary modules; runtime plugin discovery; a `cargo install`-style ecosystem where users don't compile. |

**Recommendation (the doc's opinion, not a ruling).** **Take (a) now, shape its
registration seam so (c) is a later drop-in, defer (b) indefinitely pending a
capability model.**

(a) is the status quo and needs no decision. The only *new* work worth doing
today is making `install_primitives` accept a registry of module descriptors
rather than hard-coding each — (a) and (c)'s shared prefix, costing nothing if
(c) never happens. ADR-0045 §Consequences made the analogous move for imports:
it "keeps the loader's resolve-seam abstract enough for a verified-bytecode
source to slot in behind it."

(b) is the only tier that buys an ecosystem and the only one that cannot be made
safe. Make that trade deliberately, later, with a permission model — do not
inherit it by accident because a loader seemed convenient.

**What this precludes.** Binary-distributed native modules; a third-party native
package registry; any install-a-driver-at-runtime story; and — while (a) is the
only built tier — third-party native modules at all, since (a) requires a fork.
That last cost is why (c)'s seam is worth the small effort now.

## 7. Precedent — and what it cost each of them

Precedent without consequence is trivia. Each entry names the price.

**Python C-API — the memory model leaked into the ABI.** `Py_INCREF`/`Py_DECREF`
are public contract, so every C extension ever written encodes CPython's
*refcounting algorithm*. **Cost:** CPython can never adopt a moving or
non-refcounting collector without breaking the ecosystem; PyPy must emulate the
C-API (`cpyext`), and the emulation's slowness is a large part of why PyPy never
displaced CPython; removing the GIL (PEP 703) is a decade-plus project *because
the ABI exposes it*. **Lesson — the most important one here:** never let
`ObjRef`, `Value`, the collector, or the arena into the FFI contract. An FFI
binding taking an `ObjRef` is this exact mistake. Motivates §4's condition (c).

**Lua C API — stack discipline; the right model for a stack VM.** C never holds a
GC pointer. Values live on a VM-managed stack, manipulated by *index*
(`lua_pushvalue`, `lua_gettop`, `lua_tolstring`); long-lived references go in the
registry via `luaL_ref`. The collector stays free to do anything, because it can
see every reference C holds. **Cost:** verbosity, and a silent failure mode —
unbalanced-stack and index-arithmetic bugs are easy to write and hard to detect;
`luaL_checkstack` is a discipline, not a type. **Why it fits Phalcom:** Phalcom
already has this shape. ADR-0050 §Context: "Roots are fully reified — `VM::stack`
and `VM::frames` are owned `Vec`s ... so root enumeration is *precise*", and
§Decision 7's temp-root stack **is** `luaL_ref`/`wrenPushRoot` renamed. Phalcom
need not invent an FFI memory discipline — it needs to build the one ADR-0050
already specified (F-2). Study this first.

**JNI — pinning windows and verbosity.** Local/global reference tables keep the
collector informed, but zero-copy array access via
`GetPrimitiveArrayCritical` opens a window in which the collector must not run.
**Cost:** those windows stall GC and are a latency cliff in a pause-sensitive
collector; the API is verbose enough that correctness is a specialty; after ~25
years the JDK conceded and built Panama/FFM to replace it. **Lesson:** the
pinning window is the price of zero-copy *against a moving collector*. Phalcom
does not pay it (§3) — non-moving plus out-of-line payloads means no window is
needed. This is the concrete form of the "unusually cheap" claim.

**Ruby C extensions — no stable ABI, and conservative scanning as a permanent
tax.** MRI exposes internal structs, so extensions break across versions and gems
recompile per Ruby. Worse, MRI **conservatively scans the C stack** for roots
held by extensions. **Cost:** conservative scanning means MRI can never be
precise or generally moving (`GC.compact` is limited and opt-in precisely
because of this), and false retention is unfixable. **Lesson:** Phalcom's precise
root enumeration (ADR-0050 §Context: "no conservative stack scanning, no false
retention, no `unsafe`") is an asset an FFI design can destroy in one commit. Any
FFI letting native code hold a root in a Rust local without telling the VM forces
conservative scanning later. Do not.

**N-API (Node) — stable ABI bought with opacity.** After the churn of raw V8
handles across Node majors, N-API exposes the opaque `napi_value` plus explicit
handle scopes; modules compile once and survive major versions. **Cost:** an
extra indirection per access, and V8 internals became unreachable, so some
high-performance modules could not migrate. **Lesson:** ABI stability comes from
*hiding* the object model, not versioning it. At tier (b), `napi_value`-style
opacity is the design — the same conclusion as the Python and Lua rows reached
from a third direction. Three ecosystems converging is the strongest signal here.

**Wasm — sandboxed FFI, the alternative shape.** A wasm module cannot touch host
memory: it has its own linear memory and reaches out only through imported
functions, so capability is granted explicitly (WASI) rather than assumed.
**Cost:** no zero-copy (data is copied in/out of linear memory), no syscalls
without a WASI implementation, toolchain and startup weight, and numeric-only
boundary types mean marshalling everything else. **Lesson:** wasm is the *only*
tier-(b) shape that answers `dynamic power ⊗ untrusted input`, because loaded
code is sandboxed by construction rather than trusted by assumption. If Phalcom
ever wants runtime-loadable third-party native modules, **wasm is a more
plausible door than `dlopen`** — trading §3's zero-copy win for the security
story tier (b) otherwise lacks. Evaluate it before anyone writes a dylib loader
(F-10).

## 8. Open questions

Numbered for citation. Add rows; do not renumber.

| # | Question | Why it is open | Would resolve via |
|---|---|---|---|
| **F-1** | What is the *actual* binding count for a minimum crypto suite? | §2's table is an estimate, not a measurement. The argument is magnitude-robust, but the number should not be quoted as fact. | Scope a throwaway spike; count bindings. |
| **F-2** | ADR-0050 §7's `push_temp_root`/`pop_temp_root` is specified but **not built** (`vm/gc.rs:119-121`, `:128-134`; no definition in `phalcom-core/src`). Is FFI blocked on U-GC step 4? | It is the *only* mechanism protecting a native-held handle across a re-entrant send. Any FFI that re-enters the interpreter needs it. Non-re-entrant FFI may not. | Determine whether any plausible FFI call re-enters (callbacks do). If yes: hard dependency on U-GC step 4. |
| **F-3** | Do native resources (sockets, files) force finalizers? | ADR-0050 §Context banks "No finalizers exist" as a reason the collector is hazard-free. Native handles are the classic reason a language grows them. | Rule: explicit `ensure`-scoped ownership vs. a finalizer/`Drop` admission. The latter reopens ADR-0050. |
| **F-4** | Should ADR-0050 be amended to record the FFI benefit? | It is a real consequence of a ratified decision that the ADR does not claim. But it must be worded as §3 does (*out-of-line payloads are address-stable*), **not** as "non-moving ⇒ zero-copy", which is false. | An ADR-0050 amendment, only if FFI is taken up. |
| **F-5** | Is ADR-0009's memory-safety commitment object-graph-scoped or crate-wide? | ADR-0009 **never says `unsafe`** — the phrasing is the overlay's. `interner.rs:105` already carries `unsafe` in `phalcom-core` with no ADR treating it as a violation, which strongly implies "object graph". But the overlay's row should say so explicitly rather than relying on inference. §4 drafts the precise statement. | A user ruling + an overlay edit; possibly an ADR-0009 clarifying amendment. **This is the doc's most important question.** |
| **F-6** | Should the workspace adopt `#![forbid(unsafe_code)]` with a single `#[allow]`d FFI module? | Grep: **zero** `unsafe_code` attributes exist. The no-`unsafe` commitment is documentary, not mechanical. A forbid-plus-carve-out makes the auditable region a compiler-enforced fact. Blocked on F-5. | An ADR; cheap to do, and cheapest *before* FFI exists. |
| **F-7** | Does `Module#doesNotUnderstand(_)` (ADR-0045) actually work as a *native-backed* module seam, or only for `.ph`-source modules? | §5.1's "+0 floor bindings" claim rests on this and it is **not verified**. `module_does_not_understand` routes to a `ModuleObject`'s `globals`/`name_to_slot`; whether a native module can populate those, or needs a different `Object` variant, is unchecked. | Read `primitive/module.rs` + `heap/module.rs`; spike a native module. |
| **F-8** | Does the trailing-`_` convention extend to "marks the unsafe boundary"? | U-NATIVE-MARKER is explicitly a naming convention with **no governing ADR** (`plan.md:29`). §5.2 would give it semantic weight it does not currently carry. | A ruling; note in `lexical-structure.md` conventions. |
| **F-9** | The floor census disagrees with itself: §1.1 table says 113/98, its Baseline note says 112/97, §7 says the live audit asserts 117. | Same defect class the overlay already flagged ("Floor census numbers don't chain", §Known documentation defects item 4) — but that entry was about *ADRs* not chaining. This is the census, which the overlay designates authoritative. | A census reconciliation pass. Out of scope here; recorded because §2 had to quote a range. |
| **F-10** | If tier (b) is ever wanted, is wasm the door rather than `dlopen`? | §7's wasm row: it is the only tier-(b) shape that answers `dynamic power ⊗ untrusted input`, at the cost of §3's zero-copy win. | A real evaluation, before anyone writes a dylib loader. |
| **F-11** | Does an FFI module need a *class* in the tower, or only a module? | §5.1 argues module — a post-bootstrap module has no kernel-load-DAG edge (ADR-0019 §Context). But `crypto.Digest` as a value implies a class, which implies a bootstrap-order question. | Design work, gated on F-7. |

## 9. What this document precludes

Nothing. It is a draft with no owning unit. It is recorded so that the next time
someone proposes admitting a native capability to the floor, the census math in
§2 and the admission rule in ADR-0019 are one link away — and so that the
`unsafe`-scope question (F-5) is answered before it is answered by accident.
