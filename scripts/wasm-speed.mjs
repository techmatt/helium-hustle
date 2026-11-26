// Engine speed in wasm, the same reads as `hh-sim speed` natively: the golden game (the
// script reads a view each tick) and a plain advance of a new game, in ticks per second.
// Runs the page's build, web/src/wasm. Usage (after `node web/scripts/build-wasm.mjs`):
//   node scripts/wasm-speed.mjs            prints one line
import { readFileSync } from 'node:fs';
import { fileURLToPath, pathToFileURL } from 'node:url';
import path from 'node:path';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const pkg = path.join(root, 'web', 'src', 'wasm');
const m = await import(pathToFileURL(path.join(pkg, 'hh_wasm.js')).href);
await m.default({ module_or_path: readFileSync(path.join(pkg, 'hh_wasm_bg.wasm')) });

const cfg = readFileSync(path.join(root, 'configs', 'baseline.json'), 'utf8');
let t = performance.now();
const golden = m.play_golden(cfg);
const seconds = (performance.now() - t) / 1000;
// The golden game's length from its own record, not a copy of driver.rs's GOLDEN_TICKS.
const scripted = JSON.parse(golden).record.ticks / seconds;
const N = 20000;
const state = m.new_game(cfg, '1', '');
t = performance.now();
m.advance(state, N);
const plain = N / ((performance.now() - t) / 1000);
console.log(`engine speed, wasm: ${scripted.toFixed(0)} ticks/s scripted (golden game, a view a tick), ${plain.toFixed(0)} ticks/s plain advance`);
