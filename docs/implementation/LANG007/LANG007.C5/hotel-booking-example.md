---
id: LANG007.C5.hotel-example
category: LANG
program: LANG007
checkpoint: LANG007.C5
kind: acceptance-example
status: PROPOSED
completion: NOT_STARTED
verification: UNVERIFIED
prepared: 2026-10-13
---

# A hotel quote with conflicts, room moves, and recurring service slots

## 1. The actual problem

A guest requests five nights, October 18–23, 2026. The hotel has bookings,
maintenance closures, and temporary holds. It must explain exact blocked nights,
find usable pieces in each room, propose a continuous stay with at most one room
move, and rank valid quotes by moves first, then price.

The same application offers arrival dates on a weekly grid and plans recurring
service visits. An immutable Progression supplies those grids. Clipping a grid
must preserve its anchor; an inspection between two visits must not be reported
as colliding with a visit just because their hulls overlap.

This is proposed Phalcom source and precise application pseudocode. Interval /
Progression APIs are not implemented yet. P1.T6 must turn this example into
compiled source fixtures and record actual results. No snippet is presented as
already runnable or runtime-verified.

## 2. Time and business rules

- A stay occupies civil nights in `[checkInDate, checkOutDate)`.
- October 18–23 includes nights 18, 19, 20, 21, and 22, never night 23.
- A booking ending on October 18 can precede the requested stay without sharing
  an occupied night. It may still require a housekeeping check.
- Maintenance also uses half-open civil-night intervals.
- Holds block only when their expiry Timestamp is strictly after the one `now`
  supplied to the quotation. An expiry equal to now is inactive.
- The reservation being amended is excluded by ID before interval comparison.
- All listed rooms meet the example's party-size and accessibility requirement;
  application filtering must run before generating alternatives.
- Money is integer cents. Rates are constant per occupied night in this fixture.
- Quotes are observations of one inventory snapshot, not committed bookings.
  Confirmation must revalidate availability in the application's transaction.

Civil-night order and physical housekeeping time are separate domains.
Housekeeping uses `Timestamp` intervals after explicit time-zone resolution;
Date intervals must never be compared with Timestamp intervals.

## 3. Inventory and expected conflict explanations

All dates below are in October 2026.

| Room | Rate/night | Booking / maintenance | Relation of requested `[18,23)` to block | Shared nights |
| --- | --- | --- | --- | --- |
| 101 | 150.00 | Booking `[16,20)` | Allen OverlappedBy; set Intersects | `[18,20)` — two nights |
| 101 | 150.00 | Maintenance `[22,24)` | Allen Overlaps; set Intersects | `[22,23)` — one night |
| 202 | 180.00 | Booking `[21,24)` | Allen Overlaps; set Intersects | `[21,23)` — two nights |
| 303 | 240.00 | Booking `[16,18)` | Allen MetBy; set TouchesAfter | None; turnover boundary Oct 18 |
| 303 | 240.00 | Maintenance `[19,20)` | Allen Contains; set Intersects | `[19,20)` — one night |

An expired hold on room 202 `[18,20)` is ignored. A hold with that same extent
and a future expiry removes those nights and changes the winning quote. Testing
both states proves that interval results are combined with ordinary business
policy rather than pretending the interval algebra knows booking status.

After all active blockers are subtracted:

| Room | Available portions of requested stay |
| --- | --- |
| 101 | `[20,22)` |
| 202 | `[18,21)` |
| 303 | `[18,19)` and `[20,23)` |

No room covers the entire request. A hull-based “room 303 is free from 18 to 23”
would incorrectly erase its maintenance night 19.

## 4. Matching calculates values the policy consumes

Representative proposed source types and conflict handler:

