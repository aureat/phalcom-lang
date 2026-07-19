// Ported from wren/test/benchmark/string_equals.wren. `1..1000000` range-in-for
// replaced with a while-counter. Everything else ports directly (string `==`,
// cross-type compare).
let count = 0
let i = 1
while (i <= 1000000) {
  if ("abc" == "abc") { count = count + 1 }
  if ("a slightly longer string" ==
      "a slightly longer string") { count = count + 1 }
  if ("a significantly longer string but still not overwhelmingly long string" ==
      "a significantly longer string but still not overwhelmingly long string") { count = count + 1 }

  if ("" == "abc") { count = count + 1 }
  if ("abc" == "abcd") { count = count + 1 }
  if ("changed one character" == "changed !ne character") { count = count + 1 }
  if ("123" == 123) { count = count + 1 }
  if ("a slightly longer string" ==
      "a slightly longer string!") { count = count + 1 }
  if ("a slightly longer string" ==
      "a slightly longer strinh") { count = count + 1 }
  if ("a significantly longer string but still not overwhelmingly long string" ==
      "another") { count = count + 1 }
  i = i + 1
}

System.print(count)
