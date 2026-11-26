// Native = wasm parity check: plays the golden game in wasm (the wasm build's own copy of
// hh-core's driver, `play_golden`) and checks it reaches the golden fingerprint
// `golden_game` pins natively (crates/hh-core/src/tests/mod.rs); then plays the same
// record again step by step through the JSON API (new_game, advance, apply) and replays it.
// Runs the page's own build, web/src/wasm. Usage (after `node web/scripts/build-wasm.mjs`,
// which skips when its inputs are unchanged):
//   node scripts/wasm-smoke.mjs            exit 0 ok, 1 on any mismatch
import { readFileSync } from 'node:fs';
import { fileURLToPath, pathToFileURL } from 'node:url';
import path from 'node:path';

// Must equal GOLDEN_* in crates/hh-core/src/tests/mod.rs (a fast-lane test checks it).
const GOLDEN_REVISION = 37;
const GOLDEN_FINGERPRINT = '062942f2ff52682a';
const GOLDEN_CONTENT_HASH = 'f67a63efc195f121';
const GOLDEN_CONFIG_HASH = '99a96d52255bdd04';
// The golden game (crates/hh-core/src/driver.rs): baseline, seed u64::MAX, 2000 ticks of
// the scripted policy.
const SEED = '18446744073709551615';
const TICKS = 2000;

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const pkg = path.join(root, 'web', 'src', 'wasm');
const m = await import(pathToFileURL(path.join(pkg, 'hh_wasm.js')).href);
await m.default({ module_or_path: readFileSync(path.join(pkg, 'hh_wasm_bg.wasm')) });

const failures = [];
const check = (ok, what) => {
  console.log(`${ok ? 'ok  ' : 'FAIL'} ${what}`);
  if (!ok) failures.push(what);
};

const cfg = readFileSync(path.join(root, 'configs', 'baseline.json'), 'utf8');
const resolved = JSON.parse(m.resolve_config(cfg));

const golden = JSON.parse(m.play_golden(cfg));
const record = golden.record;
check(m.rules_revision() === GOLDEN_REVISION, `rules revision ${m.rules_revision()} (golden ${GOLDEN_REVISION})`);
check(golden.fingerprint === GOLDEN_FINGERPRINT, `golden fingerprint ${golden.fingerprint} (native ${GOLDEN_FINGERPRINT}), ${record.actions.length} actions`);
check(m.content_hash() === GOLDEN_CONTENT_HASH, `content hash ${m.content_hash()} (golden ${GOLDEN_CONTENT_HASH})`);
check(golden.config_hash === GOLDEN_CONFIG_HASH, `config hash ${golden.config_hash} (golden ${GOLDEN_CONFIG_HASH})`);
check(record.seed === SEED && record.ticks === TICKS, `golden record: seed ${record.seed}, ${record.ticks} ticks`);

// The same actions through the JSON API, one call at a time, from the golden's
// start (the `golden_start` scenario: `if` open).
let state = m.new_game(cfg, SEED, 'golden_start');
let tick = 0;
for (const { tick: at, action } of record.actions) {
  if (at > tick) state = m.advance(state, at - tick);
  tick = at;
  state = m.apply(state, JSON.stringify(action));
}
state = m.advance(state, TICKS - tick);
check(m.fingerprint(state) === golden.fingerprint, 'the record played through new_game / advance / apply');
const view = JSON.parse(m.view(state));
check(view.tick === TICKS && view.resources.length > 0 && view.buildings.length > 0, `view at tick ${view.tick}`);

const replayed = JSON.parse(m.replay(JSON.stringify(record)));
check(replayed.fingerprint === golden.fingerprint && replayed.warnings.length === 0, 'record replays in wasm');
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
throws(() => m.resolve_config(cfg.replace('"land_base_cost"', '"land_base_costs"')), /land_base_cost: missing/, 'bad config names the key');
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
