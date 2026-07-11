# Lexing & Parsing

> Generic design-space layer of the `language-design` skill — matrices + hazards, no textbook prose. Phalcom's committed choice on any axis here: see [../phalcom/overlay.md](../phalcom/overlay.md).
> **Load when:** implementing/critiquing a lexer or parser — tokenization, parser-algorithm choice, precedence, error recovery, incremental/CST.

## Contents
- Lexer construction
- Lexer state & modes
- Significant whitespace / layout
- Parser algorithm family
- Operator precedence & associativity
- Error recovery
- Ambiguity, lookahead & backtracking
- CST / lossless / incremental

## Lexer construction
| Option | Langs | Consequence |
|---|---|---|
| Hand-written scanner | rustc, Go, Clang, V8 | Full control, best errors/perf; verbose, easy to skew |
| Table/DFA generator | lex/flex, re2c | Provably maximal-munch; opaque tables, awkward mid-token hooks |
| Derive-macro DFA | Rust `logos` | Declarative + fast DFA; escape hatches for stateful tokens |
| Regex-per-rule, ordered | many toy/PEG lexers | Trivial to write; backtracking, slow, rule-order bugs |
| Scannerless (no lexer) | PEG grammars, SGLR | One grammar, no token/parse split; maximal-munch by hand |

**Impl.** maximal munch = longest match wins at each position; hand-written scanners encode it as greedy peek-loops, generators bake it into the DFA's accepting states.

**Hazard — maximal munch ⊗ token boundaries.** Greedy longest-match lexes `>>` as one shift token, breaking nested generics `Vec<Vec<T>>`; `1..10` vs float `1.`, `a-->b`, `x=-1`. Fixes: parser splits `>>` (C++03 rescan), lexer lookahead disambiguation, or a lexer/parser context bit. Each re-couples the phases. → overlay

## Lexer state & modes
| Option | Langs | Consequence |
|---|---|---|
| Stateless token function | Go, Lisp readers | Reentrant, testable; can't express nesting/interpolation |
| Mode stack | Ruby, Swift, Kotlin | Push/pop lexer states; interpolation & heredocs work, stateful |
| Recursive sub-lexer | JS/Swift `"${…}"` | Interp body re-enters full lexer; handles arbitrary nesting |
| Parser feeds lexer ("lexer hack") | C/C++ `typedef` | Resolves `A * b` decl-vs-mul; destroys phase separation |

**Syntax.** interpolation `"hi ${user.name}!"`, heredoc `<<~SQL … SQL`, raw `r#"…"#` — each needs the lexer to know when the string literal *pauses* for code and *resumes*.

**Hazard — lexer hack (context-sensitive lexing) ⊗ clean phase separation (CROWN JEWEL).** Feeding parser/symbol context back to the lexer couples the phases irreversibly: C's `(A)*b` needs the parser's `typedef` table to tokenize; C++'s `A<B>` needs it to pick generic-`<` over less-than. Once the lexer depends on semantic state you lose independent testing, straightforward incremental relex, and any clean token stream. Prefer parser-side disambiguation or context-free grammar. → overlay

