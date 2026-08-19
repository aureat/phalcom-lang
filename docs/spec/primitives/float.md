```ts
class Float is Number {
	@class
	NaN { 0.0 / 0.0 }

	@class
	Infinity { 1.0 / 0.0 }

	@class
	-Infinity { -1.0 / 0.0 }

	@class
	call() { 0.0 }

	@class
	call(_ value) {
		if value is Float {
			value
		} else if value is Int {

		} else if value is String {
			const lower = value.lowercase
			if lower == "infinity" or lower == "+infinity" {
				Float.Infinity
			} else if lower == "-infinity" {
				Float.-Infinity
			} else if lower == "nan" {
				Float.NaN
			}
		}
	}
}
```