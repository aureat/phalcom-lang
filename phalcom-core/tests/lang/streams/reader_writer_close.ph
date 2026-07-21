let reader = BytesReader.new(Bytes.fromString_("abc"))
System.print(reader.close.toString)
System.print(reader.close.toString)
try {
  reader.read(Bytes.new(1))
} catch e {
  System.print(e.class.name)
}

let writer = BytesWriter.new()
writer.close
System.print(writer.close.toString)
try {
  writer.write(Bytes.fromString_("x"))
} catch e {
  System.print(e.class.name)
}
