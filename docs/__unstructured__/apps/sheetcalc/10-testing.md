# SheetCalc — Testing

Part of the [SheetCalc specification](README.md).
Grounded in [00-language-findings.md](00-language-findings.md) §2, §6, §7, §9;
[02-value-model.md](02-value-model.md) §6.

This is the payoff for Phalcom's reflection layer. `perform`, `methodFor`,
`Behavior#methods`, `Method#attributesOfType`, and `Method#invokeOn` (all
verified working — findings §9) are exactly the primitives an
attribute-driven test framework needs, and nothing here required a single
Rust change. All code below uses ordinary double-quoted Phalcom strings —
see [09-rendering.md](09-rendering.md)'s opening note on why single quotes do
not belong in this document either.

## 1. The `Test` attribute

```phalcom
// test/framework.ph
class Test is Attribute {
  @constructor
  new(desc) { _desc = desc }

  desc => _desc
}
```

`Attribute` and `Error` are kernel classes (core.ph), visible in every module
without an `import` (modules.md §7), so `test/framework.ph` needs no imports
of its own.

A test suite tags its test methods:

```phalcom
class ValueFormatSuite is TestCase {
  @Test("0.1 + 0.2 formats as 0.3, not the raw float noise")
  testFloatNoise() {
    self.assertEqual("0.3", Num.format(0.1 + 0.2))
  }
}
```

Test bodies use the explicit 0-arity **method** form, `name() { ... }`, not
the getter form `name => expr`. The findings §9 probe that established
attributes work used a getter (`testAdd => 1 + 1`) because it was a
one-expression probe; a real test needs several statements (multiple
`assert*` calls), and `=>` only ever holds a single expression (`call` vs
`call()` are different signature kinds — findings §10). Attributes attach to
the method declaration itself, not to one particular signature kind, so this
is a reasonable extrapolation from the verified example, not a re-probe of
it — flagged as such for honesty.

**REQ-TEST-1.** Every test suite class extends `TestCase` (§3) and tags each
of its test methods with `@Test("description")`.

## 2. The runner

```phalcom
// test/framework.ph (continued)
class Runner {
  // Discovers and runs every @Test-tagged method on `suiteClass`.
  //
  // `Behavior#methods` returns the class's method SYMBOLS, not `Method`
  // objects (findings §9) — so each one is round-tripped through
  // `methodFor(_)` to reify it before its attributes can be inspected.
  // A method with no `Test` attribute, however named, is never invoked —
  // this is what makes it safe for `TestCase` itself to contribute
  // `assertEqual`/`assertTrue`/`assertError` methods to the same class
  // without the runner mistaking them for tests.
  static run(suiteClass) {
    let instance = suiteClass.new()
    var results = List.new()
    suiteClass.methods.each { sym =>
      let method = instance.methodFor(sym)
      let tags = method.attributesOfType(Test)
      if (tags.size > 0) {
        let desc = tags.at(0).desc
        let outcome = { method.invokeOn(instance, []) }.attempt()
        if (outcome.isOk) {
          results.add(TestResult.pass(sym, desc))
        } else {
          results.add(TestResult.fail(sym, desc, outcome.unwrapErr))
        }
      }
    }
    return results
  }

  // Prints one PASS/FAIL line per test plus a summary count. This IS the
  // pass/fail signal (§4) — there is no process exit code (findings §2)
  // to carry it instead.
  static report(results) {
    var passCount = 0
    var failCount = 0
    var i = 0
    while (i < results.size) {
      let r = results.at(i)
      if (r.ok) {
        System.print("PASS " + r.name.toString + " - " + r.desc)
        passCount = passCount + 1
      } else {
        System.print("FAIL " + r.name.toString + " - " + r.desc + ": " + r.error.message.unwrapOr("(no message)"))
        failCount = failCount + 1
      }
      i = i + 1
    }
    System.print(passCount.toString + " passed, " + failCount.toString + " failed")
  }
}
```

`r.error.message` returns an `Option` (the native `error_message` primitive
surfaces an unset message slot as `None` — every reified `Error`, including
`MessageNotUnderstood`, carries this slot), hence `.unwrapOr("(no message)")`
rather than a bare string read.

**REQ-TEST-2.** `Runner.run(suiteClass)` discovers tests exclusively via
`Behavior#methods` + `Method#attributesOfType(Test)`. There is no separate
registration step, and there could not be one: there is no way to install,
replace, or wrap a method on a class from `.ph` (findings §9) — the classic
"decorator registers itself" pattern is unavailable, so discovery has to be
structural (walk the class, ask each method "were you tagged?"), not
imperative (each test calls `register(self)`).

