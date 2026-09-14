# Worked VM Disassembly: Tuple Destructuring Binding

**Status:** Draft 0.1 — worked normative example for the Phalcom VM and bytecode specification  
**Specification layer:** Phalcom VM and Internals  
**Example source:** `let (a, b) = (5, 6)`  
**Implementation baseline:** `aureat/phalcom-lang`, repository state audited on 2026-09-14  
**Primary implementation references:**

- `phalcom-core/src/compiler/lib/mod.rs`
- `phalcom-core/src/compiler/lib/patterns.rs`
- `phalcom-core/src/compiler/lib/expr.rs`
- `phalcom-core/src/compiler/lib/jumps.rs`
- `phalcom-core/src/compiler/lib/scope.rs`
- `phalcom-core/src/chunk.rs`
- `phalcom-core/src/bytecode.rs`
- `phalcom-core/src/vm/dispatch.rs`
- `phalcom-core/src/vm/mod.rs`
- `phalcom-core/src/primitive/tuple.rs`
- `phalcom-core/core/universe/src/collections/tuple.ph`
- `phalcom-core/bin/phalcom/disasm.rs`

---

## 1. Purpose and scope

This document is a complete worked example of how a small Phalcom source program is lowered into executable VM bytecode, transformed by the reference implementation's post-compilation superinstruction fusion pass, printed by the disassembler, and executed by the virtual machine.

The example is intentionally more substantial than a minimal constant or arithmetic expression:

```phalcom
let (a, b) = (5, 6)
```

Despite its small surface form, this program exercises a broad cross-section of the Phalcom implementation:

- tuple/product literal construction;
- module-level binding semantics;
- structured destructuring;
- compiler-managed scratch locals;
- exact runtime class validation;
- linked universe/prelude reads;
- `Same` comparison;
- runtime arity validation;
- compiler-generated error construction and raising;
- ordinary selector dispatch;
- tuple element projection;
- global definition;
- relative branch offsets;
- root callable completion;
- constant-pool indexing;
- post-compilation superinstruction fusion;
- preserved shadowed instruction slots;
- inline-cache and source-span attribution for fused sends.

The example therefore serves four purposes.

1. It is a **worked reading guide** for Phalcom disassembly.
2. It documents the relationship between **source semantics, compiler lowering, and VM execution**.
3. It demonstrates which parts of printed disassembly are **semantic** and which are **implementation-run artifacts**.
4. It acts as a useful **regression fixture** for the canonical bytecode specification, especially for `InvokeLocal`, `InvokeConst`, relative branches, scratch locals, and fused-send shadow slots.

This document is not a substitute for the instruction-by-instruction bytecode specification. The canonical instruction semantics remain defined by `bytecodes.md`. Where this example exposes an ambiguity or implementation invariant that the instruction specification must state explicitly, this document calls it out.

---

## 2. Source program

The complete source program is:

```phalcom
let (a, b) = (5, 6)
```

At module scope, the observable source semantics are:

1. evaluate the initializer `(5, 6)` exactly once;
2. require the initializer value to satisfy the tuple destructuring contract;
3. require the tuple to contain exactly two elements;
4. bind element `0` to module-global `a`;
5. bind element `1` to module-global `b`;
6. complete the root module callable with surface absence, `None`.

A useful semantic expansion is:

```text
tmp = evaluate (5, 6) exactly once

require tmp.class same Tuple
require tmp.size == 2

a = tmp.at(0)
b = tmp.at(1)

return None
```

This is explanatory pseudocode, not a source-level desugaring. The actual compiler lowering uses a compiler-generated scratch local, ordinary message sends for most protocol operations, explicit VM control-flow instructions, and compiler-generated error paths.

---

## 3. Reference disassembly

The reference disassembly is:

```text
<main>   <main>   slots=1 upvalues=0
constants:
  [0] 5
  [1] 6
  [2] Symbol(class)
  [3] <obj ObjRef(3785v1)>
  [4] Symbol(new(_))
  [5] Symbol(raise())
  [6] Symbol(size)
  [7] 2
  [8] Symbol(!=(_))
  [9] <obj ObjRef(3786v1)>
  [10] Symbol(new(_))
  [11] Symbol(raise())
  [12] 0
  [13] Symbol(at(_))
  [14] Symbol(a)
  [15] 1
  [16] Symbol(at(_))
  [17] Symbol(b)

bytecode:
  0000  line 1   Constant(0)
  0001  line 1   Constant(1)
  0002  line 1   BuildTuple { positional: 2, labeled: 0 }
  0003  line 1   ReserveScratchLocal(0)
  0004  line 1   SetLocal(0)
  0005  line 1   InvokeLocal(0, 0, class)
  0006  line 1   [shadowed dead slot] Invoke(0, 2)
  0007  line 1   GetLinked(99)
  0008  line 1   Same
  0009  line 1   JumpIfFalse(1)
  0010  line 1   Jump(5)
  0011  line 1   GetLinked(22)
  0012  line 1   InvokeConst(3, 1, new(_))
  0013  line 1   [shadowed dead slot] Invoke(1, 4)
  0014  line 1   Invoke(raise(), 0)
  0015  line 1   Pop
  0016  line 1   InvokeLocal(0, 0, size)
  0017  line 1   [shadowed dead slot] Invoke(0, 6)
  0018  line 1   InvokeConst(7, 1, !=(_))
  0019  line 1   [shadowed dead slot] Invoke(1, 8)
  0020  line 1   JumpIfFalse(5)
  0021  line 1   GetLinked(22)
  0022  line 1   InvokeConst(9, 1, new(_))
  0023  line 1   [shadowed dead slot] Invoke(1, 10)
  0024  line 1   Invoke(raise(), 0)
  0025  line 1   Pop
  0026  line 1   GetLocal(0)
  0027  line 1   InvokeConst(12, 1, at(_))
  0028  line 1   [shadowed dead slot] Invoke(1, 13)
  0029  line 1   DefineGlobal(14)
  0030  line 1   GetLocal(0)
  0031  line 1   InvokeConst(15, 1, at(_))
  0032  line 1   [shadowed dead slot] Invoke(1, 16)
  0033  line 1   DefineGlobal(17)
  0034  line 1   Nil
  0035  line 1   Return
```

