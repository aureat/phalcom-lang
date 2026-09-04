class RowCapabilities {
    @class
    port<R: RecordRow>(_ environment: #{ config: #{ port: Int }, | R }) -> Int {
        match environment {
            #{ config: c } => match c {
                #{ port: p } => p
                _ => 0
            }
            _ => 0
        }
    }

    @class
    host<R: RecordRow>(_ environment: #{ config: #{ host: String }, | R }) -> String {
        match environment {
            #{ config: c } => match c {
                #{ host: h } => h
                _ => ""
            }
            _ => ""
        }
    }
}

class RowStructuralProbe {
    @class
    widthAndDepth() {
        let env = #{ config: #{ port: 8080, host: "localhost", debug: true }, cache: "redis", requestId: "req-123" }
        let port = RowCapabilities.port(env)
        let host = RowCapabilities.host(env)
        port
    }

    @class
    widthOnly() {
        let env = #{ config: #{ port: 9000 }, extra: "value" }
        RowCapabilities.port(env)
    }
}
