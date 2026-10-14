//! Generic immutable intervals and boundary algebra.
//!
//! Provides `Bound<T>` and `Interval<T>` over ordered domains using canonical
//! bilateral `<=>` comparison. Interval semantics are independent of numeric `Range`.

/// Generic interval endpoint bound.
///
/// An endpoint is either `Unbounded` (representing -infinity for a lower bound,
/// +infinity for an upper bound), `Included(value)`, or `Excluded(value)`.
data Bound<T> {
  @variant Unbounded
  @variant Included(_ endpoint: T)
  @variant Excluded(_ endpoint: T)
}

impl<T> Bound<T> {
  @class
  unbounded -> Bound<T> {
    Bound<T>::Unbounded
  }

  @class
  included(_ value: T) -> Bound<T> {
    Bound<T>::Included(value)
  }

  @class
  excluded(_ value: T) -> Bound<T> {
    Bound<T>::Excluded(value)
  }

  /// True iff this bound is Unbounded.
  isUnbounded -> Bool {
    match self {
      Unbounded => true
      _ => false
    }
  }

  /// True iff this bound includes its finite endpoint value.
  isIncluded -> Bool {
    match self {
      Included(_) => true
      _ => false
    }
  }

  /// True iff this bound excludes its finite endpoint value.
  isExcluded -> Bool {
    match self {
      Excluded(_) => true
      _ => false
    }
  }

  /// The contained finite value, or None if Unbounded.
  value -> Option<T> {
    match self {
      Included(v) => Option::Some(v)
      Excluded(v) => Option::Some(v)
      Unbounded => None
    }
  }

  ==(_ other) -> Bool {
    if other is not Bound { return false }
    match self {
      Unbounded => {
        match other {
          Bound::Unbounded => true
          _ => false
        }
      }
      Included(value) => {
        match other {
          Bound::Included(otherValue) => value == otherValue
          _ => false
        }
      }
      Excluded(value) => {
        match other {
          Bound::Excluded(otherValue) => value == otherValue
          _ => false
        }
      }
    }
  }

  hash -> Int {
    match self {
      Unbounded => 7
      Included(value) => 17 + value.hash
      Excluded(value) => 31 + value.hash
    }
  }

  toString -> String {
    match self {
      Unbounded => "Unbounded"
      Included(v) => "Included(\(v))"
      Excluded(v) => "Excluded(\(v))"
    }
  }
}

/// Generic interval defined by lower and upper bounds over an ordered domain.
data Interval<T>(_ lower: Bound<T>, _ upper: Bound<T>)

impl<T> Interval<T> {
  // --- Convenience Factory Methods ---

  /// Creates a closed interval [a, b].
  @class
  closed(_ a: T, _ b: T) -> Interval<T> {
    Interval(Bound<T>::Included(a), Bound<T>::Included(b))
  }

  /// Creates an open interval (a, b).
  @class
  open(_ a: T, _ b: T) -> Interval<T> {
    Interval(Bound<T>::Excluded(a), Bound<T>::Excluded(b))
  }

  /// Creates a half-open interval [a, b).
  @class
  halfOpenRight(_ a: T, _ b: T) -> Interval<T> {
    Interval(Bound<T>::Included(a), Bound<T>::Excluded(b))
  }

  /// Creates a half-open interval (a, b].
  @class
  halfOpenLeft(_ a: T, _ b: T) -> Interval<T> {
    Interval(Bound<T>::Excluded(a), Bound<T>::Included(b))
  }

  /// Creates an unbounded-right interval [a, +inf).
  @class
  atLeast(_ a: T) -> Interval<T> {
    Interval(Bound<T>::Included(a), Bound<T>.unbounded)
  }

  /// Creates an unbounded-right interval (a, +inf).
  @class
  greaterThan(_ a: T) -> Interval<T> {
    Interval(Bound<T>::Excluded(a), Bound<T>.unbounded)
  }

  /// Creates an unbounded-left interval (-inf, b].
  @class
  atMost(_ b: T) -> Interval<T> {
    Interval(Bound<T>.unbounded, Bound<T>::Included(b))
  }

  /// Creates an unbounded-left interval (-inf, b).
  @class
  lessThan(_ b: T) -> Interval<T> {
    Interval(Bound<T>.unbounded, Bound<T>::Excluded(b))
  }

  /// Creates the fully unbounded interval (-inf, +inf).
  @class
  all -> Interval<T> {
    Interval(Bound<T>.unbounded, Bound<T>.unbounded)
  }

