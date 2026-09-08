# Phalcom Specification — Resolutions

Companion to [`spec-consistency-audit.md`](spec-consistency-audit.md). Every finding
in that audit is resolved below with (a) a **decision** and (b) the **concrete
edits** that carry it out. Three items were genuine language-design forks and were
decided by the maintainer:

| Fork | Decision |
|------|----------|
| **B1** — canonical interned selector string | **Comma form** (`move(_,to,duration)`). Surface declaration/call syntax keeps trailing colons; the *interned string* drops them. |
| **B2** — `True`/`False` visibility | **Surface-visible real classes**, exactly mirroring `Some`/`None`. |
| **C6** — default arguments | **Deferred**, promoted into `open-questions.md` with a shipping-blocking marker. |

Everything else has an objectively-correct or clearly-recommended resolution and is
decided here.

---

## A. Structural fixes

### A1 — `selectors.md` is orphaned → adopt it and back it with an ADR

**Decision.** `selectors.md` becomes a first-class part of the spec, renamed for
what it covers, and is ratified by a new **ADR-0017**.

**Edits.**
1. `README.md` reading-order table — insert a row after *Messages & Selectors*:
   | [Symbols & Method References](selectors.md) | `#` symbol literals, `::` method references, `@` attributes, field visibility |
2. Rename the document's H1 from the standalone "Selectors, Symbols, and Method
   References" to **"Symbols & Method References"** (selector *identity* proper now
   lives in `messages-and-selectors.md`, which `selectors.md` should defer to
   rather than re-declare — see B1).
3. Create `docs/adr/accepted/0017-symbol-literals-method-references-and-attributes.md`
   recording the `#`/`::`/`@` decisions, so "design-locked" is backed the same way
   every other resolved decision is. Change `selectors.md`'s status line from a
   bare "Decided (design-locked)" to `**Governing ADR:** ADR-0017`.

### A2 — three open-questions registries → one

**Decision.** `open-questions.md` is the single registry. `selectors.md §7` and
every inline table-punt are folded into it (resolved rows struck through, open rows
numbered).

**Edits.**
- Move `selectors.md §7.1` (var→None) into `open-questions.md` as **resolved** (it
  is — ADR-0014; see B6) and delete it from `selectors.md`.
- Move `selectors.md §7.3` (default args) in as a **new open question** with the
  shipping-blocking marker (see C6).
- Move `selectors.md §7.2` (`ifTrue`/`ifFalse` chaining) in as **resolved** (see
  B4) and `§7.4`/`§7.5` (`Option` bootstrap, `Family` introspection) in as open.
- Replace `selectors.md §7` with a one-line pointer: *"Open questions for this part
  are tracked in [Open Questions](open-questions.md)."*
- Resolve the `system.md` `gc` inline punt by pointing at the new Unit convention
  (see D1).

### A3 — ADR-0004 is "Accepted" but says "pending approval"

**Decision.** With B2 decided (surface-visible), ADR-0004 is genuinely Accepted.

**Edits.** In `0004-...md`: delete "**Recommendation (pending approval):**" and
state the decision as adopted. Strengthen the Decision section to match B2:
`True`/`False` are surface-visible subclasses of `Bool`, `true.class == True`,
`isA(True)` is meaningful. Keep the "no new `Value` variant" note — it is correct
and independent of visibility.

### A4 — broken link

**Edit.** `values-and-absence.md §3.1`: `[ADR-0004](../../../adr/)` →
`[ADR-0004](../../../adr/0004-boolean-as-abstract-bool-with-true-false.md)`.

---

## B. Contradiction resolutions

### B1 — one canonical selector string: **comma form**

**Decision.** There are two distinct notations and the spec must name each:

- **Surface syntax** (what you type) keeps trailing colons in declarations and
  calls: `move(to:, duration:) { }`, `p.move(to: a, duration: b)`. Unchanged.
