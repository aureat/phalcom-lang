# Caches & Fusion

> **The VM resolves a selector once per call *site*, not once per call.** Each `Invoke` site owns a
> one-entry memo — *(receiver class, resolved method, version)* — and trusts it until a single global
> `world_version` counter says the world changed. One bump invalidates every site in the program at
> once; the VM never hunts for stale caches, it lets each one fail its own stamp check the next time
> it runs.

[Message Send](message-send.md) ended on a debt. It described lookup as a walk paid *every* send —
hash the selector, probe the receiver's class, climb the superclass chain — and flagged that as
**Lie #1**: real code does the walk *once* per site and remembers the answer. This doc pays that debt.
It is the optimization layer over Doc 4's dispatch, and it has two levers that are easy to conflate
and must be kept apart:

- **Inline caching** makes a *lookup* cheap — remember the binding, replace the walk with a compare.
- **Superinstruction fusion** deletes a *dispatch* — merge two adjacent opcodes so the interpreter
  loop turns once where it used to turn twice. It removes work from Doc 1's loop, not from the walk.

The first is runtime state that fills, goes stale, and refills — exactly the moving-state machinery
the reader finds hardest to hold. So the grip does the holding: *a memo per site, one global stamp,
lazy invalidation.* By the end you should be able to throw away `invoke_at` and rebuild both the hit
path and — the harder half — the moment a cache learns the world moved.

## The debt, stated precisely

Doc 4's walk is a **pure function** of `(receiver class, selector)` — the same inputs give the same
method, for as long as nobody redefines a method in between. Put that walk in a loop over a million
same-class receivers and the VM recomputes one answer a million times. The fix rests on one empirical
claim, worth stating exactly because everything here is a variation on it:

> **The binding at a given call site is stable across calls, far more often than it is not.**

Not "the receiver's class never varies" — a stronger, often-false claim. The weaker, almost-always-
true one: *the class this `Invoke` saw last time is overwhelmingly likely to be the class it sees this
time.* Most polymorphism is **inter-site** — different call sites see different classes — not
**intra-site**. A single `Invoke` in a loop body tends to see one shape of the world even when the
*selector* it sends is wildly polymorphic across the whole program. So the unit of memory is the
**call site**, not the selector and not the class: **call-site specialization**.

This is old. L. Peter Deutsch and Allan Schiffman, *Efficient Implementation of the Smalltalk-80
System* (POPL 1984), named it the **inline cache**. Smalltalk's send *was* Doc 4's walk, and it was
the dominant cost of running Smalltalk; their fix was not a faster walk but to stop walking on the
common path — the first send at a site did the lookup and then *rewrote the call instruction in
place* to jump straight at the method it found, behind a guard checking the receiver's class. The
next time control reached that point there was no lookup: the guard-and-branch sat *inline* where the
generic send had been. Hence the name.

Phalcom keeps the *idea* and drops the *self-modifying code*, which is the first thing to get
straight, because it is where theory and this representation part ways.

## What a Phalcom cache slot actually is

A bytecode VM has no native call instruction to patch. So the cache is not the instruction rewritten
— it is a **side table** riding parallel to the code, addressed by instruction position:

```rust
// chunk.rs::InlineCache — one monomorphic slot, owned by a single Bytecode::Invoke site.
pub struct InlineCache {
    pub class: ClassId,          // receiver class the cached resolution was recorded for
    pub method: crate::heap::ObjRef, // the resolved MethodObject handle
    pub version: u64,            // VM.world_version at record time
}

// chunk.rs::Chunk
pub caches: Vec<Cell<Option<InlineCache>>>,   // parallel to `code`; only Invoke indices are ever Some
pub gcaches: Vec<Cell<Option<GlobalCache>>>,  // parallel to `code`; only GetGlobal/SetGlobal
```

Three representation facts do all the downstream work:

