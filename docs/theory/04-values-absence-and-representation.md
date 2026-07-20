# 04 — Values, absence, and representation

> **Thesis:** the shape of your value word is a budget, and every semantic commitment spends it.
> Absence, numeric identity, and object references all compete for the same bits, and the order
> in which you commit to them determines which optimizations remain reachable. Most of the
> interesting decisions here are about *sequencing*, not about the choices themselves.

---

## 1. The value word

**`[V]`** `Value` is a 16-byte `Copy` tagged Rust enum:

```rust
enum Value { Nil, Bool(bool), Number(f64), Symbol(..), Obj(ObjRef) }
```

Not NaN-boxed. `ObjRef` is 8 bytes — a 32-bit slot index plus a 32-bit generation counter — and
`Option<ObjRef>` is *also* 8 bytes, because Rust's niche optimization uses an invalid bit pattern
for `None`. That the compiler performs niche encoding automatically for the host language's
`Option`, while the guest language's `Option` is a full heap object, is a nice miniature of the
whole subject.

**`[M]`** The size comparison against the reference implementation is unflattering and precise:
`Value` 16 B versus Wren's 8 B (2.0×) and `CallFrame` 96 B versus 24 B (4.0×). Since fiber stacks
are stacks of `Value`s, those ratios directly explain a measured 2.0× resident-memory figure on
the concurrency benchmark. Representation is not an abstract concern; it is multiplied by every
live frame in the system.

**`[V]`** One deliberate piece of forward planning deserves attention: `as_obj()` is maintained as
the **sole garbage-collection seam into `Value`**. A future NaN-boxing migration therefore rewrites
one function and leaves mark and sweep untouched. Choosing a single accessor as the chokepoint,
before the migration exists, is cheap discipline that keeps an expensive option open.

---

## 2. Why NaN-boxing is blocked, and by what

**`[R]`** The technique: IEEE-754 doubles have a large space of NaN bit patterns, so a 64-bit word
can hold either a real double or a tagged payload hidden inside a quiet NaN. Pointers on
mainstream 64-bit platforms fit in 48 bits, so a pointer fits in the payload with room for a tag.
The result is an 8-byte universal value with no boxing for numbers.

**`[V]`** Phalcom has a hard blocker, and it is arithmetic rather than aesthetic: `ObjRef` is
a 32-bit index plus a 32-bit generation = **64 bits**, and does not fit a 48-to-51-bit NaN
payload. Adopting NaN-boxing therefore requires first shrinking the handle below 48 bits — which
means shrinking either the index space or the generation counter, and the generation counter is
what makes use-after-free *detectable*. So the sequence is: shrink the handle, audit the
use-after-free detection that the shrink weakens, and only then box.

**`[V]`** A second, independent blocker arrived from a different direction. ADR-0024 commits to a
split numeric tower — abstract `Number` over an auto-promoting bignum `Int` and an IEEE-754
`Float`. NaN-boxing's premise is that *the* numeric type is a double. Under a split tower, the
question is not "does NaN-boxing pay" but "which of the two numeric types does it pay for, and
what does the other one cost." That is a different question, and the deferral predates it.

**The transferable point:** deferred work does not merely wait, it **decays**. The reasons for a
deferral are premises about the surrounding system, and the surrounding system moves. A deferral
record should name its premises so a later reader can check whether they still hold, rather than
inheriting a verdict whose basis has silently changed.

---

## 3. One absence, and the bootstrap cycle it appears to create

**`[V]`** Phalcom has abstract `Option` with `Some(_value)` and a `None` singleton, and **no
surface `nil`**. A private VM sentinel named `nil` exists but has no class, no literal, and cannot
leak into a `Some` — an invariant with standing enforcement obligations.

The design payoff is that absence becomes *dispatch* rather than *branching*: `map`, `orElse`,
`unwrapOr`, and `ifNone` are methods on the two variants, so the caller never tests a tag.

