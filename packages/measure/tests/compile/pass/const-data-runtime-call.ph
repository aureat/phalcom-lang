data Scale(_ factor: Int)

impl Scale {
  call(_ value: Int) -> Int { value * factor }
}

const thousand = Scale(1000)
let result = thousand(5)