  /// Creates a singleton interval [point, point].
  @class
  singleton(_ point: T) -> Interval<T> {
    Interval(Bound<T>::Included(point), Bound<T>::Included(point))
  }

  // --- Endpoint Algebra Kernel Methods ---

  /// Compares two finite domain values via <=>, raising InvalidIntervalError if Unordered.
  @class
  compareDomain<K>(_ a: K, _ b: K) -> Ordering {
    let ord = a <=> b
    if ord == Ordering::Unordered {
      InvalidIntervalError.new("Ordered domain comparison yielded Ordering::Unordered").raise()
    }
    ord
  }

  /// Canonical lower bound comparison: Unbounded (-inf) < Included(x) < Excluded(x).
  @class
  compareLower(_ a: Bound<T>, _ b: Bound<T>) -> Ordering {
    match a {
      Unbounded => {
        match b {
          Unbounded => Ordering::Equal
          _ => Ordering::Less
        }
      }
      Included(xa) => {
        match b {
          Unbounded => Ordering::Greater
          Included(xb) => {
            Interval.compareDomain(xa, xb)
          }
          Excluded(xb) => {
            let ord = Interval.compareDomain(xa, xb)
            if ord == Ordering::Equal { Ordering::Less } else { ord }
          }
        }
      }
      Excluded(xa) => {
        match b {
          Unbounded => Ordering::Greater
          Included(xb) => {
            let ord = Interval.compareDomain(xa, xb)
            if ord == Ordering::Equal { Ordering::Greater } else { ord }
          }
          Excluded(xb) => {
            Interval.compareDomain(xa, xb)
          }
        }
      }
    }
  }

  /// Canonical upper bound comparison: Excluded(y) < Included(y) < Unbounded (+inf).
  @class
  compareUpper(_ a: Bound<T>, _ b: Bound<T>) -> Ordering {
    match a {
      Unbounded => {
        match b {
          Unbounded => Ordering::Equal
          _ => Ordering::Greater
        }
      }
      Included(ya) => {
        match b {
          Unbounded => Ordering::Less
          Included(yb) => {
            Interval.compareDomain(ya, yb)
          }
          Excluded(yb) => {
            let ord = Interval.compareDomain(ya, yb)
            if ord == Ordering::Equal { Ordering::Greater } else { ord }
          }
        }
      }
      Excluded(ya) => {
        match b {
          Unbounded => Ordering::Less
          Included(yb) => {
            let ord = Interval.compareDomain(ya, yb)
            if ord == Ordering::Equal { Ordering::Less } else { ord }
          }
          Excluded(yb) => {
            Interval.compareDomain(ya, yb)
          }
        }
      }
    }
  }

  /// Complement of a lower boundary for a left remainder's upper boundary.
  @class
  complementLowerForLeftUpper(_ bound: Bound<T>) -> Bound<T> {
    match bound {
      Included(x) => Bound<T>::Excluded(x)
      Excluded(x) => Bound<T>::Included(x)
      Unbounded => Bound<T>::Unbounded
    }
  }

  /// Complement of an upper boundary for a right remainder's lower boundary.
  @class
  complementUpperForRightLower(_ bound: Bound<T>) -> Bound<T> {
    match bound {
      Included(x) => Bound<T>::Excluded(x)
      Excluded(x) => Bound<T>::Included(x)
      Unbounded => Bound<T>::Unbounded
    }
  }

  // --- Instance Semantic Queries ---

  /// True iff lower <= upper (or at least one endpoint is Unbounded).
  /// Raises InvalidIntervalError if a finite endpoint comparison is unordered.
  isValid -> Bool {
    match self.lower {
      Unbounded => true
      Included(lVal) => {
        match self.upper {
          Unbounded => true
          Included(uVal) => {
            let ord = Interval.compareDomain(lVal, uVal)
            ord == Ordering::Less or ord == Ordering::Equal
          }
          Excluded(uVal) => {
            let ord = Interval.compareDomain(lVal, uVal)
            ord == Ordering::Less or ord == Ordering::Equal
          }
        }
      }
      Excluded(lVal) => {
        match self.upper {
          Unbounded => true
          Included(uVal) => {
            let ord = Interval.compareDomain(lVal, uVal)
            ord == Ordering::Less or ord == Ordering::Equal
          }
          Excluded(uVal) => {
            let ord = Interval.compareDomain(lVal, uVal)
            ord == Ordering::Less or ord == Ordering::Equal
          }
        }
      }
    }
  }

