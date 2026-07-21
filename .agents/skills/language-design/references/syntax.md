# Syntax & Grammar Design

> Generic design-space layer of the `language-design` skill — matrices + hazards, no textbook prose. Phalcom's committed choice on any axis here: see [../phalcom/overlay.md](../phalcom/overlay.md).
> **Load when:** making surface-syntax decisions — expression vs statement, delimiters, operators, literals, readability/ambiguity. (For parsing technique see parsing.md.)

## Contents
- Axis 1 — Expression- vs statement-orientation
- Axis 2 — Block delimiters
- Axis 3 — Statement termination
- Axis 4 — Operator design
- Axis 5 — Uniformity vs sugar
- Axis 6 — Literals & notation
- Axis 7 — Readability & visual ambiguity
- Axis 8 — Consistency & least-surprise

## Axis 1 — Expression- vs statement-orientation
| Option | Langs | Consequence |
|---|---|---|
| Everything is an expression | Rust, Ruby, ML, Lisp, Scala | `if`/`match`/block yield values; no ternary needed |
| Statements + expressions split | C, Java, Python, Go | `if` is control-only; needs `?:`, temp vars for value flow |
| Expression-oriented but some stmt holes | Rust (`let`/`;`), Kotlin | Mostly value-y; a few constructs (`let`, loops) still don't yield |

**Syntax.** Rust `let x = if c { 1 } else { 2 }` · Ruby `x = case k when 1 then …` · ML `if c then a else b` · C needs `x = c ? a : b` · Lisp `(if c a b)` (all forms are values).
**Hazard — expression-orientation ⊗ statement-only constructs.** Once `if`/`match` are values, every construct is expected to yield one; a loop or `return` that has no natural value forces a unit/`()` type and "expected value, found `;`" papercuts. Decide the value of *every* block-producing form up front, not per-construct. → overlay
**Hazard — value-block ⊗ early return.** In an expression language a non-local `return` inside a value-position block makes the block's static type diverge from its dynamic one; the type of `{ return x }` is bottom, poisoning inference around it.
**Hazard — trailing-`;` semantics ⊗ block value.** When `{ e }` yields `e` but `{ e; }` yields unit (Rust), a stray or missing trailing `;` silently changes a function's return value — a whitespace-invisible bug that typechecks whenever the two types unify.

## Axis 2 — Block delimiters
| Option | Langs | Consequence |
|---|---|---|
| Braces `{…}` | C, Java, Rust, JS | Layout-free, paste-safe; brace-style bikeshedding; `{}` overloaded |
| Significant indentation (offside) | Python, Haskell, F#, YAML | No closers; whitespace is semantic; copy/paste & codegen fragile |
| `if … end` keyword pairs | Ruby, Lua, Pascal, Elixir | Readable, greppable; verbose; nesting needs matched `end`s |
| S-expr parens | Lisp, Scheme, Clojure | Uniform, macro-friendly; "paren soup"; editor-dependent |

