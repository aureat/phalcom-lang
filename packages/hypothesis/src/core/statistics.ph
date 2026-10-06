/// Counters captured at the end of a property run.
data Statistics(
  generatedExamples: Int,
  shrinkEvaluations: Int,
  acceptedShrinks: Int,
  discards: Int,
  verificationReplays: Int,
)

export Statistics
