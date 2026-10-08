@!documentation("Primitive scalar types: Number, String, Char, Bool, Symbol.")

expose .number
expose .nil
expose .string
expose .string_index
expose .char
expose .bool
expose .symbol

from .string_index import StringIndex

export StringIndex
