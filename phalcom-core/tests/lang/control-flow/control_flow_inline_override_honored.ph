// area: control flow
// spec: control-flow.md §1; ADR-0017
// status: PASS
// U5 Layer 1: `whileTrue(_:)` is speculatively inlined, but the inliner is
// guarded by an override epoch. Reopening `Block` to redefine `whileTrue`
// flips the epoch, so the guard fails and the send deopts to the user's
// method — the inline fast path must NOT win over a runtime override.

class Block {
  whileTrue(body) {
    return "overridden-loop"
  }
}
var i = 0
System.print({ i < 3 }.whileTrue { i = i + 1 })