1. **Keyed by `ip`, not stored in the operand.** `caches` is `Vec<_>` the same length as `code`;
   `Chunk::add_instruction` pushes a `Cell::new(None)` onto `caches`, `gcaches`, and `spans` on
   *every* opcode, lockstep with `code`, so any of them can be indexed by a raw `ip` with no
   bounds special-case. `Bytecode::Invoke(u8, u16)` is exactly what Doc 4 said — arity + selector-
   constant index — **unchanged**. The cache lives *beside* the instruction, at the same index. This
   is the seam ADR-0012 reserved and Doc 4 pointed at: a `ClassId`-keyed slot per call site, shape
   fixed at zero cost, population deferred to here.
2. **`Cell` buys interior mutability.** A refill has to write the slot while the VM holds only a
   shared `&Chunk` borrow (the running code it is reading from). `Cell<Option<InlineCache>>` makes
   `.set(...)` legal through `&`. No `unsafe`, no `RefCell` runtime check — the slot is `Copy`, so a
   `Cell` is exactly the right tool.
3. **Two tables, one shape, deliberately split.** `gcaches` is the *same* idea (a side slot + a
   version stamp) for a *different* opcode: global-variable reads. They are separate `Vec`s, not one
   `enum`, because `Invoke` and `GetGlobal`/`SetGlobal` never occupy the same instruction, and a union
   variant would pay the wider slot's size at *every* site of either kind. (More on `gcaches` below.)

So a hit is: index `caches[ip]`, compare a `ClassId` and a `u64`, use the remembered method. No hash,
no chain climb. The representation's whole personality is in those two comparisons — and in what the
`u64` means.

## How much can a site afford to remember?

Before the stamp, the other axis: *how many* answers does a slot hold? This is a real fork with real
occupants — not a ladder from worse to better, but four bets on what call sites actually see. <a id="lie-1"></a>**Lie
#1:** this section describes designs Phalcom **does not run** — it is monomorphic only. The design
space is the pedagogy; where Phalcom sits in it is the point.

**No cache — walk every send.** The honest baseline, and not a strawman: it is *always correct* (there
is nothing to invalidate), the interpreter core is smaller, there is zero soundness risk. Its whole
bill is that for a message-send-heavy language the walk *is* the dominant cost — the VM spends more
time deciding what to run than running it. That is the cost Doc 4 quietly carried.

**Monomorphic — one slot: `(class, method, stamp)`.** The Deutsch–Schiffman bet, and Phalcom's: *this
site sees one class, consistently.* True at a striking fraction of sites (a getter on one class, a
loop over a homogeneous collection). A hit is one identity compare. The failure mode is a site fed
two or three classes in rotation: the single slot **thrashes** — every send finds the *other* class
cached, re-walks, refills, and pays the walk *plus* a failed compare, strictly worse than no cache.
Monomorphic caching bets that intra-site polymorphism is rare enough to eat the occasional thrash.

**Polymorphic (PIC) — a few slots, scanned.** SELF's answer (Hölzle, Chambers, Ungar, ECOOP 1991):
some sites are *stably, boundedly* polymorphic — two or three classes, forever. Give them N slots,
scan on miss, append rather than evict. A hit anywhere in the short list still beats a walk. What the
PIC bought beyond speed is the detail that mattered historically: because it *records every class it
has seen*, it is a runtime type histogram — **type feedback** — which a JIT can read to inline a
polymorphic site down to a guarded direct call. **Phalcom has no JIT and no PIC**, so even if it kept
the histogram there would be nothing to spend it on. Worth naming once and setting down.

**Megamorphic fallback — give up, share a table.** Past the slot limit, per-site caching stops paying;
V8 and HotSpot fall back to a shared, program-wide `(class, selector)` **global method cache** — which
is, structurally, the *pre-inline-cache* Smalltalk technique kept alive as the floor. **Phalcom has no
such floor.** A thrashing site here just re-runs `lookup_method_in_hierarchy` (Doc 4) every time — the
no-cache cost, per site, with no shared backstop. That this is *correct* is verifiable: one call site
fed `A`, `B`, `A`, `C`, `B`, `A`, `C` instances all understanding `tag()` prints exactly that
sequence, no crash — the monomorphic slot silently re-walking on every receiver-class change.

