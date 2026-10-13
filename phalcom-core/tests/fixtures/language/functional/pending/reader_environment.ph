// area: functional
// status: PENDING
from universe.functional import Reader

data Config(_ scale: Int, _ offset: Int)
let mut calls = 0
let scale = Reader<Config, Int>.new(|config| {
  calls = calls + 1
  config.scale
})
let offset = Reader<Config, Int>.new(|config| { config.offset })
let pipeline = scale.flatMap(|factor| { offset.map(|shift| { factor * 10 + shift }) })
let config = Config(2, 3)
System.print(calls)
System.print(pipeline.run(config))
System.print(calls)
System.print(pipeline.local(|c| { Config(c.scale + 1, c.offset) }).run(config))
System.print(config.scale)
System.print(pipeline.contramap<Int>(|n| { Config(n, 1) }).run(4))
System.print(scale.zipWith(offset, combine: |a, b| { a + b }).run(config))
System.print(Reader<Config, Config>.ask().run(config).offset)
System.print(Reader<Config, String>.pure("constant").run(config))
