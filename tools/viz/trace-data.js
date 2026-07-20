"use strict";
const SLOTS = 9;
const esc = s => String(s).replace(/[&<>]/g,c=>({'&':'&amp;','<':'&lt;','>':'&gt;'}[c]));

/* ===========================================================================
   Opcode vocabulary is real (bytecode.rs). Module-level disassembly is real.
   Block/method chunk *sequences* are reconstructed — disasm does not recurse.
   =========================================================================== */

const EXAMPLES = {};

/* ------------------------------- E1 -------------------------------------- */
EXAMPLES.e1 = {
  tab: 'E1 · Ping-pong',
  title: 'A switch is a move, not a swap',
  lede: 'One fiber yields twice and finishes. Watch what leaves the VM, what stays behind, and the beat in the middle where the VM holds nothing at all.',
  source: [
    'let f = Fiber.new {',
    '  let n = 0',
    '  Fiber.yield(n + 1)',
    '  Fiber.yield(n + 2)',
    '  99',
    '}',
    '',
    'f.call    // 1',
    'f.call    // 2',
  ],
  chunks: {
    '<module>': { locals:['self'], code:[
      {t:'GetGlobal',a:'Fiber'},
      {t:'Closure',a:'0'},
      {t:'Invoke',a:'1 new(_)'},
      {t:'DefineGlobal',a:'f'},
      {t:'GetGlobal',a:'f'},
      {t:'Invoke',a:'0 call'},
      {t:'Pop'},
      {t:'GetGlobal',a:'f'},
      {t:'Invoke',a:'0 call'},
      {t:'Pop'},
      {t:'Return'},
    ]},
    'F1 entry block': { locals:['self','n'], code:[
      {t:'Constant',a:'0'},
      {t:'GetGlobal',a:'Fiber'},
      {t:'GetLocal',a:'1',lit:true},
      {t:'Constant',a:'1'},
      {t:'Invoke',a:'1 +(_)'},
      {t:'Invoke',a:'1 yield(_)'},
      {t:'Pop'},
      {t:'GetGlobal',a:'Fiber'},
      {t:'GetLocal',a:'1',lit:true},
      {t:'Constant',a:'2'},
      {t:'Invoke',a:'1 +(_)'},
      {t:'Invoke',a:'1 yield(_)'},
      {t:'Pop'},
      {t:'Constant',a:'99'},
      {t:'Return'},
    ]},
  },
  gate: {
    q: 'The fiber is about to yield. Where does its stack go?',
    opts: [
      { t:'Nothing moves — the VM keeps a pointer to each fiber\'s stack and just switches which one it reads.', ok:false },
      { t:'The four live buffers are moved out of the VM into the fiber object, leaving the VM empty for an instant.', ok:true },
      { t:'The stack is copied into the fiber object; the VM keeps its own copy too.', ok:false },
    ],
    because: 'Lua and Wren do the first one — a coroutine is a state object and switching reassigns a pointer. Phalcom does the second: <span class="mono">mem::take</span> on four VM fields. Because it is a <b>move</b>, there is a real instant where the VM holds nothing — beat 2 below. That instant is why every bug in this area is a move that did not finish.',
  },
  events: [
    { line:0, ip:0, chunk:'<module>', hostDepth:1,
      framePush:{name:'<module>',offset:0,gen:1,kind:'m',locals:['self']}, push:'<module>',
      note:'Start at the top. <b>The root fiber is already running</b> — module code is fiber code. Its frame opens at slot 0.' },
    { line:0, ip:0, push:'<Fiber>', note:'<span class="mono">GetGlobal Fiber</span> — the receiver.' },
    { line:0, ip:1, push:'<blk>', note:'<span class="mono">Closure 0</span> builds the body block.' },
    { line:0, ip:2, pop:2, push:'<Fiber F1>', fiberNew:{id:1,name:'F1'}, resumer:{id:1,to:0},
      note:'<span class="mono">Fiber.new(_)</span>. <b>F1 appears in the rail with all four compartments empty</b> — it has never run, so it owns nothing yet. <span class="mono">started == false</span>.' },
    { line:0, ip:3, pop:1,
      note:'<span class="mono">DefineGlobal f</span> — <b>not</b> <span class="mono">SetLocal</span>. Module-level <span class="mono">let</span> compiles to a global; this was checked with <span class="mono">phalcom disasm</span>. Only function and block bodies have slots, which is why the index row below shows just <span class="mono">self</span>.' },
    { line:7, ip:4, push:'<Fiber F1>', note:'<span class="mono">GetGlobal f</span> — the fiber, as receiver.' },
    { line:7, ip:5, pop:1, note:'<span class="mono">Invoke call</span>. The switch begins — watch the three beats, and watch the cards.' },

    { line:7, ip:5, switch:{from:0,to:1,phase:'take'}, rootStatus:'suspended',
      note:'<b>Beat 1 — take.</b> <span class="mono">store_live_into</span>: <span class="mono">frames</span>, <span class="mono">stack</span>, <span class="mono">open_upvalues</span> and <span class="mono">checking</span> are <span class="mono">mem::take</span>n out of the VM into <b>F0</b>. The root\'s compartments fill; the VM empties.' },
    { line:7, ip:5, switch:{from:0,to:1,phase:'hole'},
      note:'<b>Beat 2 — the hole.</b> The VM holds nothing. F0 is full; <b>F1 is still empty</b>, because it has never run. This is the one switch where only one card is full — every later switch has two.' },
    { line:0, ip:0, switch:{from:0,to:1,phase:'install'}, current:1, fiberStatus:{id:1,s:'running'},
      framePush:{name:'F1 entry block',offset:0,gen:4,kind:'blk',locals:['self','n']}, push:'<blk>',
      note:'<b>Beat 3 — install.</b> F1 had nothing parked, so instead of restoring buffers the VM <b>pushes its entry frame now</b>. First resume and later resumes are different code paths for exactly this reason.' },

    { line:1, ip:0, push:'0', note:'<span class="mono">Constant 0</span> — <b>n</b> lands at slot 1 and is named by <i>arriving there</i>. There is no store instruction.' },
    { line:2, ip:1, push:'<Fiber>', note:'Receiver for the yield.' },
    { line:2, ip:2, push:'0', readfrom:1, note:'<span class="mono">GetLocal 1</span>: base 0 + 1 = slot 1. The index was fixed at compile time; the name <span class="mono">n</span> does not exist here.' },
    { line:2, ip:3, push:'1', note:'The literal.' },
    { line:2, ip:4, pop:2, push:'1', note:'<span class="mono">Invoke +(_)</span> — two off, one on.' },
    { line:2, ip:5, pop:2, note:'<span class="mono">Invoke yield(_)</span> consumes receiver and argument. Switch begins.' },

    { line:2, ip:5, switch:{from:1,to:0,phase:'take'}, fiberStatus:{id:1,s:'suspended'},
      note:'<b>Take.</b> F1\'s buffers move into its card.' },
    { line:2, ip:5, switch:{from:1,to:0,phase:'hole'},
      note:'<b>The hole — and now look at the rail.</b> <b>Both cards are full and the centre is empty.</b> Every byte of live state is sitting in a heap object and the VM holds none of it. <b>Lua and Wren cannot enter this state</b> — a pointer swap has no interval where nothing is current. Every bug in this area is a move that did not finish.' },
    { line:7, ip:5, switch:{from:1,to:0,phase:'install'}, current:0, fiberStatus:{id:0,s:'running'},
      pop:1, push:'1',
      note:'<b>Install.</b> The root\'s buffers return and the yielded <span class="mono">1</span> replaces the receiver. Note what never travelled: <span class="mono">next_frame_generation</span>, the heap, the class table. <b>Movable is position; pinned is identity.</b>' },

    { line:7, ip:6, pop:1, note:'<span class="mono">Pop</span>. F1 is suspended with its whole stack parked inside it — a live GC root that a collector scanning only <span class="mono">current</span> would free.' },
    { line:8, ip:7, push:'<Fiber F1>', note:'<span class="mono">GetGlobal f</span> again.' },
    { line:8, ip:8, pop:1, note:'<span class="mono">Invoke call</span> — second resume.' },

    { line:8, ip:8, switch:{from:0,to:1,phase:'take'}, rootStatus:'suspended', note:'Take. The root parks again.' },
    { line:8, ip:8, switch:{from:0,to:1,phase:'hole'}, note:'<b>The hole.</b> Two full cards, empty centre — the ordinary case from here on.' },
    { line:3, ip:6, switch:{from:0,to:1,phase:'install'}, current:1, fiberStatus:{id:1,s:'running'},
      push:'<resumed>',
      note:'Install. F1\'s buffers move back — <b>including its instruction pointer</b>, which rode inside <span class="mono">frames</span> the whole time. Execution resumes at the exact opcode after the yield.' },

    { line:3, ip:6, pop:1, note:'<span class="mono">Pop</span> discards the resume value.' },
    { line:3, ip:7, push:'<Fiber>', note:'Receiver.' },
    { line:3, ip:8, push:'0', readfrom:1, note:'<span class="mono">GetLocal 1</span> — and <b>n is still 0</b>. The local survived the park because it was inside the buffer that moved.' },
    { line:3, ip:9, push:'2', note:'The second literal.' },
    { line:3, ip:10, pop:2, push:'2', note:'<span class="mono">Invoke +(_)</span> → 2.' },
    { line:3, ip:11, pop:2, note:'Second yield.' },
    { line:3, ip:11, switch:{from:1,to:0,phase:'take'}, fiberStatus:{id:1,s:'suspended'}, note:'Take.' },
    { line:3, ip:11, switch:{from:1,to:0,phase:'hole'}, note:'The hole.' },
    { line:8, ip:8, switch:{from:1,to:0,phase:'install'}, current:0, fiberStatus:{id:0,s:'running'},
      pop:1, push:'2', note:'Install. The root has <span class="mono">2</span>.' },
  ],
  takeawayGrip: 'A fiber is not a thread of execution. It is the set of buffers a fiber is <i>not</i> currently using — and switching is <span class="mono">mem::take</span> on four of them.',
  takeaway: [
    'Four fields move: <span class="mono">frames</span>, <span class="mono">stack</span>, <span class="mono">open_upvalues</span>, <span class="mono">checking</span>. Everything else in the VM stays — and <span class="mono">next_frame_generation</span> staying behind is what makes frame tokens unique across every fiber, not just within one.',
    '<span class="mono">ip</span> is a <span class="mono">CallFrame</span> field, so it rides inside <span class="mono">frames</span> and parks with the fiber. The partition is clean: <b>movable is position, pinned is identity.</b>',
    'Because it is a move rather than a swap, there is a real instant with no live state. That instant does not exist in Lua or Wren, and it is the shape of every bug in this area.',
  ],
  fwd: 'Deeper: <span class="mono">docs/learn/concurrency/parked-fiber.md</span> · <span class="mono">docs/learn/concurrency/restricted-loop.md</span>',
};

