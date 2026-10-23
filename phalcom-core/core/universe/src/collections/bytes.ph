from universe.collections.iteration import Iterator as CanonicalIterator, Iterable as CanonicalIterable

/// Mutable sequence of octets with byte-oriented operations.
native class Bytes is Collection {
  @class
  native new(_ value: Dynamic) -> Bytes
  @class
  internal native rawFromString(_ value: String) -> Bytes
  internal native rawSize -> Int
  internal native rawAt(_ index: Int) -> Int
  internal native rawSet(_ index: Int, _ value: Int) -> Dynamic
  internal native rawFill(_ value: Int) -> Dynamic
  internal native rawSlice(_ start: Int, _ end: Int) -> Bytes
  internal native rawCopyInto(_ target: Dynamic, _ offset: Int) -> Dynamic
  internal native rawUtf8 -> Option<String>
  internal native rawUtf8Lossy -> String
  internal native rawEqualsConstantTime(_ other: Bytes) -> Bool
  size { self.rawSize }

  first {
    if self.size == 0 { return None }
    return Some(self.at(0))
  }

  last {
    if self.size == 0 { return None }
    return Some(self.at(self.size - 1))
  }

  at(_ i) { self.rawAt(i) }

  get(_ index) {
    let mut raw = self.rawAt(index)
    let mut len = self.size
    let mut i = index
    if i < 0 { i = len + i }
    if i >= 0 and i < len {
      return Some(raw)
    }
    return None
  }

  [_ index] {
    if index is Range {
      return index.rawSliceBounds(self.size).match(
        ok: |bounds| {
          let mut start = bounds[0]
          let mut end = bounds[1]
          if start > end {
            end = start
          }
          self.rawSlice(start, end)
        },
        err: |error| { error.raise() }
      )
    }
    let mut raw = self.rawAt(index)
    let mut len = self.size
    let mut i = index
    if i < 0 { i = len + i }
    if i >= 0 and i < len {
      return raw
    }
    throw IndexError.new("Bytes index out of range")
  }

  [_ index, default] {
    let mut raw = self.rawAt(index)
    let mut len = self.size
    let mut i = index
    if i < 0 { i = len + i }
    if i >= 0 and i < len {
      return raw
    }
    return default
  }

  get(_ index, orElse) {
    let mut raw = self.rawAt(index)
    let mut len = self.size
    let mut i = index
    if i < 0 { i = len + i }
    if i >= 0 and i < len {
      return raw
    }
    return orElse.call(index)
  }

  /// Returns a fresh iterator over the bytes.
  iterator -> BytesIterator { BytesIterator.new(self) }

  /// Writes an octet at `i` and returns the byte buffer.
  set(_ i, _ v) {
    if v is not Number or v < 0 or v > 255 or v % 1 != 0 {
      throw ArgumentError.new("Bytes#set: value must be an integer in 0..255")
    }
    let mut len = self.size
    let mut norm = i
    if norm < 0 { norm = len + norm }
    if norm < 0 or norm >= len {
      throw IndexError.new("Bytes index out of range")
    }
    self.rawSet(i, v)
    return self
  }

  /// Writes an octet using the collection assignment protocol.
  at(_ i, put) { return self.set(i, put) }

  /// Writes an octet using subscript assignment.
  [_ i]=(_ val) { return self.set(i, val) }

  /// Fills the buffer with one octet and returns the buffer.
  fill(_ v) {
    if v is not Number or v < 0 or v > 255 or v % 1 != 0 {
      throw ArgumentError.new("Bytes#fill: value must be an integer in 0..255")
    }
    self.rawFill(v)
    return self
  }

  /// Overwrites every byte with zero and returns the byte buffer.
  zeroize {
    self.rawFill(0)
    return self
  }

  /// Decodes bytes as UTF-8, returning `None` for invalid data.
  utf8 { self.rawUtf8 }

  /// Decodes bytes as UTF-8, replacing invalid sequences.
  utf8Lossy { self.rawUtf8Lossy }

  /// Returns the half-open slice `[start, end)`.
  slice(_ start, _ end) {
    if start < 0 or end < start or end > self.size {
      throw ArgumentError.new("Bytes#slice: range must satisfy 0 <= start <= end <= size")
    }
    return self.rawSlice(start, end)
  }

  /// Copies this buffer into `dst` starting at `offset`.
  copyInto(_ dst, _ offset) {
    if dst is not Bytes {
      throw ArgumentError.new("Bytes#copyInto: destination must be a Bytes")
    }
    if offset < 0 or offset + self.size > dst.size {
      throw ArgumentError.new("Bytes#copyInto: offset + size must fit the destination")
    }
    self.rawCopyInto(dst, offset)
    return self
  }

  /// Returns a new buffer containing this buffer followed by `other`.
  concat(_ other) {
    if other is not Bytes {
      throw ArgumentError.new("Bytes#concat: argument must be a Bytes")
    }
    let out = Bytes.new(self.size + other.size)
    self.rawCopyInto(out, 0)
    other.rawCopyInto(out, self.size)
    return out
  }

  /// Compares buffers without early exit on differing bytes.
  equalsConstantTime(_ other) { self.rawEqualsConstantTime(other) }

  /// Compares byte buffers by length and corresponding octets.
  ==(_ other) {
    if other is Bytes {
      let mut same = self.size == other.size
      let mut i = 0
      while same and i < self.size {
        same = self.at(i) == other.at(i)
        i = i + 1
      }
      return same
    }
    return false
  }

  /// Returns the inverse of structural byte-buffer equality.
  !=(_ other) {
    return not (self == other)
  }

  /// Renders the byte count without exposing contents.
  toString { "Bytes(" + self.size.toString + ")" }

  /// Copies the bytes into a list of integers.
  toList {
    let out: List<Int> = []
    for b in self {
      out.append(b)
    }
    return out
  }

  /// Returns an immutable tuple snapshot of the bytes.
  toTuple { Tuple.rawFromList(self.toList) }

  @class
  fromString(_ s) {
    if s is not String {
      throw ArgumentError.new("Bytes.fromString: argument must be a String")
    }
    return Bytes.rawFromString(s)
  }

  /// Builds a byte buffer from a list of octet values.
  @class
  fromList(_ list) {
    if list is not List {
      throw ArgumentError.new("Bytes.fromList: argument must be a List")
    }
    let out = Bytes.new(list.size)
    let mut i = 0
    while i < list.size {
      out.set(i, list.at(i))
      i = i + 1
    }
    return out
  }
}

