// area: decorators
// spec: decorators-behavioral.md B-2; decorators-dispatch-observability.md D-2/D-3
// status: PASS
// contract: Tracer/OffBehavior/Backoff ship as standalone core classes ahead
// of the Install/Dispatch/Runtime decorator mechanism itself (ADR-0054).
// Backoff.fixed/.exponential raise until System.sleep(_) lands (system.md).

Tracer.stdout.enter("deposit", [100])
Tracer.stdout.exit("deposit", 100, None)

let ob = OffBehavior.fallback(#cachedPrice)
System.print(ob.kind)
System.print(ob.payload.isSome)

System.print(Backoff.none.waitBefore(1))

try {
  Backoff.fixed(50).waitBefore(1)
} catch e {
  System.print(e.message)
}
