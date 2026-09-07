# F.2 Dynamic-Pack Scratch Window Correctness — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` (recommended) or `superpowers:executing-plans` to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Fix the F.2 static outgoing positional-spread return-window regression and eliminate the underlying compiler bug that allows compiler-synthesized pack scratch locals to alias already-evaluated operand-stack values in nested expressions.

**Architecture:** Keep F.2/F.3 pack assembly, `ArgumentPackBuilderObject`, `InvokePack`, `SuperSendPack`, selector derivation, rest fallback, and evaluation order unchanged. Replace the unsafe assumption that a newly declared scratch local can be materialized by appending `Nil` to the top of the operand stack with explicit VM stack-splice bytecodes that insert/remove compiler-only scratch slots at their true frame-local indices while preserving any operand suffix above the local window. Refactor every F.2 pack scratch allocation/release through one shared helper, then promote the existing pending regression fixture and add adversarial nested-window coverage.

**Tech Stack:** Rust; Phalcom AST/compiler; bytecode VM; `cargo test`; end-to-end `.ph` language fixtures.

## Global Constraints

- Baseline repository state: commit `5e3a417a03a88f9001d09ace276f9cda65dc1667` (`test(lang): retire stale pending fixtures`).
- F.3 rest lanes are already landed in the baseline through commit `938486c7f9d6b642c98cfa730c1d8e0ff25e7ec7`; this fix MUST preserve F.3 behavior.
- Public primitive-floor delta: **0**.
- Do not redesign F.2 pack representation, dynamic selector derivation, argument ordering, rest dispatch, or dNU forwarding.
- Do not change source-language evaluation order to work around a stack-layout bug.
- Do not special-case `System.print`, nested method calls, or the specific regression fixture.
- Do not introduce a fixed-size global scratch pool or an arbitrary maximum nesting depth.
- Do not route `*` exhaustion through Rust-side iteration; generic positional expansion must continue to use the ordinary Phalcom cursor protocol so fibers/yields remain valid.
- Scratch slots are compiler implementation details and MUST NOT be source-addressable, capturable, or observable.
- Existing static non-pack `Invoke` fast paths MUST remain byte-for-byte semantically unchanged.
- Runtime stack manipulation introduced by this fix must preserve every value above the local window in the same relative order.
- Every task must keep the tree testable; no task may knowingly leave a broken intermediate bytecode index/name table.

---

## 1. Baseline and confirmed regression

The cleanup commit leaves this intentional pending test:

- harness: `phalcom-core/tests/lang.rs::variadics_pending`
- source: `phalcom-core/tests/lang/variadics/pending/variadics_spread_call.ph`
- expected: `phalcom-core/tests/lang/variadics/pending/variadics_spread_call.expected`

Current fixture:

```phalcom
// area: variadics
// spec: F.2-outgoing-pack-assembly-and-dynamic-send-amended.md
// status: PENDING
// Static outgoing `*` forwarding returns correctly from `add`, but currently
// leaves the outer receiver window misbound for `System.print`.

class Adder {
  add(_ a, _ b, _ c) {
    return a + b + c;
  }
}
const args = [1, 2, 3]
System.print(Adder.new().add(*args))
```

Expected stdout:

```text
6
```

This is not a missing language feature. F.2 specifies outgoing positional spread and the inner `add(*args)` dispatch works. The failure is a compiler/VM stack-window correctness bug that becomes visible because the dynamic-pack expression is nested inside an already-partially-evaluated outer send.

### 1.1 Current unsafe helper

`phalcom-core/src/compiler/lib/expr.rs` currently contains:

```rust
fn reserve_pack_scratch(
    &mut self,
    base: &str,
    range: SourceRange,
) -> Result<u16, CompilerError> {
    let name = self.fresh_scratch_symbol(base);
    self.add_local(name, true)?;
    let slot = (self.functions.last().unwrap().num_locals - 1) as u16;
    self.emit(Bytecode::Nil, range);
    Ok(slot)
}
```

The compiler metadata assumes the newly-added local occupies runtime frame slot:

```text
frame.stack_offset + slot
```

but `Bytecode::Nil` only appends to the top of the current value stack.

Those are the same location only when the current frame has **no operand suffix above its live locals**.

### 1.2 Why the regression occurs

For the nested expression:

```phalcom
System.print(Adder.new().add(*args))
```

the outer static send evaluates `System` before its argument. At the point the inner dynamic send asks for its first pack scratch local, the conceptual stack is:

```text
[ frame locals ... | System ]
                    ^ already-evaluated outer receiver
```

Assume there are `N` live locals. `add_local` declares scratch slot `N`, but appending `Nil` produces:

```text
[ frame locals ... | System | None ]
                    ^ slot N   ^ actual appended value
```

`SetLocal(N)` then writes into `System`, because local slot lookup is defined by:

```rust
let local_idx = stack_offset + slot as usize;
```

The scratch declaration therefore aliases the outer receiver instead of the appended placeholder.

The inner dynamic call may still compute `6`, but the enclosing call window has already been corrupted.

### 1.3 The bug is broader than `System.print`

Any F.2/F.3 dynamic-pack lowering can currently be unsafe when compiled after one or more already-evaluated operands. Examples include:

```phalcom
100 + Adder.new().add(*args)
```

where `100` is already on the stack while compiling the RHS;

```phalcom
sink.accept("first", Adder.new().add(*args))
```

where the outer receiver and an earlier argument are live;

```phalcom
["first", Adder.new().add(*args)]
```

where an earlier literal element is live;

and deeper combinations containing dynamic callable sends, dynamic super sends, dynamic subscript operations, or dynamic Tuple assembly.

The implementation MUST fix the general invariant, not only the first observed fixture.

---

## 2. Root invariant to establish

For every executing frame, define:

```text
L = frame-local window
O = transient operand suffix
```

The runtime stack must always have the form:

```text
[ ... caller state ... | L | O ]
```

A compiler-synthesized scratch local increases `L` by one **without changing `O`**:

```text
before:
[ ... | L | O ]

reserve scratch S:
[ ... | L | S | O ]
```

Releasing that scratch performs the exact inverse:

```text
before release:
[ ... | L | S | O ]

after release:
[ ... | L | O ]
```

The relative ordering and values of every element of `O` must be identical before and after the splice.

This invariant must hold whether `O` is empty or contains:

- an enclosing receiver;
- earlier arguments;
- the LHS of a binary expression;
- earlier collection-literal elements;
- intermediate receiver/property values;
- the result of an inner call;
- any combination of the above.

---

## 3. Ratified implementation decision

### D1 — Add two compiler-only local-window splice bytecodes

Add:

```rust
Bytecode::ReserveScratchLocal(u16)
Bytecode::ReleaseScratchLocal(u16)
```

These are VM bytecodes, but they are **not language operations** and create no primitive binding.

They are intentionally general scratch-window mechanics rather than F.2-specific pack opcodes because the defect is a frame-layout defect, not a pack semantic.

### D2 — `ReserveScratchLocal(slot)` inserts, never appends

Runtime contract:

```text
absolute = current_frame.stack_offset + slot

before:
stack = prefix ++ operand_suffix
        where len(prefix) == absolute

after:
stack = prefix ++ [Value::Nil] ++ operand_suffix
```

The operation MUST use insertion at `absolute`, not `push`.

Use the private `Value::Nil` sentinel as the initial content because this is an uninitialized compiler-only local slot, not a user-visible expression result. Any defensive read through ordinary `GetLocal` already surfaces `Value::Nil` as `None`.

Bounds rule:

```text
absolute <= stack.len()
```

Otherwise raise `RuntimeError::Internal` with enough information to diagnose the bad slot and stack length. Do not panic.

### D3 — `ReleaseScratchLocal(slot)` removes exactly that frame slot

Runtime contract:

```text
absolute = current_frame.stack_offset + slot

before:
stack = prefix ++ [scratch] ++ operand_suffix

after:
stack = prefix ++ operand_suffix
```

Bounds rule:

```text
absolute < stack.len()
```

Otherwise raise `RuntimeError::Internal`.

The bytecode MUST NOT inspect, pop, duplicate, stringify, or otherwise alter `operand_suffix`.

### D4 — Release multiple scratch locals highest-slot-first

Given scratch slots `N`, `N+1`, `N+2`, release:

```text
N+2
N+1
N
```

Removing a lower slot first shifts higher absolute indices and complicates correctness. Descending release makes each encoded slot remain valid until removed.

### D5 — Pack result remains on top; never move it into a scratch slot for cleanup

The current `collapse_two_pack_scratch` / `collapse_three_pack_scratch` scheme uses `SetLocal(first_slot)` followed by `Pop`s to preserve a top result while reclaiming scratch values. That only works if the scratch locals themselves are the topmost values.

After this change, cleanup is structural:

```text
[ ... | scratch slots | enclosing operands | result ]
```

`ReleaseScratchLocal` removes scratch slots in place, leaving:

```text
[ ... | enclosing operands | result ]
```

No `SetLocal` result shuffle is necessary or permitted.

### D6 — Do not pre-reserve a fixed scratch pool

Rejected alternatives:

- reserving an arbitrary fixed count of scratch locals per frame;
- statically estimating a maximum and emitting a function-wide pool;
- special-casing only nested method-call arguments;
- reordering receiver/argument evaluation;
- keeping dead scratch slots alive for the rest of the function;
- spilling enclosing operands into a second ad-hoc temporary mechanism.

The two splice bytecodes are smaller, compositional under arbitrary nesting, and make the runtime invariant explicit.

---

## 4. File map

### Modify

`phalcom-core/src/bytecode.rs`
- Add `ReserveScratchLocal(u16)` and `ReleaseScratchLocal(u16)`.
- Append them to the bytecode index/name bookkeeping.
- Increase `Bytecode::VARIANTS` from 60 to 62.
- Add rustdoc specifying exact stack contracts and compiler-only status.

`phalcom-core/src/vm/dispatch.rs`
- Implement both bytecode handlers using current frame `stack_offset`.
- Use `Vec::insert` / `Vec::remove` at the absolute frame-local index.
- Return structured internal errors for invalid indices; never panic.

