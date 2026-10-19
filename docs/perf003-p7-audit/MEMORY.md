# PERF003.C2.P7 memory and allocation evidence

## Allocation accounting

Measured implementation revision: `f2550f7193ae776f78e58d1ed187a3ae6c7efdd2`.

- Whole-load requested bytes, N: **373,069,634**.
- Independently observed allocator requests: **373,069,634**.
- Signed delta: **0**.
- N−1 returns typed `BudgetExceeded` without publishing a partial image.
- Exact-N retry succeeds.
- Total allocation transaction limit remains 512 MiB; decoded section limit is 320 MiB.

The focused core allocation log is `logs/core-artifact-allocation.txt`; the command and exact boundary are in [COMMANDS.md](COMMANDS.md).

## Peak RSS observations

Peak RSS is the process high-water mark. The cold and warm datasets report it as a measured process metric, not retained memory.

- Cold source runs have per-workload median peak RSS values from 809,208 to 822,416 KB.
- Cold artifact runs have per-workload median peak RSS values from 324,736 to 349,912 KB.
- Warm source process peak RSS: 828,112,896 bytes.
- Warm artifact process peak RSS: 334,823,424 bytes.
- Warm modes used separate child processes.

These values do not prove leak behavior or current retained memory. No leak test was run; those claims remain **UNVERIFIED**.
