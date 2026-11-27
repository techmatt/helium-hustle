// Browser check: drives headless Edge or Chrome over the DevTools protocol (no
// packages) against a served page.
//   1. A fresh game through the normal UI only (no debug menu): only Solar Panel and Data
//      Center offered, the right-column program, Add to Program and loadout buttons but no
//      Run and no Programs tile (two quick adds both land), 1.1's story modal alone,
//      then the opening through 1.3 Full Load (each
//      quest change a modal chain holding the clock; the first pad offers Titanium only;
//      Launch Full Pads never shown), no Retirement
//      tile yet (it opens at the first retirement), then on at 10x (Act 1's fastest) to the forced retirement at the
//      boredom cap, the run summary holding the clock (a double-click on Continue buys
//      nothing under it), run 2 (known tiles and resources still shown, not cheated), the
//      career's play log on disk as a bot's.
//   2. Every panel: scenario late_unlocks opens Research, Projects and Ideologies (its
//      Research Lab built, past a second run); no locked project shown, a funded one; every nav tile opens; the
//      cheated mark, a jump ahead, the cheat building and Complete all research, and the
//      replay check.
//   3. Act 1's speeds (pause to 10x), a program's current-line frame at 10x and (scenario
//      everything, Accelerated Scheduling's flag set) 50x, each speed's days per second,
//      pause, click-to-buy (a green card's text buys, its −/+ and Sell 1 don't, space after
//      a click pauses and buys nothing, a gray card says why), a reload keeping the save,
//      a save from another revision discarded, the scenarios and the replay check.
//   6. (Run before 5.) The program instructions, from scenario late_unlocks: an `if` / `build` / `buy land`
//      / `if` program written through the editor, live values, unattended builds, verb
//      lines, a false `if` taking its step and jumping its body. Shot program-logic.png.
//   5. (Run before 4.) Readability: a capped resource's positive rate (Resources, Energy's
//      green; the status bar's unmarked), Resources' two groups, an affordable card's tinted
//      header and the Land row's green, scenario pad_loading's full pad short on fuel
//      then Ready (green Launch, border, nav dot), a boredom beat that leaves the main
//      quest pinned and counts unread in a folded Story until it is open and on screen
//      for a second, and phase 3 superseding it in Story. Shots read-*.png, parked-*.png.
//   4. The theme: light by default; the Options and debug toggles switch it with no
//      element moving, and a reload keeps it.
//   7. (Run before 5.) The program text editor, from scenario everything: a pasted
//      program with a red (junk) and an amber (locked) line, Apply, green and gray dots
//      and an if's fired count; the block editor keeps every line (unparsed and locked
//      rows); the text round-trips; a tab switch saves; a loadout of it loads in the
//      next run with every line. Shot text-editor.png.
//   8. (Run before 5.) else: an if/else typed in the text editor shows the same lines in
//      the block view and back, and at 1x each pass runs the branch its if picks. Shot
//      program-else.png.
//   9. (Run before 5.) The text editor's All instructions panel: on a fresh career with
//      Programming bought (scenario rich), only usable entries (nothing locked or
//      amber); clicks writing an `if ... > 0` with its body, else only where it fits,
//      hover cards; in scenario everything the same clicks Apply cleanly, the block view
//      matches and the `if` runs at 1x. Shot instructions.png.
//  10. (Run before 5.) The Programs view's Blocks mode, from scenario everything: the nav tile opens
//      it in Blocks (a new player's mode), an if/else and a 4x line built by clicks and
//      number fields only, Text shows the same lines (the count first), and at 1x the 4x
//      line's run dots fill day by day. Shots programs-blocks.png, programs-text.png,
//      programs-dots.png.
//  11. (Run before 5.) Story and Achievements: a new game played through 1.4 by the debug
//      autopilot (its step stops at 1.5 A Voice; it never buys Programming: the program shows, the Programs tile doesn't); Story shows the Act 1 band, every finished beat
//      lit, the active one glowing with its task, one "???" below and no achievements; a lit beat's click expands it (Reward line) and its Replay replays its modal; the
//      Achievements tile shows every badge grouped by category. Shots story-1.5.png,
//      achievements.png.
// Any uncaught page error fails the run.
//   HH_URL      the page, required: a `vite preview` of a static build (TESTING.md)
//   HH_BROWSER  a browser off the usual paths
//   HH_PROFILE  browser profile dir (default ../out/e2e-profile, wiped first)
//   HH_SHOTS    <dir>: save a PNG of the page at each step worth seeing
//   HH_ONLY     <n>[,<n>...]: run only these sections, in the usual order; a skipped one's
//               leftovers that later ones need (a loaded game with story modals clicked
//               through, the cheats open, 5's notice) are set up directly
// The DevTools harness (launch, waitFor, load, panel...) is scripts/cdp.mjs.
import { existsSync, mkdirSync, readFileSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { capture, driver, launch, pageUrl, q, shortCommand, sleep } from './cdp.mjs';

const here = dirname(fileURLToPath(import.meta.url));
const url = pageUrl('e2e');
/** HH_ONLY's sections, or null for all of them. */
const ONLY = process.env.HH_ONLY ? new Set(process.env.HH_ONLY.split(',').map((s) => Number(s.trim()))) : null;
if (ONLY && [...ONLY].some((n) => !Number.isInteger(n) || n < 1 || n > 11)) {
  console.error(`e2e: HH_ONLY takes section numbers 1 to 11, comma-separated: ${process.env.HH_ONLY}`);
  process.exit(2);
}
const b = await launch({ width: 1600, height: 900, profileRoot: process.env.HH_PROFILE ?? resolve(here, '../../out/e2e-profile') });
const { send, evaluate } = b;
const pageErrors = b.errors;
// settle: a dev server just after an edit may reload the page once more.
const { waitFor, exists, click, setValue, load: loadPage, day, run, speed, panel, debugMenu, cheats, grant, runOnly, readStory, autoStory, buildings } = driver(b, { url, settle: 700 });
/** Past section 1's opening, story modals are clicked through as they open, after every load too. */
let storyAuto = false;
const load = async (u = url) => {
  await loadPage(u);
  if (storyAuto) await autoStory();
};
const noticeText = () => evaluate(`document.getElementById('notice')?.textContent ?? ''`);
const enabled = (sel) => evaluate(`(() => { const b = document.querySelector(${q(sel)}); return !!b && !b.disabled; })()`);
const navVisible = (id) => exists(`.nav-tile[data-nav="${id}"]`);
/** Opens the Programs view from its nav tile, in Text mode (the view remembers the mode). */
const openText = async () => {
  await click('.nav-tile[data-nav="programs"]');
  await waitFor(`!!document.querySelector('[data-testid="mode-text"]')`, 'the Programs view', 3000);
  if (!(await exists('[data-testid="text-editor"]'))) await click('[data-testid="mode-text"]');
  await waitFor(`!!document.querySelector('[data-testid="text-editor"]')`, 'its Text mode', 3000);
};

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
/** check() for a check that directly follows an act: on a failure, also prints #notice
 * (the engine's refusal, if it gave one). */
const checkAct = async (ok, what) => {
  check(ok, what);
  if (ok) return;
  const n = await noticeText().catch(() => '');
  if (n) console.log(`     #notice: ${n}`);
};

/** The Programs view's Text mode text area, and its text. */
const AREA = '[data-testid="text-editor-area"]';
const area = () => evaluate(`document.querySelector(${q(AREA)}).value`);
/** A scenario (cheats on) played at 1x until program logic opens (the first day two
 * processors work), then paused. */
async function freshLogicGame(scenario = 'everything') {
  await debugMenu();
  await speed(0);
  await setValue('#scenario', scenario);
  await click('#start-scenario');
  await speed(1);
  await waitFor(`!!document.querySelector('[data-testid="logic-add"]')`, 'the instructions to unlock', 8000);
  await speed(0);
}
/** Every processor onto the program shown. */
const maxProcs = async () => {
  while (await enabled('[data-testid="proc-plus"]')) await click('[data-testid="proc-plus"]');
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

/** Program 1, one processor, lines of counts 2, 1, 3: at 10x (a day a redraw) the
 * current-line frame walks the queue in order, the lines keep their order and no line's
 * name moves; at 50x (several days a redraw) no frame hops around, and the lines show
 * what ran over the span. Leaves the game paused. */
async function programOrder(fast) {
  // fast: also 50x (a career holding Accelerated Scheduling's flag).
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
  if (!fast) {
    await evaluate('window.__ob.disconnect()');
    return;
  }
  await evaluate('window.__ptr = []');
  await speed(50);
  await sleep(1000);
  // Read before pausing: paused, the frame is back.
  const ptr50 = await evaluate('window.__ptr');
  const lit = await evaluate(`document.querySelectorAll('.lines .line[class*="out-"]').length`);
  const note = await exists('[data-testid="lines-span"]');
  const fastOrder = await order();
  await speed(0);
  check(
    ptr50.length > 0 && ptr50.every((p) => p === -1) && note && lit >= 2,
    `50x: no hopping pointer, lines lit by the span (pointer ${ptr50.join(',')}, ${lit} lit)`,
  );
  check(fastOrder === queued, `50x: lines keep their order: ${fastOrder}`);
  await evaluate('window.__ob.disconnect()');
}

// The Act 1 opening, as a player would: two Solar Panels (1.1), Buy Science and Basic
// Blueprints, the extraction chain, storage, the Launch Pad and its titanium shipment
// (1.2), then Advanced Blueprints, Refineries and a full pad of Helium-3 (1.3).
const ORDER = [
  ['solar_panel', 2], ['excavator', 1], ['smelter', 1], ['solar_panel', 3], ['excavator', 2],
  ['storage_depot', 1], ['solar_panel', 4], ['launch_pad', 1], ['solar_panel', 5],
  ['refinery', 1], ['solar_panel', 6], ['refinery', 2], ['solar_panel', 7],
];
/** Where play stops: 1.3 Full Load done. */
const OPENING_END = 'qtrade';

/** Plays the opening from the screen, reading each story modal as it opens. Returns the
 * quests reached and the transitions seen; checks as it goes that every quest change came
 * with a modal that held the clock, that the first pad offered Titanium only, and that
 * Launch Full Pads never showed. qa.mjs's run 1 is the other copy of this loop, kept apart
 * on purpose: it plays on past 1.3 to look at screens (a longer ORDER, a launch at 10
 * cargo), where this one stops at 1.3 checking the story (a full 20 / 100 before a launch). */
async function playOpening() {
  await speed(10);
  const started = Date.now();
  let lastLog = 0;
  let quest = (await look()).quest;
  const reached = [quest];
  /** The quests a modal told the start of (its last page's quest). */
  const told = new Set();
  /** The quests a modal showed the payoff of. */
  const paid = new Set();
  let modals = 0;
  let pausedAll = true;
  let pickerChecked = false;
  let he3Checked = false;
  let fullPads = false;
  while (Date.now() - started < 400_000) {
    if (await exists('[data-testid="story-modal"]')) {
      const d0 = await day();
      await sleep(300);
      const held = (await day()) === d0;
      modals++;
      if (modals === 2) {
        // The first transition's chain, one page read: a reload shows the next unread page
        // again, held (readStory checks paused below; the 10x check, the speed it resumes).
        const first = await evaluate(`(() => { const m = document.querySelector('[data-testid="story-modal"]'); return { quest: m.dataset.quest, kind: m.dataset.kind, task: !!m.querySelector('.story-task') }; })()`);
        if (first.kind === 'payoff') paid.add(first.quest);
        if (first.task) told.add(first.quest);
        await click('[data-testid="story-next"]', 'story next');
        const pageText = () => evaluate(`document.querySelector('[data-testid="story-modal"] .story-text')?.textContent ?? null`);
        const next = await pageText();
        if (next !== null) {
          await load(url);
          check((await pageText()) === next, `a reload mid-chain shows its next unread page again ("${next.slice(0, 40)}...")`);
        }
      }
      // A shot of each transition's last page (its task).
      const pages = await readStory(async (p) => {
        if (p.task) await shot(`story-modal-${modals}-${p.quest}`);
      });
      const last = pages.at(-1);
      if (!held || !pages.every((p) => p.paused)) pausedAll = false;
      if (last?.task) told.add(last.quest);
      for (const p of pages) if (p.kind === 'payoff') paid.add(p.quest);
      console.log(`     story modal (day ${d0}): ${pages.map((p) => `${p.quest}/${p.kind}`).join(', ')}; task "${last?.task ?? ''}"`);
      await sleep(100);
      check(await exists('#speeds button[data-speed="10"][aria-pressed="true"]'), `the modal's last page resumes 10x (${last?.quest})`);
      continue;
    }
    const s = await look();
    if (s.quest !== quest) {
      // The wake that moved the quest may have opened its modal after the check above:
      // read it first (the modal holds the clock, so if it isn't open now it never opened).
      if (await exists('[data-testid="story-modal"]')) continue;
      check(told.has(s.quest) && paid.has(quest), `${quest} -> ${s.quest} opened a story modal chain: ${quest}'s payoff, then ${s.quest}'s story ending on its task`);
      quest = s.quest;
      reached.push(quest);
    }
    if (quest === OPENING_END) break;
    if (await exists('[data-testid="summary"]')) {
      check(false, `the opening finished 1.3 before boredom retired run 1 (quest ${quest})`);
      break;
    }
    if (Date.now() - lastLog > 5000) {
      lastLog = Date.now();
      const pad = await evaluate(`(() => { const p = document.querySelector('[data-pad="0"]'); return p ? p.querySelector('select').value + ' ' + p.dataset.cargo + ' ' + p.dataset.padState : '-'; })()`);
      console.log(`     day ${s.day}, quest ${s.quest}, credits ${s.res.credits}, science ${s.res.science ?? '-'}, titanium ${s.res.titanium ?? '-'}, he3 ${s.res.he3 ?? '-'}, propellant ${s.res.propellant ?? '-'}, pad ${pad}, program ${s.lines.join(',')}`);
    }
    if (await exists('[data-panel="commands"] [data-id="launch_pads"], [data-testid="add-launch_pads"], [data-panel="commands"] option[value="launch_pads"]')) fullPads = true;
    // Research: Basic Blueprints for 1.2's extraction chain, Advanced Blueprints for 1.3's Refinery.
    const want = quest === 'qfirst_shipment' ? 'basic_blueprints' : quest === 'qfull_load' ? 'advanced_blueprints' : null;
    let science = false;
    if (want && (await navVisible('research'))) {
      await panel('research');
      const r = await evaluate(`(() => { const c = document.querySelector('[data-testid="research-${want}"]'); return c ? { done: c.classList.contains('done'), can: c.classList.contains('can-buy') } : null; })()`);
      if (r && !r.done && r.can) {
        await click(`[data-testid="research-${want}"]`);
        continue;
      }
      science = !!r && !r.done;
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
    if (science) cmd = credits >= 10 ? 'buy_science' : 'cloud_compute';
    else if ((b.launch_pad?.owned ?? 0) > 0) {
      await panel('launch_pads');
      const pad = await evaluate(`(() => { const p = document.querySelector('[data-pad="0"]'); return p && { good: p.querySelector('select').value,
        offered: [...p.querySelectorAll('select option')].map((o) => o.value), cargo: Number(p.dataset.cargo), launch: !p.querySelector('[data-testid="launch-0"]').disabled }; })()`);
      if (!pickerChecked && quest === 'qfirst_shipment') {
        pickerChecked = true;
        check(pad.offered.join(',') === ',titanium', `while 1.2 is active the first pad offers Titanium only: ${JSON.stringify(pad.offered)}`);
        await evaluate(`document.querySelector('.col-mid').scrollTop = 1e6`);
        // A closed select draws one option: list them for the shot.
        await evaluate(`document.querySelector('[data-pad="0"] select').size = 2`);
        await shot('pad-titanium-only');
        await evaluate(`document.querySelector('[data-pad="0"] select').removeAttribute('size')`);
      }
      // 1.2 ships titanium; 1.3 a full pad of Helium-3 (a switch returns what is loaded).
      const good = quest === 'qfirst_shipment' ? 'titanium' : 'he3';
      if (pad.good !== good && pad.offered.includes(good)) {
        if (good === 'he3' && !he3Checked) {
          he3Checked = true;
          check(pad.offered.length > 2, `after 1.2 a pad can carry every good: ${JSON.stringify(pad.offered)}`);
        }
        await setValue('[data-pad="0"] select', good);
        continue;
      }
      const need = good === 'titanium' ? 20 : 100;
      if (pad.good === good && pad.cargo >= need && pad.launch) {
        await click('[data-testid="launch-0"]');
        continue;
      }
      if (pad.good === good && pad.cargo < need) {
        if ((s.res[good] ?? 0) >= 5) cmd = 'load_pads';
        else if (good === 'titanium' && credits >= 25) cmd = 'buy_titanium';
      } else if (pad.good === good && (s.res.propellant ?? 0) < 20 && credits >= 12) cmd = 'buy_propellant';
    } else if (goal) cmd = shortCommand(b[goal[0]].short, credits) ?? cmd;
    await runOnly(cmd, s.lines);
    await sleep(60);
  }
  check(pausedAll, 'every story modal held the clock (paused, no day passed)');
  check(!fullPads, 'Launch Full Pads stays hidden through the opening (it opens when 1.3 Full Load completes)');
  return reached;
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
  check(!(await exists('[data-testid="logic-add"]')), 'before the unlock no logic buttons, and no hint says how they unlock');
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
    await click(`${line(i)} button[data-testid="line-indent"]`);
    await sleep(60);
  };
  await add('Build');
  await pick(`${line(0)} select[aria-label="Building"]`, 'Battery');
  await pick(`${line(0)} input[aria-label="Up to"]`, '1');
  await add('If');
  await setClause(1, 1, 'Energy', '<', '50');
  const unit = await evaluate(`document.querySelector(${q(`${clause(1, 1)} [data-unit-toggle]`)})?.textContent ?? ''`);
  check(unit === '%', `a fill reading's number shows its unit, a toggle: "${unit}"`);
  // The unit toggle: the same entry read as an amount, then back to a percent.
  await click(`${clause(1, 1)} [data-unit-toggle]`);
  await sleep(60);
  const asAmount = await evaluate(`[document.querySelector(${q(`${clause(1, 1)} [data-unit-toggle]`)}).textContent, document.querySelector(${q(line(1))}).title.split(':')[0]]`);
  check(asAmount[0] === 'amt' && asAmount[1] === 'If Energy < 50', `the toggle reads an amount: ${asAmount.join(', ')}`);
  await click(`${clause(1, 1)} [data-unit-toggle]`);
  await sleep(60);
  const asPercent = await evaluate(`document.querySelector(${q(line(1))}).title.split(':')[0]`);
  check(asPercent === 'If Energy < 50%', `and back to a percent: ${asPercent}`);
  // Process Telemetry's two readings are not offered: the research is out of Act 1 (111),
  // its readings checked in hh-core's tests.
  const readingOptions = () => evaluate(`[...document.querySelectorAll(${q(`${clause(1, 1)} select[aria-label="Reading"] option`)})].map((o) => o.textContent.trim())`);
  const offered = await readingOptions();
  check(!offered.includes('Energy per day') && !offered.includes('Days since a purchase') && offered.length <= 13, `without Process Telemetry, ${offered.length} readings`);
  await debugMenu();
  await add('Build');
  await pick(`${line(2)} select[aria-label="Building"]`, 'Solar Panel');
  await indent(2);
  await add('If');
  await setClause(3, 1, 'Free land', '<', '1');
  await pick(`${line(3)} select[aria-label="Join"]`, 'and');
  await setClause(3, 2, 'Credits', '>', '500');
  check(!(await exists(`${clause(3, 2)} [data-unit]`)) && !(await exists(`${clause(3, 2)} [data-unit-toggle]`)), 'an uncapped reading shows no unit and no toggle');
  await add('Buy Land');
  await indent(4);
  await add('If');
  await setClause(5, 1, 'Credits', '>', '800');
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
  // The depot's 30 regolith too: the scenario starts with just that, and the Smelter eats it.
  await grant('regolith', '60');
  await speed(10);
  await panel('buildings');
  await waitFor(`document.querySelector('.card.building[data-id="storage_depot"]')?.dataset.owned === '1'`, 'the Storage Depot once the if holds', 8000);
  await speed(0);
  const after = await now();
  // The depot is built (waited for above) and the if reads true. Not that day's line
  // outcomes: the new Battery can drop Energy below 50%, and the top `if` runs instead.
  check(/✓$/.test(after[5]), `the if held once Credits > 800: ${after[5]} (${await classes()})`);
  // A number being typed survives the game's redraws, and is sent on Enter or leaving.
  await speed(10);
  await evaluate(`(() => { const el = document.querySelector(${q(`${clause(5, 1)} input`)}); el.focus(); el.value = '12'; el.dispatchEvent(new Event('input', { bubbles: true })); })()`);
  await sleep(500);
  const typed = await evaluate(`document.querySelector(${q(`${clause(5, 1)} input`)}).value`);
  await speed(0);
  check(typed === '12', `a number being typed survives the day's redraws: ${typed}`);
  await pick(`${clause(5, 1)} input`, '120');
  const sent = await evaluate(`document.querySelector(${q(line(5))}).title`);
  check(/^If Credits > 120:/.test(sent),`and is sent when the field is left: ${sent.split(':')[0]}`);
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
  check(cmps === '> < >= <=', `four comparisons: ${cmps}`);
  await setClause(0, 1, 'Credits', '>', '-1');
  await add('Verb');
  await pick(`${line(1)} select[aria-label="Verb"]`, 'Pause');
  await pick(`${line(1)} select[aria-label="Target"]`, 'Solar Panel');
  await indent(1);
  await add('If');
  await setClause(2, 1, 'Free processors', '>', '-1');
  await add('Verb');
  await pick(`${line(3)} select[aria-label="Verb"]`, 'Resume');
  await pick(`${line(3)} select[aria-label="Target"]`, 'Solar Panel');
  await pick(`${line(3)} input[aria-label="Amount"]`, 'all');
  await indent(3);
  const verbShape = await evaluate(`[...document.querySelectorAll('.lines li')].map((l) => l.dataset.op + (l.classList.contains('indented') ? '>' : '')).join(' ')`);
  check(verbShape === 'if act> if act>', `verb lines as written: ${verbShape}`);
  const verbText = await evaluate(`[...document.querySelectorAll('.lines li')].map((l) => l.title.split(':')[0]).join(' | ')`);
  check(/If Credits > -1/.test(verbText) && /Pause Solar Panel/.test(verbText) && /If Free processors > -1/.test(verbText) &&/Resume Solar Panel all/.test(verbText), `verb lines read as text: ${verbText}`);
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
  await setClause(0, 1, 'Credits', '<', '-1');
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
 * and a locked building (amber); Apply; days run (green and gray dots, an if's fired
 * count); the block editor keeps every line, the unparsed and locked ones marked; the
 * text round-trips; another tab's paste saves on switching back; a loadout saved with
 * those lines loads after a retirement with all of them, the locked one shown locked.
 * Shot `text-editor.png`. */
async function textEditor() {
  await freshLogicGame();
  await click('.prog-tabs button[data-program="0"]');
  await maxProcs();
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

  await openText();
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
  await checkAct(!(await enabled('[data-testid="text-editor-apply"]')), 'and off once applied');
  const applied = await area();
  // Days run: the gutter lights by each line's outcome, and an if counts its days true.
  await speed(10);
  await waitFor(`(() => { const s = [...document.querySelectorAll('.te-gutter [data-line-status]')].map((d) => d.dataset.lineStatus); return s.includes('green') && s.includes('gray'); })()`, 'green and gray lines', 10000);
  await sleep(1500);
  await speed(0);
  const g1 = await gutter();
  const fired = g1.filter((l) => l.fired);
  check(fired.length > 0 && fired.every((l) => /^if /.test(l.text) && /^\d+\/\d+$/.test(l.fired) && l.note.includes(`fired ${l.fired}`)),
    `if lines show their fired count: ${fired.map((l) => `${l.text.slice(0, 24)}: ${l.fired}`).join('; ')}`);
  check(lineOf(g1, 'frobnicate 3')?.status === 'red' && lineOf(g1, 'build fuel cell array')?.status === 'amber', 'red and amber stay flagged after the days');
  check((await area()) === applied, 'the draft and caret text survive the redraws');
  await shot('text-editor');
  await click('[data-takeover-close="programs"]');
  await sleep(100);
  const block = () =>
    evaluate(`[...document.querySelectorAll('.lines li')].map((l) => ({ op: l.dataset.op ?? 'command', locked: l.classList.contains('is-locked'), why: l.querySelector('.line-why')?.textContent ?? '' }))`);
  const b1 = await block();
  check(b1.length === entries(p1), `the block editor keeps every line: ${b1.length} of ${entries(p1)}`);
  const unparsed = b1.filter((l) => l.op === 'unparsed');
  check(unparsed.length === 1 && /frobnicate/.test(unparsed[0].why), `the junk line is an unparsed row with its reason: ${JSON.stringify(unparsed)}`);
  check(b1.filter((l) => l.locked).length === 1 && b1.some((l) => l.locked && /locked this run/.test(l.why)), `the locked line is dimmed, saying why: ${JSON.stringify(b1.filter((l) => l.locked))}`);
  check(b1.filter((l) => l.op === 'note').length === 1, 'the # line is a note row');
  await openText();
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
  const tooMany = Array.from({ length: 130 }, () => 'idle').join('\n');
  await openText();
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
  await click('[data-takeover-close="programs"]');
  await sleep(200);
  check((await block()).length === entries(p2) && /not saved/.test(await noticeText()), `closing can't save them either, and says so: ${await noticeText()}`);
  await openText();
  await sleep(200);
  check((await area()) === tooMany, 'reopened, the unsaved 130-line draft is still there');
  await paste(p2);
  await click('[data-testid="text-editor-apply"]');
  await sleep(200);
  check(!(await enabled('[data-testid="text-editor-apply"]')) && (await area()).trimEnd() === p2, 'a valid program applies again');
  await click('[data-takeover-close="programs"]');
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
  await checkAct(b2.length === entries(p1) && b2.some((l) => l.op === 'unparsed') && b2.some((l) => l.locked && /locked this run/.test(l.why)),
    `the loadout loads every line in the new run, unparsed and locked ones kept: ${b2.length} lines, ${b2.filter((l) => l.locked).length} locked`);
  await click('[data-testid="loadout-load"]');
  await click(`[data-testid="loadout-delete-${slot}"]`);
  await sleep(100);
  await click('[data-testid="loadout-load"]');
}

/** Section 8, from scenario everything: an if/else typed in the text editor shows the
 * same lines in the block view (an Else row, both bodies indented) and reads back; at 1x
 * each day's pass runs the branch its if picks (a true if's else body jumped, a false
 * if's body jumped and its else body run), the else line free (a note's dot). Shot
 * `program-else.png`. */
async function programElse() {
  await freshLogicGame();
  await click('.prog-tabs button[data-program="0"]');
  await maxProcs();
  // Free land is never 100000: the first if is true every pass, the second false.
  const typed = 'if free land < 100000\n  idle\nelse\n  2x idle\nif free land > 100000\n  idle\nELSE\n  2x idle';
  await openText();
  await sleep(200);
  await setValue(AREA, typed);
  await sleep(300);
  const status = await evaluate(`[...document.querySelectorAll('.te-gutter [data-line-status]')].map((d) => d.dataset.lineStatus)`);
  check(!status.includes('red') && !status.includes('amber') && status[2] === 'note' && status[6] === 'note', `the if/else text reads, each else a free line: ${status.join(' ')}`);
  await click('[data-testid="text-editor-apply"]');
  await sleep(200);
  const applied = await area();
  await click('[data-takeover-close="programs"]');
  await sleep(100);
  const rows = () =>
    evaluate(`[...document.querySelectorAll('.lines li')].map((l) => ({ op: l.dataset.op ?? 'command', indented: l.classList.contains('indented'), out: [...l.classList].find((c) => c.startsWith('out-')) ?? null }))`);
  const shape = (rs) => rs.map((r) => `${r.indented ? '>' : ''}${r.op}`).join(' ');
  const r0 = await rows();
  const want = 'if >command else >command if >command else >command';
  check(shape(r0) === want, `the block view shows the same lines: ${shape(r0)}`);
  await openText();
  await sleep(200);
  // The engine writes the canonical form: `ELSE` as `else`.
  const back = (await area()).trimEnd();
  check(back === typed.replace('ELSE', 'else') && applied.trimEnd() === typed, `and back: the text reads the same (${JSON.stringify(back)})`);
  await click('[data-takeover-close="programs"]');
  await sleep(100);
  // At 1x: each day's outcomes, once a whole pass ran (the processors cover it from day 1).
  const wantOut = 'out-true out-ran - out-jumped out-false out-jumped - out-ran';
  await speed(1);
  const seen = [];
  for (let k = 0; k < 4; k++) {
    const d = await day();
    for (let t = 0; t < 50 && (await day()) === d; t++) await sleep(100);
    seen.push((await rows()).map((r) => r.out ?? '-').join(' '));
  }
  await speed(0);
  await shot('program-else');
  check(seen.slice(1).every((s) => s === wantOut), `at 1x each pass runs the branch its if picks: ${seen.join(' | ')}`);
}

/** Section 9 (before 5): the text editor's All instructions panel. On a new game (a fresh
 * career) it lists only what can be used now: Commands and a `#`, nothing locked. Then in
 * scenario everything (`if` open) clicking `if ... > 0` then a command writes an `if` with
 * its body, the caret on it; `else` is offered only below an `if` or its body; hovering a
 * command or a building shows its card; it Applies cleanly, the block view shows the same
 * lines, and at 1x the `if` runs. Shots `instructions-fresh.png`, `instructions.png`. */
async function instructionPanel() {
  const entries = (sel) => evaluate(`[...document.querySelectorAll(${q(sel)})].map((b) => ({ text: b.dataset.instruction }))`);
  const hover = async (sel) => {
    await evaluate(`document.querySelector(${q(sel)}).dispatchEvent(new MouseEvent('mouseenter'))`);
    await sleep(150);
    return evaluate(`document.querySelector('[data-testid="instruction-card"]')?.dataset.card ?? null`);
  };
  const build = async () => {
    await click('.prog-tabs button[data-program="0"]');
    await openText();
    await sleep(300);
    const groups = await evaluate(`[...document.querySelectorAll('.ins-group')].map((g) => g.dataset.group)`);
    check(groups[0] === 'Commands' && groups.at(-1) === 'Control flow', `instruction groups: ${groups.join(', ')}`);
    const ifs = await entries('[data-group="Control flow"] [data-instruction^="if "]');
    check(ifs.length > 0 && ifs.every((x) => / > 0%?$/.test(x.text)), `one if per reading, each > 0: ${ifs.map((x) => x.text).join(' | ')}`);
    check(!(await exists('[data-instruction="else"]')), 'no else in an empty program');
    await click('[data-group="Control flow"] [data-instruction^="if "]');
    await sleep(300);
    const [cmd] = await entries('[data-group="Commands"] .ins-item');
    await click(`[data-group="Commands"] [data-instruction=${q(cmd.text)}]`);
    await sleep(300);
    const text = await area();
    const caret = await evaluate(`(() => { const a = document.querySelector(${q(AREA)}); return a.value.slice(0, a.selectionStart).split('\\n').length - 1; })()`);
    check(new RegExp(`^${ifs[0].text}\\n  ${cmd.text}$`).test(text) && caret === 1, `the clicks write an if and its body, the caret on it: ${JSON.stringify(text)} (line ${caret})`);
    check(await exists('[data-instruction="else"]'), 'else offered below an if body');
    const cmdCard = await hover(`[data-group="Commands"] [data-instruction=${q(cmd.text)}]`);
    const bldCard = await hover('[data-group="Build"] .ins-item');
    check(/^command:/.test(cmdCard ?? '') && /^building:/.test(bldCard ?? ''), `hovers show the cards: ${cmdCard}, ${bldCard}`);
    // With the caret on the `if` line, a line goes into its body; no else there (it would take the body).
    await evaluate(`(() => { const a = document.querySelector(${q(AREA)}); a.focus(); a.setSelectionRange(0, 0); a.dispatchEvent(new KeyboardEvent('keyup')); })()`);
    await sleep(300);
    check(!(await exists('[data-instruction="else"]')), 'no else on an if line with a body');
    await click(`[data-group="Commands"] [data-instruction=${q(cmd.text)}]`);
    await sleep(300);
    const under = await area();
    check(under === `${ifs[0].text}\n  ${cmd.text}\n  ${cmd.text}`, `below an if line, into its body: ${JSON.stringify(under)}`);
    return { ifs, cmd, text };
  };

  await debugMenu();
  await speed(0);
  // A fresh career with Programming bought (a new game has no Programs view until then).
  await setValue('#scenario', 'rich');
  await click('#start-scenario');
  await sleep(500);
  await click('.prog-tabs button[data-program="0"]');
  await openText();
  await sleep(300);
  // Only what the player can use now: the commands and a note, nothing locked (no amber).
  const freshGroups = await evaluate(`[...document.querySelectorAll('.ins-group')].map((g) => g.dataset.group)`);
  const flowFresh = (await entries('[data-group="Control flow"] .ins-item')).map((x) => x.text);
  const panelText = await evaluate(`document.querySelector('[data-testid="instructions"]').textContent`);
  check(
    freshGroups.join(',') === 'Commands,Control flow' && flowFresh.join(',') === '#' && !/opens at|isn't|Locked/.test(panelText),
    `on a fresh career only usable entries: ${freshGroups.join(', ')}; control flow ${flowFresh.join(' ')}`,
  );
  await shot('instructions-fresh');
  await click('[data-takeover-close="programs"]');
  await sleep(100);

  await freshLogicGame();
  await click('.prog-tabs button[data-program="0"]');
  await click('.prog-procs button[title="Clear this program"]');
  await maxProcs();
  const open = await build();
  await shot('instructions');
  await click('[data-testid="text-editor-apply"]');
  await sleep(300);
  const status = await evaluate(`[...document.querySelectorAll('.te-gutter [data-line-status]')].map((d) => d.dataset.lineStatus)`);
  const saved = await evaluate(`document.querySelector('[data-testid="text-editor-status"]').textContent`);
  check(/Saved/.test(saved) && !status.includes('red') && !status.includes('amber'), `it applies cleanly: ${saved} (${status.join(' ')})`);
  await click('[data-takeover-close="programs"]');
  await sleep(100);
  const rows = () => evaluate(`[...document.querySelectorAll('.lines li')].map((l) => ({ op: l.dataset.op ?? 'command', indented: l.classList.contains('indented'), out: [...l.classList].find((c) => c.startsWith('out-')) ?? null }))`);
  const shape = (await rows()).map((r) => `${r.indented ? '>' : ''}${r.op}`).join(' ');
  check(shape === 'if >command >command', `the block view shows the same lines: ${shape}`);
  await speed(1);
  const d = await day();
  for (let t = 0; t < 50 && (await day()) <= d + 1; t++) await sleep(100);
  await speed(0);
  const ran = (await rows()).map((r) => r.out ?? '-');
  check(ran[0] === 'out-true' || ran[0] === 'out-false', `at 1x the if runs: ${ran.join(' ')}`);
}

/** Section 10: in scenario everything (`if` open; a fresh career's panel lists no `if`), a
 * program with an if/else and a 4x line built in Blocks only, the same lines in Text, and
 * the 4x line's run dots filling at 1x. */
/** The last row's repeat-count box under a real mouse: a click selects its text (typing
 * replaces the 4), a second click does again, and a drag inside it starts no row drag
 * (a drag from the row's name does). Leaves the count at 4. */
async function numberBox(rows) {
  const box = `document.querySelector('[data-testid="block-lines"] > li:last-child [data-testid="repeat-count"]')`;
  const rect = (sel) => evaluate(`(() => { const r = ${sel}.getBoundingClientRect(); return { l: r.left, r: r.right, y: r.top + r.height / 2 }; })()`);
  const mouse = async (type, x, y) => send('Input.dispatchMouseEvent', { type, x, y, button: 'left', buttons: type === 'mouseReleased' ? 0 : 1, clickCount: type === 'mouseMoved' ? 0 : 1 });
  const clickBox = async () => {
    const b = await rect(box);
    const x = (b.l + b.r) / 2;
    await mouse('mousePressed', x, b.y);
    await mouse('mouseReleased', x, b.y);
    await sleep(100);
  };
  await clickBox();
  await send('Input.insertText', { text: '7' });
  const typed = await evaluate(`${box}.value`);
  check((await evaluate(`document.activeElement === ${box}`)) && typed === '7', `a click in the number box selects it: typing gives "${typed}"`);
  await clickBox();
  await send('Input.insertText', { text: '4' });
  const again = await evaluate(`${box}.value`);
  await evaluate(`${box}.blur()`);
  await sleep(250);
  check(again === '4' && (await evaluate(`${box}.value`)) === '4', `a second click selects it again: "${again}"`);
  // A drag inside the box: no row drag (the row is undraggable while the button is down).
  await evaluate(`window.__drags = 0; document.addEventListener('dragstart', () => window.__drags++, true)`);
  const drag = async (x0, x1, y, mid) => {
    await mouse('mousePressed', x0, y);
    const during = mid ? await evaluate(mid) : null;
    for (let k = 1; k <= 6; k++) await mouse('mouseMoved', x0 + ((x1 - x0) * k) / 6, y + k * 3);
    await mouse('mouseReleased', x1, y + 18);
    await sleep(150);
    return during;
  };
  const before = (await rows()).join(' ');
  const b = await rect(box);
  const during = await drag(b.l + 3, b.r - 3, b.y, `${box}.closest('li').draggable`);
  const after = (await rows()).join(' ');
  const drags = await evaluate('window.__drags');
  const back = await evaluate(`${box}.closest('li').draggable`);
  check(during === false && back === true && drags === 0 && after === before, `a drag in the box moves no row (draggable while down: ${during}, after: ${back}, drags ${drags}, ${after})`);
  // The drag highlighted the digit: typing replaces it. Then back to 4.
  await send('Input.insertText', { text: '5' });
  const dragged = await evaluate(`${box}.value`);
  check(dragged === '5', `the drag selected the box's text: typing gives "${dragged}"`);
  // A drag that starts on text a click selected selects again, rather than dragging it off.
  await clickBox();
  const b2 = await rect(box);
  const during2 = await drag(b2.l + 3, b2.r - 3, b2.y, `${box}.closest('li').draggable`);
  await send('Input.insertText', { text: '4' });
  const redragged = await evaluate(`${box}.value`);
  const drags2 = await evaluate('window.__drags');
  check(during2 === false && drags2 === 0 && redragged === '4' && (await rows()).join(' ') === before, `a drag over selected text selects, no drag: typing gives "${redragged}", drags ${drags2}`);
  await evaluate(`${box}.blur()`);
  await sleep(250);
  // The same drag from the row's name does start one.
  const n = await rect(`document.querySelector('[data-testid="block-lines"] > li:last-child .line-name')`);
  await drag(n.l + 3, n.r - 3, n.y, null);
  const control = await evaluate('window.__drags');
  check(control === 1, `a drag from the row's name starts a row drag: ${control} dragstart`);
  const end = (await rows()).join(' ');
  check(end === before && (await evaluate(`${box}.value`)) === '4', `the rows and count are as before: ${end}`);
  // Esc while typing in the box closes the view and keeps the number, as a click away does
  // (082: the box was removed while focused and its typed count dropped).
  await clickBox();
  await send('Input.insertText', { text: '6' });
  await send('Input.dispatchKeyEvent', { type: 'keyDown', key: 'Escape', code: 'Escape', windowsVirtualKeyCode: 27 });
  await send('Input.dispatchKeyEvent', { type: 'keyUp', key: 'Escape', code: 'Escape', windowsVirtualKeyCode: 27 });
  await sleep(150);
  const closed = !(await exists('[data-takeover="programs"]'));
  await click('.nav-tile[data-nav="programs"]');
  await waitFor(`!!document.querySelector('[data-testid="block-editor"]')`, 'the Blocks mode', 3000);
  const kept = await evaluate(`${box}.value`);
  check(closed && kept === '6', `Esc in the box closes the view and keeps the typed count: closed ${closed}, count "${kept}"`);
  await clickBox();
  await send('Input.insertText', { text: '4' });
  await evaluate(`${box}.blur()`);
  await sleep(250);
}

/** Section 11: Story and Achievements. A new game played on by the debug autopilot's
 * "Next quest step" through 1.4, to 1.5 A Voice; Story then shows the Act 1 band, every finished beat
 * lit with its picture's thumbnail, the active beat glowing with its task, one "???" below
 * it and no achievements or event texts; a finished beat's click expands it to its story
 * and its Reward line, and its Replay button replays its story modal.
 * The Achievements tile shows badges grouped by category. Then scenario late_unlocks: 1.b's
 * completion banner shows its picture. Shots story-1.5.png, achievements.png,
 * side-quest-banner.png. */
async function storyView() {
  await debugMenu();
  await speed(0);
  await click('#new-game');
  await sleep(500);
  await autoStory();
  const content = (f) => JSON.parse(readFileSync(resolve(here, `../../content/${f}`), 'utf8'));
  const beat = () => evaluate(`document.querySelector('[data-testid="quest"]')?.textContent.match(/^1\\.(\\d+)/)?.[1] ?? '0'`).then(Number);
  // Step until 1.5 A Voice is active; it can't finish in the step that opens it
  // (Geopolitical Intelligence opens with it).
  for (let step = 0; step < 8 && (await beat()) < 5; step++) {
    await cheats();
    await click('#accel-quest', 'autopilot: next quest step');
    await waitFor(`document.getElementById('accel-status')?.dataset.running === '0'`, 'the autopilot step', 90000);
    if (await exists('[data-testid="continue"]')) await click('[data-testid="continue"]', 'continue');
  }
  const at = await beat();
  check(at === 5, `the autopilot reaches 1.5 A Voice (stops at 1.${at})`);
  // The autopilot never buys Programming (091): the program shows, the Programs tile doesn't.
  check(
    !(await exists('.nav-tile[data-nav="programs"]')) && (await exists('[data-testid="programs"]')),
    'after the autopilot the right-column program shows and the Programs tile does not',
  );
  await panel('story');
  await sleep(300);
  const st = await evaluate(`(() => {
    const p = document.querySelector('[data-panel="story"]');
    const ids = (sel) => [...p.querySelectorAll(sel)].map((e) => e.dataset.beat ?? e.dataset.act ?? '');
    const mid = document.querySelector('.col-mid').getBoundingClientRect();
    const a = p.querySelector('[data-testid="story-beat-active"]');
    const r = a?.getBoundingClientRect();
    return {
      bands: [...p.querySelectorAll('[data-testid="act-band"]')].map((b) => b.textContent),
      bandArt: [...p.querySelectorAll('[data-testid="act-band"]')].map((b) => /url\\(/.test(getComputedStyle(b).backgroundImage)),
      thumbs: [...p.querySelectorAll('[data-testid="story-beat-done"]')].map((b) => b.querySelector('[data-testid="quest-pic-thumb"]')?.dataset.picture ?? ''),
      leftPic: document.querySelector('.col-left > .quest-scene [data-testid="quest-pic-left"]')?.dataset.picture ?? null,
      activeThumb: a?.querySelector('[data-testid="quest-pic-thumb"]')?.dataset.picture ?? null,
      thumbsLoaded: [...p.querySelectorAll('[data-testid="quest-pic-thumb"]')].every((i) => i.complete && i.naturalWidth > 0),
      done: ids('[data-testid="story-beat-done"]'),
      // 096: one line a card, its first sentence and "…".
      lines: [...p.querySelectorAll('[data-testid="story-beat-done"]')].map((b) => {
        const l = b.querySelector('[data-testid="story-card-line"]');
        return l && l.getBoundingClientRect().height < parseFloat(getComputedStyle(l).lineHeight) * 1.3 ? l.textContent : '';
      }),
      active: a ? a.dataset.beat : null,
      // 096: a ring pulses out of the dot (no filled glow over the spine).
      glow: a ? getComputedStyle(a.querySelector('.beat-dot'), '::after').animationName === 'beat-ring' && getComputedStyle(a.querySelector('.beat-dot')).boxShadow === 'none' : false,
      task: a ? a.textContent : '',
      bar: !!a?.querySelector('.bar, [role="progressbar"]'),
      unknown: [...p.querySelectorAll('[data-testid="story-unknown"]')].map((e) => e.textContent),
      unknownAfter: !!a && !!p.querySelector('[data-testid="story-unknown"]') && (a.compareDocumentPosition(p.querySelector('[data-testid="story-unknown"]')) & Node.DOCUMENT_POSITION_FOLLOWING) > 0,
      inView: !!r && r.top < mid.bottom && r.bottom > mid.top,
      badges: p.querySelectorAll('[data-testid="badge"], [data-achievement]').length,
      text: p.textContent,
    };
  })()`);
  check(st.bands.length === 1 && /Act 1/.test(st.bands[0]) && /The Lunar Operation/.test(st.bands[0]), `Story opens with the Act 1 band alone: ${JSON.stringify(st.bands)}`);
  check(st.bandArt.every(Boolean), `the act band is painted: ${JSON.stringify(st.bandArt)}`);
  const chain = content('events.json').filter((e) => e.category === 'story').map((e) => e.id);
  // Every Act 1 quest has its picture: each finished beat's thumbnail, the active beat's,
  // and the active quest's at the left column's foot (the horizon only for one without).
  const pictureOf = Object.fromEntries(content('events.json').map((e) => [e.id, e.picture ?? '']));
  const want = chain.slice(0, at - 1).map((id) => pictureOf[id]);
  check(want.every(Boolean) && st.thumbs.join() === want.join() && st.thumbsLoaded, `each finished beat shows its picture's thumbnail (loaded ${st.thumbsLoaded}): ${JSON.stringify(st.thumbs)}`);
  check(!!pictureOf[chain[at - 1]] && st.activeThumb === pictureOf[chain[at - 1]] && st.leftPic === st.activeThumb, `the active beat's thumbnail and the left column's foot show 1.${at}'s picture: ${st.activeThumb} / ${st.leftPic}`);
  check(st.done.join() === chain.slice(0, at - 1).join(), `the ${at - 1} finished beats lit, in order: ${st.done.join(', ')}`);
  check(st.lines.length > 0 && st.lines.every((t) => t.endsWith('…')), `each finished beat shows one line ending "…": ${JSON.stringify(st.lines.slice(0, 2))}`);
  check(st.active === chain[at - 1] && st.glow, `1.${at} is the glowing active beat: ${st.active} glow ${st.glow}`);
  check(st.task.trim().startsWith(`1.${at}`) && st.task.length > 20 && st.bar, `the active beat shows its task and progress bar (${st.bar}): ${st.task.slice(0, 80)}`);
  check(st.unknown.length === 1 && st.unknown[0].trim() === '???' && st.unknownAfter, `one "???" node below it: ${JSON.stringify(st.unknown)}`);
  check(st.inView, 'the active beat is in view on opening');
  check(st.badges === 0 && !/Achievements|Events this run/.test(st.text), 'Story shows no achievements and no Events this run');
  await shot('story-1.5');
  // A finished beat's click expands it (108: whole story, Goal and Reward lines in the UI
  // font); its Replay button replays its story modal (pages read by hand, not auto-clicked).
  await evaluate(`window.__autoStory?.disconnect(); window.__autoStory = undefined`);
  await click('[data-testid="story-beat-done"][data-beat="qfirst_shipment"] [data-testid="story-beat-toggle"]', 'expand 1.2');
  const open12 = await evaluate(`(() => {
    const c = document.querySelector('[data-testid="story-beat-done"][data-beat="qfirst_shipment"]');
    const m = c.querySelector('[data-testid="story-card-body"] .mechanics');
    return { reward: m?.querySelector('.mech-reward')?.textContent ?? '', font: m ? getComputedStyle(m).fontFamily : '' };
  })()`);
  // 1.2 grants nothing, so it has no Reward line (111: rewards list only real grants).
  check(open12.reward === '' && !!open12.font && !/Mono|monospace/i.test(open12.font), `expanded 1.2 shows no reward line ("${open12.reward}") in ${open12.font.slice(0, 20)}`);
  await click('[data-testid="story-beat-done"][data-beat="qfirst_shipment"] [data-testid="story-replay"]', 'replay 1.2');
  const pages = await readStory();
  check(pages.length > 0 && pages.every((p) => p.quest === 'qfirst_shipment'), `a finished beat's Replay replays its modal chain: ${pages.map((p) => `${p.quest}/${p.kind}`).join(' ')}`);
  await autoStory();
  // Achievements: its own tile, badges grouped by category.
  check(await navVisible('achievements'), 'an Achievements nav tile');
  await panel('achievements');
  const ach = await evaluate(`(() => {
    const p = document.querySelector('[data-panel="achievements"]');
    return {
      groups: [...p.querySelectorAll('.badge-group')].map((g) => g.dataset.category + ':' + g.querySelectorAll('[data-testid="badge"]').length),
      states: [...p.querySelectorAll('[data-testid="badge"]')].map((b) => (b.classList.contains('is-earned') ? 'e' : 'u')).join(''),
      rewards: [...p.querySelectorAll('.badge-reward')].length,
      medals: [...p.querySelectorAll('[data-testid="badge"] .badge-medal')].map((m) => (m.textContent === '★' ? 'e' : 'u')).join(''),
      head: p.querySelector('h2').textContent,
    };
  })()`);
  // Each badge earned or unearned, its medal and the panel's count agreeing with its state.
  const earnedN = [...ach.states].filter((x) => x === 'e').length;
  check(
    ach.states === ach.medals && ach.head.includes(`${earnedN}/${ach.states.length}`),
    `badges show earned and unearned states (${ach.states}; head "${ach.head}")`,
  );
  const all = content('achievements.json');
  const cats = [...new Set(all.map((a) => a.category))];
  check(
    ach.groups.join(' ') === cats.map((c) => `${c}:${all.filter((a) => a.category === c).length}`).join(' ') && ach.rewards === all.length,
    `Achievements shows every badge grouped by category, each with its reward: ${ach.groups.join(' ')} (${ach.states.length} badges)`,
  );
  await shot('achievements');
  // A side quest has no modal: its completion banner carries its picture (scenario
  // late_unlocks, a Research Lab built and 1.b open; Complete all research finishes 1.b).
  await debugMenu();
  await speed(0);
  await setValue('#scenario', 'late_unlocks');
  await click('#start-scenario');
  await speed(1);
  await sleep(1000);
  await speed(0);
  await debugMenu();
  // The popup is read here, not clicked away (autoStory, back on since the Replay).
  await evaluate(`window.__autoStory?.disconnect(); window.__autoStory = undefined`);
  await click('#cheat-complete-research');
  await speed(1);
  // Any popup before it (events, other quests) is read on the way.
  await waitFor(
    `(() => { const m = document.querySelector('[data-testid="story-modal"]'); if (m && m.dataset.quest !== 'qfull_cluster') document.querySelector('[data-testid="story-next"]').click(); return m?.dataset.quest === 'qfull_cluster'; })()`,
    "1.b's completion popup",
    20000,
  );
  const popup = await evaluate(`(() => { const i = document.querySelector('[data-testid="story-modal"] [data-testid="quest-pic-modal"]'); return { pic: i?.dataset.picture ?? null, loaded: !!i && i.complete && i.naturalWidth > 0, listed: !!document.querySelector('[data-side-quest="qfull_cluster"]') }; })()`);
  const sidePic = content('events.json').find((e) => e.id === 'qfull_cluster').picture;
  check(!!sidePic && popup.pic === sidePic && popup.loaded && !popup.listed, `a side quest's completion popup shows its picture; the quest left the right panel: ${JSON.stringify(popup)}`);
  await shot('side-quest-popup');
  await click('[data-testid="story-next"]');
}

async function programsBlocks() {
  const rows = () =>
    evaluate(`[...document.querySelectorAll('[data-testid="block-lines"] > li')].map((l) => (l.classList.contains('indented') ? '>' : '') + (l.dataset.op ?? 'command'))`);
  await freshLogicGame();
  // A new player's mode: Blocks.
  await evaluate(`localStorage.removeItem('hh-programs-mode')`);
  await click('.prog-tabs button[data-program="0"]');
  await click('.prog-procs button[title="Clear this program"]');
  await click('.nav-tile[data-nav="programs"]');
  await waitFor(`!!document.querySelector('[data-takeover="programs"]')`, 'the Programs view', 3000);
  const title = await evaluate(`document.querySelector('[data-takeover="programs"] h2').textContent`);
  check(title === 'Programs' && (await exists('[data-testid="block-editor"]')), `the Programs tile opens "${title}" in Blocks`);
  check(!(await exists('[data-testid="text-editor-open"]')), 'no Text editor button in the program panel');
  const ins = (sel) => evaluate(`[...document.querySelectorAll(${q(sel)})].map((b) => ({ text: b.dataset.instruction }))`);
  const [cmd] = await ins('[data-group="Commands"] .ins-item');
  const add = async (sel) => {
    await click(sel);
    await sleep(250);
  };
  await add('[data-group="Control flow"] [data-instruction^="if "]');
  await add(`[data-group="Commands"] [data-instruction=${q(cmd.text)}]`);
  await add('[data-instruction="else"]');
  await add(`[data-group="Commands"] [data-instruction=${q(cmd.text)}]`);
  await add(`[data-group="Commands"] [data-instruction=${q(cmd.text)}]`);
  // The last one out of the else's body, then 4 runs in its number field.
  await click('[data-testid="block-lines"] > li:last-child [data-testid="line-outdent"]');
  await sleep(250);
  await evaluate(`(() => {
    const el = document.querySelector('[data-testid="block-lines"] > li:last-child [data-testid="repeat-count"]');
    el.value = '4';
    el.dispatchEvent(new Event('input', { bubbles: true }));
    el.dispatchEvent(new Event('change', { bubbles: true }));
  })()`);
  await sleep(250);
  const shape = (await rows()).join(' ');
  check(shape === 'if >command else >command command', `Blocks builds an if/else and a line after it: ${shape}`);
  check(!(await exists('.block-main .hint')), 'no hint lines in the Programs view');
  await numberBox(rows);
  await shot('programs-blocks');
  // Text: the same lines, the count first.
  await click('[data-testid="mode-text"]');
  await waitFor(`!!document.querySelector('[data-testid="text-editor-area"]')`, 'the Text mode', 3000);
  const text = await evaluate(`document.querySelector('[data-testid="text-editor-area"]').value`);
  const ifText = (await ins('[data-group="Control flow"] [data-instruction^="if "]'))[0]?.text;
  const want = `${ifText}\n  ${cmd.text}\nelse\n  ${cmd.text}\n4x ${cmd.text}\n`;
  check(text === want, `Text shows the same lines: ${JSON.stringify(text)}`);
  await shot('programs-text');
  await click('[data-testid="mode-blocks"]');
  await waitFor(`!!document.querySelector('[data-testid="block-editor"]')`, 'the Blocks mode', 3000);
  check((await evaluate(`localStorage.getItem('hh-programs-mode')`)) === 'blocks', 'the view remembers the mode');
  // At 1x with one processor on it, the 4x line's dots fill a run a day (more would run it whole in a day).
  await click('[data-takeover-close="programs"]');
  while (await enabled('[data-testid="proc-minus"]')) await click('[data-testid="proc-minus"]');
  await click('[data-testid="proc-plus"]');
  await click('.nav-tile[data-nav="programs"]');
  await waitFor(`!!document.querySelector('[data-testid="block-editor"]')`, 'the Blocks mode', 3000);
  const rep = () => evaluate(`document.querySelector('[data-testid="block-lines"] > li:last-child [data-repeat]')?.dataset.repeat ?? null`);
  const seen = [await rep()];
  await speed(1);
  for (let t = 0; t < 150 && !seen.includes('4/4'); t++) {
    await sleep(100);
    const r = await rep();
    if (r !== seen.at(-1)) seen.push(r);
  }
  await speed(0);
  const dots = await evaluate(`document.querySelectorAll('[data-testid="block-lines"] > li:last-child .rep-dot').length`);
  check(dots === 4 && ['1/4', '2/4', '3/4', '4/4'].every((x) => seen.includes(x)), `at 1x the 4x line's 4 dots fill: ${seen.join(' → ')}`);
  await shot('programs-dots');
  // Above 8 runs: a short bar, "done/total, N failed" on hover.
  await evaluate(`(() => {
    const el = document.querySelector('[data-testid="block-lines"] > li:last-child [data-testid="repeat-count"]');
    el.value = '12';
    el.dispatchEvent(new Event('input', { bubbles: true }));
    el.dispatchEvent(new Event('change', { bubbles: true }));
  })()`);
  await speed(1);
  const bar = `document.querySelector('[data-testid="block-lines"] > li:last-child .rep-bar')`;
  await waitFor(`/^[1-9]\\d*\\/12$/.test(${bar}?.dataset.repeat ?? '')`, 'a run of the 12x line', 8000);
  await speed(0);
  const barTitle = await evaluate(`${bar}.title`);
  check(/^\d+\/12(, \d+ failed)?$/.test(barTitle) && !(await exists('[data-testid="block-lines"] > li:last-child .rep-dot')), `a 12x line shows a bar, not dots: "${barTitle}"`);
  await click('[data-takeover-close="programs"]');
}

/** Section 5, from scenario at_cap (paused, cheats on; `grant` from section 2). Shots
 * `read-*.png`. Leaves a notice up (the replay check's) for section 4. */
async function readability(grant) {
  // Income at cap: Energy at 100 / 100 under 20 Solar Panels still shows its surplus.
  const capped = await evaluate(`document.querySelector('tr[data-res="energy"] [data-capped]')?.textContent ?? ''`);
  check(/^\+[\d.]+ cap$/.test(capped), `a capped resource shows its positive rate: "${capped}"`);
  const bar = await evaluate(`document.querySelector('[data-testid="energy-rate"]').textContent`);
  // The status bar never says "capped" for Energy (Matt, 2026-11-25): its plain rate.
  check(/^\+[\d.]+\/day$/.test(bar), `the status bar's Energy rate, unmarked: "${bar}"`);
  // Capped Energy's surplus is spare power: green (the text token), not muted.
  const spare = await evaluate(`(() => {
    const el = document.querySelector('tr[data-res="energy"] [data-capped]');
    const root = getComputedStyle(document.documentElement);
    const probe = (v) => { const s = document.createElement('span'); s.style.color = v; document.body.append(s); const c = getComputedStyle(s).color; s.remove(); return c; };
    return { color: getComputedStyle(el).color, green: probe(root.getPropertyValue('--pos-text')), muted: probe(root.getPropertyValue('--muted')) };
  })()`);
  check(spare.color === spare.green && spare.color !== spare.muted, `capped Energy's rate is green: ${JSON.stringify(spare)}`);
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
  // at_cap builds only Solar Panels and Data Centers, past what its caps can pay: the
  // Blueprints (by the cheat) open the cheaper cards.
  await click('#cheat-complete-research');
  await grant('titanium', '20');
  await grant('regolith', '50');
  await panel('buildings');
  const heads = await evaluate(`(() => {
    const bg = (sel) => { const h = document.querySelector(sel + ' .card-head'); return h ? getComputedStyle(h).backgroundColor : null; };
    const card = (sel) => { const c = document.querySelector(sel); return c ? getComputedStyle(c).backgroundColor : null; };
    return { yes: bg('.card.building.can-buy:not(.cheat-card)'), no: bg('.card.building.cannot-buy'), body: card('.card.building.can-buy:not(.cheat-card)') };
  })()`);
  check(!!heads.yes && !!heads.no && heads.yes !== heads.no && heads.yes !== heads.body, `an affordable card's header is tinted, the card body isn't: ${JSON.stringify(heads)}`);
  // The Land row, affordable: the cards' green outline and tinted head, its box unchanged.
  const land = await evaluate(`(() => {
    const b = document.querySelector('[data-testid="land-bar"]'), r = b.getBoundingClientRect(), s = getComputedStyle(b);
    return { can: !document.querySelector('[data-testid="buy-land"]').disabled, cls: b.classList.contains('can-buy'), border: s.borderTopColor, bg: s.backgroundColor,
      head: getComputedStyle(document.querySelector('.card.building.can-buy:not(.cheat-card) .card-head')).backgroundColor,
      cardBorder: getComputedStyle(document.querySelector('.card.building.can-buy:not(.cheat-card)')).borderTopColor, h: r.height };
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
  // Fold Events: a folded section keeps its count.
  await click('[data-testid="events-list"] .fold-head');
  // The scenario's pad starts with No cargo: idle, red nav dot; picking He-3 clears both.
  await panel('launch_pads');
  const none = await evaluate(`(() => {
    const p = document.querySelector('[data-pad="0"]');
    return { value: p.querySelector('select').value, status: p.querySelector('.pad-status').textContent,
      state: p.dataset.padState, dot: !!document.querySelector('[data-testid="pads-no-cargo"]') };
  })()`);
  check(none.value === '' && none.status === 'No cargo: pick one' && none.state === 'nocargo' && none.dot,
    `a scenario's pad starts with No cargo, nav dot on: ${JSON.stringify(none)}`);
  await evaluate(`document.querySelector('.col-mid').scrollTop = 1e6`);
  await shot('pad-no-cargo');
  await setValue('[data-pad="0"] select', 'he3');
  await sleep(200);
  const picked = await evaluate(`({ value: document.querySelector('[data-pad="0"] select').value, dot: !!document.querySelector('[data-testid="pads-no-cargo"]') })`);
  await checkAct(picked.value === 'he3' && !picked.dot,`picking He-3 clears No cargo and its dot: ${JSON.stringify(picked)}`);
  const lines = await evaluate(`[...document.querySelectorAll('.lines li')].map((li) => li.dataset.command)`);
  await runOnly('load_pads', lines);
  // Helium-3 caps at 20 and a load moves 5: top it up and play 4 days, 5 times.
  await debugMenu();
  for (let i = 0; i < 5; i++) {
    await grant('he3', '20');
    await setValue('#cheat-days', '4');
    await click('#cheat-jump');
    await sleep(100);
    // A boredom beat on the way pops up (it holds the clock: the next jump would play nothing).
    await readStory();
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
  // The boredom beat: in Events, unread; the quest still pinned.
  const beat = await evaluate(`(() => ({
    pin: document.querySelector('[data-testid="quest-pin"] [data-testid="quest"]')?.dataset.quest ?? null,
    unread: Number(document.querySelector('[data-testid="events-list"]').dataset.unread),
    head: document.querySelector('[data-testid="events-list"] .fold-head').textContent,
    // Events is folded: the beat's line in Recent Activity says it fired.
    // (Phase 2's beat is Burnout in the run Burnout fires in, Boredom Rising after it.)
    beat: [...document.querySelectorAll('[data-testid="events-activity"] li[data-kind="event"]')].some((e) => /Boredom Rising|Burnout/.test(e.textContent)),
    slot: [...document.querySelectorAll('[data-testid="quest-pin"] [data-quest]')].map((e) => e.dataset.quest),
  }))()`);
  // The slot holds the active quest (1.3 Full Load: the scenario starts past 1.2) and nothing else.
  check(/^q/.test(beat.pin ?? '') && beat.slot.length === 1 && beat.beat && beat.unread >= 1,
    `a boredom beat fired: the quest stays pinned, Events counts it unread: ${JSON.stringify(beat)}`);
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
  // Unfold Events: its count clears after about a second on screen, with no click on an item.
  await click('[data-testid="events-list"] .fold-head');
  await sleep(300);
  const early = Number(await evaluate(`document.querySelector('[data-testid="events-list"]').dataset.unread`));
  await shot('read-pad-ready-story-open');
  await waitFor(`document.querySelector('[data-testid="events-list"]').dataset.unread === '0'`, 'Events read', 4000);
  check(early >= 1, `Events' count holds for a moment once open (${early}), then clears`);
  check(
    (await evaluate(`document.querySelector('[data-testid="quest-pin"] [data-testid="quest"]')?.dataset.quest ?? null`)) === beat.pin,
    'the quest is still pinned after Events is read',
  );
  await shot('read-story-read');
  // Past day 500 boredom phase 3 supersedes phase 2 in Events; Recent Activity keeps both.
  await debugMenu();
  await setValue('#cheat-days', '320');
  await click('#cheat-jump');
  await sleep(300);
  await panel('launch_pads');
  const phases = await evaluate(`(() => ({
    story: [...document.querySelectorAll('[data-testid="events-list"] [data-event]')].map((e) => e.dataset.event),
    activity: [...document.querySelectorAll('[data-testid="events-activity"] li[data-kind="event"]')].map((e) => e.textContent),
  }))()`);
  const boredom = phases.story.filter((id) => id.startsWith('boredom_phase_'));
  // Phase 2's notice is Burnout in the run Burnout fires in, Boredom Rising after it.
  check(boredom.length === 1 && boredom[0] === 'boredom_phase_3' && phases.activity.some((t) => /Boredom Rising|Burnout/.test(t)) && phases.activity.some((t) => /Boredom Accelerating/.test(t)),
    `Events lists only the latest boredom phase, Recent Activity both: ${JSON.stringify(phases)}`);
  await shot('parked-story-latest-phase');
  // Read it now, so no count clears under section 4's layout comparison.
  await waitFor(`document.querySelector('[data-testid="events-list"]').dataset.unread === '0'`, 'phase 3 read', 4000);
  await debugMenu();
  await click('#check-replay');
  check(/^Replay ok/.test(await noticeText()), `replay check after the readability steps: ${await noticeText()}`);
}

/** Section 1, from a fresh load: the new game's first checks, then the opening, the forced
 * retirement and run 2. From its end on, story modals are clicked through (storyAuto). */
async function freshGame() {
  await load(url);
  check((await day()) < 5 && (await run()) === 1, `a fresh game loads: day ${await day()}, run ${await run()}`);
  check(!(await exists('[data-testid="cheats"]')), 'cheats are off by default');
  check(!(await navVisible('retirement')) && !(await navVisible('launch_pads')), 'gated nav tiles start hidden');
  check(/1\.1 — Online/.test(await evaluate(`document.querySelector('[data-testid="quest"]')?.textContent ?? ''`)), 'the first quest shows');
  const cards = await evaluate(`[...document.querySelectorAll('.card.building')].map((c) => c.dataset.id).sort().join(',')`);
  check(cards === 'data_center,solar_panel', `a new game offers only Solar Panel and Data Center: ${cards}`);

  // A new game tells only its first quest, in a modal that holds the clock.
  {
    const d0 = await day();
    await sleep(400);
    const held = (await day()) === d0;
    await shot('story-modal-intro');
    // 1.1's picture (earthrise) above the modal's paragraph and at the left
    // column's foot; the split-glow title; no Earth globe over the middle column.
    const pic = await evaluate(`(() => {
      const m = document.querySelector('[data-testid="story-modal"] [data-testid="quest-pic-modal"]');
      const t = document.querySelector('[data-testid="story-modal"] .story-text');
      const l = document.querySelector('.col-left > .quest-scene [data-testid="quest-pic-left"]');
      return {
        modal: m?.dataset.picture ?? null,
        above: !!m && !!t && (m.compareDocumentPosition(t) & Node.DOCUMENT_POSITION_FOLLOWING) > 0,
        loaded: !!m && m.complete && m.naturalWidth > 0,
        left: l?.dataset.picture ?? null,
        title: document.querySelector('h1.title')?.className ?? '',
        earth: !!document.querySelector('.col-mid .earth, .col-mid svg[viewBox="0 0 200 200"]'),
      };
    })()`);
    check(pic.modal === 'earthrise' && pic.above && pic.loaded, `1.1's modal shows its picture above its paragraph: ${JSON.stringify(pic)}`);
    check(pic.left === 'earthrise', `the left column's foot shows 1.1's picture: ${pic.left}`);
    check(/\bsplit-glow\b/.test(pic.title) && !pic.earth, `the split-glow title and no Earth globe: ${pic.title} earth ${pic.earth}`);
    const pages = await readStory();
    check(
      held && pages.length >= 1 && pages.every((p) => p.quest === 'qpower_boot_sequence' && p.kind === 'story' && p.paused) && !!pages.at(-1).task && pages.at(-1).button === 'Begin',
      `a new game opens only 1.1's story modal, paused, its task last: ${JSON.stringify(pages)}`,
    );
    // On 1.1, Story's active beat carries 1.1's thumbnail.
    await panel('story');
    const thumb = await evaluate(`document.querySelector('[data-testid="story-beat-active"] [data-testid="quest-pic-thumb"]')?.dataset.picture ?? null`);
    check(thumb === 'earthrise', `on 1.1 the active beat shows its thumbnail: ${thumb}`);
  }
  // Programs from day 1 (091): the right column's program, Add to Program and the loadout
  // buttons; no Run anywhere; the Programs tile (the full editor) only once Programming.
  await panel('commands');
  const day1 = await evaluate(`({
    tile: !!document.querySelector('.nav-tile[data-nav="programs"]'),
    program: !!document.querySelector('[data-testid="programs"]'),
    add: !!document.querySelector('[data-testid="add-cloud_compute"]'),
    loadout: !!document.querySelector('[data-testid="loadout-save"]') && !!document.querySelector('[data-testid="loadout-load"]'),
    run: [...document.querySelectorAll('button')].some((b) => /^run-/.test(b.dataset.testid ?? '') || b.textContent.trim() === 'Run'),
  })`);
  check(!day1.tile && day1.program && day1.add && day1.loadout && !day1.run, `a new game: program, Add and loadouts, no Run, no Programs tile: ${JSON.stringify(day1)}`);
  await shot('day1-program');
  // Two quick clicks (no render between them) both land.
  await evaluate(`['add-cloud_compute', 'add-idle'].forEach((t) => document.querySelector('[data-testid="' + t + '"]').click())`);
  await sleep(100);
  const two = await evaluate(`[...document.querySelectorAll('.lines li')].map((li) => li.dataset.command).join(',')`);
  check(two === 'cloud_compute,idle', `two quick adds both land: ${two}`);

  // ---- 1. a fresh game to the first shipment, a forced retirement and run 2, through the normal UI ----
  const reached = await playOpening();
  check(reached.at(-1) === OPENING_END, `played 1.1 to 1.3 by day ${await day()}: ${reached.join(' -> ')}`);
  // Finished quests live in the Story view (the right panel has no Completed section).
  await panel('story');
  const done = await evaluate(`[...document.querySelectorAll('[data-testid="story-beat-done"]')].map((b) => b.dataset.beat)`);
  check(!(await exists('[data-testid="events-completed"]')), 'no Completed section in the right panel');
  check(
    ['qpower_boot_sequence', 'qfirst_shipment', 'qfull_load'].every((x) => done.includes(x)),
    `1.1 Online, 1.2 First Shipment and 1.3 Full Load complete: ${done.join(', ')}`,
  );
  check(await evaluate(`!!document.querySelector('.activity li[data-kind="shipment"]')`), 'the shipment is in Recent Activity');
  // 096: its Shipments checkbox hides shipments and shows them again.
  const shipBox = '[data-testid="activity-filter"] input[data-group="shipments"]';
  const shipLines = () => evaluate(`document.querySelectorAll('.activity li[data-kind="shipment"]').length`);
  await click(shipBox);
  const hiddenShips = await shipLines();
  await click(shipBox);
  check(hiddenShips === 0 && (await shipLines()) > 0, `unchecking Shipments hides them from Recent Activity, checking shows them (${hiddenShips} while unchecked)`);
  check(await exists('tr[data-res="propellant"]'), 'Propellant shows in Resources once granted');
  check(await navVisible('launch_pads'), 'the Launch Pads tile shows');
  // Voluntary retirement opens only with the first retirement: run 1 ends at the
  // boredom cap, played on at 10x (Act 1's fastest) with no cheat.
  check(!(await navVisible('retirement')), 'no Retirement tile in run 1 (it opens at the first retirement)');
  // From here on story modals are read as they open (the section is about the retirement).
  storyAuto = true;
  await autoStory();
  await panel('buildings');
  await shot('run1-buildings');
  await panel('launch_pads');
  await shot('run1-launch-pads');
  await speed(10);
  await waitFor(`!!document.querySelector('[data-testid="summary"]')`, 'the forced retirement at the boredom cap', 300_000);
  const summaryText = await evaluate(`document.querySelector('[data-testid="summary"]').textContent`);
  const runDays = (summaryText.match(/ended on day ([\d,.]+)/) ?? [, '?'])[1];
  check(/boredom filled.*Years passed/.test(summaryText), `boredom at its cap retires run 1, the summary saying why and that years passed (day ${runDays})`);
  // The ended run's last day stays behind the summary until Continue (104).
  const paused1 = await day();
  await sleep(600);
  check((await day()) === paused1 && (await exists('[data-testid="summary"]')), `the run summary pauses the game (day ${paused1} -> ${await day()})`);
  await shot('run-summary');
  // A real double-click on Continue over the Buildings panel: its second click lands on
  // whatever the summary covered, and must not reach a card. The cards behind the summary
  // are the ended run's (104), so the save's Buy actions count, not the cards.
  await click('.nav-tile[data-nav="buildings"]');
  {
    const owned = `JSON.parse(localStorage.getItem('hh.save')).actions.filter((a) => a.action.type === 'buy').length`;
    const o0 = await evaluate(owned);
    const p = await evaluate(`(() => { const r = document.querySelector('[data-testid="continue"]').getBoundingClientRect(); return { x: r.left + r.width / 2, y: r.top + r.height / 2 }; })()`);
    for (const clickCount of [1, 2])
      for (const type of ['mousePressed', 'mouseReleased']) await send('Input.dispatchMouseEvent', { type, x: p.x, y: p.y, button: 'left', clickCount });
    await sleep(200);
    const hint = await evaluate(`[...document.querySelectorAll('.buy-hint.short')].map((h) => h.textContent).join(' ')`);
    check((await evaluate(owned)) === o0 && !hint, `a double-click on Continue buys nothing under it (${o0} -> ${await evaluate(owned)} Buy actions${hint ? `, "${hint}"` : ''})`);
  }
  await sleep(1500);
  check((await run()) === 2 && (await day()) > 0 && !(await exists('[data-testid="summary"]')), `run 2 plays on (day ${await day()}; run 1 lasted ${runDays} days)`);
  check(await exists('#speeds button[data-speed="1"][aria-pressed="true"]'), 'Continue plays the new run at 1x');
  await speed(0);
  check(await navVisible('launch_pads'), 'known nav tiles stay visible in run 2');
  check(await navVisible('retirement'), 'the first retirement opens the Retirement tile in run 2');
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
}

// ---- 2. every panel, reaching the gated ones from scenario late_unlocks ----
/** Section 2. Opens the cheats (toggled off and on again) and leaves the debug menu open. */
async function everyPanel() {
  // Projects open with 1.5 A Voice's start and Ideologies with Geopolitical Intelligence
  // (A Voice too): the scenario opens them (and Retirement) instead of playing that far.
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
  // Only fundable projects show (096: no locked cards); a funded one shows its days to done
  // (the engine's text).
  const shownProjects = await evaluate(`[...document.querySelectorAll('[data-panel="projects"] .card.project')].map((c) => (c.classList.contains('locked') || /Locked:/.test(c.textContent) ? '!' : '') + c.dataset.id)`);
  check(!shownProjects.includes('predictive_maintenance') && shownProjects.every((id) => !id.startsWith('!')), `Projects shows no locked card: ${shownProjects.join(', ')}`);
  // Before the swarm's reveal no player-facing text names a timeline (096).
  check(!/timeline/i.test(await evaluate(`document.body.innerText`)), 'nothing on the page names a timeline before the reveal');
  await evaluate(`[...document.querySelectorAll('.card.project[data-id="market_cornering"] button[title^="Fund at the most"]')].forEach((b) => b.click())`);
  await waitFor(`/Done in \\d+ days/.test(document.querySelector('.card.project[data-id="market_cornering"]')?.textContent ?? '')`, 'days to done', 5000);
  const fundedText = await evaluate(`document.querySelector('.card.project[data-id="market_cornering"]').textContent`);
  check(/Done in 17 days/.test(fundedText), `a funded project shows its days to done: ${(fundedText.match(/Done in \d+ days?/) ?? ['none'])[0]}`);
  await shot('projects-funded');
  const tiles = await evaluate(`[...document.querySelectorAll('.nav-tile')].map((t) => t.dataset.nav)`);
  // The Programs tile opens a takeover (section 10), not a panel.
  for (const id of tiles.filter((t) => t !== 'programs')) {
    await panel(id);
    const text = await evaluate(`document.querySelector('[data-panel="${id}"]').textContent.length`);
    check(text > 20, `panel ${id} opens (${text} chars)`);
    await shot(`panel-${id}`);
  }
  // Act 1's tiles: every nav row but those whose panel only a DRAFT quest (Acts 2 and 3) opens.
  const content = (f) => JSON.parse(readFileSync(resolve(here, `../../content/${f}`), 'utf8'));
  const draftPanels = new Set(
    content('events.json')
      .filter((e) => (e.goal ?? '').startsWith('DRAFT'))
      .flatMap((e) => (e.unlocks ?? []).filter((u) => u.type === 'enable_nav_panel').map((u) => u.panel)),
  );
  const act1 = content('nav.json').map((n) => n.id).filter((id) => !draftPanels.has(id));
  const missing = act1.filter((id) => !tiles.includes(id));
  check(!missing.length, `all ${act1.length} Act 1 nav tiles (content/nav.json) reachable: ${tiles.join(', ')}${missing.length ? `; missing ${missing.join(', ')}` : ''}`);
  check(!(await evaluate(`document.querySelector('.status').textContent`)).includes('CHEATED'), 'the bottom bar shows no CHEATED tag');
  await debugMenu();
  check((await evaluate(`document.getElementById('debug-info').textContent`)).includes('CHEATED'), 'a cheated game is still marked (debug info)');
  await setValue('#cheat-days', '30');
  const d0 = await day();
  await click('#cheat-jump');
  await checkAct((await day()) === d0 + 30, `jump ahead 30 days (${d0} -> ${await day()})`);
  // The cheat building (a Buildings card only while Cheats are on) and Complete all research.
  await panel('buildings');
  const cheatOwned = () => evaluate(`Number(document.querySelector('[data-testid="buy-cheat_building"]')?.dataset.owned ?? -1)`);
  const credits = () => evaluate(`Number(document.querySelector('tr.res-row[data-res="credits"]').dataset.amount)`);
  const c0 = await credits();
  const cheatText = await evaluate(`document.querySelector('[data-testid="buy-cheat_building"] .effects').textContent`);
  check(cheatText.includes('+1,000 / day credits') || cheatText.includes('+1000 / day credits'), `the Cheat Fountain reads +1000 credits a day: ${cheatText}`);
  await click('[data-testid="buy-cheat_building"]');
  await checkAct((await cheatOwned()) === 1 && c0 - (await credits()) === 1, `the cheat building buys for 1 credit (${c0} -> ${await credits()})`);
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
}

// ---- 3. speeds, saves and scenarios ----
/** Section 3, from the debug menu open with cheats on. */
/** Each speed's days per second, `[speed, seconds]` each; leaves the game paused. */
async function speedRates(list) {
  for (const [s, secs] of list) {
    await speed(s);
    const a = await day();
    const t0 = Date.now();
    await sleep(secs * 1000);
    const rate = ((await day()) - a) / ((Date.now() - t0) / 1000);
    check(rate > s * 0.6 && rate < s * 1.4, `${s}x: ${rate.toFixed(1)} days/s`);
  }
  await speed(0);
}

async function speedsAndSaves() {
  await setValue('#scenario', 'rich');
  await click('#start-scenario');
  check((await day()) === 0 && /Started scenario rich/.test(await noticeText()), 'debug menu starts scenario rich');
  // Act 1's speeds (Matt, 2026-11-26): 50x and 200x wait for Accelerated Scheduling.
  const offered = () => evaluate(`[...document.querySelectorAll('#speeds button')].map((b) => b.dataset.speed).join(',')`);
  const slow = await offered();
  check(slow === '0,1,3,10', `without Accelerated Scheduling the speeds are pause, 1x, 3x and 10x: ${slow}`);
  await programOrder(false);
  await speedRates([[1, 3], [3, 2], [10, 2]]);
  const paused = await day();
  await sleep(1200);
  check((await day()) === paused, 'pause holds the day');
  await panel('buildings');
  const owned = () => evaluate(`document.querySelector('.card.building[data-id="solar_panel"]').dataset.owned`);
  const before = await owned();
  // The card is the Buy button: a click anywhere on its text buys one.
  await click('.card.building[data-id="solar_panel"].can-buy .card-text');
  const after = await owned();
  await checkAct(Number(after) === Number(before) + 1,`a click on a green card buys a Solar Panel: ${before} -> ${after}`);
  // Its own controls keep their jobs and buy nothing.
  await click('.card.building[data-id="solar_panel"] button[title="Run one more"]');
  await click('.card.building[data-id="solar_panel"] button[title="Run one fewer"]');
  check((await owned()) === after, `the card's −/+ buttons buy nothing (${await owned()})`);
  await click('.card.building[data-id="solar_panel"] button.sell[title^="Sell one"]');
  await checkAct(Number(await owned()) === Number(after) - 1,`the card's Sell 1 sells one and buys none (${after} -> ${await owned()})`);
  // A real mouse press on the card, then space: the pause key, not a second buy (a press
  // must not leave the card focused).
  {
    const p = await evaluate(`(() => { const e = document.querySelector('.card.building[data-id="solar_panel"] .card-text'); e.scrollIntoView({ block: 'center' }); const r = e.getBoundingClientRect(); return { x: r.left + r.width / 2, y: r.top + r.height / 2 }; })()`);
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
  const gray =await evaluate(`document.querySelector('.card.building.cannot-buy')?.dataset.id ?? null`);
  if (gray) {
    const grayOwned = () => evaluate(`document.querySelector('.card.building[data-id="${gray}"]').dataset.owned`);
    const g0 = await grayOwned();
    await click(`[data-testid="buy-${gray}"]`);
    const hint = await evaluate(`document.querySelector('.card.building[data-id="${gray}"] .buy-hint').textContent`);
    check((await grayOwned()) === g0 && hint.length > 0, `a gray card buys nothing and says why (${gray}: "${hint}")`);
  }
  const savedDay = await day();
  const savedSpeed = await evaluate(`document.querySelector('#speeds button[aria-pressed="true"]')?.dataset.speed`);
  await load(url);
  console.log(`     before the reload: speed ${savedSpeed}; after: ${await evaluate(`document.querySelector('#speeds button[aria-pressed="true"]')?.dataset.speed`)}, day ${await day()}`);
  await panel('buildings');
  check((await day()) === savedDay && (await owned()) === kept, `reload keeps the save (day ${savedDay} -> ${await day()}, ${kept} -> ${await owned()} panels)`);
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
  // Above 10x, in scenario everything (its career holds Accelerated Scheduling's flag).
  await debugMenu();
  await speed(0);
  await setValue('#scenario', 'everything');
  await click('#start-scenario');
  const fastOffered = await offered();
  check(fastOffered === '0,1,3,10,50,200', `with Accelerated Scheduling's flag 50x and 200x are offered: ${fastOffered}`);
  await programOrder(true);
  await speedRates([[50, 1], [200, 1]]);
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
}

// ---- 5. readability: capped income, affordable headers, pad readiness, quest pin ----
async function readabilitySection() {
  await debugMenu();
  await setValue('#scenario', 'at_cap');
  await click('#start-scenario');
  await readability(grant);
}

// ---- 4. the theme: light by default; the Options toggle switches it, moving nothing ----
/** Section 4. Dismisses a notice first (section 5 leaves the replay check's up). */
async function themeSection() {
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
}

/** The sections in the order they run: 6 to 11 before 5, which leaves a notice for 4. */
const SECTIONS = [
  [1, freshGame],
  [2, everyPanel],
  [3, speedsAndSaves],
  // ---- 6. program instructions (before 5): if, build, buy land ----
  [6, () => programLogic(grant)],
  // ---- 7. the program text editor (before 5) ----
  [7, textEditor],
  // ---- 8. else (before 5): an if/else from the text editor ----
  [8, programElse],
  // ---- 9. the All instructions panel (before 5) ----
  [9, instructionPanel],
  // ---- 10. the Programs view's Blocks mode (before 5) ----
  [10, programsBlocks],
  // ---- 11. Story's quest timeline at 1.5 and the Achievements tile (before 5) ----
  [11, storyView],
  [5, readabilitySection],
  [4, themeSection],
];

/** With HH_ONLY, a skipped section's leftovers that later ones need, set up directly. */
async function skipped(n) {
  // 1: a loaded game, story modals clicked through from then on.
  if (n === 1) {
    storyAuto = true;
    await load(url);
  }
  // 2: the debug menu open with its cheats on (3 starts a scenario from it; 5 to 11 cheat).
  if (n === 2) await cheats();
  // 5: a notice up (the replay check's), which 4 dismisses.
  if (n === 5) {
    await debugMenu();
    await click('#check-replay');
  }
}

try {
  if (ONLY) console.log(`HH_ONLY: sections ${[...ONLY].join(', ')}`);
  for (const [i, [n, section]] of SECTIONS.entries()) {
    if (!ONLY || ONLY.has(n)) await section();
    else if (SECTIONS.slice(i + 1).some(([m]) => ONLY.has(m))) await skipped(n);
  }
  check(pageErrors.length === 0, `no page errors${pageErrors.length ? `: ${pageErrors.join(' | ')}` : ''}`);
} catch (e) {
  failures.push(String(e));
  console.error(e);
  if (pageErrors.length) console.error(`page errors: ${pageErrors.join(' | ')}`);
  console.error(`#notice: ${await noticeText().catch(() => '')}`);
  await shot('failure');
} finally {
  b.close();
}
if (failures.length) {
  console.error(`e2e FAILED: ${failures.length}`);
  process.exit(1);
}
console.log('e2e ok');
