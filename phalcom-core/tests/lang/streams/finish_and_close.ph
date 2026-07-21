let writer = BytesWriter.new()
let bw = BufferedWriter.new(writer)
bw.write(Bytes.fromString_("hello "))
bw.write(Bytes.fromString_("world"))
System.print(bw.finish.await.toString)
System.print(bw.close.toString)
System.print(writer.toBytes.utf8_)
writer.close
