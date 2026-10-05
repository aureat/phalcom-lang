let mut reader = BytesReader.new(Bytes.fromString("abc"))
System.print(reader.close.toString)
System.print(reader.close.toString)
try {
  reader.read(Bytes.new(1))
} catch e {
  System.print(e.class.name)
}

let mut writer = BytesWriter.new()
writer.close
System.print(writer.close.toString)
try {
  writer.write(Bytes.fromString("x"))
} catch e {
  System.print(e.class.name)
}

class CloseCountingReader {
  mut _closeCount
  @constructor
  new() {
    _closeCount = 0
  }

  close {
    _closeCount = _closeCount + 1
    Result::Ok(None)
  }

  closeCount { _closeCount }
}

let mut innerReader = CloseCountingReader.new()
let mut bufferedReader = BufferedReader.new(innerReader)
System.print(bufferedReader.close.toString)
System.print(innerReader.closeCount.toString)

let mut resourceReader = BytesReader.new(Bytes.fromString("abc"))
let mut bufferedResourceReader = BufferedReader.new(resourceReader)
bufferedResourceReader.close
System.print(resourceReader.isClosed.toString)
System.print(bufferedResourceReader.isClosed.toString)
bufferedResourceReader.close
System.print(resourceReader.isClosed.toString)
System.print(bufferedReader.close.toString)
System.print(innerReader.closeCount.toString)
