// U-CLASSCLOSE §2.1/§3, decision 0065 ruling 2: classes are closed. A second
// `class Point` in the same module is `class.already_defined`, not a reopen
// — the diagnostic carries both spans (this unit's option A: no
// compile-error renderer exists, so both locations go in the message text).
class Point {
    x => 1
}

class Point {
    y => 2
}
