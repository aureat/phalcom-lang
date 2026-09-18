from universe.collections.iterator_laws import Iterator as CanonicalIterator, Iterable as CanonicalIterable

native class String is Object {
  native +(_ other: String) -> String
  native hash -> Int
  @class
  native new() -> String
  @class
  native new(_ value: Dynamic) -> String
  internal native rawByteCount -> Int
  internal native rawByteAt(_ index: Int) -> Option<Int>
  internal native rawSlice(_ start: Int, _ end: Int) -> String
  // Display (U-CORE-4, R-INV-4.1): a string's display *is* itself — no
  // representation read, so this is `.ph`-derivable rather than a floor
  // primitive (ADR-0019's derivability test).
  toString { self }

  // Byte count. UTF-8 buffer length in bytes (not codepoints).
  size { self.rawByteCount }
  isEmpty { self.rawByteCount == 0 }

  // Byte-range slice. Native storage operation stays internal; public wrapper
  // preserves existing bounds/UTF-8-boundary diagnostics.
  slice(_ start, _ end) { self.rawSlice(start, end) }

  // Number of leading bytes in the UTF-8 sequence starting at byte offset `i`.
  // Read purely from the lead byte's numeric range: 1/2/3/4-byte sequences are
  // encoded by the lead byte's numeric value (no bitmask needed).
  leadByteLen(_ i) {
    const b = self.rawByteAt(i)
    return (b == None).ifTrue(|| { None }, ifFalse: || {
      (b < 128).ifTrue(|| { 1 }, ifFalse: || {
        (b < 224).ifTrue(|| { 2 }, ifFalse: || {
          (b < 240).ifTrue(|| { 3 }, ifFalse: || { 4 }) }) })
    })
  }

  // The Unicode scalar value at byte offset `i`, or `None` if out-of-range
  // or mid-sequence. UTF-8 decode via division/modulo (no bitwise ops).
  codePointAt(_ i) {
    const b0 = self.rawByteAt(i)
    return (b0 == None).ifTrue(|| { None }, ifFalse: || {
      (b0 < 128).ifTrue(|| {
        // ASCII single byte (0xxxxxxx)
        b0
      }, ifFalse: || {
        (b0 < 192).ifTrue(|| {
          // Continuation byte (10xxxxxx), not a start byte
          None
        }, ifFalse: || {
          (b0 < 224).ifTrue(|| {
            // 2-byte sequence (110xxxxx 10xxxxxx)
            const b1 = self.rawByteAt(i + 1)
            (b1 == None).ifTrue(|| { None }, ifFalse: || {
              (b1 < 128).ifTrue(|| { None }, ifFalse: || {
                (b1 >= 192).ifTrue(|| { None }, ifFalse: || {
                  ((b0 - 192) * 64) + (b1 - 128)
                })
              })
            })
          }, ifFalse: || {
            (b0 < 240).ifTrue(|| {
              // 3-byte sequence (1110xxxx 10xxxxxx 10xxxxxx)
              const b1 = self.rawByteAt(i + 1)
              const b2 = self.rawByteAt(i + 2)
              (b1 == None).ifTrue(|| { None }, ifFalse: || {
                (b2 == None).ifTrue(|| { None }, ifFalse: || {
                  (b1 < 128).ifTrue(|| { None }, ifFalse: || {
                    (b1 >= 192).ifTrue(|| { None }, ifFalse: || {
                      (b2 < 128).ifTrue(|| { None }, ifFalse: || {
                        (b2 >= 192).ifTrue(|| { None }, ifFalse: || {
                          ((b0 - 224) * 4096) + ((b1 - 128) * 64) + (b2 - 128)
                        })
                      })
                    })
                  })
                })
              })
            }, ifFalse: || {
              (b0 < 248).ifTrue(|| {
                // 4-byte sequence (11110xxx 10xxxxxx 10xxxxxx 10xxxxxx)
                const b1 = self.rawByteAt(i + 1)
                const b2 = self.rawByteAt(i + 2)
                const b3 = self.rawByteAt(i + 3)
                (b1 == None).ifTrue(|| { None }, ifFalse: || {
                  (b2 == None).ifTrue(|| { None }, ifFalse: || {
                    (b3 == None).ifTrue(|| { None }, ifFalse: || {
                      (b1 < 128).ifTrue(|| { None }, ifFalse: || {
                        (b1 >= 192).ifTrue(|| { None }, ifFalse: || {
                          (b2 < 128).ifTrue(|| { None }, ifFalse: || {
                            (b2 >= 192).ifTrue(|| { None }, ifFalse: || {
                              (b3 < 128).ifTrue(|| { None }, ifFalse: || {
                                (b3 >= 192).ifTrue(|| { None }, ifFalse: || {
                                  ((b0 - 240) * 262144) + ((b1 - 128) * 4096) + ((b2 - 128) * 64) + (b3 - 128)
                                })
                              })
                            })
                          })
                        })
                      })
                    })
                  })
                })
              }, ifFalse: || {
                // Invalid UTF-8 start byte
                None
              })
            })
          })
        })
      })
    })
  }

  // Find first occurrence of a substring, scanning left-to-right by byte.
  // O(n·m) naive search. Returns the byte offset, or -1 if not found.
  indexOf(_ needle) {
    (needle.is(String)).ifTrue(|| {}, ifFalse: || {
      throw ArgumentError.new("indexOf: needle must be a String")
    })
    (needle.isEmpty).ifTrue(|| {
      throw ArgumentError.new("indexOf: needle must be non-empty")
    })

    let i = 0
    while (i <= self.rawByteCount - needle.rawByteCount) {
      let is_match = true
      let j = 0
      while (j < needle.rawByteCount) {
        (self.rawByteAt(i + j) == needle.rawByteAt(j)).ifTrue(|| {}, ifFalse: || {
          is_match = false
        })
        (is_match).ifTrue(|| { j = j + 1 }, ifFalse: || { j = needle.rawByteCount })
      }
      (is_match).ifTrue(|| { return i })
      i = i + 1
    }
    return -1
  }

  contains(_ needle) {
    (needle.is(String)).ifFalse || {
      throw ArgumentError.new("contains: needle must be a String")
    }
    if (needle.isEmpty) { return true }
    return self.indexOf(needle) != -1
  }

  includes(_ needle) { self.contains(needle) }

  // Split by delimiter substring. Returns a List of String segments.
  split(_ delimiter) {
    (delimiter.is(String)).ifTrue(|| {}, ifFalse: || {
      throw ArgumentError.new("split: delimiter must be a String")
    })
    (delimiter.isEmpty).ifTrue(|| {
      throw ArgumentError.new("split: delimiter must be non-empty")
    })

    let result = List<Dynamic>.new()
    let prev = 0
    let i = self.indexOf(delimiter)
    while (i != -1) {
      result.rawPush(self.rawSlice(prev, i))
      prev = i + delimiter.rawByteCount
      // Search for next occurrence after this delimiter
      let rest = self.rawSlice(prev, self.rawByteCount)
      let nextIdx = rest.indexOf(delimiter)
      (nextIdx == -1).ifTrue(|| { i = -1 }, ifFalse: || { i = prev + nextIdx })
    }
    result.rawPush(self.rawSlice(prev, self.rawByteCount))
    return result
  }

  // Replace all occurrences of `from` with `to`.
  replace(_ needle, _ to) {
    (needle.is(String)).ifTrue(|| {}, ifFalse: || {
      throw ArgumentError.new("replace: from must be a String")
    })
    (to.is(String)).ifTrue(|| {}, ifFalse: || {
      throw ArgumentError.new("replace: to must be a String")
    })
    (needle.isEmpty).ifTrue(|| {
      throw ArgumentError.new("replace: from must be non-empty")
    })

    let result = ""
    let prev = 0
    let i = self.indexOf(needle)
    while (i != -1) {
      result = result + self.rawSlice(prev, i) + to
      prev = i + needle.rawByteCount
      let rest = self.rawSlice(prev, self.rawByteCount)
      let nextIdx = rest.indexOf(needle)
      (nextIdx == -1).ifTrue(|| { i = -1 }, ifFalse: || { i = prev + nextIdx })
    }
    result = result + self.rawSlice(prev, self.rawByteCount)
    return result
  }

  // Trim whitespace from start and end, default or custom charset.
  trim() {
    return self.trim(" \t\n\r")
  }
  trimStart() {
    return self.trimStart(" \t\n\r")
  }
  trimEnd() {
    return self.trimEnd(" \t\n\r")
  }

  trim(_ chars) { self.trimStart(chars).trimEnd(chars) }

  // Trim from the start using the given charset.
  trimStart(_ chars) {
    (chars.is(String)).ifTrue(|| {}, ifFalse: || {
      throw ArgumentError.new("trimStart: chars must be a String")
    })

    let i = 0
    let stop = false
    while ((i < self.rawByteCount).and(|| { not stop })) {
      const cp = self.codePointAt(i)
      let found = false
      let j = 0
      while (j < chars.rawByteCount) {
        (chars.codePointAt(j) == cp).ifTrue(|| { found = true })
        const len = chars.leadByteLen(j)
        (len == None).ifTrue(|| { j = j + 1 }, ifFalse: || { j = j + len })
      }
      (found).ifTrue(|| {
        i = i + self.leadByteLen(i)
      }, ifFalse: || {
        stop = true  // exit loop, keeping i at the first non-trimmed byte
      })
    }
    return self.rawSlice(i, self.rawByteCount)
  }

  // Trim from the end using the given charset.
  trimEnd(_ chars) {
    (chars.is(String)).ifTrue(|| {}, ifFalse: || {
      throw ArgumentError.new("trimEnd: chars must be a String")
    })

    let i = self.rawByteCount
    let stop = false
    while ((i > 0).and(|| { not stop })) {
      // Scan backward one byte at a time to find the previous lead byte
      i = i - 1
      let cp = self.codePointAt(i)
      (cp == None).ifTrue(|| {
        // Not a lead byte, keep scanning back
      }, ifFalse: || {
        // Found a lead byte; check if it's in the trim set
        let found = false
        let j = 0
        while (j < chars.rawByteCount) {
          (chars.codePointAt(j) == cp).ifTrue(|| { found = true })
          const len = chars.leadByteLen(j)
          (len == None).ifTrue(|| { j = j + 1 }, ifFalse: || { j = j + len })
        }
        (found).ifTrue(|| {}, ifFalse: || {
          // Not in the set; keep this whole character and stop scanning
          i = i + self.leadByteLen(i)
          stop = true
        })
      })
    }
    return self.rawSlice(0, i)
  }

  // Repeat the string `count` times.
  *(_ count) {
    (count.is(Number)).ifTrue(|| {}, ifFalse: || {
      throw ArgumentError.new("*: count must be a Number")
    })
    (count >= 0).ifTrue(|| {}, ifFalse: || {
      throw ArgumentError.new("*: count must be >= 0")
    })
    (count % 1 == 0).ifTrue(|| {}, ifFalse: || {
      throw ArgumentError.new("*: count must be an integer")
    })

    (count == 0).ifTrue(|| { return "" })
    (count == 1).ifTrue(|| { return self })

    let result = ""
    let i = 0
    while (i < count) {
      result = result + self
      i = i + 1
    }
    return result
  }

  // Byte sequence accessor (U-STRING §2.4).
  bytes { StringByteSequence.new(self) }

  // Codepoint sequence accessor (U-STRING §2.4).
  codePoints { StringCodePointSequence.new(self) }
}