`phalcom-core/src/compiler/lib/expr.rs`
- Rewrite `reserve_pack_scratch` to emit `ReserveScratchLocal(slot)` instead of `Nil`.
- Replace `collapse_two_pack_scratch` and `collapse_three_pack_scratch` with one general scratch-release helper.
- Replace manual source/cursor `Pop` cleanup in `compile_positional_pack_expansion` with structural release.
- Audit every dynamic-pack lowering call site for balanced reserve/release behavior.

`phalcom-core/tests/lang.rs`
- Remove `variadics_pending` after all its fixtures are promoted.

### Move/promote

`phalcom-core/tests/lang/variadics/pending/variadics_spread_call.ph`
→ `phalcom-core/tests/lang/variadics/variadics_spread_call.ph`

`phalcom-core/tests/lang/variadics/pending/variadics_spread_call.expected`
→ `phalcom-core/tests/lang/variadics/variadics_spread_call.expected`

### Create

`phalcom-core/tests/lang/variadics/variadics_spread_nested_operand_windows.ph`

`phalcom-core/tests/lang/variadics/variadics_spread_nested_operand_windows.expected`

Optional, only if the repository already centralizes bytecode table consistency tests in an existing test module:
- extend that existing module for `Bytecode::VARIANTS`, `index()`, and name-table coverage; do **not** create a second bytecode census framework.

---

## 5. Exact compiler helper contracts

### 5.1 `reserve_pack_scratch`

Replace the current body with behavior equivalent to:

```rust
fn reserve_pack_scratch(
    &mut self,
    base: &str,
    range: SourceRange,
) -> Result<u16, CompilerError> {
    let name = self.fresh_scratch_symbol(base);
    self.add_local(name, true)?;
    let slot = (self.functions.last().unwrap().num_locals - 1) as u16;
    self.emit(Bytecode::ReserveScratchLocal(slot), range);
    Ok(slot)
}
```

Required properties:

- compiler `locals.len()` and `num_locals` increase exactly once;
- `max_slots` continues to update through `add_local`;
- no ordinary `Nil` opcode is emitted;
- the scratch is present at its true local slot before any `SetLocal(slot)`;
- nested dynamic-pack compilation may safely reserve additional scratch locals.

### 5.2 Replace specialized collapse helpers

Introduce one helper, for example:

```rust
fn release_pack_scratch_from(
    &mut self,
    first_slot: u16,
    count: usize,
    range: SourceRange,
) {
    for i in (0..count).rev() {
        self.emit(
            Bytecode::ReleaseScratchLocal(first_slot + i as u16),
            range,
        );
    }

    let function = self.functions.last_mut().unwrap();
    for _ in 0..count {
        function.locals.pop();
    }
    function.num_locals -= count;
}
```

Before using this exact shape, assert during implementation that every caller reserves a contiguous suffix of locals. The current F.2 helpers do. If any audited caller violates that invariant, fix that caller rather than weakening the release helper.

Do not decrement `max_slots`; it is a peak allocation count and must remain a high-water mark.

Add debug assertions in compiler code where useful:

```rust
debug_assert_eq!(function.locals.len(), function.num_locals);
```

and, before releasing, verify that:

```text
first_slot + count == function.num_locals
```

in debug builds. A scratch release that is not the current local suffix is a compiler bug.

### 5.3 `compile_positional_pack_expansion`

Current source/cursor cleanup:

```rust
self.emit(Bytecode::Pop, range);
self.emit(Bytecode::Pop, range);
let function = self.functions.last_mut().unwrap();
function.locals.pop();
function.locals.pop();
function.num_locals -= 2;
```

must be replaced by:

```rust
self.release_pack_scratch_from(source_slot, 2, range);
```

The required order is:

```text
source_slot     = N
cursor_slot     = N + 1
release N + 1
release N
```

No operand must be popped as a side effect.

### 5.4 `compile_dynamic_method_send`

After `InvokePack`, replace:

```rust
self.collapse_two_pack_scratch(receiver_slot, range);
```

with:

```rust
self.release_pack_scratch_from(receiver_slot, 2, range);
```

The send result remains at stack top throughout cleanup.

### 5.5 Dynamic unqualified callable send

The `Expr::UnqualifiedCall` dynamic-pack branch contains an explicit comment warning that a callable receiver may sit above partial-expression values. Preserve that intent, but rely on the corrected scratch mechanism rather than assuming `reserve_pack_scratch` appends safely.

After `InvokePack`, use the shared structural release helper.

### 5.6 Audit dynamic super send, subscript get/set, and dynamic Tuple paths

Search `expr.rs` for all of:

```text
reserve_pack_scratch(
collapse_two_pack_scratch
collapse_three_pack_scratch
NewArgumentPack
InvokePack
SuperSendPack
FinishTuplePack
```

For every path, record:

1. scratch slots reserved;
2. reservation order;
3. whether they form a contiguous local suffix;
4. exact instruction that produces the user-visible expression result;
5. release point;
6. required preserved operand suffix.

Every path must end with balanced structural release. No path may retain old result-shuffling cleanup.

Expected principal shapes include:

```text
dynamic method send:
  receiver, builder

dynamic callable send:
  receiver, builder

positional `*` generic fallback:
  source, cursor

dynamic subscript write:
  receiver, builder, rhs/result preservation scratch as currently designed

dynamic Tuple:
  builder plus any nested spread source/cursor scratch
```

