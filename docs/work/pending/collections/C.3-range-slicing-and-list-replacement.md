# Spec C.3 — Range Slicing and List Slice Replacement

Status: implementation specification. Requires A.3, C.1, and C.2. B.1 is transitively required through C.1. Does not require Spec D/E/F.

## 1. Mission

Make Range values usable as contiguous slice descriptors for finite indexed sequences while preserving the strict distinction between element coordinates and slice boundaries.

Read slicing:

```phalcom
list[2..5]      // List copy
list[2..=5]
tuple[2..]      // Tuple copy, labels preserved
bytes[..5]      // Bytes copy
```

must clamp out-of-range boundaries rather than raise for ordinary bound overflow.

List additionally supports variable-length contiguous replacement:

```phalcom
list[2..4] = replacements
```

and recoverable method form:

```phalcom
list.replace(range, with: replacements)
// Result<Unit, SliceError>
```

Source-level slice assignment is strict: an invalid/malformed slice raises. Its expression value remains the original RHS via C.1's assignment machinery.

## 2. Dispatch model: one bracket selector, runtime domain branch

Phalcom dispatch is selector-based, not type-based.

Both:

```phalcom
list[2]
list[2..5]
```

send the same one-positional-argument subscript selector.

Therefore do not attempt to define a separate selector overload based on argument type.

The collection's single `[_]` implementation must branch on the runtime argument domain:

```text
Range -> slice path
otherwise -> C.1 element-index path
```

For Tuple, the non-Range branch itself already distinguishes index versus Symbol label.

For List setter `[_ ,put:]`:

```text
Range -> slice replacement
index -> single-element assignment
```

Bytes does not gain slice assignment in this spec. Tuple remains immutable.

## 3. Slice-bound semantics

For sequence length `n`, slice coordinates are boundaries, not element-only indices.

Canonical normalized boundary interval:

```text
0 <= boundary <= n
```

Unlike strict element lookup, out-of-range slice bounds clamp.

### 3.1 Lower bound

For a present lower bound `x`:

```text
if x >= 0: candidate = x
else:      candidate = n + x
start = clamp(candidate, 0, n)
```

Omitted lower bound:

```text
start = 0
```

### 3.2 Exclusive upper bound

For `..b` / `a..b`:

```text
if b >= 0: candidate = b
else:      candidate = n + b
endExclusive = clamp(candidate, 0, n)
```

Omitted upper bound:

```text
endExclusive = n
```

### 3.3 Inclusive upper bound

For `..=b` / `a..=b`, include element at upper index `b` when it denotes a valid element after normalization.

Convert conceptually to an exclusive boundary after end-relative normalization:

```text
candidateElement = (b >= 0) ? b : n + b
candidateExclusive = candidateElement + 1
endExclusive = clamp(candidateExclusive, 0, n)
```

Use checked/widened arithmetic so an enormous compatibility Number does not overflow before clamping.

Examples at size 5:

```text
[..=0]   -> [0,1)
[..=-1]  -> [0,5)
[..=99]  -> [0,5)
[..=-99] -> [0,0)
```

### 3.4 Empty/reversed result

Equal normalized bounds select no elements:

```phalcom
seq[3..3]
```

Descending/reversed Range semantics are explicitly deferred by the Range specification. Do not make slicing the place that silently defines them.

For this first implementation, if normalized `start > endExclusive`:

```text
read slice -> corresponding empty collection-family value
List replacement -> treat removed span as empty at start (insertion point = start)
```

This is a consumer-local “no selected ascending elements” rule, not a claim that the Range itself has descending semantics. Document it in tests so later Progression work does not reinterpret it accidentally.

## 4. Centralize Range-to-slice normalization in core semantics

Do not copy the normalization formula independently into List, Tuple, and Bytes.

Preferred architecture: define one internal-ish `.ph` helper on Range, e.g. repository-style spelling:

```text
sliceBounds_(size)
    -> Result<Tuple(start,endExclusive), SliceError>
```

The exact private-ish name may differ.

Why `.ph` rather than another primitive:

- C.2 already exposes `lower_` / `upper_` as Option and `upperInclusive_`;
- arithmetic/clamping is expressible in language code;
- keeping normalization above the primitive floor prevents three collection-specific native copies;
- the consumer still supplies its own `size`, preserving the semantic fact that Range is a bound structure interpreted by consumers.

The returned bounds Tuple is always arity two, so it never normalizes to Unit.

### 4.1 Validation

For sequence slicing, each present endpoint must be an integer coordinate.

Until numeric tower landing, apply the same C.1 compatibility rule: finite integral Number. Once Int/Float split lands, require Int.

A malformed endpoint produces `Err(SliceError(...))` from `sliceBounds_` rather than a raw panic.

An explicitly supplied surface None endpoint is present and malformed for sequence slicing; it is not treated as an omitted bound.