The remainder of this document explains every part of this output.

---

# Part I — Callable and constant-pool structure

## 4. Callable header

The first line is:

```text
<main>   <main>   slots=1 upvalues=0
```

The disassembler obtains the slot and upvalue counts from the compiled root closure's `Callable`.

### 4.1 `slots=1`

`slots=1` means the root callable requires a maximum of one frame-local storage slot.

It does **not** mean:

- the VM operand stack can contain only one value;
- the source program has one variable;
- only one runtime value may be live;
- one of the user variables `a` or `b` is local.

The single local slot is compiler-generated scratch storage used to keep the destructuring initializer alive:

```text
local[0] = $destructure...
```

The source bindings `a` and `b` are module globals because the `let` appears at module scope.

Conceptually:

```text
frame locals:
  slot 0 -> compiler-generated tuple destructuring temporary

module globals:
  a
  b
```

This distinction is important when reading Phalcom bytecode: callable slot counts describe activation-local storage, not source-level binding counts and not maximum operand-stack depth.

### 4.2 `upvalues=0`

The root callable captures no lexical variables from an outer activation.

Thus:

```text
source-visible locals:       0
compiler scratch locals:     1
module globals introduced:   2
captured upvalues:           0
```

---

## 5. Constant pool

The constant pool is:

```text
[0] 5
[1] 6
[2] Symbol(class)
[3] <obj ObjRef(3785v1)>
[4] Symbol(new(_))
[5] Symbol(raise())
[6] Symbol(size)
[7] 2
[8] Symbol(!=(_))
[9] <obj ObjRef(3786v1)>
[10] Symbol(new(_))
[11] Symbol(raise())
[12] 0
[13] Symbol(at(_))
[14] Symbol(a)
[15] 1
[16] Symbol(at(_))
[17] Symbol(b)
```

The semantic meaning of each entry is:

| Index | Semantic value | Use |
|---:|---|---|
| `0` | integer `5` | first tuple element |
| `1` | integer `6` | second tuple element |
| `2` | selector symbol `class` | exact runtime class check |
| `3` | string `"pattern expected Tuple"` | class-mismatch diagnostic |
| `4` | selector symbol `new(_)` | construct the first `Error` |
| `5` | selector symbol `raise()` | raise the first `Error` |
| `6` | selector symbol `size` | obtain tuple arity |
| `7` | integer `2` | required destructuring arity |
| `8` | selector symbol `!=(_)` | compute arity mismatch |
| `9` | string `"destructuring pattern expected a 2-element Tuple"` | arity-mismatch diagnostic |
| `10` | selector symbol `new(_)` | construct the second `Error` |
| `11` | selector symbol `raise()` | raise the second `Error` |
| `12` | integer `0` | first tuple index |
| `13` | selector symbol `at(_)` | project first tuple element |
| `14` | symbol `a` | first global binding name |
| `15` | integer `1` | second tuple index |
| `16` | selector symbol `at(_)` | project second tuple element |
| `17` | symbol `b` | second global binding name |

### 5.1 Why the strings print as `ObjRef(...)`

The disassembler special-cases certain heap object kinds, including classes, closures, blocks, and methods. Ordinary heap-backed values that have no dedicated disassembly presentation fall back to their generic value/debug representation.

Therefore:

```text
[3] <obj ObjRef(3785v1)>
```

is semantically the string:

```text
"pattern expected Tuple"
```

and:

```text
[9] <obj ObjRef(3786v1)>
```

is semantically:

```text
"destructuring pattern expected a 2-element Tuple"
```

The numeric object handles are allocation-run artifacts. They are neither language identities nor portable bytecode constants.

### 5.2 Constant indexes are chunk-local

A constant-pool index is meaningful only in the chunk that owns the pool.

For example:

```text
Constant(7)
```

means:

```text
push constants[7]
```

which is `2` in this chunk.

It does not mean that constant index `7` globally denotes the integer `2`.

### 5.3 Duplicate selector constants

The pool contains repeated selector values:

```text
[4]  Symbol(new(_))
[10] Symbol(new(_))

[5]  Symbol(raise())
[11] Symbol(raise())

[13] Symbol(at(_))
[16] Symbol(at(_))
```

The reference `Chunk::add_constant` operation appends a value and returns the new index. The chunk constant pool is therefore not, by itself, a canonical deduplicating table.

The selector values may refer to the same interned symbol even though they occupy different constant-pool indexes.

---

# Part II — Initializer construction and scratch storage

## 6. Tuple construction: instructions 0000–0002

The first three instructions are:

```text
0000  Constant(0)
0001  Constant(1)
0002  BuildTuple { positional: 2, labeled: 0 }
```

### 6.1 `Constant(0)`

Pushes:

```text
constants[0] = 5
```

Stack:

```text
[] -> [5]
```

### 6.2 `Constant(1)`

Pushes:

```text
constants[1] = 6
```

Stack:

```text
[5] -> [5, 6]
```

### 6.3 `BuildTuple { positional: 2, labeled: 0 }`

The VM consumes two positional values and zero labeled associations and constructs the canonical tuple product.

