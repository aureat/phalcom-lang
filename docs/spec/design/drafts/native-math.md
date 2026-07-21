# Native math — typed arrays, vectors, and the dtype question

- Status: **Draft** (exploration only — not proposed, not ratified, no owning unit)
- Date: 2026-07-15
- Depends on:
  [ADR-0024](../../../adr/accepted/0024-numeric-surface-split-int-float-and-division.md) (the committed `Number`→`Int`/`Float` tower) ·
  [ADR-0019](../../../adr/accepted/0019-freeze-vm-blessed-primitive-floor.md) (floor admission rule) ·
  [ADR-0020](../../../adr/accepted/0020-kernel-list-native-array-protocol.md) (native-storage / `.ph`-protocol template) ·
  [ADR-0010](../../../adr/accepted/0010-tagged-value-enum.md) (`Value` repr; NaN-boxing deferral) ·
  [ADR-0009](../../../adr/accepted/0009-handle-arena-heap.md) (handle heap) ·
  [ADR-0051](../../../adr/accepted/0051-performance-strategy-measure-first-tiered-optimization.md) (measure-first, behavior-invariant)
- Related: [ADR-0032](../../../adr/accepted/0032-collections-representation-and-literals.md)/[ADR-0039](../../../adr/accepted/0039-amend-floor-admit-collection-container-primitives.md) (collection arms) ·
  [ADR-0060](../../../adr/accepted/0060-index-operator-as-real-selector.md) (`[]` is a real selector) ·
  [ADR-0048](../../../adr/accepted/0048-amend-iteration-bare-cursor-sentinel-and-iterable-root.md) (`Iterable` root) ·
  `core/floor-census.md` (authoritative floor counts) ·
  `../../forge/perf-log/SCOREBOARD.md` (authoritative numbers) ·
  **`drafts/ffi.md` — does not exist yet.** Every FFI cross-reference below is a forward
  reference to an unwritten sibling, not a citation. Flagged, not assumed.

> **How to grow this doc.** It answers one commissioned question and stops. Append new
> sections rather than rewriting: §"Open questions" is the queue, and each `M-n` is meant
> to be struck through with a ruling and a date, in place. Nothing here is committed.

## Thesis

The commissioning question was: *"suggest other new natives such as f32, f64, u32 — do we
need it? does it give any benefit?"*

The honest answer is **no — not as surface classes**, and the reasoning is this document's
spine. [ADR-0024](../../../adr/accepted/0024-numeric-surface-split-int-float-and-division.md)
already committed the numeric surface: abstract `Number` over exact auto-promoting bignum
`Int` and IEEE-754 `Float`. Adding `u32`/`i8`/`f32` as *surface classes* would fork that
tower, multiply the dispatch axis ADR-0012 keeps to a single probe, and reintroduce exactly
the width/overflow semantics ADR-0024 was written to abolish. That is a superseding-ADR
question, not a draft's to answer.

The reframe: **the benefit of fixed-width types is not scalar arithmetic, it is bulk
storage.** A dtype belongs on an *array*, not in the class tower. This is precisely what
numpy does and it is the model worth copying.

**But two measured facts weaken the usual quantitative case, and this draft leads with them
rather than burying them.** Phalcom's `List` is *already* an unboxed contiguous array
(`ListObject { elements: Vec<Value> }`, `phalcom-core/src/heap/list.rs:22-25`), and `Value`
is a 16-byte immediate — measured, this session, `size_of::<Value>() == 16`, corroborated by
`SCOREBOARD.md` §2 ("`Value` is still 16 B against Wren's 8 B, a fixed 2.0×"). So a Phalcom
list of a million floats is **one** heap object holding a 16 MB contiguous buffer — *not* a
million boxed values. The memory argument for typed arrays is therefore **2×, not 30×**, and
for `f64` specifically it **goes to 1× once NaN-boxing lands**. The real case for a native
ndarray is dispatch cost, not memory — which walks straight into ADR-0019's rule that
*speed is never sufficient*. §"The ADR-0019 tension" is where this document earns its keep.

