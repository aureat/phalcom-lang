from universe.random import Xoshiro256PlusPlus, UniformInt, Bernoulli
from universe.random import Generator, Distribution, SecureRandom, EntropyError
import universe.random as random

let rng = Xoshiro256PlusPlus.seed(42)
System.print(rng.nextU64)
System.print(rng.nextBool)
System.print(rng.nextFloat < 1.0)
System.print(rng.nextInt(below: 1))
System.print(rng.nextInt(below: 10) >= 0)
System.print(rng.bytes(3).size)
let values: List<Int> = [1, 2, 3, 4]
rng.shuffle(values)
System.print(values.size)
System.print(rng.choose(values).isSome)
System.print(UniformInt.from(1, below: 7).sample(using: rng) >= 1)
System.print(Bernoulli.new(0.0).sample(using: rng))

class Check {
  @class
  ok(_ condition: Bool) -> Unit {
    if not condition { Error.new("random acceptance assertion failed").raise() }
    ()
  }
  @class
  argument(_ action) -> Unit {
    let mut caught = false
    try { action.call() } on ArgumentError error { caught = true }
    Check.ok(caught)
    ()
  }
}

class Sequence {
  mut _words: List<Int>
  mut _count: Int
  @constructor
  new(_ words: List<Int>) { _words = words; _count = 0 }
  count -> Int { _count }
  nextU64 -> Int {
    let value = if _words.size == 0 { 0 } else { _words.at(_count % _words.size) }
    _count = _count + 1
    value
  }
}
impl Generator for Sequence {}

class Vectors {
  @class
  check(_ seed: Int, _ expected: List<Int>) -> Unit {
    let generator = Xoshiro256PlusPlus.seed(seed)
    let mut index = 0
    while index < expected.size {
      let word = generator.nextU64
      Check.ok(word == expected.at(index))
      Check.ok(word >= 0)
      Check.ok(word < (1 << 64))
      index = index + 1
    }
    ()
  }
}

// Frozen independently from the published Vigna/Blackman reference algorithms.
Vectors.check(0, [5987356902031041503, 7051070477665621255, 6633766593972829180, 211316841551650330, 9136120204379184874, 379361710973160858, 15813423377499357806, 15596884590815070553, 5439680534584881407, 1369371744833522710, 5801907570356623720, 1218794611651792380])
Vectors.check(1, [14971601782005023387, 13781649495232077965, 1847458086238483744, 13765271635752736470, 3406718355780431780, 10892412867582108485, 18204613561675945223, 9655336933892813345, 1781989159761824720, 2477283028068920342, 16978024111547606601, 6336475467619303347])
Vectors.check(42, [15021278609987233951, 5881210131331364753, 18149643915985481100, 12933668939759105464, 14637574242682825331, 10848501901068131965, 2312344417745909078, 11162538943635311430, 3831705504650218695, 17217215411128672468, 10321681451779520834, 15680282660304795149])
Vectors.check(18446744073709551615, [6254647548650071986, 16610832622747802512, 16422857234328439435, 5048281510058307187, 12093889312535503841, 7417986222439541780, 16304073528878514024, 8976797394443910655, 11710966923038033957, 15754321882623561431, 6308105270406205151, 11873992268502047188])

// Mutable generator state and its LargeInt fields remain live across collection.
let gcRng = Xoshiro256PlusPlus.seed(42)
Check.ok(gcRng.nextU64 == 15021278609987233951)
System.gc
Check.ok(gcRng.nextU64 == 5881210131331364753)
System.gc
Check.ok(gcRng.nextU64 == 18149643915985481100)

Check.argument(|| { Xoshiro256PlusPlus.seed(-1) })
Check.argument(|| { Xoshiro256PlusPlus.seed(1 << 64) })
Check.argument(|| { Xoshiro256PlusPlus.seed(10 ** 100) })
let scripted = Sequence.new([0, 1, 9223372036854775808, 18446744073709551615])
Check.ok(not scripted.nextBool)
Check.ok(not scripted.nextBool)
Check.ok(scripted.nextBool)
Check.ok(scripted.nextBool)
Check.ok(scripted.count == 4)
let floating = Sequence.new([0, 1, 9223372036854775808, 18446744073709551615])
Check.ok(floating.nextFloat == 0.0)
Check.ok(floating.nextFloat == 0.0)
Check.ok(floating.nextFloat == 0.5)
Check.ok(floating.nextFloat == 0.9999999999999999)
Check.ok(floating.count == 4)

