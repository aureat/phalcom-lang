class RetryWriter {
  mut _reject
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

let mut writer = RetryWriter.new()
let mut bw = BufferedWriter.new(writer)
bw.write(Bytes.fromString("hello"))

let failure = || { bw.flush }.on(Error) |e| { e.message }
System.print(failure)
System.print(bw.pending.toString)
System.print(bw.finish.await.toString)
