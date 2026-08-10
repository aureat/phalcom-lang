# Family

[Callables](README.md) · [Dispatch and lowering](dispatch.md) · [Arguments and rest](arguments.md) · [Runtime and activation](runtime.md) · [Reflection](reflection.md) · [Function](function.md) · [Method](method.md)

`Family` is a sealed/final `Function` made by a bound `::` method-family reference. It stores receiver/reference context, but it is not an exact Method and does not pre-bind one implementation in the open form.

## 1. Reference forms

```phalcom
object::move
object::#move(_,to,duration)
```

The first is an **open Family**. It stores a bare base name and derives a full target selector from actual call shape. The second is a **pinned Family**. It stores a full selector that remains authoritative.

The left-hand receiver expression is evaluated once at Family creation and stored in the resulting value. A Family is therefore complete enough to be a Function, but differs from BoundMethod:

```text
BoundMethod  exact Method already selected + receiver
Family       receiver/reference stored + target lookup remains at call time
```

## 2. Open Family call

```phalcom
const move = object::move
move(to: point, duration: 10)
```

Application first becomes `move.call(to: point, duration: 10)`. The Function gateway combines the Family's stored base name with the actual lanes:

```text
stored base name + actual call shape
    → complete target selector
    → ordinary message send to stored receiver
```

The target selected is exactly the target of an ordinary send with that selector. An open Family does not capture a Method at reference creation, so actual shape remains dispatch identity.

## 3. Pinned Family call

```phalcom
const exact = object::#move(_,to,duration)
exact(value, to: point, duration: 10)
```

A pinned Family retains the selector from its reference. Supplied values must satisfy the pinned total slot count; they do not replace its labels with a newly derived arrangement. The VM then performs ordinary dispatch of that pinned selector on the stored receiver.

Pinned means selector identity is fixed. It does not mean a Method object is frozen into the Family. Method lookup, authorization, and ordinary target dispatch still occur at call time.

## 4. Arguments and rest

Phalcom uses only:

```text
*      positional rest/spread
**     labeled rest/spread
***    complete rest/spread
```

```phalcom
family(*values)
family(**labels)
family(***arguments)
```

The spelling `args...` is never rest/spread syntax in Phalcom. `...` is not a spread operator.

A Family preserves complete shape while routing. An open Family encodes it into a target selector. A pinned Family validates it against pinned slots. The eventual target Method then applies ordinary exact or rest-family acceptance. See [Arguments and rest](arguments.md) and [Dispatch and lowering](dispatch.md).

## 5. Creation and reference-time check

The compiler evaluates the receiver, interns either a bare base name or a full pinned selector, and emits `MakeFamily`. The VM creates an immutable payload:

```rust
pub struct FamilyObject {
    pub recv: Value,
    pub selector: Symbol,
    pub open: bool,
}
```

At reference creation the VM can reject an empty family with no matching base name and no applicable custom miss behavior. This early diagnostic does not turn a later Family call into a dNU probe.

## 6. No intentional dNU router

Family calling must not deliberately miss `call(...)`, inspect a reified Message, and reconstruct target dispatch. The gateway already has exact call shape. It routes directly to ordinary target dispatch. A genuine final target miss can still reach `doesNotUnderstand(_)`, preserving proxy semantics without making dNU part of normal callable execution.

## 7. Implementation note

The current VM selects a target selector, replaces the Family receiver in the existing stack window with `family.recv`, then calls shaped ordinary dispatch. Open routing uses `encode_selector` on base name plus actual slots. Pinned routing retains the stored selector after arity validation. The operation returns `CallOutcome`, so it is flat with Function forwarding rather than recursive interpreter execution. See [`vm/send.rs`](../../../phalcom-core/src/vm/send.rs) and [`heap/object.rs`](../../../phalcom-core/src/heap/object.rs).

## 8. Related chapters

- [Function](function.md) — Family's shared call gateway
- [Dispatch and lowering](dispatch.md) — selector identity and target sends
- [Method](method.md) — exact behavior contrasted with late lookup
- [Reflection](reflection.md) — Family versus Method reflection
- [Runtime and activation](runtime.md) — direct routing implementation
