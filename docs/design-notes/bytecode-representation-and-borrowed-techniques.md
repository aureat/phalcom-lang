# Bytecode representation vs. borrowed VM techniques — findings (2026-07-14)

**Status: FINDINGS.** Grounded in HEAD (`2b75429`) while writing
[U-IC/implementation-spec.md](../forge/units/U-IC/implementation-spec.md) and
[U-HOTPATH/implementation-spec.md](../forge/units/U-HOTPATH/implementation-spec.md). Governed by
[performance.md](../spec/v0.2/performance.md) + [ADR-0051](../adr/0051-performance-strategy-measure-first-tiered-optimization.md)
(measure-first, tiered, behavior-invariant).

Companion to [optimization-method-and-harness-fidelity.md](optimization-method-and-harness-fidelity.md).
That file is about *measuring*. This one is about a prior step: **whether a technique's precondition
holds here at all.** The trigger was U-IC's planned Change 3 (operand-free superinstructions
`LOAD_LOCAL_0..15`, ported from Wren), dropped after inspection. The generalizable lesson is not
about superinstructions.

---

## B1 — `Vec<Bytecode>` is not a bytestream, and the difference eats a class of techniques

`bytecode.rs:2-3`:

```rust
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Bytecode {
    GetLocal(u16),
    // ...
    SuperSend(u8, u16, u16),
}
```

`Chunk.code` is `Vec<Bytecode>` (`chunk.rs:8`), and the dispatch loop reads it as
(`vm/dispatch.rs:412-415`):

```rust
let chunk = &self.heap.closure(closure_id).callable.chunk;
(chunk.code[ip], chunk.spans[ip])
```

One indexed load pulls **the whole instruction** — discriminant *and* operand — into registers,
sized by the widest variant (`SuperSend`, ~8 bytes). `match opcode` (`dispatch.rs:431`) switches on
the discriminant; the operand is already in a register from the same load.

Wren runs a `uint8_t* ip` over a byte array. `CODE_LOAD_LOCAL` is one byte and its operand is a
**second** byte, fetched by a separate `READ_BYTE()` = `(*ip++)`. Read byte, dispatch, read another
byte, index.

> **The precondition for operand-folding is a separate operand fetch.** We do not have one. There is
> no second read for `GetLocal0` to delete — the load it would remove is the load that delivered the
> opcode itself, and that one is not skippable.

## B2 — Opcode budgets are a bytestream artifact, not a VM law

U-IC's plan.md carried a "**opcode-budget check first** (`u8` = 256 slots)" gate. That constraint is
Wren's because Wren's opcode *is* a byte in a byte array. Phalcom's discriminant is whatever rustc
picks, inside a value already 8 bytes wide for `SuperSend`'s payload — new variants sit in existing
padding and do not grow `Bytecode` until the tag itself overflows, far past any opcode set we would
write.

The gate was inherited along with the technique and was never true here. **A borrowed constraint
needs the same verification as a borrowed optimization.**

## B3 — `match` gives one indirect branch; computed goto gives one per opcode

The *second* win Wren takes from opcode specialization is predictor-side: with computed goto, each
opcode has its own dispatch site, so the indirect-branch predictor keeps a separate entry per opcode
and per-opcode history becomes learnable.

Rust's `match` lowers to a single jump table behind **one** indirect branch. Sixteen specialized
`GetLocal0..15` arms give sixteen more table entries behind that same branch: no added prediction,
more I-cache pressure, more code to maintain.

> Reaching for the predictor win means **threaded dispatch** (a `&&label`-style or tail-call
> dispatcher), which is a distinct, invasive change with its own risk profile — not a side effect of
> adding opcode variants.

## B4 — What survives: fusion, which cuts dispatches rather than fetches

The technique adjacent to the dropped one is still live. Collapsing `GetLocal, GetLocal, Invoke`
into one `InvokeLocals(a, b, sel)` removes **dispatches** — loop iterations, `match`es, `ip`
bookkeeping — not operand fetches. That win is immune to B1 and B3 alike, because it does not depend
on how an instruction is encoded or dispatched, only on how many there are.

Not scoped anywhere yet, and it wants profile data first (which triples actually dominate) per
ADR-0051. Registered here so the B1 finding is not read as "superinstructions are useless for
Phalcom" — the operand-folding *variant* is what does not apply.

## B5 — The generalizable rule

Three ports were on the table from the same Wren analysis; one had no precondition here, one
(register-hoisting `ip`/chunk into loop locals) does and is U-HOTPATH Change 1, and one
(`wrenGetClassInline` arm ordering) is a wash LLVM may already do.

> **Before porting a technique, name the property of the *source* VM it exploits, then check that
> property in ours.** Wren's byte-array `ip` and computed-goto dispatch are the properties behind
> superinstructions. Neither holds. The technique's *description* ("fold the operand into the
> opcode") survives the translation to our vocabulary and sounds sensible; its *mechanism* does not.

This is the same failure mode as [O1](optimization-method-and-harness-fidelity.md) from the other
direction: there, a cost that was real but invisible on the headline bench; here, a win that reads as
real but has nothing to remove. Both are caught by looking at the emitted work, not the technique's
name.