/* ------------------------------- E2 -------------------------------------- */
EXAMPLES.e2 = {
  tab: 'E2 · Upvalue across a park',
  title: 'Why an upvalue names a fiber',
  lede: 'A block captures a local, then the fiber parks with that capture still open. Watch what the cell points at — and what happens to the thing it points at.',
  source: [
    'let f = Fiber.new {',
    '  let n = 0',
    '  let bump = { n = n + 1 }',
    '  bump.call',
    '  Fiber.yield(n)',
    '  bump.call',
    '  n',
    '}',
    '',
    'f.call',
    'f.call',
  ],
  chunks: {
    '<module>': { locals:['self'], code:[
      {t:'GetGlobal',a:'Fiber'},{t:'Closure',a:'0'},{t:'Invoke',a:'1 new(_)'},
      {t:'DefineGlobal',a:'f'},{t:'GetGlobal',a:'f'},{t:'Invoke',a:'0 call'},
      {t:'Pop'},{t:'GetGlobal',a:'f'},{t:'Invoke',a:'0 call'},{t:'Return'},
    ]},
    'F1 entry block': { locals:['self','n','bump'], code:[
      {t:'Constant',a:'0'},
      {t:'Closure',a:'0'},
      {t:'GetLocal',a:'2',lit:true},
      {t:'Invoke',a:'0 call'},
      {t:'Pop'},
      {t:'GetLocal',a:'1',lit:true},
      {t:'GetGlobal',a:'Fiber'},
      {t:'Invoke',a:'1 yield(_)'},
      {t:'Pop'},
      {t:'GetLocal',a:'2',lit:true},
      {t:'Invoke',a:'0 call'},
      {t:'Pop'},
      {t:'GetLocal',a:'1',lit:true},
      {t:'CloseUpvalue',a:'1'},
      {t:'Return'},
    ]},
    'bump': { locals:['self'], code:[
      {t:'GetUpvalue',a:'0'},
      {t:'Constant',a:'1'},
      {t:'Invoke',a:'1 +(_)'},
      {t:'SetUpvalue',a:'0'},
      {t:'Return'},
    ]},
  },
  gate: {
    q: 'The fiber is about to park while <span class="mono">bump</span> still holds an open capture of <span class="mono">n</span>. What happens to the reference?',
    opts: [
      { t:'It breaks — the slot it pointed at is no longer in the VM.', ok:false },
      { t:'It follows: the cell stays put and the reference now reaches into the parked fiber object.', ok:true },
      { t:'It is forced closed early — parking copies the value into the cell.', ok:false },
    ],
    because: 'The cell is a <b>heap object</b>, so it is not one of the four fields a switch moves. It does not travel. The <i>tape moves out from under it</i>. That is exactly why <span class="mono">Upvalue::Open</span> stores <span class="mono">{ fiber, slot }</span> and not a pointer — the referent relocates, so the reference must name <b>which buffer to look in</b>. Under Lua\'s raw <span class="mono">TValue*</span> that field would be meaningless.',
  },
  events: [
    { line:9, ip:0, chunk:'F1 entry block', fiberNew:{id:1,name:'F1'}, hostDepth:1,
      framePush:{name:'F1 entry block',offset:0,gen:4,kind:'blk',locals:['self','n','bump']},
      push:'<blk>', fiberStatus:{id:1,s:'running'}, rootStatus:'suspended', rootParked:true,
      current:1, resumer:{id:1,to:0},
      note:'The fiber is already running — the root parked to get here. (E1 shows that switch in full; this opens after it.) Note <b>F0’s card is full</b>: a running child always means its resumer’s buffers are sitting in its resumer’s object.' },
    { line:1, ip:0, push:'0', note:'<b>n</b> = 0 at slot 1.' },
    { line:2, ip:1, push:'<blk bump>', upvalueOpen:{id:'c0',name:'n',slot:1,fiber:1},
      note:'<span class="mono">Closure 0</span> builds <b>bump</b> and captures <b>n</b>. The compiler marked <span class="mono">n</span> as <span class="mono">is_captured</span>, so instead of an ordinary local read, <span class="mono">bump</span> gets a <b>cell</b> pointing at slot 1 of <i>this fiber</i>.' },
    { line:3, ip:2, push:'<blk bump>', readfrom:2, note:'<span class="mono">GetLocal 2</span> → slot 2, the block itself.' },
    { line:3, ip:0, framePush:{name:'bump',offset:3,gen:5,kind:'blk',locals:['self']}, hostDepth:2,
      note:'<span class="mono">Block#call</span> — and the gutter goes to <b>depth 2</b>. Calling a block re-enters <span class="mono">run_until</span> on the Rust stack. Remember this; E3 turns it into a language restriction.' },
    { line:2, ip:0, upvalueRead:'c0', push:'0',
      note:'<span class="mono">GetUpvalue 0</span> — <b>not</b> <span class="mono">GetLocal</span>. This frame\'s base is 3; <span class="mono">n</span> is at 1. <span class="mono">stack_offset + slot</span> cannot reach it, so capture exists precisely because the addition stopped being answerable.' },
    { line:2, ip:2, pop:2, push:'1', note:'n + 1.' },
    { line:2, ip:3, upvalueWrite:{id:'c0',v:'1'}, pop:1,
      note:'<span class="mono">SetUpvalue 0</span> writes <b>through the cell into slot 1</b> — still a live stack slot, still open. Two names for one storage location.' },
    { line:3, ip:4, framePop:true, hostDepth:1, note:'<span class="mono">bump</span> returns. Gutter back to depth 1. The cell stays open — <span class="mono">n</span>\'s frame is still alive.' },
    { line:4, ip:5, push:'1', readfrom:1, note:'<span class="mono">GetLocal 1</span> → 1. The mutation through the cell is visible to the ordinary local read. Same slot.' },

    { line:4, ip:7, switch:{from:1,to:0,phase:'take'}, fiberStatus:{id:1,s:'suspended'},
      note:'<b>Take.</b> The tape moves into F1 — <b>and the cell does not move with it.</b> Cells are heap objects; they are not among the four fields.' },
    { line:4, ip:7, switch:{from:1,to:0,phase:'hole'},
      note:'<b>The hole.</b> Look at the cell: it still points at <i>slot 1 of fiber 1</i>. The slot did not change. The <b>buffer it lives in</b> did.' },
    { line:9, ip:0, switch:{from:1,to:0,phase:'install'}, current:0, fiberStatus:{id:0,s:'running'},
      note:'Install. The cell now reaches <b>into the parked fiber object</b> — unbroken, and now you can see why it has to say <i>which</i> fiber.' },

    { line:10, ip:0, switch:{from:0,to:1,phase:'take'}, fiberStatus:{id:0,s:'suspended'}, note:'Second <span class="mono">f.call</span>.' },
    { line:10, ip:0, switch:{from:0,to:1,phase:'hole'}, note:'The hole.' },
    { line:5, ip:9, switch:{from:0,to:1,phase:'install'}, current:1, fiberStatus:{id:1,s:'running'},
      note:'F1 resumes. The cell points at a live slot again. Nothing had to be fixed up on the way in or out.' },

    { line:5, ip:9, pop:1, push:'<blk bump>', readfrom:2, note:'<span class="mono">Pop</span> discards the yield’s result, then <span class="mono">GetLocal 2</span> pushes bump again — same shape as the first call.' },
    { line:5, ip:0, framePush:{name:'bump',offset:3,gen:6,kind:'blk',locals:['self']}, hostDepth:2,
      note:'Calling it again — new frame, new base, <b>same cell</b>.' },
    { line:2, ip:3, upvalueWrite:{id:'c0',v:'2'},
      note:'<span class="mono">SetUpvalue 0</span> → slot 1 becomes 2. The capture outlived a park and a resume without ever being closed.' },
    { line:5, ip:11, framePop:true, hostDepth:1, note:'Return.' },
    { line:6, ip:13, upvalueClose:'c0',
      note:'<span class="mono">CloseUpvalue 1</span> — the frame is ending, so the value is <b>copied into the cell</b> and the reference retracts. From here the cell is self-contained and no longer names any fiber.' },
  ],
  takeawayGrip: 'A capture names a slot, not an address — because in Phalcom the buffer holding that slot can move, and the reference has to survive the move.',
  takeaway: [
    'An upvalue exists exactly where <span class="mono">stack_offset + slot</span> stops working. Inside <span class="mono">bump</span> the base is different, so the local is unreachable by arithmetic; the compiler flags <span class="mono">is_captured</span> and routes it through a cell instead.',
    'Cells are heap objects and do not travel on a switch. The tape moves out from under them — which is the whole reason <span class="mono">Upvalue::Open</span> carries <span class="mono">{ fiber, slot }</span>. Lua stores a raw <span class="mono">TValue*</span> and pays for it with a pointer fix-up pass when the stack reallocates. Phalcom pays a branch on read instead.',
    'Open means two names for one storage location. Closed means the value was copied into the cell and the slot is free. The transition happens on frame exit — <b>and if a frame is ever discarded without running it, the cell is left pointing into a buffer that no longer exists.</b>',
  ],
  fwd: 'Deeper: <span class="mono">docs/learn/vm/upvalues.md</span> · the failure mode is <span class="mono">docs/learn/concurrency/</span> C3',
};