Do not change the semantic ordering of any of these paths while changing scratch lifetime mechanics.

---

## 6. VM bytecode specification

### 6.1 `ReserveScratchLocal`

Add at the end of the `Bytecode` enum so existing dense indices 0–59 remain stable:

```rust
/// Inserts one compiler-only local slot into the current frame's local window.
///
/// `slot` is frame-relative. The VM inserts `Value::Nil` at
/// `frame.stack_offset + slot`, shifting every transient operand at or above
/// that index upward by one while preserving order. This is used only for
/// compiler-synthesized scratch locals that must be declared while an enclosing
/// expression already has operands on the stack.
ReserveScratchLocal(u16),

/// Removes one compiler-only local slot from the current frame's local window.
///
/// `slot` is frame-relative. The VM removes exactly the value at
/// `frame.stack_offset + slot`, shifting the operand suffix downward by one
/// while preserving order. Multi-slot scratch regions are released
/// highest-slot-first by the compiler.
ReleaseScratchLocal(u16),
```

Then:

```rust
pub const VARIANTS: usize = 62;
```

Append dense indices:

```rust
Bytecode::ReserveScratchLocal(..) => 60,
Bytecode::ReleaseScratchLocal(..) => 61,
```

Append matching names to `BYTECODE_NAMES` in the same order.

Any exhaustive bytecode-format/disassembly match elsewhere must be updated in the same commit.

### 6.2 VM handlers

Implement next to `GetLocal` / `SetLocal`, not in the F.2 pack-builder handler cluster. These are local-window operations.

Reference behavior:

```rust
Bytecode::ReserveScratchLocal(slot) => {
    let local_idx = stack_offset + slot as usize;
    if local_idx > self.stack.len() {
        return Err(RuntimeError::Internal(format!(
            "scratch local slot {slot} insertion index {local_idx} exceeds stack length {}",
            self.stack.len()
        ))
        .into());
    }
    self.stack.insert(local_idx, Value::Nil);
}

Bytecode::ReleaseScratchLocal(slot) => {
    let local_idx = stack_offset + slot as usize;
    if local_idx >= self.stack.len() {
        return Err(RuntimeError::Internal(format!(
            "scratch local slot {slot} removal index {local_idx} exceeds stack length {}",
            self.stack.len()
        ))
        .into());
    }
    self.stack.remove(local_idx);
}
```

Implementation notes:

- `stack_offset` is already copied from the current frame at the dispatch-loop head.
- Do not call `surface_absence` during reserve/release.
- Do not allocate a heap object.
- Do not invoke user code.
- `Vec::insert`/`Vec::remove` is acceptable here: the displaced suffix is transient operand state, ordinarily only a handful of values, while correctness takes priority over introducing a much larger stack-segment abstraction.
- If profiling later proves this path hot, optimize only after the correctness regression is closed and benchmarked.

---

## 7. Required regression fixtures

### 7.1 Promote the minimal reproducer

Move the existing pending fixture to the active variadics lane and change only its status/comment wording:

`phalcom-core/tests/lang/variadics/variadics_spread_call.ph`

```phalcom
// area: variadics
// spec: F.2-outgoing-pack-assembly-and-dynamic-send-amended.md
// status: PASS
// Regression: a nested dynamic-pack send must not alias or overwrite the
// already-evaluated receiver window of the enclosing static send.

class Adder {
  add(_ a, _ b, _ c) {
    return a + b + c
  }
}

const args = [1, 2, 3]
System.print(Adder.new().add(*args))
```

Expected file:

`phalcom-core/tests/lang/variadics/variadics_spread_call.expected`

```text
6
```

Do not weaken the fixture to assign the inner result to a temporary first. The nesting is the regression.

### 7.2 Add adversarial operand-window coverage

Create:

`phalcom-core/tests/lang/variadics/variadics_spread_nested_operand_windows.ph`

```phalcom
// area: variadics
// spec: F.2-outgoing-pack-assembly-and-dynamic-send-amended.md
// status: PASS
// The dynamic-pack compiler may reserve scratch locals while arbitrary
// enclosing operands are already live. None of those operands may be moved,
// overwritten, or consumed by scratch setup/cleanup.

class Adder {
  add(_ a, _ b, _ c) => a + b + c
}

class Pairer {
  pair(_ left, _ right) => "\(left):\(right)"
}

const args = [1, 2, 3]

// Outer receiver live.
System.print(Adder.new().add(*args))

// Binary LHS live while RHS reserves pack scratch.
System.print(100 + Adder.new().add(*args))

// Outer receiver plus an earlier positional argument live.
System.print(Pairer.new().pair("kept", Adder.new().add(*args)))

// Earlier List element live while the later element reserves pack scratch.
const values = ["kept", Adder.new().add(*args)]
System.print(values[0])
System.print(values[1])
```

Expected:

`phalcom-core/tests/lang/variadics/variadics_spread_nested_operand_windows.expected`

```text
6
106
kept:6
kept
6
```

