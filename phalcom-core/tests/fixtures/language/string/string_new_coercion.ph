// area: string
// spec: core/core-classes.md
// status: PASS
// Wren precedent: test/core/string/no_constructor.wren — Wren's `String`
// metaclass has NO constructor at all (`String.new()` is a runtime error
// there). Phalcom's `String` floor deliberately diverges: `String.class::new`
// (`string_class_new`, primitive/string.rs) is a real, supported
// constructor — zero-arg gives the empty string, one-arg renders the
// argument via `Value::to_string` (the same rendering `\(expr)`
// interpolation and `System.print` use), so it doubles as Phalcom's
// stringify-any-value coercion. This fixture pins that divergence rather
// than porting Wren's "no constructor" error.
System.print(String.new())
System.print(String.new() == "")
System.print(String.new(123))
System.print(String.new(true))
System.print(String.new("already a string"))
