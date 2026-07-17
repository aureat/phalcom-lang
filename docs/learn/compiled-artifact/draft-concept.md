# The Compiled Representation of a Function

The textbook picture of a closure is a single heap object: a pair of code and
environment, `⟨code, env⟩`, drawn as one box with two fields. Every
introductory treatment of closures — SICP's environment-model diagrams, most
"how closures work" blog posts — draws it this way, and for reasoning about
*semantics* the picture is completely adequate. Lexical scoping says a
function value must carry the bindings it closed over; a box with a code
pointer and an environment pointer says exactly that and no more.

Real implementations almost never build that box literally. The moment a
function literal can be evaluated more than once against different captured
state — a closure minted inside a loop, inside a recursive call, inside any
site the compiler cannot guarantee executes exactly once — the one-box
picture forces work that is wasteful in a specific, structural way: it forces
either recompiling or recopying the *code* every time you only meant to mint
new *state*. The code — the instruction sequence, the literal values it
references, the shape of what it captures — does not change between one
evaluation of the function literal and the next. Only the captured values
change. A representation that fuses the two into one object has no way to
express "this part is invariant across N instances, this part varies per
instance" — every instantiation pays for both, and nothing in the object
graph tells you that 1000 instances share 99% of their bits.

So the pressure that forces a split is not a semantic pressure — the
one-box model is semantically sufficient — it is an economy-of-representation
pressure: **one immutable code body, N simultaneously live closures over it,
each capturing different cells.** Once you say that sentence precisely, the
box wants to become two boxes: one that is compiled once and shared, one that
is minted once per evaluation and holds only what actually varies.

## Where this pressure comes from, historically

The identification of this problem predates bytecode VMs by decades and has
a name: the **FUNARG problem**, from early Lisp implementations that
represented a function argument (a "funarg") as a bare pointer into the
interpreter's environment structure — typically the current stack frame or
an association-list threaded through recursive calls. This works if the
function value is only ever *called downward*, within the dynamic extent of
its creation (a "downward funarg": passed as an argument, used, discarded
before the creator returns). It breaks the instant the function value
escapes upward — returned from its creating call, stored in a data
structure, invoked after the frame that made it is gone (an "upward
funarg"). The classic articulation of this split, and the argument that the
real issue is not "function values" at all but which storage discipline the
*environment* needs, is usually credited to Joel Moses's MIT AI memo on why
the FUNARG problem should be called the *environment* problem
**[flagged — I'm confident this memo exists and makes this argument; less
confident on reproducing its exact title/number from memory, circa 1970]**.
The fix that generation of Lisps converged on was to heap-allocate the parts
of the environment that a closure could outlive, rather than assume stack
discipline — which is the ancestor of "captured variables live in cells that
outlive the frame," a fact the rest of this document takes for granted.

Scheme's early definition (Sussman and Steele's "lambda papers,"
mid-1970s) made closures fully first-class specifically to make the upward
case ordinary rather than exceptional — a closure had to be a real,
independently-lifetimed heap value, not a stack-frame pointer with a asterisk
attached. That decision is what makes "N live closures over one code body"
a routine situation instead of a corner case: once closures can be returned,
stored, and outlive their creating call freely, a loop or a recursive
function that mints one per iteration is not a pathological program, it is
the *normal* one — map/filter/reduce, event-handler registration, generator
state machines, memoized accumulators, all mint a fresh closure per
call and expect the old ones to keep working. A representation that treats
this as rare is wrong about the actual distribution of programs.

By the time bytecode VMs arrive (rather than tree-walking interpreters over
an AST), a second, independent pressure joins the first: bytecode wants
**fixed-width, compact instructions**, and code, once compiled, is treated
as an immutable artifact you'd like to reuse verbatim rather than
re-derive. A tree-walking interpreter gets code-sharing almost by accident —
closures over the same lambda literal already point at the same AST node,
because nobody copies AST nodes on evaluation. A bytecode compiler has to
*decide* to preserve that property, because compiling is a distinct,
non-trivial phase that produces an artifact (a chunk of bytecode plus
whatever data it references) that is only cheap to reuse if something
deliberately holds onto it and hands out references rather than copies.
That "something" is the shared **code object** — also called a **function
prototype** in the Lua tradition, a name worth adopting because it is more
precise than "code": it names not just the instructions but the whole
static, compile-time-fixed description of the function, instructions and
literal data and capture layout together.

