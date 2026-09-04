class RowPatternProbe {
    @class
    inspect<R: RecordRow>(_ value: #{ known: Int, | R }) -> Int {
        match value {
            #{ known: item } => item
            #{ missing: other } => 0
            _ => -1
        }
    }
}
