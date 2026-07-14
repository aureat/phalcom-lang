// bare_send.ph — dispatch-bound micro-benchmark (Tier 0, U-BENCH).
//
// Isolates the fixed per-send dispatch tax (performance.md §2 cost class 1:
// an `IndexMap<Symbol, ObjRef>` hash probe walked per superclass level,
// `lookup_method_in_hierarchy`, class.rs:65): a static, argument-free send
// to a user-defined method has no primitive operation on its path, so no
// per-send `Vec<Value>` allocation (vm.rs:626) happens either. Compare
// against arith_send.ph — same send count, but each send is a primitive
// arithmetic op — to attribute the delta to allocation rather than
// dispatch.
//
// Loaded by phalcom-core/benches/vm_bench.rs via `include_str!`; also
// runnable standalone: `phalcom benchmarks/vm/bare_send.ph`.
class Empty {
  static noop { return 0 }
}

var i = 0
while (i < 200000) {
  Empty.noop
  i = i + 1
}
System.print("done")
