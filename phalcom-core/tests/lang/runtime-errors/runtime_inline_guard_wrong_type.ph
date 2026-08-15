// area: errors
// spec: control-flow.md §1; ADR-0017
// status: NEGATIVE
// U5 Layer 1: the `ifTrue(_:ifFalse:)` inliner emits a GuardBool on the
// receiver. A non-Bool receiver (`5`) fails the guard and deopts to the
// real message send — which Number does not implement — so the program
// fails with a clean "not found" rather than mis-executing the inline path.

const x = 5.ifTrue { "a" }
  ifFalse: { "b" }

System.print("Never reached to print: \(x)")