# `parse` v1 — Typed Parser Combinators for Telos

`parse` should be a **general-purpose, strongly typed parser-combinator library implemented entirely in Telos**. Its purpose is not merely to parse strings. It provides a typed abstraction for transforming sequential input into domain values while preserving the relationship between the input token type and parser output type throughout composition.

The first production consumer should be `toml`, but the library must be designed independently of TOML.

The defining property of `parse` should be:

> **Parser composition is ordinary Telos type composition.**

A parser that produces `Int` is statically different from a parser that produces `String`; mapping consumes exactly the preceding parser's output; alternatives can only combine compatible output types; sequencing computes a typed product; token parsers are tied to the source's associated token type; recursive/type-erased parser boundaries preserve both source and result types.

The library should deliberately exercise Telos's existing advanced type system—**traits, associated types, associated-type specialization, generic constraints, type equivalence constraints, callable types, generic nominal data, algebraic data types and exhaustive matching**—without depending on LANG007.C4.

---

# 1. Scope of v1

`parse` v1 should be **feature-rich enough to implement a serious TOML parser**, recursive programming-language grammars, command languages, and binary protocols.

It should include:

- generic typed input sources;
- typed cursors;
- typed parser output through associated types;
- algebraic success/failure;
- mapping and dependent sequencing;
- heterogeneous sequencing;
- alternatives;
- optional and repeated parsing;
- separated sequences;
- lookahead;
- controlled backtracking;
- explicit commitment/cut;
- named grammar contexts;
- source spans;
- recursive parser rules;
- safe parser type erasure;
- operator-chain helpers;
- progress guarantees for repetition;
- useful compositional error reporting;
- text and list-based source adapters.

It should **not** attempt in v1:

- parser derivation through reflection;
- generic `derive Parser`;
- first-class parser Packs;
- arbitrary heterogeneous n-ary sequencing;
- compile-time grammar analysis;
- automatic memoization/packrat parsing;
- generalized parser AST optimization;
- incremental parsing;
- parser generators;
- lexer generators;
- streaming/asynchronous input;
- error recovery intended for IDE-quality compiler parsers;
- C4 Product/Sum reflection;
- static structural iteration;
- type-level parser computation.

Those are sensible future expansions. None are necessary for `parse` to be a substantial library.

---

# 2. Core abstraction

The center of the library is deliberately small:

```telos
trait Source {
    type Token

    size -> Int

    at(
        _ index: Int
    ) -> Self::Token
}
```

A parser is parameterized by its **source type**, while its produced value is represented by an associated type:

```telos
trait Parser<S>
where S |= Source
{
    type Output

    parse(
        _ cursor: Cursor<S>
    ) -> ParseResult<S, Self::Output>
}
```

This gives us two independent dimensions:

```text
Parser<S>::Output
       │      │
       │      └── produced semantic type
       │
       └───────── accepted source representation
```

For example:

```text
Parser<TextSource, Output = String>
Parser<TextSource, Output = Int>
Parser<TokenSource, Output = Expr>
Parser<BytesSource, Output = Packet>
```

although the actual Telos spelling is:

```telos
P |= Parser<TextSource, Output = Expr>
```

rather than a two-parameter `Parser<S, T>` trait.

That distinction is intentional.

---

# 3. Why `Output` should be an associated type

We could define:

```telos
trait Parser<S, T>
```

but I would strongly prefer:

```telos
trait Parser<S> {
    type Output
}
```

because the output is a **property of the parser implementation under a source**.

This enables constraints such as:

```telos
P |= Parser<S>
```

and projections:

```telos
P::Output
```

Then combinators can express genuine relationships.

For example:

```telos
data MapParser<S, P, U>(
    parser: P,
    transform: (P::Output) -> U,
)
where
    S |= Source,
    P |= Parser<S>
```

The transform cannot accidentally accept the wrong type.

The compiler knows:

```text
P::Output ──transform──> U
```

and therefore:

```telos
impl<S, P, U> Parser<S> for MapParser<S, P, U>
where
    S |= Source,
    P |= Parser<S>
{
    type Output = U
}
```

This is exactly the kind of type-system pressure we want this package to impose.

---

# 4. `Cursor<S>`

Parsing should not copy or slice input during ordinary traversal.

```telos
data Cursor<S>(
    source: S,
    offset: Int,
)
where S |= Source
```

Operations:

```telos
impl<S> Cursor<S>
where S |= Source
{
    current -> Option<S::Token>

    advance -> Cursor<S>

    advanceBy(
        _ amount: Int
    ) -> Cursor<S>

    atEnd -> Bool

    remaining -> Int
}
```

Conceptually:

```text
source
─────────────────────────────────────────────
        ^
        offset

Cursor(source, offset)
```

Advancing a parser creates another cursor:

```text
Cursor(source, 12)
       ↓
Cursor(source, 13)
```

not another source.

### v1 invariant

```text
Cursor advance = O(1)
```

assuming the source itself supports O(1) indexing.

---

# 5. Stable source semantics

A parser must observe a stable sequence during a parse.

Therefore:

> A `Source` used by `parse` must not semantically mutate underneath a running parser.

The library should provide owned/snapshotted adapters where necessary rather than trusting mutable caller collections.

