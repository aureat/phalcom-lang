// U-REOPEN-FIX regression coverage (reviewer-found gap): the fieldless
// `class_reopen_appends_methods.ph` fixture alone did not exercise the real
// failure mode — a method-only reopen of a class that ALREADY HAS FIELDS.
// Before the fix, `compiler/lib.rs`'s reopen-layout branch keyed its
// superclass/field-count lookup on the RUNTIME-populated `self.vm.classes`
// (empty at compile time for a same-unit reopen), fell through to the
// "implicit Object root" case, and unconditionally overwrote
// `field_layouts[name_sym]` with a zeroed layout — silently discarding
// block 1's real field slots before any bytecode ran. `Widget.new(42)`'s
// `_x` field then lived at a slot the zeroed layout no longer accounted
// for, and reading it back through `getX` raised "Field slot 0 out of
// bounds" (vm.rs, `Bytecode::GetField`).
//
// Block 1 declares the field (`_x`, via `construct new(x)`) and a getter;
// block 2 is method-only (adds `greet`, touches no fields) — exactly the
// shape the field-adding-reopen guard is supposed to let through
// unchanged. Both the inherited field read (`getX`) and the appended
// method (`greet`) must work on one instance.
class Widget {
    construct new(x) { _x = x }
    getX => _x
}

class Widget {
    greet => "hi"
}

var w = Widget.new(42)
System.print(w.getX)
System.print(w.greet)
