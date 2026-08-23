// U-BYTES (PDR-0011, docs/spec/v0.2/core/bytes.md): the kernel octet buffer's
// `.ph` protocol over the eleven floor primitives. Bulk no-user-code work is
// native (bytes.md §3.1); everything here is validation-lifting, derivation,
// and the Iterable hookup. `each`/`map`/`filter`/`reduce` are deliberately
// ABSENT — inherited from `Iterable` so `Fiber.yield` works mid-iteration
// (law 8); adding native or local overrides is a spec violation.
@native
class Bytes is Iterable {
  size { self._$size }

  first {
    if (self.size == 0) { return None }
    return Some(self.at(0))
  }

  last {
    if (self.size == 0) { return None }
    return Some(self.at(self.size - 1))
  }

  at(_ i) { self._$at(i) }

  get(_ index) {
    let raw = self._$at(index)
    let len = self.size
    let i = index
    if (i < 0) { i = len + i }
    if (i >= 0 and i < len) {
      return Some(raw)
    }
    return None
  }

  [_ index] {
    if (index.is(Range)) {
      return index._$sliceBounds(self.size).match(
        ok: |bounds| {
          let start = bounds[0]
          let end = bounds[1]
          if (start > end) { end = start }
          self._$slice(start, end)
        },
        err: |error| { error.raise() }
      )
    }
    let raw = self._$at(index)
    let len = self.size
    let i = index
    if (i < 0) { i = len + i }
    if (i >= 0 and i < len) {
      return raw
    }
    throw IndexError.new("Bytes index out of range")
  }

  [_ index, default] {
    let raw = self._$at(index)
    let len = self.size
    let i = index
    if (i < 0) { i = len + i }
    if (i >= 0 and i < len) {
      return raw
    }
    return default
  }

  get(_ index, orElse) {
    let raw = self._$at(index)
    let len = self.size
    let i = index
    if (i < 0) { i = len + i }
    if (i >= 0 and i < len) {
      return raw
    }
    return orElse.call(index)
  }

  iteratorValue(_ cursor) { self._$at(cursor) }

  // An octet is an integer Number in 0..255 (bytes.md §2). `and` is lazy,
  // so the arithmetic tests never run on a non-Number. (No trailing `_`:
  // that marker is reserved for native primitives, and this is pure .ph.)
  isOctet(_ v) {
    return v.is(Number) and (v >= 0) and (v <= 255) and ((v % 1) == 0)
  }

  // Raise-lifting writes (bytes.md law 1: precondition violations raise,
  // reads stay total). The floor's set_ reports a bad write as a native
  // type error; the .ph surface names the contract instead.
  set(_ i, _ v) {
    if (not self.isOctet(v)) {
      throw ArgumentError.new("Bytes#set: value must be an integer in 0..255")
    }
    let len = self.size
    let norm = i
    if (norm < 0) { norm = len + norm }
    if ((norm < 0) or (norm >= len)) {
      throw IndexError.new("Bytes index out of range")
    }
    self._$set(i, v)
    return self
  }

  at(_ i, put) { return self.set(i, put) }

  [_ i]=(put val) { return self.set(i, val) }

  fill(_ v) {
    if (not self.isOctet(v)) {
      throw ArgumentError.new("Bytes#fill: value must be an integer in 0..255")
    }
    self._$fill(v)
    return self
  }

  // One native memset, complete because the length is fixed (bytes.md §7).
  // The guarantee is a documented obligation, not a mechanism — scope it
  // with `ensure`. A getter so the call reads `key.zeroize` (ADR-0012:
  // `zeroize` and `zeroize()` are different selectors).
  zeroize {
    self._$fill(0)
    return self
  }

  utf8 { self._$utf8 }

  // Display decode: total, lossy (invalid sequences become U+FFFD). Never
  // round-trip the result into data (PDR-0013 ruling 4).
  utf8Lossy { self._$utf8Lossy }

