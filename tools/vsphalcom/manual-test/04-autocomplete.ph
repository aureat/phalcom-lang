// 04-autocomplete.ph
//
// Manual steps — place the cursor where marked and type, watch the
// completion popup:
//
// 1. On the blank line below "KEYWORD TEST", type `cla` — "class" should
//    appear in the completion list (keyword completion, filtered by prefix).
//
// KEYWORD TEST:


// 2. On the blank line below "SELECTOR TEST", type `self.` — the full
//    flat core-selector list should appear (no receiver-type narrowing is
//    expected — this is honest scope, not a bug). Look for `isA(_)` in the
//    list; accept it and confirm the snippet inserts `isA(${1:_})` with a
//    live tab-stop over `_`.
//
// SELECTOR TEST:
class Probe {
  check(obj) {

  }
}

// 3. On the blank line below "MOVE TEST", type `.` after `self` and look
//    for a multi-label selector (e.g. any selector with more than one
//    comma-separated slot in tools/vsphalcom/src/generated/core-table.json
//    — grep the file for a class with 2+ params if none obviously appears
//    here) to confirm keyword-labeled tab-stops render as `label: ${n:_}`,
//    not just `${n:_}`.
//
// MOVE TEST:
class Probe2 {
  check2(obj) {

  }
}
