// What one page wake costs in wasm, by the calls store.ts makes (094): advance_run of a
// wake's ticks at each speed, then view, then the page's JSON.parse of both (and the
// head() parse of the state), on an early state and a late one.
// Usage, from the repo root, after `node web/scripts/build-wasm.mjs`:
//   node experiments/results/094_profiling_pass/wake_wasm.mjs [<another web/src/wasm>]
import { readFileSync } from 'node:fs';
import { fileURLToPath, pathToFileURL } from 'node:url';
import path from 'node:path';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..', '..', '..');
// Another build (another revision's web/src/wasm) as the first argument.
const pkg = process.argv[2] ? path.resolve(process.argv[2]) : path.join(root, 'web', 'src', 'wasm');
const m = await import(pathToFileURL(path.join(pkg, 'hh_wasm.js')).href);
await m.default({ module_or_path: readFileSync(path.join(pkg, 'hh_wasm_bg.wasm')) });
const cfg = readFileSync(path.join(root, 'configs', 'baseline.json'), 'utf8');

function time(n, f) {
  for (let i = 0; i < 5; i++) f();
  const t = performance.now();
  for (let i = 0; i < n; i++) f();
  return ((performance.now() - t) / n) * 1000; // microseconds
}

function read(label, state) {
  const v = m.view(state);
  console.log(`${label}: state JSON ${(state.length / 1024).toFixed(0)} KB, view JSON ${(v.length / 1024).toFixed(0)} KB`);
  const row = (what, us) => console.log(`  ${what.padEnd(34)} ${us.toFixed(0).padStart(8)} us`);
  for (const ticks of [1, 5, 20]) row(`advance_run ${ticks} (${ticks * 10}x wake)`, time(40, () => m.advance_run(state, ticks)));
  row('view', time(40, () => m.view(state)));
  row('fingerprint', time(40, () => m.fingerprint(state)));
  row('JSON.parse(view)', time(200, () => JSON.parse(v)));
  row('JSON.parse(state) (head)', time(200, () => JSON.parse(state)));
  // As the page: each wake starts from the state the last one returned.
  let s = state;
  const wake = (ticks) => {
    const r = JSON.parse(m.advance_run(s, ticks));
    s = r.state;
    JSON.parse(m.view(s));
  };
  for (const ticks of [1, 20]) {
    s = state;
    row(`whole wake, ${ticks} tick(s)`, time(40, () => wake(ticks)));
  }
}

let early = m.advance(m.new_game(cfg, '1', ''), 20);
read('early (new game, day 20)', early);
let late = m.advance(m.new_game(cfg, '1', 'everything'), 60);
read('late (everything, +60 days)', late);
