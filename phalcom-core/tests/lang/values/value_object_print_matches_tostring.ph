// area: values
// spec: object-model.md §4; ADR-0015; U-CORE-4 (R-INV-4.2)
// status: PASS
// U-ERR-FIX PRINT-TOSTRING: `System.print` on a bare user instance, a
// class, and a metaclass all agree with an explicit `.toString` send —
// the print path (`Value::to_display_string`) now sends `toString` to any
// heap object with no bespoke native renderer instead of falling back to
// the debug form.

class Point {}
let p = Point.new()
System.print(p)
System.print(p.toString)
System.print(Point)
System.print(Point.toString)
System.print(Point.class)
System.print(Point.class.toString)