If the current string interpolation or subscript surface makes one of these lines invalid at baseline, replace only that assertion with the nearest already-supported active syntax; do not delete the semantic category it represents. The categories that must remain are: outer receiver, binary LHS, prior argument, prior literal element.

### 7.3 Optional callable-path fixture

If no active F.2 fixture already covers a **nested dynamic callable `call(*args)`** with an enclosing operand suffix, add a third fixture using the current block syntax. It must prove the dynamic `Expr::UnqualifiedCall` path is protected by the same scratch mechanism.

Do not add it if equivalent coverage already exists; prefer one regression per distinct compiler path, not duplicate examples.

---

## 8. Test matrix

The implementation is incomplete unless all of these categories are covered either by existing active fixtures or the new fixtures above.

| Category | Required result |
|---|---|
| Static outer send + inner dynamic `*` | enclosing receiver preserved |
| Binary expression + inner dynamic `*` | LHS preserved |
| Prior outer argument + inner dynamic `*` | receiver and earlier args preserved |
| Collection literal prefix + inner dynamic `*` | prior elements preserved |
| Dynamic ordinary method send | unchanged |
| Dynamic implicit-self send | unchanged |
| Dynamic callable send | unchanged |
| Dynamic super send | unchanged |
| Dynamic subscript read | unchanged |
| Dynamic subscript write | unchanged; RHS result rule preserved |
| Dynamic Tuple assembly | unchanged |
| Tuple/Unit `*` fast lane | unchanged |
| Generic iterable `*` | unchanged cursor protocol |
| F.3 exact-before-rest lookup | unchanged |
| F.3 rest capture | unchanged Unit/Tuple capture semantics |
| dNU on dynamic selector | unchanged |
| >255 dynamic send arity error | unchanged |
| pack-specific structured runtime errors | unchanged |
| static no-pack `Invoke` | no new scratch bytecodes |

---

## 9. Task-by-task implementation plan

### Task 1: Reproduce and freeze the regression

**Files:**
- Read: `phalcom-core/tests/lang/variadics/pending/variadics_spread_call.ph`
- Read: `phalcom-core/tests/lang/variadics/pending/variadics_spread_call.expected`
- No source modification yet.

**Interfaces:**
- Consumes: current ignored `variadics_pending` harness.
- Produces: captured proof that the baseline fails before the fix and passes `add(*args)` far enough to expose outer-window corruption.

- [ ] **Step 1: Verify clean baseline**

```bash
git status --short
git rev-parse HEAD
```

Expected:

```text
# no status output
5e3a417a03a88f9001d09ace276f9cda65dc1667
```

If HEAD has advanced intentionally, record the new SHA in the implementation commit message and re-run the code-location audit before editing.

- [ ] **Step 2: Run the ignored regression alone**

```bash
cargo test -p phalcom-core --test lang variadics_pending -- --ignored --nocapture
```

Expected before fix: FAIL. The fixture must not complete with exact stdout `6` under the pending harness. Preserve the actual failure output in implementation notes; do not alter `.expected` to match the bug.

- [ ] **Step 3: Run active variadics as control**

```bash
cargo test -p phalcom-core --test lang variadics -- --nocapture
```

Expected: PASS.

- [ ] **Step 4: Commit**

No commit; this task is evidence-only.

---

### Task 2: Add scratch-window splice bytecodes

**Files:**
- Modify: `phalcom-core/src/bytecode.rs`
- Modify: `phalcom-core/src/vm/dispatch.rs`
- Modify only existing bytecode census/name/disassembly files if the compiler identifies exhaustive matches.

**Interfaces:**
- Produces: `Bytecode::ReserveScratchLocal(u16)` and `Bytecode::ReleaseScratchLocal(u16)` with the contracts in §6.
- Consumed by: Task 3 compiler scratch helper.

- [ ] **Step 1: Add enum variants and documentation**

Append the variants after `FinishTuplePack` so indices 0–59 do not move.

- [ ] **Step 2: Update dense bookkeeping atomically**

Set:

```rust
Bytecode::VARIANTS = 62
```

Append indices 60 and 61 and matching names.

- [ ] **Step 3: Implement VM handlers**

Place handlers alongside `GetLocal` / `SetLocal`. Use checked absolute indices and `RuntimeError::Internal` on impossible compiler-generated slot misuse.

- [ ] **Step 4: Compile immediately**

```bash
cargo check -p phalcom-core
```

Expected: PASS. Any exhaustive-match error is a required bookkeeping location; update it now rather than deferring it.

- [ ] **Step 5: Run bytecode/disassembly tests**

```bash
cargo test -p phalcom-core --test lang iteration_disasm -- --nocapture
```