  slice(_ start, _ end) {
    if ((start < 0) or (end < start) or (end > self.size)) {
      throw ArgumentError.new("Bytes#slice: range must satisfy 0 <= start <= end <= size")
    }
    return self._$slice(start, end)
  }

  copyInto(_ dst, _ offset) {
    if (not dst.is(Bytes)) {
      throw ArgumentError.new("Bytes#copyInto: destination must be a Bytes")
    }
    if ((offset < 0) or ((offset + self.size) > dst.size)) {
      throw ArgumentError.new("Bytes#copyInto: offset + size must fit the destination")
    }
    self._$copyInto(dst, offset)
    return self
  }

  // Derivability with teeth (bytes.md §3.1): new + two native memmoves,
  // zero per-byte loops.
  concat(_ other) {
    if (not other.is(Bytes)) {
      throw ArgumentError.new("Bytes#concat: argument must be a Bytes")
    }
    const out = Bytes.new(self.size + other.size)
    self._$copyInto(out, 0)
    other._$copyInto(out, self.size)
    return out
  }

  equalsConstantTime(_ other) { self._$equalsConstantTime(other) }

  // Structural equality, List#=='s exact shape (collection-protocol §4).
  // Short-circuits — correct here, and exactly why it must never be the
  // secret-comparison spelling (bytes.md §8).
  ==(_ other) {
    if (other.is(Bytes)) {
      let same = (self.size == other.size)
      let i = 0
      // `and` is lazy: once `same` is false the loop exits without another
      // `at(i)` (List#=='s exact shape).
      while (same and (i < self.size)) {
        same = (self.at(i) == other.at(i))
        i = i + 1
      }
      return same
    } else {
      return false
    }
  }

  // MUST route through == (the ==/!= decoupling hazard) — Object#!= negates
  // identity, not this structural ==.
  !=(_ other) {
    return not (self == other)
  }

  toString { "Bytes(" + self.size.toString + ")" }

  toList {
    const out = []
    for b in self {
      out.append(b)
    }
    return out
  }

  // The immutable, value-hashable snapshot — the Map-key escape hatch
  // (PDR-0011 ruling 4; Bytes itself is mutable => identity hash, never a
  // valid Map/Set key).
  toTuple { Tuple._$fromList(self.toList) }

  @class
  fromString(_ s) {
    if (not s.is(String)) {
      throw ArgumentError.new("Bytes.fromString: argument must be a String")
    }
    return Bytes._$fromString(s)
  }

  // The builder story (bytes.md law 3 forecloses growth): build in a List,
  // freeze into Bytes — Tuple.fromList's shape. `set` (not `set_`) so a
  // non-octet element raises the named contract error.
  @class
  fromList(_ list) {
    if (not list.is(List)) {
      throw ArgumentError.new("Bytes.fromList: argument must be a List")
    }
    const out = Bytes.new(list.size)
    let i = 0
    while (i < list.size) {
      out.set(i, list.at(i))
      i = i + 1
    }
    return out
  }
}

class OpenMode {
  @constructor
  @private
  named(_ n) { _name = n }
  @class
  read { OpenMode.named("read") }
  @class
  write { OpenMode.named("write") }
  @class
  append { OpenMode.named("append") }
  @class
  readWrite { OpenMode.named("readWrite") }
  name { _name }
  ==(_ other) { return other.is(OpenMode) and (_name == other.name) }
  !=(_ other) { return not (self == other) }
  toString { "OpenMode." + _name }
}

class Path {
  @constructor
  of(_ s) {
    if (not s.is(String)) {
      throw ArgumentError.new("Path.of: argument must be a String")
    }
    _bytes = Bytes.fromString(s)
    _hash = Path.contentHash(_bytes)
  }

  @constructor
  ofBytes(_ b) {
    if (not b.is(Bytes)) {
      throw ArgumentError.new("Path.ofBytes: argument must be a Bytes")
    }
    _bytes = b.slice(0, b.size)
    _hash = Path.contentHash(_bytes)
  }