/* ------------------------------- E3 -------------------------------------- */
EXAMPLES.e3 = {
  tab: 'E3 · Legal vs illegal yield',
  title: 'The restriction is a property of the host stack',
  lede: 'Two loops that look equally reasonable. One yields fine; the other cannot. The only thing that differs on screen is the gutter.',
  variants: ['A · while — legal', 'B · each — raises'],
  gate: {
    q: 'Both programs yield from inside a loop. One raises <span class="mono">CannotYieldAcrossNativeFrame</span>. Which, and why?',
    opts: [
      { t:'B, because <span class="mono">each</span> is native Rust code and native code cannot yield.', ok:false },
      { t:'B, because calling a block re-enters the interpreter on the Rust stack, and a Rust frame cannot be parked.', ok:true },
      { t:'A, because <span class="mono">while (true)</span> never returns control to the scheduler.', ok:false },
    ],
    because: 'The line is <b>block invocation</b>, not native-vs-Phalcom. <span class="mono">each</span> is written in Phalcom (<span class="mono">core.ph</span>), but it invokes a block, and <span class="mono">Block#call</span> goes through <span class="mono">block_call</span> → <span class="mono">run_until</span>, pushing a <b>Rust</b> frame. <span class="mono">while</span> is lowered by the compiler to <span class="mono">Jump</span>/<span class="mono">Loop</span> inside one chunk — no frame, no re-entry. A fiber can park everything it owns because everything it owns is a heap <span class="mono">Vec</span>. It cannot park a Rust frame. <b>Yield is legal exactly when the fiber owns 100% of its own continuation.</b>',
  },
  takeawayGrip: 'A user-visible language restriction that falls straight out of an optimiser decision: <span class="mono">while</span> is inlined and <span class="mono">each</span> is not.',
  takeaway: [
    '<span class="mono">native_reentry_depth</span> must be <span class="mono">0</span> for a switch to be legal. That is the whole rule, and it is one integer.',
    'This is ADR-0030\'s restricted model — "Option A". The alternative that lifts it is a full trampoline: de-recurse every callback primitive so nothing ever re-enters the interpreter. Lua 5.1 had this exact restriction and lifted it in 5.2 with continuation functions.',
    'The ADR argues A→B is purely additive and A→C (real stackful coroutines with native stacks) is not. That asymmetry is why Phalcom can still claim a moving GC is droppable later: there are no native fiber stacks for a collector to scan.',
  ],
  fwd: 'Deeper: <span class="mono">docs/learn/concurrency/restricted-loop.md</span>',
  sub: [
    { /* A */
      source: ['Fiber.new {','  let n = 0','  while (true) {','    Fiber.yield(n)','  }','}'],
      chunks: {
        '<module>': { locals:['self'], code:[
          {t:'GetGlobal',a:'Fiber'},{t:'Closure',a:'0'},{t:'Invoke',a:'1 new(_)'},
          {t:'DefineGlobal',a:'f'},{t:'GetGlobal',a:'f'},{t:'Invoke',a:'0 call'},
          {t:'Pop'},{t:'Return'},
        ]},
        'F1 entry block': { locals:['self','n'], code:[
        {t:'Constant',a:'0'},
        {t:'True'},
        {t:'GuardBool',a:'+8'},
        {t:'JumpIfFalse',a:'+6'},
        {t:'GetLocal',a:'1',lit:true},
        {t:'GetGlobal',a:'Fiber'},
        {t:'Invoke',a:'1 yield(_)'},
        {t:'Pop'},
        {t:'Loop',a:'-7'},
        {t:'Return'},
      ]}},
      events: [
        { line:0, ip:0, chunk:'F1 entry block', fiberNew:{id:1,name:'F1'}, hostDepth:1,
          framePush:{name:'F1 entry block',offset:0,gen:4,kind:'blk',locals:['self','n']}, push:'<blk>',
          fiberStatus:{id:1,s:'running'}, rootStatus:'suspended', rootParked:true,
          current:1, resumer:{id:1,to:0},
          note:'Fiber starts. Gutter depth <b>1</b> — one <span class="mono">run_until</span>, the fiber\'s own.' },
        { line:1, ip:0, push:'0', note:'<b>n</b> = 0.' },
        { line:2, ip:1, push:'true', note:'The loop condition.' },
        { line:2, ip:2, pop:1, note:'<span class="mono">GuardBool</span> — the truthiness floor. A non-Bool condition is rejected here, with no coercion.' },
        { line:3, ip:4, push:'0', readfrom:1, note:'<span class="mono">GetLocal 1</span>. Note what has <b>not</b> happened: no frame was pushed for the loop. <span class="mono">while</span> lowered to jumps inside this one chunk.' },
        { line:3, ip:6, note:'<span class="mono">Invoke yield(_)</span> — and the gutter is still at depth 1. <b>Nothing is holding our place.</b>' },
        { line:3, ip:6, switch:{from:1,to:0,phase:'take'}, fiberStatus:{id:1,s:'suspended'},
          note:'Take. Legal: the fiber owns every byte of its own continuation.' },
        { line:3, ip:6, switch:{from:1,to:0,phase:'hole'}, note:'The hole.' },
        { line:0, ip:0, switch:{from:1,to:0,phase:'install'}, current:0, fiberStatus:{id:0,s:'running'},
          note:'Install. It worked — and it worked because the gutter read 1.' },
      ],
    },
    { /* B */
      source: ['Fiber.new {','  list.each { x =>','    Fiber.yield(x)','  }','}'],
      chunks: {
        'F1 entry block': { locals:['self'], code:[
          {t:'GetGlobal',a:'list'},
          {t:'Closure',a:'0'},
          {t:'Invoke',a:'1 each(_)'},
          {t:'Return'},
        ]},
        'Iterable#each(_)': { locals:['self','fn','_c'], code:[
          {t:'GetLocal',a:'2',lit:true},
          {t:'JumpIfNone',a:'+7'},
          {t:'GetLocal',a:'1',lit:true},
          {t:'Invoke',a:'1 call(_)'},
          {t:'Pop'},
          {t:'Loop',a:'-6'},
          {t:'Return'},
        ]},
        'each block': { locals:['self','x'], code:[
          {t:'GetLocal',a:'1',lit:true},
          {t:'GetGlobal',a:'Fiber'},
          {t:'Invoke',a:'1 yield(_)'},
          {t:'Return'},
        ]},
      },
      events: [
        { line:0, ip:0, chunk:'F1 entry block', fiberNew:{id:1,name:'F1'}, hostDepth:1,
          framePush:{name:'F1 entry block',offset:0,gen:4,kind:'blk',locals:['self']}, push:'<blk>',
          fiberStatus:{id:1,s:'running'}, rootStatus:'suspended', rootParked:true,
          current:1, resumer:{id:1,to:0},
          note:'Same start. Gutter depth <b>1</b>.' },
        { line:1, ip:0, push:'<List>', note:'The receiver.' },
        { line:1, ip:1, push:'<blk>', note:'<span class="mono">Closure 0</span> — the block literal passed to <span class="mono">each</span>.' },
        { line:1, ip:0, framePush:{name:'Iterable#each(_)',offset:1,gen:5,kind:'m',locals:['self','fn','_c']},
          note:'<b><span class="mono">each</span> is written in Phalcom</b>, in <span class="mono">core.ph</span>. This is an ordinary Phalcom frame. The gutter has <b>not</b> moved — so far this is no different from A.' },
        { line:1, ip:0, push:'0', readfrom:3, note:'The bare cursor lands in <span class="mono">_c</span>, slot 3. <span class="mono">iterate</span>/<span class="mono">iteratorValue</span> — zero allocation per step.' },
        { line:1, ip:2, push:'<blk>', readfrom:2, note:'<span class="mono">GetLocal 2</span> → the block. Pushed as the receiver for the call.' },
        { line:1, ip:3, push:'7', note:'The element, as the argument. Still <span class="mono">each</span>’s own scratch space.' },
        { line:1, ip:0, framePush:{name:'each block',offset:4,gen:6,kind:'blk',locals:['self','x']}, hostDepth:2,
          note:'<b>Here.</b> <span class="mono">Invoke call(_)</span> on a block goes through <span class="mono">block_call</span>, which calls <span class="mono">run_until</span> — <b>a fresh Rust frame on the machine stack.</b> Gutter: depth <b>2</b>.' },
        { line:2, ip:0, push:'7', readfrom:5, note:'<span class="mono">GetLocal 1</span> — the element <span class="mono">x</span>, at base 4 + 1 = slot 5.' },
        { line:2, ip:2, error:{kind:'CannotYieldAcrossNativeFrame',
            msg:'a fiber switch requires native_reentry_depth == 0; it is 2'},
          note:'<b>Raise.</b> <span class="mono">Invoke yield(_)</span> checks the gutter and refuses. The fiber could park its tape, its frames, its cells — but the Rust frame between it and the bottom is not its to move. It is a <b>catchable</b> error, not a panic.' },
      ],
    },
  ],
};

