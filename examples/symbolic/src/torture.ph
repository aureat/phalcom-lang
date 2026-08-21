#!/usr/bin/env phalcom
// Phalcom syntax torture test.
//
// One intentionally readable file for syntax highlighting, semantic tokens,
// completion, hover, folding, navigation, and parser recovery work. Sections
// are arranged from declarations to expressions to deliberately dense edges.

/// Module documentation comment.
//! Inner documentation comment remains visible to editor tooling.

import .module as Module
from .base import (Base, Theme)

expose .support

export Demo, render


// ---------------------------------------------------------------------------
// Classes, fields, attributes, variants, constructors, accessors
// ---------------------------------------------------------------------------

@sealed
@documentation("A small algebraic-looking value container.")
class Option {
  @variant Some(value:)
  @variant None()

  _value

  @constructor
  new(_ value) {
    _value = value
  }

  is_some() {
    _value != None
  }

  value {
    _value
  }

  value=(put next) {
    _value = next
    self
  }

  unwrap_or(_ fallback) {
    if (_value != None) {
      _value
    } else {
      fallback
    }
  }
}


@derive(Equatable, Printable)
class Demo is Base {
  _name
  _count = 0

  const _kind = #demo

  @class _default_name = "anonymous"

  @constructor
  new(_ name, count) {
    _name = name
    _count = count
  }

  @class
  default() {
    Demo.new(_default_name, 0)
  }

  name {
    _name
  }

  name=(put next) {
    _name = next
  }

  count {
    _count
  }

  increment() {
    _count += 1
    self
  }

  reset() {
    _count = 0
    self
  }

  // Operator methods are ordinary message implementations.
  +(_ other) { _count + other.count }
  -(_ other) { _count - other.count }
  *(_ factor) { _count * factor }
  /(_ divisor) { _count / divisor }
  %(_ divisor) { _count % divisor }
  ~/(_ divisor) { _count ~/ divisor }
  **(_ exponent) { _count ** exponent }
  &(_ other) { _count & other }
  |(_ other) { _count | other }
  ^(_ other) { _count ^ other }
  ~() { ~_count }
  <<(_ bits) { _count << bits }
  >>(_ bits) { _count >> bits }
  ==(_ other) { _count == other.count }
  !=(_ other) { _count != other.count }
  <(_ other) { _count < other.count }
  <=(_ other) { _count <= other.count }
  >(_ other) { _count > other.count }
  >=(_ other) { _count >= other.count }
  and(_ other) { _count and other }
  or(_ other) { _count or other }
  is(_ candidate) { self.class == candidate }
  not() { false }

  [_ index] {
    if (index == 0) {
      _name
    } else {
      _count
    }
  }

  [_ index]=(put value) {
    if (index == 0) {
      _name = value
    } else {
      _count = value
    }
  }

  @requires(_name != "")
  @ensures(result != None)
  describe() {
    let prefix = "Demo"
    "\(prefix)(name=\(_name), count=\(_count))"
  }

  @invariant(_count >= 0)
}


@native
class FileDescriptor {
  @constructor
  open(_ path) {
    _path = path
  }

  close() {
    self
  }
}


// ---------------------------------------------------------------------------
// Module-shaped declarations and bindings
// ---------------------------------------------------------------------------

const answer = 42
let mutable = 0
let uninitialized
const label = #demo
const selector = #describe
const method_selector = #move(_,to)
const operator_selector = #+
const index_selector = #[_]
const setter_selector = #name=(put)
const any_selector = #render(...)

let point = (10, 20)
let labeled_point = (x: 10, y: 20)
let empty_tuple = ()
let one_tuple = (answer,)
let tuple_with_spread = (1, 2, *point)
let labeled_tuple = (x: 1, y: 2, ** #{z: 3})

let values = [1, 2, 3,]
let empty_list = [0]
let list_with_spread = [0, *values, 4]

let record = #{
  name: "Alice",
  count: 3,
  active: true,
}
let empty_record = #{}
let record_with_spread = #{**record, selected: false}

let map = {
  name: "Ada",
  number: 13,
}
let computed_map = {
  [answer]: "computed key",
}

let (x, y) = point
let [first, second, *tail] = [1, 2, 3, 4]
let [(nested_x, nested_y), last] = [(1, 2), 3]


// ---------------------------------------------------------------------------
// Sends, selectors, references, indexing, labels, and expansion lanes
// ---------------------------------------------------------------------------

let demo = Demo.new("reader", 7)
demo.name
demo.name = "writer"
demo.increment()
demo[0]
demo[1] = 99
demo?.name
demo ?? Demo.default()

let chained =
  Demo.default()
    .increment()
    .name

