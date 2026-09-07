// area: ic
// spec: modules.md (module globals + core auto-import); perf-log F12
// status: PASS
// The global-resolution cache's invalidation contract (perf-log F12,
// `Chunk.gcaches` guarded by `ModuleObject.globals_version`). The IC twin is
// `ic_add_method_invalidates.ph`; this is the same shape for GetGlobal.
//
// A callsite linked to the canonical prelude `List` must remain linked to that
// declaration when a nested lexical binding uses the same spelling. The
// second class below separately covers invalidation of a real module-global
// slot.

class C {
  @class
  get { return List }
}

// Resolve through the core fallback, repeatedly, so any cache is warm.
System.print(C.get)
System.print(C.get)
System.print(C.get)

// A lexical shadow does not rewrite the already-linked prelude reference.
class Probe {
  @class
  run() {
    let List = 42
    System.print(C.get)
    List = 99
    System.print(C.get)
  }
}
Probe.run()

let x = 1
class D {
  @class
  get { x }
}
System.print(D.get)
x = 2
System.print(D.get)
