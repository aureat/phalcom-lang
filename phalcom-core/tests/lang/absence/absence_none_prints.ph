// area: absence
// spec: values-and-absence.md; ADR-0007
// status: PASS
// U6: `None` is a global bound to the shared singleton. Its default print is
// `<None instance>` (the Object printString); a prettier `None` printString is
// U-STD's job, so this pins today's substrate output, not the final surface.

System.print(None)
