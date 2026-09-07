# AUD-RUNTIME-S4 — Sequence views and byte contracts

## Checkpoint

Read-only runtime/library inspection at HEAD `1e22b57ff8cd0c61b12aeca4128671c9fabdef6e`, 2026-09-07. No diagnostic programs, Cargo checks, implementation fixes or commits were performed. The checkout contains substantial unrelated documentation reorganization; it was preserved. The source paths examined below had no working-tree diff where checked (`scalar/string.ph`, `heap/trace.rs`, `compiler/lib/loops.rs`). Original audit findings were not revalidated.

## L23 — String sequence views lack the compiler's iteration protocol

**Classification: confirmed source integration gap; source execution and current loader path not verified. Severity: Medium.**

`phalcom-core/core/universe/src/scalar/string.ph:305–308` constructs `StringByteSequence` and `StringCodePointSequence`. Their definitions at lines 312 and 339 have no explicit superclass and implement `size`, `at`, `each` and a private `nextCursor`. Neither definition implements `iterate(_)` or `iteratorValue(_)`.

A targeted search found no additional definitions or native bindings for these class names under `phalcom-core/src` or the Universe source tree. The Object library declaration has no iteration selectors. In contrast, `collections/bytes.ph:8` explicitly inherits Iterable and defines `iteratorValue`; `collections/iterable.ph` provides the generic cursor step. `compiler/lib/loops.rs::compile_for` sends `iterate(_)` and `iteratorValue(_)`; it does not translate a local `each` method or private `nextCursor` into the protocol.

The current core-classes document describes the views as ADR-0048-shaped. The implementation supplies a traversal convenience but lacks those protocol methods. A historical design discussion also calls them sub-iterable views; that design wording is context, not independent normative authority.

### Why this matters

For these definitions, a `for` loop cannot obtain elements through the compiler's normal two-selector protocol. It may be rejected earlier or reach missing-method behavior, depending on caller typing and loading. Neither exact outcome was executed here. Do not claim a reproduced runtime failure.

Simply inheriting the generic index cursor would also be insufficient for code points: that view's cursor is a UTF-8 byte offset, while `size` counts Unicode scalar values. Its existing private cursor advances by lead-byte length. Any future integration must preserve that distinction.

### Existing coverage and next check

`tests/fixtures/language/strings/string_bytes_codepoints.ph` checks sizes and calls `s.codePoints.each`; it does not use `for`, `iterate` or `iteratorValue`. This is a concrete coverage gap, not a claim that the fixture was run or failed.

Next: establish the current Universe loading path, then compare `each` with `for` for an empty string and a short ASCII/multibyte string. Record whether compilation rejects the view or dispatch misses the protocol. If iterable support is deliberately excluded, reconcile the documented sequence contract rather than silently assuming it.

## L24 — Bytes UTF-8 declaration disagrees with native results

**Classification: confirmed source declaration/body mismatch; downstream impact unresolved. Severity: Medium if published as a static return contract.**

`core/universe/src/collections/bytes.ph:17` declares `_$utf8 -> Bytes`, and the public `utf8` getter forwards to it at line 136. `primitive/bytes.rs::bytes_raw_utf8` validates UTF-8, allocates and returns a String on success, and returns None on invalid bytes. It does not return Bytes in either branch. The neighboring lossy decoder is declared String and returns String, providing a useful control.

This is distinct from L21's optional-wrapper concern: the successful heap object kind itself disagrees with the declaration. The primitive has no explicit return-type annotation in its macro declaration, so this pass has not established how the native surface checker or library type publication reconciles it.

Next: trace this one declaration through native source binding and exported return metadata, then inspect the public getter's inferred type and actual result. Keep any semantic inspection restricted to this runtime return contract. Determine whether a stale declaration causes an accepted wrong-type use, a bootstrap diagnostic, or is not consumed by the active path. No such outcome is claimed yet.

## Positive findings to preserve

### String view ownership

Both views assign the original string to `_string` in the constructor. They use ordinary instance storage. `heap/trace.rs::trace_object`, Instance arm, visits the class and every slot through `gc_obj_ref`. The inspected ownership path therefore retains the backing string. No new missing GC edge was found for these view fields. This is source evidence, not a GC stress test.

### UTF-8 cursor state

The code-point view starts at byte offset zero and advances by `_string.leadByteLen(cursor)`. The cursor helper is private, and String storage is valid UTF-8. For its existing `each`/`size` walks this is deliberate byte-boundary handling, not evidence of arbitrary malformed cursor admission. Empty input stops before decoding.

### Byte buffer copying and decoding

`bytes_raw_copy_into` checks `offset.checked_add(src_len)` before slicing, handles self-copy separately and uses a disjoint mutable heap-borrow helper for distinct buffers. `bytes_raw_slice` copies into a fresh buffer rather than retaining a parent view. Strict UTF-8 decoding uses `std::str::from_utf8`; invalid bytes return None. No unchecked copy-range or invalid-String construction defect was established in these paths.

## Related lead update

L21 now has caller evidence: `String.leadByteLen` and `codePointAt` compare accessor results with None, then directly use them in numeric expressions. They do not explicitly unwrap Some. This supports a bare-value convention in the library body, despite the declared Option<Int> type. It does not establish whether the activation boundary transforms results, whether these bodies type-check under current bootstrap rules, or which contract is intended. L21 remains unresolved; no public decoding failure was executed.

## Next audit slice

Resolve loader/publication and exact outcomes for L23/L24 before promoting either to an executed major issue. Keep numeric and raw-run probes paused unless specifically returning to their retained investigation plans. Provisional B and incomplete coverage remain unchanged.
