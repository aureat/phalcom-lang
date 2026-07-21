let writer = BytesWriter.new()
let bw = BufferedWriter.new(writer)
bw.write(Bytes.fromString_("unflushed data"))
bw.close