/* ------------------------------- E4 -------------------------------------- */
EXAMPLES.e4 = {
  tab: 'E4 · Dead frame',
  title: 'A return that has nowhere to return to',
  lede: 'A block escapes the method that made it, then tries to return through it. Watch the generation counter — it is the only reason this is an error instead of memory corruption.',
  source: [
    'class Box {',
    '  make() {',
    '    return { return 99 }',
    '  }',
    '}',
    '',
    'let b   = Box.new()',
    'let blk = b.make()',
    'blk.call',
  ],
  chunks: {
    '<module>': { locals:['self'], code:[
      {t:'GetGlobal',a:'Box'},{t:'Invoke',a:'0 new'},{t:'DefineGlobal',a:'b'},
      {t:'GetGlobal',a:'b'},{t:'Invoke',a:'0 make'},{t:'DefineGlobal',a:'blk'},
      {t:'GetGlobal',a:'blk'},{t:'Invoke',a:'0 call'},{t:'Pop'},{t:'Return'},
    ]},
    'Box#make': { locals:['self'], code:[
      {t:'Closure',a:'0'},
      {t:'Return'},
    ]},
    'escaping block': { locals:['self'], code:[
      {t:'Constant',a:'99'},
      {t:'ReturnNonLocal'},
    ]},
  },
  gate: {
    q: '<span class="mono">make</span> has already returned. The block it built is about to run <span class="mono">return 99</span>. What happens?',
    opts: [
      { t:'It returns 99 to whoever called the block — the block is just a function.', ok:false },
      { t:'It raises <span class="mono">DeadFrameError</span>: the home frame is gone, and the token’s generation no longer matches what sits at that index.', ok:true },
      { t:'Undefined behaviour — it writes into whatever now occupies that stack slot.', ok:false },
    ],
    because: 'The third answer is what you get if a block stores a <b>pointer</b> to its home frame, which is why C-family languages forbid the construct outright. Phalcom stamps the block with a <b>FrameToken = (index, generation)</b>. The index is reused constantly — here the escaping block lands at the very same index <span class="mono">make</span> occupied — but <span class="mono">next_frame_generation</span> is <b>pinned and monotonic</b>, so the generation never repeats. The check is one integer comparison, and it turns a dangling-pointer class of bug into a catchable error.',
  },
  events: [
    { line:6, ip:0, chunk:'<module>', hostDepth:1,
      framePush:{name:'<module>',offset:0,kind:'m',locals:['self']}, push:'<module>',
      note:'Module frame. Watch <b>next_frame_generation</b> in the rail — it is the pinned counter, and it is about to do all the work.' },
    { line:6, ip:0, push:'<Box>', note:'<span class="mono">GetGlobal Box</span>.' },
    { line:6, ip:1, pop:1, push:'<Box inst>', note:'<span class="mono">Invoke new</span>.' },
    { line:6, ip:2, pop:1, note:'<span class="mono">DefineGlobal b</span>.' },
    { line:7, ip:3, push:'<Box inst>', note:'Receiver for <span class="mono">make</span>.' },
    { line:1, ip:0, framePush:{name:'Box#make',offset:1,kind:'m',locals:['self']},
      note:'<b><span class="mono">make</span>’s frame opens — note its <span class="mono">gen</span>.</b> This is the frame the escaping block will later try to return through. It is frame <b>index 1</b>.' },
    { line:2, ip:0, captureHome:'h0', push:'<blk>',
      note:'<span class="mono">Closure 0</span> builds the block <b>and stamps it with a home_frame_token</b> — <span class="mono">(index 1, gen 2)</span>. Not a pointer. A pointer here is a dangling pointer three lines from now.' },
    { line:2, ip:1, framePop:true, push:'<blk>',
      note:'<b><span class="mono">make</span> returns and its frame is gone.</b> The counter does not rewind — nothing will ever be <span class="mono">(index 1, gen 2)</span> again. That is the entire trick.' },
    { line:7, ip:5, pop:1, note:'<span class="mono">DefineGlobal blk</span>. The block outlived its home.' },
    { line:8, ip:6, push:'<blk>', note:'<span class="mono">GetGlobal blk</span>.' },
    { line:2, ip:0, framePush:{name:'escaping block',offset:1,kind:'blk',locals:['self'],home:'h0'},
      note:'<b>Look at the bracket.</b> The block’s frame lands at <b>index 1 — the very slot <span class="mono">make</span> had</b> — but with a fresh generation. Its home token still says <span class="mono">gen 2</span>. The mismatch is already visible, before anything has gone wrong.' },
    { line:2, ip:0, push:'99', note:'<span class="mono">Constant 99</span>. The value is fine. The destination is not.' },
    { line:2, ip:1, error:{kind:'DeadFrameError', msg:'home frame (index 1, generation 2) is gone; index 1 now holds generation 3'},
      note:'<span class="mono">ReturnNonLocal</span> reads the token, compares one integer, and refuses. <b>An index alone would have matched</b> and written into a live, unrelated frame. The generation is what makes reuse detectable.' },
  ],
  takeawayGrip: 'A <span class="mono">FrameToken</span> is a pointer split in two: <i>where to look</i>, and <i>who it was</i>. The second half is what survives reuse.',
  takeaway: [
    'The moment a block can outlive its defining frame — stored, returned, or sent to another fiber — a <span class="mono">return</span> through the dead home must <b>trap</b>, not corrupt. This has to be designed when blocks gain the ability to escape, not when <span class="mono">return</span> is added.',
    'Indices get reused immediately; here the escaping block took the exact index its own home had. Only the generation distinguishes them, and only because <span class="mono">next_frame_generation</span> is <b>pinned</b> — it never travels on a fiber switch, so a token minted in one fiber cannot collide with one minted in another.',
    'Smalltalk makes the home context a first-class object that stays alive, which costs an allocation per activation. Phalcom keeps frames as flat <span class="mono">Copy</span> values in a <span class="mono">Vec</span> and pays one integer compare instead. Same safety, different bill.',
  ],
  fwd: 'Deeper: <span class="mono">docs/learn/vm/frame-identity.md</span> · ADR-0013',
};

