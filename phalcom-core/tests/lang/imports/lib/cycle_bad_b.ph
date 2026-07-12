// Imported by cycle_bad_a.ph — not a standalone test driver. Re-enters
// cycle_bad_a.ph mid-load (its "late" global is not yet defined) and reads
// it immediately — the partial-init hazard this cycle is built to trip.
import "./cycle_bad_a" as A
System.print(A.late)
