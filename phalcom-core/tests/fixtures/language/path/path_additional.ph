let invalidNameBytes: List<Int> = [97, 47, 255, 46, 116, 120, 116]
let invalidExtensionBytes: List<Int> = [97, 46, 255]
let validExtensionBytes: List<Int> = [97, 46, 116, 120, 116]
let leadingDotBytes: List<Int> = [46, 255]
let mut badName = Path.ofBytes(Bytes.fromList(invalidNameBytes))
System.print(badName.extension)
System.print(Path.ofBytes(Bytes.fromList(invalidExtensionBytes)).extension)
System.print(Path.ofBytes(Bytes.fromList(validExtensionBytes)).extension)
System.print(Path.ofBytes(Bytes.fromList(leadingDotBytes)).extension)
