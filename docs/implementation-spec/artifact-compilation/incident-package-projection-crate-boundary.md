# Package module projection crate boundary

## Finding

`phalcom-modules` owns `project_package_module_world_v5`, while
`StableIdentityContext` and `to_stable_module_with_context` are defined in
`phalcom-semantic`. The modules crate already sits below semantic in the
crate dependency graph, so PB-M1 cannot pass the semantic context directly
without introducing a dependency cycle or duplicating identity policy.

## Decision

Keep identity authority in semantic/core. The module projection accepts a
borrowed `Fn(&ModuleId) -> StableModuleRef` callback. Core captures the
publishing `StableIdentityContext` and delegates conversion to
`to_stable_module_with_context`; the module projection remains responsible
for wire indexing and validation of returned references.

PB-M1 in `05-spec-package-builder.md` is amended to this callback shape. The
accepted identity and ownership rules do not change.
