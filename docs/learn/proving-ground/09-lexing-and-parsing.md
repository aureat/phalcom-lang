# 09 — Lexing and Parsing

Turning bytes into a tree, and being useful when you cannot. The through-line: *the front
end is the only part of a compiler that must produce something valuable for input that is
wrong, and almost every hard decision here is downstream of that.*

Questions first. Answers below. Do not scroll.

---

## Questions

### Q1 — The split that leaks

Four programs the lexer cannot tokenize alone:

```c
a * b;                  // C: declaration if a is a typedef, multiplication otherwise
```
```js
x = a /re/g.test(s);    // JS: is `/` division or the start of a regex literal?
```
```cpp
vector<vector<int>> v;  // C++03: `>>` is the shift operator
```
```js
let await = 1;          // legal in a script, not in a module
```

1. The textbook pipeline says lexing is a total function from bytes to tokens, run before
   parsing. Each of these breaks it in a different way. Sort them into the ones that need
   *semantic* feedback and the ones that need only *syntactic* feedback, and say why the
   distinction matters for an IDE.
2. Name the industrial fix for the C case and the industrial fix for the JS case. They are
   different in kind — say how.
3. Contextual keywords (`await`, `async`, `yield`, `var`, `some`, `record`) are the
   deliberate version of this problem. Describe how the token stream represents them and
   what that choice permanently costs you.

### Q2 — Longest match, wrongest match

```c
int x = 10, y = 3;
int z = x---y;          // what does this compute?
```
```lua
print(1..2)             -- error: malformed number near '1..'
```

1. State the rule that produces both of these, and explain why it is the rule despite
   producing obviously unintended results.
2. C++11 made `vector<vector<int>>` legal. The fix was not in the lexer. Say where it was
   and why fixing it in the lexer was not an option.
3. You are designing a language with a range operator and float literals. Enumerate your
   options for making `1..2` work, and say what each one costs elsewhere in the grammar.

### Q3 — Binding powers

A Pratt / precedence-climbing parser keeps a table:

```
token   prefix   infix(lbp)   assoc
-        prefix       10       left
*                     20       left
^                     30       right
!                             postfix
```

1. Right associativity is usually implemented as "recurse with `lbp - 1`". Derive why that
   single subtraction produces right nesting, from the loop condition.
2. Pratt keys behaviour on the token's *position* (does it start an expression, or continue
   one) rather than on the token itself. Name two constructs this makes trivially uniform
   that a hand-rolled precedence cascade handles awkwardly.
3. Precedence climbing is standard in real compilers for expressions and essentially never
   used for statements or declarations. Say why the technique stops paying there.

### Q4 — Which minus is it

```python
>>> -2 ** 2
```
```haskell
ghci> -2 ^ 2
ghci> 2 ^ -1
```

1. Predict all three results, including which one is not a value.
2. `-` is both prefix and infix, and the two need different precedences relative to `^`.
   Explain how a Pratt table expresses that without any special case, and what the naive
   "unary minus has the highest precedence" answer gets wrong.
3. Haskell's unary minus is a documented special case in the language report rather than an
   ordinary operator. Reconstruct why: what property of Haskell's operator system makes
   prefix `-` impossible to define like everything else?

### Q5 — Four parsing technologies

You must choose: hand-written recursive descent, an LALR generator, a PEG, or GLR.

1. LALR's conflict reports are usually described as a nuisance. Argue instead that they are
   the technology's main *product*, and say what you lose by adopting a formalism that never
   reports one.
2. A PEG's ordered choice `A / B` never backtracks into `A` once it succeeds. Give a
   concrete grammar where this silently changes the language accepted versus the same rules
   as a CFG alternation, and say why it is worse than an error.
3. CPython replaced its LL(1) parser with a PEG parser; tree-sitter uses GLR; almost every
   production *compiler* uses hand-written recursive descent. Explain what makes an editor's
   requirements different enough that GLR wins there and loses in a compiler.

### Q6 — Dangling else

```c
if (a) if (b) f(); else g();
```

1. This is genuinely ambiguous in the grammar as usually written. Name the two legitimate
   ways to resolve it *within* parsing technology, and say which one yacc applies by default
   and by what mechanism.
2. There is a third resolution that is not a parsing decision at all. Name it, name two
   languages that took it, and say what it costs.
3. A grammar-level fix requires splitting the statement nonterminal in two. Sketch it, and
   explain why this fix scales badly as the language grows.

### Q7 — The offside rule

Python:

```python
if x:
    if y:
        f()
  g()          # what error, and from which compiler phase?
```

1. Describe the algorithm that turns leading whitespace into `INDENT`/`DEDENT` tokens, and
   say precisely which component reports the error above.
2. Python suppresses newlines inside brackets ("implicit line joining"). State what that
   forces the lexer to track, and why that is architecturally uncomfortable.
3. Haskell's layout rule is specified with a side condition involving a *parse error*. State
   what that means operationally, and why it makes the clean lexer/parser pipeline
   impossible for Haskell specifically.

### Q8 — Automatic semicolon insertion

```js
function f() {
  return
    { ok: true };
}
```
```js
const a = b
[1, 2].forEach(g)
```

1. Say what each of these does, and name the two distinct ASI rules responsible — they are
   not the same rule.
2. ASI is defined as *error correction*: insert a semicolon when the offending token cannot
   be parsed, plus a list of restricted productions. Explain why defining a language feature
   in terms of parse failure is the root problem, not merely an implementation detail.
3. Go also has automatic semicolons and nobody complains. State the mechanical difference,
   and name the visible syntactic restriction Go accepted in exchange.

### Q9 — The most vexing parse

```cpp
Widget w(Thing());     // declares a function, not a variable
```

1. Explain what `Thing()` is being read as, and state the disambiguation rule in the
   standard that forces this reading.
2. This is not a lexer/parser-split problem and it is not fixable by more lookahead. Justify
   both claims.
3. C++11's braced initialization fixes it for new code. Say why the committee could not
   simply change the rule, and state the general principle about grammar ambiguity and
   language evolution that this illustrates.

### Q10 — Recovering from a missing paren

```rust
fn main() {
    let x = foo(1, 2;
    let y = 3;
    bar(y);
}
```