For example:

```telos
class ListSource<T> {
    // private stable backing representation
}

impl<T> Source for ListSource<T> {
    type Token = T
}
```

Likewise:

```telos
class TextSource { ... }

impl Source for TextSource {
    type Token = Char
}
```

For v1, the canonical built-in adapters should be:

```text
TextSource
ListSource<T>
```

A `BytesSource` may be included if the necessary stable `Bytes` representation is straightforward; otherwise it can follow immediately after v1 without changing the abstractions.

---

# 6. Parse results are algebraic

Failures should not normally be represented by exceptions.

```telos
data ParseResult<S, T>
where S |= Source
{
    @variant
    Success(
        _ value: T,
        _ next: Cursor<S>,
    )

    @variant
    Failure(
        _ error: ParseError,
        _ committed: Bool,
    )
}
```

The `committed` bit is fundamental to the semantics of alternatives.

A parser either:

```text
Success(value, next)
```

or:

```text
Failure(error, committed?)
```

This gives composition ordinary exhaustive algebraic control flow.

---

# 7. `ParseError`

A useful parser library lives or dies on diagnostics.

v1 should have a structured error representation roughly like:

```telos
data ParseError(
    offset: Int,
    expected: List<String>,
    contexts: List<String>,
    message: Option<String>,
)
```

Potentially with a compact internal representation for expected items, but these semantics should exist.

A diagnostic might ultimately render as:

```text
expected integer or '('
at line 14, column 23

while parsing:
    primary expression
    multiplication expression
    expression
```

`parse` should report **parser-domain information**.

Mapping offsets to filenames, lines and columns can be a source/text utility layer.

---

# 8. Primitive parser: `satisfy`

The most fundamental token parser:

```telos
data Satisfy<S>(
    predicate: (S::Token) -> Bool,
    expected: String,
)
where S |= Source
```

Conformance:

```telos
impl<S> Parser<S> for Satisfy<S>
where S |= Source
{
    type Output = S::Token
}
```

Its type relationship is important:

```text
Source::Token
     │
     ▼
Satisfy<S>::Output
     =
S::Token
```

Example:

```telos
let digit =
    Parsers.satisfy(
        |char| char >= '0' and char <= '9',
        expected: "digit",
    )
```

The compiler knows `digit` produces `Char`.

---

# 9. `select`: typed token projection

One of the highest-value primitives should be:

```telos
select<T>(
    selector: (S::Token) -> Option<T>,
    expected: String,
)
```

This recognizes and transforms a token simultaneously.

Suppose a lexer produces:

```telos
data Token {
    @variant
    Integer(_ value: Int)

    @variant
    Identifier(_ value: String)

    @variant
    Plus

    @variant
    Star
}
```

Then:

```telos
let integer =
    Parsers.select(
        |token| {
            match token {
                Token::Integer(value) =>
                    Some(value)

                _ =>
                    None
            }
        },
        expected: "integer",
    )
```

has output:

```text
Int
```

not:

```text
Token
```

This avoids the common anti-pattern:

```text
parse Token
→ inspect variant again
→ manually extract payload
```

and naturally exploits Telos ADTs plus typed closures.

---

# 10. Pure parser

```telos
Parsers.pure(value)
```

returns a parser that succeeds without consuming input.

Its type is inferred from the value:

```telos
Parsers.pure(42)
```

produces:

```text
Int
```

while:

```telos
Parsers.pure(Expr::Integer(42))
```

produces:

```text
Expr
```

This becomes important in `flatMap`, optional branches, and runtime-dependent grammar selection.

---

# 11. Explicit failure

```telos
Parsers.fail<T>(
    _ message: String
) -> ...
```

must allow the expected output type to be established contextually/generically.

This lets grammars express semantic rejection during parsing:

```telos
number.flatMap(
    |value| {
        if value >= 0 {
            Parsers.pure(value)
        } else {
            Parsers.fail<Int>(
                "expected non-negative integer"
            )
        }
    }
)
```

---

# 12. Mapping

The central functor-like operation:

```telos
parser.map(
    transform
)
```

Semantically:

```text
Parser<A>
   +
(A -> B)
   =
Parser<B>
```

More precisely in Telos:

```telos
data MapParser<S, P, U>(
    parser: P,
    transform: (P::Output) -> U,
)
where
    S |= Source,
    P |= Parser<S>
```

and:

```telos
impl<S, P, U> Parser<S> for MapParser<S, P, U>
where
    S |= Source,
    P |= Parser<S>
{
    type Output = U
}
```

Example:

```telos
let number =
    digits
        .map(
            |chars| {
                Integer.parse(
                    String.from(chars)
                )
            }
        )
```

If `digits` produces `List<Char>`, the closure is statically required to consume `List<Char>`.

---

# 13. Typed sequencing

Two heterogeneous parsers need a nominal product.

Because v1 deliberately avoids C4 Pack machinery:

```telos
data Pair<A, B>(
    first: A,
    second: B,
)
```

Then:

```text
Parser<A> × Parser<B>
        ↓
Parser<Pair<A, B>>
```

represented by a parser such as:

```telos
data ZipParser<S, P, Q>(
    left: P,
    right: Q,
)
where
    S |= Source,
    P |= Parser<S>,
    Q |= Parser<S>
```

