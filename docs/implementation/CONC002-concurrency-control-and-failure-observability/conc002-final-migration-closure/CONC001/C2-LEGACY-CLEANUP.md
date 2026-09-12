# CONC001 C2 legacy cleanup

Current tree contains both:

```text
C2-scheduler-and-reactor/
C2-scheduler/
```

The second is the current historical closure record.

After live-link migration:

1. retain `C2-scheduler/`;
2. remove/archive `C2-scheduler-and-reactor/`;
3. ensure no file under the old directory remains executable `PROPOSED`;
4. reactor implementation authority is CONC002.C3;
5. current scheduler semantics are CONC002.C1/C2.

Historical wiki/raw references may remain.