  /// True iff interval is valid and contains no points (e.g. [x, x), (x, x], (x, x)).
  isEmpty -> Bool {
    match self.lower {
      Unbounded => false
      Included(lVal) => {
        match self.upper {
          Unbounded => false
          Included(uVal) => false
          Excluded(uVal) => {
            let ord = Interval.compareDomain(lVal, uVal)
            ord == Ordering::Equal
          }
        }
      }
      Excluded(lVal) => {
        match self.upper {
          Unbounded => false
          Included(uVal) => {
            let ord = Interval.compareDomain(lVal, uVal)
            ord == Ordering::Equal
          }
          Excluded(uVal) => {
            let ord = Interval.compareDomain(lVal, uVal)
            ord == Ordering::Equal
          }
        }
      }
    }
  }

  /// Point containment query.
  contains(_ point: T) -> Bool {
    if not self.isValid or self.isEmpty { return false }
    match self.lower {
      Unbounded => {}
      Included(lVal) => {
        let ord = Interval.compareDomain(point, lVal)
        if ord == Ordering::Less { return false }
      }
      Excluded(lVal) => {
        let ord = Interval.compareDomain(point, lVal)
        if ord == Ordering::Less or ord == Ordering::Equal { return false }
      }
    }
    match self.upper {
      Unbounded => {}
      Included(uVal) => {
        let ord = Interval.compareDomain(point, uVal)
        if ord == Ordering::Greater { return false }
      }
      Excluded(uVal) => {
        let ord = Interval.compareDomain(point, uVal)
        if ord == Ordering::Greater or ord == Ordering::Equal { return false }
      }
    }
    return true
  }

  /// Intersection of two intervals. Returns Some(interval) if valid nonempty, else None.
  intersection(_ other: Interval<T>) -> Option<Interval<T>> {
    if not self.isValid or self.isEmpty or not other.isValid or other.isEmpty {
      return None
    }
    let maxLower = if Interval.compareLower(self.lower, other.lower) == Ordering::Greater {
      self.lower
    } else {
      other.lower
    }
    let minUpper = if Interval.compareUpper(self.upper, other.upper) == Ordering::Less {
      self.upper
    } else {
      other.upper
    }
    let candidate = Interval(maxLower, minUpper)
    if candidate.isValid and not candidate.isEmpty {
      Option::Some(candidate)
    } else {
      None
    }
  }

  /// True iff both intervals are valid nonempty and share at least one point.
  overlaps(_ other: Interval<T>) -> Bool {
    match self.intersection(other) {
      Option::Some(_) => true
      _ => false
    }
  }

  /// True iff intervals are disjoint and touch at a common finite boundary point
  /// where one includes and the other excludes that point (no unmapped gap).
  touchesWithoutGap(_ other: Interval<T>) -> Bool {
    if not self.isValid or self.isEmpty or not other.isValid or other.isEmpty {
      return false
    }
    // Check self to left of other
    match self.upper {
      Unbounded => {}
      Included(uVal) => {
        match other.lower {
          Excluded(lVal) => {
            let ord = Interval.compareDomain(uVal, lVal)
            if ord == Ordering::Equal { return true }
          }
          _ => {}
        }
      }
      Excluded(uVal) => {
        match other.lower {
          Included(lVal) => {
            let ord = Interval.compareDomain(uVal, lVal)
            if ord == Ordering::Equal { return true }
          }
          _ => {}
        }
      }
    }
    // Check other to left of self
    match other.upper {
      Unbounded => {}
      Included(uVal) => {
        match self.lower {
          Excluded(lVal) => {
            let ord = Interval.compareDomain(uVal, lVal)
            if ord == Ordering::Equal { return true }
          }
          _ => {}
        }
      }
      Excluded(uVal) => {
        match self.lower {
          Included(lVal) => {
            let ord = Interval.compareDomain(uVal, lVal)
            if ord == Ordering::Equal { return true }
          }
          _ => {}
        }
      }
    }
    return false
  }

  /// Union of two intervals if they overlap or touch without gap; otherwise None.
  unionIfContiguous(_ other: Interval<T>) -> Option<Interval<T>> {
    if not self.isValid or self.isEmpty or not other.isValid or other.isEmpty {
      return None
    }
    if not self.overlaps(other) and not self.touchesWithoutGap(other) {
      return None
    }
    let minLower = if Interval.compareLower(self.lower, other.lower) == Ordering::Less {
      self.lower
    } else {
      other.lower
    }
    let maxUpper = if Interval.compareUpper(self.upper, other.upper) == Ordering::Greater {
      self.upper
    } else {
      other.upper
    }
    Option::Some(Interval(minLower, maxUpper))
  }

