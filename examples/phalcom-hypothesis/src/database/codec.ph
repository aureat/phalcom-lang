// Versioned binary codec for semantic examples and source-aware failure data.
// The authoritative payload is structured bytes rather than display rendering.

import DatabaseKey from "database/key"
import databaseModel from "database/database"
import Choice from "choices/choice"
import Span from "choices/span"
import Example from "choices/example"
import FailureOrigin from "core/failure"

class ExampleCodec {
  @class
  magic -> String => "PHALCOM-HYPOTHESIS-DB"

  @class
  schemaVersion -> Int => 1

  @class
  engineFormatVersion -> Int => 1

  @class
  encode(
    key: DatabaseKey,
    records: List<databaseModel._DatabaseRecord>
  ) -> Bytes {
    const writer = _CodecWriter.new()
    writer.string(self.magic)
    writer.int(self.schemaVersion)
    writer.string(key.canonical)
    writer.int(records.size)
    for record in records {
      self.writeRecord(writer: writer, record: record)
    }

    const body = writer.freeze
    const result = _CodecWriter.new()
    result.raw(body)
    result.u32(_CodecChecksum.checksum(body))
    return result.freeze
  }

  @class
  decode(
    payload: Bytes,
    expectedKey: DatabaseKey
  ) -> Result<List<databaseModel._DatabaseRecord>, databaseModel._DatabaseDecodeError> {
    const attempted = {
      self.decodeChecked(payload: payload, expectedKey: expectedKey)
    }.attempt()
    if attempted.isErr {
      return Err.new(
        databaseModel._DatabaseDecodeError.because(
          attempted.unwrapErr.message.unwrapOr(attempted.unwrapErr.toString)
        )
      )
    }
    return Ok.new(attempted.unwrap)
  }

  @class
  decodeChecked(
    payload: Bytes,
    expectedKey: DatabaseKey
  ) -> List<databaseModel._DatabaseRecord> {
    if payload.size < 4 {
      throw databaseModel._DatabaseDecodeError.because("payload is truncated")
    }

    const body = _CodecBytes.slice(payload, start: 0, end: payload.size - 4)
    const checksumReader = _CodecReader.new(
      _CodecBytes.slice(payload, start: payload.size - 4, end: payload.size)
    )
    const expectedChecksum = checksumReader.u32
    if _CodecChecksum.checksum(body) != expectedChecksum {
      throw databaseModel._DatabaseDecodeError.because("checksum mismatch")
    }

    const reader = _CodecReader.new(body)
    if reader.string != self.magic {
      throw databaseModel._DatabaseDecodeError.because("wrong database magic")
    }
    if reader.int != self.schemaVersion {
      throw databaseModel._DatabaseDecodeError.because("unsupported schema version")
    }
    if reader.string != expectedKey.canonical {
      throw databaseModel._DatabaseDecodeError.because("database key mismatch")
    }

    const count = reader.count(max: 1024, label: "record count")
    const records = List.new()
    let index = 0
    while index < count {
      records.add(self.readRecord(reader))
      index++
    }
    if not reader.finished {
      throw databaseModel._DatabaseDecodeError.because("trailing bytes")
    }
    return records
  }

  @class
  writeRecord(
    writer: _CodecWriter,
    record: databaseModel._DatabaseRecord
  ) -> None {
    const example = record.example
    writer.string(record.signature)
    writer.int(example.generationSize)
    writer.int(example.choices.size)
    for choice in example.choices {
      self.writeChoice(writer: writer, choice: choice)
    }
    writer.int(example.spans.size)
    for span in example.spans {
      self.writeSpan(writer: writer, span: span)
    }
    self.writeOrigin(writer: writer, origin: record.failureOrigin)
  }

  @class
  readRecord(reader: _CodecReader) -> databaseModel._DatabaseRecord {
    const signature = reader.string
    const generationSize = reader.nonNegativeInt("generationSize")
    const choiceCount = reader.count(max: 100000, label: "choice count")
    const choices = List.new()
    let choiceIndex = 0
    while choiceIndex < choiceCount {
      choices.add(self.readChoice(reader))
      choiceIndex++
    }

    const spanCount = reader.count(max: 100000, label: "span count")
    const spans = List.new()
    let spanIndex = 0
    while spanIndex < spanCount {
      spans.add(self.readSpan(reader: reader, choiceCount: choiceCount))
      spanIndex++
    }
    _CodecValidation.spans(spans: spans, choiceCount: choiceCount)

    const example = Example.from(
      choices: choices,
      spans: spans,
      generationSize: generationSize
    )
    if databaseModel._DatabaseSignatures.example(example) != signature {
      throw databaseModel._DatabaseDecodeError.because(
        "database record signature mismatch"
      )
    }
    return databaseModel._DatabaseRecord.create(
      example: example,
      failureOrigin: self.readOrigin(reader)
    )
  }

