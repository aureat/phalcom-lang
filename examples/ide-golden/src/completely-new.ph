class Person {
    const _name: String
    const _age: Int

    @constructor
    new(_ name: String, _ age: Int) {
        _name = name
        _age = age
    }

    setName(_ name: String) {
        _name = name
    }
}
