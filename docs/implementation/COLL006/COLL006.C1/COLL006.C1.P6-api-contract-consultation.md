# COLL006.C1.P6 — Public API Contract Consultation

## Request

Please settle the source-visible contracts for `MutableBag<T>`,
`IndexedDict<K,V>`, and `OrderedSet<T>` before P6 implements their public
Universe surfaces. Return explicit decisions for the questions below, identify
the normative owner for each decision, and state which tests should pin the
result. If a current specification must be added or amended, identify its
path/section and the required wording scope.

## Why consultation is needed

P6's C1 architecture records settle the storage direction and several key
invariants, but do not fully specify selector signatures, result shapes,
invalid-position behavior, constructor behavior, or all mutable-bag operations.
The current `docs/specs/` authority index has no Map/Set or P6 collection API
specification. Older P6 references to `docs/spec/current` are stale. These
choices are observable language behavior and must not be inferred by the
implementer.

The P6 Map/Set key-engine extraction is already complete and focused-tested.
Existing Map and Set behavior remains unchanged. The consultation concerns the
three new public collection APIs and their key/mutability contracts only.

## Fixed architecture and invariants

- All keyed lookup uses Telos `hash` and Telos `==`; Rust `Value` equality or
  object identity cannot replace those semantics.
- No heap borrow may cross a Telos callback. Reentrant reads remain possible;
  structural mutation of the collection being inspected remains rejected.
- `MutableBag<T>` stores only positive exact Telos-Int multiplicities, and
  `totalCount` equals the sum of counts.
- `IndexedDict<K,V>` is an ordered mapping backed by ordered entries plus a
  key-to-slot hash index. Key lookup and positional access are distinct.
  Replacing a value does not move its key; remove/reinsert appends unless the
  caller explicitly chooses a position.
- `OrderedSet<T>` is unique, ordered, and equality-sensitive with respect to
  order. Positional mutation is explicit.
- Do not change existing Map/Set selectors, return behavior, encounter order,
  equality, mutation locks, or numeric SameValueZero handling.
- The canonical type name is `OrderedSet<T>`; `SetList` is not introduced.

## Decisions requested

### 1. Type relationships, mutability, and key admission

For each new type, specify:

- whether it conforms to `Collection`, `Iterable`, or both;
- its element/entry iteration item type and iteration order;
- whether it is mutable/invariant;
- whether it is admissible as a `Map` key or `Set` element, and the semantic
  reason;
- whether equality is structural, identity-based, order-sensitive, or
  multiplicity-based; whether any `hash` contract is required or forbidden.

The existing architecture implies these are mutable and not Map/Set keys, but
that public/hashability rule needs an explicit owner and confirmation before
the central classifier is extended.

### 2. `MutableBag<T>`

Please define:

- construction forms and whether construction from repeated values increments
  multiplicity;
- selector names/signatures for total count, per-element count, insertion,
  removal, and membership;
- whether insert/remove accept one occurrence or an explicit multiplicity;
- absent-element removal behavior and return value;
- iteration behavior: distinct elements, repeated occurrences, or a separate
  entries/counts view;
- multiplicity and total-count domain, including what happens if a requested
  count is negative, zero, or cannot be represented;
- duplicate-construction and equality semantics.

### 3. `IndexedDict<K,V>`

The recorded operation names are `entryAt`, `keyAt`, `valueAt`, `indexOfKey`,
`move`, `insert(at:)`, and `removeAt`. Please settle:

- exact selector signatures and return shapes, including whether indexed reads
  return `Option` values;
- behavior for an absent key, an invalid index, a move to the current index,
  or a destination at `size`;
- whether `insert(at:)` rejects a key already present, replaces its value,
  or reports a duplicate error;
- constructor/from-iterable duplicate-key policy;
- the precise `move` argument model (source index/key and destination index)
  and whether destination indices are interpreted before or after removal;
- entry iteration type and order-sensitive equality definition;
- whether the public key lookup surface includes `get`, subscript, `contains`,
  and/or Map-like `insert(value,for:)` operations.

The existing invariant that replacement does not move a key and that integer
keys never mean positional slots remains fixed.

### 4. `OrderedSet<T>`

Please settle:

- selector signatures/results for insertion at a position, moving an existing
  element, removing by index, and finding an element's index;
- whether duplicate insertion is a no-op, error, or has a separate strict vs
  deduplicating constructor/factory policy;
- invalid-index and absent-element behavior;
- whether existing `Set.add/remove/includes/at` shapes are reused where
  sensible, while keeping `Set` semantics unchanged;
- ordered iteration and exact order-sensitive equality behavior.

### 5. Errors and diagnostics

Specify the error types/messages or the rule for choosing them for duplicate
keys/elements, invalid positions, invalid multiplicities, and count arithmetic.
State whether duplicate constructor errors stop before later input expressions
are evaluated, if construction is source-ordered.

## Expected adviser response

Return a compact decision list with:

1. fixed selector/signature/result/error contracts for all three types;
2. type relationships, iteration, equality, hashability, and key admission;
3. constructor and duplicate policies;
4. count and positional boundary rules;
5. normative spec/decision owner and required amendment;
6. minimum focused tests and any explicit non-goals.

P6 will record the approved decisions in the C1 checkpoint and amend its
implementation plan before adding the new public collection surfaces.

## T2 ruling — approved contract (2026-10-11)

The architect ruling was transmitted by the user from the attached T2 decision
and approves the recommendations in §3 of
`COLL006.C1.P6-repository-grounded-implementation-plan.md` as the P6 contract.
The user confirmed that repository-grounded plan as the implementation plan to
execute. The complete normative contract is now recorded in
[`docs/specs/collection-families.md`](../../../specs/collection-families.md).

The adopted rules include total-occurrence `MutableBag.size`, saturating bag
removal, expanded bag iteration, overwrite-in-place `IndexedDict`, exact
negative positional indexing with final-index move semantics, idempotent
`OrderedSet.add`, strict positional duplicate insertion, strict factories for
IndexedDict/OrderedSet, fail-fast source-order construction, the specified
existing error conventions, and the specified equality directions. The
existing Map/Set behavior and key admission rules remain protected.

One implementation-level question remained open in the plan's §4.6 and was
resolved directly by the user: equality takes GC-rooted snapshots of both
operands' logical contents before the first user equality callback. User
callbacks may mutate the live collections; those mutations commit normally,
but cannot affect the comparison already in progress. This policy is now part
of the normative specification and is mandatory for all three types.

The IndexedDict reentrant overwrite rule is also fixed: the outer operation
reads the value present at its own commit point as `Some(previousValue)`, then
commits the requested outer value; the outer write wins. If the outer operation
fails before commit, an already committed inner value overwrite is not rolled
back.

T2 architectural approval is complete. G2 remains open only until source-level
contract oracles have been added and observed failing before implementation.
