# Semantic Correctness Part 4 Backlog

## Set literal syntax

- **Status:** Deferred; no syntax introduced in Technical Spec 01.
- **Recorded:** 2026-08-26.
- **Issue:** Technical Spec 01 requires Set-literal regressions and migration, but current parser has no accepted Set-literal syntax. `SetLiteralExpr` exists in the AST, while brace parsing currently routes supported forms to Map and rejects other bare-brace forms.
- **Evidence:** `phalcom-ast/src/parser.rs` (`Token::LBrace` branch) and `phalcom-ast/src/ast.rs` (`SetLiteralExpr`).
- **Unblock:** Ratify or implement Set-literal grammar in its owning parser work. Then add RED coverage and migrate `synthesize_set_literal` without changing syntax in this slice.