class BytesIterator {
  mut _source: Bytes
  mut _index: Int
  mut _done: Bool

  @constructor
  new(_ source: Bytes) {
    _source = source
    _index = 0
    _done = false
  }

  next -> Option<Int> {
    if _done { return None }
    if _index >= _source.size {
      _done = true
      return None
    }
    let mut value = _source.at(_index)
    _index = _index + 1
    Some(value)
  }

  iterator { self }
}

impl CanonicalIterator for BytesIterator {
  type Item = Int
}

impl CanonicalIterable for Bytes {
  type Item = Int
  type Iterator = BytesIterator
  iterator -> BytesIterator { BytesIterator.new(self) }
}

class OpenMode {
  mut _name
  @constructor
  private named(_ n) { _name = n }
  @class
  read { OpenMode.named("read") }
  @class
  write { OpenMode.named("write") }
  @class
  append { OpenMode.named("append") }
  @class
  readWrite { OpenMode.named("readWrite") }
  name { _name }
  /// Compares open modes by name.
  ==(_ other) {
    if other is not OpenMode {
      return false
    }
    _name == other.name
  }

  /// Returns the inverse of open-mode equality.
  !=(_ other) { return not (self == other) }

  /// Returns the qualified mode name.
  toString { "OpenMode." + _name }
}

class Path {
  mut _bytes
  mut _hash
  @constructor
  of(_ s) {
    if s is not String {
      throw ArgumentError.new("Path.of: argument must be a String")
    }
    _bytes = Bytes.fromString(s)
    _hash = Path.contentHash(_bytes)
  }

