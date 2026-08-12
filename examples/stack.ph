// ============================================================
//  Phalcom feature showcase
//  A generic Stack built on the kernel List, using Options for
//  partial operations. Exercises most of the language surface:
//  classes, keyword constructors, static factories, getters,
//  operator overloading, blocks/closures, control flow,
//  Option/List protocols, string interpolation, introspection.
// ============================================================
class Stack {
  const _items = []

  // Keyword constructor (`construct`): binds instance state directly
  // from labeled arguments.
  @constructor
  new(items) { _items = items }

  // Static named constructor — the idiomatic factory pattern.
  @class
  empty { Stack.new(items: List.new()) }

  // Braced getters over private instance vars (`_items`).
  size { _items.size }
  isEmpty { _items.isEmpty }

  // Closure-bodied method returning `self` so calls chain.
  push(_ v) {
    _items.append(v)
    self
  }

  // Partial operation modelled with Option, never nil:
  //   Some(top) on a non-empty stack, None otherwise.
  top {
    if (_items.isEmpty) {
      return None
    }
    Some.new(_items.at(_items.size - 1))
  }

  // Higher-order method: fold with a caller-supplied 2-arity block.
  fold(initial seed, using f) {
    _items.fold(initial: seed, using: f)
  }

  // Operator overloading: `+` concatenates two stacks.
  +(_ other) {
    let out = List.new()
    _items.each |x| { out.append(x) }
    other.eachItem |x| { out.append(x) }
    return Stack.new(items: out)
  }

  eachItem(_ f) { _items.each(f) }
  itemAt(_ i) { _items.at(i) }

  // Structural equality via operator overload.
  ==(_ other) {
    if (self.size != other.size) { return false }
    let eq = true
    let i = 0
    while (i < self.size) {
      if (_items.at(i) != other.itemAt(i)) { eq = false }
      i = i + 1
    }
    return eq
  }

  // String interpolation (`\(expr)`), which stringifies numbers.
  toString { "Stack(size: \(self.size))" }
}

// ---- Driver: numbers, blocks, control flow, Option, interpolation ----

const s = Stack.empty
s.push(10).push(20).push(30)

System.print(s.toString)                 // Stack(size: 3)
System.print("empty? \(s.isEmpty)")      // empty? false

// Option-returning peek, handled without nil.
s.top.ifSome |v| { System.print("top is \(v)") }        // top is 30
Stack.empty.top.ifNone || { System.print("nothing to peek") } // nothing to peek

// Higher-order fold with a two-argument block.
const sum = s.fold(initial: 0, using: |acc, x| { acc + x })
System.print("sum = \(sum)")             // sum = 60

// Operator overloading: concatenation.
const t = Stack.empty.push(40).push(50)
System.print("combined \((s + t).size)") // combined 5

// Structural equality.
const a = Stack.empty.push(1).push(2)
const b = Stack.empty.push(1).push(2)
System.print("a == b: \(a == b)")        // a == b: true

// Introspection over the object model.
System.print("class of s : \(s.class)")       // <class Stack>
System.print("s isA Stack : \(s.isA(Stack))") // true
System.print("s isA Object: \(s.isA(Object))")// true
