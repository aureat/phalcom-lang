let writer = BytesWriter.new()
let bw = BufferedWriter.new(writer)
bw.write(Bytes.fromString_("abc"))
System.print(bw.pending.toString)
bw.write(Bytes.fromString_("defghijk"))
System.print(bw.pending.toString)
bw.flush.await
System.print(bw.pending.toString)
System.print(writer.toBytes.utf8_)
