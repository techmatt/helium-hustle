// Native = wasm parity check: plays the golden game through the wasm build's JSON
// API and checks it reaches the golden fingerprint `golden_game` pins natively
// (crates/hh-core/src/tests/mod.rs), then replays its record in wasm.
// Usage (after `wasm-pack build crates/hh-wasm --target web`):
//   node scripts/wasm-smoke.mjs            exit 0 ok, 1 on any mismatch
import { readFileSync } from 'node:fs';
import { fileURLToPath, pathToFileURL } from 'node:url';
import path from 'node:path';

// Must equal GOLDEN_* in crates/hh-core/src/tests/mod.rs (a fast-lane test checks it).
const GOLDEN_REVISION = 1;
const GOLDEN_FINGERPRINT = '36d8cc14b1f9de2a';
const GOLDEN_CONTENT_HASH = '8f392297a9a3af48';
// The golden game (crates/hh-core/src/driver.rs): baseline, seed u64::MAX,
// 2000 ticks, every 7 ticks buy each affordable building in view order.
const SEED = '18446744073709551615';
const TICKS = 2000;
const EVERY = 7;

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const pkg = path.join(root, 'crates', 'hh-wasm', 'pkg');
const m = await import(pathToFileURL(path.join(pkg, 'hh_wasm.js')).href);
await m.default({ module_or_path: readFileSync(path.join(pkg, 'hh_wasm_bg.wasm')) });

const failures = [];
const check = (ok, what) => {
  console.log(`${ok ? 'ok  ' : 'FAIL'} ${what}`);
  if (!ok) failures.push(what);
};

const cfg = readFileSync(path.join(root, 'configs', 'baseline.json'), 'utf8');
const resolved = JSON.parse(m.resolve_config(cfg));

let state = m.new_game(cfg, SEED, '');
const actions = [];
let tick = 0;
while (tick < TICKS) {
  const n = Math.min(EVERY, TICKS - tick);
  state = m.advance(state, n);
  tick += n;
  const count = JSON.parse(m.view(state)).buildings.length;
  for (let i = 0; i < count; i++) {
    const b = JSON.parse(m.view(state)).buildings[i]; // re-read after each buy
    if (b.affordable) {
      state = m.apply(state, JSON.stringify(b.buy));
      actions.push({ tick, action: b.buy });
    }
  }
}
const fp = m.fingerprint(state);
check(m.rules_revision() === GOLDEN_REVISION, `rules revision ${m.rules_revision()} (golden ${GOLDEN_REVISION})`);
check(fp === GOLDEN_FINGERPRINT, `golden fingerprint ${fp} (native ${GOLDEN_FINGERPRINT}), ${actions.length} buys`);
check(m.content_hash() === GOLDEN_CONTENT_HASH, `content hash ${m.content_hash()} (golden ${GOLDEN_CONTENT_HASH})`);

const record = {
  format: 'hh-record/1',
  rules_revision: m.rules_revision(),
  config: resolved.config,
  seed: SEED,
  scenario: null,
  actions,
  ticks: TICKS,
  fingerprint: fp,
};
const replayed = JSON.parse(m.replay(JSON.stringify(record)));
check(replayed.fingerprint === fp && replayed.warnings.length === 0, 'record replays in wasm');
check(m.config_hash(state) === resolved.hash, 'config hash');

const throws = (f, pattern, what) => {
  try {
    f();
    check(false, what);
  } catch (e) {
    check(typeof e === 'string' && pattern.test(e), `${what}: ${e}`);
  }
};
throws(() => m.apply(m.new_game(cfg, '1', ''), JSON.stringify({ type: 'buy', building: 'fusion' })), /unknown building/, 'illegal action throws');
throws(() => m.resolve_config(cfg.replace('"energy_cap"', '"energy_capp"')), /energy_cap: missing/, 'bad config names the key');
throws(() => m.new_game(cfg, '-1', ''), /bad seed/, 'bad seed throws');

for (const s of JSON.parse(m.scenarios())) {
  const g = JSON.parse(m.new_game(cfg, '1', s.name));
  check(g.tick === s.start.tick, `scenario ${s.name} starts at day ${g.tick}`);
}

console.log(`engine ${m.version()}, config ${resolved.hash}`);
if (failures.length) {
  console.error(`wasm smoke FAILED: ${failures.length} check(s)`);
  process.exit(1);
}
console.log('wasm smoke ok: native = wasm');