So place Phalcom on the fork before the next section, honestly: **monomorphic, no PIC, no megamorphic
floor.** The seam is deliberately shaped to *grow* a PIC without a bytecode change (the plan keeps
that option open), but a PIC is not built. Which leaves the hard half.

## The hard half: telling a cache the world moved

A cached binding is only correct until something disturbs it, and in a language where a class's method
table can be mutated *after* sends against it are already cached — reopen a class, redefine a method,
Doc 4's `reopen Widget` fixture — getting this wrong is not a slow program, it is a **wrong** one: the
slot still matches on class identity, its guard still passes, and it confidently invokes a method that
was overridden or removed. Silent wrong output. So a cache is only as correct as its answer to one
question: **how does a slot learn the world changed?** Two structurally different answers, both real,
neither a strawman.

**Per-class epoch — a version that travels with the thing that can go stale.** Every class carries its
own counter; a mutation bumps that class's counter *and every class beneath it* (a superclass change
can shadow what a subclass resolves through). A slot stores the epoch it was filled at and trusts
itself only if the class's current epoch matches. **Fine-grained**: redefining a method on `Foo`
invalidates only caches that resolved through `Foo`; an unrelated `Bar` site never notices, never
refills. The bill is bookkeeping that is easy to get catastrophically wrong — *every* mutation site
must find *every* class whose epoch needs bumping, walking down the subclass tree, and missing one is
not a crash but a silent staleness bug in the exact shape of the one this machinery exists to prevent.

**Single global counter — one number for the whole VM.** Keep one counter. *Any* method install
anywhere bumps it. Every slot stores its value at fill time and is sound iff that value still equals
the global one. **Trivially correct** — a two-line proof, not an invariant spread across every
mutation call site forever: a method install *anywhere* is by definition a change to "the world," and
the counter tracks exactly that. The bill is coarseness: a method defined on `Foo` bumps the same
counter a `Bar`-only site is watching, so `Bar`'s perfectly valid slot is discarded too — not because
`Bar` changed, but because the number it compared against moved. Each stale slot notices lazily, on
its next probe, and pays one refill. Cheap when a program defines its classes at startup and then runs
a long mutation-free steady state; expensive only when it redefines methods *inside* a hot loop.

### Predict, then check

You now have the grip. Use it before being told the answer.

> A loop sends `w.label()` a million times; the slot is warm. At iteration 500,000 the program reopens
> the class and redefines `label`. Do the remaining iterations run the old body or the new one — and
> *how could the VM possibly know*?

The chain-walk-every-time design (no cache) picks up the new method on the next send, uninterestingly,
because it never trusted anything. A *cached* design gets it right **only if the redefinition itself
participates in invalidation** — the method-install must be the thing that moves whatever the slot is
watching. If it does, the warm slot fails its next stamp check, falls through to a full lookup, finds
the new method, refills, and every later iteration runs it — *the cache misses exactly once*, at the
change. If some install path forgets to bump, the loop calls the dead method forever, and that is
indistinguishable from the outside between "working as designed" and "invalidation bug." That
indistinguishability is *why* invalidation is the hard half — a cache-shape mistake costs speed; an
invalidation mistake costs correctness and looks like nothing happened.

Phalcom's answer is the global counter — `VM::world_version: u64`, one field, initialized to `0`,
bumped at the six method-install sites and nowhere else:

```rust
// vm/dispatch.rs — Bytecode::Method arm (static and instance install)
self.heap.class_mut(class_id).add_method(selector, method_id);
self.world_version += 1;   // unconditional; identical in both branches, no per-class filtering
```

And the check runs it live. Reopening `Widget` mid-run, with the cache warmed by twenty hits first:

```
class Widget { label() { return "v1" } }
var w = Widget.new()
var i = 0
while (i < 20) { System.print(w.label())  i = i + 1 }   // 20 × "v1" — the slot is live and hitting
class Widget { label() { return "v2" } }                // reopen: add_method bumps world_version
System.print(w.label())                                 // → v2
```

