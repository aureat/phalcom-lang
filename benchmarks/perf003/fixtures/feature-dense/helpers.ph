trait Iterable { type Item }

class Helpers {
  @class snapshot<J>(_ value: J) -> Float where J |= Iterable<Item = Float> { 0.0 }
  @class local<I>(_ input: I) -> Float where I |= Iterable<Item = Float> {
    let family = &Helpers.snapshot(_)
    let values = family(input)
    values
  }
}

let snapshot = &Helpers.snapshot(_)
class Floats { @constructor new() {} }
impl Iterable for Floats { type Item = Float }
export snapshot, Iterable, Floats, Helpers
