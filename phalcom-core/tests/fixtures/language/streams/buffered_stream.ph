let mut writer = BytesWriter.new()
let mut bw = BufferedWriter.new(writer)
bw.write(Bytes.fromString("hello "))
System.print(bw.pending.toString)
bw.write(Bytes.fromString("world"))
System.print(bw.pending.toString)
bw.flush
System.print(bw.pending.toString)
let mut res = writer.toBytes
System.print(res.utf8)
bw.close
writer.close
