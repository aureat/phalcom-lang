class NominalConfig {
    @constructor
    new(_ port: Int) {}
}

class RowNominalInvalidProbe {
    @class
    run(_ config: NominalConfig) {
        RowCapabilities.port(config)
    }
}