## Do we need `f32`/`u32`/`i64` as classes?

**No.** Three independent reasons, in descending order of force:

1. **It contradicts a committed decision.** ADR-0024 §1 makes `Number` abstract over exactly
   two concrete classes and §2 makes `Int`'s small/large tiers *deliberately invisible*
   ("Unlike Smalltalk, the small/large tiers are **not** distinct surface classes"). ADR-0024
   §"Alternatives considered" already rejected exposing representation as surface class
   (`SmallInteger`/`LargeInteger`): *"Leaks a representation detail into the object model;
   users would branch on which one they got."* A `u32` class is the same rejected move under
   a different name. Reversing it needs a superseding ADR.
2. **It re-imports the footgun 0024 removed.** ADR-0024 §2's whole promise is that `Int`
   arithmetic is *total and exact* — "no trap and no wraparound". `u32` has both: it wraps or
   it traps. A `u32` class means `Number`'s subclasses no longer share an arithmetic contract,
   so `Number` stops being a meaningful abstraction.
3. **It multiplies the dispatch axis.** ADR-0012 buys one hashmap probe per send. Every new
   numeric class adds a receiver class whose `+(_)` must handle every other numeric class —
   the classic n² coercion matrix. ADR-0042 (Retired) named this constraint explicitly while
   deferring the split: *"a later numeric-type split must not add a dispatch axis"*. 0024
   spends that budget once, on a 2-class tower. `f32`+`u32`+`i8`+… spends it n times.

### The memory math — the actual numbers

One million elements, measured `Value` = 16 B, `ListObject` storage = one contiguous
`Vec<Value>`:

| Representation | 1M elements | vs `List` today | vs `List` post-NaN-box |
|---|---|---|---|
| Phalcom `List` (today, `Value`=16 B) | **16 MB** | 1.0× | — |
| Phalcom `List` (post-NaN-box, `Value`=8 B) | **8 MB** | 0.5× | 1.0× |
| raw `f64` buffer | 8 MB | **2.0×** | **1.0× — no win** |
| raw `f32` buffer | 4 MB | 4.0× | 2.0× |
| raw `i32` buffer | 4 MB | 4.0× | 2.0× |
| raw `u8` buffer (image/byte data) | 1 MB | **16×** | **8×** |

Read this table honestly and it says something the naive "boxed values" framing does not:

- **For `f64`, a dtype'd array buys nothing on memory after NaN-boxing.** NaN-boxing is
  deferred but *committed as in-scope* (ADR-0010; ADR-0051 §4 Tier 6, "`Value` 16 B → 8 B").
  A `Float` in a `List` is already an inline immediate in a contiguous buffer. An `f64`
  ndarray and a NaN-boxed `List` are byte-for-byte the same density.
- **The memory case survives only for narrow dtypes** — `f32`, `i32`, and especially `u8`.
  `u8` is the strongest single argument in this document for a dtype tag: 16× today, 8×
  forever. Image buffers, byte protocols, and mmap'd files are where dtype pays on memory.
- **Everything else is a dispatch argument wearing a memory argument's clothes.**

