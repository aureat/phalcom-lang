# 20. Kernel `List` is a native-array-backed Phalcom protocol on the critical path

- Status: Proposed
- Date: 2026-07-11
- Resolves: DEC-A (kernel `List` — unscheduled hard dependency)
- Related: [ADR-0007](0007-option-as-abstract-with-some-none.md); [ADR-0009](0009-handle-arena-heap.md); [ADR-0012](0012-selector-signature-encoding-and-dispatch.md); [ADR-0019](0019-freeze-vm-blessed-primitive-floor.md); `docs/spec/messages.md` §4 (variadics); `docs/spec/experimental/bootstrapping-and-self-hosting.md` (D2); `docs/forge/PHASE2-INDEX.md` (DEC-A)

## Context

Several already-committed features have a **kernel `List` as a hard dependency**,
yet no unit builds one and it is unscheduled (DEC-A):

- **`doesNotUnderstand(_)`** reifies a failed send as a `Message` whose arguments
  are a `List` (method-lookup §2, ADR-0012). Without `List`, dNU cannot hand the
  args to a proxy.
- **Rest parameters `*xs`** collect trailing positional arguments into a single
  `List` (messages §4). Without `List`, variadics have nowhere to put them.
- **The iteration protocol** (`do`/`collect`/`inject`) is defined over a backing
  sequence.
- **A self-hosted front-end** (experimental bootstrapping note, Rung C) needs
  `List` for its token stream, AST children, and constant pool.

DEC-A also records a scheduling collision: `List` and the U-STD standard-library
unit want to edit `core.ph` in the same wave, and `core.ph` must never be
co-scheduled between two editors.

The design tension is **how** `List` stores its elements, and it lands squarely on
the **primitive/library boundary ⊗ bootstrap-order** and **absence bootstrap
cycle** hazards. A `List` implemented as a chain of `Some`/`None` cells (a
cons-list in Phalcom) is expressible entirely above the floor — but it couples
every collection to `Option`, puts a `Some` allocation on the element hot path,
and makes the collection library depend on absence semantics. A `List` backed by a
native growable array sits partly at the floor but keeps `Option` off the hot path
and matches how Smalltalk implements `OrderedCollection`.

## Decision

`List` is a **native-array-backed Phalcom protocol**, and it is placed **on the
critical path immediately above the VM-blessed floor** (ADR-0019), before dNU,
variadics, iteration, and any self-hosted compiler.

**Storage (at the floor, native):** a growable array primitive — a Rust `Vec`
of `Value` behind the handle/arena `Heap` (ADR-0009) — exposed through a minimal
native method set: allocate, length, indexed get, indexed set, push, and a
raw-capacity grow. No `Rc`/`RefCell`; the array is a heap handle like any other
object, so it is GC-ready and has no borrow-panic surface.

**Protocol (above the floor, `.ph`):** `do`, `collect`, `inject`, `at`, `add`,
`isEmpty`, `size`, and the rest of the sequence surface are authored in `core.ph`
as ordinary dispatched methods over the native storage primitives. This is the
"hybrid: native primitives, self-defined control" row of the kernel matrix — the
`OrderedCollection` shape.

**Load order (extends ADR-0019 / D3):**

```
… Bool, Option, Number, Symbol, String   (VM-blessed floor)
List            ← native array primitive built here; .ph protocol loaded next
Map, Set        ← need List + Object identity/hash
Iteration       ← needs List
Message.args    ← needs List (dNU)
rest-params *xs ← needs List
```

**Scheduling (resolves the DEC-A collision):** `List` lands as its **own unit**,
ahead of U-STD. U-STD then grows the rest of the library additively on top. The two
never edit `core.ph` concurrently — `List` first, U-STD after.

## Consequences

- dNU, rest-parameters, and the iteration protocol become **buildable** — all three
  were blocked on a dependency nothing provided. DEC-A moves from "unscheduled hard
  dependency" to "landed, on the critical path."
- **`Option` stays off the collection hot path.** Because storage is a native array,
  adding or reading an element allocates nothing and touches no `Some`/`None`. The
  absence bootstrap cycle (ADR-0007, Invariant 4) is not re-entered by the collection
  library. `at(_)` may still *return* an `Option` for a missing index — that is an API
  choice at the protocol layer, not a storage coupling.
- The native surface added is **small and auditable** — six array primitives — and
  fits under the ADR-0019 floor without widening it meaningfully: an array cell is
  the one container primitive the language cannot bootstrap from itself.
- `List` becomes a **prerequisite of Rung C**: a self-hosted lexer/parser/compiler
  cannot be written until `List` (and `Map`) exist, so scheduling `List` early is also
  the first concrete step toward self-hosting, should that be pursued.
- Placing `List` immediately above the floor keeps `verify_invariants()` meaningful:
  the array primitive is built during `install_primitives`, and the `.ph` protocol is
  the first thing loaded after the floor, so a missing edge fails at a well-defined
  point rather than deep in user code.

## Alternatives considered

- **`Some`/`None` cons-list, entirely in `.ph`.** Needs no new native primitive and is
  maximally dogfooded. Rejected: it puts a `Some` allocation on every element, couples
  all collections to `Option`, drags absence semantics onto the collection hot path,
  and is slow — the wrong default for the one data structure dNU, variadics, and the
  compiler all lean on.
- **Fully native `List` (storage *and* protocol in Rust).** Fastest, but freezes the
  sequence API against introspection/redefinition and shrinks the dogfooded surface —
  a direct violation of the ADR-0019 floor, which puts the *protocol* above the line.
  Rejected: keep storage native, protocol in `.ph`.
- **Defer `List`, ship dNU/variadics without it.** Rejected as impossible: both
  features are *defined* in terms of `List` (Message args; rest collection). There is
  no partial version that omits the dependency.
- **Fold `List` into U-STD (no separate unit).** Rejected: it recreates the DEC-A
  collision (two editors on `core.ph`) and buries a critical-path dependency inside a
  broad library unit, making it easy to slip. `List` is load-bearing enough to own its
  own unit.
