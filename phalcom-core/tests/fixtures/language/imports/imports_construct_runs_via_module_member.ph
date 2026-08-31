// area: imports
// spec: classes.md §1; object-model.md §5; ADR-0002; DEC-U15 A+A
// status: PASS
// A constructor reached through an imported module member actually runs.
//
// `import ... as M` binds the whole module (DEC-U15 binding choice A, no
// selective form), so an imported class is *only* ever reachable as
// `M.Class` — a `GetProperty` receiver, never a bare identifier. A
// constructor must therefore resolve by ordinary metaclass-tower lookup
// (ADR-0002, object-model §5) rather than by any receiver-name-keyed
// call-site rewrite, which could never fire here. Regression: `M.Shape.new(3)`
// once silently reached the bare allocator `Class >> new()` and handed back an
// instance with unset fields instead of running `@constructor new(sides)`.

import "./lib/shape" as M

let s = M.Shape.new(3)
System.print(s.sides)
