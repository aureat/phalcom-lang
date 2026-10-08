@!documentation("Text encoding, formatting, and string manipulation utilities.")

expose .substring
expose .traits
expose .patterns

from .substring import (
  Substring,
  SubstringChars,
  SubstringCharIterator,
  SubstringBytes,
  SubstringByteIterator
)
from .traits import Text
from .patterns import TextPattern

export (
  Substring,
  SubstringChars,
  SubstringCharIterator,
  SubstringBytes,
  SubstringByteIterator,
  Text,
  TextPattern
)
