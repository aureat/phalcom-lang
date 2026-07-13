// U-REOPEN-FIX: a reopen must not change the superclass (U13 sealed
// inheritance). `Base` is unrelated to `Object` here only insofar as the
// second `class Gadget` block declares an explicit `extends Base` while the
// first block was an implicit `Object` root — that mismatch is rejected at
// compile time rather than silently reusing the wrong class.
class Base {}

class Gadget {
    greet => "hi"
}

class Gadget extends Base {
    farewell => "bye"
}
