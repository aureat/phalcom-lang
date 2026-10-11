//! Unit observation capabilities shared without concrete-unit dependencies.

from ..dimension.dimension import Dimension
from ..exact.rational import Rational

/// A unit that can express an exact reference magnitude for semantic kind K.
///
/// This protocol keeps `Quantity` independent of concrete unit classes and
/// removes any Dynamic conversion seam from exact quantity observation.
/// Exact structural unit surface independent of nominal semantic kind.
trait ExactStructuralUnit {
  dimension -> Dimension
  scale -> Rational
}

trait ExactUnitFor<K> {
  fromReference(_ magnitude: Rational) -> Rational
}

/// A unit that can express a Float reference magnitude for semantic kind K.
///
/// Exact linear units and approximate units can both implement this surface.
trait FloatUnitFor<K> {
  fromReferenceFloat(_ magnitude: Float) -> Float
}

/// An affine unit capable of expressing an exact reference point.
trait ExactPointUnitFor<P> {
  fromReferencePoint(_ magnitude: Rational) -> Rational
}

/// An affine unit capable of expressing a Float reference point.
trait FloatPointUnitFor<P> {
  fromReferencePointFloat(_ magnitude: Float) -> Float
}

export ExactStructuralUnit, ExactUnitFor, FloatUnitFor, ExactPointUnitFor, FloatPointUnitFor