⚠ **Stale number flagged.** `size_of::<Object>()` measured **40 B** this session, not the
256 B ADR-0051 §Context asserts ("Slots are 256 B (sized to the fattest variant)") — the fat
variants have since been boxed (`heap/object.rs:29-45`). It changes no conclusion above (a
`List`'s elements live in one `Vec`, not one slot each), and is recorded only so a future
session does not re-derive 256 B.

## What a native ndarray arm looks like

If it is built, it follows [ADR-0020](../../../adr/accepted/0020-kernel-list-native-array-protocol.md)'s
template exactly — the same split ADR-0032/0039 replicated for `Map`/`Set`/`Tuple`/`Range`:
**storage native, protocol `.ph`.**

```rust
// Sketch only — not proposed. Mirrors Object::List (heap/object.rs:60).
pub enum DType { U8, I32, I64, F32, F64 }        // narrow set; see M-3

pub struct NDArrayObject {
    dtype:   DType,
    shape:   Vec<usize>,      // logical dimensions
    strides: Vec<isize>,      // element strides — enables views/transpose w/o copy
    offset:  usize,
    data:    Arc<Vec<u8>>,    // ⚠ raw bytes; sharing model is M-5, NOT decided
}
// Object::NDArray(Box<NDArrayObject>)  — boxed, per heap/object.rs:29-33 rationale
```

Above the floor, in `core.ph`, as ordinary dispatched methods over the raw primitives:
`shape`, `dtype`, `reshape`, `[]`/`[]=` (real selectors — [ADR-0060](../../../adr/accepted/0060-index-operator-as-real-selector.md),
*not* `at(_)` lowering; ADR-0055 is Retired, do not cite it), `iterate`/`iteratorValue` to
join the `Iterable` root ([ADR-0048](../../../adr/accepted/0048-amend-iteration-bare-cursor-sentinel-and-iterable-root.md)),
and the arithmetic operators.

**Scalar boxing at the boundary is unavoidable — the design's honest seam.** Reading element
`i` of an `f32` array must yield a Phalcom value, and Phalcom has exactly one float. So
`arr[i]` widens `f32`→`f64`→`Value::Float`, and `arr[i] == 0.1` is then *false* on an `f32`
array. This is precisely numpy's `np.float32(0.1) != 0.1`, and it arrives whether or not
`f32` is a surface class — it is a property of storing narrow floats, not of exposing them.

## The ADR-0019 tension — the hard part

**The collision.** A million-element `a + b` must not be a million `Number#+(_)` sends. The
measured cost of one is **~113 ns/send** (`SCOREBOARD.md` §3, `arith_send`, primitive `1 + 2`
at HEAD), so a 1M-element elementwise add costs **~113 ms** dispatched. The same loop in
Rust over an `f64` buffer is order ~1 ms scalar, less vectorized — a **~100×** gap, and this
is *after* seven perf cuts. (Compare `SCOREBOARD.md` §1: the `for` row — 1M list build + sum
— is Phalcom's worst benchmark at 10.7× Wren.) numpy's entire reason to exist is that this
loop is in C. So the native arm would want *whole-array* primitives — `add_`, `mul_`, `sum_`,
`dot_` — where the vectorized op **is** the primitive.

**Why that is not admissible as written.** ADR-0019's admission rule requires proof the
capability **cannot be expressed in `.ph` at all**, and states that *speed is explicitly never
sufficient*; its named counter-move to hot-path cost is "fund an inline cache or JIT *above*
the floor." Worse, ADR-0019 §Consequences pre-commits the exact trade this would reverse:
*"Accepting that `List`, iteration, and (eventually) the compiler run as ordinary dispatched
`.ph` means accepting slower hot paths until inline-cache population … or a JIT lands."*

Now weigh the three candidate resolutions the commission proposed:

**(a) "It passes on representation grounds, not speed grounds."** The commissioning note
guessed this is probably the correct argument. **It is not — at least not for the kernels,
and this draft's recommendation is to abandon it there.** The argument fails on its own
terms: once the arm exposes scalar accessors (`[]`, `size` — and it must, to be `Iterable`),
`add_` *is* expressible in `.ph` as a loop over `[]`. It is derivable, therefore ADR-0019
bars it from the floor, and its only remaining justification is that the `.ph` version is
~100× slower — which is the one justification ADR-0019 names as insufficient. Resolution (a)
is sound for **storage** and unsound for **kernels**, and conflating the two is the error to
avoid.

**(c) "Amend the rule via a superseding ADR."** Available, but recognize the cost: ADR-0019
§Alternatives already considered and rejected the *"maximal floor (keep collections/strings
native, CPython/Lua style)"* precisely because *"the stdlib can't be reshaped or introspected
as objects."* A vectorized-kernel floor is that rejected alternative, re-proposed for one
domain — and a one-way door, since 0019's ratchet breaks in the dogfooding direction only.

**(b) FFI — the recommendation.** Split the problem along ADR-0020's existing seam:

| Layer | Home | Grounds |
|---|---|---|
| dtype'd buffer + shape/strides (`Object::NDArray`) | **floor amendment** | **Representation.** A packed `f32`/`u8` buffer genuinely cannot be expressed in `.ph` over `Value` — `.ph` has no way to say "32-bit float, packed". This is ADR-0020's own words: *"an array cell is the one container primitive the language cannot bootstrap from itself."* Passes 0019 on the same grounds `List` did. |
| protocol — `[]`, `shape`, `each`, `map`, `+` | **`.ph`** | ADR-0020's template. Derivable ⇒ must be `.ph`. |
| vectorized kernels — `add_`, `dot_`, BLAS | **FFI, not the floor** | Derivable ⇒ 0019 bars the floor. Only speed motivates them ⇒ 0019 bars them again. FFI is the sanctioned escape. |

**Why FFI is a real distinction and not a dodge.** ADR-0019 protects the *floor* — what the
VM blesses at bootstrap, what `create_core_classes`/`install_primitives` build before
`core.ph` loads, what can never be redefined, and what every program depends on to boot. An
FFI-loaded BLAS kernel is none of those: not blessed, not on the bootstrap DAG, redefinable,
and no program needs it to start. The dogfooded surface — the thing 0019 exists to protect —
does not shrink by one line. That is a genuine difference in kind.

**What it precludes, stated plainly:** vectorized math is then *not* available in a
dependency-free Phalcom. `core.ph` never gains `add_`. A build without the FFI library gets
the ~113 ns/element `.ph` loop and nothing else. That is the price, and it is the same price
Wren and Lua pay (§Precedent).

⚠ **The hazard this recommendation creates — name it now.** If "put it behind FFI" is
accepted as the answer to hot-path cost, FFI becomes a **new ratchet that routes around
ADR-0019 entirely**: any primitive rejected at the floor gets re-proposed as an FFI library,
and the floor stays nominally frozen while the native surface grows anyway. ADR-0019's ratchet
was fixed by *decision*; nothing yet fixes FFI's. **An FFI ADR must carry its own admission
policy, or it silently repeals 0019.** This is `M-6` and it is the most important open
question in this document.

## Vectors vs arrays — a different problem, probably not worth it

The commission asked separately about `Vec2`/`Vec3`/`Vec4`. **This is not a small ndarray; it
is a different problem, and the honest answer is that Phalcom cannot do it well today.**

An ndarray wants *bulk* storage amortizing one heap object over a million elements. A small
vector wants the opposite: **inline, unboxed, no heap indirection at all** — `Vec3` is 24
bytes and wants to live in a register or a stack slot, with `a + b` allocating nothing.

Phalcom offers no such placement, and the two obvious routes are both blocked:

- **As an `Object::Instance`** ([ADR-0009](../../../adr/accepted/0009-handle-arena-heap.md)
  handle/arena heap): every `Vec3` is a heap handle + slot; `a + b` is a send plus an
  allocation plus a `Heap::get` indirection per component. This is strictly worse than a
  3-element `List` and no better than writing `Vec3` in `.ph` — so it earns no native arm.
- **As a new `Value` arm** (`Value::Vec3([f64;3])`): **decisively precluded, quantitatively.**
  Three `f64`s is 24 B + tag ⇒ `Value` grows 16 B → ≥32 B, *doubling every value in the
  system* — every stack slot, every `List` element, every field. It also directly reverses
  ADR-0051 §4 / Tier 6, which wants `Value` **16 B → 8 B**. Trading a 2× regression on all
  values for a win on one class is not a close call.

What it would actually require is a **value-type / struct mechanism** — inline, unboxed,
copied-not-referenced aggregates. Phalcom **has not specified one**, and nothing in the ADR
set gestures at it. That mechanism is a language-sized decision (it interacts with identity,
`==`, the GC's tracing seam, and the object model's "everything is an object" premise) and it
would need to precede any serious `Vec3`. **Recommendation: do not pursue small vectors.
Write `Vec3` in `.ph` as an ordinary class and accept it.** Revisit only if a value-type ADR
is ever independently motivated — this is `M-4`.

## BLAS/SIMD and the FFI boundary

The end-state, stated honestly: **numpy is not fast because C loops are fast. It is fast
because it links BLAS/LAPACK.** A hand-written C triple loop for `dot` is ~10-50× slower than
a tuned BLAS `dgemm`, which is blocked for cache hierarchy and hand-vectorized per
microarchitecture. Any Phalcom ndarray that claims "fast linear algebra" without BLAS is
claiming C-loop speed, which is the *smaller* half of the win.

Therefore **binding BLAS is an FFI question, not a floor question** — reinforcing §"The
ADR-0019 tension"'s recommendation. Rust precedent, with what each costs:

| Option | What it gives | What it costs |
|---|---|---|
| `ndarray` + `ndarray-linalg` | numpy-shaped n-d arrays, views/strides, BLAS/LAPACK | **LAPACK linkage pain** — a backend feature (`openblas-static`/`intel-mkl`/`netlib`) must be chosen at build time; a notorious CI and cross-compilation headache. Drags a Fortran-lineage toolchain into a repo that currently builds with plain `cargo build`. |
| `nalgebra` | const-generic **small** vectors/matrices — the `Vec3` case, done right, unboxed | Heavy monomorphization ⇒ real compile-time cost. Solves a problem §"Vectors vs arrays" says Phalcom cannot *expose* anyway without a value-type mechanism. |
| `faer` | pure-Rust dense linear algebra, competitive `dgemm`, **no LAPACK linkage** | Younger, narrower coverage than LAPACK; smaller ecosystem. **The most interesting option for Phalcom precisely because it removes the linkage cost** that makes `ndarray-linalg` painful. |
| `std::simd` / `wide` | portable SIMD for hand-written kernels | `std::simd` is nightly-only (Phalcom is stable, edition 2024). Buys the C-loop half of the win, not the BLAS half. |

## Interaction hazards

- **dispatch cost ⊗ elementwise ops.** The canonical one. Measured: ~113 ns/send
  (`SCOREBOARD.md` §3) ⇒ ~113 ms per 1M-element op. Any array API whose elementwise ops route
  through `Number#+(_)` per element is ~100× off C. numpy's API is vectorized *because* of
  this, not for elegance. It is why the naive design fails, and why it collides with ADR-0019.
- **speculative optimization ⊗ observable semantics.** ADR-0051 §2 makes behavior-invariance
  the default gate and ADR-0018 requires a fast path equal the slow path *on every input*. A
  dtype'd kernel **cannot** meet that bar: `f32` addition is not `f64` addition (different
  rounding), and a `u32` add wraps where `Int` promotes. So a typed-array fast path is **not
  an optimization — it is a spec change**, and ADR-0051 is explicit that such a change "gets
  its own ADR + spec — not a performance sneak." This is the deepest reason dtype must be a
  *declared property of a distinct object*, never an invisible speedup on `List`.
