# REPL commands

> Sources: CLIT001
> Raw: [tooling program snapshot](../raw/tooling/2026-09-08-tooling-sources.md)
> Updated: 2026-09-08

REPL commands are the control surface for evaluating snippets, inspecting session state, requesting intelligence, and leaving the session. Command parsing and presentation must preserve the language session's source/runtime context.

## Boundary

Commands select an operation; the session owns compiler/VM state; canonical semantic products own types, diagnostics, and navigation facts. Terminal styling is delegated to [diagnostics](../diagnostics/terminal-rendering.md).

CLIT001 is proposed and not started.