with:

```telos
impl<S, P, Q> Parser<S>
    for ZipParser<S, P, Q>
where
    S |= Source,
    P |= Parser<S>,
    Q |= Parser<S>
{
    type Output =
        Pair<P::Output, Q::Output>
}
```

This is a significant use of associated-type projection.

---

# 14. Example of heterogeneous sequencing

Suppose:

```text
identifier.Output == String
integer.Output    == Int
```

Then:

```telos
identifier.zip(integer)
```

must have output:

```text
Pair<String, Int>
```

and:

```telos
identifier
    .zip(integer)
    .map(
        |pair| {
            Binding(
                name: pair.first,
                priority: pair.second,
            )
        }
    )
```

produces:

```text
Binding
```

No `Dynamic` is involved at any stage.

---

# 15. Keep-left / keep-right sequencing

Most grammars do not want every punctuation token in the result.

Therefore v1 should expose:

```telos
left.before(right)
```

meaning:

```text
parse left
parse right
return left result
```

and:

```telos
left.then(right)
```

meaning:

```text
parse left
parse right
return right result
```

Types:

```text
Parser<A>.before(Parser<B>)
    -> Parser<A>

Parser<A>.then(Parser<B>)
    -> Parser<B>
```

Example:

```telos
identifier
    .before(
        Parsers.char('=')
    )
```

still produces `String`.

And:

```telos
Parsers.char('=')
    .then(value)
```

produces whatever `value` produces.

This dramatically reduces nested `Pair`s in real grammars.

---

# 16. Alternatives use type equivalence

This should be one of the signature advanced-type features of the package.

An alternative combinator can only sensibly return one static output type.

So:

```telos
data OrElse<S, L, R>(
    left: L,
    right: R,
)
where
    S |= Source,
    L |= Parser<S>,
    R |= Parser<S>,
    L::Output == R::Output
```

The constraint is fundamental:

```text
L::Output == R::Output
```

Then:

```telos
impl<S, L, R> Parser<S>
    for OrElse<S, L, R>
where
    S |= Source,
    L |= Parser<S>,
    R |= Parser<S>,
    L::Output == R::Output
{
    type Output = L::Output
}
```

This means:

```telos
integerExpr
    .orElse(identifierExpr)
```

works if both produce:

```text
Expr
```

while:

```telos
integerParser
    .orElse(stringParser)
```

is rejected when one produces `Int` and the other `String`.

Unless the programmer deliberately maps both into a common sum:

```telos
integer
    .map(
        |value| Value::Integer(value)
    )
    .orElse(
        string
            .map(
                |value| Value::String(value)
            )
    )
```

Now both produce:

```text
Value
```

and the composition is valid.

That is excellent type-driven grammar design.

---

# 17. `flatMap`: value-dependent parsing

`map` changes a value.

`flatMap` allows a parsed value to determine **what parser comes next**.

Conceptually:

```text
Parser<A>
+
(A -> Parser<B>)
=
Parser<B>
```

A strongly typed generic representation can be:

```telos
data FlatMapParser<S, P, Q>(
    parser: P,
    next: (P::Output) -> Q,
)
where
    S |= Source,
    P |= Parser<S>,
    Q |= Parser<S>
```

Then:

```telos
impl<S, P, Q> Parser<S>
    for FlatMapParser<S, P, Q>
where
    S |= Source,
    P |= Parser<S>,
    Q |= Parser<S>
{
    type Output = Q::Output
}
```

This provides real dependent *value-guided* parser construction without requiring dependent types.

---

# 18. `flatMap` example: length-prefixed data

Suppose:

```telos
Binary.u16BE()
```

produces:

```text
Int
```

Then:

```telos
Binary.u16BE()
    .flatMap(
        |length| {
            Binary.take(length)
        }
    )
```

produces:

```text
Bytes
```

The runtime integer determines how much subsequent input is consumed.

That is a serious protocol-parsing capability, not merely a convenience operator.

---

# 19. When `flatMap` branches have different implementation types

Consider:

```telos
opcode.flatMap(
    |opcode| {
        match opcode {
            Opcode::Ping =>
                Parsers.pure(
                    Message::Ping
                )

            Opcode::Text =>
                stringPayload
                    .map(
                        |text| {
                            Message::Text(text)
                        }
                    )
        }
    }
)
```

Those two branches may have different concrete parser implementation types.

This is precisely where `Rule<S, T>` becomes useful:

```telos
opcode.flatMap(
    |opcode| {
        match opcode {
            Opcode::Ping =>
                Rule.of(
                    Parsers.pure(
                        Message::Ping
                    )
                )

            Opcode::Text =>
                Rule.of(
                    stringPayload
                        .map(
                            |text| {
                                Message::Text(text)
                            }
                        )
                )
        }
    }
)
```

Both branches now return:

```text
Rule<S, Message>
```

without falling back to `Dynamic`.

---

# 20. `Rule<S, T>` — typed parser erasure

Concrete combinator types grow rapidly.

A grammar such as:

```telos
a.then(b)
 .zip(c.orElse(d))
 .map(...)
```

may have an enormous inferred implementation type.

