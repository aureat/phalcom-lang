#!/usr/bin/env node
//! Trace validator for the execution visualizer.
//!
//! The traces in `index.html` are hand-authored, and a hand-authored trace can be
//! silently wrong — an off-by-one `stack_offset` draws a confident lie, which is
//! worse than no picture at all (REQUIREMENTS.md §8, failure mode 1). This script
//! is the only defence.
//!
//! It extracts the engine out of the page (everything above the render section,
//! which is DOM-dependent) and replays every trace, applying two kinds of check:
//!
//!   structural — the invariants the player itself runs (frame bounds, switch
//!                phase ordering, pop/push balance, cell targets)
//!   semantic   — assertions about what each trace is supposed to *teach*, which
//!                structural checks cannot catch. The load-bearing one is
//!                TWO-FULL-CARDS: at every hole except a fiber's first resume,
//!                both fibers must be parked and the VM empty. That frame is the
//!                whole reason the tool exists; if a trace stops producing it,
//!                the tool has quietly lost its point.
//!
//! Usage:  node tools/viz/check.mjs        (exit 1 on any problem)

import { readFileSync } from 'fs';
import { fileURLToPath } from 'url';
import { dirname, join } from 'path';

const here = dirname(fileURLToPath(import.meta.url));
const html = readFileSync(join(here, 'index.html'), 'utf8');
const body = html.split('<script>')[1].split('</script>')[0];
const logic = body.split('/* =========================== render')[0];
const mod = logic + '\nexport { EXAMPLES, buildStates, invariants, SLOTS };\n';
const m = await import('data:text/javascript;base64,' + Buffer.from(mod).toString('base64'));

let problems = 0;
const fail = (name, msg) => { problems++; console.log(`    ✗ ${msg}`); };

for (const [key, ex] of Object.entries(m.EXAMPLES)) {
  const subs = ex.sub ? ex.sub.map((s, i) => [`${key}[${i}] ${ex.variants?.[i] ?? ''}`, s]) : [[key, ex]];

  for (const [name, sub] of subs) {
    const states = m.buildStates(sub.events);
    const last = states[states.length - 1];
    console.log(`\n=== ${name} — ${sub.events.length} events ===`);

    const bad = [...m.invariants(states, sub.events)];

    // ---- chunk / ip / line bounds -----------------------------------------
    sub.events.forEach((e, i) => {
      const st = states[i + 1];
      const top = st.frames.length ? st.frames[st.frames.length - 1] : null;
      const cn = top ? top.name : null;          // no frames = nothing executing
      const ch = cn ? sub.chunks[cn] : null;
      if (cn && !ch) bad.push(`event ${i + 1}: no chunk named "${cn}"`);
      if (ch && st.ip >= ch.code.length) bad.push(`event ${i + 1}: ip ${st.ip} past end of "${cn}" (${ch.code.length} ops)`);
      if (e.line !== undefined && e.line >= sub.source.length) bad.push(`event ${i + 1}: line ${e.line} past source (${sub.source.length} lines)`);
    });

    // ---- semantic: the hole -------------------------------------------------
    // A hole must have an empty VM. Both participating fibers must be parked,
    // EXCEPT when the target has never run (first resume: nothing to park yet).
    const holes = [];
    sub.events.forEach((e, i) => {
      if (!(e.switch && e.switch.phase === 'hole')) return;
      const st = states[i + 1];
      const from = st.fibers[e.switch.from], to = st.fibers[e.switch.to];
      const started = sub.events.slice(0, i).some(p =>
        p.switch && p.switch.phase === 'install' && p.switch.to === e.switch.to);
      const full = [from, to].filter(f => f && f.parked).length;
      holes.push({ ev: i + 1, full, firstResume: !started && e.switch.to !== 0 });

      if (st.tape.length || st.frames.length)
        bad.push(`event ${i + 1}: hole but the VM still holds state (tape ${st.tape.length}, frames ${st.frames.length})`);
      if (!from?.parked)
        bad.push(`event ${i + 1}: hole but the outgoing fiber "${from?.name}" is not parked`);
      if (started && !to?.parked)
        bad.push(`event ${i + 1}: hole but the resuming fiber "${to?.name}" has already run and is not parked`);
    });

    // ---- semantic: switch legality -----------------------------------------
    sub.events.forEach((e, i) => {
      if (e.switch && e.switch.phase === 'take' && states[i].hostDepth > 1)
        bad.push(`event ${i + 1}: switch at native_reentry_depth ${states[i].hostDepth - 1} — must be 0`);
    });

    // ---- report -------------------------------------------------------------
    console.log(`  final tape   : [${last.tape.join(', ')}]`);
    console.log(`  final frames : ${last.frames.map(f => f.name + '@' + f.offset).join(' | ') || '(none)'}`);
    console.log(`  fibers       : ${Object.values(last.fibers).map(f => `${f.name}=${f.status}${f.parked ? '(parked)' : ''}`).join(', ')}`);
    console.log(`  cells        : ${last.cells.map(c => c.closed ? `${c.name}=closed(${c.value})` : `${c.name}→fiber${c.fiber}:slot${c.slot}`).join(', ') || '(none)'}`);
    console.log(`  error        : ${last.error ? last.error.kind : '—'}`);
    if (holes.length) console.log(`  holes        : ${holes.map(h => `ev${h.ev}=${h.full}/2 full${h.firstResume ? ' (first resume)' : ''}`).join(', ')}`);

    const twoFull = holes.filter(h => !h.firstResume);
    if (twoFull.length && twoFull.some(h => h.full !== 2))
      bad.push(`TWO-FULL-CARDS: a non-first-resume hole does not show both fibers parked — the crown-jewel frame is broken`);

    if (bad.length) { bad.forEach(b => fail(name, b)); }
    else console.log(`  ✓ clean`);
  }
}

console.log(problems ? `\n${problems} problem(s)` : `\nall traces clean`);
process.exit(problems ? 1 : 0);