/* ------------------------------- E5 -------------------------------------- */
EXAMPLES.e5 = {
  tab: 'E5 · call vs try',
  title: 'Where an unwind is allowed to stop',
  lede: 'The same failing fiber, resumed two ways. One re-raises into its resumer; the other captures at the boundary. The difference is a property of the edge, not of the fiber.',
  variants: ['A · f.call — re-raises', 'B · f.try — captured'],
  gate: {
    q: 'A fiber raises an uncaught error. What decides whether the host survives it?',
    opts: [
      { t:'Nothing — an uncaught error in any fiber terminates the program.', ok:false },
      { t:'How it was resumed. <span class="mono">call</span> re-raises into the resumer; <span class="mono">try</span> captures at the floor and the fiber goes <span class="mono">Failed</span> with the error in its result slot.', ok:true },
      { t:'Whether the fiber installed a handler before raising.', ok:false },
    ],
    because: '<span class="mono">FiberResumeMode</span> (<span class="mono">heap/fiber.rs:37</span>) is stored on the fiber but set <b>by the resume call</b> — so it is really a property of the <i>edge</i> in the resumer chain. That makes containment a caller’s decision, which is the same shape as <span class="mono">Result</span> vs <span class="mono">throw</span>: the person who knows whether failure is expected is the caller, not the callee. Watch the chain edge in the rail: solid for <span class="mono">call</span>, doubled for <span class="mono">try</span>.',
  },
  takeawayGrip: 'A fiber’s failure is contained by design — the unwind stops at its floor and the error lands in its result slot. Whether it stops is chosen by the resumer.',
  takeaway: [
    'Phalcom’s unwind is <b>terminating</b>, not resumable (ADR-0008) — it rejects Smalltalk’s <span class="mono">resume:</span>. That choice is not retrofittable: a resumable unwind has to preserve the frames it passes, which changes the entire stack discipline.',
    'The fiber floor is where <i>errors</i> ⊗ <i>concurrency</i> meet. A fiber boundary is already a natural containment edge, so the error model gets one for free — but only because the boundary was there for scheduling reasons first.',
    'Still open (overlay): <b>structured concurrency</b>. ADR-0030 gives a single-fiber <span class="mono">abort</span>, not cascading cancellation of children. Nothing here propagates <i>down</i> the chain, only up.',
  ],
  fwd: 'Deeper: ADR-0008 · ADR-0030 §6 · <span class="mono">docs/learn/concurrency/</span> C3',
  sub: (function(){
    const source = [
      'let f = Fiber.new {',
      '  Error.new("boom").raise()',
      '}',
      '',
      'f.call     // A',
      'f.try      // B',
    ];
    const chunks = {
      '<module>': { locals:['self'], code:[
        {t:'GetGlobal',a:'Fiber'},{t:'Closure',a:'0'},{t:'Invoke',a:'1 new(_)'},
        {t:'DefineGlobal',a:'f'},{t:'GetGlobal',a:'f'},{t:'Invoke',a:'0 call'},
        {t:'Pop'},{t:'Return'},
      ]},
      'F1 entry block': { locals:['self'], code:[
        {t:'GetGlobal',a:'Error'},{t:'Constant',a:'"boom"'},{t:'Invoke',a:'1 new(_)'},
        {t:'Invoke',a:'0 raise'},{t:'Return'},
      ]},
    };
    const lead = mode => ([
      { line:0, ip:0, chunk:'<module>', hostDepth:1,
        framePush:{name:'<module>',offset:0,kind:'m',locals:['self']}, push:'<module>',
        note:'Module frame.' },
      { line:0, ip:0, push:'<Fiber>', note:'Receiver.' },
      { line:0, ip:1, push:'<blk>', note:'The body block.' },
      { line:0, ip:2, pop:2, push:'<Fiber F1>', fiberNew:{id:1,name:'F1'},
        resumer:{id:1,to:0,mode},
        note:`<span class="mono">Fiber.new(_)</span>. The chain edge is <b>${mode}</b> — set by how we are about to resume it, not by the fiber itself.` },
      { line:0, ip:3, pop:1, note:'<span class="mono">DefineGlobal f</span>.' },
      { line:mode==='call'?4:5, ip:4, push:'<Fiber F1>', note:'Receiver for the resume.' },
      { line:mode==='call'?4:5, ip:5, pop:1, note:`<span class="mono">Invoke ${mode}</span>.` },
      { line:mode==='call'?4:5, ip:5, switch:{from:0,to:1,phase:'take'}, rootStatus:'suspended', note:'Take.' },
      { line:mode==='call'?4:5, ip:5, switch:{from:0,to:1,phase:'hole'}, note:'The hole.' },
      { line:0, ip:0, switch:{from:0,to:1,phase:'install'}, current:1, fiberStatus:{id:1,s:'running'},
        framePush:{name:'F1 entry block',offset:0,kind:'blk',locals:['self']}, push:'<blk>',
        note:'Install — first resume, so the entry frame is pushed.' },
      { line:1, ip:0, push:'<Error>', note:'<span class="mono">GetGlobal Error</span>.' },
      { line:1, ip:1, push:'"boom"', note:'The message.' },
      { line:1, ip:2, pop:2, push:'<Error boom>', note:'<span class="mono">Error.new(_)</span> — still an ordinary value. Nothing has gone wrong yet.' },
    ]);
    return [
      { source, chunks, events: [...lead('call'),
        { line:1, ip:3, unwind:{}, fiberStatus:{id:1,s:'failed'}, fiberResult:{id:1,v:'<Error boom>'},
          error:{kind:'Error', msg:'boom'},
          note:'<span class="mono">raise</span>. F1 goes <b>failed</b> and the error lands in its result slot. The unwind begins — <b>watch the chain edge.</b>' },
        { line:1, ip:3, framePopN:1, unwind:{},
          note:'F1’s frames unwind to the <b>fiber floor</b> — the bottom of what this fiber owns. Every fiber has one; the question is only what happens at it.' },
        { line:4, ip:5, switch:{from:1,to:0,phase:'take'}, note:'Take.' },
        { line:4, ip:5, switch:{from:1,to:0,phase:'hole'}, note:'The hole. Even a failing fiber parks by the same three beats.' },
        { line:4, ip:5, switch:{from:1,to:0,phase:'install'}, current:0, fiberStatus:{id:0,s:'running'},
          unwind:{}, error:{kind:'Error', msg:'boom'},
          note:'<b>Install — and the error comes with it.</b> The edge was <span class="mono">call</span>, so the floor <b>re-raises into the resumer</b>. The root is now unwinding for a failure that happened in another fiber. Containment did not happen because nobody asked for it.' },
      ]},
      { source, chunks, events: [...lead('try'),
        { line:1, ip:3, unwind:{}, fiberStatus:{id:1,s:'failed'}, fiberResult:{id:1,v:'<Error boom>'},
          error:{kind:'Error', msg:'boom'},
          note:'Identical raise. F1 goes <b>failed</b>, error in the result slot. So far nothing distinguishes this from A.' },
        { line:1, ip:3, framePopN:1, unwind:{},
          note:'Same unwind to the same floor. <b>The chain edge is doubled here</b> — and that is the only difference in the entire program.' },
        { line:5, ip:5, switch:{from:1,to:0,phase:'take'}, note:'Take.' },
        { line:5, ip:5, switch:{from:1,to:0,phase:'hole'}, note:'The hole.' },
        { line:5, ip:5, switch:{from:1,to:0,phase:'install'}, current:0, fiberStatus:{id:0,s:'running'},
          push:'<Error boom>',
          note:'<b>Install — and the root gets an ordinary value.</b> The edge was <span class="mono">try</span>, so the unwind <b>stopped at the floor</b>. The error crossed the boundary as <i>data</i>, not as an exception. F1 stays <span class="mono">failed</span> with its result readable via <span class="mono">f.error</span>.' },
      ]},
    ];
  })(),
};