Recursive grammars also require an indirection boundary.

So v1 needs a deliberate parser-erasure type:

```telos
class Rule<S, T>
where S |= Source
{
    _run:
        (Cursor<S>)
            -> ParseResult<S, T>

    @constructor
    new(
        run:
            (Cursor<S>)
                -> ParseResult<S, T>
    ) {
        _run = run
    }
}
```

and:

```telos
impl<S, T> Parser<S> for Rule<S, T>
where S |= Source
{
    type Output = T
}
```

The important thing is what is erased:

```text
concrete parser implementation type
```

and what remains statically preserved:

```text
source S
output T
```

Hence:

```text
HugeMapOfChoiceOfZipOf...
          ↓ erase
Rule<TextSource, Expr>
```

not:

```text
Dynamic
```

---

# 21. The “static spine, erased joints” pattern

This should be explicitly taught as the canonical design pattern for `parse`.

Use concrete generic parser types:

```text
inside small grammar expressions
```

because they preserve maximal static information and allow the compiler to optimize aggressively.

Use:

```telos
Rule<S, T>
```

at:

- recursive grammar boundaries;
- public module boundaries;
- long parser expressions;
- parser storage boundaries;
- conditional parser construction;
- compilation-complexity boundaries.

So a grammar might look like:

```text
concrete typed combinators
        │
        │
        ▼
Rule<S, Expr>
        │
        ▼
more concrete composition
        │
        ▼
Rule<S, Statement>
```

This is more disciplined than globally type-erasing every parser.

---

# 22. Recursive rules

v1 must support recursive grammars.

Expressions are the obvious example:

```text
expr
 └── atom
      └── '(' expr ')'
```

A rule API could provide:

```telos
let expression =
    Rule.defer<TextSource, Expr>()
```

followed later by:

```telos
expression.define(actualParser)
```

The public semantics should guarantee:

- a deferred rule cannot be executed before definition;
- definition occurs once;
- defining a parser with the wrong source or output type is impossible;
- recursion uses the same typed `Rule<S, T>` identity.

Example:

```telos
let expression =
    Rule.defer<TokenSource, Expr>()

let atom =
    integer
        .orElse(identifier)
        .orElse(
            expression.between(
                leftParen,
                rightParen,
            )
        )

let actual =
    Parsers.chainLeft1(
        operand: atom,
        operator: additionOperator,
    )

expression.define(actual)
```

---

# 23. Optional parsing

```telos
parser.optional()
```

should transform:

```text
Parser<T>
```

into:

```text
Parser<Option<T>>
```

with semantics:

```text
success        → Some(value)

uncommitted
failure        → None without consuming input

committed
failure        → propagate failure
```

The commitment rule is critical.

An optional parser must not swallow a genuine syntax failure after entering a committed construct.

---

# 24. Repetition

v1 should include:

```text
many
many1
repeat(count)
```

Types:

```text
Parser<T>.many()
    -> Parser<List<T>>

Parser<T>.many1()
    -> Parser<List<T>>
```

The output type remains completely known.

No variadic generic machinery is necessary because repeated values are homogeneous.

---

# 25. The repetition progress invariant

This is a non-negotiable correctness rule.

If a repeated parser succeeds without consuming input:

```text
cursor before == cursor after
```

then:

```telos
parser.many()
```

would otherwise loop forever.

Therefore:

> Every successful iteration of a repeating parser must advance the cursor.

If it does not, `parse` must terminate with an internal grammar failure such as:

```text
NonProgressingParser
```

Example of a dangerous grammar:

```telos
foo.optional().many()
```

because `optional()` can succeed with `None` while consuming nothing.

The library must detect this mechanically.

---

# 26. Separated repetition

Needed immediately for TOML arrays and practically every language:

```telos
item.sepBy(separator)
item.sepBy1(separator)
```

Types:

```text
Parser<T>
+
Parser<Sep>
=
Parser<List<T>>
```

The separator's output type is intentionally discarded.

Examples:

```telos
integer.sepBy(comma)
```

```telos
argument.sepBy(comma)
```

```telos
tomlValue.sepBy(comma)
```

---

# 27. `between`

A common grammar shape:

```text
open
value
close
```

should be expressible as:

```telos
value.between(open, close)
```

and preserve:

```text
value.Output
```

For example:

```telos
expression.between(
    leftParen,
    rightParen,
)
```

still produces:

```text
Expr
```

This is semantically equivalent to:

```telos
leftParen
    .then(expression)
    .before(rightParen)
```

but expresses intent more clearly.

---

# 28. Lookahead

v1 should include both positive and negative lookahead.

```telos
parser.lookAhead()
```

runs `parser` but restores the original cursor on success.

Type:

```text
Parser<T> -> Parser<T>
```

Negative lookahead:

```telos
parser.notFollowedBy(...)
```

or equivalent primitive, used for lexical boundaries.

Example:

A keyword parser for:

```text
true
```

should not accidentally accept the prefix of:

```text
trueValue
```

A grammar can require:

```text
literal("true")
followed by
not identifier character
```

without consuming that boundary character.

---

# 29. Backtracking semantics

Choice should not silently rewind arbitrary amounts of meaningful work.

The default should be **controlled backtracking**.

