// area: compile-errors
// spec: ADR-0064 §5; U-BINDINGS §5
// status: NEGATIVE
// A write to a `const` field outside the constructor is `field.const_write`.

class Account {
  const _n = 0
  clobber(v) { _n = v }
}
