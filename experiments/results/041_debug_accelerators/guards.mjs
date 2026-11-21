// 041's review fixes, in the browser: while the autopilot plays, a speed or a hand action
// is refused and the record stays whole; Stop ends it; *Next milestone* twice moves on
// (the milestones met before stay met); the replay check passes. Same harness as timing.mjs.
//   HH_URL  the page, required (a `vite preview`)
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { READY, driver, launch, pageUrl, sleep } from '../../../web/scripts/cdp.mjs';

const here = dirname(fileURLToPath(import.meta.url));
const repo = resolve(here, '../../..');
const url = pageUrl('guards', 'node experiments/results/041_debug_accelerators/guards.mjs');
const b = await launch({ width: 1280, height: 800, profileRoot: resolve(repo, 'out/041_debug_accelerators/profile-guards') });
const { evaluate } = b;
const { waitFor, click, setValue, load, cheats } = driver(b, { url });
const status = () => evaluate(`document.getElementById('accel-status').textContent`);
const running = `document.getElementById('accel-status')?.dataset.running`;
const fail = (m) => {
  console.error(`FAIL ${m}`);
  process.exitCode = 1;
};
await load();
await evaluate('localStorage.clear()');
await load();
await cheats();
await setValue('#seed', '1');
await click('#new-game');
await waitFor(READY, 'a new game');
await cheats();
await click('#accel-end');
await waitFor(`${running} === '1'`, 'the autopilot to start', 5000);
await sleep(300);
await click('#speeds button[data-speed="1"]');
const notice = await evaluate(`document.body.textContent.includes('The autopilot is playing')`);
if (!notice) fail('a speed during the autopilot was not refused');
else console.log('ok   a speed is refused while the autopilot plays');
await click('#accel-stop');
await waitFor(`${running} === '0'`, 'Stop', 60000);
console.log(`ok   stopped: ${await status()}`);
// Next milestone, twice: the second aims past the first.
await click('#accel-milestone');
await waitFor(`${running} === '0' && /^(Reached|Gave)/.test(document.getElementById('accel-status').textContent)`, 'a milestone', 120000);
const first = await status();
await click('#accel-milestone');
await sleep(200);
await waitFor(`${running} === '0'`, 'the next milestone', 120000);
const second = await status();
console.log(`     ${first}\n     ${second}`);
const lastOf = (s) => (s.match(/last: ([^)]*)\)/) ?? [])[1];
if (!first.startsWith('Reached') || !second.startsWith('Reached') || lastOf(first) === lastOf(second)) fail('Next milestone did not move on');
else console.log('ok   Next milestone moves on');
await click('#check-replay');
await waitFor(`/Replay (ok|MISMATCH|failed)/.test(document.body.textContent)`, 'the replay check', 120000);
const ok = await evaluate(`/Replay ok/.test(document.body.textContent)`);
if (!ok) fail('replay check');
else console.log('ok   replay check');
await b.close();