- **Interned canonical string** (what is hashed, printed in diagnostics, produced
  by `#`-literals and `encode_selector`) is **comma form**: `move(_,to,duration)`.

This makes `selectors.md`'s grammar authoritative and `messages-and-selectors.md`
the place the surface↔interned mapping is defined.

**Edits.**
1. `messages-and-selectors.md §2` — the "Selector symbol" column switches to comma
   form and gains an explicit surface/interned split.

   **Slot-encoding rule (the sole rule).** Each argument slot contributes exactly
   one selector slot, in declared order, with **no colons** in the interned string:
   - a **positional** argument (declared without a trailing colon, e.g. `a`) → `_`
   - a **labeled** argument (declared `b:`) → the bare label `b`

   Positionals precede labels (**R2**); the rest slot `*` is last (B1.3); arity =
   slot count. There is **never** a spurious leading `_` — a pure-labeled call has
   none.

   | Surface (declaration → call) | Interned selector |
   |------------------------------|-------------------|
   | `name` (getter) → `p.name` | `name` |
   | `size()` → `p.size()` | `size()` |
   | `add(a, b)` → `p.add(1, 2)` | `add(_,_)` |
   | `func(a, b:, c:)` → `p.func(x, b: y, c: z)` | `func(_,b,c)` |
   | `move(to:, duration:)` → `p.move(to: a, duration: b)` | `move(to,duration)` |
   | `move(x, duration:)` → `p.move(a, duration: b)` | `move(_,duration)` |
   | `name=(v)` → `p.name = v` | `name=(_)` |
   | `+(other)` → `a + b` | `+(_)` |

   > The `func(a, b:, c:)` → `func(_,b,c)` and `move(to:, duration:)` →
   > `move(to,duration)` rows are the ones the earlier draft got wrong: labeled
   > slots carry only the bare label, and a fully-labeled selector has **no** `_`
   > at all. A `_` appears if and only if that slot was passed positionally.

2. `messages-and-selectors.md §3` — declaration section keeps colon syntax but adds
   one sentence: *"The declared form `move(to:, duration:)` interns to the canonical
   string `move(to,duration)` ([Symbols & Method References §1](selectors.md))."*

3. **Variadic marker** — `messages-and-selectors.md §4` currently interns variadics
   as `sum(_...)`, a marker the comma grammar cannot express. **Decision:** the rest
   slot is a single trailing `*`. `sum(*numbers)` interns `sum(*)`;
   `log(fmt, *args)` interns `log(_,*)`. Extend `selectors.md §1`'s grammar:
   ```
   selector := name "(" [ slot { "," slot } ] ")"
   slot     := "_" | "*" | label        // "*" only as the final slot (R2-adjacent)
   ```
   The variadic table is keyed by `(name, count-of-slots-before-*)`. Update every
   `_...` occurrence to `*`.

4. `selectors.md` drops its own "Selector identity" section's *rules* duplication
   and cross-links to `messages-and-selectors.md §2` for identity, keeping only the
   canonical-**string** grammar (its unique contribution).

### B2 — `True`/`False` are surface-visible classes

**Decision.** They are real classes, mirroring `Some`/`None`.

**Edits.**
1. `object-model.md §4` catalog — replace the single `Bool` row + the "not
   surface-visible" note with three rows, matching the `Option`/`Some`/`None`
   treatment:

   | Class | Superclass | Kind | Role |
   |-------|-----------|------|------|
   | `Bool` | `Object` | A | Abstract boolean protocol. `not`, `and(_:)`, `or(_:)`, `ifTrue(_:)ifFalse(_:)`. |
   | `True` | `Bool` | I | Singleton; class of `Value::Bool(true)`. |
   | `False` | `Bool` | I | Singleton; class of `Value::Bool(false)`. |

   Keep the implementation note that dispatch may be realized via the inliner, but
   delete "users see one class, `Bool`."