  @class
  writeChoice(writer: _CodecWriter, choice: Choice) -> None {
    choice.match(
      integer: |value| {
        writer.byte(_ChoiceTag.integer) // Choice.Integer
        writer.int(value.value)
        writer.int(value.min)
        writer.int(value.max)
        writer.int(value.shrinkTowards)
        writer.optionSymbol(value.label)
      },
      boolean: |value| {
        writer.byte(_ChoiceTag.boolean) // Choice.Boolean
        writer.bool(value.value)
        writer.bool(value.shrinkTowards)
        writer.optionSymbol(value.label)
      },
      index: |value| {
        writer.byte(_ChoiceTag.index) // Choice.Index
        writer.int(value.value)
        writer.int(value.size)
        writer.int(value.shrinkTowards)
        writer.optionSymbol(value.label)
      },
      bytes: |value| {
        writer.byte(_ChoiceTag.bytes) // Choice.Bytes
        writer.bytes(value.value)
        writer.int(value.minSize)
        writer.int(value.maxSize)
        writer.bytes(value.shrinkTowards)
        writer.optionSymbol(value.label)
      }
    )
  }

  @class
  readChoice(reader: _CodecReader) -> Choice {
    const tag = reader.byte
    if tag == _ChoiceTag.integer {
      return Choice.integer(
        value: reader.int,
        min: reader.int,
        max: reader.int,
        shrinkTowards: reader.int,
        label: reader.optionSymbol
      )
    }
    if tag == _ChoiceTag.boolean {
      return Choice.boolean(
        value: reader.bool,
        shrinkTowards: reader.bool,
        label: reader.optionSymbol
      )
    }
    if tag == _ChoiceTag.index {
      return Choice.index(
        value: reader.int,
        size: reader.int,
        shrinkTowards: reader.int,
        label: reader.optionSymbol
      )
    }
    if tag == _ChoiceTag.bytes {
      return Choice.bytes(
        value: reader.bytes(max: 16777216),
        minSize: reader.nonNegativeInt("minimum bytes size"),
        maxSize: reader.nonNegativeInt("maximum bytes size"),
        shrinkTowards: reader.bytes(max: 16777216),
        label: reader.optionSymbol
      )
    }
    throw databaseModel._DatabaseDecodeError.because("unknown choice tag")
  }

  @class
  writeSpan(writer: _CodecWriter, span: Span) -> None {
    writer.int(span.id)
    writer.symbol(span.label)
    writer.int(span.start)
    writer.int(span.end)
    writer.optionInt(span.parent)
    writer.bool(span.discardable)
  }

  @class
  readSpan(reader: _CodecReader, choiceCount: Int) -> Span {
    const span = Span.create(
      id: reader.nonNegativeInt("span id"),
      label: reader.symbol,
      start: reader.nonNegativeInt("span start"),
      end: reader.nonNegativeInt("span end"),
      parent: reader.optionInt,
      discardable: reader.bool
    )
    if span.end > choiceCount {
      throw databaseModel._DatabaseDecodeError.because("span exceeds choices")
    }
    return span
  }

  @class
  writeOrigin(
    writer: _CodecWriter,
    origin: Option<FailureOrigin>
  ) -> None {
    writer.bool(origin.isSome)
    if origin.isNone {
      return
    }
    const value = origin.unwrap
    writer.string(value.errorType.name.toString)
    writer.symbol(value.module)
    writer.symbol(value.selector)
    writer.int(value.line)
    writer.int(value.column)
    writer.optionSymbol(value.label)
  }

  @class
  readOrigin(reader: _CodecReader) -> Option<FailureOrigin> {
    if not reader.bool {
      return None
    }
    const errorTypeName = reader.string
    return Some.new(
      FailureOrigin.new(
        errorType: _DatabaseTypes.resolve(errorTypeName),
        module: reader.symbol,
        selector: reader.symbol,
        line: reader.nonNegativeInt("failure line"),
        column: reader.nonNegativeInt("failure column"),
        label: reader.optionSymbol
      )
    )
  }
}

class _ChoiceTag {
  @class integer -> Int => 1
  @class boolean -> Int => 2
  @class index -> Int => 3
  @class bytes -> Int => 4
}

class _CodecWriter {
  @constructor
  new() {
    _values = List.new()
  }

  byte(value: Int) -> None {
    if value < 0 or value > 255 {
      throw databaseModel._DatabaseDecodeError.because("byte out of range")
    }
    _values.add(value)
  }

  bool(value: Bool) -> None {
    if value { self.byte(1) } else { self.byte(0) }
  }

  u32(value: Int) -> None {
    self.byte((value >> 24) & 255)
    self.byte((value >> 16) & 255)
    self.byte((value >> 8) & 255)
    self.byte(value & 255)
  }

  int(value: Int) -> None => self.string(value.toString)

  symbol(value: Symbol) -> None => self.string(value.toString)

  optionInt(value: Option<Int>) -> None {
    self.bool(value.isSome)
    if value.isSome { self.int(value.unwrap) }
  }

  optionSymbol(value: Option<Symbol>) -> None {
    self.bool(value.isSome)
    if value.isSome { self.symbol(value.unwrap) }
  }

  string(value: String) -> None {
    const points = value.codePoints
    self.u32(points.size)
    for point in points { self.u32(point) }
  }

  bytes(value: Bytes) -> None {
    self.u32(value.size)
    self.raw(value)
  }

