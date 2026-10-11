# `parse` — typed parser combinators for Telos

`parse` v1 is a general-purpose, typed parser-combinator library written
entirely in Telos. It transforms sequential input into domain values while
statically preserving the relationship between the input source type and
the parser output type.

The defining property:

> **Parser composition is ordinary Telos type composition.**

A `Rule<TextSource, Int>` is statically different from a
`Rule<TextSource, String>`; `map` consumes exactly the producing rule's
output; alternatives require equivalent outputs; sequencing computes typed
products; recursion and type-erasure boundaries preserve both the source
and result types.

## Layout

```
src/
├── core/
│   ├── source.ph      trait Source { type Token; size; at }
│   ├── cursor.ph      class Cursor<S> (persistent positions)
│   ├── result.ph      data ParseResult<S, T>, RunResult<T>
│   ├── error.ph       class ParseError (merge, contexts, rendering)
│   ├── span.ph        data Span, Spanned<T>
│   ├── pair.ph        data Pair<A, B>
│   ├── parser.ph      trait Parser<S> { type Output; parse }
│   └── rule.ph        class Rule<S, T> + all combinators
├── source/
│   ├── text-source.ph  byte-oriented source over String (Token = Int)
│   └── list-source.ph  owned Array snapshot retained by read-only Slice<T>
├── text.ph            Text: character-level factories over TextSource
└── run.ph             Parse.run / Parse.runAll
```

## Usage

```telos
from parse.core.result import RunResult
from parse.run import Parse
from parse.source.text_source import TextSource
from parse.text import Text

let digitValue = Text.asciiDigit.map |byte| { byte - 48 }
let number = digitValue.many1.map |digits| { /* fold */ ... }

match Parse.runAll(number, TextSource.new("1234")) {
  RunResult::Ok(value) => { System.print(value) }
  RunResult::Err(error) => { System.print(error.describe) }
}
```

Combinator algebra (for rules producing `A` / `B`):

| Operation                          | Result               |
| ---------------------------------- | -------------------- |
| `map (A)->B`                       | `Rule<S, B>`         |
| `flatMap (A)->Rule<S,B>`           | `Rule<S, B>`         |
| `zip Rule<S,B>`                    | `Rule<S, Pair<A,B>>` |
| `then Rule<S,B>` / `before`        | `Rule<S, B>` / `A`   |
| `orElse Rule<S,A>`                 | `Rule<S, A>`         |
| `optional`                         | `Rule<S, Option<A>>` |
| `many` / `many1` / `repeat(n)`     | `Rule<S, List<A>>`   |
| `sepBy` / `sepBy1`                 | `Rule<S, List<A>>`   |
| `between(open, close)`             | `Rule<S, A>`         |
| `lookAhead` / `attempt` / `cut`    | `Rule<S, A>`         |
| `named(context)`                   | `Rule<S, A>`         |
| `spanned`                          | `Rule<S, Spanned<A>>`|
| `notFollowedBy(rule, what)`        | `Rule<S, Unit>`      |
| `chainLeft1` / `chainRight1`       | operator chains      |

Recursion uses typed deferred rules:

```telos
let expression: Rule<TextSource, Int> = Rule.defer()
expression.define(digitValue.orElse(expression.between(lparen, rparen)))
```

## Semantics highlights

- **Algebraic failure.** Ordinary failure is `ParseResult` data, never an
  exception. Exceptions are reserved for invariant violations
  (non-progressing repetition, executing an undefined deferred rule).
- **Controlled backtracking.** `orElse` tries its alternative only after
  an uncommitted failure, restoring the original cursor. Sequencing
  commits failures that occur after the sequence consumed input.
  `attempt` re-enables backtracking; `cut` forbids it.
- **Furthest-error merging.** Alternative failures prefer the most
  advanced offset and merge expectations at equal offsets.
- **Repetition progress.** Successful repetition iterations must advance
  the cursor; `many`/`many1`/`sepBy`/chains enforce this mechanically.
- **Typed erasure.** `Rule<S, T>` erases implementation types only; `S`
  and `T` are never erased, and no composition step passes through
  `Dynamic`.

## Design adaptations relative to MAIN.md

The design document sketches concrete combinator *types*
(`MapParser<S, P, U>`, `ZipParser<S, P, Q>`, ...) as the static spine,
with `Parser<S>` as the composition interface. The current compiler has
two blockers on that path (see "Known compiler limitations"): trait
methods returning associated-type-projected data cannot be matched, and
generic conformance dispatch is unreliable (the same class of failure
recorded for the `hypothesis` package in LANG007.C4.P4.R1).

v1 therefore composes through `Rule<S, T>` directly: every combinator is
an inherent `Rule` method that builds a composed run closure. This keeps
every type-level guarantee that matters (`map` consumes exactly `T`;
`orElse` needs one `T`; spans stay typed; erasure never touches `S`/`T`)
while staying inside proven-compiler territory. The `Parser<S>` trait
remains the conformance surface (`Rule` conforms), so a future compiler
repair can restore the concrete static spine without changing call sites.

## Known compiler limitations encountered

These shape the implementation and are recorded here for the compiler
work queue; none require changing this package's public semantics:

1. Matching data whose type arguments contain associated-type projections
   of generic parameters retains a coverage witness and poisons pattern
   bindings (`Cursor<S>` values become unusable).
2. Values of fixture-defined ADTs flowing through `ListSource<T>` rules
   cannot be matched at the module level (witness retained) or passed to
   `@class` methods ("associated family invocation lowering is not
   implemented"). Builtin payloads (`Int`, `String`) are unaffected; the
   expression-parser fixture therefore renders trees to strings during
   parsing.
3. Closures stored in escaping fields must not use `return` (DeadFrameError);
   all combinator bodies are flag/tail-expression structured.
4. `and`/`or` compositions produce `Object`, not `Bool`, in value
   position; predicate closures wrap them in `if` expressions.
5. Impl methods on `data` declarations do not dispatch at runtime;
   behavior-carrying types (`Cursor`, `ParseError`) are classes.
6. Trailing closures may not appear mid-argument-list; factory signatures
   place closure parameters last.

## Testing

Runtime fixtures live in `tests/runtime/` (a separate project depending
on `parse` by path) and are executed by the Rust integration test
`phalcom-core/tests/core/modules/parse_package.rs`:

```sh
RUSTFLAGS='' cargo test -p phalcom-core --test core modules_parse_package
```

Each fixture is run in an isolated `phalcom` process and must print a
`PASS <name>` marker. Fixture compilation instantiates the generic
library per module (tens of seconds each); the runner parallelizes over
four workers.

ListSource's Array/Slice snapshot contract is independently executable:

```sh
RUSTFLAGS='' cargo run -p phalcom-core --bin phalcom -- packages/parse/tests/runtime/src/core/list-source-snapshot-test.ph
RUSTFLAGS='' cargo test -p phalcom-core --test core parse_list_source_snapshot -- --nocapture
```

The focused Rust filter selects the exact-marker fixture test and a native GC
test. The latter collects the caller List, verifies the source's Slice has an
Array owner, and reads/parses its retained heap-backed elements after GC.
Snapshot independence covers membership, order, and retained references;
mutable token payloads are not recursively cloned.

The complete-package command remains the integration gate. As of the P5 audit
continuation (2026-10-11), it is blocked by inherited TextSource String/Substring
and optional-byte consumer incompatibilities. The independent ListSource PASS
does not certify those text adapters or the complete parser package.
