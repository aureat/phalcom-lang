# 009 — superinstructions: fuse `GetSelf -> GetField` (Tier 3) — **NEGATIVE, reverted**

Status: **reverted, not landed** (measured at `d9370e2`) · Unit: [U-IC](../units/U-IC/plan.md) (superinstructions) · Finding: [F21](findings.md#f21--an-arms-code-is-paid-by-every-program-not-the-ones-that-execute-it) (what killed it), [F19](findings.md#f19--a-dispatch-costs-33-ns-and-that-is-what-a-fusion-buys-h13) (the price it was sized with) · Behavior-invariant (no ADR, no floor change)

The next fusion on [instruments.md](instruments.md)'s work list, built exactly as
[cut 008](008-fuse-invoke-pairs.md) built the first two. **The fusion worked and the
cut still lost.** It removed the 6,333,335 dispatches the pair counter predicted — to
the digit, and `method_call` moved −2.2% for them — but it measured **+5.9%
`string_equals`, +4.8% `fib`, +4.4% `for`**: rows that do not execute the new opcode at
all and whose bytecode it does not change.

**The diff is reverted but recorded verbatim below** ([F5](findings.md#f5--the-fiber-pool-hypothesis-was-measured-on-the-wrong-workload)'s
lesson: a negative result that keeps only prose loses its implementation).

## What was built

One fused opcode, 8 B, `Bytecode` does not grow:

```rust
GetSelfField(u16),   // slot
```

`Chunk::fuse_superinstructions` gained a third pair shape, rewriting
`GetSelf -> GetField` in place under the same branch-target guard, with the same
`ip += 2` advance and the same dead second instruction left at `p + 1`.

It is the one fusion that also deletes **work**, not just a dispatch: the unfused pair
pushes the receiver and immediately pops it, and the fused form reads it from
`stack[stack_offset]` without touching the stack top. That extra body saving is why it
was expected to beat, not merely meet, its ceiling.

## The prediction, and that it was met exactly

Ceiling from [F19](findings.md#f19--a-dispatch-costs-33-ns-and-that-is-what-a-fusion-buys-h13),
computed before any code: `6,333,335 × 3.3 ns ÷ 0.440 s` = **~4.8%** on `method_call`.

Counts from a `--features opcode-histogram` build (never timed):

| | base `d9370e2` | fused |
|---|---|---|
| `method_call` total instrs | 48,134,098 | **41,800,763** |
| Δ | | **−6,333,335** |
| `GetSelfField` retired | — | **6,333,335** |

**−6,333,335 is exactly the pair counter's prediction.** The pass fused precisely what
the instrument said it would, a third time. The instrument is not what failed.

## Result — the cut is net negative

Alternating same-session A/B vs `d9370e2`, best-of-7, both binaries built before any
timing, stdout byte-compared every run.

| Benchmark | executes `GetSelfField`? | base | fused | Δ | pairs |
|---|---|---|---|---|---|
| `method_call` | **yes**, 6,333,335 | 0.432 s | 0.422 s | **−2.2%** | `----+--` |
| `for` | no — **zero** | 0.537 s | 0.561 s | **+4.4%** | `+-+++++` |
| `fib` | no — **zero** | 0.649 s | 0.680 s | **+4.8%** | `+++++++` |
| `string_equals` | no — **zero** | 0.826 s | 0.875 s | **+5.9%** | `+++++++` |

The target row moved, under its ceiling. **Three rows that never execute the opcode
regressed ~5%, unanimously.** `for`, `fib` and `string_equals` retire *identical*
instruction counts in both binaries (`fib` 59,136,991 both; `string_equals` 84,000,673
both) and contain zero `GetSelfField`. **A fusion cannot slow down a program whose
bytecode it does not change.** Something else was paying.

## What actually happened — two probes

The suspect list was: the 38th variant (jump-table shape), `Bytecode`'s width, or the
arm's code. `size_of::<Bytecode>()` is **8 B before and after**, so width is out. The
other two were separated by building the opcode but never emitting it — so every row
below runs **byte-identical bytecode to base**:

| build | 38th variant | arm body | opcode emitted | `string_equals` |
|---|---|---|---|---|
| **probe 2** | yes | `unreachable!()` | no | **−0.6%** (`+----+-`, neutral) |
| **probe 1** | yes | real | **no** | **+6.1%** (`+++++++`) |
| cut 009 | yes | real | yes | +5.4 … +5.9% |

**The variant is free; the arm's code is not.** Probe 2 adds the 38th discriminant and
costs nothing — LLVM deletes an `unreachable!()` case, leaving base's loop. Probe 1
adds a real body, never executes it, and loses 6.1%.

Three attempts to buy it back all failed:

- **Outline the shared field read** (`field_at`, `#[inline(never)]`, so the loop holds
  one copy of the `format!` error paths instead of two): `string_equals` **+5.4%**.
- **Outline the whole arm body** (`get_self_field`, `#[inline(never)]`, loop body is a
  single call): `string_equals` **+5.4%**, and `method_call`'s win **inverted to +1.9%**.
- **Reorder the functions in the file** (different code layout, identical semantics —
  the binary's md5 changes): `string_equals` **+5.2%**, `fib` **+5.5%**.

Every build with a real arm lands on the slow side; only the build whose arm the
compiler erases lands on the fast side. It is **systematic, not a per-build coin
flip** — which is what [F21](findings.md#f21--an-arms-code-is-paid-by-every-program-not-the-ones-that-execute-it)
records.

## Why this is not just cut 009's problem

**The tax (~5%) is larger than any remaining fusion's ceiling.** The fattest candidate
left is `GetLocal -> InvokeConst` at ~5.1% on `fib`
([instruments.md](instruments.md#remaining-candidates-at-1d2baea-post-cut-008)). If
every new arm costs ~5% on rows that never run it, **the work list is gated on this
effect, not on F19's ceilings**, and no remaining row's arithmetic closes.

**It also re-prices [cut 008](008-fuse-invoke-pairs.md), which must not be assumed
safe.** 008 added two arms and measured −8.1% on `string_equals` — *better* than the
5.5% its own dispatch ceiling allowed. Under a uniform per-arm tax that is impossible,
so 008 did **not** pay one. Both cannot be true of a simple "each arm costs 5%" rule.
Either the effect has a threshold 008 sat under and 009 crosses, or loop layout is
worth several percent in a way neither cut controlled — in which case **008's headline
is part layout and part fusion, and nobody knows the split.** This repo has been here
before: [F18](findings.md#f18--presizing-a-fibers-vecs-is-negative-and-the-estimate-was-a-sign-error)
is an un-re-derived estimate that was a *sign* error, and
[F16](findings.md#f16--superinstructions-are-premature-no-opcode-histogram-and-the-inliner-already-covers-the-classic-win)'s
third reason survived two re-asks on a guess about existing code. **Do not size the
next fusion until this is settled** (hole [H16](SCOREBOARD.md#6-open-holes--what-is-empty-and-how-to-fill-it)).

## Method

Alternating same-session A/B vs `d9370e2`, best-of-7, both binaries built **before** any
timing, stdout byte-compared on every run, sign read across pairs. Counts from a
separate `--features opcode-histogram` build; **no timing was ever read from a counting
build**. Binaries were copied out to a scratch directory and timed explicitly, since
`--features opcode-histogram` overwrites `target/release/phalcom`.

`cargo test --workspace` was green (26 targets) with the cut applied, including two new
`chunk.rs` unit tests (the fusion, and the branch-target guard refusing a `GetField`
that a jump lands on). **Correctness was never the problem.**

## What is worth doing instead

The ~5% swing is now **the fattest single number on the board** — larger than any
remaining fusion, and it moves rows that execute none of the new code. `run_until_inner`
is one `match` whose code footprint is already ~8× Wren's. Hole
[H16](SCOREBOARD.md#6-open-holes--what-is-empty-and-how-to-fill-it) asks the next
question directly: **is the dispatch loop sitting on a layout cliff, and is outlining
its cold arms (`Class`, `Method`, `Import`, `MakeFamily`, `FinalizeClass`) worth more
than every fusion combined?** If it is, it both recovers this tax and unblocks the work
list. Fusion should not be extended until H16 is answered.

## The reverted diff, verbatim

Applies to `d9370e2`. Correct and green as it stands — it is reverted for being
**slower**, not for being wrong.

```diff
diff --git a/phalcom-core/src/bytecode.rs b/phalcom-core/src/bytecode.rs
index d4ee7fa..3e228b6 100644
--- a/phalcom-core/src/bytecode.rs
+++ b/phalcom-core/src/bytecode.rs
@@ -41,6 +41,7 @@ pub const BYTECODE_NAMES: [&str; Bytecode::VARIANTS] = [
     "FinalizeClass",
     "InvokeLocal",
     "InvokeConst",
+    "GetSelfField",
 ];

 // The set of instructions for our VM. This is the language the compiler "speaks".
@@ -352,12 +353,29 @@ pub enum Bytecode {
     /// Same in-place rewrite, `ip += 2` advance and `ip + 1` cache/span
     /// convention as [`Bytecode::InvokeLocal`].
     InvokeConst(u16, u8, u16),
+
+    /// Fused [`Bytecode::GetSelf`] + [`Bytecode::GetField`] — reads a field of the
+    /// current frame's receiver and pushes it, in one dispatch (perf-log cut 009).
+    ///
+    /// 0: the slot offset of the field in the receiver's slots array (ADR-0011).
+    ///
+    /// Same in-place rewrite and `ip += 2` advance as [`Bytecode::InvokeLocal`],
+    /// with the dead `GetField` left at `ip + 1`. Unlike the fused sends this reads
+    /// no `ip`-indexed side table: `GetField` has neither an inline cache nor a
+    /// span-reporting error path (its failures are
+    /// [`RuntimeError::Internal`](crate::error::RuntimeError::Internal)), so the
+    /// `ip + 1` slots are simply unused here.
+    ///
+    /// Beyond the dispatch, this is the one fusion that also deletes *work*: the
+    /// unfused pair pushes the receiver and immediately pops it, and the fused form
+    /// reads it from `stack[stack_offset]` without ever touching the stack top.
+    GetSelfField(u16),
 }

 impl Bytecode {
     /// Number of distinct opcodes — the length of [`BYTECODE_NAMES`] and of the
     /// histogram in [`opcode_stats`](crate::opcode_stats).
-    pub const VARIANTS: usize = 37;
+    pub const VARIANTS: usize = 38;

     /// This opcode's dense index in `0..VARIANTS`, for array-indexed bookkeeping.
     ///
@@ -405,6 +423,7 @@ impl Bytecode {
             Bytecode::FinalizeClass => 34,
             Bytecode::InvokeLocal(..) => 35,
             Bytecode::InvokeConst(..) => 36,
+            Bytecode::GetSelfField(..) => 37,
         }
     }

diff --git a/phalcom-core/src/chunk.rs b/phalcom-core/src/chunk.rs
index 0dc2a50..4de4415 100644
--- a/phalcom-core/src/chunk.rs
+++ b/phalcom-core/src/chunk.rs
@@ -87,42 +87,47 @@ impl Chunk {
         (self.constants.len() - 1) as u16
     }

-    /// Rewrites statically-adjacent `(GetLocal | Constant) -> Invoke` pairs into the
-    /// fused [`Bytecode::InvokeLocal`] / [`Bytecode::InvokeConst`], each of which
-    /// retires the pair's work in **one** dispatch instead of two (perf-log cut 008).
+    /// Rewrites each statically-adjacent fusible pair into a single fused opcode that
+    /// retires the pair's work in **one** dispatch instead of two:
+    ///
+    /// | pair | fused | cut |
+    /// |---|---|---|
+    /// | `GetLocal -> Invoke` | [`Bytecode::InvokeLocal`] | 008 |
+    /// | `Constant -> Invoke` | [`Bytecode::InvokeConst`] | 008 |
+    /// | `GetSelf -> GetField` | [`Bytecode::GetSelfField`] | 009 |
     ///
     /// Run once per chunk, after compilation, before the [`crate::callable::Callable`]
     /// is frozen.
     ///
     /// # The in-place rewrite, and why there is no re-layout
     ///
-    /// The fused opcode replaces the *first* instruction of the pair and the original
-    /// `Invoke` is left in place at `p + 1` as dead code, so `code.len()` never
-    /// changes. That is what keeps this pass cheap and safe: every jump offset in the
-    /// chunk stays correct, and the `ip`-indexed parallel arrays (`spans`, `caches`,
-    /// `gcaches`) stay aligned with `code`. The alternative — compacting the array —
-    /// would mean rewriting every branch offset and re-indexing three side tables, for
-    /// the same number of saved dispatches.
+    /// The fused opcode replaces the *first* instruction of the pair and the pair's
+    /// *second* instruction is left in place at `p + 1` as dead code, so `code.len()`
+    /// never changes. That is what keeps this pass cheap and safe: every jump offset in
+    /// the chunk stays correct, and the `ip`-indexed parallel arrays (`spans`,
+    /// `caches`, `gcaches`) stay aligned with `code`. The alternative — compacting the
+    /// array — would mean rewriting every branch offset and re-indexing three side
+    /// tables, for the same number of saved dispatches.
     ///
-    /// The dead `Invoke` costs 8 bytes of `code` and is never executed, because
-    /// [`Bytecode::InvokeLocal`]/[`Bytecode::InvokeConst`] advance `ip` past it.
+    /// The dead instruction costs 8 bytes of `code` and is never executed, because
+    /// every fused opcode advances `ip` past it.
     ///
     /// # Why a jump target forbids the fusion
     ///
     /// The rewrite is sound only if `p + 1` is unreachable. If any branch targets the
-    /// `Invoke` directly, that entry point must keep finding a real `Invoke` there —
-    /// so such a pair is skipped. The fallback is simply the unfused pair, which is
-    /// correct, just not fast.
+    /// pair's second instruction directly, that entry point must keep finding a real
+    /// instruction there — so such a pair is skipped. The fallback is simply the
+    /// unfused pair, which is correct, just not fast.
     pub fn fuse_superinstructions(&mut self) {
         let targets = self.branch_targets();
         for p in 0..self.code.len().saturating_sub(1) {
             if targets.contains(&(p + 1)) {
                 continue;
             }
-            let Bytecode::Invoke(arity, selector) = self.code[p + 1] else { continue };
-            self.code[p] = match self.code[p] {
-                Bytecode::GetLocal(slot) => Bytecode::InvokeLocal(slot, arity, selector),
-                Bytecode::Constant(idx) => Bytecode::InvokeConst(idx, arity, selector),
+            self.code[p] = match (self.code[p], self.code[p + 1]) {
+                (Bytecode::GetLocal(slot), Bytecode::Invoke(arity, selector)) => Bytecode::InvokeLocal(slot, arity, selector),
+                (Bytecode::Constant(idx), Bytecode::Invoke(arity, selector)) => Bytecode::InvokeConst(idx, arity, selector),
+                (Bytecode::GetSelf, Bytecode::GetField(slot)) => Bytecode::GetSelfField(slot),
                 _ => continue,
             };
         }
@@ -182,6 +187,28 @@ mod tests {
         assert_eq!(chunk.code[3], Bytecode::Invoke(0, 9));
     }

+    #[test]
+    fn fuses_get_self_get_field() {
+        let mut chunk = chunk_of(&[Bytecode::GetSelf, Bytecode::GetField(2)]);
+        chunk.fuse_superinstructions();
+
+        assert_eq!(chunk.code.len(), 2);
+        assert_eq!(chunk.code[0], Bytecode::GetSelfField(2));
+        assert_eq!(chunk.code[1], Bytecode::GetField(2));
+    }
+
+    #[test]
+    fn refuses_to_fuse_a_get_field_that_is_a_jump_target() {
+        // `Jump(1)` at 0 lands on index 2 — the `GetField`, which at that entry point
+        // must still find a receiver on the stack to pop. Fusing would make the
+        // `GetSelf` that pushes it unreachable.
+        let mut chunk = chunk_of(&[Bytecode::Jump(1), Bytecode::GetSelf, Bytecode::GetField(0)]);
+        chunk.fuse_superinstructions();
+
+        assert_eq!(chunk.code[1], Bytecode::GetSelf, "fused a pair reachable by a jump into its GetField");
+        assert_eq!(chunk.code[2], Bytecode::GetField(0));
+    }
+
     #[test]
     fn refuses_to_fuse_a_pair_whose_invoke_is_a_jump_target() {
         // `Jump(1)` at 0 lands on index 2 — the `Invoke`. Fusing would rewrite the
diff --git a/phalcom-core/src/vm/dispatch.rs b/phalcom-core/src/vm/dispatch.rs
index 2eec3fc..e59487a 100644
--- a/phalcom-core/src/vm/dispatch.rs
+++ b/phalcom-core/src/vm/dispatch.rs
@@ -378,6 +378,61 @@ impl VM {
         Value::Obj(self.heap.alloc(Object::Instance(inst)))
     }

+    /// The whole body of the fused [`Bytecode::GetSelfField`] arm: steps `ip` past
+    /// the dead `GetField` at `ip + 1`, then reads the current frame's receiver's
+    /// field at `slot` and pushes it.
+    ///
+    /// The unfused `GetSelf -> GetField` pair pushes the receiver and immediately
+    /// pops it back off; reading it straight out of `stack[stack_offset]` deletes
+    /// that push/pop along with the dispatch.
+    ///
+    /// `#[inline(never)]` is load-bearing and **measured**. Inlined, this body
+    /// measured **+4.4% `for`, +4.8% `fib`, +5.9% `string_equals`** — rows that
+    /// retire *identical* instruction counts and execute *zero* `GetSelfField`. The
+    /// same arm with an empty body is free, so the cost is this code's presence in
+    /// the dispatch loop, not the 38th variant (perf-log cut 009, F21).
+    ///
+    /// # Errors
+    ///
+    /// Propagates [`Self::field_at`]'s error if the receiver has no fields.
+    #[inline(never)]
+    fn get_self_field(&mut self, stack_offset: usize, slot: u16) -> PhResult<()> {
+        self.frames.last_mut().unwrap().ip += 1;
+        let val = self.field_at(self.stack[stack_offset], slot)?;
+        self.stack.push(val);
+        Ok(())
+    }
+
+    /// Reads `receiver`'s field at `slot`, with absence surfaced (ADR-0007).
+    ///
+    /// Shared verbatim by the [`Bytecode::GetField`] and [`Bytecode::GetSelfField`]
+    /// arms so the two cannot drift — the same reason `Invoke`'s body lives in
+    /// [`Self::invoke_at`].
+    ///
+    /// `#[inline(never)]` is load-bearing and **measured**, not hygiene: inlining
+    /// this body into both arms puts two copies of its `format!` error paths in the
+    /// dispatch loop, which measured **+4.4% `for`, +4.8% `fib`, +5.9%
+    /// `string_equals`** — on programs retiring *identical* instruction counts and
+    /// executing *zero* `GetSelfField`. An arm's code footprint is paid by every
+    /// program, not only the ones that execute it (perf-log cut 009, F21).
+    ///
+    /// # Errors
+    ///
+    /// [`RuntimeError::Internal`] if `receiver` is neither an instance nor a class.
+    #[inline(never)]
+    fn field_at(&self, receiver: Value, slot: u16) -> PhResult<Value> {
+        if let Value::Obj(id) = receiver {
+            if let Some(instance) = self.heap.as_instance(id) {
+                let val = instance.slots.get(slot as usize).copied().unwrap_or(Value::Nil);
+                return Ok(self.surface_absence(val));
+            } else if let Some(class) = self.heap.as_class(id) {
+                let val = class.static_slots.get(slot as usize).copied().unwrap_or(Value::Nil);
+                return Ok(self.surface_absence(val));
+            }
+        }
+        Err(RuntimeError::Internal(format!("Only instances and classes can have fields: {:?}", receiver)).into())
+    }
+
     /// Executes one `Invoke`-shaped send: IC probe, exact-selector lookup + refill,
     /// variadic probe, then `doesNotUnderstand(_)` forward — method-lookup.md §1's
     /// miss order, in order.
@@ -943,20 +998,8 @@ impl VM {
                 }
                 Bytecode::GetField(slot) => {
                     let receiver = self.stack.pop().ok_or("Stack underflow for GetField receiver")?;
-                    match receiver {
-                        Value::Obj(id) => {
-                            if let Some(instance) = self.heap.as_instance(id) {
-                                let val = instance.slots.get(slot as usize).copied().unwrap_or(Value::Nil);
-                                self.stack.push(self.surface_absence(val));
-                            } else if let Some(class) = self.heap.as_class(id) {
-                                let val = class.static_slots.get(slot as usize).copied().unwrap_or(Value::Nil);
-                                self.stack.push(self.surface_absence(val));
-                            } else {
-                                return Err(RuntimeError::Internal(format!("Only instances and classes can have fields: {:?}", receiver)).into());
-                            }
-                        }
-                        _ => return Err(RuntimeError::Internal(format!("Only instances and classes can have fields: {:?}", receiver)).into()),
-                    }
+                    let val = self.field_at(receiver, slot)?;
+                    self.stack.push(val);
                 }
                 Bytecode::SetField(slot) => {
                     let value_to_assign = self.stack.pop().ok_or("Stack underflow on field assignment")?;
@@ -1048,6 +1091,11 @@ impl VM {
                     self.frames.last_mut().unwrap().ip += 1;
                     self.invoke_at(callable, ip + 1, arity, selector_idx)?;
                 }
+                // Fused `GetSelf -> GetField` (cut 009). The unfused pair pushes the
+                // receiver and immediately pops it back off; reading it straight out
+                // of `stack[stack_offset]` deletes that push/pop along with the
+                // dispatch. Steps `ip` past the dead `GetField` at `ip + 1`.
+                Bytecode::GetSelfField(slot) => self.get_self_field(stack_offset, slot)?,
                 Bytecode::GetUpvalue(idx) => {
                     let cell = self.heap.closure(closure_id).upvalues[idx as usize];
                     let value = match *self.heap.upvalue(cell) {
```

## Write-set (reverted)

- `phalcom-core/src/bytecode.rs` — `GetSelfField` variant, `BYTECODE_NAMES`, `index()`, `VARIANTS` 37 → 38.
- `phalcom-core/src/chunk.rs` — third pair shape in `fuse_superinstructions`, 2 unit tests.
- `phalcom-core/src/vm/dispatch.rs` — `GetSelfField` arm, `field_at`/`get_self_field` helpers.

Nothing landed. Floor: +0. No `unsafe`.
