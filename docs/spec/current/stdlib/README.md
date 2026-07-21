# Standard library specifications

- Status: **Program index.** Not ratified, not an ADR, not a normative spec.
- Date: 2026-07-20
- Scope ratified by the user on 2026-07-20; recorded in
  [`../drafts/stdlib-catalog.md` §Amendment](../drafts/stdlib-catalog.md#amendment-2026-07-20--the-ratified-build-program).

## What this folder is

The reasoning for *why* Phalcom needs a standard library, what a modern one contains, and
what depends on what, lives in the exploration draft
[`../drafts/stdlib-catalog.md`](../drafts/stdlib-catalog.md) (Tier 0–6,
open questions S-1…S-13). **That document is the argument. This folder is the program:**
the eighteen selected items, the order they must be built in, what each will cost, and one
spec file per item.

Know which tier you are reading — the four in
[`drafts/README.md`](../spec/current/drafts/README.md) plus this one:

| Tier | Where | Means |
|---|---|---|
| Ratified decision | `docs/adr/accepted/` | Committed. Needs a superseding ADR to change. |
| Normative spec | `docs/spec/current/*.md` | The designed surface. Cites its ADR. |
| As-built | `docs/forge/units/*/as-built.md` | What shipped, with `file:line`. |
| Draft | `docs/spec/current/drafts/` | Exploration. No authority. |
| **Program** ← *you are here* | `docs/spec/current/stdlib/` | **A selected, ordered work list.** Each item's spec file, once written, is a design document — it becomes normative only by growing an ADR. |

**Nothing here is built.** No item below has an owning unit. The per-item spec files are
listed but **not yet written**.

---

## Ranking scales

Two independent axes. Conflating them is how "just add files" becomes a six-week unit.

**Complexity (1–5)** — design risk. How much can go *irreversibly* wrong: how many
ratified decisions it touches, whether a wrong choice is a breaking change, how much of it
is a ruling rather than an implementation.

| | |
|---|---|
| **1** | Mechanical. Bind an existing Rust API; the design is not in question. |
| **2** | One or two contained design choices with obvious defaults. |
| **3** | A new representation or a new heap arm; localized but novel. |
| **4** | Touches a ratified ADR, or a wrong choice breaks a public surface later. |
| **5** | Touches the VM's core invariants, or is a genuinely hard domain in its own right. |

**Work (S/M/L/XL)** — implementation volume, roughly: **S** ≤ ~300 lines and a day;
**M** a few days; **L** a week-plus; **XL** multi-week, and the item is really a track.

High complexity with low work is the dangerous combination — it *looks* cheap on a
sprint board and is where a wrong ruling gets made in an afternoon. Items **17** and **3**
are that shape.

---

## The program

Ordering is dependency-forced first, then cheapest-useful-first among items that are free
to move. Blockers listed are the ones that must be *resolved*, not merely noted.

| # | Item | Cx | Work | Blocked on | Spec file |
|---|---|:--:|:--:|---|---|
| **1** | **Numeric tower** — `Number` abstract, `Int` (exact, unbounded) / `Float`, bitwise ops, `/` vs `~/` | **5** | **XL** | ADR-0024 (ratified, unbuilt). Nothing else. | `01-numeric-tower.md` |
| **2** | **`BigInt` surface + `Decimal`** — radix conversion, `modPow`/`gcd`; `Decimal` scale + rounding modes | 4 | L | 1 | `02-bigint-decimal.md` |
| **3** | **Sealed types / enums** — exhaustiveness checking over `match` | **4** | **M** | PDR-0001 (classes are closed) | `03-sealed-enums.md` |
| **4** | **`Comparable` / `Hashable` / `sort`** — ordering contract, total-order law, stable sort | 2 | M | 1 (`Float` NaN ordering); protocols prototype-only (S-12); **ffi.md F-12** | `04-ordering-hashing.md` |
| **5** | **`Bytes`** — mutable octet buffer, new heap arm | 3 | M | 1, 4 | `05-bytes.md` |
| **6** | **`Path`** — opaque, not `String` | 3 | M | 5; **S-4** ruling | `06-path.md` |
| **7** | **`Reader` / `Writer` / `Seekable`** — the stream protocol | **4** | **M** | 5; **S-2** ruling; S-12; F-12 | `07-streams.md` |
| **8** | **`File` / `Fs`** — open, read, write, metadata, permissions, directory walk | 4 | **XL** | 6, 7; **S-1** ruling (resource lifetime) | `08-file-fs.md` |
| **9** | **`stdio` / `env`** — real writers behind `System.print`; process environment | 2 | M | 7; **S-5**, **S-13** | `09-stdio-env.md` |
| **10** | **Text** — `Encoding` (UTF-8/16/Latin-1), `StringBuilder`, `Char` | 2 | M | 5; **S-9** (what does `StringCodePointSequence` yield today?) | `10-text.md` |
| **11** | **`math`** | **1** | **S** | 1 | `11-math.md` |
| **12** | **`random`** — seeded PRNG + OS CSPRNG, kept distinct | 2 | S | 1, 5 | `12-random.md` |
| **13** | **`Instant` / `Duration`** — monotonic time only | **1** | **S** | 1 | `13-instant-duration.md` |
| **14** | **`os`** | **1** | **S** | 1; **S-13** | `14-os.md` |
| **15** | **`Uuid`** — v4, v7, parse | **1** | **S** | 5, 12, 13 | `15-uuid.md` |
| **16** | **Backtraces** — reified `Backtrace`, `Error#backtrace`, `Error#cause` | 2 | M | — | `16-backtraces.md` |
| **17** | **`WeakRef` / `WeakMap`** | **4** | **M** | ADR-0050 amendment; **S-8** (cost unscoped) | `17-weakref.md` |
| **18** | **`DateTime` / `TimeZone` / calendar** | **5** | **XL** | 13, 10 | `18-datetime.md` |
| **19** | **Timers** — `sleep`, `Timer.after/every`, and the reactor behind them | **5** | **L** | 13; **S-2** and **S-3** rulings; `open-questions.md` §15 fairness | `19-timers.md` |

Nineteen rows for eighteen items: **Time is split**. See "Deviations" below.

### Phase boundaries

```
items  1– 2   numeric substrate      — nothing else can start
items  3– 7   substrate + protocols  — 3 and 4 parallel; 5→6→7 serial
items  8–10   the platform surface   — the bulk of the user-visible library
items 11–16   cheap and parallel     — six independent items, any order
items 17–19   deep or deferred       — each needs a ruling first
```

Items 11–16 are the only stretch that parallelizes cleanly. Items 1–8 are a chain.

---

## Deviations from the order as listed

Recorded rather than applied silently.

**1. `BigInt` moved inside item 1.** ADR-0024 §Decision 1 specifies `Int` as *exact,
**unbounded***, and its Context rejects a tag-only split because `f64` is exact only to
2^53. An unbounded `Int` is a bignum — arbitrary-precision arithmetic and the promotion
channel are item 1's *semantics*, not a follow-on library. Item 2 keeps the `BigInt`
**surface** and all of `Decimal`. Building item 1 with a bounded `Int` would contradict a
ratified ADR and be a breaking change to fix afterwards.

**2. Time split into 13 and 18.** `Instant`/`Duration` are monotonic-clock arithmetic:
complexity 1, work S, no blockers past item 1. `DateTime`/`TimeZone`/calendar is time
zones, the IANA database, DST-ambiguous local times, leap seconds, and parse/format —
complexity 5, work XL, and one of the hardest domains in any standard library. Shipping
them as one item hides an XL behind an S. Almost everything that wants "time" wants
`Instant`/`Duration`; put the cheap half early and let the expensive half wait.

**3. `math`, `os`, `Instant`/`Duration`, `Uuid` pulled forward.** Four complexity-1/work-S
items. They are useful immediately, and the first of them to land establishes the
**native-module registration seam** that every later Tier-3 item reuses — ffi.md §6's
recommendation to make `install_primitives` accept a registry of module descriptors rather
than hard-coding each. Doing that once, early, on an item where nothing else can go wrong,
is worth more than the item itself. `math` is the natural first.

**4. Timers moved last, and re-ranked to complexity 5.** Listed mid-order and reading like
a small item — it is not. `System.sleep(_)` is documented **still open** in
[`system.md`](../spec/current/system.md), the scheduler is a bare ready-queue, and there is no
reactor. A timer needs a completion source, integration with `VM::ready_queue`, and an
answer to `open-questions.md` §15 on fairness. It *is* the blocking-vs-reactor fork (**S-2**)
wearing a smaller name, and S-2 changes the **signatures** of items 7, 8, and 9 — `Result`
versus `Future<Result>` on every read.

**This is the program's one real hazard.** If items 7–9 ship before S-2 is ruled, the
ruling is made by accident and reversing it breaks every IO selector in the language. The
mitigation is cheap: **rule on the signature shape before item 7, even if only the blocking
implementation is built.** A `Future`-returning selector can be backed by a synchronous
implementation. The reverse is a breaking change.

**5. `WeakRef` moved late.** Small surface, deep change: the collector has no weak worklist,
so it needs a second mark pass and weak-slot clearing before sweep, plus an ADR-0050
amendment. **S-8** records that nobody has scoped this against ADR-0050's stated algorithm.
Complexity 4 on a two-selector API — the "looks cheap, isn't" shape.

**6. Backtraces has no blockers and can land any time.** A prior session recorded that
`print_rt` / `runtime_error` / `SourceLoc` are **built but dead** — `cmd_run` bypasses them.
**Unverified in this pass; re-check before scoping.** If it holds, item 16 is mostly wiring
plus a reified `Backtrace` object and a capture point on `Error`, and it is the single
biggest debuggability win per unit of work in the whole program.

---

## Standing constraints on every spec file in this folder

1. **Protocols are prototype-only.** `Reader`, `Writer`, `Seekable`, `Comparable`,
   `Hashable`, `Disposable` are ordinary classes; conformance is documented and
   **unenforced**. A `@protocol` decorator may follow (**S-12**). Consequence: each spec
   must state its conformance requirements in prose precisely enough that a later
   `@protocol` can be derived mechanically from them. Otherwise the decorator arrives to
   find a dozen classes that each almost conform.
2. **No new floor primitives without the ADR-0019 test.** The floor is frozen and audited
   at 125 bindings (`phalcom-core/tests/invariants.rs`). Everything here is a module or a
   native-backed class, per ffi.md §5.1 — not a floor amendment.
3. **Native storage, `.ph` protocol.** ADR-0020's pattern, which `List`/`Map`/`Set`/
   `Tuple`/`Range` already follow. A spec proposing a wholly-native class must say why.
4. **Signatures in ADR-0012 selector form**, no default arguments (ADR-0043), predicates
   return real `Bool` (ADR-0021), sequences extend `Iterable` under the bare-cursor
   protocol (ADR-0048).
5. **Trailing `_` marks the native binding**, `.ph` wrapper above it — U-NATIVE-MARKER.
6. **Every spec carries a test plan.** Items 4 and 7 additionally carry **ffi.md F-12**:
   a forced collection inside a native→`.ph` callback, asserting the held handle survives.
   Whichever lands first pays that cost once, for both.

---

## Rulings still outstanding

These block specs, not implementations — a spec written before its ruling encodes the
ruling by accident. Full text in
[`stdlib-catalog.md` §Open questions](../spec/current/drafts/stdlib-catalog.md#open-questions).

| Ruling | Blocks | Why it cannot be deferred |
|---|---|---|
| **S-1** resource lifetime — `using` + `Disposable` + leak reporting, or real finalizers | 8 | ADR-0050 banks "no finalizers exist" as a safety property. The user has stated finalizers are wanted; this is the ruling that says in what form. |
| **S-2** IO shape — blocking, thread-pool, or reactor | 7, 8, 9, 19 | Changes signatures, not implementations. Hazard 4 above. |
| **S-3** are threads ever user-visible | S-2's implementation | Also changes the object model's mutability story and ADR-0050's root enumeration. Should precede S-2. |
| **S-4** `Path` opaque or `String`-backed | 6 | `Object::Str` is UTF-8-enforced; `String`-paths cannot represent real POSIX filenames. |
| **S-13** `process`/`os` as modules or blessed classes | 9, 14 | Module and blessed class are different mechanisms with different bootstrap-DAG consequences. Settle once for all Tier-3 surfaces. |
| **S-7** does `System` remain the single effect receiver | 8, 9, 14 | [`system.md`](../spec/current/system.md) §1's rule is "effects are named, not ambient". If `Fs`/`os` become plain globals, that property is lost in one commit. |

**S-1**, **S-2**, and **S-13** are the three that gate the most work. None is an
engineering question.
