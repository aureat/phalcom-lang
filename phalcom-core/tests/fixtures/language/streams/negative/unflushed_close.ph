let mut writer = BytesWriter.new()
let mut bw = BufferedWriter.new(writer)
bw.write(Bytes.fromString("unflushed data"))
bw.close
