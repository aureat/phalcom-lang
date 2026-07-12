# U-CORE-4 — Value classes: per-type `toString` (implementation spec)

> **Status:** Normative implementation order. A dispatch-ready work order a
> `phalcom-implementer` executes end to end. Adds the per-type `toString`
> **message** overrides on the value types (`Number`, `String`, `Symbol`,
> `Bool`, `Option`/`Some`/`None`) and makes the separate native
> `Value::to_string` print-path **agree** with them (catalog-delta §4.4,
> decisions.md §4.4, R-INV-4.1–4.4). Resolves **DEFERRED F4** (the
> `object_name` / instance-`toString` home, ADR-0015) and **unblocks DEFERRED
> #30** (the string-interpolation desugar's `String.new(_)` stand-in, ADR-0022):
> once this unit lands a real content `toString`, `desugar_string_interp` in
> `phalcom-ast/src/parser.rs` *can* switch its target to `expr.toString` — but
> that switch is a separate follow-up in the `phalcom-ast` crate, outside this
> unit's write-set (mirrors how this unit unblocks-but-does-not-perform the
> `List#toString` move for DEFERRED #19 — see §0.2).
>
> **Baseline (re-grounded 2026-07-12):** HEAD `aa9cdca` (clean at the time of
> this grounding pass). Recommended dispatch order is **U-CORE-3 → U-CORE-2 →
> U-CORE-4**; this unit's own delta (§2, §6.1) is expressed as *current floor
> + 1*, not a hardcoded absolute, so it stays correct regardless of exactly
> when it lands relative to its neighbors:
> - **Floor today (`aa9cdca`, before U-CORE-3 lands): 80** installed
>   `(class, selector)` bindings (`floor-census.md` §1.1).
> - **Floor after U-CORE-3 lands: 85** (U-CORE-3 adds exactly 5 new bindings —
>   its own `as-built.md` header). Per the recommended order, U-CORE-4 is
>   meant to dispatch on top of that 85, adding its own **+1**
>   (`Number#toString`) → **86**.
> - Since this unit's original baseline (`3c74a36`), **U-CORE-1** (kernel
>   `hash`/`isA`/`Behavior` reflection), **U11** (Bool tower: abstract `Bool`
>   with `True`/`False`), **U-LIST**, **U8** (dNU/`Message`), and **U-STD**
>   (`Option`/`List` combinators) have all *also* landed. None of them touch
>   this unit's *design* — the divergence this unit exists to fix
>   (`Object#toString` still aliasing `object_name`, confirmed live at
>   `aa9cdca`, §1.1) is unchanged — but they shifted nearly every `file:line`
>   anchor below and grew `core.ph`'s `Option` reopen substantially.
> - **Concurrency warning:** another session is concurrently implementing
>   U-CORE-3 in-tree right now, touching `heap.rs`, `value.rs`, `vm.rs`,
>   `primitive/{block,method,object,mod}.rs`, `universe.rs`, tests, and
>   `floor-census.md`. Anchors in those specific files were re-confirmed
>   against the working tree during this pass (two independent reads, several
>   minutes apart, agreed) but are marked **"may shift post-U-CORE-3;
>   re-confirm at dispatch"** below rather than trusted as final — this tree
>   moved even *during* this grounding pass. Anchors in files U-CORE-3 does
>   **not** touch (`primitive/{number,string,boolean,symbol,nil,list,system}.rs`,
>   `interner.rs`, `core.ph`) were confirmed stable and are cited without that
>   caveat.
>
> Read against
> [`floor-census.md`](../../../spec/v0.2/core/floor-census.md), [`catalog-delta.md`](../../../spec/v0.2/core/catalog-delta.md),
> [`decisions.md`](../../../spec/v0.2/core/decisions.md) §4.4, [`invariant-requirements.md`](../../../spec/v0.2/core/invariant-requirements.md)
> §4, [`pending-retirement.md`](../../../spec/v0.2/core/pending-retirement.md) §4,
> [`forward-compat.md`](../../../spec/v0.2/core/forward-compat.md) §3–4.

---

## 0. Prerequisites & scope gate

### 0.1 Must already be landed (all satisfied at baseline)