// Byte-level sequence view (U-STRING §2.4, ADR-0048 shaped).
class StringByteSequence {
  @constructor
  new(_ s) { _string = s }

  iterator -> StringByteIterator { StringByteIterator.new(self) }

  size { _string.rawByteCount }

  at(_ i) { _string.rawByteAt(i) }

  internal rawByteAt(_ i) { _string.rawByteAt(i) }

  each(_ f) {
    let i = 0
    while (i < self.size) {
      f.call(self.at(i))
      i = i + 1
    }
  }

  // Iterate over byte offsets: cursor steps to next lead byte.
  private nextCursor(_ cursor) {
    const next = (cursor == None).ifTrue(|| { 0 }, ifFalse: || {
      cursor + 1
    })
    return (next < _string.rawByteCount).ifTrue(|| { next }, ifFalse: || { None })
  }
}

class StringByteIterator {
  _source: StringByteSequence
  _offset: Int
  _done: Bool

  @constructor
  new(_ source: StringByteSequence) {
    _source = source
    _offset = 0
    _done = false
  }

  next -> Option<Int> {
    if (_done) { return None }
    if (_offset >= _source.size) {
      _done = true
      return None
    }
    let value = _source.rawByteAt(_offset)
    _offset = _offset + 1
    Some(value)
  }

