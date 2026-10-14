from snowflake import SnowflakeLayout, SnowflakeNode, SnowflakeId, SnowflakeGenerationError

let epoch = 1288834974657
let node = SnowflakeNode.new(1)
let first = node.generateAt(epoch)
let second = node.generateAt(epoch)
let third = node.generateAt(epoch + 1)
if first.value != 4096 or second.value != 4097 or third.value != 4198400 {
  throw Error.new("Twitter layout, epoch, or sequence")
}
if first.timeMillis != epoch or first.node != 1 or first.step != 0 {
  throw Error.new("Twitter field decoding")
}
if not (first < second and second < third) { throw Error.new("node ordering") }

let custom = SnowflakeLayout.configured(epochMillis: 0, nodeBits: 2, sequenceBits: 2)
let worker = SnowflakeNode.configured(nodeId: 3, layout: custom)
let encoded = worker.generateAt(3)
if encoded.value != 60 or encoded.node(using: custom) != 3 or
  encoded.timeMillis(using: custom) != 3 or encoded.step(using: custom) != 0 {
  throw Error.new("custom layout")
}
if worker.generateAt(3).value != 61 or worker.generateAt(3).value != 62 or
  worker.generateAt(3).value != 63 { throw Error.new("custom sequence") }
let mut exhausted = false
try {
  worker.generateAt(3)
} catch error {
  if error is not SnowflakeGenerationError { throw error }
  exhausted = true
}
if not exhausted { throw Error.new("sequence exhaustion must raise") }
let next = worker.generateAt(4)
if next.value != 76 { throw Error.new("new millisecond resets sequence") }
let mut rolledBack = false
try {
  worker.generateAt(2)
} catch error {
  if error is not SnowflakeGenerationError { throw error }
  rolledBack = true
}
if not rolledBack or worker.generateAt(4).value != 77 {
  throw Error.new("clock rollback must leave generator state intact")
}

let live = SnowflakeNode.new(2).generate()
if live.node != 2 or live.timeMillis < epoch { throw Error.new("system clock generation") }

let noNodeOrStep = SnowflakeLayout.configured(epochMillis: 0, nodeBits: 0, sequenceBits: 0)
let single = SnowflakeNode.configured(nodeId: 0, layout: noNodeOrStep)
if single.generateAt(1).value != 1 or single.generateAt(2).value != 2 {
  throw Error.new("zero-width node and step")
}

let oldEpoch = SnowflakeLayout.configured(epochMillis: -5, nodeBits: 2, sequenceBits: 2)
let oldNode = SnowflakeNode.configured(nodeId: 1, layout: oldEpoch)
let beforeUnixEpoch = oldNode.generateAt(-3)
if beforeUnixEpoch.value != 36 or beforeUnixEpoch.timeMillis(using: oldEpoch) != -3 or
  oldNode.generateAt(-3).value != 37 {
  throw Error.new("negative custom epoch and initialization")
}

let mut invalidLayout = false
try {
  SnowflakeLayout.configured(epochMillis: 0, nodeBits: 11, sequenceBits: 12)
} catch error {
  if error is not ArgumentError { throw error }
  invalidLayout = true
}
if not invalidLayout { throw Error.new("layout must reject more than 22 allocation bits") }

let mut invalidNode = false
try {
  SnowflakeNode.new(1024)
} catch error {
  if error is not ArgumentError { throw error }
  invalidNode = true
}
if not invalidNode { throw Error.new("default node ID must fit ten bits") }

let endNode = SnowflakeNode.new(1023)
let lastTime = SnowflakeLayout.twitter.epochMillis + SnowflakeLayout.twitter.maxDeltaMillis
let lastId = endNode.generateAt(lastTime)
if lastId.timeMillis != lastTime or lastId.node != 1023 {
  throw Error.new("maximal 41-bit timestamp and ten-bit node")
}
let mut overflow = false
try {
  endNode.generateAt(lastTime + 1)
} catch error {
  if error is not SnowflakeGenerationError { throw error }
  overflow = true
}
if not overflow or endNode.generateAt(lastTime).step != 1 {
  throw Error.new("timestamp overflow must preserve generator state")
}

let ids = Set<SnowflakeId>.new()
ids.add(first)
ids.add(SnowflakeId.fromInt(4096).unwrap)
if ids.size != 1 { throw Error.new("Snowflake equality/hash key") }

System.print("PASS snowflake-layout")