  raw(value: Bytes) -> None {
    let index = 0
    while index < value.size {
      self.byte(value[index])
      index++
    }
  }

  freeze -> Bytes {
    const bytes = Bytes.zeroed(_values.size)
    let index = 0
    while index < _values.size {
      bytes[index] = _values.at(index)
      index++
    }
    return bytes
  }
}

class _CodecReader {
  @constructor
  new(bytes: Bytes) {
    _bytes = bytes
    _position = 0
  }

  finished -> Bool => _position == _bytes.size

  byte -> Int {
    self.require(1)
    const value = _bytes[_position]
    _position++
    return value
  }

  bool -> Bool {
    const value = self.byte
    if value == 0 { return false }
    if value == 1 { return true }
    throw databaseModel._DatabaseDecodeError.because("invalid boolean")
  }

  u32 -> Int {
    self.require(4)
    const value = (_bytes[_position] << 24) |
      (_bytes[_position + 1] << 16) |
      (_bytes[_position + 2] << 8) |
      _bytes[_position + 3]
    _position += 4
    return value
  }

  int -> Int => self.string.toInt

  nonNegativeInt(label: String) -> Int {
    const value = self.int
    if value < 0 {
      throw databaseModel._DatabaseDecodeError.because(label + " is negative")
    }
    return value
  }

  count(max: Int, label: String) -> Int {
    const value = self.nonNegativeInt(label)
    if value > max {
      throw databaseModel._DatabaseDecodeError.because(label + " exceeds limit")
    }
    return value
  }

  symbol -> Symbol => Symbol.intern(self.string)

  optionInt -> Option<Int> {
    if self.bool { return Some.new(self.int) }
    return None
  }

  optionSymbol -> Option<Symbol> {
    if self.bool { return Some.new(self.symbol) }
    return None
  }

  string -> String {
    const count = self.u32
    if count > 1048576 {
      throw databaseModel._DatabaseDecodeError.because("string exceeds limit")
    }
    const points = List.new()
    let index = 0
    while index < count {
      points.add(self.u32)
      index++
    }
    return String.fromCodePoints(points)
  }

  bytes(max: Int) -> Bytes {
    const count = self.u32
    if count > max {
      throw databaseModel._DatabaseDecodeError.because("byte field exceeds limit")
    }
    self.require(count)
    const value = _CodecBytes.slice(
      _bytes,
      start: _position,
      end: _position + count
    )
    _position += count
    return value
  }

  require(count: Int) -> None {
    if _position + count > _bytes.size {
      throw databaseModel._DatabaseDecodeError.because("payload is truncated")
    }
  }
}

class _CodecValidation {
  @class
  spans(spans: List<Span>, choiceCount: Int) -> None {
    const ids = Set.new()
    for span in spans {
      if ids.includes(span.id) {
        throw databaseModel._DatabaseDecodeError.because("duplicate span id")
      }
      ids.add(span.id)
      if span.start > span.end or span.end > choiceCount {
        throw databaseModel._DatabaseDecodeError.because("invalid span range")
      }
    }
    for span in spans {
      if span.parent.isSome {
        const parent = self.spanWithId(spans: spans, id: span.parent.unwrap)
        if parent.isNone {
          throw databaseModel._DatabaseDecodeError.because("unknown span parent")
        }
        if parent.unwrap.id == span.id {
          throw databaseModel._DatabaseDecodeError.because("span is its own parent")
        }
        if parent.unwrap.start > span.start or parent.unwrap.end < span.end {
          throw databaseModel._DatabaseDecodeError.because("parent does not contain child span")
        }
      }
      self.noParentCycle(spans: spans, span: span)
    }
  }

  @class
  spanWithId(spans: List<Span>, id: Int) -> Option<Span> {
    for span in spans {
      if span.id == id { return Some.new(span) }
    }
    return None
  }

  @class
  noParentCycle(spans: List<Span>, span: Span) -> None {
    const visited = Set.new()
    let current = Some.new(span)
    while current.isSome {
      const value = current.unwrap
      if visited.includes(value.id) {
        throw databaseModel._DatabaseDecodeError.because("span parent cycle")
      }
      visited.add(value.id)
      if value.parent.isNone { return }
      current = self.spanWithId(spans: spans, id: value.parent.unwrap)
    }
  }
}

class _CodecChecksum {
  @class
  checksum(value: Bytes) -> Int {
    let hash = 2166136261
    let index = 0
    while index < value.size {
      hash = ((hash ^ value[index]) * 16777619) % 4294967296
      index++
    }
    if hash < 0 { return hash + 4294967296 }
    return hash
  }
}

class _CodecBytes {
  @class
  slice(value: Bytes, start: Int, end: Int) -> Bytes {
    const copied = Bytes.zeroed(end - start)
    let source = start
    let target = 0
    while source < end {
      copied[target] = value[source]
      source++
      target++
    }
    return copied
  }
}

class _DatabaseTypes {
  @class
  resolve(name: String) -> Class {
    const found = System.classNamed(Symbol.intern(name))
    if found.isSome { return found.unwrap }
    return Error
  }
}