**`[V]`** The apparent bootstrap cycle is a good puzzle and its dissolution is better. Fields
default to `None`. Constructing `None` seemingly requires a class whose fields default to `None`.
The resolution: **`None` is fieldless**. With no fields to default, the rule never re-enters its
own construction, so the cycle never forms. No code change was ever needed — the tree was already
correct, and the ADR's contribution was to *formalize why*.

**The lesson:** a bootstrap cycle is usually a claim about a dependency that has not been checked
at the right granularity. "Constructing X requires the field-default rule" was true of `Some` and
false of `None`, and the whole cycle lived in the imprecision. Before designing around a cycle,
state it as a concrete dependency between concrete steps and check each one.

**`[V]`** `None` is additionally a **zero-allocation** singleton with identity equality, while
`Some` allocates per use. This asymmetry is load-bearing for the iteration protocol and shows up
again in §5.

---

## 4. Truthiness as an enforcement problem

**`[V]`** Truthiness is banned: only a `Bool` may be a condition. The interesting part is that
banning it is not a syntax decision but an *enforcement* decision, and this project shipped
**both** halves of the enforcement recipe rather than one:

1. A runtime `GuardBool` floor primitive rejects any non-`Bool` condition, with no coercion.
2. A compile-time rejection of *syntactically literal* Option conditions — `if (None)`,
   `if (Some.new(x))` — via a dedicated recognizer.

**`[V]`** The accepted gap is documented rather than hidden: indirection defeats the compile-time
check. `let x = None; if (x)` is caught only at runtime. This is the general shape of static
enforcement in a language with no flow analysis, and stating it as an accepted cost is more useful
than either pretending the check is complete or abandoning it because it is partial.

**The generalizable framing:** "ban X" is not a design, it is a goal. The design is the pair
(*what rejects X*, *what X escapes through*). A ban with only a runtime half moves errors to
production; a ban with only a compile-time half is trivially circumvented; and stating the escape
hatch precisely is what lets a future reader decide whether closing it is worth a flow analysis.

---

## 5. Two optimizations that sound identical and are not

**`[V]`** The sharpest distinction in the representation record, and the kind of thing that is
obvious once stated and invisible until then:

- **Bare-cursor iteration** (`iterate(_)` returns the next cursor or the `None` singleton;
  `iteratorValue(_)` maps cursor to element) removes `Some` allocation **only inside iteration
  loops**. Zero per-step allocation in `for` and every combinator.
- **Niche-encoding `Option` into the value word** would remove `Some` allocation **everywhere
  `Some.new(x)` is called.**

Both are described as "removing Option allocation." They apply to disjoint call sites, have
different costs, and neither subsumes the other. **`[V]`** The bare-cursor protocol also
introduces a new global constraint that the alternative does not: *a cursor value may never itself
be `None`*, since `None` is the end sentinel. That is a real restriction on user-defined iterables
and it exists purely because of how the allocation was eliminated.

**`[M]`** A related measured result that killed a plausible optimization: `List.at` and `Map.at`
were **already zero-allocation** via the `None` singleton, so an "escape analysis for `Option`"
optimization had no premise. Independent reasoning from a redesign discussion had predicted this
outcome, which was recorded as a small validity check on the reasoning — a good habit, and cheap.

**`[V]`** Guardrails recorded in advance for any future niche work, which is the right time to
record them: `None` stays identity-comparable and zero-allocation; `nil` and `None` must never
become confusable; `match` and the combinators must stay observationally identical.

---

## 6. The numeric tower: a decision reversed, then costed properly

**`[V]`** This one has an unusual shape and is worth following as a sequence.

The first decision **deferred** the integer/float split, keeping flat `f64`, on the reasoning that
the handle heap and signature-keyed dispatch would let the split land later without perturbing
references or adding a dispatch axis. That reasoning is sound and the deferral was cheap.

It was **retired within days**. The replacement: abstract `Number` over an auto-promoting bignum
`Int` and IEEE-754 `Float`; `/` is always true division (so `Int / Int → Float`); `~/` is floor
division returning an exact `Int`.