Observed tail: twenty `v1`, then `v2`. The twenty warm sends prove the slot was hitting; the reopen
bumps `world_version`; the next probe's stamp no longer matches and re-resolves. The mechanism, traced:

```mermaid
sequenceDiagram
    participant Loop
    participant Site as Invoke @ ip
    participant Slot as caches[ip]
    participant Walk as lookup_method (chain)
    Loop->>Site: send #1
    Site->>Slot: probe — empty
    Site->>Walk: walk chain
    Walk-->>Site: (Widget, label→v1)
    Site->>Slot: fill (Widget, v1, version=S0)
    Loop->>Site: send #2 .. #20
    Site->>Slot: probe — class==Widget && version==S0?
    Slot-->>Site: hit — invoke v1, no walk
    Note over Slot: class Widget reopened → world_version S0→S1
    Loop->>Site: send #21
    Site->>Slot: probe — version S0 != S1
    Slot-->>Site: miss — stale
    Site->>Walk: walk chain again
    Walk-->>Site: (Widget, label→v2)
    Site->>Slot: refill (Widget, v2, version=S1) — invoke v2
```

### The second prediction — and the honest knot

> You define a method on class `Foo`. A call site elsewhere only ever sends to `Bar`, untouched. Is
> `Bar`'s warm cache invalidated?

**Yes.** The stamp is *global*. `Foo`'s install bumps the one counter every slot compares against, so
`Bar`'s slot — sound, unchanged — is discarded on its next probe and pays one refill. That is the
price of one `u64` instead of per-class epochs, and it teaches the representation better than any
prose: there is no "the class changed" in Phalcom's cache, only "the world changed."

Here the doc owes an honesty debt of its own. It is tempting to write that Phalcom *chose* the global
counter for its simplicity. **It did not, and no document claims it did.** <a id="lie-2"></a>**Lie #2:**
this doc presents global invalidation as a settled design; it is really an *absence of the planned
machinery*. The `U-IC` unit plan (`docs/forge/units/U-IC/plan.md`, still `Status: PLANNED`) recommends
the **per-class epoch** (its DEC-IC-A) and lists the global counter only as an "acceptable v1"
*fallback*. What runs at HEAD is that fallback — and `ClassObject` carries no epoch field at all, so
the fine-grained scheme is not merely unused, it is *unbuilt*. The coarse counter is correct, and it
landed as three small commits *outside* the U-IC plan's own scope; it is not the reasoned choice of
coarseness the word "chose" would smuggle in. What runs, against what is planned:

| | Runs at HEAD | `U-IC` plans |
|---|---|---|
| IC invalidation | single global `world_version`, bumped on *any* install | per-class epoch, bumped up the affected subtree (DEC-IC-A) |
| Selector space | mixed `Symbol(u32)` (vars/fields/selectors) | dense `SelectorId` carved out (Change 1) |
| Method dict | `IndexMap<Symbol,ObjRef>` per class, chain-walked | design-B per-class own-method arrays (Change 2) |
| Cache arity | monomorphic (1 slot) | monomorphic v1, extensible to PIC |
| Slot storage | side table (`caches`/`gcaches`) | side table — *already matches* (DEC-IC-C) |

Three of those rows are `U-IC`'s scope items still un-landed; two (side table, monomorphic) are
decisions the plan poses that shipped code already answered its way. ADR-0041, sealing the class
hierarchy, even *assumes* the epoch shape — it speaks of "invalidate every dependent inline cache
(reusing the ADR-0018 override epoch)" for a future mutable-superclass feature. That language predates
the global counter; the invalidation it imagines is not the invalidation that runs. Read the plans as
intent, HEAD as truth.

## The hazard the ordering guards

One subtlety in the hit path is load-bearing enough to trace, because it is the kind of bug that
survives testing. Here is `invoke_at`, stripped to the probe and refill:

```rust
let receiver_class = receiver.class(self);            // read BEFORE lookup
let cached = chunk.caches[cache_ip].get().filter(|s| {
    s.class == receiver_class && s.version == self.world_version
}).map(|s| s.method);
if let Some(method) = cached {
    self.call_method(&receiver, method, arity, span)?;         // hit
} else if let Some(method) = receiver.lookup_method(self, selector_sym) {
    // Both reads that stamp the slot happen AFTER the lookup, on purpose.
    let entry = InlineCache { class: receiver_class, method, version: self.world_version };
    chunk.caches[cache_ip].set(Some(entry));                   // refill
    self.call_method(&receiver, method, arity, span)?;
}
```

The `world_version` written into the slot is read **after** `lookup_method` returns, not before.
Why it must be: `lookup_method` can *re-enter the VM* — a `doesNotUnderstand` handler, a getter that
triggers a method install — and such a re-entrant send can bump `world_version` mid-lookup. Snapshot
the version *before* the walk and you would stamp the slot with a version older than the world the
method was actually resolved in; a later hit on that slot would then serve a binding chosen before an
intervening redefinition — silently. Reading `world_version` at the latest possible point, refill
time, closes the window: the stamp is always the version the world was in when the entry became valid.
(`receiver_class` needs no such care — a lookup cannot change the receiver's own class, only the
method tables, which the version alone is charged with catching.)

## The second cache: globals, same shape, different stamp

`GetGlobal`/`SetGlobal` reuse the whole idea on a different stamp. A global name would otherwise
re-probe a `name → slot` hashmap every read; instead each site owns a `gcaches[ip]` slot:

```rust
// chunk.rs::GlobalCache
pub struct GlobalCache { pub module: ObjRef, pub slot: usize, pub version: u64 }
```

The stamp is **not** `world_version` — it is `ModuleObject::globals_version`, a *per-module* counter
bumped when that module declares a new name (which could shadow the cached resolution). A hit indexes
straight into the module's `globals` vector. Two honest notes: this is the F12 cut (commit `39d9042`);
and `SetGlobal` has **no** core-module fallback — unlike `GetGlobal`, writing a name the module never
declared is an error, not a write into core, so a `SetGlobal` hit always names this module's own slot.
Same memo-and-stamp skeleton, a second time, on a counter scoped to the thing that can invalidate it.

## The other lever: deleting the dispatch

Everything so far makes a *lookup* cheap. Fusion attacks a different cost entirely: the interpreter's
own fetch-decode-execute turn — landing on an instruction, decoding it, jumping to its handler —
independent of what the handler then does. Call that **dispatch overhead**, sharply distinct from
*work*.

The compiler can see, statically, that `x.foo()` always compiles to a load followed by a send. So a
peephole pass fuses the pair into one opcode:

```rust
// chunk.rs::fuse_superinstructions — runs once at Callable construction
let Bytecode::Invoke(arity, selector) = self.code[p + 1] else { continue };
self.code[p] = match self.code[p] {
    Bytecode::GetLocal(slot) => Bytecode::InvokeLocal(slot, arity, selector),
    Bytecode::Constant(idx)  => Bytecode::InvokeConst(idx, arity, selector),
    _ => continue,
};
```

The trick is the **in-place rewrite**. The fused opcode overwrites the pair's *first* instruction; the
original `Invoke` is left at `p+1` as **dead code** (the fused arm advances `ip` by 2, nothing falls
through to it). `code.len()` never changes — so every jump offset, and every `ip`-indexed side table
(`spans`, `caches`, `gcaches`), stays aligned with **no re-layout pass**. And the payoff that ties the
two levers together: the fused arm reads its cache at `ip + 1` — *the dead `Invoke`'s own slot* — so a
fused send probes the exact same `InlineCache` the unfused pair would have. **Fusion and caching
compose**; the send body was extracted into the shared `invoke_at` precisely so `Invoke`, `InvokeLocal`
and `InvokeConst` cannot drift.

