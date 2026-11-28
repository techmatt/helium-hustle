// 134's check in the running game (vite preview at HH_URL): the fixes a player sees. The
// status bar's date whole, the run summary's date the same as the status bar behind it (a
// Retire press and a forced retirement), Foundation Grant's reward text, the Sell hovers, a
// double-click on the last Sell 1 buying nothing, the library's save name per tab,
// the Recreation Dome's relief to two places, the Research and Stats wording, and
// Electrochemistry's card at 8 Solar Panels. From the repo root:
//   HH_URL=http://localhost:<port>/ node experiments/results/134_bug_hunt/check.mjs out/134_bug_hunt/shots
import { mkdirSync, writeFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { launch, driver, capture, sleep } from '../../../web/scripts/cdp.mjs';

const url = process.env.HH_URL;
const shots = resolve(process.argv[2] ?? 'out/134_bug_hunt/shots');
mkdirSync(shots, { recursive: true });
const b = await launch({ width: 1280, height: 900, profileRoot: resolve('out/134_bug_hunt/profile') });
const { evaluate } = b;
const d = driver(b, { url, settle: 700 });
const { exists, click, setValue, speed, panel, debugMenu, cheats, grant, waitFor } = d;
let fails = 0;
const facts = {};
const check = (ok, what) => {
  console.log(`${ok ? 'ok  ' : 'FAIL'} ${what}`);
  if (!ok) fails++;
};
const shot = (n) => capture(b, resolve(shots, `134_${n}.png`));
const scenario = async (name) => {
  await debugMenu();
  await speed(0);
  await setValue('#scenario', name);
  await click('#start-scenario');
  await sleep(300);
};
const dismiss = async () => {
  for (let i = 0; i < 60 && (await exists('[data-testid="story-next"]')); i++) {
    await click('[data-testid="story-next"]');
    await sleep(120);
  }
};
const dayText = () => evaluate(`document.getElementById('day')?.textContent ?? ''`);
const dayWhole = () => evaluate(`(() => { const e = document.getElementById('day'); return e ? e.scrollWidth <= e.clientWidth : false; })()`);
const summaryDate = () => evaluate(`(document.querySelector('[data-testid="summary"] h2')?.textContent ?? '').replace(/^.*ended on /, '')`);

try {
  await d.load();
  await dismiss();

  // ---- everything: projects, buildings, research and Stats text ----
  await scenario('everything');
  await dismiss();
  check(await dayWhole(), `the status bar's date is whole: ${await dayText()}`);
  await panel('projects');
  await sleep(200);
  // Completed ones too (Hide completed off).
  await evaluate(`(() => { const c = [...document.querySelectorAll('.panel[data-panel="projects"] input[type="checkbox"]')][0]; if (c && c.checked) c.click(); })()`);
  await sleep(200);
  const grantText = await evaluate(`[...document.querySelectorAll('.card.project')].map((c) => c.textContent).find((t) => /Foundation Grant/.test(t)) ?? ''`);
  const reward = (grantText.match(/Every run starts with[^]*?free[^A-Z]*?(?:[A-Z][a-z]+ ?)+(?:×\d)?(?:, [^,]+?)*?(?=Credits|Not funded|Done|$)/) ?? [''])[0];
  facts.foundationGrant = reward;
  check(/these free: Regolith Excavator ×2, Smelter, Solar Panel ×2/.test(grantText), `Foundation Grant's reward: ${reward || grantText.slice(0, 200)}`);
  await shot('projects');
  await panel('buildings');
  await sleep(200);
  const sell = await evaluate(`[...document.querySelectorAll('button.sell')].map((x) => x.title)`);
  facts.sellTitles = [...new Set(sell)];
  check(sell.length > 0 && sell.every((t) => /half (its|their) average price paid/.test(t)), `Sell hovers name the refund: ${facts.sellTitles.join(' | ')}`);
  const relief = await evaluate(`[...document.querySelectorAll('[data-testid="boredom-relief"]')].map((x) => x.textContent.trim())`);
  facts.relief = relief;
  check(relief.length === 0 || relief.every((t) => /−\d+(\.\d\d?)? Boredom\/day/.test(t)), `Dome relief: ${relief.join(' | ') || '(no Dome owned)'}`);
  // ---- a double-click on the last copy's Sell 1: sold, not bought back ----
  await cheats();
  await grant('credits', '100000');
  await panel('buildings');
  await sleep(200);
  // One copy of something: a card owned once, else one bought now.
  let one = await evaluate(`[...document.querySelectorAll('.card.building')].find((c) => c.dataset.owned === '1' && c.querySelector('button.sell'))?.dataset.id ?? null`);
  if (!one) {
    // The fewest owned, sold down to one with single clicks.
    one = await evaluate(`[...document.querySelectorAll('.card.building')].filter((c) => Number(c.dataset.owned) > 1 && c.querySelector('button.sell') && c.dataset.id !== 'cheat_building').sort((a, z) => a.dataset.owned - z.dataset.owned)[0]?.dataset.id ?? null`);
    for (let i = 0; one && i < 40 && (await evaluate(`document.querySelector('.card.building[data-id="${one}"]')?.dataset.owned`)) !== '1'; i++) {
      await evaluate(`document.querySelector('.card.building[data-id="${one}"] button.sell')?.click()`);
      await sleep(80);
    }
    await sleep(700);
  }
  if (one) {
    const p = await evaluate(`(() => { const s = document.querySelector('.card.building[data-id="${one}"] button.sell'); s.scrollIntoView({ block: 'center' }); const r = s.getBoundingClientRect(); return { x: r.left + r.width / 2, y: r.top + r.height / 2 }; })()`);
    await sleep(200);
    const ownedAll = `Object.fromEntries([...document.querySelectorAll('.card.building')].map((c) => [c.dataset.id, c.dataset.owned]))`;
    const before = await evaluate(ownedAll);
    await b.send('Input.dispatchMouseEvent', { type: 'mouseMoved', x: p.x, y: p.y });
    const counts = [];
    for (const n of [1, 2]) {
      await b.send('Input.dispatchMouseEvent', { type: 'mousePressed', x: p.x, y: p.y, button: 'left', clickCount: n });
      await b.send('Input.dispatchMouseEvent', { type: 'mouseReleased', x: p.x, y: p.y, button: 'left', clickCount: n });
      counts.push(await evaluate(`document.querySelector('.card.building[data-id="${one}"]')?.dataset.owned ?? 'gone'`));
    }
    await sleep(300);
    const owned = await evaluate(`document.querySelector('.card.building[data-id="${one}"]')?.dataset.owned ?? 'gone'`);
    facts.sellDoubleClick = { building: one, counts, owned };
    console.log(`     owned after each click: ${counts.join(', ')}`);
    // A card that goes with its last copy lets the next one slide under the pointer.
    const after = await evaluate(ownedAll);
    const bought = Object.keys(after).filter((id) => id !== one && Number(after[id]) > Number(before[id] ?? 0));
    facts.sellDoubleClick.boughtElsewhere = bought;
    check(bought.length === 0, `nothing else bought by the second click: ${bought.join(', ') || 'none'}`);
    check(owned === '0' || owned === 'gone', `a double-click on ${one}'s last Sell 1 leaves none (owned ${owned})`);
  } else check(false, 'a building with one copy to sell');

  // ---- the library's save name follows the program tab ----
  await evaluate(`document.querySelector('.prog-tabs button[data-program="0"]')?.click()`);
  await sleep(200);
  // A line in Program A (Save needs one): the first command's Add to Program.
  await panel('commands');
  await sleep(200);
  await evaluate(`[...document.querySelectorAll('.panel[data-panel="commands"] button')].find((x) => /Add to Program/.test(x.textContent))?.click()`);
  await sleep(300);
  await evaluate(`document.querySelector('[data-testid="loadout-save"]')?.click()`);
  await sleep(200);
  const nameA = await evaluate(`document.querySelector('[data-testid="loadout-name"]')?.value ?? null`);
  await evaluate(`document.querySelector('.prog-tabs button[data-program="1"]')?.click()`);
  await sleep(300);
  const nameB = await evaluate(`document.querySelector('[data-testid="loadout-name"]')?.value ?? null`);
  facts.libraryNames = { nameA, nameB };
  check(/^Program A,/.test(nameA ?? '') && (nameB === null || /^Program B,/.test(nameB)), `the save name follows the tab: ${nameA} -> ${nameB}`);
  await evaluate(`document.querySelector('.prog-tabs button[data-program="0"]')?.click()`);

  // ---- a double-click on a research card buys that one, not the card sliding up ----
  await cheats();
  await grant('science', '50000');
  await panel('research');
  await sleep(300);
  const researched = `[...document.querySelectorAll('.card.research')].filter((c) => c.classList.contains('done')).length + ':' + [...document.querySelectorAll('.card.research:not(.done)')].map((c) => c.dataset.id).join(',')`;
  const rBefore = await evaluate(researched);
  const rp = await evaluate(`(() => { const c = document.querySelector('.card.research.can-buy:not(.done)'); if (!c) return null; c.scrollIntoView({ block: 'center' }); const r = c.getBoundingClientRect(); return { id: c.dataset.id, x: r.left + r.width / 2, y: r.top + r.height / 2 }; })()`);
  if (rp) {
    await sleep(200);
    await b.send('Input.dispatchMouseEvent', { type: 'mouseMoved', x: rp.x, y: rp.y });
    for (const n of [1, 2]) {
      await b.send('Input.dispatchMouseEvent', { type: 'mousePressed', x: rp.x, y: rp.y, button: 'left', clickCount: n });
      await b.send('Input.dispatchMouseEvent', { type: 'mouseReleased', x: rp.x, y: rp.y, button: 'left', clickCount: n });
    }
    await sleep(400);
    const rAfter = await evaluate(researched);
    const left = (s) => new Set(s.split(':')[1].split(',').filter(Boolean));
    const gone = [...left(rBefore)].filter((id) => !left(rAfter).has(id));
    facts.researchDoubleClick = { clicked: rp.id, bought: gone };
    check(gone.length === 1 && gone[0] === rp.id, `a double-click on ${rp.id} buys it alone: ${gone.join(', ')}`);
  } else check(false, 'an affordable research card');

  await panel('research');
  await sleep(200);
  const head = await evaluate(`document.querySelector('.panel[data-panel="research"] p.muted')?.textContent ?? ''`);
  check(/marked as kept/.test(head), `Research's header: ${head.trim()}`);
  await panel('stats');
  await sleep(200);
  const stats = await evaluate(`document.querySelector('.panel[data-panel="stats"]')?.textContent ?? ''`);
  check(/Base boredom growth \(before buildings. relief and commands\)/.test(stats), 'Stats names buildings’ relief');

  // ---- a Retire press: the summary's date is the status bar's behind it ----
  await speed(3);
  await waitFor(`/day ([5-9]|\\d\\d)/.test(document.getElementById('day')?.textContent ?? '')`, 'a few days played', 20000);
  await speed(0);
  await dismiss();
  await panel('retirement');
  await sleep(200);
  const before = await dayText();
  await evaluate(`[...document.querySelectorAll('.panel[data-panel="retirement"] button')].find((x) => /^Retire/.test(x.textContent))?.click()`);
  await sleep(300);
  await evaluate(`[...document.querySelectorAll('.panel[data-panel="retirement"] button')].find((x) => /Yes|Confirm|Retire now/.test(x.textContent))?.click()`);
  await waitFor(`!!document.querySelector('[data-testid="summary"]')`, 'the run summary', 8000);
  const pressed = { summary: await summaryDate(), bar: await dayText(), before };
  facts.retirePress = pressed;
  check(pressed.summary === pressed.bar, `a Retire press: summary ${pressed.summary}, status bar ${pressed.bar}`);
  await shot('summary-pressed');
  await sleep(700);
  await click('[data-testid="continue"]');
  await waitFor(`!document.querySelector('[data-testid="summary"]')`, 'the summary closed', 5000);
  await dismiss();

  // ---- a forced retirement (boredom to its cap): the same ----
  await speed(3);
  await waitFor(`/day ([5-9]|\\d\\d)/.test(document.getElementById('day')?.textContent ?? '')`, 'a few days of the next run', 30000);
  await speed(0);
  await dismiss();
  await cheats();
  // At the cap the Resources row shows ("229 / 1,200"), as qa sets it.
  const cap = await evaluate(`Number((document.querySelector('[data-res="boredom"]')?.closest('tr')?.textContent.match(/\\/\\s*([\\d,]+)/)?.[1] ?? '1000').replace(/,/g, ''))`);
  await setValue('#cheat-boredom', String(cap));
  await click('#cheat-set-boredom');
  await panel('buildings');
  await speed(1);
  await waitFor(`!!document.querySelector('[data-testid="summary"]')`, 'the forced retirement', 20000);
  const forced = { summary: await summaryDate(), bar: await dayText() };
  facts.forced = forced;
  check(forced.summary === forced.bar, `a forced retirement: summary ${forced.summary}, status bar ${forced.bar}`);
  await shot('summary-forced');

  // ---- Electrochemistry at 8 Solar Panels (a fresh `everything`: Research open) ----
  await sleep(700);
  await click('[data-testid="continue"]');
  await sleep(700);
  await dismiss();
  await scenario('everything');
  await dismiss();
  await cheats();
  await grant('credits', '100000');
  await grant('science', '2000');
  await panel('buildings');
  for (let i = 0; i < 8; i++) {
    await evaluate(`document.querySelector('[data-testid="buy-solar_panel"] button, .card.building[data-id="solar_panel"] button.buy')?.click()`);
    await sleep(150);
  }
  const solar = await evaluate(`(document.querySelector('.card.building[data-id="solar_panel"]')?.textContent.match(/Owned:? ?(\\d+)|(\\d+) owned/) ?? []).filter(Boolean)[1] ?? '?'`);
  await panel('research');
  await sleep(300);
  const echem = await evaluate(`[...document.querySelectorAll('.card.research')].find((c) => /Electrochemistry/.test(c.textContent))?.textContent ?? null`);
  facts.electrochemistry = { solar, card: echem?.slice(0, 160) ?? null };
  check(!!echem, `Electrochemistry shows at ${solar} Solar Panels`);
  await shot('research-electrochemistry');
} catch (e) {
  console.log(`FAIL ${e.stack ?? e}`);
  fails++;
} finally {
  writeFileSync(resolve(shots, 'facts.json'), JSON.stringify(facts, null, 2));
  await b.close();
}
console.log(fails ? `${fails} failed` : 'all ok');
process.exit(fails ? 1 : 0);
