// area: runtime-errors
// spec: U-STRING plan.md §2.0 Rubric (slice_ boundary safety)
// status: NEGATIVE
// slice_ with misaligned UTF-8 boundaries must return RuntimeError, not panic.

let s = "héllo"
// 'h' = 1 byte (U+0068), 'é' = 2 bytes (U+00E9: 0xC3 0xA9 in UTF-8)
// So byte offsets are: [0]=h, [1-2]=é, [3-5]=llo
// Slicing from byte 1 to 2 would split the 'é' sequence.
System.print(s.slice_(1, 2))
