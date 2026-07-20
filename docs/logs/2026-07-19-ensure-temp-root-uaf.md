# `ensure` returned a dangling handle when its cleanup block collected

- Date: 2026-07-19
- Commits: `cdd2117` (`primitive/block.rs`, `vm/gc.rs`, `vm/bootstrap.rs`), `vm/mod.rs` in the
  preceding commit, tests in `42aafce`
- Realizes: [ADR-0050](../adr/accepted/0050-non-moving-mark-sweep-collector.md) §7 (the
  `push_temp_root` escape hatch), closes [ffi.md](../spec/v0.2/drafts/ffi.md) **F-2**
- Supersedes: [U-GC IMPL-SPEC-steps-3-5.md](../forge/units/U-GC/IMPL-SPEC-steps-3-5.md) §2.1's
  "do not build `VM::temp_roots`" instruction — see §4 below

## 1. The bug

`block_ensure` ran the protected block, held its outcome in a Rust local, then ran the cleanup
block. The cleanup block re-enters the interpreter, so its back-edge safepoint can collect. The
pending outcome was reachable from neither `vm.stack` nor `vm.frames` — it lived only in Rust — so
the collector swept it and `ensure` returned a freed handle.

Reproduced from `.ph`:

```phalcom
let result = {
  let protectedList = List.new()
  protectedList.add(7)
  protectedList
}.ensure({
  let i = 0
  while (i < 6000) { List.new(); i = i + 1 }   // allocates past the 4096 threshold
})
System.print(result.size)
```

```
survived cleanup
thread 'main' panicked at phalcom-core/src/heap/mod.rs:188:48:
dangling ObjRef ObjRef(1541v1)
```

The panic lands *after* the cleanup block has completed, which is what made it hard to attribute
to `ensure` rather than to whatever ran next.

## 2. Why the shipped audit missed it

`IMPL-SPEC-steps-3-5.md` §2.1 ran an audit and concluded "the intersection is EMPTY. Zero sites
need a temp root," and on that basis instructed implementers **not** to build `temp_roots`.

Its predicate was *"functions containing both an allocation and a re-entrant call."* `block_ensure`
allocates nothing. It is **two sequential re-entrant calls with a live value between them** — a
shape the predicate cannot see.

The corrected predicate is *"a handle held in a Rust local across a re-entrant call."* Allocation
is irrelevant: Invariant L already makes `Heap::alloc` latch rather than collect, which is exactly
why the alloc-shaped hazard came out empty and why the re-entrant-shaped one was never counted.

## 3. The fix

`VM::temp_roots: Vec<ObjRef>`, enumerated by `collect_roots` (the exhaustive destructure forces
every future field to be classified, so this could not be added silently). API:

```rust
vm.push_temp_root(value)        // no-op for immediates
vm.temp_root_depth() -> usize
vm.truncate_temp_roots(depth)
```

Depth-and-truncate rather than push-and-pop: a primitive's re-entrant call can return through `Ok`,
a raised `Err`, or a non-local return, and truncation is correct on all three without the caller
counting its own pushes.

`block_ensure` roots both arms — the `Ok` value, and a `Raise`'s `error` payload (the surface
`Error` instance an enclosing `on` receives).

## 4. Re-audit result

Every re-entrant call site in `phalcom-core/src` was re-checked under the corrected predicate.
`block_ensure` is the **only** site. Specifically clean:

- `bool_if_true` / `bool_if_false` — allocate *after* the call returns, nothing held across.
- `block_while_true` — the condition is destructured to a `Bool` immediate before the body call.
- `block_on` — `error` is the receiver of the `isA` send and an argument to the handler call, so
  it is on `vm.stack` for both; the gap between them contains no re-entrant call.
- `send_hash` / `send_eq` — return Rust scalars; their callers' keys are primitive arguments,
  which stay on `vm.stack` for the primitive's whole duration.
- Everything else in the grep is a tail call.

**`IMPL-SPEC-steps-3-5.md` §2.1 is now wrong in its instruction and should be corrected**, or the
next implementer will read "dead scaffolding, do not build" and delete this.

## 5. Verification

- Both `.ph` repros: panic before, correct output after.
- `cargo test -p phalcom-core` — 137 passed, 0 failed across `gc`, `golden`, `invariants`, `lang`,
  `collections_contract`, `contracts_metadata`, `disasm_super`.
- **Negative control run** (fix reverted in a detached worktree at `293e923`, main tree untouched):

  ```
  test ensure_outcome_survives_collecting_cleanup ... FAILED
  thread '...' panicked at phalcom-core/src/heap/mod.rs:188:48:
  dangling ObjRef ObjRef(1540v1)
  test result: FAILED. 15 passed; 1 failed
  ```

  Same panic, same site as the original CLI repro. The test is non-vacuous and pins this
  regression.

## 6. Known gap — the error-path test is a guard, not a regression test

The negative control settles what §5 previously left open:
`ensure_raised_error_survives_collecting_cleanup` **passes with the fix disabled.** The raised
`Error` instance stays reachable by some other path, so this test would not catch a re-introduced
bug on the `Raise` arm.

The arm is still rooted, because the hazard is structural — a `Value` in a Rust local across a
re-entrant call — and the two arms are one code path. But the test earns no more credit than
"documents the intent." Anyone hardening this should find what keeps the raised instance alive and
either build a case that defeats it, or delete the arm's rooting as provably unnecessary. Do not
read the green as coverage.
