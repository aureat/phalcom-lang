let reader = BytesReader.new(Bytes.fromString("abc"))
System.print(reader.close.toString)
System.print(reader.close.toString)
try {
  reader.read(Bytes.new(1))
} catch e {
  System.print(e.class.name)
}

let writer = BytesWriter.new()
writer.close
System.print(writer.close.toString)
try {
  writer.write(Bytes.fromString("x"))
} catch e {
  System.print(e.class.name)
}

class CloseCountingReader {
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

let innerReader = CloseCountingReader.new()
let bufferedReader = BufferedReader.new(innerReader)
System.print(bufferedReader.close.toString)
System.print(innerReader.closeCount.toString)

let resourceReader = BytesReader.new(Bytes.fromString("abc"))
let bufferedResourceReader = BufferedReader.new(resourceReader)
bufferedResourceReader.close
System.print(resourceReader.isClosed.toString)
System.print(bufferedResourceReader.isClosed.toString)
bufferedResourceReader.close
System.print(resourceReader.isClosed.toString)
System.print(bufferedReader.close.toString)
System.print(innerReader.closeCount.toString)