1. Compare three recovery strategies on this input: panic-mode with a synchronization set,
   an explicit error production, and "insert the token that was expected." For each, say
   what tree comes out and what the next diagnostic will be.
2. Name the two invariants a recovery strategy must satisfy to be usable at all — one about
   termination, one about what it hands downstream.
3. Argue that the goal of recovery is *not* to produce a correct tree, and state what the
   goal actually is. Then say how that goal changes when the consumer is an IDE rather than
   a compiler.

### Q11 — Why the first error is the only one

A missing `}` in a large file produces 200 diagnostics. The user fixes one character and
all 200 disappear.

1. Explain the mechanism of the cascade — why one structural mistake generates errors far
   away rather than one error at the mistake.
2. Clang caps diagnostics by default and additionally tracks whether an expression "contains
   errors" to suppress downstream complaints. Explain what that bit is doing, and why
   suppression is better than just capping the count.
3. Given that the first error dominates, argue for a specific ordering guarantee your
   compiler should make, and name the case where reporting errors strictly in source order
   is the wrong choice.

### Q12 — Expected X, found Y

```
error: expected one of `!`, `.`, `::`, `;`, `?`, `{`, `}`, or an operator, found `let`
```

1. Explain where a list like that comes from mechanically, and why it is nearly always
   useless to the reader.
2. rustc's better diagnostics for unclosed delimiters point at the *opening* brace, often
   hundreds of lines away, and name the construct. Say what the parser must have retained to
   make that possible, and what a naive parser throws away that makes it impossible.
3. Spans are byte ranges. Name two things a span must be attached to beyond "the whole
   expression" for good diagnostics, and say what multi-file compilation forces about how
   spans are represented.

### Q13 — A tree for broken input

The user has typed:

```rust
fn foo(a: u32, b: 
```

and the editor must offer completion for `b`'s type.

1. State the hard requirement this puts on the parser, in one sentence, and name the two
   node kinds a resilient parser needs that a batch parser does not.
2. Reparsing on every keystroke must be cheap. Describe the representational decision that
   makes subtree reuse possible across an edit, and say what it forbids storing in a node.
3. C++ IDEs are notoriously worse at this than Rust or C# IDEs. Connect that fact to Q1 and
   explain the causal chain.

### Q14 — Two trees for one file

A formatter must reproduce every space and comment. A type checker wants none of them.

1. Say what a lossless syntax tree guarantees, in the form of an equation. Then name the
   industrial pattern that lets one tree serve both consumers, and name two implementations
   of it.
2. Trivia attachment is the hard part: a comment between two tokens must belong to exactly
   one of them. State a concrete attachment rule, then give the user-visible bug that a
   *wrong but consistent* rule produces.
3. Argue that a compiler that discards comments and whitespace at lex time has made an
   irreversible decision, and name three tools it has thereby made harder to build.

### Q15 — Identifiers that are not what they look like

Two functions whose names render identically in most fonts. A comment containing a
right-to-left override so that the displayed source and the compiled source disagree.

1. Normalization: state what goes wrong if the lexer treats identifiers as raw code-point
   sequences, and what NFC buys. Then name the new problem normalization introduces at the
   boundary with the filesystem or with FFI symbol names.
2. The bidi-override attack (Trojan Source) is not a parsing bug — the parser is correct.
   Say precisely what is wrong and where the fix belongs, and describe what a compiler
   actually shipped.
3. Supporting non-ASCII identifiers means the lexer embeds Unicode property tables. Name
   the versioning hazard that creates, and say what it does to the claim that a language has
   a stable grammar.

### Q16 — Operators the user declares

```haskell
infixl 6 <+>
```
```swift
infix operator <+> : AdditionPrecedence
```
```scala
def <+>(that: V): V     // precedence from the first character
```

1. User-declared fixity creates a circularity with parsing. State it precisely, and describe
   how GHC breaks it.
2. Swift's precedence groups form a partial order, and comparing two unrelated groups is an
   error rather than a default. Say what that prevents, and what it costs a library author.
3. Scala derives precedence from the operator's first character — no declarations at all.
   Name what this buys the tooling, and give the failure mode it accepts.

### Q17 — Nobody generates their parser

GCC's C++ front end was moved from bison to hand-written recursive descent. Clang, rustc,
Roslyn, the Go compiler, TypeScript, V8, and Lua are all hand-written.

1. Give three reasons that are about the *front end's product*, not about performance.
2. Name the two things you genuinely lose, and describe a discipline that recovers part of
   one of them.
3. Every one of those projects still publishes a grammar. Say what the published grammar is
   actually for, given that no code is generated from it, and name the failure mode of
   keeping one.

---

## Answers

### A1 — The split that leaks

**1.** **Semantic feedback**: the C `a * b` case. Deciding whether `a` is a type name
requires a symbol table, which requires having processed declarations, which requires
parsing — a genuine cycle through semantic analysis. **Syntactic feedback**: the JS regex
case and the C++ `>>` case. In JS, whether `/` begins a regex depends only on the parser's
current state (is an expression expected, or an operator?), which is available without any
name resolution. `await` as a contextual keyword is also purely syntactic — it depends on
goal symbol (script vs. module) and on whether you are inside an async function.

This matters enormously for an IDE, because an IDE must produce a usable tree for a file
whose *semantics are not available yet* — dependencies not built, the symbol table partial
or stale, the user mid-keystroke. A language whose tokenization depends only on parser state
can always be lexed; a language whose tokenization depends on the symbol table cannot be
lexed correctly until a build has happened. This is a large part of why C++ tooling lags.

**2.** For C: **the lexer hack** — the parser feeds the symbol table back into the lexer, so
the lexer emits `TYPEDEF_NAME` rather than `IDENTIFIER` for names currently in scope as
types. That is a *data* dependency on semantic analysis, and it makes the lexer stateful in
a way that is scope-sensitive and hard to run backwards. For JS: **parameterized lexical
goals** — the ECMAScript grammar defines distinct lexical goal symbols
(`InputElementDiv`, `InputElementRegExp`, and friends), and the parser tells the scanner
which one to use at each point. That is a *control* dependency on the parser's state only.
The difference in kind: one lexes on demand from a parser that knows what it wants, which
composes fine and is re-runnable; the other requires a semantic side table, which does not.