2. `object-model.md §3` value-representation table — the `true`/`false` row's Class
   column becomes `True` / `False` (its metaclass-style "selected at runtime from
   the payload" is already how ADR-0004 describes it).
3. `values-and-absence.md §3.1` — the "exactly mirrors `Bool`/`True`/`False`" claim
   is now **true**; leave it, and fix the A4 link so it points at the real ADR.
4. Consistency check to add to `verify_invariants()`: `true.class == True`,
   `false.class == False`, `True.superclass == Bool`, `False.superclass == Bool`,
   `Bool` is abstract (no direct instances).

### B3 — fix `blocks.md §7` in place

**Decision.** ADR-0006 claimed the amendment was "recorded inline in that file"; it
was not. Do it now.

**Edit.** `blocks.md` — the opening sentence ("a method is a `Block` bound to a
class under a selector") and §7's "Methods and blocks share one closure
representation" are rewritten to the sibling model:

> A block, a lambda, a method body, and a getter body all share **one closure
> representation** ([Functions §4](functions.md)). A `Method` is **not** a `Block`:
> both are siblings under the abstract [`Function`](functions.md), and a `Method`
> additionally carries a selector, a holder, and a `self` a `Block` does not. See
> [ADR-0006](../../../adr/0006-function-as-abstract-callable-root.md).

### B4 — `ifFalse(_:)` existence + the `Option`-chaining hazard

**Decision.** Adopt `selectors.md §7.2`'s own proposed fix. The **paired**
`ifTrue(_:)ifFalse(_:)` is the *primary, total* conditional and returns the value
of whichever branch ran (no `Option`). The **single-branch** `ifTrue(_:)` and
`ifFalse(_:)` are `Option`-returning sugar and are **not chainable to each other**.

**Edits.**
1. `object-model.md §4` `Bool` row (now abstract, per B2): list the real selectors —
   `not`, `and(_:)`, `or(_:)`, `ifTrue(_:)` → `Option`, `ifFalse(_:)` → `Option`,
   `ifTrue(_:)ifFalse(_:)` → branch value. `ifFalse(_:)` now explicitly *exists*.
2. `values-and-absence.md §3` — add a boxed warning:
   > `cond.ifTrue { a }.ifFalse { b }` does **not** do what it looks like: `ifTrue`
   > returns an `Option`, and `Option` has no `ifFalse`, so the second send is a
   > `doesNotUnderstand`. For a two-armed conditional use the paired
   > `ifTrue(_:)ifFalse(_:)` (or the `if/else` sugar, which desugars to it). The
   > single-branch forms are for *one* branch producing an `Option`.
3. `control-flow.md §1` — change the `if/else` desugaring so it no longer leans on
   the `ifNone` workaround: `if (c) { a } else { b }` ≡
   `c.ifTrue { a } ifFalse { b }` (the paired selector), which is exactly the
   sacred-selector the inliner already targets.
4. Fold `selectors.md §7.2` into `open-questions.md` as **resolved**.

### B5 — `#{1,2,3}` set literal is dead → remove the escape hatch

**Decision.** `#` is the symbol sigil (`selectors.md §2`); `#{` cannot lex as a set.
`Set(...)` is the sole set constructor.

**Edit.** `open-questions.md Q6` — strike the `#{1, 2, 3}` alternative and annotate:
*"`#{…}` is foreclosed: `#` is the symbol-literal sigil ([Symbols & Method
References §2](selectors.md)). `Set(…)` is canonical."* Keep the question open only
insofar as ergonomics might later motivate a *different* literal.

### B6 — `var x` → `None` is resolved; stop re-opening it

**Edit.** Delete `selectors.md §7.1`. It duplicates `open-questions.md Q1`, which is
**resolved** by ADR-0014 (`var x` uninitialized reads `None`; `let x`
uninitialized is a declaration-site error). The `Uninit`-sentinel alternative it
floats was considered and rejected by that ADR; do not resurrect it in a satellite
document.

### B7 — interpolation syntax: resolve Q5, make the assertion true

