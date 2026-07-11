// area: classes
// spec: classes.md; messages-and-selectors.md
// status: PASS
// U5: `==` is now a real dispatched send (control-flow.md §1), so `Pt`'s own
// `==(other)` (always `true`) actually runs for *every* comparison,
// including `a == Pt.new(1)`. Previously `==` bypassed method lookup
// entirely (a documented bug, MANIFEST.md's "user-defined ==(other) never
// dispatched for instances"), so this fixture's `.expected` pinned that
// bug's accidental identity-comparison output; U5 fixes the dispatch and
// this is updated to match. See class_operator_equals_custom_dispatch.ph
// (graduated from pending/ in this same unit) for the real-world case with
// a field-comparing `==`.

class Pt {
  x => _x
  static new(x) {
    let p = self.new();
    p.init(x);
    return p;
  }
  init(x) {
    _x = x;
  }
  ==(other) {
    return true;
  }
}
let a = Pt.new(1)
System.print(a == a)
System.print(a == Pt.new(1))
