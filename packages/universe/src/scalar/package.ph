@!documentation("Primitive scalar types: Number, Int, Float, String, Char, Bool, and Symbol.")

expose .number
expose .int
expose .float
expose .nil
expose .string
expose .string_index
expose .char
expose .bool
expose .symbol

from .string_index import StringIndex
from .number import Number
from .int import Int
from .float import Float

export StringIndex
export Number
export Int
export Float
