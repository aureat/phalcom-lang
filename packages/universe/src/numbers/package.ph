@!documentation("Explicit numeric values and independent closed arithmetic capabilities. Complex<T> preserves component types and exact applied identity.")

expose .capabilities
expose .algebra
expose .complex
expose .quaternion
expose .rational
expose .float_complex

from .capabilities import Addable, Subtractable, Multipliable, Negatable, IntegerPower, Zero, One
from .algebra import I, J, K, Basis, i, j, k, BasisRealizable, RealEmbeddable
from .complex import Complex, ComplexDivision
from .quaternion import Quaternion
from .rational import Rational

export Rational, Complex, ComplexDivision, Quaternion, I, J, K, Basis, i, j, k, Addable, Subtractable, Multipliable, Negatable, IntegerPower, Zero, One, BasisRealizable, RealEmbeddable