let reference_getter = demo::name
let reference_method = demo::increment()
let reference_selector = demo::move(_,to)
let reference_pattern = demo::render(...)

let positional_call = render(demo, answer)
let labeled_call = render(value: demo, count: answer)
let expanded_call = render(*values, **record)
let complete_call = render(***labeled_tuple)
let callback_call = values.map |item| { item + 1 }
let labeled_callback_call = values.reduce(initial: 0, step: |total, item| total + item)


@sealed @data
class SyntaxNode {
  /// Variant for literal values, such as numbers, strings, and booleans.
  @variant Literal(value:, kind:)

  /// Variant for identifiers, such as variable names and function names.
  @variant Identifier(name:)

  /// Variant for unary operations, such as negation and logical not.
  @variant Unary(operator:, operand:)

  /// Variant for binary operations, such as addition and multiplication.
  @variant Binary(left:, operator:, right:)

  /// Variant for function or method calls, including the callee and arguments.
  @variant Call(callee:, arguments:)

  /// Variant for conditional expressions, including the condition, then branch, and else branch.
  @variant Block(statements:)

  /// Missing variant to represent the absence of a syntax node, useful for optional fields or error recovery.
  @variant Missing()
}


// ---------------------------------------------------------------------------
// Closures, control flow, short-circuiting, and assignment
// ---------------------------------------------------------------------------

let empty_block = || {}
let expression_block = |value| value * 2
let statement_block = |value, other| {
  let sum = value + other
  sum
}
let rest_block = |*items| {
  items
}

if (demo is Demo) {
  demo.increment()
} else if (demo is! Option) {
  demo.reset()
} else {
  demo
}

let type_answer = if (answer is Number) { true } else { false }
let exact_answer = answer is! Number
let negated_kind = demo is not Option
let negated_exact = demo is! not Demo

while (mutable < 3) {
  mutable += 1
  if (mutable == 2) {
    continue
  }
  if (mutable == 3) {
    break
  }
}

for (item in values) {
  item
}

let short_circuit = true and false or not false
let coalesced = uninitialized ?? "fallback"
let conditional_send = uninitialized?.name
let arithmetic = -answer + 2 * 3 ** 2 ~/ 4 % 2
let bitwise = (answer << 1) | (answer >> 1) & ~answer ^ 3
let comparisons = answer == 42 and answer != 0 and answer <= 100
let ranges = 1..10
// Reserved range spelling, kept in a comment until exclusive ranges land: 1...10


// ---------------------------------------------------------------------------
// Errors, unwinding, and explicit returns
// ---------------------------------------------------------------------------

class DemoError is Error {
  @get @set _message

  @constructor
  new(_ message) {
    _message = message
  }

  message { _message }
}


class Harness {
  @class
  safe_operation() {
    try {
      if (answer == 42) {
        DemoError.new("expected demo failure").raise()
      }
      answer
    } on DemoError error {
      error.message
    } on Error fallback {
      fallback
    } catch unexpected {
      unexpected
    } ensure {
      System.print("cleanup")
    }
  }

  @class
  raise_directly() {
    throw DemoError.new("raised")
  }

  @class
  early_return(_ flag) {
    if (flag) {
      return #early
    }
    return #late
  }

  @class
  render(_ value, count) {
    let normalized = value?.name ?? "unknown"
    let numbers = [1, 2, 3]
    let summary = {
      value: normalized,
      count: count,
      ok: value is not None and count > 0,
    }

    for (number in numbers) {
      if (number is! Number) {
        continue
      }
      System.print("\(number): \(summary.value)")
    }

    summary
  }
}


// ---------------------------------------------------------------------------
// Lexical edge cases: comments, strings, numbers, and reserved spellings
// ---------------------------------------------------------------------------

/* Block comment with punctuation: {} [] () # @ :: ?. ?? -> ** *** */
let escaped = "quote: \" slash: \\ newline: \n tab: \t return: \r"
let interpolated = "name=\(demo.name), count=\(demo.count)"
let literal_backslash_paren = "not interpolation: \\(answer)"
let multiline = """
  first line
  second line: \(demo.name)
  third line
  """

let decimal = 1_000_000
let binary = 0b1010_0101
let octal = 0o755
let hexadecimal = 0xCAFE_BABE
let fraction = 3.141_592
let leading_fraction = .25
let scientific = 6.02e-23

// Retired or contextual words are exercised in safe selector positions.
let contextual_try = fiber.try()
let contextual_on = handler.on(Error)
let contextual_catch = block.catch()
let contextual_ensure = block.ensure()
let class_property = self.class
let superclass_property = super.class


// ---------------------------------------------------------------------------
// Final compact call site for editor inspection
// ---------------------------------------------------------------------------

const final_demo = Harness.render(demo, answer)
final_demo
