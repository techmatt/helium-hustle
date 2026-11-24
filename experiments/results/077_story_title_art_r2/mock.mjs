// `node mock.mjs [<url>]`: the picker images for story_title_art_r2, shot in the running game
// (a `vite preview` of a static build at <url>, default http://localhost:51882/) rather than a
// stand-alone page: a new game played by the debug autopilot through 1.6 (075's shots.mjs
// route), then each candidate laid into the real Story panel or left column by injected CSS and
// DOM, nothing in web/src changed. Art goes in as data URLs from process.py's
// art/story_title_r2/web/. Shots go to out/077_story_title_art_r2/shots/, which compose.py
// joins into the picker's images.
//
// Windows: 1280x800 light and 1920x1080 dark for the Story mocks (the middle column 595 and
// 1058 px wide inside its padding); titles also 1280 dark and 1920 light.
//
// Later acts aren't reachable in a screenshot run yet, so an Act 2-4 mock finishes Act 1 and
// appends the later acts' beats in the panel's own markup, from content/events.json's main
// quests and content/acts.json: every beat lit but the act's last, which takes the real
// active beat's markup, then the one "???".
import { existsSync, mkdirSync, readFileSync, readdirSync, rmSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { capture, driver, launch, q, sleep } from '../../../web/scripts/cdp.mjs';

const HERE = path.dirname(fileURLToPath(import.meta.url));
const REPO = path.resolve(HERE, '..', '..', '..');
const ART = path.join(REPO, 'art', 'story_title_r2', 'web');
const OUT = path.join(REPO, 'out', '077_story_title_art_r2');
const SHOTS = path.join(OUT, 'shots');
const url = process.argv[2] ?? 'http://localhost:51882/';
const only = process.env.MOCK_ONLY ? new RegExp(process.env.MOCK_ONLY) : null;
// A full run starts from an empty shots folder, so compose.py never picks up a dropped candidate.
if (!only) rmSync(SHOTS, { recursive: true, force: true });
mkdirSync(SHOTS, { recursive: true });
// Round 1's vertical focus for a picked side header whose subject sits off center.
const CROPS = JSON.parse(readFileSync(path.join(HERE, '..', '076_story_header_and_title_art_r1', 'crops.json'), 'utf8'));
const focus = (n) => CROPS[`act${n}-side`];

// ---- content: the acts and their main-story beats, in order ----
const ACTS = JSON.parse(readFileSync(path.join(REPO, 'content', 'acts.json'), 'utf8'));
const EVENTS = JSON.parse(readFileSync(path.join(REPO, 'content', 'events.json'), 'utf8'));
const story = EVENTS.filter((e) => e.category === 'story');
const BEATS = {};
for (const a of ACTS) BEATS[a.act] = [];
let cur = 0;
for (const e of story) {
  const a = ACTS.find((x) => x.opens_with === e.id);
  if (a) cur = a.act;
  if (!cur) throw new Error(`mock: ${e.id} comes before any act opens`);
  BEATS[cur].push({ id: e.id, title: e.title, text: e.summary.replace(/^DRAFT\. /, '') });
}

// ---- art: every processed draw as a CSS custom property ----
const art = existsSync(ART) ? readdirSync(ART).filter((f) => f.endsWith('.webp')) : [];
if (!art.length) throw new Error('mock: no art: run process.py');
const ART_CSS = `:root { ${art.map((f) => `--art-${f.slice(0, -5)}: url(data:image/webp;base64,${readFileSync(path.join(ART, f)).toString('base64')});`).join('\n')} }`;
const has = (id) => art.includes(`${id}.webp`);

// ---- the compositions' CSS, all keyed off classes the mock adds ----
const BASE_CSS = `
/* Shared: the act title on art (light text in both themes, the art being a night sky). */
.mk-art > .act-band { color: #f3eee6; text-shadow: 0 1px 3px rgb(0 0 0 / 90%), 0 0 12px rgb(0 0 0 / 70%); }
.mk-art > .act-band .act-number { font-size: 13px; letter-spacing: 0.3em; }
.mk-art > .act-band .act-name { font-size: 30px; font-weight: 700; letter-spacing: 0.02em; }
/* A band: the draw cropped 3:1 (076's mock), title at the left. */
.mk-band > .act-band { background: var(--mk-img) 50% var(--mk-y, 50%) / cover no-repeat; aspect-ratio: 3 / 1; min-height: 0;
  justify-content: center; padding-left: 28px; border-color: color-mix(in srgb, #f5a524 35%, #000); }
/* Art behind the beats: the spine and the beats run over it. */
.mk-on.story-timeline::before { z-index: 1; }
.mk-on .story-act > * { position: relative; z-index: 2; }
.mk-on .act-band { z-index: 2; }
.mk-behind { position: relative; }
.mk-behind::before { content: ''; position: absolute; z-index: 0; left: -14px; right: -14px; top: 0; pointer-events: none;
  background: var(--mk-img) 50% 0 / cover no-repeat; }
.mk-behind > .act-band { background: none; border: 0; box-shadow: none; margin-top: 0; min-height: 0;
  padding-left: 44px; }
.mk-behind .beat-card, .mk-behind .side-card { background: color-mix(in srgb, var(--card) 84%, transparent);
  backdrop-filter: blur(3px); }
/* tall: a 2:3 backdrop, its subject in the top third, fading out down the act. */
.mk-tall::before { aspect-ratio: 2 / 3; mask-image: linear-gradient(#000 0 28%, transparent 88%); }
.mk-tall > .act-band { aspect-ratio: 2.3 / 1; justify-content: flex-end; padding-bottom: 18px; }
/* bleed: the picked side header shown whole (its 3:2 draw, uncropped), the title on it, the
   painting running on behind the first beats and fading. */
.mk-bleed::before { aspect-ratio: 3 / 2; mask-image: linear-gradient(#000 0 42%, transparent 96%); }
.mk-bleed > .act-band { aspect-ratio: 3 / 1.15; justify-content: center; }
/* sky: Act 1 painted with no Earth, the band reaching up into the column's sky so the page's own
   Earth hangs over the painted base. */
.mk-sky { margin-top: -150px; padding-top: 150px; }
.mk-sky::before { aspect-ratio: 2 / 3; mask-image: linear-gradient(transparent 0, #000 12%, #000 30%, transparent 85%); }
.mk-sky > .act-band { aspect-ratio: 2.6 / 1; justify-content: flex-end; padding-bottom: 14px; }
.mk-sky-on .sky .earth { z-index: 3; }
.mk-sky-on .sky { z-index: 3; }
.mk-sky-on .sky { background: none; }
/* bridge: a 5:1 strip leading into the act, its edges fading into the page, the spine over it. */
.mk-bridge { position: relative; z-index: 0 !important; aspect-ratio: 5 / 1; margin: 18px -14px 0;
  background: var(--mk-bridge) 50% 50% / cover no-repeat;
  mask-image: linear-gradient(transparent, #000 22% 78%, transparent), linear-gradient(90deg, transparent, #000 8% 92%, transparent);
  mask-composite: intersect; }
/* spines drawn as part of the painting: a painted strip repeated down the timeline. */
.mk-spine.story-timeline::before { width: 26px; left: 4px; border-radius: 0; background: var(--mk-spine) 50% 0 / 26px auto repeat-y;
  /* the strip's black ground drops out (a luminance mask of the strip itself), so the
     painted element stands on either theme's page */
  mask: var(--mk-spine) 50% 0 / 26px auto repeat-y luminance; }
.mk-spine .beat-node.is-done .beat-dot, .mk-orbit .beat-node.is-done .beat-dot {
  background: radial-gradient(circle at 40% 38%, #fff1c8, #f5a524 55%, #7a4a08); border: 2px solid #2a1d0a;
  box-shadow: 0 0 9px 2px color-mix(in srgb, #f5a524 65%, transparent); }
.mk-spine .beat-node.is-unknown .beat-dot { background: #111; }
/* orbit (CSS only): the spine a thin dashed orbit line with a faint second track. */
.mk-orbit.story-timeline::before { width: 12px; left: 11px; border-radius: 0; background:
  repeating-linear-gradient(var(--accent) 0 7px, transparent 7px 13px) 1px 0 / 2px 100% no-repeat,
  linear-gradient(color-mix(in srgb, var(--accent) 30%, transparent), color-mix(in srgb, var(--accent) 30%, transparent)) 9px 0 / 1px 100% no-repeat; }
/* The Story takeover: over the middle and right columns, like the Cortex. */
.mk-st { position: relative; }
.mk-st-body { flex: 1; min-height: 0; position: relative; display: grid; grid-template-columns: repeat(4, 1fr);
  background: var(--art-takeover) 50% 50% / cover no-repeat, #050508; }
.mk-st-act { position: relative; display: flex; flex-direction: column; gap: 8px; padding: 22px 18px; border-right: 1px solid rgb(255 255 255 / 8%); }
.mk-st-act.is-locked { background: rgb(4 4 8 / 80%); backdrop-filter: blur(10px) saturate(0.4); }
.mk-st-title { display: flex; flex-direction: column; color: #f3eee6; text-shadow: 0 1px 3px #000, 0 0 12px #000; margin-bottom: 8px; }
.mk-st-title .act-number { font-size: 13px; letter-spacing: 0.3em; text-transform: uppercase; color: #f7b955; }
.mk-st-title .act-name { font-family: var(--font-head); font-size: 26px; font-weight: 700; }
.mk-st-beat { display: flex; align-items: center; gap: 8px; padding: 5px 10px 5px 6px; border-radius: 16px; max-width: 100%;
  background: color-mix(in srgb, var(--card) 80%, transparent); border: 1px solid var(--line); backdrop-filter: blur(3px);
  font-size: 13px; font-weight: 600; width: fit-content; }
.mk-st-beat .beat-dot { width: 14px; height: 14px; margin: 0; border-width: 2px; }
.mk-st-beat.is-done .beat-dot { background: var(--accent); }
.mk-st-beat.is-done { border-color: color-mix(in srgb, var(--accent) 45%, var(--line)); }
.mk-st-beat.is-active { border: 2px solid var(--accent); box-shadow: 0 0 14px color-mix(in srgb, var(--accent) 50%, transparent); }
.mk-st-beat.is-active .beat-dot { background: #ffd27a; box-shadow: 0 0 10px 3px color-mix(in srgb, var(--accent) 70%, transparent); }
.mk-st-beat.is-unknown { opacity: 0.6; border-style: dashed; letter-spacing: 0.2em; }
.mk-st-beat.is-unknown .beat-dot { border-style: dashed; border-color: var(--muted); }
.mk-st-beat:nth-of-type(odd) { margin-left: 18px; }
.mk-st-side { margin-left: 46px; font-weight: 400; font-size: 12px; padding: 3px 9px 3px 5px; }
.mk-st-side .beat-dot { width: 10px; height: 10px; }
.mk-st-hint { color: rgb(243 238 230 / 70%); font-size: 12px; text-shadow: 0 1px 2px #000; }
`;

// ---- the titles: CSS-lettered, on the theme tokens (title.lettered-split is round 1's pick) ----
const TITLES = {
  current: { html: 'Helium Hustle', css: '' },
  'lettered-split': {
    html: '<span class="lt-a">Helium</span> <span class="lt-b">Hustle</span>',
    css: `h1.title.lettered-split .lt-a { color: var(--text); } h1.title.lettered-split .lt-b { color: var(--accent); }
      h1.title.lettered-split::after { content: ''; display: block; height: 2px; width: 40%; margin: 3px auto 0;
        background: linear-gradient(90deg, transparent, var(--accent), transparent); }`,
  },
  'split-heavy': {
    html: '<span class="lt-a">Helium</span><span class="lt-b">Hustle</span>',
    css: `h1.title.split-heavy { display: flex; justify-content: center; gap: 0.32em; font-size: 27px; letter-spacing: 0.1em; padding-bottom: 6px;
        border-bottom: 2px solid var(--accent); }
      h1.title.split-heavy .lt-b { color: var(--accent); }`,
  },
  'split-stack': {
    html: '<span class="lt-a">Helium</span><span class="lt-b">Hustle</span>',
    css: `h1.title.split-stack { display: flex; flex-direction: column; align-items: center; line-height: 1; padding: 4px 0 9px; }
      h1.title.split-stack .lt-a { font-size: 13px; letter-spacing: 0.72em; margin-right: -0.72em; color: var(--muted); }
      h1.title.split-stack .lt-b { font-size: 34px; letter-spacing: 0.16em; margin-right: -0.16em; color: var(--accent); }
      h1.title.split-stack::after { content: ''; height: 2px; width: 60%; margin-top: 5px;
        background: linear-gradient(90deg, transparent, var(--accent), transparent); }`,
  },
  'split-outline': {
    html: '<span class="lt-a">Helium</span> <span class="lt-b">Hustle</span>',
    css: `h1.title.split-outline { font-size: 26px; }
      h1.title.split-outline .lt-a { color: transparent; -webkit-text-stroke: 1.2px var(--text); }
      h1.title.split-outline .lt-b { color: var(--accent); }
      h1.title.split-outline::after { content: ''; display: block; height: 2px; width: 40%; margin: 3px auto 0;
        background: linear-gradient(90deg, transparent, var(--accent), transparent); }`,
  },
  'split-plate': {
    html: '<span class="lt-a">Helium</span><span class="lt-b">Hustle</span>',
    css: `h1.title.split-plate { display: flex; justify-content: center; align-items: center; gap: 8px; letter-spacing: 0.14em; }
      h1.title.split-plate .lt-b { background: var(--accent); color: #1a1206; padding: 1px 4px 1px 8px; border-radius: 3px; }`,
  },
  'split-he3': {
    html: '<span class="lt-a"><sup class="lt-iso">3</sup>Helium</span> <span class="lt-b">Hustle</span>',
    css: `h1.title.split-he3 .lt-b { color: var(--accent); }
      h1.title.split-he3 .lt-iso { font-size: 0.5em; color: var(--accent); letter-spacing: 0; margin-right: 1px; vertical-align: 0.9em; }
      h1.title.split-he3::after { content: ''; display: block; height: 2px; width: 40%; margin: 3px auto 0;
        background: linear-gradient(90deg, transparent, var(--accent), transparent); }`,
  },
  'split-glow': {
    html: '<span class="lt-a">Helium</span> <span class="lt-b">Hustle</span>',
    css: `h1.title.split-glow .lt-b { color: var(--accent); text-shadow: 0 0 10px color-mix(in srgb, var(--accent) 60%, transparent); }
      h1.title.split-glow::after { content: ''; display: block; height: 2px; width: 70%; margin: 4px auto 0;
        background: linear-gradient(90deg, transparent, var(--accent) 30% 70%, transparent);
        box-shadow: 0 0 8px color-mix(in srgb, var(--accent) 70%, transparent); }`,
  },
  'tie-spine': {
    html: '<span class="lt-a">Helium</span> <span class="lt-b">Hustle</span><span class="lt-spine"><i></i><i></i><i class="on"></i><i class="dim"></i></span>',
    css: `h1.title.tie-spine { padding-bottom: 8px; }
      h1.title.tie-spine .lt-b { color: var(--accent); }
      .lt-spine { display: flex; align-items: center; justify-content: center; gap: 22px; height: 12px; margin: 5px auto 0; width: 70%;
        background: linear-gradient(90deg, transparent, var(--accent) 15% 75%, transparent) 0 50% / 100% 2px no-repeat; }
      .lt-spine i { width: 8px; height: 8px; border-radius: 50%; background: var(--accent); }
      .lt-spine i.on { width: 10px; height: 10px; background: #ffd27a; box-shadow: 0 0 7px 2px color-mix(in srgb, var(--accent) 70%, transparent); }
      .lt-spine i.dim { background: var(--panel); border: 1.5px dashed var(--muted); width: 7px; height: 7px; }`,
  },
  'tie-earth': {
    html: '<span class="lt-a">Helium</span><span class="lt-earth"></span><span class="lt-b">Hustle</span>',
    css: `h1.title.tie-earth { display: flex; justify-content: center; align-items: center; gap: 7px; }
      h1.title.tie-earth .lt-b { color: var(--accent); }
      .lt-earth { flex: none; width: 20px; height: 20px; position: relative; }
      .lt-earth svg { position: absolute; inset: 0; width: 100%; height: 100%; }`,
  },
  'tie-horizon': {
    html: '<span class="lt-a">Helium</span> <span class="lt-b">Hustle</span>',
    css: `h1.title.tie-horizon { margin: -10px -8px 8px; padding: 22px 0 12px; border-bottom: 1px solid var(--line);
        background: linear-gradient(rgb(0 0 0 / 15%), rgb(0 0 0 / 45%)), var(--mk-horizon) 50% 72% / cover no-repeat;
        color: #f3eee6; text-shadow: 0 1px 3px #000, 0 0 10px #000; }
      h1.title.tie-horizon .lt-b { color: #f7b955; }`,
  },
  'tie-band': {
    html: '<span class="lt-a">Helium</span> <span class="lt-b">Hustle</span>',
    css: `h1.title.tie-band { margin: 0 0 8px; padding: 9px 0 10px; border: 1px solid color-mix(in srgb, #f5a524 45%, #000); border-radius: 6px;
        background: radial-gradient(ellipse at 85% 30%, rgb(120 150 220 / 35%), transparent 55%),
          radial-gradient(ellipse at 15% 120%, rgb(245 165 36 / 28%), transparent 60%), linear-gradient(160deg, #1d2030, #0d0d12);
        color: #f3eee6; text-shadow: 0 1px 3px rgb(0 0 0 / 80%); }
      h1.title.tie-band .lt-b { color: #f7b955; }`,
  },
};
const TITLE_CSS = Object.values(TITLES).map((t) => t.css).join('\n');

// ---- the candidates ----
// Act rows: each a mock of the middle column. kind: current | band | tall | bleed | sky | bridge.
const ACT_CANDS = [];
for (const a of ACTS) {
  const n = a.act;
  ACT_CANDS.push({ name: `act${n}_current`, act: n, kind: 'current' });
  ACT_CANDS.push({ name: `act${n}_side`, act: n, kind: 'band', img: `act${n}-side`, y: focus(n) });
  for (const v of [2, 3, 4]) if (has(`act${n}-side${v}`)) ACT_CANDS.push({ name: `act${n}_side${v}`, act: n, kind: 'band', img: `act${n}-side${v}` });
  if (has(`act${n}-tall`)) ACT_CANDS.push({ name: `act${n}_tall`, act: n, kind: 'tall', img: `act${n}-tall` });
  ACT_CANDS.push({ name: `act${n}_bleed`, act: n, kind: 'bleed', img: `act${n}-side` });
  if (n === 1 && has('act1-skyless')) ACT_CANDS.push({ name: 'act1_sky', act: 1, kind: 'sky', img: 'act1-skyless' });
  if (n > 1 && has(`bridge-${n - 1}${n}`)) ACT_CANDS.push({ name: `act${n}_bridge`, act: n, kind: 'bridge', img: `act${n}-side`, y: focus(n), bridge: `bridge-${n - 1}${n}` });
}
// The spine row: Act 1 at the active beat, its band the picked side header.
const SPINE_CANDS = [
  { name: 'spine_current', act: 1, kind: 'current', at: 'active' },
  ...['conduit', 'trail', 'beam'].filter((s) => has(`spine-${s}`)).map((s) => ({ name: `spine_${s}`, act: 1, kind: 'band', img: 'act1-side', spine: `spine-${s}`, at: 'active' })),
  { name: 'spine_orbit', act: 1, kind: 'band', img: 'act1-side', orbit: true, at: 'active' },
];
// Whole-window mocks.
const PAGE_CANDS = [
  { name: 'page_current', act: 1, kind: 'current', page: true },
  { name: 'page_side', act: 1, kind: 'band', img: 'act1-side', page: true },
  { name: 'page_bleed-conduit', act: 1, kind: 'bleed', img: 'act1-side', spine: has('spine-conduit') ? 'spine-conduit' : null, page: true },
  ...(has('act1-skyless') ? [{ name: 'page_sky', act: 1, kind: 'sky', img: 'act1-skyless', page: true }] : []),
  { name: 'page_tall', act: 1, kind: 'tall', img: 'act1-tall', page: true },
  ...(has('bridge-12') ? [{ name: 'page_act2-bridge', act: 2, kind: 'bridge', img: 'act2-side', y: focus(2), bridge: 'bridge-12', spine: has('spine-trail') ? 'spine-trail' : null, page: true }] : []),
  ...(has('takeover') ? [{ name: 'page_takeover', kind: 'takeover', page: true }, { name: 'page_takeover-late', kind: 'takeover', all: true, page: true }] : []),
].filter((c) => !c.img || has(c.img));
const CANDS = [...ACT_CANDS, ...SPINE_CANDS, ...PAGE_CANDS].filter((c) => !only || only.test(c.name));

// ---- the page-side helpers (strings evaluated in the page) ----
const PAGE_LIB = `
window.mk = (() => {
  const BEATS = ${JSON.stringify(BEATS)};
  const ACTS = ${JSON.stringify(ACTS)};
  const esc = (s) => s.replace(/[&<>]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;' })[c]);
  const tl = () => document.querySelector('.story-timeline');
  const doneBeat = (b) => '<div class="beat-row"><button type="button" class="beat-node is-done"><span class="beat-dot"></span><span class="beat-card"><span class="beat-title">' + esc(b.title) + '</span><span class="beat-text">' + esc(b.text) + '</span></span></button></div>';
  const band = (a) => '<div class="act-band" data-act="' + a.act + '"><span class="act-number">Act ' + a.act + '</span><span class="act-name">' + esc(a.name) + '</span></div>';
  let saved = null, savedTitle = null;
  return {
    save() { saved = tl().innerHTML; savedTitle = document.querySelector('h1.title').outerHTML; },
    reset() {
      const t = tl();
      t.innerHTML = saved;
      t.className = 'story-timeline';
      document.querySelector('.app').classList.remove('mk-sky-on', 'taking-over');
      document.querySelector('.mk-st')?.remove();
      document.querySelector('h1.title').outerHTML = savedTitle;
      document.querySelector('.col-mid').scrollTop = 0;
    },
    // Acts 1..n in the timeline, n's last beat active (n > 1: the rest of Act 1 lit).
    acts(n) {
      if (n === 1) return;
      const t = tl();
      const active = t.querySelector('.beat-node.is-active');
      const activeRow = active.closest('.beat-row');
      const template = activeRow.cloneNode(true);
      const unknown = t.querySelector('.beat-node.is-unknown')?.closest('.beat-row');
      const id = active.dataset.beat;
      const a1 = BEATS[1];
      const at = a1.findIndex((b) => b.id === id);
      if (at < 0) throw new Error('mk: the active beat ' + id + ' is not an Act 1 beat');
      const sec1 = t.querySelector('.story-act[data-act="1"]');
      activeRow.outerHTML = a1.slice(at).map(doneBeat).join('');
      for (let k = 2; k <= n; k++) {
        const a = ACTS.find((x) => x.act === k);
        const beats = BEATS[k];
        const last = k === n ? beats.length - 1 : beats.length;
        let html = band(a) + beats.slice(0, last).map(doneBeat).join('');
        const sec = document.createElement('section');
        sec.className = 'story-act';
        sec.dataset.act = k;
        sec.innerHTML = html;
        if (k === n) {
          const row = template.cloneNode(true);
          row.querySelector('.side-branches')?.remove();
          row.querySelector('.quest-title').textContent = beats[last].title;
          row.querySelector('.quest-summary').textContent = beats[last].text;
          // 1.7's own progress and sub-objectives don't belong to this beat.
          row.querySelectorAll('.quest .bar, .quest .sub').forEach((x) => x.remove());
          sec.appendChild(row);
        }
        if (unknown) t.insertBefore(sec, unknown); else t.appendChild(sec);
      }
      void sec1;
    },
    apply(c) {
      const t = tl();
      const sec = t.querySelector('.story-act[data-act="' + c.act + '"]');
      const b = sec.querySelector('.act-band');
      const img = (id) => 'var(--art-' + id + ')';
      if (c.kind !== 'current') { t.classList.add('mk-on'); sec.classList.add('mk-art'); }
      if (c.kind === 'band' || c.kind === 'bridge') { sec.classList.add('mk-band'); b.style.setProperty('--mk-img', img(c.img)); if (c.y) b.style.setProperty('--mk-y', c.y + '%'); }
      if (c.kind === 'tall' || c.kind === 'bleed' || c.kind === 'sky') { sec.classList.add('mk-behind', 'mk-' + c.kind); sec.style.setProperty('--mk-img', img(c.img)); }
      if (c.kind === 'sky') document.querySelector('.app').classList.add('mk-sky-on');
      if (c.kind === 'bridge') { const d = document.createElement('div'); d.className = 'mk-bridge'; d.style.setProperty('--mk-bridge', img(c.bridge)); sec.insertBefore(d, b); }
      if (c.spine) { t.classList.add('mk-spine'); t.style.setProperty('--mk-spine', img(c.spine)); }
      if (c.orbit) t.classList.add('mk-orbit');
    },
    // Scrolls the middle column: act 1 at the top; a later act's band (or its bridge) near the
    // top with the end of the act before it showing; 'active' centers the active beat.
    scroll(c) {
      const box = document.querySelector('.col-mid');
      let el = null, off = 110;
      if (c.at === 'active') { el = tl().querySelector('.beat-node.is-active'); off = (box.clientHeight - el.offsetHeight) / 2; }
      else if (c.act > 1) el = tl().querySelector('.story-act[data-act="' + c.act + '"] .mk-bridge') ?? tl().querySelector('.story-act[data-act="' + c.act + '"] .act-band');
      if (!el) { box.scrollTop = 0; return; }
      box.scrollTop += el.getBoundingClientRect().top - box.getBoundingClientRect().top - off;
    },
    // all: late game, every act reached and Act 4's last beat active (the painting unfogged).
    takeover(all) {
      const app = document.querySelector('.app');
      const active = tl().querySelector('.beat-node.is-active')?.dataset.beat;
      const sides = [...tl().querySelectorAll('.side-node')].map((s) => ({ title: s.querySelector('.side-title').textContent.trim(), done: s.classList.contains('is-done') }));
      const a1 = BEATS[1];
      const at = a1.findIndex((b) => b.id === active);
      const chip = (cls, t) => '<div class="mk-st-beat ' + cls + '"><span class="beat-dot"></span><span>' + esc(t) + '</span></div>';
      const last = ACTS[ACTS.length - 1].act;
      const cols = ACTS.map((a) => {
        if (all) {
          const bs = BEATS[a.act];
          const n = a.act === last ? bs.length - 1 : bs.length;
          const body = bs.slice(0, n).map((b) => chip('is-done', b.title)).join('') + (a.act === last ? chip('is-active', bs[n].title) : '');
          return '<div class="mk-st-act"><div class="mk-st-title"><span class="act-number">Act ' + a.act + '</span><span class="act-name">' + esc(a.name) + '</span></div>' + body + '</div>';
        }
        if (a.act === 1) {
          const done = a1.slice(0, at).map((b) => chip('is-done', b.title)).join('');
          const side = sides.map((s) => '<div class="mk-st-beat mk-st-side ' + (s.done ? 'is-done' : '') + '"><span class="beat-dot"></span><span>' + esc(s.title) + '</span></div>').join('');
          return '<div class="mk-st-act"><div class="mk-st-title"><span class="act-number">Act 1</span><span class="act-name">' + esc(a.name) + '</span></div>' + done + side + chip('is-active', a1[at].title) + chip('is-unknown', '???') + '</div>';
        }
        return '<div class="mk-st-act is-locked"><div class="mk-st-title"><span class="act-number">Act ' + a.act + '</span><span class="act-name">???</span></div></div>';
      }).join('');
      const day = document.getElementById('day')?.textContent ?? '';
      const v = document.createElement('div');
      v.className = 'takeover mk-st';
      v.innerHTML = '<div class="takeover-head"><h2>Story</h2><div class="takeover-strip"><span class="takeover-day">' + esc(day) + '</span></div><button type="button" class="close">✕</button></div><div class="mk-st-body">' + cols + '</div>';
      app.classList.add('taking-over');
      app.appendChild(v);
    },
    title(key, html) {
      const h = document.querySelector('h1.title');
      h.className = 'title ' + key;
      h.innerHTML = html;
      const e = h.querySelector('.lt-earth');
      if (e) e.appendChild(document.querySelector('.sky svg.earth').cloneNode(true));
      const hz = document.querySelector('.horizon img');
      if (hz) h.style.setProperty('--mk-horizon', 'url(' + hz.src + ')');
    },
    rect(sel) { const r = document.querySelector(sel).getBoundingClientRect(); return { x: r.x, y: r.y, width: r.width, height: r.height }; },
  };
})();
`;

// ---- drive ----
const COMBOS = [
  { w: 1280, h: 800, theme: 'light', story: true },
  { w: 1920, h: 1080, theme: 'dark', story: true },
  { w: 1280, h: 800, theme: 'dark', story: false },
  { w: 1920, h: 1080, theme: 'light', story: false },
];
const b = await launch({ width: 1280, height: 800, emulate: true, profileRoot: path.join(OUT, 'profile') });
const { evaluate, send } = b;
const { waitFor, exists, click, load, speed, panel, cheats, autoStory } = driver(b, { url, settle: 700 });
const theme = async (t) => {
  await panel('options');
  await click(`#theme [data-theme-pick="${t}"]`);
};
try {
  await load(url);
  await cheats();
  await speed(0);
  await click('#new-game');
  await sleep(500);
  await autoStory();
  const beat = () => evaluate(`Number(document.querySelector('[data-testid="quest"]')?.textContent.match(/^1\\.(\\d+)/)?.[1] ?? 0)`);
  for (let step = 0; step < 8 && (await beat()) < 6; step++) {
    await cheats();
    await click('#accel-quest');
    await waitFor(`document.getElementById('accel-status')?.dataset.running === '0'`, 'step', 90000);
    if (await exists('[data-testid="continue"]')) await click('[data-testid="continue"]');
  }
  await speed(0);
  console.log(`mock: played to ${await evaluate(`document.querySelector('[data-testid="quest"]')?.textContent.slice(0, 40)`)}`);
  await evaluate(`(() => { for (const [id, css] of ${q([['mk-art-css', ART_CSS], ['mk-base-css', BASE_CSS + TITLE_CSS]])}) {
    const s = document.createElement('style'); s.id = id; s.textContent = css; document.head.appendChild(s); } })()`);
  await evaluate(PAGE_LIB);
  for (const combo of COMBOS) {
    await send('Emulation.setDeviceMetricsOverride', { width: combo.w, height: combo.h, deviceScaleFactor: 1, mobile: false });
    await theme(combo.theme);
    await panel('story');
    await sleep(400);
    await evaluate('mk.save()');
    const tag = `${combo.w}-${combo.theme}`;
    if (combo.story) {
      for (const c of CANDS) {
        await evaluate('mk.reset()');
        if (c.kind === 'takeover') await evaluate(`mk.takeover(${!!c.all})`);
        else {
          await evaluate(`mk.acts(${c.act})`);
          await evaluate(`mk.apply(${q(c)})`);
          await sleep(60);
          await evaluate(`mk.scroll(${q(c)})`);
        }
        await sleep(250);
        const clip = c.page ? { x: 0, y: 0, width: combo.w, height: combo.h } : await evaluate(`mk.rect('.col-mid')`);
        await capture(b, path.join(SHOTS, `${c.name}__${tag}.png`), { clip: { ...clip, scale: 1 } });
        console.log(`mock: ${c.name} ${tag}`);
      }
    }
    for (const [key, t] of Object.entries(TITLES)) {
      if (only && !only.test(`title_${key}`)) continue;
      await evaluate('mk.reset()');
      await evaluate(`mk.title(${q(key)}, ${q(t.html)})`);
      await sleep(120);
      const r = await evaluate(`(() => { const a = mk.rect('.col-left'); const n = mk.rect('.nav-grid'); return { x: a.x, y: a.y, width: a.width, height: n.y + n.height + 8 - a.y }; })()`);
      await capture(b, path.join(SHOTS, `title_${key}__${tag}.png`), { clip: { ...r, scale: 2 } });
      console.log(`mock: title_${key} ${tag}`);
    }
    await evaluate('mk.reset()');
  }
  await theme('light');
} finally {
  b.close();
}