If the repository exposes bytecode-census unit tests under another target, run those as well. Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add phalcom-core/src/bytecode.rs phalcom-core/src/vm/dispatch.rs
git commit -m "fix(vm): splice compiler scratch locals safely"
```

---

### Task 3: Refactor F.2 scratch allocation and release

**Files:**
- Modify: `phalcom-core/src/compiler/lib/expr.rs`

**Interfaces:**
- Consumes: Task 2 scratch bytecodes.
- Produces: stack-window-safe pack scratch helpers used by all dynamic-pack lowering.

- [ ] **Step 1: Rewrite `reserve_pack_scratch`**

Use `ReserveScratchLocal(slot)` and remove the ordinary `Nil` emission.

- [ ] **Step 2: Add `release_pack_scratch_from`**

Release highest slot first; pop matching compiler-local metadata; decrement `num_locals`; never decrement `max_slots`.

- [ ] **Step 3: Add debug suffix assertions**

Assert the released range is the current local suffix and that `locals.len() == num_locals` before/after the metadata update.

- [ ] **Step 4: Replace `collapse_two_pack_scratch`**

All former two-slot callers must use structural release. Delete the old helper.

- [ ] **Step 5: Replace `collapse_three_pack_scratch`**

All former three-slot callers must use structural release. Delete the old helper.

- [ ] **Step 6: Fix positional-expansion source/cursor cleanup**

Delete the two runtime `Pop`s and manual local metadata pops. Release the two scratch slots structurally.

- [ ] **Step 7: Audit every reserve call**

Run:

```bash
rg -n "reserve_pack_scratch|collapse_(two|three)_pack_scratch|NewArgumentPack|InvokePack|SuperSendPack|FinishTuplePack" phalcom-core/src/compiler/lib/expr.rs
```

Expected when complete:

- `collapse_two_pack_scratch`: 0 matches
- `collapse_three_pack_scratch`: 0 matches
- every `reserve_pack_scratch` lifetime ends in `release_pack_scratch_from`

- [ ] **Step 8: Compile**

```bash
cargo check -p phalcom-core
```

Expected: PASS.

- [ ] **Step 9: Run previously failing pending test**

```bash
cargo test -p phalcom-core --test lang variadics_pending -- --ignored --nocapture
```

Expected: PASS with exact fixture stdout `6`.

- [ ] **Step 10: Run active F.2/F.3-adjacent lanes**

```bash
cargo test -p phalcom-core --test lang dispatch -- --nocapture
cargo test -p phalcom-core --test lang variadics -- --nocapture
cargo test -p phalcom-core --test lang collections -- --nocapture
cargo test -p phalcom-core --test lang iteration -- --nocapture
```

Expected: all PASS.

- [ ] **Step 11: Commit**

```bash
git add phalcom-core/src/compiler/lib/expr.rs
git commit -m "fix(packs): preserve enclosing operand windows"
```

---

### Task 4: Promote the minimal regression fixture

**Files:**
- Move: `phalcom-core/tests/lang/variadics/pending/variadics_spread_call.ph` → `phalcom-core/tests/lang/variadics/variadics_spread_call.ph`
- Move: `phalcom-core/tests/lang/variadics/pending/variadics_spread_call.expected` → `phalcom-core/tests/lang/variadics/variadics_spread_call.expected`
- Modify: moved `.ph` status/comment only.

**Interfaces:**
- Produces: permanent active coverage for the original regression.

- [ ] **Step 1: Move both sidecars together**

```bash
git mv phalcom-core/tests/lang/variadics/pending/variadics_spread_call.ph \
       phalcom-core/tests/lang/variadics/variadics_spread_call.ph
git mv phalcom-core/tests/lang/variadics/pending/variadics_spread_call.expected \
       phalcom-core/tests/lang/variadics/variadics_spread_call.expected
