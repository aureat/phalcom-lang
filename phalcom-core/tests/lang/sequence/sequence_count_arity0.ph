// area: sequence
// spec: iteration.md §5; ADR-0035
// status: PASS
// count traverses via iterate, not size, proving generic length derivation works on List and WhereView

let list = [10, 20, 30, 40]
System.print(list.count)

let view = WhereView.new(list, { x => x > 15 })
System.print(view.count)
