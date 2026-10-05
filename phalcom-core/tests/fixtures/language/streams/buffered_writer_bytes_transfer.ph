let mut writer = BytesWriter.new()
let mut bw = BufferedWriter.new(writer)
bw.write(Bytes.fromString("hello"))
bw.flush
System.print(writer.toBytes.utf8)
bw.close
writer.close
