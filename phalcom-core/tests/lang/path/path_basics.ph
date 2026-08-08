// Path & OpenMode construction + display
let p1 = Path.of("a/b.txt")
System.print(p1.toString)

let nonUtf8Bytes = Bytes.fromList([97, 47, 255, 46, 116, 120, 116])
let p2 = Path.ofBytes(nonUtf8Bytes)
System.print(p2.toString)
System.print(p2.bytes.at(2))

// Exclusive ownership
let srcBytes = Bytes.fromList([102, 111, 111])
let p3 = Path.ofBytes(srcBytes)
srcBytes.set(0, 98) // mutate source Bytes to 'b'
System.print(p3.toString) // remains "foo"

let outBytes = p3.bytes
outBytes.set(0, 98) // mutate returned Bytes
System.print(p3.toString) // remains "foo"

// Value semantics
let p4 = Path.of("a/b.txt")
System.print(p1 == p4)
System.print(p1 != p2)

let map = Map.new()
map.at(p1, put: "entry1")
System.print(map[p4])

// Lexical operations
System.print(Path.of("/a/b").isAbsolute)
System.print(Path.of("a/b").isAbsolute)

let joined1 = Path.of("a/b").join(Path.of("c/d"))
System.print(joined1.toString)

let joined2 = Path.of("a/b/").join(Path.of("c/d"))
System.print(joined2.toString)

let joinedAbs = Path.of("a/b").join(Path.of("/c/d"))
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

let comps = Path.of("//a///b//c/").components
let compStr = []
comps.each |c| { compStr.append(c.toString) }
System.print(compStr.join(","))

System.print(Path.of("a/../b") != Path.of("b"))

// OpenMode
let m1 = OpenMode.read
let m2 = OpenMode.read
let m3 = OpenMode.write
System.print(m1 == m2)
System.print(m1 != m3)
System.print(m1.toString)
System.print(OpenMode.append.toString)
System.print(OpenMode.readWrite.toString)

