@!documentation("First-class dispatch selector representation.")

class Selector {
    init(name, kind, positionalCount, labels) {
        _name = name
        _kind = kind
        _positionalCount = positionalCount
        _labels = labels
    }

    name() { _name }
    kind() { _kind }
    positionalCount() { _positionalCount }
    labels() { _labels }
}
