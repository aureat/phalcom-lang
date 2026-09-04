let original = #{ name: "Phalcom", value: 42, cached: true }
let preserved = RowCalculus.preserve(original)
let tagged = RowCalculus.tagged(preserved)
let consumed = RowCalculus.consumeTagged(tagged)

let runtimePreservedValue = match consumed {
    #{ value: val } => val
    _ => 0
}

let runtimeTagIsEntity = match consumed {
    #{ tag: t } => t == "entity"
    _ => false
}

let runtimeCachedField = match consumed {
    #{ cached: c } => c
    _ => false
}
