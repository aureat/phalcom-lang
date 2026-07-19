// U-REOPEN-FIX (ADR-0018 "attaches methods, not shadows"): a second
// surface-level `class A { ... }` block for a name already defined in this
// compile unit APPENDS its methods onto the first block's class instead of
// shadowing it with a fresh `ClassId` (the root-cause bug: `Bytecode::Class`
// unconditionally allocated a new class, orphaning the first block's
// methods). Both `greet` (block 1) and `farewell` (block 2) must resolve on
// the same instance.
class Greeter {
    greet => "hi"
}

class Greeter {
    farewell => "bye"
}

let g = Greeter.new()
System.print(g.greet)
System.print(g.farewell)