  iterator { self }
}

impl CanonicalIterator for StringByteIterator {
  type Item = Int
}

impl CanonicalIterable for StringByteSequence {
  type Item = Int
  type Iterator = StringByteIterator
  iterator -> StringByteIterator { StringByteIterator.new(self) }
}

// Codepoint-level sequence view (U-STRING §2.4, ADR-0048 shaped).
class StringCodePointSequence {
  @constructor
  new(_ s) { _string = s }

  iterator -> StringCodePointIterator { StringCodePointIterator.new(self) }

  // Codepoint count: full scan (no native "codepoint length").
  size {
    let n = 0
    let i = self.nextCursor(None)
    while (i != None) {
      n = n + 1
      i = self.nextCursor(i)
    }
    return n
  }

  at(_ byteOffset) { _string.codePointAt(byteOffset) }

  internal byteCountView { _string.rawByteCount }

  internal rawCodePointAt(_ byteOffset) { _string.codePointAt(byteOffset) }

  internal rawLeadByteLen(_ byteOffset) { _string.leadByteLen(byteOffset) }

  each(_ f) {
    let i = self.nextCursor(None)
    while (i != None) {
      f.call(self.at(i))
      i = self.nextCursor(i)
    }
  }

  // Iterate over byte offsets: cursor steps by UTF-8 char boundary.
  private nextCursor(_ cursor) {
    const next = (cursor == None).ifTrue(|| { 0 }, ifFalse: || {
      cursor + _string.leadByteLen(cursor)
    })
    return (next < _string.rawByteCount).ifTrue(|| { next }, ifFalse: || { None })
  }
}

class StringCodePointIterator {
  _source: StringCodePointSequence
  _offset: Int
  _done: Bool

  @constructor
  new(_ source: StringCodePointSequence) {
    _source = source
    _offset = 0
    _done = false
  }

  next -> Option<Int> {
    if (_done) { return None }
    if (_offset >= _source.byteCountView) {
      _done = true
      return None
    }
    let value = _source.rawCodePointAt(_offset)
    if (value == None) {
      throw Error.new("String code-point iterator encountered an invalid sequence")
    }
    let length = _source.rawLeadByteLen(_offset)
    if (length == None) {
      throw Error.new("String code-point iterator encountered an invalid lead byte")
    }
    _offset = _offset + length
    Some(value)
  }

  iterator { self }
}

impl CanonicalIterator for StringCodePointIterator {
  type Item = Int
}

impl CanonicalIterable for StringCodePointSequence {
  type Item = Int
  type Iterator = StringCodePointIterator
  iterator -> StringCodePointIterator { StringCodePointIterator.new(self) }
}
