// U-CLASSCLOSE §4: `None` is bound as a value global (the singleton), not a
// class global (`vm/bootstrap.rs`), but its ClassId still installs via
// `add_class!`-adjacent bootstrap wiring and is reserved the same as any
// other kernel primitive.
class None {}