  @class
  contentHash(_ bytes) {
    let acc = 1
    let i = 0
    while (i < bytes.size) {
      acc = (acc * 31 + bytes.at(i)) % 999999937
      i = i + 1
    }
    return acc
  }

  bytes { _bytes.slice(0, _bytes.size) }
  hash { _hash }

  ==(_ other) {
    if (not other.is(Path)) { return false }
    if (_hash != other.hash) { return false }
    return _bytes == other.bytes
  }

  !=(_ other) { return not (self == other) }

  isAbsolute { (_bytes.size > 0) and (_bytes.at(0) == 47) }

  join(_ other) {
    if (not other.is(Path)) {
      throw ArgumentError.new("Path#join: argument must be a Path")
    }
    if (other.isAbsolute) {
      return other
    }
    let recv = _bytes
    let recvLen = recv.size
    while ((recvLen > 0) and (recv.at(recvLen - 1) == 47)) {
      recvLen = recvLen - 1
    }
    const trimmedRecv = recv.slice(0, recvLen)
    const sep = Bytes.fromString("/")
    const combined = trimmedRecv.concat(sep).concat(other.bytes)
    return Path.ofBytes(combined)
  }

  parent {
    let len = _bytes.size
    while ((len > 0) and (_bytes.at(len - 1) == 47)) {
      len = len - 1
    }
    let idx = len - 1
    while ((idx >= 0) and (_bytes.at(idx) != 47)) {
      idx = idx - 1
    }
    if (idx < 0) {
      return None
    }
    if (idx == 0) {
      return Path.of("/")
    }
    let pLen = idx
    while ((pLen > 0) and (_bytes.at(pLen - 1) == 47)) {
      pLen = pLen - 1
    }
    if (pLen == 0) {
      return Path.of("/")
    }
    return Path.ofBytes(_bytes.slice(0, pLen))
  }

  fileName {
    let len = _bytes.size
    if ((len > 0) and (_bytes.at(len - 1) == 47)) {
      return None
    }
    let idx = len - 1
    while ((idx >= 0) and (_bytes.at(idx) != 47)) {
      idx = idx - 1
    }
    if ((idx < 0) and (len == 0)) {
      return None
    }
    const nameBytes = _bytes.slice(idx + 1, len)
    if (nameBytes.size == 0) {
      return None
    }
    return Path.ofBytes(nameBytes)
  }

  extension {
    const namePath = self.fileName
    if (namePath == None) {
      return None
    }
    const nb = namePath.bytes
    let idx = nb.size - 1
    while ((idx >= 0) and (nb.at(idx) != 46)) {
      idx = idx - 1
    }
    if ((idx <= 0) or (idx == nb.size - 1)) {
      return None
    }
    const extBytes = nb.slice(idx + 1, nb.size)
    return extBytes.utf8
  }

  components {
    let res = List.new()
    let i = 0
    let len = _bytes.size
    while (i < len) {
      while ((i < len) and (_bytes.at(i) == 47)) {
        i = i + 1
      }
      if (i < len) {
        let start = i
        while ((i < len) and (_bytes.at(i) != 47)) {
          i = i + 1
        }
        res.append(Path.ofBytes(_bytes.slice(start, i)))
      }
    }
    return res
  }

  toString { _bytes.utf8Lossy }
}


// Explicit lazy iterator stages. Each is an ordinary `.ph` wrapper over the

// ============================================================================
// U-RESOURCE & U-STREAMS
// ============================================================================

@native
class Resource is Object {
  close {
    self._$close()
    return Ok.new(None)
  }
  isClosed { self._$isClosed }
}

@native
class UseAfterCloseError is Error {}

class UnflushedError is Error {}

class BytesReader is Resource {
  @constructor
  new(_ source) {
    source.is(Bytes).ifFalse || {
      throw ArgumentError.new("BytesReader source must be a Bytes")
    }
    _handle = Resource._$register("BytesReader")
    // snapshot: source is a Bytes, copied — the reader's contents never change under it
    _data = source.slice(0, source.size)
    _pos = 0
  }

