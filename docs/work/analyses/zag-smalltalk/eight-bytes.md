## Value Representation in Zig and NaN-Boxing

The Zag Smalltalk VM uses a sophisticated **tagged value encoding** system to represent objects efficiently. Rather than using a single approach for all types, the system evolved through several iterations. Let me explain the current **Zag encoding** (the default) and how NaN-boxing relates to it.

### Current Approach: Zag Tagged Encoding

The Zag encoding uses **low-bit tagging** to pack multiple value types into a 64-bit word. Instead of allocating everything on the heap, it encodes immediates directly in the `Object` bitpattern:

```zig
pub const Object = packed struct(u64) {
    tag: Tag,        // 3 bits (low bits 0-2)
    hash: u45 = 0,   // 45 bits
    extra: u11 = 0,  // 11 bits
    class: Compact = @enumFromInt(0),  // 5 bits (high bits 59-63)
};

const Tag = enum(u3) {
    heap = 0,
    smallinteger = 2,
    floatA = 4,
    floatB = 5,
    _,
};
```

**Recognition by low 3 bits:**
- **`tag & 2 != 0`** → `SmallInteger` (62-bit signed integers in high bits)
- **`tag & 6 == 4`** → `Float` (immediate floats using tags 4 and 5)
- **`tag == 0`** with class bits → Pointers (heap objects or nil)
- **`tag == 1`** → Immediate values (`Character`, `Symbol`, `True`, `False`)

### Float Encoding with Rotate-and-Shift

Floats are encoded using **left-rotate-by-5 and add**, decoded with right-rotate-by-5 and subtract. This is implemented in `floatEncoding.zig`:

```zig
// Encoding example (Fst2 with MATCH=4):
const u = rotl(u64, @bitCast(x), 5) +% (MATCH + 1);
if (u & 6 == MATCH) {
    return u;  // Encoded as immediate
}

// Decoding:
if (self & MATCH != 0) {
    return @bitCast(rotr(u64, self -% (MATCH + 1), 5));
}
```

This encodes floats in the ranges:
- `0..3.8e-270`
- `5.9e-39..6.8e38`
- `1.1e270..1.8e308`

### NaN-Boxing Alternative

While Zag is the current default, the system also supports **NaN-boxing** (in `zig/zag/encoding/nan.zig`) as an alternative encoding. This is a classical approach where:

**IEEE 754 Properties:**
- When the 11-bit exponent is all 1s (`0x7ff`) AND at least one low 52 bit is non-zero → **NaN**
- This gives you **52 bits of payload** to encode non-float values

```zig
pub const Object = packed struct(u64) {
    data: u48,
    tag: Tag,  // u16 using the upper bits of the float exponent
};
```

**Encoding Table (from Documentation):**
- `0000-7FEF` (Sign+Exponent) → Normal IEEE 754 doubles
- `7FF0` with payload → `ThunkReturnLocal`
- `7FF1` with payload → `ThunkReturnInstance`
- `7FF8` with payload → `ThunkHeap`
- `7FFA` with payload → `Symbol`
- `7FFB` → `False`
- `7FFC` → `True`
- `7FFD` with payload → `Character`
- `FFFC+` → `SmallInteger` values

**Key advantage:** All normal doubles are stored naturally without transformation—enabling vectorization and GPU acceleration. However, it has mediocre integer performance compared to Zag.

### Why Zag Over NaN-Boxing?

The design philosophy notes that:
- Zag encodes the **most common 1/4 of floats** (97% of actual usage) as immediates
- Has **exceptional integer performance** (62-bit SmallIntegers recognized instantly)
- Supports **native pointers** without conversion
- Requires fewer instructions for encode/decode than both Spur and NaN-boxing

---

## Complete Value and Object Model: Zag Smalltalk

The choice to make floats "naturally encoded" in NaN-boxing versus the rotate-shift approach in Zag represents a trade-off: Zag prioritizes CPU integer workloads and pointer handling, while NaN-boxing would prioritize float-heavy workloads and vectorization.

Yes, **Zag uses a single 64-bit one-word value representation** for all Objects.

### Value Representation Size

**Single 64-bit word (`u64` / `i64`)**

