# Wren vs Phalcom: fiber yield at native boundary

Status: recorded 2026-07-21

Baseline:
- Phalcom HEAD: current workspace snapshot on 2026-07-21
- External artifact: Wren source snapshot under `resources/wren/src/vm`

## Finding

Wren does **not** implement fiber switch as an opcode. The switch happens in the C primitive `fiber_yield`, which rewires `vm->fiber` to `current->caller`.

In Phalcom, `Fiber.yield` is also not an opcode, but the switch path is different: it is a primitive send that performs an O(1) handoff through `store_live_into` / `load_live_from`, and it is guarded by `native_reentry_depth` so yield across native re-entrant frames raises `CannotYieldAcrossNativeFrame`.

## First-hand citations

- Wren callback model in plain Wren source: [`resources/wren/src/vm/wren_core.wren:49-52`](../../../resources/wren/src/vm/wren_core.wren#L49)
- Wren fiber transfer setup in C: [`resources/wren/src/vm/wren_core.c:86-137`](../../../resources/wren/src/vm/wren_core.c#L86)
- Wren yield switch site in C: [`resources/wren/src/vm/wren_core.c:209-224`](../../../resources/wren/src/vm/wren_core.c#L209)
- Wren foreign-method boundary and non-reentry guard: [`resources/wren/src/vm/wren_core.c:384-399`](../../../resources/wren/src/vm/wren_core.c#L384) and [`resources/wren/src/vm/wren_core.c:1442`](../../../resources/wren/src/vm/wren_core.c#L1442)
- Phalcom yield guard and handoff: [`phalcom-core/src/primitive/fiber.rs:420-450`](../../../phalcom-core/src/primitive/fiber.rs#L420) and [`phalcom-core/src/primitive/fiber.rs:455-520`](../../../phalcom-core/src/primitive/fiber.rs#L455)
- Phalcom restricted-yield spec: [`docs/spec/current/concurrency.md:113-153`](../../../docs/spec/current/concurrency.md#execution-model--restricted-yield-option-a)
- Phalcom fiber ADR: [`docs/adr/accepted/0030-fibers-and-futures-cooperative-concurrency.md:22-49`](../../../docs/adr/accepted/0030-fibers-and-futures-cooperative-concurrency.md#decision)

## Interpretation

The earlier statement that Wren yield was an opcode was wrong. Wren’s yield is a primitive-level fiber switch, and the only hard boundary shown here is the foreign/native callback boundary.

Phalcom follows the same broad shape at the surface, but its VM implementation adds a stronger guard because native re-entrant Rust frames can sit between the fiber entry and the yield site. That makes some generator shapes illegal today even though the switch itself remains constant-time.

## Consequence

If Phalcom wants `.each { Fiber.yield(x) }` to work, the fix is not “add an opcode for yield.” The fix is to remove native recursion from callback paths or trampoline them so the fiber switch can occur without a live native frame in between.
