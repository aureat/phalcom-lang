// Path & OpenMode construction + display
let mut p1 = Path.of("a/b.txt")
System.print(p1.toString)

let byteValues: List<Int> = [97, 47, 255, 46, 116, 120, 116]
let mut nonUtf8Bytes = Bytes.fromList(byteValues)
let mut p2 = Path.ofBytes(nonUtf8Bytes)
System.print(p2.toString)
System.print(p2.bytes.at(2))

// Exclusive ownership
let sourceBytes: List<Int> = [102, 111, 111]
let mut srcBytes = Bytes.fromList(sourceBytes)
let mut p3 = Path.ofBytes(srcBytes)
srcBytes.set(0, 98) // mutate source Bytes to 'b'
System.print(p3.toString) // remains "foo"

let mut outBytes = p3.bytes
outBytes.set(0, 98) // mutate returned Bytes
System.print(p3.toString) // remains "foo"

// Value semantics
let mut p4 = Path.of("a/b.txt")
System.print(p1 == p4)
System.print(p1 != p2)

let mut map: Map<Dynamic, Dynamic> = Map<Dynamic, Dynamic>.new()
map.at(p1, put: "entry1")
System.print(map[p4])

// Lexical operations
System.print(Path.of("/a/b").isAbsolute)
System.print(Path.of("a/b").isAbsolute)

let mut joined1 = Path.of("a/b").join(Path.of("c/d"))
System.print(joined1.toString)

let mut joined2 = Path.of("a/b/").join(Path.of("c/d"))
System.print(joined2.toString)

let mut joinedAbs = Path.of("a/b").join(Path.of("/c/d"))
System.print(joinedAbs.toString)

System.print(Path.of("a/b/c").parent.toString)
System.print(Path.of("/a").parent.toString)
System.print(Path.of("a").parent) // None
System.print(Path.of("/").parent) // None

System.print(Path.of("a/b.txt").fileName.toString)
System.print(Path.of("/a/b/").fileName) // None
System.print(Path.of("/").fileName) // None

System.print(Path.of("a/b.txt").extension)
System.print(Path.of(".bashrc").extension) // None
System.print(Path.of("a/b/").extension) // None
System.print(Path.of("a/b").extension) // None

let mut comps = Path.of("//a///b//c/").components
let mut compStr: List<String> = []
comps.each |c| { compStr.append(c.toString) }
System.print(compStr.join(","))

System.print(Path.of("a/../b") != Path.of("b"))

// OpenMode
let mut m1 = OpenMode.read
let mut m2 = OpenMode.read
let mut m3 = OpenMode.write
System.print(m1 == m2)
System.print(m1 != m3)
System.print(m1.toString)
System.print(OpenMode.append.toString)
System.print(OpenMode.readWrite.toString)