```

- [ ] **Step 2: Update fixture header**

Use the PASS text from §7.1. Preserve the nested call itself.

- [ ] **Step 3: Run active variadics**

```bash
cargo test -p phalcom-core --test lang variadics -- --nocapture
```

Expected: PASS, including `variadics_spread_call`.

- [ ] **Step 4: Commit**

```bash
git add phalcom-core/tests/lang/variadics
git commit -m "test(variadics): promote static spread window regression"
```

---

### Task 5: Add adversarial operand-window fixture

**Files:**
- Create: `phalcom-core/tests/lang/variadics/variadics_spread_nested_operand_windows.ph`
- Create: `phalcom-core/tests/lang/variadics/variadics_spread_nested_operand_windows.expected`

**Interfaces:**
- Produces: proof that scratch splicing works for multiple operand suffix shapes, not only an outer receiver.

- [ ] **Step 1: Add the source fixture**

Use §7.2 verbatim, adjusting only syntactic details that are demonstrably unsupported on the current baseline while retaining all four semantic categories.

- [ ] **Step 2: Add exact expected stdout**

Use the five-line output in §7.2.

- [ ] **Step 3: Run the lane**

```bash
cargo test -p phalcom-core --test lang variadics -- --nocapture
```

Expected: PASS.

- [ ] **Step 4: Temporarily mutation-test the fix**

Locally revert only the compiler helper from `ReserveScratchLocal(slot)` back to the old `Nil` append behavior without committing, then rerun the new fixture.

Expected: at least the original nested receiver assertion fails; preferably additional categories fail. Restore the correct implementation immediately afterward.

This step proves the fixture actually guards the defect rather than merely exercising spread.

- [ ] **Step 5: Confirm tree contains only intended changes**

```bash
git diff --check
git status --short
```

- [ ] **Step 6: Commit**

```bash
git add phalcom-core/tests/lang/variadics/variadics_spread_nested_operand_windows.*
git commit -m "test(packs): cover nested scratch operand windows"
```

---

### Task 6: Retire `variadics_pending`

**Files:**
- Modify: `phalcom-core/tests/lang.rs`
- Remove directory only if empty: `phalcom-core/tests/lang/variadics/pending/`

**Interfaces:**
- Produces: no ignored test for an already-fixed F.2 correctness bug.

- [ ] **Step 1: Confirm pending directory has no remaining fixtures**

```bash
find phalcom-core/tests/lang/variadics/pending -maxdepth 1 -type f -print
```

Expected: no files.

If another intentional pending fixture exists, do not remove the harness; instead change its ignore reason to name that remaining feature exactly. The F.2 return-window regression itself must no longer be pending.

- [ ] **Step 2: Remove `variadics_pending` when empty**

Delete:

```rust
#[test]
#[ignore = "F.2 static outgoing positional spread return-window regression"]
fn variadics_pending() {
    support::check_pending("variadics");
}
```

- [ ] **Step 3: Run language suite**

```bash
cargo test -p phalcom-core --test lang
```

Expected: PASS for all active tests; ignored count decreases by one if the harness was removed.

- [ ] **Step 4: Commit**

```bash
git add phalcom-core/tests/lang.rs phalcom-core/tests/lang/variadics
git commit -m "test(lang): retire fixed F.2 spread pending lane"
```

---

### Task 7: Full F.2/F.3 regression gate

**Files:** none unless failures reveal a real regression.

- [ ] **Step 1: Run focused language lanes**

```bash
cargo test -p phalcom-core --test lang dispatch -- --nocapture
cargo test -p phalcom-core --test lang variadics -- --nocapture
cargo test -p phalcom-core --test lang collections -- --nocapture
cargo test -p phalcom-core --test lang sequence -- --nocapture
cargo test -p phalcom-core --test lang iteration -- --nocapture
cargo test -p phalcom-core --test lang runtime_errors -- --nocapture
```

Expected: all PASS.

- [ ] **Step 2: Run full crate tests**

```bash
cargo test -p phalcom-core
```

Expected: PASS except the intentionally ignored feature gaps that remain outside this bug-fix scope.

- [ ] **Step 3: Run workspace verification**

```bash
./scripts/verify.sh
```

Expected: exit 0.

- [ ] **Step 4: Run docs build**

```bash
cargo doc --workspace --no-deps
```

Expected: no new warnings attributable to this change.

- [ ] **Step 5: Static fast-path audit**

Use the existing disassembly tooling/fixtures for an ordinary static call with no expansion. Confirm it contains no:

```text
ReserveScratchLocal
ReleaseScratchLocal
NewArgumentPack
InvokePack
```

This fix must impose zero bytecode/runtime overhead on static no-pack sends.

- [ ] **Step 6: Final diff audit**

```bash
git diff --check
git status --short
git log --oneline -6
```

Expected: clean tree and only the planned commits.

---

## 10. Correctness proof obligations

The reviewer should reject the implementation unless each obligation below is demonstrated.

### C1 — Local-slot identity

Immediately after every `reserve_pack_scratch`, the runtime value physically located at:

```text
frame.stack_offset + returned_slot
```

is the newly reserved scratch value, regardless of operand suffix length.

### C2 — Operand-suffix preservation

For arbitrary suffix `O`:

```text
reserve(S); release(S)
```

is observationally identity on `O`.

### C3 — Nested scratch composition

For:

```text
reserve A
reserve B
release B
release A
```

both locals are independently addressable by their compiler-assigned slots and all pre-existing operands survive.

### C4 — Result preservation

If a dynamic send leaves result `R` above scratch locals and enclosing operands, scratch release leaves `R` at the top unchanged.

### C5 — Evaluation order

Receiver, pack items, expansion sources, labels, values, RHS values, and enclosing expression operands execute in the same lexical order as before this fix.

### C6 — Fiber safety

Generic positional `*` continues using `iterate` / `iteratorValue` bytecode sends. The new local splice operations do not introduce Rust-side interpreter re-entry or hold unrooted object handles across safepoints.

### C7 — F.3 compatibility

Exact selector lookup still precedes rest fallback, and F.3 rest capture remains Unit/Tuple based. No List-rest compatibility path may reappear.

### C8 — Static fast path

Expressions that do not require a dynamic outgoing pack emit no scratch splice bytecodes.

---

## 11. Error and panic policy

Both scratch bytecodes are compiler-generated and invalid slots indicate an internal compiler/bytecode consistency defect. Therefore surface them as `RuntimeError::Internal`, not user-facing `ArgumentError` or F.2 pack errors.

Required diagnostic data:

```text
opcode kind
frame-relative slot
absolute stack index
current stack length
```

Do not include arbitrary user-value stringification in these errors.

No code path may index `Vec` unchecked for these operations.

---

## 12. Performance considerations

`Vec::insert` and `Vec::remove` shift the operand suffix and are O(number of transient operands above the local window).

This is acceptable for this correction because:

1. the suffix is generally shallow;
2. only dynamic-pack paths pay it;
3. static no-pack sends are unaffected;
4. pack building itself is already a cold/complex path relative to static `Invoke`;
5. the alternative—function-wide temp-slot allocation or a second segmented stack abstraction—is substantially more complex and carries more correctness risk.

Do not add a specialized small-vector stack or preallocated scratch pool in this fix.

After correctness lands, a benchmark may compare nested dynamic-pack workloads before/after. Performance follow-up is justified only by measured regression, not by the asymptotic operation in isolation.

---

## 13. Interaction with the remaining intentional pending gaps

After this fix, the F.2 static `*args` return-window pending item should disappear completely.

Two categories may still remain intentionally pending at the repository level:

### 13.1 System runtime APIs

`System.args`, `System.clock`, and broader IO/process/runtime-environment surface are feature-design/stdlib gaps, not bugs in F.2. This plan does not implement them.

### 13.2 Collection literal spread and boundedness

The compiler still has explicit pending branches for:

```text
List literal `*` expansion
Map literal `**` expansion
Set literal `*` expansion
```

The current boundedness module already exposes `require_exhaustible` for Spec-F positional expansion and recognizes concrete List/Tuple/Map/Set literals as bounded. Those collection-literal expansion gaps deserve their own implementation specification because their destination semantics differ from outgoing argument packs:

- List/Set `*` contributes elements;
- Map `**` contributes arbitrary key/value associations in encounter order;
- Map-literal duplicates must fail rather than overwrite;
- collection construction must preserve lexical evaluation order;
- generic exhaustion must remain fiber-safe;
- Map `**` destination semantics permit arbitrary valid Map keys, unlike call/Tuple/Record labeled expansion which requires Symbols.

Do **not** opportunistically reuse `ArgumentPackBuilderObject` as a collection builder while fixing this regression. Sharing the boundedness/cursor-exhaustion machinery is appropriate; sharing outgoing-call pack representation is not.

A separate collection-spread plan should begin only after this scratch-window fix lands, because its compiler-generated exhaustion loops may themselves need safe scratch locals in nested expressions.

---

## 14. Reviewer checklist

- [ ] Original pending reproducer fails on baseline and passes after fix.
- [ ] Existing pending fixture is promoted unchanged in semantic shape.
- [ ] New adversarial fixture covers at least outer receiver, binary LHS, prior argument, and prior literal element.
- [ ] `reserve_pack_scratch` no longer emits ordinary `Nil` as a positional append surrogate for a local slot.
- [ ] Scratch reserve inserts at `stack_offset + slot`.
- [ ] Scratch release removes at `stack_offset + slot`.
- [ ] Multi-slot release is descending.
- [ ] Old `collapse_two_pack_scratch` is deleted.
- [ ] Old `collapse_three_pack_scratch` is deleted.
- [ ] Positional spread source/cursor cleanup contains no blind `Pop` pair.
- [ ] Every pack scratch reservation has balanced structural release.
- [ ] `Bytecode::VARIANTS`, dense index mapping, and bytecode names are consistent.
- [ ] No new public primitive binding.
- [ ] No change to source evaluation order.
- [ ] No change to static no-pack `Invoke` path.
- [ ] F.3 rest tests remain green.
- [ ] Full `phalcom-core` test suite passes.
- [ ] `./scripts/verify.sh` passes.
- [ ] `cargo doc --workspace --no-deps` introduces no new warnings.
- [ ] `variadics_pending` is removed if its directory becomes empty.

---

## 15. Completion criteria

This work is complete when all of the following are true:

1. `System.print(Adder.new().add(*args))` prints exactly `6`.
2. Nested dynamic-pack expressions preserve all enclosing operand-stack values.
3. F.2 scratch locals never alias a transient operand.
4. Scratch cleanup never pops or overwrites an enclosing operand to reclaim locals.
5. All F.2 dynamic send forms continue to pass.
6. All F.3 rest-lane tests continue to pass.
7. The original fixture is active, not ignored.
8. A broader adversarial operand-window regression fixture is active.
9. `variadics_pending` is gone unless another independently specified pending variadics feature remains.
10. Static no-pack sends emit no new scratch machinery.
11. Public primitive-floor delta remains zero.
12. Full repository verification is green.

---

## 16. Suggested commit sequence

Keep review boundaries small and bisectable:

```text
fix(vm): splice compiler scratch locals safely
fix(packs): preserve enclosing operand windows
test(variadics): promote static spread window regression
test(packs): cover nested scratch operand windows
test(lang): retire fixed F.2 spread pending lane
```

Do not squash until review is complete. If a later step exposes a flaw in an earlier architectural choice, amend/rework that commit rather than layering a compensating special case.

---

## 17. Final design summary

The defect exists because the compiler conflates two different operations:

```text
"append one operand to the stack"
```

and:

```text
"grow the current frame's local window by one slot"
```

They happen to be equivalent at statement boundaries with an empty operand suffix, but they are not equivalent inside arbitrary expressions.

The fix makes the distinction explicit:

```text
ReserveScratchLocal(slot)
    grows the frame-local window at the correct absolute index

ReleaseScratchLocal(slot)
    shrinks that same window while preserving the operand suffix
```

F.2/F.3 pack lowering can then remain semantically unchanged while becoming safe under arbitrary expression nesting. This is the narrowest fix that establishes the correct invariant instead of patching one observed call shape.
