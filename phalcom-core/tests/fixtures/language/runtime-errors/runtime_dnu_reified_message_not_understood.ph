// area: errors
// spec: method-lookup.md; object-model.md §4 "Errors"; ADR-0008; U-CORE-6
// status: NEGATIVE
// A genuine miss on a *user* object (no `doesNotUnderstand(_)` override)
// raises a surface `MessageNotUnderstood < Error` through the unified
// unwind — the render stays byte-identical to the retired native
// `RuntimeError::MessageNotUnderstood` path (R-INV-6.2).

class Widget {}
System.print(Widget.new().frobnicate())
