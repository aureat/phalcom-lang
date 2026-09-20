class ScriptedWriter {
  @constructor
  new(_ counts) {
    _counts = counts
    _index = 0
    _out = Bytes.new(5)
    _written = 0
  }

  write(_ src) {
    let count = _counts.at(_index)
    _index = _index + 1
    src.slice(0, count).copyInto(_out, _written)
    _written = _written + count
    Future.value(count)
  }

  flush { Future.value(None) }

  toBytes {
    _out.slice(0, _written)
  }

  close { Result::Ok(None) }
}

class ZeroWriter {
  write(_ src) { Future.value(0) }
  flush { Future.value(None) }
  close { Result::Ok(None) }
}

class FailingWriter {
  @constructor
  new() {
    _out = Bytes.new(3)
    _written = 0
    _failed = false
    _recovered = false
  }

  write(_ src) {
    if (not _failed) {
      _failed = true
      src.slice(0, 2).copyInto(_out, 0)
      _written = 2
      return Future.value(2)
    }
    if (not _recovered) {
      _recovered = true
      throw Error.new("write failed")
    }
    src.copyInto(_out, _written)
    _written = _written + src.size
    Future.value(src.size)
  }

  flush { Future.value(None) }

  toBytes {
    _out.slice(0, _written)
  }

  close { Result::Ok(None) }
}

class FlushCountingWriter {
  @constructor
  new() {
    _out = Bytes.new(32)
    _written = 0
    _flushes = 0
  }

  write(_ src) {
    src.copyInto(_out, _written)
    _written = _written + src.size
    Future.value(src.size)
  }

  flush {
    _flushes = _flushes + 1
    Future.value(None)
  }

  toBytes { _out.slice(0, _written) }
  flushes { _flushes }
  close { Result::Ok(None) }
}

class PendingWriter {
  @constructor
  new() {
    _pending = Future.new()
    _size = 0
  }

  write(_ src) {
    _size = src.size
    _pending
  }
  flush { Future.value(None) }
  release { _pending.settleValue(_size) }
  close { Result::Ok(None) }
}

class SnapshotWriter {
  @constructor
  new() {
    _pending = Future.new()
    _source = None
    _out = None
  }

  write(_ src) {
    _source = Some(src)
    _pending
  }

  flush { Future.value(None) }

  release {
    _out = Some(_source.unwrapOr(None).slice(0, _source.unwrapOr(None).size))
    _pending.settleValue(_source.unwrapOr(None).size)
  }

  toBytes { _out.unwrapOr(Bytes.new(0)) }
  close { Result::Ok(None) }
}

const shortSink = ScriptedWriter.new([2, 1, 2])
const shortBuffered = BufferedWriter.new(shortSink)
shortBuffered.write(Bytes.fromString("ABCDE"))
shortBuffered.flush.await
System.print(shortSink.toBytes.utf8)

const zeroBuffered = BufferedWriter.new(ZeroWriter.new())
zeroBuffered.write(Bytes.fromString("zero"))
try {
  zeroBuffered.flush.await
} catch e {
  System.print(e.class.name)
}
System.print(zeroBuffered.pending.toString)

const failingSink = FailingWriter.new()
const failingBuffered = BufferedWriter.new(failingSink)
failingBuffered.write(Bytes.fromString("abc"))
try {
  failingBuffered.flush.await
} catch e {
  System.print(e.message)
}
System.print(failingSink.toBytes.utf8)
System.print(failingBuffered.pending.toString)
failingBuffered.flush.await
System.print(failingSink.toBytes.utf8)
System.print(failingBuffered.pending.toString)

const layeredSink = FlushCountingWriter.new()
const innerBuffered = BufferedWriter.new(layeredSink)
const outerBuffered = BufferedWriter.new(innerBuffered)
innerBuffered.write(Bytes.fromString("layered"))
outerBuffered.flush.await
System.print(layeredSink.toBytes.utf8)
System.print(layeredSink.flushes.toString)

const pendingSink = PendingWriter.new()
const pendingBuffered = BufferedWriter.new(pendingSink)
pendingBuffered.write(Bytes.fromString("pending"))
const unresolved = pendingBuffered.flush
try {
  pendingBuffered.write(Bytes.fromString("overlap"))
} catch e {
  System.print(e.class.name)
}
pendingSink.release
unresolved.await

const snapshotSink = SnapshotWriter.new()
const snapshotBuffered = BufferedWriter.new(snapshotSink)
const original = Bytes.new(8192)
original.set(0, 65)
const snapshotWrite = snapshotBuffered.write(original)
original.set(0, 90)
snapshotSink.release
snapshotWrite.await
System.print(snapshotSink.toBytes.at(0).toString)