**3.** They are lexed as ordinary identifiers and the *parser* decides, based on position
and context, whether this occurrence is the keyword. The permanent cost is that **you can
never promote them to reserved words**, because the whole point was that existing programs
using them as names keep working — so every future grammar rule involving that word must
remain unambiguous against its use as an identifier, forever. Secondary costs: syntax
highlighting cannot colour them correctly without parsing (an editor that highlights on the
token stream will colour a variable named `record` as a keyword or vice versa), and error
messages degrade, because `await x` in a non-async function is not a syntax error at the
token level — it is `await` applied to `x`, and you get a confusing message about a missing
operator rather than "await outside async."

**Trap.** Saying the lexer hack is "just a hack" and a well-designed language avoids it.
The real content is *which direction the dependency runs*: parser→lexer state is cheap and
composes; symbol-table→lexer is what breaks tooling. A candidate who names the direction is
saying something; a candidate who says "C is badly designed" is not.

### A2 — Longest match, wrongest match

**1.** **Maximal munch**: at each position, the lexer takes the longest sequence of
characters that forms a valid token, with no regard for whether the resulting token stream
parses. So `x---y` lexes as `x`, `--`, `-`, `y` (postfix decrement of `x`, then binary
minus) — which is legal C and computes `x-- - y`, i.e. `10 - 3 == 7`, leaving `x == 9`. In
Lua, `1..2` starts a number at `1` and the number scanner munches `1..` greedily before
failing, so you get a malformed-number error rather than concat. It is the rule because the
alternative — letting the parser guide token boundaries — makes the lexer's output
non-deterministic and its cost unbounded, and because for the overwhelming majority of
input, longest-match is what you want (you want `==` not `=` `=`, `>=` not `>` `=`).

**2.** In the **parser**, as a special rule in the template-argument-list production: when
parsing template arguments, a `>>` token is treated as two `>` tokens closing two lists.
Fixing it in the lexer was not an option because `>>` genuinely *is* the shift operator
outside template argument lists, and the lexer does not know whether it is inside one —
that is parser state. Splitting `>>` unconditionally would break every shift expression, and
splitting it conditionally requires exactly the parser feedback the lexer does not have.
Note that this is the same shape as A1's JS case, solved the other way: instead of asking
the parser which lexical goal to use, the parser accepts a coarse token and re-interprets it
locally.

**3.** Options, with costs:

- **Require whitespace** (`1 .. 2`). Cheapest to implement, but now whitespace is
  significant inside expressions, which surprises everyone and interacts badly with
  formatters and with macro-generated code.
- **Forbid trailing-dot float literals** — require `1.0` and never allow `1.`. This is
  Rust's neighbourhood: it makes `1..2` unambiguous by shrinking the float grammar. Costs a
  small convenience and closes a door on `1.` forever.
- **Lexer lookahead / backtracking**: on seeing `1..`, check whether the next char is a
  digit; if not, emit integer `1` then `..`. This is a targeted maximal-munch violation and
  works well, but it is a precedent — every such special case is one more place the token
  stream is not a pure longest-match function, and they interact.
- **Pick a different operator** (`...`, `to`, `:`). Free at the lexer, costs you a
  character or a keyword, and `:` collides with type annotations, slices, and labels.

The general lesson: the lexer's problems are almost always solved by *changing the surface
syntax*, and the cost of not doing so is a permanent special case.

### A3 — Binding powers

**1.** The loop is `while lbp(peek) > min_bp { advance; rhs = parse(rbp) }`. For a
left-associative operator you recurse with `rbp = lbp`, so when the parser returns to the
loop and sees the *same* operator again, `lbp > min_bp` is false — it does not recurse, it
loops, and the second occurrence attaches to the already-built left node: `(a-b)-c`. For a
right-associative operator you recurse with `rbp = lbp - 1`, so an identical operator seen
inside the recursive call satisfies `lbp > lbp - 1` and *is* consumed by the inner call:
`a^(b^c)`. The subtraction is doing one job — deciding whether an equal-precedence operator
belongs to the inner or the outer invocation. Equivalently, left-assoc means "strictly
greater binds tighter to the right," right-assoc means "greater-or-equal does."

**2.** (a) **Prefix and postfix operators**, including mixed ones like `-` and `!`, because
the table has separate prefix and infix entries for the same token and the parser consults
whichever the position calls for — no duplicated cascade of `unary_expr`, `postfix_expr`,
`primary_expr` levels. (b) **Anything that continues an expression**: calls `f(x)`, indexing
`a[i]`, member access `a.b`, ternaries, and even `as`/`is` casts are all just infix or
postfix entries with binding powers. In a precedence cascade each of these is a separate
nonterminal in a fixed chain, and inserting a new precedence level means editing every
adjacent rule; in a Pratt table it is one row. That is the actual reason Clang, rustc, and
Go all use precedence-based expression parsing while their statement parsers are plain
recursive descent.

**3.** Because precedence climbing is a solution to a specific problem: **a flat sequence of
operands separated by operators, where the only question is nesting.** Statements and
declarations do not have that shape. They are led by a distinguishing keyword or a
determinable prefix, so there is nothing to disambiguate by precedence — a `while` statement
is recognized by seeing `while`, and its structure is fixed. Recursive descent expresses
that directly and readably: one function per construct, reading like the grammar. Applying
precedence machinery to statements would be encoding a non-problem. The division of labour
— recursive descent for the language's structure, Pratt for expressions — is the standard
answer precisely because the two halves have different shapes.

### A4 — Which minus is it

**1.** Python `-2 ** 2` is **-4**: `**` binds tighter than unary minus on its left, so it is
`-(2**2)`. Haskell `-2 ^ 2` is **-4**, for the corresponding reason. Haskell `2 ^ -1` is a
**parse error** — not a value at all — because prefix `-` cannot appear as the right operand
of an infix operator without parentheses; you must write `2 ^ (-1)`.