## The design space

None of the three forks below has a universally correct answer; each is a
real, load-bearing design taken by a real, successful implementation. Treat
each as a live option, not a strawman en route to a foregone conclusion.

### Fork (a): one fused object, or a split template + instance?

**The fused branch.** Represent a closure as one heap object holding both
the code (or a pointer to it) and the captured bindings directly, with no
separate "template" type at all. This is not a naive idea — it is exactly
what you get for free in a tree-walking interpreter whose closure is
`⟨params, body-AST, env⟩`: the "code" field is a pointer to a shared,
already-immutable AST node, so sharing happens implicitly through ordinary
pointer aliasing, without the implementation ever needing to *name* "the
template" as its own type. The fusion costs nothing extra there because the
AST was already going to be a persistent, shared, read-only structure
regardless of closures. Fusion becomes actively attractive again whenever a
language's functions are, in practice, mostly 1:1 with their instantiation:
top-level functions, methods that are compiled once and invoked without
being re-instantiated as first-class values, anything closer to C's model
where a function *is* a single static object because there is no notion of
evaluating a function literal at a call site more than once. If most of your
"closures" never have a second live instance, paying for an extra
indirection to a template that only ever has one instance pointing at it is
overhead with no payoff. Fusion is also simply *simpler*: one allocation, one
type, one lifetime to reason about, nothing to keep synchronized between
two objects.

**The split branch.** Give the compiler two distinct output types: a
**code object** (immutable, produced once per function literal, holding
the instructions, the literal data, and a static description of what to
capture) and a **closure** (small, produced once per *evaluation* of that
function literal, holding a reference to the code object plus the actual
captured values for this instantiation). This is the branch that pays off
exactly when fusion doesn't: the moment a function literal is evaluated
more than once with different captures live simultaneously, splitting turns
an O(code size) allocation into an O(number of captured variables)
allocation, every time after the first. It also gives you a natural unit of
**identity for "the function as written"** independent of any particular
closure over it — useful for debugging info, for source-location
attribution, for deciding whether two closures came from "the same"
lambda syntactically (which the fused model can only answer by comparing
code contents, not identity). The cost is real: two types instead of one,
an extra pointer indirection on every call to reach the code from the
closure, and a lifetime question the fused model never has to ask —
*who keeps the template alive once the last closure referencing it is
collected, and does the template need to outlive the closures, or vice
versa* (in practice: the template outlives every closure over it, and
closures hold it alive by reference, not the other way around).

Both branches are real. Lua and CPython split (detailed below). Many small,
purpose-built interpreters — and, historically, the direct-AST-walking
implementations of Lisp and Scheme before compilation to an intermediate
bytecode became standard — fuse, some by explicit design, most simply
because they never had a reason to name the template as a separate thing.

### Fork (b): immediates in the instruction stream, or a side constant pool?

**The immediate branch.** Encode a literal value directly into the operand
bytes of the instruction that uses it: `PUSH_INT 42` carries the `42` as
literal payload in the instruction stream itself, no lookup required. This
is tempting for exactly the reason it looks tempting: it removes an
indirection. Reading the literal costs nothing beyond decoding the
instruction you were going to decode anyway. It is the natural choice for
small, fixed-size values — a byte, a 16-bit int, sometimes a 32-bit int or
float — where "the value" and "an index that finds the value" are the same
number of bits, so there is no compactness argument for a pool at all. Many
real bytecode formats do exactly this for the small numeric case. It's also
simpler to disassemble by hand and simpler to JIT-compile from directly,
since the value is sitting right there in the trace rather than one hop
away.