## Significant whitespace / layout
| Option | Langs | Consequence |
|---|---|---|
| Whitespace-insensitive + delimiters | C, Rust, Lisp | Free-form, paste-safe; needs `{}`/`;`, more punctuation |
| Emit INDENT/DEDENT tokens | Python, occam | Layout drives blocks; tabs-vs-spaces + generated-code pain |
| Offside rule w/ implicit braces | Haskell, F# | Elegant, `{;}` still allowed; notoriously subtle rule |
| Newline as statement terminator | Go, JS (ASI), Swift | Fewer semicolons; line-continuation & ASI foot-guns |
| Explicit line continuation | Python `\`, C `\`+NL | Author overrides breaks; easy to miss, trailing-space traps |

**Impl.** a layout pass sits between lexer and parser: it tracks an indent stack and synthesizes INDENT/DEDENT (or virtual `{ ; }`) so the parser stays context-free over an explicit block-delimited stream.

**Hazard — significant whitespace ⊗ interpolation/macros (CROWN JEWEL).** Layout state and string interpolation collide: what indentation governs the code inside a multiline `"""${ if x: … }"""`, and does a heredoc/raw block suspend INDENT tracking? Macro- or template-generated code arrives with no original columns, so any indentation-carrying token stream mis-nests. Interpolation bodies and generated spans must run under a *saved/neutral* layout context, not the enclosing one. → overlay

**Hazard — newline-as-token ⊗ automatic semicolon insertion.** JS ASI inserts a terminator at line breaks by recovery rule, so a leading-`(`/`[` next line silently joins, and `return⏎ x` returns `undefined`. Terminator-by-newline needs explicit continuation rules, not "insert on parse error".

## Parser algorithm family
| Option | Langs | Consequence |
|---|---|---|
| Hand-written recursive descent (LL) | rustc, Clang, Go, V8 | Best errors, full control; you own precedence + left-recursion |
| Pratt / precedence climbing | Go exprs, Zig, `syn` | Compact operator parsing; expression-local, pairs with RD |
| LR/LALR generator | yacc/bison, GCC(old) | Handles left-recursion, wide grammars; opaque conflicts, poor errors |
| PEG / packrat | Python(3.9+ CPython), pegjs | Scannerless, ordered choice, linear; O(n) memo, hides ambiguity |
| Parser combinators | Haskell parsec/nom | Composable, in-language; backtracking cost, error tuning hard |
| GLR / Earley | tree-sitter(GLR), SGLR | Parses ambiguous/all grammars; forests, ambiguity pushed downstream |

**Impl.** production compilers overwhelmingly hand-write recursive descent for statements/declarations and drop into a Pratt loop for expressions — RD gives targeted diagnostics, Pratt gives table-free precedence. → [recipes.md#pratt-parse](recipes.md#pratt-parse)

**Hazard — backtracking/packrat ⊗ error quality & memory (CROWN JEWEL).** Unbounded backtracking (PEG, naive combinators) reports failure at wherever the *last* alternative died, not the real mistake — error locations are near-useless without hand-placed cut/commit points. Packrat's linear-time guarantee costs an O(n×rules) memo table, a large constant most inputs never need. Budget both before choosing PEG for a real language. → overlay

## Operator precedence & associativity
| Option | Langs | Consequence |
|---|---|---|
| Grammar-encoded levels | C grammar, Java | Precedence is explicit in rules; deep chains, rigid, verbose |
| Pratt binding powers | Zig, Go, Pratt/`syn` | One table of (lbp,rbp); trivial to extend, no grammar edits |
| Precedence climbing | many RD parsers | Min-precedence recursion; same power, expressed as a loop |
| User-defined ops + fixity decls | Haskell `infixl 6`, Swift | Libraries add operators; parse depends on imports/scope |

**Impl.** Pratt: each token has a left binding power; parse-expr(minbp) loops consuming operators while `lbp > minbp`, recursing with the operator's right bp — right-assoc uses `rbp = lbp-1`. Prefix/postfix/mixfix (`? :`, index) are null/left denotations. → [recipes.md#pratt-parse](recipes.md#pratt-parse)

**Hazard — user-defined fixity ⊗ parse-before-resolve.** Haskell/Swift let imports declare operator precedence, so you cannot build the *shape* of the expression tree until you know each operator's fixity — which resolution hasn't computed yet. Solutions: parse a flat operator list and re-associate after fixity is known, or a fixity pre-pass. A one-shot precedence table can't express it. → overlay

## Error recovery
| Option | Langs | Consequence |
|---|---|---|
| Bail on first error | early Pascal | Trivial; one error/run, miserable edit loop |
| Panic-mode + sync tokens | yacc-style, many C | Skip to `;`/`}` and resume; cascades phantom errors |
| Error productions | GCC, bison | Grammar anticipates common slips; targeted, grammar bloat |
| Error/missing nodes in tree | rust-analyzer, Roslyn, tree-sitter | Parse always yields a tree; IDE survives, downstream must tolerate |
| Incremental resync | tree-sitter | Reuse & realign around edit; complex, tooling-grade |

**Impl.** recovering RD parser, on an unexpected token, inserts an explicit *missing*/error node, skips to a FOLLOW/sync set, and marks the region "poisoned" so later passes report nothing new for it.

**Hazard — panic-mode resync ⊗ cascading errors.** Skipping to the next sync token routinely eats valid tokens and re-enters the grammar mid-construct, spraying phantom errors that bury the real one. What makes multi-error output usable is dedup + a poisoned marker on recovered nodes, not the resync itself. → overlay

## Ambiguity, lookahead & backtracking
| Option | Langs | Consequence |
|---|---|---|
| Bounded LL(k) lookahead | Go, Java | Predictable, fast; some constructs need k>1 or a hack |
| Unbounded backtracking | C++ parsers, PEG | Parses tricky grammars; exponential blowup risk, bad spans |
| PEG ordered choice | CPython(PEG), pegjs | `/` commits to first match; deterministic, silently masks ambiguity |
| Semantic/context disambiguation | C++ `A<B>`, `A*b` | Resolve with symbol table; reintroduces the lexer-hack coupling |

**Syntax.** classic clashes: dangling-else (`if a if b … else`), C++ most-vexing-parse (`T x(Y());` = fn decl not object), `a < b` vs generic `Foo<Bar>`, cast-vs-paren `(a)*b`.

**Hazard — PEG ordered-choice ⊗ silent ambiguity (CROWN JEWEL).** PEG's `A / B` never reports a conflict — it just takes `A` and never tries `B` if `A` matched a prefix. A grammar bug where `B` was the intended parse produces *no error*, only wrong trees, and reordering alternatives silently changes the language. LR/GLR would surface the conflict; PEG hides it by construction. Test ordered choices adversarially. → overlay

**Hazard — dangling-else ⊗ grammar ambiguity.** The `else` binds to nearest `if` only because the parser *chooses* to (shift-over-reduce, or greedy RD); the grammar itself is ambiguous. Rely on the implicit rule and a refactor/generator swap flips the meaning — encode it explicitly or require braces.

## CST / lossless / incremental
| Option | Langs | Consequence |
|---|---|---|
| Throwaway AST | most batch compilers | Compact, fast; loses trivia, poor for refactor/format tools |
| Lossless CST (trivia attached) | Roslyn, Swift libsyntax, tree-sitter | Round-trips source exactly; heavier nodes, more memory |
| Red-green trees | Roslyn, rust-analyzer | Immutable shared "green" + positioned "red"; cheap edits, subtle API |
| Incremental reparse | tree-sitter, IDE parsers | Reparse only edited span; needs stable node identity + resync |

**Impl.** green node = kind + width + children, position-free and structurally shared; incremental reparse diffs the edit range, reuses untouched green subtrees, and only re-lexes/re-parses the affected span.

**Hazard — CST retention ⊗ memory.** Keeping every whitespace/comment token plus red-node wrappers for a large project multiplies front-end footprint several-fold; a compiler that never formats or refactors pays that for nothing. Decide tooling ambitions before committing to lossless. → overlay

**Hazard — incremental reparse ⊗ lexer modes.** An edit inside a string/heredoc/interpolation can flip lexer state for everything after it, so the "affected span" is unbounded — a lone unterminated `"` re-lexes the rest of the file. Incremental relex must track mode at chunk boundaries, not just character ranges.
