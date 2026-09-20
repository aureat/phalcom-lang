# AUD-RUNTIME-009 - Bytes UTF-8 declaration contract mismatch

## Classification

- Severity: Medium
- Category: Native surface / library declaration / static contract
- Confidence: Confirmed
- Status: Implemented and focused-tested; runtime algorithm and published
  declaration now follow the normative `String | None` contract.

## Observed

`phalcom-core/core/universe/src/collections/bytes.ph:20` declares the internal
native selector `rawUtf8 -> Bytes`, while the public `utf8` getter forwards
directly to it. The native implementation at
`phalcom-core/src/primitive/bytes.rs:239-248` returns a heap `String` for valid
UTF-8 and kernel `None` for invalid bytes. Generated native metadata leaves the
return type unknown, so native-surface validation does not detect the mismatch.

## Evidence

The existing `bytes_strings.ph` fixture executed with output showing valid
decoding (`Hi`, `héllo`) and invalid decoding (`None`). Direct runtime probes
also produced `Bytes.fromString("Hi").utf8.class.name == String` and
`Bytes.fromList([255]).utf8.class.name == None`.

The effective normative bytes contract specifies `String | None`, matching the
native body and contradicting only the stale `.ph` declaration.

## Resolution

The `.ph` native declaration now publishes `Option<String>`, matching the
unchanged native decoder and the effective standard-library contract. Runtime
probes continue to return `String` for valid UTF-8 and `None` for invalid
bytes. No decoder or `Dynamic` fallback was introduced.