Soundness has one condition: `p+1` must be unreachable by any jump. If a branch targeted the dead
`Invoke`, control would land on a corpse. So `branch_targets()` collects every
`Jump`/`JumpIfFalse`/`JumpIfNone`/`Loop`/`GuardBool`/`GuardBlock` destination and the pass skips any
pair whose `Invoke` is a target. This is the general **peephole-safety** problem — any in-place
bytecode rewrite is sound only against a complete set of control-flow entry points — and getting it
wrong is a silent bug of the same profile as a missed invalidation subtree. (Per perf-log 008 the
guard fires in *zero* chunks across `core.ph` and 60 fixtures — defensive surface, covered by a unit
test that constructs the jump-into-`Invoke` case and fails without it.)

### Why removing instructions is not removing time

This is the one intuition to correct, and unlike almost everything else in the two learn-tracks it is
**measured**, not reasoned. A fusion buys back only the *dispatch overhead* of the removed instruction,
never its *work*. perf-log 008 pinned a single dispatch at **~3.3 ns** — two independent instruments
agreeing (a differential measurement at 3.56–3.68 ns/instruction as an upper bound including body, and
the cut read backwards at 3.05–3.86 ns for dispatch alone). Fusing `(GetLocal|Constant)→Invoke`
removed 13–20% of the instructions from hot programs and moved:

| benchmark | Δ | | benchmark | Δ |
|---|---|---|---|---|
| `string_equals` | −8.1% | | `fib` | −3.9% |
| `for` | −5.1% | | `binary_trees` | −3.0% |
| `variadic_send` | −4.7% | | `arith_send` | −1.6% |
| `bare_send` | −4.2% | | `map_numeric` | **−0.2%** |

`map_numeric` is the finding, not the outlier: it removed the **most** dispatches of any row (18
million) and moved essentially nothing, because its instructions cost **27.6 ns each** — hashing,
allocation, GC — so deleting a 3.3 ns dispatch from one is noise. **A fusion buys dispatch, and only a
workload whose time *is* dispatch can spend it.** 8.8% of instructions is not 8.8% of time.

The history worth attaching: fusion's ancestor is threaded code — Forth composing primitives into a
word to pay one dispatch per composite instead of per primitive — and its runtime cousin is
*quickening* (rewrite a generic opcode into a specialized one *after* first execution, which is exactly
CPython's PEP 659 adaptive interpreter, the closest living relative of a bytecode-level cache with a
version tag). Phalcom fuses at *compile* time from static adjacency; it does not quicken. One more
honest note: perf-log 008 records the F16 verdict being **overturned** — superinstructions were first
deferred as premature because "the inliner already covers the arithmetic win," and the re-ask found
that reason *false*: the inliner's set is control-flow only (`ifTrue`, `and`, `whileTrue`, …), so
`1 + 2` was never inlined and the win was real and unclaimed. A measured scar, not a tidy story.

## Where the fast paths end

