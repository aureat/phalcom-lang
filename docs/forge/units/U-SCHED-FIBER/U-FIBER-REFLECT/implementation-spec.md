# U-FIBER-REFLECT — Implementation Spec: `Fiber#isDone` / `Fiber#error`

Companion to [`plan.md`](plan.md) (work order, build order, test strategy,
traceability). This document is the Rust-level reference an implementer
codes directly against — exact signatures, exact registration diff, exact
fixture skeletons. No design decisions live here; all judgment calls are
already closed in `plan.md` §2/§7.

Grounds: [concurrency.md §1](../../../../spec/current/concurrency.md) Interface
table · [ADR-0030](../../../../adr/0030-fibers-and-futures-cooperative-concurrency.md).

---

## 1. Target files

| File | Role |
|---|---|
| `phalcom-core/src/primitive/fiber.rs` | add `fiber_is_done`, `fiber_error` |
| `phalcom-core/src/universe/primitives.rs` | register both as instance `Getter`s on `fiber_cls`, next to the existing block at L324–333 |
| `phalcom-core/tests/lang/concurrency/` + `MANIFEST.md` | goldens |
| `docs/spec/current/concurrency.md` | flip implementation-status note |
| `docs/forge/units/U-FUTURE/plan.md` | companion edit, `plan.md` §4 |

## 2. `fiber_is_done`

```rust
/// Signature: `Fiber#isDone` — `true` once the receiver is `Done` or `Failed`.
///
/// # Errors
///
/// Returns [`RuntimeError::Type`] if `receiver` is not a `Fiber`
/// ([`expect_fiber`]).
pub fn fiber_is_done(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let fiber_ref = expect_fiber(vm, receiver)?;
    let status = vm.heap.fiber(fiber_ref).status;
    Ok(Value::Bool(matches!(status, FiberStatus::Done | FiberStatus::Failed)))
}
```

Placement: directly below `fiber_current` (L127–129 of the current
`primitive/fiber.rs`), before `fiber_abort`. Reuses `expect_fiber` (L97–102)
— no new resolution helper.

## 3. `fiber_error`

```rust
/// Signature: `Fiber#error` — the captured `Error` as `Option`, if the
/// receiver is `Failed`; `None` otherwise (including `Done`, where `result`
/// holds the return value, not an `Error`).
///
/// # Errors
///
/// Returns [`RuntimeError::Type`] if `receiver` is not a `Fiber`.
pub fn fiber_error(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let fiber_ref = expect_fiber(vm, receiver)?;
    let fiber = vm.heap.fiber(fiber_ref);
    if fiber.status == FiberStatus::Failed {
        let error = fiber.result;
        Ok(crate::primitive::nil::wrap_some(vm, error))
    } else {
        Ok(vm.none_value())
    }
}
```

`wrap_some` is `pub(crate)` in `primitive/nil.rs:47` (used by `some_new`) —
confirm `primitive/fiber.rs` can see it (same crate, `pub(crate)` suffices;
widen only if module-privacy blocks it, which it should not).

## 4. Registration diff (`universe/primitives.rs`)

Insert after L333 (`primitive_static!(vm, fiber_cls, "abort", ...)`):

```rust
primitive!(vm, fiber_cls, "isDone", SignatureKind::Getter, fiber_is_done);
primitive!(vm, fiber_cls, "error", SignatureKind::Getter, fiber_error);
```

Both instance-side (`primitive!`, not `primitive_static!`) — matches
`concurrency.md §1`'s Interface table ("Side: instance" for both rows).
Import both fns in the `use` block at the top of `primitives.rs` alongside
the existing `fiber_new, fiber_call, fiber_try, fiber_yield, fiber_current,
fiber_abort` import line.

## 5. Floor-census bump

`+2` in the same commit as the registration. Locate the current total via
`graphify query "floor census primitive count"` or the invariant-check
comment in `universe/core_classes.rs`; do not hand-guess the prior total.

## 6. Golden fixture skeletons (`tests/lang/concurrency/`)

```phalcom
// isDone_false_while_suspended.ph — PASS
let f = Fiber.new { Fiber.yield(1); 2 }
f.call()
System.print(f.isDone)   // expect: false
```

```phalcom
// isDone_true_once_done.ph — PASS
let f = Fiber.new { 42 }
f.call()
System.print(f.isDone)   // expect: true
System.print(f.error)    // expect: None
```

```phalcom
// isDone_and_error_once_failed.ph — PASS
let f = Fiber.new { Error.new("boom").raise() }
let e = f.try()
System.print(f.isDone)          // expect: true
System.print(f.error == Some(e)) // expect: true — identity, not a copy
```

Reuse the setup from the already-landed
`fiber_call_finished_uncaught.ph`/`fiber_try_finished_uncaught.ph`
(`tests/lang/concurrency/negative/`) for the `Failed`-path fixture rather
than inventing new raise machinery. Check for and graduate
`pending/concurrency_fiber_wren_is_done_and_error.ph` if it exists on HEAD.

## 7. Verification

```sh
./scripts/verify.sh          # exits 0
cargo doc --workspace --no-deps   # clean, no missing-docs warnings
```

Both new `pub fn`s need full rustdoc (crate convention, `CLAUDE.md`
"Documentation is mandatory") — the doc comments above are the actual text
to ship, not illustrative.
