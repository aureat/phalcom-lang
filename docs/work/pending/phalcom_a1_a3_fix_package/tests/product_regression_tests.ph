# A1-A3 product regression tests

# Empty products normalize to Unit
assert(() == ())

# Nested tuple construction
assert(((1, 2), #{a: 3}) != None)

# Nested record construction
assert(#{a: (1, 2), b: #{c: 3}} != None)

# Labeled tuple construction
assert((a: 1) != None)
