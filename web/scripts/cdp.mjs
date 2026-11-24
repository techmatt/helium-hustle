// The DevTools-protocol harness shared by e2e.mjs, qa.mjs and
// experiments/results/027_art_in_game/shots.mjs: headless Edge or Chrome, no packages.
//   pageUrl(script)  HH_URL, required (a `vite preview` of a static build; exits if unset)
//   launch(opts)     a browser with one page: { send, evaluate, errors, close }
//   driver(b, opts)  page helpers on it: waitFor, load, click, panel, debugMenu, grant...
//   HH_BROWSER       a browser off the usual paths
import { spawn, spawnSync } from 'node:child_process';
import { existsSync, mkdirSync, rmSync, writeFileSync } from 'node:fs';
import { resolve } from 'node:path';

export const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
/** A value quoted for an evaluated expression. */
export const q = (s) => JSON.stringify(s);
/** "Loading…" until the engine has rendered once. */
export const READY = `/^Day \\d/.test(document.getElementById('day')?.textContent ?? '') && !!document.querySelector('.nav-tile')`;

/** The page under test: HH_URL, with no default (the dev server's watcher reloads the
 * page on any crates/ edit, so checks run against `vite preview`). */
export function pageUrl(script, run = `npm run ${script}`) {
  const url = process.env.HH_URL;
  if (url) return url;
  console.error(`${script}: set HH_URL to a served static build, e.g.
  npm run build && npx vite preview --port 55267 --strictPort
  HH_URL=http://localhost:55267/ ${run}`);
  process.exit(2);
}

const BROWSERS = [
  process.env.HH_BROWSER,
  'C:/Program Files (x86)/Microsoft/Edge/Application/msedge.exe',
  'C:/Program Files/Google/Chrome/Application/chrome.exe',
  '/usr/bin/google-chrome',
  '/Applications/Google Chrome.app/Contents/MacOS/Google Chrome',
].filter(Boolean);

/**
 * A headless browser with one page, driven over CDP.
 *   width, height  the window (and, with `emulate`, the page's metrics override)
 *   profileRoot    a fresh profile goes in <profileRoot>/<port> (wiped first): Windows can
 *                  hold a just-closed one a while. Made absolute: given a relative one,
 *                  Edge hands the URL to a running Edge and exits.
 *   portBase       debugging port: portBase + 0..499
 *   args           extra browser flags
 *   log            also collect Log.entryAdded errors (failed loads and the like)
 * Every call has a 30 s deadline: a hung page fails the caller instead of hanging it.
 * `errors` collects uncaught page errors and console.error calls.
 */
