let writer = BytesWriter.new()
let bw = BufferedWriter.new(writer)
bw.write(Bytes.fromString("hello "))
bw.write(Bytes.fromString("world"))
System.print(bw.finish.await.toString)
System.print(bw.close.toString)
System.print(writer.toBytes.utf8)
writer.close