**The constant-pool branch.** Give the code object a second array — the
**constant pool** — holding literal `Value`s (strings, larger numbers,
symbols/selectors, nested code objects, anything not cheaply representable
in a fixed-width operand), and have the instruction carry only an *index*
into that array. This buys three things immediates cannot: compact,
uniform instruction width regardless of how large or how referency the
literal is (a string constant and a tiny integer constant both cost the
same one or two operand bytes — an index — even though the underlying
values are wildly different sizes); a single place structural literal
identity lives, so the same literal used twice can be *deduplicated* to one
pool slot (a design choice, not a requirement — more below); and, most
important for anything with a moving garbage collector, a single, typed
place the collector can enumerate to find every literal a piece of code
keeps alive. This last point deserves emphasis: once the instruction stream
is raw untyped bytes, a collector cannot safely scan it looking for
pointers — it cannot tell a pointer-shaped operand from an integer operand
of the same width without decoding every instruction's format, which is
exactly the kind of coupling between GC and instruction encoding nobody
wants. A constant pool turns "which of this code's bytes are GC roots" into
"walk this one typed array," decoupled entirely from how the instruction
stream is encoded.

These two branches are not mutually exclusive within a single bytecode
format, and the cleanest evidence of that is the JVM, whose instruction set
does *both* deliberately: `iconst_0`..`iconst_5` bake tiny integer constants
directly into the opcode (zero operand bytes at all — the value *is* the
opcode), `bipush`/`sipush` carry a byte/short immediate operand, and `ldc`
reads an arbitrary value out of the class file's constant pool by index.
The choice of branch is made *per literal*, at compile time, based on
whether the value is small and fixed-width enough to be worth inlining.

### Fork (c): native code as a variant of the code object, or a sibling to the whole stack?

Deliberately left open here — this is a fork worth predicting before you
see how any particular system resolves it, because both answers are used by
real, mature implementations and the choice has different downstream
consequences.

**Native as a variant.** The code-object type itself becomes a small tagged
union: `Bytecode(chunk)` or `Native(fn_ptr)`, and everything above the code
layer — the closure type, the call mechanism, the constant-pool machinery
— is written once against "a code object," blind to which variant it is
holding, until the interpreter's `call` primitive switches on the tag. This
buys uniformity: one closure representation, one code-object slot on a
method table, regardless of what's underneath. It costs a little on every
non-native code object, in principle: the type now carries a discriminant
and (depending on layout) unused fields for whichever variant isn't
present, and the "purely bytecode" reasoning that held everywhere above the
code layer becomes "bytecode, unless it wasn't" — a discipline the compiler
and every consumer of the code object now has to hold in their head. Ruby's
MRI is a real, live example of exactly this shape: a method's
implementation is a tagged representation with variants including one
carrying an `ISEQ` (compiled bytecode) and one carrying a raw C function
pointer for built-ins, dispatched on the tag at call time
**[flagged — moderate confidence on the precise tag/variant names in
current MRI source; high confidence on the general tagged-union shape]**.

**Native as a sibling.** Keep the code-object type pure — it only ever
means "bytecode plus its constant pool plus its capture descriptors,"
nothing else — and put the fork one layer up, at whatever thing decides
*how to invoke a callable at all*: a method or function-value slot that is
itself either "a handle to a bytecode closure" or "a raw native function
pointer," decided before you ever touch a code object, rather than inside
one. This buys the code-object type real purity: every consumer of "a code
object" can assume bytecode, constant pool, capture descriptors, full stop,
with no defensive tag-checking anywhere in that layer. It costs a
duplicated dispatch point: something above the code layer — a method
table, a call-site — now has to carry its own fork between "this callable
is bytecode-backed" and "this callable is a bare function pointer," which
is more or less the same tag, just relocated one layer up and duplicated
across however many call-site shapes exist. CPython leans this way at the
object-type level: a plain Python function (`PyFunctionObject`, wrapping a
code object) and a built-in function (`PyCFunctionObject`, wrapping a raw C
function pointer) are genuinely different C types with different memory
layouts, unified only by both satisfying the "callable" protocol through
their own type's call slot — not by one type internally branching on
"native or not."

