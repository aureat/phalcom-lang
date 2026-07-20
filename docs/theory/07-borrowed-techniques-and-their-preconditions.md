# 07 — Borrowed techniques and their preconditions

> **The rule, in one sentence:** before porting a technique from another implementation, name
> the property of *that* implementation which the technique exploits, then check whether your
> implementation has it. A technique's *description* survives translation between vocabularies;
> its *mechanism* often does not.

**`[V]`** Source: `docs/design-notes/bytecode-representation-and-borrowed-techniques.md`
(2026-07-14, grounded at commit `2b75429`). Status: FINDINGS. Three ports were on the table from
one analysis of Wren; one had no precondition here at all, one did and was scheduled, and one was
a wash.

---

## 1. The case that produced the rule

The planned change was **operand-free superinstructions** — replacing `LOAD_LOCAL n` with sixteen
specialized opcodes `LOAD_LOCAL_0` through `LOAD_LOCAL_15`, a well-known technique ported from
Wren. It was dropped after inspection, and the reason is instructive because the technique is
genuinely real *in the source system*.

**`[V]`** Wren runs a `uint8_t *ip` over a byte array. `CODE_LOAD_LOCAL` occupies one byte and its
operand occupies a **second** byte, fetched by a separate `READ_BYTE()` = `(*ip++)`. Read a byte,
dispatch, read another byte, index. Folding the operand into the opcode deletes the second fetch.
That is the win, and it is a real win — *given a bytestream*.

**`[V]`** Phalcom has no bytestream. `Chunk.code` is a `Vec<Bytecode>` where `Bytecode` is a Rust
enum whose widest variant (`SuperSend(u8, u16, u16)`) is about eight bytes. The dispatch loop does:

```rust
let chunk = &self.heap.closure(closure_id).callable.chunk;
(chunk.code[ip], chunk.spans[ip])
```

One indexed load pulls **the entire instruction** — discriminant *and* operand — into registers.
The `match` switches on the discriminant; the operand is already in a register from the same load.
The finding states it exactly:

> **The precondition for operand-folding is a separate operand fetch.** We do not have one. There
> is no second read for `GetLocal0` to delete — the load it would remove is the load that
> delivered the opcode itself, and that one is not skippable.

---

## 2. Borrowed constraints need the same scrutiny as borrowed optimizations

**`[V]`** This is the sharper half of the finding, and the half most likely to generalize to any
project that reads other implementations for ideas.

The port's plan carried a gate: "**opcode-budget check first** (`u8` = 256 slots)." That
constraint is *Wren's*, and it is true there because Wren's opcode literally is a byte in a byte
array. Phalcom's discriminant is whatever the compiler picks, sitting inside a value already eight
bytes wide to accommodate `SuperSend`'s payload — new variants occupy existing padding and do not
grow `Bytecode` at all until the tag itself overflows, which is far past any opcode set anyone
would write.

> The gate was inherited along with the technique and was never true here. **A borrowed constraint
> needs the same verification as a borrowed optimization.**

Constraints travel more invisibly than optimizations, because nobody audits a restriction — a
restriction only ever costs you options, never correctness, so it never announces itself as wrong.
A phantom budget silently shapes years of design decisions.

---

## 3. The second win, which also does not transfer

**`[V]`** Wren takes a *second* benefit from opcode specialization, on the branch-predictor side.
With computed-goto dispatch, each opcode has its own dispatch site, so the indirect-branch
predictor maintains a separate entry per opcode and per-opcode history becomes learnable.

Rust's `match` lowers to a single jump table behind **one** indirect branch. Sixteen specialized
`GetLocal0..15` arms add sixteen table entries behind that same branch: no additional prediction
capability, more instruction-cache pressure, and more code to maintain. The finding's conclusion:

> Reaching for the predictor win means **threaded dispatch** (a `&&label`-style or tail-call
> dispatcher), which is a distinct, invasive change with its own risk profile — not a side effect
> of adding opcode variants.

