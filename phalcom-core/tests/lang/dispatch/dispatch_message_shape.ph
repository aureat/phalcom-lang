// area: dispatch
// spec: method-lookup.md §2; ADR-0012
// status: PASS
// Inside a `doesNotUnderstand(_:)` override, the reified `Message` exposes the
// missed send's `selector`/`name`/`labels`/`args`. A fully-labeled call
// (`move(to:duration:)`) verifies the selector-decoder (encoder inverse):
// `name` is the bare method name, `labels` are the keyword labels
// index-aligned with `args`.

class Recorder {
  doesNotUnderstand(msg) {
    System.print(msg.selector)
    System.print(msg.name)
    System.print(msg.labels.toString)
    System.print(msg.args.toString)
    return None
  }
}
const r = Recorder.new()
r.move(to: 1, duration: 2)