```phalcom
from universe.intervals import (
    DateIntervals, DateDomain, Interval, IntervalRelation, IntervalPieces,
    IntervalView, DateProgressions, ProgressionIntervalRelation
)
from universe.time import Date, Timestamp

type NightSpan = Interval<Date, DateDomain>

data BlockKind {
    Booking(_ reservationId: Int)
    Maintenance
    Hold(_ expiresAt: Timestamp)
}

data RoomBlock(_ room: Int, _ nights: NightSpan, _ kind: BlockKind)

data ConflictEffect {
    Clear
    Turnover(_ boundary: Date)
    Blocked(_ shared: NightSpan, _ requestedOnly: IntervalPieces<Date, DateDomain>)
}

class BookingAlgebra {
    @class
    active(_ block: RoomBlock, _ ownReservation: Int, _ now: Timestamp) -> Bool {
        match block.kind {
            BlockKind::Booking(id) => id != ownReservation
            BlockKind::Maintenance => true
            BlockKind::Hold(expiry) => expiry > now
        }
    }

    @class
    explain(_ requested: NightSpan, _ blocked: NightSpan) -> ConflictEffect {
        match requested.relate(blocked) {
            IntervalRelation::BothEmpty => ConflictEffect::Clear
            IntervalRelation::LeftEmpty(_) => ConflictEffect::Clear
            IntervalRelation::RightEmpty(_) => ConflictEffect::Clear
            IntervalRelation::Before(_) => ConflictEffect::Clear
            IntervalRelation::After(_) => ConflictEffect::Clear
            IntervalRelation::TouchesBefore(at, _) => ConflictEffect::Turnover(at)
            IntervalRelation::TouchesAfter(at, _) => ConflictEffect::Turnover(at)
            IntervalRelation::Intersects(shared, requestedOnly, _) =>
                ConflictEffect::Blocked(shared, requestedOnly)
        }
    }
}
```

`shared` is the exact conflict region. `requestedOnly` is already the result of
subtracting this block, rather than a second application implementation of the
same inequalities. Every return case has a common explicit type. The snippet
uses ordinary payload patterns, without guard clauses or new relation keywords.

For multiple blocks, operate on every remaining piece. The first successful
match against one booking cannot determine room availability against all other
bookings.

```phalcom
class RoomAvailability {
    @class
    appendPieces(_ target: List<NightSpan>, _ pieces: IntervalPieces<Date, DateDomain>) {
        match pieces {
            IntervalPieces::Zero => {}
            IntervalPieces::One(part) => { target.append(part) }
            IntervalPieces::Two(first, second) => {
                target.append(first)
                target.append(second)
            }
        }
    }

    @class
    free(_ room: Int, _ requested: NightSpan, _ blocks: List<RoomBlock>,
         _ ownReservation: Int, _ now: Timestamp) -> List<NightSpan> {
        let mut pieces = List<NightSpan>.new()
        if not requested.isEmpty { pieces.append(requested) }

        for block in blocks {
            if block.room == room {
                if BookingAlgebra.active(block, ownReservation, now) {
                    let next = List<NightSpan>.new()
                    for part in pieces {
                        RoomAvailability.appendPieces(next, part.subtract(block.nights))
                    }
                    pieces = next
                }
            }
        }
        pieces
    }
}
```

Each subtraction returns Zero, One, or Two immutable intervals. The application
uses fresh Lists to accumulate a finite collection of these pieces; the Interval
itself does not become mutable. Overlapping, repeated, or nested maintenance
blocks cannot create duplicate free nights. Starting sorted and processing each
piece in order preserves ascending piece order without a hidden sorting helper.

Worst-case repeated subtraction grows the number of pieces with blocker count;
this simple quote algorithm is intentionally bounded by hotel inventory. C5
does not claim an interval index or asymptotic query acceleration.

## 5. Complete quote-generation algorithm

The fixture has a finite request of at most 30 nights. This application bound is
not a language restriction. It allows a simple exhaustive oracle for quotations.