Conceptually:

```text
[5, 6] -> [(5, 6)]
```

The VM implementation pops positional values from the operand stack, reverses the temporary collection to restore source order, completes tuple construction through the runtime product machinery, and pushes the resulting tuple value.

The critical semantic property is that the tuple literal is constructed directly through a VM tuple-building instruction. It is not compiled as a normal source-level send equivalent to:

```phalcom
Tuple.new(5, 6)
```

The resulting value is the native tuple/product value:

```text
(5, 6)
```

---

## 7. Claiming the initializer: instructions 0003–0004

The next instructions are:

```text
0003  ReserveScratchLocal(0)
0004  SetLocal(0)
```

Structured destructuring requires the initializer value to remain available across:

- the class check;
- the arity check;
- the first element read;
- the second element read.

The compiler therefore stores the already-evaluated initializer in hidden local slot `0`.

### 7.1 `ReserveScratchLocal(0)`

This instruction creates compiler-managed local storage at slot `0`.

Scratch locals are not source-visible bindings. They exist to satisfy compiler lowering requirements while preserving source semantics.

In the current VM, reserving a scratch local inserts an initially absent/private value into the frame's local area. Because local storage lives in the same underlying stack structure as other activation data, the VM also adjusts open-upvalue bookkeeping when insertion moves an open captured slot.

The important semantic effect here is:

```text
reserve local[0]
```

### 7.2 `SetLocal(0)`

`SetLocal` stores the current stack-top value into the local slot without consuming that top value.

Its stack effect is:

```text
[..., value] -> [..., value]
```

After instruction 0004:

```text
local[0] = (5, 6)
```

The destructuring initializer has therefore been evaluated exactly once and can be reused by all following operations.

For a source expression such as:

```phalcom
let (a, b) = makeTuple()
```

the function `makeTuple()` must execute once, regardless of the number of destructuring projections.

---

# Part III — Required tuple-class validation

## 8. Overview of the class-guard region

Instructions 0005–0015 validate that the destructured value is exactly a `Tuple`:

```text
0005  InvokeLocal(0, 0, class)
0006  [shadowed dead slot] Invoke(0, 2)
0007  GetLinked(99)
0008  Same
0009  JumpIfFalse(1)
0010  Jump(5)
0011  GetLinked(22)
0012  InvokeConst(3, 1, new(_))
0013  [shadowed dead slot] Invoke(1, 4)
0014  Invoke(raise(), 0)
0015  Pop
```

The semantic intent is:

```text
if not (local[0].class same Tuple):
    Error.new("pattern expected Tuple").raise()
```

This is an exact-class check, not a structural protocol test and not an `isA(Tuple)` subclass test.

---

## 9. `InvokeLocal(0, 0, class)`

Instruction 0005 is:

```text
InvokeLocal(0, 0, class)
```

This is a fused superinstruction.

Before fusion, the compiler emitted:

```text
0005  GetLocal(0)
0006  Invoke(0, 2)
```

with:

```text
constants[2] = Symbol(class)
```

The semantic equivalence is:

```text
InvokeLocal(slot, n, selector)
≈ GetLocal(slot)
  Invoke(n, selector)
```

Thus instruction 0005 means:

```phalcom
local[0].class
```

and produces the runtime class object for `(5, 6)`, namely `Tuple`.

---

## 10. Shadowed dead slot 0006

The next printed position is:

```text
0006  [shadowed dead slot] Invoke(0, 2)
```

This instruction is physically present in `chunk.code`.

It is not merely a comment synthesized from historical compiler information.

### 10.1 Why the dead slot remains

`Chunk::fuse_superinstructions()` rewrites adjacent pairs such as:

```text
GetLocal(slot)
Invoke(n, selector)
```

into:

```text
InvokeLocal(slot, n, selector)
Invoke(n, selector)          # now shadowed
```

and similarly:

```text
Constant(k)
Invoke(n, selector)
```

into:

```text
InvokeConst(k, n, selector)
Invoke(n, selector)          # now shadowed
```

The pass deliberately does not compact the instruction vector.

This preserves:

- every previously calculated branch offset;
- every source-span index;
- every inline-cache position;
- every global-cache position;
- every other instruction-indexed side table.

The fused instruction advances the caller's instruction pointer over the original `Invoke`, so the shadowed second slot is never dispatched.

### 10.2 Why fusion is skipped at a branch target

Fusion is valid only when the second instruction of the pair cannot be entered independently.

If another branch targets `p + 1`, the original `Invoke` remains a reachable instruction and the pair must not be fused.

The fusion pass therefore computes branch targets and refuses to fuse a pair whose second slot is a control-flow entry point.

### 10.3 Cache and diagnostic ownership

The shadowed `Invoke` position remains important even though it is not executed.

For fused `InvokeLocal` and `InvokeConst`, the VM attributes the send to the original `Invoke` slot at `ip + 1`. This lets the fused instruction reuse the same:

- inline-cache slot;
- source span;
- send-site identity

that the unfused pair would have used.

Thus the dead slot is an intentional part of the current physical chunk representation.

---

## 11. `GetLinked(99)`: loading `Tuple`

Instruction 0007 is:

```text
GetLinked(99)
```

This does not index the constant pool.

The compiler's global-reference lowering may resolve canonical universe/prelude bindings into the active module's linked-read table. In this chunk, linked slot `99` resolves to the canonical `Tuple` binding.

Semantically:

```text
linked[99] -> Tuple
```

The numeric value `99` is not a globally assigned class ID.

Another compilation environment may assign a different linked-read index to the same semantic binding.

A specification example should therefore annotate this as:

```text
GetLinked(99)   # Tuple in this chunk
```

rather than implying:

```text
Tuple == linked slot 99
```

---

## 12. `Same`: exact runtime class comparison

At instruction 0008:

```text
Same
```

the relevant stack tail is:

```text
[..., actual_class, Tuple]
```

`Same` pops both values and pushes:

```text
semantic_same(actual_class, Tuple)
```

The VM delegates the operation to `VM::semantic_same`.

For class objects in this context, the comparison behaves as exact class-object identity. Therefore the destructuring requirement is effectively:

```text
local[0].class is exactly Tuple
```

This is stronger than:

```phalcom
local[0].isA(Tuple)
```

and stronger than protocol compatibility such as merely responding to `size` and `at(_)`.

The source binding is therefore guarded by exact tuple kind before arity and projection proceed.

---

# Part IV — Class-guard control flow and error construction

## 13. Relative branch convention

Phalcom branch offsets are instruction-count offsets relative to the instruction pointer after the branch instruction has been fetched and the instruction pointer has advanced.

For a branch instruction at position `ip0` with offset `d`:

```text
target = (ip0 + 1) + d
```

The compiler backpatching helper implements the same rule:

```text
offset = target - (branch_index + 1)
```

This convention explains all jump values in the example.

---

## 14. `JumpIfFalse(1)` at 0009

Instruction:

```text
0009  JumpIfFalse(1)
```

tests the result of:

```text
local[0].class same Tuple
```

Its target is:

```text
(9 + 1) + 1 = 11
```

Therefore:

- if the comparison is `false`, control transfers to instruction `0011`, the error path;
- if the comparison is `true`, execution falls through to `0010`.

---

## 15. `Jump(5)` at 0010

Instruction:

```text
0010  Jump(5)
```

has target:

```text
(10 + 1) + 5 = 16
```

Thus a successful class check skips the generated error block and resumes at the arity check.

The control flow is:

```text
                         Same
                          |
                    class matches?
                    /            \
                  no              yes
                  |                |
          JumpIfFalse(+1)        fallthrough
                  |                |
                  v                v
                0011            0010 Jump(+5)
             error path             |
                                     v
                                    0016
```

---

## 16. Class-mismatch error path: 0011–0015

The failure block is:

```text
0011  GetLinked(22)
0012  InvokeConst(3, 1, new(_))
0013  [shadowed dead slot] Invoke(1, 4)
0014  Invoke(raise(), 0)
0015  Pop
```

In this chunk:

```text
linked[22] -> Error
constants[3] -> "pattern expected Tuple"
constants[4] -> Symbol(new(_))
constants[5] -> Symbol(raise())
```

The semantic operation is:

```phalcom
Error.new("pattern expected Tuple").raise()
```

### 16.1 `GetLinked(22)`

Loads the canonical `Error` class through the linked-read table.

As with `Tuple`, linked slot `22` is chunk/environment-specific.

### 16.2 `InvokeConst(3, 1, new(_))`

This instruction is one of the most important details in the example.

Before fusion, instructions 0012–0013 were:

```text
0012  Constant(3)
0013  Invoke(1, 4)
```

The stack already contains the `Error` receiver from instruction 0011.

Immediately before the send, the stack tail is therefore:

```text
[..., Error, "pattern expected Tuple"]
```

For an invocation of arity `1`, the VM locates the receiver at:

```text
receiver_index = stack.len() - 1 - arity
```

Therefore:

- `Error` is the receiver;
- constant `3`, the message string, is the argument.

Fusion rewrites only the constant-load instruction:

```text
Constant(3)
Invoke(1, new(_))
```

into:

```text
InvokeConst(3, 1, new(_))
[shadowed dead Invoke]
```

The exact semantic equivalence is:

```text
InvokeConst(k, n, selector)
≈ Constant(k)
  Invoke(n, selector)
```

### 16.3 Critical rule: the constant is not necessarily the receiver

`InvokeConst` MUST NOT be interpreted as:

> invoke `selector` on constant `k`.

Its meaning is:

> perform the original `Constant(k); Invoke(...)` pair as one interpreter dispatch.

The pushed constant occupies the same stack position that the eliminated `Constant` instruction would have occupied.

Depending on the pre-existing stack and call arity, that constant may be:

- the receiver;
- a positional argument;
- a labeled argument contribution in a statically arranged call window.

In this example it is an argument.

This distinction is specification-significant.

### 16.4 `Invoke(raise(), 0)`

The newly constructed `Error` is then the receiver of:

```phalcom
error.raise()
```

Normal execution does not return through this path.

### 16.5 Unreachable `Pop`

Instruction 0015 is:

```text
Pop
```

The compiler intentionally emits this after the non-returning raise sequence to preserve the static stack-balance shape of generated bytecode.

It is unreachable if `raise()` performs its required abrupt control transfer.

---

# Part V — Exact arity validation

## 17. Arity-check region

After a successful class check, execution continues at:

```text
0016  InvokeLocal(0, 0, size)
0017  [shadowed dead slot] Invoke(0, 6)
0018  InvokeConst(7, 1, !=(_))
0019  [shadowed dead slot] Invoke(1, 8)
0020  JumpIfFalse(5)
0021  GetLinked(22)
0022  InvokeConst(9, 1, new(_))
0023  [shadowed dead slot] Invoke(1, 10)
0024  Invoke(raise(), 0)
0025  Pop
```

This computes a **mismatch predicate**:

```phalcom
local[0].size != 2
```

The compiler deliberately expresses failure as `true` and success as `false`.

---

## 18. `InvokeLocal(0, 0, size)`

This is equivalent to:

```text
GetLocal(0)
Invoke(0, size)
```

For the tuple `(5, 6)`:

```text
local[0].size -> 2
```

