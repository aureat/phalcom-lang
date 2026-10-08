/// Trait for types capable of receiving streaming text output.
trait TextWriter {
  write(_ text: String) -> Unit
  writeChar(_ char: Char) -> Unit
}

export TextWriter
