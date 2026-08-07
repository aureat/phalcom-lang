# Record equality/hash regression

let first = #{a: 1, b: 2}
let second = #{b: 2, a: 1}

assert(first == second)
assert(first.hash == second.hash)
