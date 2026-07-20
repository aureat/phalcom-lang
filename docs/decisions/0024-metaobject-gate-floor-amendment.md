# PDR-0024 — The metaobject gate: `Method.fromBlock`, `Method#invokeOn`, `Behavior#defineMethod` (floor amendment; amends ADR-0019)

- Status: Proposed
- Date: 2026-07-20
- Related: [`behavioral.md`](../spec/decorator/behavioral.md) plan §1 (the
  unit this record authorizes — U-METHOD-REIFY),
  [`on.md`](../spec/v0.2/decorators/on.md) (the Install hook protocol and
  D-2, the open question this closes; its `Behavior`-side ruling is settled
  input), [PDR-0018](0018-decorator-carry-forward-and-v03-runtime-mandate.md)
  §4 (build spine — this gate precedes every Install decorator),
  [PDR-0001](0001-classes-are-closed.md) (kernel closure this must respect),
  [ADR-0019](../adr/accepted/0019-freeze-vm-blessed-primitive-floor.md)
  (admission rule; census in
  [`floor-census.md`](../spec/v0.2/core/floor-census.md) — cite the census,
  never a fixed total),
  [ADR-0053](../adr/accepted/0053-runtime-decorator-interception-reuses-override-epoch-guard.md).

## Context

Every Install-tier decorator reduces to the same three missing primitives:
reify a block as an installable method, invoke a reified method against a
receiver, and install into a class's dictionary. `decorators/README.md` has
carried this as open D-2 since ratification. Each passes ADR-0019's
admission test — not for speed but inexpressibility: `.ph` code cannot
construct a `MethodObject`, cannot enter the dispatch path with an explicit
method (bypassing lookup), and cannot write a method dictionary; all three
read or write representation below the `.ph` boundary.

## Decision

### 1. Three floor primitives (+3 rows, `NEW_METAOBJECT` census delta)

| Primitive | Protocol home | Contract |
|---|---|---|
| `Method.fromBlock(selector, block)` | class-side `Method` | wraps a `Block` as an uninstalled `Method` for `selector`; block arity must equal the selector's arity **plus one** (see §2) — mismatch raises at creation, not first call |
| `Method#invokeOn(recv, args)` | `Method` (instance) | runs the method against `recv` with `args` (a `List`); arity mismatch raises `RuntimeError::Type` naming the selector. Works for both native-backed and reified methods — it is the one surface that *takes* a receiver |
| `Behavior#defineMethod(selector, method)` | `Behavior` | installs into the receiver *class's* dictionary (metaclass when sent to `X.class`) — never an instance; kernel classes rejected post-core-load (§3) |

### 2. `fromBlock` blocks take the receiver explicitly — no `self` rebinding

The block's parameter list is `(recv, *args-per-selector)`. Inside the
block, `self` remains **lexical** — whatever the block captured, per
Phalcom's ordinary closure semantics.

This corrects on.md's worked examples, which wrote
`Method.fromBlock { args => … m.invokeOn(self, args) }` and needed `self`
to mean the *future receiver* — dynamic rebinding, a new closure kind, an
exception to capture rules the VM would carry forever. The explicit-receiver
form needs nothing and is strictly more expressive: in `Memoize.wrap(m)`,
`self` is the attribute instance (giving `_cache` access for free — exactly
what the example wants) while `recv` is the receiver:

```phalcom
wrap(m) {
  return Method.fromBlock(m.selector) { recv, args =>
    _cache.at((recv, args)).orElse { Some.new(m.invokeOn(recv, args)) }.unwrap
  }
}
```

Precedent-with-consequence: Python's explicit `self` parameter is this exact
choice, and it is why Python's `types.MethodType`/descriptor machinery never
needed a special closure kind; Ruby's `define_method` rebinds `self` inside
the block instead, and pays for it with `instance_exec`'s permanent
confusion about which `self` a block sees. The examples in on.md are
corrected to this surface when the unit lands.

### 3. One install choke point; kernel classes are closed to it

`defineMethod` routes through the same internal path as `Bytecode::Method`'s
handler — `add_method` → `world_version` bump → `note_method_installed` —
in that order, so the sacred-selector pristine flags and any future IC epoch
see reflective installs exactly as compiled ones. A `defineMethod` on a
kernel class after core load is rejected (the PDR-0001 closure surfaced as a
runtime error, `#kernelSealed` kind per the PDR-0010 vocabulary), enforced
at this choke point — one gate, both entrances. User classes accept installs
at any time; this is the sanctioned reflective-install path PDR-0001's
follow-ups anticipated, and the epoch machinery it bumps is precisely why
ADR-0053's caching story survives it.

### 4. No removal, no renaming

No `removeMethod`, no `undefineMethod`, no selector aliasing. Install-tier
decorators only ever *replace at class-definition time* or *add* via the
choke point. Removal converts every "installed once, monotonic" assumption
(A-5's frozen store, ADR-0053's one-time bit reasoning, warm-site validity)
into invalidation problems. Smalltalk's `removeSelector:` is the precedent:
universally supported, vanishingly used, and a permanent tax on every
optimizing implementation. Adding removal later is a superseding PDR that
prices those consequences.

### 5. Reflection reads ride the existing gate

`Method#selector`/`#attributes` exist; this record adds no read surface
beyond what the three primitives imply. The broader object-model §8 read
API (method enumeration etc.) stays behind its own gate — this is the
*install* gate, deliberately minimal.

## Consequences

- Every Install decorator in [`behavioral.md`](../spec/decorator/behavioral.md)
  becomes pure `core.ph` library code; the hook-dispatch driver
  (instantiate → `wrap` → `defineMethod`) is small compiler work over these
  primitives.
- A wrapped method is an ordinary dictionary entry — inline-cacheable,
  ADR-0053's claim, now purchasable.
- Floor grows by 3 under the ratchet; the census is the number of record.
  The one-way door stands: these can never move back to `.ph`, so the
  minimal §5 scope is the safety margin.

## What this precludes

- `self`-rebinding blocks, as a kind, anywhere in the language — §2 chooses
  the explicit receiver once, and every future block-as-member feature
  inherits the choice.
- Instance-side `defineMethod` (per-object methods) — stays with A-6/v0.3's
  per-instance-behavior decision, untouched here.
- A second install path that skips the choke point — including for the
  compiler's own derives, should they ever move to runtime installation.
