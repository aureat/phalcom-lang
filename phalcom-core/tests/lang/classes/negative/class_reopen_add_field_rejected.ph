// U-REOPEN-FIX (scope decision): a reopen that ADDS FIELDS is out of scope —
// the runtime reopen path (`vm.rs` `Bytecode::Class`) reuses the
// already-allocated `ClassId` and does not relayout its instances, so a
// later block's new field would never get real storage. The compiler
// rejects this at compile time with a clear diagnostic instead of silently
// mishandling it.
class Widget {
    greet => "hi"
}

class Widget {
    construct new(x) { _x = x }
}
