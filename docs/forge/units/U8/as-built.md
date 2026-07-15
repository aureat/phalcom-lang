# U8 — doesNotUnderstand & perform (as-built)

- **Status:** ✅ Landed — `b99ad22` (runtime), `806c9ea` (acceptance corpus), `83221e8` (forge docs)
- **Realizes:** [ADR-0012](../../../adr/0012-selector-signature-encoding-and-dispatch.md); spec [method-lookup.md §2](../../../spec/v0.2/method-lookup.md), [messages-and-selectors.md §5](../../../spec/v0.2/messages-and-selectors.md)
- **Reviewer gate:** OFF per the load-bearing-only review policy (STATE.md) — self-verified on the green gate (`./scripts/verify.sh` exit 0, `cargo doc` clean, clippy clean).

## Mission
Turn a method-lookup miss from a hard VM error into a re-sendable `doesNotUnderstand(_:)`
message, so user code (e.g. a proxy subclass) can intercept unknown sends. Ship the
reflective send surface (`perform`/`respondsTo`), reify the missed send as a first-class
`Message`, and stand up the shared runtime-send helper (`VM::send_dynamic`) that both the
dNU forward and `perform` ride on. The `Bytecode::SendDynamic` opcode named in the plan's
write-set was deliberately deferred to U9 (nothing emits a spread call site yet).

## Surface / behavior
- **Miss → dNU forward.** A send whose selector is not found no longer raises; it is reified
  as a `Message` and re-sent as `doesNotUnderstand(_:)` up the receiver's chain.
- **`Object.doesNotUnderstand(_:)`** — default implementation raises
  `MessageNotUnderstood`, rendered `"{receiver} does not understand '{selector}'"`. A
  subclass can override it to intercept.
- **`Object.perform(_:)` / `perform(_:_:)`** — reflective send by selector symbol (plus an
  args `List` for the two-arg form).
- **`Object.respondsTo(_:)`** — exact-selector probe; returns `Bool`, never triggers dNU.
- **`Message`** — reified send with `selector`, `name`, `labels`, `args` accessors.

```phalcom
class Proxy {
  doesNotUnderstand(msg) {
    System.print("intercepted: " + msg.name)
    return None
  }
}
Proxy.new().greet()          // prints "intercepted: greet"
42.perform(#"+", List.new().add(1))   // reflective send
42.respondsTo(#"+")          // true, no dNU
```

## Implementation
- **`vm.rs`** — the `Bytecode::Invoke` miss arm reifies a `Message` and forwards
  `doesNotUnderstand(_:)`. Recursion guard: a receiver whose chain lacks
  `doesNotUnderstand(_:)` is `RuntimeError::Internal`, never re-sent as another dNU.
  New **`VM::send_dynamic(receiver, selector, args)`** — saves the frame count, pushes
  receiver+args at a fresh stack window, dispatches via `lookup_method` + `call_method`
  (falling through to the same dNU forward on a miss), then re-enters `run_until` to drain
  one activation and return a synchronous `Value`. Same re-entrancy pattern as `block_call`,
  so it is callable from inside a primitive. `VM::new_message` builds the `Message` directly.
- **`method.rs`** — new **`decode_selector`** (exact inverse of `encode_selector`, total —
  garbage decodes to `Getter`, never panics), used to decompose a selector into
  `name`/`labels` for the `Message`. 5 unit tests (round-trip over all six `SignatureKind`s,
  labeled selectors, setter-vs-operator disambiguation, garbage totality, subscripts).
- **`primitive/object.rs`** — `doesNotUnderstand(_:)`, `perform(_:)`/`perform(_:_:)`,
  `respondsTo(_:)` native methods.
- **`universe.rs`** — the kernel `Message` class: a four-slot `InstanceObject` built in Rust
  (slots `selector`/`name`/`labels`/`args`), field count stamped in `VM::new` mirroring
  `Some` — **no `.ph`**, because a `class X {}` reopen never applies a compiler field layout
  to a bootstrapped row, so a `.ph` `construct` would not work. Accessors are native getters;
  `labels` uses `""` for a positional argument so `labels.size == args.size`.
- **`error.rs`** — `RuntimeError::MethodNotFound` retired → `MessageNotUnderstood {
  selector, receiver }`.
- **No `core.ph` edit** — everything is primitives and `add_class!` already registers the
  `Message` global (a subset of the plan's write-set).

## Invariants & tests
- 5 `dispatch` PASS goldens: Proxy/dNU forwarding, `Message` shape (encoder-inverse),
  `perform` reflective parity, `respondsTo` true/false, `dispatch_dnu_preserves_dispatch`
  (dNU slow path does not corrupt dispatch — guards a future inline cache).
- 1 NEGATIVE `runtime-errors` golden: `perform` of an unknown selector re-enters dNU exactly
  once (no infinite loop).
- 4 behavior-change goldens updated to the new `MessageNotUnderstood` text:
  `runtime_unknown_method`, `runtime_and_non_boolean_operand`,
  `runtime_comparison_unsupported`, `runtime_inline_guard_wrong_type`.
- 5 `method` unit tests for `decode_selector`.

## Deviations & deferrals
- **No `Bytecode::SendDynamic` opcode this unit** — a dead opcode with a guessed operand
  layout would be untestable and pre-empt U9's design; only the `send_dynamic` *helper*
  shipped. Opcode + call-site spread (`f(*args)`) → [forge/DEFERRED.md](../../DEFERRED.md) #21
  (later superseded — spread stays a future unit's job, not U9's).
- **Per-class dNU handler cache** not built (miss path is slow-by-design) →
  [forge/DEFERRED.md](../../DEFERRED.md) #22.
- **`perform(_:_:)` selector/arity not pre-validated** — a mismatch surfaces via ordinary
  lookup (miss → dNU), not an eager `ArgumentError` → [forge/DEFERRED.md](../../DEFERRED.md) #23.
- dNU render format fixed as `"{receiver} does not understand '{selector}'"` (implementer's call).

## Sources
- [forge/archive/phase2/STATE.md](../../archive/phase2/STATE.md) "U8 — LANDED"; [forge/archive/phase2/PHASE2-INDEX.md](../../archive/phase2/PHASE2-INDEX.md).
  Per-unit planning record (`U8-plan.md`, `U7-U8-handoff.md`) folded into this spec; see git history.
- Commits `b99ad22`, `806c9ea`, `83221e8`.
- Code: `phalcom-core/src/vm.rs`, `method.rs`, `primitive/object.rs`, `universe.rs`, `error.rs`.
