# Wren vs Phalcom: fiber yield at native boundary

Status: recorded 2026-07-21

Baseline:
- Phalcom HEAD: current workspace snapshot on 2026-07-21
- External artifact: Wren source snapshot under `resources/wren/src/vm`

## Finding

Wren does **not** implement fiber switch as an opcode. The switch happens in the C primitive `fiber_yield`, which rewires `vm->fiber` to `current->caller` ([`wren_core.c:209-224`](../../../resources/wren/src/vm/wren_core.c#L209-L224)).

In Phalcom, `Fiber.yield` is also not an opcode, but the switch path is different: it is a primitive send that performs an O(1) handoff through `store_live_into` / `load_live_from`, and it is guarded by `native_reentry_depth` so yield across native re-entrant frames raises `CannotYieldAcrossNativeFrame` ([`fiber.rs:415-432`](../../../phalcom-core/src/primitive/fiber.rs#L415-L432)).

## First-hand citations

- Wren callback model in plain Wren source: [`wren_core.wren:7-53`](../../../resources/wren/src/vm/wren_core.wren#L7-L53)
- Wren `Sequence.each` / `reduce` / `map` call blocks directly in Wren code: [`wren_core.wren:49-95`](../../../resources/wren/src/vm/wren_core.wren#L49-L95)
- Wren fiber transfer setup in C: [`wren_core.c:86-137`](../../../resources/wren/src/vm/wren_core.c#L86-L137)
- Wren yield switch site in C: [`wren_core.c:209-249`](../../../resources/wren/src/vm/wren_core.c#L209-L249)
- Wren foreign-method boundary and non-reentry guard: [`wren_core.c:384-397`](../../../resources/wren/src/vm/wren_core.c#L384-L397) and [`wren_core.c:1442-1472`](../../../resources/wren/src/vm/wren_core.c#L1442-L1472)
- Wren foreign/native API stack boundary: [`wren_vm.h:93-100`](../../../resources/wren/src/vm/wren_vm.h#L93-L100)
- Wren fiber state layout and caller semantics: [`wren_value.h:298-355`](../../../resources/wren/src/vm/wren_value.h#L298-L355)
- Phalcom yield guard and handoff: [`fiber.rs:415-432`](../../../phalcom-core/src/primitive/fiber.rs#L415-L432) and [`fiber.rs:473-479`](../../../phalcom-core/src/primitive/fiber.rs#L473-L479)
- Phalcom restricted-yield spec: [`concurrency.md:113-153`](../../../docs/spec/current/concurrency.md#L113-L153)
- Phalcom fiber ADR: [`0030-fibers-and-futures-cooperative-concurrency.md:21-34`](../../../docs/adr/accepted/0030-fibers-and-futures-cooperative-concurrency.md#L21-L34)

## Interpretation

The earlier statement that Wren yield was an opcode was wrong. Wren’s yield is a primitive-level fiber switch, and the only hard boundary shown here is the foreign/native callback boundary.

Phalcom follows the same broad shape at the surface, but its VM implementation adds a stronger guard because native re-entrant Rust frames can sit between the fiber entry and the yield site. That makes some generator shapes illegal today even though the switch itself remains constant-time.

## Consequence

If Phalcom wants `.each { Fiber.yield(x) }` to work, the fix is not “add an opcode for yield.” The fix is to remove native recursion from callback paths or trampoline them so the fiber switch can occur without a live native frame in between.