  /// Computes remaining portions of self after subtracting region r.
  remaindersAfterRemoval(_ r: Interval<T>) -> Array<Interval<T>> {
    if not self.isValid or self.isEmpty { return [] }
    if not r.isValid or r.isEmpty or not self.overlaps(r) { return [self] }

    let hasLeft = Interval.compareLower(self.lower, r.lower) == Ordering::Less
    let hasRight = Interval.compareUpper(self.upper, r.upper) == Ordering::Greater

    if hasLeft and hasRight {
      let leftUpper = Interval.complementLowerForLeftUpper(r.lower)
      let rightLower = Interval.complementUpperForRightLower(r.upper)
      let leftIv = Interval(self.lower, leftUpper)
      let rightIv = Interval(rightLower, self.upper)
      let leftOk = leftIv.isValid and not leftIv.isEmpty
      let rightOk = rightIv.isValid and not rightIv.isEmpty
      if leftOk and rightOk { return [leftIv, rightIv] }
      if leftOk { return [leftIv] }
      if rightOk { return [rightIv] }
      return []
    }
    if hasLeft {
      let leftUpper = Interval.complementLowerForLeftUpper(r.lower)
      let leftIv = Interval(self.lower, leftUpper)
      if leftIv.isValid and not leftIv.isEmpty { return [leftIv] }
      return []
    }
    if hasRight {
      let rightLower = Interval.complementUpperForRightLower(r.upper)
      let rightIv = Interval(rightLower, self.upper)
      if rightIv.isValid and not rightIv.isEmpty { return [rightIv] }
      return []
    }
    return []
  }

  ==(_ other) -> Bool {
    if other is not Interval { return false }
    self.lower == other.lower and self.upper == other.upper
  }

  hash -> Int {
    self.lower.hash * 31 + self.upper.hash
  }

  toString -> String {
    let leftBracket = match self.lower {
      Included(_) => "["
      _ => "("
    }
    let rightBracket = match self.upper {
      Included(_) => "]"
      _ => ")"
    }
    let leftVal = match self.lower {
      Unbounded => "-inf"
      Included(v) => "\(v)"
      Excluded(v) => "\(v)"
    }
    let rightVal = match self.upper {
      Unbounded => "+inf"
      Included(v) => "\(v)"
      Excluded(v) => "\(v)"
    }
    "\(leftBracket)\(leftVal), \(rightVal)\(rightBracket)"
  }

  /// Structural view of this interval.
  view -> IntervalView<T> {
    if not self.isValid or self.isEmpty {
      IntervalView::Empty
    } else {
      IntervalView::Span(self.lower, self.upper)
    }
  }

  /// Computes the convex hull enclosing both intervals.
  hull(_ other: Interval<T>) -> Interval<T> {
    if not self.isValid or self.isEmpty { return other }
    if not other.isValid or other.isEmpty { return self }
    let minLower = if Interval.compareLower(self.lower, other.lower) == Ordering::Less {
      self.lower
    } else {
      other.lower
    }
    let maxUpper = if Interval.compareUpper(self.upper, other.upper) == Ordering::Greater {
      self.upper
    } else {
      other.upper
    }
    Interval(minLower, maxUpper)
  }

  /// Subtracts another interval from self, returning zero, one, or two remaining pieces.
  subtract(_ other: Interval<T>) -> IntervalPieces<T> {
    let rems = self.remaindersAfterRemoval(other)
    if rems.size == 0 {
      IntervalPieces::Zero
    } else if rems.size == 1 {
      IntervalPieces::One(rems.at(0))
    } else {
      IntervalPieces::Two(rems.at(0), rems.at(1))
    }
  }

  /// Computes the relational categorization between self and another interval.
  relate(_ other: Interval<T>) -> IntervalRelation<T> {
    let selfEmpty = not self.isValid or self.isEmpty
    let otherEmpty = not other.isValid or other.isEmpty
    if selfEmpty and otherEmpty {
      return IntervalRelation::BothEmpty
    }
    if selfEmpty {
      return IntervalRelation::LeftEmpty(other)
    }
    if otherEmpty {
      return IntervalRelation::RightEmpty(self)
    }
    if self.overlaps(other) {
      let shared = self.intersection(other).unwrap()
      let leftOnly = self.subtract(other)
      let rightOnly = other.subtract(self)
      return IntervalRelation::Intersects(shared, leftOnly, rightOnly)
    }
    if self.touchesWithoutGap(other) {
      if Interval.compareLower(self.lower, other.lower) == Ordering::Less {
        let b = self.upper.value.unwrap()
        let gap = Interval(self.upper, other.lower)
        return IntervalRelation::TouchesBefore(b, gap)
      } else {
        let b = other.upper.value.unwrap()
        let gap = Interval(other.upper, self.lower)
        return IntervalRelation::TouchesAfter(b, gap)
      }
    }
    if Interval.compareUpper(self.upper, other.lower) == Ordering::Less {
      let gap = Interval(self.upper, other.lower)
      IntervalRelation::Before(gap)
    } else {
      let gap = Interval(other.upper, self.lower)
      IntervalRelation::After(gap)
    }
  }