**Decision.** `{expr}` is canonical. It is already assumed everywhere, implemented,
and has a defined escape (`\{`). Close the question rather than hedge the assertion.

**Edits.**
- `open-questions.md Q5` — mark **RESOLVED**: `{expr}` with `\{` escape is the
  interpolation syntax; `${…}` and `\(…)` rejected (no sigil budget spent, one
  consistent brace-delimited form). Record as ADR-0018 if a paper trail is wanted,
  or inline in `lexical-structure.md` as a "Decided" note.
- `lexical-structure.md §5` — add "(Decided; see Open Questions Q5)" so the
  unconditional statement is backed rather than looking like an unflagged assumption.

### B8 — `@`-attributes need a field *declaration site* the field model lacks

**Decision.** This is the one place `selectors.md` proposes machinery that genuinely
conflicts with a ratified model, and it is **not yet designable as written**. The
implicit-by-assignment field model (`classes.md §2`) gives fields *no declaration
site* to attach `@get`/`@set` to, and the example's `var x` (no `_`, `var` reused as
a class-body keyword) violates both `lexical-structure.md §3` and ADR-0014. Rather
than silently introduce a second field grammar, **demote the attribute section to
explicitly-provisional and open the real question.**

**Edits.**
1. `selectors.md §4` — prefix with: *"**Status: unspecified/provisional.** The
   examples below are illustrative, not normative; the field-declaration site that
   `@get`/`@set` attach to is an open question (see below)."* Rewrite the code
   example so it does not use the illegal `var x` field form — or mark it
   pseudocode.
2. `open-questions.md` — new open question: *"Attribute-annotated fields require a
   field **declaration** site, which the implicit-assignment model
   ([Classes §2](classes.md)) does not provide. Either add an explicit field-decl
   form (distinct from ADR-0014's statement-scoped `var`) or restrict attributes to
   class-level (`@construct`). Blocks the `@get`/`@set` design."*
3. See C7 for the derived-vs-hand-written accessor precedence, which this unblocks.

### B9 — `#"+(_)"` is a third symbol spelling → fix to the operator branch

**Decision.** With comma form canonical (B1), a full operator selector literal is
`#+(_)` (sigil + operator name + comma-form slot list). The quoted `#"..."` form
does not exist.

**Edit.** `functions.md §3` — `3.methodFor(#"+(_)")` → `3.methodFor(#+(_))`. Add
`#+(_)` to `selectors.md §2`'s operator-selector examples (which currently show
only the *name*-symbol operator forms `#+`, `#==`, `#[]`) so a *full* operator
selector literal is exemplified, not just the bare-name ones.

---

## C. Underspecified mechanics — concrete rules

### C1 — enforcing "abstract"

**Decision.** An abstract class is one flagged `abstract` at definition; it defines
no allocator. Instantiation is rejected at the allocation primitive with a typed
error, not left to an accidental `doesNotUnderstand`.

**Edits.** `object-model.md §2` — add:
> An **abstract** class carries an `isAbstract` flag. It has no `construct` and its
> metaclass installs no default allocator. Any allocation attempt
> (`AbstractClass.new(…)`, or reflective `AbstractClass.allocate`) raises
> **`AbstractClassError`** (a new `Error` subclass, `object-model.md §4`), naming
> the class. This is checked in the allocation primitive so the diagnostic is
> precise rather than a generic missing-selector error.

Add `AbstractClassError` to the Errors catalog table.

### C2 — cross-hierarchy slot layout (the algorithm ADR-0011 omits)

