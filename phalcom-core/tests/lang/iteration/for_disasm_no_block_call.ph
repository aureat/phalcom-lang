// C-ITER-4 (the §7.1 preclusion guard, D-ITER-2): a `for` body lowers to a
// direct jump loop with no materialized block / `block_call` on the taken
// path. This fixture is disassembled by `iteration_disasm`; it also runs as an
// ordinary pass case.
for (x in [1, 2]) { System.print(x) }
