# Float, text, and numeric errors

> Sources: numeric conformance specification; STDL001
> Raw: [standard-library specification and program snapshot](../raw/standard-library/2026-09-08-standard-library-sources.md)
> Updated: 2026-09-08

Float/text behavior includes binary64 special values, canonical rendering thresholds, equality/hash consistency, and error products for invalid numeric operations. Text rendering must not expose private representation tiers such as a future LargeInt arm.

Numeric errors are semantic/runtime products rendered through [diagnostic reports](../diagnostics/report-model.md). STDL001 is in progress and partial.