**Decision.** Slots are laid out **root-first, append-only**. An instance of class
`C` has a slot vector that is the concatenation, walking `Object → … → C`, of each
class's own private field set. Each class `K` gets a fixed base offset `base_K`
assigned when `K` is finalized; `K`'s fields occupy `[base_K, base_K + n_K)`. A
method compiled in `K` addresses only `[base_K, base_K + n_K)`, so it is valid on
any subclass instance because the prefix `[0, base_K + n_K)` is preserved by the
append-only rule — even when a subclass declares a same-named `_field` (which gets a
*fresh* slot at its own base, never aliasing `K`'s).

**Edits.** Add to ADR-0011 (and summarize in `classes.md §2`):
> **Layout across a hierarchy.** Field offsets are assigned root-first: `base_Object
> = 0`; for each class `K`, `base_K = base_{K.superclass} + n_{K.superclass}`, and
> `K`'s fields fill `[base_K, base_K + n_K)`. Layout is append-only — a subclass
> never renumbers a superclass's slots — which is what keeps a method's compiled
> offsets valid under inheritance and preserves stability under a future runtime
> `superclass=` ([Q4](../../../spec/open-questions.md)) as long as re-parenting only
> appends. A subclass field sharing a superclass field's name occupies a distinct
> slot (fields are private/non-inherited), so there is no aliasing.

### C3 — read-before-write is existence-based, not flow-sensitive

**Decision.** Keep the whole-class existence check (it catches the motivating typo
class cheaply). Document the boundary honestly rather than imply it is
definite-assignment.

**Edit.** `classes.md §2` — append:
> **Scope of the check.** This is a *whole-class existence* check: a field read is
> rejected only if the field name appears as an assignment target in **no** method
> of the class (the `_naem` typo). It is deliberately **not** a per-constructor
> definite-assignment analysis — a field assigned by one `construct` but not another
> passes the check and, on an instance built by the other constructor, reads `None`
> ([Values & Absence](values-and-absence.md)). That is the same mechanism that backs
> legitimately-optional fields; distinguishing "optional" from "forgot to
> initialize on this path" is left to a future flow analysis
> ([Open Questions](open-questions.md)).

Add the flow-sensitive check as an open question.

### C4 — `Fiber` entry arity vs. the resume protocol

**Decision.** `call`/`call(_:)` on a fiber is the **resume** protocol, not the
entry's arity. Resume transfers exactly **one** value (or none) across the
yield boundary: on first resume it becomes the entry function's single parameter (or
is discarded if the entry takes none); on later resumes it becomes the value of the
suspended `Fiber.yield(_:)`. A multi-parameter unit of work is expressed by having
the entry `Function` close over what it needs, or by passing a `Tuple`/`List`.

**Edit.** `concurrency.md §1` — add a sentence under the interface table:
> `call`/`call(_:)` is the resume protocol and transfers at most one value; it is
> **not** the entry function's arity. An entry needing several inputs captures them
> in its closure or takes a single `Tuple`. This is why only `call` and `call(_:)`
> appear here while [`Function`](functions.md) has arbitrary-arity `call`.

### C5 — "root fiber" and "scheduler's root fiber" are the same fiber

**Decision.** They are identical. The main program *is* the root fiber, and the
scheduler drives that same fiber; there is no second one.

**Edit.** `concurrency.md §2` Implementation point 2 — change "runs inside the
scheduler's root fiber" to "runs inside **the** root fiber (§1) — the scheduler
drives that same fiber; there is not a second one — so `await` at top level is
legal and `Fiber.current` at top level is the root fiber."

### C6 — default arguments: defer, but track it where it can't be missed

**Decision (maintainer).** Do not decide now; **promote** it from the satellite
document into the canonical registry with a shipping-blocking marker.

**Edit.** `open-questions.md` — new numbered question:
> **Default arguments. ⚠ BLOCKS SHIPPING — decide before 1.0.** A call omitting a
> defaulted argument produces a *different* interned selector
> ([Messages & Selectors](messages-and-selectors.md)), so lookup misses. Options:
> (a) reject defaults entirely — idiom is overloaded selectors / an `Option`
> parameter; (b) arity-family expansion (compiler generates and forwards one
> selector per omitted suffix — combinatorial). Retrofitting after 1.0 is
> expensive. See the analysis formerly in `selectors.md §7.3`.

Delete `selectors.md §7.3` (now folded in).

