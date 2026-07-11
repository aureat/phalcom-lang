# Forge — Deferred Register

_Optimization / DX / speed / security ideas surfaced during forge but intentionally out of v1 scope. Ranked backlog, not commitments._

| # | Idea | Source | Spec/ADR | Rank |
|---|------|--------|----------|------|
| 1 | `SyntaxErrorKind::InvalidInteger`/`InvalidFloat` lower to a zero-width `0..0` range, losing the offending literal's span in diagnostics. Carry the real span through `LexicalError` instead. | `phalcom-ast/src/parser.rs` (`lex_error_to_syntax`) | ADR-0016 | low |
| 2 | The hand-written parser accepts a few malformed assignment targets (e.g. `a+b = c`, `(a+b) = c`) that LALRPOP rejected at parse time; they are still caught by the compiler as invalid assignment targets, but could be rejected earlier with a precise diagnostic. | `phalcom-ast/src/parser.rs` (`parse_assignment`) | ADR-0016 | low |
| 3 | Pre-existing `clippy::extra_unused_lifetimes` warning: `format_num_arguments<'a>` declares an unused lifetime. Present on `main`, in a file byte-identical to `main` and outside U1's write-set, so left untouched by the U1 heap migration. Drop the `<'a>`. | `phalcom-core/src/error.rs:30` | — | low |

## Surfaced during Phase-2 planning (2026-07-11)
| # | Idea | Source | Spec/ADR | Rank |
|---|------|--------|----------|------|
| 4 | **F4 (`object_name` / instance `toString`, ADR-0015) needs a home unit** — scoped out of U2 to keep the tower unit tight and independently verifiable. | U2 architect | ADR-0015 | high |
| 5 | **Kernel `List` + collections** (`List`/`Map`/`Set`/`Tuple`/`Range`) — need storage primitives + literals; hard-blocks dNU `Message.args` (U8) and rest-params (U9). Candidate to promote to a critical-path unit (DEC-A). | U8/U9/U-STD | messages/functions | high |
| 6 | Collection-literal lowering `(a,b)`/`[…]`/`{a:1}` — deferred out of U-LEX (no lowering spec; entangles with U4 braces). | U-LEX | (needs ADR) | med |
| 7 | Reflection surface: `Method.bind(_:)`/`invokeOn(_:_:)`/`methodFor(_:)`, `Function`/`Block`/`Method` reflection — depends on first-class `Block` (U4). | U4/U-STD | functions.md §3 | med |
| 8 | Per-class dNU handler cache (keyed on `ClassId`, gated by open-Q4); spread call sites `f(*args)`. | U8 | open-Q4 | med |
| 9 | Block variadics `{ *xs => }`; `callWith(_:List)` interaction (stub → `ArgumentError` until List lands). | U4/U9 | functions.md §2 | low |
| 10 | `for (x in xs)` runtime (needs `each`/iterables → collections); per-call-site polymorphic IC (U5 seeds only coarse epoch invalidation); derived control selectors expressed in `core.ph` once U11 lands. | U5 | control-flow.md | med |
| 11 | Concurrency runtime: `Fiber`/`Future`/`Error` classes. | U-STD | concurrency.md | low |
| 12 | Lexer polish: nested block comments; lone-`?` ternary; carry real span through `LexicalError` (dup of #1). | U-LEX | ADR-0016 | low |
| 13 | Reassignment of a *captured* `let` (an outer binding reached through an upvalue from inside a block) is not rejected — U6 only enforces immutability for a current-function local and a module global. An inner-block `count = count + 1` over an outer `let count` compiles to `SetUpvalue` with no diagnostic. Extend the assignment path to walk enclosing function-states for `let` locals. | `phalcom-core/src/compiler/lib.rs` (`Expr::Assignment`, upvalue branch) | ADR-0014 | med |
| 14 | The `if(opt)` truthiness compile check (`CompilerError::OptionTruthiness`) is literal-only per BD-U6-1 Option (A): it catches `None` and `Some.new(...)` as the condition of `ifTrue:`/`ifFalse:`/`ifTrue:ifFalse:`/`and:`/`or:`, but not an Option-typed *variable* used as a condition (that stays a hard runtime type error via the branch opcode's `Bool` requirement). No span is attached to the diagnostic (the enum variant carries no `SourceRange`, consistent with the other `CompilerError` variants). | `phalcom-core/src/compiler/lib.rs` (`is_option_literal`) | ADR-0007 | low |

## Surfaced during U7 implementation (2026-07-11)
| # | Idea | Source | Spec/ADR | Rank |
|---|------|--------|----------|------|
| 15 | Fixed slot layout + private-non-inherited fields (ADR-0011) forecloses (a) adding a field to a *live* class / `become:`-style reshape (offsets are frozen at class-definition time) and (b) shared *protected* inherited fields (a subclass must go through accessors, never a shared offset). Both are deliberate per ADR-0011 — good for a future inline cache (stable offsets) — but flag if either is ever wanted; it is a cross-cutting reshape of the object model, not a local change. | U7-plan §3 (rubric preclusion) | ADR-0011 | low |
| 16 | The `Counter.new()` → `construct` selector redirect (`VM.constructor_aliases`/`has_new_construct`) is a **compile-time, same-compilation-unit, literal-receiver** heuristic: it only fires when the call site's receiver is a bare `Expr::Var` naming the class *and* that class's `construct`(s) were already compiled earlier in the same source. An indirect receiver (`let C = Counter; C.new()`), a forward reference, or a cross-module call falls back to the plain `Method`-encoded selector — for a class with no matching `new` primitive of its own this silently reaches `Object::new`'s bare allocator again, uncaught. A real fix needs either static class-type tracking through locals or a runtime dispatch rule (e.g. always trying the `Initializer` encoding before `Method` at every send), not a compiler-side name match. | `phalcom-core/src/compiler/lib.rs` (`Expr::MethodCall`) | ADR-0011 | med |

_Closed:_ #(ex-LALRPOP) — done in U1: dead `CompilerError::ParseError` variant + `From<lalrpop_util::ParseError>` impl deleted (slice 3) and `lalrpop-util` dropped from `phalcom-core/Cargo.toml` + `Cargo.lock`.
