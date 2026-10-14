from .id import NanoId
from .alphabet import Alphabet
from .generator import NanoIdGenerator

let id = NanoId.generate()
System.print("Default Nano ID: \(id)")

let shortId = NanoId.generate(12)
System.print("Short Nano ID (12): \(shortId)")

let hexAlphabet = Alphabet.new("0123456789abcdef")
let hexGenerator = NanoIdGenerator.new(hexAlphabet, 16)
let hexId = hexGenerator.generate()
System.print("Hex Nano ID (16): \(hexId)")