```zig
pub const Object = packed struct(u64) {
    tag: Tag,        // 3 bits (bits 0-2)
    hash: u45 = 0,   // 45 bits (bits 3-47)
    extra: u11 = 0,  // 11 bits (bits 48-58)
    class: Compact = @enumFromInt(0),  // 5 bits (bits 59-63)
};
```

**Total: 3 + 45 + 11 + 5 = 64 bits**

---

## Integer Representation Range

### SmallInteger

**Size:** 62-bit signed integers (high 62 bits of the word)

**Range:** `-2,305,843,009,213,693,952` to `2,305,843,009,213,693,951`

**Encoding:**
- Recognized by: `(u64 & 2) != 0` — the low 2 bits indicate integer-ness
- The value occupies bits 2-63 (the high 62 bits)
- Encoding is done via **left-shift by 2 (intShift = 2)** and adding the tag value

```zig
pub const IntType = i62;
pub const maxInt = 0x3fff_ffff_ffff_ffff;  // 2^61 - 1 (63-bit for sign)
const intShift = 64 - @bitSizeOf(IntType);  // 64 - 62 = 2
```

**Examples of integer tags (low 3 bits):**
- `0b010` (2) — SmallInteger
- `0b011` (3) — reserved false-positive for SmallInteger
- `0b110` (6) — reserved false-positive for SmallInteger
- `0b111` (7) — reserved false-positive for SmallInteger

### Out-of-Range Integers

Integers outside the 62-bit range are stored as **heap objects** with class `Float` or allocated as `DoubleWordArray` objects.

---

## Float Representation Range

### Immediate Floats

**Recognition:** `(u64 & 6) == 4` — tags 4 and 5 encode floats

**Technique:** **Rotate-left-by-5, add offset, check low 3 bits**

The encoding uses `Fst2(4)` — a **two-tag float encoding**:

```zig
const floatEncoding = @import("floatEncoding.zig").Fst2(4);
```

**Encoded ranges:**
- `0.0` to `3.8e-270`
- `5.9e-39` to `6.8e38`
- `1.1e270` to `1.8e308`
- **NaN** and **Inf**

**Coverage:** These ranges cover **~97% of actual floating-point usage** in typical applications.

**Encoding process:**
```zig
const u = rotl(u64, @bitCast(x), 5) +% (MATCH + 1);  // MATCH = 4
if (u & 6 == 4) {
    return u;  // Successfully encoded as immediate
}
// Otherwise, falls back to heap allocation
```

**Decoding process:**
```zig
if (self & 4 != 0) {  // Check tag
    return @bitCast(rotr(u64, self -% (MATCH + 1), 5));
}
```

### Out-of-Range Floats

Floats outside the immediate ranges are **allocated on the heap** as `Float` class objects (instances of `MemoryFloat`):

```zig
pub inline fn fromNativeF(t: f64, sp: SP, context: *Context) object.Object {
    return @bitCast(encode(t) catch {
        return InMemory.float(t, sp, context);  // Heap allocation
    });
}
```

---

## Full Bitfield Layout

From Documentation (Zag.md), the complete encoding table:

| High 16 bits | Low bits | Tag (L3) | Type |
|---|---|---|---|
| `0000000000000000` | `0000...0000` | `000` | `nil` |
| `0000000000000000` | `aaaa...aaaa` | `000` | native (C) pointer |
| `00001lllllllllll` | `aaaa...aaaa` | `000` | `ThunkReturnLocal` (Closure) |
| `iiiiiiiiiiiiiiii` | `iiii...iiii` | `010` | **SmallInteger** (62-bit) |
| `ffffffffffffffff` | `ffff...ffff` | `100` | **Float (Tag 4)** |
| `ffffffffffffffff` | `ffff...ffff` | `101` | **Float (Tag 5)** |
| `0110000000000000` | `0000...0000` | `001` | **False** |
| `0110100000000000` | `0000...0000` | `001` | **True** |
| `0111000000000000` | `0...uuuuuu...uu` | `001` | **Character** (Unicode) |
| `0101100000000000` | `00...hhhh...hh` | `001` | **Symbol** (with hash/arity) |

**Immediate Class Recognition (by low 3 bits):**
- `0` — Nil or heap object (check class in header)
- `1` — Immediate value (Symbol, Character, True, False)
- `2` — SmallInteger
- `4, 5` — Float
- `3, 6, 7` — Reserved false-positives for SmallInteger

---

## Heap Object Representation

