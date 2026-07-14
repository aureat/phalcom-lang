// area: strings
// spec: core-classes.md §String; docs/forge/units/U-STRING/plan.md §2.4, ADR-0048
// status: PASS
// bytes/codePoints sequence views over a string containing a 2-byte (€) and
// a 4-byte (🎉) UTF-8 codepoint: byte-level iteration sees every raw byte,
// codepoint-level iteration sees one entry per Unicode scalar value.

let s = "a€🎉"
System.print(s.rawByteCount)
System.print(s.bytes.size)
System.print(s.codePoints.size)

s.codePoints.each({ cp => System.print(cp) })