  read(_ dst) {
    dst.is(Bytes).ifFalse || {
      throw ArgumentError.new("dst must be a Bytes")
    }
    self.isClosed.ifTrue || {
      throw UseAfterCloseError.new("cannot read from closed BytesReader")
    }
    let remaining = _data.size - _pos
    let n = dst.size
    (remaining < n).ifTrue || { n = remaining }
    (n > 0).ifTrue || {
      _data.slice(_pos, _pos + n).copyInto(dst, 0)
      _pos = _pos + n
    }
    // In-memory operation cannot block, honest return type per spec section 2
    return Future.value(n)
  }
}

class BytesWriter is Resource {
  @constructor
  new() {
    _handle = Resource._$register("BytesWriter")
    _chunks = List.new()
  }

  write(_ src) {
    src.is(Bytes).ifFalse || {
      throw ArgumentError.new("src must be a Bytes")
    }
    self.isClosed.ifTrue || {
      throw UseAfterCloseError.new("cannot write to closed BytesWriter")
    }
    _chunks._$push(src.slice(0, src.size))
    return Future.value(src.size)
  }

  flush {
    return Future.value(None)
  }

  toBytes {
    let total = 0
    _chunks.each |c| { total = total + c.size }
    let res = Bytes.new(total)
    let offset = 0
    _chunks.each |c| {
      c.copyInto(res, offset)
      offset = offset + c.size
    }
    return res
  }
}

class BufferedWriter is Resource {
  @constructor
  new(_ inner) {
    _handle = Resource._$register("BufferedWriter")
    _inner = inner
    _buf = Bytes.new(8192)
    _len = 0
  }

  pending { _len }

  write(_ src) {
    src.is(Bytes).ifFalse || {
      throw ArgumentError.new("src must be a Bytes")
    }
    self.isClosed.ifTrue || {
      throw UseAfterCloseError.new("cannot write to closed BufferedWriter")
    }

    if (src.size >= _buf.size) {
      return self.flush.then |_| {
        _inner.write(src)
      }
    }

    if ((_len + src.size) > _buf.size) {
      return self.flush.then |_| {
        src.copyInto(_buf, _len)
        _len = _len + src.size
        Future.value(src.size)
      }
    } else {
      src.copyInto(_buf, _len)
      _len = _len + src.size
      return Future.value(src.size)
    }
  }

  flush {
    if (_len == 0) {
      return Future.value(None)
    }
    let chunk = _buf.slice(0, _len)
    return _inner.write(chunk).then |bytesWritten| {
      _len = 0
      Future.value(None)
    }
  }

  close {
    if (_len > 0) {
      throw UnflushedError.new("BufferedWriter closed with " + _len.toString + " pending bytes")
    }
    super.close
    return _inner.close
  }

  finish {
    return self.flush.then |_| {
      self.close
    }
  }
}

class BufferedReader is Resource {
  @constructor
  new(_ inner) {
    _handle = Resource._$register("BufferedReader")
    _inner = inner
    _buf = Bytes.new(8192)
    _pos = 0
    _len = 0
  }

  read(_ dst) {
    dst.is(Bytes).ifFalse || {
      throw ArgumentError.new("dst must be a Bytes")
    }
    self.isClosed.ifTrue || {
      throw UseAfterCloseError.new("cannot read from closed BufferedReader")
    }

    if (_pos < _len) {
      let avail = _len - _pos
      let n = dst.size
      (avail < n).ifTrue || { n = avail }
      _buf.slice(_pos, _pos + n).copyInto(dst, 0)
      _pos = _pos + n
      return Future.value(n)
    }

    return _inner.read(_buf).then |count| {
      if (count == 0) {
        return Future.value(0)
      }
      _pos = 0
      _len = count
      let avail = _len
      let n = dst.size
      (avail < n).ifTrue || { n = avail }
      _buf.slice(_pos, _pos + n).copyInto(dst, 0)
      _pos = _pos + n
      Future.value(n)
    }
  }
}
