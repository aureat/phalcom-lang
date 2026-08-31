// area: collections
// spec: collection spread Part I §5.2
// status: NEGATIVE
// Lists have only a positional lane. Labeled expansion is rejected at the
// operator token rather than reaching compiler lowering.

const l = [**xs]