let rejecting = Sequence.new([7, 6, 5, 1])
Check.ok(rejecting.nextInt(below: 5) == 1)
Check.ok(rejecting.count == 4)
let zero = Sequence.new([])
Check.ok(zero.nextInt(below: 1) == 0)
Check.ok(zero.count == 0)
Check.argument(|| { zero.nextInt(below: 0) })
Check.argument(|| { zero.nextInt(below: -1) })
Check.argument(|| { zero.nextInt(from: 2, below: 2) })
let bounds: List<Int> = [2, 3, 7, 8, 9, (1 << 32) - 1, 1 << 32, (1 << 32) + 1, (1 << 64) - 1, 1 << 64, (1 << 64) + 1, 10 ** 100, 10 ** 1000]
let mut index = 0
while index < bounds.size {
  let bound = bounds.at(index)
  let sample = rng.nextInt(below: bound)
  Check.ok(sample >= 0)
  Check.ok(sample < bound)
  index = index + 1
}
Check.ok(zero.nextInt(from: -10, below: -2) == -10)
Check.ok(zero.nextInt(from: 3, below: 10) == 3)
Check.ok(zero.nextInt(from: -10, below: 10) == -10)
Check.ok(zero.nextInt(from: -(10 ** 1000), below: 10 ** 1000) == -(10 ** 1000))
let twoWords = Sequence.new([0xffffffffffffffff, 17])
Check.ok(twoWords.nextInt(below: 1 << 65) == (1 << 64) + 17)
Check.ok(twoWords.count == 2)

let counts: List<Int> = [0, 1, 7, 8, 9, 15, 16, 17]
index = 0
while index < counts.size {
  let generator = Sequence.new([0x0123456789abcdef])
  let bytes = generator.bytes(counts.at(index))
  Check.ok(generator.count == (bytes.size + 7) ~/ 8)
  let mut offset = 0
  while offset < bytes.size {
    Check.ok(bytes.at(offset) == ((0x0123456789abcdef >> (56 - (offset % 8) * 8)) & 255))
    offset = offset + 1
  }
  index = index + 1
}
let partial = Sequence.new([0x0123456789abcdef, 0xfedcba9876543210])
Check.ok(partial.bytes(1).at(0) == 1)
Check.ok(partial.nextU64 == 0xfedcba9876543210)
let existing = Bytes.new(9)
Check.ok(partial.fill(existing) === ())
Check.argument(|| { zero.bytes(-1) })

let empty = List<Int>.new()
let before = zero.count
Check.ok(zero.choose(empty).isNone)
Check.ok(zero.choose([7]).unwrapOr(-1) == 7)
zero.shuffle(empty)
zero.shuffle([7])
Check.ok(zero.count == before)
let last = Sequence.new([2])
Check.ok(last.choose([10, 11, 12]).unwrapOr(-1) == 12)
class User {}
let user = User.new()
let users = List<User>.new()
users.append(user)
let picked: Option<User> = zero.choose(users)
Check.ok(picked.unwrapOr(User.new()) === user)
let cards: List<Int> = [0, 1, 2, 3, 4]
Xoshiro256PlusPlus.seed(42).shuffle(cards)
Check.ok(cards.at(0) == 2 and cards.at(1) == 4 and cards.at(2) == 3 and cards.at(3) == 0 and cards.at(4) == 1)
let duplicates: List<Int> = [1, 1, 2, 2, 2]
zero.shuffle(duplicates)
Check.ok(duplicates.size == 5)
let mut sum = 0
index = 0
while index < duplicates.size { sum = sum + duplicates.at(index); index = index + 1 }
Check.ok(sum == 8)

let die = UniformInt.from(-10, below: 10)
let integer: Int = die.sample(using: zero)
Check.ok(integer == -10)
Check.ok(UniformInt.from(0, below: 1).sample(using: zero) == 0)
Check.argument(|| { UniformInt.from(1, below: 1) })
Check.argument(|| { UniformInt.from(2, below: 1) })
let beforeCoin = zero.count
let no: Bool = Bernoulli.new(0.0).sample(using: zero)
let yes: Bool = Bernoulli.new(1.0).sample(using: zero)
Check.ok(not no and yes and zero.count == beforeCoin)
Check.ok(Bernoulli.new(0.5).sample(using: zero))
Check.ok(zero.count == beforeCoin + 1)
Check.argument(|| { Bernoulli.new(-0.1) })
Check.argument(|| { Bernoulli.new(1.1) })
Check.argument(|| { Bernoulli.new(0.0 / 0.0) })

let replay = Xoshiro256PlusPlus.seed(42)
Check.ok(replay.nextBool)
Check.ok(replay.nextFloat == Float.new(2871684634439142) / 9007199254740992.0)
Check.ok(replay.nextInt(below: 10) == 8)
let replayBytes = replay.bytes(3)
Check.ok(replayBytes.at(0) == 0xcb and replayBytes.at(1) == 0x23 and replayBytes.at(2) == 0x1c)
Check.ok(replay.nextU64 == 10848501901068131965)
let qualified = random.Xoshiro256PlusPlus.seed(42)
Check.ok(qualified.nextBool)

let secure = SecureRandom.system
Check.ok(secure === SecureRandom.system)
Check.ok(secure.bytes(0).size == 0)
Check.ok(secure.bytes(1).size == 1)
Check.ok(secure.bytes(32).size == 32)
Check.ok(secure.fill(Bytes.new(32)) === ())
Check.argument(|| { secure.bytes(-1) })
System.print("random acceptance passed")
