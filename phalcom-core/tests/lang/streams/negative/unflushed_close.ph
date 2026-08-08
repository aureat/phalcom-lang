let writer = BytesWriter.new()
let bw = BufferedWriter.new(writer)
bw.write(Bytes.fromString("unflushed data"))
bw.close
