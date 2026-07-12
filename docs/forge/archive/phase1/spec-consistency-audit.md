# Phalcom Specification — Consistency & Completeness Audit

**Scope:** every file in `docs/spec/`, cross-checked against `docs/adr/0001`–`0016` and
`docs/spec/implementation-status.md`. This audit does **not** re-litigate the
spec-vs-implementation gap (that is `implementation-status.md`'s job, and it is
current and accurate). It asks a narrower question: **does the spec agree with
itself?** — and separately, **what does the spec simply never address?**

Findings are grouped: (A) structural/process issues in how the spec is organized,
(B) direct textual contradictions between two or more documents, (C) design points
asserted as settled whose mechanics are never worked out, (D) areas the spec is
silent on despite depending on them elsewhere, and (E) a roll-up of the
already-self-declared open questions, included for completeness.

---

## A. Structural issues — the spec disagrees with its own map

### A1. `selectors.md` is a second, disconnected canonical document

`docs/spec/README.md`'s **Reading order** table lists eleven parts. `selectors.md`
is not one of them — it exists on disk, headed `**Status:** Decided
(design-locked)`, but is reachable only by direct link from
`messages-and-selectors.md`'s "open question" cross-reference, not from the front
door. Every other "Decided" design point in this codebase is backed by an ADR
(`open-questions.md`'s resolved rows all cite one); `selectors.md` cites **none** —
there is no `ADR-00xx-selectors` despite the document unilaterally declaring itself
locked. The practical effect, confirmed by the notation conflict in B1 below: a
whole subsystem (`#` symbols, `::` method references, attribute macros) was
designed in isolation and never reconciled against the document it's supposed to
extend.

### A2. Three separate, non-communicating "open questions" registries

`docs/spec/open-questions.md` is presented as *the* place undecided points live
("Where a decision is not yet taken it lives in Open Questions rather than being
silently invented" — README §Reading order preamble). In practice there are at
least three registries that never cross-reference each other:

1. `open-questions.md` itself (11 numbered items, 4 resolved).
2. `selectors.md` §7 — five more open questions (default arguments, `Option`
   bootstrap, `ifTrue`/`ifFalse` chaining unsoundness, `var` defaulting to `None`,
   `Family` introspection), explicitly labeled "not part of this spec" — i.e.
   deliberately exiled from the canonical registry rather than merged into it.
3. Inline, table-embedded punts — e.g. `system.md`'s `gc` row: *"returns `nil`'s
   surface substitute — settle on the unit convention in Values & Absence"* is an
   open design question sitting inside an interface table, never promoted to
   `open-questions.md`, and — see D1 — never actually resolved in
   `values-and-absence.md` either, which the row's own cross-reference implies it
   should be.

One consequence of the fragmentation is directly contradictory framing of the same
question (B6 below): `open-questions.md` Q1 says `var` defaulting to `None` is
**RESOLVED** via ADR-0014; `selectors.md` §7.1 raises the identical question as
still philosophically unsettled, offering an alternative (`Uninit` sentinel) that
ADR-0014 already foreclosed.

### A3. ADR-0004 contradicts its own status line

`docs/adr/0004-boolean-as-abstract-bool-with-true-false.md` is headed
`- Status: Accepted`, but its own **Decision** section reads: *"**Recommendation
(pending approval):** adopt the abstract `Bool` + `True`/`False` model."* An ADR
cannot simultaneously be Accepted and pending approval; downstream documents
(`values-and-absence.md`, `object-model.md`) both cite it as settled authority
(B2), which compounds a genuinely unresolved question with a false sense of
closure.

### A4. Broken cross-reference

`values-and-absence.md` §3.1: `` [ADR-0004](../../../adr/) `` — the link target is the
ADR **directory index**, not `0004-boolean-as-abstract-bool-with-true-false.md`.
Minor in isolation, but it is the specific citation propping up the `True`/`False`
mirroring claim that turns out (B2) to be substantively contested, so the broken
link hides a live disagreement behind a dead reference.

---

## B. Direct contradictions

### B1. Two incompatible canonical selector grammars

`messages-and-selectors.md` §3 declares labels via **trailing-colon** syntax and
uses colon-suffixed strings as the selector's canonical form throughout:

> `move(to:, duration:) { ... }` → selector `move(to:duration:)`
> `add(_:_:)`, `+(_:)`, `name=(_:)`

`selectors.md` §1 declares an entirely different canonical string grammar —
**comma-separated, no colons** — and is explicit that this is *the* form:

```
selector  := name "(" [ slot { "," slot } ] ")"
slot      := "_" | label
```
> `move(_,to,duration)`, `add(_,_)`, `+(_)`, `size()`

These are not two notations for writing the same thing informally — `selectors.md`
frames its grammar as literally superseding `SignatureKind::Method(u8)` in the VM,
i.e. as the string that gets interned. `messages-and-selectors.md`'s colon form is
what ADR-0012 (label-encoded selectors) also uses in its own worked examples
(`move(to:duration:)`, `sum(_...)`). Nothing in either document says "declaration
syntax keeps colons, the interned string drops them" — that reconciliation exists
only in this session's memory of a prior audit pass, not in either spec file. As
written, a reader implementing `perform`, `#`-symbol interning, or error messages
naming "the selector" has two contradictory strings to choose from, and the
variadic-selector spelling compounds it: `messages-and-selectors.md` §4 interns
variadics as `sum(_...)`, a third slot-marker (`_...`) that `selectors.md`'s
`slot := "_" | label` grammar has no production for at all.

### B2. Are `True`/`False` real, surface-visible classes?

`values-and-absence.md` §3.1 states `Option`'s `Some`/`None` split "**exactly
mirrors** `Bool` / `True` / `False`," and `Some`/`None` are explicitly
surface-visible, constructible, dispatchable classes (`Some(v)` is "an ordinary
construction send"). By the stated mirroring, `True`/`False` should be the same:
real classes a user can name, subclass-check against, and dispatch on.

`object-model.md`'s Bool row says the opposite: *"Dispatch for `ifTrue`/`and`/`not`
may be realized internally via hidden `True`/`False` subclasses... **This is not
surface-visible: users see one class, `Bool`.**"*

ADR-0004 itself (A3) leans toward the `values-and-absence.md` reading — its
Consequences claim *"user code can meaningfully reason about `True`/`False` as
classes"* — which is hard to square with "not surface-visible." Three documents,
three different answers to whether `True`/`False` exist as things user code can
touch.

### B3. `Blocks §7`'s uncorrected claim vs. the `Function` amendment

`blocks.md` opens: *"`Block` is a real class...; **a method is a `Block` bound to
a class under a selector**."* `functions.md` carries an explicit callout: *"§7
said 'a method *is* a `Block`.' The precise relationship is: `Block` and `Method`
are **siblings** under the abstract `Function`... A `Method` is **not** a
`Block`."* `ADR-0006` (the decision backing the amendment) says of this fix:
*"Recorded inline in that file"* — referring to `blocks.md`. It was not: the
amendment note lives only in `functions.md`; `blocks.md`'s own text (read above,
verbatim, still present) makes the exact claim the amendment retracts. A reader
who opens `blocks.md` alone — which the README's reading order puts *before*
`functions.md` — gets the superseded model first and has no signal within that
document that it's wrong.

### B4. Does standalone `ifFalse(_:)` exist, and does chaining it work?

`object-model.md`'s Bool row lists exactly two selectors —`ifTrue(_:)` and the
paired `ifTrue(_:)ifFalse(_:)` — then adds the prose aside *"`ifTrue`/`ifFalse`
return `Option`,"* which presupposes a standalone `ifFalse(_:)` the selector list
never names. `control-flow.md`'s inliner section *does* list a standalone
`ifFalse(_:)` among the "sacred selectors." So: one document's own table and prose
disagree on whether `ifFalse(_:)` exists as an independent selector, and a second
document assumes it does.

This isn't cosmetic — `selectors.md` §7.2 flags the resulting semantic hole
directly: if both `ifTrue`/`ifFalse` independently return `Option`, then
`cond.ifTrue { a }.ifFalse { b }` sends `ifFalse` to an **`Option`**, not a `Bool`
— `Option`'s protocol (values-and-absence.md §3.3) has no `ifFalse`, so this is a
`doesNotUnderstand`. `control-flow.md`'s own worked desugaring for `if`/`else`
carefully avoids the trap by chaining `.ifNone` (a real `Option` selector) instead
of `.ifFalse` — but that sidesteps the general problem for one call site; nothing
in the spec tells a user why `cond.ifTrue{a}.ifFalse{b}`, which reads as the
obvious thing to write given `ifFalse(_:)` is advertised to exist, is wrong. This
is exactly the unresolved design flaw `selectors.md` names, silently patched
around in one place and left live everywhere else.

### B5. Set-literal alternative is foreclosed by a later document, uncorrected

`open-questions.md` Q6: *"Set literal. Currently `Set(...)`. `#{1, 2, 3}` remains
available if the ceremony becomes annoying."* `selectors.md` §2 (written later —
see A1, this document postdates and isn't cross-linked from `open-questions.md`)
reserves `#` as an atomic lexer token matching
`#[a-zA-Z_][a-zA-Z0-9_]*(\([^)]*\))?` — `#{` has no production in that grammar, so
`#{1, 2, 3}` cannot lex as anything coherent under the ratified selector-symbol
rule. The proposed escape hatch in `open-questions.md` is dead on arrival and
nothing updated that entry to say so.

### B6. `var` defaulting to `None` — resolved, then re-opened

Covered structurally in A2; concretely: `open-questions.md` Q1 is struck through
and marked **RESOLVED** by ADR-0014 (*"`var x` with no initializer reads as
`None`"*). `selectors.md` §7.1 restates the identical question as live, offering
the alternative of a VM-only `Uninit` sentinel that traps on read — i.e. proposing
to walk back a ratified ADR without acknowledging the ADR exists.

### B7. Interpolation syntax: stated as fact, flagged as assumption

`lexical-structure.md` §5 states interpolation unconditionally: *"Interpolation
uses `{expr}`."* No hedge, no forward reference. `open-questions.md` Q5 lists the
identical syntax as merely **assumed**, with two named alternatives
(`"${name}"`, `"\(name)"`) still on the table. A reader of `lexical-structure.md`
alone has no way to know this is not settled.

### B8. Field-declaration syntax: `_name` vs. `var x` inside a class body

`classes.md` §2 and `lexical-structure.md` §3 are unambiguous and mutually
consistent: fields are `_`-prefixed identifiers, **implicitly declared by
assignment** (`_name = name`), and are lexically a distinct token class — "a field
reference is only legal inside a class body." There is no field-declaration
keyword anywhere in this model.

`selectors.md` §4's worked example for the (admittedly "not yet specified")
attribute system uses a completely different, undeclared grammar:

```
@construct
class Point {
  var x
  var y
  @get var label
  @get @set var color
}
```

`var x` as a **field declaration inside a class body** is not legal syntax under
`classes.md`/`lexical-structure.md` — `var` there is the local-binding keyword from
ADR-0014, scoped to statement position, not class-body position; and `label`/`x`/
`y`/`color` lack the mandatory `_` prefix `lexical-structure.md` requires of every
field. Either this example is aspirational shorthand for a syntax nobody has
specified, or the attribute system silently proposes a second, parallel
field-declaration grammar that would need its own lexer/parser rules and directly
overlaps with the existing implicit-assignment model. Neither reading is stated;
the example is simply inconsistent with the two documents that define field syntax.

### B9. A third, undocumented symbol-literal spelling

`functions.md` §3 writes: `` let g = 3.methodFor(#"+(_)"); g.invokeOn(3, [4]) ``.
The token `#"+(_)"` — hash immediately followed by a **quoted string** — matches
neither `selectors.md`'s bare-identifier symbol grammar
(`#[a-zA-Z_][a-zA-Z0-9_]*(\(...\))?`) nor its separately-described operator-selector
branch (whose examples, `#+`, `#==`, `#[]`, are all unquoted). This is a third
spelling for "selector literal," present in a worked example, matching the
grammar of neither document that actually defines `#`-lexing.

---

## C. Settled-in-name, unworked-out mechanics

### C1. No mechanism for enforcing "abstract"

`object-model.md`'s catalog marks `Behavior`, `Function`, `Option`, and `Result` as
**A** (abstract) — "never the direct class of a live value." Nowhere does the spec
say what happens when user code tries anyway (`Behavior.new`, `Function.new`): is
it a compile-time error (there's a `construct` mechanism per class — does an
abstract class simply define none, making `new`/`construct` a plain
`doesNotUnderstand`?), a dedicated `AbstractClassError`, or something else? The
"abstract" property is asserted, never enforced-by-rule.

### C2. Cross-class field-slot layout is only specified for one class at a time

`classes.md` §2 and ADR-0011 both describe, precisely, how a **single** class's
field set becomes a fixed slot vector. Neither addresses the multi-class case that
inheritance forces: fields are "private to the declaring class... a subclass that
writes `_name` gets its own new slot" — meaning an instance of a subclass
physically carries the superclass's slots *and* its own, non-overlapping, even
under a shared field name. The composition rule — are superclass slots a fixed
prefix, is offset assignment order superclass-first, how does a method compiled
against the *defining* class's slot indices stay correct when invoked (via
inheritance, unchanged) on a subclass instance whose slot array is longer/shaped
differently — is asserted to work ("offsets are permanently stable") but the
actual layout algorithm across a hierarchy is never written down.

### C3. Read-before-write is whole-class, not flow-sensitive

`classes.md` §2: *"Reading a field never assigned in **any method of the class**
is rejected at compile time"* — this is a class-wide existence check (does `_foo`
appear as an assignment target anywhere in the class?), not a per-method,
per-control-flow-path definite-assignment analysis. It genuinely catches the
motivating typo example (`_naem` never appears as an LHS anywhere → compile
error). It does **not** catch — and the spec doesn't note that it doesn't catch —
the more common real bug: a field assigned in constructor A but read in a method
reachable from an instance built by constructor B, which never assigned it. That
case passes the compile-time gate (the field *is* assigned somewhere in the class)
and silently reads as `None` at runtime, which is precisely the failure mode
`Values & Absence` frames as the *deliberate* fallback for legitimately-optional
fields — but here it would be masking an initialization-order bug the spec's own
stated goal ("catching the typo class") implies should be caught.

### C4. `Fiber` entry arity is capped where `Function`'s isn't

`Fiber`'s interface table (`concurrency.md` §1) lists only `call` / `call(_:)` —
zero or one argument. `Function`'s general call protocol (`functions.md` §1)
supports arbitrary declared arity: `call`, `call(_:)`, `call(_:_:)`, etc. Since a
`Fiber`'s entry is "the `Function` the fiber runs when first resumed" with no
stated arity restriction on *what* function may be wrapped, it's unclear whether a
2+-parameter `Function` used as fiber entry is simply illegal, or whether `Fiber`
is missing `call(_:_:)`/`call(_:_:_:)` overloads by omission.

### C5. "Root fiber" vs. "the scheduler's root fiber" — one thing or two?

`concurrency.md` §1 defines *the* root fiber as "the main program; it is
`suspended` only while a callee fiber runs." §2's Implementation notes say *"The
top-level program runs inside **the scheduler's root fiber**, so `await` at top
level is legal."* Whether "the scheduler's root fiber" is the same fiber as §1's
"the root fiber," or an additional fiber the scheduler owns beneath/around it, is
never stated. If they're the same, §2 should say so explicitly (a reader has no
way to confirm identity from the text); if they're different, the relationship —
and what "current" (`Fiber.current`) resolves to when both exist — is unspecified.

### C6. Default arguments: "decide before shipping," but absent from the tracked registry

`selectors.md` §7.3 flags this in the strongest terms available in either
document: *"Largely incompatible with selector-identity dispatch... **Decide
before shipping — retrofitting is expensive.**"* This urgency marker exists
nowhere in `open-questions.md`, the document whose entire purpose is to be the
place undecided-and-consequential points are tracked so they aren't "silently
invented." A design point its own author calls shipping-blocking is one
misdirected link away from being missed entirely.

### C7. Attribute-derived accessors vs. hand-written accessors: no precedence rule

`selectors.md` §4 proposes `@get`/`@set` deriving accessor methods
(`label()`/`label=(_:)`-shaped) from field declarations. `classes.md` §3 already
specifies hand-written accessors as the idiomatic pattern (`name => _name`). Both
target the exact same selector shape for the exact same purpose. Neither document
says what happens if a class uses both — hand-writes `name => _name` *and* tags
the field `@get` — nor whether the attribute system is meant to *replace*
hand-written accessors as the recommended style going forward or merely offer a
shorthand for the boilerplate case.

---

## D. Not considered — depended on, never specified

### D1. No "unit"/void convention, despite the spec needing one twice

There is no `Unit`, `Void`, or documented "what does a side-effecting method
return" convention anywhere in the eleven spec parts. It surfaces as an open gap
in exactly the place that needs it and gives up: `system.md`'s `gc` row punts to
`values-and-absence.md` for "the unit convention," and `values-and-absence.md`
never defines one. Given `nil` is banned from the surface (Invariant 4) and
`Option`/`None` means "the *value itself* is absent" (a category distinct from "no
return value was ever a concept here"), reusing `None` for void returns would be a
type-level lie. The spec has, in effect, discovered it needs an answer and left
the row saying so, unresolved.

### D2. `Family` (`::`) is invisible to the Object Model's own catalog and tower

`selectors.md` §3 introduces `Family` as a first-class **callable value** —"a
callable value" is its own description — with `Open`/`Pinned` variants, produced
by a new operator (`::`), and (implicitly, since it's called with `f(to: p,
duration: 2)`) something that must answer `call`/apply sends. `functions.md`'s
entire subject is "the callable tower": `Function` (abstract) → `Block`, `Method`
(siblings). `Family` is not mentioned in that document, not in the tower diagram,
not in `object-model.md`'s "Callables & reflection" catalog table, and not in the
core Value-representation table (§3) that enumerates every surface type. A reader
who only reads the object-model and functions specs — the documents that exist
specifically to be exhaustive about "what is callable in this language" — would
never learn `Family` exists. Whether `Family` *is* a `Function` subtype, a sibling
class outside the tower entirely, or something else structurally, is unaddressed.

### D3. There is no Collections spec at all

Every other value category with user-visible protocol gets a dedicated document —
`Option`/`Result` (values-and-absence.md), `Block`/`Function`/`Method`
(blocks.md, functions.md), `Fiber`/`Future` (concurrency.md). `List`, `Map`,
`Set`, `Tuple`, and `Range` get one row each in `object-model.md`'s catalog table
— a class name, a superclass, and a one-line "Role" description ("Growable
ordered sequence," "Hash map (keys use `hash`/`==`)") — and nothing else. No
document specifies `List`'s mutation/indexing protocol, `Map`'s key/value
iteration, `Set`'s membership operations, or `Range`'s stepping/inclusivity rules.

This gap is not hypothetical or low-stakes — the spec's **own flagship example**,
in `README.md`, depends on undocumented collection protocol:

```phalcom
people.filter(p => p.isAdult)
      .map(p => p.name)
      .each { n => System.print(n) }
```

`filter(_:)`, `map(_:)`, and `each(_:)` on `List` are used here as if
self-evidently defined, but no spec document declares their selectors, semantics,
or return types. `values-and-absence.md` §3.6 references an `Iterable`
**protocol** by name ("`Option` conforms to `Iterable`... protocol finalized with
the iteration work") as though it's an established, cross-referenceable construct
— it is not defined anywhere in the eleven parts; the phrase forward-references
work that, per this audit, doesn't exist as a document. `control-flow.md`'s
`for (x in xs) === xs.each { x => ... }` desugaring likewise assumes `each` is
universally available across "anything iterable" without that protocol ever being
named as a class-hierarchy concept (a mixin? an implicit structural contract? Is
there an `Iterable` abstract class in the tower, parallel to `Function`? Nothing
says).

### D4. `Number.parse` — used, never declared

`error-handling.md`'s own worked example: `` { Number.parse(input) }.attempt() ``.
`object-model.md`'s `Number` row lists only "Arithmetic, comparison, `toString`."
No class-side `parse(_:)` (or its failure mode — presumably `Result`-returning,
given the example wraps it in `.attempt()`, but `.attempt()` per
`error-handling.md` §5 captures a **`throw`**, so `Number.parse` failing must
raise rather than return `Err` directly — an inference the reader has to make
unaided) is documented anywhere.

---

## E. Already self-declared open questions (for completeness)

These are correctly tracked in `open-questions.md` and are not "gaps" this audit
is newly surfacing — included here only so the document above reads as a complete
picture rather than omitting known-knowns:

| # | Question | Status |
|---|----------|--------|
| Q2 | `Number`: single type vs. `Int`/`Float` split | Open (f64-as-single-type side settled per ADR-0005; surface split undecided) |
| Q3 | External vs. internal parameter names (Swift-style `move(to target:)`) | Open (ADR-0012 reserves the field; policy undecided) |
| Q4 | Class-hierarchy mutability (`Test.superclass =` at runtime) | Open |
| Q7 | Destructuring (`let (a, b) = point`, `let [first, *rest] = list`) | Not yet specified |
| Q8 | Modules / imports semantics | Token exists; unspecified |
| Q10 | Traits / mixins / multiple inheritance | Unspecified; single inheritance is the current invariant |

---

## Summary — what to do with this

The pattern across nearly every finding is the same: **the spec suite grew by
accretion, and later documents (`selectors.md` in particular) were not merged back
into the earlier ones they extend or partially supersede.** Nothing found here
suggests a design flaw in the language itself — the underlying decisions (label
identity, Option-not-nil, the metaclass parallel rule, layered error handling) are
coherent and well-reasoned in isolation. The failure mode is purely editorial:
two documents independently describing the same concept and drifting apart (B1,
B2, B3, B4), a decision marked resolved in one place and re-litigated in another
(B6), and — the largest single gap — a whole layer of the language (collections,
`Family`) that real spec prose and the README's own example already depend on but
that no document actually specifies (D2, D3).

Recommended immediate actions, roughly in priority order:
1. Fold `selectors.md` into the README reading order and reconcile its comma-form
   grammar against `messages-and-selectors.md`'s colon-form (B1) — this blocks
   every downstream selector-related decision.
2. Resolve and record, via ADR, whether `True`/`False` are surface-visible (B2) —
   ADR-0004 needs its status/body contradiction fixed first (A3).
3. Write the missing Collections spec (D3) — it's load-bearing for the language's
   own canonical example and is the single largest hole.
4. Merge the three open-questions registries (A2) into `open-questions.md`,
   including the shipping-blocking default-arguments question (C6).
5. Fix `blocks.md` §7 in place rather than relying on a forward amendment note in
   a different document (B3).