/* =========================== engine ====================================== */
function blankState(){ return {
  tape:[], frames:[], fibers:{0:{id:0,name:'F0 root',status:'running',parked:null,resumer:null,mode:null,result:null}},
  current:0, cells:[], hostDepth:0, hole:false, ip:0, line:-1, chunk:null,
  changed:[], readfrom:null, error:null, hopper:[],
  // `next_frame_generation` is PINNED — it never travels on a switch. That is
  // what makes a FrameToken unique across every fiber, not just within one, and
  // it is the counter a dead-frame check reads.
  gen:1, tokens:{}, unwind:null,
};}

function cloneState(s){ return {
  tape:s.tape.slice(), frames:s.frames.map(f=>({...f})),
  fibers:Object.fromEntries(Object.entries(s.fibers).map(([k,v])=>[k,{...v,parked:v.parked?{tape:v.parked.tape.slice(),frames:v.parked.frames.map(f=>({...f}))}:null}])),
  current:s.current, cells:s.cells.map(c=>({...c})), hostDepth:s.hostDepth, hole:s.hole,
  ip:s.ip, line:s.line, chunk:s.chunk, changed:[], readfrom:null, error:null, hopper:s.hopper.slice(),
  gen:s.gen, tokens:{...s.tokens}, unwind:null,
};}

