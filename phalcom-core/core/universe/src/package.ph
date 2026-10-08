@!documentation("Primordial language and object universe for Phalcom.")

expose .object
expose .scalar
expose .errors
expose .callable
expose .option
expose .time
expose .concurrency
expose .collections
expose .reflection
expose .io
expose .fs
expose .path
expose .text
expose .regex
expose .json
expose .math
expose .random
expose .process
expose .net
expose .concurrent
expose .testing

import .object
import .scalar
from .scalar import StringIndex
import .errors
from .errors import (
  ArgumentError,
  KeyError,
  IndexError,
  SliceError,
  DuplicateKeyError
)
import .callable
import .option
import .time
from .time import (
  Duration,
  Timestamp
)
import .concurrency
import .collections
import .reflection
import .io
import .fs
import .path
import .text
from .text import (
  Substring,
  Text,
  TextPattern
)
import .regex
import .json
import .math
import .random
import .process
import .net
import .concurrent
import .testing

export object
export scalar
export StringIndex
export Substring
export Text
export TextPattern
export errors
export (
  ArgumentError,
  KeyError,
  IndexError,
  SliceError,
  DuplicateKeyError
)
export callable
export option
export (
  Duration,
  Timestamp
)
export time
export concurrency
export collections
export reflection
export io
export fs
export path
export text
export regex
export json
export math
export random
export process
export net
export concurrent
export testing
