# Transition 1 isolated probes

This directory holds small, new-syntax `.ph` probes created while Transition 1
is in flight. It is intentionally outside `tests/lang`: the active language
corpus remains a legacy regression corpus until its declaration syntax is
migrated in Task 2.

Every probe has a sibling `.expected` file. Run one probe with:

```bash
scripts/test-transition-1.sh probe task-02/setter_and_subscript
```

For a diagnostic probe, add `--negative`; its `.expected` file must contain the
diagnostic substring. Move a probe into the ordinary corpus only when the
relevant migration task moves its surrounding corpus to canonical syntax.