> **Commentary — this is where the language shines.** Everything else in
> this spec is a workaround for something missing. This is not: `perform`,
> `methodFor`, `Behavior#methods`, `attributesOfType`, `invokeOn` compose
> into a real attribute-driven test framework with **zero** special-casing
> and zero Rust changes. A user-defined `Test` attribute, ordinary
> inheritance for the assertion mixin, and five reflection sends is the
> whole runner. This is the positive finding this exercise was also
> supposed to produce, and it would be dishonest to bury it under the gaps.

> **Commentary — but the discovery loop pays a small tax for it.**
> `Behavior#methods` returns `Symbol`s, not `Method`s (findings §9) — a
> reasonable design (symbols are cheap, and most callers of `methods` don't
> want a reified `Method` for every entry), but it means every single
> candidate has to be round-tripped through `methodFor(_)` before its
> attributes are even visible. For a suite with a handful of test methods
> this is free; it is worth noting only because it is exactly the kind of
> two-hop indirection that would be silently expensive at 10,000 methods,
> which this program will never have.

## 3. Assertions

There is no `throw`/`catch` surface beyond `Function#attempt()` (findings
§9). A failed assertion has exactly one channel back to the runner: raise an
`Error`, and let the runner's own `.attempt()` (in `Runner.run`, §2) turn the
raise into an `Err` it records instead of propagating.

```phalcom
// test/framework.ph (continued)

// Raised by a failing assertion.
class AssertionError is Error {}

// Suites extend this for the assertion surface. There is no way to
// install a method onto a class from `.ph` (findings §9) — so assertions
// are not injected into suite classes at test-registration time; they are
// inherited the ordinary way, via `extends`, before any test runs.
class TestCase {
  assertEqual(expected, actual) {
    if (expected != actual) {
      AssertionError.new("expected " + expected.toString + " but got " + actual.toString).raise()
    }
  }

  assertTrue(cond) {
    if (not cond) {
      AssertionError.new("expected true").raise()
    }
  }

  // Runs `block` (a 0-arity block) and asserts it raises an Error whose
  // class is exactly `kind`. `.attempt()` is the only way to observe
  // whether a raise happened — there is no `try`/`catch`.
  assertError(kind, block) {
    let outcome = block.attempt()
    if (outcome.isOk) {
      AssertionError.new("expected " + kind.name.toString + " but nothing was raised").raise()
    } else {
      let err = outcome.unwrapErr
      if (not err.is(kind)) {
        AssertionError.new("expected " + kind.name.toString + " but got " + err.class.name.toString).raise()
      }
    }
  }
}

// One test's outcome. `name` is the method's Symbol (as returned by
// Behavior#methods), not a Method or a String.
class TestResult {
  @constructor
  pass(name, desc) { _name = name; _desc = desc; _ok = true; _error = None }
  @constructor
  fail(name, desc, error) { _name = name; _desc = desc; _ok = false; _error = error }

  name  => _name
  desc  => _desc
  ok    => _ok
  error => _error
}
```

A usage example:

```phalcom
class AssertionSuite is TestCase {
  @Test("assertError catches the exact kind, not just any Error")
  testAssertError() {
    self.assertError(ArgumentError, { ArgumentError.new("bad arg").raise() })
  }
}
```

**REQ-TEST-3.** A raised `AssertionError` (or any other `Error`) during a
test method is caught by `Runner.run`'s own `.attempt()` and recorded as a
failure, never propagated. One failing test must never stop the suite.

**REQ-TEST-4.** `assertEqual`, `assertTrue`, and `assertError(kind, block)`
are the only assertion primitives in v1. `assertEqual` uses the value's own
`==`/`!=` — for `CellValue` operands this is REQ-VM-3/4/5's propagation
semantics, not identity.