One more fast-path family, named and set down. `GuardBool`/`GuardBlock` are *not* method caches — they
back the sacred-selector inliner (ADR-0018), where the compiler emits jump opcodes instead of a real
send for a handful of kernel selectors (`ifTrue(_)`, `whileTrue(_)`, `and(_)`, …). Each guard reads a
`pristine` flag; `note_method_installed` flips the relevant one of five flags (`bool_sacred_pristine`,
`block_sacred_pristine`, and the `toString` flags for `Number`/`Symbol`/`String`) to `false` the
instant such a selector is redefined on its kernel class, deopting the inlined path back to a real
send. It is a third version-stamp mechanism, narrower than the method IC, and its full machinery is its
own topic. And `SuperSend` (Doc 4's forward pointer) is **uncached** — a statically-known target left
out of the IC in v1.

---

Delete `invoke_at` and you can rebuild all of it from the grip. A call site holds an empty side-slot
keyed by its own `ip` (representation, not operand). The first send walks Doc 4's chain, stamps the
slot with the receiver class and the *current* `world_version` (read after the walk, or you cache a
lie). Every later send is a `ClassId` + `u64` compare — until any method install anywhere ticks the one
global counter, at which point every slot in the program quietly fails its next stamp and re-warms,
one walk each. Globals get the same skeleton on a per-module stamp; fusion deletes whole dispatches
orthogonally and composes with the cache by sharing its slot. Monomorphic, globally-stamped, no PIC,
no epoch — the coarse-but-correct floor that the planned machinery was going to refine and hasn't yet.

## Anchors

- `phalcom-core/src/chunk.rs::InlineCache` (@ ~L10) — `{ class: ClassId, method: ObjRef, version: u64 }`.
  `GlobalCache` (@ ~L30) — `{ module, slot, version }`. `Chunk::caches`/`gcaches` (@ ~L50/~L55) —
  `Vec<Cell<Option<_>>>`, parallel to `code`, indexed by `ip`; `add_instruction` (@ ~L77) grows them lockstep.
- `phalcom-core/src/vm/mod.rs::VM::world_version` (@ ~L116) — the single global `u64`; init `0` at
  `vm/bootstrap.rs` (@ ~L39); bumped at six install sites (`vm/dispatch.rs` ~L927/~L930,
  `primitive/mod.rs` ~L117/~L133, `universe/primitives.rs` ~L164/~L197).
- `phalcom-core/src/vm/dispatch.rs::VM::invoke_at` (@ ~L398) — probe `s.class == receiver_class &&
  s.version == world_version`; refill through the `Cell`; `world_version` read **after** `lookup_method`.
- `phalcom-core/src/heap/class.rs::ClassObject` (@ ~L25) — **no** epoch/version field (the absent
  per-class machinery); `MethodsMap = IndexMap<Symbol,ObjRef>` (@ ~L17); `lookup_method_in_hierarchy`
  (@ ~L74) still chain-walks (Doc 4).
- `phalcom-core/src/chunk.rs::Chunk::fuse_superinstructions` (@ ~L116) + `branch_targets` (@ ~L137) —
  in-place rewrite, dead `Invoke` at `p+1`, jump-target guard. Fused arms `InvokeLocal`/`InvokeConst`
  (`bytecode.rs` @ ~L344/~L354; dispatch arms `vm/dispatch.rs` @ ~L1036/~L1046) read the IC at `ip+1`.
- `phalcom-core/src/heap/module.rs::ModuleObject::globals_version` (@ ~L62, bumped in `define` @ ~L141)
  — the `gcaches` stamp. `GetGlobal`/`SetGlobal` (`vm/dispatch.rs` @ ~L632/~L685); `SetGlobal` has no
  core fallback.
- `phalcom-core/src/universe/mod.rs::note_method_installed` (@ ~L188) — flips five `*_pristine` flags;
  `GuardBool`/`GuardBlock` (`vm/dispatch.rs` @ ~L1184/~L1195). ADR-0018.
- ADR-0012 (the IC seam, population deferred — Doc 4's Lie #1). ADR-0051 (Tier-3 perf strategy names
  `U-IC`). ADR-0041 (hierarchy stability; assumes an override-epoch invalidation the global counter
  does not implement). `docs/forge/units/U-IC/plan.md` (**PLANNED** — the unbuilt fine-grained form).
  `docs/forge/perf-log/008-fuse-invoke-pairs.md` (measured fusion: ~3.3 ns/dispatch, the result table,
  the F16 flip, the `map_numeric` non-result).

## Forward pointers

- **[Doc 6 (frame identity)](frame-identity.md)** — the last VM-track doc: `FrameToken`, `generation`, and how a stale
  non-local `return` is *detected* rather than corrupting memory.
- **Per-class epoch, PIC, selector-only interner** — the `U-IC` plan's fine-grained cache: none built
  at HEAD (Lie #1's single slot and Lie #2's global counter are what run). Cited as intent, not truth.
- **The sacred-selector inliner** (ADR-0018) — `GuardBool`/`GuardBlock` are named here; the
  `compile_sacred_call` machinery and its override-epoch deopt are its own future topic.
- **`SuperSend`** (ADR-0040) — its statically-known target is uncached; folding it into the IC is a
  deferred follow-on (DEC-IC-B).
