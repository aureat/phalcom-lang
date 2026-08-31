// area: imports
// spec: U15 plan §1/§6
// status: PASS
// No global-namespace pollution: `import` binds a name in the *importing*
// scope only. `lib/isolated.ph`'s own top-level `shared` does not overwrite
// (or become visible as) this file's own `shared` global — it is reached
// only through the `Iso.shared` member send.

let shared = 1
import "./lib/isolated" as Iso
System.print(shared)
System.print(Iso.shared)
