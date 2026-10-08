@!documentation("Explicit numeric values and independent closed arithmetic capabilities. Complex<T> preserves component types and exact applied identity.")

from .capabilities import Addable, Subtractable, Multipliable, Negatable, IntegerPower
from .algebra import I, J, K, Basis, i, j, k
from .complex import Complex
from .quaternion import Quaternion

export Complex, Quaternion, I, J, K, Basis, i, j, k, Addable, Subtractable, Multipliable, Negatable, IntegerPower