Which of these a given system takes has a real consequence worth noticing
without resolving here: variant-of-the-code-object couples "how do I invoke
this" to the code representation itself, so every future code-layer
concern (serialization, disassembly, introspection) has to know native
exists; sibling-to-the-stack keeps the code layer answering exactly one
question, at the cost of pushing the native/bytecode fork to however many
places above it need to make a call.

## The distinguishing program

Consider, in pseudocode, a function that builds a thousand closures in a
loop, each capturing the loop variable:

```
function makeThunks():
    thunks = []
    for i in 0 .. 1000:
        thunks.push(() -> i)          # a closure literal, evaluated once per iteration
    return thunks
```

The lambda `() -> i` is written **once**, at one point in the source, and
compiled **once**, at compile time of `makeThunks`. But it is *evaluated* —
the closure expression is reached and produces a value — one thousand
times, once per loop iteration, and each of those thousand evaluations
must, in a language with per-iteration or mutable-cell capture semantics,
be able to observe a value of `i` independent of the other 999.

Ask concretely, per iteration: what does iteration 743 need that is
identical to what iteration 12 needed, and what does it need that must be
distinct?

**Identical, and shared, across all 1000 iterations:** the instruction
sequence for the body `i` (in this trivial case: "load the captured
variable, return it"); the constant pool of that lambda, if it has one
(empty here, but in general: any literals the lambda body references);
the *description* of what to capture — "capture one variable, which is a
local of the immediately enclosing frame, at slot N" — is fixed at compile
time and is the same description on iteration 1 and iteration 1000, because
it describes the *shape* of the capture, not its value. None of this
differs between iterations; compiling it 1000 times would produce 1000
bit-identical results, so a correct implementation compiles it once.

**Freshly minted, once per iteration:** the closure object itself — the
small record that says "this particular evaluation of `() -> i`, right
now, with these captures" — and, depending on the language's loop-variable
scoping rule, either a fresh cell holding this iteration's value of `i` or
a shared cell all 1000 closures alias. This second detail is exactly where
a well-known family of bugs lives: languages whose `for`-loop reuses one
binding for `i` across all iterations (classic JavaScript `var`, or a naive
implementation of the loop) produce 1000 closures that all observe the
*final* value of `i` after the loop ends, because they share one cell,
not because they share code. Languages that create a fresh binding per
iteration (JavaScript `let`, Swift, Rust's per-iteration `move` closures,
Python's common `def f(i=i):` default-argument workaround for the same
problem) produce 1000 closures with 1000 distinct values, because the
*cell*, not the code, is what got freshly minted each time around.

This is the whole question the template/instance split exists to answer,
made concrete: **the template answers "what is the recipe, and what shape
does its capture have," compiled once; the instance answers "which actual
cells does this one activation of that recipe point at," minted fresh, up
to once per evaluation.** Everything else in the mechanism below is the
detail of how that answer gets implemented.

## The mechanism

### The shared template

The **code object** — code, prototype, function-prototype, chunk,
whichever vocabulary a given system uses — is produced once, at compile
time, and is thereafter treated as immutable by everything that reads it.
It carries, at minimum:

- the **instructions** themselves;
- the **constant pool** (detailed below) — the code object's private array
  of literal `Value`s the instructions index into;
- a static description of what this function, when instantiated, must
  capture: a list of **capture descriptors** (also called **upvalue
  descriptors**, following the Lua vocabulary this document borrows
  throughout). Each descriptor says, for one captured variable, one of two
  things: *"this is a local variable of the frame that is directly
  executing the enclosing code"* (an index into that frame's local slots),
  or *"this is itself an upvalue of the enclosing closure"* — i.e., the
  variable isn't local to the immediately enclosing function either, it's
  something *that* function closed over from further out, so the new
  closure should copy the reference, not re-resolve it against a frame.
  This second case is what makes deeply nested closures (a closure inside
  a closure inside a closure, all capturing the same outer variable) work
  without each level re-walking all the way out to the original frame:
  each level's descriptor says "take it from my immediate parent," and the
  chain does the rest.

Note what is conspicuously *not* here: no captured values. The template
describes the *shape* of capture — how many, and where each one is found
relative to the enclosing activation — never the values themselves. That is
precisely what makes it shareable: a description of shape is compile-time
fixed regardless of how many times or in what states the function is
instantiated; actual values are exactly the part that varies per
instantiation, so they cannot live here without destroying shareability.

### The per-instance object

The **closure** — the thing actually produced by *evaluating* a function
literal — is minted at the point in running code where a `MakeClosure`-style
instruction (or its local equivalent) executes, once per evaluation, and
holds:

- a reference to the shared template — a pointer, handle, or reference-counted
  pointer, small and cheap regardless of how large the underlying code is;
- the **filled captures** — one actual cell reference per capture
  descriptor on the template, resolved *at closure-creation time* by
  walking the descriptor list: for each `Local(slot)` descriptor, take the
  cell currently occupying that slot of the currently-executing frame; for
  each `Upvalue(index)` descriptor, take the cell already sitting at that
  index of the *enclosing* closure's own filled-captures vector (the copy-
  from-parent case above). This is the moment shape becomes value — the
  template's static description turns into a concrete vector of cell
  references, specific to this one instantiation;