  /// Computes qualitative temporal Allen relation for finite nonempty spans.
  allenRelation(_ other: Interval<T>) -> Option<AllenRelation> {
    if not self.isValid or self.isEmpty or not other.isValid or other.isEmpty {
      return None
    }
    let a0Opt = self.lower.value
    let a1Opt = self.upper.value
    let b0Opt = other.lower.value
    let b1Opt = other.upper.value
    match a0Opt {
      Option::Some(a0) => {
        match a1Opt {
          Option::Some(a1) => {
            match b0Opt {
              Option::Some(b0) => {
                match b1Opt {
                  Option::Some(b1) => {
                    if a0 == a1 or b0 == b1 {
                      return None
                    }
                    if a1 < b0 {
                      return Option::Some(AllenRelation::Before)
                    }
                    if a1 == b0 {
                      return Option::Some(AllenRelation::Meets)
                    }
                    if a0 < b0 and b0 < a1 and a1 < b1 {
                      return Option::Some(AllenRelation::Overlaps)
                    }
                    if a0 == b0 and a1 < b1 {
                      return Option::Some(AllenRelation::Starts)
                    }
                    if b0 < a0 and a1 < b1 {
                      return Option::Some(AllenRelation::During)
                    }
                    if b0 < a0 and a1 == b1 {
                      return Option::Some(AllenRelation::Finishes)
                    }
                    if a0 == b0 and a1 == b1 {
                      return Option::Some(AllenRelation::Equal)
                    }
                    if a0 < b0 and a1 == b1 {
                      return Option::Some(AllenRelation::FinishedBy)
                    }
                    if a0 < b0 and b1 < a1 {
                      return Option::Some(AllenRelation::Contains)
                    }
                    if a0 == b0 and b1 < a1 {
                      return Option::Some(AllenRelation::StartedBy)
                    }
                    if b0 < a0 and a0 < b1 and b1 < a1 {
                      return Option::Some(AllenRelation::OverlappedBy)
                    }
                    if b1 == a0 {
                      return Option::Some(AllenRelation::MetBy)
                    }
                    if b1 < a0 {
                      return Option::Some(AllenRelation::After)
                    }
                    None
                  }
                  _ => None
                }
              }
              _ => None
            }
          }
          _ => None
        }
      }
      _ => None
    }
  }
}

type LowerBound<T> = Bound<T>
type UpperBound<T> = Bound<T>

/// Structural decomposition view of an interval.
data IntervalView<T> {
  @variant Empty
  @variant Span(_ lower: Bound<T>, _ upper: Bound<T>)
}

/// Zero, one, or two remaining pieces produced by interval subtraction.
data IntervalPieces<T> {
  @variant Zero
  @variant One(_ piece: Interval<T>)
  @variant Two(_ first: Interval<T>, _ second: Interval<T>)
}

/// Eight-case total relational partition between two intervals.
data IntervalRelation<T> {
  @variant BothEmpty
  @variant LeftEmpty(_ right: Interval<T>)
  @variant RightEmpty(_ left: Interval<T>)
  @variant Before(_ gap: Interval<T>)
  @variant After(_ gap: Interval<T>)
  @variant TouchesBefore(_ boundary: T, _ gap: Interval<T>)
  @variant TouchesAfter(_ boundary: T, _ gap: Interval<T>)
  @variant Intersects(_ shared: Interval<T>, _ leftOnly: IntervalPieces<T>, _ rightOnly: IntervalPieces<T>)
}

/// Qualitative temporal Allen relation across thirteen geometric cases.
data AllenRelation {
  @variant Before
  @variant Meets
  @variant Overlaps
  @variant Starts
  @variant During
  @variant Finishes
  @variant Equal
  @variant FinishedBy
  @variant Contains
  @variant StartedBy
  @variant OverlappedBy
  @variant MetBy
  @variant After
}

export Bound, LowerBound, UpperBound, Interval, IntervalView, IntervalPieces, IntervalRelation, AllenRelation
