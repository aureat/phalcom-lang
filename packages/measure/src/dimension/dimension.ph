/// One axis/exponent pair in a structural dimension.
///
/// `Dimension` maintains the stronger invariant that an axis appears at most
/// once and that zero exponents are absent.
data DimensionTerm(
  _ axis: String,
  _ exponent: Int
)

/// Persistent private sequence used to keep `Dimension` deeply immutable.
///
/// The sequence is deliberately package-private implementation structure
/// rather than a mutable `List`. Dimension operations rebuild only the path
/// they change and may share untouched tails.
data DimensionTerms {
  Empty

  Cons(
    _ term: DimensionTerm,
    _ rest: DimensionTerms
  )
}

/// Structural mathematical dimension.
///
/// A Dimension is a normalized finite map from axis identifiers to non-zero
/// integral exponents. The representation is persistent and immutable.
///
/// Term order is not semantic. `equivalent` therefore compares dimensions
/// independently of internal sequence order.
///
/// ## Complexity
///
/// With the current persistent-list representation, merge and equivalence are
/// O(k²) in the number of distinct axes. This is intentional for the
/// experiment: `k` is normally tiny and never depends on the number of units
/// declared in the program.
data Dimension private(
  _ terms: DimensionTerms
)

impl Dimension {
  /// Multiplicative identity / dimensionless dimension.
  @class
  one -> Dimension {
    Dimension(DimensionTerms::Empty)
  }

  /// Creates one base dimension axis with exponent +1.
  @class
  base(_ axis: String) -> Dimension {
    if axis.size == 0 {
      ArgumentError.new("dimension axis cannot be empty").raise()
    }

    Dimension(
      DimensionTerms::Cons(
        DimensionTerm(axis, 1),
        DimensionTerms::Empty
      )
    )
  }

  isDimensionless -> Bool {
    match self.terms {
      DimensionTerms::Empty => true
      DimensionTerms::Cons(_, _) => false
    }
  }

  /// Structural multiplication.
  ///
  /// Exponents of equal axes are added and zero results are eliminated.
  *(_ other: Dimension) -> Dimension {
    Dimension(
      Dimension.apply(
        self.terms,
        other.terms,
        sign: 1
      )
    )
  }

  /// Structural division.
  ///
  /// Exponents of RHS axes are subtracted and zero results are eliminated.
  /(_ other: Dimension) -> Dimension {
    Dimension(
      Dimension.apply(
        self.terms,
        other.terms,
        sign: -1
      )
    )
  }

  /// Raises the complete dimension to an integral power.
  ///
  /// Power zero produces the dimensionless identity. Negative powers negate
  /// every exponent.
  **(_ exponent: Int) -> Dimension {
    if exponent == 0 {
      return Dimension.one
    }

    Dimension(
      Dimension.scaleExponents(
        self.terms,
        by: exponent
      )
    )
  }

  /// Order-insensitive structural equivalence.
  equivalent(_ other: Dimension) -> Bool {
    if Dimension.count(self.terms) != Dimension.count(other.terms) {
      return false
    }

    Dimension.everyTermMatches(
      self.terms,
      other.terms
    )
  }

  /// Exponent associated with `axis`, or zero if the axis is absent.
  exponent(of axis: String) -> Int {
    Dimension.findExponent(
      self.terms,
      axis: axis
    )
  }

  /// Human-readable structural rendering.
  ///
  /// Rendering preserves the persistent representation order; that order is
  /// deliberately not part of semantic equivalence.
  toString -> String {
    match self.terms {
      DimensionTerms::Empty => "1"
      DimensionTerms::Cons(_, _) => {
        Dimension.render(
          self.terms,
          first: true
        )
      }
    }
  }

  /// Applies every term from `incoming` to `base`.
  @class
  private apply(
    _ base: DimensionTerms,
    _ incoming: DimensionTerms,
    sign: Int
  ) -> DimensionTerms {
    match incoming {
      DimensionTerms::Empty => base

      DimensionTerms::Cons(term, rest) => {
        let adjusted = Dimension.adjust(
          base,
          axis: term.axis,
          by: term.exponent * sign
        )

        Dimension.apply(
          adjusted,
          rest,
          sign: sign
        )
      }
    }
  }

  /// Adds `delta` to one axis while preserving normalization.
  @class
  private adjust(
    _ terms: DimensionTerms,
    axis: String,
    by delta: Int
  ) -> DimensionTerms {
    if delta == 0 {
      return terms
    }

    match terms {
      DimensionTerms::Empty => {
        DimensionTerms::Cons(
          DimensionTerm(axis, delta),
          DimensionTerms::Empty
        )
      }

      DimensionTerms::Cons(term, rest) => {
        if term.axis == axis {
          let exponent = term.exponent + delta

          if exponent == 0 {
            rest
          } else {
            DimensionTerms::Cons(
              DimensionTerm(axis, exponent),
              rest
            )
          }
        } else {
          DimensionTerms::Cons(
            term,
            Dimension.adjust(
              rest,
              axis: axis,
              by: delta
            )
          )
        }
      }
    }
  }

  /// Multiplies every exponent by `factor`.
  @class
  private scaleExponents(
    _ terms: DimensionTerms,
    by factor: Int
  ) -> DimensionTerms {
    match terms {
      DimensionTerms::Empty => DimensionTerms::Empty

      DimensionTerms::Cons(term, rest) => {
        let exponent = term.exponent * factor

        if exponent == 0 {
          Dimension.scaleExponents(
            rest,
            by: factor
          )
        } else {
          DimensionTerms::Cons(
            DimensionTerm(
              term.axis,
              exponent
            ),
            Dimension.scaleExponents(
              rest,
              by: factor
            )
          )
        }
      }
    }
  }

  /// Finds one axis exponent. Normalization guarantees uniqueness.
  @class
  private findExponent(
    _ terms: DimensionTerms,
    axis: String
  ) -> Int {
    match terms {
      DimensionTerms::Empty => 0

      DimensionTerms::Cons(term, rest) => {
        if term.axis == axis {
          term.exponent
        } else {
          Dimension.findExponent(
            rest,
            axis: axis
          )
        }
      }
    }
  }

  /// Number of non-zero dimension axes.
  @class
  private count(
    _ terms: DimensionTerms
  ) -> Int {
    match terms {
      DimensionTerms::Empty => 0

      DimensionTerms::Cons(_, rest) => {
        1 + Dimension.count(rest)
      }
    }
  }

  /// Returns true when every term in `left` appears identically in `right`.
  @class
  private everyTermMatches(
    _ left: DimensionTerms,
    _ right: DimensionTerms
  ) -> Bool {
    match left {
      DimensionTerms::Empty => true

      DimensionTerms::Cons(term, rest) => {
        if Dimension.findExponent(
          right,
          axis: term.axis
        ) != term.exponent {
          false
        } else {
          Dimension.everyTermMatches(
            rest,
            right
          )
        }
      }
    }
  }

  @class
  private render(
    _ terms: DimensionTerms,
    first: Bool
  ) -> String {
    match terms {
      DimensionTerms::Empty => ""

      DimensionTerms::Cons(term, rest) => {
        let separator = if first { "" } else { "·" }

        let current =
          if term.exponent == 1 {
            "\(separator)\(term.axis)"
          } else {
            "\(separator)\(term.axis)^\(term.exponent)"
          }

        current + Dimension.render(
          rest,
          first: false
        )
      }
    }
  }
}

export Dimension, DimensionTerm
