// A1-A3 product regression: zero normalization and recursive construction.

const emptyTuple = ()
const emptyRecord = #{}
System.print(emptyTuple.class == Unit)
System.print(emptyRecord.class == Unit)
System.print(emptyTuple == emptyRecord)

const nestedTuple = ((1, 2), #{a: 3})
const nestedRecord = #{a: (1, 2), b: #{c: 3}}
const labeledNestedTuple = ((a: 1),)

System.print(nestedTuple == ((1, 2), #{a: 3}))
System.print(nestedRecord == #{b: #{c: 3}, a: (1, 2)})
System.print(labeledNestedTuple == ((a: 1),))
