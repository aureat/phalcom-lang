// area: bytes
// spec: bytes.md law 7, §4; PDR-0011 (utf8_), PDR-0013 ruling 4 (utf8Lossy_)
// status: PASS
// fromString/utf8 roundtrip, strict decode is total via None, lossy decode
// is total via U+FFFD, empty-buffer decode is the empty string.

System.print(Bytes.fromString("Hi").utf8)
System.print(Bytes.fromString("héllo").utf8)
System.print(Bytes.fromList([255, 255]).utf8)
System.print(Bytes.fromList([72, 255]).utf8Lossy)
System.print(Bytes.new(0).utf8 == "")
System.print(Bytes.fromString("Hi").size)
System.print(Bytes.fromString("é").size)