For:

```telos
left.orElse(right)
```

semantics are:

```text
run left

left succeeds
    -> success

left fails uncommitted
    -> restore original cursor
    -> run right

left fails committed
    -> return left failure
```

This gives grammar authors predictable performance and precise errors.

---

# 30. `attempt`

Sometimes a grammar intentionally wants backtracking after partial consumption.

Then:

```telos
parser.attempt()
```

converts a committed/consuming failure into a backtrackable one for the containing choice.

For example:

```telos
call
    .attempt()
    .orElse(identifier)
```

if call syntax genuinely shares a prefix that cannot otherwise be distinguished.

`attempt` should be explicit because unrestricted automatic backtracking can make grammars:

- slow;
- unpredictable;
- diagnostically poor.

---

# 31. `cut`

The complementary operation is:

```telos
parser.cut()
```

or a `cut()` marker after a syntactic decision point.

Conceptually:

> If parsing fails after this point, do not pretend an earlier alternative might have been intended.

Example:

```text
identifier '(' ...
```

Once `(` has been consumed after an identifier in a grammar where that uniquely denotes a call, malformed arguments should not backtrack and reinterpret the entire expression as a plain identifier.

This:

```telos
leftParen
    .then(argumentList.cut())
```

should produce a committed failure if the argument list is malformed.

`cut` is simultaneously:

- a correctness tool;
- a diagnostic tool;
- a performance tool.

---

# 32. Named parser contexts

```telos
parser.named(
    "inline table"
)
```

adds semantic context to errors.

For TOML:

```telos
inlineTable
    .named("inline table")
```

might produce:

```text
expected TOML value
at line 9, column 24

while parsing:
    inline table
    dependency declaration
```

The parser should not discard inner expectations; `named` enriches the context stack.

---

# 33. Source spans

v1 should expose:

```telos
data Span(
    start: Int,
    end: Int,
)

data Spanned<T>(
    value: T,
    span: Span,
)
```

Then:

```telos
parser.spanned()
```

transforms:

```text
Parser<T>
```

into:

```text
Parser<Spanned<T>>
```

This is critical for `toml`.

For:

```toml
namespace = "core"
```

the manifest validator needs to retain where `"core"` came from even though semantic validation occurs much later.

---

# 34. `end`

A parser that verifies input exhaustion:

```telos
Parsers.end()
```

produces:

```text
()
```

or another unit-like result.

This lets:

```telos
document.before(
    Parsers.end()
)
```

mean:

> Parse one complete document and reject trailing garbage.

---

# 35. Literal parsing

For `TextSource`, v1 should provide convenient text parsers:

```telos
Text.char('[')
Text.literal("true")
Text.literal("false")
```

These are domain-specific conveniences layered above generic `satisfy`.

They should not infect the generic parser abstraction.

Conceptually:

```text
parse.core
    token-level generic machinery

parse.text
    character/text conveniences
```

---

# 36. Character classes

The text module should provide enough primitives for real grammars:

```text
char
oneOf
noneOf

asciiLetter
asciiDigit
hexDigit
whitespaceChar
lineBreak

takeWhile
takeWhile1
```

But avoid turning `parse` into a full regular-expression library in v1.

A TOML parser needs good lexical primitives, not a second regex engine.

---

# 37. `takeWhile` and `takeWhile1`

These are worthwhile primitive optimizations.

Instead of:

```telos
satisfy(predicate)
    .many()
```

building an intermediate `List<Char>` one token at a time, text sources can support efficient contiguous scanning.

Semantically:

```telos
Text.takeWhile(predicate)
```

produces:

```text
String
```

and:

```telos
Text.takeWhile1(predicate)
```

requires at least one match.

These can still conform to ordinary `Parser<TextSource>`.

The abstraction stays pure; the implementation can be specialized.

---

# 38. Operator parsing

v1 should include operator-chain combinators because they are common, reusable, and showcase Telos's callable typing particularly well.

For left-associative operators:

```telos
Parsers.chainLeft1(
    operand: operand,
    operator: operator,
)
```

Require:

```telos
Operand |= Parser<S, Output = T>

Operator |= Parser<
    S,
    Output = (T, T) -> T
>
```

Conceptually:

```text
operand:
    Parser<T>

operator:
    Parser<(T, T) -> T>
```

The operator parser produces the semantic reduction function itself.

---

# 39. Operator parser example

Suppose:

```telos
let plus =
    token(Token::Plus)
        .map(
            |_| {
                |left, right| {
                    Expr::Add(
                        left,
                        right,
                    )
                }
            }
        )
```

Then:

```text
plus.Output
    ==
(Expr, Expr) -> Expr
```

Likewise multiplication.

Then:

```telos
let term =
    Parsers.chainLeft1(
        operand: atom,
        operator: multiplyOperator,
    )

let expression =
    Parsers.chainLeft1(
        operand: term,
        operator: addOperator,
    )
```

Both produce:

```text
Expr
```

The parser library is using callable types as first-class semantic values.

---

# 40. `chainRight1`

Right-associative parsing should also exist.

Useful for constructs such as exponentiation or type arrows:

```text
A -> B -> C
```

when interpreted as:

```text
A -> (B -> C)
```

