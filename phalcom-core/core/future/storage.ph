type data Storage {
    Inline(_ bytes: Int)
    Heap
    External
}

type data Device {
    CPU
    GPU(_ index: Int)
}

type data Layout {
    Dense
    Strided
}
