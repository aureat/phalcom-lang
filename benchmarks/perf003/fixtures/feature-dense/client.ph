from .facade import snapshot
from .helpers import Iterable
from .helpers import Floats, Helpers
import .helpers as helpers

class Client {
  @class run<I>(_ input: I) -> Float where I |= Iterable<Item = Float> {
    let local = snapshot
    let a = local(input)
    let b = helpers.snapshot(input)
    let c = snapshot(input)
    c
  }
}

let concrete = Floats.new()
let direct = Helpers.snapshot(concrete)
let localFamily = &Helpers.snapshot(_)
let localResult = localFamily(concrete)
let importedResult = snapshot(concrete)
export Client, concrete, direct, localResult, importedResult
