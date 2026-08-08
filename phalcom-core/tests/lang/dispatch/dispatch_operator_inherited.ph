// area: dispatch
// spec: method-lookup.md; messages-and-selectors.md
// status: PASS
// An operator method (`+`) defined ONLY on the parent (`Box`) is dispatched
// correctly on a subclass (`LoudBox`) instance that neither overrides nor
// redeclares it — the infix operator send resolves through the same
// inherited-lookup path as any other selector. (A `super.+(other)` override
// with an explicit fallthrough is NOT expressible here: the grammar's
// `super.<name>` production only accepts an identifier or `class` after the
// dot, not an operator token — see returned-report suspected gap.)

class Box value => _val
  @constructor
  new(_ v) { _val = v }
  +(_ other) { return Box.new(_val + other.value) }
}
class LoudBox is Box {
  @constructor
  new(_ v) { super.new(v) }
}
const a = LoudBox.new(2)
const b = LoudBox.new(3)
const c = a + b
System.print(c.value)
System.print(c.class.name)
