# Forge — Deferred Register

_Optimization / DX / speed / security ideas surfaced during forge but intentionally out of v1 scope. Ranked backlog, not commitments._

| # | Idea | Source | Spec/ADR | Rank |
|---|------|--------|----------|------|
| 1 | Remove the dead `CompilerError::ParseError` variant + `From<lalrpop_util::ParseError>` impl and drop the `lalrpop-util`/`lalrpop` deps so LALRPOP leaves the workspace dependency graph entirely (out of U-FE write-set: `phalcom-core/src`). | `phalcom-core/src/compiler/lib.rs:37`, `phalcom-core/src/compiler/lib.rs:43`, `phalcom-core/Cargo.toml:11`, `phalcom-core/Cargo.toml:25` | ADR-0016 | high |
| 2 | `SyntaxErrorKind::InvalidInteger`/`InvalidFloat` lower to a zero-width `0..0` range, losing the offending literal's span in diagnostics. Carry the real span through `LexicalError` instead. | `phalcom-ast/src/parser.rs` (`lex_error_to_syntax`) | ADR-0016 | low |
| 3 | The hand-written parser accepts a few malformed assignment targets (e.g. `a+b = c`, `(a+b) = c`) that LALRPOP rejected at parse time; they are still caught by the compiler as invalid assignment targets, but could be rejected earlier with a precise diagnostic. | `phalcom-ast/src/parser.rs` (`parse_assignment`) | ADR-0016 | low |
