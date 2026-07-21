let writer = BytesWriter.new()
let bw = BufferedWriter.new(writer)
bw.write(Bytes.fromString_("hello "))
System.print(bw.pending.toString)
bw.write(Bytes.fromString_("world"))
System.print(bw.pending.toString)
bw.flush
System.print(bw.pending.toString)
let res = writer.toBytes
System.print(res.utf8_)
bw.close
writer.close
