let mut original = { name: "Phalcom", value: 42, cached: true }
let mut preserved = RowCalculus.preserve(original)
let mut tagged = RowCalculus.tagged(preserved)
let mut consumed = RowCalculus.consumeTagged(tagged)

let mut runtimePreservedValue = match consumed {
    { value: val } => val
    _ => 0
}

let mut runtimeTagIsEntity = match consumed {
    { tag: t } => t == "entity"
    _ => false
}

let mut runtimeCachedField = match consumed {
    { cached: c } => c
    _ => false
}
