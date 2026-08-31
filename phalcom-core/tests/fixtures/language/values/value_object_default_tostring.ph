// area: values
// spec: object-model.md §4; ADR-0015; U-CORE-4 (R-INV-4.2)
// status: PASS
// A user class instance falls back to `Object#toString`'s `"<ClassName>"`
// default (re-homed off `object_name`, DEFERRED F4) when it defines no
// override of its own.

class Foo {}
System.print(Foo.new().toString)