| Prereq | Why U-CORE-4 needs it | Where |
|---|---|---|
| `Option`/`Some`/`None` substrate + `match(some, none)` eliminator | `Option#toString` is derived over `match`; `None`/`Some` are the rendering targets | `primitive/nil.rs`; `universe.rs` L359–380 (Absence-substrate wiring; **may shift post-U-CORE-3**, re-confirm at dispatch) |
| U-CORE-2 `Some`-lift + `Option` combinators (`0da64d6`) | `Option` reopen in `core.ph` already exists to hang `toString` on; `ifTrue { }` now yields `Some(None)` (a rendering case) | `core.ph` L70–124 (the `Option` reopen — now much larger than at this doc's original baseline: U-STD has since added `map`/`flatMap`/`filter`/`ifSome`/`unwrapOr` alongside the original `ifNone`/`orElse`/`isSome`/`isNone`); `primitive/boolean.rs` |
| Native `List` + `list_to_string` (U-LIST, ADR-0020) | List is in the R-INV-4.1 consistency set; its message `toString` already exists — U-CORE-4 only aligns the print path to it | `primitive/list.rs` L135 (confirmed unchanged) |
| The separate native `Value::to_string(vm)` print-path | The invariant partner U-CORE-4 must keep in agreement (decisions.md §4.4) | `value.rs` L149–160 (**may shift post-U-CORE-3**; re-confirm at dispatch) |
| Sacred-selector inliner (ADR-0018), `ifTrue(_, ifFalse)` fallback | `Bool#toString` is written in `.ph` over `ifTrue(_, ifFalse)` | `primitive/boolean.rs` L164 (`bool_if_true_if_false`; confirmed unchanged — `boolean.rs` is not in U-CORE-3's write-set) |

### 0.2 Explicitly OUT of scope (do not build here)

- **Richer value protocol** — `String` length/indexing/`toNumber`/comparison,
  `Number#toNumber`/richer math, `Symbol` interning-identity protocol. All
  **U-STD** (catalog-delta §2.2). U-CORE-4 is `toString`-only.
- **`Object#hash` / `isA(_)`** — this was **U-CORE-1**, which has *since
  landed* (`03764e3`, confirmed in `git log`). `toString` and `hash` shared the
  "reads representation → native" reasoning but were kept as separate units;
  U-CORE-4 does not revisit `hash`/`isA` (both already exist, both unaffected
  by this unit's design).
- **Moving `List#toString` to `.ph`** — DEFERRED #19: once value types have real
  `toString`, `list_to_string` can move to `.ph` over `each` + `String` concat.
  U-CORE-4 **unblocks** that but does **not** do the move (it stays native this
  unit); the move is **U-STD**.
- **Switching the string-interpolation desugar target** — DEFERRED #30:
  `phalcom-ast/src/parser.rs::desugar_string_interp` currently wraps each
  interpolated expression as `String.new(expr)` rather than `expr.toString`,
  precisely because no real content `toString` existed. U-CORE-4 **unblocks**
  the switch (the root cause — no value-type `toString` — is fixed by this
  unit) but the switch itself lives in `phalcom-ast`, a different crate outside
  this unit's write-set; treat it exactly like the #19 `List` move above — a
  follow-up, not part of this unit.
- **`#…` selector-literal syntax and the human-`_`-form symbol decode** — **U-LEX**.
  U-CORE-4 only makes the two symbol-rendering paths *agree* (§2.4, BD-CORE4-2).
- **`Some(_)` construction sugar** — **U-LEX**. U-CORE-4 tests `Some` via the
  already-supported `Some.new(_)` send (§4.1).
- **`Behavior#name` / method-dictionary reflection** — **U-CORE-1** (already
  landed). U-CORE-4 fixes the class-receiver case *inside `Object#toString`
  only* (ADR-0015); it does not touch `Object#name` / `class_superclass`
  reflection.

---

## 1. What exists vs. what's missing (grounded)

### 1.1 The confirmed divergence (catalog-delta §4.4)

**Re-confirmed live at `aa9cdca`.** `Object#toString` is still bound to the
native `object_name` (`universe.rs` **L250** as of this pass — was L230 at this
doc's original baseline; drifted because U-CORE-1/U8/U-LIST additions earlier
in `install_primitives` pushed everything down. **`universe.rs` is being
concurrently edited by U-CORE-3 — re-confirm this line at dispatch.**), which
returns the receiver's **class name** (`primitive/object.rs` **L23–27**,
confirmed unchanged):

```rust
// universe.rs L250 (current as of this pass; re-confirm before editing)
primitive!(vm, object_cls, "toString", SignatureKind::Getter, object_name);
```

So `toString` (the **message**) inherited by every value type yields a class
name, not a value:

| Expression | `toString` message today | Wanted (R-INV-4.x) |
|---|---|---|
| `42.toString` | `"Number"` (object_name) | `"42"` |
| `"hi".toString` | `"String"` | `"hi"` |
| `true.toString` | `"Bool"` | `"true"` |
| `#foo.toString` | `"foo"` (native `symbol_tostring`, **already overrides**) | `"#foo"` (align, §2.4) |
| `None.toString` | `"None"` (object_name returns class name — *accidentally right*) | `"None"` (make robust, §2.3) |
| `Some.new(42).toString` | `"Some"` | `"Some(42)"` |
| `aFoo.toString` (user) | `"Foo"` (bare) | `"<Foo>"` (ADR-0015) |

The **print path is separate**: `System.print(x)` → `system_class_print`
(`primitive/system.rs` **L13–19**, confirmed unchanged) → `arg.to_string(vm)` =
the native `Value::to_string` (`value.rs` **L149–160**; **may shift
post-U-CORE-3**). That path renders `Number`/`String`/`Bool` correctly
*already*, but renders **`None`/`Some` via `to_debug`** (`value.rs` **L157**
`_ => self.to_debug(vm)`, inside the `Value::Obj` arm at L155–158) as `<None
instance>` / `<Some instance>`, a **List** as `<list>`, and a **Symbol** as
`Symbol("…")` (`interner.rs` **L20–23**, confirmed unchanged).

### 1.2 The two paths, and where each disagrees today

| Value type | message `x.toString` today | print `Value::to_string(x)` today | Agree? |
|---|---|---|:--:|
| `Number` | `"Number"` (Object default) | `"42"` (`value.rs` L153) | ✗ |
| `String` | `"String"` | raw content (`value.rs` L156) | ✗ |
| `Bool` | `"Bool"` | `"true"`/`"false"` (`value.rs` L152) | ✗ |
| `Symbol` | `"foo"` (`symbol_tostring`) | `Symbol("foo")` (`Symbol::to_string`) | ✗ |
| `None` | `"None"` (Object default = class name) | `<None instance>` (`to_debug`) | ✗ |
| `Some(v)` | `"Some"` (Object default) | `<Some instance>` (`to_debug`) | ✗ |
| `List` | `"[1, 2, 3]"` (`list_to_string`) | `<list>` (`to_debug`) | ✗ |
| user `Foo` | `"Foo"` (bare) | `<Foo instance>` (`to_debug`) | ✗ (allowed) |

**Every value-type row disagrees.** R-INV-4.1 requires agreement for the value
types (`Number, String, Symbol, Bool, None, Some(_), List`); the user-`Foo` row
is **outside** the invariant's domain and may keep the message/print split
(`"<Foo>"` vs `<Foo instance>`) — §5.

### 1.3 Existing anchors the implementer will touch

> **Anchors marked "⚠ moving" sit in `value.rs`, `universe.rs`, or
> `primitive/object.rs` — files a concurrent U-CORE-3 session is editing.
> Re-grep the symbol name to confirm the exact line before editing; do not
> trust the number alone.**

| Symbol | File:line |
|---|---|
| `install_primitives` Object block (rebind `toString`) | `universe.rs` L245–266 ⚠ moving |
| `install_primitives` Number block (add `toString`) | `universe.rs` L293–310 ⚠ moving — Number now also carries a `hash` binding (L308, ADR-0023/U-CORE-1) not present at this doc's original baseline; insert `toString` after it, before the static `new`s at L309–310 |
| `install_primitives` Symbol block | `universe.rs` L349–357 ⚠ moving |
| `object_name` (leave as `name`) | `primitive/object.rs` L23–27 ⚠ moving (file touched by U-CORE-3, though these exact lines were unchanged across two independent reads this pass) |
| `Value::to_string` (extend Obj arm) | `value.rs` L149–160 (Obj arm itself at L155–158) ⚠ moving |
| `Value::to_debug` (optional align) | `value.rs` L166–184 ⚠ moving |
| `Symbol::to_string` renderer | `interner.rs` L19–28 (confirmed stable — not touched by U-CORE-3) |
| `symbol_tostring` message | `primitive/symbol.rs` L13–17 (confirmed stable) |
| `core.ph` `Option` reopen (add `toString`) | `core.ph` L70–124 (stable; grew since baseline — see §0.1) |
| `core.ph` `String` / `Bool` reopens | `core.ph` L25 (`class String {}`), L27 (`class Bool {}`) — both still bare one-line bodies, stable |
| `none_class` / `some_class` ids, `none_singleton` | `universe.rs` L168–175 (creation), L211–213 (`CoreClasses` literal), L675–686 (struct fields) ⚠ moving |
| `string_add` (both operands must be `String`) | `primitive/string.rs` L34–38 (confirmed stable — `string.rs` not touched by U-CORE-3) |

---

## 2. The native-vs-`.ph` split + exact insertion points

Guiding rule (ADR-0019 §1): **the default answer to "add a primitive" is NO.** A
`toString` override earns a native binding **only** if it must read a
representation the `.ph` floor cannot reach (the same test that made `hash`
native, decisions.md Q1). Under that test:

| Type | `toString` home | Why |
|---|---|---|
| `Number` | **native** (new floor binding) | f64 → decimal is unreachable in `.ph` (no `.ph` number→string path); DEFERRED #19 confirms this exactly |
| `Object` (default) | **native** (re-home existing binding) | must read the class name / discriminate a class receiver (ADR-0015); already a native binding |
| `Symbol` | **native** (`symbol_tostring` already exists) | reads the interned text; reconcile with print path (§2.4) |
| `String` | **`.ph`** (`=> self`) | a string's display *is* itself — no representation read |
| `Bool` | **`.ph`** over `ifTrue(_, ifFalse)` | derivable over an existing sacred selector |
| `Option`/`Some`/`None` | **`.ph`** over `match` | derivable over the `match` eliminator; respects inner `v.toString` |
| `List` | **already native** (`list_to_string`) — only **print path** aligned | message exists; U-CORE-4 makes `Value::to_string` agree |

Net floor change: **exactly one new binding** — `Number#toString`. This is an
**ADR-0019 amendment** (§6.1). Everything else is a `.ph` addition, a re-home of
an existing native binding, or a native-renderer change (not a bound selector).

### 2.1 Native — `Object#toString` re-home (ADR-0015 / DEFERRED F4)

Split `toString` off `object_name`. `object_name` stays as `Object#name` (bare
class name — **unchanged**, that fix belongs to U-CORE-1, which has since
landed without touching this). Add a new `object_to_string` and rebind
`Object#toString` to it.

`primitive/object.rs` — new fn:

```rust
/// Signature: `Object::toString` — the default display string (ADR-0015).
///
/// A **class** receiver renders as its own name (`"Number"`), fixing F4 (the
/// old `object_name` returned the *metaclass* name for a class). A plain
/// instance renders as `"<{ClassName}>"` (`"<Point>"`). User classes override
/// for a richer form; the default only guarantees the class is identifiable.
pub fn object_to_string(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    // Borrow-model care: bind the cloned name to its OWN `let` so the immutable
    // `vm.heap` borrow is released before the `&mut vm` alloc (the `object_name`
    // idiom, object.rs L25-26). Do NOT inline the clone into the alloc call.
    let own_name = match receiver {
        Value::Obj(id) => vm.heap.as_class(*id).map(|c| c.name.clone()),
        _ => None,
    };
    if let Some(name) = own_name {
        return Ok(vm.alloc_string_value(name)); // class → own name (fixes F4)
    }
    let class_id = receiver.class(vm);
    let name = vm.heap.class(class_id).name.clone();
    Ok(vm.alloc_string_value(format!("<{name}>")))
}
```

`universe.rs` **L250** (current as of this pass; **`universe.rs` is being
concurrently edited by U-CORE-3 — re-confirm the exact line before editing**)
— rebind:

```rust
primitive!(vm, object_cls, "toString", SignatureKind::Getter, object_to_string);
```

> `object_name` remains bound to `Object#name` (L247) — do **not** touch it. The
> census note "`toString` **aliases** `name`" (floor-census.md §2.1, confirmed
> still present verbatim as of this pass) no longer holds after this change;
> that census line must be updated (§6.1).

### 2.2 Native — `Number#toString` (the one new floor binding)

`Number` cannot render its own value in `.ph`. Add a native primitive that
delegates to the existing renderer so it is *identical* to the print path by
construction. Put a general helper in `primitive/number.rs` (or a shared
`primitive/mod.rs`), but **bind it only on `Number`**:

```rust
/// Signature: `Number::toString` — the numeric value as a decimal string.
/// Delegates to the shared native renderer so `n.toString` is byte-identical to
/// `System.print(n)` (R-INV-4.1). Reads the f64 representation — unreachable in
/// `.ph`, hence a floor primitive (ADR-0019 amendment; cf. `hash`, decisions.md Q1).
pub fn number_to_string(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let text = receiver.to_string(vm); // immutable borrow ends before the alloc below
    Ok(vm.alloc_string_value(text))
}
```

`universe.rs` — insert in the Number block, current shape as of this pass:

```rust
// L305: primitive!(vm, number_cls, "negated", ...);
// L306-307: Value-digest doc comment (ADR-0023)
// L308: primitive!(vm, number_cls, "hash", SignatureKind::Getter, number_hash);  <- added by U-CORE-1, not present at this doc's original baseline
// L309-310: the two static `new`s
```

insert the new binding after `hash` (L308), before the static `new`s (L309–310):

```rust
primitive!(vm, number_cls, "toString", SignatureKind::Getter, number_to_string);
```

> **Bind on `Number` (the abstract numeric root), never on a concrete f64
> path.** Delegating through `Value::to_string` (which renders from the numeric
> value, not hard-coded float syntax) keeps a future `Integer`/`Float` split
> additive: each subclass inherits or overrides this binding without breaking
> dispatch identity (forward-compat §4, §5).
>
> **`universe.rs` is a moving target this session** (concurrent U-CORE-3 work)
> — re-grep for `number_cls` and `hash` before editing to confirm these exact
> line numbers still hold.

### 2.3 `.ph` — `Option#toString` (covers `Some` **and** `None`)

One method on the abstract `Option` class, derived over `match`. `Some` and
`None` inherit it — **no `Some`/`None` primitive, and no `class None {}` reopen**
(the `DefineGlobal`-clobber hazard, `core.ph` L60–68, stays intact).

`core.ph` — add inside the existing `Option { … }` reopen, now **L70–124**
(grew since baseline — U-STD's `map`/`flatMap`/`filter`/`ifSome`/`unwrapOr`
were added after the original `ifNone`/`orElse`/`isSome`/`isNone` quartet;
`core.ph` is **not** touched by the concurrent U-CORE-3 session, so this range
is stable):

```phalcom
class Option {
  // ... existing ifNone / orElse / isSome / isNone / map / flatMap / filter /
  // ifSome / unwrapOr (U-STD; all landed since this unit's original baseline,
  // out of scope here) ...

  // Display (values-and-absence §3, R-INV-4.3). Derived over `match`, so a
  // user-overridden `match` is respected (R-INV-2.4) and the inner value is
  // rendered via its OWN `toString` message (so a value-typed payload agrees
  // with the print path, R-INV-4.1).
  toString => self.match(
    some: { v => "Some(" + v.toString + ")" },
    none: { "None" }
  )
}
```

Renderings this produces: `None` → `"None"`; `Some.new(42)` → `"Some(42)"`;
`Some.new("hi")` → `"Some(hi)"`; `Some.new(None)` → `"Some(None)"`; nested
`Some.new(Some.new(1))` → `"Some(Some(1))"`.

> **`String#+` is strict** (`string_add`, `primitive/string.rs` L34–38,
> confirmed stable: both operands must be `String`). `v.toString` must
> therefore return a `String` — it does for every value type. A pathological
> user `toString` returning a non-string would raise from `+`; that is out of
> the value-type domain R-INV-4.4 covers (§5).

### 2.4 Native — reconcile the two `Symbol` rendering paths (BD-CORE4-2)

The message (`symbol_tostring` → bare `"foo"`) and print (`Value::to_string` →
`Symbol::to_string` → `Symbol("foo")`) disagree. R-INV-4.1 lists `Symbol`, so they
**must be unified onto one renderer**. Route both through a single helper. The
**form** is an open sub-decision (§6.2); the recommended form is the spec's `#`
sigil (values-and-absence §1, the `messages_selector_symbol_literal` fixture):

- `interner.rs` `Symbol::to_string` (L20–23, confirmed stable) → return
  `format!("#{}", vm.resolve_symbol(*self))`.
- `symbol_tostring` (`primitive/symbol.rs` L13–17, confirmed stable) → call
  the same helper (so message == the Symbol arm of `Value::to_string`).

For a label-free symbol (`Symbol.new("foo")`), interned form == human form, so
this is byte-stable **without** U-LEX (`#foo`). The human-`_`-form decode that
turns interned `move(to:duration:)` into `#move(_,to,duration)` (retiring
`messages_selector_symbol_literal`) is **U-LEX's** job (it owns both the `#…`
literal and the canonical print form; cross-ref `decode_selector`, `method.rs`).

> This churns **one** green fixture: `dispatch/dispatch_message_shape.expected`
> line 1 `Symbol("move(to:duration:)")` → `#move(to:duration:)` (§4.3). If the
> team prefers zero surface churn now, BD-CORE4-2 Option B keeps the message==print
> requirement while deferring the sigil — see §6.2.

### 2.5 Native — align `Value::to_string` (the print path) with the messages

Extend the `Value::Obj` arm of `Value::to_string` (`value.rs` **L155–158** as
of this pass, formerly cited L142–146 — **`value.rs` is being concurrently
edited by U-CORE-3; re-grep `fn to_string` to confirm before editing**) so the
print path renders `None`/`Some`/`List` the same way the messages do. Use
**guarded `if` checks with a `_ => to_debug` fallback**, not an exhaustive match —
so a future `Fiber`/`Future` `Value` arm needs no edit here (forward-compat §1).

```rust
Value::Obj(id) => match vm.heap.get(*id) {
    Object::Str(string) => string.value(),
    Object::List(list) => {
        let parts: Vec<String> = list.elements().iter().map(|v| v.to_string(vm)).collect();
        format!("[{}]", parts.join(", "))          // == list_to_string (list.rs L135, confirmed stable)
    }
    Object::Instance(inst) if inst.class == vm.universe.classes.none_class => "None".to_string(),
    Object::Instance(inst) if inst.class == vm.universe.classes.some_class => {
        format!("Some({})", inst.slots[0].to_string(vm))  // `slots` confirmed the InstanceObject field name (instance.rs L17)
    }
    _ => self.to_debug(vm),
},
```

- The `List` case duplicates `list_to_string`'s exact formatting; both recurse
  through `Value::to_string`, so they are identical by construction (R-INV-4.1
  for `List`). Optionally have `list_to_string` call a shared renderer to avoid
  drift — implementer's choice.
- `Some` renders its inner via `Value::to_string` (native recursion), which for a
  **value-typed** payload equals `v.toString` (message). For a **user-typed**
  payload the two differ (native default vs. user override) — permitted (§5).
- Leaves the generic-instance case (`_ => to_debug`) untouched: `System.print(aFoo)`
  keeps rendering `<Foo instance>` (no green print fixture regresses beyond §4.3).

> **Optional:** mirror the same three cases in `Value::to_debug` (`value.rs`
> L166–184 as of this pass, formerly L153–171; ⚠ moving) so diagnostics
> (`object_does_not_understand` builds `receiver.to_string`, `object.rs` —
> already `to_string`, so covered) stay consistent. Not required by any
> invariant.

---

## 3. Concrete change list (implementer checklist)

| # | File | Change | Kind |
|---|---|---|---|
| 1 | `primitive/object.rs` | add `object_to_string` (§2.1) | native fn (new distinct fn) |
| 2 | `universe.rs` (⚠ re-confirm line — L250 as of this pass) | rebind `Object#toString` → `object_to_string` | re-home (not a new binding) |
| 3 | `primitive/number.rs` | add `number_to_string` (§2.2) | native fn |
| 4 | `universe.rs` (Number block, ⚠ re-confirm — L293–310 as of this pass, insert after `hash` L308) | bind `Number#toString` → `number_to_string` | **+1 floor binding** (ADR-0019 amendment) |
| 5 | `value.rs` (⚠ re-confirm — L155–158 as of this pass) | extend `Value::to_string` Obj arm: `None`/`Some`/`List` (§2.5) | native renderer (no binding) |
| 6 | `interner.rs` L20–23 (stable) + `primitive/symbol.rs` L13 (stable) | unify symbol rendering onto one helper, `#`-form (§2.4, BD-CORE4-2) | native renderer + existing binding semantics |
| 7 | `core.ph` `Option` reopen (L70–124, stable) | add `toString => self.match(...)` (§2.3) | `.ph` |
| 8 | `core.ph` `String` reopen (L25, stable) | add `toString => self` | `.ph` |
| 9 | `core.ph` `Bool` reopen (L27, stable) | add `toString { return self.ifTrue({ "true" }, ifFalse: { "false" }) }` | `.ph` |
| 10 | 9 green `.expected` files (§4.3) | update `<None instance>`/`<Some instance>` → new render | golden re-pin |
| 11 | `dispatch_message_shape.expected` | line 1 → `#move(to:duration:)` (BD-CORE4-2 A only) | golden re-pin |
| 12 | 3 pending fixtures (§4.2) | `git mv` into active lane (confirmed on disk at `tests/lang/{absence,bindings}/pending/*.ph`) | retirement |
| 13 | new unit-local fixtures (§4.1) | add (new `tests/lang/values/` dir — confirmed does not yet exist) | golden |
| 14 | `tests/invariants.rs` | add R-INV-4.1–4.4 (§4.4) | corpus |
| 15 | `floor-census.md` | current count (80, or 85 post-U-CORE-3) → +1; §2.1/§2.4 edits (§6.1) — *census doc, done by implementer, and itself in U-CORE-3's concurrent write-set — re-diff before editing* | doc |

> **`.ph` `Bool#toString` syntax is proven.** The labeled call
> `(3 > 2).ifTrue({ "yes" }, ifFalse: { "no" })` is a live green fixture
> (`control-flow/control_flow_send_equivalence.ph` L9), so
> `self.ifTrue({ "true" }, ifFalse: { "false" })` parses and returns the raw
> branch value (R-INV-2.3: the paired form is not `Some`-lifted). Adding a
> **non-sacred** `toString` to `Bool` does **not** flip `bool_sacred_pristine`
> (floor-census §5 — `bool_sacred_pristine` specifically tracks the **six**
> `Bool` selectors `and`/`or`/`not`/`ifTrue`/`ifFalse`/`ifTrue(_,ifFalse)`; the
> census's *total* sacred-selector count is now **7**, since it separately
> counts `Block`'s `whileTrue(_)` under its own `block_sacred_pristine` flag —
> the two flags are independent, so this parenthetical's "six" was and remains
> correct for `Bool` specifically), so there is **no inliner deopt**. Keep the
> six sacred selector shapes untouched.

---

## 4. Test strategy

### 4.1 Acceptance bar — new unit-local fixtures (already-supported syntax)

These are the pass gate. All use syntax that works **today** (no U-LEX). Add
under `tests/lang/values/` (new label, confirmed not yet present on disk)
unless noted:

| Fixture | Body | `.expected` | Asserts |
|---|---|---|---|
| `value_number_tostring` | `System.print(42.toString)` | `42` | Number message |
| `value_number_tostring_frac` | `System.print((3 / 2).toString)` | `1.5` | non-integer f64 render |
| `value_string_tostring` | `System.print("hi".toString)` | `hi` | String `.ph` `=> self` |
| `value_bool_tostring` | `System.print(true.toString)` then `false.toString` | `true`⏎`false` | Bool `.ph` |
| `value_none_tostring` | `System.print(None.toString)` | `None` | None via `Option#toString` (message path, distinct from the print-path fixture in §4.2) |
| `value_some_tostring` | `System.print(Some.new(42).toString)` | `Some(42)` | Some via `Some.new` (no U-LEX) |
| `value_some_print` | `System.print(Some.new(42))` | `Some(42)` | print path agrees with message |
| `value_object_default_tostring` | `class Foo {}` … `System.print(Foo.new().toString)` | `<Foo>` | R-INV-4.2 (ADR-0015) |
| `value_class_own_name` | `System.print(Number.toString)` | `Number` | ADR-0015 class-receiver / F4 fix |
| `value_symbol_tostring` | `System.print(Symbol.new("foo").toString)` | `#foo` (BD-CORE4-2 A) | label-free symbol, U-LEX-independent |

### 4.2 `_pending` tests this unit flips

Quoting pending-retirement §4 (U-CORE-4 row, re-confirmed unchanged this
pass). U-CORE-4 has the **most direct flips** of any U-CORE unit.

**Direct flips (go green on their own, plain syntax — `git mv` into the active
lane). Confirmed to exist on disk this pass at
`tests/lang/<category>/pending/<name>.{ph,expected}` (note: the `pending`
segment is a subdirectory of the category, not a sibling of it):**

| Fixture (`<category>/pending/<name>`) | Goes green because | Move to |
|---|---|---|
| `absence/pending/absence_option_none` | `System.print(None)` → `None` (§2.5) | `absence/` |
| `absence/pending/absence_var_defaults_to_none` | `var x; print(x)` surfaces `None` → `None` | `absence/` |
| `bindings/pending/binding_var_uninitialized` | same (`None` print) | `bindings/` |

**Unblocks but gated (capability lands here; fixture waits on U-LEX):**

| Fixture | Waits on | Half that lands here |
|---|---|---|
| `absence/pending/absence_option_some` | U-LEX `Some(_)` sugar | `Some#toString` → `Some(42)` (test it via `Some.new` in §4.1) |
| `messages/pending/messages_selector_symbol_literal` | U-LEX `#…` literal + human-form decode | Symbol rendering unification (§2.4) |

> The three direct flips are near-duplicates of pre-existing green fixtures whose
> expected output U-CORE-4 also updates (§4.3): `binding_var_uninitialized`
> mirrors `binding_var_default_none`; `absence_var_defaults_to_none` mirrors the
> same `var x` case. Retire them per pending-retirement §2.1 (housekeeping) — the
> updated pre-existing fixtures are the real regression guard.

### 4.3 Golden re-pins — the regression surface (load-bearing)

Changing the render of `None`/`Some`/`Symbol` re-pins **currently-green** fixtures
that deliberately captured the substrate output (their comments say so — e.g.
`absence_none_prints`: *"a prettier `None` printString is U-STD's job, so this
pins today's substrate output, not the final surface"*). All ten fixture files
below confirmed present on disk at `aa9cdca`. **Update these `.expected`
in the same change or the green run goes red:**

| `.expected` file | old | new |
|---|---|---|
| `absence/absence_none_prints` | `<None instance>` | `None` |
| `absence/absence_print_result_is_none` | `1`⏎`<None instance>` | `1`⏎`None` |
| `absence/absence_match_empty_none_branch_is_none` | `<None instance>` | `None` |
| `absence/absence_empty_block_call_is_none` | `<None instance>` | `None` |
| `absence/absence_iftrue_false_branch_is_none` | `<None instance>` | `None` |
| `absence/absence_root_superclass_is_none` | `<None instance>` | `None` |
| `classes/class_field_unassigned_reads_none` | `<None instance>` | `None` |
| `bindings/binding_var_default_none` | `<None instance>` | `None` |
| `absence/absence_iftrue_empty_body_is_some_none` | `<Some instance>` | `Some(None)` |
| `dispatch/dispatch_message_shape` (BD-CORE4-2 A only) | `Symbol("move(to:duration:)")` (line 1) | `#move(to:duration:)` |

Also refresh the now-stale **comments** in those fixtures (they narrate the old
`<None instance>` output). Comments are non-load-bearing but should not lie.

**Verify no *other* green fixture regresses:** grep the corpus for `System.print`
of a bare list, a bare symbol, or a `None`/`Some`-valued expression before
committing; `list_to_string_renders_brackets` (numbers only) and the two
`.toString` users are already accounted for and stay green.

### 4.4 Invariants this unit adds — all **corpus** (`tests/invariants.rs`, "C")

Per invariant-requirements §4 (re-confirmed unchanged this pass — R-INV-4.1–4.4
still exact at `docs/spec/v0.2/core/invariant-requirements.md` §4, "U-CORE-4 —
value classes"), U-CORE-4 rows are all corpus (behavioral) — none go in
`verify_invariants` (boot). `tests/invariants.rs` currently defines R-INV-0.x
(L532+) and R-INV-1.x (L749+, U-CORE-1); R-INV-4.x does not exist yet — this
unit adds it fresh, it is not editing an existing block:

| # | Assertion | Test shape |
|---|---|---|
| **R-INV-4.1** | For each of `Number, String, Symbol, Bool, None, Some(_), List`: `x.toString` (dispatched message) equals `Value::to_string(x)` (the `System.print` path). | Build each value in-VM, send `toString`, compare to `to_string`; assert equal strings. |
| **R-INV-4.2** | A user class `Foo`'s instance `toString` is `"<Foo>"` (ADR-0015), and a `Number#toString` override does **not** change it. | Define `Foo`, assert `foo.toString == "<Foo>"`. |
| **R-INV-4.3** | `None.toString == "None"` and `Some(x).toString == "Some(" + x.toString + ")"`; the three §4.2 fixtures are green. | Assert both equalities; run the promoted fixtures. |
| **R-INV-4.4** | Value `toString` is **total** (never raises) over the value types and never surfaces the `Nil` sentinel — no output contains `"nil"`, `Some` never wraps/prints `nil` (empty-body `ifTrue` → `Some(None)`, not `Some(nil)`). | Sweep the value types; assert `Ok(_)` and no `"nil"` substring. |

> R-INV-4.1's `List` and `Some(_)` rows are the reason §2.5 exists — without the
> `Value::to_string` alignment the message/print pair fails for exactly those two.

---

## 5. Must-not-preclude check (forward-compat)

U-CORE-4 clears sections **§4** (int/float) and **§3** (names → core module)
of `forward-compat.md` (section numbers re-confirmed unchanged this pass).

- **§4 Integer/Float split.** `Number#toString` is bound on `Number` (the row a
  future abstract `Number → Integer/Float` split refines — note ADR-0024,
  "Numeric surface split: Int/Float and division," has since been formalized,
  confirming this axis is real and tracked) and delegates to `Value::to_string`,
  which renders from the numeric *value*, not hard-coded float syntax. A split
  adds per-subclass rendering (`Integer` → `"2"`, `Float` → `"2.0"`) by ordinary
  override/inheritance — no dispatch-identity break, no change to the `Number`
  binding. **PASS.** (Do **not** add a concrete-f64-only `toString` that a
  `Float` subclass could not override.)
- **§3 Modules / imports.** Every new name is added through `install_primitives`
  and the `core.ph` core-module reopen — i.e. "the core module's exports,
  auto-imported" (decisions.md Q4). No primitive resolves a global by raw string.
  A future `import` can re-scope `toString` like any core name. **PASS.**
- **§1 Value openness (touched incidentally by §2.5).** The `Value::to_string`
  extension uses guarded `if class ==` checks plus a `_ => to_debug` fallback, so a
  future `Value::Fiber`/`Future` `Value` arm compiles without editing this renderer.
  **PASS.**
- **User-typed `Some` payload divergence (documented, allowed).** `Some(aFoo)`
  renders `"Some(<Foo>)"` via the message (inner `aFoo.toString`) but
  `"Some(<Foo instance>)"` via native print (inner `to_debug`). R-INV-4.1's domain
  is the **value types**, which agree; richer user-type consistency in the print
  path is **U-STD** (and the ultimate fix is moving `List`/`Some` rendering fully
  to `.ph` once value `toString`s exist — DEFERRED #19). Not a preclusion.
- **String-interpolation desugar (DEFERRED #30).** This unit's `Value::to_string`
  content-render for `Number`/`String`/`Bool` was already correct before this
  unit (only the *message* `toString` was wrong); what this unit adds is the
  **message** parity. `desugar_string_interp` (`phalcom-ast/src/parser.rs`)
  targeting `String.new(expr)` already renders correctly today via
  `Value::to_string` — switching it to `expr.toString` post-U-CORE-4 is a
  pure surface change (both now agree), not a behavior fix. Not a preclusion.

---

## 6. Open sub-decisions + traceability

### 6.1 ADR-0019 amendment (REQUIRED — one new floor binding)

U-CORE-4 adds **one** native binding: `Number#toString`. Per ADR-0019 the floor
is frozen; this is an amendment, not an ordinary commit. **Precedent since this
doc's original draft:** ADR-0019 has already been amended twice —
[ADR-0023](../../../adr/0023-amend-floor-admit-hash-and-kernel-reflection.md)
(hash + kernel reflection, U-CORE-1, landed) and
[ADR-0028](../../../adr/0028-amend-floor-admit-method-reflection.md) (Method
reflection, U-CORE-3, in flight as of this pass). The established mechanism is
therefore **a new, separately-numbered ADR**, not an edit to 0019 itself; this
unit's amendment should claim the next free number at dispatch time (0029 is
already taken, by list-literal syntax, as of this pass — check `docs/adr/` for
the current max before numbering).

> *Amends ADR-0019.* Add to the frozen floor: **`Number#toString`** (the numeric
> value as a decimal string). Justification: rendering an `f64` as text reads the
> numeric representation below the `.ph` boundary — no `.ph` number→string path
> exists (DEFERRED #19; same derivability failure as `hash`, decisions.md Q1).
> `String#toString` (`=> self`), `Bool#toString` (over `ifTrue(_, ifFalse)`), and
> `Option#toString` (over `match`) are **derivable and stay in `core.ph`** — not
> added to the floor. `Object#toString` is **re-homed** from `object_name` to a
> new `object_to_string` (ADR-0015 `"<ClassName>"` default + class-own-name; F4
> fix): the `(Object, toString)` binding already exists, so this changes the fn
> behind an existing binding, not the binding set. Constraint (forward-compat §4):
> `Number#toString` must render the mathematical value so a future `Integer`/`Float`
> split renders each correctly.
>
> **Floor count — do not hardcode.** At this unit's *original* baseline the
> floor was 73; that number is stale. As of this grounding pass (`aa9cdca`,
> before U-CORE-3) it is **80**; per the recommended dispatch order
> (U-CORE-3 → U-CORE-2 → U-CORE-4) it will be **85** by the time this unit
> lands, going to **86**. Whatever the count is *at actual dispatch time*,
> this unit's contribution is **exactly +1** — confirm the live count in
> `floor-census.md` §1.1 immediately before editing it, rather than trusting
> any number in this document. Update `floor-census.md` §1.1 (count), §2.1
> (drop "`toString` aliases `name`" — confirmed still present verbatim as of
> this pass), and §2.4 (Number gains `toString`) in the same change. If the
> R-INV-0.1 floor-census audit test (it exists now — landed with U-CORE-1,
> `tests/invariants.rs` L540 area) hardcodes an expected count, bump it by
> this unit's actual delta, not by a hardcoded "73 → 74".

### 6.2 BD-CORE4-2 — Symbol canonical rendering form (needs a ruling)

R-INV-4.1 **forces** message==print for `Symbol` (they disagree today). The
**form** is open because it touches U-LEX's surface territory and churns a green
fixture:

- **Option A (recommended):** unify onto `#{interned-text}` (`#foo`,
  `#move(to:duration:)`). Matches the spec's `#` sigil (values-and-absence §1,
  `messages_selector_symbol_literal`). Cost: re-pin `dispatch_message_shape.expected`
  line 1. Leaves the human-`_`-form decode to U-LEX.
- **Option B (zero-churn):** unify onto the current `Symbol("…")` debug form
  (point `symbol_tostring` at it). No fixture churn; satisfies R-INV-4.1; but
  contradicts the spec's visible `#` surface and defers all sigil work to U-LEX.
- **Option C:** unify onto bare text (`foo`). Least aligned with the spec; still
  re-pins `dispatch_message_shape`.

**Recommendation: A.** It is forward-moving, U-LEX-independent for label-free
symbols, and puts the sigil where the spec fixture already expects it. **Does not
block the critical path** — the three direct None flips (§4.2) and the R-INV-4.1
rows for the non-Symbol value types land regardless of this ruling; Symbol can
follow immediately after under whichever form is chosen.

> There are no other open sub-decisions. `Object#toString` = `"<ClassName>"` /
> class-own-name is **ruled by ADR-0015** (not open); the native/`.ph` split is
> ruled by ADR-0019's derivability test; `None`/`Some` renderings are pinned by
> R-INV-4.3.

### 6.3 Traceability

| Claim / requirement | Source |
|---|---|
| U-CORE-4 owns per-type `toString`; keep print-path separate but agreeing | [`decisions.md`](../../../spec/v0.2/core/decisions.md) §4.4; [`catalog-delta.md`](../../../spec/v0.2/core/catalog-delta.md) §4.4 |
| Resolves DEFERRED F4 (`object_name`/instance-`toString` home) | decisions.md §4.4; [`DEFERRED.md`](../../phase-next/DEFERRED.md) #4; [ADR-0015](../../../adr/0015-object-default-tostring.md) |
| Unblocks DEFERRED #30 (interpolation desugar's `String.new(_)` stand-in) — desugar-target switch is a `phalcom-ast` follow-up, out of this unit's write-set | [`DEFERRED.md`](../../phase-next/DEFERRED.md) #30; ADR-0022 |
| `"<ClassName>"` instance default; class `toString` = own name | [ADR-0015](../../../adr/0015-object-default-tostring.md) |
| `Object#toString` aliases `object_name` today (the divergence) | `universe.rs` L250 (⚠ moving); `primitive/object.rs` L23 |
| Print path = native `Value::to_string`; renders `None`/`Some`/`List` via `to_debug` | `primitive/system.rs` L15; `value.rs` L149–160 (⚠ moving) |
| Symbol paths disagree (`Symbol("…")` vs bare) | `interner.rs` L20–23; `primitive/symbol.rs` L13 |
| `Number#toString` must be native; not `.ph`-derivable | [ADR-0019](../../../adr/0019-freeze-vm-blessed-primitive-floor.md) §1; DEFERRED #19; decisions.md Q1 |
| ADR-0019 amendment precedent (already amended twice since this unit's draft) | [ADR-0023](../../../adr/0023-amend-floor-admit-hash-and-kernel-reflection.md); [ADR-0028](../../../adr/0028-amend-floor-admit-method-reflection.md) |
| `Some`/`None` `toString` derivable over `match` | [`values-and-absence.md`](../../../spec/v0.2/values-and-absence.md) §3.2; `primitive/nil.rs` `option_match` |
| `Bool#toString` `.ph` syntax proven; non-sacred (no deopt) | `control-flow/control_flow_send_equivalence.ph` L9; floor-census §5 |
| Direct flips + gated flips | [`pending-retirement.md`](../../../spec/v0.2/core/pending-retirement.md) §4 |
| Green fixtures pinning `<None instance>`/`<Some instance>` to re-pin | corpus audit (§4.3), fixture headers ("pins today's substrate output") |
| R-INV-4.1–4.4 (all corpus) | [`invariant-requirements.md`](../../../spec/v0.2/core/invariant-requirements.md) §4 |
| int/float-safe `toString`; names → core module | [`forward-compat.md`](../../../spec/v0.2/core/forward-compat.md) §4, §3, §5 |
| `List#toString` → `.ph` is a later move (unblocked, not done here) | [`DEFERRED.md`](../../phase-next/DEFERRED.md) #19; catalog-delta §2.4 (U-STD) |
| Universal `toString` on `Object`, overridable everywhere | [`object-model.md`](../../../spec/v0.2/object-model.md) §4, §8 |