**2.** The table gives `-` two independent entries: a **prefix** entry with its own binding
power used when `-` appears where an expression is expected, and an **infix** entry with a
different binding power used when `-` appears where an operator is expected. The prefix
power is compared against the *right-hand* operand's operators, so setting prefix-minus
below `^` yields `-(2**2)` and setting it above yields `(-2)**2`. No special case is needed
— the two roles were never the same table row.

What "unary minus has the highest precedence" gets wrong: it predicts `(-2)**2 == 4`, which
is false in Python, Haskell, Ruby, and Fortran. Exponentiation is the one binary operator
conventionally placed *above* unary minus, matching mathematical notation where $-x^2$ means
$-(x^2)$. So the naive rule is wrong in exactly the case people test.

**3.** Because in Haskell **every operator is a function and can be sectioned**, and prefix
`-` cannot be, without colliding with the section syntax. `(- x)` would have to mean "the
section that subtracts `x`", by the same rule that makes `(+ x)` a function. Haskell resolves
this by making `-` the sole reserved exception: `(- x)` is negation, `(subtract x)` exists
because the section you wanted is unavailable, and prefix `-` is defined in the report as a
grammatical special case at precedence 6 rather than as an operator you could have declared.
The general principle: a language that lets users declare operators (Q16) cannot also have
ad-hoc prefix forms of those same operators, because the parser has no way to tell a prefix
occurrence from a section. Haskell paid for user-defined operators with this one wart, and
the wart is unavoidable given the two features.

**Trap.** Answering the first part with "-4, -4, 0.5" — reading Haskell as Python. Haskell's
`2 ^ -1` being a *syntax* error, not a type error about negative exponents, is the whole
point of the question, and it follows directly from part 3.

### A5 — Four parsing technologies

**1.** A shift/reduce or reduce/reduce conflict is a **machine-checked proof that your
grammar is ambiguous or requires more context than the formalism has**. That is a design
signal you cannot get any other way: the tool is telling you that a human reader will also
be confused here, before you ship the syntax. When you adopt a formalism that never reports
a conflict — PEG, or hand-written recursive descent — you have not removed the ambiguity,
you have removed *the report*. The ambiguity is still in the language, now silently resolved
by whatever your rule ordering happens to be, and it will surface as a user bug report years
later. The honest statement of the trade is: generators tell you about problems in your
grammar and are bad at telling users about problems in their programs; hand-written parsers
are the reverse.

**2.**

```
ident  <- "if" / [a-z]+
```

As a CFG alternation over a longest-match tokenizer this is fine. As a PEG, `ident` applied
to `iffy` matches `"if"` first, succeeds, and returns having consumed two characters —
`ident` never matches `iffy` as a whole, and no backtracking will ever revisit that choice
because the *enclosing* rule saw a success. Same disease in the classic
`expr <- term / term "+" expr`: the first alternative always wins and the addition rule is
dead. It is worse than an error because PEG guarantees a parse — you get a tree, it is
plausible, and the language you actually accept is not the one you wrote. An LALR generator
would have reported a conflict; a PEG reports success. This is why PEG grammars require you
to order alternatives longest-first as a discipline, and why that discipline is invisible in
the grammar text.

**3.** An editor needs: a tree for *every* input including broken ones, incremental reparse
on each keystroke, and no dependency on semantic information (A1). GLR is a good fit because
it explores multiple parses in parallel and can carry ambiguity forward rather than
committing, and tree-sitter layers error recovery and subtree reuse on top; the grammar
being declarative also means one implementation serves hundreds of languages, which is the
actual product. A compiler needs the opposite bundle: **precise, actionable diagnostics**,
the ability to consult semantic state mid-parse, and a single canonical tree. GLR gives you
a parse forest you must then disambiguate, and it makes "why did this fail, and what did you
mean" nearly impossible to answer well, because the failure is a property of the whole
forest collapsing rather than of a specific expectation at a specific token. CPython's PEG
migration is instructive on this point: PEG lifted the LL(1) restrictions that had been
distorting the grammar for decades, but the team then had to add a separate pass of explicit
`invalid_*` rules, reparsing failed input a second time purely to produce good error
messages — because the generated parser's natural failure output was unusable.

**Trap.** "PEGs are unambiguous by construction." True and irrelevant — they are unambiguous
because ordered choice *defines away* ambiguity by fiat, not because your grammar was
unambiguous. The property you want is that the grammar means what you think, and PEG gives
you no evidence about that.

### A6 — Dangling else

**1.** (a) **Rewrite the grammar** so that only a fully-matched statement may appear in the
`then` branch of an `if` with an `else` (below). (b) **Resolve the conflict in the parser
generator** — yacc reports a shift/reduce conflict here and defaults to **shift**, which
attaches the `else` to the nearest `if`, matching every mainstream language's rule. The
mechanism is exactly that: a documented default for shift/reduce conflicts, which is why the
dangling else compiles with a warning rather than an error in a bison grammar. Note that
relying on this means the grammar file no longer fully specifies the language — the
generator's tie-break does.

**2.** **Change the syntax so the ambiguity cannot arise**: require braces on the branches.
Go and Rust both do this — `if a { if b { f() } } else { g() }` has no ambiguity because the
block delimiters make the nesting explicit. The cost is verbosity on the one-line `if`, which
is why both languages also removed the parentheses around the condition to compensate, and
why Go's `gofmt` exists partly to stop anyone arguing about it. Perl and Python's
`elif`/`else` chaining, and Ada's `end if`, are the same move with different delimiters.

**3.**

```
stmt        -> matched | unmatched
matched     -> "if" e "then" matched "else" matched | other
unmatched   -> "if" e "then" stmt
             | "if" e "then" matched "else" unmatched
```

It scales badly because the split is not local to `if`: **every** statement form that can
end with an optional trailing clause must be duplicated into matched and unmatched variants,
and the duplication multiplies when you have several such forms (a `try` with optional
`catch`/`finally`, a `loop` with an optional `else` as in Python). You are encoding "can
this statement swallow a following keyword?" into the nonterminal name, which is a boolean
you have to thread through the whole statement grammar. That is why real grammars overwhelmingly
choose (1b) or (2) instead — the CFG-purist fix is correct and unmaintainable.

### A7 — The offside rule