Again:

```text
Operator.Output == (T, T) -> T
```

but reduction direction differs.

---

# 41. Parsing entire sources

The library should provide execution helpers distinct from parser construction.

For example:

```telos
ParserRunner.run(
    parser,
    source,
)
```

and:

```telos
ParserRunner.runAll(
    parser,
    source,
)
```

where:

```text
run
    does not require exhaustion

runAll
    requires end-of-input
```

The returned public result may omit the cursor when running a complete document:

```telos
data RunResult<T> {
    @variant
    Success(_ value: T)

    @variant
    Failure(_ error: ParseError)
}
```

But internally parser composition continues to use `ParseResult<S,T>`.

---

# 42. Error merging for alternatives

Consider:

```telos
integer
    .orElse(identifier)
    .orElse(leftParen)
```

If all fail, the error should not simply report the last branch.

The rule should be:

1. prefer failure at the greatest offset;
2. when failures occur at the same greatest offset, merge expectations;
3. preserve appropriate context;
4. committed failure wins immediately.

Thus input:

```text
@
```

could yield:

```text
expected:
    integer
    identifier
    '('
```

rather than:

```text
expected '('
```

This behavior should be built into `orElse`.

---

# 43. No implicit exception semantics

Ordinary failure such as:

```text
expected identifier
```

must remain data.

Exceptions should be reserved for:

- violated parser-library invariants;
- impossible internal states;
- erroneous use such as executing an undefined deferred rule if the API cannot prevent it statically.

This distinction is especially important when `parse` becomes infrastructure for `toml`.

---

# 44. Parser state is deliberately excluded from v1

Some combinator libraries provide:

```text
Parser<State, Input, Output>
```

or mutable user state.

I would **not** add this to v1.

It substantially complicates:

- rollback semantics;
- lookahead;
- attempt;
- commitment;
- parser equivalence;
- error behavior;
- source abstractions.

TOML does not require arbitrary parser state because its semantic table construction belongs in the subsequent assembler.

That reinforces our earlier architecture:

```text
stateful semantic interpretation
    should not be smuggled into
    syntactic parsing.
```

---

# 45. No generic context/environment parameter in v1

Likewise, do not prematurely turn:

```telos
Parser<S>
```

into:

```telos
Parser<S, Environment, State, ...>
```

If grammars need contextual values, ordinary Telos closures already provide lexical capture:

```telos
let expectedVersion = ...

let parser =
    version.flatMap(
        |value| {
            ...
        }
    )
```

Keep the parser type algebra small.

---

# 46. No heterogeneous n-ary `sequence` in v1

Without C4 Packs, this becomes awkward.

We should not simulate variadic type programming with complicated nominal machinery.

Use:

```telos
a.zip(b)
```

and immediately map meaningful combinations into domain types.

Good:

```telos
name
    .zip(value)
    .map(
        |pair| {
            Entry(
                pair.first,
                pair.second,
            )
        }
    )
```

Less desirable:

```text
Pair<A, Pair<B, Pair<C, Pair<D, E>>>>
```

So the ergonomic guidance should be:

> Nominalize meaningful grammar boundaries early.

---

# 47. Nominalizing grammar boundaries

For TOML, instead of carrying:

```text
Pair<TomlKey, Pair<TomlValue, Span>>
```

through multiple layers, create:

```telos
data TomlEntry(
    key: TomlKey,
    value: TomlValue,
    span: Span,
)
```

then map into it immediately.

This is not merely a workaround for lack of Packs.

It produces better software architecture.

---

# 48. Typed examples of the main combinators

Assume:

```text
P.Output = A
Q.Output = B
```

Then the v1 algebra should be roughly:

| Operation | Input | Output |
|---|---|---|
| `map` | `P`, `(A)->B` | `Parser<B>` |
| `flatMap` | `P`, `(A)->Q` | `Parser<Q::Output>` |
| `zip` | `P`, `Q` | `Parser<Pair<A,B>>` |
| `then` | `P`, `Q` | `Parser<B>` |
| `before` | `P`, `Q` | `Parser<A>` |
| `orElse` | `P`, `Q`, `A == B` | `Parser<A>` |
| `optional` | `P` | `Parser<Option<A>>` |
| `many` | `P` | `Parser<List<A>>` |
| `many1` | `P` | `Parser<List<A>>` |
| `sepBy` | `P`, `Q` | `Parser<List<A>>` |
| `between` | `P`, open, close | `Parser<A>` |
| `lookAhead` | `P` | `Parser<A>` |
| `attempt` | `P` | `Parser<A>` |
| `cut` | `P` | `Parser<A>` |
| `named` | `P` | `Parser<A>` |
| `spanned` | `P` | `Parser<Spanned<A>>` |

This is a compact but substantial algebra.

---

# 49. The advanced typing is not ornamental

Each major Telos typing mechanism should have a real job.

### Associated types

```telos
Source::Token
Parser::Output
```

encode fundamental semantic relationships.

### Associated specialization

```telos
P |= Parser<S, Output = Expr>
```

lets combinators constrain exact output types.

### Type equivalence

```telos
L::Output == R::Output
```

makes alternatives type-safe.

### Generic nominal types