  @constructor
  ofBytes(_ b) {
    if b is not Bytes {
      throw ArgumentError.new("Path.ofBytes: argument must be a Bytes")
    }
    _bytes = b.slice(0, b.size)
    _hash = Path.contentHash(_bytes)
  }

  @class
  contentHash(_ bytes) {
    let mut acc = 1
    let mut i = 0
    while i < bytes.size {
      acc = (acc * 31 + bytes.at(i)) % 999999937
      i = i + 1
    }
    return acc
  }

  bytes { _bytes.slice(0, _bytes.size) }
  hash { _hash }

  /// Compares paths by cached hash and byte contents.
  ==(_ other) {
    if other is not Path {
      return false
    }
    if _hash != other.hash {
      return false
    }
    _bytes == other.bytes
  }

  /// Returns the inverse of path equality.
  !=(_ other) { return not (self == other) }

  /// Reports whether the path starts at the root.
  isAbsolute { _bytes.size > 0 and _bytes.at(0) == 47 }

  join(_ other) {
    if other is not Path {
      throw ArgumentError.new("Path#join: argument must be a Path")
    }
    if other.isAbsolute {
      return other
    }
    let mut recv = _bytes
    let mut recvLen = recv.size
    while recvLen > 0 and recv.at(recvLen - 1) == 47 {
      recvLen = recvLen - 1
    }
    let trimmedRecv = recv.slice(0, recvLen)
    let sep = Bytes.fromString("/")
    let combined = trimmedRecv.concat(sep).concat(other.bytes)
    return Path.ofBytes(combined)
  }

  parent {
    let mut len = _bytes.size
    while len > 0 and _bytes.at(len - 1) == 47 {
      len = len - 1
    }
    let mut idx = len - 1
    while idx >= 0 and _bytes.at(idx) != 47 {
      idx = idx - 1
    }
    if idx < 0 {
      return None
    }
    if idx == 0 {
      return Path.of("/")
    }
    let mut pLen = idx
    while pLen > 0 and _bytes.at(pLen - 1) == 47 {
      pLen = pLen - 1
    }
    if pLen == 0 {
      return Path.of("/")
    }
    return Path.ofBytes(_bytes.slice(0, pLen))
  }

  fileName {
    let mut len = _bytes.size
    if len > 0 and _bytes.at(len - 1) == 47 {
      return None
    }
    let mut idx = len - 1
    while idx >= 0 and _bytes.at(idx) != 47 {
      idx = idx - 1
    }
    if idx < 0 and len == 0 {
      return None
    }
    let nameBytes = _bytes.slice(idx + 1, len)
    if nameBytes.size == 0 {
      return None
    }
    return Path.ofBytes(nameBytes)
  }

  extension {
    let namePath = self.fileName
    if namePath == None {
      return None
    }
    let nb = namePath.bytes
    let mut idx = nb.size - 1
    while idx >= 0 and nb.at(idx) != 46 {
      idx = idx - 1
    }
    if idx <= 0 or idx == nb.size - 1 {
      return None
    }
    let extBytes = nb.slice(idx + 1, nb.size)
    return extBytes.utf8
  }

  components {
    let mut res = List<Dynamic>.new()
    let mut i = 0
    let mut len = _bytes.size
    while i < len {
      while i < len and _bytes.at(i) == 47 {
        i = i + 1
      }
      if i < len {
        let mut start = i
        while i < len and _bytes.at(i) != 47 {
          i = i + 1
        }
        res.append(Path.ofBytes(_bytes.slice(start, i)))
      }
    }
    return res
  }

  toString { _bytes.utf8Lossy }
}


/// Resource and in-memory byte-stream types.

native class Resource is Object {
  internal native attach(_ kind: String) -> Unit
  internal native rawClose() -> Dynamic
  internal native rawIsClosed -> Bool
  internal native ensureOpen -> Unit
  close {
    self.rawClose()
    return Result::Ok(None)
  }
  isClosed { self.rawIsClosed }
}

class UnflushedError is Error {}
class ConcurrentOperationError is Error {}
class StreamProtocolError is Error {}
class WriteZeroError is Error {}

