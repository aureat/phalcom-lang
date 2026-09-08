# REPL sessions

> Sources: CLIT001
> Raw: [tooling program snapshot](../raw/tooling/2026-09-08-tooling-sources.md)
> Updated: 2026-09-08

A REPL session retains input history, compiler/VM execution context, bootstrap state, and the current source/runtime environment across snippets. Session reset and exit are lifecycle operations; they must not leave a partially updated semantic or runtime state.

[REPL commands](repl-commands.md) selects operations, while [interactive intelligence](interactive-intelligence.md) queries canonical products. CLIT001 is proposed and not started.