function apply(prev, ev){
  const s = cloneState(prev);
  if (ev.line !== undefined) s.line = ev.line;
  if (ev.chunk) s.chunk = ev.chunk;
  if (ev.hostDepth !== undefined) s.hostDepth = ev.hostDepth;

  if (ev.fiberNew) s.fibers[ev.fiberNew.id] = {id:ev.fiberNew.id,name:ev.fiberNew.name,status:'suspended',parked:null,resumer:null};
  if (ev.resumer){
    s.fibers[ev.resumer.id].resumer = ev.resumer.to;
    // FiberResumeMode (heap/fiber.rs:37) is an EDGE property: `call` re-raises an
    // uncaught failure into the resumer, `try` captures it at this link. That is
    // the fiber floor, and it is a property of how you resumed, not of the fiber.
    s.fibers[ev.resumer.id].mode = ev.resumer.mode || 'call';
  }
  if (ev.fiberResult) s.fibers[ev.fiberResult.id].result = ev.fiberResult.v;
  if (ev.unwind) s.unwind = ev.unwind;
  if (ev.fiberStatus) s.fibers[ev.fiberStatus.id].status = ev.fiberStatus.s;
  // asymmetric coroutines: a resumer is suspended for as long as its child runs
  if (ev.rootStatus) s.fibers[0].status = ev.rootStatus;
  // E2/E3 open mid-story, after the root already parked. E1 shows that switch in
  // full; here it is canned so the cards are still honest — a running child
  // always means its resumer's buffers are sitting in its resumer's object.
  if (ev.rootParked) s.fibers[0].parked = {
    tape:['<module>','<Fiber F1>'],
    frames:[{name:'<module>',offset:0,gen:1,kind:'m',locals:['self'],ip:3}] };

  if (ev.switch){
    const sw = ev.switch;
    if (sw.phase==='take'){
      s.fibers[sw.from].parked = { tape:s.tape.slice(), frames:s.frames.map(f=>({...f})) };
      s.tape = []; s.frames = []; s.hole = true;
    } else if (sw.phase==='hole'){
      // a real cursor stop, deliberately doing nothing
    } else if (sw.phase==='install'){
      const p = s.fibers[sw.to].parked;
      s.tape = p ? p.tape.slice() : [];
      s.frames = p ? p.frames.map(f=>({...f})) : [];
      s.fibers[sw.to].parked = null;
      s.hole = false;
    }
  }
  if (ev.current !== undefined) s.current = ev.current;

  // Generations are MINTED BY THE ENGINE from the pinned counter, never authored.
  // A FrameToken is (index, generation); a frame is dead when its index has been
  // reused by a later activation, which is only detectable if the counter is the
  // single source. Hand-written gen values would make E4's check theatre.
  if (ev.framePush){
    const f = {ip:0, ...ev.framePush, gen:s.gen++};
    if (ev.framePush.home) f.home = s.tokens[ev.framePush.home];
    s.frames.push(f);
  }
  if (ev.captureHome){
    const i = s.frames.length-1;
    s.tokens[ev.captureHome] = { index:i, gen:s.frames[i].gen, name:s.frames[i].name };
  }
  if (ev.pop) for (let i=0;i<ev.pop;i++) s.tape.pop();
  if (ev.framePop){ const f = s.frames.pop(); if (f) s.tape.length = f.offset; }
  if (ev.framePopN) for(let i=0;i<ev.framePopN;i++){ const f=s.frames.pop(); if(f) s.tape.length=f.offset; }
  if (ev.push !== undefined){ s.tape.push(ev.push); s.changed.push(s.tape.length-1); }
  if (ev.set){ s.tape[ev.set.slot]=ev.set.val; s.changed.push(ev.set.slot); }
  if (ev.readfrom !== undefined) s.readfrom = ev.readfrom;

  if (ev.upvalueOpen) s.cells.push({id:ev.upvalueOpen.id,name:ev.upvalueOpen.name,
    slot:ev.upvalueOpen.slot,fiber:ev.upvalueOpen.fiber,closed:false,value:null});
  if (ev.upvalueRead) s.readfrom = (s.cells.find(c=>c.id===ev.upvalueRead)||{}).slot ?? null;
  if (ev.upvalueWrite){
    const c = s.cells.find(x=>x.id===ev.upvalueWrite.id);
    if (c && !c.closed){ s.tape[c.slot]=ev.upvalueWrite.v; s.changed.push(c.slot); }
  }
  if (ev.upvalueClose){
    const c = s.cells.find(x=>x.id===ev.upvalueClose);
    if (c){ c.value = s.tape[c.slot]; c.closed = true; }
  }
  if (ev.error) s.error = ev.error;

  // `ip` is a CallFrame field (frame.rs:72), not VM state. It is stored on the
  // top frame — so a push starts a fresh one at 0, a pop restores the caller's,
  // and a park carries every frame's ip out with `frames`. An event's `ip` names
  // the ip of whatever frame is on top *after* the event.
  if (ev.ip !== undefined && s.frames.length) s.frames[s.frames.length-1].ip = ev.ip;
  s.ip = s.frames.length ? (s.frames[s.frames.length-1].ip ?? 0) : (ev.ip ?? prev.ip);
  return s;
}

