@!documentation("Primordial language and object universe for Phalcom.")

expose .object
expose .scalar
expose .errors
expose .result
expose .callable
expose .option
expose .functional
expose .time
expose .concurrency
expose .diagnostics
expose .features
expose .collections
expose .reflection
expose .io
expose .fs
expose .path
expose .text
expose .regex
expose .json
expose .math
expose .numbers
expose .random
expose .process
expose .net
expose .testing

import .object
import .scalar
from .scalar import StringIndex
import .errors
import .result
from .errors import (
  ArgumentError,
  KeyError,
  IndexError,
  SliceError,
  DuplicateKeyError
)
import .callable
import .option
import .functional
import .time
from .time import (
  Duration,
  Timestamp
)
import .concurrency
import .diagnostics
import .features
import .collections
import .reflection
import .io
import .fs
import .path
import .text
from .text import (
  Substring,
  Text,
  TextPattern,
  Split,
  SplitWhitespace,
  Lines,
  TextWriter,
  StringBuilder,
  TextCursor,
  SourceLocation,
  LineMap,
  Utf8Error,
  IntParseError,
  FloatParseError,
  BoolParseError
)
import .regex
import .json
import .math
import .numbers
import .random
import .process
import .net
import .testing

export object
export scalar
export StringIndex
export Substring
export Text
export TextPattern
export Split
export SplitWhitespace
export Lines
export TextWriter
export StringBuilder
export TextCursor
export SourceLocation
export LineMap
export Utf8Error
export IntParseError
export FloatParseError
export BoolParseError
export errors
export result
export (
  ArgumentError,
  KeyError,
  IndexError,
  SliceError,
  DuplicateKeyError
)
export callable
export option
export functional
export (
  Duration,
  Timestamp
)
export time
export concurrency
export diagnostics
export features
export collections
export reflection
export io
export fs
export path
export text
export regex
export json
export math
export numbers
export random
export process
export net
export testing