export async function launch({ width, height, emulate = false, profileRoot, portBase = 9300, args = [], log = false }) {
  const exe = BROWSERS.find((b) => existsSync(b));
  if (!exe) throw new Error('no browser found: set HH_BROWSER');
  const port = portBase + Math.floor(Math.random() * 500);
  const profile = resolve(profileRoot, String(port));
  try {
    rmSync(profile, { recursive: true, force: true });
  } catch {}
  mkdirSync(profile, { recursive: true });
  const proc = spawn(exe, [
    '--headless=new', `--remote-debugging-port=${port}`, `--user-data-dir=${profile}`,
    // --enable-automation sets navigator.webdriver: the page files its play logs as a bot's.
    '--enable-automation', `--window-size=${width},${height}`, '--no-first-run', '--no-default-browser-check',
    ...args, 'about:blank',
  ], { stdio: 'ignore' });
  // The browser's child processes outlive a plain kill on Windows (and hold the profile).
  const kill = () => {
    if (process.platform === 'win32') spawnSync('taskkill', ['/T', '/F', '/PID', String(proc.pid)], { stdio: 'ignore' });
    else proc.kill();
  };
  let page;
  for (let i = 0; i < 150 && !page; i++) {
    await sleep(200);
    const targets = await fetch(`http://127.0.0.1:${port}/json/list`).then((r) => r.json()).catch(() => []);
    page = targets.find((t) => t.type === 'page');
  }
  if (!page) {
    kill();
    throw new Error(`${exe} exposed no page on port ${port} within 30 s`);
  }
  const ws = new WebSocket(page.webSocketDebuggerUrl);
  await new Promise((r) => ws.addEventListener('open', r, { once: true }));
  let nextId = 1;
  const pending = new Map();
  const errors = [];
  ws.addEventListener('message', (ev) => {
    const msg = JSON.parse(ev.data);
    if (msg.id && pending.has(msg.id)) {
      pending.get(msg.id)(msg);
      pending.delete(msg.id);
    } else if (msg.method === 'Runtime.exceptionThrown') {
      errors.push(msg.params.exceptionDetails?.exception?.description ?? msg.params.exceptionDetails?.text);
    } else if (msg.method === 'Runtime.consoleAPICalled' && msg.params.type === 'error') {
      errors.push(msg.params.args.map((a) => a.value ?? a.description).join(' '));
    } else if (msg.method === 'Log.entryAdded' && msg.params.entry.level === 'error') {
      errors.push(`${msg.params.entry.source}: ${msg.params.entry.text} ${msg.params.entry.url ?? ''}`);
    }
  });
  const send = (method, params = {}) =>
    new Promise((r, reject) => {
      const id = nextId++;
      const timer = setTimeout(() => {
        pending.delete(id);
        reject(new Error(`${method}: no answer in 30 s`));
      }, 30_000);
      pending.set(id, (msg) => {
        clearTimeout(timer);
        r(msg);
      });
      ws.send(JSON.stringify({ id, method, params }));
    });
  const evaluate = async (expr) => {
    const r = await send('Runtime.evaluate', { expression: expr, awaitPromise: true, returnByValue: true });
    if (r.result?.exceptionDetails) throw new Error(`${expr.slice(0, 200)}: ${JSON.stringify(r.result.exceptionDetails).slice(0, 400)}`);
    return r.result?.result?.value;
  };
  const close = () => {
    ws.close();
    kill();
  };
  try {
    await send('Page.enable');
    await send('Runtime.enable');
    if (log) await send('Log.enable');
    if (emulate) await send('Emulation.setDeviceMetricsOverride', { width, height, deviceScaleFactor: 1, mobile: false });
  } catch (e) {
    close();
    throw e;
  }
  return { send, evaluate, errors, close };
}

/** Saves a viewport PNG (or `clip`'s box) to `file`. */
export async function capture(b, file, params = {}) {
  const r = await b.send('Page.captureScreenshot', { format: 'png', ...params });
  writeFileSync(file, Buffer.from(r.result.data, 'base64'));
}

/**
 * The page helpers every script shares, on a launched browser `b`.
 *   url     what load() opens by default
 *   poll    waitFor's interval in ms
 *   settle  load()'s pause after the first render (a dev server just after an edit may
 *           reload the page once more), before checking it is still ready
 *   readyMs load()'s wait for the first render
 *   click   a replacement click(sel, what) (qa times each one); speed, panel and the
 *           debug helpers go through it
 */