> **Commentary — DIV-ATTR-1 forces descriptions to be positional.**
> `attribute-classes.md` documents `@Author(name: "Ada")` — a keyword-label
> attribute argument. The parser rejects it (findings §9): only bare
> (`@Loud`) and positional (`@Test("x")`) forms parse. So `Test.desc` can
> never be self-documenting the way `@Test(desc: "...")` would read; every
> suite author has to remember that the sole positional argument to `@Test`
> is the description, not (say) a category or a priority. If DIV-ATTR-1 is
> ever resolved in the runtime's favor (parser catches up to the spec), this
> framework gains nothing structurally — it would just make the call sites
> more self-describing.

## 4. Golden corpus

Fixture "workbooks" cannot be loaded from a file — there is no file read
(findings §2). Every fixture is a `.ph` module that builds a `Grid`
programmatically from literal cell values and formula strings, then renders
it. 01-architecture.md's file layout lists a top-level `fixtures/` holding
only `*.golden` expected-output files; the fixture *programs* themselves are
`.ph` source and therefore belong under `src/`, e.g. `src/fixtures/`, a small
addition to that layout:

```phalcom
// src/fixtures/basic_sum.ph
import "../grid/grid.ph" as GridLib
import "../render/renderer.ph" as RenderLib

let Grid = GridLib.Grid
let Renderer = RenderLib.Renderer

let grid = Grid.new()
// ... populate cells (03-references-and-grid.md) ...
Renderer.render(grid, 3, 3)
```

The corresponding `fixtures/basic_sum.golden` holds the exact expected
stdout. The existing Phalcom golden-test lanes (see the repo's
[golden-test lane conventions](../../forge/README.md)) run the compiled
`phalcom` binary against a fixture `.ph` file, capture its stdout, and diff
it byte-for-byte against that `.golden` file. SheetCalc reuses this
infrastructure unchanged — there is no SheetCalc-specific test runner
process, only more fixtures added to the same corpus.

**Why a byte-exact diff is viable at all:** the engine is built on `Map`
throughout — `Grid` is `Map<Ref, Cell>` (01-architecture.md §2), the
dependency graph's edges are `Map`-backed (07, not yet written), and a
formula's referenced-cell set is a `Set`. If any of that iteration order
varied between runs, a stdout-exact golden test would be flaky by
construction. It does not: `Map`/`Set` insertion order is **verified
deterministic across runs** (findings §7). That single fact is what licenses
building a whole corpus of exact-diff tests instead of a corpus of
approximate/sorted-before-comparing ones.

**REQ-TEST-5.** Golden fixtures are `.ph` source, not an external file
format. A fixture's rendered stdout is diffed byte-for-byte against a
`.golden` file by the external test harness — never by Phalcom code itself,
which cannot read files to perform that comparison even if it wanted to.

**REQ-TEST-6.** No suite or fixture may re-sort a `Map`/`Set`'s iteration
order before rendering or asserting on it. The golden files pin whatever
order the program actually inserts in; sorting first would hide a real
ordering regression instead of catching it.

> **Commentary — the exit-code gap turns out not to matter.** Findings §2
> lists "process exit code" as `VERIFIED-ABSENT`, and at first pass that
> looks like a real problem: how does a test *runner* report pass/fail to
> whatever invokes it, with no exit status? It doesn't need to. The golden
> lane's failure signal is the stdout diff itself — a `FAIL testFoo: ...`
> line where the golden file has `PASS testFoo`, or a shifted line further
> down because an earlier failure printed an extra line, is already a
> byte-for-byte mismatch. The missing exit code is a real absence
> (13-language-gaps.md still lists it, because a hypothetical CI wrapper
> that only checks exit status would need it), but *this* test lane was
> never going to use it.

## 5. Lint tests — the interpolation lint

REQ-VM-9 (02-value-model.md §6) forbids interpolating a `CellValue`, `Ref`,
`Cell`, `Ast` node, or `Token` anywhere in SheetCalc — `"\(cell)"` silently
renders `<CellNum instance>` instead of the value (BUG-TOSTR-1), with no
diagnostic. REQ-VM-10 calls for "a source lint" to catch this.

