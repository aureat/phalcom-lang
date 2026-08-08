// U-CLASSCLOSE §2.2: a repeated method name inside one class body — no
// silent last-writer-wins.
class Point { val => 1
    val => 2
}