**Syntax.** C `if (c) { … }` · Python `if c:`⏎`    …` · Ruby `if c … end` · Lisp `(when c …)` · Haskell layout `where`/`let` blocks.
**Hazard — significant indentation ⊗ codegen/macros/interpolation. (CROWN JEWEL)** Generated, spliced, or string-interpolated code must carry correct indentation to keep its meaning — layout does not compose with metaprogramming. A macro that emits a block, or a heredoc holding source, has no brace to re-anchor on; tabs-vs-spaces and re-indentation silently reparse. Brace/keyword delimiters make emitted code position-independent (parsing.md: layout needs a lexer-level indent stack). → overlay
**Hazard — offside ⊗ line continuation.** Indentation-sensitive grammars must still define how a single logical expression wraps across lines (implicit brackets vs explicit `\`); get it wrong and aligned code changes scope.
**Hazard — offside ⊗ tabs/spaces mix.** With semantic whitespace, a tab-vs-space mismatch (or an editor's re-indent) reparses to a different block structure while looking identical; forces a normative tab-width rule or an outright tab ban (Python 3).

## Axis 3 — Statement termination
| Option | Langs | Consequence |
|---|---|---|
| Explicit `;` terminator | C, Java, Rust | Unambiguous; free line-wrapping; one-more-token noise |
| Newline-significant | Python, Go, Swift, JS-ish | Terse; a stray newline ends a statement early |
| ASI (semicolons inserted) | JavaScript | "Optional" `;`; hidden rules bite on leading `(`/`[`/`return` |
| None — delimiters self-close | Lisp, Smalltalk (`.` separates) | No terminator token; grouping carries the boundary |

**Syntax.** C `a; b;` · Go newline = `;` (lexer-inserted) · JS `return⏎ x` → `return; x` · Smalltalk `a. b` (period is a *separator*) · Lisp `(a) (b)`.
**Hazard — ASI / newline-significance ⊗ line breaks. (CROWN JEWEL)** Automatic semicolon insertion (or bare newline-as-terminator) silently changes meaning across a newline: `return`⏎`{…}` returns `undefined`; a line starting `(`/`[`/`` ` `` glues onto the previous statement. The rule lives in the lexer, invisible in the grammar — reviewers can't see the inserted token. Prefer explicit terminators or make newline-joining fully deterministic (parsing.md). → overlay
**Hazard — newline-significant ⊗ expression-orientation.** If newlines terminate statements *and* blocks are value expressions (Axis 1), a wrapped binary expression can terminate before its right operand; forces trailing-operator or open-bracket continuation rules.
**Hazard — separator vs terminator ⊗ list editing.** A statement *separator* (Smalltalk `.`, Pascal `;`) forbids a mark after the last item, so appending a line requires editing the previous one; a *terminator* (C `;`) makes every line self-contained. The choice shapes diff cleanliness and paste-safety.

## Axis 4 — Operator design
| Option | Langs | Consequence |
|---|---|---|
| Fixed built-in operator set | C, Java, Go, Python | Precedence table is closed; predictable parse; no DSL operators |
| User-defined ops + declared fixity | Haskell (`infixl`), Scala, Swift | Expressive DSLs; parse depends on in-scope fixity decls |
| Operators = ordinary methods/messages | Smalltalk, Ruby, Scala | Uniform dispatch; Smalltalk flattens all binaries to one precedence |
| Named word-operators only | Python (`and`/`or`/`not`), COBOL | Readable; no glyph zoo; verbose, fewer DSLs |

**Syntax.** Haskell `infixl 6 +` / ``x `op` y`` · Scala `def +(o) = …` · Smalltalk `3 + 4 * 5` = `(3+4)*5` (strict left, no precedence) · Swift `infix operator ~>` · Rust: overload via `impl Add`, fixity fixed.
**Hazard — user-defined operators ⊗ precedence/readability. (CROWN JEWEL)** Custom fixity makes the grammar context-dependent — the same token stream parses differently under different imported `infix` decls, so no one can read an expression without knowing every operator's precedence and associativity. The parser cannot build a fixed precedence-climbing table; it must resolve fixity after name resolution or reparse (parsing.md). Overloading `==`/`+` also silently re-routes to user code (values.md: `==`⊗hash contract). → overlay
**Hazard — symbolic operator soup ⊗ discoverability.** Glyph operators (`>>=`, `<$>`, `<|>`, `~>`) aren't greppable or nameable in prose; a reader can't look up `<$>` the way they can `map`.
**Hazard — operators-as-messages ⊗ precedence expectation.** If `+`/`*` are just binary messages evaluated left-to-right (Smalltalk), `2 + 3 * 4` = `20`, violating universal arithmetic intuition; uniformity buys simplicity at the cost of every newcomer's first surprise. → overlay

## Axis 5 — Uniformity vs sugar
| Option | Langs | Consequence |
|---|---|---|
| Homoiconic minimalism | Lisp, Clojure | Tiny grammar = code-as-data; macros trivial; verbose, alien to newcomers |
| Small regular grammar | Go, Scheme, Smalltalk | Fast to learn/tool; fewer idioms; boilerplate for common shapes |
| Rich surface sugar | Ruby, Perl, C++, Scala | Idiomatic, terse; many-ways-to-do-it; brutal for parsers & linters |
| Sugar as desugaring layer | Rust (`?`, `for`), Haskell (`do`) | Ergonomic front, small core; desugar rules are their own spec surface |

**Syntax.** Lisp: everything `(op …)` · Go: ~25 keywords, no ternary/overloading · Ruby: `unless`, `x if y`, `%w[…]`, blocks, `&:sym` · Rust `?`/`for` desugar to `match`/`Iterator`.
**Consequence.** Small/uniform → macro-friendly, easy tooling, cheap self-hosting (bootstrapping.md); rich sugar → high per-feature learnability cost and a combinatorial grammar.
**Hazard — sugar ⊗ desugaring transparency.** Each sugar (`for`, `?`, `do`, comprehensions) must desugar to core forms whose errors, spans, and stepping still point at the *written* syntax; leaky desugaring surfaces synthetic identifiers in diagnostics (compiler.md). → overlay
**Hazard — "many ways" ⊗ tooling & review.** Every redundant surface form (Ruby `unless`/`x if y`, postfix loops) multiplies formatter cases, lint rules, and diff noise; a large grammar taxes every downstream tool, not just the parser.

## Axis 6 — Literals & notation
| Option | Langs | Consequence |
|---|---|---|
| Rich built-in collection literals | Python `[]`/`{}`/`{k:v}`/`{,}`, Ruby, JS | Terse data; `{}` means both set/dict/block ambiguity |
| Only scalar literals; ctors for rest | Java (pre-`List.of`), Go | No literal ambiguity; noisy data construction |
| String interpolation in-syntax | Ruby `#{}`, JS `` `${}` ``, Swift `\(…)`, Python f`{}` | Ergonomic; parser must recurse into strings; injection risk |
| Tagged / template literals | JS `` tag`…` ``, Scala `s"…"` | User-defined literal DSLs; extra lexer mode |
| Numeric affordances | `_` separators, `0x`/`0b`, suffixes | Readable magnitudes; more lexer states, suffix-vs-ident clashes |

**Syntax.** Python `{1,2}` set vs `{}` empty-dict · JS `` `hi ${name}` `` · Rust `1_000u64`, `0xFF`, `r"raw"`, `b"bytes"` · Ruby `%i[a b]` · trailing comma `[1,2,]` (Rust/JS/Go allow, JSON forbids).
**Hazard — interpolation ⊗ layout/lexer mode.** An interpolated `${…}` re-enters full expression grammar mid-string; nesting quotes, braces, and (with Axis 2) indentation inside the hole needs a re-entrant lexer stack — and any embedded code is a template-injection vector if reused as source (security in interpolated DSLs). → overlay
**Hazard — trailing comma ⊗ grammar.** Allowing `[1,2,]` must be a deliberate grammar rule, not an accident; toggling it later is a breaking change to every tool that reparses (parsing.md).
**Hazard — collection literal `{}` ⊗ empty-case.** If `{}` is both empty-map and empty-set/block, the empty literal has no disambiguating element; the language must pick one meaning by fiat (`{}` = dict in Python) and force a ctor for the other. → overlay

## Axis 7 — Readability & visual ambiguity
| Option | Langs | Consequence |
|---|---|---|
| Dangling-else left open | C, Java, JS | `else` binds to nearest `if`; nested `if` without braces misreads |
| Declarator-follows-use | C/C++ | Spiral `int (*f)(int)`; type reads inside-out |
| `{}` block vs record/struct literal | Rust, JS, Go | `if x {}` vs `Foo{}`; literal-in-condition needs guard rule |
| Statement/expression lambda split | Python (`lambda` expr-only vs `def`) | One-liner λ can't hold statements; forces named `def` |
| "Syntax is UX" — optimize for the reader | Python, Swift design ethos | Fewer glyph puns; more keywords; larger grammar |

**Syntax.** C dangling `if(a) if(b) x; else y;` (`else`→inner) · C spiral `char *(*(*x)())[]` · Rust forbids bare struct-literal in `if`/`while` head, needs `if (Foo{..}).c` or parens · Python `f = lambda x: x+1` (no statements).
**Hazard — glyph overload ⊗ parsing ambiguity. (CROWN JEWEL)** One glyph with many meanings breeds most-vexing-parse-class ambiguity: `{}` as block *and* map/record, `<` as less-than *and* generic-args (`a < b > c`), `*`/`&` as arithmetic *and* deref/ref, `()` as call *and* grouping *and* tuple. The parser needs lookahead, backtracking, or a disambiguation rule per collision (parsing.md); readers need the same context in their head. Reserve one glyph → one role where you can. → overlay
**Hazard — dangling-else ⊗ optional braces.** Permitting brace-less single-statement bodies *and* `if/else` reintroduces the classic shift-reduce ambiguity; the "nearest `if`" fix is a parser rule invisible in the surface grammar.
**Hazard — lambda-as-expression ⊗ multiline bodies.** An expression-only λ (Python `lambda`) can't grow statements, so any block-bodied closure must switch to a named form — splitting one concept across two syntaxes and blocking inline callbacks.

## Axis 8 — Consistency & least-surprise
| Option | Langs | Consequence |
|---|---|---|
| One construct → one meaning | Go, Smalltalk, Scheme | Predictable; verbose; resists convenience shortcuts |
| Context-dependent glyphs | C (`*`), Rust (`&`/`'`), Perl sigils | Dense; each site needs surrounding context to read |
| Hard keywords (fully reserved) | Java, Go, C | No keyword-as-identifier surprises; can't add words back-compat |
| Contextual keywords | C# (`async`/`await`/`yield`), Swift, TS | Grow grammar without breaking old code; parse depends on position |
| Sigil-marked roles | Ruby `@`/`@@`/`$`, Perl `$`/`@`/`%` | Role visible at a glance; sigil zoo, paste hazards |

**Syntax.** C `*` = multiply / deref / pointer-decl · Rust `&` = ref / bitand / pattern; `'a` lifetime vs char · C# `await` is an identifier pre-`async` context · Ruby `@x` field vs `x` local.
**Hazard — contextual keywords ⊗ future evolution. (CROWN JEWEL-adjacent)** Making `async`/`match`/`yield` contextual keeps old code compiling, but every parser thereafter must decide keyword-vs-identifier by position, and any *new* contextual keyword can shadow an existing variable name and change a program's meaning silently. Hard-reserving is a one-time break; contextual-forever is permanent parser complexity (parsing.md). → overlay
**Hazard — glyph reuse ⊗ least-surprise.** Reusing one glyph across roles (`*` mult/deref/rest/glob, `:` slice/type-annot/dict/label) means a single-character typo changes *which* feature fires, not just triggers a syntax error — the worst failure mode for a reader. → overlay
