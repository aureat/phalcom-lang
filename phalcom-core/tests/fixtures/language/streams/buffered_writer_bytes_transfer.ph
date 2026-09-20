let writer = BytesWriter.new()
let bw = BufferedWriter.new(writer)
bw.write(Bytes.fromString("hello"))
bw.flush
System.print(writer.toBytes.utf8)
bw.close
writer.close
