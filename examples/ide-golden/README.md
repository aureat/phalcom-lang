# Phalcom IDE Golden Workspace

This is the canonical clean-baseline integration workspace for Phalcom compiler,
semantic, LSP, and VS Code tests. Open this directory itself as the VS Code
workspace so `project.toml` is at the workspace root.

Committed source must stay clean: no intentional parser, module, or type errors.
Negative cases are introduced by test mutations and then restored.

The dependency graph intentionally contains a diamond:

```text
ide_golden -> geo -> units
          \--------> units
```

See `EXPECTATIONS.md` for the manual smoke procedure and `expectations/*.toml`
for machine-readable contracts.