export function driver(b, { url, poll = 100, settle = 500, readyMs = 10000, click: clickWith } = {}) {
  const { evaluate, send } = b;
  const waitFor = async (expr, what, ms = 10000) => {
    for (const end = Date.now() + ms; Date.now() < end; await sleep(poll)) if (await evaluate(expr)) return;
    throw new Error(`timed out waiting for ${what}`);
  };
  const exists = (sel) => evaluate(`!!document.querySelector(${q(sel)})`);
  const plainClick = async (sel) => {
    const ok = await evaluate(`(() => { const b = document.querySelector(${q(sel)}); if (!b) return false; b.click(); return true; })()`);
    if (!ok) throw new Error(`nothing to click at ${sel}`);
  };
  const click = clickWith ?? plainClick;
  const setValue = (sel, value) =>
    evaluate(`(() => {
      const el = document.querySelector(${q(sel)});
      el.value = ${q(value)};
      el.dispatchEvent(new Event(el.tagName === 'SELECT' ? 'change' : 'input', { bubbles: true }));
    })()`);
  const load = async (u = url) => {
    // Mark the old document, so the wait below can't pass on it before the new one commits.
    await evaluate('window.__stale = true');
    await send('Page.navigate', { url: u });
    await waitFor('!window.__stale', 'the navigation');
    await waitFor(READY, 'the page', readyMs);
    await sleep(settle);
    await waitFor(READY, 'the page (settled)', readyMs);
  };
  const number = async (id) => Number((await evaluate(`document.getElementById('${id}').textContent`)).replace(/\D/g, ''));
  const panel = async (id) => {
    await click(`.nav-tile[data-nav="${id}"]`, `panel ${id}`);
    await waitFor(`!!document.querySelector('[data-panel="${id}"]')`, `the ${id} panel`, 3000);
  };
  /** Opens the debug section of Options (it starts folded). */
  const debugMenu = async () => {
    await panel('options');
    if (!(await exists('#scenario'))) await click('[data-testid="debug"] .fold-head', 'debug fold');
  };
  /** The debug menu with its cheats section open. */
  const cheats = async () => {
    await debugMenu();
    if (!(await exists('[data-testid="cheats"]'))) await click('#cheats-toggle', 'cheats');
  };
  /** Grants `amount` of `res` through the cheats (they must be open). */
  const grant = async (res, amount) => {
    await setValue('#cheat-resource', res);
    await setValue('#cheat-amount', String(amount));
    await click('#cheat-grant', 'grant');
  };
  /** A per-target command (Buy, Promote, Fund, Overclock) shares its verb's card: pick it
   * in the card's target dropdown so its Add button shows. */
  const pickCommand = (cmd) =>
    evaluate(`(() => {
      const o = document.querySelector('.panel[data-panel="commands"] select option[value="${cmd}"]');
      if (o && o.parentElement.value !== '${cmd}') {
        o.parentElement.value = '${cmd}';
        o.parentElement.dispatchEvent(new Event('change', { bubbles: true }));
      }
    })()`);
  /** Program 1 runs just `cmd`: Reset, then add it from the Commands panel. `lines` is the
   * program's commands now (nothing to do when it is just `cmd`). With `skipDisabled`, a
   * disabled Add button is left alone instead of failing the click. */
  const runOnly = async (cmd, lines, { skipDisabled = false } = {}) => {
    if (lines.length === 1 && lines[0] === cmd) return;
    await click('.prog-tabs button[data-program="0"]', 'program 1 tab');
    if (lines.length) await click('.prog-procs button[title="Clear this program"]', 'clear program');
    await panel('commands');
    await pickCommand(cmd);
    if (skipDisabled && !(await exists(`[data-testid="add-${cmd}"]:not([disabled])`))) return;
    await click(`[data-testid="add-${cmd}"]`, `add ${cmd}`);
  };
  /** Reads the main-story modal through, page by page (it pauses the game; actions and
   * speeds are refused while it is open). Returns the pages read: quest, kind, text, task,
   * whether the clock was paused on it. `onPage(page)` runs on each before its button. */
  const readStory = async (onPage = async () => {}) => {
    const pages = [];
    for (let i = 0; i < 40 && (await exists('[data-testid="story-modal"]')); i++) {
      pages.push(
        await evaluate(`(() => {
          const m = document.querySelector('[data-testid="story-modal"]');
          return { quest: m.dataset.quest, kind: m.dataset.kind, text: m.querySelector('.story-text').textContent,
            task: m.querySelector('.story-task')?.textContent ?? null, button: m.querySelector('[data-testid="story-next"]').textContent,
            paused: document.querySelector('#speeds button[data-speed="0"]')?.getAttribute('aria-pressed') === 'true' };
        })()`),
      );
      await onPage(pages.at(-1));
      await click('[data-testid="story-next"]', 'story next');
    }
    return pages;
  };
  /** Clicks through every story modal as it opens, from now until the page is left (a
   * section that isn't about the story; load() turns it off). */
  const autoStory = () =>
    evaluate(`(() => {
      if (window.__autoStory) return;
      const go = () => { for (let i = 0; i < 40; i++) { const b = document.querySelector('[data-testid="story-next"]'); if (!b) break; b.click(); } };
      window.__autoStory = new MutationObserver(go);
      window.__autoStory.observe(document.body, { childList: true, subtree: true });
      go();
    })()`);
  return {
    waitFor,
    exists,
    click,
    setValue,
    load,
    pickCommand,
    runOnly,
    readStory,
    autoStory,
    day: () => number('day'),
    run: () => number('run'),
    speed: (s) => click(`#speeds button[data-speed="${s}"]`, `speed ${s}`),
    panel,
    debugMenu,
    cheats,
    grant,
  };
}
