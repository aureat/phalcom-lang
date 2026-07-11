// area: errors
// spec: error-handling.md; object-model.md
// status: PENDING

try {
  throw ArgumentError("age must be >= 0")
} catch (e: ArgumentError) {
  System.print(e.message)
} finally {
  System.print("done")
}
