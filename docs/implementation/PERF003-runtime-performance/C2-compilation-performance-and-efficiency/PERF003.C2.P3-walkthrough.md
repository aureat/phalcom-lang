# PERF003.C2.P3 walkthrough — first increment

Status: IN_PROGRESS / PARTIAL / UNVERIFIED at full P3 scope.

Exact conformance witness lookup now constructs the canonical conformance-owned
CallableId and consults the existing hash table, replacing one global signature
scan per requirement. All downstream compatibility, visibility, generic and body
checks remain in place. Effective inherent surfaces now enumerate the maintained
module index before applying the unchanged declaration-owner filter.

One exact owner/side/clone/removal regression passed, including final module
enumeration assertions. Core compilation with perf-observability passed. Broader
conformance and settlement verification is deferred at the user's request.
No wall-time improvement or complete P3 delivery is claimed.
