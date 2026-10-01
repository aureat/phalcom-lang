@!documentation("Language and runtime error types.")

expose .error
expose .result
expose .argument
expose .iteration
expose .indexing
expose .contracts
expose .unsupported
expose .unimplemented

from .argument import ArgumentError, KeyError, IndexError, SliceError, DuplicateKeyError
export ArgumentError, KeyError, IndexError, SliceError, DuplicateKeyError