### 4.2 `SliceError`

Add `SliceError < Error` through the same Error convention as `IndexError`/`KeyError`.

Exact structured payload is deferred. Preserve at minimum a useful message and, where practical, the original Range/bad endpoint.

## 5. List read slicing — derive in `.ph`, no new raw primitive

List can construct a fresh List through existing allocation/add operations.

For Range argument:

1. call `range.sliceBounds_(self.size)`;
2. strict-unpack the Result, raising its `SliceError` on Err;
3. allocate `List.new()`;
4. loop `i = start; i < endExclusive; i += 1`;
5. append raw/list element at canonical nonnegative index;
6. return the fresh List.

Do not route through Range iteration. A one-sided `2..` slice is a finite operation because the consumer clamps to its own length; it is not eager exhaustion of the unbounded Range.

The result never aliases source storage.

An empty slice returns a new empty List.

## 6. Tuple read slicing — native product reconstruction

Tuple slicing is not just value copying: labels for selected labeled components must survive.

Add the smallest raw primitive necessary, conceptually:

```text
Tuple::slice_(start, endExclusive)
```

Arguments are already normalized canonical nonnegative boundaries from `sliceBounds_`; this primitive does not re-interpret Range.

### 6.1 Total product order

Slice over:

```text
[positionals..., labeled-values...]
```

### 6.2 Preserve labels

Suppose:

```phalcom
(
    "request",
    10,
    timeout: 5,
    retries: 2,
)
```

Then total indices are:

```text
0 "request" positional
1 10        positional
2 5         label #timeout
3 2         label #retries
```

`slice_(1,3)` must reconstruct:

```phalcom
(
    10,
    timeout: 5,
)
```

not `(10, 5)`.

For selected range:

- selected values before original `positional_len` remain positional;
- selected values at/after original `positional_len` carry their corresponding labels;
- encounter order is unchanged.

Call A.2/A.3's canonical `finish_tuple` builder. Never allocate an empty TupleObject directly.

Consequently:

```phalcom
tuple[3..3]
```

returns Unit.

### 6.3 No mutation

Do not add Tuple slice setter.

## 7. Bytes read slicing — reuse existing bulk primitive

HEAD already has:

```text
Bytes::slice_(start,end)
```

which copies a normalized half-open span to a fresh Bytes.

Keep that bulk primitive. C.3 moves clamping/negative/inclusive interpretation above it through `Range.sliceBounds_`.

After normalization, arguments must satisfy:

```text
0 <= start <= end <= size
```

If the consumer-local reversed rule produced `start > rawEnd`, pass `start,start` for the empty slice.

Do not modify `Bytes::slice_` to understand Range objects or one-sided semantics; keep it a simple bulk copy substrate.

## 8. Public bracket methods

### 8.1 List

Conceptual shape:

```phalcom
[key] {
    if (key is Range) {
        return self.sliceByRange_(key)
    }
    return self.strictElementAt_(key)
}
```

Use actual Phalcom `is` syntax/methods current on implementation HEAD.

### 8.2 Tuple

Range branch first, then C.1 index/Symbol branch.

A Range is not treated as a label or numeric index.

### 8.3 Bytes

Range branch first, otherwise C.1 element path.

## 9. List in-place slice replacement primitive

The current List floor cannot express variable-length in-place splice while preserving receiver identity using only `set_` and `push_`. Add one raw primitive:

```text
replaceSlice_(start, endExclusive, replacements)
```

where `replacements` in this phase is a List.

### 9.1 Why List-only replacement source in C

The ratified core spec names `replacements` but does not yet fix the full accepted source capability. Accepting arbitrary Iterable here would immediately require:

- eager materialization rules;
- behavior for provably unbounded sources;
- re-entrant user iteration while mutating the destination;
- Spec E boundedness/exhaustion policy.

Do not drag those decisions into slicing.

C.3's executable minimum accepts List replacement values, matching the ratified examples. General finite Iterable replacement may widen later without changing selector identity.

Document this as a temporary capability restriction, not as a claim that the final language forever accepts only List.

### 9.2 Native algorithm

In `phalcom-core/src/primitive/list.rs` or a focused helper:

1. validate receiver List and replacement List handles;
2. validate already-normalized `start <= end <= receiver.len`;
3. snapshot replacement elements **before** taking the mutable destination borrow;
4. this snapshot is mandatory when `replacements` is the same List as receiver;
5. use `Vec::splice(start..end, replacement_snapshot)` or equivalent stable algorithm;
6. return canonical Unit.

Do not hold heap borrows across user code; this primitive executes no user code at all.

Relative order before `start` and after `end` remains unchanged.

Replacement may shrink, preserve, or grow length.

## 10. Recoverable `List.replace(range, with:)`

Surface signature:

```phalcom
list.replace(range, with: replacements)
// Result<Unit, SliceError>
```

Behavior:

1. if first argument is not Range -> `Err(SliceError)` rather than mutating;
2. normalize through `range.sliceBounds_(size)`;
3. if normalization returns Err -> return that Err;
4. if replacement source is unsupported under C's temporary List-only rule -> return `Err(SliceError)` or the repository's ordinary argument/type error convention; use one stable choice and test it;
5. call `replaceSlice_` with canonical bounds;
6. return `Ok(())`.

Use canonical Unit in Ok.

Do not raise merely because the slice bounds lie beyond sequence ends; clamping makes them valid.

## 11. Strict source-level slice assignment

List's existing `[_,put:]` method must branch by first argument:

```text
index -> C.1 single element setter
Range -> strict slice replacement
```

For Range:

- call recoverable replacement logic or shared helper;
- if Err, raise contained SliceError;
- if Ok, setter method may return Unit/receiver/anything consistent with internal convention because C.1 compiler lowering discards it;
- source assignment expression itself evaluates to the original RHS.

Do not special-case assignment result inside List.

Example:

```phalcom
let replacement = [9, 10, 11]
let result = xs[1..3] = replacement
result === replacement
```

must be true.

## 12. Evaluation order

For read:

```phalcom
receiver()[lower()..upper()]
```

ordinary language evaluation means:

```text
receiver
lower endpoint
upper endpoint
subscript dispatch
```

For write:

```phalcom
receiver()[lower()..upper()] = rhs()
```

C.1 requires:

```text
receiver
lower
upper
RHS
setter dispatch
```

exactly once each.

Range construction happens as the subscript argument expression before RHS.

## 13. Family-preserving results

Pin these outcomes:

```text
List Range slice  -> fresh List
Bytes Range slice -> fresh Bytes
Tuple Range slice -> Tuple semantic value
                    except zero components -> Unit
```

No zero-copy views.

No Map slicing.

No Record slicing.

## 14. Test matrix

For each List/Tuple/Bytes length 0,1,5 cover:

### 14.1 Bounds

```text
[..]
[0..n]
[0..=n-1] where nonempty
[2..]
[..2]
[..=2]
[-3..-1]
[..=-1]
[-100..100]
[n..n]
[n+10..]
[..-100]
start > end -> empty consumer result
```

### 14.2 Inclusive/exclusive distinction

At size 5:

```text
[1..3]   -> indices 1,2
[1..=3]  -> indices 1,2,3
```

### 14.3 Tuple labels

Use mixed-lane Tuple and verify slices before, across, and entirely inside labeled lane preserve exact labels/order.

Verify empty Tuple slice is Unit and is not an empty Tuple heap object.

### 14.4 Explicit None endpoint

A Range with explicit endpoint None must fail slicing as malformed rather than being treated as omitted.

### 14.5 List replacement

Cover shrink/equal/grow, insertion through empty span, replace whole List, replace with empty List, self-replacement snapshot, negative/clamped bounds, and no mutation on normalization/type error.

### 14.6 Assignment result/order

Verify original RHS identity and evaluation order with side-effecting receiver/endpoints/RHS.

## 15. Primitive floor/governance

Planned new underivable bindings:

```text
Tuple::slice_(_,_)                 +1
List::replaceSlice_(_,_,_)         +1
```

Expected C.3 maximum delta: +2.

Justification:

- Tuple dynamic reconstruction with preserved labels and Unit canonicalization cannot be expressed through the exposed immutable product observations without a dynamic constructor primitive; routing through List would lose labels and regress Spec A architecture.
- List variable-length in-place splice preserving receiver identity cannot be expressed with the current set/push floor.

Do **not** add List read `slice_`; it is derivable in `.ph`.

Do **not** add another Bytes slice primitive; reuse the existing one.

Update floor census and scoped governance record with exact counts against implementation HEAD.

## 16. Deliberate deferrals

C.3 does not decide:

- stepped slicing;
- generic Iterable replacement sources;
- unbounded replacement materialization;
- descending Range traversal;
- Range/Progression iteration semantics;
- slice views;
- Map/Record slicing;
- Bytes slice assignment;
- Tuple mutation;
- final SliceError payload schema.

## 17. Completion checklist

C.3 is complete only when:

- Range arguments route to slicing without type-based dispatch machinery;
- normalization is shared, negative/end-relative, inclusive-aware, and clamped;
- explicit None bounds are distinct from omitted bounds;
- List/Bytes return fresh same-family copies;
- Tuple preserves labels and canonicalizes empty slice to Unit;
- List replacement can shrink/grow and handles self-source safely;
- recoverable `replace` returns Result and source syntax raises on its errors;
- source slice assignment returns original RHS;
- only the justified +2-or-less primitive delta is admitted/documented;
- `./scripts/verify.sh --full` passes.