- typically, a link to the defining **module** or global namespace the
  closure should resolve free (non-local, non-captured) names against —
  CPython names this explicitly (`__globals__`); it matters because two
  closures over syntactically identical code compiled in different modules
  must still resolve their globals independently, so this link cannot live
  on the shared template either, unless the language guarantees one code
  object is only ever instantiated within one fixed module (some do; then
  this link legitimately moves to the template, and the "always per-
  instance" claim softens to "per-instance unless the language rules it
  out").

Where a captured variable's *cell* lives is a mechanism worth naming
precisely, even briefly: it must be able to outlive the stack frame it
started in, because closures can escape upward (the FUNARG problem again,
now at the mechanism level). Implementations that want to avoid
heap-boxing every local variable "just in case it gets captured" typically
distinguish an **open** upvalue — a cell that still physically lives inside
a live stack frame's slot, being aliased in place, cheap because it needed
no extra allocation — from a **closed** upvalue — the same cell, copied out
to independent heap storage the moment the frame that owned it is about to
die, so the closure's reference stays valid after the frame is gone. This
open/closed distinction is one of Lua's specific, well-documented
contributions to how upvalues get implemented cheaply, and it is a large
enough mechanism in its own right that it deserves its own treatment rather
than a full accounting here — noted as a pointer forward, not developed.

### The identity layer: home-frame stamping (a bridge, not the subject)

A small number of languages need a *third* thing a closure can carry
beyond code-reference and captured cells: an explicit token identifying the
specific activation — the specific live call — that gave birth to it. This
is what a **non-local return** needs: Smalltalk's `^expr` written inside a
block does not return from the block, it returns from the *enclosing
method activation*, wherever the block happens to be running when `^`
executes — which might be nested several calls deep if the block itself
was passed somewhere else and invoked from there. To make that work, a
block value carries a reference to its **home context** — the method
activation it was lexically created inside — established at Smalltalk-80's
`BlockContext`/`MethodContext` split (Goldberg and Robson's description in
the "Blue Book" is the standard reference here **[flagged — moderate
confidence reproducing exact class names from memory; the general
mechanism, a block stamped with the identity of its creating method
activation so `^` can find and unwind to it, is solid]**). Evaluating `^`
means finding that stamped activation and unwinding to it directly, and if
that activation has already returned (the block outlived its home — an
upward funarg that overstayed), the attempt is a runtime error, not silent
misbehavior. The mechanism this needs — some kind of frame or activation
identity a closure can be stamped with at creation and compared against
later — is a distinct concern from either the template/instance split or
the constant pool, worth naming as its own layer, but developed no further
here.

## The constant pool, in depth

The term itself is the JVM's: the class file format's `constant_pool` (Java
Virtual Machine Specification, chapter 4.4) is where the name enters common
usage, though the *idea* — a side table of literal values indexed by a
small operand rather than encoded inline — predates the JVM specification
and shows up wherever a compact instruction format meets literals too large
or too numerous to inline.

The case for a pool over immediates, restated precisely: instruction width.
An instruction set wants every instruction of a given kind to decode in
constant, ideally uniform, time, which favors fixed small operand widths
(a byte or two). Literal values do not respect that constraint — a string
constant, a large integer, a float, a nested code object are all
different sizes, some unbounded. Indexing into a side array converts "an
operand large enough to hold any literal" into "an operand large enough to
hold any *index*," which is a much smaller, much more uniform bound
(bounded by how many distinct literals one code object can reasonably
have, not by how large any one literal can be).

Three real systems, three variations on the same idea:

- **The JVM constant pool** — per class file, holds not just numeric and
  string literals but symbolic references used for linking: class names,
  field and method names paired with type descriptors, interface method
  references. Bytecode operands that need a "name" — `invokevirtual`,
  `getfield`, `new` — carry a constant-pool index, not an inline name
  string, which is also where deduplication becomes practically important:
  a class file referencing the same method name or the same string literal
  many times is expected to reuse one constant-pool entry rather than
  storing the same UTF-8 bytes redundantly per use site. This is a
  compiler/assembler-level choice about how the file is *produced*, not
  something the JVM's runtime semantics require — nothing stops a
  compliant class file from having duplicate, distinct entries for the
  same logical constant, it is simply wasteful and the standard `javac`
  output does not do it.

- **CPython's `co_consts`** — an immutable tuple attached to each code
  object, read by the `LOAD_CONST` opcode via index. CPython additionally
  names the sibling table `co_names` for identifier-like references (global
  variable names, attribute names) used by opcodes like `LOAD_GLOBAL` and
  `LOAD_ATTR`, keeping "names used as names" conceptually separate from
  "arbitrary literal values," even though both are, mechanically,
  index-into-a-side-array. Within a single code object's compilation,
  CPython's compiler deduplicates equal constants (compiling `1 + 1 + 1`
  does not produce three separate entries for the literal `1`) using an
  auxiliary dict keyed on value-and-type during code generation
  **[flagged — high confidence dedup happens within one code object at
  compile time; lower confidence on exact current implementation
  mechanics, which have shifted across CPython versions]**; there is no
  general guarantee of dedup *across* different code objects, beyond
  whatever CPython's separate small-int and short-string interning caches
  provide independently of the constant-pool machinery.