function buildStates(events){
  const out=[blankState()];
  for (const e of events) out.push(apply(out[out.length-1], e));
  return out;
}

function invariants(states, events){
  const p=[];
  states.forEach((s,i)=>{
    if (s.tape.length>SLOTS) p.push(`event ${i}: tape (${s.tape.length}) exceeds ${SLOTS} rendered slots`);
    if (s.readfrom!==null && s.readfrom>=s.tape.length && !s.hole)
      p.push(`event ${i}: reads slot ${s.readfrom}, past tape top ${s.tape.length}`);
    s.frames.forEach((f,j)=>{
      if (f.offset>s.tape.length) p.push(`event ${i}: frame ${j} (${f.name}) offset ${f.offset} past tape top ${s.tape.length}`);
      const nx=s.frames[j+1];
      if (nx && nx.offset<f.offset) p.push(`event ${i}: frame ${j+1} opens below its caller`);
    });
    s.cells.forEach(c=>{
      if (!c.closed && c.fiber===s.current && !s.hole && c.slot>=s.tape.length)
        p.push(`event ${i}: open cell "${c.name}" points at slot ${c.slot}, past the live tape top ${s.tape.length}`);
    });
  });
  let d=0, phase=null;
  events.forEach((e,i)=>{
    if (e.framePush) d++;
    if (e.framePop){ d--; if(d<0) p.push(`event ${i+1}: frame_pop without a matching push`); }
    if (e.switch){
      const seq={take:'hole',hole:'install',install:null};
      if (e.switch.phase==='take' && phase!==null) p.push(`event ${i+1}: switch 'take' while a switch is already in progress`);
      if (e.switch.phase!=='take' && seq[phase]!==e.switch.phase) p.push(`event ${i+1}: switch phase '${e.switch.phase}' out of order (expected '${seq[phase]}')`);
      phase = e.switch.phase==='install' ? null : e.switch.phase;
    }
    if (e.switch && e.switch.phase==='take'){
      const st=states[i]; if (st.hostDepth>1) p.push(`event ${i+1}: switch attempted at native_reentry_depth ${st.hostDepth-1} (must be 0)`);
    }
  });
  return p;
}
