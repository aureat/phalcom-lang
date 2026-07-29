# Target Project Architecture

The package uses a shared search kernel and feature slices.

```text
choices + spans + immutable examples
              │
         strategy layer
              │
     generation / replay / shrink
              │
  ┌───────────┼────────────┬─────────────┐
property API  find/search  stateful API  databases/reporters
```

The kernel owns primitive decisions and search semantics. Property discovery, stateful programs, databases, and reporters depend on it but do not redefine it.

Public imports come from `src/hypothesis.ph`. Internal implementation names are module-private or `_`-prefixed.