### C7 — derived vs. hand-written accessors: a duplicate selector is an error

**Decision.** `@get`/`@set` *generate* an ordinary method-table entry. If a class
also hand-writes a method with the same selector, that is a **duplicate definition**
— the same compile-time error as defining any selector twice — not a silent
override. Attributes are shorthand for the boilerplate case; hand-write when you
need custom logic; you use exactly one per selector.

**Edit.** Fold into the provisional `selectors.md §4` (B8) and note in `classes.md
§3`: *"An `@get`/`@set`-derived accessor and a hand-written method of the same
selector collide at compile time (duplicate definition). Use one or the other."*
Gated behind the B8 open question since the whole attribute surface is provisional.

---

## D. Missing pieces — what to add

### D1 — define the Unit convention

**Decision.** Introduce **`()` — the empty tuple — as the Unit value**: a single,
boring, identity-comparable value meaning "no meaningful result." It reuses the
existing `Tuple` type (no new `Value` arm), and it is *categorically distinct* from
`None` (which means a *value* is absent), so side-effecting returns don't lie
through `Option`.

**Edits.**
1. `values-and-absence.md` — new short section:
   > **§5. Unit.** A method with no meaningful result returns `()`, the empty
   > `Tuple` — the Unit value. It is a real object (responds to `toString`, `==`),
   > distinct from `None`: `None` means *a value is absent*; `()` means *there was
   > no value to speak of*. `()` is a singleton, identity-comparable.
2. `system.md` — the `gc` row: "returns `()`" (resolves the A2/D1 punt). Audit the
   other class-side effects (`exit` doesn't return; `print`/`write` return their
   argument) for consistency.
3. Add `()` lexing note to `lexical-structure.md §7` (the tuple section): `()` is
   the empty tuple / Unit literal, unambiguous since `(` never begins a parameter
   list.

### D2 — place `Family` in the callable tower

**Decision.** `Family` is a **third concrete subclass of `Function`**, alongside
`Block` and `Method`. It is callable (that is its whole point — `f(to: p)` applies
it), so it must answer `Function`'s protocol; making it a `Function` subclass keeps
the invariant "`Function` is the root of everything callable" intact. Unlike
`Block`/`Method` it does **not** wrap a `ClosureObject` — it holds `(recv, name)` or
`(recv, selector)` and its `call` performs an ordinary send. The "one closure
representation" claim in `functions.md §4` is scoped to `Block`/`Method` and is
lightly reworded to say so.

**Edits.**
1. `object-model.md §4` "Callables & reflection" table — add:
   | `Family` | `Function` | U | A method reference (`obj::move`, `Point::#move(_,to,duration)`). Callable; its `call` is an ordinary send. [Symbols & Method References §3](selectors.md). |
2. `functions.md` — tower diagram gains `Family` as a third leaf under `Function`;
   §4's invariant sentence changes to *"`Block` and `Method` share one
   `ClosureObject`; `Family` is a third `Function` subtype that carries a receiver +
   name/selector instead of a closure, and applies by re-entering the send path."*
3. `selectors.md §3` — cross-link to `functions.md`/`object-model.md` for `Family`'s
   place in the class hierarchy (currently it defines `Family` only as a Rust enum).
4. `values-and-absence.md §1` value-types table — add a `Family` row (it is a
   surface value with a class, like `Block`).

### D3 — write the Collections spec (the largest hole)

**Decision.** Add a new spec part, `collections.md`, and define the **`Iterable`**
protocol it and the README example depend on. This is the single most load-bearing
gap: `filter`/`map`/`each` on `List` and the `for … in` desugaring are already used
as if specified.

**Edits.**
1. `README.md` reading order — insert *[Collections](collections.md) — `Iterable`,
   `List`, `Map`, `Set`, `Tuple`, `Range`, and the iteration protocol* before
   *Control Flow* (since `for`/`each` desugaring depends on it).
