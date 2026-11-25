// Where the page's main thread goes while the game runs (094): for each case (early: a
// new game; late: the `everything` scenario) and speed, the busy milliseconds per wall
// second by kind (script, layout, style) from Performance.getMetrics, and the top
// functions by self time from a CPU profile. Against `vite preview` of a static build:
//   cd web && npm run build && npx vite preview --port 60104 --strictPort
//   HH_URL=http://localhost:60104/ node experiments/results/094_profiling_pass/page_profile.mjs [out.json]
import { writeFileSync, mkdirSync } from 'node:fs';
import { resolve } from 'node:path';
import { launch, driver, pageUrl, sleep } from '../../../web/scripts/cdp.mjs';

const url = pageUrl('page_profile', 'node experiments/results/094_profiling_pass/page_profile.mjs');
const SECONDS = Number(process.env.HH_SECONDS ?? 6);
const out = process.argv[2];
mkdirSync(resolve('out/094_profiling_pass'), { recursive: true });
const b = await launch({ width: 1600, height: 1000, profileRoot: resolve('out/094_profiling_pass/profiles') });
const d = driver(b, { url });
const results = [];

async function metrics() {
  const r = await b.send('Performance.getMetrics');
  return Object.fromEntries(r.result.metrics.map((m) => [m.name, m.value]));
}

async function measure(label, speed) {
  await d.speed(speed);
  await sleep(1000);
  await b.send('Profiler.setSamplingInterval', { interval: 200 });
  await b.send('Profiler.start');
  const m0 = await metrics();
  const day0 = await d.day();
  await sleep(SECONDS * 1000);
  const m1 = await metrics();
  const day1 = await d.day();
  const prof = (await b.send('Profiler.stop')).result.profile;
  await d.speed(0);
  const per = (k) => ((m1[k] - m0[k]) * 1000) / SECONDS; // ms busy per wall second
  // Self time by function (and file), from the sample counts.
  const dt = (prof.endTime - prof.startTime) / 1000 / prof.samples.length; // ms per sample
  const counts = new Map();
  for (const s of prof.samples) counts.set(s, (counts.get(s) ?? 0) + 1);
  const self = new Map();
  for (const n of prof.nodes) {
    const c = counts.get(n.id) ?? 0;
    if (!c) continue;
    const f = n.callFrame;
    const file = f.url.split('/').pop() || '';
    const key = `${f.functionName || '(anon)'} ${file}`;
    self.set(key, (self.get(key) ?? 0) + c * dt);
  }
  // Inclusive time (self and callees) by function, each counted once per sample.
  const parent = new Map();
  const byId = new Map(prof.nodes.map((n) => [n.id, n]));
  for (const n of prof.nodes) for (const c of n.children ?? []) parent.set(c, n.id);
  const incl = new Map();
  for (const s of prof.samples) {
    const seen = new Set();
    for (let id = s; id !== undefined; id = parent.get(id)) {
      const f = byId.get(id).callFrame;
      const key = `${f.functionName || '(anon)'} ${f.url.split('/').pop() || ''}`;
      if (!seen.has(key)) {
        seen.add(key);
        incl.set(key, (incl.get(key) ?? 0) + dt);
      }
    }
  }
  const inclusive = [...incl.entries()].filter(([k]) => /^[A-Z][A-Za-z]+ index|^(runTicks|refresh|track|tell|emit|wake|advanceRun|view) /.test(k))
    .sort((a, b) => b[1] - a[1]).slice(0, 15).map(([k, ms]) => [k, +(ms / SECONDS).toFixed(1)]);
  const top = [...self.entries()].filter(([k]) => !k.startsWith('(idle)')).sort((a, b) => b[1] - a[1]).slice(0, 15)
    .map(([k, ms]) => [k, +(ms / SECONDS).toFixed(1)]);
  const row = {
    case: label, speed, days_per_s: +((day1 - day0) / SECONDS).toFixed(1),
    task_ms_per_s: +per('TaskDuration').toFixed(0), script: +per('ScriptDuration').toFixed(0),
    layout: +per('LayoutDuration').toFixed(0), style: +per('RecalcStyleDuration').toFixed(0), top, inclusive,
  };
  results.push(row);
  console.log(`${label} ${speed}x: ${row.days_per_s} days/s, busy ${row.task_ms_per_s} ms/s (script ${row.script}, layout ${row.layout}, style ${row.style})`);
  for (const [k, ms] of top.slice(0, 10)) console.log(`    ${String(ms).padStart(6)} ms/s  ${k}`);
  console.log('  inclusive (named functions, unminified builds):');
  for (const [k, ms] of inclusive.slice(0, 12)) console.log(`    ${String(ms).padStart(6)} ms/s  ${k}`);
}

try {
  await b.send('Performance.enable');
  await b.send('Profiler.enable');
  await d.load();
  await d.autoStory(); // a fresh profile loads a new game
  await sleep(500);
  for (const s of [1, 200]) await measure('early', s);
  await d.debugMenu();
  await d.setValue('#scenario', 'everything');
  await d.click('#start-scenario');
  await sleep(500);
  await d.autoStory();
  await d.panel('buildings');
  for (const s of [1, 200]) await measure('late', s);
  if (out) writeFileSync(out, JSON.stringify(results, null, 1));
  if (b.errors.length) console.log('page errors:', b.errors.slice(0, 5));
} finally {
  b.close();
}
