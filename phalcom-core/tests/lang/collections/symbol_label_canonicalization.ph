// A1-A3 product regression: equivalent static labels intern to one Symbol.

const tuple = (field: 1)
const record = #{field: 1}

System.print(tuple.labelAt_(0) == #field)
System.print(record.labelAt_(0) == #field)
System.print(tuple.labelAt_(0) == record.labelAt_(0))
