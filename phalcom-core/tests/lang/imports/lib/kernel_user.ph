// Imported by imports_kernel_visible_without_import.ph — not a standalone
// test driver. Uses `List` with no `import` of it — kernel classes are
// visible in every module without import (U15 plan §4).
var xs = List.new()
xs.add(1)
xs.add(2)
var total = xs.at(0) + xs.at(1)