Note the structure of that reasoning. Two independent benefits are bundled together in the source
system by an implementation accident; in the destination system they decouple, and *both* turn out
to depend on properties the destination lacks. A summary of the technique ("specialize opcodes,
get faster") preserves neither dependency.

---

## 4. What survived, and why the distinction matters

**`[V]`** The adjacent technique is still live. **Fusion** — collapsing `GetLocal, GetLocal,
Invoke` into a single `InvokeLocals(a, b, sel)` — removes *dispatches*: loop iterations, `match`
evaluations, `ip` bookkeeping. It removes no operand fetches.

That difference is exactly what makes it immune to both objections above. It does not depend on
how an instruction is *encoded* (so the `Vec<Bytecode>` representation is irrelevant) nor on how
it is *dispatched* (so the single-indirect-branch `match` is irrelevant). It depends only on how
*many* instructions there are.

The finding is explicit that this must be recorded, so the earlier result is not overread:

> Registered here so the B1 finding is not read as "superinstructions are useless for Phalcom" —
> the operand-folding *variant* is what does not apply.

**A generalizable habit:** when a technique fails a precondition check, immediately ask which
*neighboring* technique in the same family does not depend on the failed precondition. Failure
analyses that stop at "no" throw away the most valuable output — the map of which property was
actually load-bearing, which is precisely what tells you where to look next.

---

## 5. The rule, and its symmetry with the measurement failure mode

**`[V]`**

> **Before porting a technique, name the property of the *source* VM it exploits, then check that
> property in ours.** Wren's byte-array `ip` and computed-goto dispatch are the properties behind
> superinstructions. Neither holds. The technique's *description* ("fold the operand into the
> opcode") survives the translation to our vocabulary and sounds sensible; its *mechanism* does
> not.

The design note closes by connecting this to its companion finding on measurement, and the pairing
is the most useful part:

> This is the same failure mode as O1 from the other direction: there, a cost that was real but
> invisible on the headline bench; here, a win that reads as real but has nothing to remove. Both
> are caught by looking at the emitted work, not the technique's name.

**"Look at the emitted work, not the name"** is the whole discipline compressed. A name is a
compression of a mechanism, lossy in exactly the dimensions that determine whether the mechanism
applies to you. See [`08-performance-epistemology.md`](08-performance-epistemology.md) for the
measurement half.

---

## 6. Where else this bites

**`[O]`** Candidate list of techniques whose preconditions are worth stating explicitly *before*
anyone schedules them, since each is routinely described in terms that hide its dependency:

- **NaN-boxing** — exploits that pointers fit in 48 bits and that a language's numeric type is
  IEEE-754 doubles. **`[V]`** Phalcom's ADR-0024 commits to a split numeric surface with an
  auto-promoting bignum `Int`, which introduces a heap kind. Whether NaN-boxing still pays under a
  split tower is a genuinely different question from whether it pays under flat `f64`, and the
  deferral (ADR-0010/ADR-0044) predates the split.
- **Inline caches** — exploit that a call site's receiver class is stable *and* that invalidation
  is cheap. **`[V]`** Phalcom's preconditions are recorded as unmet: `Symbol` is one mixed
  namespace and needs a selector-only interner first; the IC seam is a comment. Sealed superclasses
  (ADR-0026/0041) remove one invalidation axis entirely, which is a genuine simplification —
  a future IC keys on `ClassId` with **no invalidate-on-reparent case**.
- **Precompiled bootstrap image** — exploits that startup cost dominates for short-lived processes.
  **`[V]`** Filed as a *startup* lever explicitly barred from being re-sold as a throughput lever,
  because the steady-state benchmark harness is blind to it by construction. The reasoning behind
  that bar is itself borrowed carefully: AOT trades peak for predictability, which is why an
  AOT-shaped runtime with no type feedback has a real ceiling.