The public `Tuple.size` method delegates to the tuple's internal native size primitive.

---

## 19. `InvokeConst(7, 1, !=(_))`

Constant `7` is:

```text
2
```

Before fusion, this pair was:

```text
Constant(7)
Invoke(1, !=(_))
```

The receiver is the previously produced size value and the constant is the argument.

Thus the send is:

```phalcom
local[0].size != 2
```

For `(5, 6)`:

```text
2 != 2 -> false
```

The emitted predicate is intentionally phrased as:

```text
true  => mismatch
false => valid arity
```

---

## 20. `JumpIfFalse(5)` at 0020

The target is:

```text
(20 + 1) + 5 = 26
```

Therefore:

- `false` means the tuple has exactly two elements and jumps directly to extraction at 0026;
- `true` means the tuple has the wrong size and falls through to the error block at 0021.

Control flow:

```text
local[0].size
      |
      v
    != 2
      |
   mismatch?
   /       \
 yes       no
  |         |
  v         v
0021      JumpIfFalse(+5)
error        |
             v
            0026
```

---

## 21. Arity-mismatch error path

The generated error sequence is:

```text
0021  GetLinked(22)
0022  InvokeConst(9, 1, new(_))
0023  [shadowed dead slot] Invoke(1, 10)
0024  Invoke(raise(), 0)
0025  Pop
```

Constant `9` is:

```text
"destructuring pattern expected a 2-element Tuple"
```

Semantically:

```phalcom
Error.new(
    "destructuring pattern expected a 2-element Tuple"
).raise()
```

This runtime validation is what prevents a destructuring binding from silently truncating, silently ignoring missing elements, or proceeding into invalid indexed reads.

---

# Part VI — Element projection and global definition

## 22. Extracting `a`: instructions 0026–0029

The first binding is emitted as:

```text
0026  GetLocal(0)
0027  InvokeConst(12, 1, at(_))
0028  [shadowed dead slot] Invoke(1, 13)
0029  DefineGlobal(14)
```

The relevant constants are:

```text
constants[12] = 0
constants[13] = Symbol(at(_))
constants[14] = Symbol(a)
```

### 22.1 `GetLocal(0)`

Pushes the tuple:

```text
(5, 6)
```

### 22.2 `InvokeConst(12, 1, at(_))`

Before fusion:

```text
Constant(12)      # 0
Invoke(1, at(_))
```

The tuple from 0026 is the receiver and integer `0` is the argument.

Semantically:

```phalcom
local[0].at(0)
```

The compiler helper responsible for positional pattern projection emits exactly the logical sequence:

```text
GetLocal(value_slot)
Constant(index)
Invoke at(_)
```

### 22.3 `Tuple.at(_)` returns the element directly

The public universe definition of `Tuple.at(_)` delegates directly to the internal raw tuple index operation:

```phalcom
at(_ i) { self._$at(i) }
```

Therefore:

```phalcom
(5, 6).at(0)
```

produces:

```text
5
```

not:

```text
Some(5)
```

Optional lookup behavior belongs to the tuple's `get(_)` protocol, not this direct pattern projection.

The earlier exact class and exact arity checks ensure that the destructuring projection is valid.

### 22.4 `DefineGlobal(14)`

Constant `14` is `Symbol(a)`.

The VM defines the top stack value in the active module:

```text
module.global[a] = 5
```

and consumes that initializer value from the operand stack.

The first user-visible binding is now complete.

---

## 23. Extracting `b`: instructions 0030–0033

The second binding is symmetrical:

```text
0030  GetLocal(0)
0031  InvokeConst(15, 1, at(_))
0032  [shadowed dead slot] Invoke(1, 16)
0033  DefineGlobal(17)
```

with:

```text
constants[15] = 1
constants[16] = Symbol(at(_))
constants[17] = Symbol(b)
```

Semantically:

```phalcom
local[0].at(1)
```

produces:

```text
6
```

and:

```text
DefineGlobal(b)
```

establishes:

```text
module.global[b] = 6
```

At this point the observable module state includes:

```text
a = 5
b = 6
```

---

# Part VII — Root completion

## 24. `Nil` and `Return`

The final instructions are:

```text
0034  Nil
0035  Return
```

The VM instruction name `Nil` is historical implementation nomenclature. At the language boundary its effect is surface absence:

```text
None
```

It must not expose the VM's private uninitialized-storage sentinel as a user-visible value.

The root compilation unit has no remaining source expression result after defining its globals, so the compiler emits an absent result and returns it.

Semantically:

```text
root result = None
```

The destructuring scratch local ceases to exist with the root activation.

The tuple initializer is therefore not the root callable's result.

---

# Part VIII — Complete execution trace

## 25. High-level state trace

A compact execution trace is:

| Phase | Relevant state |
|---|---|
| start | empty operand state for source expression |
| `Constant(0)` | `5` pushed |
| `Constant(1)` | `6` pushed |
| `BuildTuple` | `(5, 6)` constructed |
| `ReserveScratchLocal(0)` | hidden destructuring slot reserved |
| `SetLocal(0)` | `local[0] = (5, 6)` |
| `local[0].class` | produces `Tuple` |
| `GetLinked(99)` | loads canonical `Tuple` |
| `Same` | exact tuple-class test |
| class success | error block skipped |
| `local[0].size` | produces `2` |
| `2 != 2` | produces `false` |
| `JumpIfFalse(5)` | successful arity path jumps to extraction |
| `local[0].at(0)` | produces `5` |
| `DefineGlobal(a)` | `a = 5` |
| `local[0].at(1)` | produces `6` |
| `DefineGlobal(b)` | `b = 6` |
| `Nil` | pushes surface `None` |
| `Return` | root callable returns `None` |

