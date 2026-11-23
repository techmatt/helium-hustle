// Browser check: drives headless Edge or Chrome over the DevTools protocol (no
// packages) against a served page.
//   1. A fresh game through the normal UI only (no debug menu): a build order to the
//      first pad loaded and launched (1.1 Online and 1.2 First Shipment), no Retirement
//      tile yet (it opens at 1.5 Burnout), then on at 200x to the forced retirement at the
//      boredom cap, the run summary holding the clock (a double-click on Continue buys
//      nothing under it), run 2 (known tiles and resources still shown, not cheated), the
//      career's play log on disk as a bot's.
//   2. Every panel: scenario late_unlocks opens Research, Projects and Ideologies (1.6 Lab
//      is past a second run); a locked and a funded project; every nav tile opens; the
//      cheated mark, a jump ahead, the cheat building and Complete all research, and the
//      replay check.
//   3. A program's current-line frame at 10x and 50x, each speed's days per second,
//      pause, click-to-buy (a green card's text buys, its −/+ and Sell 1 don't, space after
//      a click pauses and buys nothing, a grey card says why), a reload keeping the save,
//      a save from another revision discarded, the scenarios and the replay check.
//   5. (Run before 4.) Readability: a capped resource's positive rate (Resources and the
//      status bar; Energy's green), Resources' two groups, an affordable card's tinted
//      header and the Land row's green, scenario pad_loading's full pad short on fuel
//      then Ready (green Launch, border, nav dot), a boredom beat that leaves the main
//      quest pinned and counts unread in a folded Story until it is open and on screen
//      for a second, and phase 3 superseding it in Story. Shots read-*.png, parked-*.png.
//   4. The theme: light by default; the Options and debug toggles switch it with no
//      element moving, and a reload keeps it.
//   7. (Run before 5.) The program text editor, from scenario everything: a pasted
//      program with a red (junk) and an amber (locked) line, Apply, green and grey dots
//      and an if's fired count; the block editor keeps every line (unparsed and locked
//      rows); the text round-trips; a tab switch saves; a loadout of it loads in the
//      next run with every line. Shot text-editor.png.
// Any uncaught page error fails the run.
//   HH_URL      the page, required: a `vite preview` of a static build (TESTING.md)
//   HH_BROWSER  a browser off the usual paths
//   HH_PROFILE  browser profile dir (default ../out/e2e-profile, wiped first)
//   HH_SHOTS    <dir>: save a PNG of the page at each step worth seeing
// The DevTools harness (launch, waitFor, load, panel...) is scripts/cdp.mjs.
import { existsSync, mkdirSync, readFileSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { capture, driver, launch, pageUrl, q, sleep } from './cdp.mjs';

const here = dirname(fileURLToPath(import.meta.url));
const url = pageUrl('e2e');
const b = await launch({ width: 1600, height: 900, profileRoot: process.env.HH_PROFILE ?? resolve(here, '../../out/e2e-profile') });
const { send, evaluate } = b;
const pageErrors = b.errors;
// settle: a dev server just after an edit may reload the page once more.
const { waitFor, exists, click, setValue, load, day, run, speed, panel, debugMenu, grant, runOnly } = driver(b, { url, settle: 700 });
const noticeText = () => evaluate(`document.getElementById('notice')?.textContent ?? ''`);
const enabled = (sel) => evaluate(`(() => { const b = document.querySelector(${q(sel)}); return !!b && !b.disabled; })()`);
const navVisible = (id) => exists(`.nav-tile[data-nav="${id}"]`);

/** With HH_SHOTS=<dir>, saves a PNG of the page at each named moment. */
const shot = async (name) => {
  if (!process.env.HH_SHOTS) return;
  mkdirSync(process.env.HH_SHOTS, { recursive: true });
  await capture(b, resolve(process.env.HH_SHOTS, `${name}.png`));
};

const failures = [];
const check = (ok, what) => {
  console.log(`${ok ? 'ok  ' : 'FAIL'} ${what}`);
  if (!ok) failures.push(what);
};

/** What the player reads off the screen each look: resources, buildings, pads. */
const look = () =>
  evaluate(`(() => {
    const res = {};
    for (const tr of document.querySelectorAll('tr[data-res]')) res[tr.dataset.res] = Number(tr.dataset.amount);
    const quest = document.querySelector('[data-testid="quest"]')?.dataset.quest ?? null;
    const lines = [...document.querySelectorAll('.lines li')].map((li) => li.dataset.command);
    return { res, quest, lines, day: Number(document.getElementById('day').textContent.replace(/\\D/g, '')) };
  })()`);
const buildings = () =>
  evaluate(`(() => {
    const out = {};
    for (const c of document.querySelectorAll('.card.building')) {
      out[c.dataset.id] = { owned: Number(c.dataset.owned), short: c.dataset.short.split(' ').filter(Boolean), buyable: c.classList.contains('can-buy') };
    }
    return out;
  })()`);

/** Program 1, one processor, lines of counts 2, 1, 3: at 10x (a day a redraw) the
 * current-line frame walks the queue in order, the lines keep their order and no line's
 * name moves; at 50x (several days a redraw) no frame hops around, and the lines show
 * what ran over the span. Leaves the game paused. */
async function programOrder() {
  const queue = [['idle', 2], ['cloud_compute', 1], ['idle', 3]];
  await speed(0);
  await click('.prog-tabs button[data-program="0"]');
  if (await exists('.lines .line')) await click('.prog-procs button[title="Clear this program"]');
  while (Number(await evaluate(`document.querySelector('.prog-procs span').textContent.match(/\\d+/)[0]`)) > 1) {
    await click('[data-testid="proc-minus"]');
  }
  if (!(await enabled('[data-testid="proc-minus"]'))) await click('[data-testid="proc-plus"]');
  await panel('commands');
  for (const [c] of queue) await click(`[data-testid="add-${c}"]`);
  for (const [i, [, n]] of queue.entries()) {
    for (let k = 1; k < n; k++) await click(`.lines .line[data-line="${i}"] button:nth-of-type(2)`);
  }
  const order = () => evaluate(`[...document.querySelectorAll('.lines .line')].map((l) => l.dataset.command + 'x' + l.textContent.match(/\\(x(\\d+)\\)/)[1]).join(' ')`);
  const queued = queue.map(([c, n]) => `${c}x${n}`).join(' ');
  check((await order()) === queued, `program queued in order: ${await order()}`);
  // Each redraw's pointer, and where each line's name starts (it must never move).
  await evaluate(`(() => { window.__ptr = []; window.__left = new Set(); window.__ob?.disconnect();
    window.__ob = new MutationObserver(() => {
      const p = [...document.querySelectorAll('.lines .line')].findIndex((l) => l.classList.contains('current'));
      if (window.__ptr.at(-1) !== p) window.__ptr.push(p);
      for (const n of document.querySelectorAll('.lines .line-name')) {
        const r = document.createRange();
        r.selectNodeContents(n);
        window.__left.add(Math.round(r.getBoundingClientRect().left));
      }
    });
    window.__ob.observe(document.querySelector('.lines'), { subtree: true, attributes: true, childList: true }); })()`);
  await speed(10);
  await sleep(2500);
  await speed(0);
  const ptr = await evaluate('window.__ptr');
  const forward = ptr.every((p, i) => i === 0 || p === (ptr[i - 1] + 1) % queue.length);
  check(ptr.length >= 4 && forward, `10x: the pointer walks the queue in order: ${ptr.join(',')}`);
  check((await order()) === queued, `10x: lines keep their order: ${await order()}`);
  await shot('program-lines');
  const lefts = await evaluate('[...window.__left]');
  check(lefts.length === 1, `10x: no line's text moves as the pointer walks (name starts at ${lefts.join(', ')} px)`);
  await evaluate('window.__ptr = []');
  await speed(50);
  await sleep(1000);
  // Read before pausing: paused, the frame is back.
  const fast = await evaluate('window.__ptr');
  const lit = await evaluate(`document.querySelectorAll('.lines .line[class*="out-"]').length`);
  const note = await exists('[data-testid="lines-span"]');
  const fastOrder = await order();
  await speed(0);
  check(
    fast.length > 0 && fast.every((p) => p === -1) && note && lit >= 2,
    `50x: no hopping pointer, lines lit by the span (pointer ${fast.join(',')}, ${lit} lit)`,
  );
  check(fastOrder === queued, `50x: lines keep their order: ${fastOrder}`);
  await evaluate('window.__ob.disconnect()');
}

// The opening, as a player would: Solar Panels, then the extraction chain, storage for
// the Launch Pad's titanium, the pad, and more Solar Panels for its upkeep.
const ORDER = [
  ['solar_panel', 2], ['excavator', 1], ['smelter', 1], ['refinery', 1], ['solar_panel', 3],
  ['excavator', 2], ['storage_depot', 1], ['solar_panel', 4], ['launch_pad', 1], ['solar_panel', 5],
];

async function playToFirstShipment() {
  await speed(50);
  const started = Date.now();
  let lastLog = 0;
  const seenQuests = new Set();
  while (Date.now() - started < 180_000) {
    const s = await look();
    if (s.quest) seenQuests.add(s.quest);
    // 1.2 First Shipment done: the pinned quest has moved past it.
    if (s.quest && s.quest !== 'qshipment_proof_of_concept' && seenQuests.has('qshipment_proof_of_concept')) return true;
    if (Date.now() - lastLog > 5000) {
      lastLog = Date.now();
      console.log(`     day ${s.day}, quest ${s.quest}, credits ${s.res.credits}, he3 ${s.res.he3 ?? '-'}`);
    }
    await panel('buildings');
    const b = await buildings();
    const goal = ORDER.find(([id, n]) => b[id] && b[id].owned < n);
    if (goal && b[goal[0]].buyable) {
      await click(`[data-testid="buy-${goal[0]}"]`);
      continue;
    }
    const credits = s.res.credits ?? 0;
    let cmd = 'cloud_compute';
    if ((b.launch_pad?.owned ?? 0) > 0) {
      await panel('launch_pads');
      const cargo = Number(await evaluate(`document.querySelector('[data-pad="0"]')?.dataset.cargo ?? 0`));
      if (cargo >= 10 && (await enabled('[data-testid="launch-0"]'))) {
        await click('[data-testid="launch-0"]');
        continue;
      }
      if ((s.res.propellant ?? 0) < 20 && credits >= 12) cmd = 'buy_propellant';
      else if ((s.res.he3 ?? 0) >= 5 && cargo < 10) cmd = 'load_pads';
    } else if (goal) {
      const short = b[goal[0]].short;
      if (short.includes('titanium') && credits >= 25) cmd = 'buy_titanium';
      else if (short.includes('regolith') && credits >= 12) cmd = 'buy_regolith';
    }
    await runOnly(cmd, s.lines);
    await sleep(60);
  }
  return false;
}

/** Section 6, from scenario late_unlocks (two processors; cheats on): the instructions
 * unlock after a day, a program of the shape `if` / `build` / `buy land` / `if` is
 * written through the editor, shows live values and builds unattended; verb lines;
 * a done line costs its step and a false `if` takes one and jumps its body. Shot `program-logic.png`. */
async function programLogic(grant) {
  await debugMenu();
  await speed(0);
  await setValue('#scenario', 'late_unlocks');
  await click('#start-scenario');
  check(await exists('[data-testid="logic-locked"]'), 'before the unlock the editor says how If and Build unlock');
  await speed(1);
  await waitFor(`!!document.querySelector('[data-testid="logic-add"]')`, 'the instructions to unlock', 8000);
  await speed(0);
  await click('.prog-tabs button[data-program="0"]');
  await click('.prog-procs button[title="Clear this program"]');
  const line = (i) => `.lines li[data-line="${i}"]`;
  const add = async (text) => {
    const ok = await evaluate(`(() => { const b = [...document.querySelectorAll('[data-testid="logic-add"] button')].find((b) => b.textContent.trim() === ${q(text)}); if (!b) return false; b.click(); return true; })()`);
    if (!ok) throw new Error(`no Add button ${text}`);
    await sleep(60);
  };
  /** Picks a dropdown's option by its text (display names), or types a number. */
  const pick = async (sel, text) => {
    const ok = await evaluate(`(() => {
      const el = document.querySelector(${q(sel)});
      if (!el) return false;
      if (el.tagName === 'SELECT') {
        const o = [...el.options].find((o) => o.textContent.trim() === ${q(text)});
        if (!o) return false;
        el.value = o.value;
      } else el.value = ${q(text)};
      el.dispatchEvent(new Event('change', { bubbles: true }));
      return true;
    })()`);
    if (!ok) throw new Error(`nothing to pick at ${sel}: ${text}`);
    await sleep(60);
  };
  const clause = (i, n) => `${line(i)} .line-logic > span.clause:nth-of-type(${n})`;
  const setClause = async (i, n, reading, cmp, value) => {
    await pick(`${clause(i, n)} select[aria-label="Reading"]`, reading);
    await pick(`${clause(i, n)} select[aria-label="Comparison"]`, cmp);
    await pick(`${clause(i, n)} input`, value);
  };
  const indent = async (i) => {
    await click(`${line(i)} button[title="Move into the if block above"]`);
    await sleep(60);
  };
  await add('Build');
  await pick(`${line(0)} select[aria-label="Building"]`, 'Battery');
  await pick(`${line(0)} input[aria-label="Up to"]`, '1');
  await add('If');
  await setClause(1, 1, 'Energy', 'below', '50');
  const unit = await evaluate(`document.querySelector(${q(`${clause(1, 1)} [data-unit-toggle]`)})?.textContent ?? ''`);
  check(unit === '%', `a fill reading's number shows its unit, a toggle: "${unit}"`);
  // The unit toggle: the same entry read as an amount, then back to a percent.
  await click(`${clause(1, 1)} [data-unit-toggle]`);
  await sleep(60);
  const asAmount = await evaluate(`[document.querySelector(${q(`${clause(1, 1)} [data-unit-toggle]`)}).textContent, document.querySelector(${q(line(1))}).title.split(':')[0]]`);
  check(asAmount[0] === 'amt' && asAmount[1] === 'If Energy below 50', `the toggle reads an amount: ${asAmount.join(', ')}`);
  await click(`${clause(1, 1)} [data-unit-toggle]`);
  await sleep(60);
  const asPercent = await evaluate(`document.querySelector(${q(line(1))}).title.split(':')[0]`);
  check(asPercent === 'If Energy below 50%', `and back to a percent: ${asPercent}`);
  // Process Telemetry opens two readings in the reading list.
  const readingOptions = () => evaluate(`[...document.querySelectorAll(${q(`${clause(1, 1)} select[aria-label="Reading"] option`)})].map((o) => o.textContent.trim())`);
  const offeredBefore = await readingOptions();
  check(!offeredBefore.includes('Energy per day') && !offeredBefore.includes('Days since a purchase') && offeredBefore.length <= 13, `before Process Telemetry, ${offeredBefore.length} readings`);
  await grant('science', '50');
  await panel('research');
  await click('[data-testid="research-process_telemetry"]');
  await sleep(100);
  const offeredAfter = await readingOptions();
  check(offeredAfter.slice(-2).join(', ') === 'Energy per day, Days since a purchase', `after it, two more: ${offeredAfter.slice(-2).join(', ')}`);
  await debugMenu();
  await add('Build');
  await pick(`${line(2)} select[aria-label="Building"]`, 'Solar Panel');
  await indent(2);
  await add('If');
  await setClause(3, 1, 'Free land', 'below', '1');
  await pick(`${line(3)} select[aria-label="Join"]`, 'and');
  await setClause(3, 2, 'Credits', 'above', '500');
  check(!(await exists(`${clause(3, 2)} [data-unit]`)) && !(await exists(`${clause(3, 2)} [data-unit-toggle]`)), 'an uncapped reading shows no unit and no toggle');
  await add('Buy Land');
  await indent(4);
  await add('If');
  await setClause(5, 1, 'Credits', 'above', '800');
  await add('Build');
  await pick(`${line(6)} select[aria-label="Building"]`, 'Storage Depot');
  await pick(`${line(6)} input[aria-label="Up to"]`, '1');
  await indent(6);
  const shape = await evaluate(`[...document.querySelectorAll('.lines li')].map((l) => l.dataset.op + (l.classList.contains('indented') ? '>' : '')).join(' ')`);
  check(shape === 'build if build> if buy_land> if build>', `the program as written: ${shape}`);
  const ids = await evaluate(`document.querySelector('.lines').textContent`);
  check(!/[a-z]_[a-z]/.test(ids), 'lines show display names, never ids');
  await grant('credits', '300');
  const now = () => evaluate(`[...document.querySelectorAll('.lines li')].map((l) => [...l.querySelectorAll('[data-cond-now]')].map((c) => c.textContent.trim()).join(' | '))`);
  const before = await now();
  check(/% [✓✗]$/.test(before[1]) && before[3].split(' | ').length === 2 && /[✓✗]$/.test(before[5]), `conditions show live values: ${before[1]}; ${before[3]}; ${before[5]}`);
  await speed(10);
  await waitFor(`document.querySelector(${q(line(5))})?.classList.contains('out-false')`, 'the program to reach its false if', 8000);
  await speed(0);
  await shot('program-logic');
  await panel('buildings');
  const b1 = await buildings();
  check(b1.battery.owned === 1 && b1.storage_depot.owned === 0, `built unattended, the guarded line held back: ${b1.battery.owned} Battery, ${b1.storage_depot.owned} Storage Depot`);
  const classes = () => evaluate(`[...document.querySelectorAll('.lines li')].map((l) => (l.className.match(/out-\\w+/) ?? ['-'])[0]).join(' ')`);
  console.log(`     lines at the false if: ${await classes()}`);
  await debugMenu();
  await grant('credits', '2000');
  await speed(10);
  await panel('buildings');
  await waitFor(`document.querySelector('.card.building[data-id="storage_depot"]')?.dataset.owned === '1'`, 'the Storage Depot once the if holds', 8000);
  await speed(0);
  const after = await now();
  const built = (await classes()).split(' ');
  // The `if` takes its own step, so the day its body builds it may show no outcome.
  check(/✓$/.test(after[5]) && (built[5] === 'out-true' || built[6] === 'out-ran'),
    `the if held once Credits above 800: ${after[5]} (${built[5]} ${built[6]})`);
  check(built[6] === 'out-ran' || built[6] === 'out-done', `the guarded line built, then is done: ${built.join(' ')}`);
  // A number being typed survives the game's redraws, and is sent on Enter or leaving.
  await speed(10);
  await evaluate(`(() => { const el = document.querySelector(${q(`${clause(5, 1)} input`)}); el.focus(); el.value = '12'; el.dispatchEvent(new Event('input', { bubbles: true })); })()`);
  await sleep(500);
  const typed = await evaluate(`document.querySelector(${q(`${clause(5, 1)} input`)}).value`);
  await speed(0);
  check(typed === '12', `a number being typed survives the day's redraws: ${typed}`);
  await pick(`${clause(5, 1)} input`, '120');
  const sent = await evaluate(`document.querySelector(${q(line(5))}).title`);
  check(/^If Credits above 120:/.test(sent),`and is sent when the field is left: ${sent.split(':')[0]}`);
  // "Up to" refuses 0 (it would mean no limit) and goes back to what it was.
  await pick(`${line(0)} input[aria-label="Up to"]`, '0');
  const upTo = await evaluate(`[document.querySelector(${q(line(0))}).title.split(':')[0], document.querySelector(${q(`${line(0)} input[aria-label="Up to"]`)}).value]`);
  check(upTo[0] === 'Build Battery up to 1' && upTo[1] === '1', `an up-to of 0 is refused and reverts: ${upTo.join(', field ')}`);
  // Removing an `if` leaves its body at the top level, not in the block above.
  await click(`${line(3)} button[title="Remove"]`);
  await sleep(60);
  const left = await evaluate(`[...document.querySelectorAll('.lines li')].map((l) => l.dataset.op + (l.classList.contains('indented') ? '>' : '')).join(' ')`);
  check(left === 'build if build> buy_land if build>',`removing an if un-indents its body: ${left}`);
  await shot('program-logic-built');

  // The loadout library: a program saved in this run loads in the next and runs.
  await click('[data-testid="loadout-save"]');
  await setValue('[data-testid="loadout-name"]', 'Builder');
  await click('[data-testid="loadout-save-new"]');
  await sleep(100);
  const runBefore = await run();
  await debugMenu();
  await setValue('#cheat-boredom', '100000');
  await click('#cheat-set-boredom');
  await speed(1);
  await waitFor(`!!document.querySelector('[data-testid="summary"]')`, 'the forced retirement', 8000);
  await click('[data-testid="continue"]');
  await sleep(800);
  await speed(0);
  check((await run()) > runBefore, `a new run: ${await run()}`);
  await click('.prog-tabs button[data-program="0"]');
  check((await evaluate(`document.querySelectorAll('.lines li').length`)) === 0, 'programs start empty in the new run');
  await click('[data-testid="loadout-load"]');
  const listed = await evaluate(`document.querySelector('[data-loadout="0"] .loadout-name')?.textContent ?? ''`);
  check(listed === 'Builder', `the library kept the saved program across retirement: ${listed}`);
  await click('[data-testid="loadout-load-0"]');
  await sleep(100);
  const loaded = await evaluate(`[...document.querySelectorAll('.lines li')].map((l) => l.dataset.op + (l.classList.contains('indented') ? '>' : '')).join(' ')`);
  check(loaded === left, `the loaded program: ${loaded}`);
  await speed(1);
  await waitFor(`!!document.querySelector('.lines li[data-line="1"]')?.className.match(/out-(true|false)/)`, 'the loaded program to run', 8000);
  await speed(0);
  check(true, 'a program saved in run 1 loads in run 2 and runs');
  await shot('program-loadout');
  // Delete is one click.
  await click('[data-testid="loadout-load"]');
  await click('[data-testid="loadout-delete-0"]');
  await sleep(100);
  check((await evaluate(`document.querySelectorAll('[data-loadout]').length`)) === 0, 'a loadout deletes');
  await click('[data-testid="loadout-load"]');

  // Verb lines: one dropdown editor for every verb, each under an `if` that holds.
  await click('.prog-tabs button[data-program="0"]');
  await click('.prog-procs button[title="Clear this program"]');
  await add('If');
  const cmps = await evaluate(`[...document.querySelectorAll(${q(`${clause(0, 1)} select[aria-label="Comparison"] option`)})].map((o) => o.textContent.trim()).join(' ')`);
  check(cmps === 'above below', `two comparators: ${cmps}`);
  await setClause(0, 1, 'Credits', 'above', '-1');
  await add('Verb');
  await pick(`${line(1)} select[aria-label="Verb"]`, 'Pause');
  await pick(`${line(1)} select[aria-label="Target"]`, 'Solar Panel');
  await indent(1);
  await add('If');
  await setClause(2, 1, 'Free processors', 'above', '-1');
  await add('Verb');
  await pick(`${line(3)} select[aria-label="Verb"]`, 'Resume');
  await pick(`${line(3)} select[aria-label="Target"]`, 'Solar Panel');
  await pick(`${line(3)} input[aria-label="Amount"]`, 'all');
  await indent(3);
  const verbShape = await evaluate(`[...document.querySelectorAll('.lines li')].map((l) => l.dataset.op + (l.classList.contains('indented') ? '>' : '')).join(' ')`);
  check(verbShape === 'if act> if act>', `verb lines as written: ${verbShape}`);
  const verbText = await evaluate(`[...document.querySelectorAll('.lines li')].map((l) => l.title.split(':')[0]).join(' | ')`);
  check(/If Credits above -1/.test(verbText) && /Pause Solar Panel/.test(verbText) && /If Free processors above -1/.test(verbText) &&/Resume Solar Panel all/.test(verbText), `verb lines read as text: ${verbText}`);
  // One processor: a day runs the pause, a day the resume (each a step).
  const ran = (i) => `/out-(ran|done)/.test(document.querySelector(${q(line(i))})?.className ?? '')`;
  await speed(3);
  await waitFor(ran(1), 'the pause line to run', 8000);
  await waitFor(ran(3), 'the resume line to run', 8000);
  await speed(0);
  check(true, 'pause and resume ran from verb lines');
  await shot('program-verbs');

  // Every line costs a step: on one processor a done `build ... up to 1` takes the day's
  // step and Idle the next; guarded by a false `if`, the `if` takes the day's step and jumps
  // its body, and Idle runs the next day.
  while (Number(await evaluate(`document.querySelector('.prog-procs span').textContent.match(/\\d+/)[0]`)) > 1) {
    await click('[data-testid="proc-minus"]');
  }
  const days = async (ms) => {
    await evaluate(`(() => { window.__days = new Set(); window.__ob2?.disconnect();
      window.__ob2 = new MutationObserver(() => window.__days.add([...document.querySelectorAll('.lines li')].map((l) => (l.className.match(/out-\\w+/) ?? ['-'])[0]).join(' ')));
      window.__ob2.observe(document.querySelector('.lines'), { subtree: true, attributes: true }); })()`);
    await speed(1);
    await sleep(ms);
    await speed(0);
    await evaluate('window.__ob2.disconnect()');
    return evaluate('[...window.__days]');
  };
  await click('.prog-procs button[title="Clear this program"]');
  await add('Build');
  await pick(`${line(0)} select[aria-label="Building"]`, 'Solar Panel');
  await pick(`${line(0)} input[aria-label="Up to"]`, '1');
  await panel('commands');
  await click('[data-testid="add-idle"]');
  const done = await days(3500);
  check(done.includes('out-done -') && done.includes('- out-ran') && !done.includes('out-done out-ran'),
    `one processor: the done line takes a day's step, Idle the next (${done.join('; ')})`);
  await click('.prog-procs button[title="Clear this program"]');
  await add('If');
  // Credits never go below 0.
  await setClause(0, 1, 'Credits', 'below', '-1');
  await add('Build');
  await pick(`${line(1)} select[aria-label="Building"]`, 'Solar Panel');
  await pick(`${line(1)} input[aria-label="Up to"]`, '1');
  await indent(1);
  await click('[data-testid="add-idle"]');
  const guarded = await days(3500);
  check(guarded.includes('out-false out-jumped -') && guarded.includes('- - out-ran') && guarded.every((d) => !d.includes('out-done')),
    `guarded by a false if: the if takes a day's step and jumps its body, Idle the next (${guarded.join('; ')})`);
}

/** Section 7, from scenario everything (cheats on): the program text editor. L4's Program 1
 * (experiments/results/063_program_text_editor) pasted with a junk line (red, its reason)
 * and a locked building (amber); Apply; days run (green and grey dots, an if's fired
 * count); the block editor keeps every line, the unparsed and locked ones marked; the
 * text round-trips; another tab's paste saves on switching back; a loadout saved with
 * those lines loads after a retirement with all of them, the locked one shown locked.
 * Shot `text-editor.png`. */
async function textEditor() {
  await debugMenu();
  await speed(0);
  await setValue('#scenario', 'everything');
  await click('#start-scenario');
  // Program logic opens the first day two processors work.
  await speed(1);
  await waitFor(`!!document.querySelector('[data-testid="logic-add"]')`, 'the instructions to unlock', 8000);
  await speed(0);
  await click('.prog-tabs button[data-program="0"]');
  while (await enabled('[data-testid="proc-plus"]')) await click('[data-testid="proc-plus"]');
  const AREA = '[data-testid="text-editor-area"]';
  const area = () => evaluate(`document.querySelector(${q(AREA)}).value`);
  const paste = async (text) => {
    await setValue(AREA, text);
    await sleep(300); // the debounced parse
  };
  /** Each text line: its gutter status, fired count and note. */
  const gutter = () =>
    evaluate(`(() => {
      const dots = [...document.querySelectorAll('.te-gutter [data-line-status]')];
      const notes = [...document.querySelectorAll('.te-notes .te-row')];
      return document.querySelector(${q(AREA)}).value.split('\\n').map((text, i) => ({ text: text.trim(), status: dots[i]?.dataset.lineStatus, fired: dots[i]?.dataset.fired ?? null, note: notes[i]?.textContent ?? '' }));
    })()`);
  const lineOf = (g, text) => g.find((l) => l.text === text);
  const results = resolve(here, '../../experiments/results/063_program_text_editor');
  const p1 = readFileSync(resolve(results, 'L4_program1.txt'), 'utf8').replace(/\r/g, '').trimEnd() + '\nfrobnicate 3\nbuild fuel cell array';
  const p2 = readFileSync(resolve(results, 'L4_program2.txt'), 'utf8').replace(/\r/g, '').trimEnd();
  const entries = (text) => text.split('\n').filter((l) => l.trim() !== '').length;

  await click('[data-testid="text-editor-open"]');
  await waitFor(`!!document.querySelector('[data-testid="text-editor"]')`, 'the text editor', 3000);
  check((await evaluate(`document.querySelector('[data-testid="text-editor-tab-0"]').getAttribute('aria-selected')`)) === 'true', 'the text editor opens on the active program');
  await paste(p1);
  const g0 = await gutter();
  const junk = lineOf(g0, 'frobnicate 3');
  const locked = lineOf(g0, 'build fuel cell array');
  check(junk?.status === 'red' && /frobnicate/.test(junk.note), `a junk line is red with its reason: ${JSON.stringify(junk)}`);
  check(locked?.status === 'amber' && /locked/.test(locked.note), `a locked building is amber with its reason: ${JSON.stringify(locked)}`);
  check(g0.filter((l) => l.status === 'red' || l.status === 'amber').length === 2 && lineOf(g0, "# 060's L4, Program 1: build, and seat processors as Data Centers add them")?.status === 'note',
    `only those two flagged, the # line a note: ${g0.map((l) => l.status).join(' ')}`);
  check(/1 line unparsed, 1 locked/.test(await evaluate(`document.querySelector('[data-testid="text-editor-status"]').textContent`)), 'the status line counts them');
  check(await enabled('[data-testid="text-editor-apply"]'), 'Apply is on for a changed draft');
  await click('[data-testid="text-editor-apply"]');
  await sleep(200);
  check(!(await enabled('[data-testid="text-editor-apply"]')), 'and off once applied');
  const applied = await area();
  // Days run: the gutter lights by each line's outcome, and an if counts its days true.
  await speed(10);
  await waitFor(`(() => { const s = [...document.querySelectorAll('.te-gutter [data-line-status]')].map((d) => d.dataset.lineStatus); return s.includes('green') && s.includes('grey'); })()`, 'green and grey lines', 10000);
  await sleep(1500);
  await speed(0);
  const g1 = await gutter();
  const fired = g1.filter((l) => l.fired);
  check(fired.length > 0 && fired.every((l) => /^if /.test(l.text) && /^\d+\/\d+$/.test(l.fired) && l.note.includes(`fired ${l.fired}`)),
    `if lines show their fired count: ${fired.map((l) => `${l.text.slice(0, 24)}: ${l.fired}`).join('; ')}`);
  check(lineOf(g1, 'frobnicate 3')?.status === 'red' && lineOf(g1, 'build fuel cell array')?.status === 'amber', 'red and amber stay flagged after the days');
  check((await area()) === applied, 'the draft and caret text survive the redraws');
  await shot('text-editor');
  await click('[data-takeover-close="text_editor"]');
  await sleep(100);
  const block = () =>
    evaluate(`[...document.querySelectorAll('.lines li')].map((l) => ({ op: l.dataset.op ?? 'command', locked: l.classList.contains('is-locked'), why: l.querySelector('.line-why')?.textContent ?? '' }))`);
  const b1 = await block();
  check(b1.length === entries(p1), `the block editor keeps every line: ${b1.length} of ${entries(p1)}`);
  const unparsed = b1.filter((l) => l.op === 'unparsed');
  check(unparsed.length === 1 && /frobnicate/.test(unparsed[0].why), `the junk line is an unparsed row with its reason: ${JSON.stringify(unparsed)}`);
  check(b1.filter((l) => l.locked).length === 1 && b1.some((l) => l.locked && /locked this run/.test(l.why)), `the locked line is dimmed, saying why: ${JSON.stringify(b1.filter((l) => l.locked))}`);
  check(b1.filter((l) => l.op === 'note').length === 1, 'the # line is a note row');
  await click('[data-testid="text-editor-open"]');
  await sleep(200);
  // The engine's text ends with a newline; the pasted one didn't.
  check((await area()).trimEnd() === applied.trimEnd(), 'reopened, the text round-trips');
  // Program 2: pasted, then saved by switching back to Program 1.
  await click('[data-testid="text-editor-tab-1"]');
  await sleep(100);
  await paste(p2);
  await click('[data-testid="text-editor-tab-0"]');
  await sleep(200);
  check((await area()).trimEnd() === applied.trimEnd(), 'switching tabs shows each program its own text');
  await click('[data-testid="text-editor-tab-1"]');
  await sleep(200);
  check((await area()).trimEnd() === p2 && !(await enabled('[data-testid="text-editor-apply"]')), 'the other tab saved on switching away');
  // Esc closes, back on the block editor of the tab open in the editor.
  await evaluate(`document.querySelector(${q(AREA)}).dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true }))`);
  await sleep(100);
  check(!(await exists('[data-testid="text-editor"]')) && (await block()).length === entries(p2), `Esc closes onto Program 2's ${entries(p2)} lines`);
  // Too many lines: the ones past the limit are red, Apply refuses, and a leave that
  // couldn't save keeps the draft for the next open.
  const tooMany = Array.from({ length: 130 }, () => 'idle 1').join('\n');
  await click('[data-testid="text-editor-open"]');
  await sleep(200);
  await paste(tooMany);
  const g2 = await gutter();
  const over = g2.filter((l) => l.status === 'red');
  check(over.length === 2 && over.every((l) => /at most 128 lines/.test(l.note)) && /2 lines unparsed/.test(await evaluate(`document.querySelector('[data-testid="text-editor-status"]').textContent`)),
    `130 lines: the 2 past the limit are red, "${over[0]?.note}"`);
  await click('[data-testid="text-editor-apply"]');
  await sleep(200);
  const refused = await evaluate(`document.querySelector('[data-testid="text-editor-status"]').textContent`);
  check(/^Not applied/.test(refused), `Apply refuses them: ${refused}`);
  await click('[data-takeover-close="text_editor"]');
  await sleep(200);
  check((await block()).length === entries(p2) && /not saved/.test(await noticeText()), `closing can't save them either, and says so: ${await noticeText()}`);
  await click('[data-testid="text-editor-open"]');
  await sleep(200);
  check((await area()) === tooMany, 'reopened, the unsaved 130-line draft is still there');
  await paste(p2);
  await click('[data-testid="text-editor-apply"]');
  await sleep(200);
  check(!(await enabled('[data-testid="text-editor-apply"]')) && (await area()).trimEnd() === p2, 'a valid program applies again');
  await click('[data-takeover-close="text_editor"]');
  await sleep(100);

  // A loadout of those lines, loaded at a fresh run: every line kept, the locked one dimmed.
  await click('.prog-tabs button[data-program="0"]');
  await click('[data-testid="loadout-save"]');
  await setValue('[data-testid="loadout-name"]', 'Text L4');
  await click('[data-testid="loadout-save-new"]');
  await sleep(100);
  const runBefore = await run();
  await debugMenu();
  await setValue('#cheat-boredom', '100000');
  await click('#cheat-set-boredom');
  await speed(1);
  await waitFor(`!!document.querySelector('[data-testid="summary"]')`, 'the forced retirement', 8000);
  await click('[data-testid="continue"]');
  await sleep(800);
  await speed(0);
  check((await run()) > runBefore, `a new run: ${await run()}`);
  await click('.prog-tabs button[data-program="0"]');
  await click('[data-testid="loadout-load"]');
  const slot = await evaluate(`[...document.querySelectorAll('[data-loadout]')].find((l) => l.querySelector('.loadout-name')?.textContent === 'Text L4')?.dataset.loadout ?? null`);
  await click(`[data-testid="loadout-load-${slot}"]`);
  await sleep(100);
  const b2 = await block();
  check(b2.length === entries(p1) && b2.some((l) => l.op === 'unparsed') && b2.some((l) => l.locked && /locked this run/.test(l.why)),
    `the loadout loads every line in the new run, unparsed and locked ones kept: ${b2.length} lines, ${b2.filter((l) => l.locked).length} locked`);
  await click('[data-testid="loadout-load"]');
  await click(`[data-testid="loadout-delete-${slot}"]`);
  await sleep(100);
  await click('[data-testid="loadout-load"]');
}

/** Section 5, from scenario at_cap (paused, cheats on; `grant` from section 2). Shots
 * `read-*.png`. Leaves a notice up (the replay check's) for section 4. */
async function readability(grant) {
  // Income at cap: Energy at 100 / 100 under 20 Solar Panels still shows its surplus.
  const capped = await evaluate(`document.querySelector('tr[data-res="energy"] [data-capped]')?.textContent ?? ''`);
  check(/^\+[\d.]+ cap$/.test(capped), `a capped resource shows its positive rate: "${capped}"`);
  const bar = await evaluate(`document.querySelector('[data-testid="energy-rate"]').textContent`);
  check(/^\+[\d.]+\/day \(capped\)$/.test(bar), `the status bar's Energy rate too: "${bar}"`);
  // Capped Energy's surplus is spare power: green (the text token), not muted.
  const spare = await evaluate(`(() => {
    const el = document.querySelector('tr[data-res="energy"] [data-capped]');
    const root = getComputedStyle(document.documentElement);
    const probe = (v) => { const s = document.createElement('span'); s.style.color = v; document.body.append(s); const c = getComputedStyle(s).color; s.remove(); return c; };
    return { color: getComputedStyle(el).color, green: probe(root.getPropertyValue('--pos-text')), muted: probe(root.getPropertyValue('--muted')) };
  })()`);
  const barSpare = await evaluate(`getComputedStyle(document.querySelector('[data-testid="energy-rate"] [data-capped]')).color`);
  check(spare.color === spare.green && spare.color !== spare.muted && barSpare === spare.green, `capped Energy's rate is green, in the status bar too: ${JSON.stringify({ ...spare, barSpare })}`);
  // Resources in two groups: Energy, Credits, Land, Boredom, Processors (those shown), then the rest.
  const groups = await evaluate(`[...document.querySelectorAll('[data-testid="resources"] tbody')].map((b) => [...b.querySelectorAll('tr')].map((r) => r.dataset.res))`);
  const first = ['energy', 'credits', 'land', 'boredom', 'processors'];
  check(groups.length === 2 && groups[0].join() === first.filter((id) => groups.flat().includes(id)).join() && groups[0][0] === 'energy' && groups[1].every((id) => !first.includes(id)) && groups[1].length > 0,
    `Resources in two groups, the first in its fixed order: ${JSON.stringify(groups)}`);
  // Affordable vs not: only the affordable card's name row is tinted.
  await debugMenu();
  // (20 Solar Panels hold all the land: free some too.)
  await grant('credits', '5000');
  await grant('land', '10');
  await grant('titanium', '20');
  await grant('regolith', '50');
  await panel('buildings');
  const heads = await evaluate(`(() => {
    const bg = (sel) => { const h = document.querySelector(sel + ' .card-head'); return h ? getComputedStyle(h).backgroundColor : null; };
    const card = (sel) => { const c = document.querySelector(sel); return c ? getComputedStyle(c).backgroundColor : null; };
    return { yes: bg('.card.building.can-buy'), no: bg('.card.building.cannot-buy'), body: card('.card.building.can-buy') };
  })()`);
  check(!!heads.yes && !!heads.no && heads.yes !== heads.no && heads.yes !== heads.body, `an affordable card's header is tinted, the card body isn't: ${JSON.stringify(heads)}`);
  // The Land row, affordable: the cards' green outline and tinted head, its box unchanged.
  const land = await evaluate(`(() => {
    const b = document.querySelector('[data-testid="land-bar"]'), r = b.getBoundingClientRect(), s = getComputedStyle(b);
    return { can: !document.querySelector('[data-testid="buy-land"]').disabled, cls: b.classList.contains('can-buy'), border: s.borderTopColor, bg: s.backgroundColor,
      head: getComputedStyle(document.querySelector('.card.building.can-buy .card-head')).backgroundColor,
      cardBorder: getComputedStyle(document.querySelector('.card.building.can-buy')).borderTopColor, h: r.height };
  })()`);
  check(land.can && land.cls && land.border === land.cardBorder && land.bg === land.head, `the Land row shows green when land is affordable: ${JSON.stringify(land)}`);
  await shot('read-capped-and-cards');
  await shot('parked-land-resources-capped');

  // A pad short on fuel, then Ready; a boredom beat fires on the way (day 200).
  await debugMenu();
  await setValue('#scenario', 'pad_loading');
  await click('#start-scenario');
  const quest = await evaluate(`document.querySelector('[data-testid="quest-pin"] [data-testid="quest"]')?.dataset.quest ?? null`);
  check(!!quest, `the main quest has its pinned slot: ${quest}`);
  // Fold Story: a folded section keeps its count.
  await click('[data-testid="events-story"] .fold-head');
  const lines = await evaluate(`[...document.querySelectorAll('.lines li')].map((li) => li.dataset.command)`);
  await runOnly('load_pads', lines);
  // Helium-3 caps at 20 and a load moves 5: top it up and play 4 days, 5 times.
  await debugMenu();
  for (let i = 0; i < 5; i++) {
    await grant('he3', '20');
    await setValue('#cheat-days', '4');
    await click('#cheat-jump');
    await sleep(100);
  }
  await panel('launch_pads');
  await waitFor(`Number(document.querySelector('[data-pad="0"]')?.dataset.cargo ?? 0) >= 100`, 'the pad full', 3000);
  await sleep(200);
  const short = await evaluate(`(() => {
    const p = document.querySelector('[data-pad="0"]');
    return { fuel: p.querySelector('[data-testid="fuel-0"]').textContent, fueled: p.querySelector('[data-testid="fuel-0"]').dataset.fueled,
      status: p.querySelector('.pad-status').textContent, ready: p.classList.contains('ready'),
      go: p.querySelector('[data-testid="launch-0"]').classList.contains('go'), dot: !!document.querySelector('[data-testid="pads-ready"]') };
  })()`);
  // The chip's <title> ("Propellant") leads the line's text.
  check(short.fuel.endsWith('Fuel 19.4 / 20 Propellant') &&short.fueled === 'false' && short.status !== 'Ready' && !short.ready && !short.go && !short.dot,
    `a full pad short on fuel: red fuel line, not Ready: ${JSON.stringify(short)}`);
  // The pads are the panel's last section: scroll only the middle column to them.
  const toPads = `document.querySelector('.col-mid').scrollTop = 1e6`;
  await evaluate(toPads);
  // The boredom beat: in Story, unread; the quest still pinned and no banner over it.
  const beat = await evaluate(`(() => ({
    pin: document.querySelector('[data-testid="quest-pin"] [data-testid="quest"]')?.dataset.quest ?? null,
    banner: document.querySelector('[data-testid="story-banner"]')?.dataset.story ?? null,
    unread: Number(document.querySelector('[data-testid="events-story"]').dataset.unread),
    head: document.querySelector('[data-testid="events-story"] .fold-head').textContent,
    // Story is folded: the beat's line in Recent Activity says it fired.
    beat: [...document.querySelectorAll('[data-testid="events-activity"] li[data-kind="event"]')].some((e) => /Boredom Rising/.test(e.textContent)),
    slot: [...document.querySelectorAll('[data-testid="quest-pin"] [data-quest]')].map((e) => e.dataset.quest),
  }))()`);
  // Q1 completes on the way (Q2 opens): the slot holds the active quest and nothing else;
  // a banner may still hold the scenario's quest texts (quests only, never the beat).
  check(/^q/.test(beat.pin ?? '') && beat.slot.length === 1 && beat.beat && !(beat.banner ?? '').startsWith('boredom') && beat.unread >= 1,
    `a boredom beat fired: the quest stays pinned, Story counts it unread: ${JSON.stringify(beat)}`);
  await shot('read-pad-short-story-unread');
  await debugMenu();
  await grant('propellant', '0.6');
  await panel('launch_pads');
  await evaluate(toPads);
  await sleep(200);
  const ready = await evaluate(`(() => {
    const p = document.querySelector('[data-pad="0"]');
    return { fuel: p.querySelector('[data-testid="fuel-0"]').textContent, fueled: p.querySelector('[data-testid="fuel-0"]').dataset.fueled,
      status: p.querySelector('.pad-status').textContent, ready: p.classList.contains('ready'),
      go: p.querySelector('[data-testid="launch-0"]').classList.contains('go'), dot: !!document.querySelector('[data-testid="pads-ready"]') };
  })()`);
  check(ready.fuel.endsWith('Fuel 20 / 20 Propellant') &&ready.fueled === 'true' && ready.status === 'Ready' && ready.ready && ready.go && ready.dot,
    `fuel enough: Ready, green Launch, green border, nav dot: ${JSON.stringify(ready)}`);
  // Unfold Story: its count clears after about a second on screen, with no click on an item.
  await click('[data-testid="events-story"] .fold-head');
  await sleep(300);
  const early = Number(await evaluate(`document.querySelector('[data-testid="events-story"]').dataset.unread`));
  await shot('read-pad-ready-story-open');
  await waitFor(`document.querySelector('[data-testid="events-story"]').dataset.unread === '0'`, 'Story read', 4000);
  check(early >= 1, `Story's count holds for a moment once open (${early}), then clears`);
  check(
    (await evaluate(`document.querySelector('[data-testid="quest-pin"] [data-testid="quest"]')?.dataset.quest ?? null`)) === beat.pin,
    'the quest is still pinned after Story is read',
  );
  await shot('read-story-read');
  // Past day 500 boredom phase 3 supersedes phase 2 in Story; Recent Activity keeps both.
  await debugMenu();
  await setValue('#cheat-days', '320');
  await click('#cheat-jump');
  await sleep(300);
  await panel('launch_pads');
  const phases = await evaluate(`(() => ({
    story: [...document.querySelectorAll('[data-testid="events-story"] [data-event]')].map((e) => e.dataset.event),
    activity: [...document.querySelectorAll('[data-testid="events-activity"] li[data-kind="event"]')].map((e) => e.textContent),
  }))()`);
  const boredom = phases.story.filter((id) => id.startsWith('boredom_phase_'));
  check(boredom.length === 1 && boredom[0] === 'boredom_phase_3' && phases.activity.some((t) => /Boredom Rising/.test(t)) && phases.activity.some((t) => /Boredom Accelerating/.test(t)),
    `Story lists only the latest boredom phase, Recent Activity both: ${JSON.stringify(phases)}`);
  await shot('parked-story-latest-phase');
  // Read it now, so no count clears under section 4's layout comparison.
  await waitFor(`document.querySelector('[data-testid="events-story"]').dataset.unread === '0'`, 'phase 3 read', 4000);
  await debugMenu();
  await click('#check-replay');
  check(/^Replay ok/.test(await noticeText()), `replay check after the readability steps: ${await noticeText()}`);
}

try {
  await load(url);
  check((await day()) < 5 && (await run()) === 1, `a fresh game loads: day ${await day()}, run ${await run()}`);
  check(!(await exists('[data-testid="cheats"]')), 'cheats are off by default');
  check(!(await navVisible('retirement')) && !(await navVisible('launch_pads')), 'gated nav tiles start hidden');
  check(/1\.1 — Online/.test(await evaluate(`document.querySelector('[data-testid="quest"]')?.textContent ?? ''`)), 'the first quest shows');

  check((await evaluate(`document.querySelector('[data-testid="story-banner"]')?.dataset.story`)) === 'qpower_boot_sequence', 'a new game tells its first quest');
  await click('[data-testid="story-banner"] button');
  // Two quick clicks (no render between them) both land.
  await panel('commands');
  await evaluate(`['add-cloud_compute', 'add-idle'].forEach((t) => document.querySelector('[data-testid="' + t + '"]').click())`);
  await sleep(100);
  const two = await evaluate(`[...document.querySelectorAll('.lines li')].map((li) => li.dataset.command).join(',')`);
  check(two === 'cloud_compute,idle', `two quick adds both land: ${two}`);

  // ---- 1. a fresh game to the first shipment, a forced retirement and run 2, through the normal UI ----
  const shipped = await playToFirstShipment();
  check(shipped, `played to the first shipment by day ${await day()}`);
  await click('[data-testid="events-completed"] .fold-head'); // it starts folded
  const done = await evaluate(`[...document.querySelectorAll('[data-testid="events-completed"] [data-quest]')].map((li) => li.dataset.quest)`);
  check(
    ['qpower_boot_sequence', 'qshipment_proof_of_concept'].every((x) => done.includes(x)),
    `1.1 Online and 1.2 First Shipment complete: ${done.join(', ')}`,
  );
  check(await evaluate(`!!document.querySelector('.activity li[data-kind="shipment"]')`), 'the shipment is in Recent Activity');
  check(await exists('tr[data-res="propellant"]'), 'Propellant shows in Resources once bought');
  const told = await evaluate(`document.querySelector('[data-testid="story-banner"]')?.textContent ?? ''`);
  check(/First Shipment|Processors|Speculators/.test(told), `later quests are told as they open: ${told.slice(0, 40)}`);
  check(await navVisible('launch_pads'), 'the Launch Pads tile shows');
  // Voluntary retirement opens only at 1.5 Burnout (after a first retirement): run 1
  // ends at the boredom cap, played on at 200x with no cheat.
  check(!(await navVisible('retirement')), 'no Retirement tile in run 1 (it opens at 1.5 Burnout)');
  await panel('buildings');
  await shot('run1-buildings');
  await panel('launch_pads');
  await shot('run1-launch-pads');
  await speed(200);
  await waitFor(`!!document.querySelector('[data-testid="summary"]')`, 'the forced retirement at the boredom cap', 60_000);
  const summaryText = await evaluate(`document.querySelector('[data-testid="summary"]').textContent`);
  const runDays = (summaryText.match(/ended on day ([\d,.]+)/) ?? [, '?'])[1];
  check(/forced retirement/.test(summaryText), `boredom at its cap retires run 1 (day ${runDays})`);
  const paused1 = await day();
  await sleep(600);
  check((await day()) === paused1 && (await run()) === 2, `the run summary pauses the game (run 2, day ${paused1})`);
  await shot('run-summary');
  // A real double-click on Continue over the Buildings panel: its second click lands on
  // whatever the summary covered, and must not reach a card.
  await click('.nav-tile[data-nav="buildings"]');
  {
    const owned = `[...document.querySelectorAll('.card.building')].reduce((n, c) => n + Number(c.dataset.owned), 0)`;
    const o0 = await evaluate(owned);
    const p = await evaluate(`(() => { const r = document.querySelector('[data-testid="continue"]').getBoundingClientRect(); return { x: r.left + r.width / 2, y: r.top + r.height / 2 }; })()`);
    for (const clickCount of [1, 2])
      for (const type of ['mousePressed', 'mouseReleased']) await send('Input.dispatchMouseEvent', { type, x: p.x, y: p.y, button: 'left', clickCount });
    await sleep(200);
    const hint = await evaluate(`[...document.querySelectorAll('.buy-hint.short')].map((h) => h.textContent).join(' ')`);
    check((await evaluate(owned)) === o0 && !hint, `a double-click on Continue buys nothing under it (${o0} -> ${await evaluate(owned)}${hint ? `, "${hint}"` : ''})`);
  }
  await sleep(800);
  check((await run()) === 2 && (await day()) > 0 && !(await exists('[data-testid="summary"]')), `run 2 plays on (day ${await day()}; run 1 lasted ${runDays} days)`);
  check(await exists('#speeds button[data-speed="200"][aria-pressed="true"]'), 'Continue restores the speed before the summary');
  await speed(0);
  check(await navVisible('launch_pads'), 'known nav tiles stay visible in run 2');
  check(await evaluate(`!!document.querySelector('tr[data-res="he3"]')`), 'known resources stay visible in run 2');
  check(!(await evaluate(`document.getElementById('status')?.textContent ?? document.querySelector('.status').textContent`)).includes('CHEATED'), 'an honest game is not marked');

  // The play log of that career, on disk through the dev server.
  await panel('options');
  const career = await evaluate(`document.querySelector('[data-testid="playlog-status"]').dataset.career`);
  const logDir = await evaluate(`fetch('/__playlog?career=${career}').then((r) => r.json()).then((j) => j.dir)`);
  check(career.startsWith('bot_'), `an automated browser's career is filed as a bot's: ${career}`);
  await sleep(2500); // the page sends its log every 2 s
  const logFile = resolve(logDir, `${career}.jsonl`);
  const logLines = existsSync(logFile) ? readFileSync(logFile, 'utf8').trim().split('\n').map((l) => JSON.parse(l)) : [];
  const kinds = new Set(logLines.map((l) => l.kind));
  check(
    logLines[0]?.kind === 'header' && logLines[0].commit && ['action', 'speed', 'panel', 'retirement', 'checkpoint', 'session_start'].every((k) => kinds.has(k)),
    `the play log is on disk: ${logFile} (${logLines.length} lines, ${logLines.filter((l) => l.kind === 'action').length} actions)`,
  );

  // ---- 2. every panel, reaching the gated ones from scenario late_unlocks ----
  // Research and Projects open at 1.6 Lab, after 1.5 Burnout (a second run): the scenario
  // opens them (and Ideologies and Retirement) instead of playing that far.
  await speed(0);
  await debugMenu();
  await click('#cheats-toggle');
  check(await exists('[data-testid="cheats"]'), 'the cheats section opens');
  await setValue('#scenario', 'late_unlocks');
  await click('#start-scenario');
  await grant('credits', '100000');
  await grant('circuits', '100');
  await grant('science', '5000');
  check(await navVisible('research') && await navVisible('projects') && await navVisible('ideologies'), 'late_unlocks opens Research, Projects and Ideologies');
  await panel('research');
  await click('[data-testid="research-market_awareness"]');
  // Quests and events check at the end of a day: let days pass.
  await speed(1);
  await panel('projects');
  await waitFor(`!!document.querySelector('.card.project[data-id="market_cornering"]:not(.locked)')`, 'Market Cornering unlocked by Market Awareness', 5000);
  await speed(0);
  // A locked project shows its condition; a funded one its days to done (both the engine's text).
  const lockedText = await evaluate(`document.querySelector('.card.project.locked[data-id="predictive_maintenance"]')?.textContent ?? ''`);
  check(lockedText.includes('Locked: Research Daydreaming'), `a locked project shows its condition: ${lockedText.slice(0, 160)}`);
  await evaluate(`[...document.querySelectorAll('.card.project[data-id="market_cornering"] button[title^="Fund at the most"]')].forEach((b) => b.click())`);
  await waitFor(`/Done in \\d+ days/.test(document.querySelector('.card.project[data-id="market_cornering"]')?.textContent ?? '')`, 'days to done', 5000);
  const fundedText = await evaluate(`document.querySelector('.card.project[data-id="market_cornering"]').textContent`);
  check(/Done in 17 days/.test(fundedText), `a funded project shows its days to done: ${(fundedText.match(/Done in \d+ days?/) ?? ['none'])[0]}`);
  await shot('projects-locked-and-funded');
  const tiles = await evaluate(`[...document.querySelectorAll('.nav-tile')].map((t) => t.dataset.nav)`);
  for (const id of tiles) {
    await panel(id);
    const text = await evaluate(`document.querySelector('[data-panel="${id}"]').textContent.length`);
    check(text > 20, `panel ${id} opens (${text} chars)`);
    await shot(`panel-${id}`);
  }
  // Act 1's tiles: every nav row but those whose panel only a DRAFT quest (Acts 2 and 3) opens.
  const content = (f) => JSON.parse(readFileSync(resolve(here, `../../content/${f}`), 'utf8'));
  const draftPanels = new Set(
    content('events.json')
      .filter((e) => (e.summary ?? '').startsWith('DRAFT'))
      .flatMap((e) => (e.unlocks ?? []).filter((u) => u.type === 'enable_nav_panel').map((u) => u.panel)),
  );
  const act1 = content('nav.json').map((n) => n.id).filter((id) => !draftPanels.has(id));
  const missing = act1.filter((id) => !tiles.includes(id));
  check(!missing.length, `all ${act1.length} Act 1 nav tiles (content/nav.json) reachable: ${tiles.join(', ')}${missing.length ? `; missing ${missing.join(', ')}` : ''}`);
  check((await evaluate(`document.querySelector('.status').textContent`)).includes('CHEATED'), 'a cheated game is marked');
  await debugMenu();
  await setValue('#cheat-days', '30');
  const d0 = await day();
  await click('#cheat-jump');
  check((await day()) === d0 + 30, `jump ahead 30 days (${d0} -> ${await day()})`);
  // The cheat building (a Buildings card only while Cheats are on) and Complete all research.
  await panel('buildings');
  const cheatOwned = () => evaluate(`Number(document.querySelector('[data-testid="buy-cheat_building"]')?.dataset.owned ?? -1)`);
  const credits = () => evaluate(`Number(document.querySelector('tr.res-row[data-res="credits"]').dataset.amount)`);
  const c0 = await credits();
  await click('[data-testid="buy-cheat_building"]');
  check((await cheatOwned()) === 1 && c0 - (await credits()) === 1, `the cheat building buys for 1 credit (${c0} -> ${await credits()})`);
  await debugMenu();
  await click('#cheat-complete-research');
  await panel('research');
  const undone = await evaluate(`document.querySelectorAll('.card.research:not(.done)').length`);
  check(undone === 0 && (await exists('.card.research.done')), `Complete all research leaves no item undone (${undone})`);
  await debugMenu();
  await click('#cheats-toggle');
  await panel('buildings');
  check(!(await exists('[data-testid="buy-cheat_building"]')), 'no cheat card with Cheats off');
  await debugMenu();
  await click('#cheats-toggle');
  await click('#check-replay');
  check(/^Replay ok/.test(await noticeText()), `replay check of the scenario's game: ${await noticeText()}`);

  // ---- 3. speeds, saves and scenarios ----
  await setValue('#scenario', 'rich');
  await click('#start-scenario');
  check((await day()) === 0 && /Started scenario rich/.test(await noticeText()), 'debug menu starts scenario rich');
  await programOrder();
  for (const [s, secs] of [[1, 3], [3, 2], [10, 2], [50, 1], [200, 1]]) {
    await speed(s);
    const a = await day();
    const t0 = Date.now();
    await sleep(secs * 1000);
    const rate = ((await day()) - a) / ((Date.now() - t0) / 1000);
    check(rate > s * 0.6 && rate < s * 1.4, `${s}x: ${rate.toFixed(1)} days/s`);
  }
  await speed(0);
  const paused = await day();
  await sleep(1200);
  check((await day()) === paused, 'pause holds the day');
  await panel('buildings');
  const owned = () => evaluate(`document.querySelector('.card.building[data-id="solar_panel"]').dataset.owned`);
  const before = await owned();
  // The card is the Buy button: a click anywhere on its text buys one.
  await click('.card.building[data-id="solar_panel"].can-buy .desc');
  const after = await owned();
  check(Number(after) === Number(before) + 1, `a click on a green card buys a Solar Panel: ${before} -> ${after}`);
  // Its own controls keep their jobs and buy nothing.
  await click('.card.building[data-id="solar_panel"] button[title="Run one more"]');
  await click('.card.building[data-id="solar_panel"] button[title="Run one fewer"]');
  check((await owned()) === after, `the card's −/+ buttons buy nothing (${await owned()})`);
  await click('.card.building[data-id="solar_panel"] button.sell[title^="Sell one"]');
  check(Number(await owned()) === Number(after) - 1, `the card's Sell 1 sells one and buys none (${after} -> ${await owned()})`);
  // A real mouse press on the card, then space: the pause key, not a second buy (a press
  // must not leave the card focused).
  {
    const p = await evaluate(`(() => { const e = document.querySelector('.card.building[data-id="solar_panel"] .desc'); e.scrollIntoView({ block: 'center' }); const r = e.getBoundingClientRect(); return { x: r.left + r.width / 2, y: r.top + r.height / 2 }; })()`);
    const n0 = Number(await owned());
    const buyable = await exists('.card.building[data-id="solar_panel"].can-buy');
    for (const type of ['mousePressed', 'mouseReleased']) await send('Input.dispatchMouseEvent', { type, x: p.x, y: p.y, button: 'left', clickCount: 1 });
    await sleep(150);
    const n1 = Number(await owned());
    for (const type of ['keyDown', 'keyUp']) await send('Input.dispatchKeyEvent', { type, key: ' ', code: 'Space', text: type === 'keyDown' ? ' ' : undefined, windowsVirtualKeyCode: 32 });
    await sleep(150);
    const sp = await evaluate(`document.querySelector('#speeds button[aria-pressed="true"]')?.dataset.speed`);
    check((!buyable || n1 === n0 + 1) && Number(await owned()) === n1 && sp !== '0', `space after a click on a card resumes the clock and buys nothing (${n0} -> ${n1} -> ${await owned()}, speed ${sp})`);
    await speed(0);
    if (Number(await owned()) > n0) await click('.card.building[data-id="solar_panel"] button.sell[title^="Sell one"]');
  }
  const kept = await owned();
  const grey =await evaluate(`document.querySelector('.card.building.cannot-buy')?.dataset.id ?? null`);
  if (grey) {
    const greyOwned = () => evaluate(`document.querySelector('.card.building[data-id="${grey}"]').dataset.owned`);
    const g0 = await greyOwned();
    await click(`[data-testid="buy-${grey}"]`);
    const hint = await evaluate(`document.querySelector('.card.building[data-id="${grey}"] .buy-hint').textContent`);
    check((await greyOwned()) === g0 && hint.length > 0, `a grey card buys nothing and says why (${grey}: "${hint}")`);
  }
  const savedDay = await day();
  await load(url);
  await panel('buildings');
  check((await day()) === savedDay && (await owned()) === kept, `reload keeps the save (day ${savedDay}, ${kept} panels)`);
  // As if the next load ran a newer engine: retag the save after each of the page's own
  // leaving saves (pagehide, then visibilitychange; listeners run in order, so ours last).
  await evaluate(`(() => {
    const retag = () => { const s = JSON.parse(localStorage.getItem('hh.save')); s.revision = 999; localStorage.setItem('hh.save', JSON.stringify(s)); };
    window.addEventListener('pagehide', retag);
    document.addEventListener('visibilitychange', retag);
  })()`);
  await load(url);
  const n = await noticeText();
  check(/rules revision 999/.test(n) && (await day()) < savedDay, `revision mismatch discards the save: ${n}`);
  await debugMenu();
  // Paused: a scenario keeps the speed in play, and on a loaded box a day can pass
  // between the start and the read.
  await speed(0);
  await setValue('#scenario', 'at_cap');
  await click('#start-scenario');
  await panel('buildings');
  const [capDay, capOwned] = [await day(), await owned()];
  check(capDay === 100 && capOwned === '20', `debug menu starts scenario at_cap (day ${capDay}, ${capOwned} Solar Panels)`);
  await debugMenu();
  await click('#check-replay');
  check(/^Replay ok/.test(await noticeText()), `replay check: ${await noticeText()}`);

  // ---- 5. readability: capped income, affordable headers, pad readiness, quest pin ----
  // ---- 6. program instructions (before 5): if, build, buy land ----
  await programLogic(grant);
  // ---- 7. the program text editor (before 5) ----
  await textEditor();
  await debugMenu();
  await setValue('#scenario', 'at_cap');
  await click('#start-scenario');
  await readability(grant);

  // ---- 4. the theme: light by default; the Options toggle switches it, moving nothing ----
  const theme = () => evaluate('document.documentElement.dataset.theme');
  // Every element's box, to the hundredth of a pixel.
  const layout = () =>
    evaluate(`JSON.stringify([...document.querySelectorAll('body *')].map((e) => {
      const r = e.getBoundingClientRect();
      return [r.x, r.y, r.width, r.height].map((n) => Math.round(n * 100)).join();
    }))`);
  const pick = async (t, where = '#theme') => {
    const before = await layout();
    await click(`${where} [data-theme-pick="${t}"]`);
    await sleep(200);
    check((await theme()) === t && (await layout()) === before, `${where} switches to ${t} with nothing moving`);
  };
  check((await theme()) === 'light' && (await evaluate(`localStorage.getItem('hh.theme')`)) === null, 'the game opens in the light theme');
  await speed(0);
  await click('#notice button');
  await panel('commands');
  await evaluate(`[...document.querySelectorAll('[data-testid^="add-"]')].filter((b) => !b.disabled).slice(0, 3).forEach((b) => b.click())`);
  await panel('buildings');
  const looks = {};
  for (const t of ['light', 'dark']) {
    if (t === 'dark') {
      await panel('options');
      await pick('dark');
    }
    for (const p of ['buildings', 'commands']) {
      await panel(p);
      // From the top: a scroll left by an earlier step is not the theme's doing.
      await evaluate(`document.querySelector('.col-mid').scrollTop = 0`);
      await sleep(200);
      looks[`${t} ${p}`] = await layout();
      await shot(`theme-${t}-${p}`);
    }
  }
  // On a mismatch, name the first element that differs (or the count that does).
  const firstDiff = async (a, b) => {
    const [x, y] = [JSON.parse(a), JSON.parse(b)];
    const i = x.findIndex((v, k) => v !== y[k]);
    const what = await evaluate(`(() => { const e = [...document.querySelectorAll('body *')][${Math.max(0, i)}]; return e ? e.tagName + '.' + e.className + ' "' + (e.textContent || '').trim().slice(0, 40) + '"' : '?'; })()`);
    return ` (${x.length} vs ${y.length} elements; first difference #${i}: ${x[i]} vs ${y[i]}, now ${what})`;
  };
  for (const p of ['buildings', 'commands']) {
    const same = looks[`light ${p}`] === looks[`dark ${p}`];
    check(same, `${p}: the same layout in both themes${same ? '' : await firstDiff(looks[`light ${p}`], looks[`dark ${p}`])}`);
  }
  await load(url);
  check((await theme()) === 'dark', 'a reload keeps the dark theme');
  await speed(0);
  await panel('options');
  await debugMenu();
  await pick('light', '#debug-theme');
  await pick('dark');
  await pick('light');
  check(pageErrors.length === 0, `no page errors${pageErrors.length ? `: ${pageErrors.join(' | ')}` : ''}`);
} catch (e) {
  failures.push(String(e));
  console.error(e);
  if (pageErrors.length) console.error(`page errors: ${pageErrors.join(' | ')}`);
  await shot('failure');
} finally {
  b.close();
}
if (failures.length) {
  console.error(`e2e FAILED: ${failures.length}`);
  process.exit(1);
}
console.log('e2e ok');
