# 47. `::` method references (Open form, callable-only); amend the frozen floor +1 (`Family#doesNotUnderstand`)

- Status: Accepted
- Date: 2026-07-13
- Amends: [ADR-0019](0019-freeze-vm-blessed-primitive-floor.md) (the frozen floor);
  amendment precedent: [ADR-0023](0023-amend-floor-admit-hash-and-kernel-reflection.md),
  [ADR-0037](0037-amend-floor-admit-error-root.md),
  [ADR-0038](0038-amend-floor-admit-block-on-ensure.md),
  [ADR-0045](0045-module-import-relative-path-whole-module-binding.md)
- Related: [ADR-0012](0012-selector-signature-encoding-and-dispatch.md) (`encode_selector`/
  `decode_selector`, reused verbatim, not re-derived); `docs/spec/current/selectors.md` §3
  ("Method references (`::`)") + §3.1 (base-name index); `docs/spec/current/open-questions.md`
  Q14 (`Family` callable-only — reflective mirror deferred); `../../forge/units/U16/u16.md`
  (superseded original scope — see "Scope" below)

## Context

[selectors.md §3](../../spec/current/selectors.md#3-method-references-) specifies `::` producing a
callable **Family** value in two forms — **Open** (`obj::name`, selector built at call time)
and **Pinned** (`obj::#sel(...)`, selector fixed at compile time) — each either bound or
unbound. The `::` token (`Token::ColonColon`) already lexed; nothing else existed: no
`Expr::MethodRef`, no `Family` runtime representation, no base-name index.

**Scope, adjudicated at the U16 pre-edit gate (two real plan-vs-reality blockers):**

1. **Pinned form deferred.** `obj::#sel(...)` needs an atomic `#`-symbol-literal token
   (selectors.md §2), which does not exist (`lexer.rs`/`token.rs` are outside this unit's
   write-set). Deferred to a new prerequisite unit, **U-LEX-HASH**.
2. **Reflective surface deferred.** Q14 is already *resolved* in the spec ledger:
   `Family` is **callable-only**; a reflective mirror (`name`/`candidates`/`isBound`/…) is
   deferred to a future unified reflection unit alongside `Message`/`perform`/`respondsTo`.
   No `name`/`candidates`/`isBound` surface lands here.
3. **Unbound form deferred.** Both `obj::name` and `Type::name` bind a concrete receiver
   (the object, or the class object itself) — there is no first-argument-is-receiver form
   in this cut. `Family`'s `recv` field is a plain `Value`, not `Option<Value>`.

This unit — **U16-Open** — is exactly: Open-form `::`, bound-only, callable-only.

## Decision

**Grammar.** `Expr::MethodRef { receiver: Expr, name: String }` — a postfix `::name` added
to `parse_call`'s postfix loop alongside `.`/`(`/`{`. One grammar rule serves both surface
forms (`obj::name`, `Type::name`): `receiver` is whatever postfix chain preceded `::`.

**Runtime representation.** A new heap variant, `Object::Family { recv: Value, name: Symbol }`
(`heap.rs`) — **not** a `Value` arm (keeps `Value` minimal, ADR-0010), reached through
`Value::Obj` exactly as `Object::List`/`Object::Fiber` are. `name` is the bare base name, not
a full selector. `Value::class` routes it to a new `family_class`, sitting directly under
`Object`.

**A Family call *is* a send — no second dispatch mechanism.** Every bare-call syntax
(`f(...)`) already lowers to `MethodCall { method: "call", args, .. }` (existing parser
behavior, `parse_call`'s `LParen` arm — not new). Since `Family` defines no `call(...)`
selector at all, **every** call shape misses its method table and reaches exactly one new
primitive, `Family#doesNotUnderstand(_)` (`primitive/family.rs`): it decodes the missed
selector back into `(labels, kind)` via the existing `decode_selector` (ADR-0012, the exact
inverse of `encode_selector`), re-encodes with the family's own base `name` in place of the
literal `"call"`, and re-dispatches that real selector to `recv` via `VM::send_dynamic` — an
ordinary send, reusing U8's `send_dynamic`/`doesNotUnderstand` machinery unchanged. A
target-selector miss falls straight through `send_dynamic`'s own miss path to `recv`'s
`doesNotUnderstand(_)`, per selectors.md §3's error table.

**Base-name index (`base_names: HashMap<Symbol, Vec<Symbol>>`, selectors.md §3.1),
flattened through inheritance.** Added to `ClassObject` (`class.rs`). Built by
`VM::finalize_class_base_names`: merge the row's own directly-bound `methods`' base names
(via `decode_selector`) with its already-finalized superclass's index — a from-scratch
rebuild each time, not an accumulation, so a class reopen or a bootstrap re-finalization is
always correct. Two triggers populate it:

- **`Bytecode::FinalizeClass`** (new, no operand) — peeks the class value the compiler leaves
  on the stack at the tail of every class-body compile (`compiler/lib.rs`, right after the
  member loop, right before `DefineGlobal`), and finalizes both the class row and its
  metaclass (static methods live on the metaclass). Reopening re-runs it, idempotently.
- **`VM::finalize_all_core_base_names`** — a dependency-ordered pass over every kernel row,
  run once in `VM::new` right after `Universe::install_primitives`. Needed because several
  kernel rows (`Behavior`, `Metaclass`, `Message`, `Fiber`, …) carry **no** `.ph` class body
  in `core.ph` and would otherwise never reach `Bytecode::FinalizeClass` at all — without
  this pass, `::` against those receivers would spuriously report "empty family" for
  legitimate selectors. A later `.ph` reopen still re-finalizes its own row on top; the two
  triggers cannot disagree because the rebuild is always from-scratch.

**Reference-time empty-family check** (`Bytecode::MakeFamily`, new — the `Expr::MethodRef`
compile target): pops the receiver, resolves its class, and errors *at this point* — naming
the class, via `RuntimeError::Message` (no new error variant; matches the existing
`Undefined variable` miss style) — **iff** the class's base-name index has no entry for
`name` **and** the class's resolved `doesNotUnderstand(_)` is not `Object`'s default handler
(a hierarchy walk via the existing `lookup_method_in_hierarchy`, comparing method-handle
identity). A class with *any* `doesNotUnderstand` override — even a native one, like
`Module`'s — makes every family on it callable regardless of the base-name index; this is
exactly selectors.md §3's "Empty family, but class defines `doesNotUnderstand` → Not an
error" row, and it is why the call router above is itself framed as a `doesNotUnderstand`
primitive — reference-time and call-time both key off the same mechanism.

## Floor amendment

**+1 binding** (`floor-census.md` §2.16, 112 → 113): `Family#doesNotUnderstand(_)`
(`family_does_not_understand`). This is the *only* selector `Family` carries — there is
deliberately no `call(...)` binding of its own (the whole point is that every call shape
misses and lands on the router), and no reflective selector (Q14, deferred). Fails the
[ADR-0019](0019-freeze-vm-blessed-primitive-floor.md) §1 derivability test for the same
reason `Module#doesNotUnderstand` (ADR-0045) did: a `.ph` body cannot see the raw missed
`Message`'s selector text to decode-and-rebuild it, and `Family`'s own bound `recv`/`name`
are Rust-side heap fields with no `.ph`-reachable accessor (no reflective surface exists to
read them from Phalcom in this unit). Floor-carrying classes: **21 → 22**.

## Consequences

- **Unblocks nothing beyond itself; blocks nothing else.** `Family`'s call path is layered
  entirely over U8's `send_dynamic`/`doesNotUnderstand` and ADR-0012's `encode_selector`/
  `decode_selector` — no other unit's dispatch surface changes.
- **U-LEX-HASH is now a named prerequisite** for a follow-on `U16-Pinned` unit (Pinned
  `obj::#sel(...)`) and, independently, for the deferred `#IDENT` map-symbol-key item and
  future `perform`/reflection selector-symbol literals.
- **The reflective mirror (`Family#name`/`#candidates`/`#isBound`/`#receiver`) is explicitly
  open**, reserved for a future unified reflection unit alongside `Message`/`perform`/
  `respondsTo` (Q14 ruling stands unchanged by this ADR — merely realized as "callable-only
  for now", not re-litigated).
- **The base-name index is a hierarchy-wide, always-rebuilt cache**, not a mutable ledger:
  nothing in this unit invalidates it incrementally (there is no incremental path — every
  finalize is a from-scratch rebuild), so a hypothetical future `superclass=` mutation (U13,
  if ever ruled in) would need to explicitly re-finalize every affected subclass, exactly as
  it already must for method-lookup caches.
- `floor-census.md` (+1; 112 → 113), `tests/invariants.rs`
  (`floor_census_matches_installed_bindings`, `core_class_rows`) updated in the same change
  that installs the primitive (R-INV-0.1).

## Alternatives considered

- **A dedicated `MakeFamily`-adjacent call opcode instead of `doesNotUnderstand`.** Rejected
  — selectors.md §3 is explicit that "a family call *is* a send"; a bespoke opcode would be
  exactly the second dispatch mechanism the spec forbids, and would have to reimplement
  `send_dynamic`'s miss-forwarding/argument-marshalling machinery redundantly.
  `doesNotUnderstand` is the *existing* uniform "selector I don't define — try something
  else" hook (U8), and `Family` defining literally nothing else makes it land there for
  free.
- **Binding `call()`/`call(_:)`/… at every arity directly on `Family`** (mirroring
  `Function`/`Block`'s `call`/`call(_:)`/…/`call(_:_:_:_:)`). Rejected — those fixed arities
  cannot carry *labels* (`f(to: p, duration: 2)` needs the labels to rebuild the selector,
  not just a positional arg count), so it would need a labeled variant per arity per label
  combination — combinatorially unbounded. The `doesNotUnderstand` router is uniform across
  every label shape by construction.
- **Storing `base_names` on `Value` or computing it on demand at every `::` reference**
  (walk `methods` live, no cached index). Rejected — selectors.md §3.1 specifies the index as
  a finalize-time artifact serving three purposes (empty-family check, DNU candidate list, a
  future reflection surface); an on-demand walk would have to re-flatten the entire
  superclass chain on every single `::` reference, and would not exist yet for the
  candidate-list/reflection consumers a later unit adds.
- **Skipping the kernel bootstrap `finalize_all_core_base_names` pass, relying only on
  `core.ph` reopens.** Rejected — several kernel rows have no `.ph` body at all
  (`Behavior`, `Metaclass`, `Message`, `Fiber`, `Family` itself), so `::` against them would
  spuriously report every legitimate selector as an empty family.