That lint **cannot be a `.ph` program**. Reading source files is
`VERIFIED-ABSENT` (findings §2) — a Phalcom program has no way to open and
scan its own sibling `.ph` files. So the lint lives outside the language
entirely: a small external check (a shell script using `grep`, or a Rust
test in the existing test harness — either is fine, neither is Phalcom)
that scans `docs/apps/sheetcalc/src/**/*.ph` for the pattern `\(` applied to
an identifier bound to a known domain type, and fails the build/test run if
it finds one. This is a refinement of how 02-value-model.md phrases
REQ-VM-10 ("`test/framework.ph` includes a source lint") — the *intent*
(fail the suite on a stray interpolation) is unchanged, but the mechanism
has to sit beside the test harness, not inside `framework.ph`, precisely
because of the same file-read gap findings §2 already recorded.

A sketch of the check (illustrative, not Phalcom):

```sh
# test/lint_interpolation.sh — run as part of the same lane as the golden diffs
grep -rnE '\\\(\s*(cell|ref|ast|token|grid)\b' docs/apps/sheetcalc/src && {
  echo "FAIL: found string interpolation applied to a domain object (BUG-TOSTR-1)"
  exit 1
}
```

**REQ-TEST-8.** The interpolation lint runs as an external, non-Phalcom
check over `src/**/*.ph`, wired into the same test lane as the golden-diff
suites, and fails that lane (not a Phalcom test run) if `\(` is found applied
to an identifier of a known domain type. This is the concrete mechanism
behind REQ-VM-9/10 and REQ-RENDER-6.

## 6. Traceability

Every `REQ-*` across every SheetCalc document maps to at least one entry in
[14-traceability.md](14-traceability.md) (not yet written), which owns the
full `REQ -> spec § -> test` index. A requirement with no test listed there
is a spec bug (01-architecture.md §7). This document's own `REQ-TEST-*` and
`REQ-RENDER-*` (09-rendering.md §7) are both meta-requirements about the
test suite itself and are traced the same way as any other requirement — the
suite that tests the suite is not exempt.

**REQ-TEST-7.** Every `REQ-*` in the whole SheetCalc spec has at least one
row in 14-traceability.md mapping it to a suite file (or, for REQ-TEST-8, to
the external lint script).

## 7. Requirements and the suite list

| REQ | Statement |
|---|---|
| REQ-TEST-1 | Suites extend `TestCase`; test methods carry `@Test("description")`. |
| REQ-TEST-2 | Discovery is structural (`methods` + `attributesOfType`), never a registration call. |
| REQ-TEST-3 | A raised `Error` during a test is caught and recorded, never propagated. |
| REQ-TEST-4 | `assertEqual`/`assertTrue`/`assertError(kind, block)` are the only assertions in v1. |
| REQ-TEST-5 | Golden fixtures are `.ph` source; comparison is external, byte-exact. |
| REQ-TEST-6 | No suite/fixture re-sorts `Map`/`Set` order before comparing. |
| REQ-TEST-7 | Every `REQ-*` traces to a test in 14-traceability.md. |
| REQ-TEST-8 | The interpolation lint is external, not a `.ph` program. |

Current and planned suites (only the `value_*` and the lint entry are
grounded in documents that exist today; the rest are forward references to
02, 03-08, and 09, listed here so the roster is visible in one place):

| Suite | Covers |
|---|---|
| `suites/value_propagation.ph` | REQ-VM-3/4/5 |
| `suites/value_divzero.ph` | REQ-VM-6 |
| `suites/value_format.ph` | REQ-VM-7/8, REQ-RENDER-5 |
| `suites/value_compare.ph` | REQ-VM-11/12 |
| `suites/render_borders.ph` | REQ-RENDER-1 |
| `suites/render_widths.ph` | REQ-RENDER-2/3 |
| `suites/render_alignment.ph` | REQ-RENDER-4 |
| `test/lint_interpolation.sh` (external) | REQ-VM-9/10, REQ-RENDER-6, REQ-TEST-8 |
| `suites/ref_a1.ph` *(pending 03)* | A1 encode/decode round-trip |
| `suites/lex_formula.ph` *(pending 04)* | formula tokenization |
| `suites/parse_formula.ph` *(pending 05)* | Pratt parser, `Result` errors |
| `suites/eval_functions.ph` *(pending 06, 08)* | `SUM`/`IF`/... dispatch |
| `suites/recalc_topo.ph` *(pending 07)* | topological recalc order |
| `suites/recalc_cycles.ph` *(pending 07)* | `#CIRC!` detection |

Every suite class above is expected to extend `TestCase` and be invoked as
`Runner.report(Runner.run(SuiteClass))` from `main.ph`, one suite at a time.
