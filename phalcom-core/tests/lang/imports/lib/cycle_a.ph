// Imported by imports_cyclic_load_no_hang.ph — not a standalone test
// driver. Half of a mutual-import cycle with cycle_b.ph (U15 plan §4/§6).
import "./cycle_b" as B
var late = 10