2. New `collections.md`, minimally covering:
   - **`Iterable` (abstract).** The one required primitive is `each(_:)`; `map`,
     `filter`, `reduce(_:_:)`, `find(_:)`, `count`, `toList`, `contains(_:)` are
     defined *in terms of* `each` on the abstract class, so any conformer gets them
     free. State explicitly whether conformance is a superclass relationship
     (`Iterable` in the tower, parallel to `Function`) or a structural contract —
     **recommend an abstract `Iterable` class** in the kernel, with `List`/`Map`/
     `Set`/`Range`/`Option` as subclasses/conformers, so `isA(Iterable)` is
     meaningful and `object-model.md`'s catalog can show the superclass.
   - **`List`** — indexing (`[](_:)`/`[]=(_:_:)` → `RangeError` out of bounds),
     `add(_:)`, `size`, slicing, the transform protocol return types.
   - **`Map`** — `[](_:)` → `Option`, `[]=(_:_:)`, key protocol (`hash`/`==`),
     iteration yielding `(k, v)` tuples.
   - **`Set`** — membership, `add`/`remove`, algebra (`union`/`intersection`).
   - **`Tuple`** — fixed arity, positional access, destructuring hook
     ([Q7](open-questions.md)), and its role as **Unit** when empty (D1).
   - **`Range`** — `a..b` (inclusive) vs `a...b` (exclusive) — *pin the
     inclusivity*, currently only hinted — `each`, `step(_:)`.
3. `values-and-absence.md §3.6` — the dangling "conforms to `Iterable` … protocol
   finalized with the iteration work" now resolves to `collections.md`; fix the
   cross-reference (it currently points at `open-questions.md`).
4. `control-flow.md §1` — `for (x in xs) === xs.each { x => … }` now cites
   `collections.md`'s `Iterable.each`.

### D4 — document `Number.parse`

**Decision.** `Number.parse(_:)` is a class-side method returning **`Result<Number,
Error>`** (expected, local failure = value channel, per `error-handling.md`). This
also lets `error-handling.md`'s `.attempt()` example use a genuinely *throwing*
operation instead of implying `parse` throws.

**Edits.**
1. `object-model.md §4` `Number` row — add class-side `parse(_:) -> Result`.
2. `error-handling.md §5` — either keep `Number.parse` and note it returns a
   `Result` directly (so the example becomes `Number.parse(input).map { … }`,
   dropping `.attempt()`), or swap in a genuinely-throwing operation for the
   `.attempt()` demonstration so the two channels aren't conflated. **Recommend**
   the latter: show `.attempt()` on something that actually `throw`s, and show
   `Number.parse` returning `Result` elsewhere, so each channel's example is honest.

---

## E. Already-tracked open questions

No action beyond the A2 consolidation — Q2 (Int/Float split), Q3 (external/internal
param names), Q4 (hierarchy mutability), Q7 (destructuring), Q8 (modules/imports),
Q10 (traits/mixins) stay open and correctly registered. The C3 flow-analysis item,
the B8 field-declaration-site item, and the C6 default-args item are the **new**
entries this pass adds to that registry.

---

## Execution order

The edits are mostly independent, but there is one dependency spine:

1. **B1 (selector form)** first — it changes strings quoted in B2, B4, B9, D2 and
   in ADR-0012's examples. Do it before anything that writes a selector.
2. **A1 + A2** (adopt `selectors.md`, consolidate registries) — creates the homes
   that B4/B6/C6/B8 fold their content into.
3. **B2 → A3 → A4** (True/False, then the ADR-0004 status/link fixes that depend on
   B2's outcome).
4. **D3 (collections)** — largest, independent, unblocks the README example and the
   `Iterable` references in `values-and-absence.md` / `control-flow.md`.
5. Everything else in any order.

Each edited spec file should get its `Status` line touched and, where a decision was
made, a one-line "Decided; see ADR-00xx / Open Questions Qn" so no future reader
hits the same "asserted-but-unbacked" ambiguity this audit found.
