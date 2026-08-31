let writer = BytesWriter.new()
let bw = BufferedWriter.new(writer)
bw.write(Bytes.fromString("abc"))
System.print(bw.pending.toString)
bw.write(Bytes.fromString("defghijk"))
System.print(bw.pending.toString)
bw.flush.await
System.print(bw.pending.toString)
System.print(writer.toBytes.utf8)
