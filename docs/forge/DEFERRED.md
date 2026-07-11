# Forge — Deferred Register

_Optimization / DX / speed / security ideas surfaced during forge but intentionally out of v1 scope. Ranked backlog, not commitments._

| # | Idea | Source | Spec/ADR | Rank |
|---|------|--------|----------|------|
| 1 | Remove the dead `CompilerError::ParseError` variant + `From<lalrpop_util::ParseError>` impl and drop the `lalrpop-util`/`lalrpop` deps so LALRPOP leaves the workspace dependency graph entirely (out of U-FE write-set: `phalcom-core/src`). | `phalcom-core/src/compiler/lib.rs:37`, `phalcom-core/src/compiler/lib.rs:43`, `phalcom-core/Cargo.toml:11`, `phalcom-core/Cargo.toml:25` | ADR-0016 | high |
| 2 | `SyntaxErrorKind::InvalidInteger`/`InvalidFloat` lower to a zero-width `0..0` range, losing the offending literal's span in diagnostics. Carry the real span through `LexicalError` instead. | `phalcom-ast/src/parser.rs` (`lex_error_to_syntax`) | ADR-0016 | low |
| 3 | The hand-written parser accepts a few malformed assignment targets (e.g. `a+b = c`, `(a+b) = c`) that LALRPOP rejected at parse time; they are still caught by the compiler as invalid assignment targets, but could be rejected earlier with a precise diagnostic. | `phalcom-ast/src/parser.rs` (`parse_assignment`) | ADR-0016 | low |

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
| 12 | Lexer polish: nested block comments; lone-`?` ternary; carry real span through `LexicalError` (dup of #2). | U-LEX | ADR-0016 | low |