```text
quote(request, rooms, blocks, ownReservation, now):
    require request finite, half-open, and checkIn < checkOut
    eligible = rooms satisfying party-size and accessibility requirements
    for each eligible room:
        free[room] = RoomAvailability.free(room, request, blocks, ownReservation, now)

    alternatives = []
    for each eligible room:
        if any one free[room] piece covers request:
            add one-room quote with cost nights(request) * rate(room)

    for each civil transfer date t strictly between checkIn and checkOut:
        firstStay  = DateIntervals.closedOpen(checkIn, t)
        secondStay = DateIntervals.closedOpen(t, checkOut)
        for each ordered pair of distinct eligible rooms (a,b):
            if one free[a] piece covers firstStay
               and one free[b] piece covers secondStay:
                add quote a:firstStay, b:secondStay
                cost = nights(firstStay)*rate(a) + nights(secondStay)*rate(b)

    reject alternatives that exceed the requested move limit
    sort by (number of moves, total cents, first room number,
             second room number, transfer date)
    return all alternatives in that deterministic order, and the first as recommendation
    if there are none, return an explicit NoAvailability result with conflict explanations
```

`nights([a,b)) = b.daysSince − a.daysSince`. Transfer dates advance by
`Period.days(1)`; they are not computed by adding elapsed seconds. Covering a
segment requires one actual free piece; covering it with the convex hull of
disconnected pieces is forbidden. Zero-night segments and self-to-self moves
are not generated.

The quote result can be another ordinary closed sum:

```text
QuoteResult = NoAvailability(explanations)
            | Available(recommendedQuote, orderedAlternatives)
Quote = Single(room, stay, cents)
      | Split(firstRoom, firstStay, secondRoom, secondStay, cents)
```

Every produced quote must additionally satisfy an independent oracle: segment
union equals the requested night set, segment intersections are empty, all
segments have positive length, and no segment shares a night with an active
block of its assigned room.

## 6. Worked alternatives and ranking

| Alternative | Occupied-night partition | Moves | Total |
| --- | --- | --- | --- |
| Recommended | 202 `[18,21)`, then 303 `[21,23)` | 1 | 3×180 + 2×240 = **1,020.00** |
| Valid alternative | 202 `[18,20)`, then 303 `[20,23)` | 1 | 2×180 + 3×240 = 1,080.00 |
| Cheaper, outside requested policy | 202 `[18,20)`, 101 `[20,22)`, 303 `[22,23)` | 2 | 2×180 + 2×150 + 1×240 = 900.00 |
| Invalid attractive combination | 202 `[18,21)`, then 101 `[21,23)` | 1 | Conflicts with room 101 maintenance on night 22 |
| Invalid hull-based suggestion | 303 `[18,23)` | 0 | Crosses room 303 maintenance on night 19 |

With a one-move limit, the cheaper two-move itinerary must not win. If the guest
explicitly changes that policy, a separate enumeration with two transfers can
consider it. The P1 fixture must verify the one-move algorithm above; the
900.00 itinerary is an independently validated policy counterexample, not a
claim that the one-move generator produces it.

For the recommended split, `[18,21)` and `[21,23)` touch at Oct 21 with no shared
night and union to the request. Their relation supplies the transfer boundary;
the application can explain the room change without recomputing endpoint tests.

## 7. Immutable Progression adds usable booking and service grids

Weekly arrival policy:

```phalcom
from universe.time import Period

let arrivalPolicy = DateProgressions.unbounded(
    start: Date.parse("2026-10-13"),
    step: Period.days(7)
)
let requestWindow = DateIntervals.closedOpen(
    Date.parse("2026-10-18"), Date.parse("2026-10-23")
)

match arrivalPolicy.relate(requestWindow) {
    ProgressionIntervalRelation::EmptyProgression => ...
    ProgressionIntervalRelation::EmptyInterval(_) => ...
    ProgressionIntervalRelation::Disjoint(_) => ...
    ProgressionIntervalRelation::Contained(arrivals) => ...
    ProgressionIntervalRelation::Clipped(arrivals, _) => {
        for date in arrivals { System.print(date.toString) }
    }
}
```

The clipped result contains **October 20 only**. It does not become October 18
by reanchoring at the request's lower bound. This grid is an optional alternative
arrival policy; it does not alter the already quoted Oct 18 request retroactively.

Inspection starts every two days, October 18, 20, and 22:

```phalcom
let inspectionDates = DateProgressions.finite(
    start: Date.parse("2026-10-18"),
    step: Period.days(2),
    count: 3
)

for availablePiece in freeRoom303 {
    for inspectionDate in inspectionDates.clip(availablePiece) {
        System.print(inspectionDate.toString)
    }
}
```

Here `freeRoom303` is the result of the availability algorithm above. Output is
Oct 18, Oct 20, Oct 22. Clipping the same progression to maintenance `[19,20)`
returns Empty, although its hull `[18,23)` intersects the maintenance interval.
Oct 20 belongs to the post-maintenance piece; it is not counted twice.

The same immutable progression can be traversed again for staffing or price
calculation; one traversal's cursor does not consume it. A separate iterator
retains its own index and preserves the same dates.

These are scheduled start dates, not whole-day service reservations. A service
with duration must construct a Timestamp interval for each occurrence and use
interval algebra for its full occupied duration.

## 8. Physical housekeeping and timestamp progression

On October 21, assume an explicitly resolved fixed offset UTC+03:00 for this
fixture. Room 202 checks out at 11:00 local (=08:00 UTC); cleaning occupies
`[08:00,10:00)` UTC; its next guest arrives at 15:00 local (=12:00 UTC).

Cleaning and next occupancy have `Before` with gap `[10:00,12:00)`: two hours of
clear time. A cleaning window `[10:00,12:00)` instead touches next occupancy at
12:00 without sharing an instant; a cleaning window ending at 12:30 intersects
next occupancy on `[12:00,12:30)` and must be rejected or rescheduled.

A two-hour Timestamp progression anchored at 08:00 gives proposed service starts
08:00, 10:00, 12:00. Clipping it to `[10:00,12:00)` yields 10:00 only. For a
90-minute task, construct `[10:00,11:30)` and check the full task against both
cleaning and next occupancy. The task is clear; a candidate starting at 11:00
would end at 12:30 and conflict even though its start is inside the gap.

The fixture uses already resolved Timestamps. DST folds/gaps are handled by the
existing explicit resolution policy before constructing intervals or grids;
C5 must not silently choose an ambiguous local instant. A weekly local-clock
recurrence across DST is not an affine Timestamp progression and is outside C5.

## 9. Why the feature earns its place

The application needs several different questions answered together:

1. Does a booking share any occupied nights with the request?
2. Which exact nights, and which portions remain usable?
3. Is a boundary a valid turnover rather than a night collision?
4. Do disconnected free pieces support a continuous itinerary?
5. Which phase-aligned arrival or inspection dates fall in those pieces?
6. Does the duration of a proposed service fit between occupancies?

Typed relation payloads, small immutable piece sums, and Progression clipping
make those answers composable. The application then applies business policy to
the results. Mainstream languages can implement the same mathematics with
libraries, database ranges, ADTs, or active patterns; C5's proposed advantage is
the consistent standard model and existing exhaustive-match integration, as
discussed with sources in the [design](interval-relational-matching-spec.md#11-precedent-novelty-and-what-this-choice-precludes).

## 10. Acceptance scenarios

P1.T6 must supply independent expected results for:

- the inventory, conflict explanations, free pieces, and 1,020.00 recommendation;
- rejection of the maintenance-crossing and hull-based quotes;
- expired, exactly-expired, and live holds, with `now` captured once;
- excluding the guest's own reservation when amending it;
- a room that covers the full request, which must outrank every split quote;
- no eligible rooms / no valid itinerary, producing explicit NoAvailability;
- duplicate and overlapping blockers producing the same night set;
- month/year/leap-day boundaries, with zero-night and reversed requests rejected;
- domain mismatch and unresolved local-clock input errors;
- sparse progression clipping, preserved weekly phase, and independent iteration;
- exact cleaning-boundary touch versus duration intersection;
- equality between an interval-operations result and a small per-night oracle.

Do not assert that producing a quote commits inventory or proves future
availability. Transactional confirmation belongs to the booking application.