class BytesReader is Resource {
  mut _data
  mut _pos
  @constructor
  new(_ source) {
    if source is not Bytes {
      throw ArgumentError.new("BytesReader source must be a Bytes")
    }
    self.attach("BytesReader")
    _data = source.slice(0, source.size)
    _pos = 0
  }

  /// Reads up to the destination capacity and returns the byte count.
  read(_ dst) {
    if dst is not Bytes {
      throw ArgumentError.new("dst must be a Bytes")
    }
    self.ensureOpen
    if self.isClosed {
      throw UseAfterCloseError.new("cannot read from closed BytesReader")
    }
    let mut remaining = _data.size - _pos
    let mut n = dst.size
    if remaining < n {
      n = remaining
    }
    if n > 0 {
      _data.slice(_pos, _pos + n).copyInto(dst, 0)
      _pos = _pos + n
    }
    Future.value(n)
  }
}

class BytesWriter is Resource {
  mut _chunks
  @constructor
  new() {
    self.attach("BytesWriter")
    _chunks = List<Dynamic>.new()
  }

  /// Appends a copy of `src` and returns the number of bytes written.
  write(_ src) {
    if src is not Bytes {
      throw ArgumentError.new("src must be a Bytes")
    }
    self.ensureOpen
    if self.isClosed {
      throw UseAfterCloseError.new("cannot write to closed BytesWriter")
    }
    _chunks.rawPush(src.slice(0, src.size))
    return Future.value(src.size)
  }

  flush {
    return Future.value(None)
  }

  toBytes {
    let mut total = 0
    let mut it = _chunks.iterator
    let mut step = it.next
    while step.isSome {
      let mut c = step.match(some: |value| { value }, none: || { None })
      total = total + c.size
      step = it.next
    }
    let mut res = Bytes.new(total)
    let mut offset = 0
    let mut copyIt = _chunks.iterator
    let mut copyStep = copyIt.next
    while copyStep.isSome {
      let mut c = copyStep.match(some: |value| { value }, none: || { None })
      c.copyInto(res, offset)
      offset = offset + c.size
      copyStep = copyIt.next
    }
    return res
  }
}

class BufferedWriter is Resource {
  mut _inner
  mut _buf
  mut _len
  mut _busy
  @constructor
  new(_ inner) {
    self.attach("BufferedWriter")
    _inner = inner
    _buf = Bytes.new(8192)
    _len = 0
    _busy = false
  }

  pending { _len }

  restorePending(_ chunk, _ offset) {
    let remaining = chunk.size - offset
    if remaining == 0 {
      _len = 0
      return ()
    }
    chunk.slice(offset, chunk.size).copyInto(_buf, 0)
    _len = remaining
    ()
  }

  drainFrom(_ chunk, _ offset) {
    let remaining = chunk.size - offset
    if remaining == 0 {
      _len = 0
      return Future.value(None)
    }

    let mut submitted
    try {
      submitted = _inner.write(chunk.slice(offset, chunk.size))
    } catch e {
      self.restorePending(chunk, offset)
      throw e
    }

    return submitted.catch |error| {
      self.restorePending(chunk, offset)
      throw error
    }.then |count| {
      if count is not Number or count < 0 or count > remaining or count % 1 != 0 {
        self.restorePending(chunk, offset)
        throw StreamProtocolError.new("writer returned an invalid byte count")
      }
      if count == 0 {
        self.restorePending(chunk, offset)
        throw WriteZeroError.new("writer made no progress while draining")
      }
      self.drainFrom(chunk, offset + count)
    }
  }

  drainBuffer {
    if _len == 0 {
      return Future.value(None)
    }
    let chunk = _buf.slice(0, _len)
    self.drainFrom(chunk, 0)
  }

  closeClean {
    let result = _inner.close
    super.close()
    result
  }

