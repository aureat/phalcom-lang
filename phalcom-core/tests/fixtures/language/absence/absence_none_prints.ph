// area: absence
// spec: values-and-absence.md; ADR-0007
// status: PASS
// U6: `None` is a global bound to the shared singleton. U-CORE-4 gives
// `Option#toString` a real `.ph` render ("None"), and the native print path
// (`Value::to_string`) agrees, so this fixture now pins the final surface,
// not a substrate placeholder.

System.print(None)