**1.** The lexer keeps a **stack of indentation columns**, initialized to `[0]`. At the start
of each logical line, measure the indent. If it is greater than the top of stack, push it and
emit one `INDENT`. If equal, emit nothing. If less, pop and emit one `DEDENT` per popped
level until the top equals the new indent — and **if no stack entry equals it, that is an
error**. At end of file, emit `DEDENT` for every remaining level. The example dedents to
column 2, which matches neither 4 nor 0, so it is that "no matching level" case:
`IndentationError: unindent does not match any outer indentation level`, reported by the
**tokenizer**, not the parser. That is worth being able to say precisely — indentation
errors in Python are lexical errors, which is why they are reported before any syntax error
later in the file.

**2.** It forces the lexer to track **bracket nesting depth** — a count of unclosed `(`,
`[`, `{` — and to suppress `NEWLINE` (and indentation processing entirely) whenever the
depth is nonzero. That is uncomfortable because bracket matching is *parsing*: the lexer is
now maintaining a piece of the parser's state, and the two can disagree. Concretely, a
stray unmatched `(` on line 3 means every subsequent line of the file is treated as a
continuation, indentation is ignored to end of file, and the reported error is at the end of
the file rather than at line 3 — which is exactly the cascade problem of Q11, arising in the
lexer. It is also why the `\` line continuation exists separately: it is the lexical form for
the case where you are not inside brackets.

**3.** Haskell's layout algorithm includes a rule usually written as: if the enclosing
context requires it and the tokens so far constitute a parse error, insert a virtual close
brace. Operationally, **the lexer's output depends on whether the parser fails** — the layout
algorithm must be able to ask "would the parser reject the next token here?" and change its
tokenization accordingly. There is no way to run this as a pipeline: you cannot finish lexing
before parsing begins, and you cannot even run them as coroutines cleanly, because the query
is about a hypothetical failure rather than about parser state. Implementations approximate
it — GHC handles the common cases (`let ... in`, `where`) with targeted rules and parser
feedback rather than implementing the side condition literally. The lesson is that a layout
rule defined by "whatever makes it parse" is a specification that cannot be implemented as
written, and it is the single most-cited wart in the Haskell report.

**Trap.** Describing `INDENT`/`DEDENT` and stopping there. The interesting content is that
the lexer needs bracket depth (part 2) and, in Haskell's case, parser feedback (part 3) —
i.e. significant whitespace does not merely add tokens, it moves parsing responsibility into
the lexer. That is the trade the question is about.

### A8 — Automatic semicolon insertion

**1.** The first returns **`undefined`** — a semicolon is inserted after `return`, and the
object literal becomes unreachable code. The second is **not** two statements: no semicolon
is inserted after `b`, so it parses as `const a = b[1, 2].forEach(g)`, indexing `b` with the
comma expression `(1,2)` → `b[2]`. Two different rules: the first is a **restricted
production** — the grammar explicitly forbids a line terminator between `return` and its
argument, so a newline there *forces* a semicolon regardless of whether the next line would
have parsed. The second is the **offending-token rule** — a semicolon is inserted only when
the next token cannot be parsed as a continuation, and `[` can continue an expression, so
nothing is inserted. The rules point in opposite directions, which is why semicolon-less
style guides tell you to prefix lines starting with `(`, `[`, `` ` ``, `+`, `-`, or `/` with
a semicolon.

**2.** Because "the offending token cannot be parsed" is a property of **the entire rest of
the parse**, not of the local text. Whether a semicolon appears after line N depends on what
line N+1 starts with, which means a purely local edit on a later line changes the meaning of
an earlier line. That makes the rule impossible to internalize (you cannot look at a line and
know whether it ends), impossible to check locally in a linter without full parsing, and it
means adding a new syntactic form to the language can *retroactively change the meaning of
existing programs* — if a token that previously could not continue an expression becomes able
to, a semicolon that used to be inserted no longer is. That last consequence is the reason
TC39 treats new prefix-position syntax with extreme care. A feature defined as recovery from
failure inherits every future change to what counts as failure.

**3.** Go inserts semicolons **in the lexer, unconditionally, based only on the last token of
the line**: if a line's final token is an identifier, a literal, one of a small set of
keywords (`break`, `continue`, `fallthrough`, `return`), `++`, `--`, or a closing `)`, `]`,
`}`, insert a semicolon. No parser involvement, no lookahead, total function of the token
stream. You can determine locally and mechanically whether a line ends. The price is a
visible syntactic restriction: **the opening brace must be on the same line as the construct**
— `func f()\n{` inserts a semicolon after `)` and breaks, so Go bans the Allman brace style
outright. That is the whole trade, and it is the right one: a small fixed syntactic
constraint in exchange for a rule that is local, total, and cannot be destabilized by future
grammar additions.

**Trap.** "ASI is fine if you just always write semicolons." It is not, because the
restricted productions fire regardless: `return\n  value` is broken whether or not you use
semicolons elsewhere, as is a newline between a variable and its postfix `++`. The
restricted-production half of ASI is not opt-out.

### A9 — The most vexing parse

**1.** `Thing()` is being read as a **parameter declaration**: an unnamed parameter of type
"function taking no arguments and returning `Thing`", which then decays to a function
pointer. So the whole line declares `w` as a function taking such a parameter and returning
`Widget`. The rule is the standard's disambiguation: **if a construct can be parsed as a
declaration or as an expression statement, it is a declaration.** The same rule makes
`Widget w();` a function declaration rather than a default-constructed variable.

**2.** Not a lexer/parser-split problem: every token here is unambiguous, and no amount of
symbol-table feedback helps — `Thing` is a type name in both readings. Not fixable by
lookahead: the two parses are **complete, valid derivations of the entire construct** in the
same grammar. Lookahead resolves cases where a longer prefix picks out a unique production;
here, arbitrarily long lookahead still admits both, because `Widget w(Thing(), Other());` is
still both a declaration and (in principle) a call. The ambiguity is in the language, and it
must be resolved by a rule *outside* the grammar — which is exactly what the standard's
prose disambiguation is.

**3.** Because the "prefer declaration" rule is load-bearing for enormous amounts of existing
code: flipping it would change the meaning of valid programs silently, with no diagnostic,
which is the worst class of breaking change. C++11 instead added a **new syntax** —
`Widget w{Thing{}}` — that is unambiguous by construction, leaving the old spelling's meaning
untouched. The general principle: **once a grammar ambiguity has shipped with a
disambiguation rule, the rule is permanent, and the only remedy is new syntax.** You cannot
fix ambiguity retroactively, because both readings have users. This is the strongest argument
for taking a generator's conflict reports seriously (A5) during design: a conflict you
tie-break in version 1 becomes a rule you can never revisit.

### A10 — Recovering from a missing paren

**1.**

- **Panic mode with a synchronization set** (`;`, `}`, statement-start keywords): report the
  error at `;`, then discard tokens until a synchronizing token. Since `;` is itself in the
  set, you resynchronize immediately and continue at `let y = 3;`. The tree contains a
  malformed `let x` statement, and the *next* diagnostic is likely none at all — this input
  recovers cleanly. Change the input to a missing `}` at the end of a function and the same
  strategy skips a great deal and produces junk.
- **Error production**: if you wrote a rule like `call -> ident "(" args ")" | ident "(" args`
  you get a precise diagnostic ("unclosed argument list") and a well-shaped call node with a
  flag. The cost is that you must anticipate the mistake, the grammar grows a rule per
  anticipated mistake, and error productions frequently introduce conflicts with real rules.
- **Insert what was expected**: the parser wants `)`, does not see it, so it synthesizes a
  *missing* `)` token with a zero-width span, reports "expected `)`", and continues as if it
  were there. The tree is well-formed, with one node marked missing, and parsing continues
  normally into `let y = 3;`. For this input it is clearly the best of the three, and it is
  what modern resilient parsers do.

**2.** (a) **Termination**: every recovery path must consume at least one token, or you loop
forever on the same offending token. This is why "insert what was expected" needs a guard —
you cannot keep inserting tokens indefinitely without advancing. (b) **Downstream honesty**:
whatever tree you hand to the next phase must be *marked* as containing errors, so name
resolution and type checking do not report consequences of your invention as if they were
the user's mistakes. A recovery that produces a plausible-looking but fabricated node with no
error marker is worse than no recovery at all.

**3.** The goal is to **keep finding real errors** — to reach the rest of the file in a state
where genuine, independent mistakes are still detected and reported, so the user fixes five
things per compile instead of one. Correctness of the tree is not achievable (you do not know
what they meant) and not the point. When the consumer is an **IDE**, the goal changes: now
the tree must support *queries* — completion, hover, go-to-definition — for the region around
the cursor, which is precisely the broken region. An IDE would rather have a well-formed tree
with an explicit hole at the cursor than a correct tree of the rest of the file, and it does
not care about finding additional errors at all. Same machinery, opposite priorities, which is
why a compiler front end retrofitted into an IDE usually disappoints.

### A11 — Why the first error is the only one

**1.** A structural token — a brace, a paren, a `begin`/`end` — establishes *context* for
everything that follows. When one is missing, the parser's state is wrong from that point on,
so every subsequent construct is interpreted against the wrong expectation: statements are
read as being inside a function that has not ended, a `fn` keyword appears where the parser
expects an expression, a closing `}` closes the wrong thing. Each of those is a genuine
mismatch against the parser's state, so the parser is not malfunctioning — it is faithfully
reporting that the token stream does not match the grammar, having lost track of which
grammar position is correct. The errors are all real, and all consequences of one cause. Note
the same phenomenon in a lexer, from A7: one unmatched `(` in Python makes the rest of the
file a continuation line.

**2.** The "contains errors" bit **poisons** a node: once an expression is built from a
subexpression that failed, it is marked, and later phases check the mark before diagnosing.
A poisoned expression has no reliable type, so a type error involving it is meaningless, and
a name lookup failure inside it may be a consequence of the parser having invented a node.
Suppression is better than capping because a cap is *positional* — it silences the last N
errors regardless of whether they are independent, so a file with one brace error and one
genuine type error on the last line loses the type error. Suppression is *causal*: it
silences the errors that are consequences and keeps the ones that are not, so unrelated
errors elsewhere in the file still surface. The cap is a backstop for when suppression fails,
not a substitute.

**3.** The guarantee worth making: **the first reported diagnostic is the one closest to the
actual cause, and everything after it is best-effort.** Concretely, that means never letting a
later phase's diagnostics be printed ahead of a parse error, and never reporting a derived
error before its cause. Where strict source order is wrong: **an unclosed delimiter**. The
parse fails at end of file, which is where source order would put the message, but the
useful location is the opening delimiter far earlier — so the primary span must be the
opener, out of source order relative to where the parser noticed. Same for "this `}` closes
the `impl` block, did you mean it to close the `fn`?" — the useful report is anchored where
the user's intent diverged, not where the machine discovered it.

### A12 — Expected X, found Y

**1.** It is the **set of terminals with a shift or reduce action in the parser's current
state** (for a generated parser), or the accumulated set of tokens the hand-written parser
tried to match before giving up. It is useless because that set is a description of the
*machine's* position, not of the user's intent: it lists everything syntactically permissible,
which at most points in an expression grammar is a dozen unrelated things, and it says
nothing about *why* those are permissible or which one the user probably wanted. The reader
gets a menu when they needed a diagnosis. It is also frequently misleading — the token
`found` is often perfectly fine and the actual mistake was several lines earlier (A11).

**2.** The parser must have retained a **stack of open delimiters with their spans and the
construct each one opened** — not merely a counter, and not merely the fact that a brace is
open, but the source position and the syntactic role ("this `{` opened a `fn` body at line
40"). A naive parser matches delimiters implicitly through its recursion and throws the
opener's position away as soon as it recurses, so at the point of failure the only position
it has is the current one. Keeping an explicit delimiter stack alongside the recursion — which
costs almost nothing — is what enables "unclosed delimiter" pointing at the opener, and also
enables the heuristic "this later `}` probably closes the wrong thing" that rustc uses to
suggest the likely insertion point.

**3.** Beyond the whole expression, you need: (a) **the span of the operator or keyword
itself**, so "cannot apply `+` to these types" can underline the `+` and label each operand
separately; (b) **the span of the opening and closing delimiters individually**, for the
unclosed-delimiter case and for suggestions that insert or remove one; also worth naming are
spans for individual tokens, so a suggestion can be a precise textual replacement. Multi-file
compilation forces spans to be **offsets into a global coordinate space** rather than
`(file, line, col)` triples: a compact pair of 32-bit positions into one concatenated address
space, with a side table mapping ranges back to files and line starts. rustc's `SourceMap`
and `BytePos` are exactly this. The reasons are size — spans are on every node, so they must
be small — and comparability, since you want span containment and ordering to be integer
comparisons rather than tuple comparisons with a file identity check.

**Trap.** Proposing that the fix is to shorten the expected list to the "most likely" token.
That is a heuristic on top of the wrong information. The fix is to report *the construct
being parsed and where it started* — "unclosed delimiter", "this `fn` body", "while parsing
the arguments of this call" — which requires the parser to carry context (part 2), not to
filter its expectation set.

### A13 — A tree for broken input

**1.** The parser must **always produce a tree, and never fail**. The two node kinds a batch
parser does not need: an **error node** (a node covering tokens the parser could not fit into
any production, so that no bytes are lost from the tree) and a **missing node** (a
zero-width placeholder standing where a required child was absent, so the tree's shape stays
regular and downstream code can walk it without special-casing). With both, `fn foo(a: u32, b:`
yields a function node, a parameter list with two parameters, and a second parameter whose
type is a missing node — which is precisely the anchor completion needs to know it should
offer types.

**2.** Nodes must store **lengths, not absolute offsets** — each node knows its own text
width and its children's, and absolute positions are computed on demand while walking down
from the root. That makes a subtree **position-independent**, so an unchanged subtree can be
reused verbatim after an edit that shifted it, and identical subtrees can even be
deduplicated and shared. This is exactly what rust-analyzer's rowan green trees do, and it is
the representational half of Roslyn's red-green design. What it forbids storing in a node:
absolute offsets, a parent pointer, and any identity or semantic annotation — all three would
make a node valid only at one position in one tree, destroying reuse. Parents and absolute
positions come back in a separate, cheap, lazily-constructed "red" layer built during
traversal.

**3.** Chain: C++ tokenization depends on the symbol table (A1's lexer hack), the symbol
table depends on having processed all the `#include`s, and the includes depend on
preprocessor state that varies per translation unit. So a C++ IDE **cannot lex a file
correctly without effectively performing a build**, which means it cannot produce a reliable
tree at keystroke latency, cannot reuse a parse across translation units that include the
same header with different macros, and degrades exactly when the file is mid-edit and the
symbol table is stale. Rust and C# have no such dependency — the token stream is a function of
the bytes, so a resilient parse is always available and semantic information is layered on
afterwards, asynchronously and cancellably. The feature that looked like a lexer detail in Q1
determines whether good tooling is achievable at all.

### A14 — Two trees for one file

**1.** The equation is **`print(parse(source)) == source`**, byte for byte, for every input
including invalid ones. That requires the tree to retain whitespace, comments, the exact text
of every token, and every delimiter — collectively "trivia". The industrial pattern is a
**single lossless tree plus a typed, generated API layered over it**: the underlying tree is
untyped, complete, and reusable, and the compiler-facing view exposes it as `FnDef`,
`Expr`, and so on, skipping trivia. Implementations: **Roslyn**'s red-green trees for C# and
VB, **rust-analyzer**'s rowan, and **swift-syntax**. Note this dissolves the premise of the
question — you do not build two trees, you build one lossless tree and two views.

**2.** A workable rule, and roughly Roslyn's: **trivia following a token on the same line,
up to and including the newline, is that token's trailing trivia; everything else is the
leading trivia of the next token.** So `f(); // note` attaches the comment to `;`, while a
comment on its own line attaches to whatever follows it. The user-visible bug from a wrong
but consistent rule: **the formatter moves your comments.** If a trailing comment is
attached to the following token instead, then reformatting a statement that spans lines
relocates the comment to the next statement — and it does so deterministically, on every
run, so it survives review and lands in the diff. That is why the rule must be chosen for
what humans mean by proximity, and why once chosen it can never be changed: changing it
reflows every comment in every codebase the formatter touches.

**3.** Irreversible because the information is not recoverable from anything downstream — no
later phase can reconstruct where a comment was or how many blank lines separated two
functions, and the original bytes are not part of the tree. Tools made harder: a
**formatter** (cannot preserve or reflow comments), an **automated refactoring / codemod**
tool (rewriting a file means regenerating text, which loses everything you did not model, so
diffs become whole-file rewrites), a **linter that emits fixes** (same reason), and
**documentation extraction** that needs comments positioned relative to declarations. Add
IDE features that render source (inlay hints anchored to token positions, semantic
highlighting of trivia). The decision to discard trivia at lex time is the single cheapest
early decision that costs the most later, which is why every language that grew a serious
tooling ecosystem eventually retrofitted a lossless tree — and retrofitting means writing a
second parser.

**Trap.** "Just keep the comments in a side table keyed by offset." That works until the
first edit: offsets shift, the side table has to be remapped, and any transformation that
moves code has to decide which comments moved with it — which is the attachment problem
(part 2) in a form where the tree cannot help you, because attachment is a *structural*
question and you have deliberately kept the data unstructured.

### A15 — Identifiers that are not what they look like

**1.** Raw code points mean two identifiers that are canonically equivalent — the same
character composed versus decomposed, say `é` as U+00E9 versus `e` + U+0301 — are **different
symbols**, so a declaration and a use that look identical and compare equal in the user's
editor fail to resolve. Worse, whether they are equal depends on which editor or input method
produced the file. NFC normalization makes canonically equivalent spellings the same
identifier, which is what Rust and Swift do. The new problem: the identifier in the source no
longer matches the byte sequence anywhere the name escapes the compiler — **exported symbol
names, mangled names, filenames derived from module names, debug info, and FFI lookups**.
Some filesystems normalize differently (Apple's HFS+ historically stored NFD), so a module
name normalized to NFC by the compiler may not match the file on disk. You now have two name
domains and a conversion between them, and every place you forgot the conversion is a bug
that only reproduces on one platform with one alphabet.

**2.** The parser is correct and the compiler is correct; what is wrong is that **the
rendering of the source and the meaning of the source disagree**, because bidirectional
override characters inside a string literal or comment cause a reviewer's editor to display
tokens in an order different from their logical order. The exploit is a code review that
passes because the reviewer literally cannot see the code that will be compiled. The fix
therefore belongs in the **lexer as a diagnostic**, not in the grammar: detect bidi control
characters that are unterminated within a literal or comment and reject or warn. rustc
shipped `text_direction_codepoint_in_literal` as a deny-by-default lint, and other compilers
and code-hosting sites added equivalent warnings and rendering markers. The general lesson
worth stating: **the front end is responsible for the correspondence between what a human
sees and what the machine does**, and that responsibility does not appear anywhere in a
grammar.

**3.** Unicode property tables change between Unicode versions — characters are added,
and derived properties like `XID_Start`/`XID_Continue` (the UAX #31 identifier properties)
grow. So **which programs lex depends on which Unicode version your compiler was built
against**, and upgrading the table can make previously-invalid programs valid — or, rarely
and much worse, change a character's category. Consequences: your grammar is not stable
unless you pin a Unicode version, and pinning means you fall behind and reject new scripts;
not pinning means the language accepted is a function of the toolchain build. Every language
that supports non-ASCII identifiers has quietly accepted that its lexical specification is
parameterized by an external, versioned standard it does not control. That is worth saying
out loud in an interview, because it is the one place where "the grammar" is genuinely not a
fixed object.

### A16 — Operators the user declares

**1.** The circularity: **you cannot build an expression tree without knowing fixity, fixity
is declared in a module, and finding that declaration requires resolving imports — which is
name resolution, a phase that consumes the tree you have not built.** GHC breaks it by
deferring the decision: the parser reads a chain of operators and operands as a **flat,
right-nested placeholder structure** without committing to associativity or precedence, and
the **renamer** — after imports are resolved and every operator's fixity is known —
re-associates the chain into the correct tree. So the parse is fixity-agnostic and a later
phase fixes the shape. This is why a fixity error in Haskell ("cannot mix `<+>` and `<*>` in
the same infix expression") is not reported by the parser.

**2.** A partial order means two precedence groups with no declared relation cannot be
compared, so `a <+> b <*> c` where `<+>` and `<*>` are in unrelated groups is a **compile
error demanding parentheses**, rather than silently resolving by some default. What it
prevents: a library adding an operator that silently changes the parse of expressions mixing
it with someone else's operator — with a total order, every new operator gets a position
relative to all others whether or not anyone thought about it, and the wrong position is a
silent misparse. What it costs the library author: real work. You must place your operator in
an existing group or declare a new one with explicit `higherThan`/`lowerThan` relations to the
groups you care about, and users mixing your operators with a third library's may hit errors
you cannot fix from your side. That is the honest price of refusing to guess.

**3.** Deriving precedence from the first character buys **parseability without any semantic
information**: a Scala file can be tokenized and shaped into an expression tree with no
imports resolved, no fixity table, and no second pass — which is exactly the property Q13
says IDEs live and die by, and exactly what Haskell gives up. It also means a syntax
highlighter or a formatter can produce a correct tree standalone. The failure mode accepted:
**you cannot choose your operator's precedence.** An operator whose meaning suggests
low precedence but whose name starts with `*` binds tightly, and the only remedy is renaming
it — the character-class table is a fixed, global convention the user must design around. In
practice this pushes Scala library authors to pick operator spellings for their precedence
rather than for their meaning, which is a real cost that shows up as unreadable names.

**Trap.** Claiming user-defined precedence is "just a table the parser consults." The table
is not available at parse time in any language with modules, and saying so is the whole
question. If the answer does not mention that fixity crosses a module boundary and therefore
requires name resolution, it has missed why the feature is hard rather than tedious.

### A17 — Nobody generates their parser

**1.** (a) **Error recovery and diagnostics.** Hand-written code can carry the context needed
for "unclosed delimiter, opened here" (A12), decide per-construct how to resynchronize, and
emit a targeted suggestion — none of which a generated parser's uniform failure mechanism
expresses. (b) **Contextual and semantic feedback.** The lexer hack, JS's lexical goals,
contextual keywords, C++'s `>>` rule — all are trivial as an `if` in a hand-written parser
and awkward-to-impossible in a declarative grammar. (c) **Resilient and incremental parsing**
(A13): always producing a tree, with error and missing nodes, cancellable mid-parse, reusing
subtrees. Generated parsers commit to a failure model that is not "return a tree anyway".
Speed and allocation control are real too, but they are not the deciding reasons — the
deciding reasons are all about what the front end must *produce*.

**2.** You lose (a) **the machine-checked ambiguity proof** — nothing tells you your syntax
is ambiguous, so a conflict that a generator would have flagged during design becomes a
permanent disambiguation rule (A9); and (b) **the grammar as a single source of truth** — the
grammar is now the code, so the specification drifts from the implementation, and there is no
second implementation to cross-check against. The discipline that recovers part of (a): keep
a reference grammar and use it as a **generator of test inputs and as a differential oracle**
— fuzz random derivations from the grammar and require the hand-written parser to accept
them, and fuzz random byte strings and require the two to agree on accept/reject. That finds
divergence without requiring the generated parser to be the one you ship. A generated parser
used purely as a checker, never as the product, is a real and underused pattern.

**3.** The published grammar is for **humans and for the ecosystem**: it is the normative
specification other implementers work from, the artifact the standards process argues over,
the reference that linters, syntax highlighters, editor grammars, and alternative
implementations are built against, and the document that makes "is this a language change?"
a decidable question. It is documentation with authority, not an input to a build. The
failure mode is exactly that: **because nothing checks it, it drifts.** The shipped compiler
accepts things the grammar forbids and rejects things it allows, and since the compiler is
what everyone actually targets, the grammar quietly becomes fiction while remaining the
document new implementers trust. The only defence is the differential testing from part 2 —
which means the grammar is only worth publishing if something mechanically checks it, and
that is the argument for keeping a generated parser around even after you stop shipping it.