---

## 26. Complete control-flow graph

```text
                +----------------------+
                | Constant 5           |
                | Constant 6           |
                | BuildTuple(2, 0)     |
                +----------+-----------+
                           |
                           v
                +----------------------+
                | Reserve scratch[0]   |
                | scratch[0] = tuple   |
                +----------+-----------+
                           |
                           v
                +----------------------+
                | scratch[0].class     |
                | linked Tuple         |
                | Same                 |
                +----------+-----------+
                           |
                   exact Tuple?
                    /       \
                  no         yes
                  |           |
                  v           v
        +----------------+   Jump over
        | linked Error   |   class-error block
        | error message  |        |
        | new(_)         |        v
        | raise()        |  +----------------------+
        +----------------+  | scratch[0].size      |
                            | push 2               |
                            | !=(_)                |
                            +----------+-----------+
                                       |
                                  wrong arity?
                                  /          \
                                yes          no
                                |             |
                                v             v
                      +----------------+    Jump to
                      | linked Error   |    extraction
                      | error message  |       |
                      | new(_)         |       v
                      | raise()        |  +----------------------+
                      +----------------+  | scratch[0].at(0)     |
                                          | DefineGlobal(a)      |
                                          +----------+-----------+
                                                     |
                                                     v
                                          +----------------------+
                                          | scratch[0].at(1)     |
                                          | DefineGlobal(b)      |
                                          +----------+-----------+
                                                     |
                                                     v
                                          +----------------------+
                                          | Nil / None           |
                                          | Return               |
                                          +----------------------+
```

---

# Part IX — Compiler-emitted form versus final fused form

## 27. Pre-fusion logical bytecode

Before `Chunk::fuse_superinstructions()`, the relevant compiler-emitted sequence is approximately:

```text
0000  Constant(0)
0001  Constant(1)
0002  BuildTuple { positional: 2, labeled: 0 }

0003  ReserveScratchLocal(0)
0004  SetLocal(0)

0005  GetLocal(0)
0006  Invoke(0, 2)            # class
0007  GetLinked(99)           # Tuple
0008  Same
0009  JumpIfFalse(1)
0010  Jump(5)

0011  GetLinked(22)           # Error
0012  Constant(3)             # "pattern expected Tuple"
0013  Invoke(1, 4)            # new(_)
0014  Invoke(0, 5)            # raise()
0015  Pop

0016  GetLocal(0)
0017  Invoke(0, 6)            # size
0018  Constant(7)             # 2
0019  Invoke(1, 8)            # !=(_)
0020  JumpIfFalse(5)

0021  GetLinked(22)           # Error
0022  Constant(9)             # "destructuring pattern expected a 2-element Tuple"
0023  Invoke(1, 10)           # new(_)
0024  Invoke(0, 11)           # raise()
0025  Pop

0026  GetLocal(0)
0027  Constant(12)            # 0
0028  Invoke(1, 13)           # at(_)
0029  DefineGlobal(14)        # a

0030  GetLocal(0)
0031  Constant(15)            # 1
0032  Invoke(1, 16)           # at(_)
0033  DefineGlobal(17)        # b

0034  Nil
0035  Return
```

This form is often easier to read because every ordinary send has the canonical stack-preparation shape visible immediately before it.

---

## 28. Fusion transformation

The final peephole pass recognizes:

```text
GetLocal(slot)
Invoke(n, selector)
```

and rewrites the first position to:

```text
InvokeLocal(slot, n, selector)
```

Likewise:

```text
Constant(k)
Invoke(n, selector)
```

becomes:

```text
InvokeConst(k, n, selector)
```

The original `Invoke` remains at the following instruction position as a shadowed dead slot.

The transformation for this example is therefore:

```text
GetLocal(0)
Invoke(0, class)
```

to:

```text
InvokeLocal(0, 0, class)
[shadowed] Invoke(0, class)
```

and:

```text
Constant(7)
Invoke(1, !=(_))
```

to:

```text
InvokeConst(7, 1, !=(_))
[shadowed] Invoke(1, !=(_))
```

The optimization saves interpreter dispatches without changing bytecode layout.

---

## 29. Why the pass does not compact code

A physically compacted transformation such as:

```text
GetLocal(0)
Invoke(...)
```

becoming a single instruction and deleting the second position would require relayout of the whole chunk.

That would entail at least:

- recomputing every branch offset whose source or target lies after the deletion;
- moving source-span entries;
- moving inline-cache entries;
- moving global-cache entries;
- updating any other instruction-indexed side metadata.

The current design instead pays the memory cost of an unreachable dead `Invoke` slot to keep fusion local, cheap, and index-preserving.

This is an implementation optimization, but because the reference disassembler exposes the dead slots and the VM deliberately attributes fused sends to them, the convention is relevant to an implementation-facing VM specification.

---

# Part X — Important semantic distinctions exposed by the example

## 30. `InvokeConst` is positional stack fusion, not “constant receiver invocation”

The most important interpretive rule in this document is:

```text
InvokeConst(k, n, selector)
≈ Constant(k)
  Invoke(n, selector)
```

The constant is inserted exactly where the removed `Constant(k)` instruction would have inserted it.

It is **not** inherently the receiver.

For ordinary static invocation, the VM determines the receiver from the complete call window:

```text
receiver_index = stack.len() - 1 - arity
```

Consequently, in this example:

```text
InvokeConst(3, 1, new(_))
```

means:

```text
[..., Error]
push constants[3]
invoke arity 1
```

so:

```text
receiver = Error
argument = "pattern expected Tuple"
```

Likewise:

```text
InvokeConst(7, 1, !=(_))
```

