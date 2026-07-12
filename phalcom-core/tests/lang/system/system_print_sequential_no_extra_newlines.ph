// area: system
// spec: system.md
// status: PASS
// Adversarial byte-level check: five sequential `System.print` calls emit
// EXACTLY one `\n` per call and nothing else — no accumulating blank lines,
// no leading/trailing padding beyond the final call's own newline.

System.print(1)
System.print("a")
System.print(true)
System.print(2)
System.print("b")