- **Lua's `Proto.k` array** — the constant array attached to each function
  prototype, read by instructions like `LOADK`/`OP_LOADK` via index,
  documented in Ierusalimschy, de Figueiredo, and Celes's paper on the
  implementation of Lua 5.0. Lua's compiler also performs within-function
  constant deduplication during code generation — registering a constant
  looks it up against already-registered constants for that same
  prototype and reuses the slot on a match — again a compile-time policy
  of the specific implementation, not a property inherent to "having a
  constant pool" as an idea **[flagged — moderate confidence on the exact
  lookup mechanism and whether it has changed across the 5.x line; high
  confidence dedup-within-one-prototype is the general Lua behavior]**.

The recurring lesson across all three: **a constant pool does not, by
definition, deduplicate.** Whether identical literals used twice in the
same code object get one pool slot or two is a *policy choice made by the
compiler that builds the pool*, independent of the pool's role as "a side
array indexed by opcode operand." A pool that never deduplicates is not a
malformed pool, just a compiler that didn't bother — wasteful in pool size,
not wrong in any semantic sense, since nothing about executing `LOAD_CONST 4`
and `LOAD_CONST 9` cares whether slots 4 and 9 happen to hold equal values.

The other benefit worth stating plainly, because it matters more as a
system's garbage collector gets more sophisticated: a constant pool is a
**GC root enumeration mechanism for free**. Once literals live in a typed
array rather than scattered through raw instruction bytes, "find every
literal this code object keeps alive" is "iterate this array," full stop —
no need to understand instruction encoding to find pointers, no risk of a
collector misinterpreting an integer operand as a heap pointer or vice
versa. This decoupling — GC root-finding does not need to know the
instruction format — is easy to undervalue when a system is young and has
a simple collector, and becomes close to mandatory once the collector needs
to be correct under adversarial or complex object graphs.

## Tensions

### Sharing vs. capture

