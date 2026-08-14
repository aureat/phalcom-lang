# Family Test Refresh Design

## Goal

Bring `phalcom-core/tests/lang/family/` in line with
`docs/spec/callables/family.md` §§1–5 and
`docs/spec/current/syntax/expressions.md` §4. Those sections are authoritative:
`::` creates a bound Family, exact references retain selector identity, pattern
references perform live matching, and construction never rejects an absent
selector. Older Open/Pinned wording in `docs/spec/current/selectors.md` is not
used as test authority.

## Scope

Only language corpus fixtures and their expected output files change. Runtime,
compiler, and test-harness code stay untouched. If a fixture exposes a runtime
or compiler defect rather than a stale expectation, stop and report it instead
of widening this task.

### Existing fixtures to adapt

- Rewrite Open/Pinned comments and scenarios as exact or pattern references.
- Keep useful exact method, overload, class-side, and inherited-method cases,
  but use current selector forms and call semantics.
- Change the empty-family case from reference-time failure to call-time
  `doesNotUnderstand` failure.
- Remove or replace the obsolete bare `::#name` parser case and pinned-only
  empty-family case.
- Retain an exact argument-count mismatch as a negative call-shape test.

### New coverage

Add focused cases for:

1. exact getter and exact nullary method references, using `b::value` with
   `family.get()` versus `b::value()` with `family()`, proving their distinct
   selector shapes;
2. named structural-pattern routing using `b::value(...)` across nullary and
   unary `value` methods, with calls `family()` and `family(4)` selecting the
   corresponding current routes;
3. exact setter Family invocation using `b::value=(put)` followed by
   `family.set(8)`;
4. an exact Family whose selector is absent at construction, then reaches
   ordinary target `doesNotUnderstand` at call time. The expected diagnostic
   substring is `does not understand 'missing'`;
5. exact call-shape rejection, with an exact unary Family called without its
   required argument. The expected diagnostic substring is ``exact family
   `value` does not accept``; and
6. receiver evaluation once when constructing an escaping Family, using a
   factory that prints an observable creation marker and calling the Family
   twice. The marker must appear once.

The existing Rust integration target already covers live method-table
replacement and immutable reflection snapshots; this language corpus refresh
does not duplicate those implementation-level tests.

Each case should assert observable language behavior through `.expected`
sidecars. The adapted DNU-backed fixture keeps its custom output (`caught
typo`); the default DNU case uses the stable `does not understand 'missing'`
substring above. The exact-shape negative uses the stable ``exact family
`value` does not accept`` substring above.

## Verification

Run `cargo test -p phalcom-core --test lang family family_negative --
--nocapture`, then `cargo test -p phalcom-core --test lang`, and finally
`cargo test -p phalcom-core --test integration`. Classify any unrelated
baseline failures separately from fixture failures. Run `graphify update .`
after modifications so the repository graph reflects the changed corpus.
