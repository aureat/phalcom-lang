// area: ic
// spec: modules.md (module globals + core auto-import); perf-log F12
// status: PASS
// The global-resolution cache's invalidation contract (perf-log F12,
// `Chunk.gcaches` guarded by `ModuleObject.globals_version`). The IC twin is
// `ic_add_method_invalidates.ph`; this is the same shape for GetGlobal.
//
// A callsite that resolves a kernel name through the core-module fallback must
// stop doing so the moment this module declares that name. The F12 prototype
// cached (module, slot) with no guard and returned core's `List` forever — this
// fixture is what fails if that regresses.

class C {
  static get { return List }
}

// Resolve through the core fallback, repeatedly, so any cache is warm.
System.print(C.get)
System.print(C.get)
System.print(C.get)

// Shadow the kernel name in this module. `declare` allocates a NEW slot, which
// bumps globals_version and must invalidate the cached core resolution above.
var List = 42
System.print(C.get)

// Assignment rewrites the slot's value without moving it, so the cache stays
// valid and must still observe the write.
List = 99
System.print(C.get)

// Same-scope re-declaration is now a compile error (L-3/L-5) — the
// idempotent-declare case this fixture used to exercise moved to
// binding_const_redeclared_same_scope.ph / binding_var_is_not_a_keyword.ph's
// sibling negative fixtures. Only the fresh-slot declare and plain-assignment
// cases above remain here.