```telos
Cursor<S>
ParseResult<S, T>
Rule<S, T>
Pair<A, B>
Spanned<T>
```

carry information across the parser pipeline.

### Callable types

```telos
(P::Output) -> U
```

power mapping.

```telos
(P::Output) -> Q
```

powers dependent sequencing.

```telos
(T, T) -> T
```

powers operator parsing.

### ADTs

```telos
ParseResult
Option
```

model control flow without exceptions.

### Exhaustive matching

Parser implementations process:

```telos
Success(...)
Failure(...)
```

and token/value variants with ordinary Telos pattern matching.

This library genuinely **consumes Telos's type system**.

---

# 50. A realistic TOML fragment

The first major consumer should be able to write grammar like:

```telos
class TomlGrammar {
    @class
    bareKey()
        -> Rule<TextSource, String>
    {
        Rule.of(
            Text.takeWhile1(
                |char| {
                    char.isAsciiLetter
                        or char.isAsciiDigit
                        or char == '_'
                        or char == '-'
                }
            )
            .named("bare key")
        )
    }

    @class
    dottedKey()
        -> Rule<TextSource, TomlKey>
    {
        Rule.of(
            simpleKey()
                .sepBy1(
                    whitespace()
                        .then(Text.char('.'))
                        .then(whitespace())
                )
                .map(
                    |parts| {
                        TomlKey(
                            parts: parts
                        )
                    }
                )
        )
    }

    @class
    keyValue()
        -> Rule<TextSource, TomlStatement>
    {
        Rule.of(
            dottedKey()
                .before(
                    whitespace()
                        .then(Text.char('='))
                        .then(whitespace())
                )
                .zip(value())
                .spanned()
                .map(
                    |parsed| {
                        TomlStatement::KeyValue(
                            parsed.value.first,
                            parsed.value.second,
                            parsed.span,
                        )
                    }
                )
                .named("key/value pair")
        )
    }
}
```

That is approximately the complexity target the public API should comfortably support.

---

# 51. A tokenized language grammar

The same package should comfortably support:

```telos
data Expr {
    @variant
    Integer(_ value: Int)

    @variant
    Name(_ name: String)

    @variant
    Add(
        _ left: Expr,
        _ right: Expr,
    )

    @variant
    Multiply(
        _ left: Expr,
        _ right: Expr,
    )
}
```

with token selection:

```telos
let integer =
    Parsers.select(
        |token| {
            match token {
                Token::Integer(value) =>
                    Some(
                        Expr::Integer(value)
                    )

                _ =>
                    None
            }
        },
        expected: "integer",
    )
```

and typed operator output:

```telos
let addition =
    Parsers.token(Token::Plus)
        .map(
            |_| {
                |left: Expr, right: Expr| {
                    Expr::Add(
                        left,
                        right,
                    )
                }
            }
        )
```

Then:

```telos
Parsers.chainLeft1(
    operand: term,
    operator: addition,
)
```

The same abstractions work for text and lexed token streams.

---

# 52. Binary parsing is still within the abstraction

If we include `BytesSource`:

```telos
impl Source for BytesSource {
    type Token = Int
}
```

then:

```telos
Binary.u16BE()
    .flatMap(
        |length| {
            Binary.take(length)
        }
    )
```

works through exactly the same `Parser<S>` interface.

That validates that the abstraction is:

> typed sequential decoding

rather than:

> string parser library.

---

# 53. Performance model

v1 does not need sophisticated parser optimization, but it should be designed sensibly.

Core expectations:

```text
cursor advance             O(1)

ordinary parser success    no source copies

text scanning primitives   contiguous where possible

many / sepBy               iterative internally

spans                       store offsets, not substrings

Rule                        one intentional erased call boundary

concrete combinators        remain statically dispatchable where possible

failure structures          concentrated on failure paths

cut                         prunes impossible alternatives
```

The implementation should avoid building a universal parser AST solely to interpret it at runtime.

---

# 54. Concrete combinators, not a giant parser GADT

One tempting implementation is:

```telos
data Parser<T> {
    Pure(...)
    Map(...)
    Zip(...)
    Choice(...)
    Many(...)
    ...
}
```

with one interpreter.

I would not make that the v1 architecture.

Instead use concrete parser types:

```text
Satisfy<S>
MapParser<S,P,U>
ZipParser<S,P,Q>
OrElse<S,L,R>
ManyParser<S,P>
...
```

all conforming to:

```telos
Parser<S>
```

Benefits:

- the host language type system represents the grammar naturally;
- combinators carry exact types;
- compiler dispatch can potentially specialize them;
- no large runtime tagged-parser interpreter is required;
- `Rule` gives controlled erasure only where needed.

---

# 55. Public namespaces

Conceptually, the package surface might be:

```text
parse
├── Parser
├── Source
├── Cursor
├── ParseResult
├── ParseError
├── Rule
├── Span
├── Spanned
├── Pair
│
├── Parsers
│   ├── pure
│   ├── fail
│   ├── satisfy
│   ├── select
│   └── end
│
├── text
│   ├── TextSource
│   ├── char
│   ├── literal
│   ├── oneOf
│   ├── takeWhile
│   └── takeWhile1
│
└── operators
    ├── chainLeft1
    └── chainRight1
```

