# Semantic analysis

> Sources: `docs/spec/semantic-analyzer/`, SEMA001–SEMA009
> Raw: [semantic specification and program snapshot](../raw/semantic/2026-09-08-semantic-sources.md)
> Updated: 2026-09-08

The semantic domain turns parsed language into authoritative, identity-stable products for the compiler and tools. It neighbors [language](../language/overview.md) for syntax, [type-system](../type-system/overview.md) for type structures, [modules](../modules/overview.md) for project topology, [editor](../editor/overview.md) for protocol products, and [runtime](../runtime/overview.md) for executable projection.

## Reading order

- [Type formation and inference](type-formation-and-inference.md)
- [Authority and identity](authority-and-identity.md)
- [Workspace incrementality](workspace-incrementality.md)
- [Capability and flow](capability-and-flow.md)
- [Products and reflection](products-and-reflection.md)
- [Pattern coverage and constructors](pattern-coverage-and-constructors.md)

The normative semantic analyzer chapters own the effective rules. Implementation program metadata is used here only to label lifecycle and verification state.
