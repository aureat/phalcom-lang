// A1-A3 product regression: zero normalization and recursive construction.

let emptyTuple = ()
let emptyRecord = {}
System.print(emptyTuple.class == Tuple)
System.print(emptyRecord.class == Record)
System.print(emptyTuple != emptyRecord)

let nestedTuple = ((1, 2), {a: 3})
let nestedRecord = {a: (1, 2), b: {c: 3}}
let labeledNestedTuple = ((a: 1),)

System.print(nestedTuple == ((1, 2), {a: 3}))
System.print(nestedRecord == {b: {c: 3}, a: (1, 2)})
System.print(labeledNestedTuple == ((a: 1),))