The precise Telos module layout can follow repository conventions, but the public conceptual surface should remain compact.

---

# 56. Suggested internal module structure

```text
packages/parse/
├── project.toml
│
├── src/
│   ├── parse.ph
│   │
│   ├── core/
│   │   ├── source.ph
│   │   ├── cursor.ph
│   │   ├── parser.ph
│   │   ├── result.ph
│   │   ├── error.ph
│   │   ├── rule.ph
│   │   ├── span.ph
│   │   └── pair.ph
│   │
│   ├── primitive/
│   │   ├── pure.ph
│   │   ├── fail.ph
│   │   ├── satisfy.ph
│   │   ├── select.ph
│   │   └── end.ph
│   │
│   ├── combinator/
│   │   ├── map.ph
│   │   ├── flat-map.ph
│   │   ├── sequence.ph
│   │   ├── choice.ph
│   │   ├── optional.ph
│   │   ├── repeat.ph
│   │   ├── separated.ph
│   │   ├── lookahead.ph
│   │   ├── commitment.ph
│   │   ├── context.ph
│   │   ├── span.ph
│   │   └── operator.ph
│   │
│   ├── source/
│   │   ├── list.ph
│   │   └── text.ph
│   │
│   └── run.ph
│
├── tests/
└── examples/
```

---

# 57. Hard semantic invariants

I would make these part of the actual v1 specification.

**P1 — Output preservation.** Every parser has exactly one statically known `Output` for its `Source` conformance.

**P2 — Source token authority.** Primitive token parsers consume `S::Token`; no unrelated token type is introduced through dynamic coercion.

**P3 — Typed transformation.** `map` consumes exactly `P::Output`.

**P4 — Typed alternatives.** `orElse` requires equivalent output types.

**P5 — Typed sequencing.** `zip` computes `Pair<P::Output,Q::Output>`.

**P6 — No ordinary `Dynamic` boundary.** Routine parser composition must not erase values to `Dynamic`.

**P7 — Algebraic failure.** Ordinary parse failure is represented by `ParseResult`, not exceptions.

**P8 — Stable input.** Running parsers observe an immutable logical source sequence.

**P9 — Cursor persistence.** Backtracking restores positions, not input state.

**P10 — Explicit commitment.** Committed failure prevents alternative branches.

**P11 — Explicit exceptional backtracking.** Backtracking after meaningful consumption requires `attempt`.

**P12 — Repetition progress.** Successful repetition iterations must advance the cursor.

**P13 — Typed erasure.** `Rule<S,T>` may erase implementation type but never `S` or `T`.

**P14 — Span fidelity.** `spanned()` records the exact consumed source interval.

**P15 — Furthest-error preference.** Alternative failures report the most advanced relevant failure.

**P16 — No C4 dependency.** All v1 semantics are implementable using currently established Telos language features.

---

# 58. The v1 boundary

A useful way to state the boundary is:

### Definitely v1

```text
typed parser core
text source
list/token source

map
flatMap

zip
then
before

choice
optional

many
many1
repeat
sepBy
sepBy1

between

lookAhead
negative lookahead

attempt
cut

named
spanned

Rule / recursion

chainLeft1
chainRight1

structured errors
full-run helpers
```

### Explicitly post-v1

```text
packrat/memoization
left-recursion support
incremental parsing
streaming input
async parsing
general parser state
IDE error recovery
lexer generator
regex engine
parser derivation
reflection-driven decoding
Pack-based heterogeneous sequencing
static grammar computation
parser optimizer / compiler
```

That is a strong boundary: v1 remains implementable and reviewable, but it is absolutely not a toy.

---

# 59. Success criterion

I would consider `parse` v1 complete when these four substantially different clients can be implemented cleanly:

1. **TOML grammar**
  - recursive values;
  - strings;
  - arrays;
  - inline tables;
  - document syntax;
  - precise spans.

2. **Expression parser**
  - lexed token source;
  - recursive parentheses;
  - typed operator functions;
  - precedence and associativity.

3. **Command DSL**
  - alternatives;
  - optional sections;
  - good errors.

4. **Length-prefixed binary example**
  - if `BytesSource` is available;
  - otherwise equivalent typed list-token example;
  - demonstrates `flatMap`.

The TOML library is the decisive one. If a production-quality general TOML parser can naturally be built on `parse`, then the abstraction has proven itself.

---

# 60. Final characterization

`parse` v1 should not feel like an attempt to recreate Haskell abstractions for their own sake, nor like an elaborate demonstration of Telos generics.

It should feel like a normal Telos library whose problem domain **forces** useful type relationships:

```text
Source
   │ associated Token
   ▼
Parser<S>
   │ associated Output
   │
   ├── map ─────── exact transformation
   ├── zip ─────── heterogeneous product
   ├── orElse ──── output equivalence
   ├── flatMap ─── typed value-guided parsing
   ├── many ────── homogeneous collection
   ├── spanned ─── generic output lifting
   └── Rule ────── typed abstraction boundary
```

That is the right ambition for v1: **small enough to implement now, large enough to be production-meaningful, and uncompromising about using Telos's advanced existing type system to make invalid parser compositions difficult or impossible to express.**