Then the *implementation* analysis found three things the design analysis had not:

1. **`[V]` The lexer destroys the discriminant.** `Token::Number(f64)` makes `1` and `1.0` lex
   identically. The split therefore has to happen at the parser boundary, and the AST crate — not
   previously in the write set — enters it. A representation decision reached backwards into the
   front end.
2. **`[V]` The floor census was wrong in every document that stated it.** The true count, obtained
   by running the census test in a clean worktree at a pinned commit, was **137**; existing docs
   said 125 and 136. The lesson is stated bluntly in this project's own operating rules: never
   quote a census number from a document, run the check.
3. **`[V]` The per-class split roughly doubles the numeric floor**, 14 → 30 primitives.

**`[V]`** And two live defects fell directly out of the split, both of which are instructive
because they are *latent today and load-bearing later*:

- **`Float#hash` breaks the hash-equality contract.** After the split, a large integral `f64` will
  compare equal to an `Int` under the new promotion rules, while hashing via `to_bits()` gives a
  different hash. `a == b ⇒ a.hash == b.hash` fails, and `Map`/`Set` depend on it. Harmless today,
  broken the moment the split lands.
- **`Int#%` must be floored**, not Rust's remainder, to satisfy
  `a == (a ~/ b) * b + (a % b)`. Two operators that must agree, defined in different places.

**`[V]`** Plus a *sequencing* hazard, which is the kind most easily missed: the numeric
specification must land **before** the arithmetic inliner hardens, or the inliner bakes `f64`
assumptions into its guards and deoptimization edges. Two units with no code overlap, ordered by a
dependency that exists only in the assumptions one would embed.

---

## 7. Dead variants are archaeology

**`[V]`** `RuntimeError::ZeroDivision` is defined and **never raised**. Division follows `f64`
exactly: `1/0` is `inf`, `-1/0` is `-inf`, `0/0` is `NaN`. Adopting IEEE-754 semantics deleted an
error case, and the enum variant is the fossil of the decision that preceded it.

A worthwhile habit follows: **an unreachable variant, an unused field, or a dead flag is a record
of a superseded design.** Before deleting one, read it as evidence — it tells you what the system
used to believe. And note the corollary, which is that dead variants are also how a specification
lies: readers reasonably assume a defined error is a raisable error.

**`[V]`** A related probe result, small and genuinely surprising: `1 + e` and `e + 1` produce
*different* error types when `e` is an error object — `Error` versus `MessageNotUnderstood` —
because under pure message-send arithmetic the receiver determines which lookup fails. Operand
order changes the failure mode. This is a real cost of "everything is a message," and it is
exactly the kind of thing that surfaces only by probing.

---

## 8. Object layout, and the cost of a fat variant

**`[V]`** Instances use a fixed per-class slot vector with `GetField(slot)` / `SetField(slot)`
opcodes — no per-object hash map, no dynamic shape. Fields are **private and non-inherited**, and
read-before-write is a compile error.

**`[V]`** The heap is a slot map, and a slot map sizes every slot to its **fattest variant**.
`ClassObject` at 280 B is the fattest `Object` arm, so leaving it inline would tax every `String`,
`Tuple`, and `Instance` on the hot `Heap::get` path. Hence selective boxing of the fat arm.

**The generalizable rule:** in any uniform-slot arena, the largest variant sets the price for
everything. This is the same arithmetic as a struct-of-arrays versus array-of-structs decision,
and it is worth checking early, because the fix (boxing the outlier) is easy while the diagnosis
(a slow `get` on unrelated types) points nowhere near the cause.

**`[V]`** A consequence in a different domain entirely, and a nice illustration of how far a
layout decision reaches: because fields are private and non-inherited, the payload-destructuring
half of ML-style structural pattern matching is **already precluded by the object model**. There
is no way to read another object's slots except through its accessor protocol. Any `match` syntax
could only desugar to protocol sends — strictly worse than calling an eliminator method directly,
since it would add grammar, gain no totality, and need its own fallback-arm story that selector
identity already provides for free.