uses:

```text
receiver = tuple size
argument = 2
```

and:

```text
InvokeConst(12, 1, at(_))
```

uses:

```text
receiver = tuple
argument = 0
```

Any specification wording that defines `InvokeConst` as “load the referenced constant as receiver and invoke” is therefore too narrow and incorrect.

---

## 31. Required destructuring performs an explicit class check

This `let` binding follows the required/irrefutable destructuring path.

The emitted sequence explicitly computes:

```text
value.class same Tuple
```

and raises:

```phalcom
Error.new("pattern expected Tuple").raise()
```

on failure.

Therefore a non-`Tuple` operand does not merely proceed to `size` or `at(_)` and fail through an eventual message-not-understood path.

That distinction should be preserved in specification prose describing current required destructuring behavior.

---

## 32. Exact class identity versus inheritance

The use of:

```text
value.class
Tuple
Same
```

means this tuple pattern checks the runtime class object directly.

The example does not use:

```phalcom
value.isA(Tuple)
```

and therefore does not express a general subtype acceptance rule.

This exact-kind behavior is an important property of the current lowering.

---

## 33. Tuple pattern projection uses ordinary public protocol sends

The compiler does not emit a dedicated bytecode such as:

```text
GetTupleElement(0)
```

for these pattern leaves.

Instead it emits ordinary selector dispatch:

```phalcom
tuple.at(0)
tuple.at(1)
```

Similarly, it obtains arity through:

```phalcom
tuple.size
```

The VM has a dedicated instruction for tuple construction, but destructuring observation substantially reuses the ordinary object/message model.

This illustrates an important Phalcom architectural principle: compiler-generated behavior frequently uses the same selector semantics as source-written operations instead of bypassing the language object model with special-purpose bytecodes.

---

# Part XI — Stable semantics versus incidental printed artifacts

## 34. Normatively meaningful elements

The following aspects of the example are semantically meaningful for the current VM model:

| Element | Meaning |
|---|---|
| `Constant`, `BuildTuple`, `GetLocal`, `Same`, `Jump*`, `DefineGlobal`, `Nil`, `Return` | canonical VM operations |
| selector identities such as `class`, `size`, `!=(_)`, `new(_)`, `raise()`, `at(_)` | message-send semantics |
| tuple shape `positional: 2, labeled: 0` | product-construction shape |
| relative branch-offset convention | VM control-flow semantics |
| scratch-local use | concrete compiler lowering |
| `InvokeLocal` / `InvokeConst` | current fused VM instructions |
| shadowed second slot | current fusion/chunk-layout convention |
| error messages | current compiler-generated diagnostics |
| module-global binding of `a` and `b` | source-observable binding behavior |

---

## 35. Incidental or environment-specific elements

The following printed values must not be interpreted as globally stable identities.

### 35.1 Heap object references

Examples:

```text
ObjRef(3785v1)
ObjRef(3786v1)
```

These are heap handles from a particular execution/bootstrap allocation history.

They are not portable object IDs.

### 35.2 Linked-read indexes

Examples:

```text
GetLinked(22)
GetLinked(99)
```

In this chunk they resolve to:

```text
22 -> Error
99 -> Tuple
```

but these numeric positions are linker/module-environment coordinates, not permanent language assignments.

### 35.3 Constant-pool indexes

For example:

```text
13 -> Symbol(at(_))
```

is true only for this chunk.

A compiler that emits equivalent semantics may organize constants differently.

### 35.4 Object debug rendering

The fact that string constants appear as generic object references reflects the disassembler's current presentation policy, not the language representation of strings.

A future disassembler may choose to print:

```text
[3] "pattern expected Tuple"
```

without changing bytecode semantics.

---

# Part XII — Architectural reading of the example

## 36. The source/VM boundary

This single declaration shows three distinct layers interacting.

### 36.1 Source semantics

The source language specifies a destructuring binding:

```phalcom
let (a, b) = (5, 6)
```

with single initializer evaluation and two resulting bindings.

### 36.2 Compiler lowering

The compiler chooses to implement this through:

- direct tuple construction;
- one scratch local;
- class and arity guards;
- generated `Error` paths;
- ordinary message sends for observations;
- module-global definitions.

Another bytecode producer could in principle produce a semantically equivalent sequence, subject to canonical VM constraints.

### 36.3 VM execution

The VM executes the resulting stack program, including:

- stack manipulation;
- linked reads;
- message dispatch;
- relative branches;
- scratch local storage;
- tuple construction;
- global definition;
- return.

The later fusion pass changes dispatch efficiency without changing source-observable semantics.

---

## 37. Why this is a strong specification example

A trivial example such as:

```phalcom
42
```

would explain `Constant` and `Return`, but little else.

This destructuring example demonstrates, in one short source line:

- product construction;
- compiler-generated storage;
- object model reflection;
- exact identity testing;
- dynamic dispatch;
- failure control flow;
- generated exception construction;
- linker interaction;
- module binding;
- optimization;
- cache-site preservation;
- disassembler conventions.

It therefore makes a strong companion example for the VM specification.

---

# Part XIII — Specification requirements highlighted by this example

## 38. Requirement: define `InvokeLocal` by equivalence

The bytecode specification should define:

```text
InvokeLocal(slot, n, selector)
≈ GetLocal(slot)
  Invoke(n, selector)
```

while separately documenting the physical shadow-slot convention used by the reference implementation.

---

## 39. Requirement: define `InvokeConst` by equivalence

The bytecode specification should define:

```text
InvokeConst(k, n, selector)
≈ Constant(k)
  Invoke(n, selector)
```

It should explicitly state:

> The referenced constant occupies the same call-window position that the eliminated `Constant(k)` instruction would have occupied. It is not inherently the receiver.

This clause prevents a misleading interpretation of the opcode name.

---

## 40. Requirement: describe shadowed send slots

The reference VM's fusion contract should state that:

1. fusion rewrites the first instruction in place;
2. the original `Invoke` at `p + 1` remains physically present;
3. the fused instruction skips that slot during execution;
4. fusion is prohibited when the second slot is an independent branch target;
5. the second slot remains the logical send-site position for cache/span attribution.

Even if future implementations are allowed to use a different physical strategy, this describes the reference chunk/disassembler contract accurately.

---

## 41. Requirement: distinguish linked indexes from semantic identities

Worked disassemblies should annotate:

```text
GetLinked(99)  # Tuple in this chunk
GetLinked(22)  # Error in this chunk
```

and state that the indexes are link-environment-specific.

The printed number is not the identity of the linked object.

---

## 42. Requirement: distinguish internal object handles from constant semantics

When the disassembler prints:

```text
<obj ObjRef(...)>
```

a worked specification example should provide the semantic constant where known.

For this example:

```text
constant[3] = "pattern expected Tuple"
constant[9] = "destructuring pattern expected a 2-element Tuple"
```

This keeps the explanation independent of run-specific heap handles.

---

# Part XIV — Condensed annotated disassembly

## 43. Fully annotated listing

The entire final disassembly can be summarized as follows:

```text
<main> slots=1 upvalues=0
# slot 0 is a compiler-generated destructuring temporary.

constants:
  [0]  5
  [1]  6
  [2]  class
  [3]  "pattern expected Tuple"
  [4]  new(_)
  [5]  raise()
  [6]  size
  [7]  2
  [8]  !=(_)
  [9]  "destructuring pattern expected a 2-element Tuple"
  [10] new(_)
  [11] raise()
  [12] 0
  [13] at(_)
  [14] a
  [15] 1
  [16] at(_)
  [17] b

0000 Constant(0)                            # push 5
0001 Constant(1)                            # push 6
0002 BuildTuple { positional: 2, labeled: 0 } # -> (5, 6)

0003 ReserveScratchLocal(0)                 # reserve $destructure
0004 SetLocal(0)                            # local[0] = (5, 6)

0005 InvokeLocal(0, 0, class)               # local[0].class
0006 [shadowed] Invoke(0, 2)
0007 GetLinked(99)                          # Tuple in this chunk
0008 Same                                   # exact class identity
0009 JumpIfFalse(1)                         # mismatch -> 0011
0010 Jump(5)                                # success -> 0016

0011 GetLinked(22)                          # Error in this chunk
0012 InvokeConst(3, 1, new(_))              # Error.new(message)
0013 [shadowed] Invoke(1, 4)
0014 Invoke(raise(), 0)                     # raise class mismatch
0015 Pop                                    # unreachable balancing pop

0016 InvokeLocal(0, 0, size)                # local[0].size
0017 [shadowed] Invoke(0, 6)
0018 InvokeConst(7, 1, !=(_))               # size != 2
0019 [shadowed] Invoke(1, 8)
0020 JumpIfFalse(5)                         # false => correct size => 0026

0021 GetLinked(22)                          # Error
0022 InvokeConst(9, 1, new(_))              # Error.new(arity message)
0023 [shadowed] Invoke(1, 10)
0024 Invoke(raise(), 0)
0025 Pop

0026 GetLocal(0)                            # tuple receiver
0027 InvokeConst(12, 1, at(_))              # tuple.at(0)
0028 [shadowed] Invoke(1, 13)
0029 DefineGlobal(14)                       # a = 5

0030 GetLocal(0)                            # tuple receiver
0031 InvokeConst(15, 1, at(_))              # tuple.at(1)
0032 [shadowed] Invoke(1, 16)
0033 DefineGlobal(17)                       # b = 6

0034 Nil                                    # surface None
0035 Return
```

---

# Part XV — Final semantic summary

## 44. Equivalent semantic execution

Ignoring implementation-specific optimization details, the entire bytecode program performs:

```text
tmp = Tuple(5, 6)

if tmp.class is not exactly Tuple:
    raise Error("pattern expected Tuple")

if tmp.size != 2:
    raise Error("destructuring pattern expected a 2-element Tuple")

define module global a = tmp.at(0)
define module global b = tmp.at(1)

return None
```

The final observable bindings are:

```text
a = 5
b = 6
```

and the root callable completes with:

```text
None
```

---

## 45. Key invariants demonstrated

This worked example establishes the following implementation facts for the audited reference VM:

1. A tuple literal is constructed with `BuildTuple`, not an ordinary `Tuple.new(...)` send.
2. Structured destructuring evaluates its initializer once and retains it in compiler scratch storage.
3. Module-level destructuring leaves become module globals.
4. Required tuple destructuring performs an explicit exact-class check.
5. Exact tuple arity is checked at runtime before projection.
6. Pattern projection uses ordinary `at(_)` sends.
7. Generated mismatch failures use ordinary `Error.new(...).raise()` behavior.
8. Linked-read indexes are chunk/module linkage coordinates, not global object identifiers.
9. `InvokeLocal` is semantically `GetLocal + Invoke`.
10. `InvokeConst` is semantically `Constant + Invoke`; the constant is not inherently the receiver.
11. Fused sends retain the original `Invoke` as a shadowed dead slot.
12. The shadowed slot remains relevant for send-site cache and source attribution.
13. Relative branch offsets are measured from the already-advanced instruction pointer.
14. The root compilation unit finishes with surface absence, `None`.

These properties together make the example suitable as a canonical explanatory companion to the Phalcom VM bytecode specification.
