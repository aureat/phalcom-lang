// C-ITER-6 (spec §4, §8.3), depth-3 case: `break` inside the INNERMOST of
// three nested `for` loops leaves only that innermost loop; the middle and
// outer loops are unaffected. Over a=[1,2], b=[1,2], c=[1,2,3]: the
// innermost breaks at c==2 every time, so c==3 never prints, but middle/outer
// still run their full ranges.
for a in [1, 2] {
  for b in [1, 2] {
    for c in [1, 2, 3] {
      (c == 2).ifTrue || { break }
      System.print(a)
      System.print(b)
      System.print(c)
    }
  }
}