- **dtype width ⊗ ADR-0024's exactness promise.** The sharpest unavoidable conflict. ADR-0024
  §2 promises `Int` is total and exact — "no trap and no wraparound". But `arr[i] = 2**40` on
  a `u32` array cannot be total: it must raise, or wrap, or silently promote the array. Every
  answer reintroduces width semantics at the array boundary — the exact thing 0024 abolished.
  The *least-bad* answer is probably **raise** (consistent with ADR-0060's ruling that
  out-of-bounds *write* raises while read is total), but this is `M-1` and it is not decided.
- **primitive/library boundary ⊗ bootstrap order.** ADR-0019 §Context: the floor *is* the
  kernel-load-DAG boundary. An `Object::NDArray` arm would be built by `install_primitives`
  before `core.ph` loads, and its `.ph` protocol must load after `List` and `Iterable`
  (ADR-0020 §Load order). An ndarray is *not* on the critical path — nothing in the kernel
  needs it, unlike `List`, which dNU/variadics/iteration all required. **It therefore has no
  claim to bootstrap priority and should load last**, if ever.
- **FFI escape valve ⊗ the ADR-0019 ratchet.** See §"The ADR-0019 tension". `M-6`.

## What this precludes

- **The dtype-on-array reframe precludes a fixed-width surface tower.** Once `[]` returns
  `Float`/`Int`, user code cannot branch on `f32` vs `f64` — by design. Adding scalar `f32`
  later would be *additive but incoherent* (numpy's actual outcome), so this is close to a
  one-way door.
- **FFI-for-kernels precludes vectorized math in dependency-free Phalcom**, and precludes
  `core.ph` ever shipping `add_`.
- **Declining small vectors precludes Phalcom as a graphics/game-math host** until a
  value-type mechanism exists — a real domain, given up knowingly.
- **A dtype'd buffer arm would make `NDArray` the first arm whose payload is opaque to the
  object graph.** Every existing arm stores `Value`s; `NDArrayObject.data` is raw bytes the GC
  must *not* trace. ADR-0050's tracer reaches children only via `Value::as_obj`
  (`value/mod.rs:56-73`), so a byte buffer is trivially untraced — **compatible today**, but
  worth stating before it is rediscovered.
- **Nothing here precludes NaN-boxing** — which actively *erodes* the `f64` memory case
  (§"The memory math"). If NaN-boxing lands first, re-derive this document's cost/benefit
  rather than re-reading it.

## Precedent — with what each one cost

- **numpy / Python — the model to copy.** Python has exactly **one** `int` (bignum, exactly
  ADR-0024's `Int`) and **one** `float`. Dtypes live on `ndarray`, never in the scalar tower;
  `np.float32` exists as a scalar type only as a *boxing artifact of indexing an array* —
  strong evidence that ADR-0024's tower + a dtype'd array is sufficient.
  **What it cost:** (i) **two numeric type systems that disagree** — `np.float64` vs `float`,
  `np.bool_` vs `bool`, `np.float32(0.1) != 0.1`, and a scalar-vs-0-d-array confusion that has
  generated bugs for two decades; promotion rules were incoherent enough to need a **rewrite
  in 2023 (NEP 50)** — a breaking change to a mature library. That is the "leaked boxing
  artifact becomes a de-facto surface type" hazard, realized. (ii) **The C-extension bill**:
  ABI churn (each CPython minor rebuilds the world), the GIL shaping the whole "vectorize or
  lose" API philosophy, and build hell — manylinux wheels vendoring OpenBLAS at tens of MB.
  **Lesson:** copy the dtype-on-array *structure*; design against the leaked-scalar outcome
  (§"What a native ndarray arm looks like" names the same widening seam).
- **Julia — the opposite bet.** Types in the *language*: `Int64`, `Float32`, `UInt8` are
  first-class and user-definable, and multiple dispatch specializes generic numeric code to
  machine speed with no C boundary. Genuinely solves what numpy works around.
  **What it cost:** a **JIT and a type-inference engine as hard dependencies**, paid in
  compile latency — the notorious TTFX / "time to first plot" problem — plus a large runtime
  and invalidation cascades. **Decisive for Phalcom:** ADR-0051 §Alternatives *rejected a JIT
  outright* ("Wren — the parity target — is itself a pure interpreter"), and Phalcom's
  start-instant CLI profile is where a tiering JIT loses. Julia's route is closed by a
  committed decision, not by taste.
- **APL / J — the array *is* the primitive.** No dispatch-per-element because there is no
  element-at-a-time: everything is rank-polymorphic, a scalar is a 0-cell. The most coherent
  answer to "dispatch ⊗ elementwise" ever built. **What it cost:** an entire notation and a
  tiny ecosystem; J's rank operator is a language-sized concept itself. Unavailable to a
  Smalltalk-shaped language already committed to scalar message sends.
- **MATLAB — dtype by default.** Everything is a double matrix; a scalar is 1×1.
  **What it cost:** integers were bolted on late and stay second-class; every abstraction is
  forced through matrices; OOP arrived late and awkwardly. The inverse of numpy's failure —
  instead of a leaked array type, leaked *array-ness* in everything.
- **Lua 5.3 — the closest cautionary tale.** A small dynamic language that **did** add an
  integer/float split — the same move ADR-0024 commits to. **What it cost:** lasting
  compatibility pain (`1` vs `1.0` printing, `//` vs `/`, `string.format`, division rules that
  broke existing code), and **LuaJIT never followed**, staying at 5.1 semantics and
  fragmenting the ecosystem to this day. **Directly relevant:** ADR-0024 is unbuilt
  (`class Number {}`, `core.ph:82`). Lua 5.3 is evidence that the split itself is the
  expensive part; `f32`/`u32` on top would compound an unpaid bill.
- **Wren / Lua — deliberate abstention. The most important precedent here.** Wren has exactly
  **one** `Num` (f64) — no integer at all — and nothing beyond `List`. LuaJIT's answer to
  numerics is its **FFI**, widely held to be its best feature. **What it cost:** you simply
  cannot do numerics in Wren; nobody tries, and numeric work leaves the language entirely.
  Wren is Phalcom's *declared parity target* (ADR-0051 §Decision) — so its abstention is
  evidence that a small, fast, embeddable language can decline this whole domain at no cost to
  its actual goals, and LuaJIT's FFI is direct precedent for recommendation (b) above.

## Open questions

| # | Question | Notes |
|---|---|---|
| **M-1** | On a `u32`/`f32` array, what does storing an out-of-range `Int` do — raise, wrap, or promote the array? | **The sharpest conflict with ADR-0024 §2's "no trap and no wraparound".** Every answer reintroduces width semantics. Recommendation leans **raise**, by analogy to ADR-0060 (OOB read total, write raises). Not decided. |
| **M-2** | Is the memory win real enough to justify the arm *at all*, given `f64` → 1.0× post-NaN-boxing? | The whole quantitative case reduces to narrow dtypes (`u8` 8-16×, `f32` 2-4×). If `u8`/bytes is the real motivation, **a `Bytes`/`Buffer` arm is a far smaller ask than an ndarray** and may moot this document. **Weigh this before anything else.** |
| **M-3** | Which dtypes? | `u8`+`f64` is the minimal defensible set (bytes + the one float that round-trips `Float` exactly). Every added dtype multiplies the kernel matrix. |
| **M-4** | Does Phalcom ever want a **value-type / struct** mechanism? | Gates small vectors entirely (§"Vectors vs arrays"). Language-sized; interacts with identity, `==`, GC tracing, and "everything is an object". Should be motivated independently, never by `Vec3`. |
| **M-5** | Views/slices — copy or share? | The `Arc<Vec<u8>>` sketch implies sharing (numpy's model), which imports numpy's aliasing-bug surface *and* a mutation-visibility question ADR-0050's non-moving collector does not answer. Copy-only is duller and safer. |
| **M-6** | **Does an FFI ADR carry its own admission policy?** | **Highest-priority open question here.** Without one, "put it behind FFI" silently repeals ADR-0019 (§"The ADR-0019 tension"). Belongs in `drafts/ffi.md` — **which does not exist**. |
| **M-7** | Does `NDArray` join `Iterable` (ADR-0048), and at what rank? | A 2-d array iterating rows vs elements is numpy's `for row in arr` vs `arr.flat` split. Cursor protocol forbids a cursor ever being `None` (ADR-0048 §1). |
| **M-8** | Is any of this **motivated**? | No unit owns it; no benchmark demands it; no `.ph` program in-tree does numerics. ADR-0051's measure-first law (P1) implies **the honest default is "not yet"** — this document is exploration, and its most likely correct outcome is *decline, and revisit if a real workload appears.* |
