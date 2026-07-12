# 15. `Object` default `toString` is `"<ClassName>"`

- Status: Accepted
- Date: 2026-07-11
- Related: `docs/spec/object-model.md` §2, §8; `docs/spec/classes.md`; forge finding F4; [ADR-0009](0009-handle-arena-heap.md)

## Context

`toString` is universal `Object` protocol — a display representation, overridable
everywhere ([Object Model §8](../spec/object-model.md)). The spec pins that `name`
is `Behavior`-side (a class's own name) but leaves the *instance* display string
open. The forge audit found the current implementation actively wrong here:

- **F4** — `object_name` returns `receiver.class(vm).name()`, so a **class**'s
  `name`/`toString` returns its **metaclass** name: `Number.toString → "Number.class"`
  instead of `"Number"`. The same routine is installed as both `name` and `toString`
  on `Object`, so `anInstance.name` wrongly yields the class name too.

Fixing "the class name is wrong" requires deciding what `toString` means on a plain
instance, since the spec left the instance display string open (a "display
representation" with no fixed format).

## Decision

- A plain instance's default `toString` renders as **`"<ClassName>"`** — e.g. a
  `Point` instance renders `"<Point>"`.
- A **class**'s own `name` and `toString` are its **own** name: `Number.name` and
  `Number.toString` both yield `"Number"` (and `(Number class).name` yields
  `"Number class"`). This fixes F4 — the class no longer reports its metaclass name.
- There is **no `printString` selector**. Display goes through `toString` only;
  `printString` is not introduced.

`name` is `Behavior`-side (own name), not universal `Object` protocol; `toString`
stays universal on `Object`, and a class's `toString` is defined to be its own name.

## Consequences

- `Number.toString` → `"Number"`, not `"Number.class"` — F4 resolved.
- A plain instance has a deterministic, golden-stable default display (`"<Point>"`)
  that names its class without any allocation-heavy or vowel-sensitive formatting.
- Users override `toString` per class for a richer representation; the default only
  guarantees the class is identifiable.
- No `printString`/`toString` split to teach or keep consistent — one display
  selector.

## Alternatives considered

- **`"a ClassName"` / `"an ClassName"`** (Smalltalk `printString` style).
  Familiar, but needs article/vowel logic and is less predictable for golden tests.
  Rejected.
- **`"ClassName instance"`.** Deterministic and golden-stable, but more verbose;
  the project owner chose the bracketed `"<ClassName>"` form. Rejected in favor of
  the chosen form.
- **Introduce a separate `printString` selector** (Smalltalk's display/print split).
  Rejected explicitly: it is not a spec selector, and one `toString` is enough for
  the display contract.
