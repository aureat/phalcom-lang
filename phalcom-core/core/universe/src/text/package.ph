@!documentation("Text encoding, formatting, and string manipulation utilities.")

expose .substring
expose .traits
expose .patterns
expose .split
expose .lines
expose .writer
expose .builder

from .substring import (
  Substring,
  SubstringChars,
  SubstringCharIterator,
  SubstringBytes,
  SubstringByteIterator
)
from .traits import Text
from .patterns import TextPattern
from .split import (
  Split,
  SplitIterator,
  SplitWhitespace,
  SplitWhitespaceIterator
)
from .lines import (
  Lines,
  LinesIterator
)
from .writer import TextWriter
from .builder import StringBuilder

export (
  Substring,
  SubstringChars,
  SubstringCharIterator,
  SubstringBytes,
  SubstringByteIterator,
  Text,
  TextPattern,
  Split,
  SplitIterator,
  SplitWhitespace,
  SplitWhitespaceIterator,
  Lines,
  LinesIterator,
  TextWriter,
  StringBuilder
)