  /// Writes `src`, buffering small writes and draining large writes.
  write(_ src) {
    if src is not Bytes {
      throw ArgumentError.new("src must be a Bytes")
    }
    self.ensureOpen
    if self.isClosed {
      throw UseAfterCloseError.new("cannot write to closed BufferedWriter")
    }
    if _busy {
      throw ConcurrentOperationError.new("BufferedWriter operation is already in progress")
    }
    if src.size == 0 {
      return Future.value(0)
    }

    let owned = src.slice(0, src.size)

    if owned.size >= _buf.size {
      _busy = true
      let mut operation
      try {
        operation = self.drainBuffer.then |_| {
          _inner.write(owned)
        }.then |count| {
          if count is not Number or count < 0 or count > owned.size or count % 1 != 0 {
            throw StreamProtocolError.new("writer returned an invalid byte count")
          }
          count
        }
      } catch e {
        _busy = false
        throw e
      }
      return operation.then |count| {
        _busy = false
        count
      }.catch |error| {
        _busy = false
        throw error
      }
    }

    if _len + owned.size > _buf.size {
      _busy = true
      let mut operation
      try {
        operation = self.drainBuffer.then |_| {
          owned.copyInto(_buf, 0)
          _len = owned.size
          owned.size
        }
      } catch e {
        _busy = false
        throw e
      }
      return operation.then |count| {
        _busy = false
        count
      }.catch |error| {
        _busy = false
        throw error
      }
    }

    owned.copyInto(_buf, _len)
    _len = _len + owned.size
    Future.value(owned.size)
  }

  /// Drains buffered data and flushes the wrapped writer.
  flush {
    self.ensureOpen
    if self.isClosed {
      throw UseAfterCloseError.new("cannot flush closed BufferedWriter")
    }
    if _busy {
      throw ConcurrentOperationError.new("BufferedWriter operation is already in progress")
    }
    _busy = true
    let mut operation
    try {
      operation = self.drainBuffer.then |_| {
        _inner.flush
      }.then |_| {
        _busy = false
        None
      }
    } catch e {
      _busy = false
      throw e
    }
    operation.catch |error| {
      _busy = false
      throw error
    }
  }

  close {
    if self.isClosed {
      return Result::Ok(None)
    }
    if _busy {
      throw ConcurrentOperationError.new("BufferedWriter operation is already in progress")
    }
    if _len > 0 {
      throw UnflushedError.new("BufferedWriter closed with " + _len.toString + " pending bytes")
    }
    self.closeClean
  }

  /// Drains, flushes, and closes the wrapped writer.
  finish {
    if self.isClosed {
      throw UseAfterCloseError.new("cannot finish closed BufferedWriter")
    }
    if _busy {
      throw ConcurrentOperationError.new("BufferedWriter operation is already in progress")
    }
    _busy = true
    let mut operation
    try {
      operation = self.drainBuffer.then |_| {
        _inner.flush
      }.then |_| {
        self.closeClean
      }.then |result| {
        _busy = false
        result
      }
    } catch e {
      _busy = false
      throw e
    }
    operation.catch |error| {
      _busy = false
      throw error
    }
  }
}

class BufferedReader is Resource {
  mut _inner
  mut _buf
  mut _pos
  mut _len
  @constructor
  new(_ inner) {
    self.attach("BufferedReader")
    _inner = inner
    _buf = Bytes.new(8192)
    _pos = 0
    _len = 0
  }

  /// Reads from the wrapped reader into `dst`.
  read(_ dst) {
    if dst is not Bytes {
      throw ArgumentError.new("dst must be a Bytes")
    }
    self.ensureOpen
    if self.isClosed {
      throw UseAfterCloseError.new("cannot read from closed BufferedReader")
    }

    if _pos < _len {
      let mut avail = _len - _pos
      let mut n = dst.size
      if avail < n {
        n = avail
      }
      _buf.slice(_pos, _pos + n).copyInto(dst, 0)
      _pos = _pos + n
      return Future.value(n)
    }

    return _inner.read(_buf).then |count| {
      if count == 0 {
        return Future.value(0)
      }
      _pos = 0
      _len = count
      let mut avail = _len
      let mut n = dst.size
      if avail < n {
        n = avail
      }
      _buf.slice(_pos, _pos + n).copyInto(dst, 0)
      _pos = _pos + n
      Future.value(n)
    }
  }

  close {
    if self.isClosed {
      return Result::Ok(None)
    }
    let mut result = _inner.close
    super.close()
    return result
  }
}
