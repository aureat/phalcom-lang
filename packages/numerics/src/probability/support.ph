from universe.collections.iterator_laws import Iterable

/// Runtime support descriptions. These describe mathematical membership,
/// independently of whether a finite-precision sampler can resolve every point.
data RealSupport private(
  _ lower: Float,
  _ upper: Float,
  _ lowerClosed: Bool,
  _ upperClosed: Bool
)

impl RealSupport {
  @class
  closed(_ lower: Float, _ upper: Float) -> RealSupport {
    if not lower.isFinite or not upper.isFinite or lower > upper {
      ArgumentError.new("closed real support requires finite lower <= upper").raise()
    }
    return RealSupport(lower, upper, true, true)
  }

  @class
  atLeast(_ lower: Float) -> RealSupport {
    if not lower.isFinite {
      ArgumentError.new("real support lower bound must be finite").raise()
    }
    return RealSupport(lower, 1.0 / 0.0, true, false)
  }

  @class
  allReal() -> RealSupport {
    RealSupport(-1.0 / 0.0, 1.0 / 0.0, false, false)
  }

  contains(_ value: Float) -> Bool {
    if not value.isFinite { return false }
    if value < self.lower or value > self.upper { return false }
    if value == self.lower and not self.lowerClosed { return false }
    if value == self.upper and not self.upperClosed { return false }
    return true
  }
}

/// Integer interval support. A missing upper bound denotes an unbounded interval.
data IntegerSupport private(_ lower: Int, _ upperBound: Option<Int>)

impl IntegerSupport {
  @class
  closed(_ lower: Int, _ upper: Int) -> IntegerSupport {
    if lower > upper { ArgumentError.new("integer support requires lower <= upper").raise() }
    IntegerSupport(lower, Some(upper))
  }

  @class
  atLeast(_ lower: Int) -> IntegerSupport {
    IntegerSupport(lower, None)
  }

  @class
  singleton(_ value: Int) -> IntegerSupport {
    IntegerSupport(value, Some(value))
  }

  contains(_ value: Int) -> Bool {
    if value < self.lower { return false }
    return self.upperBound.match(
      some: |upper| { value <= upper },
      none: || { true }
    )
  }
}

/// Finite label support with copied labels and stable equality membership.
data FiniteSupport<T> private(_ labels: List<T>, _ members: Set<T>)

impl<T> FiniteSupport<T> {
  @class
  from<I>(_ values: I) -> FiniteSupport<T>
    where I |= Iterable, <I as Iterable>::Item == T
  {
    let labels = List<T>.new()
    let members = Set<T>.new()
    let mut iterator = values.iterator
    let mut step = iterator.next
    while step.isSome {
      let value: T = step.match(some: |item| { item }, none: || { None })
      if members.contains(value) {
        ArgumentError.new("finite support labels must be unique under stable equality").raise()
      }
      labels.append(value)
      members.add(value)
      step = iterator.next
    }
    if labels.size == 0 { ArgumentError.new("finite support must be nonempty").raise() }
    return FiniteSupport(labels, members)
  }

  contains(_ value: T) -> Bool { self.members.contains(value) }

  values -> List<T> {
    let result = List<T>.new()
    let mut i = 0
    while i < self.labels.size {
      result.append(self.labels.at(i))
      i = i + 1
    }
    return result
  }
}

export RealSupport, IntegerSupport, FiniteSupport
