class RetryWriter {
  @constructor
  new() { _reject = true }

  write(_ src) {
    if (_reject) {
      _reject = false
      throw Error.new("not accepted")
    }
    return Future.value(src.size)
  }

  close { Result::Ok(None) }
}

let writer = RetryWriter.new()
let bw = BufferedWriter.new(writer)
bw.write(Bytes.fromString("hello"))

const failure = || { bw.flush }.on(Error) |e| { e.message }
System.print(failure)
System.print(bw.pending.toString)
System.print(bw.finish.await.toString)
