# Symbol canonicalization regression

let tuple = (field: 1)
let record = #{field: 1}

assert(tuple.labels[0] === #field)
assert(record.keys[0] === #field)