The recipe must be one object, shared by every closure over it, however
many there are; each closure must nonetheless capture its own, private set
of cells, however many closures share the recipe. These look like opposite
requirements bolted onto the same value until the layer split resolves
them by *disagreeing about which field lives where*: the sharing
requirement is satisfied entirely on the template side (one allocation, N
references to it, however references are managed — reference-counted
pointer, tracing-GC pointer, arena handle, whatever the host language
offers), and the capture requirement is satisfied entirely on the instance
side (a freshly allocated small vector of cell references, sized exactly
to however many captures this function's descriptors say it needs, filled
at the moment of closure creation and never shared with any other
instance's vector). The tension dissolves once "the function" is
understood to not be one thing with two properties, but two things, each
answering one of the two demands and structurally incapable of being asked
the other.

### Immutability vs. the mutable memo table

A code object is, in the strongest sense the design intends, **frozen
after compilation** — nothing about instructions, constant pool, or
capture descriptors is meant to change once compilation finishes, and
"immutable" is the right word for reasoning about correctness, thread
safety, and sharing. And yet, real systems routinely attach *mutable*
runtime state to exactly this supposedly-frozen object: a per-call-site
inline cache remembering which concrete method resolved last time and
short-circuiting the lookup if the same shape shows up again; a memoized
resolution of a global-variable reference to the slot it was found in, so
the next execution of the same instruction skips re-resolving the name.
This state has to live *somewhere*, and the natural place is right beside
the instruction that benefits from it — which means beside, or inside, the
code object those instructions belong to, the very artifact just described
as frozen.

The honest resolution is not that the immutability claim was wrong, but
that it was too coarse: the *code* — instructions, constants, capture
shape, everything that determines *what the function means* — is frozen;
what sits alongside it is not code but a **side table of interior-mutable
memo cells**, observationally invisible to anything that only cares about
what the function computes, mutated purely as an optimization that must be
sound to skip entirely (a correct implementation with the memo table
deleted and every cache miss taken every time must compute the identical
result, just slower). "Frozen after compile" describes the code object's
*meaning*; it says nothing about whether some field on the struct
happens to be a plain value or a `Cell`/atomic that gets poked by a fast
path. Both are true simultaneously, at different levels of description,
and treating them as contradictory is a category error — one is a claim
about semantics, the other about implementation-level mutability of
bits that carry no semantic weight of their own.

### Where native lives

Left open deliberately, per fork (c) above: whether "this is a built-in,
not bytecode" is a tag inside the code-object type itself or a fork made
one layer above, before a code object is ever touched, is a live design
decision with real, working examples of both. Nothing in the theory
developed here settles it — it's exactly the kind of question worth
predicting before checking against any one system's actual choice.

## What's cut, and why

**JS engine hidden-class/shape machinery** — the mechanism V8 and similar
engines use to give property-access on plain objects near-array speed
(assigning objects to "shapes" so property lookup becomes an offset rather
than a hash lookup) is a different problem wearing similar vocabulary
("shape," sometimes even "descriptor"). It is about the layout of ordinary
*objects*, not about the representation of *functions*; including it here
would import a large, genuinely interesting mechanism that answers a
question this document never asked.

**Any JIT tier** — tiered compilation (bytecode → baseline JIT → optimizing
JIT, deoptimization back down) is an axis of *how many compiled
representations one function has simultaneously and how the runtime picks
among them*, which is real and important and entirely orthogonal to
whether a given tier's representation is itself split into template and
instance. A JIT-compiled function still faces fork (a) inside whatever tier
it's compiled at; tiering just adds more representations to the pile, not
a different answer to the questions this document asks.

**Wren** — a small, Lua-descended scripting language whose function
representation (`ObjFn` holding code/constants, `ObjClosure` holding an
`ObjFn` plus upvalues) is, deliberately, essentially a restatement of Lua's
`Proto`/`Closure` split under different names. It earns no separate
treatment: everything it would illustrate, Lua already illustrates first
and with more historical weight, and the vocabulary Wren introduces adds no
term of art Lua's own vocabulary doesn't already supply.
