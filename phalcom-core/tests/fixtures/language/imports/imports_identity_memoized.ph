// area: imports
// spec: U15 plan §1/§6; object-model.md §4
// status: PASS
// A unit is compiled exactly once and memoized by canonical path: two
// `import`s of the same file (here, two bindings in the same importer) hand
// back the same underlying `Module`, so a class defined inside it keeps one
// identity across both bindings (`==` on the class values, the observable
// proxy for module identity — `Module` values themselves are never `==`,
// a pre-existing kernel rule this unit does not change, `value.rs`).

import "./lib/shared" as A
import "./lib/shared" as B
System.print(A.Point == B.Point)
System.print(A.value)
