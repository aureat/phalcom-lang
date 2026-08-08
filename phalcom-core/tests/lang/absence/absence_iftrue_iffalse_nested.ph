// area: absence
// spec: values-and-absence.md §3.3
// status: PASS
// Adversarial: a paired `ifTrue(_, ifFalse:_)` nested inside another paired
// form's `ifFalse:` arm — since each arm's block yields a raw value (not an
// Option), the inner paired send composes directly with no unwrap step.

const x = 5
const result = (x > 10).ifTrue(|| { "big" }, ifFalse: || { (x > 0).ifTrue(|| { "small positive" }, ifFalse: || { "non positive" }) })
System.print(result)
