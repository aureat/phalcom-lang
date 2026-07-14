// area: sequence
// spec: iteration.md §5; ADR-0035
// status: PASS
// find(f) returns None when no element matches

var result = [1, 2, 3].find { x => x == 10 }
System.print(result == None)