Objects that **must be allocated on the heap** (cannot be encoded as immediates):

### HeapHeader Structure (64-bit)

```zig
pub const HeapHeader = packed struct(u64) {
    classIndex: ClassIndex,     // u16 (16 bits, LSB)
    hash: u24 = 0,              // u24 (24 bits)
    objectFormat: Format = .free, // u7 (7 bits)
    immutable: bool = false,    // 1 bit
    age: Age = .nursery,        // u4 (4 bits)
    length: Length = 0,         // u11 (11 bits, MSB)
    forwarded: bool = false,    // 1 bit
    // Total: 16 + 24 + 7 + 1 + 4 + 11 + 1 = 64 bits
};

pub const Length = u11;  // Max addressable size: 2047 longwords (besides header)
pub const maxLength: Length = 2047;
```

**Field Meanings:**
- **classIndex (u16):** Which class the object belongs to (0-65535 possible classes)
- **hash (u24):** Identity hash for the object
- **objectFormat (u7):** How the object's array portion is structured (128 possible formats)
- **immutable (1 bit):** Whether the object can be modified
- **age (u4):** Generational GC age (onStack, nursery1-5, static, global, etc.)
- **length (u11):** Number of 64-bit longwords allocated (besides header) — max 2047
- **forwarded (1 bit):** Is this a forwarding pointer during GC?

### HeapObject Layout in Memory

```
HeapObject = HeapHeader (64-bit) + variable fields
```

**Example for Array of 100 Objects:**

```
[HeapHeader: 64-bit] [Object 0] [Object 1] ... [Object 99] [optional footer]
 (1 word)             (1 word)   (1 word)      (1 word)
```

**Maximum heap object size:** (2047 + 1) × 8 bytes = **16,376 bytes** (16 KB)

---

## Immediate Class Encodings

All of these fit in a single 64-bit word (no heap allocation):

### SmallInteger (62-bit)
- **Range:** ±2.3 × 10^18
- **Storage:** High 62 bits

### Float (64-bit IEEE 754 subset)
- **Immediate ranges:** ~97% of typical floats
- **Storage:** Rotation-encoded in low 60 bits, tags in low 3

### Symbol
- **Storage:** 24-bit symbol table index (hashed), 4-bit arity
- **Max symbols:** 2^24 = 16 million (typical image has ~90K)
- **Advantage:** Symbol equality is **identity check only** (pointer-free dispatch)

### Character
- **Storage:** Full Unicode code point (up to 1.1M characters)
- **Bits used:** ~21 bits in the hash field

### Boolean Singletons
- **False:** `0x6000_0000_0000_0001`
- **True:** `0x6800_0000_0000_0001`
- Differ by 1 bit for fast testing

### nil (UndefinedObject)
- **Representation:** `0x0000_0000_0000_0000` (all zeros)
- **Equivalent to:** C/C++/Rust `null`

### Thunk/Closure Immediates
- **ThunkReturnLocal, ThunkLocal, etc.:** Context address (48 bits) + variable index (11 bits)
- Used for efficient **non-local returns** and **block evaluation**

---

## Object Taxonomy

**Immediate Values (1 word, no GC):**
- SmallInteger (62-bit)
- Float (subset of IEEE 754)
- Symbol, Character
- True, False, nil
- Thunk/Closure variants (11 types)

**Heap Objects (allocated, GC'd):**
- **String** — UTF-8 bytes or ASCII
- **Array** — Object pointers
- **DoubleWordArray** — 64-bit integer array
- **BlockClosure** — Full closures with captured variables
- **Context** — Method execution context
- **CompiledMethod** — Threaded or JIT code
- **Class** — Class definition object
- **Symbol** (rarely, if table grows beyond 2^24)
- Custom user-defined classes

---

## Memory Efficiency

| Type | Storage | Advantage |
|------|---------|-----------|
| SmallInteger | 1 word | No GC, no indirection, instant dispatch |
| Float (common) | 1 word | No heap allocation, vector-friendly |
| Float (rare) | ~2 words (heap) | Can represent any IEEE 754 value |
| Symbol | 1 word | Unique, identity-based comparison |
| Nil | 0 bytes | Single word, all-zeros pattern |

**Result:** The vast majority of values are **immediate** (no heap allocation), reducing garbage collection pressure and improving CPU cache locality.