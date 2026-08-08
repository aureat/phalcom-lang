// Typed example database extension boundary and immutable stored record model.

import DatabaseKey from "database/key"
import Example from "choices/example"
import FailureOrigin from "core/failure"

protocol ExampleDatabase {
  fetch(key: DatabaseKey) -> List<Example>

  save(
    key: DatabaseKey,
    example: Example,
    failureOrigin: Option<FailureOrigin>
  ) -> ExampleDatabase

  delete(key: DatabaseKey, example: Example) -> ExampleDatabase
}

@data
@immutable
class _DatabaseRecord {
  const _example: Example
  const _failureOrigin: Option<FailureOrigin>
  const _signature: String

  @class
  create(
    example: Example,
    failureOrigin: Option<FailureOrigin>
  ) -> _DatabaseRecord {
    return _DatabaseRecord.new(
      example: example,
      failureOrigin: failureOrigin,
      signature: _DatabaseSignatures.example(example)
    )
  }
}

@data
@immutable
class _DatabaseDecodeError is Error {
  const _reason: String

  @class
  because(reason: String) -> _DatabaseDecodeError {
    return _DatabaseDecodeError.new(reason: reason)
  }

  message -> Option<String> => Some.new(_reason)
}


class _DatabaseSignatures {
  @class
  example(value: Example) -> String {
    const parts = List.new()
    for choice in value.choices || {
      parts.add(self.choice(choice))
    }
    const spanParts = List.new()
    for span in value.spans || {
      spanParts.add(
        span.id.toString + "," + span.label.toString + "," +
        span.start.toString + "," + span.end.toString + "," +
        self.optionInt(span.parent) + "," + span.discardable.toString
      )
    }
    let choiceText = parts.join(";")
    if parts.size > 0 {
      choiceText += ";"
    }
    let spanText = spanParts.join(";")
    if spanParts.size > 0 {
      spanText += ";"
    }
    return "g=" + value.generationSize.toString + "|c=" +
      choiceText + "|s=" + spanText
  }

  @class
  choice(value: Any) -> String {
    return value.match(
      integer: |item| {
        "i(" + item.value.toString + "," + item.min.toString + "," +
          item.max.toString + "," + item.shrinkTowards.toString + "," +
          self.optionSymbol(item.label) + ")"
      },
      boolean: |item| {
        "b(" + item.value.toString + "," + item.shrinkTowards.toString +
          "," + self.optionSymbol(item.label) + ")"
      },
      index: |item| {
        "x(" + item.value.toString + "," + item.size.toString + "," +
          item.shrinkTowards.toString + "," +
          self.optionSymbol(item.label) + ")"
      },
      bytes: |item| {
        "y(" + self.bytes(item.value) + "," + item.minSize.toString +
          "," + item.maxSize.toString + "," +
          self.bytes(item.shrinkTowards) + "," +
          self.optionSymbol(item.label) + ")"
      }
    )
  }

  @class
  bytes(value: Bytes) -> String {
    const parts = List.new()
    let index = 0
    while index < value.size || {
      parts.add(value[index].toString)
      index++
    }
    return parts.join(".")
  }

  @class
  optionInt(value: Option<Int>) -> String {
    if value.isNone || { return "-" }
    return value.unwrap.toString
  }

  @class
  optionSymbol(value: Option<Symbol>) -> String {
    if value.isNone || { return "-" }
    return value.unwrap.toString
  }
}
