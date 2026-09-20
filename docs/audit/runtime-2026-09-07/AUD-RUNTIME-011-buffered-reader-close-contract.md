# AUD-RUNTIME-011 - BufferedReader close does not close its inner reader

## Classification

- Severity: Medium
- Category: Streams-v2 lifecycle
- Confidence: Confirmed source contract gap and public execution failure.

## Observed

`BufferedReader` registers its own Resource handle and stores `_inner`, but it
does not override `close`. It inherits `Resource.close`, which closes only its
own handle. The stream protocol requires wrapper close to close the inner
reader.

## Executed evidence

The public probe
`BytesReader.new(Bytes.fromString("abc")) -> BufferedReader.new -> close`
returned `Result::Ok(None)`, then reported `inner.isClosed == false` and
`wrapper.isClosed == true`; the process also reported an unclosed
`BytesReader` resource.

## Reproducer and repair direction

Run `inner = BytesReader.new(...); wrapper = BufferedReader.new(inner);
wrapper.close; inner.isClosed`, expecting `true`. The canonical repair belongs
in the `.ph` wrapper lifecycle, with idempotent ordering preserved; do not
alter ResourceTable ownership to hide the missing inner close.
